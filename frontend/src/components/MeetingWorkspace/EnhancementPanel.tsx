'use client';

import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from 'react';
import dynamic from 'next/dynamic';
import { invoke } from '@tauri-apps/api/core';
import { useSearchParams } from 'next/navigation';
import type { Block } from '@blocknote/core';
import { Button } from '@/components/ui/button';
import { useConfig } from '@/contexts/ConfigContext';
import { useRecordingState } from '@/contexts/RecordingStateContext';
import { loadNotesSession } from '@/hooks/useMeetingNotes';
import { MeetingNotesSession } from '@/lib/meeting-notes-session';
import { createEnhancementSession, enhancementSessions } from '@/lib/enhancement-session';
import { enhancementMarkdown, parseEnhancementDocument, type EnhancementDocument, type EnhancementVersion, type GenerationProgress, type MeetingEnhancements } from '@/types/enhancement';
import { readMeetingSummaryLanguage } from '@/lib/summary-language-preferences';
import { CustomEnhancementTemplate } from './CustomEnhancementTemplate';
import { ClaimSourcePanel } from './ClaimSourcePanel';
import { isMeetingDeletionBlocked } from '@/services/meetingDeletionService';
import { flushAllLegacySummaries } from '@/lib/legacy-summary-flush';

const Editor = dynamic(() => import('@/components/BlockNoteEditor/Editor'), { ssr: false });
const message = (error: unknown) => (error as { message?: string })?.message ?? String(error);
const field = 'mt-1.5 block w-full min-w-0 rounded-xl border border-input bg-card px-3 py-2.5 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring';

function VersionEditor({ version, locked, onReload, onEdited }: { version: EnhancementVersion; locked: boolean; onReload: () => void; onEdited: (versionId: string) => void }) {
  const [session, setSession] = useState<MeetingNotesSession | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let stale = false;
    const load = async () => {
      if (isMeetingDeletionBlocked(version.meeting_id)) return;
      let entry = enhancementSessions.get(version.id);
      if (!entry) {
        const document = parseEnhancementDocument(version.document_json);
        let blocks = document.summary_json;
        if (!blocks) {
          const { BlockNoteEditor } = await import('@blocknote/core');
          const parser = BlockNoteEditor.create();
          blocks = await parser.tryParseMarkdownToBlocks(enhancementMarkdown(document));
        }
        if (!Array.isArray(blocks)) throw new Error('Stored enhanced document is malformed; its original is preserved.');
        if (stale || isMeetingDeletionBlocked(version.meeting_id)) return;
        // Parsing is asynchronous; another mounted view may already own this version.
        entry = enhancementSessions.get(version.id);
        if (!entry) {
          const created = createEnhancementSession(version, blocks, input => invoke('api_save_enhancement_version', { input }));
          entry = { meetingId: version.meeting_id, session: created };
          enhancementSessions.set(version.id, entry);
        }
      }
      if (!stale) setSession(entry.session);
    };
    void load().catch(error => { if (!stale) setError(message(error)); });
    return () => { stale = true; };
  }, [version.id, version.meeting_id]);
  const snapshot = useSyncExternalStore(session?.subscribe ?? (() => () => {}), session?.getSnapshot ?? (() => null), () => null);
  useEffect(() => { if (snapshot && snapshot.status !== 'Saved') onEdited(version.id); }, [snapshot?.status, version.id, onEdited]);
  if (error && !session) return <p role="alert" className="p-4 text-red-700">{error}</p>;
  if (!session || !snapshot) return <p className="p-4">Loading enhanced notes…</p>;
  return <div>
    <div className="glass-surface flex flex-wrap items-center gap-2 border-b border-border px-4 py-3 text-sm">
      <span role="status" aria-live="polite">{snapshot.status}</span>
      <Button size="sm" variant="outline" onClick={() => { void session.flush().catch(error => setError(message(error))); }}>Save edits</Button>
      <span className="text-muted-foreground">Edited text requires your review; original sources stay preserved.</span>
      {snapshot.error && <span role="alert" className="text-red-700">{snapshot.error}</span>}
      {error && <span role="alert" className="text-red-700">{error}</span>}
      {snapshot.status === 'Conflict' && <Button size="sm" variant="outline" disabled={locked} onClick={() => { if (window.confirm('Reload the saved version and discard these local edits?')) onReload(); }}>Reload saved version</Button>}
    </div>
    <Editor initialContent={snapshot.blocks} editable={!locked && version.state !== 'archived' && ['ready', 'unverified'].includes(version.status)} onChange={blocks => session.edit(blocks)} onReady={editor => session.setConverter(blocks => editor.blocksToMarkdownLossy(blocks))} />
  </div>;
}

export function EnhancementPanel({ meetingId }: { meetingId: string }) {
  const params = useSearchParams();
  const requestedVersion = params.get('source') === 'approved' ? params.get('sourceId') : null;
  const { modelConfig, isModelConfigLoading } = useConfig();
  const recording = useRecordingState();
  const [progress, setProgress] = useState<Record<string, GenerationProgress | null>>({});
  const [record, setRecord] = useState<MeetingEnhancements | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [templates, setTemplates] = useState<Array<{ id: string; name: string }>>([]);
  const [templateId, setTemplateId] = useState('standard_meeting');
  const [instructions, setInstructions] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [remote, setRemote] = useState<{ destination: string; payload_scope: string } | null>(null);
  const [showCustom, setShowCustom] = useState(false);
  const [inspectSources, setInspectSources] = useState(false);
  const [editorEpoch, setEditorEpoch] = useState(0);
  const [editedIds, setEditedIds] = useState(new Set<string>());
  const markEdited = useCallback((id: string) => setEditedIds(ids => ids.has(id) ? ids : new Set([...ids, id])), []);
  const owner = useRef(0);
  const actionInFlight = useRef(false);
  const settingsLoaded = useRef<string | null>(null);
  const valid = record?.meeting_id === meetingId ? record : null;
  const refresh = useCallback(async () => {
    const epoch = owner.current;
    const data = await invoke<MeetingEnhancements>('api_get_meeting_enhancements', { meetingId });
    if (epoch !== owner.current || data.meeting_id !== meetingId) return;
    setRecord(data);
    setSelected(previous => previous && data.versions.some(version => version.id === previous)
      ? previous : data.versions.find(version => version.id === requestedVersion)?.id ?? data.approved_version_id ?? data.versions.find(version => version.state === 'candidate')?.id ?? data.versions[0]?.id ?? null);
    if (settingsLoaded.current !== meetingId) {
      settingsLoaded.current = meetingId;
      setTemplateId(data.settings.template_id);
      setInstructions(data.settings.instructions);
    }
  }, [meetingId, requestedVersion]);
  const fetchTemplates = useCallback(async () => {
    const data = await invoke<Array<{ id: string; name: string }>>('api_list_templates');
    setTemplates(data);
  }, []);
  useEffect(() => {
    owner.current++;
    settingsLoaded.current = null;
    setRecord(null); setSelected(null); setError(null); setRemote(null);
    void refresh().catch(error => setError(message(error)));
    void fetchTemplates().catch(error => setError(message(error)));
    return () => { owner.current++; };
  }, [refresh, fetchTemplates]);
  useEffect(() => {
    const changed = () => { void refresh().catch(error => setError(message(error))); };
    const committed = (event: Event) => {
      const detail = (event as CustomEvent<{ meetingId: string; versionId: string; document: EnhancementDocument; revision: number; updatedAt: string }>).detail;
      if (detail.meetingId !== meetingId || isMeetingDeletionBlocked(meetingId)) return;
      setRecord(record => record ? { ...record, versions: record.versions.map(version => version.id === detail.versionId ? {
        ...version, document_json: JSON.stringify(detail.document), revision: detail.revision, updated_at: detail.updatedAt,
      } : version) } : record);
    };
    window.addEventListener('meeting-data-changed', changed);
    window.addEventListener('enhancement-document-committed', committed);
    return () => { window.removeEventListener('meeting-data-changed', changed); window.removeEventListener('enhancement-document-committed', committed); };
  }, [refresh, meetingId]);
  const generating = valid?.versions.some(version => version.status === 'generating');
  const generatingIds = valid?.versions.filter(version => version.status === 'generating').map(version => version.id).join(',') ?? '';
  useEffect(() => {
    if (!generatingIds) return;
    let alive = true;
    let inFlight = false;
    const interval = setInterval(() => {
      if (inFlight) return;
      inFlight = true;
      void Promise.all(generatingIds.split(',').map(versionId => invoke<{ id: string; status: string; generation_progress?: GenerationProgress | null }>('api_get_enhancement_status', { meetingId, versionId })))
        .then(statuses => { if (alive) setProgress(previous => ({ ...previous, ...Object.fromEntries(statuses.map(status => [status.id, status.generation_progress ?? null])) })); if (alive && statuses.some(version => version.status !== 'generating')) return refresh(); })
        .catch(error => { if (alive) setError(message(error)); })
        .finally(() => { inFlight = false; });
    }, 1500);
    return () => { alive = false; clearInterval(interval); };
  }, [generatingIds, meetingId, refresh]);
  const current = valid?.versions.find(version => version.id === selected);
  useEffect(() => {
    if (!current || current.status === 'generating') return;
    let alive = true;
    void invoke<{ generation_progress?: GenerationProgress | null }>('api_get_enhancement_status', { meetingId, versionId: current.id })
      .then(status => { if (alive) setProgress(previous => ({ ...previous, [current.id]: status.generation_progress ?? null })); })
      .catch(() => {});
    return () => { alive = false; };
  }, [meetingId, current?.id, current?.status]);
  const action = async (work: () => Promise<void>) => {
    if (actionInFlight.current) return;
    actionInFlight.current = true;
    const epoch = owner.current;
    setBusy(true); setError(null);
    try { await work(); if (epoch === owner.current) await refresh(); }
    catch (error) { if (epoch === owner.current) setError(message(error)); }
    finally { actionInFlight.current = false; if (epoch === owner.current) setBusy(false); }
  };
  const flushVersion = async () => {
    if (current) await enhancementSessions.get(current.id)?.session.flush();
    return current ? enhancementSessions.get(current.id)?.session.getSnapshot().revision ?? current.revision : 0;
  };
  const generate = (remoteDestination?: string) => action(async () => {
    if (!valid) throw new Error('Meeting enhancement settings have not loaded.');
    await flushAllLegacySummaries();
    await flushVersion();
    const notes = await loadNotesSession(meetingId);
    await notes.flush();
    const destination = await invoke<{ destination: string; local: boolean; payload_scope: string }>('api_get_enhancement_destination', { model: modelConfig.provider, modelName: modelConfig.model });
    if (!destination.local && remoteDestination !== destination.destination) { setRemote(destination); return; }
    const settings = await invoke<MeetingEnhancements['settings']>('api_save_enhancement_settings', { meetingId, templateId, instructions, expectedRevision: valid.settings.revision });
    setRecord(record => record ? { ...record, settings } : record);
    const language = await readMeetingSummaryLanguage(meetingId);
    const started = await invoke<{ version_id: string }>('api_start_meeting_enhancement', { input: {
      meeting_id: meetingId, expected_notes_revision: notes.getSnapshot().revision,
      model: modelConfig.provider, model_name: modelConfig.model, template_id: templateId, instructions,
      summary_language: language.language, remote_destination: remoteDestination ?? null,
      retry_of_version_id: current && ['failed', 'cancelled'].includes(current.status) ? current.id : null,
    } });
    setRemote(null); setSelected(started.version_id);
  });
  let document: EnhancementDocument | null = null;
  try { if (current?.document_json) document = parseEnhancementDocument(current.document_json); } catch { /* Editor reports malformed data without altering it. */ }
  const showClaims = current && !editedIds.has(current.id) && !document?.user_edited;
  const captureActive = recording.isRecording || recording.isProcessing || recording.isSaving;
  return <section className="enhancement-panel paper-surface h-full min-h-0 min-w-0 overflow-auto" aria-label="Enhanced notes">
    <div className="space-y-4 border-b border-border bg-muted/20 p-4 sm:p-5">
      <h2 className="text-base font-semibold tracking-tight">Enhance and review your notes</h2>
      <p className="max-w-prose text-sm leading-relaxed text-muted-foreground">New results are candidates. Approve a reviewed version to select it; your original notes and previous approved edits are retained.</p>
      <label className="block text-sm font-medium">Template<select value={templateId} onChange={event => setTemplateId(event.target.value)} className={field}>{templates.map(template => <option key={template.id} value={template.id}>{template.name}</option>)}</select></label>
      <label className="block text-sm font-medium">Your instructions<textarea value={instructions} onChange={event => setInstructions(event.target.value)} rows={2} maxLength={32000} className={field} placeholder="Topics to prioritize and how to organize the notes" /></label>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" disabled={busy || generating || captureActive || isModelConfigLoading || !modelConfig.model || !valid} onClick={() => { void generate(); }}>{current && ['failed', 'cancelled'].includes(current.status) ? 'Retry enhancement' : current ? 'Generate new candidate' : 'Enhance notes'}</Button>
        <Button size="sm" variant="outline" onClick={() => setShowCustom(value => !value)}>Custom template</Button>
        <Button size="sm" variant="ghost" disabled={busy} onClick={() => { void action(async () => {}); }}>Refresh versions</Button>
      </div>
      {captureActive && <p className="text-sm">Finish recording and transcript saving before enhancement. You can keep writing.</p>}
      {remote && <div className="space-y-2 rounded-xl border border-amber-200 bg-amber-50 p-3 text-sm leading-relaxed text-amber-900"><p>Send meeting content to <strong>{remote.destination}</strong>.</p><p className="mt-1">{remote.payload_scope}</p><div className="flex gap-2 mt-2"><Button size="sm" disabled={busy} onClick={() => { void generate(remote.destination); }}>Use this destination</Button><Button size="sm" variant="outline" onClick={() => setRemote(null)}>Cancel</Button></div></div>}
      {error && <p role="alert" className="text-sm text-red-700">{error}</p>}
      {showCustom && <CustomEnhancementTemplate selectedId={templateId} onSaved={async id => { await fetchTemplates(); setTemplateId(id); setShowCustom(false); }} />}
    </div>
    {valid?.versions.length ? <div className="space-y-3 border-b border-border p-4 sm:px-5">
      <label className="text-sm font-medium">Document version<select disabled={busy} className={field} value={selected ?? ''} onChange={event => { const id = event.target.value; void action(async () => { await flushVersion(); setSelected(id); }); }}>{valid.versions.map(version => <option key={version.id} value={version.id}>{version.state} · {version.status} · {new Date(version.created_at).toLocaleString()}</option>)}</select></label>
      {current?.source_stale && <p className="text-sm text-amber-800">This version uses an earlier source revision. Review the changes or generate another candidate.</p>}
      {current && progress[current.id] && <p role="status" className="text-sm text-muted-foreground">{progress[current.id]!.completed_chunks} of {progress[current.id]!.total_chunks} parts complete.{progress[current.id]!.retryable ? ' Retry reuses completed work when sources and settings match.' : ''}</p>}
      {current?.error && <p role="alert" className="text-sm text-red-700">{current.error}</p>}
      <div className="flex flex-wrap gap-2">
        {current?.state === 'candidate' && current.document_json && current.status === 'ready' && <>
          <Button size="sm" disabled={busy} onClick={() => { void action(async () => { const revision = await flushVersion(); await invoke('api_approve_enhancement_version', { meetingId, versionId: current.id, expectedRevision: revision, expectedApprovedVersionId: valid.approved_version_id }); enhancementSessions.get(current.id)?.session.dispose(); enhancementSessions.delete(current.id); window.dispatchEvent(new Event('meeting-data-changed')); }); }}>Approve this version</Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => { void action(async () => { const revision = await flushVersion(); await invoke('api_discard_enhancement_version', { meetingId, versionId: current.id, expectedRevision: revision }); }); }}>Discard candidate</Button>
        </>}
        {current?.status === 'generating' && <Button size="sm" variant="outline" disabled={busy} onClick={() => { void action(async () => { await invoke('api_cancel_meeting_enhancement', { meetingId, versionId: current.id }); }); }}>Cancel generation</Button>}
        {showClaims && !!document?.claims?.length && <Button size="sm" variant="outline" onClick={() => setInspectSources(value => !value)} aria-expanded={inspectSources}>Supporting sources</Button>}
      </div>
    </div> : <p className="m-4 rounded-xl border border-dashed border-border bg-muted/20 p-5 text-sm leading-relaxed text-muted-foreground">No enhanced versions yet. Notes-only meetings can also be enhanced.</p>}
    {inspectSources && showClaims && document?.claims && current && <ClaimSourcePanel key={current.id} meetingId={meetingId} claims={document.claims} snapshotJson={current.source_snapshot_json} />}
    {current?.document_json && <VersionEditor key={`${current.id}:${current.state}:${editorEpoch}`} version={current} locked={busy} onEdited={markEdited} onReload={() => { void action(async () => { await refresh(); enhancementSessions.get(current.id)?.session.dispose(); enhancementSessions.delete(current.id); setEditedIds(ids => { const next = new Set(ids); next.delete(current.id); return next; }); setEditorEpoch(value => value + 1); }); }} />}
    {current?.status === 'generating' && <p role="status" className="p-5 text-sm text-muted-foreground">Generating a separate candidate… Original notes remain editable.</p>}
  </section>;
}
