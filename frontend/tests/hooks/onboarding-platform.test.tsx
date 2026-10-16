import { afterAll, expect, mock, test } from 'bun:test';
import { act, create } from 'react-test-renderer';

const onboarding = { ...await import('../../src/contexts/OnboardingContext') };
const steps = { ...await import('../../src/components/onboarding/steps') };
const os = { ...await import('@tauri-apps/plugin-os') };
let currentStep = 4;
let target = 'windows';
const navigations: number[] = [];
mock.module('../../src/contexts/OnboardingContext', () => ({ ...onboarding, useOnboarding: () => ({
  currentStep, goToStep: (step: number) => navigations.push(step),
}) }));
mock.module('../../src/components/onboarding/steps', () => ({ ...steps,
  WelcomeStep: () => <div data-step="welcome" />,
  SetupOverviewStep: () => <div data-step="overview" />,
  DownloadProgressStep: () => <div data-step="downloads" />,
  PermissionsStep: () => <div data-step="permissions" />,
}));
mock.module('@tauri-apps/plugin-os', () => ({ ...os, platform: () => target }));
const { OnboardingFlow } = await import('../../src/components/onboarding/OnboardingFlow');
afterAll(() => {
  mock.module('../../src/contexts/OnboardingContext', () => onboarding);
  mock.module('../../src/components/onboarding/steps', () => steps);
  mock.module('@tauri-apps/plugin-os', () => os);
});

test('a resumed Windows profile on step four returns to downloads instead of a blank page', async () => {
  target = 'windows'; currentStep = 4; navigations.length = 0;
  let renderer: ReturnType<typeof create>;
  await act(async () => { renderer = create(<OnboardingFlow onComplete={() => {}} />); });
  expect(navigations).toEqual([3]);
  expect(renderer!.root.findByProps({ role: 'status' }).children).toContain('Restoring setup…');
  await act(async () => { currentStep = 3; renderer!.update(<OnboardingFlow onComplete={() => {}} />); });
  expect(renderer!.root.findByProps({ 'data-step': 'downloads' })).toBeDefined();
  await act(async () => renderer!.unmount());
});

test('a resumed macOS profile retains its permissions step', async () => {
  target = 'macos'; currentStep = 4; navigations.length = 0;
  let renderer: ReturnType<typeof create>;
  await act(async () => { renderer = create(<OnboardingFlow onComplete={() => {}} />); });
  expect(navigations).toEqual([]);
  expect(renderer!.root.findByProps({ 'data-step': 'permissions' })).toBeDefined();
  await act(async () => renderer!.unmount());
});
