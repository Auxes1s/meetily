//! Debug-only orchestration for an explicitly isolated native workflow profile.
use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub(crate) fn enabled(app: &AppHandle) -> bool {
    cfg!(debug_assertions)
        && app.config().identifier == "com.meetily.workflow.test"
        && std::env::var("MEETILY_WORKFLOW_SMOKE").as_deref() == Ok("1")
}
pub(crate) fn run_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if !enabled(app) {
        return Err("Workflow automation requires its isolated debug profile".into());
    }
    let root = PathBuf::from(
        std::env::var("MEETILY_WORKFLOW_RUN_DIR").map_err(|_| "Missing workflow run directory")?,
    );
    let root = root
        .canonicalize()
        .map_err(|_| "Workflow run directory does not exist")?;
    if !root
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.starts_with("meetily-q01-"))
    {
        return Err("Workflow run directory must be a dedicated meetily-q01 directory".into());
    }
    Ok(root)
}

#[tauri::command]
pub async fn workflow_smoke_benchmark(app: AppHandle) -> Result<Value, String> {
    use crate::audio::{transcription::streaming::LiveBuffer, vad::ContinuousVadProcessor};
    use std::sync::Arc;
    let root = run_dir(&app)?;
    let bytes = std::fs::read(root.join("inputs/bench.f32")).map_err(|e| e.to_string())?;
    if bytes.len() % 4 != 0 {
        return Err("Benchmark PCM is truncated".into());
    }
    let decoded: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    if decoded.is_empty() || decoded.iter().any(|sample| !sample.is_finite()) {
        return Err("Benchmark PCM contains no valid finite audio".into());
    }
    // Compressed audio can overshoot full scale. Both routes receive the same
    // bounded PCM, as required by Silero's capture-input contract.
    let clipped = decoded.iter().filter(|sample| sample.abs() > 1.0).count();
    let audio: Vec<f32> = decoded
        .into_iter()
        .map(|sample| sample.clamp(-1.0, 1.0))
        .collect();
    let model_dir =
        std::env::var("MEETILY_SMOKE_MODEL_DIR").map_err(|_| "Missing existing model directory")?;
    let engine = Arc::new(
        crate::whisper_engine::WhisperEngine::new_with_models_dir(Some(model_dir.into()))
            .map_err(|e| e.to_string())?,
    );
    engine.discover_models().await.map_err(|e| e.to_string())?;
    engine
        .load_model("large-v3")
        .await
        .map_err(|e| e.to_string())?;
    // Equal warm-model conditions; this is a replay/virtual capture-clock
    // benchmark, explicitly distinct from a paced native device run.
    engine
        .transcribe_audio_with_confidence(audio[..audio.len().min(80_000)].to_vec(), None)
        .await
        .map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for overlapping in [false, true] {
        let mut vad = if overlapping {
            ContinuousVadProcessor::new_streaming(16_000, 500)
        } else {
            ContinuousVadProcessor::new_live(16_000, 500)
        }
        .map_err(|e| e.to_string())?;
        let mut events = Vec::new();
        let mut supplied = 0;
        for frame in audio.chunks(9_600) {
            supplied += frame.len();
            events.extend(
                vad.process_audio(frame)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(|segment| (segment, supplied as f64 / 16_000.0)),
            );
        }
        events.extend(
            vad.flush()
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|segment| (segment, audio.len() as f64 / 16_000.0)),
        );
        let mut decoder = engine.live_decoder(None).await.map_err(|e| e.to_string())?;
        let mut buffer = LiveBuffer::default();
        let mut inference = Vec::new();
        let mut deliveries = Vec::new();
        let mut ready = 0_f64;
        let mut final_text = String::new();
        let mut timed = 0;
        let mut token_count = 0;
        for (segment, available) in events {
            if overlapping {
                let mut position = 0;
                loop {
                    let room =
                        crate::audio::transcription::streaming::MAX_WINDOW_SAMPLES - buffer.len();
                    let take = room.min(segment.samples.len() - position);
                    if take > 0 {
                        buffer
                            .append(
                                &segment.samples[position..position + take],
                                segment.start_timestamp_ms / 1000.0 + position as f64 / 16_000.0,
                            )
                            .map_err(|e| e.to_string())?;
                        position += take;
                    }
                    let endpoint = position == segment.samples.len() && segment.is_final;
                    let full = buffer.must_finalize();
                    if buffer.len() > 0 && (ready <= available || endpoint || full) {
                        let started = std::time::Instant::now();
                        let tokens = decoder
                            .decode(&buffer.samples(), buffer.start())
                            .map_err(|e| e.to_string())?;
                        timed += tokens
                            .iter()
                            .filter(|t| t.start.is_some() && t.end.is_some())
                            .count();
                        token_count += tokens.len();
                        let elapsed = started.elapsed().as_secs_f64();
                        inference.push(elapsed);
                        ready = ready.max(available) + elapsed;
                        let passages = buffer.accept_boundary(tokens, endpoint || full, endpoint);
                        if passages.iter().any(|p| !p.text.is_empty()) {
                            deliveries.push(ready);
                        }
                        for passage in passages.into_iter().filter(|p| !p.is_partial) {
                            final_text.push_str(&passage.text);
                            final_text.push('\n');
                        }
                    }
                    if position == segment.samples.len() {
                        break;
                    }
                }
            } else if !segment.samples.is_empty() {
                let started = std::time::Instant::now();
                let (text, _, _) = engine
                    .transcribe_audio_with_confidence(segment.samples, None)
                    .await
                    .map_err(|e| e.to_string())?;
                let elapsed = started.elapsed().as_secs_f64();
                inference.push(elapsed);
                ready = ready.max(available) + elapsed;
                if !text.trim().is_empty() {
                    deliveries.push(ready);
                    final_text.push_str(&text);
                    final_text.push('\n');
                }
            }
        }
        let label = if overlapping { "overlapping" } else { "legacy" };
        let mut intervals: Vec<_> = deliveries.windows(2).map(|w| w[1] - w[0]).collect();
        intervals.sort_by(f64::total_cmp);
        let p95 = intervals
            .get(intervals.len().saturating_sub(1) * 95 / 100)
            .copied();
        let duration = audio.len() as f64 / 16_000.0;
        std::fs::write(root.join(format!("{label}-transcript.txt")), &final_text)
            .map_err(|e| e.to_string())?;
        results.push(json!({"routing":label,"audio_seconds":duration,"decode_count":inference.len(),
            "inference_seconds":inference.iter().sum::<f64>(),"inference_rtf":inference.iter().sum::<f64>()/duration,
            "first_update_seconds":deliveries.first(),"p95_update_interval_seconds":p95,
            "simulated_final_backlog_seconds":(ready-duration).max(0.0),"timed_tokens":timed,"decoded_tokens":token_count,
            "final_text_hash":crate::summary::grounding::fingerprint(&final_text)}));
    }
    let report = json!({"ok":true,"phase":"finished","benchmark_only":true,"model":"large-v3","results":results,
        "virtual_capture_clock":true,"clipped_input_samples":clipped,"physical_capture":false,"gpu_utilization_measured":false,
        "transcript_fidelity_acceptance":false,"default_enabled":false});
    crate::database::export::atomic_write(
        &root.join("benchmark.json"),
        &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.message)?;
    Ok(report)
}
#[tauri::command]
pub async fn workflow_smoke_control(
    app: AppHandle,
    action: String,
    payload: Option<Value>,
) -> Result<Value, String> {
    let root = run_dir(&app)?;
    let pool = app
        .state::<crate::state::AppState>()
        .db_manager
        .pool()
        .clone();
    match action.as_str() {
        "info" => {
            let info: Value = serde_json::from_slice(
                &std::fs::read(root.join("run.json")).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let endpoint = info["fixture_endpoint"]
                .as_str()
                .ok_or("Missing fixture endpoint")?;
            let (_, local) = crate::summary::enhancement::classify_endpoint(endpoint)?;
            if !local {
                return Err("Workflow fixture provider must use loopback".into());
            }
            let config:crate::summary::CustomOpenAIConfig=serde_json::from_value(json!({
                "endpoint":endpoint,"apiKey":"","model":"workflow-fixture","maxTokens":512,"temperature":0,"topP":1
            })).map_err(|e|e.to_string())?;
            crate::database::repositories::setting::SettingsRepository::save_custom_openai_config(
                &pool, &config,
            )
            .await
            .map_err(|e| e.to_string())?;
            Ok(info)
        }
        "checkpoint" | "report" => {
            let value = payload.ok_or("Missing report")?;
            let filename = if action == "checkpoint" {
                "checkpoint.json"
            } else {
                "report.json"
            };
            crate::database::export::atomic_write(
                &root.join(filename),
                &serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.message)?;
            Ok(value)
        }
        "load_checkpoint" => {
            let path = root.join("checkpoint.json");
            if !path.exists() {
                return Ok(Value::Null);
            }
            serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        "inspect" => {
            let id = payload
                .as_ref()
                .and_then(|v| v["meeting_id"].as_str())
                .ok_or("Missing meeting ID")?;
            serde_json::to_value(
                crate::database::export::read_record(&pool, id)
                    .await
                    .map_err(|e| e.message)?,
            )
            .map_err(|e| e.to_string())
        }
        "artifacts" => {
            let input = payload.ok_or("Missing fixture identity")?;
            let id = input["meeting_id"].as_str().ok_or("Missing meeting ID")?;
            let session = input["recording_session_id"]
                .as_str()
                .ok_or("Missing session")?;
            if !id.starts_with("meeting-") || uuid::Uuid::parse_str(session).is_err() {
                return Err("Invalid fixture identity".into());
            }
            let folder = root.join("recordings").join(session);
            std::fs::create_dir_all(folder.join(".checkpoints")).map_err(|e| e.to_string())?;
            std::fs::write(
                folder.join("metadata.json"),
                json!({"meeting_id":id,"recording_session_id":session,
                "audio_source":"mixed","status":"completed","version":"1.0"})
                .to_string(),
            )
            .map_err(|e| e.to_string())?;
            // Exercise cleanup of disk-backed ASR checkpoints without private audio.
            std::fs::write(
                folder
                    .join(".checkpoints")
                    .join(format!("asr-pending-{}-0.pcm", uuid::Uuid::new_v4())),
                [0_u8; 4],
            )
            .map_err(|e| e.to_string())?;
            sqlx::query("UPDATE meetings SET folder_path=? WHERE id=? AND recording_session_id=?")
                .bind(folder.to_str())
                .bind(id)
                .bind(session)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!({"folder":folder}))
        }
        "exit" => {
            app.exit(0);
            Ok(Value::Null)
        }
        _ => Err("Unknown workflow control action".into()),
    }
}
