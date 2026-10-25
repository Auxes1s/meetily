//! Final passages are appended durably once. A compatible atomic JSON snapshot
//! is published at initialization and Stop; recovery overlays the journal.
use super::recording_saver::TranscriptSegment;
use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
};
pub(super) const JOURNAL: &str = "transcripts.journal.jsonl";
#[derive(Serialize, Deserialize)]
struct Entry {
    version: u32,
    meeting_id: Option<String>,
    recording_session_id: Option<String>,
    segment: TranscriptSegment,
}
#[derive(Default)]
pub(super) struct TranscriptHistory {
    pub segments: Vec<TranscriptSegment>,
    positions: HashMap<u64, usize>,
    journal: Option<File>,
}
impl std::ops::Deref for TranscriptHistory {
    type Target = [TranscriptSegment];
    fn deref(&self) -> &Self::Target {
        &self.segments
    }
}
impl TranscriptHistory {
    fn upsert(&mut self, mut segment: TranscriptSegment) -> usize {
        if let Some(&index) = self.positions.get(&segment.sequence_id) {
            if segment.audio_source.unwrap_or_default() == super::source::AudioSource::Unknown {
                segment.audio_source = self.segments[index].audio_source.or(segment.audio_source);
            }
            self.segments[index] = segment;
            index
        } else {
            let index = self.segments.len();
            self.positions.insert(segment.sequence_id, index);
            self.segments.push(segment);
            index
        }
    }
    pub fn record(
        &mut self,
        segment: TranscriptSegment,
        folder: Option<&Path>,
        meeting: Option<&str>,
        session: Option<&str>,
    ) -> Result<()> {
        let index = self.upsert(segment);
        let Some(folder) = folder else {
            return Ok(());
        };
        if self.journal.is_none() {
            let mut options = OpenOptions::new();
            options.append(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            self.journal = Some(options.open(folder.join(JOURNAL))?);
            #[cfg(unix)]
            File::open(folder)?.sync_all()?;
        }
        let entry = Entry {
            version: 1,
            meeting_id: meeting.map(str::to_owned),
            recording_session_id: session.map(str::to_owned),
            segment: self.segments[index].clone(),
        };
        let mut bytes = serde_json::to_vec(&entry)?;
        bytes.push(b'\n');
        let file = self.journal.as_mut().unwrap();
        let before = file.metadata()?.len();
        if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_data()) {
            // Do not append another record behind a torn line on a later retry.
            if file.set_len(before).is_err() {
                self.journal = None;
            }
            return Err(error.into());
        }
        Ok(())
    }
}
#[derive(Serialize)]
struct Snapshot<'a> {
    version: &'static str,
    segments: &'a [TranscriptSegment],
    last_updated: String,
    total_segments: usize,
}
pub(super) fn publish(folder: &Path, segments: &[TranscriptSegment]) -> Result<()> {
    let mut temporary = tempfile::Builder::new()
        .prefix(".transcripts.json.")
        .suffix(".tmp")
        .tempfile_in(folder)?;
    {
        let mut writer = BufWriter::new(temporary.as_file_mut());
        serde_json::to_writer_pretty(
            &mut writer,
            &Snapshot {
                version: "1.1",
                segments,
                last_updated: chrono::Utc::now().to_rfc3339(),
                total_segments: segments.len(),
            },
        )?;
        writer.flush()?;
    }
    temporary.as_file().sync_all()?;
    temporary
        .persist(folder.join("transcripts.json"))
        .map_err(|e| e.error)?;
    #[cfg(unix)]
    File::open(folder)?.sync_all()?;
    Ok(())
}
fn regular(path: &Path) -> Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            ensure!(
                metadata.file_type().is_file(),
                "Transcript artifact must be a regular file"
            );
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
pub(super) fn recover(
    folder: &Path,
    meeting: Option<&str>,
    session: Option<&str>,
) -> Result<Vec<TranscriptSegment>> {
    let mut history = TranscriptHistory::default();
    let snapshot = folder.join("transcripts.json");
    if regular(&snapshot)? {
        let value: serde_json::Value =
            serde_json::from_reader(BufReader::new(File::open(snapshot)?))?;
        for segment in serde_json::from_value::<Vec<TranscriptSegment>>(
            value
                .get("segments")
                .cloned()
                .ok_or_else(|| anyhow!("Transcript snapshot has no segments"))?,
        )? {
            history.upsert(segment);
        }
    }
    let path = folder.join(JOURNAL);
    if regular(&path)? {
        let mut reader = BufReader::new(File::open(path)?);
        loop {
            let mut line = Vec::new();
            let bytes = (&mut reader)
                .take(16 * 1024 * 1024 + 1)
                .read_until(b'\n', &mut line)?;
            if bytes == 0 {
                break;
            }
            ensure!(
                bytes <= 16 * 1024 * 1024,
                "Transcript journal record exceeds recovery limit"
            );
            if line.last() != Some(&b'\n') {
                break;
            } // Unacknowledged crash tail.
            let entry: Entry = serde_json::from_slice(&line)?;
            ensure!(
                entry.version == 1
                    && entry.meeting_id.as_deref() == meeting
                    && entry.recording_session_id.as_deref() == session,
                "Transcript journal ownership mismatch"
            );
            history.upsert(entry.segment);
        }
    }
    Ok(history.segments)
}

#[tauri::command]
pub async fn recover_transcript_history<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: tauri::State<'_, crate::state::AppState>,
    meeting_folder: String,
    meeting_id: String,
) -> Result<Vec<TranscriptSegment>, String> {
    let folder = std::path::PathBuf::from(meeting_folder);
    let (_, roots) = crate::database::deletion::app_paths(&app).await?;
    let lease = crate::database::deletion::prepare_recovery_artifact_write(
        state.db_manager.pool(),
        &folder,
        &meeting_id,
        &roots,
    )
    .await?;
    let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meetings WHERE id=? AND status IN ('recording','finalizing')) OR EXISTS(SELECT 1 FROM recording_sessions WHERE meeting_id=? AND status IN ('starting','recording','finalizing'))")
        .bind(&meeting_id).bind(&meeting_id).fetch_one(state.db_manager.pool()).await.map_err(|e|e.to_string())?;
    if active { return Err("Finish recording and finalization before recovering transcripts".into()); }
    tokio::task::spawn_blocking(move || {
        let _lease = lease;
        let _write = crate::database::deletion::recording_artifact_write(Some(&meeting_id))
            .map_err(anyhow::Error::msg)?;
        let metadata: serde_json::Value =
            serde_json::from_reader(File::open(folder.join("metadata.json"))?)?;
        recover(
            &folder,
            metadata.get("meeting_id").and_then(|v| v.as_str()),
            metadata
                .get("recording_session_id")
                .and_then(|v| v.as_str()),
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn segment(sequence: u64, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            id: format!("s-{sequence}"),
            text: text.into(),
            audio_start_time: sequence as f64,
            audio_end_time: sequence as f64 + 1.,
            duration: 1.,
            display_time: "00:00".into(),
            confidence: 0.9,
            sequence_id: sequence,
            audio_source: Some(super::super::source::AudioSource::Microphone),
        }
    }
    #[test]
    fn journal_replays_final_updates_and_torn_tail_without_losing_source() {
        let directory = tempfile::tempdir().unwrap();
        publish(directory.path(), &[]).unwrap();
        let mut history = TranscriptHistory::default();
        history
            .record(
                segment(1, "first"),
                Some(directory.path()),
                Some("m"),
                Some("s"),
            )
            .unwrap();
        let mut retry = segment(1, "updated");
        retry.audio_source = None;
        history
            .record(retry, Some(directory.path()), Some("m"), Some("s"))
            .unwrap();
        history
            .record(
                segment(2, "second"),
                Some(directory.path()),
                Some("m"),
                Some("s"),
            )
            .unwrap();
        drop(history);
        OpenOptions::new()
            .append(true)
            .open(directory.path().join(JOURNAL))
            .unwrap()
            .write_all(b"{unfinished")
            .unwrap();
        let recovered = recover(directory.path(), Some("m"), Some("s")).unwrap();
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].text, "updated");
        assert!(recovered[0].audio_source.is_some());
        assert!(recover(directory.path(), Some("other"), Some("s")).is_err());
        publish(directory.path(), &recovered).unwrap();
        assert_eq!(
            recover(directory.path(), Some("m"), Some("s"))
                .unwrap()
                .len(),
            2
        );
    }
    #[test]
    fn journal_size_scales_with_passages_not_snapshots() {
        let directory = tempfile::tempdir().unwrap();
        let mut history = TranscriptHistory::default();
        for i in 0..1000 {
            history
                .record(
                    segment(i, "fixture passage"),
                    Some(directory.path()),
                    Some("m"),
                    Some("s"),
                )
                .unwrap();
        }
        assert!(
            std::fs::metadata(directory.path().join(JOURNAL))
                .unwrap()
                .len()
                < 600_000
        );
        assert_eq!(
            recover(directory.path(), Some("m"), Some("s"))
                .unwrap()
                .len(),
            1000
        );
    }
}
