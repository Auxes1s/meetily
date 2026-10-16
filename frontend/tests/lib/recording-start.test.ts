import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
const core = { ...await import('@tauri-apps/api/core') };
const notes = { ...await import('../../src/lib/meeting-notes-session') };
const originals = { sessionStorage: globalThis.sessionStorage, window: globalThis.window, CustomEvent: globalThis.CustomEvent, timeout: globalThis.setTimeout };
let calls: { command: string; args?: any }[] = [];
let native: () => Promise<any>, flush: () => Promise<void>, check: () => Promise<any>;
mock.module('@tauri-apps/api/core', () => ({ ...core, invoke: async (command: string, args?: any) => {
  calls.push({ command, args });
  if (command === 'get_recording_state') return check();
  if (command === 'api_get_meeting_metadata') return { title: 'Keep my title', status: 'completed' };
  if (command === 'start_recording_with_devices_and_meeting') return native();
  if (command === 'cancel_recording_start') return true;
  throw Error(`Unexpected command: ${command}`);
} }));
mock.module('../../src/lib/meeting-notes-session', () => ({ ...notes, noteSessions: { get: (id: string) => id === 'selected' ? { flush: () => flush() } : undefined } }));
const { RecordingService } = await import('../../src/services/recordingService');
beforeEach(() => {
  calls = []; flush = async () => {}; check = async () => ({ is_recording: false, phase: 'idle' });
  native = async () => ({ meeting_id: 'selected', recording_session_id: 'session', timeline_offset: 10 });
  const storage = new Map([['workspace_meeting_id', 'selected']]);
  Object.defineProperty(globalThis, 'sessionStorage', { configurable: true, value: { getItem: (k: string) => storage.get(k) ?? null, setItem: (k: string, v: string) => storage.set(k, v), removeItem: (k: string) => storage.delete(k) } });
  Object.defineProperty(globalThis, 'window', { configurable: true, value: { dispatchEvent: () => {} } });
  Object.defineProperty(globalThis, 'CustomEvent', { configurable: true, value: class { constructor(public type: string, public detail: any) {} } });
  globalThis.setTimeout = ((fn: any, delay: number, ...args: any[]) => originals.timeout(fn, delay >= 2000 ? 20 : delay, ...args)) as typeof setTimeout;
});
afterEach(() => { globalThis.setTimeout = originals.timeout; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => core); mock.module('../../src/lib/meeting-notes-session', () => notes);
  for (const key of ['sessionStorage', 'window', 'CustomEvent'] as const) Object.defineProperty(globalThis, key, { configurable: true, value: originals[key] });
});
test('completed meeting keeps its identity and title', async () => {
  const result = await new RecordingService().startRecordingWithDevices(null, null, 'Title', 'selected');
  expect(result.meeting_id).toBe('selected');
  const args = calls.find(c => c.command === 'start_recording_with_devices_and_meeting')!.args;
  expect(args.meetingId).toBe('selected'); expect(args.meetingName).toBe('Keep my title');
  expect(calls.some(c => c.command === 'api_create_meeting_draft')).toBe(false);
});
test('navigation during saving cannot redirect recording', async () => {
  let finish!: () => void; flush = () => new Promise(resolve => { finish = resolve; });
  const operation = new RecordingService().startRecordingWithDevices(null, null, 'Title', 'selected');
  for (let i = 0; i < 8; i++) await Promise.resolve();
  sessionStorage.setItem('workspace_meeting_id', 'different'); finish(); await operation;
  expect(calls.find(c => c.command === 'start_recording_with_devices_and_meeting')!.args.meetingId).toBe('selected');
});
test('hung save times out and late completion cannot launch capture', async () => {
  let finish!: () => void; flush = () => new Promise(resolve => { finish = resolve; });
  await expect(new RecordingService().startRecordingWithDevices(null, null, 'Title', 'selected')).rejects.toThrow('timed out');
  finish(); await new Promise(resolve => originals.timeout(resolve, 1));
  expect(calls.some(c => c.command === 'start_recording_with_devices_and_meeting')).toBe(false);
});
test('native timeout settles without an unbounded recovery read', async () => {
  native = () => new Promise(() => {});
  await expect(new RecordingService().startRecordingWithDevices(null, null, 'Title', 'selected')).rejects.toThrow('timed out');
  expect(calls.filter(c => c.command === 'get_recording_state')).toHaveLength(1);
  expect(calls.some(c => c.command === 'cancel_recording_start')).toBe(true);
});
test('model readiness shares the attempt deadline', async () => {
  await expect(new RecordingService().startRecordingWithDevices(null, null, 'Title', 'selected', () => new Promise(() => {}))).rejects.toThrow('timed out');
  expect(calls.some(c => c.command === 'start_recording_with_devices_and_meeting')).toBe(false);
});
test('rapid starts and a still-pending native attempt cannot start twice', async () => {
  native = () => new Promise(() => {}); const service = new RecordingService();
  const first = service.startRecordingWithDevices(null, null, 'Title', 'selected');
  await expect(service.startRecordingWithDevices(null, null, 'Title', 'selected')).rejects.toThrow('already in progress');
  await expect(first).rejects.toThrow('timed out');
  check = async () => ({ is_recording: false, phase: 'starting' });
  await expect(service.startRecordingWithDevices(null, null, 'Title', 'selected')).rejects.toThrow('already in progress');
  expect(calls.filter(c => c.command === 'start_recording_with_devices_and_meeting')).toHaveLength(1);
});

test('Cancel start settles a pending save and prevents its late native call', async () => {
  let finish!: () => void; flush = () => new Promise(resolve => { finish = resolve; });
  const service = new RecordingService();
  const operation = service.startRecordingWithDevices(null, null, 'Title', 'selected');
  for (let i = 0; i < 8; i++) await Promise.resolve();
  expect(service.cancelPendingStart()).toBe(true);
  await expect(operation).rejects.toThrow('cancelled'); finish();
  await new Promise(resolve => originals.timeout(resolve, 1));
  expect(calls.some(c => c.command === 'start_recording_with_devices_and_meeting')).toBe(false);
});
