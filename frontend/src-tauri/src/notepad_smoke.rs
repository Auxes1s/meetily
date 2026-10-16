//! Opt-in native WebView smoke checks; never run against the user's library.
use tauri::{AppHandle, Emitter, Listener, Manager};

fn require_transcript_smoke(app: &AppHandle) -> Result<(), String> {
    let workflow = app.config().identifier == "com.meetily.workflow.test"
        && std::env::var("MEETILY_WORKFLOW_SMOKE").as_deref() == Ok("1");
    if !cfg!(debug_assertions) || (!workflow && app.config().identifier != "com.meetily.notepad.test")
        || std::env::var("MEETILY_TRANSCRIPT_SMOKE").as_deref() != Ok("1") {
        return Err("Transcript smoke requires an explicitly enabled isolated debug profile".into());
    }
    Ok(())
}

/// Feed known microphone speech through the production VAD, Whisper worker,
/// native event, and persistence paths. No hardware or private meeting audio.
#[tauri::command]
pub async fn transcript_smoke_run(app: AppHandle, meeting_id: String) -> Result<serde_json::Value, String> {
    require_transcript_smoke(&app)?;
    use crate::audio::{recording_state::{AudioChunk, DeviceType, RecordingState}, pipeline::AudioPipeline};
    use std::sync::Arc;
    let model_dir = std::env::var("MEETILY_SMOKE_MODEL_DIR").map_err(|_| "Set MEETILY_SMOKE_MODEL_DIR to existing local models")?;
    let model = std::env::var("MEETILY_SMOKE_MODEL").unwrap_or_else(|_| "large-v3".into());
    let engine = Arc::new(crate::whisper_engine::WhisperEngine::new_with_models_dir(Some(model_dir.into())).map_err(|e| e.to_string())?);
    engine.discover_models().await.map_err(|e| e.to_string())?;
    engine.load_model(&model).await.map_err(|e| e.to_string())?;
    *crate::whisper_engine::commands::WHISPER_ENGINE.lock().unwrap() = Some(engine.clone());
    let state = app.state::<crate::state::AppState>();
    crate::database::repositories::setting::SettingsRepository::save_transcript_config(state.db_manager.pool(), "localWhisper", &model).await.map_err(|e| e.to_string())?;
    let session = uuid::Uuid::new_v4().to_string();
    sqlx::query("UPDATE meetings SET status='recording', recording_session_id=? WHERE id=? AND status='draft'")
        .bind(&session).bind(&meeting_id).execute(state.db_manager.pool()).await.map_err(|e| e.to_string())?;
    let mut saver = crate::audio::recording_saver::RecordingSaver::new();
    saver.set_identity(Some(meeting_id.clone()), Some(session.clone()));
    let sink = saver.clone();
    let expected = session.clone();
    let listener = app.listen("transcript-update", move |event| {
        if let Ok(update) = serde_json::from_str::<crate::audio::transcription::TranscriptUpdate>(event.payload()) {
            if update.recording_session_id.as_deref() == Some(&expected) && !update.is_partial {
                sink.add_transcript_segment(crate::audio::recording_saver::TranscriptSegment {
                    id: format!("transcript-{expected}-{}", update.sequence_id), text: update.text,
                    audio_start_time: update.audio_start_time, audio_end_time: update.audio_end_time, duration: update.duration,
                    display_time: update.timestamp, confidence: update.confidence, sequence_id: update.sequence_id, audio_source: update.audio_source,
                });
            }
        }
    });
    let fixture = crate::audio::decoder::decode_audio_file(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio-source-speech.wav")).map_err(|e| e.to_string())?;
    let capture = RecordingState::new();
    capture.start_recording().map_err(|e| e.to_string())?;
    let (audio_tx, audio_rx) = crate::audio::queue::pcm_channel();
    let (transcript_tx, transcript_rx) = crate::audio::queue::pcm_channel();
    let mut pipeline = AudioPipeline::new(audio_rx, transcript_tx, capture.clone(), 2000, 16000,
        "Fixture microphone".into(), crate::audio::InputDeviceKind::Wired,
        "Fixture silent system".into(), crate::audio::InputDeviceKind::Wired).map_err(|e| e.to_string())?;
    let overlapping = crate::audio::transcription::preferences::continuous_whisper_enabled(&app)?;
    if overlapping {
        pipeline = pipeline.with_overlapping_transcription().map_err(|e|e.to_string())?;
    }
    app.emit("recording-started", serde_json::json!({"meeting_id":meeting_id, "recording_session_id":session})).map_err(|e| e.to_string())?;
    let worker = crate::audio::transcription::worker::start_transcription_task_for_session(app.clone(), transcript_rx, (Some(meeting_id.clone()), Some(session.clone())), overlapping);
    let pipeline_task = tokio::spawn(pipeline.run());
    let samples: Vec<f32> = fixture.samples.into_iter().chain(std::iter::repeat(0.0).take(16000)).collect();
    for (i, frame) in samples.chunks(9600).enumerate() {
        if i == 8 {
            capture.pause_recording().map_err(|e|e.to_string())?;
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            capture.resume_recording().map_err(|e|e.to_string())?;
        }
        audio_tx.send(AudioChunk { data: frame.to_vec(), sample_rate:16000, timestamp:i as f64*0.6, chunk_id:i as u64, device_type:DeviceType::Microphone }).map_err(|e| e.to_string())?;
        audio_tx.send(AudioChunk { data:vec![0.0;frame.len()],sample_rate:16000,timestamp:i as f64*0.6,chunk_id:i as u64,device_type:DeviceType::System }).map_err(|e|e.to_string())?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    drop(audio_tx);
    pipeline_task.await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    worker.await.map_err(|e| e.to_string())?;
    // Tauri listeners dispatch asynchronously; wait for the native sink acknowledgment.
    for _ in 0..100 { if !saver.get_transcript_segments().is_empty() { break; } tokio::time::sleep(std::time::Duration::from_millis(20)).await; }
    app.unlisten(listener);
    let segments = saver.get_transcript_segments();
    if segments.is_empty() { return Err("Known speech produced no final native transcript".into()); }
    let transcripts: Vec<_> = segments.iter().map(|segment| crate::api::TranscriptSegment {
        id:segment.id.clone(), text:segment.text.clone(), timestamp:segment.display_time.clone(), audio_source:segment.audio_source,
        audio_start_time:Some(segment.audio_start_time), audio_end_time:Some(segment.audio_end_time), duration:Some(segment.duration), sequence_id:Some(segment.sequence_id as i64),
    }).collect();
    crate::database::repositories::transcript::TranscriptsRepository::save_transcript(state.db_manager.pool(), "Speech fixture", &transcripts, None, Some(meeting_id.clone()), Some(session.clone())).await.map_err(|e| e.to_string())?;
    crate::audio::transcription::streaming::cleanup_pending(&app.path().app_data_dir().map_err(|e|e.to_string())?.join("workflow-pending"),&session).map_err(|e|e.to_string())?;
    Ok(serde_json::json!({"meeting_id":meeting_id,"recording_session_id":session,"segments":segments,"model":model,"pause_resume":true,"overlapping":overlapping}))
}
// Capture the test WKWebView itself. This avoids screen-capture permissions,
// other applications' pixels, and dependence on the Mac's lock screen.
#[cfg(all(target_os = "macos", debug_assertions))]
async fn responsive_snapshot(app: &AppHandle, name: &str) -> Result<(), String> {
    use objc::{class, msg_send, sel, sel_impl, runtime::Object};
    if !name.ends_with(".png") || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'.') {
        return Err("Invalid test snapshot filename".into());
    }
    let folder = app.path().app_data_dir().map_err(|e| e.to_string())?.join("responsive-screenshots");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let destination = folder.join(name);
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.get_webview_window("main").ok_or("Missing test WebView")?.with_webview(move |webview| {
        let mut sender = Some(sender);
        let block = cidre::blocks::ResultCh::<cidre::ns::Id>::new2(move |image, error| {
            let result = if let Some(image) = image {
                unsafe {
                    let image = image as *const _ as *mut Object;
                    let tiff: *mut Object = msg_send![image, TIFFRepresentation];
                    let bitmap: *mut Object = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff];
                    let properties: *mut Object = msg_send![class!(NSDictionary), dictionary];
                    let png: *mut Object = msg_send![bitmap, representationUsingType: 4usize properties: properties];
                    if png.is_null() { Err("WKWebView snapshot encoding failed".into()) }
                    else {
                        let bytes: *const u8 = msg_send![png, bytes];
                        let length: usize = msg_send![png, length];
                        std::fs::write(&destination, std::slice::from_raw_parts(bytes, length)).map_err(|e| e.to_string())
                    }
                }
            } else { Err(format!("WKWebView snapshot failed: {error:?}")) };
            if let Some(sender) = sender.take() { let _ = sender.send(result); }
        });
        unsafe {
            let view = webview.inner() as *mut Object;
            let nil: *mut Object = std::ptr::null_mut();
            let completion = (&*block as *const cidre::blocks::ResultCh<cidre::ns::Id>).cast::<std::ffi::c_void>();
            let _: () = msg_send![view, takeSnapshotWithConfiguration: nil completionHandler: completion];
        }
    }).map_err(|e| e.to_string())?;
    tokio::time::timeout(std::time::Duration::from_secs(10), receiver).await
        .map_err(|_| "WKWebView snapshot timed out".to_string())?
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn notepad_smoke_report(app: AppHandle, report: serde_json::Value) -> Result<(), String> {
    let summary_smoke=app.config().identifier=="com.meetily.summary.test" && std::env::var("MEETILY_SUMMARY_SMOKE").as_deref()==Ok("1");
    let recording_smoke=app.config().identifier=="com.meetily.recording.test"
        && std::env::var("MEETILY_RECORDING_SMOKE").as_deref()==Ok("1");
    let responsive_smoke = app.config().identifier == "com.meetily.responsive.test"
        && std::env::var("MEETILY_RESPONSIVE_SMOKE").as_deref() == Ok("1");
    let export_smoke = app.config().identifier == "com.meetily.export.test"
        && std::env::var("MEETILY_EXPORT_SMOKE").as_deref() == Ok("1");
    if !cfg!(debug_assertions) || (app.config().identifier != "com.meetily.notepad.test" && !export_smoke && !responsive_smoke && !recording_smoke && !summary_smoke) {
        return Err("Smoke reports require an isolated debug test profile".into());
    }
    let path = app.path().app_data_dir().map_err(|e| e.to_string())?.join(if summary_smoke {"summary-generation-smoke.json"} else if recording_smoke {"recording-start-smoke.json"} else if responsive_smoke { "responsive-layout-smoke.json" } else if export_smoke { "meeting-export-smoke.json" } else { "notepad-smoke.json" });
    #[cfg(all(target_os = "macos", debug_assertions))]
    if responsive_smoke || recording_smoke {
        if let Some(name) = report.get("screenshot").and_then(|name| name.as_str()) { responsive_snapshot(&app, name).await?; }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub fn run(webview: &tauri::Webview, event: tauri::webview::PageLoadEvent) {
    if cfg!(debug_assertions) && webview.app_handle().config().identifier=="com.meetily.summary.test"
        && std::env::var("MEETILY_SUMMARY_SMOKE").as_deref()==Ok("1")
        && event==tauri::webview::PageLoadEvent::Finished {
        let _=webview.eval(include_str!("../smoke/summary.js")); return;
    }

    if crate::q02_validation::enabled(webview.app_handle()) && event == tauri::webview::PageLoadEvent::Finished {
        let _ = webview.eval(include_str!("../smoke/q02.js"));
        return;
    }
    if cfg!(debug_assertions) && webview.app_handle().config().identifier=="com.meetily.recording.test"
        && std::env::var("MEETILY_RECORDING_SMOKE").as_deref()==Ok("1")
        && event==tauri::webview::PageLoadEvent::Finished {
        let _=webview.eval(include_str!("../smoke/recording.js"));
        return;
    }
    if crate::workflow_smoke::enabled(webview.app_handle()) && event == tauri::webview::PageLoadEvent::Finished {
        let _ = webview.eval(include_str!("../smoke/workflow.js"));
        return;
    }
    if cfg!(debug_assertions) && webview.app_handle().config().identifier == "com.meetily.responsive.test" {
        log::info!("Responsive native navigation: {:?}, {:?}", event, webview.url());
    }
    if cfg!(debug_assertions) && webview.app_handle().config().identifier == "com.meetily.responsive.test"
        && std::env::var("MEETILY_RESPONSIVE_SMOKE").as_deref() == Ok("1")
        && event == tauri::webview::PageLoadEvent::Finished {
        let _ = webview.eval(include_str!("../smoke/responsive.js"));
        return;
    }
    if cfg!(debug_assertions) && webview.app_handle().config().identifier == "com.meetily.export.test"
        && std::env::var("MEETILY_EXPORT_SMOKE").as_deref() == Ok("1")
        && event == tauri::webview::PageLoadEvent::Finished {
        let _ = webview.eval(include_str!("../smoke/export.js"));
        return;
    }
    if require_transcript_smoke(webview.app_handle()).is_ok() && event == tauri::webview::PageLoadEvent::Finished {
        let _ = webview.eval(include_str!("../smoke/transcript.js"));
        return;
    }
    log::info!("Notepad smoke page event {:?}, isolated={}, requested={}", event, webview.app_handle().config().identifier == "com.meetily.notepad.test", std::env::var("MEETILY_NOTEPAD_SMOKE").as_deref() == Ok("1"));
    if cfg!(debug_assertions) && webview.app_handle().config().identifier == "com.meetily.notepad.test"
        && std::env::var("MEETILY_NOTEPAD_SMOKE").as_deref() == Ok("1")
        && event == tauri::webview::PageLoadEvent::Finished {
        let _ = webview.eval(include_str!("../smoke/notepad.js"));
    }
}

/// Hardware checks are available only to the explicitly enabled, isolated
/// recording test app. The release application cannot play a fixture or probe
/// recording files through this command.
#[tauri::command]
pub async fn recording_smoke_probe(app: AppHandle, meeting_id: Option<String>, play_fixture: bool, exercise_controls: Option<bool>) -> Result<serde_json::Value,String> {
    if !cfg!(debug_assertions) || app.config().identifier!="com.meetily.recording.test"
        || std::env::var("MEETILY_RECORDING_SMOKE").as_deref()!=Ok("1") {
        return Err("Recording hardware checks require the isolated debug profile".into());
    }
    let provider = match std::env::var("MEETILY_Q02_CAPTURE_PROVIDER").as_deref() {
        Ok("parakeet") => "parakeet",
        Ok("whisper") | Err(_) => "localWhisper",
        _ => return Err("Capture fixture requires an installed local provider".into()),
    };
    let controls = exercise_controls == Some(true);
    if controls {
        crate::audio::recording_commands::pause_recording(app.clone(), None).await?;
        if crate::audio::recording_commands::get_recording_state().await["is_paused"] != true {
            return Err("Physical capture did not enter paused state".into());
        }
        let window = app.get_webview_window("main").ok_or("Missing capture window")?;
        window.hide().map_err(|e|e.to_string())?;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        if window.is_visible().map_err(|e|e.to_string())? { return Err("Capture window did not hide".into()); }
        window.show().map_err(|e|e.to_string())?;
        if !window.is_visible().map_err(|e|e.to_string())? { return Err("Capture window did not reopen".into()); }
        crate::audio::recording_commands::resume_recording(app.clone(), None).await?;
        if crate::audio::recording_commands::get_recording_state().await["is_paused"] != false {
            return Err("Physical capture did not resume".into());
        }
    }
    if play_fixture {
        #[cfg(target_os="macos")]
        {
            let path=app.path().app_data_dir().map_err(|e|e.to_string())?.join("recording-speech-fixture.wav");
            std::fs::write(&path,include_bytes!("../tests/fixtures/audio-source-speech.wav")).map_err(|e|e.to_string())?;
            let status=tokio::task::spawn_blocking(move ||std::process::Command::new("/usr/bin/afplay").arg(path).status()).await.map_err(|e|e.to_string())?.map_err(|e|e.to_string())?;
            if !status.success(){return Err("Known speech playback failed".into());}
        }
    }
    let mut files=Vec::new();
    if let Some(id)=meeting_id {
        let state=app.state::<crate::state::AppState>();
        for session in crate::database::recording_sessions::list(state.db_manager.pool(),&id).await? {
            let audio=session.folder_path.as_ref().map(|folder|std::path::Path::new(folder).join("audio.mp4"));
            files.push(serde_json::json!({"id":session.id,"audio_saved":audio.is_some_and(|p|p.metadata().is_ok_and(|m|m.len()>0))}));
        }
    }
    Ok(serde_json::json!({"files":files,"fixture_provider":provider,"native_device_pause_resume":controls,"native_hide_reopen":controls}))
}
