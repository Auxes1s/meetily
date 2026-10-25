# Q02 — native package validation and recording continuity

## Owner closeout — 17 October 2026

**Long-recording test: APPROVED by the project owner after personal manual
verification.** This closes the long-recording acceptance item for the retained
app. Existing automated results remain as recorded; the manual approval adds
no measured duration, provider coverage or performance figures.

The owner accepted the current app and directed that unavailable Windows
verification be left as is. Windows acceptance is **deferred and unverified**.
Preserve the app and close the current work at that scope. Other Q02 limitations
remain recorded separately. Earlier recommendations below to launch more
duration runs or continue Windows verification are historical and superseded.
See [release readiness](../v0-release.md#owner-closeout--17-october-2026).

## Historical status — 15 October 2026

Status as of 15 October 2026: **Q02 is incomplete.** The user's latest priority is
continuous useful processing with large Whisper, allowing slower-than-real-time
inference: process fresh audio whenever the worker is free, retain the loaded
decoder and avoid redundant previews. The earlier strict RTF/legacy-latency gate
remains recorded as failed historical acceptance; it is not declared passed by
this clarification. The final short paced check observes continuous useful
processing: one persistent live decoder, 96.2% session time in inference, zero
stranded fresh-buffer waits and zero unchanged-preview decodes. Full Q02
acceptance remains pending.
Continuous Whisper remains enabled by default, with the persistent Settings →
Beta fallback applying to the next recording. L05 export is delivered;
Q01 has passing isolated macOS workflow evidence. Q03/Q04 and E08 remediation
were not entered.

The user clarified: “I just need the inference or processing to be continuous
so we dont waste compute time.” Continuity now means eager admission of fresh
source audio, no stranded fresh buffer while waiting for input, no unchanged
preview decode and a resident reused decoder. Inference speed, display latency,
Stop time, backlog and CPU/RAM/GPU counters remain reported diagnostics. The
notepad's AC05 responsiveness/persistence requirements and source fidelity/
retention requirements remain separate acceptance obligations.

## Latest follow-up verification — 15 October 2026

The [automated capture report](evidence/capture-verification-20261015/README.md)
supersedes the earlier next-task recommendation to repeat short capture/source
smoke. Fresh isolated packages from the current source passed all eight paced
source cases with both providers, retaining exact supplied PCM and all 35 final
passages across ten sessions through Stop, SQLite and export. Short actual system
capture, native pause/resume state transitions and two captures appended to one
meeting passed with both providers; four separate audio files decoded, notes and
titles survived, and second-session offsets were continuous.

The installed production release matched all 15 files of the newest archived
release and stayed unchanged, as did the models and all 20 production source
tables. Runtime capture evidence belongs to the fresh isolated packages. The
user selected automated checks only; human microphone speech, distinct physical
sources, extended paused-speech exclusion and independent listening remain open.
Authored-reference word differences persist (Whisper 6.1–11.1%, Parakeet 7.4–9.1%);
these are diagnostics, not independently reviewed accuracy acceptance. Direct
installed-production capture was not tested.

The focused frontend suite passed 52 tests; nine tooling checks passed. The
native library suite reported 361 passed, one known microphone-classifier failure
and eight ignored. The packages reused the existing frontend export; no new
frontend build, TypeScript or lint pass is claimed by this follow-up.

**Next available automated task:** current-revision 45-minute normal workloads,
continuous large-v3 first, then Parakeet, before the four-hour AC05 stress run.
Measure actual foreground editor/save behavior, resources, queues/backlog,
dropped chunks and Stop/source/export reconciliation. Replay cannot close a
physical-meeting gate. Remaining human fidelity/capture, failure/recovery/upgrade
and Windows/minimum-platform acceptance stays open. Q02 is still incomplete;
this documentation update starts no new recording or duration work.

## Changes and purpose

- Shared effective Whisper decode parameters between complete and continuous
  routes, including language/translation, beam, temperature, thresholds and the
  actual four decoder threads. Earlier hardware recommendations were logging,
  not the applied thread count. Model, quality thresholds and the default route
  are unchanged.
- Admit each fresh preview as soon as the worker is free; coalesce a superseded
  preview only when that same source has pending delivery. The earlier adaptive
  two-to-five-second admission delay was removed after the user clarified the
  continuity priority. Endpoints, full windows and Stop flush unfinalized audio.
  Already finalized retained overlap does not require another decode. Reuse one
  decoder and its input allocation; all delivered PCM remains retained.
- Added validation-only worker wait/queue-wake instrumentation, fresh-buffer
  wait detection and unchanged-preview counts. These distinguish waiting for
  new input from leaving available speech idle; GPU occupancy alone is not a
  useful-work acceptance measure.
- A full window that cannot advance after its forced decode now reports an
  error with checkpoints retained, instead of repeatedly decoding the same
  audio forever. Repetition, Unicode and source clocks retain their existing
  reconciliation rules.
- Bound unresolved windows at 20 seconds instead of 30 to limit expensive
  autoregressive decode spikes. The five-second target and three-second overlap
  remain. This scheduling change retains source PCM and does not close the
  fidelity, sustained-backlog or latency acceptance gates.
- Corrected append-time pruning to retain the declared three seconds behind
  committed audio. It previously trimmed to the commit itself after a later
  append, undoing the overlap retained at finalization. Regression checks cover
  early commits and forced-window continuation with exact retained samples.
- Overlap reconciliation removes identical token prefixes only with timed
  anchors on intersecting already-committed source audio. Untimed punctuation
  inside such an anchored prefix can match; wholly untimed prefixes, unbounded
  tails and adjacent later repetitions do not qualify. Timestamp revisions
  cannot move a canonical commit backwards. The public alternating fixture
  exposed an extra repeated prefix in an intermediate run; the final fixture
  no longer shows it, while word/Unicode diagnostics and listening review remain
  open.
- Added the non-default `q02-validation` feature, isolated test configuration,
  guarded native replay/measurement commands and WebView driver. Both the
  feature and exact `com.meetily.q02.test` identity plus explicit environment
  opt-in are required. Production identities reject the harness.
- Added `scripts/run-q02-validation.py`, public pre-authored speech fixtures and
  diagnostic fidelity comparison. Runs use installed immutable model files,
  disposable records, separate native/browser storage and a temporary working
  directory. They record actual paced VAD/ASR/IPC/persistence, source PCM,
  checkpoint verification, final manifests, exports, timing and resources.
  They do **not** invoke microphone/system devices or the full RecordingManager
  capture path. VAD pause-boundary replay is labeled separately from native
  device pause/resume.
- The prepared Windows runner requires a Q02 build manifest with the exact
  executable hash before launch, supplementing native guards. Unit checks reject
  missing/stale manifests, production identity and absent validation feature;
  these are pre-launch contract checks, not Windows native acceptance.

## Environment and reproduction

Available host: macOS 27.0.1 (26A434), Apple M5, ten cores, 24 GiB RAM; arm64
Rust/Cargo 1.99. Existing dependencies and lockfile were retained. The manifest
already declared Rust 1.91; this work does not establish a new supported floor.
Whisper uses installed `ggml-large-v3.bin` (3,095,033,483 bytes), Metal and flash
attention. CoreML was compiled but the encoder asset is absent; logged CoreML
loading failure falls back to Metal. Parakeet uses the installed
`parakeet-tdt-0.6b-v3-int8` ONNX files. The packaged helper probe uses the existing
Qwen3.5 4B Q4_K_M GGUF. No models were downloaded.

Build the frontend, then check TypeScript **sequentially**:

```sh
cd frontend
pnpm build
pnpm exec tsc --noEmit --incremental false
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/../scripts/macos-clt:$PATH" \
  CARGO_NET_OFFLINE=true pnpm exec tauri build --features q02-validation \
  --config src-tauri/tauri.q02-test.json --bundles app
cd ..
```

Use only the isolated bundle. Set `Q02_MODELS` to an existing model directory;
`Q02_PCM` is a protected copy of the original bounded 16 kHz mono f32le input.

```sh
Q02_BUNDLE="$PWD/target/release/bundle/macos/Meetily Q02 Test.app"
Q02_MODELS="$HOME/Library/Application Support/com.meetily.ai/models"
python3 scripts/run-q02-validation.py --bundle "$Q02_BUNDLE" \
  --model-dir "$Q02_MODELS" --pcm "$Q02_PCM" --seconds 90.112 \
  --source system --provider whisper --language auto --routing legacy
python3 scripts/run-q02-validation.py --bundle "$Q02_BUNDLE" \
  --model-dir "$Q02_MODELS" --pcm "$Q02_PCM" --seconds 90.112 \
  --source system --provider whisper --language auto --routing overlapping
```

The input SHA-256 is
`17f6663681fec5356ed3841e0683fcc8539c8edfcba17188a6f592a5123b7dd1`.
It is exactly 90.112 seconds; the two routes receive identical audio and effective
decoder settings, with the same warmup. The private excerpt is not committed.
The earlier virtual-clock benchmark can be reproduced with
`run-meeting-workflow.py --synthetic-only --skip-build --benchmark-audio
<protected-f32-WAV> --benchmark-start 0 --benchmark-seconds 90.112` using the
preserved Q01 baseline bundle. That older benchmark loop does not apply the new
production preview admission and must not be called the optimized paced route.

Public reproduction, source matrix, long workloads and Windows steps are in
[the acceptance checklist](evidence/q02-native-validation/README.md).
Run directories print their own location. Logs, transcripts, raw source PCM and
archived test profiles remain local; the evidence package contains sanitized
results. `/tmp/meetily-q02-current.txt` locates the protected intake, build/check
logs, private run registry and source snapshot for this session.

## Before/after evidence

**Current eager-admission result:** the same 90.112-second protected PCM,
installed large-v3 model and effective settings yielded 90.42 seconds of
inference over 94.01 seconds of session wall time: **96.2% decode busy time**.
Forty-four worker wait observations showed zero stranded fresh-buffer waits and
zero unchanged-preview decodes. Aggregate ready-queue wake time was 0.133 ms;
the maximum was 0.0291 ms. One live decoder was reused. A stranded wait means
fresh buffered audio, no queued delivery, and a wait longer than the 1 ms
instrumentation noise floor; ordinary input-empty waits and busy inference are
reported separately.

All 43 source checkpoints and both retained PCM streams verified; final
source/text/timing matched exports. Unique delivered PCM still awaiting completed
decoding peaked at 6.18 seconds. The older capture-clock backlog proxy peaked at
8.60 seconds; that proxy also counts silence/source-start gaps. Its definition
is preserved in historical receipts. The analyzer now separately unions delivered
PCM intervals so alternating sources cannot mistake initial silence for backlog.

Speed remains diagnostic under the user's clarified priority: RTF 1.00344,
native-event p95 10.95 seconds, selective displayed-text p95 15.20 seconds and
Stop-to-commit 3.90 seconds. The matched previous route measured 48.81 seconds
of inference, native-event p95 3.30 seconds and Stop 3.05 seconds. No original
speed/parity gate is relabeled passing. Current results, CPU/RAM and available
GPU evidence are in
[the continuity receipt](evidence/q02-native-validation/processing-continuity.json).

The final public alternating-source replay also showed one decoder, zero
stranded fresh-buffer waits and zero unchanged-preview decodes, with exact source
bytes/manifests/exports. Its microphone diagnostic preserves `yes yes` and
`repeat repeat` without the earlier extra prefix; it has two word edits out of
33. System `budget budget` is retained, with three edits out of 27, including
`source` → `soars`. Accent losses remain. These pre-authored references still
require independent listening review; diagnostic improvement is not accuracy
acceptance.

The original virtual-clock continuous result was 94.02 seconds of inference and
20.01 seconds simulated final backlog. Reproduction with the unchanged baseline
and exact PCM produced **93.45 s inference / 19.70 s simulated final backlog**;
the previous route produced 47.08 s / 3.37 s. Output hashes reproduced the
earlier results; this verifies reproducibility, not transcript accuracy.

Actual paced native replay is a separate experiment. The following measurements
precede the user's continuity clarification and final eager-admission change:

| Measure | Before legacy | Before continuous | After legacy | After continuous |
| --- | ---: | ---: | ---: | ---: |
| Inference seconds | 49.29 | 98.20 | 52.64 | 79.33 |
| Inference real-time factor | 0.547 | 1.090 | 0.584 | 0.880 |
| Decode count | 21 | 28 | 21 | 23 |
| Maximum completed-decode backlog, seconds | 2.72 | 18.94 | 1.95 | 12.00 |
| Middle / late p95 backlog, seconds | 1.49 / 0 | 4.44 / 6.00 | 1.49 / 0 | 4.44 / 12.00 |
| p95 capture-to-native-event latency, seconds | 3.35 | 25.03 | 3.80 | 14.01 |
| Stop-to-canonical-commit latency, seconds | See JSON | See JSON | 3.50 | 5.79 |

In that pre-eager-admission comparison, inference work fell **19.2%**, but late backlog still grows and
latency/finalization remain worse than the previous route. **The original gate
(RTF < 1, no growing backlog, display/Stop latency no worse than legacy, plus
unchanged acceptance criteria) is failed.** No route substitution closes it.

Trace evidence identifies redundant growing-window decodes as a bottleneck.
Incomplete token timing prevents many prefixes from being committed; unresolved
audio therefore reaches large windows. After correcting overlap retention, the
30-second-limit run needed 96.03 seconds of inference and 24.54 seconds to Stop.
The final 20-second limit improves those figures to 79.33 and 5.79 seconds. An
earlier cadence-only run reported 75.89 seconds/22.7% improvement before the
overlap-retention correction; it is retained as intermediate evidence, not the
final result. No speech was discarded to obtain the final measurement.

The final native RSS peaks were 3,656,240 KiB for legacy and 3,683,056 KiB for
continuous; attributable process-tree peaks were 3,706,896 and 3,735,712 KiB.
Process-tree CPU peaked at 134.0% and 136.2% respectively (multicore percentages).
These short-run peaks do not establish long-run memory stability. Full measurements
and matched effective settings are in
[the final paced receipt](evidence/q02-native-validation/final-paced-comparison.json).
Available host-wide GPU device utilization reached 100% in both runs (also the
p95 sample); IORegistry in-use memory peaked near 4.01/4.02 billion bytes. Those
unified-memory counters are not attributable ASR allocations and must not be
added to process RSS. Process GPU/power measurements remain unavailable.

The completed 45-minute Whisper normal workload produced 1,080 final passages,
verified all 2,160 source-VAD checkpoints, retained both complete PCM streams
byte for byte and matched live final passages to exported source/timing/text.
It required 2,700.47 seconds of inference (RTF 1.00018), with maximum summed
completed-decode backlog 11.99 seconds; middle and late p95 backlog both remained
11.99 seconds. Native-event p95 was 6.05 seconds, selective displayed-text p95
6.72 seconds, and Stop-to-commit 3.11 seconds. The strict RTF criterion still
fails. This periodic speech/silence workload is different from the dense
90-second comparison, and is not used to close its gate.

Native RSS peak was 3,312,080 KiB. Early/middle/late native RSS p95 was
3,301,728 / 3,308,832 / 3,312,048 KiB; attributable tree p95 was
3,364,064 / 3,379,952 / 3,392,960 KiB. This modest increase includes retained
transcripts, test traces and editor content; the measurements do not isolate
each allocation. Maximum worker pending audio was 8.56 seconds (including the
current chunk), and maximum unfinalized source audio was two seconds.

The driver observed 470 successful note edits/acknowledgments (p95 1.427 seconds)
and 111 foreground scripted insertion-to-paint samples (p95 24 ms, maximum
26 ms). The other 359 samples were background; all-sample paint p95 was
1.177 seconds. Scripted DOM insertion is not an OS keyboard trial, and observed
successful saves do not establish complete attempt/drop coverage or full U01/
U06 acceptance. Results and early/middle/late resource samples are in
[the long-run receipt](evidence/q02-native-validation/long-paced-workloads.json).

Backlog is measured from delivered source-VAD audio ends to completed decode
ends, summed per source. Unfinalized hypothesis duration is also traced; it is
not interchangeable with queued audio. Visible-text samples were selective,
and early comparisons did not record focus/visibility. They cannot establish
complete display-latency or AC05 acceptance. Later instrumentation labels
foreground samples and suspended animation frames, and measures observed native
note acknowledgments independently of waiting for paint.

## Acceptance evidence

| Evidence class | Executed result | Limit |
| --- | --- | --- |
| Unit/contract | 361 Rust tests passed after eager admission; eight ignored; known microphone-classifier test explicitly excluded and rerun failing. 128 frontend tests passed. Nine Q02 Python diagnostic/launch-guard/continuity/PCM-coverage tests, five existing workflow-isolation tests and 14 portable checks pass. Production frontend build and subsequent TypeScript check pass. | These are not native Windows or physical-device acceptance. |
| Native IPC and paced replay | Earlier installed Whisper/Parakeet mic-only/system-only/overlap/alternating matrix and two-session checks completed. Final eager-admission Whisper system and distinct alternating replay verified continuity, Unicode notes, exact PCM, Stop manifests and exports. | Source-VAD/worker injection, not native capture. Earlier full matrix predates eager admission. Independently reviewed accuracy remains pending. |
| Physical capture | Prior isolated macOS debug package completed two actual recordings in one meeting with each provider, separate audio files, unchanged notes/title and continuous offset. Native pause/resume and window hide/reopen passed for both; canceled-start passed; missing-model recovery passed in the Whisper case. | Prior-scheduling evidence. Whisper emitted system-only passages; Parakeet emitted both. Distinct physical speech matrix, listening review and latest-revision tray/denial/disconnect/Windows acceptance remain pending. |
| Packaged macOS | Isolated release bundle loads installed Whisper/Parakeet files from a temporary working directory. Packaged FFmpeg and helper execute; actual native helper loads the installed GGUF and returns nonempty text. Relocated package/startup preservation check is recorded separately. | Ad hoc validation bundles; no signed installer/release or supported-minimum-OS acceptance. Relocation/restart is not a previous-release installer upgrade. |
| Long workloads | Pre-eager-admission 45-minute Whisper replay completed with exact source retention and final manifests/exports. Parakeet was terminated at 34 minutes when its controller was stopped during the priority change; no 45-minute pass. The four-hour run was held before startup. | Duration/manifest success does not pass full AC05. The completed Whisper run is prior-scheduling evidence. Revised long runs remain pending. |
| Windows x64 | Not available. Reproduction and acceptance checklist prepared. | No macOS or mocked result substitutes for Windows. |

The public reference passages were authored before ASR evaluation. Diagnostic
alignment preserves accents and repeated words, reports word edits/repetition/
replacement characters, and explicitly requires independent source-audio review
before accuracy acceptance. The private 90-second excerpt also lacks an
independently reviewed reference. Hash equality alone is not an accuracy result.
The intermediate append correction restored `repeat repeat` but still added
an extra `Yes yes` prefix. The final eager-admission replay no longer shows that
extra prefix in this fixture. Accent losses in `café`/`José` and system `soars`
remain in diagnostics. See the current continuity receipt and the intermediate
[boundary evidence](evidence/q02-native-validation/overlap-boundary-after.json);
no automatic text normalization converts those findings into an accuracy pass.

## Remaining failures and unavailable scenarios

- The user's continuous-processing priority has passing short-run observations;
  the original RTF/legacy-latency gate remains failed historical acceptance.
  Full fidelity/phrase/boundary acceptance requires independent listening review.
- AC05 requires completed 45-minute/four-hour evidence, sufficient actual
  foreground editor/paint samples, explainable native/WebView memory growth,
  bounded save/render queues and full Stop reconciliation. It remains open.
- Physical microphone-only/system-only/distinct overlapping speech with both
  providers, tray Stop/Quit, permission denial and actual device disconnect
  remain separate scenarios. Native pause/resume and hide/reopen passed in the
  isolated physical driver with both providers; a replay pause marker alone
  does not prove those controls. Full U03 acceptance remains open.
- U04 forced-process termination at each native workflow boundary and U09
  previous-release install/upgrade/interrupt scenarios remain open beyond the
  executed unit/subprocess contracts and isolated package startup checks.
- Windows x64 and established minimum OS/backend targets are unavailable. The
  macOS package's 10.13 metadata conflicts with ScreenCaptureKit/backend
  requirements in repository documentation; this host establishes no minimum.
- `audio::device_detection::tests::test_builtin_mic_detection` still fails:
  actual `Wired`, expected `Unknown`. The detector was not changed.
- Eight default-suite tests remain ignored: hardware capture/detection, two
  installed-ASR tests, installed-model echo, long import, E08 evaluation, and a
  subprocess helper invoked by its parent test. The `large-v3-turbo` opt-in
  fixture requires an absent model; no download or substitution was made.
- Lint exits 1 at Next's unresolved ESLint setup prompt. No lint pass is claimed.
- CPU/native RSS and attributable WebKit process metrics are sampled. Available
  IORegistry GPU utilization is host-wide. Process GPU/power attribution is
  unavailable: `powermetrics` requires root and `xctrace` requires full Xcode.
  No privileged installation or permission escalation was performed.

## Protection and stop point

The pre-existing dirty working tree was snapshotted, not reset/stashed. The
production database was opened read-only for an online protected backup. No
production-library migration, recording write, installed application replacement
or model modification is part of Q02. Test records/browser caches are archived
under their exact isolated identifiers. Final preservation receipts confirm all
36 protected model/bundle file hashes and all 20 canonical production tables
unchanged, with no unexpected changes among 868 pre-existing source files. The
pre-existing dirty lockfile is unchanged.

**Next eligible work is the remaining Q02 fidelity, duration and native
acceptance gates.** Q03 becomes eligible only after its dependencies, including
Q02, are met. This handoff does not authorize Q03/Q04, release publication or E08
model-quality remediation.

## Project-plan synchronization — 15 October 2026

After this Q02 validation, the separate UI design/interaction workstream was
delivered and its verified release build installed as `/Applications/meetily.app`.
See the [installation record](evidence/ui-install-20261015/README.md). That
separately authorized action does not change the protection claims for the Q02
run above or supply its missing physical/fidelity/duration acceptance. The
UI-only fixture checks block capture lifecycle commands. Subsequent dedicated
capture/replay verification is recorded separately in the latest follow-up above.

The [current project plan](meeting-experience-upgrade.md#current-project-status--15-october-2026)
now records the short automated source/capture smoke complete and recommends
the next bounded automated Q02 assignment: current 45-minute normal workloads,
continuous large-v3 first then Parakeet, before four-hour AC05 stress. Human
microphone/listening/extended-pause checks and remaining native/platform gates
stay open. Keep the user's continuous-useful-processing priority and the original
failed speed comparison distinct. Earlier long-run evidence predates eager
admission and cannot close latest-revision duration acceptance.

Earlier bundles, including the Q02 test app, were moved to
`target/archived-app-bundles.noindex/20261015T103927/`. The latest follow-up rebuilt
the isolated recording debug and Q02 release packages at their normal paths;
their hashes are recorded in its receipt. Verify the exact artifact before
running reproduction commands; never use the installed production identity as
the Q02 runner. This synchronization changes documentation only.


## ASR and UI pipeline reinforcement — 15 October 2026

R10–R13 / U12–U15 are implemented. See the
[reinforcement handoff](pipeline-reinforcement-handoff.md) and
[verification evidence](evidence/pipeline-reinforcement-20261015/README.md).
The changes cover durable bounded ASR intake, checkpoint/Stop drainage and
recovery, source continuity, soft limiting, row-level transcript updates,
scrolling, shared controls and session-fenced frontend cleanup. The installed
production app/library are preserved; the test package is isolated.

Short replay and regression results do not close Q02. The latest 45-minute and
four-hour workloads remain next; foreground paint (unavailable with loginwindow
in front), physical capture/fidelity and Windows gates remain open. E08/AC07,
Q03/Q04 and release acceptance are unchanged.
