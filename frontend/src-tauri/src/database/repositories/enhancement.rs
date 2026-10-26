//! Protected derived documents. This repository never updates meeting_notes.
use crate::summary::grounding::{fingerprint, SourceSnapshot};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnhancementVersion {
    pub id: String,
    pub meeting_id: String,
    pub state: String,
    pub status: String,
    pub revision: i64,
    pub document_json: Option<String>,
    pub source_snapshot_json: Option<String>,
    pub source_fingerprint: Option<String>,
    pub notes_revision: Option<i64>,
    pub template_id: String,
    pub instructions: String,
    pub provider_config_json: Option<String>,
    #[serde(skip)]
    pub last_commit_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub error: Option<String>,
    #[sqlx(skip)]
    pub source_stale: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnhancementSettings {
    pub template_id: String,
    pub instructions: String,
    pub revision: i64,
}
#[derive(Debug, Serialize)]
pub struct EnhancementCollection {
    pub meeting_id: String,
    pub approved_version_id: Option<String>,
    pub settings: EnhancementSettings,
    pub versions: Vec<EnhancementVersion>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct VersionSave {
    pub meeting_id: String,
    pub version_id: String,
    pub expected_revision: i64,
    pub document: Value,
    pub commit_id: Option<String>,
}

fn db(error: sqlx::Error) -> String {
    format!("Enhancement storage error: {error}")
}
async fn guard(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    meeting_id: &str,
) -> Result<(), String> {
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM meetings WHERE id = ?")
        .bind(meeting_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db)?;
    match status.as_deref() {
        None => Err("Meeting not found".into()),
        Some("deleting") => Err("Meeting deletion is pending".into()),
        _ => Ok(()),
    }
}
/// Idempotent adoption preserves the exact legacy result, including unknown keys.
pub async fn adopt_legacy(pool: &SqlitePool, meeting_id: &str) -> Result<(), String> {
    // Ordinary reads of an adopted collection must not acquire SQLite's writer.
    let adopted: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meeting_enhancement_settings e JOIN meetings m ON m.id=e.meeting_id WHERE e.meeting_id=? AND m.status != 'deleting' AND (e.approved_version_id IS NOT NULL OR NOT EXISTS(SELECT 1 FROM summary_processes p WHERE p.meeting_id=e.meeting_id AND p.result IS NOT NULL)))")
        .bind(meeting_id).fetch_one(pool).await.map_err(db)?;
    if adopted { return Ok(()); }
    let mut tx = pool.begin().await.map_err(db)?;
    sqlx::query("INSERT OR IGNORE INTO meeting_enhancement_settings(meeting_id) SELECT id FROM meetings WHERE id = ? AND status != 'deleting'")
        .bind(meeting_id).execute(&mut *tx).await.map_err(db)?;
    guard(&mut tx, meeting_id).await?;
    let prior: Option<String> = sqlx::query_scalar(
        "SELECT approved_version_id FROM meeting_enhancement_settings WHERE meeting_id = ?",
    )
    .bind(meeting_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    if prior.is_none() {
        let raw: Option<String> = sqlx::query_scalar(
            "SELECT result FROM summary_processes WHERE meeting_id = ? AND result IS NOT NULL",
        )
        .bind(meeting_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db)?;
        if let Some(raw) = raw {
            let id = format!("legacy-{meeting_id}");
            let now = Utc::now().to_rfc3339();
            sqlx::query("INSERT OR IGNORE INTO meeting_enhancement_versions(id,meeting_id,state,status,document_json,created_at,updated_at) VALUES (?,?,'legacy','unverified',?,?,?)")
                .bind(&id).bind(meeting_id).bind(raw).bind(&now).bind(&now).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("UPDATE meeting_enhancement_settings SET approved_version_id = ? WHERE meeting_id = ? AND approved_version_id IS NULL")
                .bind(id).bind(meeting_id).execute(&mut *tx).await.map_err(db)?;
        }
    }
    tx.commit().await.map_err(db)
}
pub async fn get_version(
    pool: &SqlitePool,
    meeting_id: &str,
    id: &str,
) -> Result<EnhancementVersion, String> {
    sqlx::query_as("SELECT v.* FROM meeting_enhancement_versions v JOIN meetings m ON m.id=v.meeting_id WHERE v.id=? AND v.meeting_id=? AND m.status != 'deleting'")
        .bind(id).bind(meeting_id).fetch_optional(pool).await.map_err(db)?.ok_or_else(|| "Enhancement version not found".into())
}
pub async fn list(pool: &SqlitePool, meeting_id: &str) -> Result<EnhancementCollection, String> {
    list_page(pool, meeting_id, false, None).await
}
pub async fn list_page(pool: &SqlitePool, meeting_id: &str, metadata_only: bool, cursor: Option<&str>) -> Result<EnhancementCollection, String> {
    adopt_legacy(pool, meeting_id).await?;
    let settings = sqlx::query_as("SELECT template_id,instructions,revision FROM meeting_enhancement_settings WHERE meeting_id=?")
        .bind(meeting_id).fetch_one(pool).await.map_err(db)?;
    let approved_version_id = sqlx::query_scalar(
        "SELECT approved_version_id FROM meeting_enhancement_settings WHERE meeting_id=?",
    )
    .bind(meeting_id)
    .fetch_one(pool)
    .await
    .map_err(db)?;
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new(if metadata_only {
        "SELECT id,meeting_id,state,status,revision,NULL AS document_json,NULL AS source_snapshot_json,source_fingerprint,notes_revision,template_id,'' AS instructions,NULL AS provider_config_json,last_commit_id,created_at,updated_at,error FROM meeting_enhancement_versions WHERE meeting_id="
    } else { "SELECT * FROM meeting_enhancement_versions WHERE meeting_id=" });
    query.push_bind(meeting_id);
    if metadata_only {
        if let Some(raw) = cursor {
            if raw.len() > 4096 { return Err("Invalid enhancement history cursor".into()); }
            let (created, id): (String, String) = serde_json::from_str(raw).map_err(|_| "Invalid enhancement history cursor")?;
            query.push(" AND (created_at < ").push_bind(created.clone()).push(" OR (created_at = ").push_bind(created).push(" AND id > ").push_bind(id).push("))");
        }
    }
    query.push(" ORDER BY created_at DESC,id");
    if metadata_only { query.push(" LIMIT 51"); }
    let mut versions: Vec<EnhancementVersion> = query.build_query_as().fetch_all(pool).await.map_err(db)?;
    let next_cursor = if metadata_only && versions.len() > 50 {
        versions.truncate(50);
        versions.last().map(|v| serde_json::to_string(&(&v.created_at, &v.id)).unwrap())
    } else { None };
    let current = SourceSnapshot::read(pool, meeting_id, None, true)
        .await
        .ok();
    for v in &mut versions {
        v.source_stale = match (&v.source_fingerprint, &v.source_snapshot_json, &current) {
            (Some(stored), _, Some(current)) => stored != &current.source_fingerprint,
            (_, Some(raw), Some(current)) => serde_json::from_str::<SourceSnapshot>(raw)
                .map(|snapshot| snapshot.source_fingerprint != current.source_fingerprint)
                .unwrap_or(true),
            _ => v.state != "legacy",
        };
    }
    Ok(EnhancementCollection {
        meeting_id: meeting_id.into(),
        approved_version_id,
        settings,
        versions,
        next_cursor,
    })
}
pub async fn save_settings(
    pool: &SqlitePool,
    meeting_id: &str,
    template_id: &str,
    instructions: &str,
    expected_revision: i64,
) -> Result<EnhancementSettings, String> {
    if instructions.len() > 64 * 1024 {
        return Err("Instructions exceed 64 KiB".into());
    }
    adopt_legacy(pool, meeting_id).await?;
    let result = sqlx::query("UPDATE meeting_enhancement_settings SET template_id=?,instructions=?,revision=revision+1 WHERE meeting_id=? AND revision=? AND EXISTS(SELECT 1 FROM meetings WHERE id=? AND status != 'deleting')")
        .bind(template_id).bind(instructions).bind(meeting_id).bind(expected_revision).bind(meeting_id).execute(pool).await.map_err(db)?;
    if result.rows_affected() != 1 {
        return Err("Settings revision conflict; reload before saving".into());
    }
    Ok(EnhancementSettings {
        template_id: template_id.into(),
        instructions: instructions.into(),
        revision: expected_revision + 1,
    })
}
pub async fn create_candidate(
    pool: &SqlitePool,
    meeting_id: &str,
    snapshot: &SourceSnapshot,
    template_id: &str,
    instructions: &str,
    provider: &Value,
) -> Result<EnhancementVersion, String> {
    adopt_legacy(pool, meeting_id).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let raw = serde_json::to_string(snapshot).map_err(|_| "Cannot serialize source snapshot")?;
    let source_key = fingerprint(&format!("{raw}\n{provider}\n{template_id}\n{instructions}"));
    let result = sqlx::query("INSERT INTO meeting_enhancement_versions(id,meeting_id,state,status,source_snapshot_json,source_fingerprint,notes_revision,template_id,instructions,provider_config_json,created_at,updated_at) SELECT ?,id,'candidate','generating',?,?,?,?,?,?,?,? FROM meetings WHERE id=? AND status NOT IN ('deleting','recording','finalizing')")
        .bind(&id).bind(raw).bind(source_key).bind(snapshot.notes_revision).bind(template_id).bind(instructions).bind(provider.to_string()).bind(&now).bind(&now).bind(meeting_id).execute(pool).await.map_err(db)?;
    if result.rows_affected() != 1 {
        return Err("Finish recording and transcript finalization before enhancement".into());
    }
    get_version(pool, meeting_id, &id).await
}
pub async fn finish(
    pool: &SqlitePool,
    meeting_id: &str,
    id: &str,
    document: Option<&Value>,
    status: &str,
    error: Option<&str>,
) -> Result<bool, String> {
    let mut tx = pool.begin().await.map_err(db)?;
    let result = sqlx::query("UPDATE meeting_enhancement_versions SET status=?,document_json=?,error=?,revision=revision+1,updated_at=? WHERE id=? AND meeting_id=? AND status='generating' AND state='candidate' AND EXISTS(SELECT 1 FROM meetings WHERE id=? AND status != 'deleting')")
        .bind(status).bind(document.map(Value::to_string)).bind(error).bind(Utc::now().to_rfc3339()).bind(id).bind(meeting_id).bind(meeting_id).execute(&mut *tx).await.map_err(db)?;
    if result.rows_affected() == 1 && matches!(status, "ready" | "discarded") {
        sqlx::query("DELETE FROM meeting_enhancement_work WHERE version_id=?").bind(id).execute(&mut *tx).await.map_err(db)?;
    }
    tx.commit().await.map_err(db)?;
    Ok(result.rows_affected() == 1)
}
pub async fn save_version(
    pool: &SqlitePool,
    mut input: VersionSave,
) -> Result<EnhancementVersion, String> {
    if !input.document.is_object() {
        return Err("Document must be an object".into());
    }
    if let Some(id) = &input.commit_id {
        uuid::Uuid::parse_str(id).map_err(|_| "Invalid save commit ID")?;
    }
    if let Some(blocks) = input.document.get("summary_json") {
        if !blocks.is_array() {
            return Err("Block document must be an array".into());
        }
    }
    if let Some(markdown) = input.document.get("markdown") {
        if !markdown.is_string() && !markdown.is_null() {
            return Err("Markdown must be text or null".into());
        }
    }
    let object = input.document.as_object_mut().unwrap();
    // Compatibility responses carry their displayed revision as presentation
    // metadata. Never save that stale revision back into canonical content.
    object.remove("_enhancement_version_id");
    object.remove("_enhancement_revision");
    object.remove("english_cache");
    object.insert("user_edited".into(), Value::Bool(true));
    if let Some(claims) = object.get_mut("claims").and_then(Value::as_array_mut) {
        for claim in claims {
            if let Some(claim) = claim.as_object_mut() {
                claim.insert("status".into(), Value::String("user_edited".into()));
                claim.insert("owner".into(), Value::Null);
                claim.insert("deadline".into(), Value::Null);
            }
        }
    }
    let mut tx = pool.begin().await.map_err(db)?;
    // Take write lock before revision reads.
    sqlx::query("UPDATE meeting_enhancement_settings SET revision=revision WHERE meeting_id=?")
        .bind(&input.meeting_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    guard(&mut tx, &input.meeting_id).await?;
    let version: EnhancementVersion =
        sqlx::query_as("SELECT * FROM meeting_enhancement_versions WHERE meeting_id=? AND id=?")
            .bind(&input.meeting_id)
            .bind(&input.version_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db)?
            .ok_or("Version not found")?;
    if input.commit_id.is_some()
        && version.last_commit_id == input.commit_id
        && version.revision == input.expected_revision + 1
    {
        tx.commit().await.map_err(db)?;
        return Ok(version);
    }
    if version.revision != input.expected_revision {
        return Err("Document revision conflict; local edits are retained".into());
    }
    if !matches!(version.state.as_str(), "candidate" | "approved" | "legacy")
        || !matches!(version.status.as_str(), "ready" | "unverified")
    {
        return Err("This version is read-only".into());
    }
    let raw = input.document.to_string();
    sqlx::query("UPDATE meeting_enhancement_versions SET document_json=?,revision=revision+1,last_commit_id=?,updated_at=? WHERE id=? AND revision=?")
        .bind(&raw).bind(&input.commit_id).bind(Utc::now().to_rfc3339()).bind(&input.version_id).bind(input.expected_revision).execute(&mut *tx).await.map_err(db)?;
    sqlx::query("UPDATE summary_processes SET result=?,updated_at=? WHERE meeting_id=? AND EXISTS(SELECT 1 FROM meeting_enhancement_settings WHERE meeting_id=? AND approved_version_id=?)")
        .bind(raw).bind(Utc::now().to_rfc3339()).bind(&input.meeting_id).bind(&input.meeting_id).bind(&input.version_id).execute(&mut *tx).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    get_version(pool, &input.meeting_id, &input.version_id).await
}
pub async fn approve(
    pool: &SqlitePool,
    meeting_id: &str,
    id: &str,
    expected_revision: i64,
    expected_approved: Option<&str>,
) -> Result<EnhancementVersion, String> {
    let mut tx = pool.begin().await.map_err(db)?;
    sqlx::query("UPDATE meeting_enhancement_settings SET revision=revision WHERE meeting_id=?")
        .bind(meeting_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    guard(&mut tx, meeting_id).await?;
    let current: Option<String> = sqlx::query_scalar(
        "SELECT approved_version_id FROM meeting_enhancement_settings WHERE meeting_id=?",
    )
    .bind(meeting_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db)?;
    if current.as_deref() != expected_approved {
        return Err("Approval changed; reload before approving".into());
    }
    let v: EnhancementVersion =
        sqlx::query_as("SELECT * FROM meeting_enhancement_versions WHERE id=? AND meeting_id=?")
            .bind(id)
            .bind(meeting_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db)?
            .ok_or("Version not found")?;
    if v.revision != expected_revision
        || v.state != "candidate"
        || v.status != "ready"
        || v.document_json.is_none()
    {
        return Err("Save the current candidate revision before approval".into());
    }
    sqlx::query("UPDATE meeting_enhancement_versions SET state='archived' WHERE meeting_id=? AND state IN ('approved','legacy') AND id IN(SELECT approved_version_id FROM meeting_enhancement_settings WHERE meeting_id=?)").bind(meeting_id).bind(meeting_id).execute(&mut *tx).await.map_err(db)?;
    sqlx::query("UPDATE meeting_enhancement_versions SET state='approved',revision=revision+1,updated_at=? WHERE id=?").bind(Utc::now().to_rfc3339()).bind(id).execute(&mut *tx).await.map_err(db)?;
    sqlx::query("UPDATE meeting_enhancement_settings SET approved_version_id=? WHERE meeting_id=?")
        .bind(id)
        .bind(meeting_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    let now = Utc::now();
    sqlx::query("INSERT INTO summary_processes(meeting_id,status,result,created_at,updated_at) VALUES(?,'completed',?,?,?) ON CONFLICT(meeting_id) DO UPDATE SET status='completed',result=excluded.result,updated_at=excluded.updated_at,error=NULL")
        .bind(meeting_id).bind(v.document_json).bind(now).bind(now).execute(&mut *tx).await.map_err(db)?;
    sqlx::query("UPDATE meetings SET updated_at=? WHERE id=?")
        .bind(now)
        .bind(meeting_id)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
    tx.commit().await.map_err(db)?;
    get_version(pool, meeting_id, id).await
}
pub async fn discard(
    pool: &SqlitePool,
    meeting_id: &str,
    id: &str,
    revision: i64,
) -> Result<EnhancementVersion, String> {
    let mut tx = pool.begin().await.map_err(db)?;
    let result=sqlx::query("UPDATE meeting_enhancement_versions SET status='discarded',revision=revision+1,updated_at=? WHERE id=? AND meeting_id=? AND state='candidate' AND revision=? AND EXISTS(SELECT 1 FROM meetings WHERE id=? AND status != 'deleting')")
        .bind(Utc::now().to_rfc3339()).bind(id).bind(meeting_id).bind(revision).bind(meeting_id).execute(&mut *tx).await.map_err(db)?;
    if result.rows_affected() != 1 {
        return Err("Version revision conflict or approved version cannot be discarded".into());
    }
    sqlx::query("DELETE FROM meeting_enhancement_work WHERE version_id=?").bind(id).execute(&mut *tx).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    get_version(pool, meeting_id, id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    async fn pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status) VALUES('meeting-fixture','Synthetic fixture',?,?, 'completed')")
            .bind(Utc::now()).bind(Utc::now()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO meeting_notes(meeting_id,notes_json,revision,created_at,updated_at) VALUES('meeting-fixture',?,1,?,?)")
            .bind(r#"[{"id":"original-block","type":"paragraph","content":[{"type":"text","text":"Proposal remains tentative."}]}]"#).bind(Utc::now()).bind(Utc::now()).execute(&pool).await.unwrap();
        pool
    }
    async fn candidate(pool: &SqlitePool) -> EnhancementVersion {
        let snapshot = SourceSnapshot::read(pool, "meeting-fixture", Some(1), false)
            .await
            .unwrap();
        let version = create_candidate(
            pool,
            "meeting-fixture",
            &snapshot,
            "standard_meeting",
            "Keep proposal tentative",
            &json!({"provider":"builtin-ai","model":"fixture"}),
        )
        .await
        .unwrap();
        finish(pool,"meeting-fixture",&version.id,Some(&json!({"markdown":"Proposal remains tentative.","claims":[],"unknown_field":{"keep":true}})),"ready",None).await.unwrap();
        get_version(pool, "meeting-fixture", &version.id)
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn metadata_history_is_bounded_and_keeps_selected_document_out_of_pages() {
        let pool = pool().await;
        let snapshot = SourceSnapshot::read(&pool, "meeting-fixture", Some(1), false).await.unwrap();
        for i in 0..55 {
            sqlx::query("INSERT INTO meeting_enhancement_versions(id,meeting_id,state,status,document_json,source_snapshot_json,source_fingerprint,notes_revision,created_at,updated_at) VALUES (?,'meeting-fixture','candidate','ready',?,?,?,1,'2026-10-25','2026-10-25')")
                .bind(format!("v-{i:03}")).bind(serde_json::json!({"markdown":"large ".repeat(5000)}).to_string()).bind(serde_json::to_string(&snapshot).unwrap()).bind(&snapshot.source_fingerprint).execute(&pool).await.unwrap();
        }
        let first = list_page(&pool,"meeting-fixture",true,None).await.unwrap();
        assert_eq!(first.versions.len(),50);
        assert!(first.versions.iter().all(|v|v.document_json.is_none() && v.source_snapshot_json.is_none() && !v.source_stale));
        assert!(serde_json::to_vec(&first).unwrap().len()<100_000);
        let second = list_page(&pool,"meeting-fixture",true,first.next_cursor.as_deref()).await.unwrap();
        assert_eq!(second.versions.len(),5); assert!(second.next_cursor.is_none());
        assert_ne!(first.versions.last().unwrap().id,second.versions[0].id);
        assert!(get_version(&pool,"meeting-fixture","v-054").await.unwrap().document_json.is_some());
        assert!(get_version(&pool,"other-meeting","v-054").await.is_err());
    }
    #[tokio::test]
    async fn enhancement_legacy_adoption_preserves_unknown_fields_and_is_idempotent() {
        let pool = pool().await;
        let raw = r#"{"summary_json":[{"content":[{"text":"legacy"}]}],"unrecognized":{"keep":7}}"#;
        sqlx::query("INSERT INTO summary_processes(meeting_id,status,created_at,updated_at,result) VALUES('meeting-fixture','completed',?,?,?)").bind(Utc::now()).bind(Utc::now()).bind(raw).execute(&pool).await.unwrap();
        for _ in 0..2 {
            adopt_legacy(&pool, "meeting-fixture").await.unwrap();
        }
        let collection = list(&pool, "meeting-fixture").await.unwrap();
        assert_eq!(collection.versions.len(), 1);
        assert_eq!(collection.versions[0].document_json.as_deref(), Some(raw));
        assert_eq!(collection.versions[0].state, "legacy");
        assert_eq!(
            collection.approved_version_id.as_deref(),
            Some("legacy-meeting-fixture")
        );
    }
    #[tokio::test]
    async fn enhancement_regeneration_preserves_original_and_approved_edits() {
        let pool = pool().await;
        let original: String = sqlx::query_scalar("SELECT notes_json FROM meeting_notes")
            .fetch_one(&pool)
            .await
            .unwrap();
        let a = candidate(&pool).await;
        let a = approve(&pool, "meeting-fixture", &a.id, a.revision, None)
            .await
            .unwrap();
        let commit = uuid::Uuid::new_v4().to_string();
        let edited=save_version(&pool,VersionSave{meeting_id:"meeting-fixture".into(),version_id:a.id.clone(),expected_revision:a.revision,document:json!({"markdown":"My approved edits","summary_json":[],"unknown_field":{"keep":true},"_enhancement_version_id":a.id,"_enhancement_revision":a.revision}),commit_id:Some(commit.clone())}).await.unwrap();
        let saved: Value = serde_json::from_str(edited.document_json.as_deref().unwrap()).unwrap();
        assert_eq!(saved["unknown_field"]["keep"], true);
        assert!(saved.get("_enhancement_revision").is_none());
        assert!(saved.get("_enhancement_version_id").is_none());
        let replay = save_version(
            &pool,
            VersionSave {
                meeting_id: "meeting-fixture".into(),
                version_id: a.id.clone(),
                expected_revision: a.revision,
                document: json!({"markdown":"My approved edits"}),
                commit_id: Some(commit),
            },
        )
        .await
        .unwrap();
        assert_eq!(replay.revision, edited.revision);
        let b = candidate(&pool).await;
        assert_eq!(
            list(&pool, "meeting-fixture")
                .await
                .unwrap()
                .approved_version_id
                .as_deref(),
            Some(a.id.as_str())
        );
        assert!(approve(&pool, "meeting-fixture", &b.id, b.revision, None)
            .await
            .is_err());
        let b = approve(&pool, "meeting-fixture", &b.id, b.revision, Some(&a.id))
            .await
            .unwrap();
        assert_eq!(
            get_version(&pool, "meeting-fixture", &a.id)
                .await
                .unwrap()
                .state,
            "archived"
        );
        assert!(get_version(&pool, "meeting-fixture", &a.id)
            .await
            .unwrap()
            .document_json
            .unwrap()
            .contains("My approved edits"));
        assert!(discard(&pool, "meeting-fixture", &b.id, b.revision)
            .await
            .is_err());
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT notes_json FROM meeting_notes")
                .fetch_one(&pool)
                .await
                .unwrap(),
            original
        );
    }
    #[tokio::test]
    async fn enhancement_stale_revisions_and_late_completion_rejected() {
        let pool = pool().await;
        let v = candidate(&pool).await;
        assert!(save_version(
            &pool,
            VersionSave {
                meeting_id: "meeting-fixture".into(),
                version_id: v.id.clone(),
                expected_revision: v.revision - 1,
                document: json!({"markdown":"stale"}),
                commit_id: None
            }
        )
        .await
        .is_err());
        discard(&pool, "meeting-fixture", &v.id, v.revision)
            .await
            .unwrap();
        assert!(!finish(
            &pool,
            "meeting-fixture",
            &v.id,
            Some(&json!({"markdown":"late"})),
            "ready",
            None
        )
        .await
        .unwrap());
        assert_eq!(
            get_version(&pool, "meeting-fixture", &v.id)
                .await
                .unwrap()
                .status,
            "discarded"
        );
    }
    #[tokio::test]
    async fn enhancement_source_snapshot_and_deletion_guard() {
        let pool = pool().await;
        assert!(
            SourceSnapshot::read(&pool, "meeting-fixture", Some(0), false)
                .await
                .is_err()
        );
        let v = candidate(&pool).await;
        sqlx::query("UPDATE meeting_notes SET revision=2 WHERE meeting_id='meeting-fixture'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(list(&pool, "meeting-fixture").await.unwrap().versions[0].source_stale);
        sqlx::query("UPDATE meetings SET status='deleting' WHERE id='meeting-fixture'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(approve(&pool, "meeting-fixture", &v.id, v.revision, None)
            .await
            .is_err());
        assert!(list(&pool, "meeting-fixture").await.is_err());
        assert!(!finish(
            &pool,
            "meeting-fixture",
            &v.id,
            None,
            "failed",
            Some("late")
        )
        .await
        .unwrap());
    }
    #[tokio::test]
    async fn enhancement_notes_only_and_active_recording_contract() {
        let pool = pool().await;
        let source = SourceSnapshot::read(&pool, "meeting-fixture", Some(1), false)
            .await
            .unwrap();
        assert_eq!(source.sources.len(), 1);
        assert_eq!(source.sources[0].source_id, "note:original-block");
        sqlx::query("UPDATE meetings SET status='recording' WHERE id='meeting-fixture'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            SourceSnapshot::read(&pool, "meeting-fixture", Some(1), false)
                .await
                .is_err()
        );
        assert!(create_candidate(
            &pool,
            "meeting-fixture",
            &source,
            "standard_meeting",
            "",
            &json!({})
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn interrupted_generation_preserves_committed_original_and_approval() {
        let pool = pool().await;
        let prior = candidate(&pool).await;
        let prior = approve(&pool, "meeting-fixture", &prior.id, prior.revision, None)
            .await
            .unwrap();
        let snapshot = SourceSnapshot::read(&pool, "meeting-fixture", Some(1), false)
            .await
            .unwrap();
        let interrupted = create_candidate(
            &pool,
            "meeting-fixture",
            &snapshot,
            "highlights",
            "",
            &json!({"provider":"fixture"}),
        )
        .await
        .unwrap();
        crate::summary::enhancement::recover_interrupted_enhancements(&pool)
            .await
            .unwrap();
        let versions = list(&pool, "meeting-fixture").await.unwrap();
        assert_eq!(
            versions.approved_version_id.as_deref(),
            Some(prior.id.as_str())
        );
        assert_eq!(
            get_version(&pool, "meeting-fixture", &prior.id)
                .await
                .unwrap()
                .document_json,
            prior.document_json
        );
        let recovered = get_version(&pool, "meeting-fixture", &interrupted.id)
            .await
            .unwrap();
        assert_eq!(recovered.status, "failed");
        assert!(recovered.error.unwrap().contains("interrupted"));
        assert_eq!(
            SourceSnapshot::read(&pool, "meeting-fixture", Some(1), false)
                .await
                .unwrap()
                .notes_json,
            snapshot.notes_json
        );
        assert!(!finish(
            &pool,
            "meeting-fixture",
            &interrupted.id,
            Some(&json!({"markdown":"late"})),
            "ready",
            None
        )
        .await
        .unwrap());
    }
}
