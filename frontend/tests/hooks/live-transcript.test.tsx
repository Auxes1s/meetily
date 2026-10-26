import { afterAll, afterEach, beforeEach, describe, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { Transcript, TranscriptUpdate } from '../../src/types';
const originalCore = { ...await import('@tauri-apps/api/core') };
mock.module('@tauri-apps/api/core', () => ({ ...originalCore, invoke: async () => null }));
const originalState = { ...await import('../../src/contexts/RecordingStateContext') };
const originalTranscripts = { ...await import('../../src/services/transcriptService') };
const originalRecording = { ...await import('../../src/services/recordingService') };
const originalDB = { ...await import('../../src/services/indexedDBService') };
const originalStorage = globalThis.sessionStorage;
let recording = false;
let history: () => Promise<Transcript[]>;
let nativeState: () => Promise<any>;
let savedHistory: () => Promise<Transcript[]>;
const listeners = new Map<string, (payload: any) => any>();
let registered = 0;
let removed = 0;
let registration: Promise<void> | null = null;
async function listen(event: string, callback: (payload: any) => any) {
  if (registration) await registration;
  registered++; listeners.set(event, callback);
  return () => { removed++; if (listeners.get(event) === callback) listeners.delete(event); };
}
const saveTranscript = mock(async (..._args: any[]) => {});
const saveMetadata = mock(async (..._args: any[]) => {});
mock.module('../../src/contexts/RecordingStateContext', () => ({ ...originalState, useRecordingControls: () => ({ isRecording: recording }), useRecordingState: () => ({ isRecording: recording }) }));
mock.module('../../src/services/transcriptService', () => ({ ...originalTranscripts, transcriptService: {
  onTranscriptUpdate: (callback: any) => listen('transcript', callback), getTranscriptHistory: () => history(),
} }));
mock.module('../../src/services/recordingService', () => ({ ...originalRecording, recordingService: {
  onRecordingStarted: (callback: any) => listen('start', callback), onRecordingStopped: (callback: any) => listen('stop', callback),
  getRecordingState: () => nativeState(), getRecordingMeetingName: async () => 'Spoken meeting',
  getSavedTranscripts: () => savedHistory(),
} }));
mock.module('../../src/services/indexedDBService', () => ({ ...originalDB, indexedDBService: {
  init: async () => {}, saveTranscript, saveMeetingMetadata: saveMetadata, getMeetingMetadata: async () => null, markMeetingSaved: async () => {},
} }));
const { TranscriptProvider, useTranscripts } = await import('../../src/contexts/TranscriptContext');
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => originalCore);
  mock.module('../../src/contexts/RecordingStateContext', () => originalState);
  mock.module('../../src/services/transcriptService', () => originalTranscripts);
  mock.module('../../src/services/recordingService', () => originalRecording);
  mock.module('../../src/services/indexedDBService', () => originalDB);
  Object.defineProperty(globalThis, 'sessionStorage', { configurable: true, value: originalStorage });
});
let state: ReturnType<typeof useTranscripts>;
let renderer: ReactTestRenderer | undefined;
function View() { state = useTranscripts(); return <output>{state.transcripts.map(item => item.text).join('|')}</output>; }
async function show() {
  await act(async () => {
    const tree = <TranscriptProvider><View /></TranscriptProvider>;
    if (renderer) renderer.update(tree); else renderer = create(tree);
  });
}
const passage = (sequence_id = 0, text = 'First spoken phrase', overrides: Partial<TranscriptUpdate> = {}): TranscriptUpdate => ({
  meeting_id: 'meeting-A', recording_session_id: 'session-A', sequence_id, text, timestamp: '12:00:00',
  source: 'Audio', audio_source: 'microphone', chunk_start_time: sequence_id, is_partial: false, confidence: 0.8,
  audio_start_time: sequence_id, audio_end_time: sequence_id + 1, duration: 1, ...overrides,
});
async function deliver(update: TranscriptUpdate) { await act(async () => { listeners.get('transcript')!(update); await new Promise(resolve => setTimeout(resolve, 20)); }); }
async function start(meeting_id = 'meeting-A', recording_session_id = 'session-A') {
  await act(async () => { sessionStorage.setItem('recording_meeting_id', meeting_id); sessionStorage.setItem('recording_session_id', recording_session_id); void listeners.get('start')!({ meeting_id, recording_session_id }); });
}
beforeEach(() => {
  const storage = new Map<string, string>();
  Object.defineProperty(globalThis, 'sessionStorage', { configurable: true, value: { getItem: (key: string) => storage.get(key) ?? null, setItem: (key: string, value: string) => storage.set(key, value), removeItem: (key: string) => storage.delete(key) } });
  sessionStorage.setItem('recording_meeting_id', 'meeting-A'); sessionStorage.setItem('recording_session_id', 'session-A');
  listeners.clear(); registered = removed = 0; registration = null; recording = false;
  history = async () => []; nativeState = async () => ({ is_recording: true, meeting_id: 'meeting-A', recording_session_id: 'session-A' });
  savedHistory = async () => [];
  saveTranscript.mockClear(); saveMetadata.mockClear();
});
afterEach(async () => { if (renderer) await act(async () => renderer!.unmount()); renderer = undefined; });
describe('live transcript delivery through the provider', () => {
  test('continuation merges earlier recordings without sequence collisions', async () => {
    nativeState = async () => ({ is_recording: true, meeting_id: 'meeting-A', recording_session_id: 'session-B', timeline_offset: 10 });
    savedHistory = async () => [{ id: 'transcript-session-A-0', text: 'Earlier recording', timestamp: '12:00', sequence_id: 0, recording_session_id: 'session-A', audio_start_time: 1, timeline_offset: 0 }];
    sessionStorage.setItem('recording_session_id', 'session-B'); recording = true;
    await show(); await deliver(passage(0, 'Continued recording', { recording_session_id: 'session-B', audio_start_time: 1 }));
    expect(state.transcripts.map(t => t.text)).toEqual(['Earlier recording', 'Continued recording']);
    expect(state.transcripts[1].timeline_offset).toBe(10);
  });
  test('renders sequence zero with an opaque ID before stop or another event', async () => {
    await show(); await start(); await deliver(passage());
    expect(state.transcripts[0].text).toBe('First spoken phrase'); expect(state.transcripts[0].id).toBe('transcript-session-A-0');
    expect(saveTranscript).toHaveBeenCalled(); expect(registered).toBe(3); expect(removed).toBe(0);
  });
  test('renders a nonzero starting sequence and gaps in audio-time order', async () => {
    await show(); await start(); await deliver(passage(418, 'Later', { audio_start_time: 4 })); await deliver(passage(415, 'Earlier', { audio_start_time: 1 }));
    expect(state.transcripts.map(item => item.text)).toEqual(['Earlier', 'Later']);
  });
  test('duplicates upsert and a late partial cannot replace a final passage', async () => {
    await show(); await start(); await deliver(passage(0, 'Part', { is_partial: true })); await deliver(passage(0, 'Final')); await deliver(passage(0, 'Older partial', { is_partial: true, audio_source: 'unknown' }));
    expect(state.transcripts).toHaveLength(1); expect(state.transcripts[0].text).toBe('Final'); expect(state.transcripts[0].audio_source).toBe('microphone');
  });
  test('switching recording identity drops old buffered events and ignores stale sessions', async () => {
    await show(); await start(); await act(async () => { listeners.get('transcript')!(passage()); }); await start('meeting-B', 'session-B'); await deliver(passage(1, 'Late A'));
    expect(state.transcripts).toHaveLength(0); await deliver(passage(0, 'B', { meeting_id: 'meeting-B', recording_session_id: 'session-B' }));
    expect(state.transcripts[0].text).toBe('B'); expect(registered).toBe(3);
  });
  test('an early event survives recording-started and metadata state updates', async () => {
    await show(); await deliver(passage()); await start(); expect(state.transcripts[0].text).toBe('First spoken phrase'); expect(registered).toBe(3);
  });
  test('late reload history merges without erasing a newer live passage', async () => {
    let finish!: (items: any[]) => void; history = () => new Promise(resolve => { finish = resolve; }); recording = true; await show(); await deliver(passage(1, 'Live'));
    await act(async () => { finish([{ id: 'old', text: 'History', sequence_id: 0, display_time: '12:00:00', audio_start_time: 0, audio_end_time: 1 }]); });
    expect(state.transcripts.map(item => item.text)).toEqual(['History', 'Live']);
  });
  test('history from a previous recording cannot leak into a new one', async () => {
    let finish!: (items: any[]) => void; history = () => new Promise(resolve => { finish = resolve; }); recording = true; await show(); await start('meeting-B', 'session-B');
    await act(async () => { finish([{ id: 'old', text: 'Old A', sequence_id: 0 }]); }); expect(state.transcripts).toHaveLength(0);
  });
  test('a confirmed finalizing session reconciles earlier history after capture has stopped', async () => {
    await show(); await start();
    savedHistory = async () => [{ id: 'earlier', text: 'Canonical earlier recording', timestamp: '', audio_start_time: 0, is_partial: false }];
    nativeState = async () => ({ is_recording: false }); recording = true;
    await show(); await deliver(passage(1, 'Finalizing tail', { audio_start_time: 2 }));
    expect(state.transcripts.map(item => item.text)).toEqual(['Canonical earlier recording', 'Finalizing tail']);
  });
  test('manual final flush updates the synchronous ref and future passages still drain', async () => {
    await show(); await start(); await act(async () => { listeners.get('transcript')!(passage()); state.flushBuffer(); }); expect(state.transcriptsRef.current).toHaveLength(1);
    await deliver(passage(2, 'Next')); expect(state.transcripts).toHaveLength(2);
  });
  test('asynchronous listener registration cleans up even after unmount', async () => {
    let finish!: () => void; registration = new Promise(resolve => { finish = resolve; }); await show(); await act(async () => renderer!.unmount()); renderer = undefined;
    await act(async () => { finish(); }); expect(listeners.size).toBe(0); expect(removed).toBe(registered);
  });
});
