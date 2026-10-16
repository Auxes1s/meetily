'use client';
import { useEffect, useRef, useState } from 'react';
import { Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { meetingExportService } from '@/services/meetingExportService';
import type { MeetingExportDocument, MeetingExportFormat, MeetingExportOptions } from '@/types/meeting-export';

const failureMessage = (failure: unknown) => (failure as { message?: string })?.message ?? String(failure);
const field = 'mt-1.5 block w-full min-w-0 rounded-xl border border-input bg-card px-3 py-2.5 text-sm font-normal text-foreground focus:outline-none focus:ring-2 focus:ring-ring';

export function MeetingExportButton({ meetingId }: { meetingId: string }) {
  const [open, setOpen] = useState(false);
  const [options, setOptions] = useState<MeetingExportOptions | null>(null);
  const [format, setFormat] = useState<MeetingExportFormat>('markdown');
  const [document, setDocument] = useState<MeetingExportDocument>('original');
  const [includeTranscript, setIncludeTranscript] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const owner = useRef(0);
  const saving = useRef(false);
  const loading = useRef<number | null>(null);
  useEffect(() => {
    owner.current++;
    loading.current = null;
    setOpen(false); setOptions(null); setError(null); setNotice(null); setBusy(false);
    return () => { owner.current++; };
  }, [meetingId]);

  const show = async () => {
    if (saving.current || loading.current !== null) return;
    const generation = ++owner.current;
    loading.current = generation;
    setOpen(true); setOptions(null); setError(null); setNotice(null);
    setFormat('markdown'); setIncludeTranscript(false);
    try {
      const data = await meetingExportService.options(meetingId);
      if (generation !== owner.current || data.meeting_id !== meetingId) return;
      setOptions(data); setDocument(data.approved_version_id ? 'approved' : 'original');
    } catch (failure) { if (generation === owner.current) setError(failureMessage(failure)); }
    finally { if (loading.current === generation) loading.current = null; }
  };
  const close = (value: boolean) => {
    if (saving.current) return;
    if (!value) { owner.current++; loading.current = null; }
    setOpen(value);
  };
  const save = async () => {
    if (!options || saving.current) return;
    const generation = owner.current;
    saving.current = true; setBusy(true); setError(null); setNotice(null);
    try {
      const result = await meetingExportService.export({
        meeting_id: meetingId, format, document, include_transcript: includeTranscript,
        expected_approved_version_id: options.approved_version_id,
      });
      if (generation === owner.current) setNotice(result.status === 'saved' ? 'Export saved.' : 'Export cancelled. No file was saved.');
    } catch (failure) { if (generation === owner.current) setError(failureMessage(failure)); }
    finally { saving.current = false; if (generation === owner.current) setBusy(false); }
  };
  return <>
    <Button size="sm" variant="outline" onClick={() => { void show(); }} aria-label="Export meeting"><Download size={15} />Export</Button>
    <Dialog open={open} onOpenChange={close}>
      <DialogContent onEscapeKeyDown={event => { if (busy) event.preventDefault(); }} onInteractOutside={event => { if (busy) event.preventDefault(); }}>
        <DialogHeader><DialogTitle>Export meeting</DialogTitle><DialogDescription>Save readable notes or a complete meeting record to a local file.</DialogDescription></DialogHeader>
        {!options && !error && <p role="status">Loading export choices…</p>}
        {options && <div className="space-y-5">
          <p className="break-words rounded-xl border border-border bg-muted/40 p-3 text-sm font-semibold">{options.title}</p>
          {['recording', 'finalizing', 'interrupted'].includes(options.status) && <p className="text-sm text-amber-800">This export will be labeled incomplete. It includes only saved content available at the snapshot time.</p>}
          <label className="block text-sm font-medium">Format<select aria-label="Export format" className={field} value={format} disabled={busy} onChange={event => setFormat(event.target.value as MeetingExportFormat)}><option value="markdown">Markdown (.md)</option><option value="json">Complete meeting record (.json)</option></select></label>
          {format === 'markdown' ? <>
            <label className="block text-sm font-medium">Notes to include<select aria-label="Notes to include" className={field} value={document} disabled={busy} onChange={event => setDocument(event.target.value as MeetingExportDocument)}>
              <option value="original">My notes</option><option value="approved" disabled={!options.approved_version_id}>Approved enhanced notes</option><option value="both" disabled={!options.approved_version_id}>My notes and approved enhanced notes</option>
            </select></label>
            <label className="flex gap-2 items-center text-sm"><input type="checkbox" className="h-4 w-4 shrink-0 accent-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2" checked={includeTranscript} disabled={busy} onChange={event => setIncludeTranscript(event.target.checked)} />Include full transcript</label>
            <p className="text-xs leading-relaxed text-muted-foreground">Supporting excerpts and timestamps are included when available.</p>
          </> : <p className="text-sm leading-relaxed text-muted-foreground">Includes original notes, the full saved transcript, all document versions and their retained sources. Older summaries remain labeled unverified.</p>}
        </div>}
        {error && <p role="alert" className="text-sm text-red-700">{error}</p>}
        {notice && <p role="status" aria-live="polite" className="rounded-xl border border-primary/20 bg-primary/5 p-3 text-sm text-primary">{notice}</p>}
        <DialogFooter>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => close(false)}>Close</Button>
          {!options && error && <Button size="sm" variant="outline" onClick={() => { void show(); }}>Retry</Button>}
          <Button size="sm" disabled={!options || busy} onClick={() => { void save(); }}>{busy ? 'Saving…' : 'Save export…'}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </>;
}
