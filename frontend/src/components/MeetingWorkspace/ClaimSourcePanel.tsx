'use client';
import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';
import type { EnhancementClaim } from '@/types/enhancement';
import { AudioSourceBadge } from '@/components/AudioSourceBadge';
import { normalizeAudioSource } from '@/lib/transcript-audio-source';

interface Passage { text: string; audio_start_time: number | null; audio_source: string }
const time = (seconds: number | null | undefined) => seconds == null ? 'Time unavailable' : `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, '0')}`;
export function ClaimSourcePanel({ meetingId, claims, snapshotJson }: { meetingId: string; claims: EnhancementClaim[]; snapshotJson: string | null }) {
  const [passage, setPassage] = useState<Passage | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const request = useRef(0);
  useEffect(() => () => { request.current++; }, [meetingId, snapshotJson]);
  let sources: Array<{ source_id: string; kind: string; text: string; audio_start_time: number | null; audio_source: string | null }> = [];
  try { sources = JSON.parse(snapshotJson ?? '{}').sources ?? []; } catch { /* Saved excerpt remains usable when older provenance is unavailable. */ }
  const open = async (sourceId: string) => {
    const requestId = ++request.current;
    setPassage(null); setNotice(null);
    if (sourceId.startsWith('note:')) {
      window.dispatchEvent(new CustomEvent('workspace-note-source', { detail: { meetingId, blockId: sourceId.slice(5) } }));
      return;
    }
    try {
      const current = await invoke<Passage | null>('api_get_meeting_passage', { meetingId, transcriptId: sourceId.replace(/^transcript:/, '') });
      if (requestId !== request.current) return;
      if (current) {
        setPassage(current);
        const saved = sources.find(source => source.source_id === sourceId);
        if (saved && saved.text !== current.text) setNotice('The current passage differs from the saved source. Review the retained supporting excerpt against the current text.');
      }
      else setNotice('This passage is no longer in the current transcript. The saved supporting excerpt is retained below.');
    } catch { if (requestId === request.current) setNotice('The current passage could not be loaded. Its saved supporting excerpt is retained below.'); }
  };
  return <aside className="border-b border-border bg-secondary/60 p-4 text-sm leading-relaxed sm:p-5" aria-label="Claim sources">
    <p className="mb-3">Review whether each excerpt supports its claim. References identify source text; audio channels do not identify participants.</p>
    {notice && <p role="status" className="mb-3 text-amber-800">{notice}</p>}
    {passage && <div className="paper-surface mb-4 rounded-xl border border-border p-4"><p className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">{time(passage.audio_start_time)}<AudioSourceBadge source={normalizeAudioSource(passage.audio_source)} /></p><p className="mt-2 break-words whitespace-pre-wrap text-foreground">{passage.text}</p><Button size="sm" variant="ghost" onClick={() => { request.current++; setPassage(null); }}>Close passage</Button></div>}
    {claims.map(claim => <div key={claim.id} className="mb-5 last:mb-0"><p className="break-words font-semibold">{claim.text}</p><p className="text-muted-foreground">{claim.status.replaceAll('_', ' ')}</p>{claim.sources.map((source, i) => {
      const saved = sources.find(item => item.source_id === source.source_id);
      return <blockquote key={i} className="mt-3 border-l-2 border-primary/30 pl-3"><div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground"><span>{saved?.kind === 'notes' || source.source_id.startsWith('note:') ? 'My notes' : time(saved?.audio_start_time)}</span>{saved?.kind === 'transcript' && <AudioSourceBadge source={normalizeAudioSource(saved.audio_source)} />}<Button size="sm" variant="ghost" onClick={() => { void open(source.source_id); }}>Open source</Button></div><p className="break-words whitespace-pre-wrap">{source.excerpt}</p></blockquote>;
    })}</div>)}
  </aside>;
}
