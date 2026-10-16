use super::*;
use crate::{
    api::TranscriptSegment,
    database::repositories::{meeting::MeetingsRepository, transcript::TranscriptsRepository},
};
async fn fixture() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status) VALUES('meeting','Preserved title','2026-10-14T00:00:00Z','2026-10-14T00:00:00Z','draft'),('other','Other','2026-10-14T00:00:00Z','2026-10-14T00:00:00Z','draft')").execute(&pool).await.unwrap();
    pool
}
fn passage(text: &str) -> TranscriptSegment {
    TranscriptSegment {
        id: "unused".into(),
        text: text.into(),
        timestamp: "12:00:00".into(),
        audio_start_time: Some(0.0),
        audio_end_time: Some(10.0),
        duration: Some(10.0),
        sequence_id: Some(0),
        audio_source: Some(crate::audio::source::AudioSource::Microphone),
    }
}
#[tokio::test]
async fn two_recordings_keep_one_meeting_and_order_session_clocks() {
    let pool = fixture().await;
    assert_eq!(
        begin(&pool, "meeting", "s1").await.unwrap().timeline_offset,
        0.0
    );
    assert!(begin(&pool, "meeting", "parallel").await.is_err());
    mark_live(&pool, "s1", None).await.unwrap();
    TranscriptsRepository::save_transcript(
        &pool,
        "Ignored rename",
        &[passage("First")],
        None,
        Some("meeting".into()),
        Some("s1".into()),
    )
    .await
    .unwrap();
    let s2 = begin(&pool, "meeting", "s2").await.unwrap();
    assert_eq!(s2.timeline_offset, 10.0);
    mark_live(&pool, "s2", None).await.unwrap();
    TranscriptsRepository::save_transcript(
        &pool,
        "Ignored rename",
        &[passage("Second")],
        None,
        Some("meeting".into()),
        Some("s2".into()),
    )
    .await
    .unwrap();
    // Retrying the earlier recording cannot replace the second session identity.
    TranscriptsRepository::save_transcript(
        &pool,
        "Ignored rename",
        &[passage("First")],
        None,
        Some("meeting".into()),
        Some("s1".into()),
    )
    .await
    .unwrap();
    let (rows, total) =
        MeetingsRepository::get_meeting_transcripts_paginated(&pool, "meeting", 100, 0)
            .await
            .unwrap();
    assert_eq!(total, 2);
    assert_eq!(rows[0].transcript, "First");
    assert_eq!(rows[1].transcript, "Second");
    assert_eq!(rows[1].timeline_offset, 10.0);
    assert_eq!(rows[1].audio_start_time, Some(0.0));
    let metadata = MeetingsRepository::get_meeting_metadata(&pool, "meeting")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(metadata.title, "Preserved title");
    assert_eq!(metadata.recording_session_id.as_deref(), Some("s2"));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meetings")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
    assert!(TranscriptsRepository::save_transcript(
        &pool,
        "Other",
        &[passage("Wrong")],
        None,
        Some("other".into()),
        Some("s2".into())
    )
    .await
    .is_err());
}
#[tokio::test]
async fn cancelled_start_preserves_meeting_and_allows_fresh_attempt() {
    let pool = fixture().await;
    begin(&pool, "meeting", "failed").await.unwrap();
    fail_start(&pool, "failed", true).await.unwrap();
    let row: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status,recording_session_id,started_at FROM meetings WHERE id='meeting'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row, ("draft".into(), None, None));
    assert!(mark_live(&pool, "failed", None).await.is_err());
    assert_eq!(
        begin(&pool, "meeting", "retry")
            .await
            .unwrap()
            .timeline_offset,
        0.0
    );
    assert!(begin(&pool, "other", "failed").await.is_err());
}
#[tokio::test]
async fn migrated_legacy_passage_keeps_identity_and_local_clock() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let migrator = sqlx::migrate!("./migrations");
    for migration in migrator.iter().filter(|m| m.version < 20261014000000) {
        sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,recording_session_id) VALUES('legacy','Legacy','2026-10-14T00:00:00Z','2026-10-14T00:00:00Z','completed','known-session')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,audio_start_time,audio_end_time) VALUES('original-id','legacy','Original text','12:00',2,5)").execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../../../migrations/20261014000000_recording_sessions.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let row: (String, String, f64, Option<String>) = sqlx::query_as(
        "SELECT id,transcript,audio_start_time,recording_session_id FROM transcripts",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            "original-id".into(),
            "Original text".into(),
            2.0,
            Some("known-session".into())
        )
    );
    assert_eq!(
        begin(&pool, "legacy", "next")
            .await
            .unwrap()
            .timeline_offset,
        5.0
    );
}

#[tokio::test]
async fn cancellation_after_metadata_commit_restores_previous_recording() {
    let pool = fixture().await;
    begin(&pool, "meeting", "first").await.unwrap();
    mark_live(&pool, "first", None).await.unwrap();
    TranscriptsRepository::save_transcript(
        &pool,
        "Title",
        &[passage("First")],
        None,
        Some("meeting".into()),
        Some("first".into()),
    )
    .await
    .unwrap();
    let before: (String, Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status,recording_session_id,started_at,ended_at FROM meetings WHERE id='meeting'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    begin(&pool, "meeting", "late").await.unwrap();
    mark_live(&pool, "late", None).await.unwrap();
    fail_start(&pool, "late", true).await.unwrap();
    let after = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>)>(
        "SELECT status,recording_session_id,started_at,ended_at FROM meetings WHERE id='meeting'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(after, before);
    assert_eq!(list(&pool, "meeting").await.unwrap()[1].status, "cancelled");
}

#[tokio::test]
async fn legacy_sequence_free_recovery_retries_do_not_duplicate_passages() {
    let pool = fixture().await;
    let mut segment = passage("Recovered text");
    segment.sequence_id = None;
    segment.id = "browser-legacy-id".into();
    for _ in 0..2 {
        TranscriptsRepository::save_transcript(
            &pool,
            "Preserved",
            std::slice::from_ref(&segment),
            None,
            Some("meeting".into()),
            None,
        )
        .await
        .unwrap();
    }
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM transcripts WHERE meeting_id='meeting'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}
