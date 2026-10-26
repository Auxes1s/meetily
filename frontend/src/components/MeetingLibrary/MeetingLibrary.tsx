'use client';
import { useEffect, useMemo, useRef, useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, Search, RefreshCw, Trash2, Tag, Users, ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { useMeetingLibrary } from '@/hooks/useMeetingLibrary';
import { useRecordingControls } from '@/contexts/RecordingStateContext';
import { localDateBound, meetingSourceUrl, passageTime, sourceLabels } from '@/lib/meeting-library';
import type { LibraryMeeting, LibraryMeetingStatus, LibrarySearchRequest, LibrarySort } from '@/types/meeting-library';
import { deleteMeetingManaged, meetingDeletionService, type DeletionStatus } from '@/services/meetingDeletionService';
import { MeetingMetadataEditor } from './MeetingMetadataEditor';
import { MeetingExportButton } from '@/components/MeetingWorkspace/MeetingExportButton';

const field = 'min-w-0 rounded-xl border border-input bg-card px-3 py-2.5 text-sm text-foreground shadow-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:ring-offset-card';
const failureMessage = (failure: unknown) => failure instanceof Error ? failure.message : String(failure);

function meetingDate(meeting: LibraryMeeting) {
  const value = meeting.started_at ?? meeting.created_at;
  const date = new Date(value);
  return Number.isFinite(date.getTime()) ? date.toLocaleString() : 'Date unavailable';
}

function activeRecordingMeeting(): string | null {
  try { return typeof window === 'undefined' ? null : window.sessionStorage.getItem('recording_meeting_id'); }
  catch { return null; }
}

export function MeetingLibrary() {
  const [query, setQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [participant, setParticipant] = useState('');
  const [tag, setTag] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [status, setStatus] = useState<LibraryMeetingStatus | ''>('');
  const [sort, setSort] = useState<LibrarySort>('date_desc');
  const [cursors, setCursors] = useState<(string | null)[]>([null]);
  const [metadataMeeting, setMetadataMeeting] = useState<string | null>(null);
  const [metadataSaving, setMetadataSaving] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<LibraryMeeting | null>(null);
  const deleteTargetRef = useRef<LibraryMeeting | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [pendingDeletion, setPendingDeletion] = useState<DeletionStatus[]>([]);
  const [cleanupBusy, setCleanupBusy] = useState<Set<string>>(new Set());
  const deletingRef = useRef(false);
  const cleanupInFlight = useRef(new Set<string>());
  const metadataSavingRef = useRef<string | null>(null);
  const pendingReadGeneration = useRef(0);
  const mounted = useRef(false);
  const input = useRef<HTMLInputElement>(null);
  const recording = useRecordingControls();
  const activeMeetingId = activeRecordingMeeting();

  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => { setDebouncedQuery(query); setCursors([null]); }, 250);
    return () => clearTimeout(timer);
  }, [query]);
  useEffect(() => {
    const focus = () => { input.current?.focus(); input.current?.select(); };
    window.addEventListener('focus-library-search', focus);
    return () => window.removeEventListener('focus-library-search', focus);
  }, []);
  const request: LibrarySearchRequest = useMemo(() => ({
    query: debouncedQuery,
    participant: participant.trim() || null,
    tag: tag.trim() || null,
    date_from: localDateBound(dateFrom),
    date_to: localDateBound(dateTo, true),
    status: status || null, sort, limit: 30, cursor: cursors[cursors.length - 1],
  }), [debouncedQuery, participant, tag, dateFrom, dateTo, status, sort, cursors]);
  const { page, loading, error, refresh } = useMeetingLibrary(request);

  useEffect(() => {
    let live = true;
    const readPending = () => {
      const owner = ++pendingReadGeneration.current;
      meetingDeletionService.listPending().then(jobs => { if (live && owner === pendingReadGeneration.current) setPendingDeletion(jobs); })
        .catch(failure => { if (live && owner === pendingReadGeneration.current) setActionError(`Pending cleanup could not be checked: ${failureMessage(failure)}`); });
    };
    const changed = () => { readPending(); refresh(); };
    readPending();
    window.addEventListener('meeting-deletion-started', changed);
    window.addEventListener('workspace-meeting-deleted', changed);
    return () => {
      live = false;
      window.removeEventListener('meeting-deletion-started', changed);
      window.removeEventListener('workspace-meeting-deleted', changed);
    };
  }, [refresh]);

  const updateFilter = (apply: () => void) => { apply(); setCursors([null]); };
  const rememberPending = (job: DeletionStatus) => {
    // A scan begun before this action completed must not resurrect its old status.
    pendingReadGeneration.current++;
    if (!mounted.current) return;
    setPendingDeletion(items => job.state === 'pending'
      ? [...items.filter(item => item.job_id !== job.job_id), job]
      : items.filter(item => item.job_id !== job.job_id));
  };
  const deleteSelected = async () => {
    const target = deleteTargetRef.current;
    if (!target || target.id !== deleteTarget?.id || deletingRef.current || metadataSavingRef.current === target.id || ['recording', 'finalizing'].includes(target.status) || (recording.isRecording && activeMeetingId === target.id)) return;
    deletingRef.current = true;
    const targetId = target.id;
    setDeleting(true); setActionError(null);
    try {
      const job = await deleteMeetingManaged(targetId);
      if (!mounted.current) return;
      rememberPending(job);
      if (deleteTargetRef.current?.id === targetId) { deleteTargetRef.current = null; setDeleteTarget(null); }
    } catch (failure) {
      if (!mounted.current) return;
      const job = (failure as { status?: DeletionStatus })?.status;
      if (job) {
        rememberPending(job);
        if (deleteTargetRef.current?.id === targetId) { deleteTargetRef.current = null; setDeleteTarget(null); }
      }
      setActionError(failureMessage(failure));
    } finally {
      deletingRef.current = false;
      if (mounted.current) { setDeleting(false); setCursors([null]); refresh(); }
    }
  };
  const retryCleanup = async (jobId: string) => {
    if (cleanupInFlight.current.has(jobId)) return;
    cleanupInFlight.current.add(jobId);
    setCleanupBusy(new Set(cleanupInFlight.current)); setActionError(null);
    try {
      const job = await meetingDeletionService.retry(jobId);
      if (mounted.current) { rememberPending(job); refresh(); }
    } catch (failure) {
      if (!mounted.current) return;
      const next = (failure as { status?: DeletionStatus })?.status;
      if (next) rememberPending(next);
      setActionError(failureMessage(failure));
    } finally {
      cleanupInFlight.current.delete(jobId);
      if (mounted.current) setCleanupBusy(new Set(cleanupInFlight.current));
    }
  };
  const closeDeletion = () => {
    if (!deletingRef.current) { deleteTargetRef.current = null; setDeleteTarget(null); }
  };

  return <div className="meeting-library h-full min-h-0 min-w-0 overflow-auto bg-background p-4 sm:p-6 lg:p-8">
    <div className="mx-auto max-w-5xl space-y-6">
      <header className="space-y-5">
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="ghost" size="sm" asChild><Link href="/"><ArrowLeft />Back to notes</Link></Button>
          <div className="ml-auto flex flex-wrap gap-2">
            {recording.isRecording && activeMeetingId && <Button size="sm" variant="outline" asChild><Link href={meetingSourceUrl(activeMeetingId)}>Return to recording</Link></Button>}
            <Button size="sm" variant="outline" onClick={refresh} disabled={loading}><RefreshCw className={loading ? 'animate-spin motion-reduce:animate-none' : ''} />Refresh</Button>
          </div>
        </div>
        <div><p className="eyebrow mb-2">Your workspace</p><h1 className="page-heading">Meeting library</h1></div>
      </header>
      <section aria-label="Search and filter meetings" className="paper-surface space-y-4 rounded-2xl border border-border p-4 sm:p-5">
        <label className="block text-sm font-semibold" htmlFor="library-search">Search all meeting content</label>
        <div className="relative"><Search aria-hidden="true" className="pointer-events-none absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" /><input ref={input} id="library-search" type="search" placeholder="Search meeting content..." maxLength={512} value={query} onChange={event => setQuery(event.target.value)} className={`${field} w-full pl-10`} /></div>
        <p className="text-xs leading-relaxed text-muted-foreground">All terms must appear somewhere in a meeting. Use quotes for a phrase within one source. Candidates and previous versions are excluded.</p>
        <div className="library-filters grid gap-3 border-t border-border pt-4">
          <label className="text-xs font-medium">Participant<input value={participant} maxLength={120} onChange={event => updateFilter(() => setParticipant(event.target.value))} placeholder="Exact manual name" className={`${field} mt-1 w-full`} /></label>
          <label className="text-xs font-medium">Tag<input value={tag} maxLength={80} onChange={event => updateFilter(() => setTag(event.target.value))} placeholder="Exact tag" className={`${field} mt-1 w-full`} /></label>
          <label className="text-xs font-medium">Status<select value={status} onChange={event => updateFilter(() => setStatus(event.target.value as LibraryMeetingStatus | ''))} className={`${field} mt-1 w-full`}><option value="">All statuses</option>{['draft', 'recording', 'finalizing', 'completed', 'interrupted'].map(value => <option key={value} value={value}>{value[0].toUpperCase() + value.slice(1)}</option>)}</select></label>
          <label className="text-xs font-medium">From date<input type="date" value={dateFrom} onChange={event => updateFilter(() => setDateFrom(event.target.value))} className={`${field} mt-1 w-full`} /></label>
          <label className="text-xs font-medium">Through date<input type="date" value={dateTo} onChange={event => updateFilter(() => setDateTo(event.target.value))} className={`${field} mt-1 w-full`} /></label>
          <label className="text-xs font-medium">Sort<select value={sort} onChange={event => updateFilter(() => setSort(event.target.value as LibrarySort))} className={`${field} mt-1 w-full`}><option value="date_desc">Newest meeting</option><option value="date_asc">Oldest meeting</option><option value="updated_desc">Recently edited</option><option value="title_asc">Title</option><option value="relevance">Most matching sources</option></select></label>
        </div>
        <p className="text-xs text-muted-foreground">Date filters use your local calendar days. Capture start is used when known; older records use their labeled creation date.</p>
      </section>

      {pendingDeletion.length > 0 && <section aria-label="Pending meeting cleanup" className="space-y-2 rounded-lg border border-amber-300 bg-amber-50 p-4">
        <h2 className="font-medium">Meeting cleanup pending</h2>
        <p className="text-sm">These meetings are hidden while associated managed data is removed.</p>
        {pendingDeletion.map(job => <div key={job.job_id} className="flex flex-wrap items-center gap-2 text-sm">
          <span className="flex-1">{job.last_error ?? 'Cleanup has not finished.'}</span>
          <Button variant="outline" size="sm" disabled={cleanupBusy.has(job.job_id)} onClick={() => { void retryCleanup(job.job_id); }}>{cleanupBusy.has(job.job_id) ? 'Retrying…' : 'Retry cleanup'}</Button>
        </div>)}
      </section>}
      {(error || actionError) && <p role="alert" className="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700">{actionError ?? error}</p>}
      <div role="status" aria-live="polite" className="flex flex-wrap items-center gap-y-1 px-1 text-sm text-muted-foreground">
        {loading ? 'Loading meetings…' : `${page?.total_count ?? 0} meeting${page?.total_count === 1 ? '' : 's'}`}
        {page?.index_updating && <span className="ml-2 text-amber-700">Search updating ({page.pending_index_count} pending). Results may change until indexing completes.</span>}
      </div>
      <section aria-label="Meeting results" className="space-y-3">
        {page?.meetings.map(meeting => <article key={meeting.id} className="paper-surface space-y-4 rounded-2xl border border-border p-4 sm:p-5">
          <div className="library-result-header flex flex-wrap items-start gap-2">
            <div className="library-result-title min-w-0 flex-1"><Link href={meetingSourceUrl(meeting.id)} className="rounded-sm break-words text-lg font-semibold leading-snug tracking-tight text-foreground underline-offset-4 hover:text-primary hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">{meeting.title}</Link>
              <p className="mt-1.5 text-xs leading-relaxed text-muted-foreground">{meetingDate(meeting)} · {meeting.date_is_creation_fallback ? 'Creation date (capture time unknown)' : 'Capture start'}{meeting.timezone ? ` · ${meeting.timezone}` : ''} · {meeting.status}</p>
            </div>
            <Button size="sm" variant="outline" disabled={metadataSaving !== null} onClick={() => { if (!metadataSavingRef.current) setMetadataMeeting(current => current === meeting.id ? null : meeting.id); }}>Metadata</Button>
            <MeetingExportButton meetingId={meeting.id} />
            <Button size="icon" variant="ghost" aria-label={`Delete ${meeting.title}`} disabled={deleting || metadataSaving === meeting.id || ['recording', 'finalizing'].includes(meeting.status) || (recording.isRecording && activeMeetingId === meeting.id)} onClick={() => { if (!deletingRef.current && metadataSavingRef.current !== meeting.id && !['recording', 'finalizing'].includes(meeting.status) && !(recording.isRecording && activeMeetingId === meeting.id)) { deleteTargetRef.current = meeting; setDeleteTarget(meeting); setActionError(null); } }}><Trash2 className="text-red-600" /></Button>
          </div>
          <div className="flex flex-wrap gap-x-4 gap-y-2 text-xs leading-relaxed text-muted-foreground">
            <span className="flex min-w-0 items-start gap-1.5 break-words"><Users size={13} className="mt-0.5 shrink-0" aria-hidden="true" />{meeting.participants.length ? meeting.participants.join(', ') : 'Participants not entered'}</span>
            <span className="flex min-w-0 items-start gap-1.5 break-words"><Tag size={13} className="mt-0.5 shrink-0" aria-hidden="true" />{meeting.tags.length ? meeting.tags.join(', ') : 'No tags'}</span>
          </div>
          {meeting.index_updating && <p className="text-xs text-amber-700">This meeting has newer saved content awaiting search indexing.</p>}
          {metadataMeeting === meeting.id && <MeetingMetadataEditor key={meeting.id} meetingId={meeting.id} participants={meeting.participants} tags={meeting.tags} onSavingChange={saving => {
            if (saving) metadataSavingRef.current = meeting.id;
            else if (metadataSavingRef.current === meeting.id) metadataSavingRef.current = null;
            if (mounted.current) setMetadataSaving(metadataSavingRef.current);
          }} onSaved={() => { setMetadataMeeting(current => current === meeting.id ? null : current); refresh(); }} onCancel={() => { if (!metadataSavingRef.current) setMetadataMeeting(current => current === meeting.id ? null : current); }} />}
          {meeting.hits.length > 0 && <ul className="space-y-2 border-t border-border pt-3">{meeting.hits.map(hit => <li key={`${hit.source_kind}:${hit.source_id}`}>
            <Link href={meetingSourceUrl(meeting.id, hit)} className="block rounded-xl bg-muted/30 p-3 transition-colors hover:bg-secondary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
              <span className="text-xs font-semibold text-primary">{sourceLabels[hit.source_kind]}{hit.source_kind === 'transcript' ? ` · ${passageTime(hit.audio_start_time)}` : ''}</span>
              <p className="mt-1.5 break-words whitespace-pre-wrap text-sm leading-relaxed text-foreground">{hit.snippet}</p>
            </Link>
          </li>)}</ul>}
        </article>)}
        {!loading && page?.meetings.length === 0 && <div className="paper-surface rounded-2xl border border-dashed border-border px-6 py-12 text-center text-muted-foreground"><Search aria-hidden="true" className="mx-auto mb-3 h-6 w-6 text-primary" /><p className="text-sm">No meetings match these filters.</p></div>}
      </section>
      <nav aria-label="Meeting result pages" className="flex flex-wrap gap-2 items-center justify-between pb-6">
        <Button variant="outline" disabled={loading || cursors.length === 1} onClick={() => setCursors(items => items.slice(0, -1))}><ChevronLeft />Previous</Button>
        <span className="text-sm text-muted-foreground">Page {cursors.length}</span>
        <Button variant="outline" disabled={loading || !page?.next_cursor} onClick={() => { if (page?.next_cursor) setCursors(items => [...items, page.next_cursor]); }} >Next<ChevronRight /></Button>
      </nav>
    </div>
    <Dialog open={deleteTarget !== null} onOpenChange={open => { if (!open) closeDeletion(); }}>
      <DialogContent role="alertdialog" className="max-w-md" onEscapeKeyDown={event => { if (deletingRef.current) event.preventDefault(); }} onInteractOutside={event => { if (deletingRef.current) event.preventDefault(); }}>
        <DialogHeader><DialogTitle className="break-words">Delete “{deleteTarget?.title}”?</DialogTitle><DialogDescription>Remove this meeting, its notes, transcripts, managed recordings and recovery copies. Imported original audio and exported copies remain in their locations.</DialogDescription></DialogHeader>
        <DialogFooter><Button variant="outline" disabled={deleting} onClick={closeDeletion}>Cancel</Button><Button variant="destructive" disabled={deleting} onClick={() => void deleteSelected()}>{deleting ? 'Deleting…' : 'Delete meeting'}</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  </div>;
}
