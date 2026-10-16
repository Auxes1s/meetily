//! Original notes only. Summary generation never calls this repository.
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tauri::{AppHandle, Manager};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct NotesError {
    pub code: &'static str,
    pub message: String,
    pub current_revision: Option<i64>,
}
impl NotesError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), current_revision: None }
    }
}
impl From<sqlx::Error> for NotesError {
    fn from(error: sqlx::Error) -> Self { Self::new("database", error.to_string()) }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NoteLink {
    pub id: String,
    pub block_id: String,
    pub recording_session_id: String,
    pub recording_offset: f64,
    pub transcript_id: Option<String>,
}
#[derive(Debug, Serialize, FromRow)]
pub struct NotesDocument {
    pub meeting_id: String,
    pub notes_json: Option<String>,
    pub notes_markdown: Option<String>,
    pub notes_plain_text: Option<String>,
    pub conversion_status: String,
    pub revision: i64,
    pub updated_at: String,
    #[sqlx(skip)]
    pub links: Vec<NoteLink>,
}
#[derive(Debug, Deserialize)]
pub struct NotesSave {
    pub meeting_id: String,
    pub notes_json: String,
    pub notes_markdown: Option<String>,
    pub notes_plain_text: Option<String>,
    pub base_revision: i64,
    pub commit_id: String,
    pub title: Option<String>,
    #[serde(default)]
    pub links: Vec<NoteLink>,
}
#[derive(Debug, Serialize)]
pub struct NotesCommit { pub revision: i64, pub updated_at: String }

pub async fn load(pool: &SqlitePool, meeting_id: &str) -> Result<Option<NotesDocument>, NotesError> {
    if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meetings WHERE id = ? AND status <> 'deleting'")
        .bind(meeting_id).fetch_one(pool).await? == 0 {
        return Err(NotesError::new("not_found", "Meeting not found"));
    }
    let mut tx = pool.begin().await?;
    let mut doc = sqlx::query_as::<_, NotesDocument>("SELECT * FROM meeting_notes WHERE meeting_id = ?")
        .bind(meeting_id).fetch_optional(&mut *tx).await?;
    if let Some(ref mut doc) = doc {
        doc.links = sqlx::query_as("SELECT * FROM meeting_note_links WHERE meeting_id = ?")
            .bind(meeting_id).fetch_all(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(doc)
}

pub async fn save(pool: &SqlitePool, input: NotesSave) -> Result<NotesCommit, NotesError> {
    let blocks: serde_json::Value = serde_json::from_str(&input.notes_json)
        .map_err(|_| NotesError::new("invalid_document", "Notes must contain valid block JSON"))?;
    let blocks = blocks.as_array().ok_or_else(|| NotesError::new("invalid_document", "Notes must be an array of blocks"))?;
    fn collect_ids(blocks: &[serde_json::Value], ids: &mut std::collections::HashSet<String>) -> bool {
        blocks.iter().all(|block| {
            let Some(id) = block.get("id").and_then(|v| v.as_str()) else { return false; };
            if block.get("type").and_then(|v| v.as_str()).is_none() || !ids.insert(id.to_owned()) { return false; }
            match block.get("children") {
                None => true,
                Some(children) => children.as_array().is_some_and(|children| collect_ids(children, ids)),
            }
        })
    }
    let mut ids = std::collections::HashSet::new();
    if input.base_revision < 0 || uuid::Uuid::parse_str(&input.commit_id).is_err() || !collect_ids(blocks, &mut ids) {
        return Err(NotesError::new("invalid_document", "Invalid note blocks or revision"));
    }
    for link in &input.links {
        if !ids.contains(&link.block_id) || !link.recording_offset.is_finite() || link.recording_offset < 0.0 {
            return Err(NotesError::new("invalid_document", "Invalid timestamp marker"));
        }
    }
    let now = Utc::now().to_rfc3339();
    let mut tx = pool.begin().await?;
    // Take the write lock before reading the revision, then compare-and-set.
    sqlx::query("INSERT OR IGNORE INTO meeting_notes (meeting_id, created_at, updated_at) VALUES (?, ?, ?)")
        .bind(&input.meeting_id).bind(&now).bind(&now).execute(&mut *tx).await?;
    let prior: (i64, Option<String>, String) = sqlx::query_as("SELECT revision, last_commit_id, updated_at FROM meeting_notes WHERE meeting_id = ?")
        .bind(&input.meeting_id).fetch_one(&mut *tx).await?;
    if prior.1.as_deref() == Some(&input.commit_id) && prior.0 == input.base_revision + 1 {
        tx.commit().await?;
        return Ok(NotesCommit { revision: prior.0, updated_at: prior.2 });
    }
    let session_id: Option<String> = sqlx::query_scalar("SELECT recording_session_id FROM meetings WHERE id = ?")
        .bind(&input.meeting_id).fetch_one(&mut *tx).await?;
    let mut link_ids = std::collections::HashSet::new();
    for link in &input.links {
        if !link_ids.insert(&link.id) || session_id.as_deref() != Some(&link.recording_session_id) || !input.notes_json.contains(&format!("meetily-note://{}", link.id)) {
            return Err(NotesError::new("invalid_document", "Timestamp marker does not belong to this document and recording"));
        }
    }
    let result = sqlx::query("UPDATE meeting_notes SET notes_json = ?, notes_markdown = ?, notes_plain_text = ?, conversion_status = ?, revision = revision + 1, updated_at = ?, last_commit_id = ? WHERE meeting_id = ? AND revision = ?")
        .bind(&input.notes_json).bind(&input.notes_markdown).bind(&input.notes_plain_text)
        .bind(if input.notes_markdown.is_some() { "ready" } else { "unavailable" })
        .bind(&now).bind(&input.commit_id).bind(&input.meeting_id).bind(input.base_revision).execute(&mut *tx).await?;
    if result.rows_affected() != 1 {
        let current_revision = sqlx::query_scalar("SELECT revision FROM meeting_notes WHERE meeting_id = ?")
            .bind(&input.meeting_id).fetch_optional(&mut *tx).await?;
        return Err(NotesError { code: "conflict", message: "A newer revision is already saved. Your edits are retained.".into(), current_revision });
    }
    sqlx::query("DELETE FROM meeting_note_links WHERE meeting_id = ?").bind(&input.meeting_id).execute(&mut *tx).await?;
    for link in &input.links {
        // Resolve only a passage in this meeting/session whose interval contains the measured offset.
        let transcript_id: Option<String> = sqlx::query_scalar("SELECT id FROM transcripts WHERE meeting_id = ? AND recording_session_id = ? AND audio_start_time <= ? AND audio_end_time >= ? ORDER BY audio_start_time DESC LIMIT 1")
            .bind(&input.meeting_id).bind(&link.recording_session_id).bind(link.recording_offset).bind(link.recording_offset)
            .fetch_optional(&mut *tx).await?;
        sqlx::query("INSERT INTO meeting_note_links (id, meeting_id, block_id, recording_session_id, recording_offset, transcript_id) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&link.id).bind(&input.meeting_id).bind(&link.block_id).bind(&link.recording_session_id)
            .bind(link.recording_offset).bind(transcript_id).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE meetings SET updated_at = ?, title = COALESCE(?, title) WHERE id = ?").bind(&now).bind(&input.title).bind(&input.meeting_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(NotesCommit { revision: input.base_revision + 1, updated_at: now })
}

#[tauri::command]
pub async fn api_create_meeting_draft(state: tauri::State<'_, AppState>, title: Option<String>, timezone: Option<String>) -> Result<serde_json::Value, NotesError> {
    let id = format!("meeting-{}", uuid::Uuid::new_v4());
    let title = title.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "Untitled meeting".into());
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO meetings (id, title, created_at, updated_at, status, timezone) VALUES (?, ?, ?, ?, 'draft', ?)")
        .bind(&id).bind(&title).bind(&now).bind(&now).bind(timezone).execute(state.db_manager.pool()).await?;
    Ok(serde_json::json!({"id": id, "title": title, "created_at": now, "updated_at": now, "status": "draft"}))
}
#[tauri::command]
pub async fn api_ensure_recovery_meeting(state: tauri::State<'_, AppState>, recovery_key: String, title: String) -> Result<serde_json::Value, NotesError> {
    if recovery_key.is_empty() || recovery_key.len() > 256 { return Err(NotesError::new("invalid_document", "Invalid recovery identity")); }
    let id = format!("recovered-{recovery_key}");
    let session = format!("legacy-recovery-{recovery_key}");
    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT OR IGNORE INTO meetings (id, title, created_at, updated_at, status, recording_session_id) VALUES (?, ?, ?, ?, 'interrupted', ?)")
        .bind(&id).bind(title).bind(&now).bind(&now).bind(&session).execute(state.db_manager.pool()).await?;
    Ok(serde_json::json!({"meeting_id": id, "recording_session_id": session}))
}
#[tauri::command]
pub async fn api_get_meeting_notes(state: tauri::State<'_, AppState>, meeting_id: String) -> Result<Option<NotesDocument>, NotesError> {
    load(state.db_manager.pool(), &meeting_id).await
}
#[tauri::command]
pub async fn api_save_meeting_notes(state: tauri::State<'_, AppState>, input: NotesSave) -> Result<NotesCommit, NotesError> {
    save(state.db_manager.pool(), input).await
}

#[tauri::command]
pub async fn complete_notes_only_setup(app: AppHandle) -> Result<(), String> {
    if app.try_state::<AppState>().is_none() {
        crate::database::commands::initialize_fresh_database(app.clone()).await?;
    }
    let mut status = crate::onboarding::load_onboarding_status(&app).await.map_err(|e| e.to_string())?;
    status.completed = true;
    crate::onboarding::save_onboarding_status(&app, &status).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub static EXIT_CONFIRMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static EXIT_PENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[tauri::command]
pub async fn confirm_workspace_quit(app: AppHandle) -> Result<(), String> {
    if crate::audio::recording_commands::is_recording().await { return Err("End the active recording before quitting".into()); }
    EXIT_CONFIRMED.store(true, std::sync::atomic::Ordering::SeqCst);
    app.exit(0);
    Ok(())
}
#[tauri::command]
pub fn cancel_workspace_quit() { EXIT_PENDING.store(false, std::sync::atomic::Ordering::SeqCst); }

#[tauri::command]
pub async fn api_get_note_passage(state: tauri::State<'_, AppState>, meeting_id: String, link_id: String) -> Result<Vec<serde_json::Value>, NotesError> {
    let link: Option<NoteLink> = sqlx::query_as("SELECT * FROM meeting_note_links WHERE meeting_id = ? AND id = ? AND EXISTS(SELECT 1 FROM meetings WHERE id=meeting_note_links.meeting_id AND status <> 'deleting')")
        .bind(&meeting_id).bind(&link_id).fetch_optional(state.db_manager.pool()).await?;
    let Some(link) = link else { return Err(NotesError::new("not_found", "Timestamp marker not found")); };
    let rows: Vec<(String, String, f64, f64, Option<String>)> = sqlx::query_as("SELECT t.id,t.transcript,t.audio_start_time,COALESCE(s.timeline_offset,0.0),s.folder_path FROM transcripts t LEFT JOIN recording_sessions s ON s.id=t.recording_session_id AND s.meeting_id=t.meeting_id WHERE t.meeting_id = ? AND t.recording_session_id = ? AND t.audio_start_time <= ? AND t.audio_end_time >= ? ORDER BY t.audio_start_time DESC LIMIT 3")
        .bind(&meeting_id).bind(&link.recording_session_id).bind(link.recording_offset).bind(link.recording_offset)
        .fetch_all(state.db_manager.pool()).await?;
    Ok(rows.into_iter().map(|(id, text, time, offset, folder)| serde_json::json!({"id": id, "text": text, "audio_start_time": time,"timeline_offset":offset,"recording_session_id":link.recording_session_id,"recording_folder":folder})).collect())
}

#[tauri::command]
pub async fn api_export_meeting_notes(app: AppHandle, state: tauri::State<'_, AppState>, meeting_id: String, format: String) -> Result<(), NotesError> {
    use tauri_plugin_dialog::DialogExt;
    let doc = load(state.db_manager.pool(), &meeting_id).await?.ok_or_else(|| NotesError::new("not_found", "Write and save notes before exporting"))?;
    let (extension, content) = match format.as_str() {
        "markdown" => ("md", doc.notes_markdown.ok_or_else(|| NotesError::new("conversion", "Markdown conversion is unavailable. Export JSON to preserve all notes."))?),
        "json" => ("json", serde_json::json!({"schema_version": 1, "meeting_id": meeting_id, "revision": doc.revision, "blocks": serde_json::from_str::<serde_json::Value>(doc.notes_json.as_deref().unwrap_or("[]")).map_err(|_| NotesError::new("invalid_document", "Stored note JSON is malformed"))?, "timestamp_links": doc.links}).to_string()),
        _ => return Err(NotesError::new("invalid_format", "Unsupported export format")),
    };
    tauri::async_runtime::spawn_blocking(move || {
        let path = app.dialog().file().set_file_name(format!("meeting-notes.{extension}"))
            .add_filter("Meeting notes", &[extension]).blocking_save_file();
        if let Some(path) = path {
            let path = path.into_path().map_err(|e| NotesError::new("export", e.to_string()))?;
            super::export::atomic_write(&path, content.as_bytes()).map_err(|e| NotesError::new("export", e.message))?;
        }
        Ok::<(), NotesError>(())
    }).await.map_err(|e| NotesError::new("export", e.to_string()))??;
    Ok(())
}
