# ASR and UI pipeline reinforcement — 15 October 2026

The R10–R13 / U12–U15 implementation is delivered. Verification receipts and
short replay results are in [the evidence record](evidence/pipeline-reinforcement-20261015/README.md).
This work does not close Q02's 45-minute/four-hour, physical-capture, fidelity,
or Windows acceptance gates. It makes no change to E08/AC07's failed status.

## Native pipeline

- Capture and ASR intake use nonblocking, byte-budgeted PCM handoffs: 8 MiB
  each. The mixed-recording handoff holds at most another 30-second batch at
  48 kHz. These are queued-PCM limits, excluding active buffers, models and
  normal runtime allocations. Saturation is an explicit failure, never a
  successful dropped-chunk acknowledgment.
- Both Whisper routes and Parakeet consume a shared durable source inbox.
  Private PCM and identity/source/timing sidecars are flushed before publication.
  A monotonic disk cursor reads one descriptor at a time; inference lag creates
  disk backlog rather than a growing in-memory PCM/descriptor queue. Source
  checkpoints survive inference until canonical persistence acknowledges them.
- Continuous Whisper accepts verified duplicate PCM overlap, preserves new
  suffixes and rejects conflicting overlap. Real forward gaps seal the preceding
  hypothesis and restart at the actual source clock with a warning. Missing
  audio is not synthesized. Two-sample timing tolerance is unchanged.
- Fresh-audio admission, one resident decoder, unchanged-preview avoidance,
  model catalogs, effective decode settings and the persistent Beta fallback
  remain in place. Stop has no elapsed-time ASR-backlog cutoff.
- One blocking saver owns checkpoint encoding. It streams existing PCM slices
  into FFmpeg, avoiding the flattened batch copy. Mixed PCM is also flushed to
  private raw checkpoints; a broken encoder switches to bounded-memory raw
  retention. Recovery can encode a raw tail after interruption. Encoded files
  publish by rename after FFmpeg succeeds. Managed deletion recognizes these
  owned artifacts through its existing ownership/tombstone safeguards.
- Stop closes producers, joins accumulation, drains inference and then finalizes
  files/SQLite. Encoding, pipeline and persistence errors remain interrupted
  outcomes. Blocking writers keep their own deletion lease if an awaiting
  operation is cancelled. A native atomic guard admits only one Stop tail.
- The saved mixed recording uses unity gain up to amplitude 0.9 and a symmetric
  tanh soft knee above it. Source-separated ASR receives the existing separate
  channel path. This is a mixer correction, not acoustic-quality acceptance.

## Frontend

- Committed passages and provisional tails have separate storage and row
  subscriptions. Stable IDs use incremental ordering; partial revisions neither
  copy/sort history nor notify unrelated rows. Lifecycle actions do not subscribe
  to transcript content. Full snapshots remain available for Copy/Stop/recovery.
- Confirmed finalizing sessions retain their live transcript and reconcile earlier
  canonical recordings after capture stops. Session offsets and provenance
  continue through existing adapters.
- The actual scroll container owns following and reading anchors. Layout/frame
  updates replace delayed smooth-scroll jobs, including partial-only DOM changes.
  Virtualizer keys use passage IDs and browser anchoring does not compete with it.
- Recovery writes serialize and coalesce superseded revisions. Transcript writes,
  metadata writes and the saved-to-SQLite marker wait for transaction completion.
  Aborted writes reject acknowledgments and remain retryable. Cache failure is
  visible without preventing note editing.
- Recording controls share a synchronous operation guard across remounts, expose
  shared pending state and retain the existing component guards. Native Stop,
  Pause and Resume reject stale supplied session identities; pause/resume events
  carry identity and stale acknowledgments cannot update the active recording.
- The registered status command now returns actual session counters rather than
  its former zeros. The status bar separates capture, pending transcription,
  finalization and interrupted processing. Missing status is unavailable, not zero.

## Verification boundary and next work

The evidence record distinguishes passing regressions and short isolated replay
from acceptance. Native replay runs exercise source VAD, installed models, IPC,
WebView editing, SQLite and export, but bypass physical devices and the full
RecordingManager capture path. Saver tests separately exercise real FFmpeg,
queue drainage and short-tail recovery on disposable folders.

Next: latest-revision 45-minute workloads with continuous large-v3, then Parakeet,
before four-hour AC05. Keep foreground editor/save measurements, queue/resource
traces and exact final source/text/timing/export reconciliation. Human listening,
physical pause exclusion, native failure/upgrade and Windows checks stay open.

The installed production app and library are preserved. The newly built bundle
uses `com.meetily.q02.test`; it is ad-hoc signed, unnotarized and not installed.
No dependencies, models, database migrations or production defaults were changed.
