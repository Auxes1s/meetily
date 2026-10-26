import { AudioLines, CircleHelp, Mic, Volume2 } from 'lucide-react';
import { audioSourceLabels, normalizeAudioSource } from '@/lib/transcript-audio-source';

export function AudioSourceBadge({ source }: { source?: unknown }) {
  const value = normalizeAudioSource(source);
  const Icon = { microphone: Mic, system: Volume2, mixed: AudioLines, unknown: CircleHelp }[value];
  const label = audioSourceLabels[value];
  return (
    <span
      className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-full border border-border bg-secondary/70 text-muted-foreground"
      role="img"
      aria-label={`Audio source: ${label}`}
      title={`Capture channel: ${label}`}
    >
      <Icon className="h-3 w-3 flex-shrink-0" aria-hidden="true" />
    </span>
  );
}
