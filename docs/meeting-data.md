# Meeting notes, enhancement and local data

**My notes** are your original editor document. **Enhanced notes** contains
separate generated candidates and approved versions. Generating again preserves
both your originals and the earlier approval. Review a candidate before approving
it. Supporting excerpts identify the saved sources; they do not establish that
every claim is correct. Owners/deadlines remain subject to review, and audio
channel badges do not name participants. Older summaries remain readable as
unverified legacy versions.

Choose a preset or save a custom template and instructions. Enhancement uses
committed notes and available committed transcript passages. Finish recording
and transcript saving before generating. A changed source revision makes older
enhanced versions stale. Failed or cancelled generation preserves your notes
and approved version. Notes-only drafts can also be enhanced when a summary
model is available; writing notes needs no model.

The meeting library searches titles, original notes, transcript passages and the
selected approved/legacy document. Candidates and archived history are excluded
by default. You can set manual participants/tags and filter/sort saved meetings.
An index-updating message means search is still catching up with saved documents.
Opening a transcript hit loads its exact passage, including beyond the visible
page. A missing or changed source is disclosed rather than presented as unchanged.

Choose **Export** in the workspace or library to save selected notes as Markdown
or a complete JSON record with every committed transcript passage, document
version and retained source. Markdown defaults to approved notes when available,
with an optional full transcript. Active/interrupted exports are labeled
incomplete snapshots. See [meeting export](meeting-export.md) for contents,
conversion failures and the versioned record contract.

Built-in local inference and loopback Ollama are local processing choices.
Custom endpoints and other configured providers may be remote. Before sending
meeting sources remotely, enhancement shows the destination and that it sends
committed notes, transcript passages, template and instructions. Local requests
do not silently follow a redirect to a remote host. Model downloads, optional
analytics and updates have their own existing network behavior; this is not a
claim that every application request is offline. Packaged offline/egress
acceptance is tracked in the implementation handoff.

The application keeps these kinds of data:

| Location | Contents and retention |
| --- | --- |
| Application data, `meeting_minutes.sqlite` and its SQLite sidecars | Original notes, transcripts, versioned enhancements/frozen sources, manual metadata, derived search index, deletion jobs and content-free identity tombstones. |
| `meeting_minutes.sqlite.backups/` | Verified pre-migration SQLite snapshots; latest three retained after successful migration. Deleting a meeting sanitizes applicable managed snapshots while retaining other meetings. A failed sanitization remains pending. |
| Managed `meeting_minutes.db` if imported | Managed legacy database copy. Deletion also sanitizes the selected meeting here; the external original chosen for import is preserved. |
| Recording folders | App-created audio, transcripts, metadata and checkpoints. Existing folder references stay authoritative; owned files are checked before cleanup. Imported source files and arbitrary user exports remain yours. |
| Application model directories | Existing Whisper/Parakeet and built-in summary model files; deleting a meeting keeps models. |
| Local templates/settings | Custom templates, configured providers/devices and preferences. |
| WebView IndexedDB `MeetilyRecoveryDB` and per-meeting browser settings | Interrupted transcript recovery and summary-language preferences. Deletion waits for cache transaction completion and removes associated aliases/settings. |

On macOS the usual app data location is
`~/Library/Application Support/com.meetily.ai`; on Windows it is the Tauri
application data directory for `com.meetily.ai`. Isolated test profiles use a
different identifier and library.

Delete uses durable, retryable cleanup. The meeting becomes unavailable while
cleanup runs. **Pending** means some native file/backup or browser cache work has
not completed; use Retry cleanup in the library/status notice, or reopen the app
to resume. Complete describes app-managed logical deletion. It does not promise
forensic erasure from an SSD, operating-system backup, or a user-owned export.

See [the implementation handoff](plans/p0-implementation-handoff.md) for executed
checks and open native/real-model acceptance gates, and
[the notepad guide](notepad.md) for original-note shortcuts and export.
