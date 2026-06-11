# item-030: TASK-029 Build Session Rename Pin And Archive Foundation

## Status

verified

## Objective

Add durable session metadata controls for rename, pin, and archive while
preserving transcript append-only semantics and deferring hard delete until
retention/delete semantics are specified.

## Dependencies

- item-028

## Deliverables

- Migration for session pin/archive metadata.
- Rename session title API.
- Pin/unpin session API.
- Archive/unarchive session API.
- Tests proving metadata changes do not edit/delete messages.

## Verification

- Migration tests.
- Rename/pin/archive tests.
- Regression test that message records remain append-only after session
  metadata updates.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm test` passed.
- `npm run build` passed.
- `npm run tauri -- build` passed.
