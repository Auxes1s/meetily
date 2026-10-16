import { describe, expect, test } from 'bun:test';
import { flushAllLegacySummaries, forgetMeetingLegacySummaries, registerLegacySummaryFlusher } from '../../src/lib/legacy-summary-flush';

describe('legacy summary lifecycle saves', () => {
  test('quit or navigation waits for an edit that arrives during the first save', async () => {
    let edited = 1;
    let acknowledged = 0;
    let release: () => void = () => {};
    let calls = 0;
    const cleanup = registerLegacySummaryFlusher('overlapping-lifecycle-fixture', async () => {
      calls++;
      const saving = edited;
      if (calls === 1) await new Promise<void>(resolve => { release = resolve; });
      acknowledged = saving;
    }, () => acknowledged !== edited);
    let finished = false;
    const flushing = flushAllLegacySummaries().then(() => { finished = true; });
    await Promise.resolve();
    edited = 2;
    expect(finished).toBe(false);
    release();
    await flushing;
    expect(calls).toBe(2);
    expect(acknowledged).toBe(2);
    cleanup();
  });
  test('detached dirty views retain failed saves for retry and release their callback after acknowledgement', async () => {
    let dirty = true;
    let calls = 0;
    const cleanup = registerLegacySummaryFlusher('detached-summary-fixture', async () => {
      calls++;
      if (calls === 1) throw new Error('Retry this save');
      dirty = false;
    }, () => dirty);
    cleanup();
    await expect(flushAllLegacySummaries()).rejects.toThrow('Retry this save');
    await flushAllLegacySummaries();
    await flushAllLegacySummaries();
    expect(calls).toBe(2);
    forgetMeetingLegacySummaries('detached-summary-fixture');
  });
  test('deletion forgets pending drafts and rejects late view registration', async () => {
    let calls = 0;
    const cleanup = registerLegacySummaryFlusher('deleted-summary-fixture', async () => { calls++; }, () => true);
    cleanup();
    forgetMeetingLegacySummaries('deleted-summary-fixture');
    registerLegacySummaryFlusher('deleted-summary-fixture', async () => { calls++; }, () => true);
    await flushAllLegacySummaries();
    expect(calls).toBe(0);
  });
});
