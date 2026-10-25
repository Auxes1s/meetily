import { afterAll, expect, mock, test } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';
import { TooltipProvider } from '../../src/components/ui/tooltip';
import type { AudioSource, TranscriptSegmentData } from '../../src/types';

const originalVirtual = { ...await import('@tanstack/react-virtual') };
const originalState = { ...await import('../../src/contexts/RecordingStateContext') };
mock.module('../../src/contexts/RecordingStateContext', () => ({
  ...originalState,
  useRecordingState: () => ({ status: 'idle', isRecording: false, isPaused: false }),
  useRecordingControls: () => ({ status: 'idle', isRecording: false, isPaused: false }),
}));
// Supply visible rows for SSR; actual webview scrolling is a native smoke gate.
mock.module('@tanstack/react-virtual', () => ({
  ...originalVirtual,
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 60,
    getVirtualItems: () => Array.from({ length: count }, (_, index) => ({ index, start: index * 60 })),
    measureElement: () => {},
  }),
}));
const { VirtualizedTranscriptView } = await import('../../src/components/VirtualizedTranscriptView');
afterAll(() => {
  mock.module('@tanstack/react-virtual', () => originalVirtual);
  mock.module('../../src/contexts/RecordingStateContext', () => originalState);
});

test.each([4, 12])('both transcript rendering paths retain passage badges (%i rows)', count => {
  const sources: AudioSource[] = ['microphone', 'system', 'mixed', 'unknown'];
  const segments: TranscriptSegmentData[] = Array.from({ length: count }, (_, index) => ({
    id: `passage-${index}`, timestamp: index, text: `Fixture passage ${index}`, audio_source: sources[index % 4],
  }));
  const render = () => renderToStaticMarkup(<TooltipProvider>
    <VirtualizedTranscriptView segments={segments} isRecording enableStreaming />
  </TooltipProvider>);
  const html = render();
  for (const label of ['Microphone', 'System audio', 'Mixed audio', 'Unknown']) {
    expect(html.split(`aria-label="Audio source: ${label}"`).length - 1).toBe(count / 4);
  }
  // Reopening the panel uses the retained passages and identical provenance.
  expect(render()).toBe(html);
});
