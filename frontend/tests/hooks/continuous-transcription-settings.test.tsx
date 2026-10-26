import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const core = { ...await import('@tauri-apps/api/core') };
const config = { ...await import('../../src/contexts/ConfigContext') };
const switchModule = { ...await import('../../src/components/ui/switch') };
let disk: boolean | undefined;
let rejectSave = false;
let read: () => Promise<boolean>;
let renderer: ReactTestRenderer | undefined;
const writes: boolean[] = [];

mock.module('@tauri-apps/api/core', () => ({ ...core, invoke: async (command: string, args?: { enabled: boolean }) => {
  if (command === 'get_continuous_transcription') return read();
  if (command === 'set_continuous_transcription') {
    if (rejectSave) throw Error('Disk write failed');
    disk = args!.enabled; writes.push(disk); return;
  }
  throw Error(`Unexpected command: ${command}`);
} }));
mock.module('../../src/contexts/ConfigContext', () => ({ ...config, useConfig: () => ({ betaFeatures: { importAndRetranscribe: true }, toggleBetaFeature() {} }) }));
mock.module('../../src/components/ui/switch', () => ({ Switch: (props: any) => <button role="switch" {...props} /> }));
const { BetaSettings } = await import('../../src/components/BetaSettings');

const toggle = () => renderer!.root.findAllByProps({ role: 'switch' }).find(node => node.props['aria-labelledby'] === 'continuous-transcription-label')!;
async function show() { await act(async () => { renderer = create(<BetaSettings />); }); }
beforeEach(() => {
  disk = undefined; rejectSave = false; writes.length = 0;
  read = async () => disk ?? true;
});
afterEach(async () => { if (renderer) await act(async () => renderer!.unmount()); renderer = undefined; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => core);
  mock.module('../../src/contexts/ConfigContext', () => config);
  mock.module('../../src/components/ui/switch', () => switchModule);
});

test('Beta tab defaults on and saved off choice survives reopening', async () => {
  await show();
  expect(toggle().props.checked).toBe(true);
  expect(toggle().props.disabled).toBe(false);
  await act(async () => toggle().props.onCheckedChange(false));
  expect(disk).toBe(false); expect(toggle().props.checked).toBe(false);
  await act(async () => renderer!.unmount()); renderer = undefined;
  await show();
  expect(toggle().props.checked).toBe(false);
  await act(async () => toggle().props.onCheckedChange(true));
  expect(writes).toEqual([false, true]); expect(toggle().props.checked).toBe(true);
});

test('pending preference read disables the switch until the saved choice loads', async () => {
  let finish!: (value: boolean) => void;
  read = () => new Promise(resolve => { finish = resolve; });
  await show(); expect(toggle().props.disabled).toBe(true);
  await act(async () => { finish(false); });
  expect(toggle().props.disabled).toBe(false); expect(toggle().props.checked).toBe(false);
});

test('failed native save keeps the previous choice and reports the failure', async () => {
  await show(); rejectSave = true;
  await act(async () => toggle().props.onCheckedChange(false));
  expect(toggle().props.checked).toBe(true); expect(disk).toBeUndefined();
  expect(renderer!.root.findByProps({ role: 'alert' }).findByType('p').children.join('')).toContain('Disk write failed');
  rejectSave = false;
  await act(async () => toggle().props.onCheckedChange(false));
  expect(toggle().props.checked).toBe(false);
  expect(renderer!.root.findAllByProps({ role: 'alert' })).toHaveLength(0);
});

test('unavailable native setting can be reloaded without silently enabling a saved off choice', async () => {
  read = async () => { throw Error('Store unavailable'); };
  await show(); expect(toggle().props.disabled).toBe(true);
  disk = false; read = async () => disk!;
  const retry = renderer!.root.findAllByType('button').find(node => node.children.join('') === 'Reload setting')!;
  await act(async () => retry.props.onClick());
  expect(toggle().props.disabled).toBe(false); expect(toggle().props.checked).toBe(false);
});
