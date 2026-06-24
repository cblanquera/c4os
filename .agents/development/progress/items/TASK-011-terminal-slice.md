# TASK-011 Terminal Slice

Status: verified
Updated: 2026-06-24

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-011`
- Planned predecessor:
  `.agents/development/progress/items/TASK-010C-artifact-preview-type-rendering.md`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Replace Terminal mock behavior with backend-owned user terminal and agent
command terminal surfaces while preserving the accepted r04 shell and existing
Browser / Files / Terminal right-panel tab contract.

## Required Scope

- Preserve TASK-005 chat UX, TASK-005A scoped update patterns, TASK-006
  provider/model boundaries, TASK-007 session ownership, TASK-008 Files/File
  Editor authority, TASK-009 product-owned artifact preview records, TASK-010
  project-scoped Browser records, TASK-010A editable Browser address bar
  behavior, TASK-010B public-page native Wry Browser host behavior, and
  TASK-010C typed artifact preview rendering.
- Keep Terminal behavior inside the existing right-panel Terminal tab.
- Keep user terminal and agent command terminal behavior product-owned and
  distinct.
- Do not render Browser, Files, or artifact preview content through Terminal.
- Do not add a new route, panel, alternate layout, settings abstraction,
  React/Preact/JSX, or bundler.

## Required Verification

- Terminal tab behavior is backend-owned and session-scoped.
- User terminal and agent command terminal surfaces remain distinct.
- Terminal does not render Browser, Files, or artifact preview content.
- Browser, Files, artifact preview, and session switching behavior from
  TASK-005 through TASK-010C still holds.

## Outputs

- Extended product-owned `TerminalState` with distinct `userTerminal`,
  `agentTerminal`, and persisted terminal action records while preserving the
  existing `output`, `title`, and `summary` compatibility fields.
- Added backend-owned terminal lifecycle commands for session-scoped PTY
  start, input, resize, stop, and `c4os://terminal-output` event delivery
  through a C4OS-owned `portable-pty` module.
- Updated the Tauri connector to expose product-owned terminal session, input,
  resize, stop, and output-listener methods while keeping the prior agent
  command execution surface distinct.
- Replaced the rejected command-form/split-pane Terminal UI with an
  `@xterm/xterm` transcript host inside the existing right-panel Terminal tab.
- Restored the distinct bottom agent command transcript while keeping user
  typing in the xterm user terminal surface.
- Vendored the minimal xterm browser assets under `frontend/vendor/xterm/` so
  the app keeps the current no-bundler frontend contract.
- Preserved Browser, Files, artifact preview, chat, and session switching
  behavior inside the accepted shell.

## User Review Feedback

On 2026-06-24, user review confirmed the user terminal command path worked but
rejected the command-form/split-pane UI direction. Proofs confirmed that the
accepted replacement is `@xterm/xterm` for the terminal renderer plus
`portable-pty` behind C4OS-owned Tauri commands/events:
`.agents/specs/mvp/poc/task-011-terminal-inline-ui.md`,
`.agents/specs/mvp/poc/task-011-xterm-renderer.md`,
`.agents/specs/mvp/poc/task-011-portable-pty.md`, and
`.agents/specs/mvp/poc/task-011-xterm-pty-bridge.md`.

## Deferred

- Approval-policy hardening remains TASK-016.
- Settings, plugin, skill, and MCP records remain TASK-012.
- Prompt-driven agent command execution is intentionally not part of TASK-011.
  The bottom "Agent command terminal" is verified as a product-owned,
  read-only output surface for this slice. A follow-up must wire C4OS prompt /
  runtime command requests to backend-owned command execution, approval
  policy, action/audit records, and bottom-pane output before prompts can run
  commands such as `ls`, `npm test`, or project scripts.

## Verification Run

- `node --test --test-concurrency=1 tests/server/task-011-terminal.test.js`
  passed: 2 tests, 2 pass.
- `node --test --test-concurrency=1 tests/frontend-task-011.test.js`
  passed: 1 test, 1 pass.
- `node --test --test-concurrency=1 tests/frontend-connectors.test.js`
  passed: 7 tests, 7 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 36 tests, 36 pass.
- `node --test --test-concurrency=1 tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js
  tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js tests/frontend-task-010B.test.js
  tests/frontend-task-010C.test.js tests/frontend-task-011.test.js` passed:
  63 tests, 63 pass.
- `npm test` passed: 114 tests, 114 pass.
- `npm run acceptance:mvp` could not run because `scripts/mvp-acceptance.js`
  is not present in this repository.
- Follow-up after user review: `node --test --test-concurrency=1
  tests/frontend-r04.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-007.test.js
  tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js tests/frontend-task-010B.test.js
  tests/frontend-task-010C.test.js tests/frontend-task-011.test.js` passed:
  56 tests, 56 pass.
- Follow-up after second user review restored vertical resize for the bottom
  agent command terminal and added real xterm click/type verification. The same
  regression command passed: 57 tests, 57 pass.
- Follow-up after third user review added visible typing coverage for real
  xterm when `write_terminal_input` succeeds but no `c4os://terminal-output`
  echo arrives. The UI now locally echoes only after a short no-output fallback
  window, preserving PTY-owned echo when backend output events arrive. The same
  regression command passed: 57 tests, 57 pass.
- Follow-up after fourth user review stopped seeding the interactive user
  terminal with stale saved/mock transcript output whenever the live terminal
  session API is available, and added an explicit xterm message when PTY startup
  fails. The impacted regression command passed: 58 tests, 58 pass.
- Follow-up after fifth user review proved the backend PTY can execute `ls`,
  then fixed frontend event filtering so terminal-output events are accepted
  when the backend resolves an active session id but the frontend workspace
  session id is not hydrated. The impacted regression command passed: 59 tests,
  59 pass.
- User requested reverting the later no-live-terminal-backend blocking behavior
  so the user terminal returns to the prior locally echoing state while the
  PTY/event-output path is investigated separately.
- Follow-up after manual QA gap review added a regression for the exact
  `#new-session` boundary: a trusted workspace with no chat session now resolves
  a workspace-scoped Terminal PTY identity instead of requiring a C4OS chat
  session. The frontend regression now opens Terminal from `#new-session` with
  an empty `sessionId` and rejects the old "C4OS session is required" failure.
- Added `npm run backend:run:qa` as a QA-only launch path with an isolated
  `C4OS_SESSION_STORE`, and updated `.agents/workflows/manual-qa.md` to warn
  that direct `.app` wrapper launches must be checked against the current
  binary before they are accepted as evidence.
- Manual QA on 2026-06-24 used `npm run backend:run:qa`, confirmed current
  desktop URL `tauri://localhost/index.html?v=task-011b`, opened the real
  folder-backed `mcp` workspace, switched to the existing right-panel Terminal
  tab, observed the top user terminal shell prompt plus the distinct bottom
  "Agent command terminal", typed `ls`, and observed real folder output
  including `LICENSE`, `README.md`, `bin.js`, `docs`, and `node_modules`.
- Latest verification: `cargo test --manifest-path backend/Cargo.toml --
  --test-threads=1` passed: 38 tests, 38 pass. `npm test` passed: 121 tests,
  121 pass. `git diff --check` passed.
