# item-028: TASK-027 Build Project Session Catalog Foundation

## Status

verified

## Objective

Provide a backend projection for listing multiple durable sessions attached to
one project while preserving the one-active-run invariant.

## Dependencies

- item-020
- item-025

## Deliverables

- Project-scoped session list API.
- Session catalog projection with latest-session marker.
- Active-run guard carried into catalog state.
- Tests proving multiple sessions can be listed without allowing multiple
  active runs.

## Verification

- Rust unit tests for multi-session listing, latest marker, active-run guard,
  and empty project state passed.
- JS tests, frontend build, and native Tauri build passed.
