# item-005: Enforce Active-Session Credential Reference Rules

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Capture the OpenRouter credential reference and model at session start, and
block key update/revoke while a session is running.

## Inputs

- `plans/validation/FINDING-004-openrouter-runtime-verification.md`
- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/model-providers.md`
- `plans/acceptance/sessions.md`

## Outputs

- Session credential-reference capture.
- Running-session key update/revoke guard.
- Tests for stable credential reference behavior.

## Constraints

- Running sessions use the credential reference and model captured at session
  start.
- Provider update or revoke must fail while any session is running or waiting
  for approval.

## Acceptance Criteria

- Running sessions keep their starting credential reference.
- Update/revoke attempts are blocked until the session stops or completes.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-004
- item-002

## Notes

- Verified on 2026-06-11 with stable session credential/model capture tests,
  provider update guard tests, Node tests, and native Tauri build.
