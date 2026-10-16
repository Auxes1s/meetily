//! Small admission/cancellation registry, independent of audio resource locks.
use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

static PENDING: Mutex<Option<Arc<Attempt>>> = Mutex::new(None);
static CANCELLED: std::sync::LazyLock<Mutex<HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

pub struct Attempt {
    pub session_id: String,
    pub meeting_id: Option<String>,
    cancelled: AtomicBool,
    published: AtomicBool,
    started: Instant,
}

pub struct Admission(pub Arc<Attempt>);
impl Admission {
    pub fn begin(session_id: String, meeting_id: Option<String>) -> Result<Self, String> {
        let mut pending = PENDING
            .lock()
            .map_err(|_| "Recording startup state is unavailable")?;
        if CANCELLED
            .lock()
            .map_err(|_| "Recording cancellation state is unavailable")?
            .remove(&session_id)
        {
            return Err("Recording start cancelled".into());
        }
        if pending.is_some() {
            return Err("Recording initialization is already in progress".into());
        }
        let attempt = Arc::new(Attempt {
            session_id,
            meeting_id,
            cancelled: AtomicBool::new(false),
            published: AtomicBool::new(false),
            started: Instant::now(),
        });
        *pending = Some(attempt.clone());
        Ok(Self(attempt))
    }
}
impl Drop for Admission {
    fn drop(&mut self) {
        if let Ok(mut pending) = PENDING.lock() {
            if pending.as_ref().is_some_and(|a| Arc::ptr_eq(a, &self.0)) {
                *pending = None;
            }
        }
    }
}
impl Attempt {
    pub fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err("Recording start cancelled".into());
        }
        if self.started.elapsed() >= Duration::from_secs(120) {
            return Err("Recording start timed out after 120s".into());
        }
        Ok(())
    }
    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
    pub fn publish<T>(&self, publish: impl FnOnce() -> T) -> Result<T, String> {
        let _pending = PENDING
            .lock()
            .map_err(|_| "Recording startup state is unavailable")?;
        self.check()?;
        let result = publish();
        self.published.store(true, Ordering::SeqCst);
        Ok(result)
    }
}

pub fn pending() -> Option<(Option<String>, String)> {
    PENDING
        .lock()
        .ok()?
        .as_ref()
        .map(|a| (a.meeting_id.clone(), a.session_id.clone()))
}

#[tauri::command]
pub async fn cancel_recording_start(recording_session_id: String) -> Result<bool, String> {
    let pending = PENDING
        .lock()
        .map_err(|_| "Recording startup state is unavailable")?;
    if let Some(attempt) = pending
        .as_ref()
        .filter(|a| a.session_id == recording_session_id)
    {
        if attempt.published.load(Ordering::SeqCst) {
            return Ok(false);
        }
        attempt.cancelled.store(true, Ordering::SeqCst);
        Ok(true)
    } else {
        // IPC dispatch can race cancellation; remember the nonce until admission.
        CANCELLED
            .lock()
            .map_err(|_| "Recording cancellation state is unavailable")?
            .insert(recording_session_id);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn admission_serializes_and_cancellation_is_identity_scoped() {
        let a = Admission::begin("first".into(), Some("meeting".into())).unwrap();
        assert!(Admission::begin("second".into(), None).is_err());
        assert!(!cancel_recording_start("foreign".into()).await.unwrap());
        assert!(a.0.check().is_ok());
        assert!(cancel_recording_start("first".into()).await.unwrap());
        assert!(a.0.check().is_err());
        drop(a);
        assert!(Admission::begin("second".into(), None).is_ok());
    }
}
