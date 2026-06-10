# item-002: Implement SQLite Schema And Migrations

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Create app-owned SQLite persistence for MVP projects, sessions, messages, tool
calls, approvals, artifacts, models, settings, diagnostics, and adapter
references.

## Inputs

- `plans/implementation/tasks.md` TASK-002
- `plans/specs/data-model-specification.md`
- `plans/mvp/mvp-freeze.md`
- `plans/acceptance/sessions.md`
- `plans/acceptance/security-and-permissions.md`

## Outputs

- SQLite migration system.
- MVP tables.
- Repository/store APIs for the first persistence paths.
- Restart persistence smoke tests.

## Constraints

- SQLite is canonical for app-owned MVP records.
- Runtime IDs must remain adapter metadata.
- Transcripts are append-only.
- Raw provider secrets must not be accepted into app metadata.

## Acceptance Criteria

- Tables initialize from a clean profile.
- Records survive app restart.
- Runtime IDs are stored only as adapter metadata.
- Raw key-shaped settings are rejected by the store API.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-001

## Notes

- The first API slice should cover migrations and core restart persistence,
  not provider credential storage. Credential storage belongs to item-003.
- Verified on 2026-06-11 with Rust migration tests, Node tests, static build,
  and native Tauri build.
