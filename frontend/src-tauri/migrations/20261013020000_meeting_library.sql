-- Manual metadata preserves the entered display text; lookup keys are supplied by
-- the native validator. Search is disposable derived data, never the source.
CREATE TABLE meeting_participants (
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    normalized TEXT NOT NULL,
    display_text TEXT NOT NULL,
    PRIMARY KEY (meeting_id, normalized)
);
CREATE INDEX meeting_participants_lookup ON meeting_participants(normalized, meeting_id);
CREATE TABLE meeting_tags (
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    normalized TEXT NOT NULL,
    display_text TEXT NOT NULL,
    PRIMARY KEY (meeting_id, normalized)
);
CREATE INDEX meeting_tags_lookup ON meeting_tags(normalized, meeting_id);

CREATE TABLE meeting_search_dirty (
    meeting_id TEXT PRIMARY KEY NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    generation INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL
);
CREATE TABLE meeting_search_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL CHECK(source_kind IN ('title', 'original', 'transcript', 'approved')),
    source_id TEXT NOT NULL,
    text_content TEXT NOT NULL,
    revision INTEGER,
    audio_start_time REAL,
    audio_end_time REAL,
    UNIQUE (meeting_id, source_kind, source_id)
);
CREATE INDEX meeting_search_sources_meeting ON meeting_search_sources(meeting_id, source_kind);
CREATE VIRTUAL TABLE meeting_search_fts USING fts5(
    text_content, content='meeting_search_sources', content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER meeting_search_sources_insert AFTER INSERT ON meeting_search_sources BEGIN
    INSERT INTO meeting_search_fts(rowid, text_content) VALUES (new.id, new.text_content);
END;
CREATE TRIGGER meeting_search_sources_delete AFTER DELETE ON meeting_search_sources BEGIN
    INSERT INTO meeting_search_fts(meeting_search_fts, rowid, text_content)
        VALUES ('delete', old.id, old.text_content);
END;
CREATE TRIGGER meeting_search_sources_update AFTER UPDATE OF text_content ON meeting_search_sources BEGIN
    INSERT INTO meeting_search_fts(meeting_search_fts, rowid, text_content)
        VALUES ('delete', old.id, old.text_content);
    INSERT INTO meeting_search_fts(rowid, text_content) VALUES (new.id, new.text_content);
END;

-- Tiny triggers participate in the primary transaction. Index work happens
-- separately after commit, so acknowledged original-note saves do not await FTS.
CREATE TRIGGER meeting_search_meeting_insert AFTER INSERT ON meetings BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_meeting_update AFTER UPDATE OF title, status, started_at, ended_at, timezone ON meetings BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_notes_insert AFTER INSERT ON meeting_notes BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_notes_update AFTER UPDATE ON meeting_notes BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_notes_delete AFTER DELETE ON meeting_notes BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at)
        SELECT old.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE EXISTS(SELECT 1 FROM meetings WHERE id = old.meeting_id)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_transcript_insert AFTER INSERT ON transcripts BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_transcript_update AFTER UPDATE ON transcripts BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
    INSERT INTO meeting_search_dirty(meeting_id, updated_at)
        SELECT old.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE old.meeting_id <> new.meeting_id
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_transcript_delete AFTER DELETE ON transcripts BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at)
        SELECT old.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE EXISTS(SELECT 1 FROM meetings WHERE id = old.meeting_id)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_enhancement_insert AFTER INSERT ON meeting_enhancement_versions BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_enhancement_update AFTER UPDATE ON meeting_enhancement_versions BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_enhancement_delete AFTER DELETE ON meeting_enhancement_versions BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at)
        SELECT old.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE EXISTS(SELECT 1 FROM meetings WHERE id = old.meeting_id)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_approval_insert AFTER INSERT ON meeting_enhancement_settings BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_approval_update AFTER UPDATE OF approved_version_id ON meeting_enhancement_settings BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_summary_insert AFTER INSERT ON summary_processes BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_summary_update AFTER UPDATE ON summary_processes BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at) VALUES (new.meeting_id, new.updated_at)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
CREATE TRIGGER meeting_search_summary_delete AFTER DELETE ON summary_processes BEGIN
    INSERT INTO meeting_search_dirty(meeting_id, updated_at)
        SELECT old.meeting_id, strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE EXISTS(SELECT 1 FROM meetings WHERE id = old.meeting_id)
        ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at;
END;
INSERT INTO meeting_search_dirty(meeting_id, updated_at) SELECT id, updated_at FROM meetings;
