# Performance optimization implementation

Authorized by the owner's “implement” response to the 25 October 2026 audit.
Preserve existing user work, installed applications, models and production data.
All runtime validation uses synthetic fixtures or isolated profiles.

## Implemented scope

| ID | Change and affected modules | Correctness safeguards |
| --- | --- | --- |
| S01 | `frontend/package.json` and lockfile pin ProseMirror view/model to 1.42.3/1.25.8. | Existing single-instance webpack aliases retained. Browser checks cover valid and invalid clipboard context plus real BlockNote editing. |
| F01 | `database/library.rs`: batch page participants/tags and rank source hits once for the selected page. Search enrichment uses three queries instead of three per result. | Parameter binding, source priority/order, eight-hit limit, snippets and metadata association compared with previous queries. |
| F02 | `audio/batch_audio.rs`, decoder, import and retranscription: packet decoding into private temporary PCM files, bounded sequential resampling/VAD, payload loading one segment at a time. | Same normalization, sinc resampler, endpoint policy and silence-split logic; existing live processing unchanged. Cancellation kills/waits for fallback FFmpeg and RAII removes temporary files. |
| F03 | `audio/transcript_journal.rs` and recording saver: append and sync one final passage per update; publish compatible atomic `transcripts.json` at initialization and Stop. | Sequence upserts preserve source labels. Journal records carry meeting/session identity. Recovery merges native final passages before SQLite acknowledgment, rejects active/deleted/wrong ownership, and retains audio checkpoints after failed reconstruction. |
| F04 | Enhancement repository/commands/panel: 50-row metadata pages with tuple cursors; fetch only the selected document body. Adopted collection reads avoid unnecessary writer acquisition. | Full legacy command defaults retained; scoped detail command checks meeting/version ownership. Same-revision completion, stale meeting responses, old selected versions and conflict reload are handled. |
| F05 | `meeting-notes-session.ts`: marker prefix index replaces the block-by-marker scan and quadratic duplicate checks. | Literal prefix behavior, history order, nested block ownership and delete/undo metadata preserved. |
| F06 | Additive `20261025000000_transcript_lookup.sql`: nonpartial meeting lookup index. | Existing partial uniqueness constraint retained; disposable migrations and query-plan regression test pass. |
| F07 | Meeting repository/API and sidebar: default sidebar loads 50 recent meetings. | Full legacy API remains available; currently selected older meeting and whole-library search results remain reachable; deterministic ordering and limit validation tested. |
| F08 | Notes/enhancement sessions and editor wrapper: prune clean inactive sessions to eight per cache and detach converter closures. | Dirty, writing, subscribed and retained sessions cannot be evicted. Explicit ownership covers the load-to-render handoff. Dirty sessions may deliberately exceed the cap. |
| F09 | Meeting documents, split view and summary panel: defer enhanced panel/summary editor until needed, then retain opened editors. | Forwarded save ref retained; tab switching, responsive layout, delayed navigation and document state tested. |
| F10 | Recording context/hooks/status bar and transcript service: separate elapsed clock subscriptions from controls; one provider owns native reconciliation; deduplicate simultaneous queue-status calls. | Lifecycle/session checks retained; elapsed-time updates do not rerender control consumers; no independent coordinator poll. |
| F11 | Library index worker: one synchronization owner, 50 ms target between meetings, durable pending-work recheck after releasing worker ownership. | Each meeting remains an atomic projection; canonical saves remain independent of indexing; failures retain dirty markers without a retry spin. One large meeting can exceed the time target. |

## Measured results

Local macOS development machine; synthetic fixtures and warm caches. These are
component/query/build measurements, not packaged end-to-end latency or ASR
accuracy claims. No percentage improvement is inferred for unmeasured workflows.

| Measurement | Audit baseline | Implementation | Scope |
| --- | --- | --- | --- |
| Meeting-details first-load JavaScript | 817 kB | 277 kB | Fresh Next production build; deferred editor bytes still load when used. |
| Library source enrichment p50 / p95 | 579.659 / 580.901 ms | 44.065 / 44.235 ms | Native test, 10,000 meetings / 200,000 sources / 30 results, 11 repeats. Old snippet queries versus actual new batched enrichment including metadata; exact snippet content/order compared. |
| Marker edit p95, 1,000 blocks / 200 remembered markers | 9.038 ms | 0.405 ms | Actual session method under Bun; excludes BlockNote, React and paint. |
| Marker edit p95, 2,000 blocks / 1,000 markers | 77.387 ms | 1.254 ms | Same method/fixture harness as baseline. |
| Transcript count p95 | 10.343 ms | 0.00346 ms | Python SQLite, same disposable 200,000-row fixture, warm cache; covering lookup replaces scan. Not API latency. |
| 45-minute / 48 kHz synthetic silence preparation | Not measured | 282.397 s; 100,859,904 bytes maximum RSS | Standalone native test process, decode/resample/Silero VAD; no ASR model or physical capture. |

The separate disk-backed SQLite comparison measured snippet p95 416.295 to
34.755 ms and checked exact rows. Its runtime differs from the native benchmark;
do not mix their numbers into one before/after pair. Transcript journaling is
linear in emitted record bytes: the 1,000-passage regression stays below 600 kB.
Per-record synchronization trades some small-write latency for durable recovery.

## Verification

Audit baseline: 193 frontend tests; scoped native database/audio/summary tests
337 passed, nine ignored. Final verification:

- Frontend: 204 passed, zero failed across 46 files.
- Native application library: 389 passed, zero failed, 11 ignored. The two new
  ignored benchmarks were also run explicitly and passed; the other nine remain
  outside this verification.
- TypeScript and fresh production build passed. Lint passed with the same
  115 existing warnings; existing Rust warnings remain.
- Effective dependency graph contains only ProseMirror view 1.42.3 and model
  1.25.8. A disposable Chrome profile passed the repository browser fixture.
  A separate original-view 1.41.3 negative control accepted malformed context
  attributes that the patched view rejected; this tests the vulnerable boundary,
  not a complete arbitrary-code-execution chain.
- Exact short WAV/AAC decoded PCM, VAD timestamps and segment payload parity
  tests pass. Long split boundaries/samples, cancellation, journal identity/torn
  tails, final snapshot, recovery acknowledgment ordering, failed audio retention,
  autosave/conflicts, pagination, deletion and recording ownership are covered.
- All 1,279 pre-existing files remain present. Hash comparison found no changes
  to pre-existing documentation/scripts, including the dirty release document.

Run from `frontend`:

```sh
bun test --isolate tests
pnpm exec tsc --noEmit --incremental false
pnpm lint
pnpm build
```

The production build for this change ran in a disposable copy of the frontend
using the installed dependencies, preserving the checkout's build outputs.
From `frontend/src-tauri`:

```sh
cargo test --locked --offline -p meetily --lib -- --test-threads=1
cargo test --locked --offline -p meetily --lib database::library::tests::enrichment_performance_benchmark -- --ignored --nocapture --test-threads=1
cargo test --locked --offline -p meetily --lib audio::batch_audio::tests::long_silent_batch_resource_benchmark -- --ignored --nocapture --test-threads=1
```

For peak native RSS, run the compiled test executable under `/usr/bin/time -l`
with the same filter and flags; timing Cargo itself includes compilation memory.
The browser fixture is `frontend/tests/fixtures/editor-paste.browser.js`. Bundle
it with Bun's browser target into a temporary directory, serve it over localhost
from a UTF-8 HTML host containing `<pre id="result">` and the bundle script,
then verify the resulting JSON has `ok: true` in a disposable browser profile.
No browser automation dependency was added.

Raw baseline/implementation diagnostics are in disposable OS temporary folders
identified by `/tmp/meetily-perf-audit-current` and
`/tmp/meetily-optimization-current`. They are not durable repository artifacts.
The security plugin's persistent-artifact helper failed because its Python
runtime was unavailable; the browser regression source remains in the repository
and its local test output remains in the implementation diagnostics directory.

## Risks, rollback and remaining acceptance

- **Temporary disk:** batch preparation uses disk proportional to duration:
  source-rate mono float PCM plus 16 kHz float PCM coexist during conversion.
  At 48 kHz, allow roughly 0.92 GB per hour, plus any fallback WAV. Disk-full
  errors stop processing; source files are never truncated. ASR model RAM is
  additional and was excluded from the resource benchmark.
- **Audio parity:** the measured long resource case is silence. Differential
  tests establish short WAV/AAC and long splitter parity; long compressed speech,
  inference quality and native UI latency still need release-fixture validation.
  No model or semantic-quality improvement is claimed.
- **Migration:** first open creates the additive lookup index using the existing
  integrity-checked backup/migrator. Index creation needs transient disk and a
  write lock. Production data was not opened or migrated during this work.
- **Journal compatibility:** external readers of `transcripts.json` see the last
  published snapshot while recording; final passages are durable in the journal.
  Stop publishes the compatible complete snapshot. Before rolling back an app
  with interrupted recordings, recover/publish those journals with this version.
  Older binaries do not understand them. Retain original journals and backups.
- **Rollback:** revert only this change's source/package/test files and new
  migration; preserve the pre-existing dirty work. Restore the matching lockfile
  and reinstall from it. Use a backed-up disposable database to validate binary
  rollback: older SQLx migrations may reject a newer migration version. Never
  restore an old production backup over newly captured data without first
  exporting/reconciling that data.
- **Release acceptance:** no app was installed, replaced or launched against the
  production library. Native WebView, Windows and physical-device acceptance for
  these changes remain unverified. The owner's earlier long-recording approval
  and Windows deferral in `q02-native-validation-handoff.md` remain unchanged.

## Deferred conditional work

The local Tauri/SQLite architecture remains appropriate. No server, cache service,
new runtime dependency, global scheduler, model/context lifetime rewrite or
dirty-kind schema was introduced. Profile collocated model contention and
single-large-meeting index lock duration before those changes. The lookup index,
bounded batch pipeline and worker ownership address the demonstrated costs first.
Do not remove audit logging, weaken durability/validation, drop the partial
uniqueness constraint, unmount dirty editors, or add blanket memoization.

Release order: dependency patch and lookup migration; library batching and marker
scan; bounded metadata/sidebar/cache/loading; recording/index ownership; journal
and batch audio. Re-run the regressions above after each independently reviewed
batch, then exercise interrupted recovery, long compressed speech and native
editor/recording interactions in an isolated release profile before distribution.

## Installed release — 25 October 2026

After the implementation closeout, the owner explicitly requested a rebuild and
replacement of the installed app. `scripts/meetily-notepad.sh build` completed;
the bundle and both helpers passed deep/strict signature verification. The
verified bundle now replaces `/Applications/meetily.app` and is running from that
path with a visible native window. Identifier remains `com.meetily.ai`, version
0.4.1. Main executable SHA-256:
`e7d7c7b8a598e4ed329e80d622a8dd90acbe410c90f7413d2333294708855b46`.

Startup applied the two pending migrations through `20261025000000`. Integrity
and foreign-key checks passed. All 17 pre-existing canonical tables matched the
pre-install hashes: nine meetings and 4,483 transcript rows. JSON settings and
model file sizes/modification times remained unchanged. Build/install changed no
repository files; this delivery note was added afterward. This is startup and
library-preservation evidence; physical capture and long-speech acceptance for
the new processing path remain separate.

Private rollback files, prior bundle, database snapshot, settings and receipts:
`~/Library/Application Support/Meetily Backups.noindex/20261025T111203/`.
The installed binary matches the freshly built bundle file-for-file. The earlier
source-only delivery boundary above describes the implementation closeout before
this separately authorized installation.
