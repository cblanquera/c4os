# item-003: Implement Keychain Credential Store

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Store OpenRouter keys only in OS keychain or platform credential storage and
persist only stable credential references in app metadata.

## Inputs

- `plans/implementation/tasks.md` TASK-003
- `plans/acceptance/openrouter-integration.md`
- `plans/specs/security-specification.md`
- `plans/acceptance/telemetry-and-diagnostics.md`

## Outputs

- Credential store interface.
- macOS keychain implementation.
- Provider credential-reference metadata API.
- Failure handling when credential storage is unavailable.
- Tests proving raw keys are absent from SQLite-visible records.

## Constraints

- Do not persist raw keys in SQLite, project files, env files, logs, or custom
  app vaults.
- Provider setup must fail if OS keychain storage fails.
- Credential update/revoke blocking while a session is running belongs to
  item-005, but item-003 should expose primitives that make it possible.

## Acceptance Criteria

- Saving a provider key returns a stable credential reference.
- Raw keys are absent from settings and adapter references.
- Fake-store tests prove failure handling.
- macOS implementation uses platform credential storage.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-002

## Notes

- Verified on 2026-06-11 with fake-store tests, a macOS Keychain round-trip
  smoke test using a dummy key, Node tests, and native Tauri build.
