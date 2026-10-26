ALTER TABLE meetings ADD COLUMN status TEXT NOT NULL DEFAULT 'completed';
ALTER TABLE meetings ADD COLUMN started_at TEXT;
ALTER TABLE meetings ADD COLUMN ended_at TEXT;
ALTER TABLE meetings ADD COLUMN timezone TEXT;
ALTER TABLE meetings ADD COLUMN recording_session_id TEXT;
CREATE UNIQUE INDEX meetings_recording_session ON meetings(recording_session_id)
    WHERE recording_session_id IS NOT NULL;
ALTER TABLE meeting_notes ADD COLUMN revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE meeting_notes ADD COLUMN last_commit_id TEXT;
ALTER TABLE meeting_notes ADD COLUMN notes_plain_text TEXT;
ALTER TABLE meeting_notes ADD COLUMN conversion_status TEXT NOT NULL DEFAULT 'unavailable';
ALTER TABLE transcripts ADD COLUMN recording_session_id TEXT;
ALTER TABLE transcripts ADD COLUMN sequence_id INTEGER;
CREATE UNIQUE INDEX transcript_session_sequence ON transcripts(meeting_id, recording_session_id, sequence_id)
    WHERE recording_session_id IS NOT NULL AND sequence_id IS NOT NULL;
CREATE TABLE meeting_note_links (
    id TEXT PRIMARY KEY NOT NULL,
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    block_id TEXT NOT NULL,
    recording_session_id TEXT NOT NULL,
    recording_offset REAL NOT NULL CHECK(recording_offset >= 0),
    transcript_id TEXT REFERENCES transcripts(id) ON DELETE SET NULL
);
CREATE INDEX meeting_note_links_meeting ON meeting_note_links(meeting_id);
