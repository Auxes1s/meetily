use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use anyhow::Result;
use log::{info, warn, error};
use tauri::{AppHandle, Runtime, Emitter, Manager};
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use super::recording_state::AudioChunk;
use super::audio_processing::create_meeting_folder;
use super::incremental_saver::IncrementalAudioSaver;

#[cfg(test)]
mod source_tests {
    use super::*;
    use crate::audio::source::AudioSource;

    #[test]
    fn source_recording_json_preserves_all_channels_and_retry_provenance() {
        let saver = RecordingSaver::new();
        let sources = [AudioSource::Microphone, AudioSource::System, AudioSource::Mixed, AudioSource::Unknown];
        for (index, source) in sources.into_iter().enumerate() {
            let segment = TranscriptSegment {
                id: format!("transcript-session-{index}"), text: "fixture speech".into(),
                audio_start_time: index as f64, audio_end_time: index as f64 + 0.5, duration: 0.5,
                display_time: "12:00:00".into(), confidence: 1.0, sequence_id: index as u64,
                audio_source: Some(source),
            };
            saver.add_transcript_segment(segment.clone());
            let mut retry = segment;
            retry.audio_source = None;
            saver.add_transcript_segment(retry);
        }
        let folder = tempfile::tempdir().unwrap();
        saver.write_transcripts_json(&folder.path().to_path_buf()).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&std::fs::read(folder.path().join("transcripts.json")).unwrap()).unwrap();
        assert_eq!(json["version"], "1.1");
        assert_eq!(json["total_segments"], 4);
        let restored: Vec<TranscriptSegment> = serde_json::from_value(json["segments"].clone()).unwrap();
        for (index, segment) in restored.iter().enumerate() {
            assert_eq!(segment.audio_source.unwrap_or_default(), sources[index]);
            assert_eq!(segment.audio_start_time, index as f64);
            assert_eq!(segment.id, format!("transcript-session-{index}"));
        }
        let mut legacy = json["segments"][0].clone();
        legacy.as_object_mut().unwrap().remove("audio_source");
        assert_eq!(serde_json::from_value::<TranscriptSegment>(legacy).unwrap().audio_source, None);
    }
}

#[cfg(test)]
mod finalization_tests {
    use super::*;
    use crate::audio::{recording_state::DeviceType, source::AudioSource};
    use crate::database::{manager::DatabaseManager, repositories::transcript::TranscriptsRepository};
    #[tokio::test]
    async fn transcript_only_stop_compacts_the_final_journal() {
        let directory = tempfile::tempdir().unwrap();
        let mut saver = RecordingSaver::new(); saver.set_recordings_folder(directory.path().into());
        saver.initialize_meeting_folder("Transcript only", false).unwrap();
        saver.add_transcript_chunk("Final notes without audio".into());
        assert!(saver.stop_and_save(tauri::test::mock_app().handle(),None).await.unwrap().is_none());
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(saver.get_meeting_folder().unwrap().join("transcripts.json")).unwrap()).unwrap();
        assert_eq!(value["segments"][0]["text"],"Final notes without audio");
    }

    #[tokio::test]
    async fn audio_save_retains_final_transcripts_for_database_and_summary_sources() {
        let directory = tempfile::tempdir().unwrap();
        let db = DatabaseManager::new(directory.path().join("meeting_minutes.sqlite").to_str().unwrap(), "missing.db").await.unwrap();
        let id = format!("save-fixture-{}", uuid::Uuid::new_v4());
        let session = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,recording_session_id) VALUES (?,'Recorded fixture','2026-10-14','2026-10-14','recording',?)")
            .bind(&id).bind(&session).execute(db.pool()).await.unwrap();
        let app = tauri::test::mock_app();
        app.manage(crate::state::AppState { db_manager: db.clone() });
        let mut saver = RecordingSaver::new();
        saver.set_identity(Some(id.clone()), Some(session.clone()));
        saver.set_recordings_folder(directory.path().join("recordings"));
        saver.set_meeting_name(Some("Recorded fixture".into()));
        saver.initialize_meeting_folder("Recorded fixture", true).unwrap();
        // Real encoding/checkpoint merging, with synthetic audio only.
        saver.incremental_saver.as_ref().unwrap().lock().await.add_chunk(AudioChunk {
            data: vec![0.0; 48_000], sample_rate: 48_000, timestamp: 0.0,
            chunk_id: 0, device_type: DeviceType::Microphone,
        }).unwrap();
        for (sequence, source) in [AudioSource::Microphone, AudioSource::System].into_iter().enumerate() {
            saver.add_transcript_segment(TranscriptSegment {
                id: format!("transcript-{session}-{sequence}"), text: format!("Saved fixture passage {sequence}"),
                audio_start_time: sequence as f64 * 0.5, audio_end_time: (sequence + 1) as f64 * 0.5,
                duration: 0.5, display_time: "00:00:00".into(), confidence: 1.0,
                sequence_id: sequence as u64, audio_source: Some(source),
            });
        }
        let sink = saver.clone();
        let audio = saver.stop_and_save(app.handle(), Some(1.0)).await.unwrap().unwrap();
        assert!(std::path::Path::new(&audio).is_file());
        // Native stop reads this buffer after audio finalization, just as here.
        let final_segments = saver.get_transcript_segments();
        assert_eq!(final_segments.len(), 2, "audio finalization erased the transcript before SQLite could commit it");
        assert_eq!(sink.get_transcript_segments().len(), 2);
        let folder = saver.get_meeting_folder().unwrap();
        let json: serde_json::Value = serde_json::from_slice(&std::fs::read(folder.join("transcripts.json")).unwrap()).unwrap();
        assert_eq!(json["total_segments"], final_segments.len());
        let transcripts: Vec<crate::api::TranscriptSegment> = final_segments.into_iter().map(|segment| crate::api::TranscriptSegment {
            id: segment.id, text: segment.text, timestamp: segment.display_time,
            audio_start_time: Some(segment.audio_start_time), audio_end_time: Some(segment.audio_end_time),
            duration: Some(segment.duration), sequence_id: Some(segment.sequence_id as i64), audio_source: segment.audio_source,
        }).collect();
        for _ in 0..2 {
            assert_eq!(TranscriptsRepository::save_transcript(db.pool(), "Recorded fixture", &transcripts,
                Some(folder.to_string_lossy().into()), Some(id.clone()), Some(session.clone())).await.unwrap(), id);
        }
        let snapshot = crate::summary::grounding::SourceSnapshot::read(db.pool(), &id, None, false).await.unwrap();
        assert_eq!(snapshot.sources.len(), 2);
        for (source, transcript) in snapshot.sources.iter().zip(&transcripts) {
            assert_eq!(source.text, transcript.text);
            assert_eq!(source.source_id, format!("transcript:{}", transcript.id));
            assert_eq!(source.audio_start_time, transcript.audio_start_time);
        }
        assert_eq!(snapshot.transcript_coverage, "committed_available");
        db.cleanup().await.unwrap();
    }
}

/// Structured transcript segment for JSON export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: String,
    pub text: String,
    pub audio_start_time: f64, // Seconds from recording start
    pub audio_end_time: f64,   // Seconds from recording start
    pub duration: f64,          // Segment duration in seconds
    pub display_time: String,   // Formatted time for display like "[02:15]"
    pub confidence: f32,
    pub sequence_id: u64,
    #[serde(default)]
    pub audio_source: Option<super::source::AudioSource>,
}

#[cfg(test)]
mod deletion_tests {
    use super::*;

    #[tokio::test]
    async fn deleted_cloned_recording_sink_cannot_recreate_files_in_preserved_export_folder() {
        let dir = tempfile::tempdir().unwrap();
        let db = crate::database::manager::DatabaseManager::new(dir.path().join("meeting_minutes.sqlite").to_str().unwrap(), "missing.db").await.unwrap();
        let id = format!("delete-sink-{}", uuid::Uuid::new_v4());
        let root = dir.path().join("recordings");
        let folder = root.join("sink-folder");
        std::fs::create_dir_all(&folder).unwrap();
        let metadata = MeetingMetadata {
            version: "1.0".into(), meeting_id: Some(id.clone()), recording_session_id: None, meeting_name: Some("Sink".into()),
            created_at: "2026-10-13".into(), completed_at: None, duration_seconds: None,
            devices: DeviceInfo { microphone: None, system_audio: None }, audio_file: "audio.mp4".into(),
            transcript_file: "transcripts.json".into(), sample_rate: 48_000, audio_source: None, status: "completed".into(),
        };
        std::fs::write(folder.join("metadata.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
        std::fs::write(folder.join("audio.mp4"), "managed audio").unwrap();
        std::fs::write(folder.join("user-export.md"), "preserved export").unwrap();
        sqlx::query("INSERT INTO meetings(id,title,created_at,updated_at,status,folder_path) VALUES (?,'Sink','2026-10-13','2026-10-13','completed',?)")
            .bind(&id).bind(folder.to_str().unwrap()).execute(db.pool()).await.unwrap();
        let mut sink = RecordingSaver::new();
        sink.set_identity(Some(id.clone()), None);
        sink.meeting_folder = Some(folder.clone());
        let late_sink = sink.clone();
        let job = crate::database::deletion::begin(db.pool(), &id).await.unwrap();
        let result = crate::database::deletion::retry(db.pool(), &job.job_id, dir.path(), &[root]).await.unwrap();
        assert!(result.native_complete, "{:?}", result.last_error);
        assert!(folder.exists(), "Export retention deliberately keeps the directory writable");
        late_sink.add_transcript_chunk("late callback must not reach disk".into());
        assert!(late_sink.write_metadata(&folder, &metadata).is_err());
        assert!(!folder.join("metadata.json").exists());
        assert!(!folder.join("transcripts.json").exists());
        assert!(!folder.join(".transcripts.json.tmp").exists());
        assert_eq!(std::fs::read_to_string(folder.join("user-export.md")).unwrap(), "preserved export");
        db.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn configured_recordings_root_receives_actual_metadata_and_transcript_folder() {
        let dir = tempfile::tempdir().unwrap();
        let configured = dir.path().join("configured recordings");
        let mut saver = RecordingSaver::new();
        saver.set_recordings_folder(configured.clone());
        saver.set_identity(Some(format!("configured-{}", uuid::Uuid::new_v4())), Some("session-configured".into()));
        saver.set_meeting_name(Some("Configured root fixture".into()));
        let (sender, receiver) = crate::audio::queue::pcm_channel();
        saver.start_accumulation(false, receiver);
        let folder = saver.get_meeting_folder().unwrap();
        assert!(folder.starts_with(&configured));
        assert!(folder.join("metadata.json").is_file());
        let metadata: serde_json::Value = serde_json::from_slice(&std::fs::read(folder.join("metadata.json")).unwrap()).unwrap();
        assert_eq!(metadata["recording_session_id"], "session-configured");
        saver.add_transcript_chunk("Configured folder transcript".into());
        assert!(folder.join("transcripts.json").is_file());
        drop(sender);
    }
}

/// Meeting metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingMetadata {
    pub version: String,
    pub meeting_id: Option<String>,
    #[serde(default)]
    pub recording_session_id: Option<String>,
    pub meeting_name: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub duration_seconds: Option<f64>,
    pub devices: DeviceInfo,
    pub audio_file: String,
    pub transcript_file: String,
    pub sample_rate: u32,
    /// Provenance of the saved combined recording, distinct from live passages.
    #[serde(default)]
    pub audio_source: Option<super::source::AudioSource>,
    pub status: String,  // "recording", "completed", "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub microphone: Option<String>,
    pub system_audio: Option<String>,
}

/// New recording saver using incremental saving strategy
#[derive(Clone)]
pub struct RecordingSaver {
    incremental_saver: Option<Arc<AsyncMutex<IncrementalAudioSaver>>>,
    meeting_folder: Option<PathBuf>,
    meeting_name: Option<String>,
    meeting_id: Option<String>,
    recording_session_id: Option<String>,
    recordings_folder: Option<PathBuf>,
    metadata: Option<MeetingMetadata>,
    transcript_segments: Arc<Mutex<super::transcript_journal::TranscriptHistory>>,
    is_saving: Arc<Mutex<bool>>,
    accumulation: Arc<AsyncMutex<Option<tokio::task::JoinHandle<Result<()>>>>>,
}

impl RecordingSaver {
    pub fn new() -> Self {
        Self {
            incremental_saver: None,
            meeting_folder: None,
            meeting_name: None,
            meeting_id: None,
            recording_session_id: None,
            recordings_folder: None,
            metadata: None,
            transcript_segments: Arc::new(Mutex::new(super::transcript_journal::TranscriptHistory::default())),
            is_saving: Arc::new(Mutex::new(false)),
            accumulation: Arc::new(AsyncMutex::new(None)),
        }
    }

    /// Set the meeting name for this recording session
    pub fn set_identity(&mut self, meeting_id: Option<String>, session_id: Option<String>) {
        self.meeting_id = meeting_id;
        self.recording_session_id = session_id;
    }
    pub fn identity(&self) -> (Option<String>, Option<String>) {
        (self.meeting_id.clone(), self.recording_session_id.clone())
    }
    pub fn set_recordings_folder(&mut self, folder: PathBuf) {
        self.recordings_folder = Some(folder);
    }
    pub fn set_meeting_name(&mut self, name: Option<String>) {
        self.meeting_name = name;
    }

    /// Set device information in metadata
    pub fn set_device_info(&mut self, mic_name: Option<String>, sys_name: Option<String>) {
        if let Some(ref mut metadata) = self.metadata {
            metadata.devices.microphone = mic_name;
            metadata.devices.system_audio = sys_name;

            // Write updated metadata to disk if folder exists
            if let Some(folder) = &self.meeting_folder {
                let metadata_clone = metadata.clone();
                if let Err(e) = self.write_metadata(folder, &metadata_clone) {
                    warn!("Failed to update metadata with device info: {}", e);
                }
            }
        }
    }

    /// Add or update a structured transcript segment (upserts based on sequence_id)
    /// Also saves incrementally to disk
    pub fn add_transcript_segment(&self, segment: TranscriptSegment) {
        let result = (|| -> Result<()> {
            let _write = crate::database::deletion::recording_artifact_write(self.meeting_id.as_deref()).map_err(anyhow::Error::msg)?;
            let mut history = self.transcript_segments.lock().map_err(|_| anyhow::anyhow!("Transcript history lock failed"))?;
            history.record(segment, self.meeting_folder.as_deref(), self.meeting_id.as_deref(), self.recording_session_id.as_deref())
        })();
        if let Err(error) = result {
            warn!("Final transcript journal remains pending: {}", error);
            super::transcription::health::fail_for_identity(&(self.meeting_id.clone(), self.recording_session_id.clone()), error.to_string());
        }
    }

    /// Add an unstructured legacy passage using its generated sequence identity.

    pub fn add_transcript_chunk(&self, text: String) {
        let segment = TranscriptSegment {
            id: format!("seg_{}", chrono::Utc::now().timestamp_millis()),
            text,
            audio_start_time: 0.0,
            audio_end_time: 0.0,
            duration: 0.0,
            display_time: "[00:00]".to_string(),
            confidence: 1.0,
            sequence_id: 0,
            audio_source: None,
        };
        self.add_transcript_segment(segment);
    }

    /// Start accumulation with optional incremental saving
    ///
    /// # Arguments
    /// * `auto_save` - If true, creates checkpoints and enables saving. If false, audio chunks are discarded.
    pub fn start_accumulation(
        &mut self,
        auto_save: bool,
        mut receiver: crate::audio::queue::PcmReceiver<AudioChunk>,
    ) {
        if auto_save {
            info!("Initializing incremental audio saver for recording (auto-save ENABLED)");
        } else {
            info!("Starting recording without audio saving (auto-save DISABLED - transcripts only)");
        }

        // Initialize meeting folder and incremental saver ONLY if auto_save is enabled
        if auto_save {
            if let Some(name) = self.meeting_name.clone() {
                match self.initialize_meeting_folder(&name, true) {
                    Ok(()) => info!("Successfully initialized meeting folder with checkpoints"),
                    Err(e) => {
                        error!("Failed to initialize meeting folder: {}", e);
                        // Continue anyway - will use fallback flat structure
                    }
                }
            }
        } else {
            // When auto_save is false, still create meeting folder for transcripts/metadata
            // but skip .checkpoints directory
            if let Some(name) = self.meeting_name.clone() {
                match self.initialize_meeting_folder(&name, false) {
                    Ok(()) => info!("Successfully initialized meeting folder (transcripts only)"),
                    Err(e) => {
                        error!("Failed to initialize meeting folder: {}", e);
                    }
                }
            }
        }

        // Start accumulation task
        let is_saving_clone = self.is_saving.clone();
        let incremental_saver_arc = self.incremental_saver.clone();
        let save_audio = auto_save;
        let meeting_id = self.meeting_id.clone();

        if let Ok(mut saving) = self.is_saving.lock() { *saving = true; }
        let identity = self.identity();
        let worker = tokio::task::spawn_blocking(move || -> Result<()> {
            let result = (|| -> Result<()> {
                let mut encoding_failure: Option<String> = None;
                while let Some(chunk) = receiver.blocking_recv() {
                    if save_audio {
                        let saver = incremental_saver_arc.as_ref().ok_or_else(||anyhow::anyhow!("Audio saver was not initialized"))?;
                        let _write = crate::database::deletion::recording_artifact_write(meeting_id.as_deref()).map_err(anyhow::Error::msg)?;
                        let mut saver = saver.blocking_lock();
                        if encoding_failure.is_some() { saver.retain_chunk(chunk)?; }
                        else if let Err(error) = saver.add_chunk(chunk) {
                            encoding_failure = Some(error.to_string());
                            crate::audio::transcription::health::fail_for_identity(&identity, error.to_string());
                            saver.begin_retention_only();
                        }
                    }
                }
                if let Some(error) = receiver.error().or(encoding_failure) { return Err(anyhow::anyhow!(error)); }
                Ok(())
            })();
            if let Ok(mut saving) = is_saving_clone.lock() { *saving = false; }
            if let Err(error) = &result { crate::audio::transcription::health::fail_for_identity(&identity, error.to_string()); }
            result
        });
        *self.accumulation.try_lock().expect("New accumulation cannot already be locked") = Some(worker);
    }

    /// Initialize meeting folder structure and metadata
    ///
    /// # Arguments
    /// * `meeting_name` - Name of the meeting
    /// * `create_checkpoints` - Whether to create .checkpoints/ directory and IncrementalAudioSaver
    fn initialize_meeting_folder(&mut self, meeting_name: &str, create_checkpoints: bool) -> Result<()> {
        let _write = crate::database::deletion::recording_artifact_write(self.meeting_id.as_deref()).map_err(anyhow::Error::msg)?;
        // Load preferences to get base recordings folder
        let base_folder = self.recordings_folder.clone().unwrap_or_else(super::recording_preferences::get_default_recordings_folder);

        // Create meeting folder structure (with or without .checkpoints/ subdirectory)
        let meeting_folder = create_meeting_folder(&base_folder, &self.recording_session_id.as_ref().map(|id| format!("{}-{}", meeting_name, id)).unwrap_or_else(|| meeting_name.to_owned()), create_checkpoints)?;

        // Only initialize incremental saver if checkpoints are needed (auto_save is true)
        if create_checkpoints {
            let incremental_saver = IncrementalAudioSaver::new(meeting_folder.clone(), 48000)?;
            self.incremental_saver = Some(Arc::new(AsyncMutex::new(incremental_saver)));
            info!("✅ Incremental audio saver initialized for meeting: {}", meeting_name);
        } else {
            info!("⚠️  Skipped incremental audio saver (auto-save disabled)");
        }

        // Create initial metadata
        let metadata = MeetingMetadata {
            version: "1.0".to_string(),
            audio_source: Some(super::source::AudioSource::Mixed),
            meeting_id: self.meeting_id.clone(),
            recording_session_id: self.recording_session_id.clone(),
            meeting_name: Some(meeting_name.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            duration_seconds: None,
            devices: DeviceInfo {
                microphone: None,  // Could be enhanced to store actual device names
                system_audio: None,
            },
            audio_file: if create_checkpoints { "audio.mp4".to_string() } else { "".to_string() },
            transcript_file: "transcripts.json".to_string(),
            sample_rate: 48000,
            status: "recording".to_string(),
        };

        // The caller holds the async meeting lock through startup. Release the
        // synchronous lease before write_metadata obtains its own lease.
        drop(_write);
        // Write initial metadata.json
        self.write_metadata(&meeting_folder, &metadata)?;
        self.write_transcripts_json(&meeting_folder)?;

        self.meeting_folder = Some(meeting_folder);
        self.metadata = Some(metadata);

        Ok(())
    }

    /// Write metadata.json to disk (atomic write with temp file)
    fn write_metadata(&self, folder: &PathBuf, metadata: &MeetingMetadata) -> Result<()> {
        let _write = crate::database::deletion::recording_artifact_write(self.meeting_id.as_deref()).map_err(anyhow::Error::msg)?;
        let metadata_path = folder.join("metadata.json");
        let temp_path = folder.join(".metadata.json.tmp");

        let json_string = serde_json::to_string_pretty(metadata)?;
        std::fs::write(&temp_path, json_string)?;
        std::fs::rename(&temp_path, &metadata_path)?;  // Atomic

        Ok(())
    }

    /// Write transcripts.json to disk (atomic write with temp file and validation)
    fn write_transcripts_json(&self, folder: &PathBuf) -> Result<()> {
        let _write = crate::database::deletion::recording_artifact_write(self.meeting_id.as_deref()).map_err(anyhow::Error::msg)?;
        let history = self.transcript_segments.lock().map_err(|_| anyhow::anyhow!("Transcript history lock failed"))?;
        super::transcript_journal::publish(folder, &history.segments)
    }

    // in frontend/src-tauri/src/audio/recording_saver.rs
    pub fn get_stats(&self) -> (usize, u32) {
        if let Some(ref saver) = self.incremental_saver {
            if let Ok(guard) = saver.try_lock() {
                (guard.get_checkpoint_count() as usize, 48000)
            } else {
                (0, 48000)
            }
        } else {
            (0, 48000)
        }
    }

    /// Stop and save using incremental saving approach
    ///
    /// # Arguments
    /// * `app` - Tauri app handle for emitting events
    /// * `recording_duration` - Actual recording duration in seconds (from RecordingState)
    pub async fn stop_and_save<R: Runtime>(
        &mut self,
        app: &AppHandle<R>,
        recording_duration: Option<f64>
    ) -> Result<Option<String>, String> {
        info!("Stopping recording saver");
        let _artifact_write = if let Some(ref id) = self.meeting_id {
            let state = app.try_state::<crate::state::AppState>().ok_or("Database is not initialized")?;
            Some(crate::database::deletion::acquire_meeting_artifact_write(state.db_manager.pool(), id).await?)
        } else { None };

        // The pipeline has closed its producer. Await every accepted sample;
        // a timer or a flag cannot establish that its queue drained.
        if let Some(worker) = self.accumulation.lock().await.take() {
            worker.await.map_err(|e|format!("Audio saver worker failed: {e}"))?
                .map_err(|e|format!("Audio checkpoint failed; recovery retained: {e}"))?;
        }
        // Check if incremental saver exists (indicates auto_save was enabled)
        let should_save_audio = self.incremental_saver.is_some();

        // Save final transcripts.json with validation
        if let Some(folder) = &self.meeting_folder {
            if let Err(e) = self.write_transcripts_json(folder) {
                error!("❌ Failed to write final transcripts: {}", e);
                return Err(format!("Failed to save transcripts: {}", e));
            }

            // Verify transcripts were written correctly
            let transcript_path = folder.join("transcripts.json");
            if !transcript_path.exists() {
                error!("❌ Transcript file was not created at: {}", transcript_path.display());
                return Err("Transcript file verification failed".to_string());
            }
            info!("✅ Transcripts saved and verified at: {}", transcript_path.display());
        }


        if !should_save_audio {
            info!("⚠️  No audio saver initialized (auto-save was disabled) - skipping audio finalization");
            info!("Final transcript snapshot saved; audio saving is disabled");
            return Ok(None);
        }

        // Finalize incremental saver (merge checkpoints into final audio.mp4)
        let final_audio_path = if let Some(saver_arc) = &self.incremental_saver {
            let saver_arc = saver_arc.clone();
            let meeting_id = self.meeting_id.clone();
            tokio::task::spawn_blocking(move || {
                // This lease belongs to the blocking task, including after an
                // awaiting future is cancelled or times out.
                let _write = crate::database::deletion::recording_artifact_write(meeting_id.as_deref()).map_err(anyhow::Error::msg)?;
                let mut saver = saver_arc.blocking_lock();
                tokio::runtime::Handle::current().block_on(saver.finalize())
            }).await.map_err(|e|format!("Audio finalization worker failed: {e}"))?
                .map_err(|e|format!("Audio finalization failed; checkpoints retained: {e}"))?
        } else {
            error!("No incremental saver initialized - cannot save recording");
            return Err("No incremental saver initialized".to_string());
        };

        // Update metadata to completed status with actual recording duration
        if let (Some(folder), Some(mut metadata)) = (&self.meeting_folder, self.metadata.clone()) {
            metadata.status = "completed".to_string();
            metadata.completed_at = Some(chrono::Utc::now().to_rfc3339());

            // Use actual recording duration from RecordingState (more accurate than transcript segments)
            // Falls back to last transcript segment if duration not provided
            metadata.duration_seconds = recording_duration.or_else(|| {
                if let Ok(segments) = self.transcript_segments.lock() {
                    segments.last().map(|seg| seg.audio_end_time)
                } else {
                    None
                }
            });

            if let Err(e) = self.write_metadata(folder, &metadata) {
                error!("❌ Failed to update metadata to completed: {}", e);
                return Err(format!("Failed to update metadata: {}", e));
            }

            info!("✅ Metadata updated with duration: {:?}s", metadata.duration_seconds);
        }

        // Emit save event with audio and transcript paths
        let save_event = serde_json::json!({
            "audio_file": final_audio_path.to_string_lossy(),
            "transcript_file": self.meeting_folder.as_ref()
                .map(|f| f.join("transcripts.json").to_string_lossy().to_string()),
            "meeting_name": self.meeting_name,
            "meeting_folder": self.meeting_folder.as_ref()
                .map(|f| f.to_string_lossy().to_string())
        });

        if let Err(e) = app.emit("recording-saved", &save_event) {
            warn!("Failed to emit recording-saved event: {}", e);
        }

        // Native stop commits these segments to SQLite after audio finalization.
        // Retain them until the owning manager and sinks are dropped, so the
        // file save cannot erase the database/summary input before that commit.

        Ok(Some(final_audio_path.to_string_lossy().to_string()))
    }

    /// Get the meeting folder path (for passing to backend)
    pub fn get_meeting_folder(&self) -> Option<&PathBuf> {
        self.meeting_folder.as_ref()
    }

    /// Get accumulated transcript segments (for reload sync)
    pub fn get_transcript_segments(&self) -> Vec<TranscriptSegment> {
        if let Ok(segments) = self.transcript_segments.lock() {
            segments.segments.clone()
        } else {
            Vec::new()
        }
    }

    /// Get meeting name (for reload sync)
    pub fn get_meeting_name(&self) -> Option<String> {
        self.meeting_name.clone()
    }
}

impl Default for RecordingSaver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)] mod drain_tests {
    use super::*;
    #[tokio::test] async fn stop_joins_all_accepted_chunks_and_short_tail_before_finalization() {
        let directory=tempfile::tempdir().unwrap();let mut saver=RecordingSaver::new();
        saver.set_recordings_folder(directory.path().into());saver.set_meeting_name(Some("Drain fixture".into()));
        let (sender,receiver)=crate::audio::queue::pcm_channel();saver.start_accumulation(true,receiver);
        for i in 0..20 {sender.send(AudioChunk {data:vec![0.05;2400],sample_rate:48000,timestamp:i as f64*0.05,
            chunk_id:i,device_type:super::super::recording_state::DeviceType::Microphone}).unwrap();}
        drop(sender);let app=tauri::test::mock_app();
        let audio=saver.stop_and_save(app.handle(),Some(1.)).await.unwrap().unwrap();
        let decoded=super::super::decoder::decode_audio_file(std::path::Path::new(&audio)).unwrap();
        let seconds=decoded.samples.len() as f64/decoded.sample_rate as f64;
        assert!((seconds-1.).abs()<0.1,"Stop lost queued audio: {seconds}s");
        assert!(saver.accumulation.lock().await.is_none());
    }
}
