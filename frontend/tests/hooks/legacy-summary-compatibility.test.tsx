import { afterAll, afterEach, describe, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import { flushAllLegacySummaries, forgetMeetingLegacySummaries } from '../../src/lib/legacy-summary-flush';

const originalCore = { ...await import('@tauri-apps/api/core') };
const originalSidebar = { ...await import('../../src/components/Sidebar/SidebarProvider') };
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => originalCore);
  mock.module('../../src/components/Sidebar/SidebarProvider', () => originalSidebar);
});
const calls: Record<string, any>[] = [];
mock.module('@tauri-apps/api/core', () => ({ ...originalCore, invoke: async (command: string, input: Record<string, any>) => {
  if (command !== 'api_save_meeting_summary') throw new Error(`Unexpected command ${command}`);
  calls.push(structuredClone(input)); return { revision: input.baseRevision + 1 };
} }));
mock.module('../../src/components/Sidebar/SidebarProvider', () => ({ ...originalSidebar,
  useSidebar: () => ({ setCurrentMeeting() {}, setMeetings() {}, meetings: [] }),
}));
const { useMeetingData } = await import('../../src/hooks/meeting-details/useMeetingData');
let renderer: ReactTestRenderer | undefined;
let state: ReturnType<typeof useMeetingData>;
let meetingId = '';
afterEach(async () => {
  if (renderer) await act(async () => renderer!.unmount());
  renderer = undefined;
  forgetMeetingLegacySummaries(meetingId);
  calls.length = 0;
});
async function show(id: string) {
  meetingId = id;
  const meeting = { id, title: 'Fixture', transcripts: [] };
  const summaryData = { _enhancement_version_id: 'legacy-fixture', _enhancement_revision: 3,
    Agenda: { title: 'Agenda', blocks: [{ id: 'Agenda-1', content: 'Original' }] }, extension: { preserve: true } } as any;
  function Probe() { state = useMeetingData({ meeting, summaryData }); return null; }
  await act(async () => { renderer = create(<Probe />); });
}
const edit = { Agenda: { title: 'Agenda', blocks: [{ id: 'Agenda-1', type: 'text', content: 'Tentative edit' }] } } as any;
describe('legacy editor integration', () => {
  test('editing metadata-free sections saves the captured displayed version and preserves extensions', async () => {
    await show('legacy-hook-save-fixture');
    await act(async () => { state.handleSummaryChange(edit); });
    expect(state.isSummaryDirty).toBe(true);
    await act(async () => { await state.handleSaveSummary(state.aiSummary!); });
    expect(calls[0].versionId).toBe('legacy-fixture');
    expect(calls[0].baseRevision).toBe(3);
    expect(calls[0].summary.extension).toEqual({ preserve: true });
    expect(calls[0].summary.Agenda.blocks[0].content).toBe('Tentative edit');
    expect(state.isSummaryDirty).toBe(false);
  });
  test('an unmounted dirty section editor retains its captured draft for lifecycle flush', async () => {
    await show('legacy-hook-detached-fixture');
    await act(async () => { state.handleSummaryChange(edit); renderer!.unmount(); });
    renderer = undefined;
    await flushAllLegacySummaries();
    expect(calls).toHaveLength(1);
    expect(calls[0].meetingId).toBe('legacy-hook-detached-fixture');
    expect(calls[0].summary.Agenda.blocks[0].content).toBe('Tentative edit');
    await flushAllLegacySummaries();
    expect(calls).toHaveLength(1);
  });
});
