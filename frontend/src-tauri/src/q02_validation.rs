//! Opt-in native replay. Production identifiers always reject this harness.
//! Replay is source-tagged VAD/ASR/IPC/persistence, never physical capture.
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub(crate) fn allowed(feature: bool, identifier: &str, requested: bool) -> bool {
    feature && identifier == "com.meetily.q02.test" && requested
}
pub(crate) fn enabled<R: tauri::Runtime>(app: &AppHandle<R>) -> bool {
    allowed(
        cfg!(feature = "q02-validation"),
        &app.config().identifier,
        std::env::var("MEETILY_Q02_VALIDATE").as_deref() == Ok("1"),
    )
}
fn root<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    if !enabled(app) {
        return Err("Q02 requires its opt-in isolated validation bundle".into());
    }
    let p =
        PathBuf::from(std::env::var("MEETILY_Q02_RUN_DIR").map_err(|_| "Missing Q02 directory")?)
            .canonicalize()
            .map_err(|e| e.to_string())?;
    if !p
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.starts_with("meetily-q02-"))
    {
        return Err("Q02 requires a dedicated meetily-q02 directory".into());
    }
    Ok(p)
}
pub(crate) fn checkpoint_root<R: tauri::Runtime>(
    app: &AppHandle<R>,
    session: &str,
) -> Result<PathBuf, String> {
    if uuid::Uuid::parse_str(session).is_err() {
        return Err("Invalid Q02 session".into());
    }
    Ok(root(app)?
        .join("recordings")
        .join(session)
        .join(".checkpoints"))
}
struct Trace {
    start: Instant,
    rows: Vec<Value>,
    checkpoint_count: u64,
    input_count: u64,
}
static TRACE: OnceLock<Mutex<Option<Trace>>> = OnceLock::new();
fn trace() -> &'static Mutex<Option<Trace>> {
    TRACE.get_or_init(|| Mutex::new(None))
}
pub(crate) fn record(mut row: Value) {
    if !cfg!(feature = "q02-validation") {
        return;
    }
    if let Ok(mut guard) = trace().lock() {
        if let Some(t) = guard.as_mut() {
            row["wall_seconds"] = json!(t.start.elapsed().as_secs_f64());
            t.rows.push(row);
        }
    }
}
pub(crate) fn input(source: crate::audio::source::AudioSource, samples: usize, end: f64) {
    if !cfg!(feature = "q02-validation") {
        return;
    }
    if let Ok(mut guard) = trace().lock() {
        if let Some(t) = guard.as_mut() {
            t.input_count += 1;
        }
    }
    record(json!({"kind":"input","source":source,"samples":samples,"audio_end":end}));
}
pub(crate) fn verify_checkpoint(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    if !cfg!(feature = "q02-validation") {
        return Ok(());
    }
    let active = trace().lock().map_err(|e| e.to_string())?.is_some();
    if active {
        if std::fs::read(path).map_err(|e| e.to_string())? != bytes {
            return Err("Q02 checkpoint audio differs from supplied source PCM".into());
        }
        trace()
            .lock()
            .map_err(|e| e.to_string())?
            .as_mut()
            .unwrap()
            .checkpoint_count += 1;
    }
    Ok(())
}
fn write(path: PathBuf, value: &Value) -> Result<(), String> {
    crate::database::export::atomic_write(
        &path,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.message)
}
#[tauri::command]
pub async fn q02_validation_control(
    app: AppHandle,
    action: String,
    payload: Option<Value>,
) -> Result<Value, String> {
    let dir = root(&app)?;
    match action.as_str() {
        "info" => {
            if let Some(window) = app.get_webview_window("main") {
                window.show().map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
            }
            let v: Value = serde_json::from_slice(
                &std::fs::read(dir.join("run.json")).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            Ok(v)
        }
        "report" => {
            let v = payload.ok_or("Missing report")?;
            write(dir.join("report.json"), &v)?;
            Ok(v)
        }
        "ui_progress" => {
            let v = payload.ok_or("Missing UI progress")?;
            write(dir.join("ui-progress.json"), &v)?;
            Ok(json!({"written":true}))
        }
        "health" => serde_json::to_value(crate::audio::transcription::health::snapshot())
            .map_err(|e| e.to_string()),
        "clock" => Ok(
            json!({"wall_seconds":trace().lock().map_err(|e|e.to_string())?.as_ref().map(|t|t.start.elapsed().as_secs_f64())}),
        ),
        "seed_stress" => {
            let id = payload
                .as_ref()
                .and_then(|v| v["meeting_id"].as_str())
                .ok_or("Missing stress meeting")?;
            let pool = app
                .state::<crate::state::AppState>()
                .db_manager
                .pool()
                .clone();
            crate::database::deletion::assert_meeting_writable(&pool, id).await?;
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
            for i in 0..10_000 {
                sqlx::query("INSERT INTO transcripts(id,meeting_id,transcript,timestamp,audio_start_time,audio_end_time,duration,speaker) VALUES(?,?,?,'00:00:00',?,?,1,'unknown')")
                    .bind(format!("q02-stress-{id}-{i}")).bind(id).bind(format!("Synthetic stored stress passage {i} — café 日本語"))
                    .bind(i as f64).bind(i as f64+1.).execute(&mut *tx).await.map_err(|e|e.to_string())?;
            }
            tx.commit().await.map_err(|e| e.to_string())?;
            Ok(json!({"synthetic_stored_transcripts":10000}))
        }
        "export" => {
            use crate::database::export::{
                read_record, DocumentSelection, ExportFormat, ExportRequest,
            };
            let id = payload
                .as_ref()
                .and_then(|v| v["meeting_id"].as_str())
                .ok_or("Missing export meeting")?;
            let pool = app
                .state::<crate::state::AppState>()
                .db_manager
                .pool()
                .clone();
            let record = read_record(&pool, id).await.map_err(|e| e.message)?;
            let folder = dir.join("exports");
            std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
            for (format, name) in [
                (ExportFormat::Json, "meeting.json"),
                (ExportFormat::Markdown, "meeting.md"),
            ] {
                let content = record
                    .content(&ExportRequest {
                        meeting_id: id.into(),
                        format,
                        document: DocumentSelection::Original,
                        include_transcript: true,
                        expected_approved_version_id: None,
                    })
                    .map_err(|e| e.message)?;
                crate::database::export::atomic_write(&folder.join(name), &content)
                    .map_err(|e| e.message)?;
            }
            Ok(json!({"native_export_content":true,"save_dialog_tested":false}))
        }
        "sidecar" => {
            let started = Instant::now();
            let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
            let text = tokio::time::timeout(
                std::time::Duration::from_secs(120),
                crate::summary::summary_engine::generate_with_builtin(
                    &app_data,
                    "qwen3.5:4b",
                    "Reply with OK for a local package validation fixture.",
                    "Q02 package fixture.",
                    None,
                ),
            )
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
            let report = json!({"installed_model":"qwen3.5:4b","elapsed_seconds":started.elapsed().as_secs_f64(),
                "nonempty_response":!text.trim().is_empty(),"native_packaged_sidecar":true,"enhancement_quality_acceptance":false});
            write(dir.join("sidecar.json"), &report)?;
            Ok(report)
        }
        "exit" => {
            app.exit(0);
            Ok(json!({"exiting":true}))
        }
        _ => Err("Unknown Q02 action".into()),
    }
}

fn read_pcm(p: PathBuf) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return Err("PCM must be nonempty 16 kHz mono f32le".into());
    }
    let audio: Vec<_> = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    if audio.iter().any(|x| !x.is_finite() || x.abs() > 1.) {
        return Err("PCM must be finite and bounded".into());
    }
    Ok(audio)
}

#[tauri::command]
pub async fn q02_validation_replay(app: AppHandle, meeting_id: String) -> Result<Value, String> {
    use crate::audio::{
        recording_state::{AudioChunk, DeviceType, TranscriptionChunk},
        source::AudioSource,
        source_vad::SourceVadProcessor,
    };
    let dir = root(&app)?;
    let cfg: Value =
        serde_json::from_slice(&std::fs::read(dir.join("run.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let microphone = read_pcm(dir.join("inputs/microphone.f32"))?;
    let system = read_pcm(dir.join("inputs/system.f32"))?;
    if microphone.len() != system.len() {
        return Err("Source fixtures must share a clock".into());
    }
    let duration = cfg["seconds"].as_f64().ok_or("Missing duration")?;
    if !duration.is_finite() || duration <= 0. || duration > 14_400. {
        return Err("Q02 duration must be in (0,14400]".into());
    }
    let provider = cfg["provider"].as_str().ok_or("Missing provider")?;
    let model = cfg["model"].as_str().ok_or("Missing model")?;
    if !matches!(provider, "localWhisper" | "parakeet") {
        return Err("Q02 only supports installed local ASR".into());
    }
    let preference = crate::audio::transcription::preferences::continuous_whisper_enabled(&app)?;
    if provider == "localWhisper" && preference != (cfg["routing"].as_str() == Some("overlapping"))
    {
        return Err("Q02 requested route disagrees with the persistent native preference".into());
    }
    let overlapping = provider == "localWhisper" && preference;
    let paced = cfg["paced"].as_bool().unwrap_or(true);
    let language = cfg["language"].as_str().unwrap_or("auto").to_string();
    let decoder_language = if language == "auto" {
        None
    } else {
        Some(language.clone())
    };
    crate::set_language_preference(language.clone()).await?;
    let pool = app
        .state::<crate::state::AppState>()
        .db_manager
        .pool()
        .clone();
    crate::database::repositories::setting::SettingsRepository::save_transcript_config(
        &pool, provider, model,
    )
    .await
    .map_err(|e| e.to_string())?;
    let models = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("models");
    if provider == "localWhisper" {
        let engine = Arc::new(
            crate::whisper_engine::WhisperEngine::new_with_models_dir(Some(models.clone()))
                .map_err(|e| e.to_string())?,
        );
        engine.discover_models().await.map_err(|e| e.to_string())?;
        engine.load_model(model).await.map_err(|e| e.to_string())?;
        let warm = if cfg["source"].as_str() == Some("system") {
            &system
        } else {
            &microphone
        };
        engine
            .transcribe_audio_with_confidence(
                warm[..warm.len().min(80_000)].to_vec(),
                decoder_language,
            )
            .await
            .map_err(|e| e.to_string())?;
        *crate::whisper_engine::commands::WHISPER_ENGINE
            .lock()
            .map_err(|e| e.to_string())? = Some(engine);
    } else {
        let engine = Arc::new(
            crate::parakeet_engine::ParakeetEngine::new_with_models_dir(Some(models))
                .map_err(|e| e.to_string())?,
        );
        engine.discover_models().await.map_err(|e| e.to_string())?;
        engine.load_model(model).await.map_err(|e| e.to_string())?;
        let warm = if cfg["source"].as_str() == Some("system") {
            &system
        } else {
            &microphone
        };
        engine
            .transcribe_audio(warm[..warm.len().min(80_000)].to_vec())
            .await
            .map_err(|e| e.to_string())?;
        *crate::parakeet_engine::commands::PARAKEET_ENGINE
            .lock()
            .map_err(|e| e.to_string())? = Some(engine);
    }
    let session = uuid::Uuid::new_v4().to_string();
    let folder = dir.join("recordings").join(&session);
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    crate::database::recording_sessions::begin(&pool, &meeting_id, &session).await?;
    crate::database::recording_sessions::mark_live(
        &pool,
        &session,
        Some(folder.to_string_lossy().into()),
    )
    .await?;
    let updates = Arc::new(Mutex::new(Vec::<
        crate::audio::transcription::TranscriptUpdate,
    >::new()));
    let received = updates.clone();
    let sid = session.clone();
    let event_app = app.clone();
    *trace().lock().map_err(|e| e.to_string())? = Some(Trace {
        start: Instant::now(),
        rows: Vec::new(),
        checkpoint_count: 0,
        input_count: 0,
    });
    let listener=app.listen("transcript-update",move|event|{
        if let Ok(update)=serde_json::from_str::<crate::audio::transcription::TranscriptUpdate>(event.payload()){
            if update.recording_session_id.as_deref()!=Some(&sid){return;}
            record(json!({"kind":"event","sequence":update.sequence_id,"revision":update.update_revision,"partial":update.is_partial,"source":update.audio_source,"audio_start":update.audio_start_time,"audio_end":update.audio_end_time}));
            let wall=trace().lock().unwrap().as_ref().unwrap().start.elapsed().as_secs_f64();
            let _=event_app.emit("q02-transcript",json!({"update":update,"wall_seconds":wall}));
            received.lock().unwrap().push(update);
        }
    });
    let (tx, rx) = crate::audio::queue::pcm_channel();
    app.emit(
        "recording-started",
        json!({"meeting_id":meeting_id,"recording_session_id":session}),
    )
    .map_err(|e| e.to_string())?;
    let worker = crate::audio::transcription::worker::start_transcription_task_for_session(
        app.clone(),
        rx,
        (Some(meeting_id.clone()), Some(session.clone())),
        overlapping,
    );
    let mut vad = if overlapping {
        SourceVadProcessor::new_streaming(16_000, 500)
    } else {
        SourceVadProcessor::new(16_000, 500)
    }
    .map_err(|e| e.to_string())?;
    let samples = (duration * 16_000.).round() as usize;
    use std::io::Write;
    let mut mic_file =
        std::fs::File::create(folder.join("microphone-source.f32")).map_err(|e| e.to_string())?;
    let mut system_file =
        std::fs::File::create(folder.join("system-source.f32")).map_err(|e| e.to_string())?;
    let started = Instant::now();
    let mut max_late = 0f64;
    let mut pauses = 0.;
    let mut paused = false;
    let mut chunk_id = 0;
    let send = |segments: Vec<crate::audio::source_vad::SourceSpeechSegment>,
                id: &mut u64|
     -> Result<(), String> {
        for segment in segments {
            let source = segment.source;
            let speech = segment.speech;
            input(
                source,
                speech.samples.len(),
                speech.end_timestamp_ms / 1000.,
            );
            tx.send(TranscriptionChunk {
                audio: AudioChunk {
                    data: speech.samples,
                    sample_rate: 16_000,
                    timestamp: speech.start_timestamp_ms / 1000.,
                    chunk_id: *id,
                    device_type: if source == AudioSource::System {
                        DeviceType::System
                    } else {
                        DeviceType::Microphone
                    },
                },
                audio_source: source,
                stream_end: overlapping.then_some(speech.is_final),
            })
            .map_err(|e| e.to_string())?;
            *id += 1;
        }
        Ok(())
    };
    for offset in (0..samples).step_by(960) {
        let n = 960.min(samples - offset);
        if paced {
            let deadline = started
                + std::time::Duration::from_secs_f64((offset + n) as f64 / 16_000. + pauses);
            tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
            max_late = max_late
                .max(started.elapsed().as_secs_f64() - ((offset + n) as f64 / 16_000. + pauses));
        }
        if !paused && cfg["pause"].as_bool() == Some(true) && offset >= samples / 2 {
            send(vad.seal_stream(), &mut chunk_id)?;
            if paced {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                pauses += 0.25;
            }
            record(json!({"kind":"pause_resume","audio_position":offset as f64/16000.}));
            paused = true;
        }
        let mic: Vec<_> = (offset..offset + n)
            .map(|i| microphone[i % microphone.len()])
            .collect();
        let sys: Vec<_> = (offset..offset + n)
            .map(|i| system[i % system.len()])
            .collect();
        mic_file
            .write_all(&mic.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>())
            .map_err(|e| e.to_string())?;
        system_file
            .write_all(&sys.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>())
            .map_err(|e| e.to_string())?;
        send(
            vad.process_audio(&mic, &sys).map_err(|e| e.to_string())?,
            &mut chunk_id,
        )?;
        if offset % 160_320 == 0 {
            let (inputs, checkpoints, events) = {
                let guard = trace().lock().map_err(|e| e.to_string())?;
                let t = guard.as_ref().ok_or("Missing trace")?;
                (
                    t.input_count,
                    t.checkpoint_count,
                    updates.lock().map_err(|e| e.to_string())?.len(),
                )
            };
            write(
                dir.join("progress.json"),
                &json!({"audio_position":(offset+n) as f64/16000.,"audio_seconds":duration,
                "source_deliveries":inputs,"checkpoint_deliveries_verified":checkpoints,"native_updates":events,"physical_capture":false,
                "worker_health":crate::audio::transcription::health::snapshot()}),
            )?;
        }
    }
    let stop = started.elapsed().as_secs_f64();
    record(json!({"kind":"stop","audio_seconds":duration}));
    mic_file.sync_all().map_err(|e| e.to_string())?;
    system_file.sync_all().map_err(|e| e.to_string())?;
    send(vad.flush().map_err(|e| e.to_string())?, &mut chunk_id)?;
    drop(tx);
    worker
        .await
        .map_err(|e| format!("Worker did not drain: {e}"))?;
    // Event taps are asynchronous; a barrier event follows all transcript emits.
    let (done, wait) = tokio::sync::oneshot::channel();
    let mut done = Some(done);
    let barrier = app.once("q02-barrier", move |_| {
        if let Some(done) = done.take() {
            let _ = done.send(());
        }
    });
    app.emit("q02-barrier", json!({}))
        .map_err(|e| e.to_string())?;
    tokio::time::timeout(std::time::Duration::from_secs(10), wait)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    app.unlisten(barrier);
    app.unlisten(listener);
    let captured = updates.lock().map_err(|e| e.to_string())?.clone();
    let final_rows: Vec<_> = captured
        .iter()
        .filter(|u| !u.is_partial && !u.text.trim().is_empty())
        .map(|u| crate::api::TranscriptSegment {
            id: format!("transcript-{session}-{}", u.sequence_id),
            text: u.text.clone(),
            timestamp: u.timestamp.clone(),
            audio_source: u.audio_source,
            audio_start_time: Some(u.audio_start_time),
            audio_end_time: Some(u.audio_end_time),
            duration: Some(u.duration),
            sequence_id: Some(u.sequence_id as i64),
        })
        .collect();
    write(
        folder.join("metadata.json"),
        &json!({"duration_seconds":duration,"source":"q02_source_replay","physical_capture":false}),
    )?;
    crate::database::repositories::transcript::TranscriptsRepository::save_transcript(
        &pool,
        "Q02 native replay",
        &final_rows,
        Some(folder.to_string_lossy().into()),
        Some(meeting_id.clone()),
        Some(session.clone()),
    )
    .await
    .map_err(|e| e.to_string())?;
    let end = started.elapsed().as_secs_f64();
    let persisted: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM transcripts WHERE recording_session_id=?")
            .bind(&session)
            .fetch_one(&pool)
            .await
            .map_err(|e| e.to_string())?;
    if persisted as usize != final_rows.len() {
        return Err("Final manifest differs from SQLite rows".into());
    }
    let t = trace()
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or("Missing trace")?;
    if overlapping && t.checkpoint_count != t.input_count {
        return Err("Not every source delivery has a verified retained checkpoint".into());
    }
    let checkpoint_dir = checkpoint_root(&app, &session)?;
    crate::audio::transcription::streaming::cleanup_pending(&checkpoint_dir, &session)
        .map_err(|e| e.to_string())?;
    write(
        dir.join(format!("trace-{session}.json")),
        &json!({"rows":t.rows}),
    )?;
    write(
        dir.join(format!("transcripts-{session}.json")),
        &json!(captured),
    )?;
    let result = json!({"ok":true,"native_ipc":true,"physical_capture":false,"paced_replay":paced,"virtual_capture_clock":false,
        "effective_whisper_settings":if provider=="localWhisper"{crate::whisper_engine::decode_parameters::description(Some(&language))}else{json!(null)},
        "provider":provider,"model":model,"language":language,"routing":if overlapping{"overlapping"}else{"legacy"},"audio_seconds":duration,"capture_wall_seconds":stop,
        "stop_to_commit_seconds":end-stop,"wall_seconds":end,"max_replay_lateness_seconds":max_late,"vad_pause_boundary_replay":paused,"native_device_pause_resume":false,"meeting_id":meeting_id,
        "recording_session_id":session,"final_segments":persisted,"source_checkpoint_deliveries_verified":t.checkpoint_count,"source_deliveries":t.input_count,
        "transcript_fidelity_acceptance":false,"display_latency_acceptance":false,"full_capture_pipeline":false});
    write(dir.join("native-result.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_identifiers_always_reject_fixture_commands() {
        for feature in [false, true] {
            for requested in [false, true] {
                assert!(!allowed(feature, "com.meetily.ai", requested));
            }
        }
        assert!(!allowed(false, "com.meetily.q02.test", true));
        assert!(!allowed(true, "com.meetily.q02.test", false));
        assert!(allowed(true, "com.meetily.q02.test", true));
    }
    #[test]
    fn pcm_rejects_nan_and_truncated_inputs() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("audio");
        for bytes in [
            vec![],
            vec![1, 2, 3],
            f32::NAN.to_le_bytes().to_vec(),
            1.1f32.to_le_bytes().to_vec(),
        ] {
            std::fs::write(&path, bytes).unwrap();
            assert!(read_pcm(path.clone()).is_err());
        }
        std::fs::write(&path, 0.2f32.to_le_bytes()).unwrap();
        assert_eq!(read_pcm(path).unwrap(), vec![0.2]);
    }
}
