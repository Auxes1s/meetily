import { invoke } from '@tauri-apps/api/core';

export interface DeletionStatus {
  job_id: string;
  meeting_id: string;
  recording_session_id: string | null;
  recording_session_ids?: string[];
  state: 'pending' | 'complete';
  native_complete: boolean;
  caches_complete: boolean;
  last_error: string | null;
}

export class MeetingDeletionPendingError extends Error {
  constructor(public readonly status: DeletionStatus, cause: unknown, message = 'Meeting cache cleanup is pending. Retry deletion after local storage becomes available.') {
    super(message, { cause });
    this.name = 'MeetingDeletionPendingError';
  }
}

const TOMBSTONES_KEY = 'meetily.deletedMeetingIdentities';
const MEETING_TOMBSTONE_PREFIX = 'meetily.deletedMeeting:';
const SESSION_TOMBSTONE_PREFIX = 'meetily.deletedRecordingSession:';
const ALIASES_PREFIX = 'meetily.deletedMeetingAliases:';
interface Tombstones { meetings: string[]; sessions: string[]; aliases?: Record<string, string[]> }
interface CacheEnvironment {
  indexedDB: IDBFactory;
  localStorage: Storage;
  sessionStorage: Storage;
}
export type DeletionGateway = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

function readTombstones(storage: Storage): Tombstones {
  const raw = storage.getItem(TOMBSTONES_KEY);
  if (!raw) return { meetings: [], sessions: [] };
  const value = JSON.parse(raw) as Tombstones;
  if (!Array.isArray(value.meetings) || !Array.isArray(value.sessions) || ![...value.meetings, ...value.sessions].every(v => typeof v === 'string')) {
    throw new Error('Deletion identity storage is invalid');
  }
  if (value.aliases && (typeof value.aliases !== 'object' || Array.isArray(value.aliases) ||
      !Object.values(value.aliases).every(aliases => Array.isArray(aliases) && aliases.every(id => typeof id === 'string')))) {
    throw new Error('Deletion alias storage is invalid');
  }
  return value;
}

function persistTombstones(storage: Storage, meetings: Iterable<string>, sessions: Iterable<string>, canonicalId: string): void {
  const old = readTombstones(storage);
  const aliases = [...meetings];
  const sessionIds = [...sessions];
  // Distinct keys keep separate WebViews deleting different meetings from
  // overwriting each other's admission barriers in an aggregate read/write.
  for (const id of aliases) storage.setItem(`${MEETING_TOMBSTONE_PREFIX}${encodeURIComponent(id)}`, '1');
  for (const id of sessionIds) storage.setItem(`${SESSION_TOMBSTONE_PREFIX}${encodeURIComponent(id)}`, '1');
  const rememberedAliases = readAliases(storage, canonicalId, old);
  const combinedAliases = [...new Set([...rememberedAliases, ...aliases])];
  storage.setItem(`${ALIASES_PREFIX}${encodeURIComponent(canonicalId)}`, JSON.stringify(combinedAliases));
  storage.setItem(TOMBSTONES_KEY, JSON.stringify({
    meetings: [...new Set([...old.meetings, ...aliases])],
    sessions: [...new Set([...old.sessions, ...sessionIds])],
    aliases: { ...old.aliases, [canonicalId]: combinedAliases },
  }));
}

function readAliases(storage: Storage, canonicalId: string, old = readTombstones(storage)): string[] {
  const raw = storage.getItem(`${ALIASES_PREFIX}${encodeURIComponent(canonicalId)}`);
  if (!raw) return old.aliases?.[canonicalId] ?? [];
  const aliases: unknown = JSON.parse(raw);
  if (!Array.isArray(aliases) || !aliases.every(id => typeof id === 'string')) throw new Error('Deletion alias storage is invalid');
  return aliases;
}

/** Existing recovery/cache writers use this before opening their transaction.
 * Storage failures fail closed: unavailable tombstones never authorize recovery.
 */
export function isMeetingDeletionBlocked(meetingId: string, recordingSessionId?: string | null, storage?: Storage): boolean {
  try {
    const identityStorage = storage ?? window.localStorage;
    if (identityStorage.getItem(`${MEETING_TOMBSTONE_PREFIX}${encodeURIComponent(meetingId)}`) === '1' ||
        !!recordingSessionId && identityStorage.getItem(`${SESSION_TOMBSTONE_PREFIX}${encodeURIComponent(recordingSessionId)}`) === '1') return true;
    const tombstones = readTombstones(identityStorage);
    return tombstones.meetings.includes(meetingId) || !!recordingSessionId && tombstones.sessions.includes(recordingSessionId);
  } catch {
    return true;
  }
}

function browserEnvironment(): CacheEnvironment {
  if (typeof window === 'undefined' || !window.indexedDB) throw new Error('Meeting recovery storage is unavailable');
  return { indexedDB: window.indexedDB, localStorage: window.localStorage, sessionStorage: window.sessionStorage };
}

function openRecovery(factory: IDBFactory): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    let failed = false;
    const request = factory.open('MeetilyRecoveryDB', 1);
    request.onerror = () => { failed = true; reject(request.error ?? new Error('Cannot open meeting recovery storage')); };
    request.onblocked = () => { failed = true; reject(new Error('Meeting recovery storage is blocked')); };
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains('meetings')) {
        const meetings = db.createObjectStore('meetings', { keyPath: 'meetingId' });
        meetings.createIndex('lastUpdated', 'lastUpdated');
        meetings.createIndex('savedToSQLite', 'savedToSQLite');
      }
      if (!db.objectStoreNames.contains('transcripts')) {
        const transcripts = db.createObjectStore('transcripts', { keyPath: 'id', autoIncrement: true });
        transcripts.createIndex('meetingId', 'meetingId');
        transcripts.createIndex('storedAt', 'storedAt');
      }
    };
    request.onsuccess = () => { if (failed) request.result.close(); else resolve(request.result); };
  });
}

function removePreferences(environment: CacheEnvironment, identities: Set<string>, sessionIds: Set<string>): void {
  for (const id of identities) {
    environment.localStorage.removeItem(`summaryLanguageFallback:${id}`);
    environment.localStorage.removeItem(`detectedSummaryLanguageFallback:${id}`);
    environment.localStorage.removeItem(`meetily.enhancementDraft:${id}`);
  }
  if (identities.has(environment.localStorage.getItem('meetily.lastNotesMeeting') ?? '')) {
    environment.localStorage.removeItem('meetily.lastNotesMeeting');
  }
  const session = environment.sessionStorage;
  const recordingMatches = identities.has(session.getItem('recording_meeting_id') ?? '') ||
    sessionIds.has(session.getItem('recording_session_id') ?? '');
  const recoveryMatches = identities.has(session.getItem('indexeddb_current_meeting_id') ?? '');
  for (const key of ['workspace_meeting_id', 'recording_meeting_id', 'indexeddb_current_meeting_id']) {
    if (identities.has(session.getItem(key) ?? '')) session.removeItem(key);
  }
  if (recordingMatches || recoveryMatches) {
    for (const key of ['recording_session_id', 'last_recording_folder_path', 'last_recording_meeting_name']) session.removeItem(key);
  }
}

/** Resolves only on transaction.oncomplete, never on an individual delete's
 * success. Aliased legacy recovery IDs are matched by canonical/session identity.
 */
export async function cleanupMeetingCaches(status: DeletionStatus, environment: CacheEnvironment = browserEnvironment()): Promise<void> {
  const identities = new Set([status.meeting_id, ...readAliases(environment.localStorage, status.meeting_id)]);
  const sessions = new Set([...(status.recording_session_ids ?? []), ...(status.recording_session_id ? [status.recording_session_id] : [])]);
  // Persist before the transaction so late event writers cannot recreate it.
  persistTombstones(environment.localStorage, identities, sessions, status.meeting_id);
  const db = await openRecovery(environment.indexedDB);
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(['meetings', 'transcripts'], 'readwrite');
      let failure: unknown;
      const abort = (error: unknown) => { failure = error; tx.abort(); };
      tx.oncomplete = () => resolve();
      tx.onabort = () => reject(failure ?? tx.error ?? new Error('Meeting cache cleanup transaction aborted'));
      tx.onerror = () => { failure = tx.error ?? new Error('Meeting cache cleanup transaction failed'); };
      const meetings = tx.objectStore('meetings').openCursor();
      meetings.onerror = () => abort(meetings.error);
      meetings.onsuccess = () => {
        const cursor = meetings.result;
        if (cursor) {
          const row = cursor.value;
          const matches = identities.has(row.meetingId) || row.canonicalMeetingId === status.meeting_id ||
            sessions.has(row.recordingSessionId);
          if (matches) {
            identities.add(row.meetingId);
            if (typeof row.recordingSessionId === 'string') sessions.add(row.recordingSessionId);
            try { persistTombstones(environment.localStorage, identities, sessions, status.meeting_id); }
            catch (error) { abort(error); return; }
            const deletion = cursor.delete();
            deletion.onerror = () => abort(deletion.error);
          }
          cursor.continue();
          return;
        }
        const transcripts = tx.objectStore('transcripts').openCursor();
        transcripts.onerror = () => abort(transcripts.error);
        transcripts.onsuccess = () => {
          const passage = transcripts.result;
          if (!passage) return;
          const row = passage.value;
          if (identities.has(row.meetingId) || row.canonicalMeetingId === status.meeting_id ||
              sessions.has(row.recording_session_id) || sessions.has(row.recordingSessionId)) {
            if (typeof row.meetingId === 'string') identities.add(row.meetingId);
            try { persistTombstones(environment.localStorage, identities, sessions, status.meeting_id); }
            catch (error) { abort(error); return; }
            const deletion = passage.delete();
            deletion.onerror = () => abort(deletion.error);
          }
          passage.continue();
        };
      };
    });
    removePreferences(environment, identities, sessions);
  } finally {
    db.close();
  }
}

export class MeetingDeletionService {
  constructor(private readonly gateway: DeletionGateway = invoke, private readonly cleanup = cleanupMeetingCaches) {}

  private async finish(status: DeletionStatus): Promise<DeletionStatus> {
    // Repeat cleanup even if a previous WebView acknowledged it: this also
    // cleans recovery aliases in another window opened during a pending job.
    try { await this.cleanup(status); }
    catch (error) { throw new MeetingDeletionPendingError({ ...status, state: 'pending', caches_complete: false }, error); }
    let acknowledged: DeletionStatus;
    try { acknowledged = await this.gateway<DeletionStatus>('api_ack_meeting_deletion_caches', { jobId: status.job_id }); }
    catch (error) {
      throw new MeetingDeletionPendingError({ ...status, state: 'pending' }, error, 'Deletion completion acknowledgment is pending. Retry deletion to confirm cleanup.');
    }
    if (typeof window !== 'undefined') window.dispatchEvent(new CustomEvent('workspace-meeting-deleted', {
      detail: { meetingId: status.meeting_id, status: acknowledged },
    }));
    return acknowledged;
  }

  async deleteMeeting(meetingId: string): Promise<DeletionStatus> {
    const status = await this.gateway<DeletionStatus>('api_begin_meeting_deletion', { meetingId });
    if (typeof window !== 'undefined') window.dispatchEvent(new CustomEvent('meeting-deletion-started', {
      detail: { meetingId, status },
    }));
    return this.finish(status);
  }

  async retry(jobId: string): Promise<DeletionStatus> {
    return this.finish(await this.gateway<DeletionStatus>('api_retry_meeting_deletion', { jobId }));
  }

  async status(jobId: string): Promise<DeletionStatus> {
    return this.gateway<DeletionStatus>('api_get_meeting_deletion_status', { jobId });
  }

  async listPending(): Promise<DeletionStatus[]> {
    return this.gateway<DeletionStatus[]>('api_list_pending_meeting_deletions');
  }

  async resumePending(): Promise<DeletionStatus[]> {
    const pending = await this.gateway<DeletionStatus[]>('api_retry_pending_meeting_deletions');
    const outcomes = await Promise.allSettled(pending.map(status => this.finish(status)));
    // Attempt every job; callers receive pending statuses for failures and can
    // surface the retry reason without falsely claiming cleanup completed.
    return outcomes.map((result, i) => result.status === 'fulfilled' ? result.value : {
      ...pending[i], last_error: result.reason instanceof Error ? result.reason.message : 'Meeting cache cleanup is pending',
    });
  }
}

export const meetingDeletionService = new MeetingDeletionService();
export const deleteMeetingManaged = (meetingId: string) => meetingDeletionService.deleteMeeting(meetingId);
export const resumePendingMeetingDeletions = () => meetingDeletionService.resumePending();
