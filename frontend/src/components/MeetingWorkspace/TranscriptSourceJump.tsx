'use client';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AudioSourceBadge } from '@/components/AudioSourceBadge';
import { normalizeAudioSource } from '@/lib/transcript-audio-source';

interface Passage { text: string; audio_start_time: number | null; audio_source: string; timeline_offset?: number }
/** Load by canonical ID so a hit outside the visible transcript page still opens. */
export function TranscriptSourceJump({ meetingId, transcriptId }: { meetingId: string; transcriptId: string }) {
  const [result, setResult] = useState<{ passage: Passage | null; error?: string } | null>(null);
  useEffect(() => {
    let current = true;
    setResult(null);
    void invoke<Passage | null>('api_get_meeting_passage', { meetingId, transcriptId })
      .then(passage => { if (current) setResult({ passage }); })
      .catch(() => { if (current) setResult({ passage: null, error: 'The passage could not be opened. Retry the search.' }); });
    return () => { current = false; };
  }, [meetingId, transcriptId]);
  const seconds = result?.passage?.audio_start_time == null ? null : result.passage.audio_start_time + (result.passage.timeline_offset ?? 0);
  return <aside aria-label="Matching transcript passage" className="m-3 rounded border border-primary/20 bg-primary/5 p-3 text-sm">
    <p className="font-medium mb-2">Matching transcript passage</p>
    {!result ? <p role="status">Loading passage…</p> : result.passage ? <>
      <p className="flex gap-2 items-center mb-2">{seconds == null ? 'Time unavailable' : `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, '0')}`}<AudioSourceBadge source={normalizeAudioSource(result.passage.audio_source)} /></p>
      <p className="whitespace-pre-wrap">{result.passage.text}</p>
    </> : <p role="status">{result.error ?? 'This passage has changed or is no longer available. Search again for current content.'}</p>}
  </aside>;
}
