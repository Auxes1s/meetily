use std::collections::VecDeque;

use anyhow::Result;
use sonora::config::EchoCanceller;
use sonora::{AudioProcessing, Config, StreamConfig};

/// One recording owns one adaptive echo path. Feed the unmodified speaker
/// reference before the microphone, before gain/noise processing changes it.
/// AEC3 estimates the acoustic delay and retains independent near-end speech.
pub(crate) struct AcousticEchoCanceller {
    processor: Option<AudioProcessing>,
    frame_size: usize,
    sample_rate: u32,
    microphone: VecDeque<f32>,
    system: VecDeque<f32>,
    reference_tail: usize,
}

impl AcousticEchoCanceller {
    pub fn new(sample_rate: u32, enabled: bool) -> Result<Self> {
        anyhow::ensure!(
            matches!(sample_rate, 16000 | 32000 | 48000),
            "Unsupported AEC sample rate"
        );
        let stream = StreamConfig::new(sample_rate, 1);
        let processor = enabled.then(|| {
            AudioProcessing::builder()
                .config(Config {
                    echo_canceller: Some(EchoCanceller {
                        // The pipeline applies its existing 80Hz filter after
                        // AEC; keep this stage linear against the raw reference.
                        enforce_high_pass_filtering: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .capture_config(stream)
                .render_config(stream)
                .build()
        });
        Ok(Self {
            processor,
            frame_size: sample_rate as usize / 100,
            sample_rate,
            microphone: VecDeque::new(),
            system: VecDeque::new(),
            reference_tail: 0,
        })
    }

    /// Buffer arbitrary callback/window sizes into the 10ms frames AEC3 needs.
    /// Both returned channels have exactly the same sample count and clock.
    pub fn process(&mut self, microphone: &[f32], system: &[f32]) -> Result<(Vec<f32>, Vec<f32>)> {
        anyhow::ensure!(
            microphone.len() == system.len(),
            "AEC inputs must share a clock"
        );
        self.microphone.extend(microphone);
        self.system.extend(system);
        self.drain(false)
    }

    pub fn flush(&mut self) -> Result<(Vec<f32>, Vec<f32>)> {
        self.drain(true)
    }

    fn drain(&mut self, final_tail: bool) -> Result<(Vec<f32>, Vec<f32>)> {
        let mut cleaned = Vec::with_capacity(self.microphone.len());
        let mut reference = Vec::with_capacity(self.system.len());
        while self.microphone.len() >= self.frame_size
            || (final_tail && !self.microphone.is_empty())
        {
            let length = self.microphone.len().min(self.frame_size);
            let mut mic: Vec<_> = self.microphone.drain(..length).collect();
            let mut sys: Vec<_> = self.system.drain(..length).collect();
            reference.extend_from_slice(&sys);
            mic.resize(self.frame_size, 0.0);
            sys.resize(self.frame_size, 0.0);
            if sys.iter().any(|sample| sample.abs() > 0.00001) {
                // Retain the acoustic tail after playback ends.
                self.reference_tail = self.sample_rate as usize / 2;
            } else {
                self.reference_tail = self.reference_tail.saturating_sub(self.frame_size);
            }
            let mut output = mic.clone();
            if let Some(processor) = self.processor.as_mut() {
                let mut render_output = vec![0.0; self.frame_size];
                let result = processor
                    .process_render_f32(&[&sys], &mut [&mut render_output])
                    .and_then(|()| processor.process_capture_f32(&[&mic], &mut [&mut output]));
                if let Err(error) = result {
                    // Fail open: a DSP failure must not erase the user's speech.
                    log::warn!(
                        "Echo cancellation failed; retaining microphone frame: {}",
                        error
                    );
                    output.copy_from_slice(&mic);
                } else if self.reference_tail == 0 {
                    // Mic-only / unavailable reference: preserve the input exactly.
                    output.copy_from_slice(&mic);
                }
            }
            cleaned.extend_from_slice(&output[..length]);
        }
        Ok((cleaned, reference))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn energy(samples: &[f32]) -> f32 {
        samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32
    }

    fn noise(length: usize, mut seed: u32) -> Vec<f32> {
        // Broadband, repeatable reference rather than an ambiguous single tone.
        (0..length)
            .map(|_| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                ((seed >> 8) as f32 / 16777216.0 - 0.5) * 0.4
            })
            .collect()
    }

    #[test]
    fn echo_cancellation_preserves_mic_only_and_partial_tail() {
        for enabled in [false, true] {
            let mic = noise(1897, 41);
            let sys = vec![0.0; mic.len()];
            let mut aec = AcousticEchoCanceller::new(48000, enabled).unwrap();
            let mut output = Vec::new();
            for (mic, sys) in mic.chunks(113).zip(sys.chunks(113)) {
                output.extend(aec.process(mic, sys).unwrap().0);
            }
            output.extend(aec.flush().unwrap().0);
            assert_eq!(output, mic);
            assert!(aec.flush().unwrap().0.is_empty());
            assert!(aec.process(&[0.0], &[]).is_err());
        }
    }

    #[test]
    fn echo_cancellation_reduces_delayed_room_echo_and_preserves_double_talk() {
        for rate in [16000, 48000] {
            let length = rate as usize * 12;
            let render = noise(length, 7);
            let fixture = super::super::decoder::decode_audio_file(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/echo-near-end-speech.wav"),
            )
            .unwrap();
            let near_16k = fixture.samples;
            let near: Vec<_> = (0..length)
                .map(|i| near_16k[(i * 16000 / rate as usize) % near_16k.len()] * 0.8)
                .collect();
            let delay = rate as usize * 80 / 1000;
            let reflection = rate as usize * 127 / 1000;
            let mut mic = vec![0.0; length];
            for i in 0..length {
                if i >= delay {
                    mic[i] += render[i - delay] * 0.55;
                }
                if i >= reflection {
                    mic[i] += render[i - reflection] * 0.15;
                }
                if i >= rate as usize * 8 {
                    mic[i] += near[i];
                }
            }
            let mut aec = AcousticEchoCanceller::new(rate, true).unwrap();
            let mut output = Vec::new();
            let mut untouched_render = Vec::new();
            for (mic, sys) in mic.chunks(997).zip(render.chunks(997)) {
                let (cleaned, reference) = aec.process(mic, sys).unwrap();
                output.extend(cleaned);
                untouched_render.extend(reference);
            }
            let (cleaned, reference) = aec.flush().unwrap();
            output.extend(cleaned);
            untouched_render.extend(reference);
            assert_eq!(output.len(), length);
            assert_eq!(untouched_render, render);
            let range = rate as usize * 5..rate as usize * 7;
            let reduction = 10.0 * (energy(&mic[range.clone()]) / energy(&output[range])).log10();
            eprintln!("{}Hz echo reduction: {:.1}dB", rate, reduction);
            assert!(reduction > 15.0, "AEC did not converge: {}dB", reduction);
            // AEC has a small filter-bank delay. Find correlation with independent
            // near-end content, rather than accepting any nonzero residual echo.
            let start = rate as usize * 10;
            let end = rate as usize * 11;
            let mut best = 0.0f32;
            let mut best_lag = 0;
            for lag in 0..rate as usize / 10 {
                let a = &near[start..end];
                let b = &output[start + lag..end + lag];
                let dot = a.iter().zip(b).map(|(a, b)| a * b).sum::<f32>();
                let norm = (a.iter().map(|x| x * x).sum::<f32>()
                    * b.iter().map(|x| x * x).sum::<f32>())
                .sqrt();
                let correlation = dot / norm.max(1e-12);
                if correlation > best {
                    best = correlation;
                    best_lag = lag;
                }
            }
            eprintln!(
                "{}Hz double-talk correlation: {:.3}, lag={}, energy ratio={:.3}",
                rate,
                best,
                best_lag,
                energy(&output[start..end]) / energy(&near[start..end])
            );
            assert!(best > 0.65, "near-end speech was suppressed: {}", best);
            assert!(energy(&output[start..end]) > energy(&near[start..end]) * 0.2);
        }
    }
}
