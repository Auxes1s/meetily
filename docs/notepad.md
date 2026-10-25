# Meetily Notepad on macOS

For complete meeting records and selected original/approved Markdown, use the
workspace or library **Export** control. See [meeting export](meeting-export.md).
The original-note shortcuts below remain available.

Section 5.1 adds durable original notes to the existing desktop app. Start a draft,
write in BlockNote, and optionally record. The transcript is collapsed initially;
existing summaries stay separate from original notes. No model is needed to write
or reopen notes. On a fresh installation, choose **Use notes only**.

## Launch

Open `/Applications/meetily.app`, or run:

```sh
open "/Applications/meetily.app"
```

The single installed copy uses `com.meetily.ai` and the existing Meetily library.
On 15 October, the verified warm/sage UI build with compact source icons and
interaction hardening replaced this app. Fourteen timestamped backups, the
duplicate user-local copy and the superseded canonical app were moved to Trash;
meeting data and recordings stayed in place. Launch by name was verified against
the canonical application. See the
[UI installation receipt](plans/evidence/ui-install-20261015/README.md) and the
[earlier launch repair receipt](launch-repair.md). Local upstream updates are
disabled. Before an additive migration the app makes an integrity
checked SQLite snapshot, including committed WAL data, beside the database in
`meeting_minutes.sqlite.backups/`. It retains the latest three snapshots after
successful migration. It never repairs corruption by deleting WAL files.

## Develop or rebuild

From the repository root:

```sh
./scripts/meetily-notepad.sh dev
./scripts/meetily-notepad.sh build
./scripts/meetily-notepad.sh install
```

`install` installs a new bundle without replacing an existing local bundle.
It targets `~/Applications/Meetily Notepad.app`, so it does not update the current
canonical installation at `/Applications/meetily.app`.
`build` leaves it in `target/release/bundle/macos/Meetily Notepad.app`.
Prerequisites are Node, pnpm, Rust/Cargo 1.91 or later, CMake, Python 3, and Apple's Command Line
Tools. Install frontend dependencies with `pnpm --dir frontend install --frozen-lockfile`.
The launcher builds the local inference sidecar, selects Metal, and supports the
pinned cidre Objective-C shims with either Xcode or Command Line Tools. The CLT
fallback only handles cidre's `pomace` project. Optional llama HTTPS code is
excluded to avoid an OpenSSL 4 incompatibility; Meetily retains its native model
downloader. No lockfile refresh is required.

Speaker echo cancellation runs automatically when both microphone and system
audio are captured. It uses the system channel as a reference before microphone
gain, VAD, and mixing, while preserving independent microphone speech. Keep
system audio capture enabled when using speakers; without a playback reference,
the app cannot cancel speaker pickup. Headphones also avoid acoustic pickup.

## Notes and recovery

The save indicator says **Saved** only after SQLite acknowledges the current
revision. Saves serialize per meeting; revision conflicts retain local edits and
allow saving a copy or explicitly reloading the saved revision. Failed Markdown
conversion cannot prevent the authoritative JSON from saving. Markdown and JSON
export flush notes first. JSON preserves editor blocks and timestamp metadata.

Shortcuts: Cmd/Ctrl+N creates a draft, Cmd/Ctrl+S saves, Cmd/Ctrl+Shift+T toggles
transcripts, and Cmd/Ctrl+K focuses library search outside the editor. Inside the
editor its link shortcut remains available. Timestamp inserts a link at the cursor
using the recording's native audio clock. It opens only matching passages from
the same recording; missing passages leave the marker intact.

Recording and recovery carry the draft ID and recording session. Native stop
persists final segments before returning, even if the UI has navigated away.
Live microphone and system speech is split into chunks of at most five seconds,
with shorter chunks after qualifying pauses. Text appears once the selected
model finishes decoding each chunk.
Repeated finalization uses the same meeting and sequence IDs. Original-note
saves and transcript recovery use independent storage; purging the transcript
cache does not delete notes. Normal quit waits for note saves and refuses an
active recording. Closing the window retains the existing tray behavior.

## Verification

```sh
cd frontend
bun test tests/lib/meeting-notes-session.test.ts
pnpm exec tsc --noEmit
cd ..
PATH="$PWD/scripts/macos-clt:$PATH" CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE \
  cargo test --locked -p meetily --lib database::tests
./scripts/meetily-notepad.sh test
```

`test` builds `Meetily Notepad Test.app` with a separate identifier and library,
`com.meetily.notepad.test`. It never migrates the real library. To run the opt-in
native WebView smoke scenario in that profile, quit its release test app, then:

```sh
MEETILY_NOTEPAD_SMOKE=1 NEXT_PUBLIC_MEETILY_LOCAL_BUILD=1 \
PATH="$PWD/scripts/macos-clt:$PATH" CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE \
  pnpm --dir frontend exec tauri dev --config src-tauri/tauri.notepad-test.json
```

The scenario exercises model-free setup, the actual editor, an acknowledged
native save, a draft switch, and a reload. Its report is
`~/Library/Application Support/com.meetily.notepad.test/notepad-smoke.json`.
Smoke reporting is rejected in release builds and in the real-library profile.

Windows validation, 45-minute real capture, and a four-hour real meeting are
separate acceptance gates. Synthetic and short native smoke checks do not establish
those gates or microphone/system transcription quality.
