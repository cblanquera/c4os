# item-029: TASK-028 Build Project-Scoped Session Selection

## Status

verified

## Objective

Allow a caller to open a selected session only when it belongs to the selected
project, returning transcript state through the existing session screen
projection.

## Dependencies

- item-028
- item-020

## Deliverables

- Project-scoped session selection API.
- Cross-project session rejection.
- Latest-session fallback when no explicit session is selected.
- Tests for owned selection, cross-project rejection, and missing sessions.

## Verification

- Rust unit tests for selected session ownership, latest fallback, missing
  session, and cross-project rejection passed.
- JS tests, frontend build, and native Tauri build passed.
