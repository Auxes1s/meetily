//! Session-scoped counters; content-free snapshots remain available during drain.
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, LazyLock, Mutex,
};
static CURRENT: LazyLock<Mutex<Option<Arc<Health>>>> = LazyLock::new(|| Mutex::new(None));
#[derive(Default)]
pub struct Health {
    pub identity: (Option<String>, Option<String>),
    pub queued: AtomicU64,
    pub completed: AtomicU64,
    pub pending_samples: AtomicU64,
    pub retained_bytes: AtomicU64,
    pub closed: AtomicBool,
    pub drained: AtomicBool,
    pub error: Mutex<Option<String>>,
    pub repairs: AtomicU64,
    pub last_activity: AtomicU64,
}
#[derive(Clone, Serialize, Default)]
pub struct Snapshot {
    pub meeting_id: Option<String>,
    pub recording_session_id: Option<String>,
    pub chunks_completed: u64,
    pub chunks_queued: u64,
    pub chunks_in_queue: u64,
    pub is_processing: bool,
    pub last_activity_ms: u64,
    pub in_memory_audio_bytes: u64,
    pub pending_audio_seconds: f64,
    pub retained_audio_bytes: u64,
    pub drained: bool,
    pub storage_error: Option<String>,
    pub continuity_repairs: u64,
}
impl Health {
    pub fn begin(identity: (Option<String>, Option<String>)) -> Arc<Self> {
        let health = Arc::new(Self {
            identity,
            ..Self::default()
        });
        *CURRENT.lock().unwrap() = Some(health.clone());
        health
    }
    pub fn touch(&self) {
        self.last_activity.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Ordering::SeqCst,
        );
    }
    pub fn complete(&self, samples: usize) {
        self.touch();
        self.completed.fetch_add(1, Ordering::SeqCst);
        self.pending_samples
            .fetch_sub(samples as u64, Ordering::SeqCst);
    }
    pub fn fail(&self, error: String) {
        *self.error.lock().unwrap() = Some(error);
    }
}
pub fn fail(error: String) {
    if let Some(h) = CURRENT.lock().unwrap().as_ref() {
        h.fail(error);
    }
}
pub fn snapshot() -> Snapshot {
    let current = CURRENT.lock().unwrap();
    let Some(h) = current.as_ref() else {
        return Snapshot::default();
    };
    let total = h.queued.load(Ordering::SeqCst);
    let completed = h.completed.load(Ordering::SeqCst);
    let queued = h
        .queued
        .load(Ordering::SeqCst)
        .saturating_sub(h.completed.load(Ordering::SeqCst));
    let storage_error = h.error.lock().unwrap().clone();
    Snapshot {
        meeting_id: h.identity.0.clone(),
        recording_session_id: h.identity.1.clone(),
        chunks_queued: total,
        chunks_completed: completed,
        chunks_in_queue: queued,
        is_processing: !h.drained.load(Ordering::SeqCst),
        in_memory_audio_bytes: crate::audio::queue::memory_bytes(),
        pending_audio_seconds: h.pending_samples.load(Ordering::SeqCst) as f64 / 16000.,
        retained_audio_bytes: h.retained_bytes.load(Ordering::SeqCst),
        drained: h.drained.load(Ordering::SeqCst),
        storage_error,
        continuity_repairs: h.repairs.load(Ordering::SeqCst),
        last_activity_ms: h.last_activity.load(Ordering::SeqCst),
    }
}

pub fn canonical_ack(session: &str) {
    if let Some(health) = CURRENT.lock().unwrap().as_ref() {
        if health.identity.1.as_deref() == Some(session) {
            health.retained_bytes.store(0, Ordering::SeqCst);
        }
    }
}

/// A delayed old-session failure cannot poison the current session's status.
pub fn fail_for_identity(identity: &(Option<String>, Option<String>), error: String) {
    if let Some(health) = CURRENT.lock().unwrap().as_ref() {
        if identity.1.is_some() && &health.identity == identity {
            health.fail(error);
        }
    }
}
