//! Retryable removal of app-managed meeting data. This is logical deletion, not
//! a promise of forensic erasure of SSD, OS, or user-owned export copies.
use super::manager::DatabaseManager;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions}, Row, SqliteConnection, SqlitePool};
use std::{collections::{BTreeMap, HashMap, HashSet}, fs, path::{Component, Path, PathBuf}, sync::{Arc, Mutex, RwLock, RwLockReadGuard, Weak}, time::Duration};
use tauri::{AppHandle, Manager, Runtime};

static DELETION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static ARTIFACT_LOCKS: once_cell::sync::Lazy<Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>> = once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));
static BLOCKED_ARTIFACTS: once_cell::sync::Lazy<RwLock<HashSet<String>>> = once_cell::sync::Lazy::new(|| RwLock::new(HashSet::new()));

/// Serialize final file writes with deletion for this meeting. New imports use
/// this before inserting their meeting; existing writers must also validate the
/// canonical meeting after acquiring it. Guards never span model inference.
pub(crate) async fn lock_meeting_artifacts(meeting_id: &str) -> tokio::sync::OwnedMutexGuard<()> {
    let lock = {
        let mut locks = ARTIFACT_LOCKS.lock().unwrap_or_else(|e| e.into_inner());
        locks.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock) = locks.get(meeting_id).and_then(Weak::upgrade) { lock }
        else {
            let lock = Arc::new(tokio::sync::Mutex::new(()));
            locks.insert(meeting_id.to_owned(), Arc::downgrade(&lock)); lock
        }
    };
    lock.lock_owned().await
}

pub(crate) async fn acquire_meeting_artifact_write(pool: &SqlitePool, meeting_id: &str) -> Result<tokio::sync::OwnedMutexGuard<()>, String> {
    let guard = lock_meeting_artifacts(meeting_id).await;
    assert_meeting_writable(pool, meeting_id).await?;
    Ok(guard)
}

/// Synchronous recording callbacks cannot query SQLite. Their shared read lease
/// makes committing intent wait for admitted file writes, then blocks late
/// cloned sinks/checkpoint callbacks. After restart, canonical validation at
/// recording startup prevents a deleted identity from creating a new sink.
pub(crate) fn recording_artifact_write(meeting_id: Option<&str>) -> Result<RwLockReadGuard<'static, HashSet<String>>, String> {
    let guard = BLOCKED_ARTIFACTS.read().map_err(|_| "Recording artifact lifecycle is unavailable".to_string())?;
    if meeting_id.is_some_and(|id| guard.contains(id)) { return Err("Meeting deletion is pending or complete".into()); }
    Ok(guard)
}

fn block_artifacts(meeting_id: &str) {
    BLOCKED_ARTIFACTS.write().unwrap_or_else(|e| e.into_inner()).insert(meeting_id.to_owned());
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeletionStatus {
    pub job_id: String,
    pub meeting_id: String,
    pub recording_session_id: Option<String>,
    #[sqlx(skip)]
    pub recording_session_ids: Vec<String>,
    pub state: String,
    pub native_complete: bool,
    pub caches_complete: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OwnedArtifacts {
    folder: PathBuf,
    root: PathBuf,
    files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}

pub async fn is_meeting_deleted(pool: &SqlitePool, meeting_id: &str) -> Result<bool, String> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = ?)")
        .bind(meeting_id).fetch_one(pool).await.map_err(|e| e.to_string())
}

pub async fn assert_meeting_writable(pool: &SqlitePool, meeting_id: &str) -> Result<(), String> {
    let writable: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meetings WHERE id = ? AND status <> 'deleting' AND NOT EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = ?))")
        .bind(meeting_id).bind(meeting_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
    if writable { Ok(()) } else { Err("Meeting is unavailable or deletion is pending or complete".into()) }
}

pub async fn status(pool: &SqlitePool, job_id: &str) -> Result<DeletionStatus, String> {
    let mut status:DeletionStatus=sqlx::query_as("SELECT job_id, meeting_id, recording_session_id, CASE WHEN native_complete = 1 AND caches_complete = 1 THEN 'complete' ELSE 'pending' END AS state, native_complete, caches_complete, last_error FROM meeting_deletion_jobs WHERE job_id = ?")
        .bind(job_id).fetch_optional(pool).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Deletion job not found".to_owned())?;
    let json:Option<String>=sqlx::query_scalar("SELECT recording_session_ids_json FROM meeting_deletion_jobs WHERE job_id=?").bind(job_id).fetch_one(pool).await.map_err(|e|e.to_string())?;
    status.recording_session_ids=json.map(|json|serde_json::from_str(&json)).transpose().map_err(|_|"Stored session identities are invalid")?.unwrap_or_default();
    Ok(status)
}

/// Capture deletion intent and identities before any irreversible cleanup. A
/// transaction obtains the write lock before checking recording/finalizing.
pub async fn begin(pool: &SqlitePool, meeting_id: &str) -> Result<DeletionStatus, String> {
    let _artifacts = lock_meeting_artifacts(meeting_id).await;
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.map_err(|e| e.to_string())?;
    if let Some(job_id) = sqlx::query_scalar::<_, String>("SELECT job_id FROM meeting_deletion_jobs WHERE meeting_id = ?")
        .bind(meeting_id).fetch_optional(&mut *tx).await.map_err(|e| e.to_string())? {
        tx.commit().await.map_err(|e| e.to_string())?;
        block_artifacts(meeting_id);
        return status(pool, &job_id).await;
    }
    let meeting = sqlx::query("SELECT status, folder_path, recording_session_id FROM meetings WHERE id = ?")
        .bind(meeting_id).fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Meeting not found".to_string())?;
    let state: String = meeting.get("status");
    if matches!(state.as_str(), "recording" | "finalizing") {
        return Err("End the recording and wait for finalization before deleting this meeting".into());
    }
    let session: Option<String> = meeting.get("recording_session_id");
    let folder: Option<String> = meeting.get("folder_path");
    let sessions:Vec<(String,Option<String>,String)>=sqlx::query_as("SELECT id,folder_path,status FROM recording_sessions WHERE meeting_id=? ORDER BY ordinal")
        .bind(meeting_id).fetch_all(&mut *tx).await.map_err(|e|e.to_string())?;
    if sessions.iter().any(|s|matches!(s.2.as_str(),"starting"|"recording"|"finalizing")) {return Err("End recording initialization or finalization before deleting this meeting".into());}
    let mut folders:Vec<(String,Option<String>)>=sessions.iter().filter_map(|s|s.1.clone().map(|folder|(folder,Some(s.0.clone())))).collect();
    if let Some(folder)=folder.clone() {
        if !folders.iter().any(|f|f.0==folder) {folders.push((folder,session.clone()));}
    }
    let session_ids:Vec<String>=sessions.iter().map(|s|s.0.clone()).chain(session.clone()).collect();
    let now = chrono::Utc::now().to_rfc3339();
    let job_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("UPDATE meetings SET status = 'deleting' WHERE id = ?")
        .bind(meeting_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO meeting_deletion_tombstones(meeting_id, recording_session_id, deleted_at) VALUES (?, ?, ?)")
        .bind(meeting_id).bind(&session).bind(&now).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO meeting_deletion_jobs(job_id,meeting_id,recording_session_id,folder_path,created_at,updated_at) VALUES (?,?,?,?,?,?)")
        .bind(&job_id).bind(meeting_id).bind(&session).bind(&folder).bind(&now).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("UPDATE meeting_deletion_jobs SET session_folders_json=?,recording_session_ids_json=? WHERE job_id=?")
        .bind(serde_json::to_string(&folders).map_err(|e|e.to_string())?).bind(serde_json::to_string(&session_ids).map_err(|e|e.to_string())?).bind(&job_id)
        .execute(&mut *tx).await.map_err(|e|e.to_string())?;
    for id in &session_ids {
        sqlx::query("INSERT OR IGNORE INTO meeting_deletion_sessions(id,meeting_id,deleted_at) VALUES(?,?,?)")
            .bind(id).bind(meeting_id).bind(&now).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    }
    // Install table-local guards in the same transaction. Dynamic inspection also
    // protects enhancement/library tables without coupling shipped migrations.
    let tables = meeting_tables(&mut tx).await?;
    for table in tables {
        // Source DELETE triggers may enqueue disposable indexing work while the
        // meeting still exists. It contains no source text and cascades away at
        // final removal; guarding it would incorrectly block cleanup itself.
        if table == "meeting_search_dirty" { continue; }
        for operation in ["INSERT", "UPDATE"] {
            let name = quoted(&format!("deletion_guard_{table}_{}", operation.to_lowercase()));
            let predicate = if operation == "UPDATE" { "meeting_id = NEW.meeting_id OR meeting_id = OLD.meeting_id" } else { "meeting_id = NEW.meeting_id" };
            let sql = format!("CREATE TRIGGER IF NOT EXISTS {name} BEFORE {operation} ON {} WHEN EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE {predicate}) BEGIN SELECT RAISE(ABORT, 'Meeting deletion is pending or complete'); END", quoted(&table));
            sqlx::query(&sql).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        }
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    block_artifacts(meeting_id);
    status(pool, &job_id).await
}

fn quoted(identifier: &str) -> String { format!("\"{}\"", identifier.replace('"', "\"\"")) }

async fn meeting_tables(conn: &mut SqliteConnection) -> Result<Vec<String>, String> {
    let tables: Vec<(String,)> = sqlx::query_as("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'meeting_deletion_%' AND name NOT LIKE 'meeting_search_fts%' AND sql NOT LIKE 'CREATE VIRTUAL TABLE%'")
        .fetch_all(&mut *conn).await.map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for (name,) in tables {
        let columns = sqlx::query(&format!("PRAGMA table_info({})", quoted(&name))).fetch_all(&mut *conn).await.map_err(|e| e.to_string())?;
        if columns.iter().any(|column| column.get::<String, _>("name") == "meeting_id") { result.push(name); }
    }
    // Deleting links before transcripts avoids ON DELETE SET NULL mutations in
    // unrelated rows if an invalid legacy cross-meeting link exists.
    result.sort_by_key(|name| if name == "meeting_note_links" { 0 } else if name.contains("settings") { 1 } else if name == "transcripts" { 3 } else { 2 });
    Ok(result)
}

async fn remove_rows(conn: &mut SqliteConnection, meeting_id: &str) -> Result<(), String> {
    for table in meeting_tables(conn).await? {
        sqlx::query(&format!("DELETE FROM {} WHERE meeting_id = ?", quoted(&table)))
            .bind(meeting_id).execute(&mut *conn).await.map_err(|e| e.to_string())?;
    }
    sqlx::query("DELETE FROM meetings WHERE id = ?").bind(meeting_id).execute(&mut *conn).await.map_err(|e| e.to_string())?;
    Ok(())
}

fn no_symlinks(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for component in path.components() {
        if component == Component::ParentDir { return Err("Managed path contains parent traversal".into()); }
        current.push(component.as_os_str());
        if fs::symlink_metadata(&current).map_err(|_| "Cannot inspect managed path".to_string())?.file_type().is_symlink() {
            return Err("Managed cleanup refuses symbolic links".into());
        }
    }
    Ok(())
}

fn owned_file(path: &Path, folder: &Path, root: &Path) -> Result<(), String> {
    no_symlinks(path)?;
    let canonical = fs::canonicalize(path).map_err(|_| "Cannot resolve managed artifact".to_string())?;
    if !canonical.starts_with(folder) || !canonical.starts_with(root) || canonical == folder {
        return Err("Managed artifact escapes its meeting directory".into());
    }
    Ok(())
}

/// Identity plus a trusted recording root establishes ownership; a database
/// folder string alone never authorizes recursive deletion.
fn managed_folder(folder: &Path, roots: &[PathBuf]) -> Result<Option<(PathBuf, PathBuf)>, String> {
    if !folder.is_absolute() { return Err("Managed recording folder must be absolute".into()); }
    // Resolve the trusted root first. System aliases such as macOS /var ->
    // /private/var are allowed above it; links inside a recording root are not.
    let (root, relative) = roots.iter().filter_map(|root| {
        let canonical = fs::canonicalize(root).ok()?;
        let relative = folder.strip_prefix(root).or_else(|_| folder.strip_prefix(&canonical)).ok()?.to_path_buf();
        Some((canonical, relative))
    }).find(|(_, relative)| !relative.as_os_str().is_empty())
        .ok_or_else(|| "Recording folder is outside the managed recording roots".to_string())?;
    if relative.components().any(|c| !matches!(c, Component::Normal(_))) { return Err("Managed recording folder contains parent traversal".into()); }
    let resolved_folder = root.join(relative);
    match fs::symlink_metadata(&resolved_folder) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Cannot inspect recording folder".into()),
        Ok(_) => {},
    }
    no_symlinks(&resolved_folder)?;
    let folder = fs::canonicalize(&resolved_folder).map_err(|_| "Cannot resolve recording folder".to_string())?;
    Ok(Some((root, folder)))
}

fn capture_artifacts(folder: &Path, roots: &[PathBuf], meeting_id: &str, session_id: Option<&str>) -> Result<Option<OwnedArtifacts>, String> {
    capture_artifacts_with_legacy_ownership(folder, roots, meeting_id, session_id, false)
}

fn capture_artifacts_with_legacy_ownership(folder: &Path, roots: &[PathBuf], meeting_id: &str, session_id: Option<&str>, legacy_verified: bool) -> Result<Option<OwnedArtifacts>, String> {
    let Some((root, folder)) = managed_folder(folder, roots)? else { return Ok(None); };
    let metadata_path = folder.join("metadata.json");
    owned_file(&metadata_path, &folder, &root)?;
    let metadata: serde_json::Value = serde_json::from_slice(&fs::read(&metadata_path).map_err(|_| "Cannot read recording ownership metadata".to_string())?)
        .map_err(|_| "Recording ownership metadata is invalid".to_string())?;
    let missing_identity = metadata.is_object() && metadata.get("meeting_id").is_none_or(|v| v.is_null());
    if metadata.get("meeting_id").and_then(|v| v.as_str()) != Some(meeting_id) && !(legacy_verified && missing_identity) {
        return Err("Recording ownership is unverified: meeting identity does not match".into());
    }
    if let Some(stored_session) = metadata.get("recording_session_id").and_then(|v| v.as_str()) {
        if session_id != Some(stored_session) { return Err("Recording session ownership does not match".into()); }
    }
    let mut files = Vec::new();
    for field in ["audio_file", "transcript_file"] {
        if let Some(name) = metadata.get(field).and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
            let path = Path::new(name);
            if path.components().count() != 1 || !matches!(path.components().next(), Some(Component::Normal(_))) {
                return Err("Recording metadata contains an unsafe artifact name".into());
            }
            // Imports own their copied audio inside this directory, never the
            // original selected file. Exports/model artifacts are not audio.
            if field == "audio_file" && !path.extension().and_then(|s| s.to_str()).is_some_and(|extension| crate::audio::constants::AUDIO_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())) {
                return Err("Recording metadata contains an unsupported audio artifact".into());
            }
            if field == "transcript_file" && name != "transcripts.json" { return Err("Recording metadata contains an unrecognized transcript artifact".into()); }
            files.push(folder.join(path));
        }
    }
    files.extend(["transcripts.json", ".transcripts.json.tmp", ".metadata.json.tmp"].map(|name| folder.join(name)));
    let mut directories = Vec::new();
    let checkpoints = folder.join(".checkpoints");
    if checkpoints.try_exists().map_err(|_| "Cannot inspect checkpoint directory".to_string())? {
        owned_file(&checkpoints, &folder, &root)?;
        for entry in fs::read_dir(&checkpoints).map_err(|_| "Cannot enumerate checkpoints".to_string())? {
            let path = entry.map_err(|_| "Cannot inspect checkpoint".to_string())?.path();
            let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
            let chunk = name.strip_prefix("audio_chunk_").and_then(|v| v.strip_suffix(".mp4")).is_some_and(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()));
            let raw = name.strip_prefix("audio_raw_").and_then(|v|v.strip_suffix(".pcm"))
                .is_some_and(|v| !v.is_empty() && v.bytes().all(|c|c.is_ascii_digit()));
            let encoded_temp = name.strip_prefix("audio_chunk_").and_then(|v|v.strip_suffix(".mp4.tmp"))
                .is_some_and(|v| !v.is_empty() && v.bytes().all(|c|c.is_ascii_digit()));
            let pending = raw || encoded_temp || crate::audio::transcription::streaming::is_pending_filename(name);
            if !(chunk || pending || name == "concat_list.txt") { return Err("Checkpoint directory contains unrecognized artifacts; cleanup is pending".into()); }
            files.push(path);
        }
        directories.push(checkpoints);
    }
    for entry in fs::read_dir(&folder).map_err(|_| "Cannot enumerate recording artifacts".to_string())? {
        let path = entry.map_err(|_| "Cannot inspect recording artifact".to_string())?.path();
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("");
        if name.starts_with(".metadata.json.") && name.ends_with(".tmp") { files.push(path); }
    }
    files.sort(); files.dedup();
    // Metadata is last so a failed first attempt still carries ownership proof.
    files.push(metadata_path);
    for path in &files {
        if path.try_exists().map_err(|_| "Cannot inspect managed artifact".to_string())? {
            owned_file(path, &folder, &root)?;
            if !path.is_file() { return Err("Managed artifact is not a regular file".into()); }
        }
    }
    Ok(Some(OwnedArtifacts { folder, root, files, directories }))
}

/// Older recorders wrote null meeting IDs and generated different transcript IDs
/// in SQLite. Adopt only a uniquely registered folder whose saved passage text
/// and timestamps independently match this meeting. An explicit foreign identity
/// is never adopted, and the ordinary artifact/path checks still apply.
async fn capture_deletion_artifacts(pool: &SqlitePool, folder: &Path, roots: &[PathBuf], meeting_id: &str, session_id: Option<&str>) -> Result<Option<OwnedArtifacts>, String> {
    match capture_artifacts(folder, roots, meeting_id, session_id) {
        Err(error) if error == "Recording ownership is unverified: meeting identity does not match" => {},
        result => return result,
    }
    let (root, canonical_folder) = managed_folder(folder, roots)?.ok_or("Recording folder is unavailable")?;
    let metadata_path = canonical_folder.join("metadata.json");
    owned_file(&metadata_path, &canonical_folder, &root)?;
    let metadata: serde_json::Value = serde_json::from_slice(&fs::read(&metadata_path).map_err(|_| "Cannot read recording ownership metadata")?)
        .map_err(|_| "Recording ownership metadata is invalid")?;
    if !metadata.is_object() || !metadata.get("meeting_id").is_none_or(|v| v.is_null()) {
        return Err("Recording ownership is unverified: meeting identity does not match".into());
    }
    if metadata.get("version").and_then(|v| v.as_str()) != Some("1.0") ||
        metadata.get("status").and_then(|v| v.as_str()) != Some("completed") {
        return Err("Legacy recording ownership is unverified: unsupported metadata".into());
    }
    let registered:Vec<String>=sqlx::query_scalar("SELECT folder_path FROM meetings WHERE id=? AND status='deleting' AND folder_path IS NOT NULL UNION SELECT folder_path FROM recording_sessions WHERE meeting_id=? AND folder_path IS NOT NULL")
        .bind(meeting_id).bind(meeting_id).fetch_all(pool).await.map_err(|e|e.to_string())?;
    if !registered.iter().any(|path|fs::canonicalize(path).ok().as_ref()==Some(&canonical_folder)) {
        return Err("Legacy recording ownership is unverified: folder is not registered to this meeting".into());
    }
    let other_folders: Vec<String> = sqlx::query_scalar("SELECT folder_path FROM meetings WHERE id <> ? AND folder_path IS NOT NULL UNION SELECT folder_path FROM meeting_deletion_jobs WHERE meeting_id <> ? AND folder_path IS NOT NULL UNION SELECT folder_path FROM recording_sessions WHERE meeting_id <> ? AND folder_path IS NOT NULL")
        .bind(meeting_id).bind(meeting_id).bind(meeting_id).fetch_all(pool).await.map_err(|e| e.to_string())?;
    if other_folders.iter().any(|path| fs::canonicalize(path).ok().as_ref() == Some(&canonical_folder)) {
        return Err("Legacy recording ownership is unverified: folder is shared with another meeting".into());
    }
    let transcript_path = canonical_folder.join("transcripts.json");
    owned_file(&transcript_path, &canonical_folder, &root)?;
    let transcript: serde_json::Value = serde_json::from_slice(&fs::read(&transcript_path).map_err(|_| "Cannot read legacy recording transcripts")?)
        .map_err(|_| "Legacy recording transcripts are invalid")?;
    let segments = transcript.get("segments").and_then(|v| v.as_array()).filter(|v| !v.is_empty())
        .ok_or("Legacy recording ownership is unverified: no saved passages")?;
    if transcript.get("total_segments").and_then(|v| v.as_u64()) != Some(segments.len() as u64) {
        return Err("Legacy recording ownership is unverified: passage count differs".into());
    }
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT transcript, timestamp FROM transcripts WHERE meeting_id = ?")
        .bind(meeting_id).fetch_all(pool).await.map_err(|e| e.to_string())?;
    let mut saved = HashMap::<(String, String), usize>::new();
    for row in rows { *saved.entry(row).or_default() += 1; }
    for segment in segments {
        let text = segment.get("text").and_then(|v| v.as_str()).filter(|v| !v.trim().is_empty())
            .ok_or("Legacy recording ownership is unverified: passage text is missing")?;
        let timestamp = segment.get("display_time").and_then(|v| v.as_str())
            .ok_or("Legacy recording ownership is unverified: passage timestamp is missing")?;
        let remaining = saved.get_mut(&(text.to_owned(), timestamp.to_owned())).filter(|count| **count > 0)
            .ok_or("Legacy recording ownership is unverified: passages do not match this meeting")?;
        *remaining -= 1;
    }
    capture_artifacts_with_legacy_ownership(folder, roots, meeting_id, session_id, true)
}

/// Recovery establishes a durable canonical folder identity before merging any
/// audio. It never adopts a deleted identity or a folder owned by another live
/// meeting. Legacy metadata without an identity is accepted only for the native
/// deterministic recovery draft (or an already registered canonical folder).
pub(crate) async fn prepare_recovery_artifact_write(pool: &SqlitePool, folder: &Path, meeting_id: &str, roots: &[PathBuf]) -> Result<tokio::sync::OwnedMutexGuard<()>, String> {
    let guard = acquire_meeting_artifact_write(pool, meeting_id).await?;
    let (root, folder) = managed_folder(folder, roots)?.ok_or("Recovery recording folder is unavailable")?;
    let row = sqlx::query("SELECT folder_path, recording_session_id FROM meetings WHERE id = ?")
        .bind(meeting_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
    let stored_folder: Option<String> = row.get("folder_path");
    let sessions:Vec<(String,Option<String>)>=sqlx::query_as("SELECT id,folder_path FROM recording_sessions WHERE meeting_id=?")
        .bind(meeting_id).fetch_all(pool).await.map_err(|e|e.to_string())?;
    let owned_session=sessions.iter().find(|s|s.1.as_deref().is_some_and(|path|fs::canonicalize(path).ok().as_ref()==Some(&folder)));
    let registered = owned_session.is_some() || stored_folder.as_deref().is_some_and(|path| fs::canonicalize(path).ok().as_ref() == Some(&folder));
    let target_session=owned_session.map(|s|s.0.clone()).or_else(||row.get::<Option<String>,_>("recording_session_id"));
    if stored_folder.as_deref().is_some_and(|path| !path.is_empty()) && !registered {
        return Err("Recovery folder differs from this meeting's registered recording".into());
    }
    let owners: Vec<(String, String)> = sqlx::query_as("SELECT id, folder_path FROM meetings WHERE folder_path IS NOT NULL AND id <> ? UNION SELECT meeting_id,folder_path FROM recording_sessions WHERE folder_path IS NOT NULL AND meeting_id <> ?")
        .bind(meeting_id).bind(meeting_id).fetch_all(pool).await.map_err(|e| e.to_string())?;
    if owners.iter().any(|(_, path)| fs::canonicalize(path).ok().as_ref() == Some(&folder)) {
        return Err("Recovery folder belongs to another meeting".into());
    }
    let metadata_path = folder.join("metadata.json");
    owned_file(&metadata_path, &folder, &root)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&metadata_path).map_err(|_| "Cannot read recovery ownership metadata")?)
        .map_err(|_| "Recovery ownership metadata is invalid")?;
    if !metadata.is_object() { return Err("Recovery ownership metadata is invalid".into()); }
    if let Some(old_id) = metadata.get("meeting_id").and_then(|v| v.as_str()) {
        let unavailable: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meeting_deletion_tombstones WHERE meeting_id = ?) OR EXISTS(SELECT 1 FROM meetings WHERE id = ? AND id <> ?)")
            .bind(old_id).bind(old_id).bind(meeting_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
        if unavailable { return Err("Recovery metadata identifies a deleted or different meeting".into()); }
    } else if !registered && !meeting_id.starts_with("recovered-") {
        return Err("Recovery recording ownership has no canonical identity".into());
    }
    if let Some(session) = metadata.get("recording_session_id").and_then(|v| v.as_str()) {
        let unavailable: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meeting_deletion_sessions WHERE id=?) OR EXISTS(SELECT 1 FROM recording_sessions WHERE id=? AND meeting_id <> ?)")
            .bind(session).bind(session).bind(meeting_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
        if unavailable || target_session.as_deref().is_some_and(|expected|expected!=session) { return Err("Recovery session identifies a deleted or different recording".into()); }
    }
    for path in [folder.join("audio.mp4"), folder.join(".checkpoints")] {
        match fs::symlink_metadata(&path) {
            Ok(_) => owned_file(&path, &folder, &root)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(_) => return Err("Cannot inspect recovery artifact".into()),
        }
    }
    if folder.join(".checkpoints").is_dir() {
        for entry in fs::read_dir(folder.join(".checkpoints")).map_err(|_| "Cannot inspect recovery checkpoints")? {
            owned_file(&entry.map_err(|_| "Cannot inspect recovery checkpoint")?.path(), &folder, &root)?;
        }
    }
    metadata["meeting_id"] = serde_json::json!(meeting_id);
    metadata["recording_session_id"] = serde_json::json!(target_session);
    if owned_session.is_none() {
        sqlx::query("UPDATE meetings SET folder_path = ? WHERE id = ?")
            .bind(folder.to_string_lossy().as_ref()).bind(meeting_id).execute(pool).await.map_err(|e| e.to_string())?;
    }
    let partial = folder.join(format!(".metadata.json.{}.tmp", uuid::Uuid::new_v4()));
    fs::write(&partial, serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?).map_err(|_| "Cannot persist recovery ownership metadata")?;
    fs::rename(&partial, &metadata_path).map_err(|_| "Cannot commit recovery ownership metadata")?;
    Ok(guard)
}

fn cleanup_artifacts(artifacts: &OwnedArtifacts) -> Result<(), String> {
    if !artifacts.folder.try_exists().map_err(|_| "Cannot inspect recording folder".to_string())? { return Ok(()); }
    no_symlinks(&artifacts.folder)?;
    if fs::canonicalize(&artifacts.folder).map_err(|_| "Cannot resolve recording folder".to_string())? != artifacts.folder {
        return Err("Recording directory changed since deletion began".into());
    }
    // Preflight the complete inventory before removing anything on this retry.
    for path in artifacts.files.iter().chain(&artifacts.directories) {
        // symlink_metadata also detects dangling links, unlike exists().
        match fs::symlink_metadata(path) {
            Ok(_) => owned_file(path, &artifacts.folder, &artifacts.root)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(_) => return Err("Cannot inspect managed artifact".into()),
        }
    }
    for path in &artifacts.files {
        match fs::remove_file(path) {
            Ok(_) => {},
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(_) => return Err("A managed recording artifact could not be removed; retry cleanup".into()),
        }
    }
    for path in &artifacts.directories {
        match fs::remove_dir(path) {
            Ok(_) => {},
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
            Err(_) => return Err("Checkpoint cleanup is incomplete; retry cleanup".into()),
        }
    }
    // Preserve unrelated user files (including exports) in the same folder.
    if fs::read_dir(&artifacts.folder).map_err(|_| "Cannot inspect recording directory".to_string())?.next().is_none() {
        fs::remove_dir(&artifacts.folder).map_err(|_| "Cannot remove empty recording directory".to_string())?;
    }
    Ok(())
}

async fn integrity(conn: &mut SqliteConnection) -> Result<(), String> {
    let checks: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check").fetch_all(&mut *conn).await.map_err(|_| "Cannot read SQLite snapshot".to_string())?;
    if checks != ["ok"] { return Err("SQLite snapshot integrity failed; source preserved".into()); }
    if !sqlx::query("PRAGMA foreign_key_check").fetch_all(conn).await.map_err(|e| e.to_string())?.is_empty() {
        return Err("SQLite snapshot foreign keys failed; source preserved".into());
    }
    Ok(())
}

/// Compare typed row bytes outside this meeting, including settings and migration
/// metadata. Hex text/blob fields preserve embedded NULs (SQLite quote() truncates
/// them); 26-digit real formatting preserves double precision. Structural/FTS
/// shadow pages can legitimately differ after rebuilding or VACUUM.
async fn retained_rows(conn: &mut SqliteConnection, meeting_id: &str) -> Result<BTreeMap<String, Vec<String>>, String> {
    let tables: Vec<(String,)> = sqlx::query_as("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'meeting_search_fts%' AND sql NOT LIKE 'CREATE VIRTUAL TABLE%'")
        .fetch_all(&mut *conn).await.map_err(|e| e.to_string())?;
    let mut result = BTreeMap::new();
    for (table,) in tables {
        let columns = sqlx::query(&format!("PRAGMA table_info({})", quoted(&table))).fetch_all(&mut *conn).await.map_err(|e| e.to_string())?;
        let names: Vec<String> = columns.iter().map(|r| r.get("name")).collect();
        if names.is_empty() { continue; }
        let expression = names.iter().map(|c| {
            let c = quoted(c);
            format!("typeof({c}) || ':' || CASE WHEN typeof({c}) = 'real' THEN printf('%!.26g', {c}) ELSE hex({c}) END")
        }).collect::<Vec<_>>().join(" || '|' || ");
        let filter = if table == "meetings" { " WHERE id <> ?" } else if names.iter().any(|n| n == "meeting_id") { " WHERE meeting_id IS NULL OR meeting_id <> ?" } else { "" };
        let sql = format!("SELECT {expression} FROM {}{filter}", quoted(&table));
        let mut rows: Vec<String> = if filter.is_empty() { sqlx::query_scalar(&sql).fetch_all(&mut *conn).await } else { sqlx::query_scalar(&sql).bind(meeting_id).fetch_all(&mut *conn).await }.map_err(|e| e.to_string())?;
        rows.sort(); result.insert(table, rows);
    }
    Ok(result)
}

async fn sanitize_snapshot(path: &Path, meeting_id: &str) -> Result<(), String> {
    no_symlinks(path)?;
    // Validate read-only first: corrupt/partial snapshots are never modified.
    let source_options = SqliteConnectOptions::new().filename(path).read_only(true).busy_timeout(Duration::from_millis(500));
    let source = SqlitePoolOptions::new().max_connections(1).connect_with(source_options).await.map_err(|_| "Cannot open migration snapshot; source preserved".to_string())?;
    let mut read = source.acquire().await.map_err(|e| e.to_string())?;
    let validation = integrity(&mut read).await;
    drop(read); source.close().await; validation?;
    // Exclusive ownership is required before replacing a managed snapshot. DELETE
    // journaling checkpoints committed WAL rather than copying stale main pages.
    let source = SqlitePoolOptions::new().max_connections(1).connect_with(SqliteConnectOptions::new().filename(path).foreign_keys(true).journal_mode(SqliteJournalMode::Delete).busy_timeout(Duration::from_millis(500))).await
        .map_err(|_| "Migration snapshot is locked; cleanup remains pending".to_string())?;
    let partial = path.with_file_name(format!(".{}.deletion-{}.tmp", path.file_name().and_then(|v| v.to_str()).unwrap_or("snapshot"), uuid::Uuid::new_v4()));
    let result = async {
        let mut src = source.acquire().await.map_err(|e| e.to_string())?;
        sqlx::query("PRAGMA locking_mode = EXCLUSIVE").execute(&mut *src).await.map_err(|e| e.to_string())?;
        let before = retained_rows(&mut src, meeting_id).await?;
        sqlx::query("VACUUM INTO ?").bind(partial.to_string_lossy().as_ref()).execute(&mut *src).await.map_err(|_| "Cannot create disposable snapshot; source preserved".to_string())?;
        sqlx::query("BEGIN EXCLUSIVE").execute(&mut *src).await.map_err(|_| "Migration snapshot is locked; source preserved".to_string())?;
        if retained_rows(&mut src, meeting_id).await? != before { return Err("Snapshot changed during cleanup; source preserved".into()); }
        let copy = SqlitePoolOptions::new().max_connections(1).connect_with(SqliteConnectOptions::new().filename(&partial).foreign_keys(true).journal_mode(SqliteJournalMode::Delete)).await.map_err(|e| e.to_string())?;
        let copy_result = async {
            let mut tx = copy.begin_with("BEGIN IMMEDIATE").await.map_err(|e| e.to_string())?;
            let has_meetings: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='meetings' AND type='table')").fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            if has_meetings { remove_rows(&mut tx, meeting_id).await?; }
            let has_jobs: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='meeting_deletion_jobs')").fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
            if has_jobs {
                sqlx::query("DELETE FROM meeting_deletion_jobs WHERE meeting_id = ?").bind(meeting_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
            }
            integrity(&mut tx).await?;
            if retained_rows(&mut tx, meeting_id).await? != before { return Err("Snapshot cleanup changed another meeting or settings; source preserved".into()); }
            tx.commit().await.map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        }.await;
        copy.close().await; copy_result?;
        sqlx::query("ROLLBACK").execute(&mut *src).await.map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    }.await;
    source.close().await;
    if let Err(error) = result { let _ = fs::remove_file(&partial); return Err(error); }
    fs::File::open(&partial).and_then(|file| file.sync_all()).map_err(|_| "Cannot sync sanitized snapshot; source preserved".to_string())?;
    fs::rename(&partial, path).map_err(|_| "Cannot replace sanitized snapshot; source preserved".to_string())?;
    sync_directory(path.parent().ok_or_else(|| "Snapshot parent unavailable".to_string())?)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    fs::File::open(path).and_then(|file| file.sync_all()).map_err(|_| "Cannot sync managed cleanup directory".to_string())?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

async fn sanitize_managed_snapshots(app_data: &Path, meeting_id: &str) -> Result<(), String> {
    let app_data = fs::canonicalize(app_data).map_err(|_| "Cannot resolve managed application directory".to_string())?;
    let backups = app_data.join("meeting_minutes.sqlite.backups");
    if backups.try_exists().map_err(|_| "Cannot inspect migration backups".to_string())? {
        no_symlinks(&backups)?;
        let paths: Vec<PathBuf> = fs::read_dir(&backups).map_err(|_| "Cannot enumerate migration backups".to_string())?
            .map(|entry| entry.map(|e| e.path()).map_err(|_| "Cannot inspect migration backup".to_string()))
            .collect::<Result<_, _>>()?;
        for path in paths {
            if matches!(path.extension().and_then(|v| v.to_str()), Some("sqlite" | "partial")) { sanitize_snapshot(&path, meeting_id).await?; }
            // Failed/crashed sanitizer copies also carry source content. Sanitize
            // them before removing them; a corrupt copy stays pending for review.
            if path.file_name().and_then(|v| v.to_str()).is_some_and(|v| v.contains(".deletion-") && v.ends_with(".tmp")) {
                sanitize_snapshot(&path, meeting_id).await?;
                fs::remove_file(path).map_err(|_| "Cannot remove disposable deletion snapshot".to_string())?;
            }
        }
    }
    let legacy = app_data.join("meeting_minutes.db");
    if legacy.try_exists().map_err(|_| "Cannot inspect managed legacy database".to_string())? { sanitize_snapshot(&legacy, meeting_id).await?; }
    // A process can terminate after VACUUM INTO but before replacing the legacy
    // database. Those disposable copies are managed source-bearing artifacts too.
    for entry in fs::read_dir(&app_data).map_err(|_| "Cannot enumerate application cleanup copies".to_string())? {
        let path = entry.map_err(|_| "Cannot inspect application cleanup copy".to_string())?.path();
        if path.file_name().and_then(|v| v.to_str()).is_some_and(|v| v.trim_start_matches('.').starts_with("meeting_minutes.db.deletion-") && v.ends_with(".tmp")) {
            sanitize_snapshot(&path, meeting_id).await?;
            fs::remove_file(&path).map_err(|_| "Cannot remove disposable legacy cleanup snapshot".to_string())?;
        }
    }
    Ok(())
}

pub async fn retry(pool: &SqlitePool, job_id: &str, app_data: &Path, roots: &[PathBuf]) -> Result<DeletionStatus, String> {
    let _cleanup_guard = DELETION_LOCK.lock().await;
    let current = status(pool, job_id).await?;
    if current.native_complete { return Ok(current); }
    let _artifacts = lock_meeting_artifacts(&current.meeting_id).await;
    block_artifacts(&current.meeting_id);
    let cleanup = async {
        cancel_generations(pool, &current.meeting_id).await?;
        let row = sqlx::query("SELECT folder_path, artifacts_json,session_folders_json FROM meeting_deletion_jobs WHERE job_id = ?")
            .bind(job_id).fetch_one(pool).await.map_err(|e| e.to_string())?;
        let artifacts_json: Option<String> = row.get("artifacts_json");
        let artifacts:Vec<OwnedArtifacts>=if let Some(json)=artifacts_json {
            let value:serde_json::Value=serde_json::from_str(&json).map_err(|_|"Stored cleanup manifest is invalid")?;
            if value.is_null(){Vec::new()}else if value.is_array(){serde_json::from_value(value).map_err(|_|"Stored cleanup manifest is invalid")?}
            else{vec![serde_json::from_value(value).map_err(|_|"Stored cleanup manifest is invalid")?]}
        }else{
            let stored:Option<String>=row.get("session_folders_json");
            let folders:Vec<(String,Option<String>)>=match stored {
                Some(json)=>serde_json::from_str(&json).map_err(|_|"Stored session folders are invalid")?,
                None=>row.get::<Option<String>,_>("folder_path").filter(|s|!s.is_empty()).map(|f|vec![(f,current.recording_session_id.clone())]).unwrap_or_default(),
            };
            let mut artifacts=Vec::new();
            for (folder,session) in folders {
                if let Some(owned)=capture_deletion_artifacts(pool,Path::new(&folder),roots,&current.meeting_id,session.as_deref()).await? {artifacts.push(owned);}
            }
            sqlx::query("UPDATE meeting_deletion_jobs SET artifacts_json=?,updated_at=? WHERE job_id=?")
                .bind(serde_json::to_string(&artifacts).map_err(|e|e.to_string())?).bind(chrono::Utc::now().to_rfc3339()).bind(job_id).execute(pool).await.map_err(|e|e.to_string())?;
            artifacts
        };
        // Validate/sanitize archives first; failure retains current canonical
        // content and verified ownership data for a future retry.
        sanitize_managed_snapshots(app_data, &current.meeting_id).await?;
        for artifacts in artifacts { cleanup_artifacts(&artifacts)?; }
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await.map_err(|e| e.to_string())?;
        let retained = retained_rows(&mut tx, &current.meeting_id).await?;
        remove_rows(&mut tx, &current.meeting_id).await?;
        if retained_rows(&mut tx, &current.meeting_id).await? != retained {
            return Err("Canonical cleanup would change another meeting; cleanup remains pending".into());
        }
        sqlx::query("UPDATE meeting_deletion_jobs SET native_complete = 1, folder_path = NULL, session_folders_json = NULL, artifacts_json = NULL, last_error = NULL, updated_at = ? WHERE job_id = ?")
            .bind(chrono::Utc::now().to_rfc3339()).bind(job_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    }.await;
    if let Err(error) = cleanup {
        sqlx::query("UPDATE meeting_deletion_jobs SET last_error = ?, updated_at = ? WHERE job_id = ?")
            .bind(error).bind(chrono::Utc::now().to_rfc3339()).bind(job_id).execute(pool).await.map_err(|e| e.to_string())?;
    }
    status(pool, job_id).await
}

pub async fn acknowledge_caches(pool: &SqlitePool, job_id: &str) -> Result<DeletionStatus, String> {
    // Completion is a conjunction. A cache acknowledgment can safely arrive
    // before native cleanup, but never clears a native pending reason.
    sqlx::query("UPDATE meeting_deletion_jobs SET caches_complete = 1, updated_at = ? WHERE job_id = ?")
        .bind(chrono::Utc::now().to_rfc3339()).bind(job_id).execute(pool).await.map_err(|e| e.to_string())?;
    status(pool, job_id).await
}

async fn record_pending_error(pool: &SqlitePool, current: DeletionStatus, error: String) -> DeletionStatus {
    let _ = sqlx::query("UPDATE meeting_deletion_jobs SET last_error = ?, updated_at = ? WHERE job_id = ?")
        .bind(&error).bind(chrono::Utc::now().to_rfc3339()).bind(&current.job_id).execute(pool).await;
    DeletionStatus { last_error: Some(error), ..current }
}

pub async fn list_pending(pool:&SqlitePool)->Result<Vec<DeletionStatus>,String> {
    let jobs:Vec<String>=sqlx::query_scalar("SELECT job_id FROM meeting_deletion_jobs WHERE native_complete=0 OR caches_complete=0 ORDER BY created_at")
        .fetch_all(pool).await.map_err(|e|e.to_string())?;
    let mut pending=Vec::new();for job in jobs{pending.push(status(pool,&job).await?);}Ok(pending)
}

pub(crate) async fn app_paths<R: Runtime>(app: &AppHandle<R>) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let prefs = crate::audio::recording_preferences::load_recording_preferences(app).await.map_err(|_| "Cannot load managed recording roots".to_string())?;
    let roots = vec![prefs.save_folder, crate::audio::recording_preferences::get_default_recordings_folder()];
    Ok((data, roots))
}

async fn cancel_generations(pool: &SqlitePool, meeting_id: &str) -> Result<(), String> {
    crate::summary::enhancement::cancel_meeting_enhancements(meeting_id);
    let started: Option<Option<String>> = sqlx::query_scalar("SELECT start_time FROM summary_processes WHERE meeting_id = ?")
        .bind(meeting_id).fetch_optional(pool).await.map_err(|e| e.to_string())?;
    if let Some(Some(started)) = started {
        if let Ok(time) = chrono::DateTime::parse_from_rfc3339(&started) {
            crate::summary::service::SummaryService::cancel_summary(meeting_id, time.with_timezone(&chrono::Utc));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn api_begin_meeting_deletion<R: Runtime>(app: AppHandle<R>, state: tauri::State<'_, AppState>, meeting_id: String) -> Result<DeletionStatus, String> {
    let recording = crate::audio::recording_commands::get_recording_state().await;
    if (recording["is_recording"] == true || recording["is_active"] == true) && recording["meeting_id"].as_str() == Some(&meeting_id) {
        return Err("End the recording and wait for finalization before deleting this meeting".into());
    }
    let current = begin(state.db_manager.pool(), &meeting_id).await?;
    if current.native_complete { return Ok(current); }
    if let Err(error) = cancel_generations(state.db_manager.pool(), &meeting_id).await {
        return Ok(record_pending_error(state.db_manager.pool(), current, error).await);
    }
    let (data, roots) = match app_paths(&app).await {
        Ok(paths) => paths,
        Err(error) => return Ok(record_pending_error(state.db_manager.pool(), current, error).await),
    };
    match retry(state.db_manager.pool(), &current.job_id, &data, &roots).await {
        Ok(status) => Ok(status),
        Err(error) => Ok(record_pending_error(state.db_manager.pool(), current, error).await),
    }
}

#[tauri::command]
pub async fn api_retry_meeting_deletion<R: Runtime>(app: AppHandle<R>, state: tauri::State<'_, AppState>, job_id: String) -> Result<DeletionStatus, String> {
    let current = status(state.db_manager.pool(), &job_id).await?;
    if current.native_complete { return Ok(current); }
    if let Err(error) = cancel_generations(state.db_manager.pool(), &current.meeting_id).await {
        return Ok(record_pending_error(state.db_manager.pool(), current, error).await);
    }
    let (data, roots) = match app_paths(&app).await {
        Ok(paths) => paths,
        Err(error) => return Ok(record_pending_error(state.db_manager.pool(), current, error).await),
    };
    match retry(state.db_manager.pool(), &job_id, &data, &roots).await {
        Ok(status) => Ok(status),
        Err(error) => Ok(record_pending_error(state.db_manager.pool(), current, error).await),
    }
}

#[tauri::command]
pub async fn api_get_meeting_deletion_status(state: tauri::State<'_, AppState>, job_id: String) -> Result<DeletionStatus, String> {
    status(state.db_manager.pool(), &job_id).await
}

#[tauri::command]
pub async fn api_ack_meeting_deletion_caches(state: tauri::State<'_, AppState>, job_id: String) -> Result<DeletionStatus, String> {
    acknowledge_caches(state.db_manager.pool(), &job_id).await
}

/// Called after database setup and on WebView startup, never during a migration.
pub async fn retry_pending(db: &DatabaseManager, app_data: &Path, roots: &[PathBuf]) -> Result<Vec<DeletionStatus>, String> {
    let pending = list_pending(db.pool()).await?;
    let mut statuses = Vec::new();
    for current in pending {
        match retry(db.pool(), &current.job_id, app_data, roots).await {
            Ok(status) => statuses.push(status),
            Err(error) => statuses.push(record_pending_error(db.pool(), current, error).await),
        }
    }
    Ok(statuses)
}

#[tauri::command]
pub async fn api_retry_pending_meeting_deletions<R: Runtime>(app: AppHandle<R>, state: tauri::State<'_, AppState>) -> Result<Vec<DeletionStatus>, String> {
    let (data, roots) = match app_paths(&app).await {
        Ok(paths) => paths,
        Err(error) => {
            let mut statuses = Vec::new();
            for current in list_pending(state.db_manager.pool()).await? {
                statuses.push(record_pending_error(state.db_manager.pool(), current, error.clone()).await);
            }
            return Ok(statuses);
        },
    };
    retry_pending(&state.db_manager, &data, &roots).await
}

#[tauri::command]
pub async fn api_list_pending_meeting_deletions(state: tauri::State<'_, AppState>) -> Result<Vec<DeletionStatus>, String> {
    list_pending(state.db_manager.pool()).await
}

#[cfg(test)]
mod tests;
