export interface DisplayedSummary {
  versionId: string;
  revision: number;
  document: Record<string, unknown>;
}
export interface LegacySummaryCommit {
  meetingId: string;
  versionId: string;
  baseRevision: number;
  commitId: string;
  summary: Record<string, unknown>;
}

export function readDisplayedSummary(value: unknown): DisplayedSummary | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const { _enhancement_version_id, _enhancement_revision, ...document } = value as Record<string, unknown>;
  if (typeof _enhancement_version_id !== 'string' || !Number.isInteger(_enhancement_revision) || (_enhancement_revision as number) < 0) return null;
  return { versionId: _enhancement_version_id, revision: _enhancement_revision as number, document: structuredClone(document) };
}

/** Legacy section renderers must never turn document metadata into editable sections. */
export function legacySummarySections(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  return Object.fromEntries(Object.entries(value).filter(([, section]) =>
    section && typeof section === 'object' && !Array.isArray(section)
      && Array.isArray((section as { blocks?: unknown }).blocks)));
}

/** Manual legacy saves use the same exact-commit replay contract as autosave. */
export class LegacySummarySaveQueue {
  private document: Record<string, unknown>;
  private pending: LegacySummaryCommit | null = null;
  private flight: Promise<void> | null = null;
  get hasPending() { return this.pending !== null; }
  revision: number;
  constructor(readonly meetingId: string, readonly versionId: string, displayed: DisplayedSummary,
    private gateway: (input: LegacySummaryCommit) => Promise<{ revision: number }>) {
    this.document = structuredClone(displayed.document);
    this.revision = displayed.revision;
  }
  private merge(patch: Record<string, unknown>) {
    const document: Record<string, unknown> = { ...this.document, ...structuredClone(patch), claims: [], user_edited: true };
    delete (document as Record<string, unknown>).english_cache;
    delete (document as Record<string, unknown>)._enhancement_version_id;
    delete (document as Record<string, unknown>)._enhancement_revision;
    // Failed conversion must not retain Markdown from before the rich-text edit.
    if ('summary_json' in patch && !('markdown' in patch)) document.markdown = null;
    return document;
  }
  save(patch: Record<string, unknown>): Promise<void> {
    if (this.flight) return Promise.reject(new Error('Wait for the current document save to finish.'));
    const requested = this.merge(patch);
    this.flight = (async () => {
      if (this.pending) {
        const replay = this.pending;
        const result = await this.gateway(replay);
        this.revision = result.revision;
        this.document = replay.summary;
        this.pending = null;
        if (JSON.stringify(requested) === JSON.stringify(replay.summary)) return;
      }
      this.pending = {
        meetingId: this.meetingId, versionId: this.versionId, baseRevision: this.revision,
        commitId: crypto.randomUUID(), summary: this.merge(patch),
      };
      const result = await this.gateway(this.pending);
      this.revision = result.revision;
      this.document = this.pending.summary;
      this.pending = null;
    })().finally(() => { this.flight = null; });
    return this.flight;
  }
  retryPending(): Promise<void> {
    if (this.flight) return this.flight;
    if (!this.pending) return Promise.resolve();
    const pending = this.pending;
    this.flight = (async () => {
      const result = await this.gateway(pending);
      this.revision = result.revision;
      this.document = pending.summary;
      this.pending = null;
    })().finally(() => { this.flight = null; });
    return this.flight;
  }
}
