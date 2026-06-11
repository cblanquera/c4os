# item-022: TASK-022 Implement Stop, Retry, And Restart Recovery UX

## Status

verified

## Objective

Expose stop control, retry as a new appended action/status, and restart
recovery states without transcript mutation or pending approval replay.

## Dependencies

- item-012
- item-020

## Deliverables

- Stop button behavior projection.
- Retry-as-new-action behavior.
- Restart recovery banners/states.
- Minimized-window execution messaging.

## Verification

- Rust tests for stop controls, provider failure retry append, crash marker, and
  pending approval discard policy passed.
- JS smoke test for status surface passed.
- Native Tauri build passed.
