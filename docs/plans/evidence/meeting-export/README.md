# L05 export evidence — 14 October 2026

This package contains synthetic/disposable local evidence for complete meeting
export. It establishes macOS arm64 debug/native behavior and the deterministic
contracts below. It does not establish Windows native acceptance, real-model
enhancement quality or packaged release acceptance.

| Check | Result |
| --- | --- |
| Bun suite, including export orchestration/choices | 108 passed, 0 failed |
| Full native suite excluding documented baseline microphone classifier | 331 passed, 8 ignored, 1 excluded |
| Final focused native export contracts | 11 passed |
| Production Next.js static build, then TypeScript check | Both exit 0 |
| Native debug app build for `com.meetily.export.test` | Exit 0; no bundle/installation |
| Actual native Save cancellation and JSON/Markdown save | Passed, in isolated app |
| Parsed JSON/raw-note comparison to isolated SQLite | Exact original JSON and revision match |
| Native JSON and Markdown full transcript | All 1,207 passages, through sequence 1206 |
| Library export dialog and visual inspection | Passed |
| Lint | Existing ESLint configuration prompt, exit 1 |
| Unrelated pre-existing working-tree files | Preserved; see `preservation.json` |

`native-record.json` and `native-notes-with-transcript.md` are files saved by the
actual native Save dialog, renamed only for this evidence directory. Their
original filenames were `Native export smoke — 日本語.json` and
`Native export smoke — 日本語.md`. `native-webview-result.json` records their
SHA-256 hashes and comparisons. Both contain a synthetic meeting; no production
meeting was exported. The selected original-note document contains the edit made
immediately before export. Captured start time stays unknown and creation time
is labeled separately. Source badges round-trip as microphone/system/mixed/unknown
and readable labels; the Markdown transcript includes every passage.

`native-save-dialog.png` shows the actual macOS Save panel. Its first invocation
was cancelled with Escape; the driver verified that no output existed before the
second invocation. JSON and Markdown were then saved through native dialogs.
`native-library-export.png` shows the shared dialog opened from the library.
Screenshots and the beginning/end of the Markdown file were inspected locally.
The opt-in WebView script is `frontend/src-tauri/smoke/export.js`; the retained
`drive-native-save.py` controls only the test process and validates saved files.

The eleven native tests use disposable SQLx fixtures to cover approved/candidate/
archived/legacy states and history, actions/review labels, malformed and
double-encoded raw JSON, frozen excerpts after source edits/deletion, read-only
WAL snapshot consistency across transcript pages, incomplete states, projection
failure with lossless JSON, approval changes, overlapping timestamp/session
ownership, filename limits, protected destinations and atomic failure cleanup.
These deterministic enhanced-document cases are distinct from the native UI
smoke, which does not run an inference model.

## Commands

From the repository root:

```sh
cd frontend
bun test tests
pnpm build
pnpm exec tsc --noEmit --incremental false
CI=1 pnpm lint
cd ..
PATH="$PWD/scripts/macos-clt:$PATH" CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE \
  cargo test --locked --offline -p meetily --lib -- \
  --skip audio::device_detection::tests::test_builtin_mic_detection
PATH="$PWD/scripts/macos-clt:$PATH" CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE \
  cargo test --locked --offline -p meetily --lib database::export::tests
```

The excluded baseline classifier is rerun separately in `baseline-mic-test.log`;
its unchanged assertion failure is not an export regression. Ignored native
hardware/model tests and Windows file-lock/Save-dialog scenarios remain open.

To repeat the isolated WebView run, copy `native-smoke-config.json` and
`drive-native-save.py` to `/tmp/meetily-l05-checks/`. Preserve or move previous
synthetic export files before repeating: the driver deliberately requires their
absence to check cancellation. Build with the copied configuration after the
frontend production build:

```sh
cd frontend
PATH="$PWD/../scripts/macos-clt:$PATH" CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE \
  pnpm exec tauri build --debug --no-bundle \
  --config /tmp/meetily-l05-checks/native-smoke-config.json -- --locked --offline
cd ..
python3 /tmp/meetily-l05-checks/drive-native-save.py
# In a second terminal, while the driver is waiting:
MEETILY_EXPORT_SMOKE=1 ./target/debug/meetily
```

This uses `com.meetily.export.test`, a separate library, synthetic data and an
explicit debug-only opt-in. Native automation needs existing macOS accessibility
and screenshot access. It does not download a model, install a bundle, or replace
the installed application. The test process was stopped after this run.

`baseline-sha256.json` and `git-status-before.txt` capture the incoming dirty
workspace. `preservation.json` compares all 320 pre-existing changed/untracked
files and lists the bounded existing-file edits; `source-sha256.json` identifies
the final implementation/test files. `SHA256SUMS` covers this evidence package.
