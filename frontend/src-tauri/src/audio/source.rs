use serde::{Deserialize, Serialize};

/// Capture-channel provenance, never a participant identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioSource {
    Microphone,
    System,
    Mixed,
    #[default]
    Unknown,
}

impl<'de> Deserialize<'de> for AudioSource {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Unsupported provenance must not make an otherwise valid legacy
        // passage disappear during save/recovery.
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::from_storage(value.as_str()))
    }
}

impl AudioSource {
    /// Adapter for the shipped nullable `transcripts.speaker` column.
    pub fn from_storage(value: Option<&str>) -> Self {
        match value {
            Some("mic" | "microphone") => Self::Microphone,
            Some("system") => Self::System,
            Some("mixed") => Self::Mixed,
            _ => Self::Unknown,
        }
    }

    pub fn storage_value(self) -> &'static str {
        match self {
            Self::Microphone => "mic",
            Self::System => "system",
            Self::Mixed => "mixed",
            Self::Unknown => "unknown",
        }
    }

    /// Saved recorder files combine capture channels. Arbitrary imports and
    /// absent metadata cannot establish a microphone/system origin.
    pub fn recording_file_source(folder: &std::path::Path) -> Self {
        let metadata = std::fs::read(folder.join("metadata.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
        match metadata {
            Some(value) if value.get("audio_source").and_then(|s| s.as_str()) == Some("mixed") => {
                Self::Mixed
            }
            Some(value)
                if value.get("source").and_then(|s| s.as_str()) != Some("import")
                    && value
                        .get("devices")
                        .is_some_and(|devices| devices.is_object()) =>
            {
                Self::Mixed
            }
            _ => Self::Unknown,
        }
    }
}

/// Keep raw SQLite storage names out of model JSON used by export/inspection.
pub fn serialize_storage_source<S: serde::Serializer>(
    value: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    AudioSource::from_storage(value.as_deref()).serialize(serializer)
}

pub fn deserialize_storage_source<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(Option::<AudioSource>::deserialize(deserializer)?
        .map(|source| source.storage_value().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_storage_and_json_round_trip() {
        for source in [
            AudioSource::Microphone,
            AudioSource::System,
            AudioSource::Mixed,
            AudioSource::Unknown,
        ] {
            assert_eq!(
                AudioSource::from_storage(Some(source.storage_value())),
                source
            );
            let json = serde_json::to_string(&source).unwrap();
            assert_eq!(serde_json::from_str::<AudioSource>(&json).unwrap(), source);
        }
        assert_eq!(
            serde_json::to_string(&AudioSource::Microphone).unwrap(),
            "\"microphone\""
        );
        assert_eq!(
            AudioSource::from_storage(Some("microphone")),
            AudioSource::Microphone
        );
    }

    #[test]
    fn legacy_and_unreliable_sources_stay_unknown() {
        for value in [None, Some("Audio"), Some("Alice"), Some("unsupported")] {
            assert_eq!(AudioSource::from_storage(value), AudioSource::Unknown);
        }
        assert_eq!(
            serde_json::from_str::<AudioSource>("\"Audio\"").unwrap(),
            AudioSource::Unknown
        );
        let legacy: crate::api::TranscriptSegment =
            serde_json::from_str(r#"{"id":"legacy","text":"old passage","timestamp":"12:00:00"}"#)
                .unwrap();
        assert_eq!(
            legacy.audio_source.unwrap_or_default(),
            AudioSource::Unknown
        );
        for value in [
            serde_json::json!(1),
            serde_json::json!({"device":"mic"}),
            serde_json::json!("Audio"),
        ] {
            let passage: crate::api::TranscriptSegment = serde_json::from_value(serde_json::json!({
                "id":"legacy", "text":"retained passage", "timestamp":"12:00:00", "audio_source":value
            })).unwrap();
            assert_eq!(
                passage.audio_source.unwrap_or_default(),
                AudioSource::Unknown
            );
            assert_eq!(passage.text, "retained passage");
        }
    }

    #[test]
    fn combined_recordings_and_unclassified_imports_have_truthful_sources() {
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(
            AudioSource::recording_file_source(folder.path()),
            AudioSource::Unknown
        );
        std::fs::write(
            folder.path().join("metadata.json"),
            r#"{"devices":{"microphone":"fixture","system_audio":"fixture"}}"#,
        )
        .unwrap();
        assert_eq!(
            AudioSource::recording_file_source(folder.path()),
            AudioSource::Mixed
        );
        std::fs::write(
            folder.path().join("metadata.json"),
            r#"{"source":"import"}"#,
        )
        .unwrap();
        assert_eq!(
            AudioSource::recording_file_source(folder.path()),
            AudioSource::Unknown
        );
    }
}
