import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const originals = {
  core: { ...await import('@tauri-apps/api/core') }, event: { ...await import('@tauri-apps/api/event') },
  path: { ...await import('@tauri-apps/api/path') }, state: { ...await import('../../src/contexts/RecordingStateContext') },
  tooltip: { ...await import('../../src/components/ui/tooltip') }, analytics: { ...await import('../../src/lib/analytics') },
  alert: globalThis.alert,
};
let calls: string[] = [], state: any, start: () => Promise<void>, stopped: () => Promise<void>;
let native: (command: string) => Promise<any>;
let registerListen: () => Promise<() => void>;
const wrapper = ({ children }: any) => <div>{children}</div>;
mock.module('@tauri-apps/api/core', () => ({ ...originals.core, invoke: async (command: string) => { calls.push(command); return native(command); } }));
mock.module('@tauri-apps/api/event', () => ({ ...originals.event, listen: () => registerListen() }));
mock.module('@tauri-apps/api/path', () => ({ ...originals.path, appDataDir: async () => '/isolated-test' }));
mock.module('../../src/contexts/RecordingStateContext', () => ({ ...originals.state, useRecordingControls: () => state, useRecordingState: () => state }));
mock.module('../../src/components/ui/tooltip', () => ({ Tooltip: wrapper, TooltipContent: wrapper, TooltipProvider: wrapper, TooltipTrigger: wrapper }));
mock.module('../../src/lib/analytics', () => ({ ...originals.analytics, default: { trackButtonClick() {}, trackTranscriptionSuccess() {}, trackTranscriptionError() {} } }));
const { RecordingControls } = await import('../../src/components/RecordingControls');
let renderer: ReactTestRenderer | undefined;
const props = () => ({ isRecording: state.isRecording, barHeights: ['4px'], onRecordingStart: () => start(), onRecordingStop: () => stopped(), onTranscriptReceived() {}, isRecordingDisabled: false, isParentProcessing: state.isProcessing || state.isSaving });
const control = (label: string) => renderer!.root.findByProps({ 'aria-label': label });
const deferred = () => { let resolve!: () => void, reject!: (e: Error) => void; const promise = new Promise<void>((a,b) => { resolve=a; reject=b; }); return { promise, resolve, reject }; };
async function show() { await act(async () => { renderer = create(<RecordingControls {...props()} />); }); calls=[]; }
beforeEach(() => {
  calls=[]; state={ isRecording: true, isPaused: false, isStartingRecording: false, isStopping: false, isProcessing: false, isSaving: false, status: 'recording' };
  native=async () => {}; start=async () => {}; stopped=async () => {};
  registerListen=async () => () => {};
  globalThis.alert=() => {};
});
afterEach(async () => { if(renderer) await act(async () => renderer!.unmount()); renderer=undefined; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => originals.core); mock.module('@tauri-apps/api/event', () => originals.event); mock.module('@tauri-apps/api/path', () => originals.path);
  mock.module('../../src/contexts/RecordingStateContext', () => originals.state); mock.module('../../src/components/ui/tooltip', () => originals.tooltip); mock.module('../../src/lib/analytics', () => originals.analytics);
  globalThis.alert=originals.alert;
});
test('rapid pause and overlapping stop admit only one lifecycle action, then recover after failure', async () => {
  const gate=deferred(); native=command => command==='pause_recording' ? gate.promise : Promise.resolve(); await show();
  const pause=control('Pause recording').props.onClick, stop=control('Stop recording').props.onClick;
  await act(async () => { for(let i=0;i<30;i++) pause(); stop(); });
  expect(calls.filter(c => c==='pause_recording')).toHaveLength(1); expect(calls).not.toContain('stop_recording');
  await act(async () => gate.reject(new Error('Paused request failed')));
  native=async () => {};
  await act(async () => control('Pause recording').props.onClick());
  expect(calls.filter(c => c==='pause_recording')).toHaveLength(2);
});
test('rapid resume admits one request and releases its guard on rejection', async () => {
  state.isPaused=true; const gate=deferred(); native=command => command==='resume_recording' ? gate.promise : Promise.resolve(); await show();
  const resume=control('Resume recording').props.onClick;
  await act(async () => { for(let i=0;i<30;i++) resume(); });
  expect(calls.filter(c => c==='resume_recording')).toHaveLength(1);
  await act(async () => gate.reject(new Error('Resume failed'))); native=async () => {};
  await act(async () => control('Resume recording').props.onClick());
  expect(calls.filter(c => c==='resume_recording')).toHaveLength(2);
});
test('rapid stop sends one native stop and awaits finalization before releasing controls', async () => {
  const gate=deferred(), finalization=deferred(); native=command => command==='stop_recording' ? gate.promise : Promise.resolve(); stopped=() => finalization.promise; await show();
  const stop=control('Stop recording').props.onClick;
  await act(async () => { for(let i=0;i<30;i++) stop(); });
  expect(calls.filter(c => c==='stop_recording')).toHaveLength(1);
  await act(async () => gate.resolve());
  expect(JSON.stringify(renderer!.toJSON())).toContain('Processing recording...');
  expect(renderer!.root.findAllByProps({ 'aria-label': 'Stop recording' })).toHaveLength(0);
  await act(async () => finalization.resolve());
  expect(control('Stop recording').props.disabled).toBe(false);
});
test('already stopped native response cannot leave the processing spinner stuck', async () => {
  native=async command => { if(command==='stop_recording') throw new Error('No recording in progress'); }; await show();
  await act(async () => control('Stop recording').props.onClick());
  expect(JSON.stringify(renderer!.toJSON())).not.toContain('Processing recording...');
  expect(control('Stop recording').props.disabled).toBe(false);
});
test('rapid start invokes its callback once and parent finalization disables start', async () => {
  state.isRecording=false; state.status='idle'; const gate=deferred(); let count=0; start=() => { count++; return gate.promise; }; await show();
  const click=control('Start recording').props.onClick;
  await act(async () => { for(let i=0;i<30;i++) click(); }); expect(count).toBe(1);
  await act(async () => gate.resolve());
  state.isSaving=true; state.status='saving'; await act(async () => renderer!.update(<RecordingControls {...props()} />));
  expect(control('Start recording').props.disabled).toBe(true);
});
test('failed parent finalization releases busy controls and exposes the failure', async () => {
  stopped=async () => { throw new Error('Durable save unavailable'); }; await show();
  await act(async () => control('Stop recording').props.onClick());
  expect(JSON.stringify(renderer!.toJSON())).toContain('Durable save unavailable');
  expect(JSON.stringify(renderer!.toJSON())).not.toContain('Processing recording...');
  expect(control('Stop recording').props.disabled).toBe(false);
});
test('listener registrations that finish after unmount are all released', async () => {
  const registrations: Array<(unlisten: () => void) => void> = [];
  let released=0;
  registerListen=() => new Promise(resolve => registrations.push(resolve));
  await show();
  await act(async () => renderer!.unmount()); renderer=undefined;
  for (let i=0;i<3;i++) await act(async () => registrations[i](() => { released++; }));
  expect(released).toBe(3);
});

test('a pending native action remains guarded across control remounts', async () => {
  const gate=deferred();native=command=>command==='pause_recording'?gate.promise:Promise.resolve();await show();
  await act(async()=>{void control('Pause recording').props.onClick();});
  expect(calls.filter(c=>c==='pause_recording')).toHaveLength(1);
  await act(async()=>renderer!.unmount());renderer=undefined;await show();
  await act(async()=>{void control('Pause recording').props.onClick();});
  expect(calls.filter(c=>c==='pause_recording')).toHaveLength(0);
  await act(async()=>gate.resolve());
  await act(async()=>{void control('Pause recording').props.onClick();});
  expect(calls.filter(c=>c==='pause_recording')).toHaveLength(1);
});

test('shared control guard survives remount until the frontend save acknowledges Stop',async()=>{
  const gate=deferred();stopped=()=>gate.promise;let starts=0;start=async()=>{starts++;};await show();
  await act(async()=>{void control('Stop recording').props.onClick();});
  await act(async()=>renderer!.unmount());renderer=undefined;state.isRecording=false;await show();
  expect(control('Start recording').props.disabled).toBe(true);
  await act(async()=>{void control('Start recording').props.onClick();});expect(starts).toBe(0);
  await act(async()=>gate.resolve());expect(control('Start recording').props.disabled).toBe(false);
  await act(async()=>{void control('Start recording').props.onClick();});expect(starts).toBe(1);
});
