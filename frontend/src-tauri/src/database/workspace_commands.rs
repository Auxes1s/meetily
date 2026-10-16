//! Bounded passage lookup for source inspection, independent of UI pagination.
use crate::{audio::source::AudioSource, state::AppState};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Serialize, FromRow)]
pub struct SourcePassage {
    pub id: String,
    pub text: String,
    pub audio_start_time: Option<f64>,
    pub audio_end_time: Option<f64>,
    pub recording_session_id: Option<String>,
    pub timeline_offset: f64,
    pub recording_folder: Option<String>,
    #[serde(skip)]
    pub speaker: Option<String>,
    #[sqlx(skip)]
    pub audio_source: AudioSource,
}
pub async fn get_passage(pool: &SqlitePool, meeting_id: &str, transcript_id: &str) -> Result<Option<SourcePassage>, String> {
    let mut passage: Option<SourcePassage> = sqlx::query_as("SELECT t.id,t.transcript AS text,t.audio_start_time,t.audio_end_time,t.speaker,t.recording_session_id,COALESCE(s.timeline_offset,0.0) AS timeline_offset,s.folder_path AS recording_folder FROM transcripts t JOIN meetings m ON m.id=t.meeting_id LEFT JOIN recording_sessions s ON s.id=t.recording_session_id AND s.meeting_id=t.meeting_id WHERE t.id=? AND t.meeting_id=? AND m.status <> 'deleting'")
        .bind(transcript_id).bind(meeting_id).fetch_optional(pool).await.map_err(|_| "The supporting passage could not be loaded")?;
    if let Some(passage) = passage.as_mut() { passage.audio_source = AudioSource::from_storage(passage.speaker.as_deref()); }
    Ok(passage)
}
#[tauri::command]
pub async fn api_get_meeting_passage(state: tauri::State<'_, AppState>, meeting_id: String, transcript_id: String) -> Result<Option<SourcePassage>, String> {
    get_passage(state.db_manager.pool(), &meeting_id, &transcript_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn off_page_source_lookup_checks_meeting_ownership_and_deletion() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at) VALUES('a','A','2026-10-13','2026-10-13'),('b','B','2026-10-13','2026-10-13')").execute(&pool).await.unwrap();
        for i in 0..150 { sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,speaker) VALUES(?,'a',?,'00:01','system')").bind(format!("t{i}")).bind(format!("source {i}")).execute(&pool).await.unwrap(); }
        let passage = get_passage(&pool, "a", "t149").await.unwrap().unwrap();
        assert_eq!(passage.text, "source 149"); assert_eq!(passage.audio_source, AudioSource::System);
        assert!(get_passage(&pool, "b", "t149").await.unwrap().is_none());
        sqlx::query("UPDATE meetings SET status='deleting' WHERE id='a'").execute(&pool).await.unwrap();
        assert!(get_passage(&pool, "a", "t149").await.unwrap().is_none());
        assert!(crate::database::repositories::meeting::MeetingsRepository::get_meeting_metadata(&pool, "a").await.unwrap().is_none());
        let (passages, total) = crate::database::repositories::meeting::MeetingsRepository::get_meeting_transcripts_paginated(&pool, "a", 100, 0).await.unwrap();
        assert!(passages.is_empty()); assert_eq!(total, 0);
        assert_eq!(crate::database::notes_commands::load(&pool, "a").await.unwrap_err().code, "not_found");
    }
}
