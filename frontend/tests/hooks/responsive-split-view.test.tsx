import { afterAll, afterEach, expect, mock, test } from 'bun:test';
import { forwardRef, useEffect, useState } from 'react';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const tabsModule = { ...await import('../../src/components/ui/tabs') };
const originalObserver = globalThis.ResizeObserver;
let resize: ResizeObserverCallback;
let observed: unknown;
let disconnected = false;
const container = { getBoundingClientRect: () => ({ width: 400, left: 0 }) };
globalThis.ResizeObserver = class {
  constructor(callback: ResizeObserverCallback) { resize = callback; }
  observe(node: unknown) { observed = node; }
  disconnect() { disconnected = true; }
} as unknown as typeof ResizeObserver;
const Wrapper = forwardRef<HTMLDivElement, any>((props, ref) => <div {...props} ref={ref} />);
mock.module('../../src/components/ui/tabs', () => ({ Tabs: Wrapper, TabsContent: Wrapper, TabsList: Wrapper, TabsTrigger: Wrapper }));
const { MeetingDetailsSplitView } = await import('../../src/components/MeetingDetails/MeetingDetailsSplitView');
let renderer: ReactTestRenderer | undefined;
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; });
afterAll(() => { globalThis.ResizeObserver = originalObserver; mock.module('../../src/components/ui/tabs', () => tabsModule); });

function Pane({ name, mounts }: { name: string; mounts: string[] }) {
  const [text, setText] = useState('');
  useEffect(() => { mounts.push(name); }, [name, mounts]);
  return <input aria-label={name} value={text} onChange={event => setText(event.target.value)} />;
}
async function width(value: number) {
  await act(async () => resize([{ contentRect: { width: value } } as ResizeObserverEntry], {} as ResizeObserver));
}

test('the nested split view observes its container and preserves mounted content across width changes', async () => {
  const mounts: string[] = [];
  const props = { activeTab: 'summary' as const, onTabChange: () => {}, transcript: <Pane name="transcript" mounts={mounts} />, summary: <Pane name="summary" mounts={mounts} /> };
  await act(async () => { renderer = create(<MeetingDetailsSplitView {...props} />, { createNodeMock: () => container }); });
  expect(observed).toBe(container);
  await act(async () => renderer!.root.findByProps({ 'aria-label': 'summary', value: '' }).props.onChange({ target: { value: 'Pending enhanced notes — 日本語' } }));
  for (const availableWidth of [400, 1100, 700, 960, 480, 900]) {
    await width(availableWidth);
    const region = renderer!.root.findAll(node => node.type === 'div' && node.props['aria-label'] === 'Summary');
    expect(region.length).toBe(availableWidth >= 900 ? 1 : 0);
    expect(renderer!.root.findAllByType('input')[1].props.value).toBe('Pending enhanced notes — 日本語');
    expect(renderer!.root.findAll(node => node.type === 'div' && node.props.forceMount)).toHaveLength(2);
  }
  expect(mounts).toEqual(['transcript', 'summary']);
  await act(async () => renderer!.unmount()); renderer = undefined;
  expect(disconnected).toBe(true);
});

test('pane ratio and selected tab survive returning from a narrow container', async () => {
  await act(async () => { renderer = create(<MeetingDetailsSplitView activeTab="summary" onTabChange={() => {}} transcript={<p>Transcript</p>} summary={<p>Summary</p>} />, { createNodeMock: () => container }); });
  await width(1000);
  await act(async () => renderer!.root.findByProps({ role: 'separator' }).props.onKeyDown({ key: 'End', preventDefault() {} }));
  expect(renderer!.root.findByProps({ role: 'separator' }).props['aria-valuenow']).toBe(50);
  await width(400); await width(1000);
  expect(renderer!.root.findByProps({ role: 'separator' }).props['aria-valuenow']).toBe(50);
  expect(renderer!.root.findAllByProps({ value: 'summary' })[0].props.value).toBe('summary');
});
