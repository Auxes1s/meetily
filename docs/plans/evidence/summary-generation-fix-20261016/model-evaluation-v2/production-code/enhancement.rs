//! Explicit generation into an isolated candidate from committed native sources.
use crate::database::repositories::{enhancement as repo, setting::SettingsRepository};
use crate::state::AppState;
use crate::summary::{
    grounding::{self, SourceSnapshot},
    llm_client::{self, LLMProvider},
    templates::{self, Template},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;
use url::{Host, Url};

mod wire;
mod generation;
pub mod work;
pub use work::GenerationProgress;

const PAYLOAD_SCOPE: &str =
    "Committed original notes, transcript passages, selected template and instructions";
static JOBS: LazyLock<Mutex<HashMap<String, (String, CancellationToken)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
pub fn cancel_meeting_enhancements(meeting_id: &str) {
    let jobs = JOBS.lock().unwrap_or_else(|p| p.into_inner());
    for (id, token) in jobs.values() {
        if id == meeting_id {
            token.cancel();
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Destination {
    pub destination: String,
    pub local: bool,
    pub payload_scope: &'static str,
}
#[derive(Debug, Deserialize)]
pub struct EnhancementStart {
    pub meeting_id: String,
    pub expected_notes_revision: i64,
    pub model: String,
    pub model_name: String,
    pub template_id: Option<String>,
    pub instructions: Option<String>,
    pub remote_destination: Option<String>,
    pub summary_language: Option<String>,
    pub retry_of_version_id: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct EnhancementStarted {
    pub version_id: String,
    pub status: &'static str,
}
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EnhancementStatus {
    pub id: String,
    pub status: String,
    pub revision: i64,
    pub error: Option<String>,
    #[sqlx(skip)]
    pub generation_progress: Option<GenerationProgress>,
}
struct ProviderConfig {
    provider: LLMProvider,
    model: String,
    api_key: String,
    ollama_endpoint: Option<String>,
    custom_endpoint: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    destination: Destination,
    context: usize,
    summary_language: Option<String>,
}
pub fn classify_endpoint(endpoint: &str) -> Result<(String, bool), String> {
    let parsed = Url::parse(endpoint).map_err(|_| "Invalid inference endpoint URL")?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err("Inference endpoint must be an HTTP(S) URL without embedded credentials, query or fragment".into());
    }
    let local = match parsed.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    };
    Ok((parsed.to_string(), local))
}
async fn provider_config(
    pool: &SqlitePool,
    model_provider: &str,
    model_name: &str,
) -> Result<ProviderConfig, String> {
    let provider = LLMProvider::from_str(model_provider)?;
    let settings = SettingsRepository::get_model_config(pool)
        .await
        .map_err(|_| "Cannot read inference settings")?;
    let custom = if provider == LLMProvider::CustomOpenAI {
        Some(
            SettingsRepository::get_custom_openai_config(pool)
                .await
                .map_err(|_| "Cannot read custom inference settings")?
                .ok_or("Custom OpenAI endpoint is not configured")?,
        )
    } else {
        None
    };
    let ollama_endpoint = if provider == LLMProvider::Ollama {
        Some(
            settings
                .and_then(|s| s.ollama_endpoint)
                .unwrap_or_else(|| "http://localhost:11434".into())
                .trim_end_matches('/')
                .to_owned(),
        )
    } else {
        None
    };
    let custom_endpoint = custom
        .as_ref()
        .map(|c| c.endpoint.trim_end_matches('/').to_owned());
    let endpoint = match provider {
        LLMProvider::BuiltInAI => None,
        LLMProvider::Ollama => Some(format!(
            "{}/v1/chat/completions",
            ollama_endpoint.as_ref().unwrap()
        )),
        LLMProvider::CustomOpenAI => Some(format!(
            "{}/chat/completions",
            custom_endpoint.as_ref().unwrap()
        )),
        LLMProvider::OpenAI => Some("https://api.openai.com/v1/chat/completions".into()),
        LLMProvider::Claude => Some("https://api.anthropic.com/v1/messages".into()),
        LLMProvider::Groq => Some("https://api.groq.com/openai/v1/chat/completions".into()),
        LLMProvider::OpenRouter => Some("https://openrouter.ai/api/v1/chat/completions".into()),
    };
    let (destination, local) = if let Some(endpoint) = endpoint {
        classify_endpoint(&endpoint)?
    } else {
        ("Built-in AI (local process)".into(), true)
    };
    let api_key = if let Some(custom) = &custom {
        custom.api_key.clone().unwrap_or_default()
    } else if provider == LLMProvider::BuiltInAI {
        String::new()
    } else {
        SettingsRepository::get_api_key(pool, model_provider)
            .await
            .map_err(|_| "Cannot read provider credential")?
            .unwrap_or_default()
    };
    let context = if provider == LLMProvider::BuiltInAI {
        crate::summary::summary_engine::models::get_model_by_name(model_name)
            .ok_or("Select an installed built-in model")?
            .context_size as usize
    } else {
        4096
    };
    Ok(ProviderConfig {
        provider,
        model: model_name.into(),
        api_key,
        ollama_endpoint,
        custom_endpoint,
        max_tokens: custom
            .as_ref()
            .and_then(|c| c.max_tokens)
            .map(|n| n.max(1) as u32),
        temperature: custom.as_ref().and_then(|c| c.temperature),
        top_p: custom.as_ref().and_then(|c| c.top_p),
        destination: Destination {
            destination,
            local,
            payload_scope: PAYLOAD_SCOPE,
        },
        context,
        summary_language: None,
    })
}
#[tauri::command]
pub async fn api_get_enhancement_destination(
    state: tauri::State<'_, AppState>,
    model: String,
    model_name: String,
) -> Result<Destination, String> {
    Ok(
        provider_config(state.db_manager.pool(), &model, &model_name)
            .await?
            .destination,
    )
}
#[tauri::command]
pub async fn api_get_meeting_enhancements(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
) -> Result<repo::EnhancementCollection, String> {
    repo::list(state.db_manager.pool(), &meeting_id).await
}
/// Polling must not reload or serialize frozen meeting documents and history.
#[tauri::command]
pub async fn api_get_enhancement_status(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    version_id: String,
) -> Result<EnhancementStatus, String> {
    let mut status: EnhancementStatus = sqlx::query_as("SELECT v.id,v.status,v.revision,v.error FROM meeting_enhancement_versions v JOIN meetings m ON m.id=v.meeting_id WHERE v.id=? AND v.meeting_id=? AND m.status != 'deleting'")
        .bind(&version_id).bind(meeting_id).fetch_optional(state.db_manager.pool()).await
        .map_err(|_|"Cannot read enhancement status")?.ok_or_else(||"Enhancement version not found".to_string())?;
    status.generation_progress = work::progress(state.db_manager.pool(), &version_id).await?;
    Ok(status)
}
#[tauri::command]
pub async fn api_save_enhancement_settings(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    template_id: String,
    instructions: String,
    expected_revision: i64,
) -> Result<repo::EnhancementSettings, String> {
    templates::get_template(&template_id)?;
    repo::save_settings(
        state.db_manager.pool(),
        &meeting_id,
        &template_id,
        &instructions,
        expected_revision,
    )
    .await
}
#[tauri::command]
pub async fn api_save_enhancement_version(
    state: tauri::State<'_, AppState>,
    input: repo::VersionSave,
) -> Result<repo::EnhancementVersion, String> {
    repo::save_version(state.db_manager.pool(), input).await
}
#[tauri::command]
pub async fn api_approve_enhancement_version(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    version_id: String,
    expected_revision: i64,
    expected_approved_version_id: Option<String>,
) -> Result<repo::EnhancementVersion, String> {
    repo::approve(
        state.db_manager.pool(),
        &meeting_id,
        &version_id,
        expected_revision,
        expected_approved_version_id.as_deref(),
    )
    .await
}
#[tauri::command]
pub async fn api_discard_enhancement_version(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    version_id: String,
    expected_revision: i64,
) -> Result<repo::EnhancementVersion, String> {
    if let Some((id, token)) = JOBS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&version_id)
    {
        if id == &meeting_id {
            token.cancel();
        }
    }
    repo::discard(
        state.db_manager.pool(),
        &meeting_id,
        &version_id,
        expected_revision,
    )
    .await
}
#[tauri::command]
pub async fn api_cancel_meeting_enhancement(
    state: tauri::State<'_, AppState>,
    meeting_id: String,
    version_id: String,
) -> Result<repo::EnhancementVersion, String> {
    if let Some((id, token)) = JOBS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&version_id)
    {
        if id == &meeting_id {
            token.cancel();
        }
    }
    repo::finish(
        state.db_manager.pool(),
        &meeting_id,
        &version_id,
        None,
        "cancelled",
        Some("Generation cancelled"),
    )
    .await?;
    repo::get_version(state.db_manager.pool(), &meeting_id, &version_id).await
}
#[tauri::command]
pub async fn api_start_meeting_enhancement<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    input: EnhancementStart,
) -> Result<EnhancementStarted, String> {
    start(app, state.db_manager.pool().clone(), input).await
}

pub async fn start<R: Runtime>(
    app: AppHandle<R>,
    pool: SqlitePool,
    input: EnhancementStart,
) -> Result<EnhancementStarted, String> {
    let collection = repo::list(&pool, &input.meeting_id).await?;
    let template_id = input.template_id.unwrap_or(collection.settings.template_id);
    let instructions = input
        .instructions
        .unwrap_or(collection.settings.instructions);
    if instructions.len() > 64 * 1024 {
        return Err("Instructions exceed 64 KiB".into());
    }
    let template = templates::get_template(&template_id)?;
    let mut config = provider_config(&pool, &input.model, &input.model_name).await?;
    if !config.destination.local
        && input.remote_destination.as_deref() != Some(config.destination.destination.as_str())
    {
        return Err(format!(
            "Explicit remote choice required. Destination: {}. Payload: {}",
            config.destination.destination, PAYLOAD_SCOPE
        ));
    }
    if !config.destination.local {
        sqlx::query("UPDATE meeting_enhancement_settings SET remote_destination=?,remote_payload_scope=?,remote_selected_at=? WHERE meeting_id=? AND EXISTS(SELECT 1 FROM meetings WHERE id=? AND status != 'deleting')")
            .bind(&config.destination.destination).bind(PAYLOAD_SCOPE).bind(chrono::Utc::now().to_rfc3339()).bind(&input.meeting_id).bind(&input.meeting_id).execute(&pool).await.map_err(|_|"Cannot save remote destination choice")?;
    }
    if config.provider == LLMProvider::Ollama {
        // Reuse the existing model metadata adapter, bounded by the established
        // 4k request profile. Smaller models must not receive oversized context.
        static MODEL_METADATA: LazyLock<crate::ollama::metadata::ModelMetadataCache> =
            LazyLock::new(|| {
                crate::ollama::metadata::ModelMetadataCache::new(std::time::Duration::from_secs(
                    300,
                ))
            });
        config.context = MODEL_METADATA
            .get_or_fetch(&config.model, config.ollama_endpoint.as_deref())
            .await
            .map(|m| m.context_size.min(4096))
            .map_err(|_| {
                "Cannot verify the selected Ollama model context. Start its server and retry."
            })?;
    }
    let snapshot = SourceSnapshot::read(
        &pool,
        &input.meeting_id,
        Some(input.expected_notes_revision),
        false,
    )
    .await?;
    let saved_language =
        crate::database::repositories::meeting::MeetingsRepository::get_meeting_metadata(
            &pool,
            &input.meeting_id,
        )
        .await
        .map_err(|_| "Cannot read meeting language preference")?
        .and_then(|m| m.folder_path)
        .and_then(|folder| {
            crate::summary::metadata::read_summary_language_from_metadata(std::path::Path::new(
                &folder,
            ))
            .ok()
            .flatten()
        });
    config.summary_language = input
        .summary_language
        .filter(|language| !language.trim().is_empty())
        .or(saved_language)
        .or_else(|| {
            crate::summary::language_detection::detect_summary_language(
                &snapshot
                    .sources
                    .iter()
                    .map(|source| source.text.clone())
                    .collect::<Vec<_>>(),
            )
            .language
        });
    if config.summary_language.as_ref().is_some_and(|language| {
        crate::summary::processor::language_name_from_code(language).is_none()
    }) {
        return Err("Unsupported summary language preference".into());
    }
    if snapshot.sources.is_empty() {
        return Err("Write notes or finish transcription before enhancement".into());
    }
    let provider_json = serde_json::json!({"provider":input.model,"model":config.model,"destination":config.destination.destination,"context_size":config.context,"max_tokens":config.max_tokens,"temperature":config.temperature,"top_p":config.top_p,"summary_language":config.summary_language,"template":template,"schema_version":1,"generation_format":wire::FORMAT_VERSION});
    let v = repo::create_candidate(
        &pool,
        &input.meeting_id,
        &snapshot,
        &template_id,
        &instructions,
        &provider_json,
    )
    .await?;
    let token = CancellationToken::new();
    JOBS.lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(v.id.clone(), (input.meeting_id.clone(), token.clone()));
    let app_data_dir = app.path().app_data_dir().ok();
    let version_id = v.id.clone();
    let meeting_id = input.meeting_id;
    tauri::async_runtime::spawn(async move {
        let store = work::Store { pool: &pool, meeting: &meeting_id, version: &version_id, retry: input.retry_of_version_id.as_deref() };
        let result = generation::run(
            &config,
            &snapshot,
            &template,
            &instructions,
            app_data_dir.as_ref(),
            &token,
            None, Some(&store),
        )
        .await;
        let (document, status, error) = match result {
            Ok(document) if !token.is_cancelled() => (Some(document), "ready", None),
            _ if token.is_cancelled() => {
                (None, "cancelled", Some("Generation cancelled".to_string()))
            }
            Err(e) => (None, "failed", Some(e)),
            _ => (None, "cancelled", Some("Generation cancelled".to_string())),
        };
        if let Err(_) = repo::finish(
            &pool,
            &meeting_id,
            &version_id,
            document.as_ref(),
            status,
            error.as_deref(),
        )
        .await
        {
            tracing::warn!("Enhancement terminal commit failed; source documents remain unchanged");
        }
        JOBS.lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&version_id);
    });
    Ok(EnhancementStarted {
        version_id: v.id,
        status: "generating",
    })
}

fn language_instruction(language: Option<&str>) -> String {
    let language = language
        .and_then(crate::summary::processor::language_name_from_code)
        .unwrap_or("English");
    format!("Write every claim's text in {language}, translating the source meaning when necessary. Preserve qualifications and uncertainty. Keep JSON keys, status/kind values, section titles, source IDs, exact source excerpts, and literal owner/deadline values unchanged.")
}
async fn generate(
    config: &ProviderConfig,
    snapshot: &SourceSnapshot,
    template: &Template,
    instructions: &str,
    app_data_dir: Option<&PathBuf>,
    token: &CancellationToken,
) -> Result<Value, String> {
    generate_observed(config, snapshot, template, instructions, app_data_dir, token, None).await
}

/// Same production path, with opt-in evidence capture for synthetic evaluation.
async fn generate_observed(
    config: &ProviderConfig, snapshot: &SourceSnapshot, template: &Template,
    instructions: &str, app_data_dir: Option<&PathBuf>, token: &CancellationToken,
    mut audit: Option<&mut Vec<Value>>,
) -> Result<Value, String> {
    generation::run(config,snapshot,template,instructions,app_data_dir,token,audit,None).await
}

#[cfg(test)]
#[path = "enhancement/evaluation.rs"]
mod evaluation;
#[cfg(test)]
mod tests {
    use super::*;
    pub(super) async fn read_request(stream: &mut tokio::net::TcpStream) -> Value {
        use tokio::io::AsyncReadExt;
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let read = stream.read(&mut buffer).await.unwrap();
            assert!(read > 0);
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(header_end) = bytes
                .windows(4)
                .position(|s| s == b"\r\n\r\n")
                .map(|n| n + 4)
            {
                let headers = String::from_utf8_lossy(&bytes[..header_end]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        line.split_once(':')
                            .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                            .map(|(_, value)| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap();
                if bytes.len() >= header_end + length {
                    return serde_json::from_slice(&bytes[header_end..header_end + length])
                        .unwrap();
                }
            }
        }
    }
    pub(super) fn fixture_snapshot() -> SourceSnapshot {
        let text = "Tentative proposal, awaiting review. ".repeat(250);
        SourceSnapshot {
            schema_version: 1,
            meeting_id: "synthetic".into(),
            notes_revision: 3,
            notes_json: None,
            source_fingerprint: grounding::fingerprint(&text),
            transcript_coverage: "committed_available".into(),
            sources: vec![grounding::Source {
                source_id: "note:synthetic-block".into(),
                kind: "notes".into(),
                fingerprint: grounding::fingerprint(&text),
                text,
                audio_start_time: None,
                audio_end_time: None,
                audio_source: None,
            }],
        }
    }
    pub(super) fn fixture_config(endpoint: String) -> ProviderConfig {
        ProviderConfig {
            provider: LLMProvider::CustomOpenAI,
            model: "synthetic-model".into(),
            api_key: String::new(),
            ollama_endpoint: None,
            custom_endpoint: Some(endpoint.clone()),
            max_tokens: Some(512),
            temperature: None,
            top_p: None,
            context: 8192,
            summary_language: None,
            destination: Destination {
                destination: endpoint,
                local: true,
                payload_scope: PAYLOAD_SCOPE,
            },
        }
    }
    #[tokio::test]
    async fn enhancement_generation_covers_chunks_and_repairs_once() {
        use tokio::io::AsyncWriteExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut config = fixture_config(format!("http://{}/v1", listener.local_addr().unwrap()));
        config.summary_language = Some("ja".into());
        let captured = std::sync::Arc::new(Mutex::new(Vec::<Vec<grounding::Source>>::new()));
        let server_captured = captured.clone();
        let server = tokio::spawn(async move {
            let mut request_count = 0;
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_request(&mut stream).await;
                let system = request["messages"][0]["content"].as_str().unwrap();
                assert!(system.contains("Write every claim's text in Japanese"));
                assert!(system.contains("Cite only supplied passage IDs"));
                let prompt = request["messages"][1]["content"].as_str().unwrap();
                let passages: Vec<Value> = serde_json::from_str(prompt.rsplit_once("passages:\n").unwrap().1).unwrap();
                let sources: Vec<grounding::Source> = passages.iter().map(|p| grounding::Source {
                    source_id: p["id"].as_str().unwrap().into(), kind: p["kind"].as_str().unwrap().into(),
                    text: p["text"].as_str().unwrap().into(), fingerprint: String::new(),
                    audio_start_time: None, audio_end_time: None, audio_source: None,
                }).collect();
                let citation_id = if request_count == 0 {
                    "transcript:wrong-meeting"
                } else {
                    sources[0].source_id.as_str()
                };
                let excerpt: String = sources[0].text.chars().take(25).collect();
                let claims=serde_json::json!({"claims":[{"kind":"highlight","text":"仮の提案は確認待ちです","status":"confirmed","section":"Highlights","owner":null,"deadline":null,"citations":[citation_id]}]}).to_string();
                let body =
                    serde_json::json!({"choices":[{"message":{"content":claims}}]}).to_string();
                let response=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body);
                stream.write_all(response.as_bytes()).await.unwrap();
                server_captured.lock().unwrap().push(sources);
                request_count += 1;
            }
        });
        let template = templates::get_template("highlights").unwrap();
        let snapshot = fixture_snapshot();
        let output = generate(
            &config,
            &snapshot,
            &template,
            "",
            None,
            &CancellationToken::new(),
        )
        .await
        .unwrap();
        server.abort();
        let captured = captured.lock().unwrap();
        assert!(captured.len() > 2);
        assert_eq!(
            captured[0][0].text, captured[1][0].text,
            "repair must preserve the frozen chunk"
        );
        let text: String = captured
            .iter()
            .skip(1)
            .flatten()
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(text, snapshot.sources[0].text);
        assert_eq!(
            output["coverage"]["chunk_count"].as_u64().unwrap() + 1,
            captured.len() as u64
        );
        assert!(output["claims"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] == "notes_only"));
        assert!(output["claims"].as_array().unwrap().iter().all(|c| {
            c["text"] == "仮の提案は確認待ちです"
                && c["sources"][0]["excerpt"]
                    .as_str()
                    .is_some_and(|excerpt| snapshot.sources[0].text.contains(excerpt))
        }));
    }
    #[tokio::test]
    async fn enhancement_local_requests_do_not_follow_redirects() {
        use tokio::io::AsyncWriteExt;
        let sink = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let sink_address = sink.local_addr().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let config = fixture_config(format!("http://{}/v1", listener.local_addr().unwrap()));
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let _ = read_request(&mut stream).await;
            stream.write_all(format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{sink_address}/collect\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).await.unwrap();
        });
        let result = generate(
            &config,
            &fixture_snapshot(),
            &templates::get_template("highlights").unwrap(),
            "",
            None,
            &CancellationToken::new(),
        )
        .await;
        assert!(result.is_err());
        server.await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(150), sink.accept())
                .await
                .is_err()
        );
    }

    #[test]
    fn parsed_locality_rejects_deceptive_hosts() {
        for endpoint in [
            "http://localhost:11434",
            "http://127.0.0.1:8080/v1",
            "http://[::1]:8080/v1",
        ] {
            assert!(classify_endpoint(endpoint).unwrap().1);
        }
        for endpoint in [
            "http://localhost.example.com:11434",
            "http://192.168.1.10:11434",
            "https://example.com/localhost",
        ] {
            assert!(!classify_endpoint(endpoint).unwrap().1);
        }
        assert!(classify_endpoint("http://localhost@example.com").is_err());
        assert!(classify_endpoint("file:///tmp/model").is_err());
    }
}

/// Call once on database startup, after migrations and before serving commands.
pub async fn recover_interrupted_enhancements(pool: &SqlitePool) -> Result<(), String> {
    let mut tx=pool.begin().await.map_err(|_|"Cannot recover interrupted generation")?;
    sqlx::query("UPDATE meeting_enhancement_work SET error_code='interrupted' WHERE version_id IN (SELECT id FROM meeting_enhancement_versions WHERE status='generating')").execute(&mut *tx).await.map_err(|_|"Cannot recover interrupted checkpoints")?;
    sqlx::query("UPDATE meeting_enhancement_versions SET status='failed',error='Generation was interrupted by application restart. Original and approved documents are preserved.',revision=revision+1,updated_at=? WHERE status='generating'")
        .bind(chrono::Utc::now().to_rfc3339()).execute(&mut *tx).await.map_err(|e|e.to_string())?;
    tx.commit().await.map_err(|_|"Cannot commit generation recovery")?;
    Ok(())
}
