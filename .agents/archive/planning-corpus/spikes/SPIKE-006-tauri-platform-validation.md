# Objective

Name the MVP platform matrix and validate whether Tauri WebView is sufficient for the accepted MVP UI and text-like artifact preview scope on those platforms.

# Context

ADR-002 finalizes Tauri for MVP while deferring Chromium/Electron strategy. The MVP excludes browser panels, browser automation, screenshots, DOM extraction, active HTML rendering, and rich artifact previews. FINDING-015 notes that the target platforms for Tauri validation are not yet named, which makes Phase 0 validation either too broad or too narrow.

# Related Findings

 - FINDING-015 Which Target Platforms Must Pass Tauri Validation?
 - FINDING-009 Tauri Choice Is Reasonable But Still Needs Platform Validation.

# Questions To Answer

 - Which platforms are mandatory for MVP validation: macOS Apple Silicon, macOS Intel, Windows, Linux, or a narrower first validation target?
 - Which platforms are launch targets versus later compatibility targets?
 - What minimum OS versions and WebView engines must be validated for each target platform?
 - Does Tauri support the MVP app UI, settings, session transcript, tool activity, approval prompts, file browser, Git diff viewer, and text-like artifacts on each target platform?
 - Are there platform-specific issues with focus, clipboard, text selection, keyboard shortcuts, scrolling, file dialogs, keychain integration, subprocess supervision, local file access, or accessibility that affect MVP acceptance?
 - Do text, Markdown, logs, diffs, generated files, and basic artifact links render consistently enough for MVP?
 - Which post-MVP browser or rich-preview requirements would force a Chromium/Electron decision?
 - What validation failures would block MVP versus merely become platform caveats?

# Assumptions Being Validated

 - Tauri is sufficient because MVP is a desktop coding control center, not a browser automation or rich artifact rendering product.
 - System WebView differences do not block MVP UI and text-like preview workflows on required platforms.
 - Launch platform scope can be narrowed intentionally without weakening the product thesis.
 - Backend boundaries should remain portable enough to revisit Chromium/Electron after MVP.

# Investigation Plan

 - Propose and justify an MVP platform matrix with mandatory, optional, and deferred platforms.
 - Review Tauri platform requirements, WebView engine versions, updater/package implications, keychain support, and subprocess/process-tree support.
 - Validate representative MVP screens and workflows on each mandatory platform using a minimal UI/prototype or equivalent evidence.
 - Check text-like artifact rendering for text, Markdown, logs, diffs, generated-file links, and settings disclosure copy.
 - Record platform-specific limitations in focus behavior, clipboard/text selection, keyboard shortcuts, file dialogs, local file access, process supervision, and accessibility.
 - Define which failures require Electron/Chromium reconsideration and which are acceptable MVP caveats.

# Success Criteria

 - The MVP platform matrix is explicitly named.
 - Tauri WebView support is confirmed or rejected for every mandatory platform.
 - MVP UI and text-like artifact preview requirements have platform-specific evidence.
 - Platform-specific caveats are documented and classified as blocker or non-blocker.
 - Chromium/Electron reconsideration triggers are documented for post-MVP browser-heavy or rich-preview needs.
 - ADR-002 can remain finalized for MVP or be reopened with evidence.

# Deliverables

 - MVP platform matrix.
 - Tauri validation checklist and results.
 - Platform caveat list.
 - WebView and OS version notes.
 - Post-MVP Chromium/Electron trigger list.
 - Recommendation for ADR-002 validation status.

# ADRs Impacted

 - ADR-002 Desktop Application Shell.
 - ADR-017 Artifact Model, currently referenced by rollups.
 - ADR-018 Browser/Web Viewing, currently referenced by rollups.

# Decisions Unlocked

 - Which platforms must pass before MVP implementation planning proceeds.
 - Whether Tauri remains accepted for MVP on the named platform matrix.
 - Whether any launch platform should be deferred.
 - When Chromium/Electron should be reconsidered after MVP.

# Failure Conditions

 - The platform matrix remains unnamed.
 - Tauri fails basic MVP UI or text-like preview workflows on a mandatory platform.
 - Keychain integration, file access, or subprocess supervision fails on a mandatory platform without an accepted workaround.
 - Platform WebView behavior forces browser-heavy or rich-preview architecture into MVP.
 - Validation criteria are too vague to distinguish launch blockers from post-MVP caveats.
