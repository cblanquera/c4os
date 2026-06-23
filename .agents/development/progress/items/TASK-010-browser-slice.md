# TASK-010 Browser Slice

Status: verified
Updated: 2026-06-23

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-010`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-009-artifact-safe-html-preview.md`
- Requirements: `.agents/specs/mvp/requirements.md` `REQ-010`, `REQ-013`,
  `REQ-016`
- Acceptance: `.agents/specs/mvp/acceptance.md` `AC-010`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Replace Browser mock behavior behind the accepted right-panel Browser tab with
MVP Browser state for project-scoped profiles, trusted local file opening,
public browsing, request-scoped agent browsing, and C4OS-owned Browser action
records.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-005 chat UX, TASK-005A scoped DOM updates, TASK-006
  provider/model boundaries, TASK-007 per-session ownership, TASK-008
  trusted-root Files/File Editor authority, and TASK-009 artifact preview
  records.
- Keep Browser behavior inside the existing right-panel Browser/Preview tab.
- Do not add Browser routes, panels, tabs, alternate layouts, React/Preact/JSX,
  a bundler, or settings abstractions.
- Keep downloads out of scope.
- Do not treat artifact preview as general browsing.
- Keep Terminal execution behavior, extensions, local memory, advanced
  approvals, and audit-log hardening mocked beyond the Browser boundary.

## Outputs

- Added session-owned Browser profile state fields to `BrowserState` while
  keeping existing Browser payload compatibility.
- Added C4OS-owned `BrowserActionRecord` records on each session.
- Added backend `open_browser` command support for public URLs and trusted
  project-local files.
- Routed local Browser file opening through the existing trusted-root path
  authority, including traversal and `.git` rejection.
- Added request-scoped agent Browser gating; agent browsing requires an
  explicit clear request and records an action.
- Updated `open_browser_preview(session_id)` so a known session returns
  product-owned Browser state while the no-session compatibility path remains
  the old mock inventory read.
- Updated the frontend connector inventory and Tauri connector with
  `openBrowser`.
- Rendered public and local Browser state in the existing Browser tab with a
  Browser frame, separate from TASK-009 artifact preview frames.
- Preserved the right-panel tab list as Browser, Files, and Terminal.
- Preserved Files and Terminal tab behavior in Browser session-switch tests.

## Deferred

- Downloads remain out of scope.
- Full browser security hardening is not claimed beyond the TASK-010 Browser
  boundary and the already accepted TASK-009 artifact sandbox boundary.
- Terminal execution behavior remains TASK-011.
- Extensions, local memory, advanced approvals, and audit-log hardening remain
  deferred to later slices.

## Verification Run

- `cargo test --manifest-path backend/Cargo.toml task_010 -- --test-threads=1`
  passed: 3 tests, 3 pass.
- `node --test tests/frontend-task-010.test.js` passed: 4 tests, 4 pass.
- `node --test tests/frontend-r04.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js
  tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-connectors.test.js` passed: 74 tests, 74 pass.
- `node --test tests/server/task-005-openrouter-chat.test.js
  tests/server/task-006-provider-models.test.js
  tests/server/task-007-runtime-sessions.test.js
  tests/server/task-008-files.test.js` passed: 11 tests, 11 pass.
- `npm test` passed: 97 tests, 97 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 33 tests, 33 pass.
- `rustfmt --check backend/src/commands.rs backend/src/lib.rs
  backend/src/mock_data.rs backend/src/runtime_sessions.rs` passed.
- `git diff --check` passed.
