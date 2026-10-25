'use client';
import { Component, useCallback, useEffect, useRef, useState } from 'react';
import dynamic from 'next/dynamic';
import type { Block, BlockNoteEditor } from '@blocknote/core';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import { Clock, Download, MoreHorizontal, Save } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { useMeetingNotes } from '@/hooks/useMeetingNotes';

const Editor = dynamic(() => import('@/components/BlockNoteEditor/Editor'), { ssr: false });
class NotesEditorBoundary extends Component<{ children: React.ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  render() { return this.state.failed ? <p role="alert" className="p-6">The saved blocks could not be displayed. Original data is preserved.</p> : this.props.children; }
}
export function MeetingNotesPanel({ meetingId, onCopyDraft }: { meetingId: string; onCopyDraft: () => Promise<void> }) {
  const { session, snapshot, error, reload } = useMeetingNotes(meetingId);
  const editorRef = useRef<BlockNoteEditor | null>(null);
  const timestampPending = useRef(false);
  const exportPending = useRef(false);
  const [exporting, setExporting] = useState(false);
  const [source, setSource] = useState<{ text: string; audio_start_time: number; recording_session_id?: string; recording_folder?: string }[] | null>(null);
  useEffect(() => {
    const openSource = (event: Event) => {
      const detail = (event as CustomEvent<{ meetingId: string; blockId: string }>).detail;
      if (detail.meetingId !== meetingId) return;
      requestAnimationFrame(() => {
        try {
          const editor = editorRef.current;
          if (!editor?.getBlock(detail.blockId)) { toast.info('This block has changed. The saved supporting excerpt is retained.'); return; }
          editor.setTextCursorPosition(detail.blockId, 'start');
          editor.focus();
          document.querySelector(`[data-id="${CSS.escape(detail.blockId)}"]`)?.scrollIntoView({ block: 'center' });
        } catch { toast.info('The saved supporting excerpt is retained.'); }
      });
    };
    window.addEventListener('workspace-note-source', openSource);
    return () => window.removeEventListener('workspace-note-source', openSource);
  }, [meetingId]);
  useEffect(() => { editorRef.current = null; setSource(null); }, [meetingId]);
  const onReady = useCallback((editor: BlockNoteEditor) => {
    editorRef.current = editor;
    const detach = session?.setConverter(blocks => editor.blocksToMarkdownLossy(blocks));
    return () => { detach?.(); if (editorRef.current === editor) editorRef.current = null; };
  }, [session]);
  const onChange = useCallback((blocks: Block[]) => session?.edit(blocks), [session]);
  const save = async () => { try { await session?.flush(); } catch { toast.error('Notes could not be saved. Your edits are retained.'); } };
  const insertTimestamp = async () => {
    const editor = editorRef.current;
    if (!editor || !session || timestampPending.current) return;
    timestampPending.current = true;
    try {
      const state = await invoke<{ is_recording: boolean; meeting_id?: string; recording_session_id?: string; audio_position?: number; is_paused: boolean; timeline_offset?: number }>('get_recording_state');
      if (!state.is_recording || state.meeting_id !== meetingId || !state.recording_session_id || state.audio_position == null) {
        toast.info('Timestamp unavailable: this meeting has no active recording.');
        return;
      }
      const id = crypto.randomUUID();
      const seconds = Math.floor(state.audio_position + (state.timeline_offset ?? 0));
      const label = `[${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}]`;
      const blockId = editor.getTextCursorPosition().block.id;
      editor.insertInlineContent([{ type: 'link', href: `meetily-note://${id}`, content: [{ type: 'text', text: label, styles: {} }] }]);
      session.edit(editor.document, [...session.getSnapshot().links, { id, block_id: blockId, recording_session_id: state.recording_session_id, recording_offset: state.audio_position, transcript_id: null }]);
      editor.focus();
    } catch { toast.error('Timestamp unavailable.'); }
    finally { timestampPending.current = false; }
  };
  const openMarker = async (linkId: string) => {
    try {
      await session?.flush();
      const passages = await invoke<{ text: string; audio_start_time: number; recording_session_id?: string; recording_folder?: string }[]>('api_get_note_passage', { meetingId, linkId });
      setSource(passages);
    } catch { toast.error('The timestamp source could not be loaded.'); }
  };
  const exportNotes = async (format: 'markdown' | 'json') => {
    if (exportPending.current) return;
    exportPending.current = true;
    setExporting(true);
    try {
      await session?.flush();
      await invoke('api_export_meeting_notes', { meetingId, format });
    } catch (error: any) { toast.error(error?.message ?? String(error)); }
    finally { exportPending.current = false; setExporting(false); }
  };
  if (error) return <div role="alert" className="p-6">Notes could not be opened. Existing data is preserved.<p className="text-sm mt-2">{error}</p><Button onClick={() => { void reload().catch(() => toast.error('Save other pending notes before reloading.')); }}>Retry</Button></div>;
  if (!session || !snapshot) return <p className="p-6" role="status">Loading notes…</p>;
  return <section aria-label="My notes" className="notes-panel flex flex-col h-full min-h-0 min-w-0 bg-card">
    <div className="notes-toolbar shrink-0 flex flex-wrap items-center gap-2">
      <input aria-label="Meeting title" className="notes-title flex-1 min-w-0 bg-transparent outline-none focus:ring-2 focus:ring-ring rounded px-1" value={snapshot.title} onChange={e => session.setTitle(e.target.value)} />
      <span role="status" aria-live="polite" className={`text-xs ${snapshot.status === 'Saved' ? 'text-muted-foreground' : 'text-amber-700'}`}>{snapshot.status}</span>
      <Button variant="ghost" size="sm" aria-label="Insert recording timestamp" title="Insert recording timestamp" onPointerDown={e => e.preventDefault()} onClick={insertTimestamp}><Clock size={15} /><span className="notes-action-label">Timestamp</span></Button>
      <Button variant="ghost" size="sm" aria-label="Save notes" onClick={save} title="Save notes (⌘/Ctrl+S)"><Save size={15} /><span className="notes-action-label">Save</span></Button>
      <DropdownMenu>
        <DropdownMenuTrigger asChild><Button variant="ghost" size="sm" disabled={exporting} aria-label="More note actions" title="More note actions"><MoreHorizontal size={16} /></Button></DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuLabel>Export original notes</DropdownMenuLabel>
          <DropdownMenuItem onSelect={() => { void exportNotes('markdown'); }}><Download size={15} />Markdown</DropdownMenuItem>
          <DropdownMenuItem onSelect={() => { void exportNotes('json'); }}><Download size={15} />JSON</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
    {snapshot.error && <div role="alert" className="bg-amber-50 border-b px-4 py-2 text-sm text-amber-900">{snapshot.error}
      {snapshot.status === 'Conflict' && <div className="flex gap-2 mt-2"><Button size="sm" onClick={onCopyDraft}>Save as new draft</Button><Button size="sm" variant="outline" onClick={() => { if (window.confirm('Reload the saved revision and discard these local edits?')) { void reload().catch(() => toast.error('Save other pending notes before reloading.')); } }}>Reload saved revision</Button></div>}
    </div>}
    {source && <aside aria-label="Timestamp source" className="max-h-[40%] overflow-auto shrink-0 p-4 bg-accent border-b text-sm"><Button variant="ghost" size="sm" onClick={() => setSource(null)}>Close source</Button>{source[0]?.recording_folder && source[0]?.recording_session_id && <Button variant="ghost" size="sm" onClick={() => { void invoke('api_open_recording_session', { meetingId, recordingSessionId: source[0].recording_session_id }).catch(error => toast.error(String(error))); }}>Open recording</Button>}{source.length ? source.map((passage, i) => <p key={i} className="mt-2 whitespace-pre-wrap">{passage.text}</p>) : <p>No matching passage is available. The timestamp remains saved.</p>}</aside>}
    <div className="notes-scroll flex-1 overflow-auto py-6 min-h-0 min-w-0">
      <NotesEditorBoundary key={meetingId}><Editor key={meetingId} initialContent={snapshot.blocks} onChange={onChange} onReady={onReady} onMarkerClick={openMarker} /></NotesEditorBoundary>
    </div>
  </section>;
}
