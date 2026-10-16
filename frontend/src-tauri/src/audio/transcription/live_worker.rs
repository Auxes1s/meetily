//! One blocking decoder and disk-backed pending source audio. The async intake
//! keeps inference off Tokio/capture threads; superseded previews can be skipped.
use super::{
    streaming::{LiveBuffer, LiveDecoder},
    worker::TranscriptUpdate,
};
use crate::audio::{recording_state::TranscriptionChunk, source::AudioSource};
use anyhow::{anyhow, Result};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Runtime};

struct SourceState {
    buffer: LiveBuffer,
    admission: super::scheduling::PreviewAdmission,
    sequence: Option<u64>,
    revision: u64,
}
impl Default for SourceState {
    fn default() -> Self {
        Self {
            buffer: LiveBuffer::default(),
            admission: super::scheduling::PreviewAdmission::default(),
            sequence: None,
            revision: 0,
        }
    }
}
fn source_index(source: AudioSource) -> usize {
    match source {
        AudioSource::Microphone => 0,
        AudioSource::System => 1,
        AudioSource::Mixed => 2,
        AudioSource::Unknown => 3,
    }
}
fn emit<R: Runtime>(
    app: &AppHandle<R>,
    identity: &(Option<String>, Option<String>),
    source: AudioSource,
    state: &mut SourceState,
    tokens: Vec<super::streaming::TimedToken>,
    final_decode: bool,
    endpoint: bool,
) -> Result<()> {
    let passages = state.buffer.accept_boundary(tokens, final_decode, endpoint);
    for passage in passages {
        if !passage.text.trim().is_empty() {
            super::worker::mark_live_speech_detected(app)?;
        }
        let sequence = *state
            .sequence
            .get_or_insert_with(super::worker::next_sequence);
        state.revision += 1;
        app.emit(
            "transcript-update",
            TranscriptUpdate {
                meeting_id: identity.0.clone(),
                recording_session_id: identity.1.clone(),
                text: passage.text,
                timestamp: super::worker::format_current_timestamp(),
                source: "Audio".into(),
                audio_source: Some(source),
                sequence_id: sequence,
                update_revision: state.revision,
                chunk_start_time: passage.start,
                is_partial: passage.is_partial,
                confidence: 0.85,
                audio_start_time: passage.start,
                audio_end_time: passage.end,
                duration: (passage.end - passage.start).max(0.0),
            },
        )?;
        if !passage.is_partial {
            state.sequence = None;
            state.revision = 0;
        }
    }
    // A final decode can legitimately retract a noise-only provisional result.
    if endpoint {
        if let Some(sequence) = state.sequence.take() {
            state.revision += 1;
            app.emit(
                "transcript-update",
                TranscriptUpdate {
                    meeting_id: identity.0.clone(),
                    recording_session_id: identity.1.clone(),
                    text: String::new(),
                    timestamp: super::worker::format_current_timestamp(),
                    source: "Audio".into(),
                    audio_source: Some(source),
                    sequence_id: sequence,
                    update_revision: state.revision,
                    chunk_start_time: 0.0,
                    is_partial: false,
                    confidence: 0.0,
                    audio_start_time: 0.0,
                    audio_end_time: 0.0,
                    duration: 0.0,
                },
            )?;
            state.revision = 0;
        }
    }
    Ok(())
}

pub async fn run<R: Runtime>(
    app: AppHandle<R>,
    mut input: super::spool::Inbox,
    identity: (Option<String>, Option<String>),
    decoder: LiveDecoder,
) -> Result<()> {
    let inbox_state = input.state.clone();
    let health = inbox_state.health.clone();
    let worker_sources = inbox_state.clone();
    let worker_app = app.clone();
    let worker_identity = identity.clone();
    let worker_health = health.clone();
    let worker = tokio::task::spawn_blocking(move || {
        let result = (|| -> Result<()> {
        let mut decoder = decoder;
        #[cfg(feature="q02-validation")]
        crate::q02_validation::record(serde_json::json!({"kind":"worker_ready","decoder_instances":1}));
        let mut sources: [SourceState; 4] = std::array::from_fn(|_| SourceState::default());
        let mut completed = 0_u64;
        let mut decodes = 0_u64;
        let mut decode_audio = Vec::with_capacity(super::streaming::WINDOW_SAMPLES);
        loop {
            #[cfg(feature = "q02-validation")]
            let wait_started = std::time::Instant::now();
            #[cfg(feature = "q02-validation")]
            let fresh_at_wait = sources.iter().any(|state| state.buffer.duration() > 0.
                && state.admission.has_fresh_audio(state.buffer.end()));
            #[cfg(feature = "q02-validation")]
            let queued_at_wait: u64 = worker_sources.by_source.iter().map(|count|count.load(Ordering::SeqCst)).sum();
            let next = input.blocking_recv()?;
            #[cfg(feature = "q02-validation")]
            crate::q02_validation::record(serde_json::json!({"kind":"worker_wait",
                "wait_seconds":wait_started.elapsed().as_secs_f64(),
                "idle_with_queued_audio_seconds":next.as_ref().map(|p|wait_started.max(p.enqueued_at).elapsed().as_secs_f64()).unwrap_or(0.),
                "fresh_buffered_audio_at_wait":fresh_at_wait,"queued_chunks_at_wait":queued_at_wait,
                "input_closed":next.is_none()}));
            let Some(pending) = next else { break; };
            let bytes = std::fs::read(&pending.path)?;
            if bytes.len() != pending.samples * 4 {
                return Err(anyhow!("Pending source audio size mismatch"));
            }
            let samples: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
                .collect();
            let source = pending.chunk.audio_source;
            let state = &mut sources[source_index(source)];
            if state.buffer.has_gap(pending.chunk.timestamp) {
                state.buffer.copy_samples(&mut decode_audio);
                let tokens = if state.buffer.unresolved_duration() > 0. { decoder.decode(&decode_audio, state.buffer.start())? } else { Vec::new() };
                emit(&worker_app, &worker_identity, source, state, tokens, true, true)?;
                state.admission.observed(pending.chunk.timestamp, true);
                worker_health.repairs.fetch_add(1, Ordering::SeqCst);
                worker_app.emit("transcription-warning", "Source clock gap: preceding speech finalized; original timing retained")?;
            }
            let mut position = 0;
            loop {
                let room = super::streaming::MAX_WINDOW_SAMPLES - state.buffer.len();
                let take = room.min(samples.len() - position);
                if take > 0 {
                    state.buffer.append(
                        &samples[position..position + take],
                        pending.chunk.timestamp + position as f64 / 16_000.0,
                    )?;
                    position += take;
                }
                let endpoint =
                    position == samples.len() && pending.chunk.stream_end.unwrap_or(true);
                let full = state.buffer.must_finalize();
                let same_source_pending = worker_sources.by_source[source_index(source)].load(Ordering::SeqCst)>1;
                let needs_decode = state.buffer.unresolved_duration() > 0. || state.sequence.is_some();
                #[cfg(feature="q02-validation")]
                let fresh_audio = state.admission.has_fresh_audio(state.buffer.end());
                if state.admission.eligible(state.buffer.end(),endpoint,full,same_source_pending)
                    && state.buffer.duration() > 0.0 && needs_decode {
                    let start = std::time::Instant::now();
                    state.buffer.copy_samples(&mut decode_audio);
                    let tokens = decoder.decode(&decode_audio, state.buffer.start())?;
                    #[cfg(feature="q02-validation")]
                    let elapsed=start.elapsed().as_secs_f64();
                    state.admission.observed(state.buffer.end(),endpoint);
                    #[cfg(feature="q02-validation")]
                    crate::q02_validation::record(serde_json::json!({"kind":"decode","source":source,
                        "window_seconds":decode_audio.len() as f64/16_000.,"audio_start":state.buffer.start(),
                        "audio_end":state.buffer.end(),
                        "fresh_source_audio":fresh_audio,"finalization_decode":endpoint || full,
                        "inference_seconds":elapsed,"queued_audio_seconds":worker_health.pending_samples.load(Ordering::SeqCst) as f64/16_000.,
                        "timed_tokens":tokens.iter().filter(|t|t.start.is_some()&&t.end.is_some()).count(),"tokens":tokens.len()}));
                    decodes += 1;
                    emit(
                        &worker_app,
                        &worker_identity,
                        source,
                        state,
                        tokens,
                        endpoint || full,
                        endpoint,
                    )?;
                    if full { state.buffer.ensure_window_progress()?; }
                    #[cfg(feature="q02-validation")]
                    crate::q02_validation::record(serde_json::json!({"kind":"window_state","source":source,
                        "retained_audio_seconds":state.buffer.duration(),"unfinalized_audio_seconds":state.buffer.unresolved_duration()}));
                    worker_app.emit("transcription-progress",serde_json::json!({"worker_id":0,"chunks_completed":completed,
                        "chunks_queued":worker_health.queued.load(Ordering::SeqCst),"inference_ms":start.elapsed().as_secs_f64()*1000.0,
                        "progress_percentage":completed*100/worker_health.queued.load(Ordering::SeqCst).max(1),"message":"Transcribing speech…",
                        "pending_audio_seconds":worker_health.pending_samples.load(Ordering::SeqCst) as f64/16_000.0,"decode_count":decodes,
                        "routing":"overlapping","failures":0}))?;
                } else if endpoint && !needs_decode {
                    // Only committed overlap remains. Close the source without
                    // spending another decode on already finalized audio.
                    state.buffer.accept_boundary(Vec::new(), true, true);
                    state.admission.observed(state.buffer.end(), true);
                }
                if position == samples.len() {
                    break;
                }
            }
            // Keep source audio until final SQLite acknowledgment, including
            // audio already decoded into a still-provisional hypothesis.
            worker_sources.complete(&pending);
            completed += 1;
        }
        for (index, state) in sources.iter_mut().enumerate() {
            if state.buffer.unresolved_duration() > 0. || state.sequence.is_some() {
                #[cfg(feature="q02-validation")]
                let started=std::time::Instant::now();
                state.buffer.copy_samples(&mut decode_audio);
                let tokens = decoder.decode(&decode_audio, state.buffer.start())?;
                #[cfg(feature="q02-validation")]
                crate::q02_validation::record(serde_json::json!({"kind":"decode","source":([AudioSource::Microphone,AudioSource::System,AudioSource::Mixed,AudioSource::Unknown][index]),
                    "window_seconds":decode_audio.len() as f64/16_000.,"audio_start":state.buffer.start(),"audio_end":state.buffer.end(),
                    "inference_seconds":started.elapsed().as_secs_f64(),"stop_flush":true}));
                emit(
                    &worker_app,
                    &worker_identity,
                    [
                        AudioSource::Microphone,
                        AudioSource::System,
                        AudioSource::Mixed,
                        AudioSource::Unknown,
                    ][index],
                    state,
                    tokens,
                    true,
                    true,
                )?;
            }
        }
        worker_app.emit("transcription-progress",serde_json::json!({"worker_id":0,"chunks_completed":completed,
            "chunks_queued":worker_health.queued.load(Ordering::SeqCst),
            "progress_percentage":if worker_health.error.lock().unwrap().is_some() {0} else {100},"pending_audio_seconds":0,
            "routing":"overlapping","failures":if worker_health.error.lock().unwrap().is_some() {1} else {0},
            "drained":!worker_health.error.lock().unwrap().is_some(),"message":"Transcription input finished"}))?;
        Ok(())
        })();
        if let Err(error) = &result { worker_sources.health.fail(error.to_string()); }
        (input, result)
    });
    let (mut input, result) = worker.await?;
    let writer = input.join_writer().await;
    result?;
    writer?;
    health.drained.store(true, Ordering::SeqCst);
    Ok(())
}
