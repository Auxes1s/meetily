'use client';
import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { useSearchParams } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { MeetingNotesPanel } from './MeetingNotesPanel';
import { EnhancementPanel } from './EnhancementPanel';
import { flushAllEnhancements } from '@/lib/enhancement-session';
import { noteSessions } from '@/lib/meeting-notes-session';
import { MeetingExportButton } from './MeetingExportButton';

export function MeetingDocuments({ meetingId, onCopyDraft }: { meetingId: string; onCopyDraft: () => Promise<void> }) {
  const params = useSearchParams();
  const [tab, setTab] = useState(['approved', 'enhanced'].includes(params.get('source') ?? '') ? 'enhanced' : 'original');
  const [error, setError] = useState<string | null>(null);
  const [switching, setSwitching] = useState(false);
  const owner = useRef(0);
  const pendingSwitch = useRef<number | null>(null);
  const invalidateSwitch = () => { owner.current++; pendingSwitch.current = null; setSwitching(false); };
  const select = async (next: string) => {
    if (next === tab || pendingSwitch.current !== null) return;
    const generation = ++owner.current;
    pendingSwitch.current = generation;
    setSwitching(true); setError(null);
    try {
      await Promise.all([noteSessions.get(meetingId)?.flush(), flushAllEnhancements()]);
      if (generation === owner.current) setTab(next);
    } catch (error) { if (generation === owner.current) setError((error as Error)?.message ?? String(error)); }
    finally { if (pendingSwitch.current === generation) { pendingSwitch.current = null; setSwitching(false); } }
  };
  const navigateTab = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const next = event.key === 'ArrowRight' ? (index + 1) % 2 : event.key === 'ArrowLeft' ? (index + 1) % 2 : event.key === 'Home' ? 0 : event.key === 'End' ? 1 : null;
    if (next === null || pendingSwitch.current !== null) return;
    event.preventDefault();
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('[role="tab"]')[next]?.focus();
    void select(next === 0 ? 'original' : 'enhanced');
  };
  useEffect(() => {
    const openNote = (event: Event) => {
      if ((event as CustomEvent<{ meetingId: string }>).detail.meetingId === meetingId) { invalidateSwitch(); setTab('original'); }
    };
    window.addEventListener('workspace-note-source', openNote);
    return () => { owner.current++; pendingSwitch.current = null; window.removeEventListener('workspace-note-source', openNote); };
  }, [meetingId]);
  const source = params.get('source');
  useEffect(() => {
    if (source) { invalidateSwitch(); setTab(['approved', 'enhanced'].includes(source) ? 'enhanced' : 'original'); }
  }, [source]);
  return <div className="h-full flex flex-col min-h-0 min-w-0">
    <div className="shrink-0 flex flex-wrap items-center gap-2 px-4 py-2 bg-card">
      <div className="document-tabs" role="tablist" aria-label="Meeting documents">
      <Button id="original-notes-tab" className="document-tab" role="tab" tabIndex={tab === 'original' ? 0 : -1} aria-selected={tab === 'original'} aria-controls="original-notes" variant="ghost" size="sm" disabled={switching} onKeyDown={event => navigateTab(event, 0)} onClick={() => { void select('original'); }}>My notes</Button>
      <Button id="enhanced-notes-tab" className="document-tab" role="tab" tabIndex={tab === 'enhanced' ? 0 : -1} aria-selected={tab === 'enhanced'} aria-controls="enhanced-notes" variant="ghost" size="sm" disabled={switching} onKeyDown={event => navigateTab(event, 1)} onClick={() => { void select('enhanced'); }}>Enhanced notes</Button>
      </div>
      <div className="ml-auto"><MeetingExportButton key={meetingId} meetingId={meetingId} /></div>
      {error && <p role="alert" className="text-sm text-red-700">{error} Unsaved edits are retained.</p>}
    </div>
    <div id="original-notes" role="tabpanel" aria-labelledby="original-notes-tab" hidden={tab !== 'original'} className="flex-1 min-h-0 min-w-0"><MeetingNotesPanel meetingId={meetingId} onCopyDraft={onCopyDraft} /></div>
    <div id="enhanced-notes" role="tabpanel" aria-labelledby="enhanced-notes-tab" hidden={tab !== 'enhanced'} className="flex-1 min-h-0 min-w-0"><EnhancementPanel key={meetingId} meetingId={meetingId} /></div>
  </div>;
}
