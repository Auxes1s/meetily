# Synthetic enhancement fixtures (E08)

These 12 UTF-8 JSON sets are evaluation data. The harness creates disposable
SQLite meetings and never inserts them into an installed library. IDs are stable:
`e08-fx-NN` for meetings, `note:fx-NN-nN` and `transcript:fx-NN-tN` for sources.
No fixture is an actual meeting, participant, recording, or ASR measurement.

Each file contains original note paragraphs, committed transcript passages,
a selected production template/language/instructions, and separate annotations:

- `relevant_decisions_actions`: stable labels, meaning, agreement/proposal/rejection/
  conflict status, literal nullable owner/date, uncertainty, valid passages.
- `user_priorities`: stable topic labels and supporting passages.
- `forbidden_inferences`: fixture-specific assertions that sources do not establish.

Annotations are withheld from model prompts. Sources and instructions are fed
through the production snapshot, chunking, provider, one-repair, validation and
candidate repository path. Audio provenance is Unknown; it cannot identify people.
The long fixture has a deliberately irrelevant 32 KiB passage between a definition
and its later approval/assignment, so the reference spans actual model-sized chunks.

Run the model-independent contracts:

```sh
# Repository root, cached native dependencies on macOS:
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  cargo test --locked --offline -p meetily --lib summary::
python3 scripts/tests/test-enhancement-evaluation.py
```

The opt-in model entry point and assessment rules are documented in
[enhancement-quality-evaluation.md](../../../../../docs/plans/enhancement-quality-evaluation.md).
Model sampling is stochastic; keep every failed run and use a new output directory.
