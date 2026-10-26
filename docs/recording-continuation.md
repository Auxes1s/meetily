# Record in the selected meeting

Record now attaches a new capture session to the meeting displayed when the
button was pressed. Completed and interrupted meetings keep their identity,
title, original notes, reviewed enhancements and prior transcript. With no
selected meeting, Record creates one draft; New meeting remains explicit.

Each capture has a separate identity and audio folder. Transcript passage IDs
include that session, so sequence numbers restarting at zero cannot replace
earlier passages. The meeting view presents one continuous timeline using the
preceding sessions' audio durations while preserving each passage's local audio
clock for playback, note links and retranscription. Missing historical timing
stays unknown; legacy offsets use metadata or the available transcript bound.
Existing links, passages and audio remain owned by their original session.

The start deadline covers model checks, saving that meeting's notes, draft
creation and native initialization. Rapid clicks share one admission. Successful
native responses and bounded state polling can both release Starting, even if
a lifecycle event is missed. Cancel start and Check status offer recovery.
Cancelled or timed-out work cannot publish a later capture. Native device calls
are cooperatively cancelled: if an OS call does not return, the pending owner
stays reserved and the error asks for an app restart. Other meetings' pending
editor saves do not block Record.

The additive migration creates legacy session records without deleting or
rewriting transcript text, IDs or local timestamps. Saving/retrying an older
session cannot finalize the latest recording. Startup marks unfinished sessions
interrupted. Deletion owns all session folders and tombstones all identities;
recovery and retranscription validate the selected session's ownership. The
recordings menu opens a chosen audio folder. Complete exports use schema 2,
include portable session metadata and offsets, and omit local folder paths.

## Validation on macOS, 14 October 2026

- Frontend: 124 tests passed; TypeScript and production build passed.
- Rust library: 352 passed, 8 ignored, one existing failure in
  `audio::device_detection::tests::test_builtin_mic_detection`. Its recorded
  baseline also reports Wired versus expected Unknown; this change does not
  alter that detector.
- New regression cases cover selected-meeting identity, navigation during a
  start, double clicks, stalled saves/preflight/native calls, cancellation,
  response/event reconciliation, two sessions with repeated sequence numbers,
  recovery retries, additive migration, cancellation metadata restoration,
  multi-folder deletion and continuous portable exports.
- The isolated `com.meetily.recording.test` native webview captured microphone
  and system audio in two recordings of one meeting, with two saved audio files,
  a continuous offset and unchanged original notes/title. Native speech playback
  uses a fixture; this does not establish transcript/model accuracy, Bluetooth,
  Windows or long-session acceptance.
- The final isolated run also rejected a pre-cancelled start, recovered from
  unavailable Parakeet models, and then completed both Whisper recordings.

The isolated hardware check is debug-only and requires
`MEETILY_RECORDING_SMOKE=1` with `tauri.recording-test.json`. The production
application rejects its fixture/probe command. Screenshots and reports live in
the isolated profile, and command logs live under `/tmp/meetily-recording-*`.
The source snapshot and private library backup are in
`/tmp/meetily-recording-fix-20261014-213425`; the original dirty workspace and
unrelated concurrent work are retained.

Both `/Applications/meetily.app` and `~/Applications/Meetily Notepad.app` now
match the verified release bundle file for file. The normal Applications app
was launched and remained running. At the post-install check, the user library
had 3 meetings, 432 passages, 3 original-note documents and 3 legacy sessions.
Every original field in the 15 source tables matched the earlier online
pre-migration snapshot; passage IDs and existing session ownership survived,
and SQLite integrity/foreign-key checks passed. The additive session migration
was already present by the final pre-replacement snapshot, so this preservation
check also compares against the earlier snapshot rather than attributing its
timing to replacement.

Previous launch copies remain at
`/Applications/meetily.recording-before-20261014-230531.app` and
`~/Applications/Meetily Notepad.recording-before-20261014-230531.app`.
Private backups, build hashes and verification receipts remain in the temporary
directory above. Those files contain the library and are not sharing artifacts.
