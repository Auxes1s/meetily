use crate::api::{TranscriptSearchResult, TranscriptSegment};
use chrono::Utc;
use sqlx::{Error as SqlxError, SqlitePool};
use tracing::{error, info};
use uuid::Uuid;

pub struct TranscriptsRepository;

impl TranscriptsRepository {
    /// Saves a new meeting and its associated transcript segments.
    /// This function uses a transaction to ensure that either both the meeting
    /// and all its transcripts are saved, or none of them are.
    pub async fn save_transcript(
        pool: &SqlitePool,
        meeting_title: &str,
        transcripts: &[TranscriptSegment],
        folder_path: Option<String>,
        existing_meeting_id: Option<String>,
        recording_session_id: Option<String>,
    ) -> Result<String, SqlxError> {
        let existing = existing_meeting_id.is_some();
        let meeting_id = existing_meeting_id.unwrap_or_else(|| format!("meeting-{}", Uuid::new_v4()));
        let recording_session_id = Some(recording_session_id.unwrap_or_else(||format!("legacy-{meeting_id}")));
        let duration_folder=folder_path.clone();
        let measured=tokio::task::spawn_blocking(move ||duration_folder.as_deref().and_then(crate::database::recording_sessions::stored_duration)).await
            .map_err(|e|SqlxError::Protocol(e.to_string()))?;
        let transcript_end=transcripts.iter().filter_map(|s|s.audio_end_time).filter(|n|n.is_finite() && *n>=0.0).max_by(f64::total_cmp);
        let duration=match (measured,transcript_end) { (Some(a),Some(b))=>Some(a.max(b)),(a,b)=>a.or(b) };
        let mut tx = pool.begin().await?;
        let now = Utc::now().to_rfc3339();
        if existing {
            let owner: Option<(String,String)>=sqlx::query_as("SELECT meeting_id,status FROM recording_sessions WHERE id=?")
                .bind(&recording_session_id).fetch_optional(&mut *tx).await?;
            if owner.as_ref().is_some_and(|(id,status)|id!=&meeting_id || matches!(status.as_str(),"failed"|"cancelled"|"starting")) {
                return Err(SqlxError::Protocol("Recording session ownership or lifecycle mismatch".into()));
            }
            let current:Option<Option<String>>=sqlx::query_scalar("SELECT recording_session_id FROM meetings WHERE id=? AND status <> 'deleting'")
                .bind(&meeting_id).fetch_optional(&mut *tx).await?;
            let current=current.ok_or(SqlxError::RowNotFound)?;
            if owner.is_none() && current.is_some() && current!=recording_session_id {
                return Err(SqlxError::Protocol("Meeting recording session mismatch".into()));
            }
            // A retry of an earlier session cannot finalize a later recording.
            if current.is_none() || current==recording_session_id {
                sqlx::query("UPDATE meetings SET updated_at=?,folder_path=COALESCE(?,folder_path),status='completed',ended_at=?,recording_session_id=? WHERE id=?")
                    .bind(&now).bind(&folder_path).bind(&now).bind(&recording_session_id).bind(&meeting_id).execute(&mut *tx).await?;
            }
        } else {
            sqlx::query("INSERT INTO meetings (id, title, created_at, updated_at, folder_path, recording_session_id) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(&meeting_id).bind(meeting_title).bind(&now).bind(&now).bind(&folder_path).bind(&recording_session_id).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT OR IGNORE INTO recording_sessions(id,meeting_id,ordinal,status,created_at,ended_at,duration_seconds,duration_basis,folder_path) SELECT ?,?,COALESCE(MAX(ordinal),0)+1,'completed',?,?,?,?,? FROM recording_sessions WHERE meeting_id=?")
            .bind(&recording_session_id).bind(&meeting_id).bind(&now).bind(&now).bind(duration)
            .bind(if measured.is_some(){"audio"}else if duration.is_some(){"transcript"}else{"unknown"}).bind(&folder_path).bind(&meeting_id)
            .execute(&mut *tx).await?;
        sqlx::query("UPDATE recording_sessions SET status='completed',ended_at=COALESCE(ended_at,?),duration_seconds=CASE WHEN duration_seconds IS NULL AND ? IS NULL THEN NULL ELSE MAX(COALESCE(duration_seconds,0),COALESCE(?,0)) END,duration_basis=CASE WHEN ? THEN 'audio' WHEN duration_basis='unknown' AND ? IS NOT NULL THEN 'transcript' ELSE duration_basis END,folder_path=COALESCE(?,folder_path) WHERE id=? AND meeting_id=?")
            .bind(&now).bind(duration).bind(duration).bind(measured.is_some()).bind(duration).bind(&folder_path).bind(&recording_session_id).bind(&meeting_id)
            .execute(&mut *tx).await?;
        for segment in transcripts {
            let sequence = segment.sequence_id;
            let transcript_id = if let (Some(session), Some(sequence)) = (&recording_session_id, sequence) {
                format!("transcript-{session}-{sequence}")
            } else {
                if segment.id.is_empty() {return Err(SqlxError::Protocol("Legacy passage identity is unavailable".into()));}
                let owner:Option<(String,Option<String>)>=sqlx::query_as("SELECT meeting_id,recording_session_id FROM transcripts WHERE id=?")
                    .bind(&segment.id).fetch_optional(&mut *tx).await?;
                match owner {
                    Some((owner,session)) if owner==meeting_id && session==recording_session_id=>segment.id.clone(),
                    Some(_)=>return Err(SqlxError::Protocol("Transcript identity belongs to another recording".into())),
                    None=>format!("transcript-{}-legacy-{}",recording_session_id.as_deref().unwrap(),segment.id),
                }
            };
            sqlx::query("INSERT INTO transcripts (id, meeting_id, transcript, timestamp, audio_start_time, audio_end_time, duration, recording_session_id, sequence_id, speaker) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET transcript = excluded.transcript, timestamp = excluded.timestamp, audio_start_time = COALESCE(excluded.audio_start_time, transcripts.audio_start_time), audio_end_time = COALESCE(excluded.audio_end_time, transcripts.audio_end_time), duration = COALESCE(excluded.duration, transcripts.duration), speaker = CASE WHEN excluded.speaker = 'unknown' THEN transcripts.speaker ELSE excluded.speaker END")
                .bind(&transcript_id).bind(&meeting_id).bind(&segment.text).bind(&segment.timestamp)
                .bind(segment.audio_start_time).bind(segment.audio_end_time).bind(segment.duration)
                .bind(&recording_session_id).bind(sequence).bind(segment.audio_source.unwrap_or_default().storage_value()).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE meeting_note_links SET transcript_id = (SELECT id FROM transcripts t WHERE t.meeting_id = meeting_note_links.meeting_id AND t.recording_session_id = meeting_note_links.recording_session_id AND t.audio_start_time <= meeting_note_links.recording_offset AND t.audio_end_time >= meeting_note_links.recording_offset ORDER BY t.audio_start_time DESC LIMIT 1) WHERE meeting_id = ?")
            .bind(&meeting_id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(meeting_id)
    }

    /// Searches for a query string within the transcripts.
    /// It returns a list of matching transcripts with context.
    pub async fn search_transcripts(
        pool: &SqlitePool,
        query: &str,
    ) -> Result<Vec<TranscriptSearchResult>, SqlxError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        if query.chars().count() > 512 || query.chars().any(char::is_control) {
            return Err(SqlxError::Protocol("Search requires at most 512 characters and no control characters".into()));
        }

        let literal = query.to_lowercase().replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let search_query = format!("%{literal}%");

        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT m.id, m.title, t.transcript, t.timestamp
             FROM meetings m
             JOIN transcripts t ON m.id = t.meeting_id
             WHERE m.status <> 'deleting' AND LOWER(t.transcript) LIKE ? ESCAPE '\\'
             ORDER BY m.created_at DESC, m.id, t.id LIMIT 100",
        )
        .bind(&search_query)
        .fetch_all(pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|(id, title, transcript, timestamp)| {
                let match_context = Self::get_match_context(&transcript, query);
                TranscriptSearchResult {
                    id,
                    title,
                    match_context,
                    timestamp,
                }
            })
            .collect();

        Ok(results)
    }

    /// Helper function to extract a snippet of text around the first match of a query.
    fn get_match_context(transcript: &str, query: &str) -> String {
        let characters: Vec<char> = transcript.chars().collect();
        let mut transcript_lower = String::new();
        let mut folded_offsets = Vec::with_capacity(characters.len());
        // Unicode lowercasing can change byte length (for example İ -> i + ◌̇).
        // Track original character positions instead of slicing original UTF-8
        // at byte offsets returned by a search of the lowercased string.
        for character in &characters {
            folded_offsets.push(transcript_lower.len());
            transcript_lower.extend(character.to_lowercase());
        }
        let query_lower = query.to_lowercase();

        match (!query_lower.is_empty()).then(|| transcript_lower.find(&query_lower)).flatten() {
            Some(match_index) => {
                let first_character = folded_offsets.partition_point(|offset| *offset <= match_index).saturating_sub(1);
                let last_character = folded_offsets.partition_point(|offset| *offset < match_index + query_lower.len());
                let start_index = first_character.saturating_sub(100);
                let end_index = (last_character + 100).min(characters.len());

                let mut context = String::new();
                if start_index > 0 {
                    context.push_str("...");
                }
                context.extend(&characters[start_index..end_index]);
                if end_index < characters.len() {
                    context.push_str("...");
                }
                context
            }
            None => transcript.chars().take(200).collect(), // Fallback to the start of the transcript
        }
    }
}

#[cfg(test)]
mod search_context_tests {
    use super::TranscriptsRepository;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn unicode_context_preserves_original_characters_and_character_boundaries() {
        let transcript = format!("{}İstanbul résumé技術{}", "語".repeat(120), "界".repeat(120));
        let context = TranscriptsRepository::get_match_context(&transcript, "İSTANBUL");
        assert_eq!(context, format!("...{}İstanbul résumé技術{}...", "語".repeat(100), "界".repeat(100 - " résumé技術".chars().count())));
        assert!(TranscriptsRepository::get_match_context(&transcript, "RÉSUMÉ技術").contains("İstanbul résumé技術"));
        assert_eq!(TranscriptsRepository::get_match_context("語界", "missing"), "語界");
        assert_eq!(TranscriptsRepository::get_match_context("語界", ""), "語界");
    }

    #[tokio::test]
    async fn compatibility_search_is_literal_bounded_and_excludes_pending_deletions() {
        let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::raw_sql("CREATE TABLE meetings(id TEXT PRIMARY KEY, title TEXT, created_at TEXT, status TEXT);
            CREATE TABLE transcripts(id TEXT PRIMARY KEY, meeting_id TEXT, transcript TEXT, timestamp TEXT);
            INSERT INTO meetings VALUES ('kept', 'Kept', '2026-10-13', 'completed'), ('deleted', 'Deleted', '2026-10-13', 'deleting');
            INSERT INTO transcripts VALUES ('literal', 'kept', 'literal 100%_\\ case', '00:00'), ('ordinary', 'kept', 'ordinary case', '00:01'), ('removed', 'deleted', 'literal 100%_\\ case', '00:00');")
            .execute(&pool).await.unwrap();
        let literal = TranscriptsRepository::search_transcripts(&pool, "%_\\").await.unwrap();
        assert_eq!(literal.len(), 1);
        assert_eq!(literal[0].id, "kept");
        for index in 0..125 {
            sqlx::query("INSERT INTO transcripts VALUES (?, 'kept', 'bounded', '00:02')")
                .bind(format!("bounded-{index}")).execute(&pool).await.unwrap();
        }
        assert_eq!(TranscriptsRepository::search_transcripts(&pool, "bounded").await.unwrap().len(), 100);
        assert!(TranscriptsRepository::search_transcripts(&pool, &"a".repeat(513)).await.is_err());
    }
}

#[cfg(test)]
mod audio_source_tests {
    use super::*;
    use crate::audio::source::AudioSource;
    use crate::database::models::Transcript;

    #[tokio::test]
    async fn source_metadata_round_trips_and_retries_do_not_erase_it() {
        let directory = tempfile::tempdir().unwrap();
        let database_url = format!("sqlite://{}?mode=rwc", directory.path().join("source-fixture.sqlite").display());
        let pool = SqlitePool::connect(&database_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let sources = [AudioSource::Microphone, AudioSource::System, AudioSource::Mixed, AudioSource::Unknown];
        let segments: Vec<_> = sources.iter().enumerate().map(|(index, source)| TranscriptSegment {
            id: format!("fixture-{index}"), text: format!("speech-{index}"), timestamp: "12:00:00".into(),
            audio_start_time: Some(index as f64), audio_end_time: Some(index as f64 + 0.5), duration: Some(0.5),
            sequence_id: Some(index as i64), audio_source: Some(*source),
        }).collect();
        let id = TranscriptsRepository::save_transcript(&pool, "Fixture", &segments, None, None, Some("session".into())).await.unwrap();
        // A legacy retry may omit the additive source field.
        let retries: Vec<_> = segments.iter().enumerate().map(|(index, segment)| TranscriptSegment {
            id: segment.id.clone(), text: format!("final-{index}"), timestamp: segment.timestamp.clone(),
            audio_start_time: segment.audio_start_time, audio_end_time: segment.audio_end_time, duration: segment.duration,
            sequence_id: segment.sequence_id, audio_source: None,
        }).collect();
        TranscriptsRepository::save_transcript(&pool, "Fixture", &retries, None, Some(id.clone()), Some("session".into())).await.unwrap();
        pool.close().await;
        let pool = SqlitePool::connect(&database_url).await.unwrap();
        let rows = sqlx::query_as::<_, Transcript>("SELECT * FROM transcripts WHERE meeting_id = ? ORDER BY sequence_id")
            .bind(&id).fetch_all(&pool).await.unwrap();
        assert_eq!(rows.len(), 4);
        for (index, row) in rows.iter().enumerate() {
            assert_eq!(AudioSource::from_storage(row.speaker.as_deref()), sources[index]);
            assert_eq!(row.transcript, format!("final-{index}"));
            assert_eq!(row.audio_start_time, Some(index as f64));
            assert_eq!(row.id, format!("transcript-session-{index}"));
            let json = serde_json::to_value(row).unwrap();
            assert_eq!(json["audio_source"], serde_json::to_value(sources[index]).unwrap());
            assert!(json.get("speaker").is_none());
            let restored: Transcript = serde_json::from_value(json).unwrap();
            assert_eq!(AudioSource::from_storage(restored.speaker.as_deref()), sources[index]);
        }
    }
}
