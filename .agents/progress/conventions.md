# Conventions

## Scope

- MVP scope comes from `.agents/archive/planning-corpus/mvp/mvp-freeze.md`.
- Durable records live under `.agents/specs/c4os-mvp/`.
- Active execution state lives under `.agents/progress/`.
- Build order comes from `.agents/archive/planning-corpus/implementation/recommended-build-order.md`.
- Do not introduce excluded MVP capabilities as visible settings or UI.
- Do not add new updates to `.task-bank/`.

## Architecture

- Keep app authority in backend-owned modules.
- Keep UI modules thin and command-driven.
- Store app-owned records independently of runtime IDs.

## Verification

- Prefer focused automated tests for the changed behavior.
- Record blocked native Tauri verification explicitly when local Rust tooling
  is unavailable.
- Mark items verified only after the stated verification actually passed.

