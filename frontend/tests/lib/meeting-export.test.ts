import { afterEach, describe, expect, test } from 'bun:test';
import { createMeetingExportService } from '../../src/services/meetingExportService';
import { flushMeetingDocuments } from '../../src/lib/workspace-save';
import { MeetingNotesSession, noteSessions, type SaveNotesInput } from '../../src/lib/meeting-notes-session';
import { enhancementSessions } from '../../src/lib/enhancement-session';
import { registerLegacySummaryFlusher } from '../../src/lib/legacy-summary-flush';
import type { MeetingExportRequest } from '../../src/types/meeting-export';

const request = (meetingId = 'export-target'): MeetingExportRequest => ({ meeting_id: meetingId, format: 'json', document: 'original', include_transcript: false, expected_approved_version_id: null });
const blocks = (text: string) => [{ id: 'block', type: 'paragraph', content: [{ type: 'text', text, styles: {} }], children: [] }] as any;
const callbacks: (() => void)[] = [];
function session(meetingId: string, save: (input: SaveNotesInput) => Promise<{ revision: number; updated_at: string }>) {
  return new MeetingNotesSession(meetingId, { blocks: [], links: [], title: 'Meeting', revision: 0, updatedAt: null }, save);
}
afterEach(() => {
  callbacks.splice(0).forEach(cleanup => cleanup());
  noteSessions.forEach(session => session.dispose()); noteSessions.clear();
  enhancementSessions.forEach(entry => entry.session.dispose()); enhancementSessions.clear();
});

describe('complete meeting export orchestration', () => {
  test('opens choices read-only, then waits for acknowledged original, enhancement and detached legacy edits', async () => {
    const calls: string[] = [];
    let release!: () => void;
    const pending = new Promise<void>(resolve => { release = resolve; });
    const original = session('export-target', async input => { calls.push(`original:${input.base_revision + 1}`); if (input.base_revision === 0) await pending; return { revision: input.base_revision + 1, updated_at: 'ack' }; });
    noteSessions.set('export-target', original); original.edit(blocks('Before export'));
    const enhanced = session('version-target', async input => { calls.push('enhanced'); return { revision: input.base_revision + 1, updated_at: 'ack' }; });
    enhancementSessions.set('version-target', { meetingId: 'export-target', session: enhanced }); enhanced.edit(blocks('Edited approval'));
    let legacyDirty = true;
    const detach = registerLegacySummaryFlusher('export-target', async () => { calls.push('legacy'); legacyDirty = false; }, () => legacyDirty);
    detach();
    const unrelated = session('unrelated', async () => { throw new Error('Unrelated conflict'); });
    noteSessions.set('unrelated', unrelated); unrelated.edit(blocks('Keep this edit'));
    const service = createMeetingExportService(async <T>(command: string) => {
      calls.push(command);
      return (command === 'api_get_meeting_export_options' ? { meeting_id: 'export-target', title: 'Meeting', status: 'completed', approved_version_id: 'approval' } : { status: 'saved' }) as T;
    }, flushMeetingDocuments, () => false);
    const options = await service.options('export-target');
    expect(options.approved_version_id).toBe('approval');
    expect(calls).toEqual(['api_get_meeting_export_options']); expect(original.dirty).toBe(true);
    const exporting = service.export(request());
    for (let i = 0; i < 10; i++) await Promise.resolve();
    expect(calls).not.toContain('api_export_meeting');
    original.edit(blocks('Typed during save')); release();
    expect(await exporting).toEqual({ status: 'saved' });
    expect(calls).toContain('legacy'); expect(calls).toContain('enhanced');
    expect(calls).toContain('original:2'); expect(calls.at(-1)).toBe('api_export_meeting');
    expect(original.dirty).toBe(false); expect(unrelated.dirty).toBe(true);
  });

  test('an original edit arriving while an enhancement saves is acknowledged before export', async () => {
    let release!: () => void;
    const pending = new Promise<void>(resolve => { release = resolve; });
    const original = session('overlap', async input => ({ revision: input.base_revision + 1, updated_at: 'ack' }));
    noteSessions.set('overlap', original);
    const enhanced = session('overlap-version', async input => { await pending; return { revision: input.base_revision + 1, updated_at: 'ack' }; });
    enhancementSessions.set('overlap-version', { meetingId: 'overlap', session: enhanced }); enhanced.edit(blocks('Enhanced'));
    const flush = flushMeetingDocuments('overlap');
    for (let i = 0; i < 10; i++) await Promise.resolve();
    original.edit(blocks('New original')); release(); await flush;
    expect(original.dirty).toBe(false); expect(original.getSnapshot().revision).toBe(1);
  });

  test('save failure retains edits and never opens the native export dialog', async () => {
    let invoked = false;
    const original = session('conflict', async () => { throw { code: 'conflict', message: 'A newer revision is saved' }; });
    noteSessions.set('conflict', original); original.edit(blocks('Preserve these notes'));
    const service = createMeetingExportService(async <T>() => { invoked = true; return { status: 'saved' } as T; }, flushMeetingDocuments, () => false);
    await expect(service.export(request('conflict'))).rejects.toMatchObject({ code: 'conflict' });
    expect(invoked).toBe(false); expect(original.dirty).toBe(true); expect(original.getSnapshot().blocks).toEqual(blocks('Preserve these notes'));
  });

  test('failed detached legacy saves block their meeting and remain retryable', async () => {
    let dirty = true; let calls = 0;
    const detach = registerLegacySummaryFlusher('legacy-export', async () => { if (++calls === 1) throw new Error('Retry legacy save'); dirty = false; }, () => dirty);
    detach();
    await expect(flushMeetingDocuments('legacy-export')).rejects.toThrow('Retry legacy save');
    await flushMeetingDocuments('legacy-export'); expect(calls).toBe(2);
  });

  test('native cancellation is distinct from success and request options reach the native command', async () => {
    const sent: unknown[] = [];
    const service = createMeetingExportService(async <T>(command: string, args: unknown) => { sent.push({ command, args }); return { status: 'cancelled' } as T; }, async () => {}, () => false);
    const selected = { ...request(), format: 'markdown' as const, document: 'both' as const, include_transcript: true, expected_approved_version_id: 'approved' };
    expect(await service.export(selected)).toEqual({ status: 'cancelled' });
    expect(sent).toEqual([{ command: 'api_export_meeting', args: { request: selected } }]);
  });

  test('deletion during the editor flush prevents export publication', async () => {
    let deleted = false; let invoked = false;
    const service = createMeetingExportService(async <T>() => { invoked = true; return {} as T; }, async () => { deleted = true; }, () => deleted);
    await expect(service.export(request())).rejects.toThrow('deletion is pending'); expect(invoked).toBe(false);
    await expect(service.options('export-target')).rejects.toThrow('deletion is pending');
  });
});
