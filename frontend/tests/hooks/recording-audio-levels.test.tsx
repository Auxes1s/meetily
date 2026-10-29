import { afterAll, afterEach, beforeEach, describe, expect, mock, spyOn, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { RecordingAudioLevel } from '../../src/hooks/useRecordingAudioLevels';

const originalEvents = { ...await import('@tauri-apps/api/event') };
const originalCore = { ...await import('@tauri-apps/api/core') };
let updateMonitoring: (enabled: boolean) => Promise<void>;
const invoke = mock((_command: string, args: { enabled: boolean }) => updateMonitoring(args.enabled));
mock.module('@tauri-apps/api/core', () => ({ ...originalCore, invoke }));
type Handler = (event: { payload: RecordingAudioLevel }) => void;
const subscriptions: { handler: Handler; remove: ReturnType<typeof mock> }[] = [];
let registration: (remove: () => void) => Promise<() => void>;
const listen = mock((_name: string, handler: Handler) => {
  const remove = mock(() => {});
  subscriptions.push({ handler, remove });
  return registration(remove);
});
mock.module('@tauri-apps/api/event', () => ({ ...originalEvents, listen }));
const { useRecordingAudioLevels } = await import('../../src/hooks/useRecordingAudioLevels');
afterAll(() => {
  mock.module('@tauri-apps/api/event', () => originalEvents);
  mock.module('@tauri-apps/api/core', () => originalCore);
});

const realSetTimeout = globalThis.setTimeout;
const realClearTimeout = globalThis.clearTimeout;
const timers = new Map<number, { at: number; callback: () => void }>();
let now = 0;
let nextTimer = 0;
let renderer: ReactTestRenderer | undefined;
let bars: string[];
let renders = 0;
const originalDocument = globalThis.document;
let documentStub: EventTarget & { visibilityState: string };
const silent = ['4px', '4px', '4px'];

function View({ enabled }: { enabled: boolean }) {
  renders += 1;
  bars = useRecordingAudioLevels(enabled);
  return <output>{bars.join(',')}</output>;
}

beforeEach(() => {
  subscriptions.length = 0;
  listen.mockClear();
  invoke.mockClear();
  updateMonitoring = async () => {};
  renders = 0;
  documentStub = Object.assign(new EventTarget(), { visibilityState: 'visible' });
  globalThis.document = documentStub as unknown as Document;
  registration = async remove => remove;
  now = 0;
  timers.clear();
  globalThis.setTimeout = ((callback: () => void, delay: number) => {
    const id = ++nextTimer;
    timers.set(id, { at: now + delay, callback });
    return id;
  }) as unknown as typeof setTimeout;
  globalThis.clearTimeout = ((id: number) => timers.delete(id)) as unknown as typeof clearTimeout;
});

afterEach(async () => {
  if (renderer) await act(async () => renderer!.unmount());
  renderer = undefined;
  globalThis.setTimeout = realSetTimeout;
  globalThis.clearTimeout = realClearTimeout;
  globalThis.document = originalDocument;
});

async function show(enabled = true) {
  await act(async () => {
    if (renderer) renderer.update(<View enabled={enabled} />);
    else renderer = create(<View enabled={enabled} />);
  });
}

function levels(mic = 0, system = 0): RecordingAudioLevel {
  return { microphone: mic, system };
}

async function emit(payload: RecordingAudioLevel, index = subscriptions.length - 1) {
  await act(async () => subscriptions[index].handler({ payload }));
}

async function advance(ms: number) {
  now += ms;
  await act(async () => {
    for (const [id, timer] of timers) {
      if (timer.at <= now) {
        timers.delete(id);
        timer.callback();
      }
    }
  });
}

describe('recording audio bars', () => {
  test('starts flat, responds to either source, and follows increasing volume', async () => {
    await show();
    expect(bars).toEqual(silent);
    expect(listen.mock.calls[0][0]).toBe('recording-audio-level');
    await emit(levels(0.01));
    const quiet = bars.map(parseFloat);
    expect(quiet.every(height => height > 4)).toBe(true);
    await emit(levels(0.1));
    expect(bars.map(parseFloat).every((height, i) => height > quiet[i])).toBe(true);
    const microphoneBars = [...bars];
    await emit(levels(0, 0.1));
    expect(bars).toEqual(microphoneBars);
    await emit(levels(0.01, 0.1));
    expect(bars).toEqual(microphoneBars);
    await emit(levels());
    expect(bars).toEqual(silent);
  });

  test('uses a decorative profile and bounds invalid or overloaded values', async () => {
    await show();
    await emit(levels(0.01));
    expect(parseFloat(bars[1])).toBeGreaterThan(parseFloat(bars[0]));
    expect(bars[0]).toBe(bars[2]);
    await emit(levels(4, NaN));
    expect(bars).toEqual(['24px', '30px', '24px']);
    await emit(levels(NaN, -1));
    expect(bars).toEqual(silent);
    await emit(levels(Infinity, -Infinity));
    expect(bars).toEqual(silent);
    await emit(levels(0.0001));
    expect(bars).toEqual(silent);
  });

  test('resets after 500 ms without events and extends the deadline on fresh audio', async () => {
    await show();
    await emit(levels(0.1));
    await advance(400);
    expect(bars).not.toEqual(silent);
    await emit(levels(0.2));
    await advance(499);
    expect(bars).not.toEqual(silent);
    expect(timers.size).toBe(1);
    await advance(1);
    expect(bars).toEqual(silent);
  });

  test('pause or stop clears levels, ignores old events, and resumes on a fresh listener', async () => {
    await show(false);
    expect(listen).not.toHaveBeenCalled();
    await show(true);
    await emit(levels(0.5));
    await show(false);
    expect(bars).toEqual(silent);
    expect(subscriptions[0].remove).toHaveBeenCalledTimes(1);
    expect(timers.size).toBe(0);
    await emit(levels(1), 0);
    expect(bars).toEqual(silent);
    await show(true);
    expect(bars).toEqual(silent);
    await emit(levels(1), 0);
    expect(bars).toEqual(silent);
    await emit(levels(0, 0.5), 1);
    expect(bars).not.toEqual(silent);
    await show(false);
    expect(bars).toEqual(silent);
    expect(subscriptions[1].remove).toHaveBeenCalledTimes(1);
  });

  test('unmount removes listener and timeout', async () => {
    await show();
    await emit(levels(0.1));
    await act(async () => renderer!.unmount());
    renderer = undefined;
    expect(subscriptions[0].remove).toHaveBeenCalledTimes(1);
    expect(timers.size).toBe(0);
  });

  test('cleans up registration that resolves after unmount', async () => {
    let finish!: (remove: () => void) => void;
    registration = () => new Promise(resolve => { finish = resolve; });
    await show();
    await act(async () => renderer!.unmount());
    renderer = undefined;
    const { remove } = subscriptions[0];
    await act(async () => finish(remove));
    expect(remove).toHaveBeenCalledTimes(1);
    await emit(levels(1));
    expect(timers.size).toBe(0);
  });

  test('late registration after pause cannot replace the resumed subscription', async () => {
    let finish!: (remove: () => void) => void;
    registration = () => new Promise(resolve => { finish = resolve; });
    await show();
    await show(false);
    registration = async remove => remove;
    await show(true);
    await act(async () => finish(subscriptions[0].remove));
    expect(subscriptions[0].remove).toHaveBeenCalledTimes(1);
    expect(subscriptions[1].remove).not.toHaveBeenCalled();
    await emit(levels(0, 1));
    expect(bars).toEqual(['24px', '30px', '24px']);
  });

  test('registration failure leaves the bars flat without an unhandled rejection', async () => {
    registration = async () => { throw new Error('Event bridge unavailable'); };
    const error = spyOn(console, 'error').mockImplementation(() => {});
    try {
      await show();
      expect(bars).toEqual(silent);
      expect(error).toHaveBeenCalledTimes(1);
      expect(timers.size).toBe(0);
      expect(invoke.mock.calls.map(call => call[1].enabled)).toEqual([false]);
    } finally {
      error.mockRestore();
    }
  });

  test('hidden window disables native events and restarts flat when visible', async () => {
    await show();
    expect(invoke.mock.calls).toEqual([
      ['set_recording_level_monitoring', { enabled: false }],
      ['set_recording_level_monitoring', { enabled: true }],
    ]);
    await emit(levels(0.5));
    await act(async () => {
      documentStub.visibilityState = 'hidden';
      documentStub.dispatchEvent(new Event('visibilitychange'));
    });
    expect(bars).toEqual(silent);
    expect(timers.size).toBe(0);
    expect(subscriptions[0].remove).toHaveBeenCalledTimes(1);
    expect(invoke.mock.calls.at(-1)?.[1]).toEqual({ enabled: false });
    await emit(levels(1), 0);
    expect(bars).toEqual(silent);
    await act(async () => {
      documentStub.visibilityState = 'visible';
      documentStub.dispatchEvent(new Event('visibilitychange'));
    });
    expect(bars).toEqual(silent);
    expect(invoke.mock.calls.at(-1)?.[1]).toEqual({ enabled: true });
    await emit(levels(0, 0.5));
    expect(bars).not.toEqual(silent);
  });

  test('does not enable monitoring when mounted hidden or after late registration', async () => {
    documentStub.visibilityState = 'hidden';
    await show();
    expect(listen).not.toHaveBeenCalled();
    expect(invoke.mock.calls.map(call => call[1].enabled)).toEqual([false]);
    let finish!: (remove: () => void) => void;
    registration = () => new Promise(resolve => { finish = resolve; });
    await act(async () => {
      documentStub.visibilityState = 'visible';
      documentStub.dispatchEvent(new Event('visibilitychange'));
    });
    await show(false);
    await act(async () => finish(subscriptions[0].remove));
    expect(invoke.mock.calls.map(call => call[1].enabled)).toEqual([false]);
  });

  test('serializes native enables and disables across a rapid pause/resume', async () => {
    let finishEnable!: () => void;
    let first = true;
    updateMonitoring = enabled => {
      if (!enabled || !first) return Promise.resolve();
      first = false;
      return new Promise(resolve => { finishEnable = resolve; });
    };
    await show();
    await show(false);
    await show(true);
    expect(invoke).toHaveBeenCalledTimes(2);
    await act(async () => finishEnable());
    expect(invoke.mock.calls.map(call => call[1].enabled)).toEqual([false, true, false, true]);
  });

  test('steady levels do not keep rerendering the recording controls', async () => {
    await show();
    await emit(levels(0.5));
    // React can render once to confirm a newly repeated state before bailing out.
    await emit(levels(0.5));
    const settledRenders = renders;
    await emit(levels(0.5));
    await emit(levels(0.5));
    expect(renders).toBe(settledRenders);
    expect(timers.size).toBe(1);
  });
});
