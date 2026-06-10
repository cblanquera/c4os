# Objective

Validate Tauri WebView for MVP UI and document post-MVP Chromium triggers.

# Context

Tauri is accepted for MVP because the MVP excludes browser panels, browser automation, screenshots, DOM extraction, active HTML rendering, and rich artifact previews. Reviews still warn that post-MVP browser viewing, screenshots, rich previews, and automation may favor Electron or Chromium.

# Questions To Answer

 - Which platforms must be supported at launch?
 - Are Tauri WebViews sufficient for MVP app UI and text-like artifact previews?
 - Which future browser automation, screenshot, local app preview, generated HTML preview, or rich artifact features require Chromium consistency?
 - Can future browser content be isolated from privileged backend IPC in Tauri?
 - What migration cost would switching from Tauri to Electron create later?

# Hypothesis

Tauri is acceptable for MVP if its WebView supports the basic app UI and text-like previews. Future browser-heavy or rich-preview requirements may require a Chromium strategy.

# Investigation Plan

 - Validate Tauri against MVP UI needs.
 - Validate basic rendering needs for text, Markdown, logs, diffs, and settings.
 - Document post-MVP browser/artifact features that would force Chromium reconsideration.
 - Review future security boundary options for remote content.
 - Estimate migration risk.

# Success Criteria

 - Tauri WebView is confirmed or challenged for MVP UI and text-like previews.
 - Post-MVP browser/artifact requirements that require Chromium are identified.
 - Migration risks are documented.

# Decisions Unlocked

 - ADR-002: Desktop Application Shell.
 - ADR-018: Browser/Web Viewing.
 - ADR-017: Artifact Model.

# Estimated Effort

2 to 4 engineering days.
