import { describe, expect, test } from 'bun:test';
import { cleanupMeetingCaches, isMeetingDeletionBlocked, MeetingDeletionPendingError, MeetingDeletionService, type DeletionGateway, type DeletionStatus } from '../../src/services/meetingDeletionService';

const pending: DeletionStatus = {
  job_id: 'job-a', meeting_id: 'canonical-a', recording_session_id: 'session-a',
  state: 'pending', native_complete: true, caches_complete: false, last_error: null,
};
const completed = { ...pending, state: 'complete' as const, caches_complete: true };
const tick = () => new Promise(resolve => setTimeout(resolve, 0));

function storage(): Storage {
  const values = new Map<string, string>();
  return {
    get length() { return values.size; },
    key: index => [...values.keys()][index] ?? null,
    getItem: key => values.get(key) ?? null,
    setItem: (key, value) => { values.set(key, value); },
    removeItem: key => { values.delete(key); },
    clear: () => values.clear(),
  };
}

// A controlled IDB event boundary tests the critical distinction between request
// success and transaction commit. No timing assumption can auto-complete it.
function cacheFixture() {
  const meetings: Record<string, unknown>[] = [
    { meetingId: 'legacy-a', canonicalMeetingId: 'canonical-a', recordingSessionId: 'session-a' },
    { meetingId: 'keep-b', canonicalMeetingId: 'canonical-b', recordingSessionId: 'session-b' },
  ];
  const transcripts: Record<string, unknown>[] = [
    { id: 1, meetingId: 'legacy-a', text: 'remove' },
    { id: 2, meetingId: 'alternate-a', recording_session_id: 'session-a', text: 'remove by session' },
    { id: 3, meetingId: 'keep-b', text: 'retain' },
  ];
  let closed = false;
  const transaction: Record<string, any> = {
    oncomplete: null, onabort: null, onerror: null, error: null,
    abort() { transaction.onabort?.(); },
    objectStore(name: string) {
      const rows = name === 'meetings' ? meetings : transcripts;
      return { openCursor() {
        const request: Record<string, any> = { result: null, onsuccess: null, onerror: null };
        const snapshot = [...rows]; let index = 0;
        const advance = () => queueMicrotask(() => {
          const row = snapshot[index++];
          request.result = row ? {
            value: row,
            delete() {
              rows.splice(rows.indexOf(row), 1);
              const deletion: Record<string, any> = { onerror: null, onsuccess: null };
              queueMicrotask(() => deletion.onsuccess?.());
              return deletion;
            },
            continue: advance,
          } : null;
          request.onsuccess?.();
        });
        advance(); return request;
      } };
    },
  };
  const database = {
    transaction: () => transaction,
    close: () => { closed = true; },
  };
  const factory = { open() {
    const request: Record<string, any> = { result: database, onsuccess: null, onerror: null };
    queueMicrotask(() => request.onsuccess?.()); return request;
  } } as unknown as IDBFactory;
  const environment = { indexedDB: factory, localStorage: storage(), sessionStorage: storage() };
  return { meetings, transcripts, transaction, environment, get closed() { return closed; } };
}

describe('managed deletion completion', () => {
  test('acknowledges cache cleanup only after its promise completes', async () => {
    const commands: string[] = [];
    let finish: () => void = () => {};
    const gateway: DeletionGateway = async <T>(command: string) => {
      commands.push(command); return (command.includes('begin') ? pending : completed) as T;
    };
    const service = new MeetingDeletionService(gateway, () => new Promise(resolve => { finish = resolve; }));
    const deletion = service.deleteMeeting('canonical-a'); await tick();
    expect(commands).toEqual(['api_begin_meeting_deletion']);
    finish(); expect((await deletion).state).toBe('complete');
    expect(commands).toEqual(['api_begin_meeting_deletion', 'api_ack_meeting_deletion_caches']);
  });

  test('a cache failure carries pending job identity and never acknowledges completion', async () => {
    const commands: string[] = [];
    const gateway: DeletionGateway = async <T>(command: string) => { commands.push(command); return pending as T; };
    const service = new MeetingDeletionService(gateway, async () => { throw new Error('IndexedDB disabled'); });
    const error = await service.deleteMeeting('canonical-a').catch(error => error);
    expect(error).toBeInstanceOf(MeetingDeletionPendingError);
    expect(error.status.job_id).toBe('job-a'); expect(error.cause.message).toBe('IndexedDB disabled');
    expect(commands).toEqual(['api_begin_meeting_deletion']);
  });

  test('native file failure remains pending after successful cache cleanup', async () => {
    const filePending = { ...pending, native_complete: false, last_error: 'Audio file locked' };
    const gateway: DeletionGateway = async <T>(command: string) => ({ ...filePending, caches_complete: command.includes('ack') }) as T;
    const service = new MeetingDeletionService(gateway, async () => {});
    const result = await service.deleteMeeting('canonical-a');
    expect(result.state).toBe('pending'); expect(result.last_error).toBe('Audio file locked');
  });

  test('a lost cache acknowledgment retains the durable deletion job for retry', async () => {
    const commands: string[] = [];
    const gateway: DeletionGateway = async <T>(command: string) => {
      commands.push(command);
      if (command.includes('ack')) throw new Error('WebView response lost');
      return pending as T;
    };
    const service = new MeetingDeletionService(gateway, async () => {});
    const error = await service.deleteMeeting('canonical-a').catch(error => error);
    expect(error).toBeInstanceOf(MeetingDeletionPendingError);
    expect(error.status.job_id).toBe('job-a');
    expect(error.status.state).toBe('pending');
    expect(error.message).toContain('acknowledgment');
    expect(commands).toEqual(['api_begin_meeting_deletion', 'api_ack_meeting_deletion_caches']);
  });

  test('failed repeat cache cleanup never carries a completed status', async () => {
    const gateway: DeletionGateway = async <T>() => completed as T;
    const service = new MeetingDeletionService(gateway, async () => { throw new Error('Storage disabled'); });
    const error = await service.retry('job-a').catch(error => error);
    expect(error.status.state).toBe('pending');
    expect(error.status.caches_complete).toBe(false);
  });

  test('startup attempts every pending job and exposes failed cleanup', async () => {
    const second = { ...pending, job_id: 'job-b', meeting_id: 'canonical-b' };
    const gateway: DeletionGateway = async <T>(command: string, args?: Record<string, unknown>) => {
      if (command.includes('pending')) return [pending, second] as T;
      return { ...completed, job_id: args?.jobId } as T;
    };
    const cleaned: string[] = [];
    const service = new MeetingDeletionService(gateway, async status => {
      cleaned.push(status.job_id); if (status.job_id === 'job-a') throw new Error('Storage blocked');
    });
    const result = await service.resumePending();
    expect(cleaned).toEqual(['job-a', 'job-b']);
    expect(result[0].state).toBe('pending'); expect(result[0].last_error).toContain('pending');
    expect(result[1].state).toBe('complete');
  });
});

describe('awaited recovery cache transaction', () => {
  test('request deletes alone cannot report completion; aliases and session match without deleting other meetings', async () => {
    const cache = cacheFixture(); let resolved = false;
    cache.environment.localStorage.setItem('summaryLanguageFallback:legacy-a', 'fil');
    cache.environment.localStorage.setItem('summaryLanguageFallback:keep-b', 'en');
    cache.environment.localStorage.setItem('meetily.lastNotesMeeting', 'legacy-a');
    cache.environment.sessionStorage.setItem('workspace_meeting_id', 'legacy-a');
    cache.environment.sessionStorage.setItem('indexeddb_current_meeting_id', 'legacy-a');
    const cleanup = cleanupMeetingCaches(pending, cache.environment).then(() => { resolved = true; });
    await tick();
    expect(cache.meetings.map(row => row.meetingId)).toEqual(['keep-b']);
    expect(cache.transcripts.map(row => row.id)).toEqual([3]);
    expect(resolved).toBe(false);
    expect(isMeetingDeletionBlocked('legacy-a', undefined, cache.environment.localStorage)).toBe(true);
    expect(isMeetingDeletionBlocked('another-id', 'session-a', cache.environment.localStorage)).toBe(true);
    cache.transaction.oncomplete(); await cleanup;
    expect(resolved).toBe(true); expect(cache.closed).toBe(true);
    expect(cache.environment.localStorage.getItem('summaryLanguageFallback:legacy-a')).toBeNull();
    expect(cache.environment.localStorage.getItem('summaryLanguageFallback:keep-b')).toBe('en');
    expect(cache.environment.sessionStorage.getItem('workspace_meeting_id')).toBeNull();
  });

  test('transaction abort prevents cache completion and closes the connection', async () => {
    const cache = cacheFixture();
    const cleanup = cleanupMeetingCaches(pending, cache.environment); await tick();
    cache.transaction.error = new Error('Disk transaction failed'); cache.transaction.onabort();
    await expect(cleanup).rejects.toThrow('Disk transaction failed'); expect(cache.closed).toBe(true);
  });

  test('per-identity barriers survive an aggregate overwrite by another WebView', async () => {
    const cache = cacheFixture();
    const cleanup = cleanupMeetingCaches(pending, cache.environment); await tick();
    cache.environment.localStorage.setItem('meetily.deletedMeetingIdentities', JSON.stringify({ meetings: ['another-meeting'], sessions: ['another-session'] }));
    expect(isMeetingDeletionBlocked('canonical-a', null, cache.environment.localStorage)).toBe(true);
    expect(isMeetingDeletionBlocked('legacy-a', null, cache.environment.localStorage)).toBe(true);
    expect(isMeetingDeletionBlocked('new-alias', 'session-a', cache.environment.localStorage)).toBe(true);
    cache.transaction.oncomplete(); await cleanup;
  });

  test('storage failures fail closed for delayed recovery and cleanup', async () => {
    const cache = cacheFixture();
    cache.environment.localStorage.getItem = () => { throw new Error('Storage disabled'); };
    expect(isMeetingDeletionBlocked('canonical-a', null, cache.environment.localStorage)).toBe(true);
    await expect(cleanupMeetingCaches(pending, cache.environment)).rejects.toThrow('Storage disabled');
  });

  test('retry remembers legacy aliases after metadata deletion but preference cleanup failure', async () => {
    const cache = cacheFixture();
    cache.environment.localStorage.setItem('summaryLanguageFallback:legacy-a', 'fil');
    const remove = cache.environment.localStorage.removeItem;
    cache.environment.localStorage.removeItem = () => { throw new Error('Preference storage locked'); };
    const first = cleanupMeetingCaches(pending, cache.environment); await tick(); cache.transaction.oncomplete();
    await expect(first).rejects.toThrow('Preference storage locked');
    cache.environment.localStorage.removeItem = remove;
    const retry = cleanupMeetingCaches(pending, cache.environment); await tick(); cache.transaction.oncomplete(); await retry;
    expect(cache.environment.localStorage.getItem('summaryLanguageFallback:legacy-a')).toBeNull();
  });
});
