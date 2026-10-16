//! Durable source inbox. A monotonically numbered disk cursor replaces an
//! unbounded queue of PCM or file descriptors. Metadata is the publish marker.
use super::health::Health;
use crate::audio::{
    queue::PcmReceiver,
    recording_state::{AudioChunk, DeviceType, TranscriptionChunk},
    source::AudioSource,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, Runtime};
#[derive(Serialize, Deserialize)]
struct Metadata {
    version: u32,
    recording_session_id: Option<String>,
    meeting_id: Option<String>,
    audio_source: AudioSource,
    timestamp: f64,
    sample_rate: u32,
    samples: usize,
    stream_end: Option<bool>,
    chunk_id: u64,
    enqueued_seconds: f64,
}
pub(super) struct Pending {
    pub chunk: TranscriptionChunk,
    pub path: PathBuf,
    pub samples: usize,
    pub enqueued_at: Instant,
}
impl Pending {
    pub fn load(&mut self) -> Result<()> {
        anyhow::ensure!(
            self.samples <= crate::audio::queue::PCM_BUDGET / 4,
            "Pending source PCM exceeds intake budget"
        );
        anyhow::ensure!(
            std::fs::symlink_metadata(&self.path)?.file_type().is_file(),
            "Source PCM is not a regular file"
        );
        let bytes = std::fs::read(&self.path)?;
        anyhow::ensure!(
            bytes.len()
                == self
                    .samples
                    .checked_mul(4)
                    .ok_or_else(|| anyhow!("Invalid PCM size"))?,
            "Pending source audio size mismatch"
        );
        self.chunk.audio.data = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        Ok(())
    }
}
pub(super) struct State {
    root: PathBuf,
    run_id: String,
    started: Instant,
    published: Mutex<u64>,
    ready: Condvar,
    pub health: Arc<Health>,
    pub by_source: [AtomicU64; 4],
}
impl State {
    fn read(&self, index: u64) -> Result<Option<Pending>> {
        let mut published = self.published.lock().unwrap();
        while index >= *published && !self.health.closed.load(Ordering::SeqCst) {
            published = self.ready.wait(published).unwrap();
        }
        if index >= *published {
            if let Some(e) = self.health.error.lock().unwrap().clone() {
                return Err(anyhow!(e));
            }
            return Ok(None);
        }
        drop(published);
        let path = self
            .root
            .join(format!("asr-pending-{}-{index}.pcm", self.run_id));
        anyhow::ensure!(
            std::fs::symlink_metadata(path.with_extension("json"))?
                .file_type()
                .is_file(),
            "Source sidecar is not a regular file"
        );
        let meta: Metadata = serde_json::from_slice(&std::fs::read(path.with_extension("json"))?)?;
        anyhow::ensure!(
            meta.version == 1
                && meta.sample_rate == 16000
                && meta.recording_session_id == self.health.identity.1
                && meta.meeting_id == self.health.identity.0,
            "Invalid source checkpoint identity"
        );
        anyhow::ensure!(
            meta.enqueued_seconds.is_finite() && meta.enqueued_seconds >= 0.,
            "Invalid source checkpoint clock"
        );
        Ok(Some(Pending {
            chunk: TranscriptionChunk {
                audio: AudioChunk {
                    data: Vec::new(),
                    sample_rate: 16000,
                    timestamp: meta.timestamp,
                    chunk_id: meta.chunk_id,
                    device_type: if meta.audio_source == AudioSource::System {
                        DeviceType::System
                    } else {
                        DeviceType::Microphone
                    },
                },
                audio_source: meta.audio_source,
                stream_end: meta.stream_end,
            },
            path,
            samples: meta.samples,
            enqueued_at: self.started + Duration::from_secs_f64(meta.enqueued_seconds),
        }))
    }
    pub fn complete(&self, p: &Pending) {
        self.health.complete(p.samples);
        self.by_source[source_index(p.chunk.audio_source)].fetch_sub(1, Ordering::SeqCst);
    }
}
pub(super) fn source_index(source: AudioSource) -> usize {
    match source {
        AudioSource::Microphone => 0,
        AudioSource::System => 1,
        AudioSource::Mixed => 2,
        AudioSource::Unknown => 3,
    }
}
pub(super) struct Inbox {
    pub state: Arc<State>,
    next: u64,
    pub writer: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<Result<()>>>>>,
}
impl Inbox {
    pub fn blocking_recv(&mut self) -> Result<Option<Pending>> {
        let result = self.state.read(self.next)?;
        if result.is_some() {
            self.next += 1;
        }
        Ok(result)
    }
    pub async fn recv(&mut self) -> Result<Option<Pending>> {
        let state = self.state.clone();
        let next = self.next;
        let result = tokio::task::spawn_blocking(move || state.read(next)).await??;
        if result.is_some() {
            self.next += 1;
        }
        Ok(result)
    }
    pub async fn join_writer(&mut self) -> Result<()> {
        if let Some(writer) = self.writer.lock().await.take() {
            writer.await??;
        }
        Ok(())
    }
}
fn write_private(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension().unwrap().to_string_lossy()
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(&temporary, path)?;
    #[cfg(unix)]
    std::fs::File::open(path.parent().unwrap())?.sync_all()?;
    Ok(())
}
pub(super) async fn start<R: Runtime>(
    app: &AppHandle<R>,
    mut input: PcmReceiver<TranscriptionChunk>,
    identity: (Option<String>, Option<String>),
) -> Result<Inbox> {
    let health = Health::begin(identity);
    let folder = crate::audio::recording_commands::get_meeting_folder_path()
        .await
        .ok()
        .flatten();
    let root = if crate::q02_validation::enabled(app) {
        crate::q02_validation::checkpoint_root(app, health.identity.1.as_deref().unwrap_or(""))
            .map_err(|e| anyhow!(e))?
    } else if let Some(folder) = folder {
        PathBuf::from(folder).join(".checkpoints")
    } else if cfg!(debug_assertions)
        && ["com.meetily.workflow.test", "com.meetily.notepad.test"]
            .contains(&app.config().identifier.as_str())
    {
        app.path().app_data_dir()?.join("workflow-pending")
    } else {
        return Err(anyhow!("Transcription requires a managed recording folder"));
    };
    open(root, input, health)
}
fn open(
    root: PathBuf,
    mut input: PcmReceiver<TranscriptionChunk>,
    health: Arc<Health>,
) -> Result<Inbox> {
    std::fs::create_dir_all(&root)?;
    anyhow::ensure!(
        !std::fs::symlink_metadata(&root)?.file_type().is_symlink(),
        "Source checkpoint root is a symlink"
    );
    let run_id = health
        .identity
        .1
        .clone()
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let state = Arc::new(State {
        root,
        run_id,
        started: Instant::now(),
        published: Mutex::new(0),
        ready: Condvar::new(),
        health,
        by_source: std::array::from_fn(|_| AtomicU64::new(0)),
    });
    let writer_state = state.clone();
    let writer = tokio::task::spawn_blocking(move || {
        let s = writer_state;
        let result = (|| -> Result<()> {
            while let Some(mut chunk) = input.blocking_recv() {
                let _lease = crate::database::deletion::recording_artifact_write(
                    s.health.identity.0.as_deref(),
                )
                .map_err(anyhow::Error::msg)?;
                anyhow::ensure!(
                    chunk.timestamp.is_finite() && chunk.timestamp >= 0. && chunk.sample_rate > 0,
                    "Invalid source clock/rate"
                );
                if chunk.sample_rate != 16000 {
                    chunk.audio.data = crate::audio::audio_processing::resample_audio(
                        &chunk.data,
                        chunk.sample_rate,
                        16000,
                    );
                }
                anyhow::ensure!(
                    chunk.data.iter().all(|x| x.is_finite()),
                    "Non-finite source PCM"
                );
                let number = s.health.queued.load(Ordering::SeqCst);
                let samples = chunk.data.len();
                let path = s
                    .root
                    .join(format!("asr-pending-{}-{number}.pcm", s.run_id));
                let bytes = bytemuck::cast_slice(&chunk.data);
                write_private(&path, bytes)?;
                crate::q02_validation::verify_checkpoint(&path, bytes).map_err(|e| anyhow!(e))?;
                let metadata = Metadata {
                    version: 1,
                    recording_session_id: s.health.identity.1.clone(),
                    meeting_id: s.health.identity.0.clone(),
                    audio_source: chunk.audio_source,
                    timestamp: chunk.timestamp,
                    sample_rate: 16000,
                    samples,
                    stream_end: chunk.stream_end,
                    chunk_id: chunk.chunk_id,
                    enqueued_seconds: s.started.elapsed().as_secs_f64(),
                };
                write_private(
                    &path.with_extension("json"),
                    &serde_json::to_vec(&metadata)?,
                )?;
                s.health
                    .pending_samples
                    .fetch_add(samples as u64, Ordering::SeqCst);
                s.health
                    .retained_bytes
                    .fetch_add(bytes.len() as u64, Ordering::SeqCst);
                s.by_source[source_index(chunk.audio_source)].fetch_add(1, Ordering::SeqCst);
                s.health.queued.fetch_add(1, Ordering::SeqCst);
                s.health.touch();
                *s.published.lock().unwrap() = number + 1;
                s.ready.notify_all();
            }
            if let Some(error) = input.error() {
                return Err(anyhow!(error));
            }
            Ok(())
        })();
        if let Err(e) = &result {
            s.health.fail(e.to_string());
        }
        // Publish closure under the same lock as the waiter predicate.
        let _lock = s.published.lock().unwrap();
        s.health.closed.store(true, Ordering::SeqCst);
        s.ready.notify_all();
        result
    });
    Ok(Inbox {
        state,
        next: 0,
        writer: Arc::new(tokio::sync::Mutex::new(Some(writer))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chunk(index: u64) -> TranscriptionChunk {
        TranscriptionChunk {
            audio: AudioChunk {
                data: vec![index as f32; 160],
                sample_rate: 16000,
                timestamp: index as f64 / 100.,
                chunk_id: index,
                device_type: DeviceType::System,
            },
            audio_source: AudioSource::System,
            stream_end: Some(true),
        }
    }
    #[tokio::test]
    async fn slow_reader_uses_disk_cursor_and_reconciles_every_sample() {
        let directory = tempfile::tempdir().unwrap();
        let (sender, receiver) = crate::audio::queue::pcm_channel();
        let health = Health::begin((
            Some("spool-fixture".into()),
            Some(uuid::Uuid::new_v4().to_string()),
        ));
        let mut inbox = open(directory.path().into(), receiver, health.clone()).unwrap();
        for i in 0..100 {
            sender.send(chunk(i)).unwrap();
        }
        drop(sender);
        inbox.join_writer().await.unwrap();
        assert_eq!(health.queued.load(Ordering::SeqCst), 100);
        assert_eq!(health.pending_samples.load(Ordering::SeqCst), 16000);
        for index in 0..100 {
            let mut pending = inbox.recv().await.unwrap().unwrap();
            pending.load().unwrap();
            assert_eq!(pending.chunk.audio.data, vec![index as f32; 160]);
            assert_eq!(pending.chunk.audio_source, AudioSource::System);
            inbox.state.complete(&pending);
        }
        assert!(inbox.recv().await.unwrap().is_none());
        assert_eq!(health.pending_samples.load(Ordering::SeqCst), 0);
        assert_eq!(
            std::fs::read_dir(directory.path()).unwrap().count(),
            200,
            "source checkpoints must survive decoding"
        );
    }
    #[tokio::test]
    async fn publication_failure_is_not_empty_success() {
        let directory = tempfile::tempdir().unwrap();
        let (sender, receiver) = crate::audio::queue::pcm_channel();
        let session = uuid::Uuid::new_v4().to_string();
        let health = Health::begin((None, Some(session.clone())));
        let mut inbox = open(directory.path().into(), receiver, health.clone()).unwrap();
        // A conflicting private temporary file deterministically simulates write failure.
        std::fs::write(
            directory
                .path()
                .join(format!("asr-pending-{session}-0.pcm.tmp")),
            "protected",
        )
        .unwrap();
        sender.send(chunk(0)).unwrap();
        drop(sender);
        assert!(inbox.join_writer().await.is_err());
        assert!(inbox.recv().await.is_err());
        assert!(!health.drained.load(Ordering::SeqCst));
        assert!(health.error.lock().unwrap().is_some());
    }
}
