// audio/transcription/worker.rs
//
// Parallel transcription worker pool and chunk processing logic.

use super::engine::TranscriptionEngine;
use super::provider::TranscriptionError;
use crate::audio::AudioChunk;
use crate::audio::recording_state::TranscriptionChunk;
use crate::audio::source::AudioSource;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, Runtime};

// Sequence counter for transcript updates
static SEQUENCE_COUNTER: AtomicU64 = AtomicU64::new(0);
pub(super) fn next_sequence() -> u64 { SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst) }

// Speech detection flag - reset per recording session
static SPEECH_DETECTED_EMITTED: AtomicBool = AtomicBool::new(false);
pub(super) fn mark_live_speech_detected<R: Runtime>(app: &AppHandle<R>) -> anyhow::Result<()> {
    if !SPEECH_DETECTED_EMITTED.swap(true, Ordering::SeqCst) {
        app.emit("speech-detected", serde_json::json!({"message":"Speech activity detected"}))?;
    }
    Ok(())
}

/// Reset the speech detected flag for a new recording session
pub fn reset_speech_detected_flag() {
    SPEECH_DETECTED_EMITTED.store(false, Ordering::SeqCst);
    info!("🔍 SPEECH_DETECTED_EMITTED reset to: {}", SPEECH_DETECTED_EMITTED.load(Ordering::SeqCst));
}

/// Returns true if the transcript text is non-trivial and should be emitted.
/// Filters empty/whitespace-only text; no confidence gating is applied.
fn should_emit_transcript(text: &str) -> bool {
    !text.trim().is_empty()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptUpdate {
    #[serde(default)]
    pub meeting_id: Option<String>,
    #[serde(default)]
    pub recording_session_id: Option<String>,
    pub text: String,
    pub timestamp: String, // Wall-clock time for reference (e.g., "14:30:05")
    pub source: String,
    #[serde(default)]
    pub audio_source: Option<AudioSource>,
    pub sequence_id: u64,
    #[serde(default)]
    pub update_revision: u64,
    pub chunk_start_time: f64, // Legacy field, kept for compatibility
    pub is_partial: bool,
    pub confidence: f32,
    // NEW: Recording-relative timestamps for playback sync
    pub audio_start_time: f64, // Seconds from recording start (e.g., 125.3)
    pub audio_end_time: f64,   // Seconds from recording start (e.g., 128.6)
    pub duration: f64,          // Segment duration in seconds (e.g., 3.3)
}

// NOTE: get_transcript_history and get_recording_meeting_name functions
// have been moved to recording_commands.rs where they have access to RECORDING_MANAGER

/// Optimized parallel transcription task ensuring ZERO chunk loss
pub fn start_transcription_task<R: Runtime>(
    app: AppHandle<R>,
    transcription_receiver: crate::audio::queue::PcmReceiver<TranscriptionChunk>,
    overlapping: bool,
) -> tokio::task::JoinHandle<()> {
    let session_identity = crate::audio::recording_commands::current_identity();
    start_transcription_task_for_session(app, transcription_receiver, session_identity, overlapping)
}

pub(crate) fn start_transcription_task_for_session<R: Runtime>(
    app: AppHandle<R>,
    transcription_receiver: crate::audio::queue::PcmReceiver<TranscriptionChunk>,
    session_identity: (Option<String>, Option<String>),
    overlapping: bool,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut inbox = super::spool::start(&app, transcription_receiver, session_identity.clone()).await
            .unwrap_or_else(|error| { super::health::fail(error.to_string()); panic!("Cannot retain ASR input: {error}") });
        let health = inbox.state.health.clone();
        let writer = inbox.writer.clone();
        let result = async {
            let engine = super::engine::get_or_init_transcription_engine(&app).await.map_err(anyhow::Error::msg)?;
            if overlapping {
                if let TranscriptionEngine::Whisper(engine) = &engine {
                    let decoder = engine.live_decoder(crate::get_language_preference_internal()).await?;
                    return super::live_worker::run(app.clone(), inbox, session_identity.clone(), decoder).await;
                }
            }
            let result = async {
                while let Some(mut pending) = inbox.recv().await? {
                    pending.load()?;
                    anyhow::ensure!(engine.is_model_loaded().await, "ASR model unloaded; pending source audio retained");
                    let timestamp = pending.chunk.timestamp;
                    let duration = pending.samples as f64 / 16000.;
                    let source = pending.chunk.audio_source;
                    #[cfg(feature="q02-validation")]
                    let started = std::time::Instant::now();
                    let audio = AudioChunk { data: std::mem::take(&mut pending.chunk.audio.data), ..pending.chunk.audio.clone() };
                    match transcribe_chunk_with_provider(&engine, audio, &app).await {
                        Ok((text, confidence, is_partial)) => {
                            #[cfg(feature="q02-validation")]
                            crate::q02_validation::record(serde_json::json!({"kind":"decode","source":source,
                                "window_seconds":duration,"audio_start":timestamp,"audio_end":timestamp+duration,
                                "inference_seconds":started.elapsed().as_secs_f64()}));
                            if should_emit_transcript(&text) {
                                mark_live_speech_detected(&app)?;
                                app.emit("transcript-update", TranscriptUpdate {
                                    meeting_id: session_identity.0.clone(), recording_session_id: session_identity.1.clone(),
                                    text, timestamp: format_current_timestamp(), source: "Audio".into(), audio_source: Some(source),
                                    sequence_id: next_sequence(), update_revision: 0, chunk_start_time: timestamp,
                                    is_partial, confidence: confidence.unwrap_or(0.85), audio_start_time: timestamp,
                                    audio_end_time: timestamp+duration, duration,
                                })?;
                            }
                        }
                        Err(TranscriptionError::AudioTooShort { .. }) => {},
                        Err(error) => return Err(anyhow::anyhow!("ASR failed; source PCM retained: {error}")),
                    }
                    inbox.state.complete(&pending);
                    app.emit("transcription-progress", super::health::snapshot())?;
                }
                Ok::<(),anyhow::Error>(())
            }.await;
            if let Err(error) = &result { health.fail(error.to_string()); }
            let writer = inbox.join_writer().await;
            result?; writer?;
            health.drained.store(true, Ordering::SeqCst);
            app.emit("transcription-queue-complete", serde_json::json!({"total_chunks":health.queued.load(Ordering::SeqCst)}))?;
            Ok(())
        }.await;
        if let Err(error) = &result {
            health.fail(error.to_string());
            let _ = app.emit("transcription-error", serde_json::json!({"error":error.to_string(),
                "userMessage":"Transcription could not finish. Pending audio is retained for recovery.","phase":"active","actionable":true}));
        }
        // Join even when model initialization or decoding failed. No writer may
        // outlive the session's Stop acknowledgement or later deletion.
        let writer_result = if let Some(handle) = writer.lock().await.take() { handle.await.map_err(anyhow::Error::from).and_then(|result|result) } else { Ok(()) };
        result.and(writer_result).unwrap_or_else(|error| {health.fail(error.to_string());panic!("Transcription did not drain: {error}")});

    })
}

/// Transcribe audio chunk using the appropriate provider (Whisper, Parakeet, or trait-based)
/// Returns: (text, confidence Option, is_partial)
async fn transcribe_chunk_with_provider<R: Runtime>(
    engine: &TranscriptionEngine,
    chunk: AudioChunk,
    app: &AppHandle<R>,
) -> std::result::Result<(String, Option<f32>, bool), TranscriptionError> {
    // Convert to 16kHz mono for transcription
    let transcription_data = if chunk.sample_rate != 16000 {
        crate::audio::audio_processing::resample_audio(&chunk.data, chunk.sample_rate, 16000)
    } else {
        chunk.data
    };

    // Skip VAD processing here since the pipeline already extracted speech using VAD
    let speech_samples = transcription_data;

    // Check for empty samples - improved error handling
    if speech_samples.is_empty() {
        warn!(
            "Audio chunk {} is empty, skipping transcription",
            chunk.chunk_id
        );
        return Err(TranscriptionError::AudioTooShort {
            samples: 0,
            minimum: 1600, // 100ms at 16kHz
        });
    }

    // Calculate energy for logging/monitoring only
    let energy: f32 =
        speech_samples.iter().map(|&x| x * x).sum::<f32>() / speech_samples.len() as f32;
    info!(
        "Processing speech audio chunk {} with {} samples (energy: {:.6})",
        chunk.chunk_id,
        speech_samples.len(),
        energy
    );

    // Transcribe using the appropriate engine (with improved error handling)
    match engine {
        TranscriptionEngine::Whisper(whisper_engine) => {
            // Get language preference from global state
            let language = crate::get_language_preference_internal();

            match whisper_engine
                .transcribe_audio_with_confidence(speech_samples, language)
                .await
            {
                Ok((text, confidence, is_partial)) => {
                    let cleaned_text = text.trim().to_string();
                    if cleaned_text.is_empty() {
                        return Ok((String::new(), Some(confidence), is_partial));
                    }

                    info!(
                        "Whisper transcription complete for chunk {}: '{}' (confidence: {:.2}, partial: {})",
                        chunk.chunk_id, cleaned_text, confidence, is_partial
                    );

                    Ok((cleaned_text, Some(confidence), is_partial))
                }
                Err(e) => {
                    error!(
                        "Whisper transcription failed for chunk {}: {}",
                        chunk.chunk_id, e
                    );

                    let transcription_error = TranscriptionError::EngineFailed(e.to_string());
                    let _ = app.emit(
                        "transcription-error",
                        &serde_json::json!({
                            "error": transcription_error.to_string(),
                            "userMessage": format!("Transcription failed: {}", transcription_error),
                            "actionable": false,
                            "phase": "active"
                        }),
                    );

                    Err(transcription_error)
                }
            }
        }
        TranscriptionEngine::Parakeet(parakeet_engine) => {
            match parakeet_engine.transcribe_audio(speech_samples).await {
                Ok(text) => {
                    let cleaned_text = text.trim().to_string();
                    if cleaned_text.is_empty() {
                        return Ok((String::new(), None, false));
                    }

                    info!(
                        "Parakeet transcription complete for chunk {}: '{}'",
                        chunk.chunk_id, cleaned_text
                    );

                    // Parakeet doesn't provide confidence or partial results
                    Ok((cleaned_text, None, false))
                }
                Err(e) => {
                    error!(
                        "Parakeet transcription failed for chunk {}: {}",
                        chunk.chunk_id, e
                    );

                    let transcription_error = TranscriptionError::EngineFailed(e.to_string());
                    let _ = app.emit(
                        "transcription-error",
                        &serde_json::json!({
                            "error": transcription_error.to_string(),
                            "userMessage": format!("Transcription failed: {}", transcription_error),
                            "actionable": false,
                            "phase": "active"
                        }),
                    );

                    Err(transcription_error)
                }
            }
        }
        TranscriptionEngine::Provider(provider) => {
            // NEW: Trait-based provider (clean, unified interface)
            let language = crate::get_language_preference_internal();

            match provider.transcribe(speech_samples, language).await {
                Ok(result) => {
                    let cleaned_text = result.text.trim().to_string();
                    if cleaned_text.is_empty() {
                        return Ok((String::new(), result.confidence, result.is_partial));
                    }

                    let confidence_str = match result.confidence {
                        Some(c) => format!("confidence: {:.2}", c),
                        None => "no confidence".to_string(),
                    };

                    info!(
                        "{} transcription complete for chunk {}: '{}' ({}, partial: {})",
                        provider.provider_name(),
                        chunk.chunk_id,
                        cleaned_text,
                        confidence_str,
                        result.is_partial
                    );

                    Ok((cleaned_text, result.confidence, result.is_partial))
                }
                Err(e) => {
                    error!(
                        "{} transcription failed for chunk {}: {}",
                        provider.provider_name(),
                        chunk.chunk_id,
                        e
                    );

                    let _ = app.emit(
                        "transcription-error",
                        &serde_json::json!({
                            "error": e.to_string(),
                            "userMessage": format!("Transcription failed: {}", e),
                            "actionable": false,
                            "phase": "active"
                        }),
                    );

                    Err(e)
                }
            }
        }
    }
}

/// Format current timestamp (wall-clock time)
pub(super) fn format_current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let hours = (now.as_secs() / 3600) % 24;
    let minutes = (now.as_secs() / 60) % 60;
    let seconds = now.as_secs() % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Format recording-relative time as [MM:SS]
#[allow(dead_code)]
fn format_recording_time(seconds: f64) -> String {
    let total_seconds = seconds.floor() as u64;
    let minutes = total_seconds / 60;
    let secs = total_seconds % 60;

    format!("[{:02}:{:02}]", minutes, secs)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn keeps_short_acknowledgements() {
            assert!(should_emit_transcript("Yes"));
            assert!(should_emit_transcript("ok"));
        }

        #[test]
        fn drops_empty_and_whitespace_only() {
            assert!(!should_emit_transcript(""));
            assert!(!should_emit_transcript("   "));
        }
    }
