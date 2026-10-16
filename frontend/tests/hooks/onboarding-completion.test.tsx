import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const core = { ...await import('@tauri-apps/api/core') };
const events = { ...await import('@tauri-apps/api/event') };
const calls: string[] = [];
const defer = <T,>() => {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};
const status = (completed = false) => ({
  version: '1.0', completed, current_step: 2,
  model_status: { parakeet: 'not_downloaded', summary: 'downloaded', selected_summary_model: 'fixture-model' },
  last_updated: '2026-10-16T00:00:00Z',
});
let load: () => Promise<ReturnType<typeof status> | null>;
let save: () => Promise<void>;
let complete: () => Promise<void>;
let current: ReturnType<typeof useOnboarding>;
let renderer: ReactTestRenderer | undefined;

mock.module('@tauri-apps/api/core', () => ({ ...core, invoke: async (command: string) => {
  calls.push(command);
  if (command === 'get_onboarding_status') return load();
  if (command === 'check_first_launch') return false;
  if (command === 'builtin_ai_get_recommended_model') return 'fixture-model';
  if (command === 'builtin_ai_is_model_ready') return true;
  if (command === 'parakeet_init') return;
  if (command === 'parakeet_has_available_models' || command === 'parakeet_is_downloading') return false;
  if (command === 'save_onboarding_status_cmd') return save();
  if (command === 'complete_onboarding' || command === 'complete_notes_only_setup') return complete();
  throw Error(`Unexpected onboarding command: ${command}`);
} }));
mock.module('@tauri-apps/api/event', () => ({ ...events, listen: async () => () => {} }));
const { OnboardingProvider, useOnboarding } = await import('../../src/contexts/OnboardingContext');
function Probe() { current = useOnboarding(); return null; }
const mount = async () => { await act(async () => { renderer = create(<OnboardingProvider><Probe /></OnboardingProvider>); }); };
const settleDebounce = async () => { await act(async () => { await new Promise(resolve => setTimeout(resolve, 1100)); }); };

beforeEach(() => { calls.length = 0; load = async () => status(); save = async () => {}; complete = async () => {}; });
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; });
afterAll(() => { mock.module('@tauri-apps/api/core', () => core); mock.module('@tauri-apps/api/event', () => events); });

test('slow status restoration never overwrites a completed profile with initial defaults', async () => {
  const pending = defer<ReturnType<typeof status>>(); load = () => pending.promise;
  await mount();
  await settleDebounce();
  expect(calls.filter(call => call === 'save_onboarding_status_cmd')).toHaveLength(0);
  await act(async () => { pending.resolve(status(true)); });
  await settleDebounce();
  expect(current.currentStep).toBe(2);
  expect(calls.filter(call => call === 'save_onboarding_status_cmd')).toHaveLength(0);
});

test('Finish waits for an admitted progress save and deduplicates repeated completion', async () => {
  const pendingSave = defer<void>(); save = () => pendingSave.promise;
  await mount();
  await settleDebounce();
  expect(calls.filter(call => call === 'save_onboarding_status_cmd')).toHaveLength(1);
  let first!: Promise<void>; let second!: Promise<void>;
  await act(async () => { first = current.completeOnboarding(); second = current.completeOnboarding(); });
  expect(first).toBe(second);
  expect(calls).not.toContain('complete_onboarding');
  await act(async () => { pendingSave.resolve(); await Promise.all([first, second]); });
  expect(calls.filter(call => call === 'complete_onboarding')).toHaveLength(1);
  await act(async () => { current.goToStep(3); current.setParakeetDownloaded(true); });
  await settleDebounce();
  expect(calls.filter(call => call === 'save_onboarding_status_cmd')).toHaveLength(1);
});

test('a failed Finish preserves setup and admits a retry', async () => {
  await mount();
  complete = async () => { throw Error('fixture persistence failure'); };
  await act(async () => { await expect(current.completeOnboarding()).rejects.toThrow('fixture persistence failure'); });
  expect(current.currentStep).toBe(2);
  complete = async () => {};
  await act(async () => { await current.completeOnboarding(); });
  expect(calls.filter(call => call === 'complete_onboarding')).toHaveLength(2);
});

test('notes-only completion owns a late status load and prevents duplicate setup', async () => {
  const pendingLoad = defer<ReturnType<typeof status>>(); load = () => pendingLoad.promise;
  const pendingCompletion = defer<void>(); complete = () => pendingCompletion.promise;
  await mount();
  let finishing!: Promise<void>;
  await act(async () => { finishing = current.completeNotesOnlySetup(); void current.completeNotesOnlySetup(); });
  expect(calls.filter(call => call === 'complete_notes_only_setup')).toHaveLength(1);
  await act(async () => { pendingLoad.resolve(status()); pendingCompletion.resolve(); await finishing; });
  expect(current.currentStep).toBe(1);
  await settleDebounce();
  expect(calls).not.toContain('save_onboarding_status_cmd');
});

test('a failed status read does not replace persisted setup with default state', async () => {
  load = async () => { throw Error('fixture store unavailable'); };
  await mount();
  await settleDebounce();
  expect(calls).not.toContain('save_onboarding_status_cmd');
});
