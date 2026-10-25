# P0 implementation handoff

Implementation authorized by the project owner on 2026-10-13. The planning audit
in [meeting-experience-upgrade.md](meeting-experience-upgrade.md) remains the
requirements reference. This handoff distinguishes implemented behavior from
acceptance evidence. The 13 October verification did not open, migrate or delete
a production meeting database, recording or installed application. The E08
addition below inventories selected settings read-only and hashes installed files.

For the latest cross-workstream delivery and acceptance status, use the
[current project plan](meeting-experience-upgrade.md#owner-closeout--17-october-2026).
The P0 baseline results below are historical checks, not the latest suite totals.

## Scope and ownership

The incoming section 5.1 notepad and recording/recovery work supplies N01–N06
and R01–R09. This integration preserves that work and adds A01–A03,
E01–E07, and L06, with E03–E05 and L01–L04 as required prerequisites. E08 evaluation
and L05 complete meeting export were added on 14 October as recorded below.
Release work Q01–Q04 was outside the original P0 assignments; subsequent Q01/Q02
work is recorded below and in the linked handoffs. The earlier original-note
Markdown/lossless JSON export from section 5.1 remains compatible; the separate
L05 control exports selected documents or a complete portable meeting record.

Incoming tracked changes and status were captured at
`/tmp/meetily-p0-before-1791899514/` before integration. The repository audit
baseline is commit `a2cb62e`. Changes remain in the working tree for review.

## Implemented checklist

| Tasks | Behavior and evidence | Acceptance still required |
| --- | --- | --- |
| A01 | Executable command inventory and before/after results below; disposable fixtures only. | Existing installed meeting and actual microphone/system recording on each supported native target. |
| A02 | Explicit draft/session/source contracts; linked SQLx SQLite WAL/FULL/FK/busy-timeout/FTS5 probe; paths and platform boundaries below. | Linked Windows runtime and supported minimum OS/CPU package verification. |
| A03 | Reproducible legacy/current migrations with four summary formats, existing/no notes, missing paths/times, live committed WAL, corrupt source, blocked backup destination, duplicate finalization, and abrupt subprocess termination. Exact original strings/JSON and identities are compared. | Real disk-full/locked-file and installer interruption scenarios. |
| N01–N04 | Incoming durable drafts, revisioned original blocks, acknowledged serialized saves, safe migration snapshots, and interruption recovery. | Native rich-editor and full crash/capture scenario matrix on both targets. |
| R01–R04, R05 handling | Incoming meeting/session IDs, native finalization, independently editable notes, durable recovery, and source-separated routing. Deletion integration gates late native artifact writers and checkpoint recovery. | R05's 45-minute/4-hour recordings, device/permission/tray failures, editor latency and ASR backlog measurements. |
| E01–E02 | Independent candidate/approved/archived/legacy versions; originals never used as the generation destination. Snapshot committed native notes/transcripts, template, instructions, language/provider configuration and source fingerprints. | Real provider/model and long-input acceptance. |
| E03–E05 | Bounded source-aware chunking/repair, exact source/excerpt validation, conservative review labels, named-owner/deadline literals required in excerpts, presets and locally saved custom templates. | E08 human checks for unsupported claims, omissions, uncertainty and multilingual quality. Citation validity is not semantic confirmation. |
| E06 | Separate review editor, revision/idempotent save protection, explicit approval pointer, archived previous approval, cancel/discard and preserved failed candidates. Legacy summary editing carries its displayed version/revision. | Full native editing/undo/relaunch and concurrent-view scenarios. |
| E07 | Parsed loopback classification; local inference refuses redirects/proxies; explicit destination/payload choice for remote generation; language/source preservation; touched search logging redacted. | Network-blocked installed-model workflow and egress inspection of packaged application. |
| L01–L04 | Manual participants/tags, dates/status, bounded local FTS projection, source-aware search, filters/sorts/cursors, stale-request protection, library UI and exact off-page passage lookup. Search excludes candidates/history by default and deleting meetings. | Labeled 1,000-meeting/100,000-segment ranking/latency workload and native UI source navigation. |
| L06 | Durable deletion intent, tombstones, native cancellation, validated owned artifacts, backup sanitization preserving other meetings, committed browser-cache cleanup, truthful pending status/retry on startup and in UI. Late DB/file/cache workers cannot restore a deleted identity. | Windows file locks/permissions and packaged restart/filesystem inventory scenarios. No secure-erasure claim. |
| L05 | Workspace/library export dialog; selected original/approved Markdown and versioned complete JSON, all transcript pages and frozen sources, meeting-scoped save acknowledgments, incomplete-snapshot labels, Unicode filenames and atomic local writes. Disposable native/macOS evidence below. | Windows native Save dialog and locked-destination acceptance; release workflow gates remain open. |

Unmet acceptance checks remain open in the original checklist. Passing deterministic
tests does not mark hardware, real-model, or release gates complete.

## Executable baseline and commands

Available test host: macOS 27.0.1 (26A434), Apple M5, arm64, 24 GiB RAM. Tools:
Bun 1.3.14, Node 26.7.0, pnpm 9.15.9, Rust/Cargo 1.99.0. The existing release
matrix builds `aarch64-apple-darwin` and `x86_64-pc-windows-msvc`. No OS minimum
or CPU feature requirement was changed. The existing manifest's Rust 1.77
declaration predates existing `LazyLock` use; minimum-toolchain compatibility
has not been established by a build on Rust 1.99.

At the intake checkpoint, 57 frontend tests, TypeScript, the standalone
onboarding model test and all 14 portable-GGML checks passed. Those were checks
of the incoming worktree, not a clean upstream native baseline. Final combined
results are recorded below.

| Check | Result |
| --- | --- |
| Native combined suite, excluding the unchanged baseline device-detection assertion | 306 passed, 0 failed, 5 ignored, 1 filtered out. |
| Linked SQLx runtime probe | SQLite 3.46.0; WAL; synchronous=2 (FULL); FK=1; busy timeout=5000 ms; native FTS5 query passed. |
| Synthetic abrupt termination and migration preservation | Passed in the combined native suite, including exact acknowledged JSON, independent candidate content, failed interrupted generation, source strings/IDs and integrity/FK checks. |
| Portable GGML Node suite | 14 passed. |
| Standalone onboarding model test | Exit 0. |
| Frontend Bun suite | 88 passed, 0 failed, 369 assertions across 21 files. Includes pending-commit getter/replay and edits arriving during lifecycle flush. |
| TypeScript | No-emit check passed after the production build completed. Run sequentially: Next regenerates `.next/types` during builds. |
| Production static export | Passed; library/workspace/detail routes compiled and exported. |
| Independent lint | Unavailable through the existing entry point; exits 1 at setup prompt. |

The first unfiltered native run was **303 passed, 2 failed, 5 ignored**. One
failure was the changed transcript export schema's stale `1.0` fixture assertion;
it now checks `1.1` and distinct Mixed/System provenance. The other is unchanged
at HEAD and in this worktree:
`audio::device_detection::tests::test_builtin_mic_detection` expects Unknown for
`MacBook Pro Microphone`, while the existing macOS detector returns Wired. The
green combined run uses
`-- --skip audio::device_detection::tests::test_builtin_mic_detection`; it is
not reported as an unfiltered pass. Five ignored tests comprise four
hardware/model/fixture scenarios and the subprocess helper that the crash test
executes explicitly.

```sh
# From frontend:
bun test tests
node tests/lib/onboarding-summary-model.test.mjs
pnpm exec tsc --noEmit --incremental false
pnpm build
pnpm lint

# From repository root, with cached native dependencies and Apple CLT:
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE \
PATH="$PWD/scripts/macos-clt:$PATH" \
  cargo test --locked --offline -p meetily --lib
node .github/verify-portable-ggml.test.cjs
```

Keep `pnpm build` and `tsc` sequential: an intermediate parallel run saw missing
generated `.next/types` files while Next rebuilt them. That generated-file race
was resolved by running the no-emit check after the successful build.

`pnpm lint` currently exits 1 at Next 14's ESLint configuration prompt. The
existing flat config is not picked up by that entry point, and its dependencies
are not installed in the current manifest. No interactive configuration or
unrelated dependency upgrade was performed. A passing production build/type
check is not reported as an independent lint pass.

## Frozen persistence and command contracts

- The canonical database is `<app_data>/meeting_minutes.sqlite`; startup/import
  snapshots use SQLite `VACUUM INTO`, not a raw copy that omits committed WAL.
  Connections use foreign keys, WAL, FULL synchronization, 5-second busy timeout
  and at most four pooled connections. Failed source integrity or snapshot
  creation stops migration and preserves the source. Windows directory syncing
  is separately bounded by its standard filesystem API; file contents are synced
  before snapshot rename.
- Four new additive migrations follow the shipped migration history:
  `20261013000000` notepad, `20261013010000` enhancements,
  `20261013020000` library, and `20261013030000` deletion. Their original-note,
  enhancement, index and deletion-job data have separate lifecycles.
- Notes use `(meeting_id, base_revision, commit_id)`; only a committed native
  acknowledgment advances Saved. Authoritative block JSON can save when a
  Markdown projection fails. Enhancement saves use the selected `version_id`
  and its revision/commit ID. Approval also compares the previous approved pointer.
- Recording identity uses a stable canonical meeting and recording session;
  repeated transcript finalization uses session/sequence identity. Legacy
  recovery uses a deterministic `recovered-<key>` meeting and
  `legacy-recovery-<key>` session. Content-free deletion tombstones prevent reuse.
- Typed audio provenance is `microphone`, `system`, `mixed`, or `unknown`;
  native storage reuses the nullable `speaker` column. Labels identify capture
  channels, never participants. Combined/imported audio is Mixed; missing old
  provenance is Unknown.
- An enhancement snapshot records original blocks/revision and ordered committed
  source passages with IDs, excerpts, fingerprints, timing and channel metadata.
  Changes to originals/transcripts make older versions stale. Generation status
  polling selects only status/revision/error rather than repeatedly transferring
  all frozen sources. Startup marks interrupted generation failed without
  replacing the approved document.
- Search is a rebuildable FTS5 projection. Canonical changes mark dirty IDs;
  bounded background projection is independent of acknowledgment. Queries have
  bounded literal terms, parameterized filters, deterministic pagination and
  explicit index-updating status. Source links carry canonical source IDs.
- Deletion commits intent before cleanup. Complete means both native and browser
  cleanup acknowledged. Backup copies are sanitized for the selected meeting,
  integrity/FK checked and verified to preserve unrelated rows. Invalid/locked
  backup or artifact paths keep the job pending. Imported originals, arbitrary
  exports, other meetings and models are preserved.

## Remaining mandatory gates

Windows packaged execution; minimum supported OS/CPU; native mic/system capture
with both Whisper and Parakeet; permissions/device/tray/hide/reopen scenarios;
45-minute and four-hour latency/backlog/memory measurements; installed-package
upgrade; large-library performance/ranking; real-model semantic review;
network-blocked local workflow and egress trace; native deletion lock/restart
inventory. None is counted as passed by unit tests, a frontend build, or the
synthetic subprocess crash fixture.

## E08 evaluation handoff — 14 October 2026

[Enhancement quality evaluation](enhancement-quality-evaluation.md) records the
12 source-labeled fixture sets, independent contracts, opt-in producer/reviewer
commands and all 96 generation jobs: two repetitions per fixture/model before
and after the focused fixes. Semantic assessment is Codex review; the linked
source/output sheets leave every human verdict pending.

Verified installed `builtin-ai / qwen3.5:4b` and loopback Ollama 0.40.3
`qwen3.5:9b-mlx` without downloading models or querying private sources. Exact
file/API digests, settings, template/context/hardware, raw/repair responses,
candidate/failure rows, elapsed times and original/approved protection comparisons
are in `docs/plans/evidence/enhancement-quality/`. Both matrix protection inventories
show identical installed library/WAL/model/helper hashes; all 96 synthetic original
and approval comparisons passed. The installed app, recordings and settings were
preserved. The 14 October transcript-persistence repair regression also passes in
the native suite.

Necessary E03/E04 fixes require every claim field (including nullable owner/date),
preserve explicit uncertainty through valid citations, and remove template display
metadata from the extraction prompt so the unchanged 4K planner can reach inference.
One repair remains the limit. No semantic keyword gate, dependency, model-catalog,
cloud fallback or installed-application change was added. Failing earlier runs are
retained alongside corrected ones.

| Corrected configuration | Structural candidates / jobs | Decision/action recall | Priority retention | Codex finding |
| --- | --- | --- | --- | --- |
| Built-in Qwen 3.5 4B | 19/24 | 28/38 (73.7%) | 37/50 (74%) | Unsuitable; five candidate claim occurrences assert unsupported confirmed decisions/commitments/scope. |
| Ollama Qwen 3.5 9B MLX | 16/24, including two empty candidates | 20/38 (52.6%) | 27/50 (54%) | Unsuitable; one such candidate occurrence plus other unsupported claims. |

Codex review found zero fabricated named owners/offices/deadlines and zero invalid
references accepted as valid in these runs. Semantic citation errors still occur;
reference validity never supplies AC07 acceptance. The ≥90% thresholds and zero
fabrication requirements are unchanged. Human acceptance and a suitable installed
model configuration remain pending; E08's acceptance checkbox stays open.

Final checks: Rust 320 passed, 8 ignored, one explicitly excluded baseline
microphone-classifier failure; that unchanged failure was rerun separately.
Focused summary contracts: 124 passed, one opt-in model test ignored. Both real-model
producer invocations completed, while the corrected Codex quality gate exits 1.
Python producer/metrics/review: eight passed. Bun: 97 passed. Portable GGML Node: 14 passed;
onboarding Node: exit 0. Production frontend build then no-emit TypeScript: exit 0
sequentially. Lint remains blocked at the existing ESLint setup prompt. Logs and
the child-process loopback-only sandbox probe are retained in the evidence package.
Ignored hardware/ASR cases, Windows/native WebView and whole-application/Ollama-server
egress gates are not counted as passed.

At the E08 handoff, the stop point was E08 and its demonstrated prerequisites;
L05 export was next and no package deployment had begun. L05 is now delivered
in the following bounded assignment.

## L05 complete meeting export — 14 October 2026

Implemented [meeting export](../meeting-export.md) with the project owner's
chosen defaults: approved Markdown when an actual approval exists, originals
otherwise; original/approved/both choices; full transcript optional and off by
default. Recording/finalizing and interrupted meetings may export labeled
committed snapshots. JSON always contains the full stored record independently
of Markdown choices. The existing note-only format and clipboard controls remain
compatible.

The native loader reads canonical metadata, original notes/links, every ordered
transcript page, enhancement settings and all versions/legacy results in one
SQLite snapshot. It does not adopt legacy versions or rebuild search. Raw
document/source strings and unknown content fields survive parse failures;
readable projections carry warnings. Approved evidence comes from frozen
sources, with retained excerpts/timing/channel labels even after passages change
or disappear. Original timestamp markers become offline references, including
when copied into approved notes. Capture channels do not identify participants.

Export waits for this meeting's original, enhanced and detached legacy saves,
including edits arriving while another editor saves. Failure/conflict stops
export and retains edits. Missing/deleting meetings are rejected; publication
uses the existing deletion lease after the Save dialog, so deletion during a
dialog prevents later publication. Native Save dialogs write synced same-folder
temporary files and atomically replace the destination. Cancellation is distinct
from success; partial-write/rename failures preserve existing output. Managed
application/recording folders cannot be export destinations. Generated metadata
omits absolute internal paths, endpoint configuration and credentials; original
user-authored content remains unchanged.

Evidence is retained in [the L05 evidence package](evidence/meeting-export/README.md).
Executed checks:

- Bun frontend suite: **108 passed**, including eleven new export orchestration
  and choice/ownership tests.
- Full native regression suite: **331 passed, 8 ignored**, with the previously
  documented `audio::device_detection::tests::test_builtin_mic_detection`
  excluded and recorded separately. Hardware/opt-in cases are not counted as
  passed.
- Native exporter: **11 passed**. Fixtures cover 1,207 passages (three native
  pages), all source values, exact acknowledged raw JSON, every version state,
  legacy/double-encoded/malformed data, off-page/frozen/changed/deleted evidence,
  consistent WAL snapshots, missing/tied times, incomplete states, conversion
  failure, changed approvals, overlapping timestamp ownership, reserved Unicode
  filenames, protected destinations and partial-write/rename failures.
- Production frontend build and subsequent no-emit TypeScript check: **passed**.
  Only new export Rust modules were formatted; no dependencies or migrations were
  added. Lint still exits at the pre-existing ESLint configuration prompt.
- Actual macOS arm64 debug WebView/native Save dialog in **com.meetily.export.test**:
  cancellation created no file; JSON and Markdown with the complete transcript
  saved through native dialogs; library export choices opened correctly. Parsed
  JSON and original raw blocks/revision matched the isolated SQLite fixture;
  both formats contained all 1,207 passages through sequence 1206. Source labels
  and Unicode/manual metadata survived. Dialog screenshots and readable file
  excerpts were inspected. Native default-approved/version-history cases are
  covered by deterministic fixtures, not real-model UI generation.

The test app was stopped after validation. This assignment used disposable
records and a separate test identifier; no installed application replacement or
production-library export/migration/deletion was performed. Intake hashes cover
320 pre-existing changed/untracked files; the preservation receipt confirms that
all changes to existing files are within the L05 allowlist.

Stop point: **L05 implementation and local macOS validation only**. Windows
native acceptance, Q01–Q04 release work and the previously reported E08 quality/
human-acceptance gaps remain open. Audio copying, PDF/DOCX export and record
import/restore were not added.

## Current follow-up: Q02 and delivered UI, 15 October 2026

The L05 stop point above is historical. L05 export is delivered, and Q01 has
passing isolated macOS native workflow/default/fallback-switch evidence.
The current authorized work stops at Q02. Its failed continuous Whisper
benchmark was reproduced; pre-eager-admission paced inference work improved 19.2%.
The user then prioritized continuous useful processing with large Whisper; eager
fresh-audio admission has passing short continuity observations. The original
sustained backlog and previous-route latency parity still fail. Native source
replay, physical two-session capture, packaged dependencies and remaining gates
are separated in the [Q02 handoff](q02-native-validation-handoff.md).
Q02 remains incomplete; Q03/Q04 and E08 remediation were not entered. Latest
verification ran 52 focused frontend checks and nine tooling checks, all passing;
the full native library suite reported 361 passed, one known microphone-classifier
failure and eight ignored. Lint configuration remains unresolved.

The [latest automated capture verification](evidence/capture-verification-20261015/README.md)
used fresh isolated debug capture and release replay packages from the current
source, preserving the installed production app. All eight paced source cases
passed with Whisper and Parakeet: microphone, system, simultaneous and alternating.
Ten sessions retained all 35 final passages through Stop/export, with exact source
PCM retention. Both providers also passed short actual system capture, native
pause/resume transitions, and two captures appended to the same meeting with
unchanged notes/title, separate playable audio files and continuous offsets.
The user selected automated checks only. Human microphone speech, distinct
simultaneous physical sources, extended physical pause exclusion, independent
listening and direct installed-production capture remain unverified. Authored
reference word differences persist; these receipts do not close speech fidelity.

The later whole-app UI design and interaction hardening are delivered in the
[UI plan](ui-design-upgrade.md). The latest interaction receipt records 166
frontend tests/641 assertions, production build and subsequent TypeScript,
91 native layout cases, 34 design cases, eight interaction scenarios and
122 continuous resize steps. These are isolated macOS UI checks, not physical
capture or duration acceptance.

The user subsequently authorized installation and stale-app cleanup. The
verified UI build is now `/Applications/meetily.app` (`com.meetily.ai`), and
its executable hash matches the [installation receipt](evidence/ui-install-20261015/README.md).
Earlier generated app bundles were archived under
`target/archived-app-bundles.noindex/20261015T103927/`. Latest capture verification
rebuilt the dedicated recording debug and Q02 release packages at their normal
bundle paths. Check the hashes in its receipt before reusing either artifact.

**Recommended next bounded automated assignment:** complete the latest-revision
45-minute Q02 normal workload with continuous large-v3, then Parakeet. Measure
foreground editing/save acknowledgment, native/WebView memory, queues/backlog,
dropped chunks and Stop/source/export reconciliation against AC05 before
scheduling the four-hour stress run. Keep replay separate from a physical
45-minute meeting. The latest short automated smoke is complete; human
microphone/listening/extended-pause gates remain open for an operator, alongside
the remaining native/platform matrix. Earlier 45-minute Whisper evidence predates
eager admission; the Parakeet run was aborted and the four-hour run was not started.
E08 remains an independent failed semantic acceptance gate. This documentation
refresh starts no new recording, duration run, Q03/Q04 or E08 remediation.


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
