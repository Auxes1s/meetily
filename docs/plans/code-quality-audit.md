# Meetily code-quality audit

Audit date: 15 October 2026. **Report only; no application fixes were made.**

## Assessment

Meetily has substantial safeguards and useful regression coverage, but the reviewed working tree is **not ready for an unqualified production-quality sign-off**. Four failure cases were reproduced in disposable source copies: notes with an earlier recording's timestamp cannot be saved after another recording is appended; a failed Stop database update consumes the recording manager while status still reports recording; a pipeline task panic is acknowledged as successful shutdown; and an apostrophe in the recording directory prevents final audio concatenation. The editor also resolves an affected dependency version covered by a current security advisory.

Frontend tests, the production frontend build, TypeScript, and auxiliary tooling tests passed. The native suite retains its known microphone-classifier failure. Lint, formatting, and Clippy do not pass, and repository workflows do not enforce the full checks on pull requests. These results support targeted remediation, followed by repeatable quality gates. They do not support a broad rewrite or a claim that every module has been audited.

“Industry standard” is assessed here through observable properties: clear state ownership, correct and durable acknowledgments, errors that reach the caller, bounded resource use, validation at native boundaries, deliberate access permissions, useful regression tests, and automated enforcement. Test counts and stylistic preferences alone do not establish those properties.

## Findings and recommended repair order

P1 means address before a production release; P2 means address in hardening before final quality acceptance. The evidence label distinguishes reproduced behavior, source-confirmed exposure, and assurance gaps. No P0 issue was established by this review; this is not an assertion that none exists.

| ID | Priority | Finding | Evidence |
| --- | --- | --- | --- |
| CQ01 | P1 | Earlier recording timestamps prevent subsequent note saves after append | Reproduced against native save/session code |
| CQ02 | P1 | Stop can remove the manager and return before shutdown on a database error | Reproduced with native command, mock app, and failing SQLite trigger |
| CQ03 | P1 | Pipeline task panic/cancellation becomes successful shutdown | Reproduced by injecting a failed pipeline task |
| CQ04 | P1 | Active editor resolves ProseMirror version affected by paste XSS advisory | Dependency resolution and upstream advisory; exploit not reproduced |
| CQ05 | P2 | Valid apostrophes in recording paths break final audio concatenation | Reproduced with actual FFmpeg and disposable synthetic audio |
| CQ06 | P2 | Normal logging includes transcript text and credential-bearing configurations | Source-confirmed logging paths; no private content used |
| CQ07 | P2 | Pull-request quality checks are absent from repository workflows | All eight workflow definitions inspected |
| CQ08 | P2 | Static-analysis entry points cannot provide a passing quality baseline | Lint, formatting, and two Clippy invocations executed |

### CQ01 — Appending a recording makes existing timestamped notes unsavable

**Source:** [notes_commands.rs](../../frontend/src-tauri/src/database/notes_commands.rs), lines 108–114; [recording_sessions.rs](../../frontend/src-tauri/src/database/recording_sessions.rs), `mark_live`, lines 106–123; [MeetingNotesPanel.tsx](../../frontend/src/components/MeetingWorkspace/MeetingNotesPanel.tsx), lines 54–64 and 69–83.

**Trigger:** Save a timestamp associated with session A, finish A, append session B to the same meeting, and then edit or save the original notes while retaining A's timestamp.

**Observed behavior:** `save` compares every note link's session with the single latest `meetings.recording_session_id`. `mark_live` changes that field to B. The next save returns `invalid_document` for the still-valid link to A. The disposable probe used the real session begin/mark-live operations and save repository; the prior saved note revision remained 1 and the later edit was rejected. `MeetingNotesPanel` flushes before opening markers and exporting notes, so those actions can also fail while this condition remains.

**Impact:** Notes containing timestamps from earlier recordings cannot be updated after append without removing the old links. Failed newer edits remain unacknowledged; the earlier committed version is preserved. Existing append and timestamp tests cover these behaviors separately and missed their interaction.

**Correction:** Validate timestamp ownership against any recording session belonging to this meeting, retaining compatibility for explicitly supported legacy identities. Preserve the marker's own session and local offset. Keep cross-meeting/session rejection, document/link validation, revision protection, and deletion guards.

**Verification:** Add a regression combining timestamp creation in A, append B, a document containing both A and B markers, save/reopen/export, and source navigation. Keep wrong-meeting, failed/cancelled-session policy, deleted-meeting, invalid-offset, and lost-acknowledgment cases explicit. Never fix this by rewriting A's marker to B.

### CQ02 — A failed Stop database update abandons the normal shutdown tail

**Source:** [recording_commands.rs](../../frontend/src-tauri/src/audio/recording_commands.rs), lines 622–646, 970–973, and 1156–1170; [recording_manager.rs](../../frontend/src-tauri/src/audio/recording_manager.rs), lines 620–624.

**Trigger:** The database write that changes a session to `finalizing` fails after `RECORDING_MANAGER.take()`.

**Observed behavior:** The database write uses `?`, returning before streams/pipeline are explicitly drained, files are finalized, final transcripts are committed, or the global recording flag is cleared. `StoppingGuard` clears only the stopping flag; dropping `RecordingManager` calls synchronous state cleanup and does not perform the async Stop tail. A disposable probe invoked the actual command using a Tauri mock app and SQLite trigger that rejected this write. The result was an error, no global manager, `IS_RECORDING == true`, `IS_RECORDING_STOPPING == false`, and reported phase `recording`.

**Impact:** The control/status state no longer represents an owned live recording, and retry cannot recover the removed manager's normal finalization path. Existing checkpoints may still support recovery; this probe does not establish physical audio loss or power-failure behavior.

**Correction:** Make Stop retain ownership through every fallible preparation step. Treat a status-write failure as a recorded completion error while still running the required capture-stop, drain, retention, and terminal-state cleanup. Ensure one explicit interrupted result reaches both native status and the frontend. Do not introduce an elapsed-time cutoff that drops ASR backlog.

**Verification:** Inject status-write, pool-unavailable, pipeline, encoder, and final canonical-write failures. Check terminal state, Stop retry, exactly one owning tail, retained source/checkpoints, reopen/recovery, and a subsequent new recording. The supplied probe provides the initial failing boundary without physical devices.

### CQ03 — Pipeline join failures are returned as success

**Source:** [pipeline.rs](../../frontend/src-tauri/src/audio/pipeline.rs), lines 980–995; [recording_manager.rs](../../frontend/src-tauri/src/audio/recording_manager.rs), lines 353–362; [recording_commands.rs](../../frontend/src-tauri/src/audio/recording_commands.rs), lines 657–665 and 1001–1028.

**Trigger:** The pipeline task panics or is cancelled, producing a Tokio `JoinError` rather than returning its inner `Result`.

**Observed behavior:** `AudioPipelineManager::stop` logs the join failure and returns `Ok(())`. The disposable probe supplied a task that panicked and verified that Stop returned success. The manager's force-flush path delegates to this method, so the error cannot contribute to the normal completion-error decision through that return value.

**Impact:** A failed pipeline is indistinguishable from a successful drain to the caller. When no other error is recorded, the later completion path can classify the recording as completed. The probe establishes false acknowledgment; it does not establish that every panic discards audio or that another layer can never detect a related failure.

**Correction:** Propagate the join error as a failed drain, preserve the underlying cause, and retain the interrupted outcome and recovery assets through canonical persistence. Keep normal end-of-input separate from task failure.

**Verification:** Exercise normal drain, explicit inner error, panic, cancellation, and duplicate Stop. Assert completed status and source cleanup only for the successful case; failed cases must retain recovery assets and disclose interruption.

### CQ04 — The editor uses an affected ProseMirror dependency

**Source:** [package.json](../../frontend/package.json), lines 89–95; [next.config.js](../../frontend/next.config.js), lines 29–44; [Editor.tsx](../../frontend/src/components/BlockNoteEditor/Editor.tsx), lines 18–21 and 48.

**Evidence:** The package override pins `prosemirror-view` to **1.41.3**. Resolving it through the same `@tiptap/pm/model` base used by the webpack alias confirms that installed version. The [maintainer's advisory GHSA-c8x8-7fp4-3x9w](https://github.com/ProseMirror/prosemirror-view/security/advisories/GHSA-c8x8-7fp4-3x9w) reports attacker-supplied HTML paste causing JavaScript execution in affected ProseMirror editors, with versions below **1.42.3** affected and **1.42.3** patched. Meetily exposes a normal editable BlockNote editor; a custom protective paste handler was not found in that wrapper.

**Impact and limit:** This is a confirmed affected dependency with a relevant user interaction. No exploit was executed, and the exact payload's behavior under Meetily's CSP, BlockNote configuration, and native permissions has not been established. It is not a demonstrated native-code execution finding. The main WebView's file permissions and app-data asset scope make containment review important, but configuration alone does not prove an exploit chain.

**Correction:** Test the smallest compatible dependency change that removes this affected version, using the advisory's patched range as a minimum rather than blindly selecting a major upgrade. Preserve the intentional single-instance ProseMirror resolution across BlockNote and Tiptap. Review frontend filesystem access and native command validation alongside this work using [Tauri's capabilities guidance](https://v2.tauri.app/security/capabilities/).

**Verification:** Verify the resolved dependency tree and shipped bundle, then rich-text paste, links/timestamps, undo/redo, note and enhanced-document persistence, export, and both native targets. Perform any malicious-paste regression only in an isolated test profile. Record CSP behavior separately from dependency remediation.

**Other advisory results:** `pnpm audit --prod --json` returned 52 advisory records: 2 critical, 25 high, 21 moderate, and 4 low. These are scanner records, not 52 independently confirmed exploitable application flaws. Both critical records concern Next.js server behavior: [Windows-hosted servers](https://github.com/vercel/next.js/security/advisories/GHSA-p293-qw3h-jr36) and the [AVIF Image Optimization API](https://github.com/vercel/next.js/security/advisories/GHSA-2xp9-vwfh-vxw4). Meetily currently uses `output: 'export'` and unoptimized images, so this audit does not establish those server entry points in the shipped desktop app. Other records still need individual reachability/deployment triage. Rust advisory scanning was blocked because `cargo-audit` is not installed; no tool was installed.

### CQ05 — Apostrophes in recording directories break finalization

**Source:** [incremental_saver.rs](../../frontend/src-tauri/src/audio/incremental_saver.rs), lines 166–177 and 347–350.

**Trigger:** Select a valid recording location containing an apostrophe, such as `Marc's recordings`, and finish a recording.

**Observed behavior:** Both normal finalization and recovery build FFmpeg concat entries as `file '<absolute path>'` without escaping the concat-file grammar. A real-FFmpeg probe wrote synthetic PCM under such a directory, successfully encoded its checkpoint, and received `FFmpeg concat failed` on finalization. The encoded checkpoint remained available.

**Impact:** A valid directory prevents final audio publication and the same unescaped recovery construction can prevent recovery. This is path-formatting failure, not shell injection: the subprocess is invoked with an argument array.

**Correction:** Use one tested encoder for FFmpeg concat-file paths in both finalization and recovery. Support spaces, apostrophes, Unicode, and platform-specific path syntax; deliberately handle newline-containing filenames. Do not change the user's recording root or strip characters from an existing path.

**Verification:** Run actual encode/finalize/recovery with ordinary, spaced, apostrophe, and Unicode roots. Preserve checkpoints and any prior published audio on failure. Include Windows path cases in native Windows verification.

### CQ06 — Transcript text and API-key configurations enter ordinary logs

**Source:** [worker.rs](../../frontend/src-tauri/src/audio/transcription/worker.rs), lines 205–208, 241–244, and 286–292; [main.rs](../../frontend/src-tauri/src/main.rs), lines 9–11; [ConfigContext.tsx](../../frontend/src/contexts/ConfigContext.tsx), lines 201–207 and 330–336; [settings/page.tsx](../../frontend/src/app/settings/page.tsx), lines 34–40; [api.rs](../../frontend/src-tauri/src/api/api.rs), lines 625–632.

**Trigger:** Successful transcription on the legacy Whisper, Parakeet, or provider route, or loading/updating a provider configuration that contains an API key.

**Evidence:** Success logs interpolate the recognized text at `info` level, while the native entry point enables info logging. Frontend logs print whole transcript configuration objects and model-update event payloads. The native transcript configuration response includes the API-key field; the settings page logs that response directly.

**Impact and limit:** Confidential meeting content and provider credentials can be exposed through ordinary native output or WebView console collection/support captures. The review did not inspect private profiles, capture a real credential, establish automatic log upload, or establish the exact disk-retention destination. Continuous Whisper's separate live worker is not claimed to use every legacy log site.

**Correction:** Replace content/config-object logging with explicit safe metadata fields. Never log API keys or recognized text under ordinary operation. Keep diagnostic errors useful without serializing request/response bodies or credential-bearing objects.

**Verification:** Use synthetic canary text and fake keys, exercise configuration load/update and all transcription routes, and assert absence from native output, console captures, and any collected diagnostic artifacts. Q03 must still verify packaged egress and retention.

### CQ07 — Repository workflows do not enforce pull-request quality gates

**Source:** [.github/workflows](../../.github/workflows), all eight YAML definitions; [pr-main-check.yml](../../.github/workflows/pr-main-check.yml), lines 1–7.

**Evidence:** The workflows are manually dispatched or reusable through `workflow_call`; none declares a pull-request trigger. No workflow explicitly runs the Bun suite, Rust tests, TypeScript command, lint, Clippy, or formatting check. The file named `pr-main-check.yml` is a manually dispatched version/configuration check. Native package builds can indirectly invoke the frontend build, which provides some compilation checking, but they do not substitute for the missing tests and lint.

**Impact:** The repository configuration does not automatically detect regressions on a proposed change or establish the quality baseline before packaging. Server-side branch protection and external organization checks were not queried, so this finding concerns checked-in workflows only.

**Correction and verification:** Implement the proposed gates below after the failing baseline is corrected or explicitly tracked. Open an isolated test PR and show that each intended failure blocks its job; verify required-check configuration separately. Preserve release/signing behavior and keep publication separately authorized.

### CQ08 — Static-analysis and test entry points need a usable baseline

**Source:** [package.json](../../frontend/package.json), line 28 and development dependencies; [eslint.config.mjs](../../frontend/eslint.config.mjs), lines 3 and 12–13; [build/ffmpeg.rs](../../frontend/src-tauri/build/ffmpeg.rs), lines 96 and 210; [backend_config.rs](../../frontend/src-tauri/src/audio/capture/backend_config.rs), line 55.

**Observed results:**

- Lint exits 1 at Next's interactive ESLint setup prompt even with CI mode and closed stdin. `eslint`, `eslint-config-next`, and `@eslint/eslintrc` are unavailable in the existing frontend dependency installation. The checked-in config references the latter packages; build success does not establish a lint pass.
- `cargo fmt --all -- --check` exits 1 and reports differences in 134 files across the Rust workspace. This is formatting debt, not 134 correctness defects.
- Strict Clippy stops in the build script on an unused import and an inefficient double-ended iterator operation. A second diagnostic invocation without `-D warnings` reaches application/tests but still exits 101: 195 unique warnings and four unique errors after deduplicating library/test diagnostics. The errors concern an inherent `to_string` shadowing `Display`, an always-true test comparison, and two unchecked test-server read amounts. They have different impacts; do not label all diagnostics runtime bugs.
- The native baseline has 371 passing tests, one known classifier failure, and eight ignored tests. The failed classifier must be adjudicated against intended detection behavior before changing the test or implementation.

**Correction:** Make lint deterministic using pinned compatible packages and a matching noninteractive entry point. Establish reviewed Rust lint rules, fix actionable diagnostics and the known failure, and classify external-device/model tests explicitly. Keep formatting changes in a separate mechanical batch. Do not suppress entire lint groups, weaken assertions, or silently skip the failing classifier to obtain a green badge.

**Verification:** Run the exact documented commands in a fresh dependency install/CI environment and preserve an explicit matrix of required checks and allowed environment-dependent exclusions. Local existing-dependency success is not a clean-install reproducibility test. Follow [Clippy's documented configuration and CI behavior](https://doc.rust-lang.org/clippy/usage.html), retaining the project's supported compiler floor.

## Coverage, strengths, and remaining uncertainty

This was a critical-flows-first review of the current application, including uncommitted additions. It was not an exhaustive line-by-line review of all 1,147 captured files.

| Area | Direct review and executed evidence | Remaining boundary |
| --- | --- | --- |
| Recording lifecycle | Frontend start/stop service and hooks, coordinator, native start/Stop/status, manager ownership, pipeline shutdown; CQ02/CQ03 probes | Actual device disconnect/tray/permissions and failure combinations; platform behavior |
| Audio/resources | PCM budget implementation, durable source spool publication, incremental encoding/finalization/recovery and deletion leases; actual FFmpeg probe | Sustained resource growth, full real-time callback timing, disk-full/power-loss behavior |
| Notes/persistence | Revisioned note saves, exact-commit replay queue, timestamp insertion/lookup and recording append; database startup/migrations and CQ01 probe | Full editor crash/upgrade matrix and cross-window interactions |
| Recovery/frontend | IndexedDB transaction acknowledgments, coalesced write queue, Stop ownership tests and recovery-related suite | Native/WebView crash windows and physical-session reconciliation |
| Deletion/export | Tombstones, writer leases, managed-root/metadata checks, cleanup ordering and atomic export; existing native tests executed | Windows locked files, installed-package inventories, exhaustive path/race analysis |
| Enhancement/library | Persistence/version safeguards covered by executed native/frontend suites; source-aware behavior reviewed selectively | Independent semantic/code audit of all enhancement modules, E08 failed acceptance, library-scale ranking/latency |
| Security/privacy | Native config and active logging paths; editor dependency resolution, advisory scan, smoke-harness guards | Exploit reproduction, all command/provider/import/updater paths, full Rust advisory scan, actual packaged network/retention trace |
| Delivery/maintainability | Package/scripts, strict TypeScript configuration, eight workflows, formatting and Clippy | Clean dependency install, all GPU feature combinations, native release builds and signing, remote branch protection |

Concrete strengths to retain during remediation:

- Native note writes use transactions, revisions, and idempotent commit acknowledgments. Existing tests exercise concurrent writers, stale revisions, lost acknowledgments, corrupt inputs, and WAL-aware migration backups.
- Capture and ASR handoffs use byte reservations and explicit saturation failure; the source inbox publishes synced private PCM/sidecars and uses a disk cursor rather than retaining an unlimited descriptor queue. This is a useful design property, not long-run resource acceptance.
- Deletion combines durable intent/tombstones, canonical path and identity checks, artifact writer coordination, and cache acknowledgment. Existing tests exercise late writers, protected originals/exports, and other-meeting preservation.
- Export writes through a same-directory temporary file and syncs before publishing. Smoke/validation entry points check isolated identities and opt-ins; Q02 also requires its nondefault feature.
- The frontend has focused interaction/lifecycle tests, strict TypeScript, stable transcript identifiers, and serialized/coalesced persistence queues.

Two maintainability concerns deserve follow-up rather than automatic defect counts. First, recording ownership is spread across global atomics, manager/task/listener slots, current-state/identity storage, SQLite session status, and frontend state. CQ02 shows the practical cost: terminal cleanup can update only part of that state. Prefer one owner for the Stop tail and explicit terminal results, introduced through narrow changes. Second, shared mutable registries, `any`-typed native event consumers, and global window/event hooks make contract drift hard to detect. Add typed boundary adapters and shared event types where a traced flow needs them. Avoid repository-wide renaming, forced file splitting, state-framework replacement, or cosmetic cleanup mixed into behavior fixes.

The local-repo-research scout passed health checking but timed out after 300 seconds without an evidence packet. Its run is incomplete and supports no conclusion; all report findings were verified directly against source or probes.

## Verification receipts

Full commands, elapsed times, source snapshot identity, counts, limitations, and preservation results are in [checks.json](evidence/code-quality-audit-20261015/checks.json). Logs and probe-only patch are indexed in [the evidence directory](evidence/code-quality-audit-20261015/README.md).

| Check | Result |
| --- | --- |
| `bun test frontend/tests` | **Passed:** 183 tests across 41 files, 695 assertions |
| `pnpm --dir frontend build` | **Passed:** production static frontend build |
| `pnpm --dir frontend exec tsc --noEmit --incremental false` | **Passed**, after build sequentially |
| `pnpm --dir frontend lint` | **Failed/unavailable lint baseline:** interactive setup; no lint pass |
| `cargo test --locked --offline -p meetily --lib` | **Failed baseline:** 371 passed, 1 known failed, 8 ignored |
| `cargo fmt --all -- --check` | **Failed:** differences in 134 files |
| `cargo clippy --locked --offline -p meetily --lib --tests -- -D warnings` | **Failed** in build script before full application analysis |
| Diagnostic Clippy with `--message-format=json`, without `-D warnings` | **Failed:** application/test diagnostics collected; 195 unique warnings, 4 unique errors |
| `python3 -m unittest discover -s scripts/tests` | **Passed:** 12 tests |
| `node .github/verify-portable-ggml.test.cjs` | **Passed:** 14 tests |
| `node frontend/tests/lib/onboarding-summary-model.test.mjs` | **Passed:** standalone assertion script |
| `pnpm audit --prod --json` | **Completed, exit 1:** 52 advisory records requiring applicability triage |
| `cargo audit --version` | **Blocked:** command not installed; advisory scan not run |
| Four disposable native defect probes | **Observed all four defects:** probe assertions passed; this is not an application-fix pass |
| Local evidence scout | **Incomplete:** 300-second timeout; no evidence used |

Executed host tools: Node 26.7.0, pnpm 9.15.9, Bun 1.3.14, Python 3.14.7, Cargo/Rust 1.99.0, Clippy 0.1.99, rustfmt 1.10.0. Builds/tests used existing dependencies, locked/offline Cargo resolution, `CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE`, the existing macOS CLT wrapper, and an independent APFS clone of native build artifacts. No supported minimum toolchain change is established by this host run.

The initial unmodified snapshot was checked before disposable probes were added. Probe assertions deliberately describe the faulty current behavior. The patch exists only as an evidence artifact and was not applied to the working tree.

## Proposed remediation batches and CI gates

This ordering is a proposal for a later implementation assignment. No workflow, dependency, API, schema, or application code was changed by this audit.

1. **Protect acknowledged work and shutdown:** CQ01–CQ03, each with a focused regression before its fix. Preserve marker/session ownership, source retention, and exactly one Stop tail. Add all three interaction/failure cases to the ordinary test suite.
2. **Address editor exposure and content logging:** CQ04 and CQ06 in separately reviewable dependency/security and logging changes. Test editor compatibility and use synthetic secrets/content. Triage the remaining npm advisories and obtain a complete Rust advisory scan with a deliberately installed/pinned tool in that later task.
3. **Repair audio path handling:** CQ05 with one shared concat-path implementation and real-FFmpeg path tests. Include recovery as well as normal finalization.
4. **Establish the automated baseline:** CQ07/CQ08; resolve the classifier contract, deterministic lint, actionable Clippy errors, and explicit hardware exclusions. Keep broad formatting in a dedicated batch. Record any justified remaining warning with its rule, location, rationale, and removal condition.
5. **Complete acceptance and structural follow-up:** Revisit the narrow lifecycle ownership/typed-boundary concerns after the defect fixes; then run affected Q02 checks. Keep duration, physical fidelity, Windows/minimum-platform, E08, library-scale, Q03 and Q04 gates explicitly open until their own evidence passes.

| Proposed gate | Required behavior |
| --- | --- |
| Pull-request frontend/tooling | Locked dependency install; noninteractive lint; Bun suite; build then TypeScript sequentially; Python and Node tooling checks |
| Pull-request Rust | Formatting; reviewed Clippy rules; unit/database/session/error-path tests on macOS and Windows with documented fixtures. Classify external-device/model cases by exact test identity and reason; do not skip the existing classifier failure as an environment exclusion. |
| Dependency review | npm and Rust advisory checks with versioned scan receipts; every applicable high/critical finding blocks release until fixed or explicitly risk-accepted with rationale. Unreachable server paths need recorded applicability evidence rather than blind suppression. |
| Package/release | Existing platform build/signing behavior plus isolated workflow, upgrade/recovery, export/deletion and production harness-rejection checks; no publishing in the audit/fix phase |
| Separate native acceptance | Real sources/devices, long workloads, foreground editor/save metrics, model semantics, Windows/platform floor and Q03 egress/retention; mocked/unit results cannot close these gates |

Completion of this audit means the findings, evidence, coverage limits, and repair order are reviewable. It does not mark the application or existing MVP plan fully accepted.
