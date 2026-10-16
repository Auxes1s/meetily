import { useCallback, useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import { useTranscriptActions } from '@/contexts/TranscriptContext';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';
import { useRecordingState, RecordingStatus } from '@/contexts/RecordingStateContext';
import { recordingService, type RecordingStoppedPayload } from '@/services/recordingService';
import { storageService } from '@/services/storageService';
import { flushAllNotes } from '@/lib/meeting-notes-session';
import { applyPinnedSummaryLanguageToMeeting, detectAndCacheSummaryLanguage } from '@/lib/summary-language-preferences';
import Analytics from '@/lib/analytics';

/** Native stop drains and commits final segments before it acknowledges completion. */
export function useRecordingStop(setIsRecording: (value: boolean) => void, setIsRecordingDisabled: (value: boolean) => void) {
  const state = useRecordingState();
  const { transcriptsRef, flushBuffer, clearTranscripts, meetingTitle, markMeetingAsSaved } = useTranscriptActions();
  const { refetchMeetings, setCurrentMeeting, setIsMeetingActive } = useSidebar();
  const router = useRouter();
  const stopping = useRef(false);
  const stopped = useRef<RecordingStoppedPayload | null>(null);
  const completed = useRef<string | null>(null);
  useEffect(() => {
    let disposed = false;
    const listeners: (() => void)[] = [];
    const register = async () => {
      for (const [event, callback] of [
        ['recording-stopped', (event: any) => { stopped.current = event.payload; }],
        ['recording-started', () => { stopped.current = null; completed.current = null; }],
      ] as const) {
        const unlisten = await listen(event, callback);
        if (disposed) unlisten(); else listeners.push(unlisten);
      }
    };
    void register();
    return () => { disposed = true; listeners.forEach(fn => fn()); };
  }, []);

  const handleRecordingStop = useCallback(async (callApi = true, expectedSessionId?: string | null) => {
    if (stopping.current) return;
    stopping.current = true;
    const sessionId = expectedSessionId ?? sessionStorage.getItem('recording_session_id') ?? stopped.current?.recording_session_id;
    const ownsCleanup = () => { const current = sessionStorage.getItem('recording_session_id'); return !current || current === sessionId; };
    try {
      if (!ownsCleanup()) return;
      const native = await recordingService.getRecordingState();
      if (!ownsCleanup()) return;
      if (native.is_recording || native.phase === 'starting') {
        if (native.is_recording) {
          setIsRecording(true);
          toast.error('Recording is still active. Use Stop to finish capture before saving.');
        }
        return;
      }
      const activeId = sessionStorage.getItem('recording_meeting_id') || stopped.current?.meeting_id;
      if (sessionId && completed.current === sessionId) return;
      if (!callApi && !activeId) return;
      setIsRecording(false);
      setIsRecordingDisabled(true);
      state.setStatus(RecordingStatus.SAVING);
      await flushAllNotes().catch(() => toast.error('Original notes are still unsaved. Your edits are retained.'));
      if (!ownsCleanup()) return;
      flushBuffer();
      const finalSegments = transcriptsRef.current.filter(segment => !segment.is_partial && (!sessionId || segment.recording_session_id === sessionId));
      let meetingId = activeId;
      if (meetingId) {
        const metadata = await invoke<{ status: string; recording_session_id?: string }>('api_get_meeting_metadata', { meetingId });
        if (metadata.status !== 'completed' || (sessionId && metadata.recording_session_id !== sessionId)) {
          throw new Error(stopped.current?.persistence_error || 'Recording finalization was interrupted. Recover the available transcript from the recovery panel.');
        }
      } else {
        // Compatibility for recordings started by the old native tray path.
        if (!callApi || !finalSegments.length) { state.setStatus(RecordingStatus.IDLE); return; }
        const response = await storageService.saveMeeting(stopped.current?.meeting_name || meetingTitle || 'New Meeting', finalSegments, stopped.current?.folder_path ?? null);
        meetingId = response.meeting_id;
      }
      if (!meetingId) throw new Error('No saved meeting was returned');
      try {
        if (!(await applyPinnedSummaryLanguageToMeeting(meetingId)) && finalSegments.length) {
          await detectAndCacheSummaryLanguage(meetingId, finalSegments.map(segment => segment.text));
        }
      } catch { toast.warning('Meeting saved; summary language preference could not be applied.'); }
      if (!ownsCleanup()) return;
      await markMeetingAsSaved().catch(() => toast.warning('Meeting saved in SQLite; transcript recovery cache cleanup will be retried.'));
      if (!ownsCleanup()) return;
      clearTranscripts();
      completed.current = sessionId ?? meetingId;
      sessionStorage.removeItem('recording_meeting_id');
      sessionStorage.removeItem('recording_session_id');
      sessionStorage.removeItem('indexeddb_current_meeting_id');
      sessionStorage.removeItem('last_recording_folder_path');
      sessionStorage.removeItem('last_recording_meeting_name');
      await refetchMeetings();
      if (!ownsCleanup()) return;
      window.dispatchEvent(new CustomEvent('meeting-data-changed', { detail: { meetingId } }));
      setIsMeetingActive(false);
      state.setStatus(RecordingStatus.IDLE);
      const savedId = meetingId;
      toast.success('Recording saved', { action: { label: 'View meeting', onClick: () => router.push(`/meeting-details?id=${savedId}`) } });
      if (sessionStorage.getItem('workspace_meeting_id') === savedId) {
        const meeting = await storageService.getMeeting(savedId);
        if (!ownsCleanup()) return;
        setCurrentMeeting({ id: savedId, title: meeting.title });
        router.push(`/meeting-details?id=${savedId}&source=recording`);
      }
      void Analytics.updateMeetingCount().catch(() => {});
    } catch (error: any) {
      if (!ownsCleanup()) return;
      state.setStatus(RecordingStatus.ERROR, error?.message ?? String(error));
      toast.error(error?.message ?? 'Recording could not be finalized. Recovery data is preserved.');
    } finally { stopping.current = false; if (ownsCleanup()) setIsRecordingDisabled(false); }
  }, [setIsRecording, setIsRecordingDisabled, state.setStatus, transcriptsRef, flushBuffer, clearTranscripts, meetingTitle, markMeetingAsSaved, refetchMeetings, setCurrentMeeting, setIsMeetingActive, router]);

  const handler = useRef(handleRecordingStop); handler.current = handleRecordingStop;
  useEffect(() => {
    (window as any).handleRecordingStop = (callApi = true, sessionId?: string | null) => handler.current(callApi, sessionId);
    return () => { delete (window as any).handleRecordingStop; };
  }, []);
  return { handleRecordingStop, isStopping: state.isStopping, isProcessingTranscript: state.isProcessing, isSavingTranscript: state.isSaving, summaryStatus: state.isProcessing ? 'processing' : 'idle', setIsStopping: (value: boolean) => state.setStatus(value ? RecordingStatus.STOPPING : RecordingStatus.IDLE) };
}
