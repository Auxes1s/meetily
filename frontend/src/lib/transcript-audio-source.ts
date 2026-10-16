import type { AudioSource, Transcript, TranscriptSegmentData } from '@/types';

export function normalizeAudioSource(value: unknown): AudioSource {
  switch (value) {
    case 'microphone':
    case 'mic': return 'microphone';
    case 'system': return 'system';
    case 'mixed': return 'mixed';
    default: return 'unknown';
  }
}

export const audioSourceLabels: Record<AudioSource, string> = {
  microphone: 'Microphone',
  system: 'System audio',
  mixed: 'Mixed audio',
  unknown: 'Unknown',
};

export function transcriptToSegment(transcript: Transcript): TranscriptSegmentData {
  return {
    id: transcript.id,
    timestamp: transcriptTime(transcript) ?? 0,
    endTime: transcript.audio_end_time == null ? undefined : transcript.audio_end_time + (transcript.timeline_offset ?? 0),
    text: transcript.text,
    confidence: transcript.confidence,
    audio_source: normalizeAudioSource(transcript.audio_source),
    ...(transcript.audio_start_time == null ? { timestampAvailable: false } : {}),
  };
}

export function transcriptTime(transcript: Transcript): number | undefined {
  return transcript.audio_start_time == null ? undefined : transcript.audio_start_time + (transcript.timeline_offset ?? 0);
}

/** A final update must replace its partial without erasing known provenance. */
export function mergeTranscript(previous: Transcript | undefined, incoming: Transcript): Transcript {
  if (previous && previous.is_partial === false && incoming.is_partial) return previous;
  if (previous && (incoming.update_revision ?? 0) < (previous.update_revision ?? 0)) return previous;
  const source = normalizeAudioSource(incoming.audio_source);
  return {
    ...previous,
    ...incoming,
    id: previous?.id ?? incoming.id,
    audio_source: source === 'unknown' && previous
      ? normalizeAudioSource(previous.audio_source)
      : source,
  };
}
