use super::{manager::DatabaseManager, notes_commands::{self, NoteLink, NotesSave}, repositories::transcript::TranscriptsRepository};
use sqlx::SqlitePool;

async fn fixture() -> (tempfile::TempDir, DatabaseManager) {
    let dir = tempfile::tempdir().unwrap();
    let db = DatabaseManager::new(dir.path().join("notes.sqlite").to_str().unwrap(), "missing.db").await.unwrap();
    sqlx::query("INSERT INTO meetings (id, title, created_at, updated_at, status, recording_session_id) VALUES ('meeting-a', 'Original', '2026-10-13T00:00:00Z', '2026-10-13T00:00:00Z', 'draft', 'session-a')").execute(db.pool()).await.unwrap();
    (dir, db)
}
#[tokio::test]
async fn transcript_lookup_index_serves_reads_that_include_legacy_rows() {
    let (_directory, db) = fixture().await;
    let plan: Vec<(i64,i64,i64,String)> = sqlx::query_as("EXPLAIN QUERY PLAN SELECT COUNT(*) FROM transcripts WHERE meeting_id = ?")
        .bind("meeting-a").fetch_all(db.pool()).await.unwrap();
    assert!(plan.iter().any(|row|row.3.contains("transcripts_meeting_lookup")));
}
#[tokio::test]
async fn recent_sidebar_is_bounded_with_stable_order_and_legacy_full_access() {
    use super::repositories::meeting::MeetingsRepository;
    let (_directory, db) = fixture().await;
    for i in 0..55 {
        sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status) VALUES (?,?,'2026-10-25T00:00:00Z','2026-10-25T00:00:00Z',?)")
            .bind(format!("recent-{i:03}")).bind(format!("Recent {i}"))
            .bind(if i == 0 { "deleting" } else { "draft" }).execute(db.pool()).await.unwrap();
    }
    let page = MeetingsRepository::get_recent_meetings(db.pool(),50).await.unwrap();
    assert_eq!(page.len(),50);
    assert_eq!(page.first().unwrap().id,"recent-001");
    assert_eq!(page.last().unwrap().id,"recent-050");
    assert_eq!(MeetingsRepository::get_meetings(db.pool()).await.unwrap().len(),55);
    assert!(MeetingsRepository::get_recent_meetings(db.pool(),0).await.is_err());
    assert!(MeetingsRepository::get_recent_meetings(db.pool(),101).await.is_err());
    db.cleanup().await.unwrap();
}
fn input(base_revision: i64, text: &str) -> NotesSave {
    NotesSave { meeting_id: "meeting-a".into(), notes_json: serde_json::json!([{"id":"block", "type":"paragraph", "content":[{"type":"text", "text":text}], "children":[]}]).to_string(), notes_markdown: Some(text.into()), notes_plain_text: Some(text.into()), base_revision, commit_id: uuid::Uuid::new_v4().to_string(), title: Some("User title".into()), links: vec![] }
}
async fn count(pool: &SqlitePool, query: &str) -> i64 { sqlx::query_scalar(query).fetch_one(pool).await.unwrap() }

#[tokio::test]
async fn notes_stale_revision_and_idempotent_acknowledgment() {
    let (_dir, db) = fixture().await;
    let save = input(0, "Acknowledged — 日本語");
    let retry = input(0, "Acknowledged — 日本語");
    let commit_id = save.commit_id.clone();
    assert_eq!(notes_commands::save(db.pool(), save).await.unwrap().revision, 1);
    let mut retry = retry; retry.commit_id = commit_id;
    assert_eq!(notes_commands::save(db.pool(), retry).await.unwrap().revision, 1);
    assert_eq!(notes_commands::save(db.pool(), input(0, "stale")).await.unwrap_err().code, "conflict");
    let stored = notes_commands::load(db.pool(), "meeting-a").await.unwrap().unwrap();
    assert_eq!(stored.revision, 1);
    assert!(stored.notes_json.unwrap().contains("Acknowledged — 日本語"));
    db.cleanup().await.unwrap();
}
#[tokio::test]
async fn notes_concurrent_compare_and_set_has_one_winner() {
    let (_dir, db) = fixture().await;
    let (a, b) = tokio::join!(notes_commands::save(db.pool(), input(0, "A")), notes_commands::save(db.pool(), input(0, "B")));
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    assert_eq!(a.err().or(b.err()).unwrap().code, "conflict");
    db.cleanup().await.unwrap();
}
#[tokio::test]
async fn notes_invalid_json_and_links_do_not_replace_committed_document() {
    let (_dir, db) = fixture().await;
    notes_commands::save(db.pool(), input(0, "original")).await.unwrap();
    let mut malformed = input(1, "bad"); malformed.notes_json = "{bad".into();
    assert_eq!(notes_commands::save(db.pool(), malformed).await.unwrap_err().code, "invalid_document");
    let mut invalid = input(1, "meetily-note://marker");
    invalid.links.push(NoteLink { id: "marker".into(), block_id: "block".into(), recording_session_id: "another-session".into(), recording_offset: 2.0, transcript_id: None });
    assert_eq!(notes_commands::save(db.pool(), invalid).await.unwrap_err().code, "invalid_document");
    assert_eq!(notes_commands::load(db.pool(), "meeting-a").await.unwrap().unwrap().revision, 1);
    db.cleanup().await.unwrap();
}
#[tokio::test]
async fn notes_finalization_reuses_draft_and_resolves_timestamp_without_overwriting_notes() {
    let (_dir, db) = fixture().await;
    let mut note = input(0, "meetily-note://marker");
    note.links.push(NoteLink { id: "marker".into(), block_id: "block".into(), recording_session_id: "session-a".into(), recording_offset: 2.0, transcript_id: None });
    notes_commands::save(db.pool(), note).await.unwrap();
    let segments = [crate::api::TranscriptSegment { audio_source: None, id: "ignored".into(), text: "Final passage".into(), timestamp: "00:02".into(), audio_start_time: Some(1.0), audio_end_time: Some(3.0), duration: Some(2.0), sequence_id: Some(1) }];
    for _ in 0..2 { assert_eq!(TranscriptsRepository::save_transcript(db.pool(), "Generated title", &segments, None, Some("meeting-a".into()), Some("session-a".into())).await.unwrap(), "meeting-a"); }
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings").await, 1);
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM transcripts").await, 1);
    let title: String = sqlx::query_scalar("SELECT title FROM meetings").fetch_one(db.pool()).await.unwrap();
    assert_eq!(title, "User title");
    let doc = notes_commands::load(db.pool(), "meeting-a").await.unwrap().unwrap();
    assert_eq!(doc.revision, 1); assert_eq!(doc.links[0].transcript_id.as_deref(), Some("transcript-session-a-1"));
    sqlx::query("DELETE FROM meetings WHERE id = 'meeting-a'").execute(db.pool()).await.unwrap();
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meeting_notes").await, 0);
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meeting_note_links").await, 0);
    db.cleanup().await.unwrap();
}
#[tokio::test]
async fn notes_reopen_preserves_acknowledged_json_and_marks_interruption() {
    let (dir, db) = fixture().await;
    notes_commands::save(db.pool(), input(0, "Saved content")).await.unwrap();
    sqlx::query("UPDATE meetings SET status = 'recording'").execute(db.pool()).await.unwrap();
    db.cleanup().await.unwrap();
    let reopened = DatabaseManager::new(dir.path().join("notes.sqlite").to_str().unwrap(), "missing.db").await.unwrap();
    assert_eq!(notes_commands::load(reopened.pool(), "meeting-a").await.unwrap().unwrap().revision, 1);
    let status: String = sqlx::query_scalar("SELECT status FROM meetings").fetch_one(reopened.pool()).await.unwrap();
    assert_eq!(status, "interrupted");
    assert_eq!(count(reopened.pool(), "PRAGMA synchronous").await, 2);
    assert_eq!(count(reopened.pool(), "PRAGMA foreign_keys").await, 1);
    reopened.cleanup().await.unwrap();
}
#[tokio::test]
async fn notes_migration_backup_contains_committed_wal_and_legacy_rows() {
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.sqlite");
    let old_migrations = dir.path().join("migrations"); std::fs::create_dir(&old_migrations).unwrap();
    for entry in std::fs::read_dir("migrations").unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        let version = name.split('_').next().unwrap().parse::<i64>().unwrap();
        if version < 20261013000000 { std::fs::copy(entry.path(), old_migrations.join(entry.file_name())).unwrap(); }
    }
    let legacy = SqlitePool::connect_with(SqliteConnectOptions::from_str(path.to_str().unwrap()).unwrap().create_if_missing(true).journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)).await.unwrap();
    sqlx::migrate::Migrator::new(old_migrations).await.unwrap().run(&legacy).await.unwrap();
    sqlx::query("INSERT INTO meetings (id, title, created_at, updated_at) VALUES ('legacy', 'Preserved', '2026-10-13T00:00:00Z', '2026-10-13T00:00:00Z')").execute(&legacy).await.unwrap();
    let db = DatabaseManager::new(path.to_str().unwrap(), "missing.db").await.unwrap();
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings WHERE id = 'legacy'").await, 1);
    let backup = std::fs::read_dir(path.with_extension("sqlite.backups")).unwrap().next().unwrap().unwrap().path();
    let snapshot = SqlitePool::connect_with(SqliteConnectOptions::from_str(backup.to_str().unwrap()).unwrap().read_only(true)).await.unwrap();
    assert_eq!(count(&snapshot, "SELECT COUNT(*) FROM meetings WHERE id = 'legacy'").await, 1);
    assert_eq!(count(&snapshot, "SELECT COUNT(*) FROM pragma_table_info('meetings') WHERE name = 'status'").await, 0);
    snapshot.close().await; legacy.close().await; db.cleanup().await.unwrap();
}
#[tokio::test]
async fn notes_corrupt_source_is_preserved() {
    let dir = tempfile::tempdir().unwrap(); let path = dir.path().join("bad.sqlite");
    let bytes = b"This is not SQLite; preserve this evidence";
    std::fs::write(&path, bytes).unwrap();
    assert!(DatabaseManager::new(path.to_str().unwrap(), "missing.db").await.is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
