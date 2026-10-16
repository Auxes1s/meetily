import { expect, test } from 'bun:test';
import { LiveTranscriptStore } from '../../src/lib/live-transcript-store';
import type { Transcript } from '../../src/types';
const row = (i: number, extra: Partial<Transcript> = {}): Transcript => ({ id: `row-${i}`, text: `${i}`, timestamp: '', sequence_id: i, audio_start_time: i, is_partial: false, ...extra });
test('10,000 canonical rows retain their index and only the changed tail is notified', () => {
  const store = new LiveTranscriptStore(); store.upsert(Array.from({ length: 10000 }, (_, i) => row(i)));
  store.upsert([row(10000, { is_partial: true })]);
  const ids = store.getIds(), original = store.get('row-5000'); let lists = 0, tail = 0, canonical = 0;
  store.subscribe(() => lists++); store.subscribeRow('row-10000', () => tail++); store.subscribeRow('row-5000', () => canonical++);
  for (let revision = 1; revision <= 100; revision++) store.upsert([row(10000, { is_partial: true, text: `revision ${revision}`, update_revision: revision })]);
  expect(store.getIds()).toBe(ids); expect(lists).toBe(0); expect(tail).toBe(100); expect(canonical).toBe(0); expect(store.get('row-5000')).toBe(original);
  store.upsert([row(10000, { update_revision: 101, text: 'Final', audio_source: 'system' })]);
  store.upsert([row(10000, { is_partial: true, update_revision: 102, text: 'Late partial' })]);
  expect(store.snapshot().at(-1)?.text).toBe('Final'); expect(store.snapshot()).toHaveLength(10001);
});
test('earlier arrivals and timing revisions reorder stable identities without duplicates', () => {
  const store = new LiveTranscriptStore(); store.upsert([row(2), row(0), row(1)]);
  expect(store.getIds()).toEqual(['row-0','row-1','row-2']);
  store.upsert([row(2, { audio_start_time: 0.5, update_revision: 1 })]);
  expect(store.getIds()).toEqual(['row-0','row-2','row-1']);store.clear();expect(store.getIds()).toEqual([]);
});
