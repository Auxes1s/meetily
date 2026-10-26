# Transcript audio sources

Section 5.1(10), R07–R09: live passages retain their capture channel before audio
is mixed. Each channel has its own VAD state; both consume the existing aligned
input windows and share a recording-relative clock. One ASR worker uses the
existing Whisper or Parakeet engine. The saved audio remains a combined recording.

Live speech is dispatched in chunks of at most five seconds, or earlier after a
500ms qualifying pause. Continuous speech no longer waits for an utterance to
end before reaching ASR. Each channel keeps its detector context across these
boundaries; completed and stopped utterances emit only the unreported suffix.
The final padded VAD frame cannot extend a chunk beyond the real recording
endpoint. Text appears after the model finishes decoding, so five seconds is an
audio-chunk limit rather than a guaranteed display interval. Imported recordings
and retranscription retain their existing whole-utterance segmentation.

Badges identify capture channels, not people:

| API/JSON value | Accessible and hover label | SQLite `speaker` adapter |
| --- | --- | --- |
| `microphone` | Microphone | `mic` |
| `system` | System audio | `system` |
| `mixed` | Mixed audio | `mixed` |
| `unknown` | Unknown | `unknown` or legacy null/unrecognized value |

Transcript badges use compact icons for these four channels. Each icon retains
its full channel name for screen readers and a capture-channel description on
hover. The labels and source values in copied or exported records are unchanged.

The additive optional `audio_source` field travels through events, buffered
passages, history, recording JSON, recovery, SQLite, pagination, source inspection,
and transcript copying. Recording transcript JSON is version 1.1; missing fields
remain readable. Repeated or final updates preserve known provenance when an
older payload omits it. New session/sequence identities keep recovery retries
from duplicating passages.

Retranscription of an app-created combined recording is Mixed audio, regardless
of the original live passage badges. Arbitrary imported files remain Unknown
unless file metadata establishes combined input. Generic `source: "Audio"`,
selected devices, signal levels, and participant names never establish provenance.
When both channels are captured, WebRTC AEC3 removes correlated speaker playback
from the microphone before high-pass filtering, bounded microphone gain, VAD,
and recording mixing. The system channel is unchanged. Independent simultaneous
microphone speech remains eligible for transcription. No transcript-text matching
or whole-microphone muting is used. Without system capture, microphone audio
passes through without echo cancellation because there is no playback reference.

The pinned pure Rust implementation is [Sonora 0.2.0](https://docs.rs/sonora/0.2.0/sonora/)
(BSD-3-Clause, Rust 1.91+). Each recording has its own adaptive state. Arbitrary
window sizes buffer into 10ms AEC frames; the final short tail drains once without
extending the recording clock. Both device callbacks are awaited up to the existing
device-specific jitter allowance before an absent input is padded with silence.
Microphone gain follows AEC and is capped at 12dB so residual echo is not amplified
without a bound. The echo filter estimates delay and adapts to the acoustic path;
startup, volume/device changes, unavailable system capture, clipping, and heavily
reverberant rooms can still leave some residual echo.

## Verification

Run frontend checks from `frontend`:

```sh
bun test tests
pnpm exec tsc --noEmit --incremental false
pnpm build
```

Run native checks from the repository root:

```sh
cargo test -p meetily --lib source --locked
cargo test -p meetily --lib test_write_transcripts_json --locked
cargo test -p meetily --lib echo_cancellation --locked
cargo test -p meetily --lib echo_alignment --locked
cargo test -p meetily --lib live_vad --locked
cargo test -p meetily --lib source_live_pipeline --locked
```

The source fixtures cover single, alternating and simultaneous channels, silence,
shared timestamps, unique sequence assignment, 48 kHz input, final partial windows,
repeated flushes, unchanged mixer output, all four saved source values, legacy
payloads, JSON, SQLite reopen, and partial/final updates. Frontend fixtures cover
both transcript rendering branches and accessible labels.

For installed-model testing, set the model directory explicitly; this test does
not download models:

```sh
MEETILY_ASR_TEST_MODEL_DIR='/path/to/installed/models' \
  cargo test -p meetily --lib source_existing_local_providers_smoke --locked -- --ignored --nocapture
```

On 2026-10-13, the local macOS provider smoke passed with Whisper large-v3-turbo
and Parakeet v3 int8. A real Chromium IndexedDB check retained all four sources
after page reload and rejected stale partial updates without duplicate rows.
The 360-pixel transcript layout was visually inspected: badge text and icons fit.
These fixture checks do not constitute physical-device/native-UI acceptance.

The five-second live limit is checked against uncapped VAD output for exact
audio preservation, continuous speech before Stop, natural endings, new
utterances, partial final frames and repeated flushes. A two-channel 48kHz
pipeline fixture checks short dispatch, matching timing and unique chunk IDs.
The opt-in short-chunk provider smoke uses installed large-v3 and Parakeet v3
int8 without downloading models:

```sh
MEETILY_ASR_TEST_MODEL_DIR='/path/to/installed/models' \
  cargo test -p meetily --lib live_short_segments_existing_local_providers --locked -- --ignored --nocapture
```

On 2026-10-14, delayed two-path echo fixtures at 16kHz and 48kHz measured about
69dB of echo attenuation after convergence. Independent near-end speech retained
0.725/0.764 correlation and 43%/48% of input energy during simultaneous playback.
The actual 48kHz pipeline, VAD, microphone gain, and installed large-v3 model
transcribed the independent microphone fixture during playback. Speaker-only
playback emitted no microphone speech segments in that fixture. These results
concern simulated acoustics, not measurements of the user's physical speakers.

```sh
MEETILY_ASR_TEST_MODEL_DIR='/path/to/installed/models' \
  cargo test -p meetily --lib echo_pipeline_existing_whisper_preserves_independent_speech --locked -- --ignored --nocapture
```

Remaining acceptance gates: Windows physical mic/system capture with both
providers; native macOS physical-device/UI workflow; permission/device/tray and
pause/resume scenarios; 45-minute and 4-hour throughput, backlog, memory, and
editor-responsiveness measurements. Separate-channel inference increases work
when both inputs contain speech; no unchanged-throughput claim is made.
