import { afterAll, afterEach, describe, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { LibraryMeeting, LibraryPage } from '../../src/types/meeting-library';
import type { DeletionStatus } from '../../src/services/meetingDeletionService';

const originals = {
  core: { ...await import('@tauri-apps/api/core') },
  service: { ...await import('../../src/services/meetingLibraryService') },
  deletion: { ...await import('../../src/services/meetingDeletionService') },
  library: { ...await import('../../src/hooks/useMeetingLibrary') },
  recording: { ...await import('../../src/contexts/RecordingStateContext') },
  dialog: { ...await import('../../src/components/ui/dialog') },
};
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
const meeting = (id: string): LibraryMeeting => ({ id, title: id, created_at: '2026-10-15T00:00:00Z', updated_at: '2026-10-15T00:00:00Z', started_at: null, ended_at: null, timezone: null, status: 'completed', date_is_creation_fallback: true, index_updating: false, participants: [], tags: [], hits: [] });
const deletion = (id: string, state: 'pending' | 'complete' = 'pending'): DeletionStatus => ({ job_id: `job-${id}`, meeting_id: id, recording_session_id: null, state, native_complete: state === 'complete', caches_complete: state === 'complete', last_error: null });
const page: LibraryPage = { meetings: [meeting('a'), meeting('b')], total_count: 2, next_cursor: null, index_updating: false, pending_index_count: 0 };
let pending: DeletionStatus[] = [];
let pendingRead: ReturnType<typeof deferred<DeletionStatus[]>> | null = null;
const refresh = mock(() => {});
const metadataWrites: Array<{ meetingId: string; people: string[]; labels: string[]; reply: ReturnType<typeof deferred<void>> }> = [];
const deletes: Array<{ meetingId: string; reply: ReturnType<typeof deferred<DeletionStatus>> }> = [];
const retries: Array<{ jobId: string; reply: ReturnType<typeof deferred<DeletionStatus>> }> = [];
const templates: Array<{ command: string; input: any; reply: ReturnType<typeof deferred<any>> }> = [];
mock.module('@tauri-apps/api/core', () => ({ ...originals.core, invoke: (command: string, input: unknown) => {
  const reply = deferred<any>(); templates.push({ command, input, reply }); return reply.promise;
} }));
mock.module('../../src/services/meetingLibraryService', () => ({ meetingLibraryService: { ...originals.service.meetingLibraryService, updateMetadata: (meetingId: string, people: string[], labels: string[]) => {
  const reply = deferred<void>(); metadataWrites.push({ meetingId, people, labels, reply }); return reply.promise;
} } }));
mock.module('../../src/services/meetingDeletionService', () => ({ ...originals.deletion,
  deleteMeetingManaged: (meetingId: string) => { const reply = deferred<DeletionStatus>(); deletes.push({ meetingId, reply }); return reply.promise; },
  meetingDeletionService: { listPending: () => pendingRead?.promise ?? Promise.resolve(pending), retry: (jobId: string) => { const reply = deferred<DeletionStatus>(); retries.push({ jobId, reply }); return reply.promise; } },
}));
mock.module('../../src/hooks/useMeetingLibrary', () => ({ useMeetingLibrary: () => ({ page, loading: false, error: null, refresh }) }));
mock.module('../../src/contexts/RecordingStateContext', () => ({ ...originals.recording, useRecordingState: () => ({ isRecording: false }) }));
const container = ({ children }: any) => <div>{children}</div>;
mock.module('../../src/components/ui/dialog', () => ({
  Dialog: ({ open, children, onOpenChange }: any) => open ? <div data-dialog>{children}</div> : null,
  DialogContent: container, DialogHeader: container, DialogTitle: container, DialogDescription: container, DialogFooter: container,
}));
const { MeetingLibrary } = await import('../../src/components/MeetingLibrary/MeetingLibrary');
const { MeetingMetadataEditor } = await import('../../src/components/MeetingLibrary/MeetingMetadataEditor');
const { CustomEnhancementTemplate } = await import('../../src/components/MeetingWorkspace/CustomEnhancementTemplate');
let renderer: ReactTestRenderer | undefined;
const previousWindow = globalThis.window;
const browser = new EventTarget() as EventTarget & { sessionStorage: { getItem: () => null } };
browser.sessionStorage = { getItem: () => null };
(globalThis as any).window = browser;
afterEach(async () => {
  await act(async () => renderer?.unmount()); renderer = undefined;
  metadataWrites.length = 0; deletes.length = 0; retries.length = 0; templates.length = 0; pending = []; pendingRead = null; refresh.mockClear();
});
afterAll(() => {
  (globalThis as any).window = previousWindow;
  mock.module('@tauri-apps/api/core', () => originals.core);
  mock.module('../../src/services/meetingLibraryService', () => originals.service);
  mock.module('../../src/services/meetingDeletionService', () => originals.deletion);
  mock.module('../../src/hooks/useMeetingLibrary', () => originals.library);
  mock.module('../../src/contexts/RecordingStateContext', () => originals.recording);
  mock.module('../../src/components/ui/dialog', () => originals.dialog);
});
const button = (label: string) => renderer!.root.findAllByType('button').find(node => node.children.filter(child => typeof child === 'string').join('') === label)!;
const buttons = (label: string) => renderer!.root.findAllByType('button').filter(node => node.children.filter(child => typeof child === 'string').join('') === label);
const submit = () => renderer!.root.findByType('form').props.onSubmit({ preventDefault() {} });
const draft = () => renderer!.root.findAllByType('textarea').map(node => node.props.value);
const mountLibrary = async () => { await act(async () => { renderer = create(<MeetingLibrary />); }); };
const mountMetadata = async (onSaved = mock(() => {}), onCancel = mock(() => {})) => {
  await act(async () => { renderer = create(<MeetingMetadataEditor meetingId="a" participants={['Alice']} tags={['draft']} onSaved={onSaved} onCancel={onCancel} />); });
  return { onSaved, onCancel };
};
const mountTemplate = async (onSaved = mock(async (_id: string) => {})) => {
  await act(async () => { renderer = create(<CustomEnhancementTemplate selectedId="user_a" onSaved={onSaved} />); });
  return onSaved;
};

describe('rapid library and metadata actions', () => {
  test('same-tick metadata submissions send one captured request and failed drafts can be retried', async () => {
    const { onSaved } = await mountMetadata();
    await act(async () => { const handler = renderer!.root.findByType('form').props.onSubmit; for (let i = 0; i < 8; i++) void handler({ preventDefault() {} }); });
    expect(metadataWrites).toHaveLength(1);
    expect(metadataWrites[0]).toMatchObject({ meetingId: 'a', people: ['Alice'], labels: ['draft'] });
    await act(async () => metadataWrites[0].reply.reject(new Error('Save failed')));
    expect(draft()).toEqual(['Alice', 'draft']);
    expect(button('Save metadata').props.disabled).toBe(false);
    expect(JSON.stringify(renderer!.toJSON())).toContain('Save failed');
    await act(async () => { void submit(); });
    await act(async () => metadataWrites[1].reply.resolve());
    expect(onSaved).toHaveBeenCalledTimes(1);
  });
  test('same-tick edits and cancel cannot discard an in-flight metadata save', async () => {
    const { onCancel } = await mountMetadata();
    const change = renderer!.root.findAllByType('textarea')[0].props.onChange;
    const cancel = button('Cancel').props.onClick;
    await act(async () => { void submit(); change({ target: { value: 'Unsaved overwrite' } }); cancel(); });
    expect(draft()).toEqual(['Alice', 'draft']);
    expect(onCancel).not.toHaveBeenCalled();
    expect(renderer!.root.findAllByType('textarea').every(node => node.props.disabled)).toBe(true);
  });
  test('metadata submit captures the latest same-tick input', async () => {
    await mountMetadata();
    const change = renderer!.root.findAllByType('textarea')[0].props.onChange;
    const save = renderer!.root.findByType('form').props.onSubmit;
    await act(async () => { change({ target: { value: 'Alice\n日本語' } }); void save({ preventDefault() {} }); });
    expect(metadataWrites[0].people).toEqual(['Alice', '日本語']);
  });
  test('unmounted metadata saves cannot close a later editor', async () => {
    const { onSaved } = await mountMetadata();
    await act(async () => { void submit(); });
    await act(async () => renderer!.unmount()); renderer = undefined;
    await act(async () => metadataWrites[0].reply.resolve());
    expect(onSaved).not.toHaveBeenCalled();
  });
  test('metadata editor switches stay blocked until its save fails or completes', async () => {
    await mountLibrary();
    await act(async () => buttons('Metadata')[0].props.onClick());
    const switchEditor = buttons('Metadata')[1].props.onClick;
    await act(async () => { void submit(); switchEditor(); });
    expect(renderer!.root.findByType(MeetingMetadataEditor).props.meetingId).toBe('a');
    expect(buttons('Metadata').every(node => node.props.disabled)).toBe(true);
    await act(async () => metadataWrites[0].reply.reject(new Error('Still editable')));
    expect(buttons('Metadata').every(node => !node.props.disabled)).toBe(true);
    expect(renderer!.root.findByType('form')).toBeTruthy();
  });
  test('rapid delete confirmation stays on one target and permits retry after failure', async () => {
    await mountLibrary();
    await act(async () => renderer!.root.findByProps({ 'aria-label': 'Delete a' }).props.onClick());
    const cancel = button('Cancel').props.onClick;
    await act(async () => { const confirm = button('Delete meeting').props.onClick; for (let i = 0; i < 8; i++) confirm(); cancel(); });
    expect(deletes).toHaveLength(1);
    expect(deletes[0].meetingId).toBe('a');
    expect(button('Deleting…')).toBeTruthy();
    await act(async () => deletes[0].reply.reject(new Error('Try deletion again')));
    expect(button('Delete meeting').props.disabled).toBe(false);
    await act(async () => button('Delete meeting').props.onClick());
    await act(async () => deletes[1].reply.resolve(deletion('a', 'complete')));
    expect(renderer!.root.findAllByProps({ 'data-dialog': true })).toHaveLength(0);
  });
  test('cleanup retries deduplicate each job while independent jobs remain available', async () => {
    pending = [deletion('a'), deletion('b')]; await mountLibrary();
    await act(async () => { const retry = buttons('Retry cleanup')[0].props.onClick; for (let i = 0; i < 8; i++) void retry(); });
    expect(retries).toHaveLength(1);
    expect(button('Retry cleanup').props.disabled).toBe(false);
    await act(async () => { void button('Retry cleanup').props.onClick(); });
    expect(retries).toHaveLength(2);
    await act(async () => retries[0].reply.reject(new Error('Cache unavailable')));
    expect(button('Retry cleanup').props.disabled).toBe(false);
    await act(async () => retries[1].reply.resolve(deletion('b', 'complete')));
    expect(button('Retry cleanup')).toBeTruthy();
  });
  test('cancelled or replaced delete confirmation handlers cannot delete another target', async () => {
    await mountLibrary();
    await act(async () => renderer!.root.findByProps({ 'aria-label': 'Delete a' }).props.onClick());
    const oldConfirm = button('Delete meeting').props.onClick;
    await act(async () => { button('Cancel').props.onClick(); oldConfirm(); });
    expect(deletes).toHaveLength(0);
    await act(async () => renderer!.root.findByProps({ 'aria-label': 'Delete b' }).props.onClick());
    await act(async () => oldConfirm());
    expect(deletes).toHaveLength(0);
  });
  test('a pending-cleanup read started before retry completion cannot resurrect that job', async () => {
    pending = [deletion('a')]; await mountLibrary();
    pendingRead = deferred<DeletionStatus[]>();
    await act(async () => { browser.dispatchEvent(new Event('meeting-deletion-started')); });
    await act(async () => { void button('Retry cleanup').props.onClick(); });
    await act(async () => retries[0].reply.resolve(deletion('a', 'complete')));
    await act(async () => pendingRead!.resolve([deletion('a')]));
    expect(buttons('Retry cleanup')).toHaveLength(0);
  });
});

describe('rapid custom-template actions', () => {
  test('same-tick save/load abuse issues one save and failure preserves the draft for retry', async () => {
    const onSaved = await mountTemplate();
    const save = button('Save and use').props.onClick;
    const load = button('Load saved template').props.onClick;
    await act(async () => { for (let i = 0; i < 8; i++) save(); load(); });
    expect(templates).toHaveLength(1);
    expect(templates[0].command).toBe('api_save_custom_template');
    expect(templates[0].input.templateId).toBe('user_a');
    await act(async () => templates[0].reply.reject(new Error('Template save failed')));
    expect(button('Save and use').props.disabled).toBe(false);
    expect(renderer!.root.findAllByType('input')[0].props.value).toBe('user_a');
    await act(async () => button('Save and use').props.onClick());
    await act(async () => templates[1].reply.resolve(undefined));
    expect(onSaved).toHaveBeenCalledWith('user_a');
  });
  test('stale edit/section handlers cannot mutate the draft while loading', async () => {
    await mountTemplate();
    const edit = renderer!.root.findAllByType('input')[0].props.onChange;
    const add = button('Add section').props.onClick;
    await act(async () => { button('Load saved template').props.onClick(); edit({ target: { value: 'user_b' } }); add(); });
    expect(renderer!.root.findAllByType('input')[0].props.value).toBe('user_a');
    expect(renderer!.root.findAllByType('fieldset')).toHaveLength(1);
    expect(button('Add section').props.disabled).toBe(true);
    await act(async () => templates[0].reply.resolve({ name: 'Loaded A', description: 'Saved A', section_definitions: [{ title: 'Saved section', instruction: 'Keep context', format: 'list' }] }));
    expect(renderer!.root.findAllByType('input')[1].props.value).toBe('Loaded A');
    expect(button('Save and use').props.disabled).toBe(false);
  });
  test('section action bursts preserve boundaries and save the latest draft', async () => {
    await mountTemplate();
    const add = button('Add section').props.onClick;
    await act(async () => { for (let i = 0; i < 40; i++) add(); });
    expect(renderer!.root.findAllByType('fieldset')).toHaveLength(32);
    const remove = buttons('Remove section')[1].props.onClick;
    await act(async () => { for (let i = 0; i < 40; i++) remove(); });
    expect(renderer!.root.findAllByType('fieldset')).toHaveLength(31);
    const edit = renderer!.root.findAllByType('input')[0].props.onChange;
    const save = button('Save and use').props.onClick;
    await act(async () => { edit({ target: { value: 'user_latest' } }); save(); });
    expect(templates[0].input.templateId).toBe('user_latest');
    expect(JSON.parse(templates[0].input.templateJson).sections).toHaveLength(31);
  });
  test('a selected-template change makes an older load inert and releases its lock', async () => {
    const onSaved = await mountTemplate();
    await act(async () => button('Load saved template').props.onClick());
    await act(async () => renderer!.update(<CustomEnhancementTemplate selectedId="user_b" onSaved={onSaved} />));
    await act(async () => templates[0].reply.resolve({ name: 'Stale A', description: 'Old', section_definitions: [] }));
    expect(renderer!.root.findAllByType('input')[1].props.value).toBe('My meeting notes');
    expect(button('Save and use').props.disabled).toBe(false);
  });
  test('an unmounted save or a selection change cannot select its old template', async () => {
    const onSaved = await mountTemplate();
    await act(async () => button('Save and use').props.onClick());
    await act(async () => renderer!.update(<CustomEnhancementTemplate selectedId="user_b" onSaved={onSaved} />));
    await act(async () => templates[0].reply.resolve(undefined));
    expect(onSaved).not.toHaveBeenCalled();
    await act(async () => button('Save and use').props.onClick());
    await act(async () => renderer!.unmount()); renderer = undefined;
    await act(async () => templates[1].reply.resolve(undefined));
    expect(onSaved).not.toHaveBeenCalled();
  });
});
