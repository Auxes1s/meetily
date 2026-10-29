//! Decorative recording bars reuse existing chunk metrics; never scan audio here.
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::recording_state::DeviceType;

// The UI subscribes only while the recording controls are visible.
static MONITORING_ENABLED: AtomicBool = AtomicBool::new(false);
static WINDOW_REVISION: AtomicU32 = AtomicU32::new(0);

pub fn set_monitoring_enabled(enabled: bool) {
    if MONITORING_ENABLED.swap(enabled, Ordering::SeqCst) != enabled {
        invalidate_window();
    }
}

pub fn monitoring_enabled() -> bool {
    MONITORING_ENABLED.load(Ordering::SeqCst)
}

pub fn invalidate_window() {
    WINDOW_REVISION.fetch_add(1, Ordering::SeqCst);
}

pub fn window_revision() -> u32 {
    WINDOW_REVISION.load(Ordering::SeqCst)
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RecordingAudioLevel {
    /// Maximum existing mean-absolute chunk level during this display interval.
    pub microphone: f32,
    pub system: f32,
}

pub type RecordingLevelCallback = Arc<dyn Fn(RecordingAudioLevel) + Send + Sync>;

/// Runs on the existing pipeline task; no extra stream, worker, or polling task.
pub struct RecordingLevelMonitor {
    callback: RecordingLevelCallback,
    levels: RecordingAudioLevel,
    revision: u32,
    last_update: Instant,
}

impl RecordingLevelMonitor {
    pub fn new(callback: RecordingLevelCallback) -> Self {
        Self {
            callback,
            levels: RecordingAudioLevel::default(),
            revision: 0,
            last_update: Instant::now(),
        }
    }

    fn sync_revision(&mut self, revision: u32) {
        if self.revision != revision {
            self.levels = RecordingAudioLevel::default();
            self.revision = revision;
            self.last_update = Instant::now();
        }
    }

    pub fn observe(&mut self, source: &DeviceType, average_level: f32, revision: u32) {
        self.sync_revision(revision);
        let level = if average_level.is_finite() {
            average_level.clamp(0.0, 1.0)
        } else {
            0.0
        };
        match source {
            DeviceType::Microphone => self.levels.microphone = self.levels.microphone.max(level),
            DeviceType::System => self.levels.system = self.levels.system.max(level),
        }
    }

    pub fn update(&mut self, active: bool, revision: u32) {
        // A revision catches a pause/resume even when both happen between polls.
        self.sync_revision(revision);
        if !active {
            self.levels = RecordingAudioLevel::default();
            return;
        }
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(200) {
            (self.callback)(std::mem::take(&mut self.levels));
            self.last_update = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn monitor() -> (RecordingLevelMonitor, Arc<Mutex<Vec<RecordingAudioLevel>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let monitor = RecordingLevelMonitor::new(Arc::new(move |level| {
            captured.lock().unwrap().push(level);
        }));
        (monitor, events)
    }

    fn tick(monitor: &mut RecordingLevelMonitor, active: bool, revision: u32) {
        monitor.last_update = Instant::now() - Duration::from_millis(201);
        monitor.update(active, revision);
    }

    #[test]
    fn reuses_chunk_metrics_and_keeps_sources_independent() {
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, 0.5, 0);
        monitor.observe(&DeviceType::Microphone, 0.1, 0);
        monitor.observe(&DeviceType::System, 0.25, 0);
        tick(&mut monitor, true, 0);
        let events = events.lock().unwrap();
        assert_eq!(events[0].microphone, 0.5);
        assert_eq!(events[0].system, 0.25);
    }

    #[test]
    fn silence_and_missing_metrics_emit_zero() {
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, 0.5, 0);
        tick(&mut monitor, true, 0);
        monitor.observe(&DeviceType::Microphone, 0.0, 0);
        tick(&mut monitor, true, 0);
        let events = events.lock().unwrap();
        assert_eq!(events[1].microphone, 0.0);
        assert_eq!(events[1].system, 0.0);
    }

    #[test]
    fn bounds_invalid_metrics() {
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, f32::NAN, 0);
        monitor.observe(&DeviceType::Microphone, f32::INFINITY, 0);
        monitor.observe(&DeviceType::Microphone, -1.0, 0);
        monitor.observe(&DeviceType::System, 2.0, 0);
        tick(&mut monitor, true, 0);
        let events = events.lock().unwrap();
        assert_eq!(events[0].microphone, 0.0);
        assert_eq!(events[0].system, 1.0);
    }

    #[test]
    fn throttles_events_and_does_not_emit_when_hidden_or_paused() {
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, 0.5, 0);
        monitor.update(true, 0);
        assert!(events.lock().unwrap().is_empty());
        tick(&mut monitor, false, 0);
        assert!(events.lock().unwrap().is_empty());
        tick(&mut monitor, true, 0);
        assert_eq!(events.lock().unwrap()[0].microphone, 0.0);
    }

    #[test]
    fn rapid_pause_resume_discards_pre_pause_levels_without_observing_inactive_state() {
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, 0.8, 0);
        monitor.observe(&DeviceType::System, 0.9, 0);
        // Pause invalidates the revision, then resume happens before update runs.
        monitor.update(true, 1);
        assert!(events.lock().unwrap().is_empty());
        tick(&mut monitor, true, 1);
        assert_eq!(events.lock().unwrap()[0].microphone, 0.0);
        assert_eq!(events.lock().unwrap()[0].system, 0.0);
        monitor.observe(&DeviceType::System, 0.25, 1);
        tick(&mut monitor, true, 1);
        assert_eq!(events.lock().unwrap()[1].system, 0.25);
    }

    #[test]
    fn first_fresh_chunk_also_invalidates_the_old_window() {
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, 0.9, 0);
        monitor.observe(&DeviceType::Microphone, 0.1, 1);
        tick(&mut monitor, true, 1);
        assert_eq!(events.lock().unwrap()[0].microphone, 0.1);
    }

    #[test]
    fn recording_pause_invalidates_meter_history_before_resume_returns() {
        let state = super::super::recording_state::RecordingState::new();
        state.start_recording().unwrap();
        let before_pause = window_revision();
        let (mut monitor, events) = monitor();
        monitor.observe(&DeviceType::Microphone, 0.9, before_pause);
        state.pause_recording().unwrap();
        state.resume_recording().unwrap();
        let after_resume = window_revision();
        assert_ne!(before_pause, after_resume);
        monitor.update(state.is_active(), after_resume);
        tick(&mut monitor, true, after_resume);
        assert_eq!(events.lock().unwrap()[0].microphone, 0.0);
        state.stop_recording();
    }
}
