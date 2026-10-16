use super::source::AudioSource;
use super::vad::{ContinuousVadProcessor, SpeechSegment};
use anyhow::Result;

pub struct SourceSpeechSegment {
    pub source: AudioSource,
    pub speech: SpeechSegment,
}

/// Both inputs consume the same aligned windows, including silence, so neither
/// channel's speech activity can advance or compress the other channel's clock.
pub struct SourceVadProcessor {
    microphone: ContinuousVadProcessor,
    system: ContinuousVadProcessor,
}

impl SourceVadProcessor {
    pub fn seal_stream(&mut self) -> Vec<SourceSpeechSegment> {
        tag_segments(self.microphone.seal_stream(), self.system.seal_stream())
    }
    pub fn new_streaming(sample_rate: u32, redemption_time_ms: u32) -> Result<Self> {
        Ok(Self {
            microphone: ContinuousVadProcessor::new_streaming(sample_rate, redemption_time_ms)?,
            system: ContinuousVadProcessor::new_streaming(sample_rate, redemption_time_ms)?,
        })
    }
    pub fn new(sample_rate: u32, redemption_time_ms: u32) -> Result<Self> {
        Ok(Self {
            microphone: ContinuousVadProcessor::new_live(sample_rate, redemption_time_ms)?,
            system: ContinuousVadProcessor::new_live(sample_rate, redemption_time_ms)?,
        })
    }

    pub fn process_audio(
        &mut self,
        microphone: &[f32],
        system: &[f32],
    ) -> Result<Vec<SourceSpeechSegment>> {
        anyhow::ensure!(
            microphone.len() == system.len(),
            "Audio source windows must share a clock"
        );
        // Process both even if one fails; the recording path remains independent.
        let mic = self.microphone.process_audio(microphone);
        let sys = self.system.process_audio(system);
        Ok(tag_segments(
            retain_channel(mic, "microphone"),
            retain_channel(sys, "system"),
        ))
    }

    pub fn flush(&mut self) -> Result<Vec<SourceSpeechSegment>> {
        let mic = self.microphone.flush();
        let sys = self.system.flush();
        Ok(tag_segments(
            retain_channel(mic, "microphone"),
            retain_channel(sys, "system"),
        ))
    }
}

fn retain_channel(result: Result<Vec<SpeechSegment>>, channel: &str) -> Vec<SpeechSegment> {
    result.unwrap_or_else(|error| {
        log::warn!("{} VAD failed: {}", channel, error);
        Vec::new()
    })
}

fn tag_segments(
    microphone: Vec<SpeechSegment>,
    system: Vec<SpeechSegment>,
) -> Vec<SourceSpeechSegment> {
    let mut segments: Vec<_> = microphone
        .into_iter()
        .map(|speech| SourceSpeechSegment {
            source: AudioSource::Microphone,
            speech,
        })
        .chain(system.into_iter().map(|speech| SourceSpeechSegment {
            source: AudioSource::System,
            speech,
        }))
        .collect();
    segments.sort_by(|a, b| {
        a.speech
            .start_timestamp_ms
            .total_cmp(&b.speech.start_timestamp_ms)
    });
    segments
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn speech(start: f64, marker: f32) -> SpeechSegment {
        SpeechSegment {
            samples: vec![marker; 800],
            start_timestamp_ms: start,
            end_timestamp_ms: start + 50.0,
            confidence: 1.0,
            is_final: true,
        }
    }

    #[test]
    fn overlapping_sources_keep_audio_and_timing_separate() {
        let segments = tag_segments(
            vec![speech(100.0, 1.0), speech(300.0, 3.0)],
            vec![speech(100.0, 2.0), speech(200.0, 4.0)],
        );
        assert_eq!(
            segments.iter().map(|s| s.source).collect::<Vec<_>>(),
            vec![
                AudioSource::Microphone,
                AudioSource::System,
                AudioSource::System,
                AudioSource::Microphone
            ]
        );
        assert_eq!(
            segments
                .iter()
                .map(|s| s.speech.samples[0])
                .collect::<Vec<_>>(),
            vec![1.0, 2.0, 4.0, 3.0]
        );
        assert_eq!(
            segments[0].speech.start_timestamp_ms,
            segments[1].speech.start_timestamp_ms
        );
        assert!(tag_segments(vec![], vec![]).is_empty());
    }

    pub(crate) fn fixture_speech(seconds: f32, sample_rate: u32) -> Vec<f32> {
        assert_eq!(sample_rate, 16000);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio-source-speech.wav");
        let decoded = super::super::decoder::decode_audio_file(&path).unwrap();
        assert_eq!(decoded.sample_rate, sample_rate);
        decoded
            .samples
            .into_iter()
            .take((seconds * sample_rate as f32) as usize)
            .collect()
    }

    #[test]
    fn source_vad_routes_single_alternating_and_overlapping_inputs_and_flushes_once() {
        let speech = fixture_speech(3.0, 16000);
        let silence = vec![0.0; speech.len()];
        for (case, (mic, sys, expected)) in [
            (
                speech.clone(),
                silence.clone(),
                vec![AudioSource::Microphone],
            ),
            (silence.clone(), speech.clone(), vec![AudioSource::System]),
            (
                speech.clone(),
                speech.clone(),
                vec![AudioSource::Microphone, AudioSource::System],
            ),
            (
                [speech.clone(), silence.clone()].concat(),
                [silence.clone(), speech.clone()].concat(),
                vec![AudioSource::Microphone, AudioSource::System],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let mut vad = SourceVadProcessor::new(16000, 500).unwrap();
            let mut segments = Vec::new();
            // Odd-sized input windows exercise buffered final frames.
            for (mic_window, sys_window) in mic.chunks(997).zip(sys.chunks(997)) {
                segments.extend(vad.process_audio(mic_window, sys_window).unwrap());
            }
            segments.extend(vad.flush().unwrap());
            for source in expected {
                assert!(
                    segments.iter().any(|s| s.source == source),
                    "case {} missing {:?}; segments {:?}",
                    case,
                    source,
                    segments
                        .iter()
                        .map(|s| (
                            s.source,
                            s.speech.start_timestamp_ms,
                            s.speech.end_timestamp_ms
                        ))
                        .collect::<Vec<_>>()
                );
            }
            if mic == silence {
                assert!(segments.iter().all(|s| s.source == AudioSource::System));
            }
            if sys == silence {
                assert!(segments.iter().all(|s| s.source == AudioSource::Microphone));
            }
            let end = mic.len() as f64 / 16.0;
            assert!(segments
                .iter()
                .all(|s| s.speech.start_timestamp_ms >= 0.0 && s.speech.end_timestamp_ms <= end));
            assert!(
                vad.flush().unwrap().is_empty(),
                "repeated flush emitted duplicates"
            );
        }
        let mut vad = SourceVadProcessor::new(16000, 500).unwrap();
        assert!(vad.process_audio(&silence, &silence).unwrap().is_empty());
        assert!(vad.flush().unwrap().is_empty());
        assert!(vad.process_audio(&[0.0], &[]).is_err());
    }

    #[test]
    fn failure_in_one_channel_does_not_discard_the_other() {
        assert!(retain_channel(Err(anyhow::anyhow!("fixture failure")), "system").is_empty());
        let segments = tag_segments(
            retain_channel(Ok(vec![speech(100.0, 1.0)]), "microphone"),
            vec![],
        );
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].source, AudioSource::Microphone);
    }

    #[tokio::test]
    #[ignore = "Requires installed Whisper/Parakeet models and MEETILY_ASR_TEST_MODEL_DIR"]
    async fn live_short_segments_existing_local_providers() {
        use crate::audio::transcription::provider::TranscriptionProvider;
        use crate::audio::transcription::{parakeet_provider::ParakeetProvider, whisper_provider::WhisperProvider};
        use std::sync::Arc;
        let directory = std::path::PathBuf::from(
            std::env::var("MEETILY_ASR_TEST_MODEL_DIR").expect("set the installed model directory"),
        );
        let speech: Vec<_> = fixture_speech(3.0, 16000).into_iter().cycle().take(12 * 16000).collect();
        let mut vad = SourceVadProcessor::new(16000, 500).unwrap();
        let mut passages = Vec::new();
        for window in speech.chunks(9600) {
            passages.extend(vad.process_audio(window, &vec![0.0; window.len()]).unwrap());
        }
        assert!(passages.len() >= 2, "must deliver speech while capture is open");
        passages.extend(vad.flush().unwrap());
        assert!(passages.iter().all(|passage| passage.source == AudioSource::Microphone && passage.speech.samples.len() <= 80_000));

        async fn verify(provider: &dyn TranscriptionProvider, passages: &[SourceSpeechSegment]) {
            for passage in passages {
                let started = std::time::Instant::now();
                let result = provider.transcribe(passage.speech.samples.clone(), Some("en".into())).await.unwrap();
                assert!(!result.text.trim().is_empty(), "{} returned empty speech", provider.provider_name());
                assert!(!result.is_partial, "completed short chunk must be final");
                eprintln!("{} live {:.3}s chunk: {:.3}s decode, {}",
                    provider.provider_name(), passage.speech.samples.len() as f64 / 16000.0,
                    started.elapsed().as_secs_f64(), result.text);
            }
        }
        let whisper = Arc::new(crate::whisper_engine::WhisperEngine::new_with_models_dir(Some(directory.clone())).unwrap());
        whisper.discover_models().await.unwrap();
        whisper.load_model("large-v3").await.unwrap();
        verify(&WhisperProvider::new(whisper.clone()), &passages).await;
        whisper.unload_model().await;
        let parakeet = Arc::new(crate::parakeet_engine::ParakeetEngine::new_with_models_dir(Some(directory)).unwrap());
        parakeet.discover_models().await.unwrap();
        parakeet.load_model("parakeet-tdt-0.6b-v3-int8").await.unwrap();
        verify(&ParakeetProvider::new(parakeet.clone()), &passages).await;
        parakeet.unload_model().await;
    }

    #[tokio::test]
    #[ignore = "Requires installed Whisper/Parakeet models and MEETILY_ASR_TEST_MODEL_DIR"]
    async fn source_existing_local_providers_smoke() {
        use crate::audio::transcription::provider::TranscriptionProvider;
        use crate::audio::transcription::{
            parakeet_provider::ParakeetProvider, whisper_provider::WhisperProvider,
        };
        use std::sync::Arc;
        let directory = std::path::PathBuf::from(
            std::env::var("MEETILY_ASR_TEST_MODEL_DIR").expect("set the installed model directory"),
        );
        let speech = fixture_speech(3.0, 16000);
        let mut vad = SourceVadProcessor::new(16000, 500).unwrap();
        let mut passages = vad.process_audio(&speech, &speech).unwrap();
        passages.extend(vad.flush().unwrap());
        assert!(passages.iter().any(|s| s.source == AudioSource::Microphone));
        assert!(passages.iter().any(|s| s.source == AudioSource::System));
        async fn verify(provider: &dyn TranscriptionProvider, passages: &[SourceSpeechSegment]) {
            for passage in passages {
                let result = provider
                    .transcribe(passage.speech.samples.clone(), Some("en".into()))
                    .await
                    .unwrap();
                assert!(
                    !result.text.trim().is_empty(),
                    "{} emitted empty fixture text",
                    provider.provider_name()
                );
                eprintln!(
                    "{} {:?}: {}",
                    provider.provider_name(),
                    passage.source,
                    result.text
                );
            }
        }
        let whisper = Arc::new(
            crate::whisper_engine::WhisperEngine::new_with_models_dir(Some(directory.clone()))
                .unwrap(),
        );
        whisper.discover_models().await.unwrap();
        whisper.load_model("large-v3-turbo").await.unwrap();
        verify(&WhisperProvider::new(whisper.clone()), &passages).await;
        whisper.unload_model().await;
        let parakeet = Arc::new(
            crate::parakeet_engine::ParakeetEngine::new_with_models_dir(Some(directory)).unwrap(),
        );
        parakeet.discover_models().await.unwrap();
        parakeet
            .load_model("parakeet-tdt-0.6b-v3-int8")
            .await
            .unwrap();
        verify(&ParakeetProvider::new(parakeet.clone()), &passages).await;
        parakeet.unload_model().await;
    }
    #[test]
    fn streaming_frames_preserve_all_speech_and_emit_natural_endpoints_once() {
        let mut audio=fixture_speech(14.0,16_000);audio.extend(vec![0.0;16_000]);
        let mut batch=ContinuousVadProcessor::new(16_000,500).unwrap();
        let mut expected=batch.process_audio(&audio).unwrap();expected.extend(batch.flush().unwrap());
        let mut live=ContinuousVadProcessor::new_streaming(16_000,500).unwrap();let mut frames=Vec::new();
        for window in audio.chunks(997) {frames.extend(live.process_audio(window).unwrap());}
        frames.extend(live.flush().unwrap());
        assert_eq!(frames.iter().flat_map(|s|s.samples.iter()).copied().collect::<Vec<_>>(),
            expected.iter().flat_map(|s|s.samples.iter()).copied().collect::<Vec<_>>());
        assert!(frames.iter().any(|s|!s.is_final));
        assert_eq!(frames.iter().filter(|s|s.is_final).count(),expected.len());
        assert!(frames.iter().all(|s|s.samples.len()<=32_000));
        assert!(live.flush().unwrap().is_empty());
    }
    #[test]
    fn pause_seals_the_tail_once_and_resume_keeps_detector_audio_and_clock() {
        let mut audio=fixture_speech(14.0,16_000);audio.extend(vec![0.0;16_000]);
        let mut uninterrupted=ContinuousVadProcessor::new_streaming(16_000,500).unwrap();
        let mut expected=uninterrupted.process_audio(&audio).unwrap();expected.extend(uninterrupted.flush().unwrap());
        let mut paused=ContinuousVadProcessor::new_streaming(16_000,500).unwrap();
        let split=16_000*3;
        let mut actual=paused.process_audio(&audio[..split]).unwrap();
        let boundary=paused.seal_stream();assert!(!boundary.is_empty());assert!(boundary.iter().all(|s|s.is_final));
        actual.extend(boundary);assert!(paused.seal_stream().is_empty());
        actual.extend(paused.process_audio(&audio[split..]).unwrap());actual.extend(paused.flush().unwrap());
        assert_eq!(actual.iter().flat_map(|s|s.samples.iter()).copied().collect::<Vec<_>>(),expected.iter().flat_map(|s|s.samples.iter()).copied().collect::<Vec<_>>());
        assert!(actual.iter().all(|s|s.end_timestamp_ms<=audio.len() as f64/16.0));
        assert!(paused.flush().unwrap().is_empty());
    }
}
