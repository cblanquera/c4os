# ADR-002: Desktop Application Shell

Status: Provisional.

## Context

The research and architecture specifications recommend Tauri for the desktop app because it has a smaller footprint, lower memory use, Rust backend compatibility, native system integration, and a better fit for local-first desktop tooling. Electron is acknowledged as more mature for Chromium consistency, Node integration, and extension ecosystem support.

The review flags a migration risk: the app also wants browser viewing, artifact rendering, screenshots, and possible automation. Those requirements may favor Electron or a controlled Chromium surface if Tauri WebView differences become material.

## Decision

Use Tauri for MVP unless runtime embedding, browser automation, or artifact rendering research identifies a strong reason to use Electron.

This decision is provisional.

## Alternatives Considered

 - Tauri desktop app.
 - Electron desktop app.
 - Native Swift/Kotlin/WinUI desktop clients.

## Alternatives That Should Be Considered

 - Tauri with an external controlled Chromium service for browser-heavy features.
 - Electron for MVP if browser/artifact consistency is more important than footprint.
 - Web app plus local daemon if team collaboration becomes central.

## Tradeoffs

Tauri provides a Rust-centered backend and a tighter local-first security posture, but system WebViews may vary by platform.

Electron provides consistent Chromium behavior and broader Node ecosystem compatibility, but increases memory, bundle size, and IPC hardening requirements.

Native platform stacks provide strong OS integration, but increase cross-platform cost and slow product iteration.

## Consequences

 - Backend service boundaries should remain portable enough to avoid trapping the product in Tauri-specific assumptions.
 - Browser and artifact rendering must be validated early.
 - Tauri capabilities should be used for frontend/backend boundary control if Tauri remains selected.

## Follow-Up Questions

 - Which launch platforms are required?
 - Do browser and artifact requirements need consistent Chromium?
 - Can OpenCode be controlled cleanly from a Rust backend?
 - Are Tauri WebView limitations acceptable for the product's first workflows?

## ADR Recommendation

Keep this ADR as provisional until Phase 0 validation answers browser, runtime, and platform questions.

