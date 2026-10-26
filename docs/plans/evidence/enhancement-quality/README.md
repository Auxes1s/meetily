# E08 evidence package

Synthetic evaluation only. No private meeting text or recording is included.
All semantic verdicts are Codex review; all human acceptance is pending.
See [the evaluation report](../../enhancement-quality-evaluation.md) for model
identity, unchanged thresholds, per-run results, commands and limitations.

- `20261014-before-fixes/`: 48 generation jobs, frozen production code/digests,
  complete raw/repair/final evidence, installed-file protection, Codex adjudications,
  metrics and a source/output review sheet.
- `20261014-corrected/`: the same 48-job matrix after the three focused fixes.
  Both model configurations fail the Codex AC07 gate and remain unsuitable.
- `20261014-baseline/`: retained initial sandbox-profile startup error; no model
  generation jobs, excluded from quality metrics.
- `20261014-inventory-smoke/`: non-loopback endpoint refusal and installed built-in
  inventory only; no generation and no external request.
- `checks/`: intake/after status, preservation audit, Bun/Node/Rust/build/type/lint/
  Python logs, expected failing Codex gate, sandbox probe and final runner smoke.
- `SHA256SUMS`: checksums of every package file except the checksum manifest itself.
  Paths are relative to this directory. Verify with `shasum -a 256 -c SHA256SUMS`.

The producer's successful process exit means evaluation and preservation completed.
Individual generation failures are stored, not promoted to success. A structural
candidate is unapproved, not semantically verified or accepted by a human. Review
sheets contain pending human verdict/initials columns. Preserve original run files
when adding corrected runs or human verdicts; use a new phase/output directory.

Model files and disposable databases are not distributed. The model manifest's
isolated symlink directory existed only for the original run. Reproduce with the
Python producer, which creates a fresh directory and inventories installed models.
