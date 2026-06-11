# item-031: TASK-030 Build Project Selector State Foundation

## Status

verified

## Objective

Provide a backend projection for listing registered local Git projects and
persisting exactly one selected active project.

## Dependencies

- item-006
- item-025

## Deliverables

- Registered-project list API.
- Selected-project persistence API.
- Project selector projection with exactly one active project marker.
- Tests proving postponed project-management controls stay unavailable.

## Verification

- Registered-project listing tests.
- Selected-project persistence tests.
- Missing-project rejection tests.
- Excluded project-management controls tests.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm test` passed.
- `npm run build` passed.
- `npm run tauri -- build` passed.
