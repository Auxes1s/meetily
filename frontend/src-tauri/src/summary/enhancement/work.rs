use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationProgress {
    pub completed_chunks: usize,
    pub total_chunks: usize,
    pub failed_chunk: Option<usize>,
    pub error_code: Option<String>,
    pub retryable: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct Work {
    pub accepted: BTreeMap<String, Vec<grounding::Claim>>,
    pub split: BTreeSet<String>,
    pub completed: BTreeSet<usize>,
    pub total: usize,
    pub failed_chunk: Option<usize>,
    pub error_code: Option<String>,
    pub attempts: usize,
    pub prompt_tokens: Option<u64>,
    pub generated_tokens: Option<u64>,
}

pub(super) struct Store<'a> {
    pub pool: &'a SqlitePool,
    pub meeting: &'a str,
    pub version: &'a str,
    pub retry: Option<&'a str>,
}
impl Store<'_> {
    pub async fn load(&self, key: &str, total: usize) -> Result<Work, String> {
        let mut work = if let Some(previous) = self.retry {
            let raw: Option<String> = sqlx::query_scalar("SELECT w.checkpoint_json FROM meeting_enhancement_work w JOIN meeting_enhancement_versions v ON v.id=w.version_id WHERE w.version_id=? AND v.meeting_id=? AND v.status IN ('failed','cancelled') AND w.generation_key=?")
                .bind(previous).bind(self.meeting).bind(key).fetch_optional(self.pool).await.map_err(|_| "Cannot read generation checkpoints")?;
            raw.and_then(|s| serde_json::from_str::<Work>(&s).ok())
                .unwrap_or_default()
        } else {
            Work::default()
        };
        if work.total != total {
            work = Work::default();
        }
        work.total = total;
        work.error_code = None;
        work.failed_chunk = None;
        work.attempts = 0;
        work.prompt_tokens = None;
        work.generated_tokens = None;
        self.save(key, &work).await?;
        Ok(work)
    }
    pub async fn save(&self, key: &str, work: &Work) -> Result<(), String> {
        let raw =
            serde_json::to_string(work).map_err(|_| "Cannot encode generation checkpoints")?;
        let result = sqlx::query("INSERT INTO meeting_enhancement_work(version_id,generation_key,checkpoint_json,completed_chunks,total_chunks,failed_chunk,error_code,attempts,prompt_tokens,generated_tokens,updated_at) SELECT ?,?,?,?,?,?,?,?,?,?,? WHERE EXISTS(SELECT 1 FROM meeting_enhancement_versions v JOIN meetings m ON m.id=v.meeting_id WHERE v.id=? AND v.meeting_id=? AND v.status='generating' AND v.state='candidate' AND m.status NOT IN ('deleting','recording','finalizing')) ON CONFLICT(version_id) DO UPDATE SET checkpoint_json=excluded.checkpoint_json,completed_chunks=excluded.completed_chunks,total_chunks=excluded.total_chunks,failed_chunk=excluded.failed_chunk,error_code=excluded.error_code,attempts=excluded.attempts,prompt_tokens=excluded.prompt_tokens,generated_tokens=excluded.generated_tokens,updated_at=excluded.updated_at")
            .bind(self.version).bind(key).bind(raw).bind(work.completed.len() as i64).bind(work.total as i64)
            .bind(work.failed_chunk.map(|n|n as i64)).bind(&work.error_code).bind(work.attempts as i64)
            .bind(work.prompt_tokens.map(|n|n as i64)).bind(work.generated_tokens.map(|n|n as i64))
            .bind(chrono::Utc::now().to_rfc3339()).bind(self.version).bind(self.meeting)
            .execute(self.pool).await.map_err(|_| "Cannot save generation checkpoints")?;
        if result.rows_affected() != 1 {
            return Err("Generation is no longer active".into());
        }
        Ok(())
    }
}

pub async fn progress(
    pool: &SqlitePool,
    version: &str,
) -> Result<Option<GenerationProgress>, String> {
    let row: Option<(i64,i64,Option<i64>,Option<String>,String)> = sqlx::query_as("SELECT w.completed_chunks,w.total_chunks,w.failed_chunk,w.error_code,v.status FROM meeting_enhancement_work w JOIN meeting_enhancement_versions v ON v.id=w.version_id WHERE w.version_id=?")
        .bind(version).fetch_optional(pool).await.map_err(|_| "Cannot read generation progress")?;
    Ok(row.map(
        |(completed, total, failed, error, status)| GenerationProgress {
            completed_chunks: completed as usize,
            total_chunks: total as usize,
            failed_chunk: failed.map(|n| n as usize),
            error_code: error,
            retryable: matches!(status.as_str(), "failed" | "cancelled"),
        },
    ))
}
