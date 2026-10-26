CREATE TABLE recording_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK(ordinal > 0),
    status TEXT NOT NULL CHECK(status IN ('starting','recording','finalizing','completed','interrupted','failed','cancelled')),
    created_at TEXT NOT NULL,
    started_at TEXT,
    ended_at TEXT,
    duration_seconds REAL CHECK(duration_seconds >= 0),
    timeline_offset REAL NOT NULL DEFAULT 0 CHECK(timeline_offset >= 0),
    duration_basis TEXT NOT NULL DEFAULT 'unknown',
    folder_path TEXT,
    previous_meeting_status TEXT NOT NULL DEFAULT 'draft',
    previous_meeting_json TEXT,
    UNIQUE(meeting_id, ordinal)
);
CREATE UNIQUE INDEX recording_session_active ON recording_sessions(meeting_id)
    WHERE status IN ('starting','recording','finalizing');

-- Backfill container identity without changing passage IDs, text or local times.
INSERT INTO recording_sessions(id,meeting_id,ordinal,status,created_at,started_at,ended_at,duration_seconds,duration_basis,folder_path,previous_meeting_status)
SELECT COALESCE(m.recording_session_id,'legacy-'||m.id),m.id,1,
    CASE WHEN m.status IN ('recording','finalizing','interrupted') THEN 'interrupted' ELSE 'completed' END,
    m.created_at,m.started_at,m.ended_at,
    (SELECT MAX(t.audio_end_time) FROM transcripts t WHERE t.meeting_id=m.id),
    CASE WHEN EXISTS(SELECT 1 FROM transcripts t WHERE t.meeting_id=m.id AND t.audio_end_time IS NOT NULL) THEN 'transcript' ELSE 'unknown' END,
    m.folder_path,m.status
FROM meetings m WHERE m.status <> 'deleting' AND
    (m.recording_session_id IS NOT NULL OR m.folder_path IS NOT NULL OR EXISTS(SELECT 1 FROM transcripts t WHERE t.meeting_id=m.id));
UPDATE transcripts SET recording_session_id =
    (SELECT id FROM recording_sessions s WHERE s.meeting_id=transcripts.meeting_id AND s.ordinal=1)
WHERE recording_session_id IS NULL AND EXISTS(SELECT 1 FROM recording_sessions s WHERE s.meeting_id=transcripts.meeting_id);

ALTER TABLE meeting_deletion_jobs ADD COLUMN session_folders_json TEXT;
ALTER TABLE meeting_deletion_jobs ADD COLUMN recording_session_ids_json TEXT;
CREATE TABLE meeting_deletion_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    meeting_id TEXT NOT NULL,
    deleted_at TEXT NOT NULL
);
INSERT OR IGNORE INTO meeting_deletion_sessions SELECT recording_session_id,meeting_id,deleted_at FROM meeting_deletion_tombstones WHERE recording_session_id IS NOT NULL;
CREATE TRIGGER deleted_recording_session_insert BEFORE INSERT ON recording_sessions
WHEN EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id=NEW.meeting_id)
    OR EXISTS(SELECT 1 FROM meeting_deletion_sessions WHERE id=NEW.id)
BEGIN SELECT RAISE(ABORT,'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_recording_session_update BEFORE UPDATE ON recording_sessions
WHEN EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id=NEW.meeting_id OR meeting_id=OLD.meeting_id)
    OR EXISTS(SELECT 1 FROM meeting_deletion_sessions WHERE id=NEW.id OR id=OLD.id)
BEGIN SELECT RAISE(ABORT,'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_session_meeting_insert BEFORE INSERT ON meetings
WHEN EXISTS(SELECT 1 FROM meeting_deletion_sessions WHERE id=NEW.recording_session_id)
BEGIN SELECT RAISE(ABORT,'Meeting deletion is pending or complete'); END;
CREATE TRIGGER deleted_session_meeting_update BEFORE UPDATE ON meetings
WHEN EXISTS(SELECT 1 FROM meeting_deletion_sessions WHERE id=NEW.recording_session_id)
BEGIN SELECT RAISE(ABORT,'Meeting deletion is pending or complete'); END;
