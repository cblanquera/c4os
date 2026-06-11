# ADR-002: Desktop Application Shell

Status: Finalized for MVP with macOS-first platform matrix. Post-MVP Chromium
strategy remains deferred.

## Context

The research and architecture specifications recommend Tauri for the desktop app because it has a smaller footprint, lower memory use, Rust backend compatibility, native system integration, and a better fit for local-first desktop tooling. Electron is acknowledged as more mature for Chromium consistency, Node integration, and extension ecosystem support.

The review flags a migration risk: future browser viewing, artifact rendering, screenshots, and possible automation may favor Electron or a controlled Chromium surface if Tauri WebView differences become material.

Accepted MVP scope excludes browser automation, browser panels, screenshots, DOM extraction, active HTML rendering, and rich artifact previews. MVP artifacts are text-like records and file links only.

## Decision

Use Tauri for MVP. Tauri WebView is sufficient for the MVP app UI and text-like artifact previews unless Phase 0 proves it cannot support basic app rendering.

Chromium, Electron, or an external controlled Chromium surface are post-MVP options for browser-heavy or rich-preview features.

MVP platform matrix:

 - Mandatory launch validation: macOS Apple Silicon on macOS 13 Ventura or
   newer.
 - Required before public MVP release: Windows 11 x64 with WebView2 Evergreen.
 - Optional before public MVP release: macOS Intel on macOS 13 Ventura or
   newer, when hardware is available.
 - Deferred: Linux x64 until post-MVP or explicit beta scope because WebKitGTK
   and distro packaging variability add support burden before the core desktop
   control-center thesis is validated.

## Alternatives Considered

 - Tauri desktop app.
 - Electron desktop app.
 - Native Swift/Kotlin/WinUI desktop clients.

## Alternatives That Should Be Considered

 - Tauri with an external controlled Chromium service for browser-heavy features.
 - Electron if browser/artifact consistency becomes more important than footprint after MVP.
 - Web app plus local daemon if team collaboration becomes central.

## Tradeoffs

Tauri provides a Rust-centered backend and a tighter local-first security posture, but system WebViews may vary by platform.

Electron provides consistent Chromium behavior and broader Node ecosystem compatibility, but increases memory, bundle size, and IPC hardening requirements.

Deferring Chromium keeps the MVP smaller and avoids prematurely accepting browser-content and rich-preview security obligations.

Native platform stacks provide strong OS integration, but increase cross-platform cost and slow product iteration.

## Consequences

 - Backend service boundaries should remain portable enough to avoid trapping the product in Tauri-specific assumptions.
 - MVP UI and text-like artifact preview rendering must be validated on macOS
   Apple Silicon before implementation is considered ready for release
   planning.
 - Windows 11 x64 compatibility validation must pass before public MVP
   release.
 - Browser and rich artifact rendering are post-MVP decision points.
 - Tauri capabilities should be used for frontend/backend boundary control if Tauri remains selected.

## Follow-Up Questions

 - Can OpenCode be controlled cleanly from a Rust backend?
 - Does Tauri WebView support the basic app UI and text-like artifact previews
   on macOS Apple Silicon once an app build exists?
 - Which post-MVP browser or rich-preview features would require Chromium consistency?

## ADR Recommendation

Validate Tauri WebView for MVP UI and text-like previews in Phase 0. Revisit Chromium only after MVP if browser panels, browser automation, screenshots, DOM extraction, generated HTML previews, or rich artifact rendering become validated needs.
