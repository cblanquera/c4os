# item-043: TASK-042 Build Archived Session Delete

## Status

verified

## Objective

Implement the accepted `archived_session_delete_only` tier.

## Scope

Add explicit archived-session deletion for archived, unpinned sessions only.
Deletion must remove the selected archived session and app-owned records linked
only to that session, while preserving protected sessions and unrelated project
state.

## Dependencies

- item-030
- item-041
- item-042

## Acceptance Criteria

- Archived, unpinned sessions can be deleted explicitly.
- Active, latest, running, pending-approval, and pinned sessions cannot be
  deleted.
- Deletion removes the deleted session from the project session catalog.
- Selection falls back to a remaining valid project session after deletion.
- Delete does not remove another session's records.
- Delete does not remove project files, provider credentials, or artifact files
  outside app-managed artifact storage.
- App status reports `archived_session_delete_only` and keeps automatic
  cleanup, quotas, memory, import, and round-trip compatibility unavailable.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml archived_session` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml deletes_archived_unpinned_session_records_only` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.
