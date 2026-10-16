import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const core = { ...await import('@tauri-apps/api/core') };
const events = { ...await import('@tauri-apps/api/event') };
const os = { ...await import('@tauri-apps/plugin-os') };
const context = { ...await import('../../src/contexts/OnboardingContext') };
const container = { ...await import('../../src/components/onboarding/OnboardingContainer') };
const toasts = { ...await import('sonner') };
let available: () => Promise<boolean>;
let checks = 0;
let completed = 0;
const states: boolean[] = [];
const errors: string[] = [];
let renderer: ReactTestRenderer | undefined;

mock.module('@tauri-apps/api/core', () => ({ ...core, invoke: async (command: string) => {
  if (command === 'parakeet_init') return;
  if (command === 'parakeet_has_available_models') { checks++; return available(); }
  throw Error('Unexpected command: ' + command);
} }));
mock.module('@tauri-apps/api/event', () => ({ ...events, listen: async () => () => {} }));
mock.module('@tauri-apps/plugin-os', () => ({ ...os, platform: () => 'windows' }));
mock.module('../../src/contexts/OnboardingContext', () => ({ ...context, useOnboarding: () => ({
  goNext: () => { completed++; }, completeOnboarding: async () => { completed++; },
  selectedSummaryModel: 'fixture-model', recommendedSummaryModel: 'fixture-model',
  parakeetDownloaded: true, summaryModelDownloaded: true,
  setParakeetDownloaded: (value: boolean) => states.push(value),
  setSummaryModelDownloaded: () => {}, startBackgroundDownloads: async () => {},
}) }));
mock.module('../../src/components/onboarding/OnboardingContainer', () => ({ ...container, OnboardingContainer: ({ children }: any) => <div>{children}</div> }));
mock.module('sonner', () => ({ ...toasts, toast: { ...toasts.toast, error: (message: string) => errors.push(message) } }));
const { DownloadProgressStep } = await import('../../src/components/onboarding/steps/DownloadProgressStep');

beforeEach(() => { available = async () => true; checks = 0; completed = 0; states.length = 0; errors.length = 0; });
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => core); mock.module('@tauri-apps/api/event', () => events);
  mock.module('@tauri-apps/plugin-os', () => os); mock.module('../../src/contexts/OnboardingContext', () => context);
  mock.module('../../src/components/onboarding/OnboardingContainer', () => container); mock.module('sonner', () => toasts);
});
const mount = async () => { await act(async () => { renderer = create(<DownloadProgressStep />); }); };
const continueButton = () => renderer!.root.findAllByType('button').find(button => button.children.includes('Continue'))!;

test('a stale downloaded flag cannot complete setup when the native model is missing', async () => {
  available = async () => false;
  await mount();
  await act(async () => { await continueButton().props.onClick(); });
  expect(completed).toBe(0);
  expect(states).toContain(false);
  expect(errors).toContain('Transcription engine required');
  expect(renderer!.root.findAllByType('button').some(button => button.children.includes('Try Again'))).toBe(true);
});

test('repeated Continue clicks share one verification and a failed check remains retryable', async () => {
  let reject!: (error: Error) => void;
  available = () => new Promise((_, no) => { reject = no; });
  await mount();
  const click = continueButton().props.onClick;
  let pending!: Promise<void>;
  await act(async () => { pending = click(); void click(); });
  expect(checks).toBe(1);
  await act(async () => { reject(Error('fixture native verification error')); await pending; });
  expect(completed).toBe(0);
  expect(errors).toContain('Could not verify transcription engine');
  expect(continueButton().props.disabled).toBe(false);
});
