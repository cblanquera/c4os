# item-033: TASK-032 Expose Project Selector Capability Status

## Status

verified

## Objective

Expose project-selector capability state through the app status surface so the
UI can distinguish the accepted one-active-project workflow from postponed full
project-management behavior.

## Dependencies

- item-031
- item-032

## Deliverables

- App status fields for project-selector availability.
- Status fields for one-active-project behavior.
- Status fields proving postponed project-management controls remain
  unavailable.
- Shell display of the selector state.

## Verification

- JS status command tests.
- Static build.
- `npm test` passed.
- `npm run build` passed.
- `npm run tauri -- build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
