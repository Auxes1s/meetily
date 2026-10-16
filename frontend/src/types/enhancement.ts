import type { Block } from '@blocknote/core';

export interface EnhancementClaim {
  id: string;
  kind: string;
  text: string;
  status: string;
  sources: Array<{ source_id: string; excerpt: string }>;
  owner: string | null;
  deadline: string | null;
}
export interface EnhancementDocument {
  schema_version?: number;
  markdown?: string | null;
  summary_json?: Block[];
  claims?: EnhancementClaim[];
  user_edited?: boolean;
  [key: string]: unknown;
}
export interface EnhancementVersion {
  id: string;
  meeting_id: string;
  state: string;
  status: string;
  revision: number;
  document_json: string | null;
  source_snapshot_json: string | null;
  source_fingerprint: string | null;
  notes_revision: number | null;
  template_id: string;
  instructions: string;
  created_at: string;
  updated_at: string;
  error: string | null;
  source_stale: boolean;
}
export interface EnhancementSettings { template_id: string; instructions: string; revision: number }
export interface MeetingEnhancements {
  meeting_id: string;
  approved_version_id: string | null;
  settings: EnhancementSettings;
  versions: EnhancementVersion[];
}

/** Older summaries may be JSON encoded twice; never silently replace malformed data. */
export function parseEnhancementDocument(value: string | null): EnhancementDocument {
  let parsed: unknown = value == null ? {} : value;
  for (let depth = 0; depth < 2 && typeof parsed === 'string'; depth++) parsed = JSON.parse(parsed);
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('Stored enhanced document is malformed; its original is preserved.');
  }
  return parsed as EnhancementDocument;
}

/** Convert old section summaries for editing without modifying their stored originals. */
export function enhancementMarkdown(document: EnhancementDocument): string {
  if (typeof document.markdown === 'string') return document.markdown;
  const ordered = Array.isArray(document._section_order) ? document._section_order : Object.keys(document);
  return ordered.flatMap(key => {
    const section = document[String(key)] as { title?: string; blocks?: Array<{ content?: string }> } | undefined;
    return section && Array.isArray(section.blocks)
      ? [`## ${section.title ?? String(key)}\n\n${section.blocks.map(block => block.content ?? '').join('\n\n')}`]
      : [];
  }).join('\n\n');
}

export interface GenerationProgress {
  completed_chunks: number;
  total_chunks: number;
  failed_chunk: number | null;
  error_code: string | null;
  retryable: boolean;
}
