//! Native routing preference shared by the WebView and tray recording commands.
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "transcription_preferences.json";
const CONTINUOUS_KEY: &str = "continuous_whisper";

fn stored_preference(value: Option<serde_json::Value>) -> Result<bool, String> {
    match value {
        None => Ok(true),
        Some(serde_json::Value::Bool(enabled)) => Ok(enabled),
        Some(_) => Err("Continuous transcription preference must be a boolean".into()),
    }
}

/// Read once at recording start; the pipeline and worker keep the same choice.
pub fn continuous_whisper_enabled<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let store = app
        .store_builder(STORE_PATH)
        .disable_auto_save()
        .build()
        .map_err(|error| error.to_string())?;
    stored_preference(store.get(CONTINUOUS_KEY))
}

#[tauri::command]
pub fn get_continuous_transcription<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    continuous_whisper_enabled(&app)
}

#[tauri::command]
pub fn set_continuous_transcription<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<(), String> {
    let store = app
        .store_builder(STORE_PATH)
        .disable_auto_save()
        .build()
        .map_err(|error| error.to_string())?;
    let previous = store.get(CONTINUOUS_KEY);
    store.set(CONTINUOUS_KEY, serde_json::Value::Bool(enabled));
    if let Err(error) = store.save() {
        // A rejected save must not silently change the next recording's route.
        match previous {
            Some(value) => store.set(CONTINUOUS_KEY, value),
            None => {
                store.delete(CONTINUOUS_KEY);
            }
        }
        return Err(format!(
            "Could not save continuous transcription preference: {error}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_preference_enables_continuous_transcription() {
        assert!(stored_preference(None).unwrap());
    }

    #[test]
    fn explicit_choices_survive_reload() {
        for enabled in [false, true] {
            let json = serde_json::to_string(&serde_json::Value::Bool(enabled)).unwrap();
            assert_eq!(
                stored_preference(Some(serde_json::from_str(&json).unwrap())).unwrap(),
                enabled
            );
        }
    }

    #[test]
    fn invalid_preference_is_not_silently_enabled() {
        assert!(stored_preference(Some(serde_json::json!("false"))).is_err());
    }
}
