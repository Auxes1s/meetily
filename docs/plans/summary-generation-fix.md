# Summary and enhancement repair — 16 October 2026

The operational repair is implemented and staged for macOS. Both UI entrypoints
completed real built-in inference in an isolated native WebView. The installed
application and production library were not replaced or migrated by this task.
**The model still fails AC07 semantic acceptance.** Successful generation creates
an unapproved candidate requiring review.

## Changes

- Both summary and enhancement use the same repaired generator. Built-in requests
  use a strict GBNF claims grammar. Models select short passage IDs; the app
  resolves them to exact frozen source excerpts and validates owner/deadline
  literals. Existing saved documents and the supporting-source inspector retain
  their canonical citation format.
- Unicode-preserving passages contain at most 512 characters. Initial source
  chunks are capped at 8 KiB and the conservative context allowance. The helper
  verifies actual prompt-token capacity before inference. The built-in output
  limit remains 4,096 tokens.
- The helper reports request identity, completion reason and token counts. A
  capability handshake rejects incompatible helpers. Request/response exchanges,
  model selection and cancellation are serialized, including background pings.
  The pinned library's `sample()` already accepts its selected token: removing
  the old second acceptance fixes grammar-state corruption and duplicated
  repetition-penalty history.
- Truncation/context exhaustion splits the affected partition immediately.
  Structural/reference failures receive one targeted repair before subdivision.
  Subdivision stops after two levels, at most 14 provider calls per original
  chunk. Provider errors terminate without automatic provider changes.
- Migration `20261016000000_enhancement_work.sql` adds version-owned validated
  checkpoints and diagnostic metadata. Retry creates a separate candidate and
  reuses work only when frozen sources, complete template, instructions,
  language, provider configuration, built-in sampling/file metadata and format
  version match. Checkpoints survive restart; no inference starts automatically.
  Cancellation/deletion guards reject late writers, and meeting deletion
  cascades to checkpoints. Successful publication removes its redundant work.
- `retry_of_version_id` is optional on both generation commands. Status responses
  optionally include `generation_progress` with completed/total chunks, failure
  category/part and retry availability. Failed attempts reload correctly, while
  displayed approved documents retain their own save identities.
- Notes-only meetings can use the summary controls, including Stop. Both views
  show progress and actionable errors. Failed regeneration retains the previous
  document and offers retry. No incomplete candidate is published; wholly empty
  model output is a failure.

Only validated derived claims are checkpointed. Runtime diagnostics retain error
categories, counts and timing metadata; rejected responses and private prompts
are not retained. Raw capture remains limited to explicitly enabled synthetic
evaluation. No model download, cloud fallback or provider-setting change is part
of the repair.

## Verification

Final checks are retained in [checks](evidence/summary-generation-fix-20261016/checks/).

| Check | Result and boundary |
| --- | --- |
| Rust application suite | 377 passed, 9 ignored, one known microphone-detection baseline test explicitly excluded. |
| Helper protocol tests | 3 passed. |
| Supported frontend suite | 184 passed. |
| Evaluation/reviewer and workflow isolation Python tests | 10 passed. |
| Production frontend/build and TypeScript | Passed; the matching helper is bundled in the staged application. |
| Native UI | Both Enhanced notes and Generate Summary completed `qwen3.5:4b` inference with synthetic Taglish/Unicode notes; originals preserved. [Report](evidence/summary-generation-fix-20261016/native-validation-release-candidate/native-webview.json). |
| Standard real-model matrix | 24/24 complete structural candidates, 34 provider calls; all original/approved-document comparisons preserved. [Evidence](evidence/summary-generation-fix-20261016/model-evaluation-v2/manifest.json). |
| Scale real-model matrix | 4/4 complete candidates: two notes-only repetitions and two nine-chunk Taglish/Unicode repetitions. All original/approved-document comparisons preserved. [Evidence](evidence/summary-generation-fix-20261016/scale-evaluation/manifest.json). |
| Final captured-response replay | All accepted responses from both matrices revalidated through the final resolver with identical claim content and citations. Replay is not fresh inference. |

Deterministic regressions cover Unicode/source preservation, required field
types, unknown references, bounded repairs, truncation/subdivision, empty output,
checkpoint reuse and invalidation, restart, cancellation, deletion races,
concurrent helper exchanges, incompatible helper response metadata, and
frontend retry/reopening. The synthetic scale fixture establishes input handling
and completion, not recall or fidelity on a real long meeting.

Earlier failed diagnostic runs are retained. The first grammar run exposed the
double token-acceptance bug; later native test-script corrections addressed
startup readiness, route spelling and a test assertion that assumed a nullable
plain-text field. Final native verification uses the packaged helper. An
accidental repository-root Bun run also discovered an unavailable addon in the
archived Python/Whisper example tree; the supported `frontend/` suite passed.

The model matrices preceded final checkpoint-key, diagnostic and UI refinements;
their inference prompt, grammar and sampling behavior were retained. Final
source receipts and captured-response replay distinguish that evidence from the
subsequent native/build checks. No Windows acceptance, physical recording,
four-hour recording, human semantic sign-off or full offline-app audit is claimed.

## Remaining accuracy blocker

The [source-by-source Codex review](evidence/summary-generation-fix-20261016/model-evaluation-v2/source-output-review.md)
finds decision/action recall **32/38 (84.2%)**, priority retention **45/50 (90%)**,
and nine unsupported delivered assertions. Citation identities/excerpts resolve
correctly, but the model still converts non-acceptance to rejection, adds task
scope and fails to reconcile an early proposal with later approval. The existing
AC07 gate exits **1**; no threshold was lowered. Human acceptance remains pending.

## Staged delivery and reproduction

The release bundle is at `target/release/bundle/macos/Meetily Notepad.app`.
The adjacent `Meetily Notepad-summary-fix-20261016.zip` contains that bundle.
The installed `/Applications/meetily.app` was left in place. The staged release
uses the production library identity: quit the existing app before manually
opening it. Existing startup logic backs up the database before migrations.

Build the helper and application with `scripts/meetily-notepad.sh build`.
To rerun the isolated native check, build with
`src-tauri/tauri.summary-test.json`, then run `scripts/run-summary-validation.py`
with `--bundle`, a new `--output` directory, and `--model` pointing to the existing
GGUF. The runner only admits the marked `com.meetily.summary.test` profile.
Model evaluation uses `scripts/evaluate-meeting-enhancement.py --provider builtin-ai`
with the newly built `--helper`, an isolated `--app-data` root, a fresh `--output`,
and optionally `--scenario-matrix scale`. Ordinary tests require no inference.
