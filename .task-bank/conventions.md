# Conventions

## Scope

- MVP scope comes from `plans/mvp/mvp-freeze.md`.
- Build order comes from `plans/implementation/recommended-build-order.md`.
- Do not introduce excluded MVP capabilities as visible settings or UI.

## Architecture

- Keep app authority in backend-owned modules.
- Keep UI modules thin and command-driven.
- Store app-owned records independently of runtime IDs.

## Verification

- Prefer `npm test` for JavaScript smoke tests while the first scaffold is
  being established.
- Record blocked native Tauri verification explicitly when local Rust tooling
  is unavailable.
