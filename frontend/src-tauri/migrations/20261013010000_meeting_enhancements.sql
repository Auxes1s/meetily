-- Enhancement documents own their content; original notes remain independent.
CREATE TABLE meeting_enhancement_versions (
    id TEXT PRIMARY KEY NOT NULL,
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK(state IN ('candidate','approved','archived','legacy')),
    status TEXT NOT NULL CHECK(status IN ('generating','ready','failed','cancelled','discarded','unverified')),
    revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0),
    document_json TEXT,
    source_snapshot_json TEXT,
    source_fingerprint TEXT,
    notes_revision INTEGER,
    template_id TEXT NOT NULL DEFAULT 'standard_meeting',
    instructions TEXT NOT NULL DEFAULT '',
    provider_config_json TEXT,
    last_commit_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    error TEXT
);
CREATE UNIQUE INDEX meeting_enhancement_single_approval ON meeting_enhancement_versions(meeting_id) WHERE state = 'approved';
CREATE INDEX meeting_enhancement_history ON meeting_enhancement_versions(meeting_id, created_at DESC);
CREATE TABLE meeting_enhancement_settings (
    meeting_id TEXT PRIMARY KEY NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    template_id TEXT NOT NULL DEFAULT 'standard_meeting',
    instructions TEXT NOT NULL DEFAULT '',
    revision INTEGER NOT NULL DEFAULT 0,
    approved_version_id TEXT REFERENCES meeting_enhancement_versions(id) ON DELETE SET NULL,
    remote_destination TEXT,
    remote_payload_scope TEXT,
    remote_selected_at TEXT
);
