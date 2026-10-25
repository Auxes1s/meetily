//! Disk-backed batch audio. Decoding, normalization, sinc resampling and speech
//! discovery retain bounded PCM windows; only segment descriptors grow with time.
use super::{
    audio_processing::{audio_to_mono, resample},
    decoder::decode_packets,
    vad::{configured_session, SpeechSegment},
};
use anyhow::{anyhow, ensure, Result};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

const RATE: usize = 16_000;
const FRAME: usize = 480;
const MAX_SEGMENT: usize = 25 * RATE;

#[derive(Clone)]
pub(super) struct BatchSegment {
    audio: Arc<tempfile::NamedTempFile>,
    start: usize,
    pub sample_count: usize,
    pub start_timestamp_ms: f64,
    pub end_timestamp_ms: f64,
    confidence: f32,
}
impl BatchSegment {
    pub fn load(&self) -> Result<SpeechSegment> {
        let mut file = File::open(self.audio.path())?;
        Ok(SpeechSegment {
            samples: read_pcm(&mut file, self.start, self.sample_count)?,
            start_timestamp_ms: self.start_timestamp_ms,
            end_timestamp_ms: self.end_timestamp_ms,
            confidence: self.confidence,
            is_final: true,
        })
    }
}
pub(super) struct PreparedBatch {
    pub duration_seconds: f64,
    pub segments: Vec<BatchSegment>,
}
fn cancelled(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::SeqCst), "Audio processing cancelled");
    Ok(())
}
fn read_pcm(file: &mut File, start: usize, count: usize) -> Result<Vec<f32>> {
    file.seek(SeekFrom::Start(
        (start as u64)
            .checked_mul(4)
            .ok_or_else(|| anyhow!("Invalid audio offset"))?,
    ))?;
    let mut bytes = vec![
        0;
        count
            .checked_mul(4)
            .ok_or_else(|| anyhow!("Invalid audio length"))?
    ];
    file.read_exact(&mut bytes)?;
    Ok(bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect())
}
fn write_pcm(file: &mut File, samples: &[f32], clamp: bool) -> Result<()> {
    let mut bytes = Vec::with_capacity(64 * 1024);
    for chunk in samples.chunks(16 * 1024) {
        bytes.clear();
        for &sample in chunk {
            bytes.extend_from_slice(
                &if clamp { sample.clamp(-1., 1.) } else { sample }.to_le_bytes(),
            );
        }
        file.write_all(&bytes)?;
    }
    Ok(())
}
fn normalize(samples: &mut [f32], peak: f32) {
    let scale = if peak > 1. { 1. / peak } else { 1. };
    for sample in samples {
        *sample = if sample.is_finite() {
            (*sample * scale).clamp(-1., 1.)
        } else {
            0.
        };
    }
}

/// Match the former short-file sinc operation and long-file 60s/100ms
/// cross-fades, but write completed output rather than collecting every chunk.
fn convert(
    raw: &mut File,
    output: &mut File,
    samples: usize,
    rate: u32,
    peak: f32,
    cancel: &AtomicBool,
    progress: &mut impl FnMut(&str, u32, usize),
) -> Result<()> {
    ensure!(rate > 0, "Invalid decoded sample rate");
    if rate == RATE as u32 {
        for start in (0..samples).step_by(RATE * 10) {
            cancelled(cancel)?;
            let mut pcm = read_pcm(raw, start, (samples - start).min(RATE * 10))?;
            normalize(&mut pcm, peak);
            write_pcm(output, &pcm, false)?;
        }
        progress("resampling", 100, 0);
        return Ok(());
    }
    // Retain the original short-file result while bounding the allocation for
    // unusual source rates as well as ordinary 44.1/48kHz meeting recordings.
    let output_samples = samples as f64 * RATE as f64 / rate as f64;
    if samples <= 14_400_000
        && (samples as f64 * 3. + output_samples * 2.) * 4. <= 192. * 1024. * 1024.
    {
        cancelled(cancel)?;
        let mut pcm = read_pcm(raw, 0, samples)?;
        normalize(&mut pcm, peak);
        let converted = resample(&pcm, rate, RATE as u32)?;
        cancelled(cancel)?;
        write_pcm(output, &converted, true)?;
        progress("resampling", 100, 0);
        return Ok(());
    }
    let size = (rate as usize * 60).min(14_400_000);
    let overlap = rate as usize / 10;
    let output_overlap = RATE / 10;
    let mut tail = Vec::<f32>::new();
    for start in (0..samples).step_by(size) {
        cancelled(cancel)?;
        let mut pcm = read_pcm(raw, start, (samples - start).min(size + overlap))?;
        normalize(&mut pcm, peak);
        let converted = resample(&pcm, rate, RATE as u32)?;
        let fade = tail.len().min(converted.len()).min(output_overlap);
        let tail_start = tail.len() - fade;
        for i in 0..fade {
            let t = i as f32 / fade as f32;
            tail[tail_start + i] = tail[tail_start + i] * (1. - t) + converted[i] * t;
        }
        tail.extend_from_slice(&converted[fade..]);
        let keep = output_overlap.min(tail.len());
        let ready = tail.len() - keep;
        write_pcm(output, &tail[..ready], true)?;
        tail.drain(..ready);
        progress(
            "resampling",
            (((start + pcm.len()).min(samples) * 100) / samples.max(1)) as u32,
            0,
        );
    }
    write_pcm(output, &tail, true)
}

#[derive(Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
    confidence: f32,
}

/// The pinned Silero process() owns an entire active utterance. Use its public
/// recurrent forward() with the same configured thresholds/timing to discover
/// ranges without duplicating PCM. Differential tests lock this batch policy to
/// ContinuousVadProcessor; live recording continues through its existing path.
struct Detector {
    session: silero_rs::VadSession,
    processed: usize,
    silence: usize,
    active: Option<(usize, Duration, bool)>,
}
impl Detector {
    fn new(redemption: u32) -> Result<Self> {
        Ok(Self {
            session: configured_session(RATE as u32, redemption)?,
            processed: 0,
            silence: 0,
            active: None,
        })
    }
    fn frame(&mut self, pcm: &[f32]) -> Result<Option<Span>> {
        let result = self
            .session
            .forward(pcm.to_vec())
            .map_err(|e| anyhow!(e.to_string()))?;
        let probability = *result
            .try_extract_array::<f32>()?
            .first()
            .ok_or_else(|| anyhow!("Empty VAD probability"))?;
        let config = self.session.config_mut();
        if probability < config.negative_speech_threshold {
            self.silence += pcm.len();
        } else {
            self.silence = 0;
        }
        let mut ended = None;
        match self.active {
            None if probability > config.positive_speech_threshold => {
                let start_ms = Duration::from_secs_f64(self.processed as f64 / RATE as f64)
                    .saturating_sub(config.pre_speech_pad)
                    .as_millis() as usize;
                self.active = Some((start_ms * 16, Duration::ZERO, false));
            }
            Some((start, elapsed, mut confirmed)) => {
                let elapsed = elapsed + Duration::from_secs_f64(pcm.len() as f64 / RATE as f64);
                if !confirmed && elapsed > config.min_speech_time {
                    confirmed = true;
                }
                self.active = Some((start, elapsed, confirmed));
                if probability < config.negative_speech_threshold {
                    if !confirmed {
                        self.active = None;
                    } else if Duration::from_secs_f64(self.silence as f64 / RATE as f64)
                        > config.redemption_time
                    {
                        let end_ms = (self.processed + pcm.len() - self.silence) / 16
                            + config.post_speech_pad.as_millis() as usize;
                        ended = Some(Span {
                            start,
                            end: end_ms * 16,
                            confidence: 0.9,
                        });
                        self.active = None;
                    }
                }
            }
            _ => {}
        }
        self.processed += pcm.len();
        Ok(ended)
    }
    fn finish(&self, real_end: usize) -> Option<Span> {
        self.active
            .filter(|(_, _, confirmed)| *confirmed)
            .map(|(start, _, _)| Span {
                start,
                end: real_end,
                confidence: 0.8,
            })
    }
}

/// Preserve split_segment_at_silence's range search and advancement exactly,
/// reading only its +/-3s search window and loading each final payload on demand.
fn split(
    file: &mut File,
    span: Span,
    audio: &Arc<tempfile::NamedTempFile>,
    output: &mut Vec<BatchSegment>,
) -> Result<()> {
    ensure!(span.end >= span.start, "Invalid speech interval");
    let total = span.end - span.start;
    let mut pos = 0;
    while pos < total {
        let remaining = total - pos;
        let end = if remaining <= MAX_SEGMENT {
            total
        } else {
            let target = pos + MAX_SEGMENT;
            let search_start = target.saturating_sub(3 * RATE).max(pos + RATE);
            let search_end = (target + 3 * RATE).min(total.saturating_sub(RATE / 10));
            let mut best = target.min(total);
            let mut best_rms = f32::MAX;
            if search_start + RATE / 10 <= search_end {
                let pcm = read_pcm(file, span.start + search_start, search_end - search_start)?;
                let mut index = 0;
                while index + RATE / 10 <= pcm.len() {
                    let rms = (pcm[index..index + RATE / 10]
                        .iter()
                        .map(|s| s * s)
                        .sum::<f32>()
                        / (RATE / 10) as f32)
                        .sqrt();
                    if rms < best_rms {
                        best_rms = rms;
                        best = search_start + index + RATE / 20;
                    }
                    index += RATE / 100;
                }
            }
            if best_rms > 0.02 {
                (best + RATE).min(total)
            } else {
                best
            }
        };
        ensure!(end > pos, "Speech splitting did not advance");
        output.push(BatchSegment {
            audio: audio.clone(),
            start: span.start + pos,
            sample_count: end - pos,
            start_timestamp_ms: (span.start + pos) as f64 / 16.,
            end_timestamp_ms: (span.start + end) as f64 / 16.,
            confidence: span.confidence,
        });
        pos = end;
    }
    Ok(())
}

pub(super) fn prepare(
    path: &Path,
    redemption: u32,
    cancel: &AtomicBool,
    mut progress: impl FnMut(&str, u32, usize),
) -> Result<PreparedBatch> {
    cancelled(cancel)?;
    progress("decoding", 0, 0);
    let mut raw = tempfile::NamedTempFile::new()?;
    let mut peak = 0f32;
    let mut decoded_percent = 0;
    let info = decode_packets(path, None, Some(cancel), |packet, _, channels, percent| {
        if percent > decoded_percent {
            progress("decoding", percent, 0);
            decoded_percent = percent;
        }
        cancelled(cancel)?;
        let mono = if channels > 1 {
            audio_to_mono(packet, channels)
        } else {
            packet.to_vec()
        };
        for sample in &mono {
            if sample.is_finite() {
                peak = peak.max(sample.abs());
            }
        }
        write_pcm(raw.as_file_mut(), &mono, false)
    })?;
    progress("decoding", 100, 0);
    let samples = (raw.as_file().metadata()?.len() / 4) as usize;
    let mut normalized = tempfile::NamedTempFile::new()?;
    convert(
        raw.as_file_mut(),
        normalized.as_file_mut(),
        samples,
        info.sample_rate,
        peak,
        cancel,
        &mut progress,
    )?;
    drop(raw);
    let audio = Arc::new(normalized);
    let total = (audio.as_file().metadata()?.len() / 4) as usize;
    let mut file = File::open(audio.path())?;
    let mut detector = Detector::new(redemption)?;
    let mut segments = Vec::new();
    let mut last_progress = 0;
    for start in (0..total).step_by(FRAME) {
        cancelled(cancel)?;
        let mut pcm = read_pcm(&mut file, start, (total - start).min(FRAME))?;
        pcm.resize(FRAME, 0.);
        if let Some(span) = detector.frame(&pcm)? {
            split(&mut file, span, &audio, &mut segments)?;
        }
        let current = (((start + FRAME).min(total) * 100) / total.max(1)) as u32;
        if current > last_progress {
            progress("vad", current, segments.len());
            last_progress = current;
        }
    }
    if let Some(span) = detector.finish(total) {
        split(&mut file, span, &audio, &mut segments)?;
    }
    progress("vad", 100, segments.len());
    Ok(PreparedBatch {
        duration_seconds: info.duration_seconds,
        segments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "45-minute synthetic duration/resource benchmark; no physical recording or ASR model"]
    fn long_silent_batch_resource_benchmark() {
        let mut source = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        let rate = 48_000u32;
        let bytes = 2700 * rate * 4;
        let file = source.as_file_mut();
        file.write_all(b"RIFF").unwrap();
        file.write_all(&(bytes + 36).to_le_bytes()).unwrap();
        file.write_all(b"WAVEfmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap();
        file.write_all(&3u16.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap();
        file.write_all(&rate.to_le_bytes()).unwrap();
        file.write_all(&(rate * 4).to_le_bytes()).unwrap();
        file.write_all(&4u16.to_le_bytes()).unwrap();
        file.write_all(&32u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&bytes.to_le_bytes()).unwrap();
        file.set_len(44 + bytes as u64).unwrap();
        let start = std::time::Instant::now();
        let prepared = prepare(source.path(), 2000, &AtomicBool::new(false), |_, _, _| {}).unwrap();
        assert_eq!(prepared.duration_seconds, 2700.);
        assert!(prepared.segments.is_empty());
        println!(
            "45-minute synthetic batch completed in {:.3}s",
            start.elapsed().as_secs_f64()
        );
    }
    #[test]
    fn disk_batch_matches_existing_decoder_vad_and_silence_splits() {
        for name in ["audio-source-speech.wav", "he_aac_48k_5s.m4a"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name);
            let decoded = super::super::decoder::decode_audio_file(&path).unwrap();
            let mono = decoded.to_whisper_format();
            let old = super::super::vad::get_speech_chunks(&mono, 2000).unwrap();
            let expected: Vec<_> = old
                .iter()
                .flat_map(|segment| {
                    super::super::common::split_segment_at_silence(segment, MAX_SEGMENT)
                })
                .collect();
            let batch = prepare(&path, 2000, &AtomicBool::new(false), |_, _, _| {}).unwrap();
            assert_eq!(batch.duration_seconds, decoded.duration_seconds);
            assert_eq!(batch.segments.len(), expected.len());
            for (descriptor, expected) in batch.segments.iter().zip(&expected) {
                let actual = descriptor.load().unwrap();
                assert_eq!(actual.start_timestamp_ms, expected.start_timestamp_ms);
                assert_eq!(actual.end_timestamp_ms, expected.end_timestamp_ms);
                assert_eq!(actual.samples, expected.samples);
            }
        }
    }
    #[test]
    fn disk_split_preserves_long_utterance_boundaries_and_pcm() {
        for silence in [false, true] {
            let pcm: Vec<f32> = (0..90 * RATE)
                .map(|i| {
                    if silence && i % (25 * RATE) < RATE {
                        0.
                    } else {
                        0.2
                    }
                })
                .collect();
            let mut file = tempfile::NamedTempFile::new().unwrap();
            write_pcm(file.as_file_mut(), &pcm, false).unwrap();
            let file = Arc::new(file);
            let old = SpeechSegment {
                samples: pcm,
                start_timestamp_ms: 0.,
                end_timestamp_ms: 90_000.,
                confidence: 0.9,
                is_final: true,
            };
            let expected = super::super::common::split_segment_at_silence(&old, MAX_SEGMENT);
            let mut actual = Vec::new();
            split(
                &mut File::open(file.path()).unwrap(),
                Span {
                    start: 0,
                    end: old.samples.len(),
                    confidence: 0.9,
                },
                &file,
                &mut actual,
            )
            .unwrap();
            for (a, e) in actual.iter().zip(&expected) {
                let a = a.load().unwrap();
                assert_eq!(a.start_timestamp_ms, e.start_timestamp_ms);
                assert_eq!(a.end_timestamp_ms, e.end_timestamp_ms);
                assert_eq!(a.samples, e.samples);
            }
            assert_eq!(actual.len(), expected.len());
        }
    }
    #[test]
    fn cancellation_prevents_decode_and_leaves_source_untouched() {
        let source = tempfile::NamedTempFile::new().unwrap();
        assert!(prepare(source.path(), 2000, &AtomicBool::new(true), |_, _, _| {}).is_err());
        assert!(source.path().exists());
    }
}
