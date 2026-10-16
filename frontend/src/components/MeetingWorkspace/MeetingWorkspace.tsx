'use client';
import { useCallback, useEffect, useRef, useState } from 'react';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Plus, PanelRight, ArrowLeft } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { MeetingDocuments } from './MeetingDocuments';
import { noteSessions } from '@/lib/meeting-notes-session';
import { flushWorkspaceDocuments as flushAllNotes } from '@/lib/workspace-save';
import { loadNotesSession } from '@/hooks/useMeetingNotes';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';
import { useRecordingState } from '@/contexts/RecordingStateContext';
import { useRecordingCoordinator } from '@/contexts/RecordingCoordinatorContext';
import { useConfig } from '@/contexts/ConfigContext';
import { RecordingControls } from '@/components/RecordingControls';
import { TranscriptSourceJump } from './TranscriptSourceJump';
import { isMeetingDeletionBlocked } from '@/services/meetingDeletionService';

export function MeetingWorkspace({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const params = useSearchParams();
  const router = useRouter();
  const [homeId, setHomeId] = useState<string | null>(null);
  const [drawer, setDrawer] = useState(false);
  const creating = useRef(false);
  const [isCreating, setIsCreating] = useState(false);
  const toggleButton = useRef<HTMLButtonElement>(null);
  const { refetchMeetings, setCurrentMeeting } = useSidebar();
  const state = useRecordingState();
  const coordinator = useRecordingCoordinator();
  const { selectedDevices } = useConfig();
  const workspaceRoute = pathname === '/' || pathname === '/meeting-details';
  const meetingId = pathname === '/meeting-details' ? params.get('id') : homeId;
  const route = useRef({ pathname, id: params.get('id') });
  route.current = { pathname, id: params.get('id') };
  const sourceId = params.get('source') === 'transcript' ? params.get('sourceId') : null;
  useEffect(() => { if (sourceId) setDrawer(true); }, [sourceId]);
  useEffect(() => {
    const showNotes = (event: Event) => {
      if ((event as CustomEvent<{ meetingId: string }>).detail.meetingId === meetingId) setDrawer(false);
    };
    window.addEventListener('workspace-note-source', showNotes);
    return () => window.removeEventListener('workspace-note-source', showNotes);
  }, [meetingId]);
  const newDraft = useCallback(async (copy = false) => {
    if (creating.current) return;
    creating.current = true;
    setIsCreating(true);
    try {
      const old = meetingId ? noteSessions.get(meetingId)?.getSnapshot() : null;
      if (!copy) await flushAllNotes();
      const draft = await invoke<{ id: string; title: string }>('api_create_meeting_draft', { title: copy && old ? `${old.title} (copy)` : null, timezone: Intl.DateTimeFormat().resolvedOptions().timeZone });
      if (copy && old) {
        const copied = await loadNotesSession(draft.id);
        // A copied draft has no recording session or source ownership.
        const blocks = JSON.parse(JSON.stringify(old.blocks).replaceAll('meetily-note://', 'unavailable-note://'));
        copied.edit(blocks, []);
        await copied.flush();
        if (meetingId) {
          noteSessions.get(meetingId)?.dispose();
          noteSessions.delete(meetingId);
        }
      }
      setHomeId(draft.id);
      sessionStorage.setItem('workspace_meeting_id', draft.id);
      localStorage.setItem('meetily.lastNotesMeeting', draft.id);
      setCurrentMeeting(draft);
      await refetchMeetings();
      router.push('/');
    } catch (error: any) { toast.error(error?.message ?? String(error)); }
    finally { creating.current = false; setIsCreating(false); }
  }, [meetingId, router, refetchMeetings, setCurrentMeeting]);

  useEffect(() => {
    let alive = true;
    let selection = 0;
    const remembered = sessionStorage.getItem('workspace_meeting_id') || localStorage.getItem('meetily.lastNotesMeeting');
    if (remembered && !isMeetingDeletionBlocked(remembered)) invoke('api_get_meeting_metadata', { meetingId: remembered }).then(() => {
      if (alive && selection === 0 && !isMeetingDeletionBlocked(remembered)) setHomeId(remembered);
    }).catch(() => {
      if (alive && selection === 0) {
        sessionStorage.removeItem('workspace_meeting_id'); localStorage.removeItem('meetily.lastNotesMeeting');
      }
    });
    const open = (event: Event) => {
      const id = (event as CustomEvent<string>).detail;
      selection++;
      if (isMeetingDeletionBlocked(id)) return;
      setHomeId(id);
      sessionStorage.setItem('workspace_meeting_id', id);
      localStorage.setItem('meetily.lastNotesMeeting', id);
    };
    window.addEventListener('workspace-open-meeting', open);
    const deleted = (event: Event) => {
      const detail = (event as CustomEvent<string | { meetingId: string }>).detail;
      const id = typeof detail === 'string' ? detail : detail.meetingId;
      selection++;
      setHomeId(value => value === id ? null : value);
      if (route.current.pathname === '/meeting-details' && route.current.id === id) router.push('/');
    };
    const deleting = (event: Event) => deleted(new CustomEvent('workspace-meeting-deleted', { detail: (event as CustomEvent<{ meetingId: string }>).detail }));
    window.addEventListener('workspace-meeting-deleted', deleted);
    window.addEventListener('meeting-deletion-started', deleting);
    return () => { alive = false; window.removeEventListener('workspace-open-meeting', open); window.removeEventListener('workspace-meeting-deleted', deleted); window.removeEventListener('meeting-deletion-started', deleting); };
  }, [router]);
  useEffect(() => {
    if (meetingId && workspaceRoute) {
      sessionStorage.setItem('workspace_meeting_id', meetingId);
      localStorage.setItem('meetily.lastNotesMeeting', meetingId);
    }
    void flushAllNotes().catch(() => toast.error('Unsaved edits are retained. Return to that meeting to retry.'));
  }, [meetingId, workspaceRoute]);
  useEffect(() => {
    let disposed = false;
    const unlisteners: (() => void)[] = [];
    const register = async () => {
      const flush = await listen('workspace-flush-request', async () => {
        try {
          if (state.isRecording || state.isProcessing || state.isSaving) throw new Error('End the active recording and wait for saving before quitting.');
          await flushAllNotes();
          await invoke('confirm_workspace_quit');
        } catch (error: any) {
          await invoke('cancel_workspace_quit');
          toast.error(error?.message ?? 'Notes could not be saved. The app remains open.');
        }
      });
      if (disposed) flush(); else unlisteners.push(flush);
      const timeout = await listen('workspace-quit-timeout', () => toast.error('Quit is waiting for note saves. Retry saving before quitting.'));
      if (disposed) timeout(); else unlisteners.push(timeout);
    };
    void register();
    return () => { disposed = true; unlisteners.forEach(fn => fn()); };
  }, [state.isRecording, state.isProcessing, state.isSaving]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey) || event.isComposing || document.querySelector('[role="dialog"][data-state="open"], [role="alertdialog"][data-state="open"]')) return;
      const k = event.key.toLowerCase();
      if (event.repeat && k === 'n') { event.preventDefault(); return; }
      if (k === 'n' && !event.shiftKey) { event.preventDefault(); void newDraft(); }
      if (k === 's' && !event.shiftKey) { event.preventDefault(); void flushAllNotes().catch(() => toast.error('Notes could not be saved.')); }
      if (k === 't' && event.shiftKey && workspaceRoute) { event.preventDefault(); setDrawer(value => !value); }
      if (k === 'k' && !(event.target as HTMLElement).closest('[contenteditable="true"]')) {
        event.preventDefault();
        const input = document.querySelector<HTMLInputElement>('input[placeholder="Search meeting content..."]');
        if (input) { input.focus(); input.select(); }
        else window.dispatchEvent(new CustomEvent('focus-library-search'));
      }
    };
    window.addEventListener('keydown', key);
    return () => window.removeEventListener('keydown', key);
  }, [newDraft, workspaceRoute]);
  const activeId = typeof window !== 'undefined' ? sessionStorage.getItem('recording_meeting_id') : null;
  if (!workspaceRoute) return <>{children}</>;
  return <div className="meeting-workspace h-full flex flex-col min-h-0 min-w-0 bg-transparent">
    <header className="workspace-header glass-surface shrink-0 flex flex-wrap gap-3 items-center px-5 py-3">
      <Button size="sm" disabled={isCreating} onClick={() => { void newDraft(); }}><Plus size={15} />{isCreating ? 'Creating…' : 'New meeting'}</Button>
      {state.isRecording && activeId && activeId !== meetingId && <Button size="sm" variant="outline" onClick={async () => { await flushAllNotes(); setHomeId(activeId); router.push('/'); }}><ArrowLeft size={15} />Return to recording</Button>}
      <span role="status" className="min-w-0 flex-1 text-xs text-muted-foreground">{state.isRecording ? state.isPaused ? 'Recording paused' : 'Recording' : state.isProcessing ? 'Finishing transcript…' : state.isSaving ? 'Saving recording…' : 'Your meeting workspace'}</span>
      <Button ref={toggleButton} size="sm" variant={drawer ? 'secondary' : 'ghost'} aria-label={`${drawer ? 'Hide' : 'Show'} transcript${pathname === '/meeting-details' ? ' and summary' : ''}`} aria-expanded={drawer} aria-controls="meeting-transcript-panel" onClick={() => { setDrawer(value => !value); toggleButton.current?.focus(); }}><PanelRight size={15} />{drawer ? 'Hide' : 'Show'} transcript{pathname === '/meeting-details' && <span className="workspace-summary-label"> / summary</span>}</Button>
    </header>
    <div className="workspace-panes flex-1 flex min-h-0 min-w-0 overflow-hidden relative" data-transcript-open={drawer}>
      <div className="workspace-notes min-h-0 min-w-0 flex-1">
        {meetingId ? <MeetingDocuments key={meetingId} meetingId={meetingId} onCopyDraft={() => newDraft(true)} /> : <div className="h-full flex flex-col items-center justify-center gap-4 px-6 text-center"><span className="eyebrow">A little space to think</span><h1 className="page-heading">Start with your notes</h1><p className="max-w-sm text-base text-muted-foreground">Write before, during, or without a recording.</p><Button disabled={isCreating} onClick={() => { void newDraft(); }}><Plus size={16} />{isCreating ? 'Creating…' : 'New meeting'}</Button></div>}
      </div>
      <div id="meeting-transcript-panel" aria-label="Meeting transcript and summary" hidden={!drawer} className="workspace-transcript w-full min-h-0 min-w-0 border-l border-border bg-card overflow-hidden">{meetingId && sourceId && <TranscriptSourceJump key={`${meetingId}:${sourceId}`} meetingId={meetingId} transcriptId={sourceId} />}{children}</div>
    </div>
    {meetingId && <footer className="workspace-recording shrink-0 max-h-[35%] overflow-auto p-3 flex flex-col items-center gap-2">
      <RecordingControls isRecording={state.isRecording} onRecordingStart={() => coordinator.handleRecordingStart(meetingId)} onRecordingStop={(callApi = true, sessionId) => coordinator.handleRecordingStop(callApi, sessionId)} onTranscriptReceived={() => {}} onStopInitiated={() => coordinator.setIsStopping(true)} barHeights={['12px', '22px', '12px']} onTranscriptionError={message => coordinator.showModal('errorAlert', message)} isRecordingDisabled={coordinator.isRecordingDisabled || (!!activeId && activeId !== meetingId && state.isRecording)} isParentProcessing={state.isProcessing || state.isSaving} selectedDevices={selectedDevices} meetingName={noteSessions.get(meetingId)?.getSnapshot().title ?? 'Untitled meeting'} />
      <p className="text-[11px] text-muted-foreground">Recording needs a local transcription model. Notes save independently.</p>
    </footer>}
  </div>;
}
