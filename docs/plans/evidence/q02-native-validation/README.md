# Q02 native validation evidence

The [Q02 handoff](../../q02-native-validation-handoff.md) is authoritative for
current scope/status. **Q02 is incomplete.** The user now prioritizes continuous
useful processing with large Whisper; the earlier RTF/legacy-latency gate remains
a failed historical result. Eager admission has passing short worker-continuity observations; full acceptance
remains pending. L05 is delivered and Q01's isolated macOS workflow passes.
Physical capture, native IPC, paced replay, virtual-clock comparison and package
acceptance are separate evidence classes.

## Evidence files

- `historical-virtual-reproduction.json`: exact 90.112-second reproduction with
  the preserved baseline; virtual capture clock and simulated backlog only.
- `paced-comparison.json`: intermediate cadence-only comparison, superseded by
  the final overlap-corrected and 20-second-window-limit measurement.
- `processing-continuity.json`: final eager-admission paired/alternating replays,
  single-decoder busy time, queue wake/stranded-buffer/stale-preview observations,
  PCM coverage excluding silence, fidelity diagnostics, resources and manifests.
- `final-paced-comparison.json`: final pre-eager-admission comparison with the
  20-second limit, latency/resources/PCM/manifests; original performance gate fails.
- `overlap-boundary-after.json` and `fidelity-diagnostics.json`: phrase/Unicode
  diagnostics, including an unresolved extra repeated prefix. Independent
  source-audio review is pending; text hashes are not accuracy acceptance.
- `native-source-matrix.json`: native worker/IPC source matrix and two-session
  checks with installed Whisper and Parakeet, distinct pre-authored source input.
- `physical-two-sessions.json`: actual macOS debug capture with public playback,
  initial two-session Whisper check; emitted source was system audio.
- `physical-controls-both-providers.json`: actual two sessions, native pause/
  resume and window hide/reopen with both providers. No tray/device-denial claim.
- `packaged-sidecar.json`: packaged FFmpeg/helper and an actual installed-GGUF
  native helper invocation. Nonempty output does not establish summary quality.
- `relocated-startup-preservation.json`: disposable canonical records survive
  package relocation/restart. This is not an old-to-new installer upgrade.
- `environment-models.json`, `checks.json`, `preservation.json`: model/runtime
  details, passing checks and explicit baseline/ignored/lint limitations, and
  protected-tree/model/bundle/database comparisons.
- Long-run receipts are added only after their checks finish. Private audio,
  transcripts, logs and profile backups stay outside the repository.
- `long-paced-workloads.json`: completed corrected 45-minute Whisper replay,
  with source/Stop/export verification, foreground/background UI measurements,
  early/middle/late resources and host GPU counters. This precedes the eager-admission change. Parakeet was aborted at 34 minutes
  during the scheduling pivot; four-hour replay was held before startup. Revised
  completed workloads are added after checks finish; full AC05 remains open.

## Public source reproduction on macOS

Build the isolated release package as described in the handoff. Each ordinary
run archives only `com.meetily.q02.test` native/WebKit/cache storage, preserving
previous test evidence. The runner refuses other macOS bundle identities and
uses an exclusive admission lock. A stale lock needs verification of its stored
PID before removing it. Do not run competing ASR workloads during measurements.

```sh
Q02_BUNDLE="$PWD/target/release/bundle/macos/Meetily Q02 Test.app"
Q02_MODELS="$HOME/Library/Application Support/com.meetily.ai/models"
Q02_FIXTURES=$(mktemp -d "${TMPDIR%/}/meetily-q02-fixtures-XXXXXXXX")
Q02_FFMPEG="$Q02_BUNDLE/Contents/MacOS/ffmpeg"
"$Q02_FFMPEG" -v error -i frontend/src-tauri/tests/fixtures/continuous_whisper/fidelity.wav \
  -ar 16000 -ac 1 -f f32le "$Q02_FIXTURES/microphone.f32"
"$Q02_FFMPEG" -v error -i frontend/src-tauri/tests/fixtures/continuous_whisper/system-fidelity.wav \
  -ar 16000 -ac 1 -f f32le "$Q02_FIXTURES/system.f32"
for Q02_PROVIDER in whisper parakeet; do
  for Q02_SOURCE in microphone system overlap alternating; do
    Q02_SECONDS=15.129875
    if [ "$Q02_SOURCE" = alternating ]; then Q02_SECONDS=30.25975; fi
    python3 scripts/run-q02-validation.py --bundle "$Q02_BUNDLE" \
      --model-dir "$Q02_MODELS" --pcm "$Q02_FIXTURES/microphone.f32" \
      --system-pcm "$Q02_FIXTURES/system.f32" --seconds "$Q02_SECONDS" \
      --provider "$Q02_PROVIDER" --language en --source "$Q02_SOURCE" --pause
  done
done
```

Add `--two-sessions` to a microphone/overlap run to exercise session identity,
notes, distinct source files, native final manifests and portable timeline
exports. The replay pause option seals VAD and inserts a 250 ms wall gap; it is
**not** a native Pause button or disconnected device. `--unpaced` remains an
actual unpaced worker run and is neither paced nor virtual-clock acceptance.

For source accuracy review, listen independently to the public WAV files and
record reviewer, date, passage boundaries and exact audible wording before
examining ASR output. Preserve uncertain pronunciations/accents and repetitions.
Only then change the corresponding reference's `independent_audio_review`
status to `reviewed`. Keep private-source review local.

```sh
python3 scripts/review-q02-fidelity.py \
  --reference frontend/src-tauri/tests/fixtures/continuous_whisper/reference.json \
  --transcripts "$Q02_RUN/transcripts-$Q02_SESSION.json" \
  --source-pcm "$Q02_FIXTURES/microphone.f32" --channel microphone \
  --output "$Q02_RUN/fidelity-review.json"
```

For system speech use `system-reference.json`, the system PCM and `--channel
system`. Repeated/alternating/paused workloads require passage-specific review,
not a whole-stream hash passed off as a transcript reference. Evaluate original
qualifiers, repeated phrases, names, Unicode, window transitions, endpoint and
final Stop coverage. Word-edit diagnostics are not human accuracy adjudication.

## Long-run reproduction

Use the same package and installed selections; keep competing heavy jobs stopped.
The runner records paced progress every minute, prevents system sleep only while
its native process runs, samples CPU/native RSS/attributable WebKit memory and
available host GPU counters, and verifies source bytes/final manifests/exports
after Stop. Resource files can be large. Check disk space before starting.

```sh
python3 scripts/run-q02-validation.py --bundle "$Q02_BUNDLE" \
  --model-dir "$Q02_MODELS" --seconds 2700 --source overlap \
  --provider whisper
python3 scripts/run-q02-validation.py --bundle "$Q02_BUNDLE" \
  --model-dir "$Q02_MODELS" --seconds 2700 --source overlap \
  --provider parakeet
python3 scripts/run-q02-validation.py --bundle "$Q02_BUNDLE" \
  --model-dir "$Q02_MODELS" --seconds 14400 --source overlap \
  --provider whisper --stress
```

Without `--pcm`, the public speech fixture is decoded by packaged FFmpeg with
two seconds of silence appended and repeated. This is a defined repeat workload,
not a four-hour human conversation. `--stress` seeds 10,000 explicitly synthetic
stored passages and 1,000 note blocks while real local ASR runs. The editor driver
inserts text every five seconds. Foreground keystroke/paint evidence requires an
active visible WebView; missing/throttled frames are recorded and cannot pass
AC05. Actual device capture and the full U01/U06 enhancement/reopen workflow
remain additional requirements. Preserve raw traces and compare early/middle/
late memory/backlog; explain retained source growth and every dropped/error chunk.

## Package and disposable preservation

`--package-only` probes the packaged binary identity, FFmpeg and helper without
opening a library. `--sidecar-probe` additionally invokes the actual native
helper with the installed Qwen GGUF and a public literal prompt. It never
downloads a model. Copy the isolated test bundle to disposable storage, run from
that copy, then use `--preserve-test-profile` with the same route. Existing
canonical meeting/note/transcript/session/link rows must survive startup, while
the new disposable meeting gets its own identity. The preservation option rejects
non-Q02 meeting titles or unfinished sessions. This verifies relocation and
restart; it does not replace a prior-version installer/migration interruption
test. Preserve source/WAL and browser recovery storage for that separate test.

## Native acceptance checklist still required

| Scenario | Evidence required |
| --- | --- |
| U01 / AC03 | Actual 45-minute mic/system meeting; formatted notes, closed/reopened transcript, physical pause/resume, Stop, full persisted manifest, local enhancement/review, explicit Quit/reopen/search/export. |
| U03 / AC10 | Separate hide/reopen, tray Stop and tray Quit; permission denied and actual mic disconnect; visible truthful capture state and preserved acknowledged notes. Record actual input signals, not permission booleans alone. |
| U04 | Kill the isolated package before first passage, after note acknowledgment, mid-capture, during Stop and candidate save; restart/recover twice. Compare IDs, revisions, passages, retained owned checkpoints and incomplete states. |
| U06 / AC05 | Completed four-hour workload; p95 actual input-to-paint ≤100 ms, normal note acknowledgment ≤2 s, no transcription-caused editor stall >500 ms; CPU/native/WebView RAM, queue depth/backlog/latency/Stop and final coverage. |
| U09 | Disposable legacy/current records with three summary formats, notes, null timestamps/folders and recovery cache; actual old→new package upgrade and interrupted migration/backup. Exact retained source fields, integrity/FKs, IDs and usable recovery. |
| U11 / AC15 | Distinct known microphone/system phrases alone, alternating and overlapping with both providers; 100% emitted provenance, matching badges, final/late passages, drawer/restart/recovery/pagination/export. Mixed inputs stay Mixed; absent origins stay Unknown. |
| Package | FFmpeg, sidecar, ONNX loader, model and templates/assets resolve in the package from a temporary working directory. Preserve independent ASR/LLM selection across restart. |
| Target | Windows x64 and macOS Apple Silicon, plus established minimum OS/backend. Do not infer the minimum from bundle metadata that conflicts with backend requirements. |

Physical recording reproduction uses the existing debug-only
`tauri.recording-test.json` and `MEETILY_RECORDING_SMOKE=1`. Close that exact test
app first, then archive **all three** dedicated stores. For Whisper, link only
`ggml-large-v3.bin`: the driver's missing-Parakeet case deliberately requires
Parakeet to be absent. For the Parakeet scenario link its existing ONNX/vocabulary
files. Never link the writable production model directory itself.

```sh
Q02_CAPTURE_PROVIDER=whisper # Repeat with parakeet after the first app exits.
export Q02_CAPTURE_PROVIDER Q02_MODELS
Q02_CAPTURE_RUN=$(mktemp -d "${TMPDIR%/}/meetily-q02-physical-XXXXXXXX")
export Q02_CAPTURE_RUN
python3 - <<'PY'
import json, os
from pathlib import Path
home = Path.home()
run = Path(os.environ['Q02_CAPTURE_RUN'])
identifier = 'com.meetily.recording.test'
for parent in ('Application Support', 'WebKit', 'Caches'):
    store = home / 'Library' / parent / identifier
    if store.exists():
        store.rename(store.with_name(store.name + '.previous-' + run.name))
profile = home / 'Library/Application Support' / identifier
destination = profile / 'models'
source = Path(os.environ['Q02_MODELS'])
if os.environ['Q02_CAPTURE_PROVIDER'] == 'whisper':
    files = [source / 'ggml-large-v3.bin']
else:
    files = [p for p in (source / 'parakeet').rglob('*')
             if p.is_file() and p.suffix in ('.onnx', '.txt')]
assert files and all(p.is_file() for p in files), 'Installed model unavailable'
for p in files:
    target = destination / p.relative_to(source)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.symlink_to(p.resolve())
(profile / 'recording_preferences.json').write_text(json.dumps({
    'preferences': {'save_folder': str(run / 'recordings'),
                    'auto_save': True, 'file_format': 'mp4'}}))
PY
cd frontend
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/../scripts/macos-clt:$PATH" \
  CARGO_NET_OFFLINE=true pnpm exec tauri build --debug \
  --config src-tauri/tauri.recording-test.json --bundles app
cd ..
MEETILY_RECORDING_SMOKE=1 MEETILY_Q02_CAPTURE_PROVIDER="$Q02_CAPTURE_PROVIDER" \
  "$PWD/target/debug/bundle/macos/Meetily Recording Test.app/Contents/MacOS/meetily"
```

Inspect the dedicated profile's `recording-start-smoke.json` and
`responsive-screenshots`, and close that exact test app after its report. The
executed capture receipt used an equivalent private configuration with product
name `Meetily Q02 Capture`; its identifier and guard are identical. Public
playback exercises actual system capture but is not a reviewed microphone-only
or distinct simultaneous-speech test. Leave production identifiers, installed
bundles, libraries and devices' permission databases alone.
Resetting system permissions or disconnecting a device requires a dedicated test
operator/scenario; it is not implied by successful replay.

## Windows x64 preparation (not executed on this Mac)

Use an actual Windows x64 host with its established native toolchain/backend and
already installed models. Build frontend then TypeScript sequentially; build the
isolated Q02 executable with `q02-validation` and the Q02 configuration (override
bundle target to the supported Windows installer when packaging). Copy binaries,
FFmpeg/helper/ONNX/assets into an isolated disposable installation directory and
write the build manifest **for that Q02 build artifact**. The runner requires its
identifier, validation-feature declaration and exact executable SHA-256 before
launch; do not create a manifest for an installed production executable.
This pre-launch guard supplements the native identity/feature/opt-in checks.

```powershell
cd frontend
pnpm build
pnpm exec tsc --noEmit --incremental false
pnpm exec tauri build --features q02-validation `
  --config src-tauri/tauri.q02-test.json --bundles nsis
cd ..
# After copying that build and its packaged dependencies to C:\Q02:
$Q02_EXE = Get-Item C:\Q02\meetily.exe
$Q02_MANIFEST = @{
  identifier = 'com.meetily.q02.test'
  native_validation_feature = $true
  binary = $Q02_EXE.Name
  binary_sha256 = (Get-FileHash $Q02_EXE.FullName -Algorithm SHA256).Hash.ToLower()
} | ConvertTo-Json
[System.IO.File]::WriteAllText(
  (Join-Path $Q02_EXE.DirectoryName 'q02-validation-package.json'), $Q02_MANIFEST)
python scripts/run-q02-validation.py --bundle C:\Q02\meetily.exe `
  --model-dir C:\Q02\InstalledModels --provider whisper --source overlap `
  --seconds 90.112 --pcm C:\Q02\ProtectedFixtures\comparison.f32
```

The NSIS command is preparation, not an executed Windows package result; retain
toolchain/backend details and installer logs from the actual host. Repeat the
source matrix, both route comparisons, two-session, long, physical
and installer/upgrade checklist on Windows itself. The native command checks the
test identity/feature/opt-in. The runner's Windows CPU/RAM/GPU sampling is not
implemented; use timestamped Windows Performance Monitor/process-tree sampling
and the established GPU/backend tooling and retain those traces alongside native
reports. Windows packages, permissions, WASAPI devices, minimum OS, dialogs,
shortcuts, install/upgrade and metrics remain pending until measured there.
