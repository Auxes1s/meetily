//! One effective decode contract for complete and continuous local Whisper.
//! Keep decoder quality/language identical when comparing scheduling routes.
use whisper_rs::{FullParams, SamplingStrategy};

pub fn effective_threads() -> i32 {
    // Match whisper.cpp's established default, rather than merely logging the
    // hardware profile's recommendation as though it had been applied.
    std::thread::available_parallelism()
        .map(|n| n.get().min(4) as i32)
        .unwrap_or(4)
}
pub fn full_params(language: Option<&str>) -> FullParams<'_, '_> {
    let config = crate::audio::HardwareProfile::detect().get_whisper_config();
    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: config.beam_size as i32,
        patience: 1.,
    });
    params.set_language(match language {
        None | Some("auto" | "auto-translate") => None,
        v => v,
    });
    params.set_translate(language == Some("auto-translate"));
    params.set_n_threads(effective_threads());
    params.set_no_context(true);
    params.set_no_timestamps(true);
    params.set_token_timestamps(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_suppress_non_speech_tokens(true);
    params.set_temperature(config.temperature);
    params.set_max_initial_ts(1.);
    params.set_entropy_thold(2.4);
    params.set_logprob_thold(-1.);
    params.set_no_speech_thold(0.55);
    params.set_max_len(200);
    params.set_single_segment(false);
    params
}
pub fn description(language: Option<&str>) -> serde_json::Value {
    let config = crate::audio::HardwareProfile::detect().get_whisper_config();
    serde_json::json!({"beam_size":config.beam_size,"patience":1.0,"temperature":config.temperature,
        "language":language.unwrap_or("auto"),"translate":language==Some("auto-translate"),
        "effective_threads":effective_threads(),"no_context":true,"no_timestamps":true,"token_timestamps":true,
        "max_len":200,"single_segment":false,"no_speech_threshold":0.55})
}
