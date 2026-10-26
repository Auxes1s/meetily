//! Source-aware enhancement inputs and bounded structural validation.
//! Citation validity is never advertised as semantic verification.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashSet;

pub fn fingerprint(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}:{}", text.len())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub source_id: String,
    pub kind: String,
    pub text: String,
    pub fingerprint: String,
    pub audio_start_time: Option<f64>,
    pub audio_end_time: Option<f64>,
    pub audio_source: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSnapshot {
    pub schema_version: i32,
    pub meeting_id: String,
    pub notes_revision: i64,
    pub notes_json: Option<String>,
    pub sources: Vec<Source>,
    pub source_fingerprint: String,
    pub transcript_coverage: String,
}
fn text_content(v: &Value, out: &mut String) {
    match v {
        Value::Array(items) => {
            for item in items {
                text_content(item, out)
            }
        }
        Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(Value::as_str) {
                out.push_str(text);
            }
            if let Some(content) = object.get("content") {
                text_content(content, out);
            }
            for key in ["rows", "cells"] {
                if let Some(items) = object.get(key).and_then(Value::as_array) {
                    for item in items {
                        text_content(item, out);
                        out.push('\n');
                    }
                }
            }
        }
        Value::String(text) => out.push_str(text),
        _ => {}
    }
}
fn note_sources(blocks: &[Value], sources: &mut Vec<Source>) -> Result<(), String> {
    for block in blocks {
        let id = block
            .get("id")
            .and_then(Value::as_str)
            .ok_or("Saved note block lacks an ID")?;
        let mut text = String::new();
        if let Some(content) = block.get("content") {
            text_content(content, &mut text);
        }
        if !text.trim().is_empty() {
            sources.push(Source {
                source_id: format!("note:{id}"),
                kind: "notes".into(),
                fingerprint: fingerprint(&text),
                text,
                audio_start_time: None,
                audio_end_time: None,
                audio_source: None,
            });
        }
        if let Some(children) = block.get("children").and_then(Value::as_array) {
            note_sources(children, sources)?;
        }
    }
    Ok(())
}
impl SourceSnapshot {
    /// A read transaction freezes the original note revision and committed segments.
    pub async fn read(
        pool: &SqlitePool,
        meeting_id: &str,
        expected: Option<i64>,
        allow_active: bool,
    ) -> Result<Self, String> {
        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
        let status: Option<String> = sqlx::query_scalar("SELECT status FROM meetings WHERE id=?")
            .bind(meeting_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        match status.as_deref() {
            None => return Err("Meeting not found".into()),
            Some("deleting") => return Err("Meeting deletion is pending".into()),
            Some("recording" | "finalizing") if !allow_active => {
                return Err(
                    "Finish recording and transcript finalization before enhancement".into(),
                )
            }
            _ => {}
        }
        let notes: Option<(i64, Option<String>)> =
            sqlx::query_as("SELECT revision,notes_json FROM meeting_notes WHERE meeting_id=?")
                .bind(meeting_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        let (revision, notes_json) = notes.unwrap_or((0, None));
        if expected.is_some_and(|expected| expected != revision) {
            return Err("Original notes changed; flush and reload the committed revision".into());
        }
        let mut sources = Vec::new();
        if let Some(raw) = &notes_json {
            let blocks: Value =
                serde_json::from_str(raw).map_err(|_| "Saved notes are malformed")?;
            note_sources(
                blocks
                    .as_array()
                    .ok_or("Saved note document is not a block array")?,
                &mut sources,
            )?;
        }
        let transcripts:Vec<(String,String,Option<f64>,Option<f64>,Option<String>)>=sqlx::query_as("SELECT id,transcript,audio_start_time,audio_end_time,speaker FROM transcripts WHERE meeting_id=? ORDER BY COALESCE(audio_start_time,1e99),COALESCE(sequence_id,9223372036854775807),timestamp,id")
            .bind(meeting_id).fetch_all(&mut *tx).await.map_err(|e|e.to_string())?;
        for (id, text, start, end, source) in transcripts {
            if !text.trim().is_empty() {
                sources.push(Source {
                    source_id: format!("transcript:{id}"),
                    kind: "transcript".into(),
                    fingerprint: fingerprint(&text),
                    text,
                    audio_start_time: start,
                    audio_end_time: end,
                    audio_source: source,
                });
            }
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        let transcript_coverage = match status.as_deref() {
            Some("interrupted") => "incomplete_interrupted",
            Some("recording" | "finalizing") => "incomplete_active",
            _ => "committed_available",
        }
        .to_string();
        let source_fingerprint=fingerprint(&serde_json::json!({"notes_revision":revision,"notes_json":notes_json,"sources":sources,"coverage":transcript_coverage}).to_string());
        Ok(Self {
            schema_version: 1,
            meeting_id: meeting_id.into(),
            notes_revision: revision,
            notes_json,
            sources,
            source_fingerprint,
            transcript_coverage,
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source_id: String,
    pub excerpt: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    #[serde(default)]
    pub id: String,
    pub kind: String,
    pub text: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub section: String,
    #[serde(default)]
    pub sources: Vec<Citation>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct GeneratedClaims {
    pub claims: Vec<Claim>,
}
/// Budget uses UTF-8 bytes conservatively, including serialized metadata. All text
/// survives splitting, including Unicode and cross-chunk source identities.
pub fn source_chunks(sources: &[Source], budget: usize) -> Result<Vec<Vec<Source>>, String> {
    if budget < 256 {
        return Err("Template and instructions leave insufficient source context".into());
    }
    let mut pieces = Vec::new();
    for source in sources {
        let overhead = serde_json::to_string(source)
            .map_err(|_| "Cannot serialize source")?
            .len()
            - serde_json::to_string(&source.text).unwrap().len()
            + 2;
        if overhead + 16 >= budget {
            return Err("Source metadata exceeds context capacity".into());
        }
        let text_budget = budget - overhead - 16;
        let mut text = String::new();
        let mut encoded_size = 2usize;
        for c in source.text.chars() {
            let encoded = serde_json::to_string(&c.to_string()).unwrap().len() - 2;
            if encoded_size + encoded > text_budget && !text.is_empty() {
                let mut piece = source.clone();
                piece.text = std::mem::take(&mut text);
                pieces.push(piece);
                encoded_size = 2;
            }
            text.push(c);
            encoded_size += encoded;
        }
        if !text.is_empty() {
            let mut piece = source.clone();
            piece.text = text;
            pieces.push(piece);
        }
    }
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut size = 2usize;
    for piece in pieces {
        let piece_size = serde_json::to_string(&piece).unwrap().len() + 1;
        if size + piece_size > budget && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            size = 2;
        }
        size += piece_size;
        current.push(piece);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    Ok(chunks)
}
pub fn parse_claims(raw: &str) -> Result<GeneratedClaims, String> {
    let raw = raw.trim();
    let raw = if raw.starts_with("```json") {
        raw.strip_prefix("```json")
            .unwrap()
            .trim()
            .strip_suffix("```")
            .unwrap_or(raw)
            .trim()
    } else if raw.starts_with("```") {
        raw.strip_prefix("```")
            .unwrap()
            .trim()
            .strip_suffix("```")
            .unwrap_or(raw)
            .trim()
    } else {
        raw
    };
    serde_json::from_str(raw).map_err(|_| "Model output is not valid structured claims JSON".into())
}
pub fn validate_claims(
    mut generated: GeneratedClaims,
    allowed: &[Source],
) -> Result<Vec<Claim>, String> {
    if generated.claims.len() > 200 {
        return Err("Model returned too many claims for one chunk".into());
    }
    for claim in &mut generated.claims {
        if claim.text.trim().is_empty()
            || claim.text.len() > 8192
            || crate::summary::processor::contains_reasoning_marker(&claim.text)
            || !matches!(
                claim.kind.as_str(),
                "summary" | "highlight" | "decision" | "action" | "question" | "conflict"
            )
        {
            return Err("Invalid claim content or kind".into());
        }
        if claim.sources.is_empty() {
            if !matches!(
                claim.status.as_str(),
                "uncertain" | "missing_source" | "conflict"
            ) {
                return Err("Consequential claim lacks a supporting source".into());
            }
            claim.owner = None;
            claim.deadline = None;
            claim.status = "missing_source".into();
        } else {
            let mut all_notes = true;
            for citation in &claim.sources {
                let source = allowed
                    .iter()
                    .find(|source| {
                        source.source_id == citation.source_id
                            && !citation.excerpt.trim().is_empty()
                            && source.text.contains(&citation.excerpt)
                    })
                    .ok_or("Unknown source or excerpt outside frozen input")?;
                all_notes &= source.kind == "notes";
            }
            for attribute in [&claim.owner, &claim.deadline].into_iter().flatten() {
                if attribute.trim().is_empty()
                    || !claim.sources.iter().any(|c| c.excerpt.contains(attribute))
                {
                    return Err("Owner or deadline is absent from cited excerpts".into());
                }
            }
            claim.status = if claim.kind == "conflict" || claim.status == "conflict" {
                "conflict"
            } else if all_notes {
                "notes_only"
            } else {
                "review_required"
            }
            .into();
        }
        claim.id = uuid::Uuid::new_v4().to_string();
    }
    Ok(generated.claims)
}
pub fn document(
    claims: Vec<Claim>,
    snapshot: &SourceSnapshot,
    chunk_count: usize,
    sections: &[String],
) -> Value {
    let mut markdown = String::from(
        "Source citations have been checked against saved input. Claims require your review.\n\n",
    );
    if snapshot.transcript_coverage != "committed_available" {
        markdown
            .push_str("Transcript coverage is incomplete; available committed sources only.\n\n");
    }
    let mut rendered = HashSet::new();
    for section in sections
        .iter()
        .map(String::as_str)
        .chain(std::iter::once("Other source findings"))
    {
        let items: Vec<_> = claims
            .iter()
            .filter(|c| {
                !rendered.contains(&c.id)
                    && (c.section == section || section == "Other source findings")
            })
            .collect();
        if items.is_empty() {
            continue;
        }
        markdown.push_str(&format!("## {section}\n\n"));
        for c in items {
            rendered.insert(c.id.clone());
            markdown.push_str(&format!(
                "- **[{}]** {}",
                c.status.replace('_', " "),
                c.text.replace('\n', " ")
            ));
            if let Some(owner) = &c.owner {
                markdown.push_str(&format!(" — owner stated: {owner}"));
            }
            if let Some(deadline) = &c.deadline {
                markdown.push_str(&format!("; deadline stated: {deadline}"));
            }
            for citation in &c.sources {
                markdown.push_str(&format!(" [{}]", citation.source_id));
            }
            markdown.push('\n');
        }
        markdown.push('\n');
    }
    if claims.is_empty() {
        markdown.push_str(
            "No supported claims were extracted. Review the original sources for omissions.\n",
        );
    }
    serde_json::json!({"schema_version":1,"markdown":markdown,"claims":claims,"coverage":{"source_count":snapshot.sources.len(),"chunk_count":chunk_count,"complete":snapshot.transcript_coverage=="committed_available","transcript_coverage":snapshot.transcript_coverage},"user_edited":false})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Source {
        Source {
            source_id: "transcript:a".into(),
            kind: "transcript".into(),
            text: "Proposed: Ana might submit by Friday. Not approved.".into(),
            fingerprint: "f".into(),
            audio_start_time: None,
            audio_end_time: None,
            audio_source: None,
        }
    }
    #[test]
    fn references_do_not_imply_confirmation() {
        let raw = r#"{"claims":[{"kind":"decision","text":"Ana submits Friday","status":"confirmed","sources":[{"source_id":"transcript:a","excerpt":"Ana might submit by Friday"}],"owner":"Ana","deadline":"Friday"}]}"#;
        let c = validate_claims(parse_claims(raw).unwrap(), &[source()]).unwrap();
        assert_eq!(c[0].status, "review_required");
    }
    #[test]
    fn wrong_source_and_fabricated_owner_rejected() {
        let raw = r#"{"claims":[{"kind":"action","text":"Submit","sources":[{"source_id":"transcript:wrong-meeting","excerpt":"Ana"}]}]}"#;
        assert!(validate_claims(parse_claims(raw).unwrap(), &[source()]).is_err());
        let raw = r#"{"claims":[{"kind":"action","text":"Submit","sources":[{"source_id":"transcript:a","excerpt":"Ana"}],"owner":"Ben"}]}"#;
        assert!(validate_claims(parse_claims(raw).unwrap(), &[source()]).is_err());
    }
    #[test]
    fn unicode_chunk_coverage_is_lossless() {
        let mut s = source();
        s.text = "Mga tala 日本語 \"quoted\" ".repeat(200);
        let chunks = source_chunks(&[s.clone()], 512).unwrap();
        assert!(chunks.len() > 1);
        let joined: String = chunks.iter().flatten().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, s.text);
        for chunk in chunks {
            assert!(serde_json::to_string(&chunk).unwrap().len() <= 512);
        }
    }
    #[test]
    fn unreferenced_claim_must_be_uncertain() {
        let raw = r#"{"claims":[{"kind":"decision","text":"Approved"}]}"#;
        assert!(validate_claims(parse_claims(raw).unwrap(), &[source()]).is_err());
    }
    #[test]
    fn enhancement_schema_rejects_missing_required_fields() {
        let valid = serde_json::json!({"claims":[{"kind":"action","text":"Tentative submission",
            "status":"uncertain","section":"Actions","sources":[{"source_id":"transcript:a",
            "excerpt":"Ana might submit by Friday"}],"owner":null,"deadline":null}]});
        for field in ["kind","text","status","section","sources","owner","deadline"] {
            let mut missing=valid.clone(); missing["claims"][0].as_object_mut().unwrap().remove(field);
            assert!(parse_claims(&missing.to_string()).is_err(),"missing {field} was accepted");
        }
    }
    #[test]
    fn explicit_uncertainty_survives_valid_citations() {
        let raw=r#"{"claims":[{"kind":"action","text":"Ana might submit by Friday; not approved","status":"uncertain","section":"Actions","sources":[{"source_id":"transcript:a","excerpt":"Ana might submit by Friday"}],"owner":null,"deadline":null}]}"#;
        let claims=validate_claims(parse_claims(raw).unwrap(),&[source()]).unwrap();
        assert_eq!(claims[0].status,"uncertain");
    }
}
