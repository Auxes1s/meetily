'use client';

import React, { createContext, useContext, useState, useEffect, useRef, useCallback, ReactNode, MutableRefObject, useMemo, useSyncExternalStore } from 'react';
import { Transcript, TranscriptUpdate } from '@/types';
import { LiveTranscriptStore } from '@/lib/live-transcript-store';
import { audioSourceLabels, mergeTranscript, normalizeAudioSource, transcriptTime } from '@/lib/transcript-audio-source';
import { toast } from 'sonner';
import { useRecordingState } from './RecordingStateContext';
import { transcriptService } from '@/services/transcriptService';
import { recordingService, type RecordingStartedPayload } from '@/services/recordingService';
import { indexedDBService } from '@/services/indexedDBService';

interface TranscriptContextType {
  store: LiveTranscriptStore;
  transcriptsRef: MutableRefObject<Transcript[]>
  addTranscript: (update: TranscriptUpdate) => void;
  copyTranscript: () => void;
  flushBuffer: () => void;
  transcriptContainerRef: React.RefObject<HTMLDivElement>;
  meetingTitle: string;
  setMeetingTitle: (title: string) => void;
  clearTranscripts: () => void;
  currentMeetingId: string | null;
  markMeetingAsSaved: () => Promise<void>;
}

const TranscriptContext = createContext<TranscriptContextType | undefined>(undefined);

export function TranscriptProvider({ children }: { children: ReactNode }) {
  const store = useMemo(() => new LiveTranscriptStore(), []);
  const [meetingTitle, setMeetingTitle] = useState('+ New Call');
  const [currentMeetingId, setCurrentMeetingId] = useState<string | null>(null);

  // Recording state context - provides backend-synced state
  const recordingState = useRecordingState();

  // Refs for transcript management
  const transcriptsRef = useMemo(() => ({ get current() { return store.snapshot(); }, set current(value: Transcript[]) { store.clear(); store.upsert(value); } }), [store]);
  const transcriptContainerRef = useRef<HTMLDivElement>(null);
  const finalFlushRef = useRef<(() => void) | null>(null);
  const bufferRef = useRef(new Map<number, Transcript>());
  const timerRef = useRef<ReturnType<typeof setTimeout>>();
  const activeRef = useRef<{ meetingId: string; sessionId: string | null; offset: number; cacheKey: string } | null>(null);

  const clearTranscripts = useCallback(() => {
    clearTimeout(timerRef.current);
    timerRef.current = undefined;
    bufferRef.current.clear();
    store.clear();
  }, [store]);
  const beginSession = useCallback((payload: RecordingStartedPayload) => {
    const sessionId = payload.recording_session_id ?? sessionStorage.getItem('recording_session_id');
    const meetingId = payload.meeting_id ?? sessionStorage.getItem('recording_meeting_id') ?? `meeting-${Date.now()}`;
    const current = activeRef.current;
    if (current?.meetingId === meetingId && current.sessionId === sessionId) {
      if (payload.timeline_offset !== undefined && current.offset !== payload.timeline_offset) {
        current.offset = payload.timeline_offset;
        const update = (item: Transcript) => item.recording_session_id === sessionId ? { ...item, timeline_offset: current.offset } : item;
        store.upsert(store.snapshot().map(update));
        for (const [key, item] of bufferRef.current) bufferRef.current.set(key, update(item));
      }
      return current;
    }
    const identity = { meetingId, sessionId, offset: payload.timeline_offset ?? 0, cacheKey: sessionId ? `${meetingId}:${sessionId}` : meetingId };
    activeRef.current = identity;
    clearTranscripts();
    setCurrentMeetingId(meetingId);
    sessionStorage.setItem('indexeddb_current_meeting_id', identity.cacheKey);
    if (payload.meeting_id) sessionStorage.setItem('recording_meeting_id', meetingId);
    if (sessionId) sessionStorage.setItem('recording_session_id', sessionId);
    return identity;
  }, [clearTranscripts]);

  const mergePassages = useCallback((incoming: Transcript[]) => {
    store.upsert(incoming);
  }, [store]);
  const flushBuffer = useCallback(() => {
    clearTimeout(timerRef.current);
    timerRef.current = undefined;
    const pending = [...bufferRef.current.values()];
    bufferRef.current.clear();
    if (pending.length) mergePassages(pending);
  }, [mergePassages]);
  finalFlushRef.current = flushBuffer;

  const addTranscript = useCallback((update: TranscriptUpdate) => {
    let identity = activeRef.current;
    if (update.recording_session_id) {
      const expected = identity?.sessionId ?? sessionStorage.getItem('recording_session_id');
      if (update.recording_session_id !== expected) return;
      if (update.meeting_id && identity && update.meeting_id !== identity.meetingId) return;
      // Native workers may emit the first result before recording-started arrives.
      if (!identity) identity = beginSession(update);
    } else if (identity?.sessionId) return;
    const passage: Transcript = {
      ...update,
      id: update.recording_session_id ? `transcript-${update.recording_session_id}-${update.sequence_id}` : `transcript-${update.sequence_id}`,
      audio_source: normalizeAudioSource(update.audio_source),
      timeline_offset: identity?.offset ?? 0,
    };
    bufferRef.current.set(update.sequence_id, mergeTranscript(bufferRef.current.get(update.sequence_id), passage));
    if (identity) void indexedDBService.saveTranscript(identity.cacheKey, update).catch(error => {
      console.warn('Transcript recovery save failed:', error);
      toast.error('Transcript recovery cache could not save. Recording audio is retained.', { id: `transcript-cache-${identity.cacheKey}` });
    });
    // Drain every received passage; sequence gaps and opaque IDs never block display.
    if (!timerRef.current) timerRef.current = setTimeout(() => { timerRef.current = undefined; flushBuffer(); }, 10);
  }, [beginSession, flushBuffer]);

  // Register once. Metadata writes and React state changes never replace listeners.
  useEffect(() => {
    let disposed = false;
    const unlisteners: (() => void)[] = [];
    const retain = (unlisten: () => void) => { if (disposed) unlisten(); else unlisteners.push(unlisten); };
    const register = async () => {
      try {
        retain(await transcriptService.onTranscriptUpdate(update => { if (!disposed) addTranscript(update); }));
        retain(await recordingService.onRecordingStarted(async payload => {
          if (disposed) return;
          const identity = beginSession(payload);
          try {
            const title = await recordingService.getRecordingMeetingName() ?? 'New meeting';
            if (disposed || activeRef.current !== identity) return;
            setMeetingTitle(title);
            await indexedDBService.saveMeetingMetadata({
              meetingId: identity.cacheKey, canonicalMeetingId: payload.meeting_id ?? sessionStorage.getItem('recording_meeting_id') ?? undefined,
              recordingSessionId: identity.sessionId ?? undefined, title, startTime: Date.now(), lastUpdated: Date.now(),
              transcriptCount: transcriptsRef.current.length, savedToSQLite: false,
            });
            const { invoke } = await import('@tauri-apps/api/core');
            const folderPath = await invoke<string>('get_meeting_folder_path');
            if (disposed || activeRef.current !== identity) return;
            const metadata = await indexedDBService.getMeetingMetadata(identity.cacheKey);
            if (metadata && folderPath && !disposed && activeRef.current === identity) {
              await indexedDBService.saveMeetingMetadata({ ...metadata, folderPath });
            }
          } catch (error) { console.warn('Transcript recovery metadata unavailable:', error); }
        }));
        retain(await recordingService.onRecordingStopped(async payload => {
          const identity = activeRef.current;
          if (disposed || !identity || (payload.recording_session_id && payload.recording_session_id !== identity.sessionId)) return;
          flushBuffer();
          try {
            const metadata = await indexedDBService.getMeetingMetadata(identity.cacheKey);
            if (metadata && payload.folder_path && !disposed && activeRef.current === identity) {
              await indexedDBService.saveMeetingMetadata({ ...metadata, folderPath: payload.folder_path });
            }
          } catch (error) { console.warn('Transcript recovery metadata unavailable:', error); }
        }));
      } catch (error) { if (!disposed) toast.error('Live transcript listener could not initialize.'); }
    };
    void register();
    void indexedDBService.init().catch(error => console.warn('Transcript recovery cache unavailable:', error));
    return () => {
      disposed = true;
      clearTimeout(timerRef.current);
      timerRef.current = undefined;
      unlisteners.forEach(unlisten => unlisten());
    };
  }, [addTranscript, beginSession, flushBuffer]);

  // Merge native reload history; a late read must never erase newer live passages.
  useEffect(() => {
    if (!recordingState.isRecording) return;
    let disposed = false;
    const before = activeRef.current;
    const sync = async () => {
      try {
        const native = await recordingService.getRecordingState();
        if (disposed || activeRef.current !== before || (!native.is_recording && !activeRef.current)) return;
        // Capture can already be stopped while a confirmed session still owns
        // its finalizing transcript. Reconcile canonical earlier passages too.
        const identity = native.is_recording ? beginSession(native) : activeRef.current!;
        const saved = await recordingService.getSavedTranscripts(identity.meetingId);
        if (disposed || activeRef.current !== identity) return;
        mergePassages(saved);
        const history = await transcriptService.getTranscriptHistory();
        if (disposed || activeRef.current !== identity) return;
        const formatted = history.map((segment: any): Transcript => ({
          id: identity.sessionId ? `transcript-${identity.sessionId}-${segment.sequence_id}` : segment.id,
          text: segment.text, timestamp: segment.display_time, sequence_id: segment.sequence_id,
          chunk_start_time: segment.audio_start_time, is_partial: false, confidence: segment.confidence,
          audio_source: normalizeAudioSource(segment.audio_source), audio_start_time: segment.audio_start_time,
          audio_end_time: segment.audio_end_time, duration: segment.duration,
          recording_session_id: identity.sessionId ?? undefined, timeline_offset: identity.offset,
        }));
        flushBuffer();
        // Prefer live values when the history snapshot contains the same sequence.
        const seen = new Set(transcriptsRef.current.filter(segment => !segment.is_partial).map(segment => segment.id));
        mergePassages(formatted.filter(segment => !seen.has(segment.id)));
        const title = await recordingService.getRecordingMeetingName();
        if (title && !disposed && activeRef.current === identity) setMeetingTitle(title);
      } catch (error) { if (!disposed) console.warn('Native transcript history unavailable:', error); }
    };
    void sync();
    return () => { disposed = true; };
  }, [recordingState.isRecording, beginSession, flushBuffer, mergePassages]);

  // Copy transcript to clipboard with recording-relative timestamps
  const copyTranscript = useCallback(() => {
    // Format timestamps as recording-relative [MM:SS] instead of wall-clock time
    const formatTime = (seconds: number | undefined): string => {
      if (seconds === undefined) return '[--:--]';
      const totalSecs = Math.floor(seconds);
      const mins = Math.floor(totalSecs / 60);
      const secs = totalSecs % 60;
      return `[${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}]`;
    };

    flushBuffer();
    const fullTranscript = store.snapshot()
      .map(t => `${formatTime(transcriptTime(t))} [${audioSourceLabels[normalizeAudioSource(t.audio_source)]}] ${t.text}`)
      .join('\n');
    navigator.clipboard.writeText(fullTranscript);

    toast.success("Transcript copied to clipboard");
  }, [store, flushBuffer]);

  // Mark current meeting as saved in IndexedDB
  const markMeetingAsSaved = useCallback(async () => {
    // Try context state first, fallback to sessionStorage
    const identity = activeRef.current;
    const meetingId = identity?.cacheKey || sessionStorage.getItem('indexeddb_current_meeting_id');

    if (!meetingId) {
      console.error('[IndexedDB] ❌ Cannot mark meeting as saved: No meeting ID available!');
      console.error('[IndexedDB] currentMeetingId:', currentMeetingId);
      console.error('[IndexedDB] sessionStorage:', sessionStorage.getItem('indexeddb_current_meeting_id'));
      return;
    }

    try {
      await indexedDBService.flushTranscriptWrites?.();
      await indexedDBService.markMeetingSaved(meetingId);

      // Clear both sources
      if (activeRef.current !== identity) return;
      activeRef.current = null;
      setCurrentMeetingId(null);
      sessionStorage.removeItem('indexeddb_current_meeting_id');
    } catch (error) {
      console.error('[IndexedDB] ❌ Failed to mark meeting as saved:', error);
    }
  }, []);

  const value: TranscriptContextType = {
    store,
    transcriptsRef,
    addTranscript,
    copyTranscript,
    flushBuffer,
    transcriptContainerRef,
    meetingTitle,
    setMeetingTitle,
    clearTranscripts,
    currentMeetingId,
    markMeetingAsSaved,
  };

  return (
    <TranscriptContext.Provider value={value}>
      {children}
    </TranscriptContext.Provider>
  );
}

export function useTranscriptActions() {
  const context = useContext(TranscriptContext);
  if (context === undefined) {
    throw new Error('useTranscripts must be used within a TranscriptProvider');
  }
  return context;
}

/** Compatibility snapshot for explicit full-history consumers and recovery. */
export function useTranscripts() {
  const context = useTranscriptActions();
  const transcripts = useSyncExternalStore(context.store.subscribeAll, context.store.snapshot, context.store.snapshot);
  return { ...context, transcripts };
}
export function useTranscriptRows() {
  const context = useTranscriptActions();
  const ids = useSyncExternalStore(context.store.subscribe, context.store.getIds, context.store.getIds);
  return { ...context, ids };
}
