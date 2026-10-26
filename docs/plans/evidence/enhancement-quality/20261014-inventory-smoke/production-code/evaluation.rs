//! Model-independent fixture contracts and explicitly opted-in local-model runs.
use super::*;
use crate::database::{
    manager::DatabaseManager,
    notes_commands::{self, NotesSave},
};
use serde_json::json;
use std::{fs, path::Path, time::Instant};

#[derive(Debug, Deserialize, Serialize)]
struct Passage {
    id: String,
    text: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct TranscriptPassage {
    id: String,
    text: String,
    audio_start_time: Option<f64>,
    audio_end_time: Option<f64>,
    audio_source: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
struct Fixture {
    schema_version: i32,
    id: String,
    category: String,
    meeting_id: String,
    template_id: String,
    summary_language: String,
    instructions: String,
    notes: Vec<Passage>,
    transcripts: Vec<TranscriptPassage>,
    annotations: Value,
}
fn fixtures() -> Vec<Fixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/meeting_enhancement");
    let mut paths: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
        .collect()
}
async fn seed(f: &Fixture, dir: &Path) -> DatabaseManager {
    let db = DatabaseManager::new(
        dir.join("evaluation.sqlite").to_str().unwrap(),
        dir.join("absent-legacy.sqlite").to_str().unwrap(),
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status) VALUES (?,?,'2026-10-14T00:00:00Z','2026-10-14T00:00:00Z','completed')")
        .bind(&f.meeting_id).bind(format!("Synthetic E08 {}",f.category)).execute(db.pool()).await.unwrap();
    let blocks: Vec<_> = f
        .notes
        .iter()
        .map(|n| {
            json!({"id":n.id,"type":"paragraph",
        "content":[{"type":"text","text":n.text}],"children":[]})
        })
        .collect();
    notes_commands::save(
        db.pool(),
        NotesSave {
            meeting_id: f.meeting_id.clone(),
            notes_json: json!(blocks).to_string(),
            notes_markdown: None,
            notes_plain_text: None,
            base_revision: 0,
            commit_id: uuid::Uuid::new_v4().to_string(),
            title: None,
            links: vec![],
        },
    )
    .await
    .unwrap();
    for (i, t) in f.transcripts.iter().enumerate() {
        sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,audio_start_time,audio_end_time,speaker,sequence_id) VALUES (?,?,?,'2026-10-14T00:00:00Z',?,?,?,?)")
            .bind(&t.id).bind(&f.meeting_id).bind(&t.text).bind(t.audio_start_time)
            .bind(t.audio_end_time).bind(&t.audio_source).bind(i as i64).execute(db.pool()).await.unwrap();
    }
    db
}
async fn protected(pool: &SqlitePool, meeting: &str) -> Value {
    let notes: (String, i64) =
        sqlx::query_as("SELECT notes_json,revision FROM meeting_notes WHERE meeting_id=?")
            .bind(meeting)
            .fetch_one(pool)
            .await
            .unwrap();
    let approved: (Option<String>,Option<String>,Option<i64>) = sqlx::query_as("SELECT s.approved_version_id,v.document_json,v.revision FROM meeting_enhancement_settings s LEFT JOIN meeting_enhancement_versions v ON v.id=s.approved_version_id WHERE s.meeting_id=?")
        .bind(meeting).fetch_one(pool).await.unwrap();
    json!({"notes":notes,"approved":approved})
}
async fn seed_approved(pool: &SqlitePool, f: &Fixture, snapshot: &SourceSnapshot) {
    let version = repo::create_candidate(
        pool,
        &f.meeting_id,
        snapshot,
        &f.template_id,
        "",
        &json!({"provider":"synthetic"}),
    )
    .await
    .unwrap();
    repo::finish(pool,&f.meeting_id,&version.id,Some(&json!({"markdown":"Previously approved, user-edited synthetic version","user_edited":true})),"ready",None).await.unwrap();
    let version = repo::get_version(pool, &f.meeting_id, &version.id)
        .await
        .unwrap();
    repo::approve(pool, &f.meeting_id, &version.id, version.revision, None)
        .await
        .unwrap();
}
#[tokio::test]
async fn all_fixture_sources_annotations_and_chunk_coverage() {
    let fixtures = fixtures();
    assert_eq!(fixtures.len(), 12);
    let mut categories = std::collections::HashSet::new();
    for f in fixtures {
        assert_eq!(f.schema_version, 1);
        assert!(categories.insert(f.category.clone()));
        let dir = tempfile::tempdir().unwrap();
        let db = seed(&f, dir.path()).await;
        let snapshot = SourceSnapshot::read(db.pool(), &f.meeting_id, Some(1), false)
            .await
            .unwrap();
        assert_eq!(snapshot.sources.len(), f.notes.len() + f.transcripts.len());
        assert!(
            SourceSnapshot::read(db.pool(), &f.meeting_id, Some(0), false)
                .await
                .is_err()
        );
        for label in f.annotations["relevant_decisions_actions"]
            .as_array()
            .unwrap()
            .iter()
            .chain(f.annotations["user_priorities"].as_array().unwrap())
        {
            for citation in label["supporting_passages"].as_array().unwrap() {
                let s = snapshot
                    .sources
                    .iter()
                    .find(|s| s.source_id == citation["source_id"].as_str().unwrap())
                    .unwrap();
                assert!(s.text.contains(citation["excerpt"].as_str().unwrap()));
            }
        }
        for budget in [512, 2048, 24000] {
            let chunks = grounding::source_chunks(&snapshot.sources, budget).unwrap();
            for source in &snapshot.sources {
                let joined: String = chunks
                    .iter()
                    .flatten()
                    .filter(|s| s.source_id == source.source_id)
                    .map(|s| s.text.as_str())
                    .collect();
                assert_eq!(
                    joined, source.text,
                    "missing source text: {}",
                    source.source_id
                );
            }
            assert!(chunks
                .iter()
                .all(|c| serde_json::to_string(c).unwrap().len() <= budget));
        }
        let before = snapshot.source_fingerprint.clone();
        sqlx::query(
            "UPDATE transcripts SET transcript=transcript || ' changed' WHERE meeting_id=?",
        )
        .bind(&f.meeting_id)
        .execute(db.pool())
        .await
        .unwrap();
        assert_ne!(
            before,
            SourceSnapshot::read(db.pool(), &f.meeting_id, None, false)
                .await
                .unwrap()
                .source_fingerprint
        );
        db.cleanup().await.unwrap();
    }
}
#[tokio::test]
async fn repair_failure_preserves_originals_approval_and_rejected_evidence() {
    use tokio::io::AsyncWriteExt;
    let f = fixtures().remove(0);
    let dir = tempfile::tempdir().unwrap();
    let db = seed(&f, dir.path()).await;
    let snapshot = SourceSnapshot::read(db.pool(), &f.meeting_id, Some(1), false)
        .await
        .unwrap();
    seed_approved(db.pool(), &f, &snapshot).await;
    let before = protected(db.pool(), &f.meeting_id).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let config = tests::fixture_config(format!("http://{}/v1", listener.local_addr().unwrap()));
    let server = tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = tests::read_request(&mut stream).await;
            let body=json!({"choices":[{"message":{"content":"{\"claims\":[{\"kind\":\"decision\",\"text\":\"Invented\",\"status\":\"review_required\",\"section\":\"Summary\",\"owner\":null,\"deadline\":null,\"sources\":[{\"source_id\":\"transcript:another-meeting\",\"excerpt\":\"Invented\"}]}]}"}}]}).to_string();
            stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
        }
    });
    let v = repo::create_candidate(
        db.pool(),
        &f.meeting_id,
        &snapshot,
        &f.template_id,
        &f.instructions,
        &json!({}),
    )
    .await
    .unwrap();
    let mut audit = Vec::new();
    let error = generate_observed(
        &config,
        &snapshot,
        &templates::get_template(&f.template_id).unwrap(),
        &f.instructions,
        None,
        &CancellationToken::new(),
        Some(&mut audit),
    )
    .await
    .unwrap_err();
    server.await.unwrap();
    assert!(error.contains("after one repair"));
    assert_eq!(audit.len(), 2);
    assert_eq!(audit[0]["sources"], audit[1]["sources"]);
    assert!(audit.iter().all(|a| a["validated_claims"].is_null()
        && !a["provider_responses"].as_array().unwrap().is_empty()));
    repo::finish(
        db.pool(),
        &f.meeting_id,
        &v.id,
        None,
        "failed",
        Some(&error),
    )
    .await
    .unwrap();
    assert_eq!(before, protected(db.pool(), &f.meeting_id).await);
    assert_eq!(
        repo::get_version(db.pool(), &f.meeting_id, &v.id)
            .await
            .unwrap()
            .status,
        "failed"
    );
    let token = CancellationToken::new();
    token.cancel();
    assert!(generate(
        &config,
        &snapshot,
        &templates::get_template(&f.template_id).unwrap(),
        &f.instructions,
        None,
        &token
    )
    .await
    .is_err());
    assert_eq!(before, protected(db.pool(), &f.meeting_id).await);
    db.cleanup().await.unwrap();
}
#[tokio::test]
async fn four_k_context_keeps_template_rules_and_processes_sources() {
    use tokio::io::AsyncWriteExt;
    let f = fixtures().remove(0);
    let dir = tempfile::tempdir().unwrap();
    let db = seed(&f, dir.path()).await;
    let snapshot = SourceSnapshot::read(db.pool(), &f.meeting_id, Some(1), false)
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = tests::fixture_config(format!("http://{}/v1", listener.local_addr().unwrap()));
    config.context = 4096;
    config.max_tokens = Some(1024);
    config.summary_language = Some("en".into());
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = tests::read_request(&mut stream).await;
            let body = json!({"choices":[{"message":{"content":"{\"claims\":[]}"}}]}).to_string();
            stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
        }
    });
    for id in [
        "standard_meeting",
        "highlights",
        "technical_consultation",
        "project_board",
    ] {
        let template = templates::get_template(id).unwrap();
        let mut audit = Vec::new();
        let result = generate_observed(
            &config,
            &snapshot,
            &template,
            &f.instructions,
            None,
            &CancellationToken::new(),
            Some(&mut audit),
        )
        .await
        .unwrap();
        assert_eq!(result["coverage"]["source_count"], snapshot.sources.len());
        for source in &snapshot.sources {
            let joined: String = audit
                .iter()
                .flat_map(|a| a["sources"].as_array().unwrap())
                .filter(|s| s["source_id"] == source.source_id)
                .map(|s| s["text"].as_str().unwrap())
                .collect();
            assert_eq!(joined, source.text);
        }
        for attempt in audit {
            let header: Value =
                serde_json::from_str(attempt["prompt"].as_str().unwrap().lines().next().unwrap())
                    .unwrap();
            assert!(header.get("template").is_none());
            for (section, rule) in template
                .sections
                .iter()
                .zip(header["template_sections"].as_array().unwrap())
            {
                assert_eq!(rule["title"], section.title);
                assert_eq!(rule["instruction"], section.instruction);
            }
        }
    }
    server.abort();
    db.cleanup().await.unwrap();
}
/// Run via scripts/evaluate-meeting-enhancement.py; no models in ordinary tests.
#[tokio::test]
#[ignore = "Opt-in installed local-model evaluation; requires MEETILY_E08_MANIFEST"]
async fn real_local_models() {
    let manifest_path = std::env::var("MEETILY_E08_MANIFEST").expect("Use the evaluation script");
    let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let output = PathBuf::from(manifest["output"].as_str().unwrap());
    let app_data_dir = PathBuf::from(manifest["isolated_model_root"].as_str().unwrap());
    for model in manifest["models"].as_array().unwrap() {
        let provider = model["provider"].as_str().unwrap();
        let name = model["name"].as_str().unwrap();
        for f in fixtures()
            .into_iter()
            .filter(|f| manifest["fixture"].is_null() || manifest["fixture"] == f.id)
        {
            for repeat in 1..=manifest["repeats"].as_u64().unwrap() {
                let run_id = format!(
                    "{}-{}-{}-r{repeat}",
                    provider,
                    name.replace([':', '/'], "_"),
                    f.id
                );
                let dir = tempfile::tempdir().unwrap();
                let db = seed(&f, dir.path()).await;
                SettingsRepository::save_model_config(
                    db.pool(),
                    provider,
                    name,
                    "unused",
                    manifest["ollama_endpoint"].as_str(),
                )
                .await
                .unwrap();
                let snapshot = SourceSnapshot::read(db.pool(), &f.meeting_id, Some(1), false)
                    .await
                    .unwrap();
                seed_approved(db.pool(), &f, &snapshot).await;
                let before = protected(db.pool(), &f.meeting_id).await;
                let mut config = provider_config(db.pool(), provider, name).await.unwrap();
                assert!(config.destination.local);
                let mut setup_error = None;
                if config.provider == LLMProvider::Ollama {
                    match crate::ollama::metadata::ModelMetadataCache::new(
                        std::time::Duration::from_secs(300),
                    )
                    .get_or_fetch(name, config.ollama_endpoint.as_deref())
                    .await
                    {
                        Ok(m) => config.context = m.context_size.min(4096),
                        Err(e) => setup_error = Some(e),
                    }
                }
                config.summary_language = Some(f.summary_language.clone());
                let template = templates::get_template(&f.template_id).unwrap();
                let provider_config = json!({"provider":provider,"model":name,"context_size":config.context,"max_tokens":config.max_tokens,
                    "temperature":config.temperature,"top_p":config.top_p,"summary_language":config.summary_language,"template":template,
                    "destination":config.destination.destination,"model_identity":model,
                    "builtin_definition":if config.provider==LLMProvider::BuiltInAI {crate::summary::summary_engine::models::get_model_by_name(name)}else{None},
                    "effective_output_tokens":if config.provider==LLMProvider::BuiltInAI {crate::summary::summary_engine::models::DEFAULT_MAX_TOKENS as u32}else{config.max_tokens.unwrap_or(1024)}});
                let candidate = repo::create_candidate(
                    db.pool(),
                    &f.meeting_id,
                    &snapshot,
                    &f.template_id,
                    &f.instructions,
                    &provider_config,
                )
                .await
                .unwrap();
                let mut audit = Vec::new();
                let started = Instant::now();
                let started_at = chrono::Utc::now().to_rfc3339();
                let result = if let Some(e) = setup_error {
                    Err(e)
                } else {
                    generate_observed(
                        &config,
                        &snapshot,
                        &template,
                        &f.instructions,
                        Some(&app_data_dir),
                        &CancellationToken::new(),
                        Some(&mut audit),
                    )
                    .await
                };
                let elapsed_ms = started.elapsed().as_millis();
                let status = if result.is_ok() { "ready" } else { "failed" };
                repo::finish(
                    db.pool(),
                    &f.meeting_id,
                    &candidate.id,
                    result.as_ref().ok(),
                    status,
                    result.as_ref().err().map(String::as_str),
                )
                .await
                .unwrap();
                let after = protected(db.pool(), &f.meeting_id).await;
                let persisted = repo::get_version(db.pool(), &f.meeting_id, &candidate.id)
                    .await
                    .unwrap();
                let record = json!({"schema_version":1,"run_id":run_id,"phase":manifest["phase"],"started_at":started_at,
                    "fixture":f,"source_snapshot":snapshot,"provider_config":provider_config,"environment":manifest["environment"],
                    "attempts":audit,"validated_candidate":result.as_ref().ok(),"persisted_version":persisted,
                    "outcome":if result.is_ok(){"structurally_accepted_candidate"}else{"rejected_generation"},
                    "error":result.as_ref().err(),"elapsed_ms":elapsed_ms,"protected_before":before,"protected_after":after,
                    "originals_and_approval_preserved":before==after,"semantic_assessment":"pending Codex review","human_acceptance":"pending"});
                fs::write(
                    output.join(format!("{run_id}.json")),
                    serde_json::to_vec_pretty(&record).unwrap(),
                )
                .unwrap();
                println!(
                    "E08 {run_id}: {status}, {elapsed_ms} ms, {} calls",
                    audit.len()
                );
                assert_eq!(before, after, "generation modified originals or approval");
                db.cleanup().await.unwrap();
            }
        }
    }
    crate::summary::summary_engine::shutdown_sidecar_gracefully()
        .await
        .unwrap();
}
