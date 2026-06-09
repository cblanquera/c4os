# Objective

Determine whether Tauri is sufficient for the MVP and future browser/artifact requirements, or whether Electron/Chromium should be preferred.

# Context

Specs prefer Tauri for footprint, Rust backend, and local-first posture. Reviews warn that browser viewing, screenshots, artifact previews, and automation may favor Electron or Chromium.

# Questions To Answer

 - Which platforms must be supported at launch?
 - Are Tauri WebViews sufficient for MVP UI and basic artifact rendering?
 - Do future browser automation or screenshot features require Chromium consistency?
 - Can browser content be isolated from privileged backend IPC in Tauri?
 - What migration cost would switching from Tauri to Electron create later?

# Hypothesis

Tauri is acceptable for the MVP if browser automation and rich artifact rendering are excluded, but future browser-heavy requirements may require a Chromium strategy.

# Investigation Plan

 - Compare Tauri and Electron against MVP UI needs.
 - Validate basic rendering needs for text, Markdown, logs, diffs, and settings.
 - Review security boundary options for remote content.
 - Document future browser/artifact features that would force reconsideration.
 - Estimate migration risk.

# Success Criteria

 - Desktop shell recommendation is confirmed or challenged for MVP.
 - Browser/artifact requirements that require Chromium are identified.
 - Migration risks are documented.

# Decisions Unlocked

 - ADR-002: Desktop Application Shell.
 - ADR-018: Browser/Web Viewing.
 - ADR-017: Artifact Model.

# Estimated Effort

2 to 4 engineering days.

