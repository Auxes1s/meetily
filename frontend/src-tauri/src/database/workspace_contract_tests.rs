//! Disposable acceptance fixtures. Never opens the user's application database.
use super::{manager::DatabaseManager, notes_commands::{self, NotesSave}};
use std::{process::{Child, Command, Stdio}, time::{Duration, Instant}};

struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
}

async fn legacy_fixture(path: &std::path::Path, with_notes: bool) -> sqlx::SqlitePool {
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;
    let migrations = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir("migrations").unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        let version = name.split('_').next().unwrap().parse::<i64>().unwrap();
        if version < 20261013000000 && (with_notes || !name.starts_with("20251223000000")) {
            std::fs::copy(entry.path(), migrations.path().join(name)).unwrap();
        }
    }
    let pool = sqlx::SqlitePool::connect_with(SqliteConnectOptions::from_str(path.to_str().unwrap()).unwrap()
        .create_if_missing(true).foreign_keys(true).journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)).await.unwrap();
    sqlx::migrate::Migrator::new(migrations.path()).await.unwrap().run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn legacy_and_current_fixtures_preserve_all_source_bytes_through_upgrade() {
    let summaries = [
        serde_json::json!({"Topics":{"title":"Topics","blocks":[{"content":"Legacy proposal — unapproved"}]}}).to_string(),
        serde_json::to_string(&serde_json::json!({"markdown":"Twice encoded — 日本語"}).to_string()).unwrap(),
        serde_json::json!({"markdown":"# Unicode\n\nTaglish — café"}).to_string(),
        serde_json::json!({"summary_json":[{"id":"rich-block","type":"paragraph","content":[{"type":"text","text":"Rich notes","styles":{"bold":true}}],"children":[]}]}).to_string(),
    ];
    for with_notes in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("compatibility.sqlite");
        let old = legacy_fixture(&path, with_notes).await;
        for (index, summary) in summaries.iter().enumerate() {
            let id = format!("legacy-{index}");
            sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at) VALUES(?,?,'2025-10-13T00:00:00Z','2025-10-13T00:00:00Z')").bind(&id).bind(format!("Fixture {index} — 日本語")).execute(&old).await.unwrap();
            sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp) VALUES(?,?,'Original passage — café','00:01')").bind(format!("passage-{index}")).bind(&id).execute(&old).await.unwrap();
            sqlx::query("INSERT INTO summary_processes(meeting_id,status,created_at,updated_at,result) VALUES(?,'completed','2025-10-13T00:00:00Z','2025-10-13T00:00:00Z',?)").bind(&id).bind(summary).execute(&old).await.unwrap();
            if with_notes {
                sqlx::query("INSERT INTO meeting_notes(meeting_id,notes_json,notes_markdown,created_at,updated_at) VALUES(?,?,'Authoritative original','2025-10-13T00:00:00Z','2025-10-13T00:00:00Z')")
                    .bind(&id).bind(r#"[{"id":"existing","type":"paragraph","content":[{"type":"text","text":"Original — unchanged"}],"children":[]}]"#).execute(&old).await.unwrap();
            }
        }
        let before: Vec<(String,String,String,Option<String>,Option<f64>)> = sqlx::query_as("SELECT m.id,m.title,t.transcript,m.folder_path,t.audio_start_time FROM meetings m JOIN transcripts t ON t.meeting_id=m.id ORDER BY m.id").fetch_all(&old).await.unwrap();
        // Keep the WAL writer open while the upgrade takes its consistent snapshot.
        let db = DatabaseManager::new(path.to_str().unwrap(), "absent.db").await.unwrap();
        let after: Vec<(String,String,String,Option<String>,Option<f64>)> = sqlx::query_as("SELECT m.id,m.title,t.transcript,m.folder_path,t.audio_start_time FROM meetings m JOIN transcripts t ON t.meeting_id=m.id ORDER BY m.id").fetch_all(db.pool()).await.unwrap();
        assert_eq!(before, after);
        let stored: Vec<String> = sqlx::query_scalar("SELECT result FROM summary_processes ORDER BY meeting_id").fetch_all(db.pool()).await.unwrap();
        assert_eq!(stored, summaries);
        for index in 0..summaries.len() {
            let id = format!("legacy-{index}");
            super::repositories::enhancement::adopt_legacy(db.pool(), &id).await.unwrap();
            let saved: String = sqlx::query_scalar("SELECT document_json FROM meeting_enhancement_versions WHERE meeting_id=?").bind(&id).fetch_one(db.pool()).await.unwrap();
            assert_eq!(saved, summaries[index]);
            let note = notes_commands::load(db.pool(), &id).await.unwrap();
            if with_notes { assert!(note.unwrap().notes_json.unwrap().contains("Original — unchanged")); }
            else { assert!(note.is_none()); }
        }
        assert!(sqlx::query("PRAGMA foreign_key_check").fetch_all(db.pool()).await.unwrap().is_empty());
        assert_eq!(sqlx::query_scalar::<_,String>("PRAGMA integrity_check").fetch_one(db.pool()).await.unwrap(), "ok");
        old.close().await; db.cleanup().await.unwrap();
    }
}

#[tokio::test]
async fn migration_backup_write_failure_preserves_unmigrated_source() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("blocked.sqlite");
    let old = legacy_fixture(&path, true).await;
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at) VALUES('preserved','Original before failed upgrade','2025-10-13T00:00:00Z','2025-10-13T00:00:00Z')").execute(&old).await.unwrap();
    // Deterministic unwritable snapshot destination, even when tests run as root.
    std::fs::write(path.with_extension("sqlite.backups"), b"Existing file must survive").unwrap();
    assert!(DatabaseManager::new(path.to_str().unwrap(), "absent.db").await.is_err());
    assert_eq!(sqlx::query_scalar::<_,String>("SELECT title FROM meetings WHERE id='preserved'").fetch_one(&old).await.unwrap(), "Original before failed upgrade");
    assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM pragma_table_info('meetings') WHERE name='status'").fetch_one(&old).await.unwrap(), 0);
    assert_eq!(std::fs::read(path.with_extension("sqlite.backups")).unwrap(), b"Existing file must survive");
    old.close().await;
}

#[tokio::test]
async fn linked_sqlite_contract_and_fts5_are_available() {
    let dir = tempfile::tempdir().unwrap();
    let db = DatabaseManager::new(dir.path().join("contract.sqlite").to_str().unwrap(), "absent.db").await.unwrap();
    let version: String = sqlx::query_scalar("SELECT sqlite_version()").fetch_one(db.pool()).await.unwrap();
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode").fetch_one(db.pool()).await.unwrap();
    let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous").fetch_one(db.pool()).await.unwrap();
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys").fetch_one(db.pool()).await.unwrap();
    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout").fetch_one(db.pool()).await.unwrap();
    assert_eq!(journal, "wal"); assert_eq!(synchronous, 2); assert_eq!(foreign_keys, 1);
    assert!(busy_timeout >= 5_000);
    // Probes SQLx's linked runtime, not the host sqlite executable.
    let mut connection = db.pool().acquire().await.unwrap();
    sqlx::query("CREATE VIRTUAL TABLE temp.contract_fts USING fts5(text, tokenize='unicode61')").execute(&mut *connection).await.unwrap();
    sqlx::query("INSERT INTO temp.contract_fts VALUES ('Café 日本語 technical budget')").execute(&mut *connection).await.unwrap();
    let matches: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM temp.contract_fts WHERE contract_fts MATCH 'cafe'").fetch_one(&mut *connection).await.unwrap();
    assert_eq!(matches, 1);
    eprintln!("linked SQLite {version}; journal={journal}; synchronous={synchronous}; foreign_keys={foreign_keys}; busy_timeout={busy_timeout}; FTS5=available");
    drop(connection); db.cleanup().await.unwrap();
}

#[test]
fn abrupt_process_termination_preserves_acknowledged_documents() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("crash.sqlite");
    let receipt_path = dir.path().join("acknowledged.json");
    let mut child = KillOnDrop(Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "database::workspace_contract_tests::fixture_crash_writer", "--ignored", "--nocapture"])
        .env("MEETILY_FIXTURE_CRASH_DB", &path).env("MEETILY_FIXTURE_CRASH_ACK", &receipt_path)
        .stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(30);
    while !receipt_path.exists() {
        assert!(Instant::now() < deadline, "Fixture writer did not acknowledge its commit");
        assert!(child.0.try_wait().unwrap().is_none(), "Fixture writer exited before acknowledgment");
        std::thread::sleep(Duration::from_millis(20));
    }
    let acknowledged: serde_json::Value = serde_json::from_slice(&std::fs::read(receipt_path).unwrap()).unwrap();
    child.0.kill().unwrap(); child.0.wait().unwrap();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // No graceful pool close/checkpoint from the writer occurred.
        let db = DatabaseManager::new(path.to_str().unwrap(), "absent.db").await.unwrap();
        let notes = notes_commands::load(db.pool(), "crash-meeting").await.unwrap().unwrap();
        assert_eq!(notes.revision, acknowledged["revision"].as_i64().unwrap());
        assert_eq!(notes.notes_json.unwrap(), acknowledged["notes_json"].as_str().unwrap());
        let status: String = sqlx::query_scalar("SELECT status FROM meetings WHERE id='crash-meeting'").fetch_one(db.pool()).await.unwrap();
        assert_eq!(status, "interrupted");
        let version: (String, String) = sqlx::query_as("SELECT status, document_json FROM meeting_enhancement_versions WHERE id='committed-candidate'").fetch_one(db.pool()).await.unwrap();
        assert_eq!(version, ("ready".into(), acknowledged["candidate"].as_str().unwrap().into()));
        let generating: String = sqlx::query_scalar("SELECT status FROM meeting_enhancement_versions WHERE id='interrupted-candidate'").fetch_one(db.pool()).await.unwrap();
        assert_eq!(generating, "failed");
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check").fetch_one(db.pool()).await.unwrap();
        assert_eq!(integrity, "ok");
        assert!(sqlx::query("PRAGMA foreign_key_check").fetch_all(db.pool()).await.unwrap().is_empty());
        db.cleanup().await.unwrap();
    });
}

#[tokio::test]
#[ignore = "Subprocess helper; executed by abrupt_process_termination_preserves_acknowledged_documents"]
async fn fixture_crash_writer() {
    let Ok(path) = std::env::var("MEETILY_FIXTURE_CRASH_DB") else { return; };
    let receipt = std::env::var("MEETILY_FIXTURE_CRASH_ACK").unwrap();
    let db = DatabaseManager::new(&path, "absent.db").await.unwrap();
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,recording_session_id) VALUES('crash-meeting','Synthetic crash fixture','2026-10-13T00:00:00Z','2026-10-13T00:00:00Z','recording','crash-session')").execute(db.pool()).await.unwrap();
    let json = serde_json::json!([{"id":"block-one","type":"paragraph","content":[{"type":"text","text":"Acknowledged — 日本語 / Taglish","styles":{"bold":true}}],"children":[]}]).to_string();
    let ack = notes_commands::save(db.pool(), NotesSave {
        meeting_id:"crash-meeting".into(), notes_json:json.clone(), notes_markdown:Some("**Acknowledged — 日本語 / Taglish**".into()), notes_plain_text:Some("Acknowledged — 日本語 / Taglish".into()),
        base_revision:0, commit_id:uuid::Uuid::new_v4().to_string(), title:None, links:vec![],
    }).await.unwrap();
    let candidate = serde_json::json!({"markdown":"Committed candidate — independent of original notes"}).to_string();
    for (id, status, document) in [("committed-candidate","ready",Some(candidate.as_str())), ("interrupted-candidate","generating",None)] {
        sqlx::query("INSERT INTO meeting_enhancement_versions(id,meeting_id,state,status,document_json,created_at,updated_at) VALUES(?,'crash-meeting','candidate',?,?, '2026-10-13T00:00:00Z','2026-10-13T00:00:00Z')")
            .bind(id).bind(status).bind(document).execute(db.pool()).await.unwrap();
    }
    std::fs::write(receipt, serde_json::json!({"revision":ack.revision,"notes_json":json,"candidate":candidate}).to_string()).unwrap();
    // Parent terminates this process while the WAL-backed pool remains open.
    std::future::pending::<()>().await;
}
