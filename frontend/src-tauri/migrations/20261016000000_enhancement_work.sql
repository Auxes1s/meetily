-- Checkpoints contain validated derived claims, never rejected model responses.
CREATE TABLE meeting_enhancement_work (
    version_id TEXT PRIMARY KEY NOT NULL REFERENCES meeting_enhancement_versions(id) ON DELETE CASCADE,
    generation_key TEXT NOT NULL,
    checkpoint_json TEXT NOT NULL,
    completed_chunks INTEGER NOT NULL DEFAULT 0,
    total_chunks INTEGER NOT NULL DEFAULT 0,
    failed_chunk INTEGER,
    error_code TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    prompt_tokens INTEGER,
    generated_tokens INTEGER,
    updated_at TEXT NOT NULL
);
