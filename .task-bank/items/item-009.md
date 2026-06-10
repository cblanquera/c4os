# item-009: Build Hardened OpenCode Runtime Launcher

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Launch OpenCode sessions through app-owned config, selected model, credential
reference, and project root.

## Inputs

- `plans/validation/FINDING-001-opencode-integration-surface.md`
- `plans/adr/003-agent-runtime-strategy.md`
- `plans/acceptance/agent-execution.md`
- `plans/acceptance/openrouter-integration.md`

## Outputs

- Runtime launcher.
- Session lifecycle state.
- Runtime process supervision.
- Adapter reference storage.

## Constraints

- Launch settings are app-owned adapter settings, not OpenCode config
  compatibility.
- Runtime launch must require a registered project and provider-ready session.
- The process must be supervised by the backend for later stop handling.

## Acceptance Criteria

- Runtime can start for a registered project and provider-ready session.
- Launch failure is captured as app-owned state.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-005
- item-007

## Notes

- Verified on 2026-06-11 with supervised spawn, invalid project root, missing
  command, Node tests, and native Tauri build checks.
