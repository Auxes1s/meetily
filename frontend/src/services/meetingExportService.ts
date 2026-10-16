import { invoke } from '@tauri-apps/api/core';
import { flushMeetingDocuments } from '@/lib/workspace-save';
import { isMeetingDeletionBlocked } from './meetingDeletionService';
import type { MeetingExportOptions, MeetingExportRequest, MeetingExportResult } from '@/types/meeting-export';

type Gateway = <T>(command: string, args: Record<string, unknown>) => Promise<T>;

export function createMeetingExportService(
  gateway: Gateway = invoke,
  flush = flushMeetingDocuments,
  deletionBlocked = isMeetingDeletionBlocked,
) {
  const available = (meetingId: string) => {
    if (deletionBlocked(meetingId)) throw new Error('Meeting is unavailable or deletion is pending.');
  };
  return {
    async options(meetingId: string): Promise<MeetingExportOptions> {
      available(meetingId);
      return gateway('api_get_meeting_export_options', { meetingId });
    },
    async export(request: MeetingExportRequest): Promise<MeetingExportResult> {
      available(request.meeting_id);
      await flush(request.meeting_id);
      available(request.meeting_id);
      return gateway('api_export_meeting', { request });
    },
  };
}

export const meetingExportService = createMeetingExportService();
