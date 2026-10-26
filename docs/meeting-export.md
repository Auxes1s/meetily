# Meeting export

Choose **Export** above the workspace’s document tabs or beside a meeting in the
library. Choose a format, then **Save export…** to open the native Save dialog.
The export first waits for that meeting’s pending original, enhanced and legacy
editor saves. A failed save or revision conflict retains edits and stops export.

Markdown defaults to approved enhanced notes when a reviewed approval exists;
otherwise it selects **My notes**. You can choose either document or both. A
legacy/unverified summary is not treated as an approval. **Include full
transcript** is off by default. Supporting excerpts and note timestamps are
included when available, with source identifiers, timing and readable channel
labels. Evidence comes from the saved version’s frozen sources, including when
the current transcript has changed or a passage has been removed. A source
reference does not establish that its claim is correct.

**Complete meeting record (.json)** always includes every committed transcript
passage, regardless of the displayed transcript page or Markdown choices. It
also includes meeting dates/status, manual participants/tags, original blocks
and Markdown, timestamp links, enhancement settings/approval pointer, and all
candidate, approved, archived, failed, cancelled, discarded and legacy versions
that remain stored. Actions, review labels, owners/deadlines and source
snapshots remain in their documents. Older summaries and backups remain
unverified.

Recording/finalizing and interrupted meetings can be exported. These files are
clearly labeled incomplete. All exports represent committed data at the
recorded snapshot time, **before** the Save dialog opens; later edits or
transcript arrivals belong to a subsequent export. Missing capture dates stay
unknown. `committed_available` describes the stored transcript and does not
certify that all speech was captured or transcribed.

Markdown is a readable projection. When a selected nonempty document has no
current projection, export reports the conversion failure; select JSON to
preserve its saved blocks. JSON retains exact stored document/source strings
alongside parsed projections when readable. Malformed historical JSON produces
a warning and remains available as raw text.

Save to a local `.md` or `.json` file outside application data and managed
recording folders. Suggested filenames retain Unicode while replacing invalid
Windows characters and protecting reserved names. Publication uses a synced
temporary file and atomic replacement. Cancel creates no output file; failed
writes preserve an existing destination. Pending editor saves acknowledged
before cancellation still remain saved in the meeting.

Generated application metadata excludes absolute recording/model/cache paths,
credentials and endpoint configuration. User-authored content and its retained
raw strings are preserved as written. Exports are user-owned copies. Meeting
deletion does not remove them. Audio copying, PDF/DOCX conversion and record
import/restore are outside this feature.

## Portable JSON version 2

`format: "meetily.meeting-record"` and `schema_version: 2` distinguish this
record from the earlier original-note JSON format. The native schema is
`MeetingRecord` in `frontend/src-tauri/src/database/export.rs`; its frontend
contract is `MeetingRecordExport` in `frontend/src/types/meeting-export.ts`.

| Field | Contents |
| --- | --- |
| `exported_at`, `transcript_coverage` | UTC snapshot time; `committed_available`, `incomplete_active` or `incomplete_interrupted`. |
| `meeting` | Identity/title, created/updated and nullable capture dates/timezone/session, status, manual participants/tags. |
| `recording_sessions` | Ordered recording identities, status, dates, nullable duration, duration basis and continuous timeline offset; excludes local audio folder paths. |
| `original_notes` | Nullable original document with exact `notes_json`, parsed `blocks`, Markdown/plain text, revision/conversion status, dates and `timestamp_links` with available passage text/timing. |
| `transcripts` | Every ordered passage, including empty and legacy passages, with IDs, text/display timestamp, session/sequence, nullable relative timing/duration, capture source and readable label, plus saved summary/action/key-point fields. |
| `enhancement_settings` | Nullable saved template/instructions/revision and the stored approval pointer. |
| `versions` | All stored versions and states/revisions, exact `document_json`/`source_snapshot_json`, parsed `document`/`source_snapshot`, fingerprints, notes revision, template/instructions, safe inference provenance, dates, generic error indication and source staleness. |
| `legacy_summary` | Nullable saved legacy result/backup strings and parsed projections, dates/status and explicit `unverified` label. |
| `warnings` | Parse/shape warnings without replacing malformed raw content. |

Transcript order uses session order, recording-relative start, sequence and
ID, with missing start/sequence values last. Each passage retains its local audio
clock and adds `timeline_offset`; Markdown displays their sum across recordings.
Source labels are **Microphone**,
**System audio**, **Mixed audio** or **Unknown**; they do not name participants.
Frozen source projections normalize the same labels while their raw snapshot
strings remain unchanged.

The IPC commands are `api_get_meeting_export_options(meetingId)` and
`api_export_meeting({ request })`. The latter takes `meeting_id`, `format`
(`markdown`/`json`), `document` (`original`/`approved`/`both`),
`include_transcript`, and `expected_approved_version_id`. JSON ignores Markdown
content choices. The result is `{ "status": "saved" }` or
`{ "status": "cancelled" }`; errors carry `code` and `message`. A changed
approval requires reopening Markdown export. Missing/deleting meetings are
rejected, and final publication is serialized with managed deletion.
