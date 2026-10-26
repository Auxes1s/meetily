# Responsive window resizing

Validation evidence for the layout changes on 2026-10-14. Native checks use the
macOS Tauri WKWebView on arm64, macOS 27.0.1, with normal app animations enabled.

Result: 91 native layout/dialog cases and 122 continuous resize steps passed.
All 111 frontend tests passed (455 assertions); explicit TypeScript checking,
the production frontend build and the final macOS release rebuild passed.

The workspace uses its available width: notes and transcripts share the space
at 960px or wider; below that, the transcript drawer occupies the content pane
and its toggle returns to notes. Transcript/summary tabs measure their own
container and enable a split only at 900px. Below a 700px window width, expanded
navigation overlays the 64px rail. The user's navigation preference survives
resizing. Editors remain mounted through layout changes.

Headers wrap, note actions compact, and editors, transcripts, the library and
dialogs have bounded scroll areas. Unicode, long titles and URLs wrap or remain
inside their own controls; wide editor tables and code scroll locally. Virtual
transcript measurement compensates only for passages wholly above the visible
reader, preventing a resize from jumping a partially visible passage to the top.

## Native test coverage

| Requested window | Web content area |
| --- | --- |
| 1440 × 900 | 1440 × 868 |
| 1100 × 700 | 1100 × 668 |
| 900 × 600 | 900 × 568 |
| 768 × 600 | 768 × 568 |
| 600 × 500 | 600 × 468 |
| 480 × 400 | 480 × 368 |
| 1100 × 400 | 1100 × 368 |

The native suite covers both sidebar states, open/closed transcript drawers,
original/enhanced documents, transcript/summary tabs, the library and both export
entry points. Continuous native resizing crosses the width and height breakpoints
in both directions over 122 steps. Disposable documents contain long titles,
Unicode, long unbroken URLs, a 12-column table and 70 long transcript passages.

Native keyboard input creates real editor transactions. The isolated test holds
original and enhanced save requests at the native IPC transport, verifies that
the database has not yet acknowledged the edits, resizes, releases the requests,
then verifies native persistence and undo/redo. Editor element identity, selected
enhanced version, transcript context, transcript scroller and source-note
navigation are checked explicitly. No recording lifecycle command should occur
during layout changes.

Fixtures use `com.meetily.responsive.test`, separate from the production library.
The smoke runner requires a debug build, that exact identifier and the explicit
`MEETILY_RESPONSIVE_SMOKE=1` environment flag. Snapshot capture uses the native
WKWebView, with no private meeting content in the screenshots.

## Evidence

- [Native cases and continuity assertions](native-webview-result.json)
- [Native driver output](native-driver.log)
- [Frontend tests](frontend-tests.log), [TypeScript](typescript.log),
  [production build](production-build.log), [release build](release-build.log)
- [Incoming working-tree preservation](preservation.json)
- [Installed bundle checks and backups](installation.json)
- [Normal launch and production library checks](normal-launch.json)

Representative screenshots:

- [Wide notes and transcript](workspace-1440x900-transcript.png)
- [Narrow original notes](workspace-480x400-original.png)
- [Narrow transcript](workspace-480x400-transcript.png)
- [Short transcript window](workspace-1100x400-transcript.png)
- [Enhanced notes](workspace-600x500-enhanced.png)
- [Wide library](library-1440x900.png), [narrow library](library-480x400.png)
- [Narrow export dialog](library-export-480x400.png),
  [short export dialog](library-export-1100x400.png)

## Limits

Windows native behavior has not been tested. Native resizing checks use an idle
recording state and assert that resizing sends no start/stop/pause/resume commands;
physical microphone/system-audio recording during resizing is not tested. Audio
capture, transcription, enhancement, export, deletion and persistence logic are
preserved; this change does not replace their separate behavioral test coverage.

## Installation and preservation

Both `/Applications/meetily.app` and
`~/Applications/Meetily Notepad.app` match the final release bundle and pass
strict code-signature verification. Prior bundles were preserved and verified:

- `/Applications/meetily.previous-responsive-20261014-131415.app`
- `~/Applications/Meetily Notepad.previous-responsive-20261014-131415.app`

No production Meetily process was running during replacement. The replacement
routine requests the existing save-flush quit flow if a production process is
running and aborts replacement if it remains open. A private SQLite backup was
made before replacement. Older backup registrations were removed from macOS
Launch Services; `open -a meetily` now launches the updated
`/Applications/meetily.app`. Its native 1100 × 700 window was visually inspected.

Production database integrity and foreign-key checks pass. All meeting,
document, transcript, enhancement and other table contents match the snapshot,
except `updated_at` timestamps in seven existing deletion jobs refreshed by
normal startup maintenance. Those jobs' other fields are unchanged; their
pre-existing pending cleanup notices remain. The production library contains
no responsive-test fixtures.

The incoming working tree was backed up before edits: 332 of its 345 dirty files
remain byte-for-byte identical; 13 existing files received the focused layout
or isolated native-test edits listed in `preservation.json`. Eight previously
clean UI files changed, and two focused frontend regression tests plus the native
fixture runner and evidence were added. Existing audio, persistence, enhancement,
export and deletion implementation files are unchanged.
