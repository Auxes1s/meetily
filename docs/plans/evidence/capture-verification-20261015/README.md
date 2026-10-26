# Automated capture verification, 15 October 2026

**Result: automated capture, routing, persistence and continuation checks pass;
full microphone speech and transcript fidelity acceptance remain open.**
The user selected automated checks only. No application code was changed.

The installed `/Applications/meetily.app` matches all 15 files in the newest
archived production release, built at 10:15 on 15 October. Its executable SHA-256
is `53408f31f259865b26794cbaf34ad7a550f51a5fc908ceedcb10f4f4e113a60a`.
Fresh isolated debug capture and release replay packages were built from the
current dirty checkout. Hardware/replay results below belong to those test
packages; the installed production executable was preserved and was not recorded
through directly. The test packages use the existing frontend export, local
Whisper large-v3 and Parakeet v3 int8 models. No models were downloaded.

## Requested behavior

| Check | Finding | Practical limit |
| --- | --- | --- |
| Microphone capture | The actual microphone device initialized alongside system capture with both providers; microphone-tagged output reached persistence. Injected, distinct microphone speech passed all automated routing cases. | Room input was not independently identified or reviewed. Microphone intelligibility and acoustic echo rejection are unaccepted. A microphone-tagged passage alone does not prove correct recognition. |
| System capture | Actual system playback retained the known seven-word fixture phrase in both recordings with each provider. All four audio files decoded successfully. | Short local playback only; no live meeting or long-session acceptance. |
| Source attribution | Eight paced native replay cases passed: microphone, system, simultaneous and alternating sources for both providers. Every emitted source matched an active injected channel; final text, sources and capture clocks matched export. Frontend tests cover Microphone, System, Mixed and Unknown badges and recovery. | Replay starts at source VAD and bypasses physical devices and the full capture manager. Physical microphone speech attribution remains unreviewed. |
| Transcript completeness | All 35 final passages across ten replay sessions survived Stop, SQLite and export. Supplied source PCM was retained exactly. All six passages in the four physical recordings matched saved transcript manifests after the database was reopened. | This establishes retention of supplied audio and emitted text, not a complete or accurate transcription of all speech. Authored-reference word differences remain. |
| Pause/resume | Actual native Pause and Resume commands succeeded with both providers; window hide/reopen also succeeded. Native regression tests preserved a paused stream's tail and clock and drained the Stop tail once. | The hardware pause lasted about 300 ms, before fixture playback. Speech exclusion during an extended physical pause was not tested. Replay's pause is a separately labeled VAD boundary. |
| Append to meeting | Both providers completed two captures in the same selected meeting, preserving its title and original notes, with separate audio files and a continuous second-session offset of 5.024 seconds. Replay also checked repeated sessions, unique identities and portable offsets. | Short sessions; no long-session, crash or installer acceptance. |

## Fidelity findings

Compared with the pre-authored references, Whisper's diagnostic word error rate
was 6.1–11.1%; Parakeet's was 7.4–9.1%. The comparison retains accents and repeated
words. Whisper changed `café`/`José` to unaccented spellings, inserted `changes`
in two system-source cases, and substituted `soars` for `source` in the alternating
system case. These are authored-reference diagnostics; the references still
require independent listening review. They cannot support a full transcript
accuracy or completeness pass.

Expected repetitions, negative qualifiers and the final word `tomorrow` survived
all public fixture cases, including the Stop tail.

The short physical fixture is three seconds long and contains the beginning of
a longer authored sentence. Its seven-word recognized phrase must not be treated
as proof that the entire longer sentence was captured. Raw hardware audio,
transcripts, screenshots and native logs remain local outside this repository.

## Fresh checks

- Focused frontend suite: **52 passed, 0 failed**, across nine files.
- Native library suite: **361 passed, 1 failed, 8 ignored**. The existing failure
  is `audio::device_detection::tests::test_builtin_mic_detection`: actual `Wired`,
  expected `Unknown`. The native suite is not fully green.
- Q02 validation/fidelity tooling: **9 passed**.
- Fresh isolated debug capture package and isolated release replay package:
  both builds succeeded. Builds reused the current frontend export; a new
  frontend production build, TypeScript check and lint were not rerun.
- Hardware capture: both providers passed the existing native WebView driver;
  four saved audio files decoded with packaged FFmpeg; both disposable databases
  passed integrity/foreign-key checks after capture processes exited.
- Paced replay: **8/8 cases passed**, ten sessions, 35 final passages; exact PCM,
  source, timing, final-manifest, export and reopened-database checks passed.
- `git diff --check` passed.

The exact targeted test command and package hashes are in [results.json](results.json).
Test output is retained in [frontend-tests.log](frontend-tests.log),
[native-tests.log](native-tests.log) and [validation-tool-tests.log](validation-tool-tests.log).

## Reproduction and preservation

Build the isolated configurations from `frontend`, reusing the prepared frontend
export, with `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE`,
`PATH="$PWD/../scripts/macos-clt:$PATH"` and `CARGO_NET_OFFLINE=true`:

```sh
pnpm exec tauri build --debug --config src-tauri/tauri.recording-test.json --bundles app
pnpm exec tauri build --features q02-validation --config src-tauri/tauri.q02-test.json --bundles app
```

Use the [Q02 hardware instructions](../q02-native-validation/README.md) for
isolated profiles and model-file links. Enable `MEETILY_RECORDING_SMOKE=1`, then
run once per `MEETILY_Q02_CAPTURE_PROVIDER=whisper` and `parakeet`. The fresh local
driver archives and restores all three dedicated native/WebKit/cache stores.
Replay uses `scripts/run-q02-validation.py`, the two public 15.129875-second
fidelity WAVs decoded to 16 kHz mono f32le, `--language en --pause`, all four source
modes and both providers; microphone-only cases also use `--two-sessions`.
Alternating input runs for 30.25975 seconds. Run ASR workloads sequentially.

[preservation.json](preservation.json) records 974 existing source/document files
unchanged, all 15 installed app files unchanged, all six model files unchanged,
all 20 production source-table fingerprints unchanged, SQLite integrity `ok`
and no foreign-key violations. The original dirty workspace was retained.
The new files in this directory are verification artifacts only.

This is macOS Apple Silicon automated evidence. Windows, real human microphone
speech, distinct simultaneous physical voices, extended pause exclusion,
independent listening review, long sessions and full release acceptance remain
outside this run. Existing Q02 acceptance status remains incomplete.
