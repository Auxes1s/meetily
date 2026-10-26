# Notepad validation — 13 October 2026

The local macOS arm64 app is installed at
`/Users/marcignacio/Applications/Meetily Notepad.app` and was launched with `open`.
Its bundle identifier is `com.meetily.ai`; `/Applications/meetily.app` was preserved.
The signed bundle passed `codesign --verify --deep --strict`.

Verified on this Mac:

- Production Next.js build and TypeScript check passed.
- `bun test tests`: 83 passed, zero failed (19 files).
- Seven native original-note/database tests passed: revision conflict,
  idempotent acknowledgment retry, concurrent compare-and-set, invalid-document
  rollback, repeated finalization and timestamp resolution, reopen/interruption,
  committed-WAL migration snapshot, and preservation of a corrupt source.
  Some cases are combined within one test.
- Source routing tests: six passed, one opt-in fixture was ignored. The overlapping
  source pipeline and transcript-JSON import checks also passed individually.
- The actual native WebView completed model-free onboarding with no installed
  models, wrote Unicode text in BlockNote, acknowledged a SQLite save, switched
  drafts, and reloaded the saved text. Report:
  `~/Library/Application Support/com.meetily.notepad.test/notepad-smoke.json`.
- The native notepad UI and the installed release window were visually inspected.
- The real library migrated through `20261013030000`. It still contains seven
  meetings and 453 transcript rows. SHA-256 fingerprints of every original
  table's original columns and rows were unchanged after migration; the database
  and pre-migration snapshot both returned `ok` from `PRAGMA integrity_check`.

The build preserves concurrent audio-source, enhancement, library, and deletion
work present in the shared checkout. These statements concern the checks above;
they do not establish complete acceptance of those other milestones.

The full native test suite includes hardware/device-enumeration cases that
stalled on this Mac; that broad run was stopped. The targeted checks above were
run to completion. Windows testing, real-model transcription-quality evaluation,
a 45-minute real capture, and a four-hour real meeting remain pending. Short or
synthetic checks do not substitute for those acceptance gates.

## Live transcript and speaker echo fixes — 14 October 2026

The installed bundle was replaced and relaunched normally. The immediately
previous bundle is retained at
`~/Applications/Meetily Notepad.previous-20261014-001726.app`; the original
`/Applications/meetily.app` and earlier rollback bundle remain available.

- The transcript provider now drains sequence zero and gaps without waiting for
  Stop, keeps listeners across recording identity changes, rejects stale sessions,
  and merges reload history with live passages. Completed Whisper calls produce
  final passages even when their duration is below 15 seconds.
- Nine provider lifecycle regressions passed. The full frontend suite passed:
  97 tests, zero failures. TypeScript and the production Next.js build passed.
- An isolated native WebView fixture using the existing large-v3 model showed a
  microphone passage before Stop, retained it through collapse/reopen, saved it,
  and reopened it after process relaunch while retaining original notes. Its
  report recorded `ok: true`; this fixture used injected speech, not hardware
  microphone capture.
- Automatic AEC3 now processes the unamplified microphone against system audio
  before VAD and mixing. The saved recording also uses the cleaned microphone.
  Delayed echo, independent simultaneous speech, mic-only passthrough, partial
  tail/repeated flush, and bounded callback alignment checks passed.
- The opt-in actual large-v3 test passed through the production 48kHz pipeline,
  microphone gain and VAD. Speaker-only fixture playback emitted no microphone
  segments; simultaneous microphone speech remained transcribable. Numerical
  fixtures measured approximately 69dB echo attenuation after convergence at
  16kHz and 48kHz. These are simulated-room measurements.
- The native `source` filter passed 22 tests with one installed-provider smoke
  ignored; that filter also covers persistence, migration, grounding, and library
  source cases. The echo/large-v3 smoke was run explicitly and passed.
- The installed app passed strict signature verification and its launched window
  was visually inspected. No new migration or model download was added. Every
  non-search library table fingerprint was identical immediately before and after
  replacement; integrity check returned `ok` (10 meetings, 453 transcript rows).
  The existing `ggml-large-v3.bin` remained 3,095,033,483 bytes and configured as
  `localWhisper / large-v3`.

Physical speaker/microphone verification is still needed with the user's setup.
Echo adaptation and difficult room acoustics can leave residual playback. The
system channel must be captured to provide the playback reference. The prior
Windows, extended-capture, and throughput acceptance gates remain pending.

## Shorter live transcription chunks — 14 October 2026

The idle local app at `~/Applications/Meetily Notepad.app` was updated with a
new production bundle. Its previous bundle is retained at
`~/Applications/Meetily Notepad.previous-20261014-080800.app`.

- Microphone and system VAD now dispatch continuous speech in chunks of at most
  five seconds. The existing 500ms pause threshold still ends shorter utterances.
  Detector context and the recording clock persist across chunk boundaries.
  Natural endings and Stop emit the unreported suffix; final-frame padding is
  excluded. Import/retranscription use the existing uncapped batch constructor.
- Twenty-one targeted native VAD, source, pipeline and echo checks passed.
  New checks compare exact audio against uncapped VAD across natural endings,
  new utterances, continuous speech, Stop close to a five-second boundary,
  non-frame-aligned tails and repeated flush. A two-channel 48kHz production
  pipeline fixture emitted short chunks before Stop with distinct IDs and
  matching source clocks.
- The explicit installed-provider smoke passed with large-v3 and Parakeet v3
  int8. Both returned nonempty final text for five-second and two-second fixture
  chunks. Whisper's five-second decodes took 1.297/1.539 seconds; Parakeet's took
  0.147/0.155 seconds on this Mac in that test. These figures are fixture decode
  timings, rather than complete capture-to-display latency or sustained backlog
  measurements. The installed large-v3 production echo/pipeline smoke also passed.
- The production frontend/native build and macOS app packaging succeeded. Both
  the staged and installed bundles passed strict deep signature verification.
  The canonical library file's SHA-256 was identical before and after replacement.
  No app launch, model download, migration or private recording was used for this
  update's checks.

Text still waits for ASR decoding and any worker backlog. Silero retains its
active utterance until a natural end, so long-session memory and throughput
remain acceptance items alongside physical-device and Windows verification.

## Recording transcript persistence repair — 14 October 2026

Saving audio previously cleared the shared transcript buffer before native Stop
read it for the SQLite commit. The audio and `transcripts.json` files could
therefore be complete while the saved meeting had zero transcript rows and no
summary sources. Audio finalization now retains the final buffer until the
owning recording manager and sinks are dropped, after native persistence.

- A new regression executed actual synthetic-audio encoding/checkpoint merging,
  audio finalization, transcript SQLite persistence, idempotent retry and the
  production summary-source reader. Before the fix it failed with zero retained
  segments; afterward both final passages remained available in SQLite and the
  summary snapshot. Tauri's mock runtime is enabled only as a development test
  dependency. Twelve targeted saver/database/source checks passed, with no
  failures or ignored cases in that run. The lockfile was unchanged.
- Two existing completed meetings had retained transcript JSON but no database
  passages. Their saved metadata and recording identities matched their folders.
  All 13 passages were restored to those same meetings, with exact text,
  timestamps, sequence IDs and capture-source adapters checked after insertion.
  The source transcript files were unchanged. Other canonical tables and all
  pre-existing transcript rows retained identical content fingerprints. The
  database integrity check returned `ok`, with zero foreign-key violations.
- A verified SQLite backup was created before recovery at
  `~/Library/Application Support/com.meetily.ai/meeting_minutes.sqlite.backups/20261014T010808-before-transcript-recovery.sqlite`.
- The production frontend/native build and macOS packaging succeeded. The idle
  installed app was replaced after strict deep signature checks; the preceding
  bundle remains at
  `~/Applications/Meetily Notepad.previous-20261014-090943.app`. Installation
  left the recovered library's file hash unchanged. Existing local summary
  selection remains `builtin-ai / qwen3.5:4b`.

The regression confirms that summary input is available after saving. It does
not evaluate generated-summary quality or physical microphone capture, and no
private meeting text was sent to a model during these checks. Reopen the app,
select the meeting in the library, and use **Enhanced notes → Enhance notes**
to generate a reviewed candidate.
