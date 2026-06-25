# TASK-011A Agent Command Terminal Bridge

Status: verified
Updated: 2026-06-25

## Source

- Follow-up from `.agents/development/progress/items/TASK-011-terminal-slice.md`
- Progress manifest: `.agents/development/progress/manifest.md`
- MVP acceptance: `.agents/specs/mvp/acceptance.md` terminal acceptance

## Goal

Bridge explicit C4OS prompt requests for command execution into backend-owned
command execution and the bottom read-only Agent command terminal, while keeping
the top user terminal interactive and separate.

## Required Scope

- Preserve the accepted r04 shell and Browser / Files / Terminal right-panel
  tab contract.
- Preserve TASK-005 chat UX, TASK-005A scoped update patterns, TASK-006
  provider/model boundaries, TASK-007 session ownership, TASK-008 Files/File
  Editor authority, TASK-009 artifact preview records, TASK-010 Browser
  records, TASK-010A Browser address behavior, TASK-010B native Wry Browser
  host behavior, TASK-010C typed artifact previews, and TASK-011 xterm user
  terminal plus distinct bottom agent command terminal.
- Run only explicit command prompts such as `run ls` or `execute ls` in this
  slice.
- Use the active trusted workspace root as command cwd.
- Record command, cwd, status, exit code, output, session id, and terminal kind
  through existing terminal action records.
- Keep TASK-016 approval-policy hardening deferred.

## Outputs

- Added explicit command request detection in the C4OS prompt path.
- Reused the existing backend-owned trusted-workspace command runner for agent
  command terminal execution.
- Persisted explicit prompt command results to `agentTerminal` and
  `TerminalActionRecord` with `terminalKind: "agent"`.
- Updated the no-bundler frontend prompt flow to call the existing terminal
  connector with `terminalKind: "agent"` for explicit command prompts.
- Added regression coverage proving `run ls` updates the bottom agent terminal,
  not the top user terminal, and that the bottom agent terminal remains
  read-only.
- Follow-up after manual review fixed the mounted Terminal tab repaint path:
  explicit command output now refreshes the visible bottom Agent command
  terminal without requiring session switching.
- Follow-up polish after manual review made the bottom Agent command terminal a
  lighter black than the top user terminal and auto-scrolls the bottom pane so
  the latest command output remains visible.

## User Review

- User verified the TASK-011A bridge after the mounted Terminal repaint fix.
- User verified the final Agent command terminal visual distinction and
  auto-scroll behavior.

## Verification Run

- `cargo test --manifest-path backend/Cargo.toml
  task_011a_explicit_prompt_command_updates_agent_terminal_records --
  --test-threads=1` passed: 1 test, 1 pass.
- `node --test --test-concurrency=1 tests/frontend-task-011.test.js` passed:
  7 tests, 7 pass.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`
  passed: 39 tests, 39 pass.
- `node --test --test-concurrency=1 tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-006.test.js
  tests/frontend-task-007.test.js tests/frontend-task-008.test.js
  tests/frontend-task-009.test.js tests/frontend-task-010.test.js
  tests/frontend-task-010A.test.js tests/frontend-task-010B.test.js
  tests/frontend-task-010C.test.js tests/frontend-task-011.test.js
  tests/frontend-connectors.test.js` passed: 77 tests, 77 pass.
- `npm test` passed: 122 tests, 122 pass.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.
- Follow-up after manual review: `node --test --test-concurrency=1
  tests/frontend-task-011.test.js` passed: 8 tests, 8 pass.
- Follow-up focused regression: `node --test --test-concurrency=1
  tests/frontend-task-005.test.js tests/frontend-task-005A.test.js
  tests/frontend-task-007.test.js tests/frontend-task-011.test.js` passed:
  24 tests, 24 pass.
- Follow-up `git diff --check` passed.
- Follow-up UI/scroll regression: `node --test --test-concurrency=1
  tests/frontend-task-011.test.js` passed: 9 tests, 9 pass.
- Follow-up focused regression after UI/scroll polish: `node --test
  --test-concurrency=1 tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-007.test.js
  tests/frontend-task-011.test.js` passed: 25 tests, 25 pass.
- Follow-up UI/scroll `git diff --check` passed.
