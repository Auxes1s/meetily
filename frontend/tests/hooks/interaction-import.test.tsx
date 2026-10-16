import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { UseImportAudioReturn } from '../../src/hooks/useImportAudio';

const core = { ...await import('@tauri-apps/api/core') };
const event = { ...await import('@tauri-apps/api/event') };
const analytics = { ...await import('../../src/lib/analytics') };
const language = { ...await import('../../src/lib/summary-language-preferences') };
const listeners = new Map<string, (event: any) => unknown>();
const calls: string[] = [];
const result = { path: '/fixture.wav', filename: 'fixture.wav', size_bytes: 100, duration_seconds: 2, format: 'wav' };
const defer = <T,>() => { let resolve!: (value: T) => void; const promise = new Promise<T>(yes => { resolve = yes; }); return { promise, resolve }; };
let ipc: (command: string) => Promise<unknown>;
let track: () => Promise<void>;
let current: UseImportAudioReturn;
let renderer: ReactTestRenderer | undefined;
const complete = mock(() => {});
mock.module('@tauri-apps/api/core', () => ({ ...core, invoke: async (command: string) => { calls.push(command); return ipc(command); } }));
mock.module('@tauri-apps/api/event', () => ({ ...event, listen: async (name: string, handler: (event: any) => unknown) => { listeners.set(name, handler); return () => listeners.delete(name); } }));
mock.module('../../src/lib/analytics', () => ({ ...analytics, default: { track: () => track(), trackError: async () => {} } }));
mock.module('../../src/lib/summary-language-preferences', () => ({ ...language, applyPinnedSummaryLanguageToMeeting: async () => {} }));
const { useImportAudio } = await import('../../src/hooks/useImportAudio');
function Harness() { current = useImportAudio({ onComplete: complete }); return null; }
beforeEach(() => {
  calls.length = 0; listeners.clear(); complete.mockClear(); track = async () => {};
  ipc = async command => command.includes('validate') ? result : command === 'is_import_in_progress_command' ? false : undefined;
});
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => core); mock.module('@tauri-apps/api/event', () => event);
  mock.module('../../src/lib/analytics', () => analytics); mock.module('../../src/lib/summary-language-preferences', () => language);
});

test('same-tick import submissions admit one job while start analytics is pending', async () => {
  await act(async () => { renderer = create(<Harness />); });
  await act(async () => { await current.validateFile('/fixture.wav'); });
  const pending = defer<void>(); track = () => pending.promise;
  let first!: Promise<void>; let second!: Promise<void>;
  await act(async () => { first = current.startImport('/fixture.wav', 'Fixture'); second = current.startImport('/fixture.wav', 'Fixture'); });
  await act(async () => { pending.resolve(); await Promise.all([first, second]); });
  expect(calls.filter(command => command === 'start_import_audio_command')).toHaveLength(1);
});

test('cancel during start preparation cannot submit an import after the dialog has closed', async () => {
  await act(async () => { renderer = create(<Harness />); });
  await act(async () => { await current.validateFile('/fixture.wav'); });
  const pending = defer<void>(); track = () => pending.promise;
  let starting!: Promise<void>;
  await act(async () => { starting = current.startImport('/fixture.wav', 'Fixture'); });
  await act(async () => current.cancelImport());
  await act(async () => { pending.resolve(); await starting; });
  expect(calls.filter(command => command === 'start_import_audio_command')).toHaveLength(0);
  expect(current.status).toBe('idle');
});

test('file-dialog admission is synchronous and reset rejects the late file result', async () => {
  const pending = defer<typeof result>(); ipc = async () => pending.promise;
  await act(async () => { renderer = create(<Harness />); });
  let first!: Promise<unknown>; let duplicate!: Promise<unknown>;
  await act(async () => { first = current.selectFile(); duplicate = current.selectFile(); current.reset(); });
  expect(calls).toEqual(['select_and_validate_audio_command']);
  await act(async () => { pending.resolve(result); await Promise.all([first, duplicate]); });
  expect(current.fileInfo).toBeNull(); expect(current.status).toBe('idle');
});

test('one terminal event is consumed once even when completion analytics is pending', async () => {
  await act(async () => { renderer = create(<Harness />); });
  await act(async () => { await current.startImport('/fixture.wav', 'Fixture'); });
  const pending = defer<void>(); track = () => pending.promise;
  const payload = { meeting_id: 'fixture', title: 'Fixture', duration_seconds: 2, segments_count: 1 };
  let first: unknown; let duplicate: unknown;
  await act(async () => { first = listeners.get('import-complete')!({ payload }); duplicate = listeners.get('import-complete')!({ payload }); });
  await act(async () => { pending.resolve(); await Promise.all([first, duplicate]); });
  expect(complete).toHaveBeenCalledTimes(1);
});

test('cancel admission is shared and keeps the import busy until the native worker is idle', async () => {
  const idle = defer<boolean>();
  ipc = async command => command === 'is_import_in_progress_command' ? idle.promise : undefined;
  await act(async () => { renderer = create(<Harness />); });
  await act(async () => current.startImport('/fixture.wav', 'Fixture'));
  let first!: Promise<void>; let duplicate!: Promise<void>;
  await act(async () => { first = current.cancelImport(); duplicate = current.cancelImport(); });
  expect(calls.filter(command => command === 'cancel_import_command')).toHaveLength(1);
  expect(current.isBusy).toBe(true); expect(current.isCancelling).toBe(true);
  await act(async () => current.startImport('/another.wav', 'Another'));
  expect(calls.filter(command => command === 'start_import_audio_command')).toHaveLength(1);
  await act(async () => { idle.resolve(false); await Promise.all([first, duplicate]); });
  expect(current.status).toBe('idle'); expect(current.isCancelling).toBe(false);
});

test('failed cancellation remains active and retryable instead of falsely reporting an idle import', async () => {
  let failCancel = true;
  ipc = async command => {
    if (command === 'cancel_import_command' && failCancel) throw Error('fixture cancellation failed');
    return command === 'is_import_in_progress_command' ? false : undefined;
  };
  await act(async () => { renderer = create(<Harness />); });
  await act(async () => current.startImport('/fixture.wav', 'Fixture'));
  await act(async () => { await expect(current.cancelImport()).rejects.toThrow('fixture cancellation failed'); });
  expect(current.status).toBe('processing'); expect(current.isCancelling).toBe(false);
  failCancel = false;
  await act(async () => current.cancelImport());
  expect(current.status).toBe('idle');
});
