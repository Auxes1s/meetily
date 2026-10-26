# Responsive interaction hardening

Follow-up to the [UI design upgrade](../../ui-design-upgrade.md), authorized on
2026-10-15. Audio source badges are now icon-only. Hover descriptions and
screen-reader names retain Microphone, System audio, Mixed audio and Unknown;
the transcript metadata column is 64px instead of 112px in the horizontal
layout. Stored provenance and exports retain their existing labels and values.

## Changes

- Recording start, stop, pause and resume share a synchronous admission guard.
  Pending controls stay locked through parent finalization, errors release the
  guard, and late listener registrations are disposed after unmount.
- Document switching waits for saved notes, admits one pending switch, retains
  edits on failure, and ignores an obsolete completion after source navigation.
  Document tabs support arrow, Home and End keys with roving keyboard focus.
- Export choices deduplicate loading and ignore stale responses after close or
  reopen. Original-note exports and timestamp insertions admit one request at
  a time. New drafts expose pending state and suppress held keyboard repeats.
- Sidebar navigation gives the most recent destination ownership after save
  flushing. Notification and recording preferences prevent overlapping writes
  and recover their prior value when saving fails.
- Import selection, validation, submission and cancellation have separate
  guards. Cancellation prevents a late start and stays pending until native
  import shutdown is acknowledged; failed cancellation remains retryable.
- Metadata saves lock their fields and Cancel, ignore obsolete callbacks and
  prevent duplicate submissions. Library deletion requires the current target;
  pending cleanup is guarded per job and obsolete scans cannot restore a job.
- Custom templates snapshot pending saves, ignore obsolete loads, and preserve
  the one-to-32 section bounds under repeated actions.

## Verification

[Frontend results](frontend-tests.log): **166 tests passed, 641 assertions**.
The production frontend build and subsequent standalone TypeScript check
passed. These tests cover delayed requests, repeated same-tick handler calls,
failure/retry, cancellation, stale responses and component disposal. They use
mocked IPC; native behavior is checked separately.

Before-fix regression evidence is retained in
[recording controls](recording-controls-before.log),
[document tabs](document-tabs-before.log),
[export options](export-before.log), and
[original-note actions](note-actions-before.log).

Final [check record](checks.json): the isolated macOS WKWebView passed
**91 layout cases, 34 design cases and all eight interaction scenarios**, with
zero unhandled runtime errors. It retained **46 native screenshots**. Existing
editor identity, undo/redo, pending saves, selected version, transcript context
and scroll checks passed through 122 continuous resize steps. The interaction
storm adds 100 sidebar toggles, 100 drawer toggles and 20 native resizes.

The eight scenarios cover the toggle/resize storm, 240 document-tab clicks,
50 tab-switch attempts during a held and then failed note save, stale export
choices plus 75 export-save clicks, 100 new-draft clicks, 100 metadata submits,
narrow delete-dialog dismissal, and settings click/keyboard/resize recovery.
The note-save failure retained the edit and its retry persisted through the
native save command. See the full [native result](native-webview-result.json).
Capture lifecycle calls, export file dialogs and export file writes were all
zero in these fixtures.

The [release bundle](bundle-validation.json) rebuilt and passed strict deep
code-signature verification. It is at
`target/release/bundle/macos/Meetily Notepad.app`; the installed application was
not replaced. [Preservation checks](preservation.json) found 944 of 964 incoming
files byte-identical, 20 intentional changes, no missing files, and unchanged
native application source and dependency files. The verified source snapshot
had no drift at the end of the run.

Representative screenshots:

- [Compact source icons beside notes](design-transcript-1440x900.png)
- [Narrow transcript](workspace-480x400-transcript.png)
- [Workspace after the interaction storm](interaction-workspace-recovered-480x400.png)
- [Settings after click and keyboard recovery](interaction-settings-recovered-480x400.png)

The first attempt reached all layout/design cases but skipped interaction
fixtures because its opt-in detector called an uninitialized plugin. The runner
rejected the missing scenarios. The detector now uses the already initialized
store plugin and does not suppress detection errors. Its original evidence is
retained in `interim-run-01-opt-in-wiring/`.

The next attempt reached the interaction phase but stopped when macOS
accessibility exposed no test window after the resize storm. The driver
now activates only its own process and waits for the exact test window before
dispatching keyboard input once. That evidence is retained in
`interim-run-02-native-input-window/`. These are test-driver corrections; native
application code and the original layout assertions were not changed.

## Isolation and interpretation

The [native driver](drive-native.py) accepts only a debug bundle with identifier
`com.meetily.responsive.test`. It retains any earlier test profile in a backup,
creates disposable meetings, and captures the WKWebView itself. The exact
process, test profile and backups are recorded in `native-run.json`. Only the
process launched by this driver is terminated. Production library data and
the installed application are not opened or changed by the driver.

The interaction fixture delays and fails selected save requests deliberately,
blocks capture lifecycle calls, and intercepts export dialogs/file writes.
Recording-state screenshots exercise UI states without physical capture.
Passing these bounded scenarios does not establish that every arbitrary
interaction is safe. Windows, real capture during resizing and long recording
sessions remain separate acceptance gates. Human aesthetic acceptance also
remains open. The prior lint setup prompt remains unresolved; no dependencies
were added for this work.

## Reproduction

```sh
NEXT_PUBLIC_MEETILY_LOCAL_BUILD=1 pnpm --dir frontend build
pnpm --dir frontend exec tsc --noEmit --incremental false
(cd frontend && bun test tests)
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  pnpm --dir frontend exec tauri build --debug \
  --config ../docs/plans/evidence/interaction-stress/native-test-config.json --bundles app
MEETILY_INTERACTION_SMOKE=1 python3 docs/plans/evidence/interaction-stress/drive-native.py \
  --app 'target/debug/bundle/macos/Meetily Responsive Test.app'
./scripts/meetily-notepad.sh build
```

After this verification, the user separately requested installation and stale-app
cleanup. The verified UI build is now installed at `/Applications/meetily.app`;
generated app bundles were retained in a project archive excluded from Spotlight.
See the [installation receipt](../ui-install-20261015/README.md) for the current
paths and launch verification. The test-time installation boundary above remains
the historical record for this run.
