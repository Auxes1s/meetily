import { describe, expect, test } from 'bun:test';
import type { Block } from '@blocknote/core';
import { createEnhancementSession, type EnhancementSave } from '../../src/lib/enhancement-session';
import type { EnhancementVersion } from '../../src/types/enhancement';
import { enhancementMarkdown, parseEnhancementDocument } from '../../src/types/enhancement';

const blocks = (text: string) => [{ id: 'b', type: 'paragraph', content: [{ type: 'text', text }], children: [] }] as unknown as Block[];
const version: EnhancementVersion = {
  id: 'version-a', meeting_id: 'meeting-a', state: 'candidate', status: 'ready', revision: 0,
  document_json: JSON.stringify({ markdown: 'Generated', claims: [{ owner: 'Old owner' }], coverage: { complete: true } }),
  source_snapshot_json: '{}', source_fingerprint: 'snapshot', notes_revision: 3,
  template_id: 'standard_meeting', instructions: '', created_at: '', updated_at: '', error: null, source_stale: false,
};
describe('protected enhancement saves', () => {
  test('historical rich and twice-encoded summaries retain their readable content', () => {
    const raw = { Topics: { title: 'Budget — 日本語', blocks: [{ content: 'Proposed; no approval or deadline.' }] }, _section_order: ['Topics'] };
    const document = parseEnhancementDocument(JSON.stringify(JSON.stringify(raw)));
    expect(enhancementMarkdown(document)).toContain('## Budget — 日本語');
    expect(enhancementMarkdown(document)).toContain('Proposed; no approval or deadline.');
    const rich = { summary_json: blocks('Original rich text') };
    expect(parseEnhancementDocument(JSON.stringify(rich)).summary_json).toEqual(rich.summary_json);
    expect(() => parseEnhancementDocument('{corrupt')).toThrow();
    expect(() => parseEnhancementDocument('[]')).toThrow('preserved');
  });
  test('a lost acknowledgment retries the same version commit before saving newer edits', async () => {
    const calls: EnhancementSave[] = [];
    let lost = true;
    const session = createEnhancementSession(version, blocks('Generated'), async input => {
      calls.push(structuredClone(input));
      if (lost) { lost = false; throw new Error('Lost acknowledgment'); }
      return { ...version, revision: input.expected_revision + 1, updated_at: 'committed' };
    });
    session.edit(blocks('First edit'));
    await expect(session.flush()).rejects.toThrow('Lost acknowledgment');
    session.edit(blocks('Newest edit'));
    await session.flush();
    expect(calls.map(call => call.expected_revision)).toEqual([0, 0, 1]);
    expect(calls[0].commit_id).toBe(calls[1].commit_id);
    expect(calls[2].commit_id).not.toBe(calls[1].commit_id);
    expect(calls.every(call => call.meeting_id === 'meeting-a' && call.version_id === 'version-a')).toBe(true);
    expect(calls[2].document.summary_json).toEqual(blocks('Newest edit'));
    expect(calls[2].document.claims).toEqual([]);
    expect(calls[2].document.coverage).toEqual({ complete: true });
    expect(session.getSnapshot().status).toBe('Saved');
    session.dispose();
  });
  test('a revision conflict retains edits and stops automatic overwrites', async () => {
    let writes = 0;
    const session = createEnhancementSession(version, blocks('Generated'), async () => {
      writes++; throw 'Enhancement revision conflict';
    });
    session.edit(blocks('Retain my changes'));
    await expect(session.flush()).rejects.toBeDefined();
    await expect(session.flush()).rejects.toBeDefined();
    expect(writes).toBe(1);
    expect(session.getSnapshot().blocks).toEqual(blocks('Retain my changes'));
    expect(session.getSnapshot().status).toBe('Conflict');
    session.dispose();
  });
});
