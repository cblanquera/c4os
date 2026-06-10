# item-004: Build OpenRouter Provider Setup

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Let users configure OpenRouter, select an MVP model, and see provider-ready
state with off-device context disclosure.

## Inputs

- `plans/implementation/tasks.md` TASK-004
- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/model-providers.md`

## Outputs

- Provider setup backend API.
- Provider readiness state.
- Selected model storage.
- Standing disclosure copy in the app shell.
- Provider authentication failure state.

## Constraints

- OpenRouter is the only MVP provider.
- Raw provider keys must not be displayed or persisted in app metadata.
- Stale or missing model metadata must not falsely downgrade readiness.

## Acceptance Criteria

- Valid credential can produce provider-ready state.
- Invalid or storage-failed setup is actionable and fail-closed.
- Missing/stale metadata does not falsely downgrade readiness.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-003

## Notes

- Verified on 2026-06-11 with provider setup unit tests, Node smoke tests, and
  native Tauri build.
