import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
const originalService = { ...await import('../../src/services/recordingService') };
const originalStorage = globalThis.sessionStorage;
const originalWindow = globalThis.window;
const listeners = new Map<string, (payload: any) => void>();
const windowListeners = new Map<string, () => void>();
let native: any;
const listener = async (name: string, callback: any) => { listeners.set(name, callback); return () => { listeners.delete(name); }; };
mock.module('../../src/services/recordingService', () => ({ ...originalService, recordingService: {
  getRecordingState: async () => native,
  onRecordingStarted: (cb: any) => listener('started', cb), onRecordingStarting: (cb: any) => listener('starting', cb),
  onRecordingStartFailed: (cb: any) => listener('failed', cb), onRecordingStopped: (cb: any) => listener('stopped', cb),
  onRecordingPaused: (cb: any) => listener('paused', cb), onRecordingResumed: (cb: any) => listener('resumed', cb),
  onMicDeviceSwitched: (cb: any) => listener('mic-switched', cb), onMicSwapFailed: (cb: any) => listener('mic-failed', cb),
  onMicUnavailable: (cb: any) => listener('mic-unavailable', cb), onMicRecoveryExhausted: (cb: any) => listener('mic-exhausted', cb),
} }));
const { RecordingStateProvider, useRecordingState, useRecordingControls, RecordingStatus } = await import('../../src/contexts/RecordingStateContext');
let state: ReturnType<typeof useRecordingState>, renderer: ReactTestRenderer | undefined;
function View() { state = useRecordingState(); return null; }
async function show() { await act(async () => { renderer = create(<RecordingStateProvider><View /></RecordingStateProvider>); }); }
beforeEach(() => {
  const storage = new Map<string, string>(); listeners.clear(); windowListeners.clear();
  native = { is_recording: false, is_paused: false, is_active: false, recording_duration: null, active_duration: null, phase: 'idle' };
  Object.defineProperty(globalThis, 'sessionStorage', { configurable: true, value: { getItem: (k: string) => storage.get(k) ?? null, setItem: (k: string, v: string) => storage.set(k, v) } });
  Object.defineProperty(globalThis, 'window', { configurable: true, value: { addEventListener: (name: string, callback: () => void) => windowListeners.set(name, callback), removeEventListener: (name: string) => windowListeners.delete(name) } });
});
afterEach(async () => { if (renderer) await act(async () => renderer!.unmount()); renderer = undefined; });
afterAll(() => {
  mock.module('../../src/services/recordingService', () => originalService);
  Object.defineProperty(globalThis, 'sessionStorage', { configurable: true, value: originalStorage });
  Object.defineProperty(globalThis, 'window', { configurable: true, value: originalWindow });
});
test('native response releases Starting even when success event is missing', async () => {
  await show();
  await act(async () => state.setStatus(RecordingStatus.STARTING));
  await act(async () => state.confirmRecordingStarted({ meeting_id: 'meeting', recording_session_id: 'current', timeline_offset: 10 }));
  expect(state.status).toBe(RecordingStatus.RECORDING); expect(state.isRecording).toBe(true); expect(state.isStartingRecording).toBe(false);
});
test('elapsed-time reconciliation updates the clock without rerendering control consumers', async () => {
  let commits = 0;
  function Controls() { useRecordingControls(); commits++; return null; }
  await act(async () => { renderer = create(<RecordingStateProvider><Controls /><View /></RecordingStateProvider>); });
  const before = commits;
  native = { ...native, recording_duration: 0.5, active_duration: 0.5 };
  await act(async () => { windowListeners.get('recording-state-refresh')!(); });
  expect(state.activeDuration).toBe(0.5); expect(commits).toBe(before);
});
test('backend reconciliation restores a live recording without event delivery', async () => {
  native = { ...native, is_recording: true, is_active: true, phase: 'recording', meeting_id: 'meeting', recording_session_id: 'current' };
  await show(); expect(state.status).toBe(RecordingStatus.RECORDING); expect(state.isStartingRecording).toBe(false);
  expect(sessionStorage.getItem('recording_session_id')).toBe('current');
});
test('stale success cannot attach to a newer request and delayed Starting cannot erase an error', async () => {
  await show(); sessionStorage.setItem('recording_session_id', 'newer');
  await act(async () => listeners.get('started')!({ meeting_id: 'meeting', recording_session_id: 'older' }));
  expect(state.isRecording).toBe(false);
  await act(async () => state.setStatus(RecordingStatus.ERROR, 'Timed out'));
  await act(async () => listeners.get('starting')!({ meeting_id: 'meeting', recording_session_id: 'newer' }));
  expect(state.status).toBe(RecordingStatus.ERROR);
});

test('stale pause and resume acknowledgements cannot change the active recording', async () => {
  await show();await act(async()=>state.confirmRecordingStarted({meeting_id:'meeting-A',recording_session_id:'session-A'}));
  await act(async()=>listeners.get('paused')!({recording_session_id:'session-old'}));expect(state.isPaused).toBe(false);
  await act(async()=>listeners.get('paused')!({recording_session_id:'session-A'}));expect(state.isPaused).toBe(true);
  await act(async()=>listeners.get('resumed')!({recording_session_id:'session-old'}));expect(state.isPaused).toBe(true);
  await act(async()=>listeners.get('resumed')!({recording_session_id:'session-A'}));expect(state.isPaused).toBe(false);
});
