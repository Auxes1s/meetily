'use client';
import { useEffect, useState, useSyncExternalStore } from 'react';
import type { Block } from '@blocknote/core';
import { invoke } from '@tauri-apps/api/core';
import { MeetingNotesSession, noteSessions, pruneDocumentSessions, type NoteLink } from '@/lib/meeting-notes-session';
import { isMeetingDeletionBlocked } from '@/services/meetingDeletionService';

const loads = new Map<string, Promise<MeetingNotesSession>>();
export async function loadNotesSession(meetingId: string): Promise<MeetingNotesSession> {
  if (isMeetingDeletionBlocked(meetingId)) throw new Error('Meeting deletion is pending or complete.');
  if (noteSessions.has(meetingId)) return noteSessions.get(meetingId)!;
  if (loads.has(meetingId)) return loads.get(meetingId)!;
  const load = (async () => {
    const [doc, meeting] = await Promise.all([
      invoke<{ notes_json: string | null; notes_markdown: string | null; revision: number; updated_at: string; links: NoteLink[] } | null>('api_get_meeting_notes', { meetingId }),
      invoke<{ title: string }>('api_get_meeting_metadata', { meetingId }),
    ]);
    let blocks: Block[] = [];
    if (doc?.notes_json != null) {
      blocks = JSON.parse(doc.notes_json);
      if (!Array.isArray(blocks)) throw new Error('Stored note JSON is malformed. Original data has been preserved.');
    } else if (doc?.notes_markdown) {
      const { BlockNoteEditor } = await import('@blocknote/core');
      const parser = BlockNoteEditor.create();
      blocks = await parser.tryParseMarkdownToBlocks(doc.notes_markdown);
    }
    if (isMeetingDeletionBlocked(meetingId)) throw new Error('Meeting deletion is pending or complete.');
    const session = new MeetingNotesSession(meetingId, {
      blocks, links: doc?.links ?? [], title: meeting.title, revision: doc?.revision ?? 0, updatedAt: doc?.updated_at ?? null,
    }, async input => {
      const commit = await invoke<{ revision: number; updated_at: string }>('api_save_meeting_notes', { input });
      window.dispatchEvent(new CustomEvent('meeting-notes-committed', { detail: { id: meetingId, title: input.title } }));
      return commit;
    });
    noteSessions.set(meetingId, session);
    return session;
  })();
  loads.set(meetingId, load);
  try { return await load; } finally { loads.delete(meetingId); }
}
export function useMeetingNotes(meetingId: string | null) {
  const [loaded, setLoaded] = useState<{ id: string; session: MeetingNotesSession } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  const reload = async () => {
    await Promise.all([...noteSessions.values()].filter(session => session.meetingId !== meetingId).map(session => session.flush()));
    if (meetingId) { noteSessions.get(meetingId)?.dispose(); noteSessions.delete(meetingId); }
    setLoaded(null); setReloadKey(value => value + 1);
  };
  useEffect(() => {
    let cancelled = false;
    let release: (() => void) | undefined;
    setError(null);
    if (meetingId) loadNotesSession(meetingId).then(session => {
      if (!cancelled) { release = session.retain(); setLoaded({ id: meetingId, session }); pruneDocumentSessions(noteSessions, value => value, meetingId); }
    }).catch(error => { if (!cancelled) setError(String(error)); });
    return () => { cancelled = true; release?.(); queueMicrotask(() => pruneDocumentSessions(noteSessions, value => value)); };
  }, [meetingId, reloadKey]);
  const session = loaded?.id === meetingId ? loaded.session : null;
  const snapshot = useSyncExternalStore(session?.subscribe ?? (() => () => {}), session?.getSnapshot ?? (() => null), () => null);
  return { session, snapshot, error, reload };
}
