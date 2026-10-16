export type LibrarySort = 'relevance' | 'date_desc' | 'date_asc' | 'title_asc' | 'updated_desc';
export type LibraryMeetingStatus = 'draft' | 'recording' | 'finalizing' | 'completed' | 'interrupted';
export type LibrarySourceKind = 'title' | 'original' | 'transcript' | 'approved';

export interface LibrarySearchRequest {
  query?: string;
  participant?: string | null;
  tag?: string | null;
  /** Inclusive RFC3339 instant. */
  date_from?: string | null;
  /** Exclusive RFC3339 instant. */
  date_to?: string | null;
  status?: LibraryMeetingStatus | null;
  sort?: LibrarySort;
  cursor?: string | null;
  limit?: number;
}

export interface LibrarySourceHit {
  source_kind: LibrarySourceKind;
  source_id: string;
  revision: number | null;
  snippet: string;
  audio_start_time: number | null;
  audio_end_time: number | null;
}

export interface LibraryMeeting {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  started_at: string | null;
  ended_at: string | null;
  timezone: string | null;
  status: LibraryMeetingStatus;
  date_is_creation_fallback: boolean;
  index_updating: boolean;
  participants: string[];
  tags: string[];
  hits: LibrarySourceHit[];
}

export interface LibraryPage {
  meetings: LibraryMeeting[];
  total_count: number;
  next_cursor: string | null;
  index_updating: boolean;
  pending_index_count: number;
}
export interface SearchSyncStatus { indexed_count: number; pending_count: number }
