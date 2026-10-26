-- The session/sequence uniqueness index is partial and cannot serve reads that
-- also include legacy rows without a session or sequence. Keep that constraint
-- and add the narrow lookup shared by pagination, source snapshots and search.
CREATE INDEX IF NOT EXISTS transcripts_meeting_lookup ON transcripts(meeting_id);
