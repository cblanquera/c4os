# TASK-007 Runtime Adapter And Persistent Sessions Slice

Status: approved
Updated: 2026-06-23

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-007`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-006-provider-model-management.md`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Replace remaining mock session/runtime behavior behind the accepted
chat/session surfaces with C4OS-owned workspace, session, run, message, and
runtime-reference records.

## Required Scope

- Preserve the accepted r04 shell, TASK-005 chat UX, TASK-005A scoped DOM update
  patterns, and TASK-006 provider/model state boundaries.
- Keep Settings/provider/model records global across the app.
- Keep each chat session isolated for thread, Browser, Terminal, Files/File
  Editor, and selected model state.
- Ensure new-session draft choices do not mutate the current chat before the
  new chat is created.
- Ensure sending a prompt creates or appends to the correct session.
- Persist and restore enough C4OS session records to prove restart/resume for
  TASK-007 scope.
- Keep native runtime concepts behind a C4OS-owned adapter boundary and out of
  frontend/user-facing records.

## Outputs

- Added `backend/src/runtime_adapter.rs` as the C4OS-owned runtime adapter
  boundary used by prompt sends.
- Added `backend/src/runtime_sessions.rs` with C4OS-owned workspace, session,
  turn, run, message, and runtime-reference records.
- Added persistence for TASK-007 session records through a JSON session store,
  with `C4OS_SESSION_STORE` override support for tests.
- Replaced mock `create_session` behavior with backend-owned C4OS session
  creation.
- Replaced direct prompt sending with a session-targeted append path that
  records prompt and assistant messages, run records, runtime events, selected
  model, and session thread state.
- Added `load_session` so sidebar switching can restore persisted session-owned
  Browser, Terminal, Files/File Editor, thread, and model state.
- Kept provider/model records from TASK-006 as the authority for per-session
  model selection and wrote selected model changes back to C4OS session records.
- Updated the Tauri connector to pass `sessionId`, `project`, and new-session
  label through existing command paths.
- Updated the frontend state layer to support persistent session ids while
  preserving legacy in-memory session snapshots and accepted shell rendering.
- Kept local fallback session ids local-only so legacy mocks do not call
  persistent session loading.
- Added TASK-007 server and frontend tests for session isolation, append
  targeting, adapter boundary, restart/resume, and session-owned surface
  restore.

## Verification Run

- `node --test tests/server/task-007-runtime-sessions.test.js` passed.
- `node --test tests/frontend-task-007.test.js` passed.
- `node --test tests/frontend-task-006.test.js tests/frontend-task-007.test.js`
  passed.
- `node --test tests/frontend-connectors.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js` passed after connector contract updates.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1` passed:
  25 tests, 25 pass.
- `npm test` passed: 75 tests, 75 pass.
- `rustfmt --check backend/src/commands.rs backend/src/mock_data.rs
  backend/src/runtime_adapter.rs backend/src/runtime_sessions.rs
  backend/src/workspace.rs` passed.
- `git diff --check` passed.

## Still Out Of Scope

- Real Files content mutation.
- Browser behavior beyond per-session record restore.
- Terminal execution behavior.
- Extensions, plugins, skills, and MCP runtime behavior.
- Local memory.
- Artifacts.
- Advanced approval policy.
- Audit-log hardening.
- OS keychain credential persistence.

## Acceptance

TASK-007 was user-approved on 2026-06-23 after verification. TASK-008 may start
from this C4OS runtime/session authority boundary.
