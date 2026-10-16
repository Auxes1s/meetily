use std::path::PathBuf;
use anyhow::{Result, anyhow};
use log::{info, warn, error};
use super::encode::encode_audio_slices;
use super::recording_state::AudioChunk;
use serde::{Serialize, Deserialize};

use super::ffmpeg::find_ffmpeg_path;

/// Audio data without device type (we only store mixed audio)
#[derive(Clone)]
struct AudioData {
    data: Vec<f32>,
    // sample_rate: u32,
}

/// Incremental audio saver that writes checkpoints every 30 seconds
/// to minimize memory usage and enable crash recovery
pub struct IncrementalAudioSaver {
    checkpoint_buffer: Vec<AudioData>,
    buffered_samples: usize,
    checkpoint_interval_samples: usize,  // 30s at 48kHz = 1,440,000 samples
    checkpoint_count: u32,
    checkpoints_dir: PathBuf,
    meeting_folder: PathBuf,
    sample_rate: u32,
}

impl IncrementalAudioSaver {
    /// Create a new incremental saver
    ///
    /// # Arguments
    /// * `meeting_folder` - Path to the meeting folder (contains .checkpoints/)
    /// * `sample_rate` - Sample rate of audio (typically 48000)
    pub fn new(meeting_folder: PathBuf, sample_rate: u32) -> Result<Self> {
        let checkpoints_dir = meeting_folder.join(".checkpoints");

        // Verify checkpoints directory exists
        if !checkpoints_dir.exists() {
            return Err(anyhow!("Checkpoints directory does not exist: {}", checkpoints_dir.display()));
        }

        Ok(Self {
            checkpoint_buffer: Vec::new(),
            buffered_samples: 0,
            checkpoint_interval_samples: sample_rate as usize * 30, // 30 seconds
            checkpoint_count: 0,
            checkpoints_dir,
            meeting_folder,
            sample_rate,
        })
    }

    /// Add an audio chunk to the buffer
    /// Automatically saves a checkpoint when buffer reaches 30 seconds
    pub fn add_chunk(&mut self, chunk: AudioChunk) -> Result<()> {
        self.append_raw(&chunk)?;
        let audio_data = AudioData {
            data: chunk.data,
            // sample_rate: chunk.sample_rate,
        };

        self.checkpoint_buffer.push(audio_data);

        self.buffered_samples += self.checkpoint_buffer.last().unwrap().data.len();
        // Save checkpoint when buffer reaches threshold (30 seconds)
        if self.buffered_samples >= self.checkpoint_interval_samples {
            self.save_checkpoint()?;
            self.checkpoint_buffer.clear();
            self.buffered_samples = 0;
        }

        Ok(())
    }

    fn append_raw(&self, chunk: &AudioChunk) -> Result<()> {
        use std::io::Write;
        let raw_path = self.checkpoints_dir.join(format!("audio_raw_{:03}.pcm", self.checkpoint_count));
        let mut options = std::fs::OpenOptions::new(); options.create(true).append(true);
        #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
        if raw_path.exists() { anyhow::ensure!(std::fs::symlink_metadata(&raw_path)?.file_type().is_file(), "Raw checkpoint is not a regular file"); }
        let mut raw = options.open(&raw_path)?;
        raw.write_all(bytemuck::cast_slice(&chunk.data))?; raw.sync_all()?;
        Ok(())
    }
    /// Once encoding fails, keep accepting durable PCM without accumulating
    /// another in-memory batch or repeatedly invoking a broken encoder.
    pub(crate) fn begin_retention_only(&mut self) {
        self.checkpoint_buffer.clear(); self.buffered_samples = 0; self.checkpoint_count += 1;
    }
    pub(crate) fn retain_chunk(&mut self, chunk: AudioChunk) -> Result<()> {
        self.append_raw(&chunk)?; self.buffered_samples += chunk.data.len();
        if self.buffered_samples >= self.checkpoint_interval_samples { self.checkpoint_count += 1; self.buffered_samples = 0; }
        Ok(())
    }

    /// Save current buffer as a checkpoint file
    fn save_checkpoint(&mut self) -> Result<()> {
        if self.buffered_samples == 0 { return Ok(()); }
        // Generate checkpoint filename
        let checkpoint_path = self.checkpoints_dir
            .join(format!("audio_chunk_{:03}.mp4", self.checkpoint_count));

        // Publish the encoded checkpoint only after FFmpeg succeeds.
        let temporary = checkpoint_path.with_extension("mp4.tmp");
        encode_audio_slices(
            self.checkpoint_buffer.iter().map(|chunk| bytemuck::cast_slice(chunk.data.as_slice())),
            self.sample_rate,
            1,  // mono
            &temporary
        )?;
        std::fs::File::open(&temporary)?.sync_all()?;
        std::fs::rename(&temporary, &checkpoint_path)?;
        #[cfg(unix)] std::fs::File::open(&self.checkpoints_dir)?.sync_all()?;
        std::fs::remove_file(self.checkpoints_dir.join(format!("audio_raw_{:03}.pcm", self.checkpoint_count)))?;

        let duration_seconds = self.buffered_samples as f32 / self.sample_rate as f32;
        self.checkpoint_count += 1;

        info!("Saved checkpoint {}: {:.2}s of audio ({} samples)",
              self.checkpoint_count,
              duration_seconds,
              self.buffered_samples);

        Ok(())
    }

    /// Finalize the recording: save final checkpoint, merge all checkpoints, cleanup
    ///
    /// Returns the path to the final merged audio.mp4 file
    pub async fn finalize(&mut self) -> Result<PathBuf> {
        info!("Finalizing incremental recording...");

        // Save final buffer if not empty
        if !self.checkpoint_buffer.is_empty() {
            info!("Saving final checkpoint with remaining {} chunks", self.checkpoint_buffer.len());
            self.save_checkpoint()?;
            self.checkpoint_buffer.clear();
            self.buffered_samples = 0;
        }

        if self.checkpoint_count == 0 {
            return Err(anyhow!("No audio checkpoints to merge - recording may have failed"));
        }

        // Merge all checkpoints using FFmpeg concat
        let final_audio_path = self.meeting_folder.join("audio.mp4");
        self.merge_checkpoints(&final_audio_path).await?;

        // The owning Stop transaction removes source checkpoints only after
        // canonical transcript acknowledgment; failures keep recovery assets.
        info!("Finalized recording: {}", final_audio_path.display());

        Ok(final_audio_path)
    }

    /// Merge all checkpoint files into final audio.mp4 using FFmpeg concat
    /// Uses concat demuxer for fast merging without re-encoding
    async fn merge_checkpoints(&self, output: &PathBuf) -> Result<()> {
        info!("Merging {} checkpoints into final audio file...", self.checkpoint_count);

        // Create concat list file for FFmpeg
        let list_file = self.checkpoints_dir.join("concat_list.txt");
        let mut list_content = String::new();

        for i in 0..self.checkpoint_count {
            let checkpoint_path = self.checkpoints_dir
                .join(format!("audio_chunk_{:03}.mp4", i));

            // Verify checkpoint exists
            if !checkpoint_path.exists() {
                return Err(anyhow!("Checkpoint file missing: {}", checkpoint_path.display()));
            }

            // Use absolute path for FFmpeg (required for safe mode)
            let abs_path = checkpoint_path.canonicalize()?;
            list_content.push_str(&format!("file '{}'\n", abs_path.display()));
        }

        std::fs::write(&list_file, list_content)?;

        let ffmpeg_path = find_ffmpeg_path()
            .ok_or_else(|| anyhow!("FFmpeg not found. Please install FFmpeg to finalize recordings."))?;
        info!("Using FFmpeg at: {:?}", ffmpeg_path);

        // Run FFmpeg concat command
        // Using concat demuxer with copy codec for fast merging (no re-encoding)
        
        let mut command = std::process::Command::new(ffmpeg_path);
        
        command.args(&[
            "-f", "concat",          // Use concat demuxer
            "-safe", "0",            // Allow absolute paths
            "-i", list_file.to_str().unwrap(),
            "-c", "copy",            // Copy codec - no re-encoding!
            "-y",                    // Overwrite output file
            output.to_str().unwrap()
        ]);

        // Hide console window on Windows to prevent CMD popup during finalization
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let ffmpeg_output = command.output()?;

        if !ffmpeg_output.status.success() {
            let stderr = String::from_utf8_lossy(&ffmpeg_output.stderr);
            error!("FFmpeg merge failed: {}", stderr);
            return Err(anyhow!("FFmpeg concat failed: {}", stderr));
        }

        // Verify output file was created
        if !output.exists() {
            return Err(anyhow!("Merged audio file was not created: {}", output.display()));
        }

        info!("Successfully merged {} checkpoints → {}",
              self.checkpoint_count, output.display());

        Ok(())
    }

    /// Get the meeting folder path
    pub fn get_meeting_folder(&self) -> &PathBuf {
        &self.meeting_folder
    }

    /// Get current checkpoint count
    pub fn get_checkpoint_count(&self) -> u32 {
        self.checkpoint_count
    }
}

/// Audio recovery status for transcript recovery feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRecoveryStatus {
    pub status: String, // "success" | "partial" | "failed" | "none"
    pub chunk_count: u32,
    pub estimated_duration_seconds: f64,
    pub audio_file_path: Option<String>,
    pub message: String,
}

/// Complete a durable mixed-audio tail left by termination or encoder failure.
fn recover_raw_checkpoints(root: &std::path::Path, sample_rate: u32) -> Result<()> {
    for entry in std::fs::read_dir(root)? {
        let entry = entry?; let name = entry.file_name(); let Some(name) = name.to_str() else { continue; };
        let Some(number) = name.strip_prefix("audio_raw_").and_then(|s|s.strip_suffix(".pcm"))
            .filter(|s|!s.is_empty() && s.bytes().all(|c|c.is_ascii_digit())) else { continue; };
        anyhow::ensure!(entry.file_type()?.is_file(), "Raw checkpoint is not a regular file");
        let target = root.join(format!("audio_chunk_{number}.mp4"));
        if target.exists() { anyhow::ensure!(std::fs::symlink_metadata(&target)?.file_type().is_file(), "Encoded checkpoint is not a regular file"); continue; }
        let length = entry.metadata()?.len();
        anyhow::ensure!(length > 0 && length % 4 == 0 && length <= 16 * 1024 * 1024, "Invalid raw checkpoint size");
        let pcm = std::fs::read(entry.path())?;
        let temporary = target.with_extension("mp4.tmp");
        if temporary.exists() { anyhow::ensure!(std::fs::symlink_metadata(&temporary)?.file_type().is_file(), "Encoded temporary checkpoint is not a regular file"); std::fs::remove_file(&temporary)?; }
        encode_audio_slices(std::iter::once(pcm.as_slice()), sample_rate, 1, &temporary)?;
        std::fs::File::open(&temporary)?.sync_all()?; std::fs::rename(&temporary, &target)?;
        #[cfg(unix)] std::fs::File::open(root)?.sync_all()?;
    }
    Ok(())
}

/// Recover audio from checkpoint files
/// This is called by the transcript recovery system to merge audio chunks after a crash
#[tauri::command]
pub async fn recover_audio_from_checkpoints<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, crate::state::AppState>,
    meeting_folder: String,
    meeting_id: Option<String>,
    _sample_rate: u32
) -> Result<AudioRecoveryStatus, String> {
    info!("Starting audio recovery for folder: {}", meeting_folder);

    let folder_path = PathBuf::from(&meeting_folder);
    let meeting_id = match meeting_id {
        Some(id) => id,
        None => sqlx::query_scalar::<_, String>("SELECT meeting_id FROM recording_sessions WHERE folder_path=? UNION SELECT id FROM meetings WHERE folder_path=?")
            .bind(&meeting_folder).bind(&meeting_folder).fetch_optional(state.db_manager.pool()).await.map_err(|e| e.to_string())?
            .ok_or("Recover the meeting's canonical identity before recovering audio")?,
    };
    let (_, roots) = crate::database::deletion::app_paths(&app).await?;
    let _artifact_write = crate::database::deletion::prepare_recovery_artifact_write(state.db_manager.pool(), &folder_path, &meeting_id, &roots).await?;
    let checkpoints_dir = folder_path.join(".checkpoints");

    // Check if checkpoints directory exists
    if !checkpoints_dir.exists() {
        info!("No checkpoints directory found at: {}", checkpoints_dir.display());
        return Ok(AudioRecoveryStatus {
            status: "none".to_string(),
            chunk_count: 0,
            estimated_duration_seconds: 0.0,
            audio_file_path: None,
            message: "No audio checkpoints found".to_string(),
        });
    }

    let raw_root = checkpoints_dir.clone();
    let metadata: serde_json::Value = serde_json::from_slice(&std::fs::read(folder_path.join("metadata.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let sample_rate = metadata.get("sample_rate").and_then(|v|v.as_u64()).unwrap_or(48000);
    if !(8000..=192000).contains(&sample_rate) { return Err("Invalid recording sample rate".into()); }
    let raw_meeting_id = meeting_id.clone();
    tokio::task::spawn_blocking(move || {
        let _write = crate::database::deletion::recording_artifact_write(Some(&raw_meeting_id)).map_err(anyhow::Error::msg)?;
        recover_raw_checkpoints(&raw_root, sample_rate as u32)
    }).await
        .map_err(|e|e.to_string())?.map_err(|e|e.to_string())?;

    // Scan for checkpoint files
    let mut checkpoint_files: Vec<_> = std::fs::read_dir(&checkpoints_dir)
        .map_err(|e| format!("Failed to read checkpoints directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().and_then(|s| s.to_str()) == Some("mp4")
        })
        .collect();

    if checkpoint_files.is_empty() {
        info!("No checkpoint files found in: {}", checkpoints_dir.display());
        return Ok(AudioRecoveryStatus {
            status: "none".to_string(),
            chunk_count: 0,
            estimated_duration_seconds: 0.0,
            audio_file_path: None,
            message: "No audio checkpoint files found".to_string(),
        });
    }

    // Sort by filename (audio_chunk_000.mp4, audio_chunk_001.mp4, etc.)
    checkpoint_files.sort_by_key(|entry| entry.path());

    let chunk_count = checkpoint_files.len() as u32;
    let estimated_duration = (chunk_count as f64) * 30.0; // 30 seconds per chunk

    info!("Found {} checkpoint files, estimated duration: {:.2}s", chunk_count, estimated_duration);

    // Create FFmpeg concat file
    let concat_file_path = checkpoints_dir.join("concat_list.txt");
    let mut concat_content = String::new();

    for entry in &checkpoint_files {
        let path = entry.path().canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {}", e))?;
        concat_content.push_str(&format!("file '{}'\n", path.display()));
    }

    std::fs::write(&concat_file_path, concat_content)
        .map_err(|e| format!("Failed to write concat file: {}", e))?;

    // Run FFmpeg to merge chunks
    let output_path = folder_path.join("audio.mp4");
    let output_path_str = output_path.to_str()
        .ok_or("Invalid output path")?
        .to_string();

    let ffmpeg_path = find_ffmpeg_path()
        .ok_or_else(|| "FFmpeg not found. Please install FFmpeg to recover audio.".to_string())?;
    info!("Using FFmpeg at: {:?}", ffmpeg_path);

    let mut command = std::process::Command::new(ffmpeg_path);

    command.args(&[
        "-f", "concat",
        "-safe", "0",
        "-i", concat_file_path.to_str().unwrap(),
        "-c", "copy",
        "-y", // Overwrite if exists
        &output_path_str
    ]);

    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let ffmpeg_result = command.output();

    match ffmpeg_result {
        Ok(output) if output.status.success() => {
            // Clean up concat file
            let _ = std::fs::remove_file(concat_file_path);

            info!("Successfully recovered audio: {}", output_path_str);

            Ok(AudioRecoveryStatus {
                status: "success".to_string(),
                chunk_count,
                estimated_duration_seconds: estimated_duration,
                audio_file_path: Some(output_path_str),
                message: format!("Successfully recovered {} audio chunks", chunk_count),
            })
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("FFmpeg recovery failed: {}", error);
            Ok(AudioRecoveryStatus {
                status: "failed".to_string(),
                chunk_count,
                estimated_duration_seconds: estimated_duration,
                audio_file_path: None,
                message: format!("FFmpeg failed: {}", error),
            })
        }
        Err(e) => {
            error!("Failed to run FFmpeg: {}", e);
            Ok(AudioRecoveryStatus {
                status: "failed".to_string(),
                chunk_count,
                estimated_duration_seconds: estimated_duration,
                audio_file_path: None,
                message: format!("Failed to run FFmpeg: {}", e),
            })
        }
    }
}

/// Clean up checkpoint files after successful recording or recovery
/// This command is called by the frontend after successful save to clean up checkpoint files
#[tauri::command]
pub async fn cleanup_checkpoints(meeting_folder: String) -> Result<(), String> {
    info!("Cleaning up checkpoints for folder: {}", meeting_folder);

    let folder_path = PathBuf::from(&meeting_folder);
    let checkpoints_dir = folder_path.join(".checkpoints");

    if checkpoints_dir.exists() {
        std::fs::remove_dir_all(&checkpoints_dir)
            .map_err(|e| format!("Failed to remove checkpoints directory: {}", e))?;
        info!("Successfully cleaned up checkpoints directory");
    } else {
        info!("No checkpoints directory to clean up");
    }

    Ok(())
}

/// Check if a meeting folder has audio checkpoint files
/// Returns true if .checkpoints/ directory exists and contains .mp4 files
#[tauri::command]
pub async fn has_audio_checkpoints(meeting_folder: String) -> Result<bool, String> {
    let folder_path = PathBuf::from(&meeting_folder);
    let checkpoints_dir = folder_path.join(".checkpoints");

    // Check if checkpoints directory exists
    if !checkpoints_dir.exists() {
        return Ok(false);
    }

    // Scan for .mp4 checkpoint files
    let has_mp4_files = std::fs::read_dir(&checkpoints_dir)
        .map_err(|e| format!("Failed to read checkpoints directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .any(|entry| {
            entry.path().extension().and_then(|s| s.to_str()) == Some("mp4")
        });

    Ok(has_mp4_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use super::super::recording_state::DeviceType;

    #[tokio::test]
    async fn test_checkpoint_creation() {
        // Create temp meeting folder
        let temp_dir = tempdir().unwrap();
        let meeting_folder = temp_dir.path().join("Test_Meeting");
        std::fs::create_dir_all(&meeting_folder).unwrap();
        std::fs::create_dir_all(meeting_folder.join(".checkpoints")).unwrap();

        let mut saver = IncrementalAudioSaver::new(
            meeting_folder.clone(),
            48000
        ).unwrap();

        // Add 60 seconds worth of audio (should create 2 checkpoints)
        for i in 0..120 {  // 120 chunks of 0.5s each
            let chunk = AudioChunk {
                data: vec![0.5f32; 24000],  // 0.5s at 48kHz
                sample_rate: 48000,
                timestamp: i as f64 * 0.5,  // timestamp in seconds
                chunk_id: i as u64,
                device_type: DeviceType::Microphone,
            };
            saver.add_chunk(chunk).unwrap();
        }

        // Verify 2 checkpoints created
        assert_eq!(saver.checkpoint_count, 2);

        // Finalize and verify merge
        let final_path = saver.finalize().await.unwrap();
        assert!(final_path.exists());

        // Source and encoded checkpoints outlive merge until canonical acknowledgment.
        assert!(meeting_folder.join(".checkpoints").exists());
    }

    #[tokio::test]
    async fn test_empty_recording() {
        let temp_dir = tempdir().unwrap();
        let meeting_folder = temp_dir.path().join("Empty_Test");
        std::fs::create_dir_all(&meeting_folder).unwrap();
        std::fs::create_dir_all(meeting_folder.join(".checkpoints")).unwrap();

        let mut saver = IncrementalAudioSaver::new(
            meeting_folder.clone(),
            48000
        ).unwrap();

        // Try to finalize without adding any chunks
        let result = saver.finalize().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No audio checkpoints"));
    }
}

#[cfg(test)] mod raw_recovery_tests {
    use super::*;
    #[test] fn unencoded_tail_is_durable_and_recovery_is_idempotent() {
        let directory=tempfile::tempdir().unwrap();let root=directory.path().join(".checkpoints");std::fs::create_dir(&root).unwrap();
        let mut saver=IncrementalAudioSaver::new(directory.path().into(),48000).unwrap();
        saver.add_chunk(AudioChunk {data:vec![0.1;24000],sample_rate:48000,timestamp:0.,chunk_id:0,device_type:super::super::recording_state::DeviceType::Microphone}).unwrap();
        let raw=root.join("audio_raw_000.pcm");assert_eq!(std::fs::metadata(&raw).unwrap().len(),96000);
        drop(saver);recover_raw_checkpoints(&root,48000).unwrap();
        let encoded=std::fs::read(root.join("audio_chunk_000.mp4")).unwrap();assert!(!encoded.is_empty());
        recover_raw_checkpoints(&root,48000).unwrap();assert_eq!(std::fs::read(root.join("audio_chunk_000.mp4")).unwrap(),encoded);
        assert!(raw.exists(),"Recovery retains PCM until canonical cleanup");
    }
}

#[cfg(test)] mod retention_tests {
    use super::*;
    #[test] fn encoder_failure_retention_keeps_pcm_without_growing_batch_memory() {
        let directory=tempfile::tempdir().unwrap();let root=directory.path().join(".checkpoints");std::fs::create_dir(&root).unwrap();
        let mut saver=IncrementalAudioSaver::new(directory.path().into(),48000).unwrap();
        let chunk=||AudioChunk {data:vec![0.2;24000],sample_rate:48000,timestamp:0.,chunk_id:0,device_type:super::super::recording_state::DeviceType::Microphone};
        saver.add_chunk(chunk()).unwrap();saver.begin_retention_only();
        for _ in 0..120 {saver.retain_chunk(chunk()).unwrap();assert!(saver.checkpoint_buffer.is_empty());}
        let retained:u64=std::fs::read_dir(root).unwrap().map(|entry|entry.unwrap().metadata().unwrap().len()).sum();
        assert_eq!(retained,121*24000*4);
    }
}

#[cfg(test)] mod encoder_failure_tests {
    use super::*;
    #[test] fn ffmpeg_failure_preserves_the_raw_recovery_tail() {
        let directory=tempfile::tempdir().unwrap();let raw=directory.path().join("audio_raw_000.pcm");
        let pcm=vec![0.1f32;4800];let bytes: &[u8]=bytemuck::cast_slice(&pcm);std::fs::write(&raw,bytes).unwrap();
        // Invalid encoder parameters exercise actual process failure and reaping,
        // without changing shared model paths, binaries or process environment.
        assert!(recover_raw_checkpoints(directory.path(),0).is_err());
        assert_eq!(std::fs::read(raw).unwrap(),bytes);
        assert!(!directory.path().join("audio_chunk_000.mp4").exists());
    }
}
