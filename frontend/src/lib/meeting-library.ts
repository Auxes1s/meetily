import type { LibrarySourceHit } from '@/types/meeting-library';

export function meetingSourceUrl(meetingId: string, hit?: LibrarySourceHit): string {
  const params = new URLSearchParams({ id: meetingId });
  if (hit) {
    params.set('source', hit.source_kind);
    params.set('sourceId', hit.source_id);
    if (hit.revision != null) params.set('sourceRevision', String(hit.revision));
  }
  return `/meeting-details?${params}`;
}

/** A date filter means the user's local calendar day, sent as exact UTC bounds. */
export function localDateBound(date: string, exclusiveEnd = false): string | null {
  if (!date) return null;
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
  if (!match) throw new Error('Choose a valid date');
  const [year, month, day] = match.slice(1).map(Number);
  const instant = new Date(year, month - 1, day);
  if (instant.getFullYear() !== year || instant.getMonth() !== month - 1 || instant.getDate() !== day) throw new Error('Choose a valid date');
  if (exclusiveEnd) instant.setDate(instant.getDate() + 1);
  return instant.toISOString();
}

export function manualMetadataLines(text: string): string[] {
  return text.split(/\r?\n/).map(value => value.trim()).filter(Boolean);
}

export const sourceLabels = { title: 'Title', original: 'My notes', transcript: 'Transcript', approved: 'Approved / legacy notes' } as const;

export function passageTime(seconds: number | null): string {
  if (seconds == null || !Number.isFinite(seconds) || seconds < 0) return 'Time unavailable';
  const whole = Math.floor(seconds);
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
}
