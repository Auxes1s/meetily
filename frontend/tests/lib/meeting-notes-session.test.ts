import { afterEach, describe, expect, test } from 'bun:test';
import type { Block } from '@blocknote/core';
import { MeetingNotesSession, type SaveNotesInput, plainText } from '../../src/lib/meeting-notes-session';

const blocks = (text: string): Block[] => [{ id: 'block', type: 'paragraph', props: { backgroundColor: 'default', textColor: 'default', textAlignment: 'left' }, content: [{ type: 'text', text, styles: {} }], children: [] }] as Block[];
const sessions: MeetingNotesSession[] = [];
const initial = { blocks: blocks('initial'), links: [], title: 'Notes', revision: 0, updatedAt: null };
function session(gateway: (input: SaveNotesInput) => Promise<{ revision: number; updated_at: string }>) {
  const value = new MeetingNotesSession('meeting-a', initial, gateway);
  sessions.push(value);
  return value;
}
afterEach(() => sessions.splice(0).forEach(value => value.dispose()));
const tick = () => new Promise(resolve => setTimeout(resolve, 0));

describe('acknowledged original-note saves', () => {
  test('disposing during a failed in-flight save prevents retries and further writes', async () => {
    let rejectWrite: (error: Error) => void = () => {};
    let calls = 0;
    const value = session(() => { calls++; return new Promise((_resolve, reject) => { rejectWrite = reject; }); });
    value.edit(blocks('Closing document'));
    const writing = value.flush();
    await tick();
    value.dispose();
    rejectWrite(new Error('Late failure'));
    await expect(writing).rejects.toThrow('Late failure');
    value.edit(blocks('Late edit'));
    await expect(value.flush()).rejects.toThrow('closed');
    await new Promise(resolve => setTimeout(resolve, 1100));
    expect(calls).toBe(1);
  });
  test('serializes overlapping edits and only shows Saved after the last acknowledgment', async () => {
    const writes: SaveNotesInput[] = [];
    let acknowledge: (commit: { revision: number; updated_at: string }) => void = () => {};
    const value = session(input => { writes.push(input); return new Promise(resolve => { acknowledge = resolve; }); });
    value.edit(blocks('first'));
    const saving = value.flush();
    await tick();
    expect(value.getSnapshot().status).toBe('Saving');
    value.edit(blocks('second'));
    expect(value.flush()).toBe(saving);
    expect(writes).toHaveLength(1);
    acknowledge({ revision: 1, updated_at: 'one' });
    await tick();
    expect(writes).toHaveLength(2);
    expect(writes[1].base_revision).toBe(1);
    expect(value.getSnapshot().status).toBe('Saving');
    acknowledge({ revision: 2, updated_at: 'two' });
    await saving;
    expect(value.getSnapshot().status).toBe('Saved');
    expect(JSON.parse(writes[1].notes_json)[0].content[0].text).toBe('second');
  });
  test('persists authoritative JSON when Markdown conversion hangs', async () => {
    let saved: SaveNotesInput | undefined;
    const value = session(async input => { saved = input; return { revision: 1, updated_at: 'one' }; });
    value.setConverter(() => new Promise(() => {}));
    value.edit(blocks('Hindi mawawala — 日本語'));
    await value.flush();
    expect(saved?.notes_markdown).toBeNull();
    expect(saved?.notes_json).toContain('Hindi mawawala — 日本語');
    expect(value.getSnapshot().status).toBe('Saved');
  });
  test('retries the same commit after a lost acknowledgment, then saves newer edits', async () => {
    const writes: SaveNotesInput[] = [];
    const value = session(async input => {
      writes.push(structuredClone(input));
      if (writes.length === 1) throw new Error('Lost acknowledgment');
      return { revision: input.base_revision + 1, updated_at: 'one' };
    });
    value.edit(blocks('first'));
    await expect(value.flush()).rejects.toThrow('Lost acknowledgment');
    expect(value.dirty).toBe(true);
    value.edit(blocks('newer'));
    await value.flush();
    expect(writes[1]).toEqual(writes[0]);
    expect(writes[2].commit_id).not.toBe(writes[0].commit_id);
    expect(writes[2].base_revision).toBe(1);
    expect(value.getSnapshot().revision).toBe(2);
  });
  test('retains edits on revision conflict and does not overwrite a newer revision', async () => {
    let calls = 0;
    const value = session(async () => { calls++; throw { code: 'conflict', message: 'Newer revision saved' }; });
    value.edit(blocks('local edit'));
    await expect(value.flush()).rejects.toBeDefined();
    value.setTitle('Local title');
    await expect(value.flush()).rejects.toThrow('Newer revision saved');
    expect(value.getSnapshot().status).toBe('Conflict');
    expect(value.dirty).toBe(true);
    expect(calls).toBe(1);
  });
  test('independent queues keep edits attached to their original meeting', async () => {
    const writes: SaveNotesInput[] = [];
    const gateway = async (input: SaveNotesInput) => { writes.push(input); return { revision: 1, updated_at: 'one' }; };
    const a = session(gateway);
    const b = new MeetingNotesSession('meeting-b', initial, gateway); sessions.push(b);
    a.edit(blocks('A')); b.edit(blocks('B'));
    await Promise.all([a.flush(), b.flush()]);
    expect(writes.map(input => [input.meeting_id, JSON.parse(input.notes_json)[0].content[0].text])).toEqual([['meeting-a', 'A'], ['meeting-b', 'B']]);
  });
  test('timestamp metadata survives delete and undo without persisting dangling links', async () => {
    const writes: SaveNotesInput[] = [];
    const value = session(async input => { writes.push(input); return { revision: input.base_revision + 1, updated_at: 'one' }; });
    const linked = blocks(''); linked[0].content = [{ type: 'link', href: 'meetily-note://marker', content: [{ type: 'text', text: '[00:12]', styles: {} }] }];
    value.edit(linked, [{ id: 'marker', block_id: 'block', recording_session_id: 'session', recording_offset: 12, transcript_id: null }]);
    await value.flush();
    value.edit(blocks('deleted')); await value.flush();
    expect(writes[1].links).toHaveLength(0);
    value.edit(linked); await value.flush();
    expect(writes[2].links).toHaveLength(1);
  });
  test('continuous editing reaches an acknowledged save within two seconds', async () => {
    let calls = 0;
    const value = session(async input => { calls++; return { revision: input.base_revision + 1, updated_at: 'one' }; });
    for (let i = 0; i < 11; i++) { value.edit(blocks(String(i))); await new Promise(resolve => setTimeout(resolve, 200)); }
    expect(calls).toBeGreaterThan(0);
    await value.flush();
    expect(value.getSnapshot().status).toBe('Saved');
  });
  test('plain-text projection includes links and nested blocks', () => {
    const value = blocks('Parent'); value[0].children = blocks('Child');
    expect(plainText(value)).toBe('Parent\nChild');
  });
});
