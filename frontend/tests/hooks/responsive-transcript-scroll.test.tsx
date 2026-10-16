import { afterAll, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { TranscriptSegmentData } from '../../src/types';

const virtualModule = { ...await import('@tanstack/react-virtual') };
const tooltipModule = { ...await import('../../src/components/ui/tooltip') };
const { Virtualizer } = virtualModule;
let instance: InstanceType<typeof Virtualizer>;
const corrections: number[] = [];
mock.module('@tanstack/react-virtual', () => ({
  ...virtualModule,
  useVirtualizer: (options: any) => {
    instance ??= new Virtualizer({
      ...options,
      getScrollElement: () => null,
      observeElementRect: () => () => {},
      observeElementOffset: () => () => {},
      initialRect: { width: 500, height: 200 },
      initialOffset: 240,
      estimateSize: () => 1000,
      scrollToFn: (offset: number, { adjustments = 0 }: { adjustments?: number }) => corrections.push(Math.max(0, offset + adjustments)),
    });
    return instance;
  },
}));
const Container = ({ children }: any) => <>{children}</>;
mock.module('../../src/components/ui/tooltip', () => ({
  ...tooltipModule, TooltipProvider: Container, Tooltip: Container, TooltipContent: Container, TooltipTrigger: Container,
}));
const { VirtualizedTranscriptView } = await import('../../src/components/VirtualizedTranscriptView');
const { TooltipProvider } = await import('../../src/components/ui/tooltip');
afterAll(() => {
  mock.module('@tanstack/react-virtual', () => virtualModule);
  mock.module('../../src/components/ui/tooltip', () => tooltipModule);
});

test('rewrapping a partly visible passage keeps the reading offset; rows above it retain anchoring', async () => {
  const segments: TranscriptSegmentData[] = Array.from({ length: 12 }, (_, i) => ({ id: `passage-${i}`, text: 'Long transcript passage — 日本語', timestamp: 0 }));
  let renderer: ReactTestRenderer;
  await act(async () => { renderer = create(<TooltipProvider><VirtualizedTranscriptView segments={segments} disableAutoScroll /></TooltipProvider>); });
  await act(async () => {
    instance.getTotalSize();
    instance.scrollOffset = 240;
    // Simulate a long passage shrinking as the transcript pane gets wider.
    // Default virtualizer anchoring subtracts 600 and jumps this offset to 0.
    instance.resizeItem(0, 400);
  });
  expect(corrections).toEqual([]);
  await act(async () => {
    instance.getTotalSize();
    instance.scrollOffset = 1400;
    instance.resizeItem(0, 600);
  });
  expect(corrections).toEqual([1600]);
  await act(async () => renderer!.unmount());
});
