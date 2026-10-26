import type { Block } from '@blocknote/core';
import { MeetingNotesSession } from './meeting-notes-session';
import { parseEnhancementDocument, type EnhancementDocument, type EnhancementVersion } from '@/types/enhancement';

export interface EnhancementSave {
  meeting_id: string;
  version_id: string;
  expected_revision: number;
  commit_id: string;
  document: EnhancementDocument;
}
export type EnhancementGateway = (input: EnhancementSave) => Promise<EnhancementVersion>;

/** The scheduler is shared, but this gateway can only save the selected enhancement. */
export function createEnhancementSession(version: EnhancementVersion, blocks: Block[], gateway: EnhancementGateway) {
  let document = parseEnhancementDocument(version.document_json);
  const session = new MeetingNotesSession(version.id, {
    blocks, links: [], title: '', revision: version.revision, updatedAt: version.updated_at,
  }, async input => {
    const updated: EnhancementDocument = {
      ...document,
      summary_json: JSON.parse(input.notes_json),
      markdown: input.notes_markdown,
      user_edited: true,
      // Free-form edits cannot keep structured owners/actions falsely authoritative.
      claims: [],
    };
    try {
      const committed = await gateway({
        meeting_id: version.meeting_id, version_id: version.id,
        expected_revision: input.base_revision, commit_id: input.commit_id, document: updated,
      });
      document = updated;
      if (typeof window !== 'undefined') window.dispatchEvent?.(new CustomEvent('enhancement-document-committed', {
        detail: { meetingId: version.meeting_id, versionId: version.id, document: updated, revision: committed.revision, updatedAt: committed.updated_at },
      }));
      return { revision: committed.revision, updated_at: committed.updated_at };
    } catch (error) {
      const message = typeof error === 'string' ? error : (error as Error)?.message ?? String(error);
      if (/conflict|revision/i.test(message)) throw { code: 'conflict', message };
      throw error;
    }
  });
  return session;
}

export const enhancementSessions = new Map<string, { meetingId: string; session: MeetingNotesSession }>();
export async function flushAllEnhancements() {
  await Promise.all([...enhancementSessions.values()].map(({ session }) => session.flush()));
}
export function forgetMeetingEnhancements(meetingId: string) {
  for (const [id, entry] of enhancementSessions) if (entry.meetingId === meetingId) {
    entry.session.dispose();
    enhancementSessions.delete(id);
  }
}
