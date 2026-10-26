import type { Block } from '@blocknote/core';

export interface NoteLink {
  id: string;
  block_id: string;
  recording_session_id: string;
  recording_offset: number;
  transcript_id: string | null;
}
export interface NotesSnapshot {
  blocks: Block[];
  links: NoteLink[];
  title: string;
  revision: number;
  updatedAt: string | null;
  status: 'Saved' | 'Saving' | 'Unsaved' | 'Save failed—retrying' | 'Conflict';
  error: string | null;
}
export interface SaveNotesInput {
  meeting_id: string;
  notes_json: string;
  notes_markdown: string | null;
  notes_plain_text: string | null;
  base_revision: number;
  commit_id: string;
  title: string;
  links: NoteLink[];
}
type Commit = { revision: number; updated_at: string };
export type NotesGateway = (input: SaveNotesInput) => Promise<Commit>;
type Converter = (blocks: Block[]) => Promise<string>;

interface MarkerNode {
  children: Map<string, MarkerNode>;
  id?: string;
}

export function plainText(blocks: Block[]): string {
  const text = (value: any): string => {
    if (typeof value === 'string') return value;
    if (Array.isArray(value)) return value.map(text).join('');
    if (!value || typeof value !== 'object') return '';
    return typeof value.text === 'string' ? value.text : text(value.content);
  };
  return blocks.map(block => `${text(block.content)}${block.children?.length ? `\n${plainText(block.children)}` : ''}`).join('\n');
}

/** One serialized write queue per meeting; mounted React views only subscribe. */
export class MeetingNotesSession {
  private snapshot: NotesSnapshot;
  private listeners = new Set<() => void>();
  private owners = 0;
  private version = 0;
  private acknowledged = 0;
  private firstDirtyAt: number | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private flight: Promise<void> | null = null;
  private converter: Converter | null = null;
  private retryDelay = 1000;
  private pending: { version: number; input: SaveNotesInput } | null = null;
  private markerHistory = new Map<string, NoteLink>();
  private markerOrder = new Map<string, number>();
  private markerRoot: MarkerNode = { children: new Map() };
  private disposed = false;
  constructor(readonly meetingId: string, initial: Omit<NotesSnapshot, 'status' | 'error'>, private gateway: NotesGateway) {
    this.snapshot = { ...initial, status: 'Saved', error: null };
    initial.links.forEach(link => this.rememberMarker(link));
  }
  getSnapshot = () => this.snapshot;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  get dirty() { return this.version !== this.acknowledged; }
  get canEvict() { return !this.dirty && !this.flight && this.listeners.size === 0 && this.owners === 0; }
  /** Own a loaded session through the asynchronous render/subscription handoff. */
  retain() {
    this.owners++;
    let released = false;
    return () => { if (!released) { released = true; this.owners--; } };
  }
  setConverter(converter: Converter) {
    this.converter = converter;
    return () => { if (this.converter === converter) this.converter = null; };
  }
  private publish(patch: Partial<NotesSnapshot>) {
    this.snapshot = { ...this.snapshot, ...patch };
    this.listeners.forEach(listener => listener());
  }
  private rememberMarker(link: NoteLink) {
    if (!this.markerHistory.has(link.id)) {
      this.markerOrder.set(link.id, this.markerOrder.size);
      let node = this.markerRoot;
      for (let i = 0; i < link.id.length; i++) {
        const character = link.id[i];
        let child = node.children.get(character);
        if (!child) node.children.set(character, child = { children: new Map() });
        node = child;
      }
      node.id = link.id;
    }
    this.markerHistory.set(link.id, link);
  }
  edit(blocks: Block[], links = this.snapshot.links) {
    // Keep marker metadata in memory so undo can restore a deleted marker.
    links.forEach(link => this.rememberMarker(link));
    const active: NoteLink[] = [];
    const seen = new Set<string>();
    const prefix = 'meetily-note://';
    const visit = (items: Block[]) => items.forEach(block => {
      // Scan content once, preserving the former literal-prefix matching and
      // history order (including IDs that are prefixes of another marker ID).
      const content = JSON.stringify(block.content) ?? '';
      const matches = new Set<string>();
      for (let start = content.indexOf(prefix); start !== -1; start = content.indexOf(prefix, start + prefix.length)) {
        let node: MarkerNode | undefined = this.markerRoot;
        if (node.id !== undefined) matches.add(node.id);
        for (let i = start + prefix.length; i < content.length; i++) {
          node = node.children.get(content[i]);
          if (!node) break;
          if (node.id !== undefined) matches.add(node.id);
        }
      }
      for (const id of [...matches].sort((a, b) => this.markerOrder.get(a)! - this.markerOrder.get(b)!)) {
        if (!seen.has(id)) {
          seen.add(id);
          active.push({ ...this.markerHistory.get(id)!, block_id: block.id });
        }
      }
      if (block.children?.length) visit(block.children);
    });
    visit(blocks);
    this.change({ blocks, links: active });
  }
  setTitle(title: string) { this.change({ title }); }
  private change(patch: Partial<NotesSnapshot>) {
    if (this.disposed) return;
    this.version++;
    this.firstDirtyAt ??= Date.now();
    this.publish({ ...patch, status: this.snapshot.status === 'Conflict' ? 'Conflict' : this.flight ? 'Saving' : 'Unsaved' });
    if (this.snapshot.status !== 'Conflict') this.schedule();
  }
  private schedule(delay = Math.max(0, Math.min(750, 2000 - (Date.now() - (this.firstDirtyAt ?? Date.now()))))) {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => { void this.flush().catch(() => {}); }, delay);
  }
  flush(): Promise<void> {
    if (this.disposed) return Promise.reject(new Error('This document session has been closed.'));
    clearTimeout(this.timer);
    if (this.flight) return this.flight;
    this.flight = Promise.resolve().then(() => this.drain()).finally(() => { this.flight = null; });
    return this.flight;
  }
  private async drain(): Promise<void> {
    if (this.snapshot.status === 'Conflict') throw new Error(this.snapshot.error ?? 'Notes conflict');
    while (this.dirty && !this.disposed) {
      const version = this.pending?.version ?? this.version;
      // Capture authoritative blocks immediately, before any async conversion.
      const input: SaveNotesInput = this.pending?.input ?? {
        meeting_id: this.meetingId, notes_json: JSON.stringify(this.snapshot.blocks),
        notes_markdown: null, notes_plain_text: plainText(this.snapshot.blocks),
        base_revision: this.snapshot.revision, commit_id: crypto.randomUUID(), title: this.snapshot.title, links: structuredClone(this.snapshot.links),
      };
      const retrying = this.pending !== null;
      this.pending = { version, input };
      const blocks = JSON.parse(input.notes_json) as Block[];
      this.publish({ status: 'Saving', error: null });
      const write = (async () => {
        // A failed/hung projection must not delay native JSON durability.
        if (this.converter && !retrying) {
          let deadline: ReturnType<typeof setTimeout> | undefined;
          try {
            input.notes_markdown = await Promise.race([
              this.converter(blocks),
              new Promise<null>(resolve => { deadline = setTimeout(() => resolve(null), 150); }),
            ]);
          } catch { input.notes_markdown = null; }
          finally { clearTimeout(deadline); }
        }
        if (this.disposed) throw new Error('This document session has been closed.');
        const commit = await this.gateway(input);
        this.acknowledged = version;
        this.pending = null;
        this.retryDelay = 1000;
        this.publish({ revision: commit.revision, updatedAt: commit.updated_at, status: this.dirty ? 'Unsaved' : 'Saved' });
        if (!this.dirty) { this.firstDirtyAt = null; clearTimeout(this.timer); }
      })();
      try { await write; }
      catch (error: any) {
        const conflict = error?.code === 'conflict';
        this.publish({ status: conflict ? 'Conflict' : 'Save failed—retrying', error: error?.message ?? String(error) });
        if (!conflict && !this.disposed) { this.schedule(this.retryDelay); this.retryDelay = Math.min(this.retryDelay * 2, 10000); }
        throw error;
      }
    }
  }
  dispose() { this.disposed = true; clearTimeout(this.timer); this.listeners.clear(); this.converter = null; }
}

/** Dirty, writing and subscribed sessions remain owned regardless of the cap. */
export function pruneDocumentSessions<T>(cache: Map<string, T>, session: (entry: T) => MeetingNotesSession, protectedId?: string, limit = 8) {
  for (const [id, entry] of cache) {
    if (cache.size <= limit) break;
    const value = session(entry);
    if (id !== protectedId && value.canEvict) { value.dispose(); cache.delete(id); }
  }
}

export const noteSessions = new Map<string, MeetingNotesSession>();
export async function flushAllNotes() {
  await Promise.all([...noteSessions.values()].map(session => session.flush()));
}
