interface Entry { meetingId: string; flush: () => Promise<void>; dirty: () => boolean; detached: boolean }
const entries = new Map<symbol, Entry>();
const deleted = new Set<string>();

/** Detached dirty views retain their captured save callback until acknowledged. */
export function registerLegacySummaryFlusher(meetingId: string, flush: () => Promise<void>, dirty: () => boolean) {
  if (deleted.has(meetingId)) return () => {};
  const token = Symbol(meetingId);
  const entry: Entry = { meetingId, flush, dirty, detached: false };
  entries.set(token, entry);
  return () => {
    entry.detached = true;
    if (!entry.dirty()) entries.delete(token);
  };
}
export async function flushAllLegacySummaries(meetingId?: string) {
  await Promise.all([...entries].filter(([, entry]) => meetingId === undefined || entry.meetingId === meetingId).map(async ([token, entry]) => {
    // Typing may continue while a save is awaiting conversion or IPC. Do not
    // let navigation/quit proceed until that newer edit is acknowledged too.
    while (entries.has(token) && entry.dirty()) await entry.flush();
    if (entry.detached && !entry.dirty()) entries.delete(token);
  }));
}
export function forgetMeetingLegacySummaries(meetingId: string) {
  deleted.add(meetingId);
  for (const [token, entry] of entries) if (entry.meetingId === meetingId) entries.delete(token);
}
