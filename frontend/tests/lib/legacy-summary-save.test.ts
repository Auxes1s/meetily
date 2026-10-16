import { describe, expect, test } from 'bun:test';
import { LegacySummarySaveQueue, legacySummarySections, readDisplayedSummary, type LegacySummaryCommit } from '../../src/lib/legacy-summary-save';

const original = {
  _enhancement_version_id: 'legacy-a', _enhancement_revision: 4,
  Agenda: { title: 'Agenda', blocks: [{ id: 'Agenda-1', content: 'Original proposal' }] },
  extension: { preserved: 17 }, coverage: { complete: true },
  claims: [{ owner: 'Original owner' }], english_cache: { markdown: 'Old translation' },
};
describe('legacy summary compatibility', () => {
  test('section edits retain captured identity and extension fields without rendering metadata', async () => {
    const displayed = readDisplayedSummary(original)!;
    const sections = legacySummarySections(original)!;
    expect(Object.keys(sections)).toEqual(['Agenda']);
    sections.Agenda = { title: 'Agenda', blocks: [{ content: 'Tentative proposal, awaiting review' }] };
    expect(readDisplayedSummary(sections)).toBeNull();
    let saved!: LegacySummaryCommit;
    const queue = new LegacySummarySaveQueue('meeting-a', displayed.versionId, displayed, async input => {
      saved = input; return { revision: input.baseRevision + 1 };
    });
    await queue.save(sections);
    expect(saved.versionId).toBe('legacy-a');
    expect(saved.baseRevision).toBe(4);
    expect(saved.summary.extension).toEqual({ preserved: 17 });
    expect(saved.summary.coverage).toEqual({ complete: true });
    expect(saved.summary.claims).toEqual([]);
    expect(saved.summary.english_cache).toBeUndefined();
    expect(saved.summary._enhancement_revision).toBeUndefined();
  });
  test('lost acknowledgment replays its exact payload before committing newer rich text', async () => {
    const calls: LegacySummaryCommit[] = [];
    let revision = 4;
    let lost = true;
    const committed = new Set<string>();
    const queue = new LegacySummarySaveQueue('meeting-a', 'legacy-a', readDisplayedSummary(original)!, async input => {
      calls.push(structuredClone(input));
      if (!committed.has(input.commitId)) {
        if (input.baseRevision !== revision) throw new Error('revision conflict');
        committed.add(input.commitId); revision++;
      }
      if (lost) { lost = false; throw new Error('Lost acknowledgement'); }
      return { revision };
    });
    expect(queue.hasPending).toBe(false);
    await expect(queue.save({ markdown: 'Edit A' })).rejects.toThrow('Lost acknowledgement');
    expect(queue.hasPending).toBe(true);
    await queue.save({ summary_json: [{ content: [{ text: 'Edit B' }] }] });
    expect(calls.map(call => call.baseRevision)).toEqual([4, 4, 5]);
    expect(calls[1]).toEqual(calls[0]);
    expect(calls[2].commitId).not.toBe(calls[1].commitId);
    expect(calls[2].summary.summary_json).toEqual([{ content: [{ text: 'Edit B' }] }]);
    expect(calls[2].summary.markdown).toBeNull();
    expect(calls[2].summary.extension).toEqual({ preserved: 17 });
    expect(queue.revision).toBe(6);
    expect(queue.hasPending).toBe(false);
  });
  test('an external revision conflict retains the pending commit without advancing its base', async () => {
    const calls: LegacySummaryCommit[] = [];
    const queue = new LegacySummarySaveQueue('meeting-a', 'legacy-a', readDisplayedSummary(original)!, async input => {
      calls.push(structuredClone(input)); throw new Error('revision conflict');
    });
    await expect(queue.save({ markdown: 'Retain edit A' })).rejects.toThrow('revision conflict');
    await expect(queue.save({ markdown: 'Retain newer edit B' })).rejects.toThrow('revision conflict');
    expect(calls[1]).toEqual(calls[0]);
    expect(queue.revision).toBe(4);
    expect(queue.hasPending).toBe(true);
  });
});
