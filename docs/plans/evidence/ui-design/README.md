# Warm light and sage UI design

Implementation and verification of the additional UI goal authorized on
2026-10-15. See [design specification](../../ui-design-upgrade.md).

## Delivered presentation

- Shared ivory/paper/sage palette, Source Sans 3, accessible focus treatment,
  warm form controls and consistent dialogs, menus and tabs.
- Bounded frosted navigation, workspace chrome and recording dock; opaque
  original/enhanced notes and transcripts.
- Quiet document switcher and title/save controls. The note-action menu labels
  the original-note Markdown/JSON exports separately from full meeting export.
- Consistent library, metadata, enhancement/source review, model/device/language
  settings, onboarding, import and update/progress/error surfaces.
- Narrow settings categories scroll within their own strip. Short transcript
  windows use compact chrome so the transcript retains usable space.
- Scoped BlockNote tokens inherit the application palette, including menus.
  Reduced motion is applied to both CSS and Framer Motion; glass has solid
  fallbacks for reduced transparency, contrast and forced-color settings.

## Checks

The machine-readable [check record](checks.json),
[native result](native-webview-result.json) and
[preservation record](preservation.json) are the current verification record.

Final result: **128 frontend tests / 510 assertions**, production frontend build,
standalone TypeScript and onboarding checks passed. Native macOS verification
passed **91 existing layout cases, 34 additional design cases and 122 continuous
resize steps**, retaining 44 WKWebView screenshots. The release app rebuilt and
passed strict deep code-signature verification; see [bundle record](bundle-validation.json).
No installed application was replaced.

Frontend verification uses `bun test tests`, the standalone onboarding assertions,
`pnpm build`, and a subsequent `pnpm exec tsc --noEmit --incremental false`.
The existing `pnpm lint` command remains blocked at Next's ESLint setup prompt;
its failure is retained in `lint.log`, separately from build/type success.

## Native screenshots and isolation

The [native driver](drive-native.py) verifies a debug bundle with identifier
`com.meetily.responsive.test`, retains any prior **test** profile in a backup,
launches its own process, and terminates only that process. The production
application and library are not launched or inspected. Exact test profile and
backup locations are recorded in [native run](native-run.json).

Screenshots come from the isolated WKWebView itself; no desktop pixels or private
meeting content are captured. Stress fixtures retain Unicode, long titles/URLs,
wide tables and long transcript passages. Separate synthetic design-review
content provides readable aesthetic examples.

Representative views:

- [Original notes](design-notes-1440x900.png)
- [Notes and transcript](design-transcript-1440x900.png)
- [Enhanced notes](design-enhanced-1440x900.png)
- [Note actions in a narrow window](design-note-actions-480x400.png)
- [Library](design-library-1440x900.png)
- [Settings](design-settings-general-1440x900.png)
- [Narrow settings](design-settings-general-480x400.png)
- [Narrow transcription settings](design-settings-transcription-480x400.png)
- [Onboarding](onboarding-welcome-1100x700.png)
- [Solid material and reduced motion](design-reduced-motion-opaque-1100x700.png)

The narrow onboarding screenshot records the scrolled position where both
actions are reachable; it does not represent the initial scroll position.

## Interpretation and limits

Recording-state screenshots are **visual fixtures**, including idle, recording,
paused, stopping, processing, saving and error. The debug harness uses existing
context actions/events and a mocked status transport, blocks native lifecycle
commands, and records the observed calls. They do not establish physical audio
capture or transcription behavior.

The solid/reduced-motion check activates the shipped media rules through CSSOM
inside the disposable test document. It does not change or verify macOS system
accessibility preferences. Human aesthetic acceptance, Windows native rendering,
physical capture during resizing, and model-quality gates remain separate.

The first native attempt exposed a 32px transcript scroll area at 480×400.
Compact transcript controls and short-window spacing corrected that regression;
the original minimum-space assertions were retained. Its diagnostic evidence is
kept in `attempt-01-transcript-height/`.

The second attempt detected a recommendation badge protruding beyond its model
card. Recommendation badges now participate in wrapping title rows, model
controls wrap, and provider labels remain bounded. That diagnostic is retained
in `attempt-02-transcription-badge/`; all settings categories passed at wide,
narrow and short sizes on the final run.

## Reproduction

```sh
NEXT_PUBLIC_MEETILY_LOCAL_BUILD=1 pnpm --dir frontend build
CMAKE_DISABLE_FIND_PACKAGE_OpenSSL=TRUE PATH="$PWD/scripts/macos-clt:$PATH" \
  pnpm --dir frontend exec tauri build --debug \
  --config ../docs/plans/evidence/ui-design/native-test-config.json --bundles app
python3 docs/plans/evidence/ui-design/drive-native.py \
  --app 'target/debug/bundle/macos/Meetily Responsive Test.app'
./scripts/meetily-notepad.sh build
```

The release build produces `target/release/bundle/macos/Meetily Notepad.app`.
Installed application bundles are not replaced by this workflow.
