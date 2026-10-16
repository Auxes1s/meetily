use super::*;

async fn fixture() -> (tempfile::TempDir, DatabaseManager) {
    let dir = tempfile::tempdir().unwrap();
    let db = DatabaseManager::new(dir.path().join("meeting_minutes.sqlite").to_str().unwrap(), "missing.db").await.unwrap();
    for (id, session) in [("delete-me", "session-delete"), ("keep-me", "session-keep")] {
        sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,recording_session_id) VALUES (?,?,'2026-10-13T00:00:00Z','2026-10-13T00:00:00Z','completed',?)")
            .bind(id).bind(format!("{id} notes" )).bind(session).execute(db.pool()).await.unwrap();
        sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp) VALUES (?,?,?,'00:01')")
            .bind(format!("passage-{id}")).bind(id).bind(format!("{id} original" )).execute(db.pool()).await.unwrap();
    }
    (dir, db)
}

async fn count(pool: &SqlitePool, sql: &str) -> i64 { sqlx::query_scalar(sql).fetch_one(pool).await.unwrap() }

fn recording(dir: &Path, id: &str) -> (PathBuf, PathBuf) {
    let root = dir.join("recordings");
    let folder = root.join("meeting-folder");
    fs::create_dir_all(folder.join(".checkpoints")).unwrap();
    fs::write(folder.join("metadata.json"), serde_json::json!({"meeting_id":id, "recording_session_id":"session-delete", "audio_file":"audio.mp4", "transcript_file":"transcripts.json"}).to_string()).unwrap();
    for name in ["audio.mp4", "transcripts.json", ".metadata.json.tmp", ".checkpoints/audio_chunk_001.mp4", ".checkpoints/concat_list.txt"] {
        fs::write(folder.join(name), "managed content").unwrap();
    }
    (root, folder)
}

#[tokio::test]
async fn recording_and_finalization_reject_deletion_without_intent() {
    let (_dir, db) = fixture().await;
    for state in ["recording", "finalizing"] {
        sqlx::query("UPDATE meetings SET status=? WHERE id='delete-me'").bind(state).execute(db.pool()).await.unwrap();
        assert!(begin(db.pool(), "delete-me").await.unwrap_err().contains("finalization"));
        assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meeting_deletion_jobs").await, 0);
        assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meeting_deletion_tombstones").await, 0);
    }
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn intent_is_idempotent_blocks_late_writes_and_survives_restart() {
    let (dir, db) = fixture().await;
    let begun = begin(db.pool(), "delete-me").await.unwrap();
    assert_eq!(begin(db.pool(), "delete-me").await.unwrap().job_id, begun.job_id);
    assert!(assert_meeting_writable(db.pool(), "delete-me").await.is_err());
    assert!(sqlx::query("UPDATE transcripts SET transcript='late' WHERE meeting_id='delete-me'").execute(db.pool()).await.is_err());
    assert!(sqlx::query("UPDATE transcripts SET meeting_id='keep-me' WHERE meeting_id='delete-me'").execute(db.pool()).await.is_err());
    assert!(sqlx::query("UPDATE meetings SET id='resurrected' WHERE id='delete-me'").execute(db.pool()).await.is_err());
    assert!(sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,recording_session_id) VALUES ('resurrected','late','2026-10-13','2026-10-13','session-delete')").execute(db.pool()).await.is_err());
    db.cleanup().await.unwrap();
    let reopened = DatabaseManager::new(dir.path().join("meeting_minutes.sqlite").to_str().unwrap(), "missing.db").await.unwrap();
    let native = retry(reopened.pool(), &begun.job_id, dir.path(), &[]).await.unwrap();
    assert_eq!(native.state, "pending");
    assert!(native.native_complete); assert!(!native.caches_complete);
    assert_eq!(acknowledge_caches(reopened.pool(), &begun.job_id).await.unwrap().state, "complete");
    assert_eq!(retry(reopened.pool(), &begun.job_id, dir.path(), &[]).await.unwrap().state, "complete");
    assert_eq!(count(reopened.pool(), "SELECT COUNT(*) FROM meetings WHERE id='keep-me'").await, 1);
    assert_eq!(count(reopened.pool(), "SELECT COUNT(*) FROM transcripts WHERE meeting_id='delete-me'").await, 0);
    assert_eq!(count(reopened.pool(), "SELECT COUNT(*) FROM meeting_deletion_tombstones").await, 1);
    reopened.cleanup().await.unwrap();
}

#[tokio::test]
async fn owned_artifacts_removed_but_exports_imported_originals_models_and_others_survive() {
    let (dir, db) = fixture().await;
    let (root, folder) = recording(dir.path(), "delete-me");
    fs::write(folder.join("user-export.md"), "export must remain").unwrap();
    let original = dir.path().join("selected-original.mp3"); fs::write(&original, "user original").unwrap();
    let models = dir.path().join("models"); fs::create_dir(&models).unwrap(); fs::write(models.join("model.gguf"), "model").unwrap();
    let other = root.join("another-meeting"); fs::create_dir(&other).unwrap(); fs::write(other.join("audio.mp4"), "another").unwrap();
    sqlx::query("UPDATE meetings SET folder_path=? WHERE id='delete-me'").bind(folder.to_str().unwrap()).execute(db.pool()).await.unwrap();
    let job = begin(db.pool(), "delete-me").await.unwrap();
    let completed = retry(db.pool(), &job.job_id, dir.path(), &[root]).await.unwrap();
    assert!(completed.native_complete, "{:?}", completed.last_error);
    assert!(!folder.join("audio.mp4").exists()); assert!(!folder.join("metadata.json").exists()); assert!(!folder.join(".checkpoints").exists());
    assert_eq!(fs::read_to_string(folder.join("user-export.md")).unwrap(), "export must remain");
    assert!(original.exists()); assert!(models.join("model.gguf").exists()); assert!(other.join("audio.mp4").exists());
    let row = sqlx::query("SELECT folder_path, artifacts_json FROM meeting_deletion_jobs WHERE job_id=?").bind(&job.job_id).fetch_one(db.pool()).await.unwrap();
    assert!(row.get::<Option<String>, _>("folder_path").is_none()); assert!(row.get::<Option<String>, _>("artifacts_json").is_none());
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn unknown_identity_or_unmanaged_root_keeps_cleanup_pending() {
    let (dir, db) = fixture().await;
    let (root, folder) = recording(dir.path(), "another-meeting");
    sqlx::query("UPDATE meetings SET folder_path=? WHERE id='delete-me'").bind(folder.to_str().unwrap()).execute(db.pool()).await.unwrap();
    let job = begin(db.pool(), "delete-me").await.unwrap();
    let pending = retry(db.pool(), &job.job_id, dir.path(), &[root]).await.unwrap();
    assert!(!pending.native_complete); assert!(pending.last_error.unwrap().contains("identity"));
    assert!(folder.join("audio.mp4").exists());
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings WHERE id='delete-me'").await, 1);
    assert!(capture_artifacts(&folder, &[dir.path().join("not-the-root")], "delete-me", Some("session-delete")).unwrap_err().contains("roots"));
    db.cleanup().await.unwrap();
}

fn legacy_recording(dir: &Path) -> (PathBuf, PathBuf) {
    let (root, folder) = recording(dir, "delete-me");
    fs::write(folder.join("metadata.json"), serde_json::json!({
        "version": "1.0", "meeting_id": null, "status": "completed",
        "audio_file": "audio.mp4", "transcript_file": "transcripts.json",
    }).to_string()).unwrap();
    // Legacy JSON and SQLite assigned different IDs to the same passage.
    fs::write(folder.join("transcripts.json"), serde_json::json!({
        "version": "1.0", "total_segments": 1,
        "segments": [{"id": "seg_0", "text": "delete-me original", "display_time": "00:01"}],
    }).to_string()).unwrap();
    (root, folder)
}

#[tokio::test]
async fn legacy_null_or_absent_identity_deletes_verified_recording_and_snapshots() {
    for absent in [false, true] {
        let (dir, db) = fixture().await;
        let (root, folder) = legacy_recording(dir.path());
        if absent {
            let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(folder.join("metadata.json")).unwrap()).unwrap();
            metadata.as_object_mut().unwrap().remove("meeting_id");
            fs::write(folder.join("metadata.json"), metadata.to_string()).unwrap();
        }
        fs::write(folder.join("user-export.md"), "preserved export").unwrap();
        sqlx::query("UPDATE meetings SET folder_path=? WHERE id='delete-me'").bind(folder.to_str().unwrap()).execute(db.pool()).await.unwrap();
        let backups = dir.path().join("meeting_minutes.sqlite.backups");
        fs::create_dir(&backups).unwrap();
        let archive = backups.join("legacy.sqlite");
        sqlx::query("VACUUM INTO ?").bind(archive.to_str().unwrap()).execute(db.pool()).await.unwrap();
        let job = begin(db.pool(), "delete-me").await.unwrap();
        acknowledge_caches(db.pool(), &job.job_id).await.unwrap();
        let complete = retry(db.pool(), &job.job_id, dir.path(), &[root]).await.unwrap();
        assert_eq!(complete.state, "complete", "{:?}", complete.last_error);
        for artifact in ["metadata.json", "transcripts.json", "audio.mp4", ".checkpoints"] {
            assert!(!folder.join(artifact).exists());
        }
        assert_eq!(fs::read_to_string(folder.join("user-export.md")).unwrap(), "preserved export");
        assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings WHERE id='delete-me'").await, 0);
        assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM transcripts WHERE meeting_id='keep-me'").await, 1);
        let snapshot = SqlitePool::connect_with(SqliteConnectOptions::new().filename(archive).read_only(true)).await.unwrap();
        assert_eq!(count(&snapshot, "SELECT COUNT(*) FROM meetings WHERE id='delete-me'").await, 0);
        assert_eq!(count(&snapshot, "SELECT COUNT(*) FROM transcripts WHERE meeting_id='keep-me'").await, 1);
        snapshot.close().await;
        db.cleanup().await.unwrap();
    }
}

#[tokio::test]
async fn legacy_identity_requires_matching_passages_and_unshared_canonical_folder() {
    for failure in ["passage", "timestamp", "multiplicity", "shared-folder", "session", "explicit-id"] {
        let (dir, db) = fixture().await;
        let (root, folder) = legacy_recording(dir.path());
        match failure {
            "shared-folder" => {
                sqlx::query("UPDATE meetings SET folder_path=? WHERE id='keep-me'")
                    .bind(folder.join(".").to_str().unwrap()).execute(db.pool()).await.unwrap();
            },
            "session" | "explicit-id" => {
                let path = folder.join("metadata.json");
                let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                if failure == "session" { metadata["recording_session_id"] = serde_json::json!("session-keep"); }
                else { metadata["meeting_id"] = serde_json::json!("keep-me"); }
                fs::write(path, metadata.to_string()).unwrap();
            },
            _ => {
                let path = folder.join("transcripts.json");
                let mut transcript: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                match failure {
                    "passage" => transcript["segments"][0]["text"] = serde_json::json!("keep-me original"),
                    "timestamp" => transcript["segments"][0]["display_time"] = serde_json::json!("00:02"),
                    _ => {
                        let duplicate = transcript["segments"][0].clone();
                        transcript["segments"].as_array_mut().unwrap().push(duplicate);
                        transcript["total_segments"] = serde_json::json!(2);
                    },
                }
                fs::write(path, transcript.to_string()).unwrap();
            },
        }
        sqlx::query("UPDATE meetings SET folder_path=? WHERE id='delete-me'").bind(folder.to_str().unwrap()).execute(db.pool()).await.unwrap();
        let before = fs::read(folder.join("metadata.json")).unwrap();
        let job = begin(db.pool(), "delete-me").await.unwrap();
        let pending = retry(db.pool(), &job.job_id, dir.path(), &[root]).await.unwrap();
        assert!(!pending.native_complete, "{failure}");
        assert!(pending.last_error.is_some(), "{failure}");
        assert_eq!(fs::read(folder.join("metadata.json")).unwrap(), before);
        assert!(folder.join("audio.mp4").exists());
        assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings").await, 2);
        db.cleanup().await.unwrap();
    }
}

#[cfg(unix)]
#[tokio::test]
async fn symbolic_links_and_retry_path_replacements_do_not_escape() {
    use std::os::unix::fs::symlink;
    let (dir, db) = fixture().await;
    let (root, folder) = recording(dir.path(), "delete-me");
    let outside = dir.path().join("outside.mp4"); fs::write(&outside, "outside original").unwrap();
    fs::remove_file(folder.join("audio.mp4")).unwrap(); symlink(&outside, folder.join("audio.mp4")).unwrap();
    assert!(capture_artifacts(&folder, &[root.clone()], "delete-me", Some("session-delete")).unwrap_err().contains("symbolic"));
    fs::remove_file(folder.join("audio.mp4")).unwrap(); fs::write(folder.join("audio.mp4"), "managed").unwrap();
    let artifacts = capture_artifacts(&folder, &[root.clone()], "delete-me", Some("session-delete")).unwrap().unwrap();
    fs::remove_file(folder.join("audio.mp4")).unwrap(); symlink(&outside, folder.join("audio.mp4")).unwrap();
    assert!(cleanup_artifacts(&artifacts).unwrap_err().contains("symbolic"));
    assert_eq!(fs::read_to_string(&outside).unwrap(), "outside original");
    assert!(folder.join("metadata.json").exists());
    let link = root.join("meeting-link"); symlink(&folder, &link).unwrap();
    assert!(capture_artifacts(&link, &[root], "delete-me", Some("session-delete")).is_err());
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn disposable_snapshot_sanitization_preserves_other_rows_and_settings() {
    let (dir, db) = fixture().await;
    let backups = dir.path().join("meeting_minutes.sqlite.backups"); fs::create_dir(&backups).unwrap();
    for name in ["before.sqlite", "before.partial"] {
        sqlx::query("VACUUM INTO ?").bind(backups.join(name).to_str().unwrap()).execute(db.pool()).await.unwrap();
    }
    sqlx::query("VACUUM INTO ?").bind(dir.path().join("meeting_minutes.db").to_str().unwrap()).execute(db.pool()).await.unwrap();
    let before = {
        let mut conn = db.pool().acquire().await.unwrap(); retained_rows(&mut conn, "delete-me").await.unwrap()
    };
    let job = begin(db.pool(), "delete-me").await.unwrap();
    let result = retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap();
    assert!(result.native_complete, "{:?}", result.last_error);
    for path in [backups.join("before.sqlite"), backups.join("before.partial"), dir.path().join("meeting_minutes.db")] {
        let snapshot = SqlitePool::connect_with(SqliteConnectOptions::new().filename(path).read_only(true)).await.unwrap();
        assert_eq!(count(&snapshot, "SELECT COUNT(*) FROM meetings WHERE id='delete-me'").await, 0);
        assert_eq!(count(&snapshot, "SELECT COUNT(*) FROM transcripts WHERE meeting_id='delete-me'").await, 0);
        let mut conn = snapshot.acquire().await.unwrap();
        integrity(&mut conn).await.unwrap();
        assert_eq!(retained_rows(&mut conn, "delete-me").await.unwrap(), before);
        drop(conn); snapshot.close().await;
    }
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn corrupt_partial_snapshot_is_preserved_and_retry_finishes_after_repair() {
    let (dir, db) = fixture().await;
    let backups = dir.path().join("meeting_minutes.sqlite.backups"); fs::create_dir(&backups).unwrap();
    let bad = backups.join("unfinished.partial");
    fs::write(&bad, "partial evidence").unwrap();
    let job = begin(db.pool(), "delete-me").await.unwrap();
    let first = retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap();
    assert!(!first.native_complete); assert_eq!(first.state, "pending");
    assert_eq!(fs::read_to_string(&bad).unwrap(), "partial evidence");
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings WHERE id='delete-me'").await, 1);
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM transcripts WHERE meeting_id='keep-me'").await, 1);
    // Simulate archive recovery using an independently valid SQLite snapshot.
    fs::remove_file(&bad).unwrap();
    sqlx::query("VACUUM INTO ?").bind(bad.to_str().unwrap()).execute(db.pool()).await.unwrap();
    let native = retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap();
    assert!(native.native_complete, "{:?}", native.last_error);
    assert!(native.last_error.is_none());
    assert_eq!(acknowledge_caches(db.pool(), &job.job_id).await.unwrap().state, "complete");
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn locked_snapshot_stays_pending_then_retry_sanitizes_committed_wal() {
    let (dir, db) = fixture().await;
    let path = dir.path().join("meeting_minutes.db");
    sqlx::query("VACUUM INTO ?").bind(path.to_str().unwrap()).execute(db.pool()).await.unwrap();
    let legacy = SqlitePoolOptions::new().max_connections(1).connect_with(SqliteConnectOptions::new().filename(&path).journal_mode(SqliteJournalMode::Wal)).await.unwrap();
    sqlx::query("UPDATE meetings SET title='Committed WAL title' WHERE id='keep-me'").execute(&legacy).await.unwrap();
    let mut locked = legacy.acquire().await.unwrap();
    sqlx::query("BEGIN EXCLUSIVE").execute(&mut *locked).await.unwrap();
    let job = begin(db.pool(), "delete-me").await.unwrap();
    assert!(!retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap().native_complete);
    sqlx::query("ROLLBACK").execute(&mut *locked).await.unwrap(); drop(locked); legacy.close().await;
    let result = retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap();
    assert!(result.native_complete, "{:?}", result.last_error);
    let snapshot = SqlitePool::connect_with(SqliteConnectOptions::new().filename(path).read_only(true)).await.unwrap();
    assert_eq!(sqlx::query_scalar::<_, String>("SELECT title FROM meetings WHERE id='keep-me'").fetch_one(&snapshot).await.unwrap(), "Committed WAL title");
    snapshot.close().await; db.cleanup().await.unwrap();
}

#[tokio::test]
async fn startup_retry_attempts_other_jobs_when_one_has_unverified_ownership() {
    let (dir, db) = fixture().await;
    let (root, folder) = recording(dir.path(), "wrong-identity");
    sqlx::query("UPDATE meetings SET folder_path=? WHERE id='delete-me'").bind(folder.to_str().unwrap()).execute(db.pool()).await.unwrap();
    let bad = begin(db.pool(), "delete-me").await.unwrap();
    let good = begin(db.pool(), "keep-me").await.unwrap();
    let outcomes = retry_pending(&db, dir.path(), &[root]).await.unwrap();
    assert_eq!(outcomes.len(), 2);
    assert!(!outcomes.iter().find(|r| r.job_id == bad.job_id).unwrap().native_complete);
    assert!(outcomes.iter().find(|r| r.job_id == good.job_id).unwrap().native_complete);
    assert!(folder.join("audio.mp4").exists());
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn failure_after_committed_intent_returns_and_persists_pending_job_identity() {
    let (_dir, db) = fixture().await;
    let begun = begin(db.pool(), "delete-me").await.unwrap();
    let failed = record_pending_error(db.pool(), begun.clone(), "Managed recording roots unavailable".into()).await;
    assert_eq!(failed.job_id, begun.job_id);
    assert_eq!(failed.state, "pending");
    assert_eq!(status(db.pool(), &begun.job_id).await.unwrap().last_error, failed.last_error);
    assert!(is_meeting_deleted(db.pool(), "delete-me").await.unwrap());
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn abandoned_legacy_sanitizer_copies_are_removed_before_native_completion() {
    let (dir, db) = fixture().await;
    let abandoned = dir.path().join(".meeting_minutes.db.deletion-interrupted.tmp");
    let nested = dir.path().join("..meeting_minutes.db.deletion-interrupted.tmp.deletion-interrupted.tmp");
    for path in [&abandoned, &nested] {
        sqlx::query("VACUUM INTO ?").bind(path.to_str().unwrap()).execute(db.pool()).await.unwrap();
    }
    let job = begin(db.pool(), "delete-me").await.unwrap();
    let result = retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap();
    assert!(result.native_complete, "{:?}", result.last_error);
    assert!(!abandoned.exists()); assert!(!nested.exists());
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meetings WHERE id='keep-me'").await, 1);
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn deletion_waits_for_admitted_artifact_writers_and_rejects_late_ones() {
    let (dir, db) = fixture().await;
    let writer = acquire_meeting_artifact_write(db.pool(), "delete-me").await.unwrap();
    let pool = db.pool().clone();
    let (started, ready) = tokio::sync::oneshot::channel();
    let deleting = tokio::spawn(async move {
        started.send(()).unwrap();
        begin(&pool, "delete-me").await.unwrap()
    });
    ready.await.unwrap();
    assert!(!deleting.is_finished());
    assert_eq!(count(db.pool(), "SELECT COUNT(*) FROM meeting_deletion_jobs").await, 0);
    drop(writer);
    let job = deleting.await.unwrap();
    assert!(acquire_meeting_artifact_write(db.pool(), "delete-me").await.is_err());
    assert!(retry(db.pool(), &job.job_id, dir.path(), &[]).await.unwrap().native_complete);
    assert!(acquire_meeting_artifact_write(db.pool(), "delete-me").await.is_err());
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn recovery_registers_legacy_folder_before_writes_and_cannot_adopt_deleted_identity() {
    let (dir, db) = fixture().await;
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,recording_session_id) VALUES ('recovered-legacy-fixture','Recovery','2026-10-13','2026-10-13','draft','legacy-recovery-fixture')")
        .execute(db.pool()).await.unwrap();
    let (root, folder) = recording(dir.path(), "legacy-unpersisted");
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(folder.join("metadata.json")).unwrap()).unwrap();
    metadata["meeting_id"] = serde_json::Value::Null;
    metadata["recording_session_id"] = serde_json::Value::Null;
    fs::write(folder.join("metadata.json"), metadata.to_string()).unwrap();
    let writer = prepare_recovery_artifact_write(db.pool(), &folder, "recovered-legacy-fixture", &[root.clone()]).await.unwrap();
    let stored: String = sqlx::query_scalar("SELECT folder_path FROM meetings WHERE id='recovered-legacy-fixture'").fetch_one(db.pool()).await.unwrap();
    assert_eq!(fs::canonicalize(stored).unwrap(), fs::canonicalize(&folder).unwrap());
    let metadata: serde_json::Value = serde_json::from_slice(&fs::read(folder.join("metadata.json")).unwrap()).unwrap();
    assert_eq!(metadata["meeting_id"], "recovered-legacy-fixture");
    assert_eq!(metadata["recording_session_id"], "legacy-recovery-fixture");
    drop(writer);
    let job = begin(db.pool(), "recovered-legacy-fixture").await.unwrap();
    assert!(prepare_recovery_artifact_write(db.pool(), &folder, "recovered-legacy-fixture", &[root.clone()]).await.is_err());
    assert!(retry(db.pool(), &job.job_id, dir.path(), &[root]).await.unwrap().native_complete);
    assert!(!folder.exists());
    db.cleanup().await.unwrap();
}

#[tokio::test]
async fn retained_row_check_includes_text_after_embedded_nul_and_binary_settings() {
    let (_dir, db) = fixture().await;
    sqlx::query("CREATE TABLE binary_settings (id TEXT, payload BLOB)").execute(db.pool()).await.unwrap();
    sqlx::query("INSERT INTO binary_settings VALUES ('setting', ?)").bind(vec![0_u8, 255, 124]).execute(db.pool()).await.unwrap();
    sqlx::query("UPDATE transcripts SET transcript=? WHERE meeting_id='keep-me'").bind("same\0before").execute(db.pool()).await.unwrap();
    let before = { let mut conn = db.pool().acquire().await.unwrap(); retained_rows(&mut conn, "delete-me").await.unwrap() };
    sqlx::query("UPDATE transcripts SET transcript=? WHERE meeting_id='keep-me'").bind("same\0after").execute(db.pool()).await.unwrap();
    let changed = { let mut conn = db.pool().acquire().await.unwrap(); retained_rows(&mut conn, "delete-me").await.unwrap() };
    assert_ne!(before.get("transcripts"), changed.get("transcripts"));
    assert_eq!(before.get("binary_settings"), changed.get("binary_settings"));
    db.cleanup().await.unwrap();
}


#[tokio::test]
async fn all_recording_folders_and_session_identities_are_deleted_together() {
    let (dir,db)=fixture().await;
    let (root,first)=recording(dir.path(),"delete-me");
    let second=root.join("second-recording");fs::create_dir(&second).unwrap();
    fs::write(second.join("metadata.json"),serde_json::json!({"meeting_id":"delete-me","recording_session_id":"session-second","audio_file":"audio.mp4","transcript_file":"transcripts.json"}).to_string()).unwrap();
    fs::write(second.join("audio.mp4"),"second audio").unwrap();
    fs::write(second.join("transcripts.json"),"second transcript").unwrap();
    for (id,ordinal,folder) in [("session-delete",1,&first),("session-second",2,&second)] {
        sqlx::query("INSERT INTO recording_sessions(id,meeting_id,ordinal,status,created_at,folder_path) VALUES(?,'delete-me',?,'completed','2026-10-14',?)")
            .bind(id).bind(ordinal).bind(folder.to_str()).execute(db.pool()).await.unwrap();
    }
    sqlx::query("UPDATE meetings SET recording_session_id='session-second',folder_path=? WHERE id='delete-me'").bind(second.to_str()).execute(db.pool()).await.unwrap();
    let job=begin(db.pool(),"delete-me").await.unwrap();
    assert!(job.recording_session_ids.contains(&"session-delete".into()));
    let result=retry(db.pool(),&job.job_id,dir.path(),&[root]).await.unwrap();assert!(result.native_complete,"{:?}",result.last_error);
    assert!(!first.exists());assert!(!second.exists());
    assert_eq!(count(db.pool(),"SELECT COUNT(*) FROM recording_sessions WHERE meeting_id='delete-me'").await,0);
    assert!(sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,recording_session_id) VALUES('resurrection','No','2026-10-14','2026-10-14','session-delete')").execute(db.pool()).await.is_err());
    assert_eq!(count(db.pool(),"SELECT COUNT(*) FROM meetings WHERE id='keep-me'").await,1);
}
