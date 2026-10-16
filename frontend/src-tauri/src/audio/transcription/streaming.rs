//! Overlapping offline Whisper decoding. Audio clocks remain capture clocks;
//! hypotheses are replaceable and only final passages enter the native sink.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const HOP_SAMPLES: usize = 32_000;
pub const WINDOW_SAMPLES: usize = 80_000;
pub const OVERLAP_SAMPLES: usize = 48_000;
// Large unresolved windows can trigger very expensive autoregressive fallback
// on the installed model. Seal at twenty seconds while retaining the same
// three-second source overlap and all checkpoints until canonical acknowledgment.
pub const MAX_WINDOW_SAMPLES: usize = 320_000;

pub(crate) fn is_pending_filename(name: &str) -> bool {
    name.strip_suffix(".tmp").unwrap_or(name).strip_prefix("asr-pending-")
        .and_then(|s| s.strip_suffix(".pcm").or_else(|| s.strip_suffix(".json")))
        .and_then(|s| s.rsplit_once('-'))
        .map_or(false, |(session, number)| {
            uuid::Uuid::parse_str(session).is_ok()
                && !number.is_empty()
                && number.chars().all(|c| c.is_ascii_digit())
        })
}
/// Called only after canonical final transcript acknowledgment. Failed/still
/// active sessions keep source PCM and its capture-clock/provenance sidecars.
pub(crate) fn cleanup_pending(root: &std::path::Path, session: &str) -> Result<()> {
    if uuid::Uuid::parse_str(session).is_err() {
        return Err(anyhow!("Invalid checkpoint session"));
    }
    if !root.exists() {
        return Ok(());
    }
    if std::fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err(anyhow!("Checkpoint directory is a symlink"));
    }
    let prefix = format!("asr-pending-{session}-");
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with(&prefix) && is_pending_filename(name) {
            if !entry.file_type()?.is_file() {
                return Err(anyhow!("Source checkpoint is not a regular file"));
            }
            std::fs::remove_file(entry.path())?;
        }
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue; };
        let encoded = name.strip_prefix("audio_chunk_").and_then(|s|s.strip_suffix(".mp4"))
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|c|c.is_ascii_digit()));
        if encoded || name == "concat_list.txt" {
            anyhow::ensure!(entry.file_type()?.is_file(), "Checkpoint is not a regular file");
            std::fs::remove_file(entry.path())?;
        }
    }
    if std::fs::read_dir(root)?.next().is_none() { std::fs::remove_dir(root)?; }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedToken {
    pub id: i32,
    pub bytes: Vec<u8>,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

pub struct LiveDecoder {
    state: whisper_rs::WhisperState,
    vocabulary: Vec<Vec<u8>>,
    language: Option<String>,
}
impl LiveDecoder {
    pub fn new(context: &whisper_rs::WhisperContext, language: Option<String>) -> Result<Self> {
        let vocabulary = (0..context.token_eot())
            .map(|id| {
                context
                    .token_to_cstr(id)
                    .map(|text| text.to_bytes().to_vec())
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(Self {
            state: context.create_state()?,
            vocabulary,
            language,
        })
    }

    pub fn decode(&mut self, samples: &[f32], offset: f64) -> Result<Vec<TimedToken>> {
        if samples.is_empty() {
            return Ok(Vec::new());
        }
        let params = crate::whisper_engine::decode_parameters::full_params(self.language.as_deref());
        self.state.full(params, samples)?;
        let duration = samples.len() as f64 / 16_000.0;
        let mut tokens = Vec::new();
        for segment in 0..self.state.full_n_segments()? {
            for index in 0..self.state.full_n_tokens(segment)? {
                let id = self.state.full_get_token_id(segment, index)?;
                let Some(bytes) = self.vocabulary.get(id as usize) else {
                    continue;
                };
                let data = self.state.full_get_token_data(segment, index)?;
                let valid =
                    data.t0 >= 0 && data.t1 > data.t0 && data.t1 as f64 * 0.01 <= duration + 0.1;
                tokens.push(TimedToken {
                    id,
                    bytes: bytes.clone(),
                    start: valid.then_some(offset + data.t0 as f64 * 0.01),
                    end: valid.then_some(offset + data.t1 as f64 * 0.01),
                });
            }
        }
        Ok(tokens)
    }
}

#[derive(Debug)]
pub struct Passage {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub is_partial: bool,
}

/// One per capture source. Window growth protects unresolved audio; decoding
/// boundaries, unlike VAD endpoints, never reset the capture timeline.
pub struct LiveBuffer {
    audio: VecDeque<f32>,
    start_sample: u64,
    end_sample: u64,
    committed_sample: u64,
    previous: Vec<TimedToken>,
    committed_tokens: Vec<TimedToken>,
}
impl Default for LiveBuffer {
    fn default() -> Self {
        Self {
            audio: VecDeque::new(),
            start_sample: 0,
            end_sample: 0,
            committed_sample: 0,
            previous: Vec::new(),
            committed_tokens: Vec::new(),
        }
    }
}
impl LiveBuffer {
    /// A real source-clock gap seals the prior utterance; never manufacture PCM.
    pub fn has_gap(&self, timestamp: f64) -> bool {
        !self.audio.is_empty() && timestamp.is_finite() && timestamp >= 0.0
            && (timestamp * 16_000.0).round() > self.end_sample as f64 + 2.0
    }
    pub fn append(&mut self, samples: &[f32], timestamp: f64) -> Result<()> {
        anyhow::ensure!(timestamp.is_finite() && timestamp >= 0.0 && timestamp < u64::MAX as f64 / 16_000.0,
            "Invalid incremental audio timestamp");
        anyhow::ensure!(samples.iter().all(|sample| sample.is_finite()), "Non-finite source PCM");
        if samples.is_empty() { return Ok(()); }
        let start = (timestamp * 16_000.0).round() as u64;
        if self.audio.is_empty() {
            self.start_sample = start;
            self.end_sample = start;
            self.committed_sample = start;
        }
        // VAD pre/post padding can overlap neighbouring utterances. The caller
        // flushes endpoints separately; within an utterance delivery is disjoint.
        anyhow::ensure!(start.checked_add(samples.len() as u64).is_some(), "Source sample clock overflow");
        let mut samples = samples;
        if start.saturating_add(2) < self.end_sample {
            anyhow::ensure!(start >= self.start_sample, "Overlap precedes retained source PCM");
            let overlap = ((self.end_sample - start) as usize).min(samples.len());
            let offset = (start - self.start_sample) as usize;
            anyhow::ensure!(self.audio.iter().skip(offset).take(overlap).copied().eq(samples[..overlap].iter().copied()),
                "Conflicting incremental source overlap; PCM retained for recovery");
            samples = &samples[overlap..];
        } else if start.abs_diff(self.end_sample) > 2 {
            return Err(anyhow!(
                "Incremental audio is not contiguous: expected {}, got {}",
                self.end_sample,
                start
            ));
        }
        self.audio.extend(samples);
        self.end_sample += samples.len() as u64;
        let desired = self.end_sample.saturating_sub(WINDOW_SAMPLES as u64);
        // Preserve the declared overlap behind a commit on EVERY append, not
        // only immediately after accept_boundary. Token timing can put a
        // boundary inside the next spoken word; subsequent hypotheses need
        // that retained context to recover it without deleting repetitions.
        let context_start = self.committed_sample.saturating_sub(OVERLAP_SAMPLES as u64);
        let keep_from = desired.min(context_start).max(self.start_sample);
        let discard = (keep_from - self.start_sample) as usize;
        self.audio.drain(..discard.min(self.audio.len()));
        self.start_sample = keep_from;
        Ok(())
    }
    pub fn samples(&self) -> Vec<f32> {
        self.audio.iter().copied().collect()
    }
    pub fn copy_samples(&self, destination: &mut Vec<f32>) {
        destination.clear();
        destination.extend(self.audio.iter().copied());
    }
    pub fn end(&self) -> f64 { self.end_sample as f64 / 16_000.0 }
    pub fn unresolved_duration(&self) -> f64 {
        self.end_sample.saturating_sub(self.committed_sample) as f64 / 16_000.0
    }
    pub fn start(&self) -> f64 {
        self.start_sample as f64 / 16_000.0
    }
    pub fn duration(&self) -> f64 {
        self.audio.len() as f64 / 16_000.0
    }
    pub fn committed_end(&self) -> f64 {
        self.committed_sample as f64 / 16_000.0
    }
    pub fn len(&self) -> usize {
        self.audio.len()
    }
    pub fn must_finalize(&self) -> bool {
        self.audio.len() >= MAX_WINDOW_SAMPLES
    }
    /// A full unresolved window with no accepted tokens cannot consume another
    /// input sample. Fail visibly, retaining checkpoints, instead of decoding
    /// the same full window forever during Stop.
    pub fn ensure_window_progress(&self) -> Result<()> {
        if self.must_finalize() {
            Err(anyhow!("Continuous Whisper could not advance a full window; source audio is retained"))
        } else { Ok(()) }
    }

    fn remaining(&self, tokens: Vec<TimedToken>) -> Vec<TimedToken> {
        // Only remove tokens supported by already committed audio AND an
        // aligned token match. Text alone must never delete repeated phrases.
        let mut skip = 0;
        for count in (1..=self.committed_tokens.len().min(tokens.len())).rev() {
            let previous = &self.committed_tokens[self.committed_tokens.len() - count..];
            if committed_prefix_match(previous, &tokens[..count], self.start(), self.committed_sample as f64 / 16_000.)
            {
                skip = count;
                break;
            }
        }
        tokens.into_iter().skip(skip).collect()
    }
    pub fn accept(&mut self, tokens: Vec<TimedToken>, endpoint: bool) -> Vec<Passage> {
        self.accept_boundary(tokens, endpoint, endpoint)
    }
    pub fn accept_boundary(
        &mut self,
        tokens: Vec<TimedToken>,
        final_decode: bool,
        reset: bool,
    ) -> Vec<Passage> {
        let tokens = self.remaining(tokens);
        let cutoff = self.end_sample as f64 / 16_000.0 - 1.0;
        let mut stable = if final_decode {
            tokens.len()
        } else {
            self.previous
                .iter()
                .zip(&tokens)
                .take_while(|(a, b)| aligned(a, b) && b.end.is_some_and(|end| end <= cutoff))
                .count()
        };
        // BPE token boundaries may fall inside a UTF-8 code point.
        while stable > 0 && String::from_utf8(token_bytes(&tokens[..stable])).is_err() {
            stable -= 1;
        }
        // Keep incomplete BPE words in the provisional tail.
        if !final_decode {
            while stable > 0 && !word_boundary(&tokens, stable) {
                stable -= 1;
            }
            // A revised timestamp can move backwards inside retained context.
            // It cannot move the canonical audio boundary backwards or prune
            // unresolved audio; wait for another hypothesis or an endpoint.
            if stable > 0 && tokens[stable - 1].end.is_some_and(|end| end <= self.committed_sample as f64 / 16_000.) {
                stable = 0;
            }
        }
        let end = self.end_sample as f64 / 16_000.0;
        let mut passages = Vec::new();
        if stable > 0 {
            let stable_end = if final_decode {
                end
            } else {
                tokens[stable - 1].end.unwrap_or(end)
            };
            passages.push(Passage {
                text: text(&tokens[..stable]),
                start: self.committed_sample as f64 / 16_000.0,
                end: stable_end,
                is_partial: false,
            });
            self.committed_sample = (stable_end * 16_000.0).round() as u64;
            self.committed_tokens.extend_from_slice(&tokens[..stable]);
            self.committed_tokens
                .retain(|token| token.end.map_or(true, |end| end >= stable_end - 4.0));
            let keep_from = self
                .committed_sample
                .saturating_sub(OVERLAP_SAMPLES as u64)
                .max(self.start_sample);
            let discard = (keep_from - self.start_sample) as usize;
            self.audio.drain(..discard.min(self.audio.len()));
            self.start_sample = keep_from;
        }
        self.previous = tokens[stable..].to_vec();
        if !final_decode && !self.previous.is_empty() {
            passages.push(Passage {
                text: text(&self.previous),
                start: self.committed_sample as f64 / 16_000.0,
                end,
                is_partial: true,
            });
        }
        if reset {
            *self = Self::default();
        }
        passages
    }
}
fn aligned(a: &TimedToken, b: &TimedToken) -> bool {
    a.id == b.id
        && a.bytes == b.bytes
        && match (a.start, b.start, a.end, b.end) {
            (Some(a0), Some(b0), Some(a1), Some(b1)) => {
                (a0 - b0).abs() <= 0.5 && (a1 - b1).abs() <= 0.5
            }
            _ => false,
        }
}
fn committed_prefix_match(prior: &[TimedToken], current: &[TimedToken], window_start: f64, committed_end: f64) -> bool {
    if prior.is_empty() || prior.len()!=current.len() || !prior.iter().zip(current).all(|(a,b)|a.id==b.id && a.bytes==b.bytes) { return false; }
    // Missing punctuation/leading BPE timing must not defeat a whole-prefix
    // match anchored on retained source audio. A rightmost spoken token needs
    // timing, and every known current token must lie before the commit. A
    // wholly untimed prefix or a later repetition therefore cannot qualify.
    let right = current.iter().rposition(|t|std::str::from_utf8(&t.bytes).map_or(true, |s|s.chars().any(char::is_alphanumeric)));
    if !right.is_some_and(|i|current[i].end.is_some() && prior[i].end.is_some()) { return false; }
    let mut anchored = false;
    let mut prior_end = window_start;
    let mut current_end = window_start;
    for (a,b) in prior.iter().zip(current) {
        if let Some(end)=b.end { if end>committed_end { return false; } current_end=current_end.max(end); }
        if let Some(end)=a.end { prior_end=prior_end.max(end); }
        anchored |= a.start.is_some() && a.end.is_some() && b.start.is_some() && b.end.is_some();
    }
    let prior_start=prior[0].start.unwrap_or(window_start);
    let current_start=current[0].start.unwrap_or(window_start);
    anchored && prior_start<current_end && current_start<prior_end
}
fn token_bytes(tokens: &[TimedToken]) -> Vec<u8> {
    tokens
        .iter()
        .flat_map(|token| token.bytes.iter().copied())
        .collect()
}
fn word_boundary(tokens: &[TimedToken], index: usize) -> bool {
    let prior = token_bytes(&tokens[..index]);
    let Ok(prior) = std::str::from_utf8(&prior) else {
        return false;
    };
    if prior.chars().last().is_some_and(|c| !c.is_alphanumeric()) {
        return true;
    }
    tokens
        .get(index)
        .and_then(|token| std::str::from_utf8(&token.bytes).ok())
        .and_then(|next| next.chars().next())
        .is_some_and(|c| !c.is_alphanumeric())
}
fn text(tokens: &[TimedToken]) -> String {
    String::from_utf8_lossy(&token_bytes(tokens))
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_forced_decode_cannot_loop_forever_or_discard_unresolved_audio() {
        let mut buffer=LiveBuffer::default();
        buffer.append(&vec![0.2;MAX_WINDOW_SAMPLES],0.).unwrap();
        assert!(buffer.accept_boundary(Vec::new(),true,false).is_empty());
        assert!(buffer.ensure_window_progress().is_err());
        assert_eq!(buffer.len(),MAX_WINDOW_SAMPLES);
        let mut copy=Vec::new();buffer.copy_samples(&mut copy);
        assert_eq!(copy,vec![0.2;MAX_WINDOW_SAMPLES]);
        // A genuine endpoint may retract noise-only text and reset the window.
        assert!(buffer.accept_boundary(Vec::new(),true,true).is_empty());
        assert!(buffer.ensure_window_progress().is_ok());
    }
    fn token(id: i32, text: &str, start: f64, end: f64) -> TimedToken {
        TimedToken {
            id,
            bytes: text.as_bytes().to_vec(),
            start: Some(start),
            end: Some(end),
        }
    }
    #[test]
    fn overlapping_hypotheses_finalize_prefix_and_preserve_repeated_words() {
        let mut buffer = LiveBuffer::default();
        buffer.append(&vec![0.1; 32_000], 0.0).unwrap();
        let first = vec![
            token(1, " yes", 0.0, 0.5),
            token(1, " yes", 0.6, 1.0),
            token(2, " later", 1.2, 1.8),
        ];
        assert!(buffer
            .accept(first.clone(), false)
            .iter()
            .all(|p| p.is_partial));
        buffer.append(&vec![0.1; 32_000], 2.0).unwrap();
        let result = buffer.accept(first.clone(), false);
        assert_eq!(result[0].text, "yes yes");
        assert!(!result[0].is_partial);
        assert_eq!(result[1].text, "later");
        buffer.append(&vec![0.1; 32_000], 4.0).unwrap();
        let mut next = first;
        next.push(token(1, " yes", 4.2, 4.7));
        let result = buffer.accept(next, true);
        assert_eq!(result[0].text, "later yes");
        assert_eq!(result[0].start, 1.0);
        assert!(buffer.accept(vec![], true).is_empty());
    }
    #[test]
    fn appending_audio_preserves_overlap_after_an_early_commit() {
        let mut buffer = LiveBuffer::default();
        let first = vec![0.1; 6 * 16_000];
        buffer.append(&first, 0.).unwrap();
        let hypothesis = vec![
            token(1, " yes yes", 0., 1.81),
            token(2, " repeat repeat", 1.82, 5.5),
        ];
        buffer.accept(hypothesis.clone(), false);
        let committed = buffer.accept(hypothesis, false);
        assert_eq!(committed[0].text, "yes yes");
        buffer.append(&vec![0.2; 2 * 16_000], 6.).unwrap();
        assert_eq!(buffer.start(), 0.);
        assert_eq!(&buffer.samples()[..first.len()], &first);
        assert_eq!(buffer.duration(), 8.);
    }
    #[test]
    fn revised_overlap_timing_never_moves_commit_backwards_or_deletes_a_later_repeat() {
        let mut buffer = LiveBuffer::default();
        buffer.append(&vec![0.1; 6 * 16_000], 0.).unwrap();
        let first = vec![token(1, " yes", 0., 1.81), token(2, " later", 4., 5.5)];
        buffer.accept(first.clone(), false);
        buffer.accept(first, false);
        assert_eq!(buffer.committed_end(), 1.81);
        let revised = vec![token(1, " yes", 0., 1.08), token(1, " yes", 2., 3.), token(2, " later", 4., 5.5)];
        let final_rows = buffer.accept(revised, true);
        assert_eq!(final_rows[0].text, "yes later");
        assert_eq!(final_rows[0].start, 1.81);
        assert_eq!(final_rows[0].end, 6.);

        // A mismatching token with an old timestamp cannot seal a backwards
        // range merely because two successive hypotheses repeat it.
        let mut buffer = LiveBuffer::default();
        buffer.append(&vec![0.1; 6 * 16_000], 0.).unwrap();
        let first = vec![token(1, " yes", 0., 1.81), token(2, " later", 4., 5.5)];
        buffer.accept(first.clone(), false); buffer.accept(first, false);
        let shifted = vec![token(3, " YES", 0., 1.08), token(2, " later", 4., 5.5)];
        buffer.accept(shifted.clone(), false);
        assert!(buffer.accept(shifted, false).iter().all(|p|p.is_partial));
        assert_eq!(buffer.committed_end(), 1.81);
    }
    #[test]
    fn overlap_prefix_requires_an_audio_anchor_and_preserves_later_repetitions() {
        let mut leading = token(1, " yes", 0., 0.5);
        leading.start=None; leading.end=None;
        let prior=vec![leading.clone(),token(2," yes",0.6,1.81)];
        let current=vec![leading.clone(),token(2," yes",0.5,1.08)];
        assert!(committed_prefix_match(&prior,&current,0.,1.81));
        let later=vec![leading.clone(),token(2," yes",2.,3.)];
        assert!(!committed_prefix_match(&prior,&later,0.,1.81));
        assert!(!committed_prefix_match(&[leading.clone()],&[leading.clone()],0.,1.81));
        let unbounded_tail=vec![token(2," yes",0.5,1.08),leading];
        assert!(!committed_prefix_match(&unbounded_tail,&unbounded_tail,0.,1.81));
        assert!(!committed_prefix_match(&[token(1," yes",0.,1.)],&[token(1," yes",2.,3.)],0.,4.));
    }
    #[test]
    fn missing_timing_waits_for_endpoint_and_long_unresolved_audio_is_retained() {
        let mut buffer = LiveBuffer::default();
        buffer.append(&vec![0.0; MAX_WINDOW_SAMPLES], 0.0).unwrap();
        assert!(buffer.must_finalize());
        assert_eq!(buffer.samples().len(), MAX_WINDOW_SAMPLES);
        let tokens = vec![TimedToken {
            id: 1,
            bytes: b"still here".to_vec(),
            start: None,
            end: None,
        }];
        assert!(buffer.accept(tokens.clone(), false)[0].is_partial);
        assert_eq!(buffer.accept(tokens, true)[0].text, "still here");
    }
    #[test]
    fn source_buffers_reject_clock_gaps_and_utf8_is_joined_before_decoding() {
        let mut buffer = LiveBuffer::default();
        buffer.append(&[0.1; 100], 10.0).unwrap();
        assert!(buffer.append(&[0.1], 11.0).is_err());
        let tokens = vec![
            TimedToken {
                id: 1,
                bytes: vec![0xe6, 0x97],
                start: None,
                end: None,
            },
            TimedToken {
                id: 2,
                bytes: vec![0xa5],
                start: None,
                end: None,
            },
        ];
        assert_eq!(buffer.accept(tokens, true)[0].text, "日");
    }
    #[test]
    fn forced_window_boundary_keeps_overlap_without_replaying_committed_words() {
        let mut buffer = LiveBuffer::default();
        buffer.append(&vec![0.1; MAX_WINDOW_SAMPLES], 0.0).unwrap();
        let tokens = vec![
            token(1, " boundary", 17.0, 18.0),
            token(2, " word", 18.1, 19.0),
        ];
        let final_passage = buffer.accept_boundary(tokens.clone(), true, false);
        assert_eq!(final_passage[0].text, "boundary word");
        assert_eq!(buffer.len(), OVERLAP_SAMPLES);
        assert_eq!(buffer.start(), 17.0);
        assert_eq!(buffer.committed_end(), 20.0);
        assert_eq!(buffer.unresolved_duration(), 0., "retained committed context needs no Stop decode");
        buffer.append(&vec![0.1; HOP_SAMPLES], 20.0).unwrap();
        assert_eq!(buffer.unresolved_duration(), 2., "new speech still requires a final flush");
        assert_eq!(buffer.start(), 17.0, "append must keep the full boundary overlap");
        let mut next = tokens;
        next.push(token(3, " continuation", 20.1, 21.0));
        assert_eq!(buffer.accept(next, true)[0].text, "continuation");
    }
    #[test]
    fn acknowledged_session_cleanup_preserves_other_sessions_and_user_files() {
        let dir = tempfile::tempdir().unwrap();
        let session = uuid::Uuid::new_v4().to_string();
        let other = uuid::Uuid::new_v4().to_string();
        for name in [
            format!("asr-pending-{session}-0.pcm"),
            format!("asr-pending-{session}-0.json"),
            format!("asr-pending-{other}-0.pcm"),
            "user-import.pcm".into(),
            "asr-pending-invalid-0.pcm".into(),
        ] {
            std::fs::write(dir.path().join(name), b"preserved until acknowledgment").unwrap();
        }
        cleanup_pending(dir.path(), &session).unwrap();
        assert!(!dir
            .path()
            .join(format!("asr-pending-{session}-0.pcm"))
            .exists());
        assert!(!dir
            .path()
            .join(format!("asr-pending-{session}-0.json"))
            .exists());
        assert!(dir
            .path()
            .join(format!("asr-pending-{other}-0.pcm"))
            .exists());
        assert!(dir.path().join("user-import.pcm").exists());
        assert!(dir.path().join("asr-pending-invalid-0.pcm").exists());
    }
    #[test]
    fn partial_bpe_words_cannot_be_sealed_into_separate_passages() {
        let mut buffer = LiveBuffer::default();
        buffer.append(&vec![0.1; 32_000], 0.0).unwrap();
        let first = vec![token(1, " recom", 0.0, 0.8)];
        buffer.accept(first.clone(), false);
        buffer.append(&vec![0.1; 32_000], 2.0).unwrap();
        let mut next = first;
        next.push(token(2, "mend", 0.8, 1.2));
        assert!(buffer
            .accept(next.clone(), false)
            .iter()
            .all(|p| p.is_partial));
        assert_eq!(buffer.accept(next, true)[0].text, "recommend");
    }
}

#[cfg(test)] mod reinforcement_tests {
    use super::*;
    #[test] fn overlap_requires_exact_retained_pcm_and_preserves_new_suffix() {
        let mut buffer=LiveBuffer::default();
        buffer.append(&[1.,2.,3.,4.,5.,6.],1.).unwrap();
        buffer.append(&[3.,4.,5.,6.,7.,8.],1.+2./16000.).unwrap();
        assert_eq!(buffer.samples(),vec![1.,2.,3.,4.,5.,6.,7.,8.]);
        let before=buffer.samples();
        assert!(buffer.append(&[99.,4.,5.],1.+2./16000.).is_err());
        assert_eq!(buffer.samples(),before);
    }
    #[test] fn invalid_clocks_do_not_mutate_pcm_and_gaps_are_explicit() {
        let mut buffer=LiveBuffer::default();buffer.append(&[0.2;160],2.).unwrap();
        for timestamp in [f64::NAN,f64::INFINITY,-1.] {assert!(buffer.append(&[0.3],timestamp).is_err());}
        assert!(buffer.has_gap(3.));assert!(buffer.append(&[0.3],3.).is_err());
        assert_eq!(buffer.samples(),vec![0.2;160]);
        buffer.accept_boundary(Vec::new(),true,true);
        buffer.append(&[0.3;160],3.).unwrap();assert_eq!(buffer.start(),3.);assert_eq!(buffer.end(),3.01);
    }
}
