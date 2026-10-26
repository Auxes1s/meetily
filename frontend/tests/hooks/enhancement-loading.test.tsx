import { afterAll, afterEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { EnhancementVersion, MeetingEnhancements } from '../../src/types/enhancement';
const originals = {
  core: { ...await import('@tauri-apps/api/core') }, navigation: { ...await import('next/navigation') },
  config: { ...await import('../../src/contexts/ConfigContext') }, recording: { ...await import('../../src/contexts/RecordingStateContext') },
  dynamic: { ...await import('next/dynamic') }, window: globalThis.window,
};
const { enhancementSessions } = await import('../../src/lib/enhancement-session');
const browser = Object.assign(new EventTarget(), { localStorage: { getItem: () => null }, confirm: () => true });
Object.defineProperty(globalThis, 'window', { configurable: true, value: browser });
const meta = (meeting: string, status = 'ready', updated = '2026-10-25T00:00:00Z'): EnhancementVersion => ({
  id: `v-${meeting}`, meeting_id: meeting, state: 'candidate', status, revision, document_json: null, source_snapshot_json: null,
  source_fingerprint: 'fixture', notes_revision: 1, template_id: 'standard_meeting', instructions: '', created_at: updated, updated_at: updated, error: null, source_stale: false,
});
let status = 'ready', updated = '2026-10-25T00:00:00Z', revision = 0;
const requests: Array<{ command: string; input: any }> = [];
const details: Array<{ meeting: string; resolve: (version: EnhancementVersion) => void }> = [];
mock.module('@tauri-apps/api/core', () => ({ ...originals.core, invoke: (command: string, input: any) => {
  requests.push({ command, input });
  if (command === 'api_get_meeting_enhancements') return Promise.resolve({ meeting_id: input.meetingId, approved_version_id: null,
    settings: { template_id: 'standard_meeting', instructions: '', revision: 0 }, versions: [meta(input.meetingId, status, updated)], next_cursor: null } satisfies MeetingEnhancements);
  if (command === 'api_get_enhancement_version') return new Promise(resolve => details.push({ meeting: input.meetingId, resolve }));
  if (command === 'api_list_templates') return Promise.resolve([]);
  if (command === 'api_get_enhancement_status') return Promise.resolve({ id: input.versionId, status });
  if (command === 'api_save_enhancement_version') return Promise.reject(new Error('Revision conflict'));
  return Promise.reject(new Error(`Unexpected command: ${command}`));
} }));
mock.module('next/navigation', () => ({ ...originals.navigation, useSearchParams: () => new URLSearchParams() }));
mock.module('../../src/contexts/ConfigContext', () => ({ ...originals.config, useConfig: () => ({ modelConfig: { provider: 'builtin-ai', model: 'fixture' }, isModelConfigLoading: false }) }));
mock.module('../../src/contexts/RecordingStateContext', () => ({ ...originals.recording, useRecordingControls: () => ({ isRecording: false }) }));
mock.module('next/dynamic', () => ({ default: () => ({ initialContent }: any) => <div data-version-editor>{JSON.stringify(initialContent)}</div> }));
const { EnhancementPanel } = await import('../../src/components/MeetingWorkspace/EnhancementPanel');
let renderer: ReactTestRenderer | undefined;
const body = (meeting: string, label: string, state = status) => ({ ...meta(meeting, state, updated), document_json: JSON.stringify({ summary_json: [{ id: 'block', type: 'paragraph', content: label, children: [] }], claims: [] }) });
afterEach(async () => {
  await act(async () => renderer?.unmount()); renderer = undefined;
  for (const entry of enhancementSessions.values()) entry.session.dispose(); enhancementSessions.clear();
  details.length = 0; requests.length = 0; status = 'ready'; updated = '2026-10-25T00:00:00Z'; revision = 0;
});
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => originals.core); mock.module('next/navigation', () => originals.navigation);
  mock.module('../../src/contexts/ConfigContext', () => originals.config); mock.module('../../src/contexts/RecordingStateContext', () => originals.recording);
  mock.module('next/dynamic', () => originals.dynamic); Object.defineProperty(globalThis, 'window', { configurable: true, value: originals.window });
});
test('metadata loading fetches only the selected body and rejects a previous meeting response', async () => {
  await act(async () => { renderer = create(<EnhancementPanel meetingId="a" />); });
  expect(requests.find(request => request.command === 'api_get_meeting_enhancements')!.input.metadataOnly).toBe(true);
  expect(details.map(request => request.meeting)).toEqual(['a']);
  await act(async () => renderer!.update(<EnhancementPanel meetingId="b" />));
  await act(async () => details[0].resolve(body('a', 'Stale A')));
  expect(renderer!.root.findAllByProps({ 'data-version-editor': true })).toHaveLength(0);
  await act(async () => details[1].resolve(body('b', 'Current B')));
  expect(renderer!.root.findByProps({ 'data-version-editor': true }).children.join('')).toContain('Current B');
});
test('completion with the same revision fetches the newly published document', async () => {
  status = 'generating';
  await act(async () => { renderer = create(<EnhancementPanel meetingId="a" />); });
  await act(async () => details[0].resolve(meta('a', 'generating', updated)));
  status = 'ready'; updated = '2026-10-25T00:01:00Z';
  await act(async () => { browser.dispatchEvent(new Event('meeting-data-changed')); });
  expect(details).toHaveLength(2);
  await act(async () => details[1].resolve(body('a', 'Completed document')));
  expect(renderer!.root.findByProps({ 'data-version-editor': true }).children.join('')).toContain('Completed document');
});
test('conflict reload fetches the authoritative body before recreating the editor session', async () => {
  await act(async () => { renderer = create(<EnhancementPanel meetingId="a" />); });
  await act(async () => details[0].resolve(body('a', 'Old document')));
  const old = enhancementSessions.get('v-a')!.session;
  await act(async () => { old.edit(old.getSnapshot().blocks); await old.flush().catch(() => {}); });
  const reload = renderer!.root.findAllByType('button').find(button => button.children.join('') === 'Reload saved version')!;
  await act(async () => reload.props.onClick());
  expect(enhancementSessions.get('v-a')!.session).toBe(old);
  revision = 2; updated = '2026-10-25T00:02:00Z';
  await act(async () => details[1].resolve(body('a', 'Latest saved document')));
  expect(renderer!.root.findByProps({ 'data-version-editor': true }).children.join('')).toContain('Latest saved document');
  expect(enhancementSessions.get('v-a')!.session.getSnapshot().revision).toBe(2);
});
