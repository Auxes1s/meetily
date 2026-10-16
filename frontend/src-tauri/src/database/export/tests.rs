use super::*;
use crate::database::notes_commands::{self, NotesSave};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

const NOW: &str = "2026-10-14T03:00:00Z";

async fn fixture() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    insert_meeting(&pool).await;
    pool
}

async fn insert_meeting(pool: &SqlitePool) {
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,recording_session_id,folder_path,timezone) VALUES('export-meeting','Kumusta — 日本語 / café',?,?,'completed','export-session','/private/internal/recording','Asia/Manila')")
        .bind(NOW).bind(NOW).execute(pool).await.unwrap();
}

fn request(format: ExportFormat, document: DocumentSelection) -> ExportRequest {
    ExportRequest {
        meeting_id: "export-meeting".into(),
        format,
        document,
        include_transcript: false,
        expected_approved_version_id: Some("approved".into()),
    }
}

async fn add_transcripts(pool: &SqlitePool, count: usize) {
    let mut tx = pool.begin().await.unwrap();
    for i in (0..count).rev() {
        sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,audio_start_time,audio_end_time,duration,speaker,recording_session_id,sequence_id) VALUES(?,'export-meeting',?,'12:00:00',?,?,1,?,'export-session',?)")
            .bind(format!("p-{i:04}")).bind(format!("Passage {i} — Taglish 日本語"))
            .bind(i as f64).bind(i as f64 + 1.0).bind(["mic","system","mixed","Audio"][i % 4]).bind(i as i64).execute(&mut *tx).await.unwrap();
    }
    tx.commit().await.unwrap();
}

async fn add_notes(pool: &SqlitePool) -> String {
    let raw = r#"[ { "id": "original-block", "type": "paragraph", "content": [ {"type":"text","text":"Original — café 日本語", "styles":{}}, {"type":"link","href":"meetily-note://marker","content":[{"type":"text","text":"[18:43]","styles":{}}]} ], "children": [], "extension":{"kept":17} } ]"#.to_owned();
    notes_commands::save(
        pool,
        NotesSave {
            meeting_id: "export-meeting".into(),
            notes_json: raw.clone(),
            notes_markdown: Some(
                "**Original — café 日本語** [18:43](meetily-note://marker)".into(),
            ),
            notes_plain_text: Some("Original — café 日本語".into()),
            base_revision: 0,
            commit_id: uuid::Uuid::new_v4().to_string(),
            title: None,
            links: vec![NoteLink {
                id: "marker".into(),
                block_id: "original-block".into(),
                recording_session_id: "export-session".into(),
                recording_offset: 1123.5,
                transcript_id: None,
            }],
        },
    )
    .await
    .unwrap();
    raw
}

async fn add_version(
    pool: &SqlitePool,
    id: &str,
    state: &str,
    status: &str,
    document: Option<&str>,
    snapshot: Option<&str>,
) {
    sqlx::query("INSERT INTO meeting_enhancement_versions(id,meeting_id,state,status,document_json,source_snapshot_json,revision,template_id,instructions,provider_config_json,error,created_at,updated_at) VALUES(?,'export-meeting',?,?,?,?,4,'project_board','Keep uncertainty',?,NULL,?,?)")
        .bind(id).bind(state).bind(status).bind(document).bind(snapshot)
        .bind(r#"{"provider":"ollama","model":"fixture-model","summary_language":"Filipino","api_key":"SECRET-DO-NOT-EXPORT","model_path":"/private/internal/model","destination":"http://localhost:11434","template":{"name":"Board","description":"Saved custom definition","sections":[{"title":"Actions","instruction":"Retain uncertainty","format":"list"}],"api_key":"SECRET-DO-NOT-EXPORT"}}"#)
        .bind(NOW).bind(NOW).execute(pool).await.unwrap();
}

async fn approve_pointer(pool: &SqlitePool, id: &str) {
    sqlx::query("INSERT INTO meeting_enhancement_settings(meeting_id,approved_version_id,template_id,instructions,revision) VALUES('export-meeting',?,'project_board','Keep uncertainty',2)")
        .bind(id).execute(pool).await.unwrap();
}

#[tokio::test]
async fn complete_record_preserves_acknowledged_content_history_and_off_page_sources() {
    let pool = fixture().await;
    add_transcripts(&pool, 1207).await;
    let raw = add_notes(&pool).await;
    sqlx::query("INSERT INTO meeting_participants VALUES('export-meeting','josé','José'); INSERT INTO meeting_tags VALUES('export-meeting','予算','予算')").execute(&pool).await.unwrap();
    let snapshot = SourceSnapshot::read(&pool, "export-meeting", None, true)
        .await
        .unwrap();
    let snapshot_raw = serde_json::to_string(&snapshot).unwrap();
    let document = json!({"schema_version":1,"markdown":"## Approved\n\nReviewed action — 日本語","claims":[{"id":"action-1","kind":"action","text":"Review the proposal","status":"needs_review","owner":"José","deadline":"Friday","sources":[{"source_id":"transcript:p-1123","excerpt":"Passage 1123 — Taglish 日本語"}]}],"extension":{"not_lost":17}}).to_string();
    for (id, state, status) in [
        ("approved", "approved", "ready"),
        ("archived", "archived", "ready"),
        ("candidate", "candidate", "ready"),
        ("running", "candidate", "generating"),
        ("failed", "candidate", "failed"),
        ("cancelled", "candidate", "cancelled"),
        ("discarded", "candidate", "discarded"),
        ("legacy", "legacy", "unverified"),
    ] {
        add_version(
            &pool,
            id,
            state,
            status,
            if status == "generating" {
                None
            } else {
                Some(&document)
            },
            Some(&snapshot_raw),
        )
        .await;
    }
    approve_pointer(&pool, "approved").await;
    sqlx::query("UPDATE meeting_enhancement_versions SET error='/private/internal/model: fixture failure' WHERE id='failed'").execute(&pool).await.unwrap();
    let before: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let record = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(record.transcripts.len(), 1207);
    assert_eq!(record.transcripts[0].id, "p-0000");
    assert_eq!(record.transcripts[1206].id, "p-1206");
    assert_eq!(
        record
            .original_notes
            .as_ref()
            .unwrap()
            .notes_json
            .as_deref(),
        Some(raw.as_str())
    );
    assert_eq!(record.original_notes.as_ref().unwrap().revision, 1);
    assert_eq!(
        record.original_notes.as_ref().unwrap().timestamp_links[0]
            .passage
            .as_ref()
            .unwrap()["id"],
        "p-1123"
    );
    assert_eq!(record.versions.len(), 8);
    assert_eq!(
        record.approved().unwrap().document_json.as_deref(),
        Some(document.as_str())
    );
    assert_eq!(
        record.approved().unwrap().source_snapshot_json.as_deref(),
        Some(snapshot_raw.as_str())
    );
    assert_eq!(record.approved().unwrap().source_stale, Some(false));
    assert_eq!(
        record.approved().unwrap().provenance["template"]["name"],
        "Board"
    );
    for (i, expected) in [
        AudioSource::Microphone,
        AudioSource::System,
        AudioSource::Mixed,
        AudioSource::Unknown,
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(&record.transcripts[i].audio_source, expected);
    }
    let content = record
        .content(&request(ExportFormat::Json, DocumentSelection::Original))
        .unwrap();
    let json: Value = serde_json::from_slice(&content).unwrap();
    assert_eq!(json["format"], "meetily.meeting-record");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["meeting"]["participants"][0], "José");
    assert_eq!(json["meeting"]["tags"][0], "予算");
    assert_eq!(json["original_notes"]["blocks"][0]["extension"]["kept"], 17);
    assert_eq!(json["transcripts"][3]["audio_source"], "unknown");
    assert_eq!(
        json["transcripts"][1123]["recording_session_id"],
        "export-session"
    );
    let content = String::from_utf8(content).unwrap();
    assert!(!content.contains("SECRET-DO-NOT-EXPORT"));
    assert!(!content.contains("/private/internal"));
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(&pool)
            .await
            .unwrap(),
        before
    );
    let md = String::from_utf8(
        record
            .content(&request(ExportFormat::Markdown, DocumentSelection::Both))
            .unwrap(),
    )
    .unwrap();
    assert!(md.contains("Original — café 日本語"));
    assert!(md.contains("Reviewed action — 日本語"));
    assert!(md.contains("Passage 1123 — Taglish 日本語"));
    assert!(md.contains("00:18:43"));
    assert!(md.contains("owner: José"));
    assert!(md.contains("deadline: Friday"));
    assert!(md.contains("needs\\_review"));
    assert!(md.contains("](#note-timestamp-1)"));
    assert!(!md.contains("meetily-note://"));
    assert!(!md.contains("## Full transcript"));
    let mut full = request(ExportFormat::Markdown, DocumentSelection::Approved);
    full.include_transcript = true;
    let md = String::from_utf8(record.content(&full).unwrap()).unwrap();
    assert!(md.contains("## Full transcript"));
    assert!(md.contains("Passage 1206 — Taglish 日本語"));
}

#[tokio::test]
async fn frozen_evidence_keeps_deleted_and_changed_passages_and_review_status() {
    let pool = fixture().await;
    add_transcripts(&pool, 3).await;
    let snapshot = SourceSnapshot::read(&pool, "export-meeting", None, true)
        .await
        .unwrap();
    let document = json!({"markdown":"A proposal, not a resolution","claims":[{"id":"proposal","kind":"decision","text":"Consider the pilot","status":"needs_review","sources":[{"source_id":"transcript:p-0000","excerpt":"Passage 0 — Taglish 日本語"},{"source_id":"transcript:p-0001","excerpt":"Passage 1 — Taglish 日本語"},{"source_id":"transcript:unavailable","excerpt":"Retained excerpt"}]}]}).to_string();
    add_version(
        &pool,
        "approved",
        "approved",
        "ready",
        Some(&document),
        Some(&serde_json::to_string(&snapshot).unwrap()),
    )
    .await;
    approve_pointer(&pool, "approved").await;
    sqlx::query("DELETE FROM transcripts WHERE id='p-0000'; UPDATE transcripts SET transcript='Changed current text' WHERE id='p-0001'").execute(&pool).await.unwrap();
    let record = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(record.approved().unwrap().source_stale, Some(true));
    let md = String::from_utf8(
        record
            .content(&request(
                ExportFormat::Markdown,
                DocumentSelection::Approved,
            ))
            .unwrap(),
    )
    .unwrap();
    assert!(md.contains("no longer in the current transcript"));
    assert!(md.contains("differs from the current transcript"));
    assert!(md.contains("Passage 0 — Taglish 日本語"));
    assert!(!md.contains("Changed current text"));
    assert!(md.contains("00:00:00 – 00:00:01 · Microphone"));
    assert!(md.contains("System audio"));
    assert!(md.contains("Saved source details and timing unavailable"));
    assert!(md.contains("Retained excerpt"));
}

#[tokio::test]
async fn legacy_and_malformed_json_are_preserved_without_adoption_or_approval() {
    let pool = fixture().await;
    let encoded =
        serde_json::to_string(&r#"{"markdown":"Legacy — café", "extension":17}"#).unwrap();
    sqlx::query("INSERT INTO summary_processes(meeting_id,status,created_at,updated_at,result,result_backup) VALUES('export-meeting','completed',?,?,?,'{broken backup')")
        .bind(NOW).bind(NOW).bind(&encoded).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO meeting_notes(meeting_id,notes_json,created_at,updated_at) VALUES('export-meeting','{broken original',?,?)").bind(NOW).bind(NOW).execute(&pool).await.unwrap();
    add_version(
        &pool,
        "malformed",
        "candidate",
        "failed",
        Some("{broken document"),
        Some("{broken snapshot"),
    )
    .await;
    let before: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let record = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(
        record.legacy_summary.as_ref().unwrap().result.as_deref(),
        Some(encoded.as_str())
    );
    assert_eq!(
        record
            .legacy_summary
            .as_ref()
            .unwrap()
            .document
            .as_ref()
            .unwrap()["extension"],
        17
    );
    assert!(record.versions[0].document.is_none());
    assert!(record.versions[0]
        .document_json
        .as_ref()
        .unwrap()
        .contains("broken document"));
    assert_eq!(record.warnings.len(), 4);
    assert!(record
        .content(&request(ExportFormat::Json, DocumentSelection::Approved))
        .is_ok());
    assert_eq!(
        record
            .content(&request(
                ExportFormat::Markdown,
                DocumentSelection::Original
            ))
            .unwrap_err()
            .code,
        "conversion"
    );
    assert!(record.approved().is_none());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM meeting_enhancement_settings")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT total_changes()")
            .fetch_one(&pool)
            .await
            .unwrap(),
        before
    );
}

#[tokio::test]
async fn all_pages_and_documents_share_a_single_wal_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let options = SqliteConnectOptions::new()
        .filename(dir.path().join("snapshot.sqlite"))
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    insert_meeting(&pool).await;
    add_transcripts(&pool, 1001).await;
    let mut snapshot = pool.begin().await.unwrap();
    sqlx::query("SELECT title FROM meetings WHERE id='export-meeting'")
        .fetch_one(&mut *snapshot)
        .await
        .unwrap();
    let mut writer = pool.begin().await.unwrap();
    sqlx::query(
        "UPDATE meetings SET title='New title'; UPDATE transcripts SET transcript='New transcript'",
    )
    .execute(&mut *writer)
    .await
    .unwrap();
    writer.commit().await.unwrap();
    let record = read_record_snapshot(snapshot, "export-meeting")
        .await
        .unwrap();
    assert_eq!(record.transcripts.len(), 1001);
    assert!(record.meeting.title.contains("Kumusta"));
    assert!(record
        .transcripts
        .iter()
        .all(|p| p.text.starts_with("Passage")));
    let latest = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(latest.meeting.title, "New title");
    assert_eq!(latest.transcripts[1000].text, "New transcript");
    pool.close().await;
}

#[tokio::test]
async fn optional_transcript_empty_drafts_and_missing_times_have_truthful_output() {
    let pool = fixture().await;
    sqlx::query("UPDATE meetings SET status='draft'")
        .execute(&pool)
        .await
        .unwrap();
    let empty = read_record(&pool, "export-meeting").await.unwrap();
    let mut input = request(ExportFormat::Markdown, DocumentSelection::Original);
    input.include_transcript = true;
    let md = String::from_utf8(empty.content(&input).unwrap()).unwrap();
    assert!(md.contains("No original notes were saved"));
    assert!(md.contains("No committed transcript passages"));
    assert!(md.contains("creation date is not a capture time"));
    sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,speaker,sequence_id) VALUES('z','export-meeting','Last tie','12:02','Audio',NULL),('b','export-meeting','Second sequence','12:01','mixed',2),('a','export-meeting','First sequence','12:01',NULL,1),('y','export-meeting','Earlier display tie','12:01',NULL,NULL)").execute(&pool).await.unwrap();
    for (status, expected) in [
        ("recording", "incomplete_active"),
        ("finalizing", "incomplete_active"),
        ("interrupted", "incomplete_interrupted"),
    ] {
        sqlx::query("UPDATE meetings SET status=?")
            .bind(status)
            .execute(&pool)
            .await
            .unwrap();
        let record = read_record(&pool, "export-meeting").await.unwrap();
        assert_eq!(record.transcript_coverage, expected);
        assert_eq!(
            record
                .transcripts
                .iter()
                .map(|p| p.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "y", "z"]
        );
        let md = String::from_utf8(record.content(&input).unwrap()).unwrap();
        assert!(md.contains("Incomplete snapshot"));
        assert!(md.contains("Time unavailable"));
        assert!(md.contains("Mixed audio"));
        assert!(md.contains("Unknown"));
    }
}

#[tokio::test]
async fn conversion_and_selection_failures_leave_lossless_json_available() {
    let pool = fixture().await;
    add_notes(&pool).await;
    sqlx::query("UPDATE meeting_notes SET notes_markdown=NULL,conversion_status='unavailable'")
        .execute(&pool)
        .await
        .unwrap();
    let doc = r#"{"summary_json":[{"id":"rich","type":"paragraph","content":[{"type":"text","text":"Rich approved"}]}],"markdown":null}"#;
    add_version(&pool, "approved", "approved", "ready", Some(doc), None).await;
    approve_pointer(&pool, "approved").await;
    let record = read_record(&pool, "export-meeting").await.unwrap();
    for choice in [
        DocumentSelection::Original,
        DocumentSelection::Approved,
        DocumentSelection::Both,
    ] {
        assert_eq!(
            record
                .content(&request(ExportFormat::Markdown, choice))
                .unwrap_err()
                .code,
            "conversion"
        );
        assert!(record.content(&request(ExportFormat::Json, choice)).is_ok());
    }
    let mut input = request(ExportFormat::Markdown, DocumentSelection::Approved);
    input.expected_approved_version_id = Some("older-approval".into());
    assert_eq!(record.content(&input).unwrap_err().code, "approval_changed");
    assert!(serde_json::from_value::<ExportRequest>(
        json!({"meeting_id":"export-meeting","format":"pdf"})
    )
    .is_err());
}

#[tokio::test]
async fn missing_and_deleting_meetings_cannot_be_exported() {
    let pool = fixture().await;
    assert_eq!(
        read_record(&pool, "absent").await.unwrap_err().code,
        "not_found"
    );
    sqlx::query("UPDATE meetings SET status='deleting'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        read_record(&pool, "export-meeting").await.unwrap_err().code,
        "not_found"
    );
    assert!(
        crate::database::deletion::acquire_meeting_artifact_write(&pool, "export-meeting")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn overlapping_note_timestamps_prefer_the_saved_passage_and_keep_approved_links_offline() {
    let pool = fixture().await;
    sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,speaker,recording_session_id,audio_start_time,audio_end_time) VALUES('early','export-meeting','Earlier overlapping passage','12:00','mic','export-session',1100,1140),('late','export-meeting','Later overlapping passage','12:00','system','export-session',1120,1140),('foreign','export-meeting','Other session','12:00','mixed','other-session',1123,1140)").execute(&pool).await.unwrap();
    add_notes(&pool).await;
    let record = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(
        record.original_notes.unwrap().timestamp_links[0]
            .passage
            .as_ref()
            .unwrap()["id"],
        "late"
    );
    sqlx::query("UPDATE meeting_note_links SET transcript_id='early'")
        .execute(&pool)
        .await
        .unwrap();
    let record = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(
        record.original_notes.unwrap().timestamp_links[0]
            .passage
            .as_ref()
            .unwrap()["id"],
        "early"
    );
    sqlx::query("UPDATE meeting_note_links SET transcript_id=NULL")
        .execute(&pool)
        .await
        .unwrap();
    add_version(
        &pool,
        "approved",
        "approved",
        "ready",
        Some(r#"{"markdown":"Reviewed [18:43](meetily-note://marker)"}"#),
        None,
    )
    .await;
    approve_pointer(&pool, "approved").await;
    let record = read_record(&pool, "export-meeting").await.unwrap();
    assert_eq!(
        record.original_notes.as_ref().unwrap().timestamp_links[0]
            .passage
            .as_ref()
            .unwrap()["id"],
        "late"
    );
    let md = String::from_utf8(
        record
            .content(&request(
                ExportFormat::Markdown,
                DocumentSelection::Approved,
            ))
            .unwrap(),
    )
    .unwrap();
    assert!(md.contains("Reviewed [18:43](#note-timestamp-1)"));
    assert!(md.contains("Later overlapping passage"));
    assert!(md.contains("## Note timestamps"));
    assert!(!md.contains("Other session"));
    assert!(!md.contains("**Original"));
}

#[test]
fn filenames_preserve_unicode_and_handle_windows_devices_and_limits() {
    for name in [
        "CON", "con.txt", "PRN", "AUX", "NUL.", "COM1", "LPT9", "COM¹", "LPT²", "CONIN$",
    ] {
        assert!(render::filename(name, "md").starts_with('_'), "{name}");
    }
    assert_eq!(render::filename(" ... ", "json"), "meeting.json");
    assert_eq!(
        render::filename("Kumusta — 日本語 / café: ?*", "md"),
        "Kumusta — 日本語 - café- --.md"
    );
    let name = render::filename(&"🙂日本語".repeat(100), "json");
    assert!(name.len() < 256);
    assert!(name.encode_utf16().count() < 256);
    assert!(name.ends_with(".json"));
    assert_eq!(render::filename("NUL. .  ", "md"), "_NUL.md");
}

#[test]
fn atomic_write_replaces_only_after_success_and_cleans_failed_temporary_files() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("record.json");
    std::fs::write(&target, b"existing export").unwrap();
    let failure = atomic_write_with(&target, |file| {
        file.write_all(b"partial new export")?;
        Err(std::io::Error::other("fixture disk full"))
    });
    assert_eq!(failure.unwrap_err().code, "write");
    assert_eq!(std::fs::read(&target).unwrap(), b"existing export");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    atomic_write(&target, "complete — 日本語".as_bytes()).unwrap();
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "complete — 日本語"
    );
    let blocked = dir.path().join("directory.json");
    std::fs::create_dir(&blocked).unwrap();
    assert!(atomic_write(&blocked, b"cannot replace directory").is_err());
    assert!(blocked.is_dir());
    assert!(atomic_write(
        &dir.path().join("absent/record.json"),
        b"cannot create parent"
    )
    .is_err());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    assert_eq!(
        serde_json::to_value(ExportResult::Cancelled).unwrap(),
        json!({"status":"cancelled"})
    );
}

#[test]
fn destinations_cannot_overwrite_managed_sources_even_through_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let managed = dir.path().join("managed");
    std::fs::create_dir(&managed).unwrap();
    let metadata = managed.join("metadata.json");
    std::fs::write(&metadata, b"source metadata").unwrap();
    assert!(validate_destination(&metadata, "json", std::slice::from_ref(&managed)).is_err());
    assert!(validate_destination(
        &managed.join("new.json"),
        "json",
        std::slice::from_ref(&managed)
    )
    .is_err());
    assert!(validate_destination(
        &dir.path().join("out.json"),
        "json",
        std::slice::from_ref(&managed)
    )
    .is_ok());
    assert!(validate_destination(
        &dir.path().join("source.sqlite"),
        "json",
        std::slice::from_ref(&managed)
    )
    .is_err());
    #[cfg(unix)]
    {
        let link = dir.path().join("symlink");
        std::os::unix::fs::symlink(&managed, &link).unwrap();
        assert!(validate_destination(
            &link.join("out.json"),
            "json",
            std::slice::from_ref(&managed)
        )
        .is_err());
        let file_link = dir.path().join("metadata-link.json");
        std::os::unix::fs::symlink(&metadata, &file_link).unwrap();
        assert!(validate_destination(&file_link, "json", std::slice::from_ref(&managed)).is_err());
    }
    assert_eq!(std::fs::read(&metadata).unwrap(), b"source metadata");
}

#[tokio::test]
async fn session_exports_preserve_local_clocks_and_portable_continuous_offsets() {
    let pool=fixture().await;
    for (id,ordinal,status,offset) in [("export-session",1,"interrupted",0.0),("continuation",2,"completed",1200.0)] {
        sqlx::query("INSERT INTO recording_sessions(id,meeting_id,ordinal,status,created_at,duration_seconds,timeline_offset,folder_path) VALUES(?,'export-meeting',?,?,?,1200,?,'/private/internal/recording')")
            .bind(id).bind(ordinal).bind(status).bind(NOW).bind(offset).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,recording_session_id,sequence_id,audio_start_time,audio_end_time) VALUES('continuation-passage','export-meeting','Continued speech','12:00','continuation',0,2,3)").execute(&pool).await.unwrap();
    let record=read_record(&pool,"export-meeting").await.unwrap();
    assert_eq!(record.recording_sessions.len(),2);assert_eq!(record.transcript_coverage,"incomplete_interrupted");
    let passage=record.transcripts.last().unwrap();assert_eq!(passage.id,"continuation-passage");assert_eq!(passage.audio_start_time,Some(2.0));assert_eq!(passage.timeline_offset,1200.0);
    let json=serde_json::to_string(&record).unwrap();assert!(!json.contains("/private/internal/recording"));
    let text=render::markdown(&record,&ExportRequest{meeting_id:"export-meeting".into(),format:ExportFormat::Markdown,document:DocumentSelection::Original,include_transcript:true,expected_approved_version_id:None}).unwrap();
    assert!(text.contains("20:02"));
}
