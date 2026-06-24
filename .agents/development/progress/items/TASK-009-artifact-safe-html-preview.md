# TASK-009 Artifact And Safe HTML Preview Slice

Status: accepted
Updated: 2026-06-23

## Review

- User approved the TASK-009 artifact preview boundary on 2026-06-23.
- No commit was requested after approval.

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-009`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-008-files-slice.md`
- Requirements: `.agents/specs/mvp/requirements.md` `REQ-009`
- Acceptance: `.agents/specs/mvp/acceptance.md` `AC-009`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Replace artifact/preview mocks with C4OS product-owned artifact records and
safe rendering for generated or untrusted HTML inside the existing right-panel
Browser/Preview tab.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-005 chat UX, TASK-005A scoped DOM update behavior, TASK-006
  provider/model boundaries, TASK-007 session ownership, and TASK-008 trusted
  Files/File Editor authority.
- Render generated or untrusted HTML previews in the existing Browser tab only.
- Do not add an Artifacts tab, route, panel, alternate layout, React/Preact/JSX,
  bundler, settings abstraction, or Terminal-tab artifact rendering.
- Keep Browser browsing behavior, Terminal execution behavior, extensions,
  local memory, advanced approvals, and audit-log hardening mocked beyond the
  artifact preview boundary.

## Outputs

- Added C4OS-owned artifact records to workspace/session payloads.
- Added session-owned artifact preview creation behind the backend command
  boundary.
- Updated persistent session records so Browser preview state and artifact
  records restore together when sessions switch.
- Updated the frontend connector payload contract to include artifact records
  and artifact preview creation.
- Rendered active HTML artifact previews inside the existing Browser tab using
  a sandboxed data-document iframe.
- Kept the right-panel tab list limited to Browser, Files, and Terminal.
- Kept Terminal output separate from artifact preview rendering.
- Updated older frontend test fixtures to include the new top-level artifact
  payload field without changing accepted UI behavior.

## Deferred

- Full Browser browsing behavior remains TASK-010.
- Terminal execution behavior remains TASK-011.
- Extensions, local memory, advanced approvals, and audit-log hardening remain
  deferred to their later slices.
- This slice does not claim complete browser security hardening beyond isolated
  generated/untrusted HTML artifact preview rendering.

## Verification Run

- `cargo test --manifest-path backend/Cargo.toml task_009 -- --test-threads=1`
  passed: 1 test, 1 pass.
- `node --test tests/frontend-task-009.test.js` passed: 4 tests, 4 pass.
- `node --test tests/frontend-r04.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js
  tests/frontend-task-009.test.js tests/frontend-connectors.test.js` passed:
  70 tests, 70 pass.
- `node --test tests/server/task-005-openrouter-chat.test.js
  tests/server/task-006-provider-models.test.js
  tests/server/task-007-runtime-sessions.test.js
  tests/server/task-008-files.test.js` passed: 11 tests, 11 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1` passed:
  30 tests, 30 pass.
- `npm test` passed: 93 tests, 93 pass.
- `rustfmt --check backend/src/commands.rs backend/src/lib.rs
  backend/src/mock_data.rs backend/src/runtime_sessions.rs` passed.
- `git diff --check` passed.
