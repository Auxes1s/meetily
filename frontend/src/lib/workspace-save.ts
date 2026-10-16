import { flushAllNotes, noteSessions } from './meeting-notes-session';
import { flushAllEnhancements, enhancementSessions } from './enhancement-session';
import { flushAllLegacySummaries } from './legacy-summary-flush';

export async function flushWorkspaceDocuments() {
  await Promise.all([flushAllNotes(), flushAllEnhancements(), flushAllLegacySummaries()]);
}

/** Export only waits for the selected meeting, including detached dirty views. */
export async function flushMeetingDocuments(meetingId: string) {
  // A legacy view may commit the same enhanced version as the workspace. Finish
  // its save first so optimistic revision protection can disclose any conflict.
  do {
    await flushAllLegacySummaries(meetingId);
    await Promise.all([
      noteSessions.get(meetingId)?.flush(),
      ...[...enhancementSessions.values()].filter(entry => entry.meetingId === meetingId).map(entry => entry.session.flush()),
    ]);
    await flushAllLegacySummaries(meetingId);
  } while (noteSessions.get(meetingId)?.dirty || [...enhancementSessions.values()].some(entry => entry.meetingId === meetingId && entry.session.dirty));
}
