'use client';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Button } from '@/components/ui/button';
import { noteSessions } from '@/lib/meeting-notes-session';
import { forgetMeetingEnhancements } from '@/lib/enhancement-session';
import { forgetMeetingLegacySummaries } from '@/lib/legacy-summary-flush';
import { resumePendingMeetingDeletions, type DeletionStatus } from '@/services/meetingDeletionService';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';

/** Maintenance starts only after onboarding, using the established native DB. */
export function MeetingDataMaintenance() {
  const [pending, setPending] = useState<DeletionStatus[]>([]);
  const [error, setError] = useState<string | null>(null);
  const { refetchMeetings } = useSidebar();
  const retry = async () => {
    try { setPending((await resumePendingMeetingDeletions()).filter(job => job.state !== 'complete')); setError(null); await refetchMeetings(); }
    catch (error) { setError((error as Error)?.message ?? String(error)); }
  };
  useEffect(() => {
    let alive = true;
    let indexing: ReturnType<typeof setTimeout>;
    const schedule = () => {
      clearTimeout(indexing);
      indexing = setTimeout(() => { void invoke('api_sync_meeting_search', { limit: 20 }).catch(() => {}); }, 1000);
    };
    const started = (event: Event) => {
      const { meetingId, status } = (event as CustomEvent<{ meetingId: string; status: DeletionStatus }>).detail;
      noteSessions.get(meetingId)?.dispose(); noteSessions.delete(meetingId);
      forgetMeetingEnhancements(meetingId);
      forgetMeetingLegacySummaries(meetingId);
      setPending(jobs => [...jobs.filter(job => job.meeting_id !== meetingId), status]);
      void refetchMeetings();
    };
    const deleted = (event: Event) => {
      const detail = (event as CustomEvent<string | { meetingId: string; status: DeletionStatus }>).detail;
      const id = typeof detail === 'string' ? detail : detail.meetingId;
      noteSessions.get(id)?.dispose(); noteSessions.delete(id); forgetMeetingEnhancements(id);
      forgetMeetingLegacySummaries(id);
      if (typeof detail !== 'string') setPending(jobs => [...jobs.filter(job => job.meeting_id !== id), ...(detail.status.state === 'complete' ? [] : [detail.status])]);
      schedule();
    };
    window.addEventListener('meeting-deletion-started', started);
    window.addEventListener('workspace-meeting-deleted', deleted);
    window.addEventListener('meeting-notes-committed', schedule);
    window.addEventListener('meeting-data-changed', schedule);
    window.addEventListener('enhancement-document-committed', schedule);
    const listener = listen('database-initialized', () => { void retry(); schedule(); });
    void resumePendingMeetingDeletions().then(jobs => { if (alive) setPending(jobs.filter(job => job.state !== 'complete')); }).catch(error => { if (alive) setError(String(error)); });
    schedule();
    return () => {
      alive = false; clearTimeout(indexing);
      window.removeEventListener('meeting-deletion-started', started);
      window.removeEventListener('workspace-meeting-deleted', deleted);
      window.removeEventListener('meeting-notes-committed', schedule);
      window.removeEventListener('meeting-data-changed', schedule);
      window.removeEventListener('enhancement-document-committed', schedule);
      void listener.then(unlisten => unlisten());
    };
  }, [refetchMeetings]);
  if (!pending.length && !error) return null;
  return <aside className="fixed bottom-3 right-3 z-50 max-w-sm rounded border bg-amber-50 p-3 shadow text-sm" aria-label="Pending meeting cleanup"><p>{pending.length ? `${pending.length} meeting deletion${pending.length === 1 ? '' : 's'} pending.` : 'Meeting cleanup could not be checked.'}</p><p className="mt-1 break-words">{error ?? pending[0]?.last_error ?? 'Cleanup will be retried after restart.'}</p><Button size="sm" variant="outline" className="mt-2" onClick={() => { void retry(); }}>Retry cleanup</Button></aside>;
}
