# item-001: Scaffold Tauri App And Backend Boundary

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Create the minimal Tauri-compatible application scaffold and backend command
boundary used by MVP features.

## Inputs

- `.agents/archive/planning-corpus/implementation/tasks.md` TASK-001
- `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
- `.agents/archive/planning-corpus/adr/002-desktop-application-shell.md`
- `.agents/archive/planning-corpus/acceptance/telemetry-and-diagnostics.md`
- `.agents/archive/planning-corpus/acceptance/settings-and-configuration.md`

## Outputs

- Frontend shell.
- Backend command registration/invocation pattern.
- Local diagnostics initialization with no telemetry upload path.
- Smoke test covering backend command invocation.

## Constraints

- Do not add product telemetry, crash upload, support bundle upload, or
  non-MVP settings.
- Keep the backend command boundary testable without a native shell so later
  Tauri command handlers can reuse the same core behavior.

## Acceptance Criteria

- App scaffold exists.
- Backend command can be invoked from the UI path.
- Local diagnostics initialize as local-only.
- No product telemetry request path exists.

## Verification

- `npm test`
- `npm run tauri -- build`

## Dependencies

- DEC-001
- DEC-002

## Notes

- JavaScript smoke tests, static build, dependency audit, and Tauri CLI version
  checks passed on 2026-06-11.
- Native Tauri build passed on 2026-06-11 after Rust/Cargo was installed.
