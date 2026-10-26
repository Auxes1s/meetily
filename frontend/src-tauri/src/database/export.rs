//! Portable meeting records. Export reads canonical data without adopting legacy
//! summaries, indexing, or changing document/approval state.
use super::{notes_commands::NoteLink, repositories::enhancement::EnhancementVersion};
use crate::{
    audio::source::AudioSource,
    state::AppState,
    summary::grounding::{self, Source, SourceSnapshot},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, SqlitePool};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

mod render;
#[cfg(test)]
mod tests;

const TRANSCRIPT_PAGE_SIZE: i64 = 500;

#[derive(Debug, Serialize)]
pub struct ExportError {
    pub code: &'static str,
    pub message: String,
}
impl ExportError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
impl From<sqlx::Error> for ExportError {
    fn from(_: sqlx::Error) -> Self {
        Self::new(
            "database",
            "The meeting export could not read saved data. Please retry.",
        )
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Json,
}
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentSelection {
    Original,
    Approved,
    Both,
}
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub meeting_id: String,
    pub format: ExportFormat,
    #[serde(default = "original_selection")]
    pub document: DocumentSelection,
    #[serde(default)]
    pub include_transcript: bool,
    pub expected_approved_version_id: Option<String>,
}
fn original_selection() -> DocumentSelection {
    DocumentSelection::Original
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExportResult {
    Saved,
    Cancelled,
}

#[derive(Debug, Serialize)]
pub struct ExportOptions {
    pub meeting_id: String,
    pub title: String,
    pub status: String,
    pub approved_version_id: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExportMeeting {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub timezone: Option<String>,
    pub recording_session_id: Option<String>,
    #[sqlx(skip)]
    pub participants: Vec<String>,
    #[sqlx(skip)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExportTranscript {
    pub id: String,
    pub text: String,
    pub timestamp: String,
    pub recording_session_id: Option<String>,
    pub sequence_id: Option<i64>,
    pub audio_start_time: Option<f64>,
    pub audio_end_time: Option<f64>,
    pub duration: Option<f64>,
    pub timeline_offset: f64,
    pub summary: Option<String>,
    pub action_items: Option<String>,
    pub key_points: Option<String>,
    #[serde(skip)]
    speaker: Option<String>,
    #[sqlx(skip)]
    pub audio_source: AudioSource,
    #[sqlx(skip)]
    pub audio_source_label: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExportNotes {
    pub notes_json: Option<String>,
    pub notes_markdown: Option<String>,
    pub notes_plain_text: Option<String>,
    pub conversion_status: String,
    pub revision: i64,
    pub created_at: String,
    pub updated_at: String,
    #[sqlx(skip)]
    pub blocks: Option<Value>,
    #[sqlx(skip)]
    pub timestamp_links: Vec<ExportNoteLink>,
}

#[derive(Debug, Serialize)]
pub struct ExportNoteLink {
    #[serde(flatten)]
    pub link: NoteLink,
    pub passage: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ExportVersion {
    pub id: String,
    pub state: String,
    pub status: String,
    pub revision: i64,
    pub document_json: Option<String>,
    pub document: Option<Value>,
    pub source_snapshot_json: Option<String>,
    pub source_snapshot: Option<Value>,
    pub source_fingerprint: Option<String>,
    pub notes_revision: Option<i64>,
    pub template_id: String,
    pub instructions: String,
    pub provenance: Value,
    pub created_at: String,
    pub updated_at: String,
    pub error: Option<String>,
    pub source_stale: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExportSettings {
    pub template_id: String,
    pub instructions: String,
    pub revision: i64,
    pub approved_version_id: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExportLegacySummary {
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub result: Option<String>,
    pub result_backup: Option<String>,
    pub result_backup_timestamp: Option<String>,
    #[sqlx(skip)]
    pub verification_status: String,
    #[sqlx(skip)]
    pub document: Option<Value>,
    #[sqlx(skip)]
    pub backup_document: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct MeetingRecord {
    pub format: &'static str,
    pub schema_version: u32,
    pub exported_at: String,
    pub transcript_coverage: String,
    pub meeting: ExportMeeting,
    pub original_notes: Option<ExportNotes>,
    pub transcripts: Vec<ExportTranscript>,
    pub recording_sessions: Vec<ExportRecordingSession>,
    pub enhancement_settings: Option<ExportSettings>,
    pub versions: Vec<ExportVersion>,
    pub legacy_summary: Option<ExportLegacySummary>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportRecordingSession {
    pub id: String,
    pub ordinal: i64,
    pub status: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub timeline_offset: f64,
    pub duration_basis: String,
}
impl From<super::recording_sessions::RecordingSession> for ExportRecordingSession {
    fn from(s:super::recording_sessions::RecordingSession)->Self {
        Self{id:s.id,ordinal:s.ordinal,status:s.status,started_at:s.started_at,ended_at:s.ended_at,duration_seconds:s.duration_seconds,timeline_offset:s.timeline_offset,duration_basis:s.duration_basis}
    }
}

fn coverage(status: &str) -> &'static str {
    match status {
        "recording" | "finalizing" => "incomplete_active",
        "interrupted" => "incomplete_interrupted",
        _ => "committed_available",
    }
}

fn parse_stored(raw: Option<&str>, label: &str, warnings: &mut Vec<String>) -> Option<Value> {
    let raw = raw?;
    let parse = || -> Result<Value, ()> {
        let mut value: Value = serde_json::from_str(raw).map_err(|_| ())?;
        // Compatibility with historical double-encoded summary documents.
        for _ in 0..2 {
            if let Some(encoded) = value.as_str() {
                value = serde_json::from_str(encoded).map_err(|_| ())?;
            } else {
                break;
            }
        }
        if value.is_object() || value.is_array() {
            Ok(value)
        } else {
            Err(())
        }
    };
    match parse() {
        Ok(value) => Some(value),
        Err(()) => {
            warnings.push(format!(
                "{label} could not be parsed; its raw saved text is preserved."
            ));
            None
        }
    }
}

/// Read every page in the same snapshot, including empty/legacy passages. UI
/// pagination and the derived search index are deliberately not export sources.
pub async fn read_record(
    pool: &SqlitePool,
    meeting_id: &str,
) -> Result<MeetingRecord, ExportError> {
    read_record_snapshot(pool.begin().await?, meeting_id).await
}

async fn read_record_snapshot(
    mut tx: sqlx::Transaction<'_, sqlx::Sqlite>,
    meeting_id: &str,
) -> Result<MeetingRecord, ExportError> {
    let mut meeting: ExportMeeting = sqlx::query_as("SELECT id,title,created_at,updated_at,status,started_at,ended_at,timezone,recording_session_id FROM meetings WHERE id=? AND status <> 'deleting' AND NOT EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id=meetings.id)")
        .bind(meeting_id).fetch_optional(&mut *tx).await?.ok_or_else(|| ExportError::new("not_found", "Meeting is unavailable or deletion is pending."))?;
    let exported_at = Utc::now().to_rfc3339();
    meeting.participants = sqlx::query_scalar("SELECT display_text FROM meeting_participants WHERE meeting_id=? ORDER BY normalized,display_text")
        .bind(meeting_id).fetch_all(&mut *tx).await?;
    meeting.tags = sqlx::query_scalar(
        "SELECT display_text FROM meeting_tags WHERE meeting_id=? ORDER BY normalized,display_text",
    )
    .bind(meeting_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut warnings = Vec::new();
    let mut notes: Option<ExportNotes> = sqlx::query_as("SELECT notes_json,notes_markdown,notes_plain_text,conversion_status,revision,created_at,updated_at FROM meeting_notes WHERE meeting_id=?")
        .bind(meeting_id).fetch_optional(&mut *tx).await?;
    let links: Vec<NoteLink> = sqlx::query_as("SELECT id,block_id,recording_session_id,recording_offset,transcript_id FROM meeting_note_links WHERE meeting_id=? ORDER BY recording_offset,id")
        .bind(meeting_id).fetch_all(&mut *tx).await?;
    let mut transcripts = Vec::new();
    loop {
        let page: Vec<ExportTranscript> = sqlx::query_as("SELECT t.id,t.transcript AS text,t.timestamp,t.recording_session_id,t.sequence_id,t.audio_start_time,t.audio_end_time,t.duration,t.summary,t.action_items,t.key_points,t.speaker,COALESCE(s.timeline_offset,0.0) AS timeline_offset FROM transcripts t LEFT JOIN recording_sessions s ON s.id=t.recording_session_id AND s.meeting_id=t.meeting_id WHERE t.meeting_id=? ORDER BY COALESCE(s.ordinal,1),COALESCE(t.audio_start_time,1e99),COALESCE(t.sequence_id,9223372036854775807),t.timestamp,t.id LIMIT ? OFFSET ?")
            .bind(meeting_id).bind(TRANSCRIPT_PAGE_SIZE).bind(transcripts.len() as i64).fetch_all(&mut *tx).await?;
        let complete = page.len() < TRANSCRIPT_PAGE_SIZE as usize;
        transcripts.extend(page);
        if complete {
            break;
        }
    }
    for passage in &mut transcripts {
        passage.audio_source = AudioSource::from_storage(passage.speaker.as_deref());
        passage.audio_source_label = render::channel(passage.audio_source).into();
    }
    if let Some(notes) = &mut notes {
        notes.blocks = parse_stored(
            notes.notes_json.as_deref(),
            "Original blocks",
            &mut warnings,
        );
        if notes.blocks.as_ref().is_some_and(|v| !v.is_array()) {
            warnings
                .push("Original blocks are not an array; the raw saved text is preserved.".into());
            notes.blocks = None;
        }
        notes.timestamp_links = links.into_iter().map(|link| {
            let passage = transcripts.iter().find(|p| p.recording_session_id.as_deref() == Some(&link.recording_session_id) && link.transcript_id.as_deref() == Some(&p.id))
                .or_else(|| transcripts.iter().rev().find(|p| p.recording_session_id.as_deref() == Some(&link.recording_session_id)
                    && p.audio_start_time.is_some_and(|t| t <= link.recording_offset)
                    && p.audio_end_time.is_some_and(|t| t >= link.recording_offset)))
                .map(|p| json!({"id":p.id,"text":p.text,"audio_start_time":p.audio_start_time,"audio_end_time":p.audio_end_time,"audio_source":p.audio_source,"audio_source_label":p.audio_source_label,"recording_session_id":p.recording_session_id,"timeline_offset":p.timeline_offset}));
            ExportNoteLink { link, passage }
        }).collect();
    }
    let settings: Option<ExportSettings> = sqlx::query_as("SELECT template_id,instructions,revision,approved_version_id FROM meeting_enhancement_settings WHERE meeting_id=?")
        .bind(meeting_id).fetch_optional(&mut *tx).await?;
    let stored_versions: Vec<EnhancementVersion> = sqlx::query_as(
        "SELECT * FROM meeting_enhancement_versions WHERE meeting_id=? ORDER BY created_at,id",
    )
    .bind(meeting_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut legacy: Option<ExportLegacySummary> = sqlx::query_as("SELECT status,created_at,updated_at,result,result_backup,result_backup_timestamp FROM summary_processes WHERE meeting_id=?")
        .bind(meeting_id).fetch_optional(&mut *tx).await?;
    // Never hold the database transaction across serialization or the Save dialog.
    let recording_sessions:Vec<super::recording_sessions::RecordingSession>=sqlx::query_as("SELECT * FROM recording_sessions WHERE meeting_id=? ORDER BY ordinal")
        .bind(meeting_id).fetch_all(&mut *tx).await?;
    tx.commit().await?;
    let transcript_coverage = if recording_sessions.iter().any(|s|matches!(s.status.as_str(),"starting"|"recording"|"finalizing")) {"incomplete_active"}
        else if recording_sessions.iter().any(|s|s.status=="interrupted") {"incomplete_interrupted"}
        else {coverage(&meeting.status)}.to_owned();
    let current_fingerprint =
        current_fingerprint(notes.as_ref(), &transcripts, &transcript_coverage);
    let versions = stored_versions
        .into_iter()
        .map(|v| {
            let document = parse_stored(
                v.document_json.as_deref(),
                &format!("Version {} document", v.id),
                &mut warnings,
            );
            let mut source_snapshot = parse_stored(
                v.source_snapshot_json.as_deref(),
                &format!("Version {} sources", v.id),
                &mut warnings,
            );
            let source_stale = match (
                v.source_snapshot_json.as_deref(),
                current_fingerprint.as_deref(),
            ) {
                (Some(raw), Some(current)) => Some(
                    serde_json::from_str::<SourceSnapshot>(raw)
                        .map(|s| s.source_fingerprint != current)
                        .unwrap_or(true),
                ),
                _ if v.state == "legacy" => None,
                _ => Some(true),
            };
            // Keep exact stored provenance above, and normalize the readable projection.
            if let Some(sources) = source_snapshot
                .as_mut()
                .and_then(|s| s.get_mut("sources"))
                .and_then(Value::as_array_mut)
            {
                for source in sources {
                    if source.get("kind").and_then(Value::as_str) == Some("transcript") {
                        let origin = AudioSource::from_storage(
                            source.get("audio_source").and_then(Value::as_str),
                        );
                        source["audio_source"] = json!(origin);
                        source["audio_source_label"] = json!(render::channel(origin));
                    }
                }
            }
            let mut provenance = serde_json::Map::new();
            if let Some(config) = v
                .provider_config_json
                .as_deref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
            {
                for key in [
                    "provider",
                    "model",
                    "summary_language",
                    "context_size",
                    "max_tokens",
                    "temperature",
                    "top_p",
                ] {
                    if let Some(value) = config
                        .get(key)
                        .filter(|value| value.is_string() || value.is_number() || value.is_null())
                    {
                        provenance.insert(key.into(), value.clone());
                    }
                }
                if let Some(template) = config.get("template").and_then(|v| {
                    serde_json::from_value::<crate::summary::templates::Template>(v.clone()).ok()
                }) {
                    if let Ok(value) = serde_json::to_value(template) {
                        provenance.insert("template".into(), value);
                    }
                }
            }
            ExportVersion {
                id: v.id,
                state: v.state,
                status: v.status,
                revision: v.revision,
                document_json: v.document_json,
                document,
                source_snapshot_json: v.source_snapshot_json,
                source_snapshot,
                source_fingerprint: v.source_fingerprint,
                notes_revision: v.notes_revision,
                template_id: v.template_id,
                instructions: v.instructions,
                provenance: Value::Object(provenance),
                created_at: v.created_at,
                updated_at: v.updated_at,
                // Stored runtime failures can contain model/cache paths. Document
                // content is preserved, while application diagnostics stay local.
                error: v.error.map(|_| {
                    "Generation reported an error. Review this version in Meetily for details."
                        .into()
                }),
                source_stale,
            }
        })
        .collect();
    if let Some(legacy) = &mut legacy {
        legacy.verification_status = "unverified".into();
        legacy.document = parse_stored(legacy.result.as_deref(), "Legacy summary", &mut warnings);
        legacy.backup_document = parse_stored(
            legacy.result_backup.as_deref(),
            "Legacy summary backup",
            &mut warnings,
        );
    }
    Ok(MeetingRecord {
        format: "meetily.meeting-record",
        schema_version: 2,
        exported_at,
        transcript_coverage,
        meeting,
        original_notes: notes,
        transcripts,
        recording_sessions:recording_sessions.into_iter().map(ExportRecordingSession::from).collect(),
        enhancement_settings: settings,
        versions,
        legacy_summary: legacy,
        warnings,
    })
}

fn current_fingerprint(
    notes: Option<&ExportNotes>,
    transcripts: &[ExportTranscript],
    coverage: &str,
) -> Option<String> {
    let mut sources = Vec::new();
    if let Some(raw) = notes.and_then(|n| n.notes_json.as_deref()) {
        let blocks: Value = serde_json::from_str(raw).ok()?;
        grounding::note_sources(blocks.as_array()?, &mut sources).ok()?;
    }
    for p in transcripts.iter().filter(|p| !p.text.trim().is_empty()) {
        sources.push(Source {
            source_id: format!("transcript:{}", p.id),
            kind: "transcript".into(),
            text: p.text.clone(),
            fingerprint: grounding::fingerprint(&p.text),
            audio_start_time: p.audio_start_time,
            audio_end_time: p.audio_end_time,
            audio_source: p.speaker.clone(),
        });
    }
    Some(grounding::fingerprint(&json!({"notes_revision":notes.map(|n| n.revision).unwrap_or(0),"notes_json":notes.and_then(|n| n.notes_json.as_deref()),"sources":sources,"coverage":coverage}).to_string()))
}

impl MeetingRecord {
    fn approved(&self) -> Option<&ExportVersion> {
        let id = self
            .enhancement_settings
            .as_ref()?
            .approved_version_id
            .as_deref()?;
        self.versions
            .iter()
            .find(|v| v.id == id && v.state == "approved" && v.status == "ready")
    }

    pub fn content(&self, request: &ExportRequest) -> Result<Vec<u8>, ExportError> {
        match request.format {
            ExportFormat::Json => serde_json::to_vec_pretty(self).map_err(|_| {
                ExportError::new(
                    "serialization",
                    "The meeting record could not be serialized.",
                )
            }),
            ExportFormat::Markdown => {
                if request.document != DocumentSelection::Original
                    && request.expected_approved_version_id.as_deref()
                        != self.approved().map(|v| v.id.as_str())
                {
                    return Err(ExportError::new("approval_changed", "The approved version changed. Reopen export to select the current document."));
                }
                render::markdown(self, request).map(String::into_bytes)
            }
        }
    }
}

#[tauri::command]
pub async fn api_get_meeting_export_options(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
) -> Result<ExportOptions, ExportError> {
    let row: Option<(String,String,String,Option<String>)> = sqlx::query_as("SELECT m.id,m.title,m.status,(SELECT v.id FROM meeting_enhancement_settings s JOIN meeting_enhancement_versions v ON v.id=s.approved_version_id AND v.meeting_id=s.meeting_id WHERE s.meeting_id=m.id AND v.state='approved' AND v.status='ready') FROM meetings m WHERE m.id=? AND m.status <> 'deleting' AND NOT EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id=m.id)")
        .bind(&meeting_id).fetch_optional(state.db_manager.pool()).await?;
    let (meeting_id, title, status, approved_version_id) = row.ok_or_else(|| {
        ExportError::new(
            "not_found",
            "Meeting is unavailable or deletion is pending.",
        )
    })?;
    Ok(ExportOptions {
        meeting_id,
        title,
        status,
        approved_version_id,
    })
}

/// A failed write/flush/rename leaves an existing destination intact. The temp
/// file lives beside the destination, so persist never crosses filesystems.
pub(crate) fn atomic_write(path: &Path, content: &[u8]) -> Result<(), ExportError> {
    atomic_write_with(path, |file| file.write_all(content))
}
fn atomic_write_with(
    path: &Path,
    write: impl FnOnce(&mut std::fs::File) -> std::io::Result<()>,
) -> Result<(), ExportError> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let failure = || {
        ExportError::new(
            "write",
            "The export could not be saved. Check the destination and available space, then retry.",
        )
    };
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|_| failure())?;
    write(file.as_file_mut()).map_err(|_| failure())?;
    file.as_file().sync_all().map_err(|_| failure())?;
    file.persist(path).map_err(|_| failure())?;
    Ok(())
}

fn validate_destination(
    path: &Path,
    extension: &str,
    managed_roots: &[PathBuf],
) -> Result<(), ExportError> {
    if !path
        .extension()
        .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(extension))
    {
        return Err(ExportError::new(
            "destination",
            format!("Choose a .{extension} file for this export."),
        ));
    }
    // Resolve the parent even when the output does not exist yet. A symlinked
    // destination folder must not turn an export into an application-data write.
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let resolved_parent = parent
        .canonicalize()
        .map_err(|_| ExportError::new("destination", "The export folder is unavailable."))?;
    let resolved = resolved_parent.join(
        path.file_name()
            .ok_or_else(|| ExportError::new("destination", "Choose a local export file."))?,
    );
    let target = path.canonicalize().unwrap_or_else(|_| resolved.clone());
    if managed_roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| resolved.starts_with(&root) || target.starts_with(&root))
    {
        return Err(ExportError::new(
            "destination",
            "Choose an export folder outside application data and managed recording folders.",
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn api_export_meeting(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    request: ExportRequest,
) -> Result<ExportResult, ExportError> {
    use tauri_plugin_dialog::DialogExt;
    let record = read_record(state.db_manager.pool(), &request.meeting_id).await?;
    let content = record.content(&request)?;
    let extension = if request.format == ExportFormat::Markdown {
        "md"
    } else {
        "json"
    };
    let filename = render::filename(&record.meeting.title, extension);
    let path = if crate::workflow_smoke::enabled(&app) {
        let root = crate::workflow_smoke::run_dir(&app).map_err(|error|ExportError::new("destination",error))?;
        let folder=root.join("exports");std::fs::create_dir_all(&folder).map_err(|_|ExportError::new("write","Cannot create test export directory"))?;
        folder.join(filename)
    } else {
    let dialog_app = app.clone();
    let destination = tauri::async_runtime::spawn_blocking(move || {
        dialog_app
            .dialog()
            .file()
            .set_file_name(filename)
            .add_filter(
                if extension == "md" {
                    "Markdown notes"
                } else {
                    "JSON meeting record"
                },
                &[extension],
            )
            .blocking_save_file()
    })
    .await
    .map_err(|_| ExportError::new("dialog", "The Save dialog could not be opened."))?;
    let Some(destination) = destination else {
        return Ok(ExportResult::Cancelled);
    };
    let chosen = destination
        .into_path()
        .map_err(|_| ExportError::new("destination", "Choose a local file for this export."))?;
    chosen

    };
    // Serialize publication with deletion, but never keep this lease while the
    // user is choosing a destination. Exports remain user-owned after saving.
    let _lease = super::deletion::acquire_meeting_artifact_write(
        state.db_manager.pool(),
        &request.meeting_id,
    )
    .await
    .map_err(|_| {
        ExportError::new(
            "not_found",
            "Meeting is unavailable or deletion is pending.",
        )
    })?;
    let mut managed_roots = vec![app.path().app_data_dir().map_err(|_| {
        ExportError::new("destination", "Application storage could not be checked.")
    })?];
    let folders: Vec<String> = sqlx::query_scalar("SELECT DISTINCT folder_path FROM meetings WHERE folder_path IS NOT NULL AND folder_path <> ''")
        .fetch_all(state.db_manager.pool()).await?;
    managed_roots.extend(folders.into_iter().map(PathBuf::from));
    tauri::async_runtime::spawn_blocking(move || {
        validate_destination(&path, extension, &managed_roots)?;
        atomic_write(&path, &content)
    })
    .await
    .map_err(|_| ExportError::new("write", "The export could not be saved."))??;
    Ok(ExportResult::Saved)
}
