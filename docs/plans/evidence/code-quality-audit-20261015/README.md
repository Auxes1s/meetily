# Code-quality audit evidence — 15 October 2026

The [audit report](../../code-quality-audit.md) contains the findings and their limits. These receipts concern a captured working tree, including uncommitted files, at base commit `a2cb62e827da7ef59f65064c97233efb2313878e`. The commit alone does not identify the reviewed source: use [the initial file hashes](initial-source-sha256.json).

## Checks and reproduction

- [checks.json](checks.json): exact commands, exit codes, durations, tool versions, snapshot paths, counts and initial preservation check.
- [run_checks.py](run_checks.py): initial grouped check runner used outside the repository. The captured copy retains its absolute disposable-artifact paths; use a new disposable checkout/snapshot to reproduce, with existing dependencies and FFmpeg binaries, and adjust the isolated native target path. Do not point it at production storage.
- [tool-versions.txt](tool-versions.txt): executed host tools.
- Frontend: [tests](frontend-tests.txt), [build](frontend-build.txt), [TypeScript](typecheck.txt), [lint](lint.txt).
- Native: [tests](rust-tests.txt), [strict Clippy](clippy.txt), [diagnostic Clippy status](clippy-diagnostics.txt), [structured diagnostics](clippy-summary.json), [formatting file inventory](rustfmt-files.txt).
- Auxiliary: [Python](python-tests.txt), [portable GGML Node checks](portable-ggml-tests.txt), [standalone onboarding assertions](onboarding-node-tests.txt).
- Dependencies: [npm advisory summary](dependency-advisories-summary.json), [unavailable cargo-audit](cargo-audit-version.txt). Evidence logs use `.txt` filenames so repository log-ignore rules do not omit the receipts. The npm summary omits repeated dependency paths to keep it reviewable. The full registry response and raw formatting/Clippy streams remain in the disposable run directory identified by `checks.json`.

## Disposable defect probes

[disposable-audit-probes.patch](disposable-audit-probes.patch) adds four tests to a copy of the audited source; it contains no application fixes and **was not applied to the working tree**. Each assertion confirms faulty current behavior. [audit-probes-complete.txt](audit-probes-complete.txt) records four passing probe assertions:

1. A session-A note marker rejects a newer save after session B is appended.
2. A failing native Stop SQLite write removes the manager while status reports recording.
3. A pipeline task panic produces `Ok(())` from pipeline shutdown.
4. An apostrophe in a valid recording root prevents actual FFmpeg finalization, with the encoded checkpoint retained.

The Stop probe uses a Tauri mock app, a temporary database and a rejecting SQLite trigger. The encoding probe uses synthetic PCM and actual FFmpeg. None captures a microphone/system device or touches a production library.

To reproduce in a disposable source copy matching the initial hashes, apply only this probe patch there and run:

```sh
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  CARGO_TARGET_DIR=/absolute/path/to/independent-audit-target \
  cargo test --locked --offline -p meetily --lib audit_probe -- \
  --nocapture --test-threads=1
```

Provide the existing sidecar binaries/native prerequisites without downloading or modifying production assets. Baseline tests/Clippy were executed before applying probes in that disposable copy. The initial source snapshot and all original working-tree files are retained; final preservation is recorded in [preservation.json](preservation.json).

## Boundaries

No application fix, dependency installation, production app launch/install, release, duration workload, physical capture, real-model quality evaluation, Windows run, or live egress/retention experiment was performed. The local evidence scout timed out without a packet and supplied no findings. The Rust suite's known classifier failure and ignored tests remain explicit. These receipts cannot substitute for the open Q02/E08/library/Q03/Q04 acceptance gates.
