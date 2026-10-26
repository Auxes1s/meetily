# Continuous Whisper transcription and Q01

Whisper can now receive incremental speech audio and repeatedly decode overlapping
windows while the latest live passage remains revisable. The selected model and
microphone/system provenance stay intact. Final passages are persisted once.

The route is **enabled by default** for local Whisper. **Settings → Beta →
Continuous Whisper Transcription** switches it off to use the previous route.
The choice persists in the native `transcription_preferences.json` store, applies
to the next recording (including tray starts), and survives app restarts. The
pipeline and decoder use the same choice for the entire session. Parakeet retains
its existing route. No launch environment variable is required.
Real-model throughput, transcript fidelity, memory, and paced native replay
acceptance remain open; changing the default does not close those gates.
Q02 reproduced the failed benchmark. Its pre-eager-admission measurement reduced
inference work by 19.2%, with the original backlog/latency gate still failed. The
user subsequently prioritized continuous useful processing with large Whisper;
fresh speech now becomes eligible whenever the decoder is free. See the
[Q02 handoff](plans/q02-native-validation-handoff.md) for current evidence.
GPU occupancy alone is not an acceptance measure: silence and CPU decoding work
can leave the GPU idle even with a correctly scheduled continuous pipeline.

## Routing contract

- Incremental VAD delivery advances every two seconds of speech, with explicit
  natural endpoints and Stop flushes. VAD/capture clocks are not reset at delivery
  boundaries. Source audio remains separate after echo cancellation.
- Windows target five seconds with three seconds of overlapping context. Audio
  which has not been finalized is retained; unresolved windows can extend to
  20 seconds before a final decode.
- One blocking worker reuses one Whisper decoder allocation. Each source has
  separate audio/hypothesis state. Superseded preview work can be coalesced while
  every source sample remains retained.
- Fresh source audio is admitted as soon as the worker is free. A superseded
  preview coalesces only when that same source has pending delivery, retaining
  all PCM. Endpoints, full windows and Stop finalize uncommitted audio; already
  finalized retained context does not require another decode. A forced full window that cannot advance reports an error and
  retains source audio rather than looping forever. Decoder parameters are
  shared with the previous route for controlled comparisons.
- Matching timed token prefixes in two successive decodes can finalize. Wholly
  untimed speech remains provisional until an endpoint. Overlap reconciliation
  requires matching token IDs/bytes and intersecting absolute source positions,
  with timed anchors around any untimed prefix tokens. The public alternating
  fixture now retains repeats without the intermediate extra prefix; word edits,
  accent losses and independent listening review keep fidelity acceptance open.
- A provisional passage retains its sequence ID and increments `update_revision`.
  A final passage seals that ID. Missing revisions default to zero for existing
  event producers. Partial text is excluded from committed exports/enhancements.
- Private source PCM and capture-clock/provenance sidecars remain under the
  recording's `.checkpoints` until final SQLite acknowledgment. Failed sessions
  retain them. Managed deletion recognizes these files; cleanup preserves other
  sessions and unknown/user-owned files.

This adapts persistent-engine/source-routing ideas from
[foresight-zh](https://github.com/michaelpersonal/foresight-zh/tree/8f15aea6100623daef04613918fba2edf7f50f4a)
and overlapping audio from [Whisper's streaming example](https://github.com/ggml-org/whisper.cpp/tree/master/examples/stream).
The Foresight shim uses an offline CPU recognizer at speech endpoints; it does
not establish that continuous GPU occupancy improves performance.

## Run Q01

```sh
python3 scripts/run-meeting-workflow.py
```

The macOS runner builds an isolated debug `com.meetily.workflow.test` application,
uses existing Whisper `large-v3` weights read-only, and runs a deterministic
loopback enhancement provider. It copies the longest completed meeting from the
live library by default. `--meeting-id` selects another completed meeting;
`--synthetic-only` omits the real library copy. `--routing legacy` selects the
comparison route. `--skip-build` reuses an existing workflow test bundle.
The runner exercises the Beta switch using the same native preference as the
ordinary app; the default run starts with no saved transcription preference.

The original library is read-only. Recording paths in the copied database are
rebased into the disposable run directory before the test application opens it.
Previous workflow profiles are preserved under a unique `.previous-*` name.
Concurrent workflow runners are excluded by a process lock.

The workflow writes notes in the native editor, replays public speech through
production VAD/Whisper/events/persistence, generates a candidate through the
production enhancement API, edits and approves it, exits and restarts the actual
process, searches saved content, exports JSON/Markdown, and deletes through the
library UI. It also exports/deletes the real copied meeting. The runner parses
the exports and compares original meeting/recording hashes before and after.

Results and local logs are kept in the printed `meetily-q01-*` run directory.
Those directories can contain private copied recordings and exports. Reports
distinguish native IPC/persistence coverage from physical capture and model
quality acceptance. The debug export destination hook replaces the Save dialog
only in this explicitly enabled isolated profile; ordinary exports retain their
native destination dialog and ownership checks.

## Compare real audio

```sh
python3 scripts/run-meeting-workflow.py --synthetic-only \
  --benchmark-audio /path/to/disposable/audio.mp4 \
  --benchmark-start 60 --benchmark-seconds 90
```

Both routes receive identical bounded 16 kHz PCM; compressed-audio full-scale
overshoots are counted and clamped. `benchmark.json` contains inference time,
real-time factor, first-update timing, update intervals, simulated drain backlog,
token-timing availability, and output hashes. Transcripts remain local files.

This benchmark uses measured inference durations with a **virtual capture
clock**. It is not a paced native capture run, a GPU utilization trace, or a
transcript accuracy reference. Full-length paced replay, device/dual-source
checks, manual boundary/correctness review, and platform acceptance remain
separate acceptance gates. The dense-speech comparison currently shows more
inference work and backlog for the overlapping route; the Beta switch provides
the previous route when needed.
