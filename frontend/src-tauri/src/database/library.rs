//! Rebuildable local keyword search. Canonical saves only mark dirtiness;
//! a separate bounded worker projects committed documents and passages.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::state::AppState;

const MAX_QUERY_CHARS: usize = 512;
const MAX_TERMS: usize = 24;
const MAX_PAGE: i64 = 100;
static INDEX_WORKER_RUNNING: AtomicBool = AtomicBool::new(false);
static INDEX_SYNC_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibrarySearchRequest {
    #[serde(default)] pub query: String,
    pub participant: Option<String>,
    pub tag: Option<String>,
    /// Inclusive RFC3339 instant. UI date controls convert local midnight to UTC.
    pub date_from: Option<String>,
    /// Exclusive RFC3339 instant; avoids midnight/timezone ambiguity.
    pub date_to: Option<String>,
    pub status: Option<String>,
    pub sort: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LibraryMeeting {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub timezone: Option<String>,
    pub status: String,
    pub date_is_creation_fallback: bool,
    pub index_updating: bool,
    #[sqlx(skip)] pub participants: Vec<String>,
    #[sqlx(skip)] pub tags: Vec<String>,
    #[sqlx(skip)] pub hits: Vec<LibrarySourceHit>,
    #[serde(skip)] pub sort_value: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LibrarySourceHit {
    pub source_kind: String,
    pub source_id: String,
    pub revision: Option<i64>,
    pub snippet: String,
    pub audio_start_time: Option<f64>,
    pub audio_end_time: Option<f64>,
}

#[derive(FromRow)]
struct MeetingSourceHit {
    meeting_id: String,
    #[sqlx(flatten)]
    hit: LibrarySourceHit,
}

fn push_meeting_ids<'a>(query: &mut QueryBuilder<'a, Sqlite>, ids: &'a [String]) {
    let mut values = query.separated(", ");
    for id in ids { values.push_bind(id); }
}

/// Enrich only the selected page. FTS matches are ranked once for all meetings,
/// then snippets are computed for at most eight sources per meeting. Keeping
/// snippet() outside the window query preserves its FTS cursor context.
async fn enrich_page(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    meetings: &mut [LibraryMeeting],
    union_query: Option<&str>,
) -> Result<(), sqlx::Error> {
    if meetings.is_empty() { return Ok(()); }
    let ids: Vec<String> = meetings.iter().map(|m| m.id.clone()).collect();
    let positions: HashMap<&str, usize> = ids.iter().enumerate().map(|(i, id)| (id.as_str(), i)).collect();
    for (table, participants) in [("meeting_participants", true), ("meeting_tags", false)] {
        let mut query = QueryBuilder::<Sqlite>::new(format!("SELECT meeting_id, display_text FROM {table} WHERE meeting_id IN ("));
        push_meeting_ids(&mut query, &ids);
        query.push(") ORDER BY meeting_id, normalized");
        let rows: Vec<(String, String)> = query.build_query_as().fetch_all(&mut **tx).await?;
        for (id, text) in rows {
            if let Some(&index) = positions.get(id.as_str()) {
                if participants { meetings[index].participants.push(text); }
                else { meetings[index].tags.push(text); }
            }
        }
    }
    if let Some(terms) = union_query {
        let mut query = QueryBuilder::<Sqlite>::new("WITH ranked AS MATERIALIZED (SELECT s.id, ROW_NUMBER() OVER (PARTITION BY s.meeting_id ORDER BY CASE s.source_kind WHEN 'title' THEN 0 WHEN 'original' THEN 1 WHEN 'approved' THEN 2 ELSE 3 END, s.audio_start_time, s.id) AS position FROM meeting_search_fts JOIN meeting_search_sources s ON s.id = meeting_search_fts.rowid WHERE meeting_search_fts MATCH ");
        query.push_bind(terms).push(" AND s.meeting_id IN (");
        push_meeting_ids(&mut query, &ids);
        query.push(")) SELECT s.meeting_id, s.source_kind, s.source_id, s.revision, snippet(meeting_search_fts, 0, '', '', ' … ', 40) AS snippet, s.audio_start_time, s.audio_end_time FROM ranked r JOIN meeting_search_sources s ON s.id = r.id JOIN meeting_search_fts ON meeting_search_fts.rowid = s.id WHERE r.position <= 8 AND meeting_search_fts MATCH ")
            .push_bind(terms).push(" ORDER BY s.meeting_id, r.position");
        let hits: Vec<MeetingSourceHit> = query.build_query_as().fetch_all(&mut **tx).await?;
        for row in hits {
            if let Some(&index) = positions.get(row.meeting_id.as_str()) { meetings[index].hits.push(row.hit); }
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct LibraryPage {
    pub meetings: Vec<LibraryMeeting>,
    pub total_count: i64,
    pub next_cursor: Option<String>,
    pub index_updating: bool,
    pub pending_index_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchSyncStatus { pub indexed_count: i64, pub pending_count: i64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchCursor { signature: String, value: String, id: String }

/// Every user term/phrase becomes a quoted literal. FTS operators and punctuation
/// never reach MATCH as executable query syntax. Terms are ANDed across a meeting's
/// sources, while words inside a quoted phrase must occur together in one source.
pub fn compile_query(query: &str) -> Result<Vec<String>, String> {
    if query.chars().count() > MAX_QUERY_CHARS { return Err("Search is limited to 512 characters".into()); }
    if query.chars().any(|c| c.is_control()) { return Err("Search cannot contain control characters".into()); }
    let mut parts = Vec::new();
    let mut term = String::new();
    let mut quoted = false;
    let push = |term: &mut String, parts: &mut Vec<String>| {
        let value = term.trim();
        if value.chars().any(char::is_alphanumeric) {
            parts.push(format!("\"{}\"", value.replace('"', "\"\"")));
        }
        term.clear();
    };
    for c in query.chars() {
        match c {
            '"' => { push(&mut term, &mut parts); quoted = !quoted; }
            c if c.is_whitespace() && !quoted => push(&mut term, &mut parts),
            c => term.push(c),
        }
    }
    push(&mut term, &mut parts);
    if parts.len() > MAX_TERMS { return Err("Search is limited to 24 terms or phrases".into()); }
    Ok(parts)
}

pub fn normalize_metadata(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

fn validated_metadata(values: Vec<String>, label: &str, max_chars: usize) -> Result<Vec<(String, String)>, String> {
    if values.len() > 50 { return Err(format!("At most 50 {label} are allowed")); }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for value in values {
        let display = value.trim().to_owned();
        if display.is_empty() { continue; }
        if display.chars().count() > max_chars || display.chars().any(char::is_control) {
            return Err(format!("Each {label} value must contain at most {max_chars} characters and no control characters"));
        }
        let normalized = normalize_metadata(&display);
        if seen.insert(normalized.clone()) { result.push((normalized, display)); }
    }
    Ok(result)
}

pub async fn update_metadata(pool: &SqlitePool, meeting_id: &str, participants: Vec<String>, tags: Vec<String>) -> Result<(), String> {
    let participants = validated_metadata(participants, "participants", 120)?;
    let tags = validated_metadata(tags, "tags", 80)?;
    let mut tx = pool.begin().await.map_err(db_error)?;
    let changed = sqlx::query("UPDATE meetings SET updated_at = ? WHERE id = ? AND status <> 'deleting'")
        .bind(Utc::now().to_rfc3339()).bind(meeting_id).execute(&mut *tx).await.map_err(db_error)?.rows_affected();
    if changed != 1 { return Err("Meeting is unavailable or deletion is pending".into()); }
    sqlx::query("DELETE FROM meeting_participants WHERE meeting_id = ?").bind(meeting_id).execute(&mut *tx).await.map_err(db_error)?;
    sqlx::query("DELETE FROM meeting_tags WHERE meeting_id = ?").bind(meeting_id).execute(&mut *tx).await.map_err(db_error)?;
    for (normalized, display) in participants {
        sqlx::query("INSERT INTO meeting_participants(meeting_id, normalized, display_text) VALUES (?, ?, ?)")
            .bind(meeting_id).bind(normalized).bind(display).execute(&mut *tx).await.map_err(db_error)?;
    }
    for (normalized, display) in tags {
        sqlx::query("INSERT INTO meeting_tags(meeting_id, normalized, display_text) VALUES (?, ?, ?)")
            .bind(meeting_id).bind(normalized).bind(display).execute(&mut *tx).await.map_err(db_error)?;
    }
    tx.commit().await.map_err(db_error)
}

fn db_error(error: sqlx::Error) -> String { error.to_string() }

/// Recover text from legacy/editor JSON without indexing IDs, schema keys or
/// hallucinated metadata. Only human-readable text/Markdown fields are projected.
fn document_text(value: &serde_json::Value) -> String {
    fn visit(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::String(s) => out.push(s.clone()),
            serde_json::Value::Array(items) => { for item in items { visit(item, out); } }
            serde_json::Value::Object(map) => {
                if let Some(markdown) = map.get("markdown").and_then(|v| v.as_str()) {
                    out.push(markdown.to_owned()); return;
                }
                if let Some(text) = map.get("text").and_then(|v| v.as_str()) { out.push(text.to_owned()); }
                for key in ["content", "children", "blocks", "summary_json", "data", "summary", "sections", "items", "bullet_points", "points"] {
                    if let Some(child) = map.get(key) { visit(child, out); }
                }
                // Legacy structured summaries use arbitrary named section keys.
                if !map.contains_key("type") && !map.contains_key("schema_version") && !map.contains_key("markdown") {
                    for (key, child) in map {
                        if !["text", "content", "children", "blocks", "summary_json", "data", "summary", "sections", "items", "bullet_points", "points",
                            "id", "meeting_id", "metadata", "status", "source_snapshot", "claims", "coverage", "references"].contains(&key.as_str()) {
                            if let serde_json::Value::Object(_) | serde_json::Value::Array(_) = child { visit(child, out); }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    // Older summary saves sometimes encoded their JSON twice. Decode only the
    // document envelope, so ordinary note text that resembles JSON stays text.
    let mut decoded = value.clone();
    for _ in 0..2 {
        let Some(text) = decoded.as_str() else { break; };
        let Ok(next) = serde_json::from_str::<serde_json::Value>(text) else { break; };
        if !next.is_object() && !next.is_array() && !next.is_string() { break; }
        decoded = next;
    }
    let mut out = Vec::new(); visit(&decoded, &mut out); out.join("\n")
}

async fn upsert_document(tx: &mut sqlx::Transaction<'_, Sqlite>, meeting_id: &str, kind: &str, source_id: &str, text: &str, revision: Option<i64>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO meeting_search_sources(meeting_id, source_kind, source_id, text_content, revision) VALUES (?, ?, ?, ?, ?) ON CONFLICT(meeting_id, source_kind, source_id) DO UPDATE SET text_content = excluded.text_content, revision = excluded.revision WHERE text_content <> excluded.text_content OR revision IS NOT excluded.revision")
        .bind(meeting_id).bind(kind).bind(source_id).bind(text).bind(revision).execute(&mut **tx).await?;
    Ok(())
}

pub async fn sync_search_index(pool: &SqlitePool, limit: i64) -> Result<SearchSyncStatus, sqlx::Error> {
    let _sync = INDEX_SYNC_LOCK.lock().await;
    let started = std::time::Instant::now();
    let limit = limit.clamp(1, 100);
    let ids: Vec<String> = sqlx::query_scalar("SELECT d.meeting_id FROM meeting_search_dirty d JOIN meetings m ON m.id = d.meeting_id WHERE m.status <> 'deleting' ORDER BY d.updated_at, d.meeting_id LIMIT ?")
        .bind(limit).fetch_all(pool).await?;
    let mut indexed = 0;
    for meeting_id in ids {
        if indexed > 0 && started.elapsed() >= std::time::Duration::from_millis(50) { break; }
        // A transaction per meeting yields between meetings to source writers.
        // Acquire the write lock before the snapshot, avoiding deferred upgrade races.
        let mut tx = pool.begin().await?;
        let locked = sqlx::query("UPDATE meeting_search_dirty SET generation = generation WHERE meeting_id = ?")
            .bind(&meeting_id).execute(&mut *tx).await?.rows_affected();
        if locked == 0 { tx.commit().await?; continue; }
        let meeting: Option<(String, String)> = sqlx::query_as("SELECT title, status FROM meetings WHERE id = ?")
            .bind(&meeting_id).fetch_optional(&mut *tx).await?;
        let Some((title, status)) = meeting else { tx.commit().await?; continue; };
        if status == "deleting" { tx.commit().await?; continue; }
        upsert_document(&mut tx, &meeting_id, "title", &meeting_id, &title, None).await?;
        let notes: Option<(Option<String>, Option<String>, Option<String>, i64)> = sqlx::query_as("SELECT notes_plain_text, notes_markdown, notes_json, revision FROM meeting_notes WHERE meeting_id = ?")
            .bind(&meeting_id).fetch_optional(&mut *tx).await?;
        if let Some((plain, markdown, json, revision)) = notes {
            let text = plain.or_else(|| json.and_then(|s| serde_json::from_str(&s).ok().map(|v| document_text(&v)))).or(markdown).unwrap_or_default();
            upsert_document(&mut tx, &meeting_id, "original", &meeting_id, &text, Some(revision)).await?;
        } else {
            sqlx::query("DELETE FROM meeting_search_sources WHERE meeting_id = ? AND source_kind = 'original'").bind(&meeting_id).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO meeting_search_sources(meeting_id, source_kind, source_id, text_content, audio_start_time, audio_end_time) SELECT meeting_id, 'transcript', id, transcript, audio_start_time, audio_end_time FROM transcripts WHERE meeting_id = ? ON CONFLICT(meeting_id, source_kind, source_id) DO UPDATE SET text_content = excluded.text_content, audio_start_time = excluded.audio_start_time, audio_end_time = excluded.audio_end_time WHERE text_content <> excluded.text_content OR audio_start_time IS NOT excluded.audio_start_time OR audio_end_time IS NOT excluded.audio_end_time")
            .bind(&meeting_id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM meeting_search_sources WHERE meeting_id = ? AND source_kind = 'transcript' AND NOT EXISTS(SELECT 1 FROM transcripts t WHERE t.id = meeting_search_sources.source_id AND t.meeting_id = ?)")
            .bind(&meeting_id).bind(&meeting_id).execute(&mut *tx).await?;
        let mut selected: Option<(String, String, i64)> = sqlx::query_as("SELECT v.id, v.document_json, v.revision FROM meeting_enhancement_settings e JOIN meeting_enhancement_versions v ON v.id = e.approved_version_id AND v.meeting_id = e.meeting_id WHERE e.meeting_id = ? AND v.state IN ('approved', 'legacy') AND v.document_json IS NOT NULL")
            .bind(&meeting_id).fetch_optional(&mut *tx).await?;
        // Legacy records remain searchable before lazy version adoption. A present
        // settings row with no selection explicitly means no approved document.
        if selected.is_none() {
            let has_settings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meeting_enhancement_settings WHERE meeting_id = ?").bind(&meeting_id).fetch_one(&mut *tx).await?;
            if has_settings == 0 {
                let legacy: Option<String> = sqlx::query_scalar("SELECT result FROM summary_processes WHERE meeting_id = ? AND status = 'completed' AND result IS NOT NULL")
                    .bind(&meeting_id).fetch_optional(&mut *tx).await?;
                selected = legacy.map(|s| (format!("legacy-{meeting_id}"), s, 0));
            }
        }
        if let Some((id, json, revision)) = selected {
            let text = serde_json::from_str(&json).map(|v| document_text(&v)).unwrap_or(json);
            upsert_document(&mut tx, &meeting_id, "approved", &id, &text, Some(revision)).await?;
            sqlx::query("DELETE FROM meeting_search_sources WHERE meeting_id = ? AND source_kind = 'approved' AND source_id <> ?")
                .bind(&meeting_id).bind(id).execute(&mut *tx).await?;
        } else {
            sqlx::query("DELETE FROM meeting_search_sources WHERE meeting_id = ? AND source_kind = 'approved'").bind(&meeting_id).execute(&mut *tx).await?;
        }
        sqlx::query("DELETE FROM meeting_search_dirty WHERE meeting_id = ?").bind(&meeting_id).execute(&mut *tx).await?;
        tx.commit().await?;
        indexed += 1;
        tokio::task::yield_now().await;
    }
    let pending_count = pending_count(pool).await?;
    Ok(SearchSyncStatus { indexed_count: indexed, pending_count })
}

async fn pending_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM meeting_search_dirty d JOIN meetings m ON m.id = d.meeting_id WHERE m.status <> 'deleting'").fetch_one(pool).await
}

/// Call after primary commits/startup. Coalesces bursts; failures keep durable dirty
/// markers for a later retry. The worker never holds up a source-save response.
pub fn schedule_search_sync(pool: SqlitePool) {
    if INDEX_WORKER_RUNNING.swap(true, Ordering::SeqCst) { return; }
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let drained = loop {
            match sync_search_index(&pool, 10).await {
                Ok(status) if status.pending_count > 0 => tokio::time::sleep(std::time::Duration::from_millis(25)).await,
                Ok(_) => break true,
                Err(error) => { log::warn!("Meeting search indexing remains pending: {error}"); break false; }
            }
        };
        INDEX_WORKER_RUNNING.store(false, Ordering::SeqCst);
        // A commit between the last empty check and clearing the running flag
        // may have coalesced its wake-up into this worker. Recheck after release.
        if drained && pending_count(&pool).await.is_ok_and(|count| count > 0) { schedule_search_sync(pool); }
    });
}

/// Rebuild only derived data. Transaction rollback preserves the previous index
/// if interrupted; canonical meeting/note/transcript/version rows are untouched.
pub async fn rebuild_search_index(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO meeting_search_fts(meeting_search_fts) VALUES ('rebuild')").execute(&mut *tx).await?;
    sqlx::query("INSERT INTO meeting_search_dirty(meeting_id, updated_at) SELECT id, updated_at FROM meetings WHERE status <> 'deleting' ON CONFLICT(meeting_id) DO UPDATE SET generation = generation + 1, updated_at = excluded.updated_at")
        .execute(&mut *tx).await?;
    tx.commit().await
}

fn append_filters<'a>(builder: &mut QueryBuilder<'a, Sqlite>, request: &'a LibrarySearchRequest, terms: &'a [String]) {
    builder.push(" WHERE m.status <> 'deleting'");
    for term in terms {
        builder.push(" AND m.id IN (SELECT s.meeting_id FROM meeting_search_fts JOIN meeting_search_sources s ON s.id = meeting_search_fts.rowid WHERE meeting_search_fts MATCH ")
            .push_bind(term).push(")");
    }
    if let Some(participant) = &request.participant {
        builder.push(" AND EXISTS(SELECT 1 FROM meeting_participants p WHERE p.meeting_id = m.id AND p.normalized = ").push_bind(normalize_metadata(participant)).push(")");
    }
    if let Some(tag) = &request.tag {
        builder.push(" AND EXISTS(SELECT 1 FROM meeting_tags t WHERE t.meeting_id = m.id AND t.normalized = ").push_bind(normalize_metadata(tag)).push(")");
    }
    if let Some(from) = &request.date_from {
        builder.push(" AND julianday(COALESCE(m.started_at, m.created_at)) >= julianday(").push_bind(from).push(")");
    }
    if let Some(to) = &request.date_to {
        builder.push(" AND julianday(COALESCE(m.started_at, m.created_at)) < julianday(").push_bind(to).push(")");
    }
    if let Some(status) = &request.status { builder.push(" AND m.status = ").push_bind(status); }
}

pub async fn search_meetings(pool: &SqlitePool, mut request: LibrarySearchRequest) -> Result<LibraryPage, String> {
    let terms = compile_query(&request.query)?;
    if !request.query.trim().is_empty() && terms.is_empty() { return Err("Enter a word or phrase to search".into()); }
    let limit = request.limit.unwrap_or(30);
    if !(1..=MAX_PAGE).contains(&limit) { return Err("Page size must be between 1 and 100".into()); }
    for date in [&request.date_from, &request.date_to].into_iter().flatten() {
        DateTime::parse_from_rfc3339(date).map_err(|_| "Date bounds must be RFC3339 instants".to_owned())?;
    }
    if let (Some(from), Some(to)) = (&request.date_from, &request.date_to) {
        if DateTime::parse_from_rfc3339(from).unwrap() >= DateTime::parse_from_rfc3339(to).unwrap() { return Err("End date must follow start date".into()); }
    }
    for value in [&request.participant, &request.tag].into_iter().flatten() {
        if value.chars().count() > 120 || value.chars().any(char::is_control) { return Err("Invalid metadata filter".into()); }
    }
    if let Some(status) = &request.status {
        if !["draft", "recording", "finalizing", "completed", "interrupted"].contains(&status.as_str()) { return Err("Invalid meeting status filter".into()); }
    }
    let sort = request.sort.clone().unwrap_or_else(|| if terms.is_empty() { "date_desc".into() } else { "relevance".into() });
    let (sort_expr, direction) = match sort.as_str() {
        "date_desc" => ("printf('%.10f', COALESCE(julianday(COALESCE(m.started_at, m.created_at)), 0))", "DESC"),
        "date_asc" => ("printf('%.10f', COALESCE(julianday(COALESCE(m.started_at, m.created_at)), 0))", "ASC"),
        "updated_desc" => ("printf('%.10f', COALESCE(julianday(m.updated_at), 0))", "DESC"),
        "title_asc" => ("lower(m.title)", "ASC"),
        // Source count ranks meeting-level matches; exact matching remains FTS5.
        "relevance" => ("printf('%012d', COALESCE(h.score, 0))", "DESC"),
        _ => return Err("Invalid meeting sort".into()),
    };
    request.sort = Some(sort);
    let cursor_raw = request.cursor.take();
    // Bind cursors to the query/filters/sort, preventing accidental cross-query pages.
    let signature = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    let cursor: Option<SearchCursor> = cursor_raw.map(|raw| -> Result<SearchCursor, String> {
        if raw.len() > 4096 { return Err::<SearchCursor, String>("Invalid search cursor".into()); }
        let cursor: SearchCursor = serde_json::from_str(&raw).map_err(|_| "Invalid search cursor".to_owned())?;
        if cursor.signature != signature || cursor.id.len() > 512 || cursor.value.len() > 2048 { return Err("Search cursor belongs to another query".into()); }
        Ok(cursor)
    }).transpose()?;
    let union_query = terms.join(" OR ");
    let mut tx = pool.begin().await.map_err(db_error)?;
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meeting_search_dirty d JOIN meetings m ON m.id = d.meeting_id WHERE m.status <> 'deleting'").fetch_one(&mut *tx).await.map_err(db_error)?;
    let mut count = QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM meetings m");
    append_filters(&mut count, &request, &terms);
    let total_count: i64 = count.build_query_scalar().fetch_one(&mut *tx).await.map_err(db_error)?;
    let mut query = QueryBuilder::<Sqlite>::new("");
    if !terms.is_empty() {
        query.push("WITH hits_count AS (SELECT s.meeting_id, COUNT(*) AS score FROM meeting_search_fts JOIN meeting_search_sources s ON s.id = meeting_search_fts.rowid WHERE meeting_search_fts MATCH ")
            .push_bind(&union_query).push(" GROUP BY s.meeting_id) ");
    }
    query.push("SELECT m.id, m.title, m.created_at, m.updated_at, m.started_at, m.ended_at, m.timezone, m.status, (m.started_at IS NULL) AS date_is_creation_fallback, EXISTS(SELECT 1 FROM meeting_search_dirty d WHERE d.meeting_id = m.id) AS index_updating, ")
        .push(sort_expr).push(" AS sort_value FROM meetings m");
    if !terms.is_empty() { query.push(" LEFT JOIN hits_count h ON h.meeting_id = m.id"); }
    else if request.sort.as_deref() == Some("relevance") { query.push(" LEFT JOIN (SELECT NULL AS meeting_id, 0 AS score) h ON h.meeting_id = m.id"); }
    append_filters(&mut query, &request, &terms);
    if let Some(cursor) = &cursor {
        let op = if direction == "ASC" { ">" } else { "<" };
        query.push(" AND (").push(sort_expr).push(" ").push(op).push(" ").push_bind(&cursor.value)
            .push(" OR (").push(sort_expr).push(" = ").push_bind(&cursor.value).push(" AND m.id > ").push_bind(&cursor.id).push("))");
    }
    query.push(" ORDER BY ").push(sort_expr).push(" ").push(direction).push(", m.id ASC LIMIT ").push_bind(limit + 1);
    let mut meetings: Vec<LibraryMeeting> = query.build_query_as().fetch_all(&mut *tx).await.map_err(db_error)?;
    let has_more = meetings.len() as i64 > limit;
    if has_more { meetings.truncate(limit as usize); }
    enrich_page(&mut tx, &mut meetings, (!terms.is_empty()).then_some(union_query.as_str())).await.map_err(db_error)?;
    let next_cursor = if has_more {
        meetings.last().map(|m| serde_json::to_string(&SearchCursor { signature, value: m.sort_value.clone(), id: m.id.clone() }).unwrap())
    } else { None };
    tx.commit().await.map_err(db_error)?;
    Ok(LibraryPage { meetings, total_count, next_cursor, index_updating: pending > 0, pending_index_count: pending })
}

#[tauri::command]
pub async fn api_search_meetings(state: tauri::State<'_, AppState>, request: LibrarySearchRequest) -> Result<LibraryPage, String> {
    schedule_search_sync(state.db_manager.pool().clone());
    search_meetings(state.db_manager.pool(), request).await
}
#[tauri::command]
pub async fn api_list_meetings(state: tauri::State<'_, AppState>, mut request: LibrarySearchRequest) -> Result<LibraryPage, String> {
    request.query.clear();
    schedule_search_sync(state.db_manager.pool().clone());
    search_meetings(state.db_manager.pool(), request).await
}
#[tauri::command]
pub async fn api_update_meeting_metadata(state: tauri::State<'_, AppState>, meeting_id: String, participants: Vec<String>, tags: Vec<String>) -> Result<(), String> {
    update_metadata(state.db_manager.pool(), &meeting_id, participants, tags).await
}
#[tauri::command]
pub async fn api_sync_meeting_search(state: tauri::State<'_, AppState>, limit: Option<i64>) -> Result<SearchSyncStatus, String> {
    sync_search_index(state.db_manager.pool(), limit.unwrap_or(25)).await.map_err(db_error)
}

#[cfg(test)]
mod tests;
