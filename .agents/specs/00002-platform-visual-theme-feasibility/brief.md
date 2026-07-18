# Brief

## User Goal

Preserve and confirm the feasibility question introduced by `.agents/references/00004-platform-visual-and-theme-contract.md` so the project does not lose its place.

## Accepted Intent

C4OS follows the host operating system's visual language and live light/dark preference. The reconstructed r012 scope has no manual C4OS-only theme override. Semantic styling, platform adaptation, and accessible state presentation remain accepted product direction.

## Scope

- Tauri 2 platform and initial-theme detection.
- Live system theme changes without state loss.
- Startup sequencing that avoids showing the wrong scheme.
- Semantic-token inputs, system colors, accent, contrast, reduced motion, fonts, controls, and scrollbars.
- Native application menus, Settings shortcuts, window decorations, titlebar integration, dragging, and platform terminology.
- macOS, Windows, and Linux behavior, including explicit Linux desktop/webview fallbacks.
- Target-specific visual and technical acceptance evidence.

## Non-Goals

- Redesigning the C4OS information architecture or interaction model.
- Changing `wireframes/r012-cleanup/` or the pending r013 revision.
- Implementing the production theme system during research setup.
- Reopening runtime, adapter, policy, or model-capability decisions from Frozen spec `00001`.
- Treating macOS-only evidence as proof for Windows or Linux.
- Promising pixel-identical controls across different operating systems or Linux desktop environments.

## Source Material

- [Usability and interface Context](../../context/usability-and-interface.md)
- [Platform visual and theme contract](../../references/00004-platform-visual-and-theme-contract.md)
- [Launch and Settings contract](../../references/00007-launch-and-settings-contract.md)
- [Reconstruction fixtures and acceptance](../../references/00008-reconstruction-fixtures-and-acceptance.md)
- [Runtime and session architecture](../../context/runtime-session-architecture.md)
- `wireframes/r013-capability-aware-chat/`, which remains grayscale and explicitly defers native menu/window behavior.

## Deliverable

A source-backed and proof-backed platform matrix that identifies supported native inputs, required fallbacks, startup and event semantics, window/menu choices, acceptance checks, and feature/target gates. Accepted reusable results may then refine the usability Context without weakening its product intent.
