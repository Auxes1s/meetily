# Pipeline reinforcement evidence — 15 October 2026

**Implementation and focused checks delivered; full Q02/AC05 acceptance remains open.**
The [handoff](../../pipeline-reinforcement-handoff.md) describes behavior and limits.

## Final checks

| Check | Result |
| --- | --- |
| Frontend regression suite | 183 passed, zero failed; 695 assertions in 41 files. |
| Native library suite, unfiltered | 371 passed, one known failure, eight ignored. Command exits 1. |
| Known native failure | `audio::device_detection::tests::test_builtin_mic_detection`: existing `MacBook Pro Microphone` Wired-versus-Unknown expectation. Its source was not modified. |
| Production frontend and sequential TypeScript | Passed. |
| Portable GGML / standalone onboarding | 14 Node tests passed; standalone onboarding assertions passed. |
| Validation tooling | 10 Python tests passed. |
| Isolated macOS package | Build and `codesign --verify --deep --strict` passed. Ad-hoc signed, unnotarized; not installed. |
| Independent lint | Existing Next ESLint setup remains unresolved; no lint pass claimed. |

New cases cover verified/conflicting PCM overlap and gaps, invalid clocks,
byte-budget exhaustion, a slow disk cursor, failed checkpoint publication,
actual FFmpeg failure with raw-tail preservation, recovery/idempotency,
encoder-failure retention without growing PCM batches, and saver drainage.
Frontend cases cover 10,000 canonical passages with partial-only row updates,
coalesced/retried recovery writes, transaction commit/abort acknowledgments,
scroll intent/anchors and cancellation, shared controls across remounts, stale
pause/resume events, and old Stop continuations racing a new recording during
native reads, cache acknowledgment and sidebar refresh.

## Latest-package native replay

All rows use the same final isolated package, both source channels, a VAD pause
boundary, the installed local model, a 1,000-block notepad and 10,000 synthetic
stored passages. These are real paced source-VAD/ASR/IPC/SQLite/export runs with
native WebView edits; they bypass physical devices and RecordingManager capture.

| Route | Replay | Final passages | Verified checkpoints | Stop to commit | Observed note-save p95 |
| --- | --- | --- | --- | --- | --- |
| Continuous Whisper large-v3 | 10s | 4 | 8/8 | 3.94s | 869 ms |
| Parakeet v3 int8 | 10s | 4 | 4/4 | 0.01s | 917 ms |
| Fallback Whisper large-v3 | 10s | 4 | 4/4 | 2.57s | 965 ms |

All 12 final passages matched native manifests, stored source/timing/text and
JSON export. All 16 delivered source checkpoints verified; both supplied source
PCM streams were retained exactly in each run. Notes and synthetic stored
records remained present; SQLite integrity was `ok`, with no foreign-key errors.
[Native receipts](native-replays.json) bind each run to the final executable hash.
The save observations are short diagnostic samples, not duration acceptance.

The final fallback run's actual WebKit IndexedDB contained one meeting and four
transcript records, verified by a read-only full-table scan. This establishes
that cache writes reached the WebView store; it is not a crash-recovery or
browser-cache content-equivalence acceptance test.
[Counts](webview-recovery-counts.json).

Earlier supplementary 20-second replays also exercised two Whisper recordings
appended to one meeting and Parakeet alternating sources. They predate the last
frontend Stop-cleanup fence and are not substituted for the final-package rows.
Private run/log locations are retained under the session backup named by
`/tmp/meetily-reinforcement-current.txt`.

## Acceptance limits and preservation

macOS `lsappinfo` reported `loginwindow` in front. Every final run recorded zero
foreground samples. Background paint samples/timeouts remain diagnostic and do
not pass the ≤100 ms foreground keystroke or ≤500 ms editor-pause gates. Actual
foreground editor/scroll acceptance must be repeated in an available desktop
session. No lock-screen control was attempted.

The 10/20-second replays do not close 45-minute/four-hour duration, sustained
backlog/memory, physical microphone/system fidelity, extended native pause,
full manager/device/tray/failure/upgrade or Windows acceptance. A replay pause
boundary is not native device Pause/Resume. ASR speed/fidelity and the older
failed parity gate are not relabeled passing; E08/AC07 remains failed.

The installed `/Applications/meetily.app` executable and all 25 production SQLite
table contents matched protected before/after hashes. [Preservation](preservation.json).
No dependencies or models were added, no migration/default was changed, and no
production installation or release was performed. Earlier isolated bundles and
incoming source copies remain outside the checkout in the session backup.

## Reproduction

From the repository root, using existing tools/models:

```sh
bun test frontend/tests
pnpm --dir frontend build
pnpm --dir frontend exec tsc --noEmit --incremental false
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  cargo test --locked --offline -p meetily --lib
node .github/verify-portable-ggml.test.cjs
node frontend/tests/lib/onboarding-summary-model.test.mjs
python3 -m unittest discover -s scripts/tests
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  CARGO_NET_OFFLINE=true pnpm --dir frontend exec tauri build \
  --features q02-validation --config src-tauri/tauri.q02-test.json --bundles app
python3 scripts/run-q02-validation.py \
  --bundle "$PWD/target/release/bundle/macos/Meetily Q02 Test.app" \
  --model-dir "$HOME/Library/Application Support/com.meetily.ai/models" \
  --provider whisper --routing overlapping --source overlap --seconds 10 \
  --stress --pause --timeout 180
```

Repeat the last command with `--provider parakeet --routing legacy`, then
`--provider whisper --routing legacy`. The runner uses only its guarded test
identity, archives prior disposable profiles and prints its private run path.
See [checks](checks.json) and [source hashes](source-sha256.json) for this receipt.
