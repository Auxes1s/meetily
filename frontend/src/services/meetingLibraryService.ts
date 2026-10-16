import { invoke } from '@tauri-apps/api/core';
import type { LibraryPage, LibrarySearchRequest, SearchSyncStatus } from '@/types/meeting-library';

export const meetingLibraryService = {
  search(request: LibrarySearchRequest): Promise<LibraryPage> {
    return invoke('api_search_meetings', { request });
  },
  list(request: LibrarySearchRequest = {}): Promise<LibraryPage> {
    return invoke('api_list_meetings', { request });
  },
  updateMetadata(meetingId: string, participants: string[], tags: string[]): Promise<void> {
    return invoke('api_update_meeting_metadata', { meetingId, participants, tags });
  },
  syncIndex(limit = 25): Promise<SearchSyncStatus> {
    return invoke('api_sync_meeting_search', { limit });
  },
};
