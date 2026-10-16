-- Jobs deliberately have no meeting FK: retries and cache acknowledgment survive
-- removal of canonical content. A completed job retains only opaque identities.
CREATE TABLE meeting_deletion_jobs (
    job_id TEXT PRIMARY KEY NOT NULL,
    meeting_id TEXT NOT NULL UNIQUE,
    recording_session_id TEXT,
    folder_path TEXT,
    artifacts_json TEXT,
    native_complete INTEGER NOT NULL DEFAULT 0 CHECK(native_complete IN (0, 1)),
    caches_complete INTEGER NOT NULL DEFAULT 0 CHECK(caches_complete IN (0, 1)),
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE meeting_deletion_tombstones (
    meeting_id TEXT PRIMARY KEY NOT NULL,
    recording_session_id TEXT,
    deleted_at TEXT NOT NULL
);
CREATE INDEX meeting_deletion_session ON meeting_deletion_tombstones(recording_session_id)
    WHERE recording_session_id IS NOT NULL;
CREATE TRIGGER deleted_meeting_insert BEFORE INSERT ON meetings
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.id
    OR (recording_session_id IS NOT NULL AND recording_session_id = NEW.recording_session_id))
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_meeting_update BEFORE UPDATE ON meetings
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.id OR meeting_id = OLD.id
    OR (recording_session_id IS NOT NULL AND recording_session_id = NEW.recording_session_id))
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_notes_insert BEFORE INSERT ON meeting_notes
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_notes_update BEFORE UPDATE ON meeting_notes
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id OR meeting_id = OLD.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_transcript_insert BEFORE INSERT ON transcripts
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_transcript_update BEFORE UPDATE ON transcripts
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id OR meeting_id = OLD.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_summary_insert BEFORE INSERT ON summary_processes
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_summary_update BEFORE UPDATE ON summary_processes
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id OR meeting_id = OLD.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_chunk_insert BEFORE INSERT ON transcript_chunks
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_chunk_update BEFORE UPDATE ON transcript_chunks
WHEN EXISTS (SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = NEW.meeting_id OR meeting_id = OLD.meeting_id)
BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END;
