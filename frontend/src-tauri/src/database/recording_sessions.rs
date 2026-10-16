//! Session ownership and append-only recording timelines. Local passage clocks
//! remain unchanged; offsets are frozen when the next recording is admitted.
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use std::path::Path;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RecordingSession {
    pub id: String,
    pub meeting_id: String,
    pub ordinal: i64,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub timeline_offset: f64,
    pub duration_basis: String,
    pub folder_path: Option<String>,
    #[serde(skip)]
    pub previous_meeting_status: String,
}

pub async fn list(pool: &SqlitePool, meeting_id: &str) -> Result<Vec<RecordingSession>, String> {
    sqlx::query_as("SELECT s.* FROM recording_sessions s JOIN meetings m ON m.id=s.meeting_id WHERE s.meeting_id=? AND m.status <> 'deleting' ORDER BY s.ordinal")
        .bind(meeting_id).fetch_all(pool).await.map_err(|e|e.to_string())
}

pub(crate) fn stored_duration(folder: &str) -> Option<f64> {
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(Path::new(folder).join("metadata.json")).ok()?)
            .ok()?;
    metadata["duration_seconds"]
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
}

pub async fn begin(
    pool: &SqlitePool,
    meeting_id: &str,
    session_id: &str,
) -> Result<RecordingSession, String> {
    super::deletion::assert_meeting_writable(pool, meeting_id).await?;
    // Read duration metadata off the executor. Old recorder metadata describes
    // captured duration; absent metadata retains the known transcript bound.
    let prior = list(pool, meeting_id).await?;
    for session in &prior {
        if matches!(
            session.status.as_str(),
            "starting" | "recording" | "finalizing"
        ) {
            return Err("Recording initialization or finalization is already in progress".into());
        }
        if let Some(folder) = session
            .folder_path
            .clone()
            .filter(|_| session.duration_basis != "audio")
        {
            let measured = tokio::task::spawn_blocking(move || stored_duration(&folder))
                .await
                .map_err(|e| e.to_string())?;
            if let Some(duration) = measured {
                sqlx::query("UPDATE recording_sessions SET duration_seconds=MAX(?,COALESCE(duration_seconds,0)),duration_basis='audio' WHERE id=?")
                    .bind(duration).bind(&session.id).execute(pool).await.map_err(|e|e.to_string())?;
            }
        }
    }
    let mut tx = pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(|e| e.to_string())?;
    let previous: Option<(String,Option<String>,Option<String>,Option<String>,Option<String>)>=sqlx::query_as("SELECT status,recording_session_id,folder_path,started_at,ended_at FROM meetings WHERE id=? AND status <> 'deleting'")
        .bind(meeting_id).fetch_optional(&mut *tx).await.map_err(|e|e.to_string())?;
    let previous = previous.ok_or("Meeting is unavailable")?;
    let status = previous.0.clone();
    if matches!(status.as_str(), "recording" | "finalizing") {
        return Err("Recording already in progress".into());
    }
    if let Some(existing) =
        sqlx::query_as::<_, RecordingSession>("SELECT * FROM recording_sessions WHERE id=?")
            .bind(session_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
    {
        return Err(if existing.meeting_id != meeting_id {
            "Recording session belongs to another meeting"
        } else {
            "Recording session has already been used"
        }
        .into());
    }
    let (ordinal,offset):(i64,f64)=sqlx::query_as("SELECT COALESCE(MAX(ordinal),0)+1,COALESCE(MAX(timeline_offset+COALESCE(duration_seconds,0)),0.0) FROM recording_sessions WHERE meeting_id=?")
        .bind(meeting_id).fetch_one(&mut *tx).await.map_err(|e|e.to_string())?;
    sqlx::query("INSERT INTO recording_sessions(id,meeting_id,ordinal,status,created_at,timeline_offset,previous_meeting_status,previous_meeting_json) VALUES(?,?,?,'starting',?,?,?,?)")
        .bind(session_id).bind(meeting_id).bind(ordinal).bind(chrono::Utc::now().to_rfc3339()).bind(offset).bind(status).bind(serde_json::to_string(&previous).map_err(|e|e.to_string())?)
        .execute(&mut *tx).await.map_err(|e|e.to_string())?;
    let session = sqlx::query_as("SELECT * FROM recording_sessions WHERE id=?")
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(session)
}

pub async fn mark_live(
    pool: &SqlitePool,
    session_id: &str,
    folder: Option<String>,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let result=sqlx::query("UPDATE recording_sessions SET status='recording',started_at=?,folder_path=? WHERE id=? AND status='starting'")
        .bind(&now).bind(&folder).bind(session_id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    if result.rows_affected() != 1 {
        return Err("Recording start was cancelled or its session is unavailable".into());
    }
    sqlx::query("UPDATE meetings SET status='recording',recording_session_id=?,started_at=COALESCE(started_at,?),ended_at=NULL,folder_path=COALESCE(?,folder_path) WHERE id=(SELECT meeting_id FROM recording_sessions WHERE id=?) AND status <> 'deleting'")
        .bind(session_id).bind(&now).bind(&folder).bind(session_id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())
}

pub async fn fail_start(
    pool: &SqlitePool,
    session_id: &str,
    cancelled: bool,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let previous:Option<(String,Option<String>)>=sqlx::query_as("SELECT meeting_id,previous_meeting_json FROM recording_sessions WHERE id=? AND status IN ('starting','recording')")
        .bind(session_id).fetch_optional(&mut *tx).await.map_err(|e|e.to_string())?;
    if let Some((id, Some(json))) = previous {
        let old: (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        sqlx::query("UPDATE meetings SET status=?,recording_session_id=?,folder_path=?,started_at=?,ended_at=? WHERE id=? AND recording_session_id=? AND status='recording'")
            .bind(old.0).bind(old.1).bind(old.2).bind(old.3).bind(old.4).bind(id).bind(session_id).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    }
    sqlx::query("UPDATE recording_sessions SET status=?,ended_at=? WHERE id=? AND status IN ('starting','recording')")
        .bind(if cancelled {"cancelled"}else{"failed"}).bind(chrono::Utc::now().to_rfc3339()).bind(session_id)
        .execute(&mut *tx).await.map_err(|e|e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn api_get_recording_sessions(
    state: tauri::State<'_, crate::state::AppState>,
    meeting_id: String,
) -> Result<Vec<RecordingSession>, String> {
    list(state.db_manager.pool(), &meeting_id).await
}

#[tauri::command]
pub async fn api_open_recording_session(
    state: tauri::State<'_, crate::state::AppState>,
    meeting_id: String,
    recording_session_id: String,
) -> Result<(), String> {
    let folder:Option<String>=sqlx::query_scalar("SELECT s.folder_path FROM recording_sessions s JOIN meetings m ON m.id=s.meeting_id WHERE s.id=? AND s.meeting_id=? AND m.status <> 'deleting'")
        .bind(recording_session_id).bind(meeting_id).fetch_optional(state.db_manager.pool()).await.map_err(|e|e.to_string())?.flatten();
    let folder = folder.ok_or("This recording has no saved audio folder")?;
    if !Path::new(&folder).is_dir() {
        return Err("This recording's audio folder is unavailable".into());
    }
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(target_os = "windows")]
    let opener = "explorer";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    std::process::Command::new(opener)
        .arg(folder)
        .spawn()
        .map_err(|_| "The recording folder could not be opened".to_owned())?;
    Ok(())
}

#[cfg(test)]
mod tests;
