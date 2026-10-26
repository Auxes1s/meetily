# Meetily UI design upgrade

Authorized on 2026-10-15 as an additional product goal, separate from the
meeting-experience MVP. The existing data, recording, enhancement and export
acceptance gates still apply independently.

## Design direction

Warm light theme, sage accents and selective glass within the existing native
window. Whole-app coverage includes navigation, original/enhanced notes,
transcripts, recording controls, library, settings, onboarding, imports,
dialogs and supporting states. Layout hierarchy may be refined; dark mode,
desktop transparency, rebranding and new product features are deferred.

References: [Sunsama's calm product presentation](https://www.sunsama.com/a)
and [Apple's material hierarchy](https://developer.apple.com/design/human-interface-guidelines/materials).
These inform the treatment rather than introducing Sunsama's planning features
or promising native Apple Liquid Glass rendering in a web view.

| Role | Value |
| --- | --- |
| Canvas | Ivory `#F6F4EF` |
| Document | Paper `#FFFEFB` |
| Text / secondary text | `#2D332F` / `#626B62` |
| Primary / primary text | Sage `#4F6B57` / white |
| Selection / selection text | `#E6EDE3` / `#3F5B45` |
| Typeface | Existing Source Sans 3 throughout shell and BlockNote |
| Type scale | 14px interface; 16px document, line-height 1.65 |
| Materials | Opaque reading surfaces; bounded 16px blur for chrome |
| Corners | 12–16px surfaces, capsule recording dock |
| Motion | Brief transitions; system reduced-motion policy |

Static palette contrast: main text/canvas 11.75:1, secondary text/canvas
5.03:1, white/primary 5.88:1, selection text/fill 6.30:1. Glass also needs
rendered background checks; these ratios alone do not establish accessibility
of every composed state.

## Delivery stages

| ID | Implementation |
| --- | --- |
| D01 | Semantic HSL tokens, warm canvas, paper/glass materials, shared controls, focus, fallback and motion rules. |
| D02 | Sage/glass navigation; quieter document switcher; unbordered document title/status/action header; labeled secondary note-export menu. |
| D03 | Reading-width editor, unified fonts, opaque transcript/document surfaces, source indicators and compact glass recording controls. |
| D04 | Consistent library, metadata, enhancement review, settings, onboarding, import, dialogs, notifications and empty/error/loading states. |
| D05 | Existing functional checks plus isolated native layout, state-continuity and screenshot verification; reviewable macOS bundle. |

Implementation uses existing React, Tailwind, Radix, BlockNote, Lucide and
Framer Motion dependencies. Shared CSS material helpers and presentation
variants are additive; native commands, API payloads, database schemas and
storage services retain their contracts.

## Preservation and acceptance

- Keep original notes, generated candidates and approved notes distinct.
- Keep save acknowledgements, conflicts, source qualifiers, deletion guards,
  export choices and recording errors visible and actionable.
- Preserve editor identity and undo/redo, selected enhancement, pending saves,
  transcript reading position and recording continuity during resizing.
- Keep 64/256px sidebar geometry, the 700px navigation overlay threshold and
  the existing 960px workspace / 900px nested split thresholds.
- Keep tables/code locally scrollable and all dialogs/actions reachable at
  480×400 and in short windows. Settings categories scroll horizontally.
- Use opaque materials for unsupported or reduced transparency, increased
  contrast and forced colors. Both CSS and Framer Motion respect reduced motion.
- Use only disposable test content and the isolated
  `com.meetily.responsive.test` identity for native visual checks.
- Preserve incoming tracked and untracked changes and the user's library.

Current verification and screenshots are recorded in
[UI design evidence](evidence/ui-design/README.md). Automated and fixture checks
are distinct from human aesthetic acceptance, physical capture and Windows
native acceptance.

D01–D05 are implemented. Their initial automated verification passed 128 frontend tests,
91 native layout cases, 34 additional design cases and 122 continuous resize
steps. The local macOS bundle rebuilt and passed code-signature verification.
The separate acceptance limits above remain open.

## Interaction hardening and compact source icons

The subsequent responsive-interaction request adds synchronous action guards,
stale-response ownership, visible pending states, save-failure recovery and
keyboard document tabs. Source badges now show icons with accessible names and
hover descriptions, and the transcript metadata rail uses less space. See the
[interaction evidence](evidence/interaction-stress/README.md) for the current
frontend, native and bundle checks and their acceptance limits.

## Latest delivery and installation — 15 October 2026

Interaction hardening is delivered. Its final receipt records 166 frontend
tests/641 assertions, production build and subsequent TypeScript, 91 native
layout cases, 34 design cases, all eight interaction scenarios, 122 continuous
resize steps and 46 native screenshots. These expand the earlier design checks;
capture lifecycle and export file dialogs/writes were blocked in the fixtures.

The user subsequently authorized installation and removal of stale app copies.
The verified release UI build is now installed at `/Applications/meetily.app`
with the existing `com.meetily.ai` identity. Sixteen old copies were moved to
Trash without emptying it; eight generated bundles were preserved under
`target/archived-app-bundles.noindex/20261015T103927/`. The installed executable
matches the [installation receipt](evidence/ui-install-20261015/README.md).
Historical build paths are recreated by future builds.

Implementation and installation are delivered; human aesthetic acceptance,
Windows rendering, physical capture during resizing and long-session behavior
remain open. The best next project assignment is the remaining Q02 fidelity/
capture and duration acceptance in the
[current project plan](meeting-experience-upgrade.md#current-project-status--15-october-2026).


## ASR and UI pipeline reinforcement — 15 October 2026

R10–R13 / U12–U15 are implemented. See the
[reinforcement handoff](pipeline-reinforcement-handoff.md) and
[verification evidence](evidence/pipeline-reinforcement-20261015/README.md).
The changes cover durable bounded ASR intake, checkpoint/Stop drainage and
recovery, source continuity, soft limiting, row-level transcript updates,
scrolling, shared controls and session-fenced frontend cleanup. The installed
production app/library are preserved; the test package is isolated.

Short replay and regression results do not close Q02. The latest 45-minute and
four-hour workloads remain next; foreground paint (unavailable with loginwindow
in front), physical capture/fidelity and Windows gates remain open. E08/AC07,
Q03/Q04 and release acceptance are unchanged.
