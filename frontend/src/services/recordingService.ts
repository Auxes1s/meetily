/**
 * Recording Service
 *
 * Handles all recording lifecycle Tauri backend calls and events.
 * Pure 1-to-1 wrapper - no error handling changes, exact same behavior as direct invoke/listen calls.
 */

import { invoke } from '@tauri-apps/api/core';
import { noteSessions } from '@/lib/meeting-notes-session';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface RecordingState {
  is_recording: boolean;
  is_paused: boolean;
  is_active: boolean;
  recording_duration: number | null;
  active_duration: number | null;
  meeting_id?: string;
  recording_session_id?: string;
  audio_position?: number;
  phase?: 'idle' | 'starting' | 'recording' | 'finalizing';
  timeline_offset?: number;
}

export interface RecordingStartedPayload {
  meeting_id?: string;
  recording_session_id?: string;
  timeline_offset?: number;
}

export interface RecordingStartResult {
  meeting_id: string;
  recording_session_id: string;
  timeline_offset: number;
}

export interface RecordingSession {
  id: string;
  meeting_id: string;
  ordinal: number;
  status: string;
  duration_seconds: number | null;
  timeline_offset: number;
  folder_path: string | null;
  duration_basis: string;
}

export interface RecordingStoppedPayload {
  message: string;
  folder_path?: string;
  meeting_name?: string;
  meeting_id?: string;
  recording_session_id?: string;
  persistence_error?: string;
}

// Bound the start invoke: > ~40s Bluetooth mic cold-start and ~90s worst-case
// Windows device enumeration, but still finite so a hung native start settles
// the UI into ERROR instead of an eternal STARTING spinner.
export const START_TIMEOUT_MS = 120000;

// ponytail: on timeout we only reject — no auto stop_recording. Calling stop
// against a stuck start could itself block; the ERROR state lets the user
// retry and the backend engine-lifecycle lock serializes that retry.
export function withRecordingDeadline<T>(promise: Promise<T>, timeout = START_TIMEOUT_MS, message = 'Recording start timed out after 120s'): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(message)),
      timeout
    );
    promise.then(
      (value) => { clearTimeout(timer); resolve(value); },
      (err) => { clearTimeout(timer); reject(err); }
    );
  });
}

/**
 * Recording Service
 * Singleton service for managing recording lifecycle operations
 */
export class RecordingService {
  private starting = false;
  private lifecycleAction: Promise<unknown> | null = null;
  private lifecycleListeners = new Set<() => void>();
  getLifecyclePending = () => this.lifecycleAction !== null;
  subscribeLifecycle = (notify: () => void) => { this.lifecycleListeners.add(notify); return () => { this.lifecycleListeners.delete(notify); }; };
  async runLifecycleAction<T>(operation: () => Promise<T>): Promise<T> {
    if (this.lifecycleAction) throw new Error('A recording action is already pending');
    const pending = Promise.resolve().then(operation);
    this.lifecycleAction = pending;
    this.lifecycleListeners.forEach(notify => notify());
    try { return await pending; } finally { if (this.lifecycleAction === pending) { this.lifecycleAction = null; this.lifecycleListeners.forEach(notify => notify()); } }
  }
  private cancelStart: (() => void) | null = null;

  cancelPendingStart(): boolean {
    if (!this.cancelStart) return false;
    this.cancelStart(); return true;
  }

  cancelRecordingStart(sessionId: string): Promise<boolean> {
    return withRecordingDeadline(invoke('cancel_recording_start', { recordingSessionId: sessionId }), 2000, 'Cancellation is waiting for the recorder. Check status.');
  }
  /**
   * Check if recording is currently active
   * @returns Promise<boolean>
   */
  async isRecording(): Promise<boolean> {
    return invoke<boolean>('is_recording');
  }

  /**
   * Get comprehensive recording state (includes durations)
   * @returns Promise with full recording state
   */
  async getRecordingState(): Promise<RecordingState> {
    return withRecordingDeadline(invoke<RecordingState>('get_recording_state'), 2000, 'Recording status is unavailable. Check status or restart Meetily.');
  }

  getRecordingSessions(meetingId: string): Promise<RecordingSession[]> {
    return invoke('api_get_recording_sessions', { meetingId });
  }

  openRecordingSession(meetingId: string, recordingSessionId: string): Promise<void> {
    return invoke('api_open_recording_session', { meetingId, recordingSessionId });
  }

  async getSavedTranscripts(meetingId: string): Promise<import('@/types').Transcript[]> {
    const transcripts: import('@/types').Transcript[] = [];
    for (let offset = 0; ; offset += 500) {
      const page = await invoke<import('@/types').PaginatedTranscriptsResponse>('api_get_meeting_transcripts', { meetingId, limit: 500, offset });
      if (!page) break;
      transcripts.push(...page.transcripts);
      if (!page.has_more || !page.transcripts.length) break;
    }
    return transcripts;
  }

  /**
   * Get current meeting name
   * @returns Promise<string | null>
   */
  async getRecordingMeetingName(): Promise<string | null> {
    return invoke<string | null>('get_recording_meeting_name');
  }

  /**
   * Start recording (no device configuration)
   * @returns Promise<void>
   */
  async startRecording(): Promise<void> {
    return withRecordingDeadline(invoke('start_recording'));
  }

  /**
   * Start recording with device configuration and meeting name
   * @param micDeviceName - Microphone device name (null for default)
   * @param systemDeviceName - System audio device name (null for default)
   * @param meetingName - Meeting name/title
   * @returns Promise<void>
   */
  async startRecordingWithDevices(
    micDeviceName: string | null,
    systemDeviceName: string | null,
    meetingName: string,
    targetMeetingId: string | null = sessionStorage.getItem('workspace_meeting_id'),
    preflight?: () => Promise<void>
  ): Promise<RecordingStartResult> {
    if (this.lifecycleAction) throw new Error('A recording action is already pending');
    if (this.starting) throw new Error('Recording initialization is already in progress');
    this.starting = true;
    const sessionId = crypto.randomUUID();
    let meetingId = targetMeetingId;
    let expired = false;
    let issued = false;
    const cancelled = new Promise<never>((_resolve, reject) => {
      this.cancelStart = () => reject(new Error('Recording start cancelled'));
    });
    const check = () => { if (expired) throw new Error('Recording start cancelled'); };
    const operation = (async () => {
      const current = await this.getRecordingState(); check();
      if (current.is_recording || current.phase === 'starting' || current.phase === 'finalizing') {
        throw new Error('Recording initialization or recording is already in progress');
      }
      await preflight?.(); check();
      if (meetingId) { await noteSessions.get(meetingId)?.flush(); check(); }
      else {
        const draft = await invoke<{ id: string }>('api_create_meeting_draft', { title: meetingName, timezone: Intl.DateTimeFormat().resolvedOptions().timeZone });
        check(); meetingId = draft.id;
        window.dispatchEvent(new CustomEvent('workspace-open-meeting', { detail: meetingId }));
      }
      const saved = await invoke<{ title: string; status: string }>('api_get_meeting_metadata', { meetingId }); check();
      if (saved.status === 'deleting') throw new Error('This meeting is being deleted');
      sessionStorage.setItem('recording_meeting_id', meetingId!);
      sessionStorage.setItem('recording_session_id', sessionId);
      issued = true;
      return await invoke<RecordingStartResult>('start_recording_with_devices_and_meeting', {
        micDeviceName, systemDeviceName, meetingName: saved.title,
        meetingId, recordingSessionId: sessionId,
      });
    })();
    try {
      return await withRecordingDeadline(Promise.race([operation, cancelled]));
    } catch (error) {
      expired = true;
      // Recovery runs independently: a stalled status IPC cannot keep Starting
      // displayed. Cancel the identity even if native admission hasn't run yet.
      if (issued) void withRecordingDeadline(invoke('cancel_recording_start', { recordingSessionId: sessionId }), 2000).catch(() => {});
      if (sessionStorage.getItem('recording_session_id') === sessionId) {
        sessionStorage.removeItem('recording_meeting_id');
        sessionStorage.removeItem('recording_session_id');
      }
      throw error;
    } finally {
      this.starting = false;
      this.cancelStart = null;
    }
  }

  /**
   * Stop recording and save to file
   * @param savePath - Path to save audio file
   * @returns Promise<void>
   */
  async stopRecording(savePath: string): Promise<void> {
    return invoke('stop_recording', {
      args: { save_path: savePath }
    });
  }

  /**
   * Pause active recording
   * @returns Promise<void>
   */
  async pauseRecording(): Promise<void> {
    return invoke('pause_recording');
  }

  /**
   * Resume paused recording
   * @returns Promise<void>
   */
  async resumeRecording(): Promise<void> {
    return invoke('resume_recording');
  }

  // Event Listeners

  /**
   * Listen for recording-started event
   * @param callback - Function to call when recording starts
   * @returns Promise that resolves to unlisten function
   */
  async onRecordingStarted(callback: (payload: RecordingStartedPayload) => void): Promise<UnlistenFn> {
    return listen<RecordingStartedPayload>('recording-started', event => callback(event.payload));
  }

  /**
   * Listen for recording-starting event (fires when start begins)
   * @param callback - Function to call when recording start begins
   * @returns Promise that resolves to unlisten function
   */
  async onRecordingStarting(callback: (payload: RecordingStartedPayload) => void): Promise<UnlistenFn> {
    return listen<RecordingStartedPayload>('recording-starting', event => callback(event.payload));
  }

  async onRecordingStartFailed(callback: (payload: RecordingStartedPayload & { message: string }) => void): Promise<UnlistenFn> {
    return listen<RecordingStartedPayload & { message: string }>('recording-start-failed', event => callback(event.payload));
  }

  /**
   * Listen for recording-stopped event (with metadata)
   * @param callback - Function to call when recording stops
   * @returns Promise that resolves to unlisten function
   */
  async onRecordingStopped(callback: (payload: RecordingStoppedPayload) => void): Promise<UnlistenFn> {
    return listen<RecordingStoppedPayload>('recording-stopped', (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for recording-paused event
   * @param callback - Function to call when recording is paused
   * @returns Promise that resolves to unlisten function
   */
  async onRecordingPaused(callback: (payload?: RecordingStartedPayload) => void): Promise<UnlistenFn> {
    return listen<RecordingStartedPayload>('recording-paused', event => callback(event.payload));
  }

  /**
   * Listen for recording-resumed event
   * @param callback - Function to call when recording resumes
   * @returns Promise that resolves to unlisten function
   */
  async onRecordingResumed(callback: (payload?: RecordingStartedPayload) => void): Promise<UnlistenFn> {
    return listen<RecordingStartedPayload>('recording-resumed', event => callback(event.payload));
  }

  /**
   * Listen for chunk-drop-warning event (audio buffer overflow)
   * @param callback - Function to call when chunks are dropped
   * @returns Promise that resolves to unlisten function
   */
  async onChunkDropWarning(callback: (warning: string) => void): Promise<UnlistenFn> {
    return listen<string>('chunk-drop-warning', (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for speech-detected event (VAD)
   * @param callback - Function to call when speech is detected
   * @returns Promise that resolves to unlisten function
   */
  async onSpeechDetected(callback: () => void): Promise<UnlistenFn> {
    return listen('speech-detected', callback);
  }

  /**
   * Listen for mic-device-switched event (mid-recording mic hot-swap succeeded)
   * @param callback - Function to call when the mic is switched to a new device
   * @returns Promise that resolves to unlisten function
   */
  async onMicDeviceSwitched(callback: (payload: { device_name: string }) => void): Promise<UnlistenFn> {
    return listen<{ device_name: string }>('mic-device-switched', (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for mic-unavailable event (no usable microphone at recording start —
   * recording proceeds with system audio only)
   * @param callback - Function to call when no microphone is available
   * @returns Promise that resolves to unlisten function
   */
  async onMicUnavailable(callback: () => void): Promise<UnlistenFn> {
    return listen('mic-unavailable', () => {
      callback();
    });
  }

  /**
   * Listen for mic-swap-failed event (mid-recording mic hot-swap failed)
   * @param callback - Function to call when the mic swap fails
   * @returns Promise that resolves to unlisten function
   */
  async onMicSwapFailed(callback: (payload: { error: string; device_name: string }) => void): Promise<UnlistenFn> {
    return listen<{ error: string; device_name: string }>('mic-swap-failed', (event) => {
      callback(event.payload);
    });
  }

  /**
   * Listen for mic-recovery-exhausted event (mid-recording mic recovery gave up
   * after repeated failures)
   * @param callback - Function to call when mic recovery is exhausted
   * @returns Promise that resolves to unlisten function
   */
  async onMicRecoveryExhausted(callback: (payload: { device_name: string }) => void): Promise<UnlistenFn> {
    return listen<{ device_name: string }>('mic-recovery-exhausted', (event) => {
      callback(event.payload);
    });
  }
}

// Export singleton instance
export const recordingService = new RecordingService();
