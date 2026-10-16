//! Admit fresh audio as soon as the decoder is free. Coalesce superseded
//! previews while retaining every sample; endpoints and full windows flush.
#[derive(Default)]
pub struct PreviewAdmission {
    last_end: Option<f64>,
}
impl PreviewAdmission {
    pub fn has_fresh_audio(&self, end: f64) -> bool {
        self.last_end.map_or(true, |prior| end > prior + 1e-6)
    }
    pub fn eligible(
        &self,
        end: f64,
        endpoint: bool,
        full: bool,
        same_source_pending: bool,
    ) -> bool {
        if endpoint || full {
            return true;
        }
        if same_source_pending {
            return false;
        }
        self.has_fresh_audio(end)
    }
    pub fn observed(&mut self, end: f64, endpoint: bool) {
        if endpoint {
            *self = Self::default();
        } else {
            self.last_end = Some(end);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_audio_never_waits_for_a_timer_after_a_slow_decode() {
        let mut c = PreviewAdmission::default();
        assert!(c.eligible(2., false, false, false));
        c.observed(2., false);
        // Even a slow previous inference must not leave fresh buffered speech
        // idle waiting for a larger source-audio step.
        assert!(c.eligible(4., false, false, false));
        assert!(c.eligible(6., false, false, false));
        assert!(!c.eligible(6., false, false, true));
        assert!(!c.eligible(2., false, false, false));
        assert!(c.eligible(2.1, true, false, true));
        assert!(c.eligible(2.1, false, true, true));
        c.observed(2.1, true);
        assert!(c.eligible(2.2, false, false, false));
    }
    #[test]
    fn one_capture_sources_pending_work_does_not_starve_the_other() {
        let mut microphone = PreviewAdmission::default();
        let system = PreviewAdmission::default();
        microphone.observed(2., false);
        assert!(!microphone.eligible(4., false, false, true));
        assert!(system.eligible(4., false, false, false));
        assert!(microphone.eligible(7., false, false, false));
    }

    #[test]
    fn coalescing_leaves_fresh_audio_eligible_when_the_source_queue_catches_up() {
        let mut admission = PreviewAdmission::default();
        admission.observed(10., false);
        assert!(!admission.eligible(12., false, false, true));
        assert!(admission.has_fresh_audio(12.));
        assert!(admission.eligible(12., false, false, false));
        admission.observed(12., false);
        assert!(!admission.has_fresh_audio(12.));
        assert!(!admission.eligible(12., false, false, false));
    }
}
