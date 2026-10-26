import { afterAll, afterEach, describe, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { MeetingExportOptions, MeetingExportRequest, MeetingExportResult } from '../../src/types/meeting-export';

const originals = {
  service: { ...await import('../../src/services/meetingExportService') },
  dialog: { ...await import('../../src/components/ui/dialog') },
};
let approved: string | null = 'approval';
let status = 'completed';
let optionsReply: ((options: MeetingExportOptions) => void) | null = null;
let pendingOptions = false;
let optionCalls = 0;
let exportGate: Promise<void> | null = null;
let requests: MeetingExportRequest[] = [];
let result: MeetingExportResult = { status: 'saved' };
let failure: unknown = null;
mock.module('../../src/services/meetingExportService', () => ({
  meetingExportService: {
    options: (meetingId: string) => { optionCalls++; return pendingOptions ? new Promise<MeetingExportOptions>(resolve => { optionsReply = resolve; }) : Promise.resolve({ meeting_id: meetingId, title: meetingId, status, approved_version_id: approved }); },
    export: async (request: MeetingExportRequest) => { requests.push(request); await exportGate; if (failure) throw failure; return result; },
  },
}));
const container = ({ children }: any) => <div>{children}</div>;
mock.module('../../src/components/ui/dialog', () => ({
  Dialog: ({ open, children }: any) => open ? <div>{children}</div> : null,
  DialogContent: container, DialogHeader: container, DialogTitle: container, DialogDescription: container, DialogFooter: container,
}));
const { MeetingExportButton } = await import('../../src/components/MeetingWorkspace/MeetingExportButton');
let renderer: ReactTestRenderer | undefined;
afterEach(async () => {
  await act(async () => renderer?.unmount()); renderer = undefined;
  approved = 'approval'; status = 'completed'; pendingOptions = false; optionsReply = null; optionCalls = 0; exportGate = null; requests = []; result = { status: 'saved' }; failure = null;
});
afterAll(() => {
  mock.module('../../src/services/meetingExportService', () => originals.service);
  mock.module('../../src/components/ui/dialog', () => originals.dialog);
});
function button(label: string) { return renderer!.root.findAllByType('button').find(node => node.children.filter(child => typeof child === 'string').join('') === label)!; }
async function open(id = 'meeting-a') {
  await act(async () => { renderer = create(<MeetingExportButton meetingId={id} />); });
  await act(async () => { renderer!.root.findByProps({ 'aria-label': 'Export meeting' }).props.onClick(); });
}

describe('meeting export choices', () => {
  test('rapid opens share one pending options request; close and reopen supersede stale choices', async () => {
    pendingOptions = true; await open();
    const first = optionsReply!;
    await act(async () => { for (let i = 0; i < 30; i++) renderer!.root.findByProps({ 'aria-label': 'Export meeting' }).props.onClick(); });
    expect(optionCalls).toBe(1);
    await act(async () => button('Close').props.onClick());
    await act(async () => renderer!.root.findByProps({ 'aria-label': 'Export meeting' }).props.onClick());
    const second = optionsReply!;
    expect(optionCalls).toBe(2);
    await act(async () => first({ meeting_id: 'meeting-a', title: 'Obsolete choices', status: 'completed', approved_version_id: null }));
    expect(JSON.stringify(renderer!.toJSON())).not.toContain('Obsolete choices');
    await act(async () => second({ meeting_id: 'meeting-a', title: 'Current choices', status: 'completed', approved_version_id: 'approval' }));
    expect(JSON.stringify(renderer!.toJSON())).toContain('Current choices');
  });
  test('rapid saves invoke one export, reject closing while saving, and permit retry after failure', async () => {
    await open(); let finish!: () => void;
    exportGate = new Promise(resolve => { finish = resolve; });
    const save = button('Save export…').props.onClick;
    await act(async () => { for (let i = 0; i < 30; i++) save(); });
    expect(requests).toHaveLength(1);
    await act(async () => button('Close').props.onClick());
    expect(renderer!.root.findAllByProps({ 'aria-label': 'Export format' })).toHaveLength(1);
    failure = new Error('Save failed; original notes retained.');
    await act(async () => finish());
    expect(renderer!.root.findByProps({ role: 'alert' }).children).toContain('Save failed; original notes retained.');
    exportGate = null; failure = null;
    await act(async () => button('Save export…').props.onClick());
    expect(requests).toHaveLength(2);
    expect(JSON.stringify(renderer!.toJSON())).toContain('Export saved.');
  });
  test('defaults to approved notes with the full transcript off', async () => {
    await open();
    expect(renderer!.root.findByProps({ 'aria-label': 'Notes to include' }).props.value).toBe('approved');
    expect(renderer!.root.findByType('input').props.checked).toBe(false);
    await act(async () => button('Save export…').props.onClick());
    expect(requests[0]).toMatchObject({ document: 'approved', format: 'markdown', include_transcript: false, expected_approved_version_id: 'approval' });
    expect(JSON.stringify(renderer!.toJSON())).toContain('Export saved.');
  });
  test('falls back to originals and disables unavailable approval choices', async () => {
    approved = null; status = 'recording'; await open();
    const choices = renderer!.root.findByProps({ 'aria-label': 'Notes to include' });
    expect(choices.props.value).toBe('original');
    expect(choices.findAllByType('option').filter(node => node.props.disabled).length).toBe(2);
    expect(JSON.stringify(renderer!.toJSON())).toContain('labeled incomplete');
  });
  test('permits both documents with transcript, and JSON remains available after a Markdown failure', async () => {
    await open();
    await act(async () => {
      renderer!.root.findByProps({ 'aria-label': 'Notes to include' }).props.onChange({ target: { value: 'both' } });
      renderer!.root.findByType('input').props.onChange({ target: { checked: true } });
    });
    failure = { code: 'conversion', message: 'Export JSON to preserve all saved blocks.' };
    await act(async () => button('Save export…').props.onClick());
    expect(requests[0]).toMatchObject({ document: 'both', include_transcript: true });
    expect(renderer!.root.findByProps({ role: 'alert' }).children).toContain('Export JSON to preserve all saved blocks.');
    await act(async () => renderer!.root.findByProps({ 'aria-label': 'Export format' }).props.onChange({ target: { value: 'json' } }));
    expect(renderer!.root.findAllByProps({ 'aria-label': 'Notes to include' })).toHaveLength(0);
    failure = null; result = { status: 'cancelled' };
    await act(async () => button('Save export…').props.onClick());
    expect(requests.at(-1)!.format).toBe('json');
    const rendered = JSON.stringify(renderer!.toJSON());
    expect(rendered).toContain('Export cancelled.'); expect(rendered).not.toContain('Export saved.');
  });
  test('closing while choices load prevents a stale response from selecting a document', async () => {
    pendingOptions = true; await open();
    const reply = optionsReply!;
    await act(async () => button('Close').props.onClick());
    await act(async () => { reply({ meeting_id: 'meeting-a', title: 'Stale meeting', status: 'completed', approved_version_id: 'old' }); });
    expect(JSON.stringify(renderer!.toJSON())).not.toContain('Stale meeting');
  });
  test('a response for an earlier meeting cannot populate the current meeting dialog', async () => {
    pendingOptions = true; await open(); const reply = optionsReply!;
    await act(async () => renderer!.update(<MeetingExportButton meetingId="meeting-b" />));
    pendingOptions = false;
    await act(async () => renderer!.root.findByProps({ 'aria-label': 'Export meeting' }).props.onClick());
    await act(async () => { reply({ meeting_id: 'meeting-a', title: 'Old meeting', status: 'completed', approved_version_id: 'old' }); });
    await act(async () => button('Save export…').props.onClick());
    expect(requests[0].meeting_id).toBe('meeting-b');
    expect(JSON.stringify(renderer!.toJSON())).not.toContain('Old meeting');
  });
});
