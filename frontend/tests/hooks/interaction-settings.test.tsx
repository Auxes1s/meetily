import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const core = { ...await import('@tauri-apps/api/core') };
const config = { ...await import('../../src/contexts/ConfigContext') };
const recording = { ...await import('../../src/contexts/RecordingStateContext') };
const device = { ...await import('../../src/components/DeviceSelection') };
const analyticsConsent = { ...await import('../../src/components/AnalyticsConsentSwitch') };
const switchModule = { ...await import('../../src/components/ui/switch') };
const analytics = { ...await import('../../src/lib/analytics') };
const storeModule = { ...await import('@tauri-apps/plugin-store') };
const prefs = { save_folder: '/fixtures', auto_save: true, file_format: 'mp4', preferred_mic_device: null, preferred_system_device: null };
const settings = { notification_preferences: { show_recording_started: true, show_recording_stopped: true } };
const writes: any[] = [];
const deviceSelections: any[] = [];
let write: (args: any) => Promise<void>;
let notificationWrite: (value: any) => Promise<void>;
let renderer: ReactTestRenderer | undefined;
const defer = () => { let resolve!: () => void; let reject!: (reason: Error) => void; const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; };

mock.module('@tauri-apps/api/core', () => ({ ...core, invoke: async (command: string, args: any) => {
  if (command === 'get_recording_preferences') return prefs;
  if (command === 'set_recording_preferences') { writes.push(args.preferences); return write(args.preferences); }
  throw Error(`Unexpected command: ${command}`);
} }));
mock.module('../../src/contexts/ConfigContext', () => ({ ...config, useConfig: () => ({
  setSelectedDevices: (value: any) => deviceSelections.push(value), notificationSettings: settings,
  storageLocations: {}, isLoadingPreferences: false, loadPreferences: async () => {},
  updateNotificationSettings: async (value: any) => { writes.push(value); await notificationWrite(value); },
}) }));
mock.module('../../src/contexts/RecordingStateContext', () => ({ ...recording, useRecordingState: () => ({ isRecording: false }) }));
mock.module('../../src/components/DeviceSelection', () => ({ ...device, DeviceSelection: (props: any) => <div data-device-picker {...props} /> }));
mock.module('../../src/components/AnalyticsConsentSwitch', () => ({ ...analyticsConsent, default: () => null }));
mock.module('../../src/components/ui/switch', () => ({ Switch: (props: any) => <button role="switch" {...props} /> }));
mock.module('../../src/lib/analytics', () => ({ ...analytics, default: { track: async () => {} } }));
mock.module('@tauri-apps/plugin-store', () => ({ ...storeModule, Store: { load: async () => ({ get: async () => true, set: async () => {}, save: async () => {} }) } }));
const { RecordingSettings } = await import('../../src/components/RecordingSettings');
const { PreferenceSettings } = await import('../../src/components/PreferenceSettings');

beforeEach(() => { writes.length = 0; deviceSelections.length = 0; write = async () => {}; notificationWrite = async () => {}; });
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => core); mock.module('../../src/contexts/ConfigContext', () => config);
  mock.module('../../src/contexts/RecordingStateContext', () => recording); mock.module('../../src/components/DeviceSelection', () => device);
  mock.module('../../src/components/AnalyticsConsentSwitch', () => analyticsConsent); mock.module('../../src/components/ui/switch', () => switchModule);
  mock.module('../../src/lib/analytics', () => analytics); mock.module('@tauri-apps/plugin-store', () => storeModule);
});

test('recording preference clicks admit one save and lock related snapshot controls until acknowledgment', async () => {
  const pending = defer(); write = () => pending.promise;
  await act(async () => { renderer = create(<RecordingSettings />); });
  const toggle = renderer!.root.findAllByProps({ role: 'switch' })[0].props.onCheckedChange;
  const changeDevice = renderer!.root.findByProps({ 'data-device-picker': true }).props.onDeviceChange;
  let first: Promise<void>;
  await act(async () => { first = toggle(false); void toggle(true); void changeDevice({ micDevice: 'duplicate', systemDevice: null }); });
  expect(writes).toHaveLength(1);
  expect(renderer!.root.findAllByProps({ role: 'switch' })[0].props.disabled).toBe(true);
  expect(renderer!.root.findByProps({ 'data-device-picker': true }).props.disabled).toBe(true);
  await act(async () => { pending.resolve(); await first; });
  expect(renderer!.root.findAllByProps({ role: 'switch' })[0].props.checked).toBe(false);
});

test('failed recording preference save restores the last committed choice and allows retry', async () => {
  const pending = defer(); write = () => pending.promise;
  await act(async () => { renderer = create(<RecordingSettings />); });
  let saving: Promise<void>;
  await act(async () => { saving = renderer!.root.findAllByProps({ role: 'switch' })[0].props.onCheckedChange(false); });
  await act(async () => { pending.reject(Error('fixture disk failure')); await saving; });
  expect(renderer!.root.findAllByProps({ role: 'switch' })[0].props.checked).toBe(true);
  expect(renderer!.root.findAllByProps({ role: 'switch' })[0].props.disabled).toBe(false);
  write = async () => {};
  await act(async () => renderer!.root.findAllByProps({ role: 'switch' })[0].props.onCheckedChange(false));
  expect(writes).toHaveLength(2);
});

test('general notification writes cannot overlap and failed saves leave an operable acknowledged choice', async () => {
  const pending = defer(); notificationWrite = () => pending.promise;
  await act(async () => { renderer = create(<PreferenceSettings />); });
  const toggle = renderer!.root.findByProps({ role: 'switch' }).props.onCheckedChange;
  await act(async () => { void toggle(false); });
  expect(writes).toHaveLength(1);
  expect(renderer!.root.findByProps({ role: 'switch' }).props.disabled).toBe(true);
  await act(async () => { void toggle(true); });
  expect(writes).toHaveLength(1);
  await act(async () => { pending.reject(Error('fixture notification failure')); });
  expect(renderer!.root.findByProps({ role: 'switch' }).props.checked).toBe(true);
  expect(renderer!.root.findByProps({ role: 'switch' }).props.disabled).toBe(false);
});
