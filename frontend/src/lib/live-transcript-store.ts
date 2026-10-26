import type { Transcript } from '@/types';
import { mergeTranscript, transcriptTime } from './transcript-audio-source';

/** Canonical rows and replaceable tails have independent subscriptions. */
export class LiveTranscriptStore {
  private committed = new Map<string, Transcript>();
  private tails = new Map<string, Transcript>();
  private ids: string[] = [];
  private listeners = new Set<() => void>();
  private allListeners = new Set<() => void>();
  private rows = new Map<string, Set<() => void>>();
  private cached: Transcript[] | null = null;
  get = (id: string): Transcript | undefined => this.committed.get(id) ?? this.tails.get(id);
  getIds = () => this.ids;
  snapshot = (): Transcript[] => this.cached ??= this.ids.map(id => this.get(id)!);
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  subscribeAll = (listener: () => void) => { this.allListeners.add(listener); return () => { this.allListeners.delete(listener); }; };
  subscribeRow = (id: string, listener: () => void) => {
    let listeners = this.rows.get(id); if (!listeners) this.rows.set(id, listeners = new Set());
    listeners.add(listener);
    return () => { listeners!.delete(listener); if (!listeners!.size) this.rows.delete(id); };
  };
  private compare = (a: Transcript, b: Transcript) =>
    (transcriptTime(a) ?? a.chunk_start_time ?? 0) - (transcriptTime(b) ?? b.chunk_start_time ?? 0)
    || (a.sequence_id ?? 0) - (b.sequence_id ?? 0) || a.id.localeCompare(b.id);
  upsert(incoming: Transcript[]) {
    let idsChanged = false;
    const changed = new Set<string>();
    for (const item of incoming) {
      const previous = this.get(item.id);
      const merged = mergeTranscript(previous, item);
      if (merged === previous) continue;
      const reposition = !previous || this.compare(previous, merged) !== 0;
      if (merged.is_partial) this.tails.set(item.id, merged);
      else { this.tails.delete(item.id); this.committed.set(item.id, merged); }
      if (reposition) {
        if (!idsChanged) this.ids = this.ids.slice();
        if (previous) this.ids.splice(this.ids.indexOf(item.id), 1);
        let low = 0, high = this.ids.length;
        while (low < high) { const middle = (low + high) >>> 1; if (this.compare(this.get(this.ids[middle])!, merged) <= 0) low = middle + 1; else high = middle; }
        this.ids.splice(low, 0, item.id); idsChanged = true;
      }
      changed.add(item.id);
    }
    if (!changed.size) return;
    this.cached = null;
    for (const id of changed) this.rows.get(id)?.forEach(notify => notify());
    if (idsChanged) this.listeners.forEach(notify => notify());
    this.allListeners.forEach(notify => notify());
  }
  clear() {
    const changed = this.ids; this.committed.clear(); this.tails.clear(); this.ids = []; this.cached = [];
    for (const id of changed) this.rows.get(id)?.forEach(notify => notify());
    this.listeners.forEach(notify => notify()); this.allListeners.forEach(notify => notify());
  }
}
