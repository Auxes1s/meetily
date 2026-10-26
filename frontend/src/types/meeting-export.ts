import type { NoteLink } from '@/lib/meeting-notes-session';
import type { AudioSource } from '@/types';

export type MeetingExportFormat = 'markdown' | 'json';
export type MeetingExportDocument = 'original' | 'approved' | 'both';
export interface MeetingExportRequest {
  meeting_id: string;
  format: MeetingExportFormat;
  document: MeetingExportDocument;
  include_transcript: boolean;
  expected_approved_version_id: string | null;
}
export interface MeetingExportOptions {
  meeting_id: string;
  title: string;
  status: string;
  approved_version_id: string | null;
}
export type MeetingExportResult = { status: 'saved' } | { status: 'cancelled' };
export type ExportJson = null | boolean | number | string | ExportJson[] | { [key: string]: ExportJson };
export interface ExportPassage {
  id: string;
  text: string;
  audio_start_time: number | null;
  audio_end_time: number | null;
  audio_source: AudioSource;
  audio_source_label: string;
}

/** Portable records retain version 1 read compatibility. Version 2 adds sessions. */
export interface MeetingRecordExport {
  format: 'meetily.meeting-record';
  schema_version: 1 | 2;
  exported_at: string;
  transcript_coverage: 'committed_available' | 'incomplete_active' | 'incomplete_interrupted';
  meeting: {
    id: string; title: string; created_at: string; updated_at: string; status: string;
    started_at: string | null; ended_at: string | null; timezone: string | null;
    recording_session_id: string | null; participants: string[]; tags: string[];
  };
  original_notes: {
    notes_json: string | null; notes_markdown: string | null; notes_plain_text: string | null;
    conversion_status: string; revision: number; created_at: string; updated_at: string;
    blocks: ExportJson | null; timestamp_links: (NoteLink & { passage: ExportPassage | null })[];
  } | null;
  transcripts: (ExportPassage & {
    timestamp: string; recording_session_id: string | null; sequence_id: number | null;
    duration: number | null; summary: string | null; action_items: string | null; key_points: string | null;
    timeline_offset?: number;
  })[];
  recording_sessions?: {
    id: string; ordinal: number; status: string; started_at: string | null; ended_at: string | null;
    duration_seconds: number | null; timeline_offset: number; duration_basis: string;
  }[];
  enhancement_settings: { template_id: string; instructions: string; revision: number; approved_version_id: string | null } | null;
  versions: {
    id: string; state: string; status: string; revision: number;
    document_json: string | null; document: ExportJson | null;
    source_snapshot_json: string | null; source_snapshot: ExportJson | null;
    source_fingerprint: string | null; notes_revision: number | null;
    template_id: string; instructions: string; provenance: ExportJson;
    created_at: string; updated_at: string; error: string | null; source_stale: boolean | null;
  }[];
  legacy_summary: {
    status: string; created_at: string; updated_at: string; result: string | null;
    result_backup: string | null; result_backup_timestamp: string | null;
    verification_status: 'unverified'; document: ExportJson | null; backup_document: ExportJson | null;
  } | null;
  warnings: string[];
}
