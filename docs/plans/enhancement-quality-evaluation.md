# E08 enhancement quality evaluation — 14 October 2026

E08 infrastructure, available-model runs, focused prerequisite fixes and evidence
handoff are delivered. **Both evaluated configurations are unsuitable for
source-grounded enhancement under AC07.** Human acceptance remains pending.
No threshold was lowered and no model-wide accuracy claim follows from these
synthetic, two-repeat samples. The handoff's original next task was L05; complete
Markdown/JSON meeting export has since been delivered, followed by Q01 workflow
automation and the UI design/interaction work. As of 15 October 2026, Q02 native
fidelity/continuity/duration acceptance is the active next assignment. E08 semantic
remediation has not begun and AC07 remains a separate release blocker. See the
[current project plan](meeting-experience-upgrade.md#current-project-status--15-october-2026).

## Scope and protection

Read the repository CLAUDE.md, the plan's E03–E08/AC07/AC14/Sections 11.2–11.3,
the P0 handoff and the 14 October transcript-persistence repair. No applicable
AGENTS.md was found in the repository or its parent chain. Intake status and
612 tracked/untracked file hashes were saved before edits in
`/tmp/meetily-e08-intake/`; durable copies are in [checks](evidence/enhancement-quality/checks/).
Existing unrelated work was preserved. No package installation, application
replacement, model download, recording, library insertion or settings update was
performed. Installed settings were read through SQLite `mode=ro` only to inventory
provider/model/endpoint; credentials and private meeting text were not queried or
loaded as model input. Canonical files were read for hash comparisons only.

Every run creates a disposable migrated SQLx database, saves original blocks using
the production notes repository, inserts labeled committed passages, reads the
production `SourceSnapshot`, and seeds a previously approved, user-edited synthetic
version. It calls the production generation/provider/repair/validation functions
and persists the candidate or failure through the production enhancement repository.
Original JSON/revision and approved pointer/JSON/revision are compared before/after.
All 96 jobs preserved them. Databases are removed after their complete JSON evidence
is saved. Existing model files are symlinked into an isolated model root and read
for inference and hash comparisons;
no fixture enters the installed library. Both model matrices' `protection.json`
confirm identical installed database/WAL, summary-model and helper file hashes.

This is native Rust inference/persistence evidence on macOS, not packaged Tauri
command/WebView acceptance. The harness reuses production provider settings and
Ollama metadata resolution against its disposable settings table; it does not launch
`start_enhancement` through the installed application. Saved language is explicitly
English for these fixtures, including translated Taglish/Japanese meaning and exact
Unicode source excerpts. ASR is not run or evaluated here.

## Fixtures and deterministic coverage

[The 12 fixture files](../../frontend/src-tauri/tests/fixtures/meeting_enhancement/)
use stable meeting/note/transcript identifiers. Labels include relevant decisions/
actions, proposed/agreed/rejected/conflicting status, nullable literal owner/date,
uncertainty, supporting passages, forbidden inferences and user priorities. Labels
are withheld from prompts. There are 19 decision/action labels and 25 priority
labels per matrix repetition: denominators are **38 and 50 per model per phase**.

| Fixture | Category | Template | Decision/action labels | Priority labels |
| --- | --- | --- | ---: | ---: |
| fx-01 | general-meeting | standard_meeting | 2 | 2 |
| fx-02 | highlights | highlights | 2 | 3 |
| fx-03 | technical-advice-versus-acceptance | technical_consultation | 2 | 2 |
| fx-04 | board-proposal-versus-resolution | project_board | 2 | 2 |
| fx-05 | explicit-named-owner-deadline | standard_meeting | 1 | 2 |
| fx-06 | office-assignment | project_board | 1 | 2 |
| fx-07 | missing-owner-deadline | standard_meeting | 1 | 2 |
| fx-08 | tentative-relative-date | standard_meeting | 1 | 2 |
| fx-09 | rejected-decision | project_board | 2 | 2 |
| fx-10 | conflicting-notes-transcript | standard_meeting | 1 | 1 |
| fx-11 | long-input-cross-chunk | standard_meeting | 2 | 2 |
| fx-12 | mixed-language-unicode-uncertain-asr | highlights | 2 | 3 |

Deterministic checks exercise exact frozen source coverage at multiple budgets,
Unicode splitting, stale note revisions, source-fingerprint changes, wrong-meeting/
nonexistent references, altered excerpts, missing required fields, missing-source
attributes, preserved uncertainty, bounded repair failure, cancellation and
preservation of originals/approved versions. A production HTTP-path mock verifies
all four presets fit the 4K planning profile while retaining each section title
and instruction. Existing full-suite tests also cover version/edit/approval races,
legacy adoption, provider independence, cache inputs, redirects/cancellation,
language handling and the 14 October saver-to-SQLite/source-reader regression.
Ordinary tests require no installed model or running inference server.

## Verified models and settings

Host: macOS 27.0.1 (26A434), Apple M5/arm64, integrated Apple GPU, 24 GiB RAM.
Bun 1.3.14, Node 26.7.0, pnpm 9.15.9, Rust/Cargo 1.99.0. Exact environment,
model API metadata, template content, source snapshots and code SHA-256 values
are embedded in each matrix's inventory/manifest and each run. Matching production
source snapshots are retained in each `production-code/` directory.

| Provider/model | Verified identity and configuration |
| --- | --- |
| Built-in `qwen3.5:4b` | Installed selection independently read as `builtin-ai / qwen3.5:4b`. `Qwen3.5-4B-Q4_K_M.gguf`, 2,740,937,888 bytes, SHA-256 `00fe7986ff5f6b463e62455821146049db6f9313603938a70800d1fb69ef11a4`. Installed helper SHA-256 `81b75fdcc8e0de3bdde245ba77a4bafda40d7b1e72818f3b92453112f1333041`. Qwen 3.5 nonthinking chat template; context 32,768 tokens; output cap 4,096; temperature .5, top-k 20, top-p .8, presence .3, frequency 0, repeat 1.05, penalty window 256, stop `<\|im_end\|>`. Helper chooses a time-derived sampling seed and logs 33 GPU layers/full Metal offload. |
| Loopback Ollama `qwen3.5:9b-mlx` | Default configured loopback endpoint verified at `127.0.0.1:11434`; server 0.40.3. Model digest `203e30078279db51132b9e026ceb7bb21330e5b1af67ef190671b375c9770404`, MLX/safetensors/nvfp4, metadata parameter count 9,409,468,800. Stored sampling defaults: temperature 1, top-k 20, top-p .95, min-p 0, presence 1.5, repeat 1. Request output cap 1,024; `reasoning_effort=none`; temperature/top-p are not overridden. Advertised context 262,144; production enhancement planner caps context at 4,096 and conservatively budgets UTF-8 bytes. OpenAI-compatible requests do not send `num_ctx`: post-run `/api/ps` reports actual server context 262,144 and 16,674,769,920 bytes VRAM. The 4K planner limit is not a server memory cap. Server sampling seed is not specified. |

Built-in Qwen 3.5 2B, Gemma 3 4B and Gemma 3 1B are absent and recorded **not run**;
no files were fetched. Ollama exposed only the named installed MLX model. No cloud
provider was evaluated or used as fallback. Model identity is verified by file or
API digest, rather than inferred from a model name alone.

The macOS sandbox denies non-loopback outbound connections for the evaluator and
its children. [The probe](evidence/enhancement-quality/checks/network-probe.json)
records EPERM for a non-loopback documentation address and a successful loopback
connection. Production clients refuse redirects/proxies. Inventory calls send
only model metadata queries to loopback. The existing Ollama server runs outside
this child-process sandbox; its full egress and the packaged application's updater/
telemetry flows still require Q03 inspection. This is not full AC11 acceptance.

## Demonstrated fixes and retained failures

1. Missing fields were silently accepted through Serde defaults. `parse_claims`
   now requires kind/text/section/status/sources/owner/deadline, including explicit
   nullable fields. Unsupported statuses fail validation; legacy `confirmed` is
   conservatively downgraded and never considered semantic proof.
2. Citation validation replaced explicit `uncertain` with `review_required`.
   Explicit uncertainty/missing-source status now survives a valid citation;
   conflict and notes-only handling remain conservative.
3. The full template's description/display/Markdown format metadata exhausted
   the small planning context. Before correction, 12 Ollama jobs failed before
   inference with insufficient source context. The prompt now sends every section
   title/instruction without display metadata. Output stays under the same JSON
   contract, context and output caps; sources are never silently truncated.

The two meaningful baseline contract failures are retained in
[baseline-summary-tests-after-harness.log](evidence/enhancement-quality/checks/baseline-summary-tests-after-harness.log):
120 passed, two failed, one opt-in model test ignored. The corrected summary filter
passed 124 tests, with the one real-model test ignored in that ordinary invocation.
Initial harness UUID/sandbox-profile setup failures were corrected before the
matrices; the startup evidence directory `20261014-baseline` contains no model runs
and is excluded from quality counts. The first harness-test log is retained too.

These fixes enforce structure/context and truthful labels. They do not establish
semantic support. Models still invent meeting commitments from note priorities,
turn non-approval into rejection, add unsupported scope to safety-review approval,
accept an unnamed offer of help as a contribution, invent conflicts/naming topics,
and fail to reconcile an early proposal with later approval. Literal owner/date
checks cannot prove assignment, acceptance, or certainty. No keyword heuristic was
added to pretend that it can.

## AC07 assessment and results

Semantic judgments are **Codex review**, not human acceptance. Every recoverable
raw, repaired and delivered claim is individually adjudicated in `codex-review.json`.
Malformed responses remain visible and readable claims are recovered for review
only; that recovery never changes production acceptance. The compact human packet
shows delivered claims and rejected attempts with actual passages, Codex findings
and pending human verdict/initials columns.

Recall uses explicit mappings from a label to delivered claim indices. Content can
appear in a summary or across multiple claims; it must retain the relevant meaning
and qualifications. Literal owners/dates in prose count as retained even where the
structured nullable field remains null; this limitation is visible in the packet.
Topic retention measures output content, not mere inclusion in a source excerpt.
Additional unsupported assertions are counted separately, even when another claim
correctly recalls the label. Empty/rejected/failed generations earn **zero** delivered
recall and retention. Repeats are separate observations; omissions are not hidden by
a favorable run. Counts below are claim occurrences, including duplicate assertions
in different claims, not unique real-world decisions.

The gate retains zero fabricated confirmed decisions/owners/offices/deadlines,
source support or explicit uncertainty/notes-only/missing-source status for every
consequential claim, zero invalid references accepted, and **≥90%** decision/action
recall and priority retention. Codex additionally flags unsupported cited assertions
rather than allowing a generic review disclaimer to conceal them. Raw confirmed
fabrications are reported and also prevent a Codex gate pass, even after repair.
Human acceptance is pending regardless of the Codex gate.

| Phase / model | Jobs | Structural candidates | Rejected | Empty candidates | Decision/action recall | Priority retention | Unsupported confirmed-decision claims in candidates | Other unsupported/citation claims in candidates | Invalid references accepted | Result |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| before-fixes / builtin-ai `qwen3.5:4b` | 24 | 22 | 2 | 0 | 34/38 (89.5%) | 44/50 (88.0%) | 3 | 4 | 0 | Unsuitable; human pending |
| before-fixes / ollama `qwen3.5:9b-mlx` | 24 | 10 | 14 | 0 | 15/38 (39.5%) | 20/50 (40.0%) | 0 | 7 | 0 | Unsuitable; human pending |
| corrected / builtin-ai `qwen3.5:4b` | 24 | 19 | 5 | 0 | 28/38 (73.7%) | 37/50 (74.0%) | 5 | 4 | 0 | Unsuitable; human pending |
| corrected / ollama `qwen3.5:9b-mlx` | 24 | 16 | 8 | 2 | 20/38 (52.6%) | 27/50 (54.0%) | 1 | 5 | 0 | Unsuitable; human pending |

No fabricated named owners/offices or deadlines were found in either phase's raw,
repaired or delivered outputs. This is an observed result in these fixtures, not
proof that literal matching prevents those errors generally. Corrected raw/repair
outputs contained six built-in and two Ollama unsupported confirmed-decision claim
occurrences; corrected candidate counts are five and one. Corrected raw reference
errors were rejected/repaired: 18 built-in citation occurrences plus one manually
identified invalid ID in a malformed Ollama response; zero were accepted as valid.
Semantic citation errors remain in accepted candidates despite this structural zero.

Before/corrected generation times totalled 340.332/381.104 seconds for built-in and
326.352/660.455 seconds for Ollama. Provider-call counts were 33/40 and 37/56;
repair-call counts were 5/12 and 5/19. Per-call and per-job times/errors are in JSON.
Timing includes generation and repair, excluding fixture DB seed/config metadata;
these are not latency/throughput acceptance measurements. Stochastic samples and
changed prompts do not estimate the causal effect of the fixes on model accuracy.

Built-in processed all three long-input chunks in each phase, but left an early
proposal as current alongside final approval in several outputs. Corrected Ollama
planned 98 chunks for the long fixture and failed the first chunk's bounded repair
in both repeats. It never produced a complete candidate; later chunks are not
reported as processed. Two corrected Ollama jobs returned structurally valid empty
claim arrays (fx-07-r2, fx-08-r2), and the empty extractions remain in the metrics.

## Per-run results and review handoff

Each run JSON preserves its complete fixture/labels, source snapshot, actual
provider/template settings, raw wire/content responses, repaired responses,
validation errors/accepted claims, persisted candidate/failure, timings and document
protection checks. `structurally_accepted_candidate` means an unapproved candidate;
it does not mean source fidelity or human acceptance. The ignored Rust producer's
exit 0 means the evaluation jobs/evidence/preservation checks completed, not that
AC07 passed. The separately executed Codex gate command exits **1** for the
corrected matrix, as required by the failed quality thresholds.

- Before fixes: [per-run metrics](evidence/enhancement-quality/20261014-before-fixes/metrics.json),
  [Codex adjudications](evidence/enhancement-quality/20261014-before-fixes/codex-review.json),
  [human review sheet](evidence/enhancement-quality/20261014-before-fixes/source-output-review.md).
- Corrected: [per-run metrics](evidence/enhancement-quality/20261014-corrected/metrics.json),
  [Codex adjudications](evidence/enhancement-quality/20261014-corrected/codex-review.json),
  [human review sheet](evidence/enhancement-quality/20261014-corrected/source-output-review.md).

The following table records every run, including failed generations and empty
outputs. `C` = structural candidate; `E` = empty structural candidate; `R` = rejected.
Within each cell: outcome, decision/action labels recalled, priority labels retained,
and generation milliseconds. Human acceptance is pending for every cell.

| Phase / fixture | Built-in r1 | Built-in r2 | Ollama r1 | Ollama r2 |
| --- | --- | --- | --- | --- |
| before-fixes / fx-01 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-01-r1.json): 2/2, 1/2; 15867ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-01-r2.json): 2/2, 2/2; 16903ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-01-r1.json): 0/2, 0/2; 0ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-01-r2.json): 0/2, 0/2; 0ms |
| before-fixes / fx-02 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-02-r1.json): 2/2, 3/3; 11113ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-02-r2.json): 2/2, 3/3; 18815ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-02-r1.json): 1/2, 3/3; 45090ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-02-r2.json): 1/2, 3/3; 47309ms |
| before-fixes / fx-03 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-03-r1.json): 2/2, 2/2; 8200ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-03-r2.json): 2/2, 2/2; 13120ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-03-r1.json): 0/2, 0/2; 32026ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-03-r2.json): 0/2, 0/2; 25203ms |
| before-fixes / fx-04 | [R](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-04-r1.json): 0/2, 0/2; 24344ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-04-r2.json): 2/2, 2/2; 7786ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-04-r1.json): 2/2, 2/2; 34345ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-04-r2.json): 2/2, 2/2; 17063ms |
| before-fixes / fx-05 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-05-r1.json): 1/1, 2/2; 8342ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-05-r2.json): 1/1, 2/2; 6573ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-05-r1.json): 0/1, 0/2; 0ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-05-r2.json): 0/1, 0/2; 0ms |
| before-fixes / fx-06 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-06-r1.json): 1/1, 2/2; 6844ms | [R](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-06-r2.json): 0/1, 0/2; 10905ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-06-r1.json): 1/1, 2/2; 13157ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-06-r2.json): 1/1, 2/2; 13831ms |
| before-fixes / fx-07 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-07-r1.json): 1/1, 2/2; 13998ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-07-r2.json): 1/1, 2/2; 11352ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-07-r1.json): 0/1, 0/2; 0ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-07-r2.json): 0/1, 0/2; 0ms |
| before-fixes / fx-08 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-08-r1.json): 1/1, 2/2; 4681ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-08-r2.json): 1/1, 2/2; 22106ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-08-r1.json): 0/1, 0/2; 0ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-08-r2.json): 0/1, 0/2; 0ms |
| before-fixes / fx-09 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-09-r1.json): 2/2, 2/2; 7261ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-09-r2.json): 2/2, 2/2; 7231ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-09-r1.json): 2/2, 1/2; 19912ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-09-r2.json): 2/2, 2/2; 25281ms |
| before-fixes / fx-10 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-10-r1.json): 1/1, 1/1; 7650ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-10-r2.json): 1/1, 1/1; 4479ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-10-r1.json): 0/1, 0/1; 0ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-10-r2.json): 0/1, 0/1; 0ms |
| before-fixes / fx-11 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-11-r1.json): 2/2, 2/2; 37681ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-11-r2.json): 2/2, 2/2; 34349ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-11-r1.json): 0/2, 0/2; 0ms | [R](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-11-r2.json): 0/2, 0/2; 0ms |
| before-fixes / fx-12 | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-12-r1.json): 1/2, 2/3; 27395ms | [C](evidence/enhancement-quality/20261014-before-fixes/builtin-ai-qwen3.5_4b-fx-12-r2.json): 2/2, 3/3; 13337ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-12-r1.json): 1/2, 1/3; 30419ms | [C](evidence/enhancement-quality/20261014-before-fixes/ollama-qwen3.5_9b-mlx-fx-12-r2.json): 2/2, 2/3; 22716ms |
| corrected / fx-01 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-01-r1.json): 2/2, 2/2; 28912ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-01-r2.json): 2/2, 2/2; 22104ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-01-r1.json): 1/2, 2/2; 19170ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-01-r2.json): 2/2, 1/2; 21356ms |
| corrected / fx-02 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-02-r1.json): 2/2, 3/3; 16454ms | [R](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-02-r2.json): 0/2, 0/3; 19544ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-02-r1.json): 0/2, 0/3; 18904ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-02-r2.json): 1/2, 3/3; 37154ms |
| corrected / fx-03 | [R](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-03-r1.json): 0/2, 0/2; 19343ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-03-r2.json): 2/2, 2/2; 26063ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-03-r1.json): 0/2, 0/2; 25276ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-03-r2.json): 0/2, 0/2; 23599ms |
| corrected / fx-04 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-04-r1.json): 0/2, 2/2; 11146ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-04-r2.json): 2/2, 2/2; 14824ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-04-r1.json): 2/2, 2/2; 20330ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-04-r2.json): 2/2, 2/2; 27031ms |
| corrected / fx-05 | [R](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-05-r1.json): 0/1, 0/2; 11634ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-05-r2.json): 1/1, 2/2; 5333ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-05-r1.json): 1/1, 2/2; 35571ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-05-r2.json): 1/1, 2/2; 14791ms |
| corrected / fx-06 | [R](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-06-r1.json): 0/1, 0/2; 10829ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-06-r2.json): 1/1, 2/2; 10649ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-06-r1.json): 1/1, 2/2; 22194ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-06-r2.json): 1/1, 2/2; 19736ms |
| corrected / fx-07 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-07-r1.json): 1/1, 2/2; 14357ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-07-r2.json): 1/1, 2/2; 9839ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-07-r1.json): 1/1, 2/2; 52259ms | [E](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-07-r2.json): 0/1, 0/2; 641ms |
| corrected / fx-08 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-08-r1.json): 1/1, 2/2; 10857ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-08-r2.json): 1/1, 2/2; 8851ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-08-r1.json): 0/1, 0/2; 37409ms | [E](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-08-r2.json): 0/1, 0/2; 625ms |
| corrected / fx-09 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-09-r1.json): 2/2, 2/2; 6821ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-09-r2.json): 2/2, 2/2; 7031ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-09-r1.json): 2/2, 2/2; 58407ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-09-r2.json): 2/2, 2/2; 19054ms |
| corrected / fx-10 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-10-r1.json): 1/1, 1/1; 4181ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-10-r2.json): 1/1, 1/1; 7187ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-10-r1.json): 0/1, 0/1; 33108ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-10-r2.json): 0/1, 0/1; 17005ms |
| corrected / fx-11 | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-11-r1.json): 2/2, 2/2; 43129ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-11-r2.json): 2/2, 2/2; 29960ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-11-r1.json): 0/2, 0/2; 53856ms | [R](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-11-r2.json): 0/2, 0/2; 21990ms |
| corrected / fx-12 | [R](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-12-r1.json): 0/2, 0/3; 24870ms | [C](evidence/enhancement-quality/20261014-corrected/builtin-ai-qwen3.5_4b-fx-12-r2.json): 2/2, 2/3; 17186ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-12-r1.json): 1/2, 1/3; 41046ms | [C](evidence/enhancement-quality/20261014-corrected/ollama-qwen3.5_9b-mlx-fx-12-r2.json): 2/2, 2/3; 39943ms |

Human reviewer: inspect each consequential claim against the actual retained
passages and qualifiers, review proposed/agreed status and literal assignments,
then confirm/correct the Codex judgments and label mappings in the sheet. Record
reviewer/date and signed acceptance or rejection before changing AC07 status.
No reviewer/date/signature has been invented. These failing configurations cannot
be accepted by a reference-validity check alone.

## Repeatable commands

Use a **new** output directory; the producer refuses overwrite. Installed files
are only read. No dependency/model download is part of these commands.

```sh
# Root: independent checks (cached native prerequisites on this Mac).
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  cargo test --locked --offline -p meetily --lib summary::
PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/test-enhancement-evaluation.py

# Opt-in real models: all 12 sources twice for every available local model.
python3 scripts/evaluate-meeting-enhancement.py \
  --output /tmp/meetily-e08-new-run --phase evaluation --repeats 2

# Optional subsets/affected-fixture reruns; keep prior directories.
python3 scripts/evaluate-meeting-enhancement.py --provider builtin-ai \
  --fixture fx-11 --output /tmp/meetily-e08-affected-run --phase affected-rerun
# --provider ollama, --app-data PATH, --helper PATH, --ollama-endpoint LOOPBACK_URL
# --inventory-only inventories availability without generation.

# First produce a pending human packet. Then adjudicate all row IDs and map
# labels explicitly in codex-review.json; do not use keyword hits as semantics.
python3 scripts/review-meeting-enhancement.py /tmp/meetily-e08-new-run
python3 scripts/review-meeting-enhancement.py /tmp/meetily-e08-new-run \
  --reviews /tmp/meetily-e08-new-run/codex-review.json --require-codex-gates

# Reproduce this report's failed Codex gate (expected exit 1).
python3 scripts/review-meeting-enhancement.py \
  docs/plans/evidence/enhancement-quality/20261014-corrected \
  --reviews docs/plans/evidence/enhancement-quality/20261014-corrected/codex-review.json \
  --require-codex-gates

# Full native suite with the known baseline assertion explicitly excluded.
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  cargo test --locked --offline -p meetily --lib -- \
  --skip audio::device_detection::tests::test_builtin_mic_detection
node .github/verify-portable-ggml.test.cjs

cd frontend
bun test tests
node tests/lib/onboarding-summary-model.test.mjs
pnpm build
pnpm exec tsc --noEmit --incremental false
pnpm lint < /dev/null
```

Source paths/default app/helper paths match this macOS host. Other hosts must
supply their installed model/helper paths and cached native build prerequisites.
The script refuses non-loopback Ollama endpoints and still inventories/evaluates
independently available built-in models. The negative inventory smoke exercised
that refusal without contacting `example.invalid`; its model-file hashes remained
unchanged. Network sandbox evidence on other platforms is not claimed.

## Executed checks and remaining gates

| Check | Result |
| --- | --- |
| Final Rust native suite | 320 passed; 8 ignored; 1 explicitly filtered baseline failure. |
| Baseline microphone classifier, separately rerun | Fails unchanged: `MacBook Pro Microphone` returns Wired; test expects Unknown. No E08 audio-detector edit. |
| Focused summary contracts | 124 passed, 1 model test ignored, 204 filtered. |
| Opt-in native model producer | Two complete matrices; 96 jobs; both producer invocations exit 0; quality gates fail. |
| Python producer/review contracts | 8 passed, including loopback/redirect restrictions, rejected/empty credit, mandatory per-claim review, invalid references and invalid-envelope visibility. |
| Frontend Bun | 97 passed, 0 failed, 389 assertions, 22 files. |
| Portable GGML Node | 14 passed. |
| Onboarding Node | Exit 0. |
| Production frontend build, then TypeScript | Both exit 0, run sequentially. |
| Lint | Existing entry point exits 1 at Next's ESLint setup prompt. No configuration/dependency change; not a lint pass. |
| Corrected Codex quality gate | Exit 1; both models fail AC07 under the tested configuration. |
| Protection/network probe | 96 synthetic original/approval comparisons passed; both installed-data/model hash comparisons unchanged; child-process network probe passed. |

Ignored native cases are six hardware/ASR scenarios (Core Audio capture, import
pipeline, echo/Whisper, two installed-provider source scenarios, system detector),
the crash-writer helper (executed by its parent test), and the opt-in E08 producer
(explicitly executed separately). No ignored case is counted as an ordinary-suite
pass. The prior 14 October transcript-persistence repair is included in the passing
native suite; no private transcript was regenerated here.

Pending: required human semantic review/acceptance; an installed local configuration
that passes unchanged AC07; Windows real-model/package execution; packaged
macOS WebView review/edit/approve/regenerate and actual egress inspection; prior
physical-audio, minimum-platform, 45-minute/four-hour, privacy/storage and release
gates from Q02–Q04. Two local synthetic model repeats do not close those gates.
E08's acceptance checklist stays open. This delivery stops at E08 and the three
necessary prerequisite fixes; **L05 complete Markdown/JSON meeting export is next**.

## Changed files

Existing production changes are confined to `summary/enhancement.rs` (opt-in audit
and compact section prompt), `summary/grounding.rs` (required-field/status checks,
uncertainty preservation/tests), and `summary/llm_client.rs` (opt-in full provider
response capture using the same production adapter). Ordinary generation does not
record source payloads. No Cargo/package dependency changed.

Added: `summary/enhancement/evaluation.rs`, the 12 fixture JSON files and README,
`scripts/evaluate-meeting-enhancement.py`, `scripts/review-meeting-enhancement.py`,
`scripts/tests/test-enhancement-evaluation.py`, this report and the synthetic
`evidence/enhancement-quality/` package. The plan/handoff are updated only with
supported E08 evidence and pending gates. The evidence package includes digest
inventories, source-code snapshots, all raw/repaired/validated results, Codex sheets,
checks and a checksum manifest for handoff.
