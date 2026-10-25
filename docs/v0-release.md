# Meetily v0 release readiness

This fork's first candidate is **0.5.0-rc.1**, following its upstream 0.4.1
baseline so installed-version comparisons remain ordered. The product is still
v0. The user authorized an **unsigned preview** on 16 October 2026 and confirmed
that no Windows 11 test machine is available. The preview may be published after
its automated package checks pass; full v0 acceptance remains separate.

## Owner closeout — 17 October 2026

**Long-recording test: APPROVED by the project owner after manual verification.**
The owner confirmed: “mark the long recording test as approved, i have verified
it myself.” This closes the long-recording acceptance item for the retained app.
Existing automated receipts remain unchanged; this approval records manual
acceptance without adding measured durations, provider coverage or performance
results.

The owner accepted the app as it is and directed that unavailable Windows
verification be left as is. **Windows acceptance is deferred and unverified.**
At closeout, candidate run `38929108241` had passed frontend and macOS jobs;
its Windows job and Windows 11 installed-QA run `38930176529` were still running.
Those observations are a snapshot, not Windows acceptance.

Retain the installed app and the existing unpublished preview draft. This
closeout updates documentation only. Earlier duration-run recommendations are
superseded; resume development or deferred verification only on a new owner
request. Other recorded acceptance limitations remain unchanged.

## Supported targets and evidence

| Target | Package | Acceptance status |
| --- | --- | --- |
| Windows 11 x64, AVX2-capable CPU | MSI and NSIS setup | Native build and installed WebView2 smoke configured on Windows Server 2022 CI. This does not establish Windows 11 physical capture, installer upgrade or permission behavior. |
| macOS 14.2 or later, Apple Silicon | app and DMG | Local macOS UI and workflow evidence exists; fresh candidate build and native checks are recorded below. Intel Macs are not an accepted target. |

The candidate uses the existing app identifier and library, with the existing
migration-backup behavior. Quit other Meetily copies before opening it. A normal
installation must retain existing notes, transcripts, IDs and approved documents;
fixture upgrade acceptance is still required before publication.

## Onboarding and responsive UI

First launch offers **Get Started** or **Use notes only**. Notes-only setup needs
no model download. AI setup explains downloads, shows progress and retry actions,
and requests macOS recording permissions. Windows users receive microphone
settings guidance. Local model processing works offline after model installation;
explicit remote providers receive the selected source text.

Current fixes prevent initial defaults from replacing a slowly loaded setup,
serialize progress saves before completion, deduplicate Finish requests and
allow retry after persistence failures. A resumed macOS-only step returns to the
download screen on Windows. Native model verification prevents a stale download
flag or failed availability check from finishing setup. Long or short windows retain scrollable onboarding
actions and bounded notes/transcript/library/settings content.

## Build and verification

The [candidate workflow](../.github/workflows/v0-candidate.yml) runs frontend
behavioral tests, onboarding model checks, portable GGML checks, lint, production
static export and TypeScript before building both targets. It reuses the existing
Metal/CoreML macOS and Vulkan Windows paths, FFmpeg downloader, verified ONNX
runtime staging and portable Windows CPU checks. The matching llama helper is
built from the same locked source. Windows bundles the pinned official x64 Vulkan
loader and its redistribution license, with the archive checked against
[LunarG's published checksum](https://vulkan.lunarg.com/sdk/files.json). The
installer also includes the matching x64 Microsoft C++ redistributable DLLs from
the build toolset, with Microsoft Authenticode signatures and copied-file hashes
verified during staging. They are serviced with each application release. The
candidate enforces a 480×400 minimum window size, matching the smallest tested
layout, and advertises macOS 14.2 to match the compiled binary's deployment target.
The application version remains `0.5.0-rc.1`; MSI uses the equivalent numeric
installer version `0.5.0.1` because Windows Installer rejects text prerelease IDs.
The unsigned candidate build leaves Apple certificate variables unset so Tauri
uses ad-hoc signing rather than trying to import an empty certificate.

Windows smoke installs the actual NSIS package on a disposable CI host, opens
the installed executable in WebView2 through
[Tauri's WebDriver setup](https://v2.tauri.app/develop/tests/webdriver/ci/), checks
fresh notes-only onboarding at four window sizes, edits and commits Unicode
notes, resizes through five workspace sizes and reloads the saved document. It
also checks installed FFmpeg execution, helper protocol and ONNX file presence.
Its JSON report records the actual host. No hardware capture or model inference
is claimed by this smoke check.

Candidate packages disable the upstream updater and updater artifacts. They are
not Windows Authenticode signed or Apple notarized: the fork currently has no
signing secrets configured. macOS uses the existing ad-hoc signing setup. A
signed public release needs the owner's signing credentials configured and
verified. The authorized preview must not be described as signed/notarized.

Reproduce frontend checks:

```sh
pnpm --dir frontend install --frozen-lockfile
(cd frontend && pnpm test && node tests/lib/onboarding-summary-model.test.mjs)
node --test .github/verify-portable-ggml.test.cjs
pnpm --dir frontend lint
NEXT_PUBLIC_MEETILY_LOCAL_BUILD=1 pnpm --dir frontend build
pnpm --dir frontend exec tsc --noEmit --incremental false
```

Local run receipts and native screenshots are retained under
`target/v0-qa-20261016/`. They are intentionally excluded from the public source
commit. The local project plans retain the more detailed feature acceptance
history; they have not been substituted with this candidate checklist.

## Publication gates still open

- Windows 11 native permissions, microphone/system audio, pause/resume, tray,
  export/deletion locks, upgrade and restart acceptance.
- Current macOS physical-source and foreground recording/editing checks beyond
  the owner's approved long-recording test.
- Source-by-source summary/enhancement accuracy: the latest model evaluation
  still fails the existing AC07 gate; successful inference is not semantic
  acceptance. Generated candidates require review before approval.
- Packaged offline/egress and retention/deletion inventories; library scale
  ranking/latency and remaining crash/upgrade/platform gates.
- Final package integrity receipts for both targets. Signing/notarization is
  explicitly deferred for the authorized unsigned preview.

Do not call the preview fully validated or mark the full goal complete while
these required acceptance gates remain open. The preview release notes must
retain the Windows 11 and model-quality limitations. No feature acceptance
threshold has been lowered.

## Installing the unsigned preview

**Windows:** download the x64 NSIS `-setup.exe` (or MSI) from this fork's preview
release, verify its SHA-256 against the accompanying checksum file, then run it.
SmartScreen may report an unknown publisher because the preview is unsigned.
Use **More info → Run anyway** only after verifying the source and checksum.
If organizational policy blocks unsigned applications, ask your administrator;
do not change that policy. The installer supplies WebView2 when needed.

**macOS Apple Silicon:** download the arm64 DMG from this fork's preview release,
verify the checksum, open it and drag Meetily into Applications. Quit an existing
Meetily process before replacing its app bundle. macOS may block first launch
because this preview is not notarized. Use **System Settings → Privacy & Security
→ Open Anyway** for the verified Meetily download. Do not disable Gatekeeper.
Grant microphone/audio-capture access when you choose to record.

Choose **Use notes only** to start immediately. Use AI setup to download models
when internet access is available. Review generated summaries against original
notes and transcripts before approving or sharing them.
