# TASK-013 Concurrency And Restart/Resume Slice

Status: verified
Updated: 2026-06-27

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-013`
- Predecessor:
  `.agents/development/progress/items/TASK-012-settings-ia-extension-records.md`
- MVP requirements: `REQ-005`, `REQ-006`, `REQ-010`, `REQ-011`, `REQ-015`,
  `REQ-016`
- MVP acceptance: `AC-005`, `AC-006`, `AC-010`, `AC-011`, `AC-019`

## Goal

Implement concurrent session/run isolation across trusted folders and restart/
resume behavior for the C4OS-owned records already unlocked by prior MVP
slices.

## Required Scope

- Preserve TASK-011, TASK-011A, and TASK-011B Terminal/chat behavior.
- Preserve TASK-012 Settings IA order and inert app-owned plugin, skill, and
  MCP records.
- Keep runtime, session, run, transcript, provider/model, artifact, Browser,
  Files, Terminal, and extension record ownership inside C4OS records.
- Do not implement extension invocation, skill/plugin loading, runtime tool
  gateway, approval-policy hardening, memory, audit hardening, TASK-016
  security policy, or parser expansion.
- Do not renumber frozen MVP tasks.

## Outputs

- Added trusted-workspace root identity to C4OS workspace and session records.
- Session creation now derives workspace id, project name, root path, Files
  state, Browser profile id, and Terminal cwd from the active trusted workspace
  descriptor.
- Session lists, active session resolution, and workspace reload payloads now
  filter to the active trusted workspace instead of matching only by project
  name or global store order.
- Terminal command execution now uses the persisted session workspace root, so
  commands for one session do not run in a different currently active folder.
- File read/save commands now use the requested session root when a session id
  is supplied and persist the resulting Files state back to that session.
- Browser local-target resolution now uses the requested session root and
  preserves session-owned Browser action records.
- Added focused Rust and Node regression coverage for concurrent trusted-folder
  isolation, restart/resume payload filtering, and session-root Browser/File/
  Terminal command behavior.

## Verification

- `cargo test --manifest-path backend/Cargo.toml task_013 --
  --test-threads=1` passed: 4 tests.
- `node --test --test-concurrency=1
  tests/server/task-013-concurrency-resume.test.js` passed: 2 tests.
- Focused regression command passed after rerunning frontend harnesses with
  loopback permission: `tests/server/task-006-provider-models.test.js`,
  `tests/server/task-007-runtime-sessions.test.js`,
  `tests/server/task-011-terminal.test.js`,
  `tests/server/task-012-extension-records.test.js`,
  `tests/server/task-013-concurrency-resume.test.js`,
  `tests/frontend-task-006.test.js`, `tests/frontend-task-011.test.js`,
  `tests/frontend-task-011B.test.js`, and `tests/frontend-task-012.test.js`.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1` passed:
  45 tests.
- `npm test` passed: 146 tests.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.
- Built-app manual QA with Computer Use on 2026-06-27 passed after refreshing
  the stale debug `.app` wrapper executable from the current
  `backend/target/debug/c4os-backend` binary. Credential was supplied through
  environment stdin, not recorded in the QA notes. The app URL showed
  `tauri://localhost/index.html?v=task-013`.

## Notes

- A first sandboxed focused regression attempt cancelled frontend suites because
  the browser harness could not listen on `127.0.0.1` (`EPERM`). The frontend
  TASK-006, TASK-011, TASK-011B, and TASK-012 regressions passed after rerunning
  with loopback permission.
- Manual review found the left-sidebar Projects `+` button did not open the
  trusted-folder picker, then replaced the current project after the picker was
  wired. The fix reuses the existing `data-open-workspace` path, with the
  sidebar button explicitly prepending the newly opened project above the
  existing project list while App Start still replaces the initial empty/mock
  state. Focused verification:
  `node --test --test-concurrency=1 tests/frontend-task-004.test.js` passed.
- Manual review then found two different trusted folders with the same basename
  such as `~/server/projects/payable` and
  `~/server/projects/shoppable/payable` still collapsed into one Projects row.
  The backend now exposes project record id/root path from the trusted
  descriptor, and the frontend merge key uses id/root path before falling back
  to name. Focused verification reran TASK-004, TASK-007, TASK-011B, TASK-004
  server, TASK-013 server, and the full backend test suite.
- Manual review then found active chat creation/switching could still target
  the wrong same-name project because clicking an existing Projects row only
  changed frontend display state. Project rows now carry project id/root path,
  switching a saved trusted-root project reopens that root through
  `open_workspace(path)`, and frontend session/composer keys use project
  id/root path before falling back to display name. Focused verification reran
  TASK-004, TASK-007, TASK-011B, and the full backend test suite.
- Built-app manual QA opened `/Users/cblanquera/server/projects/payable` and
  `/Users/cblanquera/server/projects/shoppable/payable` as two same-name
  trusted folders. The sidebar `+` added the second `payable` above the first
  instead of replacing it. Distinct chats, `C4OS QA first payable root` and
  `C4OS QA second payable root`, appeared under their correct project rows.
  After restart with the same isolated session store, reopening both folders
  restored each chat under the correct `payable` row.
- Follow-up QA found that reopening an existing same-name project, including
  the path used before creating a new chat, could move that project row to the
  top. The frontend project merge now prepends only newly added trusted
  folders and updates existing project records in place, so new chats do not
  reorder the Projects list. Focused verification:
  `node --test --test-concurrency=1 tests/frontend-task-004.test.js` and
  `node --test --test-concurrency=1 tests/frontend-task-011B.test.js` passed.
- Built-app manual QA also verified OpenRouter-backed model context by running
  chat with `google/gemini-2.5-flash-lite`, and verified the preserved Terminal
  contract: the Terminal tab kept the top user PTY separate from the bottom
  read-only Agent command terminal, and `run ls` updated only the bottom Agent
  command output.
- Manual QA exposed that `npm run backend:build` updates
  `backend/target/debug/c4os-backend`, but the existing debug
  `backend/target/debug/C4OS.app/Contents/MacOS/C4OS` wrapper can remain stale.
  For built-app QA, confirm the wrapper binary contains the current cache token
  or refresh it from the rebuilt backend binary before using Computer Use.
- Extension records remain app-owned and inert; runtime invocation and gateway
  behavior remain deferred.
- Local memory, broad action records, and audit records remain deferred to
  TASK-014.

## Next Step

Continue with `.agents/specs/mvp/tasks.md` `TASK-014` for local memory, action
records, and audit records. Preserve the TASK-013 trusted-folder isolation and
restart/resume behavior, keep TASK-012 extension records inert until explicit
enablement, and keep approval/security hardening deferred to TASK-016.
