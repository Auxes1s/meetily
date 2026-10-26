import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranscriptActions } from '@/contexts/TranscriptContext';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';
import { useConfig } from '@/contexts/ConfigContext';
import { useRecordingControls, RecordingStatus } from '@/contexts/RecordingStateContext';
import { recordingService } from '@/services/recordingService';
import Analytics from '@/lib/analytics';
import { showRecordingNotification } from '@/lib/recordingNotification';
import { getProviderCommands, hasDownloadingModel, type ModelWithStatus } from '@/lib/transcription-model-readiness';
import { toast } from 'sonner';
import { noteSessions } from '@/lib/meeting-notes-session';

const RUNTIME_ERROR = 'TRANSCRIPTION_RUNTIME_INITIALIZATION_FAILED';
const MODEL_NOT_READY = 'TRANSCRIPTION_MODEL_NOT_READY';
const RUNTIME_MESSAGE = 'Speech recognition could not initialize. Restart Meetily. If the problem continues, repair or reinstall the app.';

/** All UI entry points share one synchronous admission guard and one watchdog. */
export function useRecordingStart(
  isRecording: boolean,
  setIsRecording: (value: boolean) => void,
  showModal?: (name: 'modelSelector', message?: string) => void
): { handleRecordingStart: (meetingId?: string | null) => Promise<void>; isAutoStarting: boolean } {
  const [isAutoStarting, setIsAutoStarting] = useState(false);
  const starting = useRef(false);
  const { setMeetingTitle } = useTranscriptActions();
  const { setIsMeetingActive } = useSidebar();
  const { selectedDevices } = useConfig();
  const { setStatus, confirmRecordingStarted } = useRecordingControls();

  const prepareModel = useCallback(async () => {
    const config = await invoke<{ provider?: string } | null>('api_get_transcript_config');
    const commands = getProviderCommands(config?.provider ?? 'parakeet');
    if (commands) {
      await invoke(commands.initialize);
      if (await invoke<boolean>(commands.hasAvailableModels)) return;
    }
    const downloading = commands && hasDownloadingModel(await invoke<ModelWithStatus[]>(commands.getAvailableModels));
    if (downloading) {
      toast.info('Model download in progress', { description: 'Please wait for the transcription model to finish downloading before recording.' });
    } else {
      toast.error('Transcription model not ready', { description: 'Please download a transcription model before recording.' });
      showModal?.('modelSelector', 'Transcription model setup required');
    }
    throw new Error(MODEL_NOT_READY);
  }, [showModal]);

  const handleRecordingStart = useCallback(async (targetMeetingId?: string | null) => {
    if (starting.current || isRecording) return;
    starting.current = true;
    setIsAutoStarting(true);
    // Snapshot selection before any awaited model/save/native operation.
    const meetingId = targetMeetingId === undefined ? sessionStorage.getItem('workspace_meeting_id') : targetMeetingId;
    const title = noteSessions.get(meetingId ?? '')?.getSnapshot().title ?? `Meeting ${new Date().toLocaleString()}`;
    setStatus(RecordingStatus.STARTING, 'Preparing recording…');
    try {
      const result = await recordingService.startRecordingWithDevices(
        selectedDevices?.micDevice ?? null, selectedDevices?.systemDevice ?? null,
        title, meetingId, prepareModel
      );
      // The command response is sufficient evidence even if its event was lost.
      confirmRecordingStarted(result);
      setStatus(RecordingStatus.RECORDING);
      setMeetingTitle(title);
      setIsRecording(true);
      setIsMeetingActive(true);
      Analytics.trackButtonClick('start_recording', 'home_page');
      void showRecordingNotification().catch(error => console.warn('Recording notification unavailable:', error));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (message === MODEL_NOT_READY) { setStatus(RecordingStatus.IDLE); return; }
      if (message === 'Recording start cancelled') {
        const native = await recordingService.getRecordingState().catch(() => null);
        if (native?.is_recording) {
          confirmRecordingStarted(native); setStatus(RecordingStatus.RECORDING); setIsRecording(true);
        } else {
          setStatus(native?.phase === 'idle' ? RecordingStatus.IDLE : RecordingStatus.ERROR, 'Cancellation requested. Check status before retrying.');
        }
        return;
      }
      if (message.includes('already in progress')) {
        const native = await recordingService.getRecordingState().catch(() => null);
        if (native?.is_recording && native.meeting_id && native.recording_session_id) {
          confirmRecordingStarted(native);
          setStatus(RecordingStatus.RECORDING);
          setIsRecording(true);
          setIsMeetingActive(true);
          return;
        }
      }
      setStatus(RecordingStatus.ERROR, message === RUNTIME_ERROR ? RUNTIME_MESSAGE : message);
      setIsRecording(false);
      Analytics.trackButtonClick('start_recording_error', 'home_page');
      if (message === RUNTIME_ERROR) return;
      throw error;
    } finally {
      starting.current = false;
      setIsAutoStarting(false);
    }
  }, [isRecording, selectedDevices, prepareModel, setStatus, confirmRecordingStarted, setMeetingTitle, setIsRecording, setIsMeetingActive]);

  const latestStart = useRef(handleRecordingStart);
  latestStart.current = handleRecordingStart;
  useEffect(() => {
    const start = (event?: Event) => { void latestStart.current((event as CustomEvent<{ meetingId?: string | null }> | undefined)?.detail?.meetingId).catch(error => toast.error(String(error))); };
    if (sessionStorage.getItem('autoStartRecording') === 'true') {
      sessionStorage.removeItem('autoStartRecording');
      start();
    }
    window.addEventListener('start-recording-from-sidebar', start);
    return () => window.removeEventListener('start-recording-from-sidebar', start);
  }, []);

  return { handleRecordingStart, isAutoStarting };
}
