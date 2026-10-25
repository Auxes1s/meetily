use super::*;
use sqlx::sqlite::SqlitePoolOptions;

async fn fixture() -> SqlitePool {
    let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys = ON;
        CREATE TABLE meetings(id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'completed', started_at TEXT, ended_at TEXT, timezone TEXT);
        CREATE TABLE meeting_notes(meeting_id TEXT PRIMARY KEY REFERENCES meetings(id) ON DELETE CASCADE, notes_plain_text TEXT, notes_markdown TEXT, notes_json TEXT, revision INTEGER DEFAULT 0, updated_at TEXT);
        CREATE TABLE transcripts(id TEXT PRIMARY KEY, meeting_id TEXT REFERENCES meetings(id) ON DELETE CASCADE, transcript TEXT, audio_start_time REAL, audio_end_time REAL);
        CREATE TABLE meeting_enhancement_versions(id TEXT PRIMARY KEY, meeting_id TEXT REFERENCES meetings(id) ON DELETE CASCADE, state TEXT, document_json TEXT, revision INTEGER, updated_at TEXT);
        CREATE TABLE meeting_enhancement_settings(meeting_id TEXT PRIMARY KEY REFERENCES meetings(id) ON DELETE CASCADE, approved_version_id TEXT);
        CREATE TABLE summary_processes(meeting_id TEXT PRIMARY KEY REFERENCES meetings(id) ON DELETE CASCADE, result TEXT, status TEXT, updated_at TEXT);")
        .execute(&pool).await.unwrap();
    // FTS5 capability is tested through the project's linked SQLx SQLite runtime,
    // independent of macOS's sqlite3 CLI and of real user databases.
    sqlx::raw_sql(include_str!("../../../migrations/20261013020000_meeting_library.sql")).execute(&pool).await.unwrap();
    pool
}

async fn meeting(pool: &SqlitePool, id: &str, title: &str, date: &str) {
    sqlx::query("INSERT INTO meetings(id, title, created_at, updated_at) VALUES (?, ?, ?, ?)")
        .bind(id).bind(title).bind(date).bind(date).execute(pool).await.unwrap();
}
async fn search(pool: &SqlitePool, query: &str) -> LibraryPage {
    search_meetings(pool, LibrarySearchRequest { query: query.into(), ..Default::default() }).await.unwrap()
}

#[tokio::test]
#[ignore = "200000-source synthetic search benchmark; no production database"]
async fn enrichment_performance_benchmark() {
    let pool = fixture().await;
    sqlx::raw_sql("WITH RECURSIVE n(i) AS (VALUES(0) UNION ALL SELECT i+1 FROM n WHERE i<9999)
        INSERT INTO meetings(id,title,created_at,updated_at) SELECT printf('m-%05d',i),'Synthetic meeting','2026-10-25','2026-10-25' FROM n;
        WITH RECURSIVE n(i) AS (VALUES(0) UNION ALL SELECT i+1 FROM n WHERE i<19)
        INSERT INTO transcripts SELECT m.id||'-'||n.i,m.id,'Synthetic agenda decision and next steps. Synthetic agenda decision and next steps.',n.i*5,n.i*5+4 FROM meetings m CROSS JOIN n;
        INSERT INTO meeting_search_sources(meeting_id,source_kind,source_id,text_content,audio_start_time,audio_end_time) SELECT meeting_id,'transcript',id,transcript,audio_start_time,audio_end_time FROM transcripts;")
        .execute(&pool).await.unwrap();
    let page = search(&pool,"agenda").await;
    let mut before = Vec::new(); let mut after = Vec::new();
    for _ in 0..11 {
        let start = std::time::Instant::now();
        for meeting in &page.meetings {
            let legacy: Vec<LibrarySourceHit> = sqlx::query_as("SELECT s.source_kind,s.source_id,s.revision,snippet(meeting_search_fts,0,'','',' … ',40) AS snippet,s.audio_start_time,s.audio_end_time FROM meeting_search_fts JOIN meeting_search_sources s ON s.id=meeting_search_fts.rowid WHERE meeting_search_fts MATCH ? AND s.meeting_id=? ORDER BY CASE s.source_kind WHEN 'title' THEN 0 WHEN 'original' THEN 1 WHEN 'approved' THEN 2 ELSE 3 END,s.audio_start_time,s.id LIMIT 8")
                .bind("\"agenda\"").bind(&meeting.id).fetch_all(&pool).await.unwrap();
            assert_eq!(serde_json::to_value(legacy).unwrap(),serde_json::to_value(&meeting.hits).unwrap());
        }
        before.push(start.elapsed().as_secs_f64()*1000.);
        let mut meetings = page.meetings.clone();
        for meeting in &mut meetings { meeting.hits.clear(); meeting.tags.clear(); meeting.participants.clear(); }
        let start = std::time::Instant::now();
        let mut tx = pool.begin().await.unwrap();
        enrich_page(&mut tx,&mut meetings,Some("\"agenda\"")).await.unwrap(); tx.commit().await.unwrap();
        after.push(start.elapsed().as_secs_f64()*1000.);
    }
    before.sort_by(f64::total_cmp); after.sort_by(f64::total_cmp);
    println!("200000 sources / 30 results: legacy snippet p50={:.3}ms p95={:.3}ms; batched enrichment p50={:.3}ms p95={:.3}ms",before[5],before[10],after[5],after[10]);
}

#[test]
fn query_compiler_quotes_operator_words_punctuation_unicode_and_bounds() {
    assert_eq!(compile_query("OR 技術 \"technical-consultation\" O'Brien").unwrap(), vec!["\"OR\"", "\"技術\"", "\"technical-consultation\"", "\"O'Brien\""]);
    assert_eq!(compile_query("\"project board\"").unwrap(), vec!["\"project board\""]);
    assert!(compile_query(&"a".repeat(513)).is_err());
    assert!(compile_query("one\ntwo").is_err());
    assert!(compile_query("--- ***").unwrap().is_empty());
}

#[tokio::test]
async fn linked_fts_probe_searches_unicode_cross_sources_and_literal_operators() {
    let pool = fixture().await;
    meeting(&pool, "a", "Board", "2026-10-13T00:00:00Z").await;
    meeting(&pool, "b", "Other", "2026-10-13T00:00:00Z").await;
    sqlx::query("INSERT INTO meeting_notes VALUES ('a', 'OR plan 技術', NULL, NULL, 1, 'now')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO transcripts VALUES ('t1', 'a', 'technical-consultation résumé', 12, 14)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO transcripts VALUES ('t2', 'b', 'ordinary plan', NULL, NULL)").execute(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    let results = search(&pool, "Board 技術 résumé").await;
    assert_eq!(results.meetings.len(), 1);
    assert_eq!(results.meetings[0].id, "a");
    assert_eq!(results.meetings[0].hits.len(), 3);
    assert_eq!(search(&pool, "OR").await.meetings.len(), 1);
    assert_eq!(search(&pool, "\"technical-consultation\"").await.meetings[0].hits[0].source_id, "t1");
    assert!(search(&pool, "\"OR plan\"").await.meetings[0].hits[0].snippet.contains("技術"));
    assert!(search_meetings(&pool, LibrarySearchRequest { query: "***".into(), ..Default::default() }).await.is_err());
}

#[tokio::test]
async fn batched_enrichment_matches_per_meeting_hits_and_keeps_metadata_separate() {
    let pool = fixture().await;
    for id in ["a", "b", "c"] {
        meeting(&pool, id, "common title", "2026-10-13T00:00:00Z").await;
        update_metadata(&pool, id, vec![format!("{id} person")], vec![format!("{id} tag")]).await.unwrap();
        for i in 0..15 {
            sqlx::query("INSERT INTO transcripts VALUES (?, ?, ?, ?, ?)")
                .bind(format!("{id}-{i}")).bind(id).bind(format!("common résumé passage {i}"))
                .bind(i as f64).bind(i as f64 + 1.).execute(&pool).await.unwrap();
        }
    }
    sync_search_index(&pool, 25).await.unwrap();
    let page = search(&pool, "common résumé").await;
    assert_eq!(page.meetings.len(), 3);
    for meeting in page.meetings {
        assert_eq!(meeting.participants, vec![format!("{} person", meeting.id)]);
        assert_eq!(meeting.tags, vec![format!("{} tag", meeting.id)]);
        let legacy: Vec<LibrarySourceHit> = sqlx::query_as("SELECT s.source_kind, s.source_id, s.revision, snippet(meeting_search_fts, 0, '', '', ' … ', 40) AS snippet, s.audio_start_time, s.audio_end_time FROM meeting_search_fts JOIN meeting_search_sources s ON s.id = meeting_search_fts.rowid WHERE meeting_search_fts MATCH ? AND s.meeting_id = ? ORDER BY CASE s.source_kind WHEN 'title' THEN 0 WHEN 'original' THEN 1 WHEN 'approved' THEN 2 ELSE 3 END, s.audio_start_time, s.id LIMIT 8")
            .bind("\"common\" OR \"résumé\"").bind(&meeting.id).fetch_all(&pool).await.unwrap();
        assert_eq!(meeting.hits.len(), 8);
        assert_eq!(serde_json::to_value(&meeting.hits).unwrap(), serde_json::to_value(legacy).unwrap());
    }
}

#[tokio::test]
async fn dirty_sync_retains_source_row_identity_updates_notes_and_removes_replaced_passages() {
    let pool = fixture().await;
    meeting(&pool, "a", "A", "2026-10-13T00:00:00Z").await;
    sqlx::query("INSERT INTO meeting_notes VALUES ('a', 'before', NULL, NULL, 1, 'now')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO transcripts VALUES ('old', 'a', 'obsolete phrase', 1, 2)").execute(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    let id: i64 = sqlx::query_scalar("SELECT id FROM meeting_search_sources WHERE source_kind = 'original'").fetch_one(&pool).await.unwrap();
    sqlx::query("UPDATE meeting_notes SET notes_plain_text = 'after', revision = 2").execute(&pool).await.unwrap();
    assert!(search(&pool, "before").await.meetings[0].index_updating);
    sqlx::query("DELETE FROM transcripts").execute(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    assert!(search(&pool, "before").await.meetings.is_empty());
    assert!(search(&pool, "obsolete").await.meetings.is_empty());
    let result = search(&pool, "after").await;
    assert_eq!(result.meetings[0].hits[0].revision, Some(2));
    assert!(!result.index_updating);
    assert_eq!(id, sqlx::query_scalar::<_, i64>("SELECT id FROM meeting_search_sources WHERE source_kind = 'original'").fetch_one(&pool).await.unwrap());
}

#[tokio::test]
async fn only_pointer_selected_approved_or_legacy_is_searchable() {
    let pool = fixture().await;
    meeting(&pool, "a", "A", "2026-10-13T00:00:00Z").await;
    for (id, state, text) in [("approved", "approved", "acceptedword"), ("candidate", "candidate", "candidateword"), ("history", "archived", "historyword")] {
        sqlx::query("INSERT INTO meeting_enhancement_versions VALUES (?, 'a', ?, ?, 1, 'now')")
            .bind(id).bind(state).bind(serde_json::json!({"schema_version": 1, "markdown": text}).to_string()).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO meeting_enhancement_settings VALUES ('a', 'approved')").execute(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    assert_eq!(search(&pool, "acceptedword").await.meetings[0].hits[0].source_id, "approved");
    assert!(search(&pool, "candidateword").await.meetings.is_empty());
    assert!(search(&pool, "historyword").await.meetings.is_empty());
    sqlx::query("UPDATE meeting_enhancement_versions SET state = 'legacy' WHERE id = 'approved'").execute(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    assert_eq!(search(&pool, "acceptedword").await.meetings[0].hits[0].source_id, "approved");
    sqlx::query("UPDATE meeting_enhancement_settings SET approved_version_id = NULL").execute(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    assert!(search(&pool, "acceptedword").await.meetings.is_empty());
}

#[tokio::test]
async fn metadata_unicode_date_timezone_filters_and_stable_cursor_are_bounded() {
    let pool = fixture().await;
    for id in ["a", "b", "c"] { meeting(&pool, id, "Same", "2026-10-12T18:00:00Z").await; }
    sqlx::query("UPDATE meetings SET started_at = '2026-10-13T02:00:00+08:00', timezone = 'Asia/Manila' WHERE id = 'a'").execute(&pool).await.unwrap();
    update_metadata(&pool, "a", vec![" María Ñ " .into(), "MARÍA Ñ".into()], vec![" 技術 " .into(), "技術".into()]).await.unwrap();
    let request = LibrarySearchRequest { participant: Some("maría ñ".into()), tag: Some("技術".into()), date_from: Some("2026-10-12T16:00:00Z".into()), date_to: Some("2026-10-13T16:00:00Z".into()), ..Default::default() };
    let filtered = search_meetings(&pool, request).await.unwrap();
    assert_eq!(filtered.meetings.len(), 1);
    assert_eq!(filtered.meetings[0].participants, vec!["María Ñ"]);
    assert!(!filtered.meetings[0].date_is_creation_fallback);
    let mut request = LibrarySearchRequest { limit: Some(1), sort: Some("date_asc".into()), ..Default::default() };
    let mut ids = Vec::new();
    loop {
        let page = search_meetings(&pool, request.clone()).await.unwrap();
        ids.extend(page.meetings.into_iter().map(|m| m.id));
        request.cursor = page.next_cursor;
        if request.cursor.is_none() { break; }
    }
    assert_eq!(ids, vec!["a", "b", "c"]);
    assert!(update_metadata(&pool, "a", vec!["bad\nname".into()], vec![]).await.is_err());
    assert!(search_meetings(&pool, LibrarySearchRequest { limit: Some(101), ..Default::default() }).await.is_err());
}

#[tokio::test]
async fn rebuild_and_bounded_backfill_do_not_change_canonical_content_and_delete_removes_fts() {
    let pool = fixture().await;
    meeting(&pool, "a", "privateword", "2026-10-13T00:00:00Z").await;
    meeting(&pool, "b", "survivingword", "2026-10-13T00:00:00Z").await;
    let before: Vec<(String, String)> = sqlx::query_as("SELECT id, title FROM meetings ORDER BY id").fetch_all(&pool).await.unwrap();
    assert_eq!(sync_search_index(&pool, 1).await.unwrap().pending_count, 1);
    sync_search_index(&pool, 1).await.unwrap();
    rebuild_search_index(&pool).await.unwrap();
    sync_search_index(&pool, 25).await.unwrap();
    assert_eq!(before, sqlx::query_as::<_, (String, String)>("SELECT id, title FROM meetings ORDER BY id").fetch_all(&pool).await.unwrap());
    sqlx::query("UPDATE meetings SET status = 'deleting' WHERE id = 'a'").execute(&pool).await.unwrap();
    assert!(search(&pool, "privateword").await.meetings.is_empty());
    assert!(update_metadata(&pool, "a", vec![], vec![]).await.is_err());
    sqlx::query("DELETE FROM meetings WHERE id = 'a'").execute(&pool).await.unwrap();
    assert_eq!(search(&pool, "survivingword").await.meetings.len(), 1);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meeting_search_fts WHERE meeting_search_fts MATCH 'privateword'").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn legacy_summary_formats_and_json_only_originals_remain_searchable_without_metadata_leak() {
    let pool = fixture().await;
    let formats = [serde_json::json!({"markdown": "markdownword"}), serde_json::json!({"summary_json": [{"id":"hiddenword", "type":"paragraph", "content":[{"type":"text", "text":"blockword"}]}]}), serde_json::json!({"MeetingName":"hiddenword", "Overview":{"blocks":[{"content":"legacyword"}]}}), serde_json::json!(serde_json::json!({"MeetingName":"hiddenword", "Overview":{"blocks":[{"content":"doubleword"}]}}).to_string())];
    for (i, format) in formats.into_iter().enumerate() {
        let id = i.to_string(); meeting(&pool, &id, "Untitled", "2026-10-13T00:00:00Z").await;
        sqlx::query("INSERT INTO summary_processes VALUES (?, ?, 'completed', 'now')").bind(id).bind(format.to_string()).execute(&pool).await.unwrap();
    }
    sync_search_index(&pool, 25).await.unwrap();
    for query in ["markdownword", "blockword", "legacyword", "doubleword"] { assert_eq!(search(&pool, query).await.meetings.len(), 1); }
    assert!(search(&pool, "hiddenword").await.meetings.is_empty());
}
