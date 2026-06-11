# item-020: TASK-020 Build Session Transcript And Runtime State UI

## Status

verified

## Objective

Render append-only transcript messages, runtime state, failure states, and the
one-active-session flow.

## Dependencies

- item-010
- item-002

## Deliverables

- Session screen projection.
- Transcript rendering data.
- Runtime state indicator.
- Failure state display.
- One-active-session guard.

## Verification

- Rust tests for message append ordering and runtime states passed.
- JS smoke test for status surface passed.
- Native Tauri build passed.
