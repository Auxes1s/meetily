# Q01 and overlapping Whisper evidence

Current follow-up (15 October 2026): L05 is delivered and Q01's isolated macOS
workflow evidence remains passing. Q02 is active and incomplete. Its reproduced
virtual-clock benchmark and actual paced before/after comparison are recorded in
the [Q02 handoff](../../q02-native-validation-handoff.md). The pre-eager-admission 19.2% paced
inference improvement does not close the failed sustained-backlog/latency gate.
The evidence below describes Q01, including its historical install receipt;
The user subsequently prioritized continuous useful processing with large Whisper;
eager admission has passing short worker-continuity observations; full acceptance remains pending.
Q02 did not replace installed applications or modify production records/models.

The isolated macOS debug workflow passed on 2026-10-14. See
[`native-workflow.json`](native-workflow.json). It covers original editor saves,
native VAD/Whisper replay including pause/resume, persisted source/timing fields,
enhancement generation through a deterministic loopback provider, candidate edit
and approval, actual process restart, keyword search, JSON/Markdown export,
library deletion, and preservation of the original meeting/recording hashes.
Parsed exports matched persisted original, approved, and transcript content.

The real approximately 136-minute meeting was copied into isolated storage for
export/deletion checks. Its 412 passages came from a later mixed-audio
retranscription and are not a timing trace or accuracy reference for live ASR.
Durable deletion jobs are removed from copied databases and every retained
session recording path is rebased before the test app launches. A synthetic
Python contract test covers that isolation, including multi-session folders.

Initial Q01 implementation checks:

- Frontend: 122 passed, zero failures.
- Rust: 347 passed, eight ignored, one explicitly excluded known baseline
  `audio::device_detection::tests::test_builtin_mic_detection` failure.
- TypeScript and the packaged native/frontend build passed.
- Streaming tests cover clock gaps, repeated words, UTF-8, word boundaries,
  unresolved/forced-window boundaries, checkpoint retention/cleanup, and exact
  VAD audio preservation across incremental delivery and pause/resume.

## Default route and Beta switch

Continuous Whisper transcription is now enabled by default without a launch flag.
The native Beta preference controls both the pipeline and decoder, including
tray-started recordings, and is fixed for the duration of each session. Switching
it off selects the previous route for the next recording.

The isolated native workflow passed with both the
[default overlapping route](default-routing-overlapping.json) and the
[saved previous route](default-routing-legacy.json). Each run toggled the actual
**Settings → Beta → Continuous Whisper Transcription** switch in both directions,
verified the replay's selected route, and verified persistence across a full
process restart. The default run began without a transcription preference file.
These runs used public fixture speech and a deterministic enhancement provider;
they do not establish physical-device or model-quality acceptance.

Latest checks: 128 frontend tests and 352 Rust tests passed; TypeScript, the Python
isolation check, and native debug/release builds passed. Eight Rust tests remain
ignored and the known microphone-detection baseline remains explicitly excluded.
A pre-existing recording-session test compilation error was corrected by borrowing
its transcript fixture instead of cloning a type without `Clone`.

Both installed Finder app copies were updated to the verified release bundle
after the previous app quit through its normal note-save flow. Old bundles and
private SQLite snapshots were retained. The [install receipt](default-routing-install.json)
records matching bundle manifests/signatures, successful relaunch, library
integrity/foreign-key checks, preserved source content, and unchanged hashes for
all nine recording files. The existing recording-session migration backfilled
412 missing session IDs without changing passage IDs, text, or timing.

Private installation backups and full file manifests are kept in the local
directory identified by `/tmp/meetily-asr-default-install-current.txt`.

## Performance gate remains failed

[`overlapping-benchmark.json`](overlapping-benchmark.json) compares the same
90.112 seconds of copied meeting audio beginning at 3,600 seconds, using the
installed `large-v3` model. It uses measured inference time and a virtual capture
clock, not a paced device session or measured GPU utilization.

| Measure | Existing route | Overlapping route |
| --- | ---: | ---: |
| Inference time | 47.73 s | 94.02 s |
| Inference real-time factor | 0.530 | 1.043 |
| First update | 3.98 s | 3.89 s |
| p95 interval between updates | 7.21 s | 14.47 s |
| Simulated final backlog | 3.39 s | 20.01 s |

The repeated decoding did not improve performance on this excerpt. At the user's
request, the new route is now enabled by default for local Whisper, with a
persistent **Settings → Beta → Continuous Whisper Transcription** switch to use
the previous route. This default change does not turn the failed performance gate
into a pass. Text hashes
differ; neither output is an independently verified transcript accuracy
reference. Usable token timing was unavailable for some words, which keeps them
provisional until finalization. Additional tuning requires its own fidelity and
throughput evidence.

Physical capture, full-length paced replay, four-hour stress, Windows/device
acceptance, GPU/power traces, and real enhancement model quality/human sign-off
remain open. Fixture success does not close those gates.

Reproduce using [the runner and routing documentation](../../../continuous-transcription.md).
Private copied audio, transcript outputs, full logs and exports remain in local
`meetily-q01-*` temporary directories and archived isolated test profiles; they
are not included in this evidence directory.
