# TASK-011B Chat Session Transition Polish

Status: verified
Updated: 2026-06-26

## Source

- User review of `plans/C4OS-Session-Example.mov`
- Comparison recording `plans/Codex-Session-Example.mov`
- User review of `/Users/cblanquera/Desktop/Screen Recording 2026-06-26 at 1.27.42 PM.mov`
- User review of `/Users/cblanquera/Desktop/Screen Recording 2026-06-26 at 1.45.24 PM.mov`
- User review of `/Users/cblanquera/Desktop/Screen Recording 2026-06-26 at 2.21.56 PM.mov`
- Follow-up before `.agents/specs/mvp/tasks.md` `TASK-012`
- MVP requirements: `REQ-004`, `REQ-005`, `REQ-013`, `REQ-014`, `REQ-016`
- MVP acceptance: `AC-004`, `AC-005`, `AC-015`, `AC-016`

## Goal

Polish the new-session prompt submission transition so C4OS behaves like a
chat-first session surface: submitting a new prompt immediately creates and
selects the new chat item, switches into the transcript, shows the submitted
user turn, and renders a clear run/working/streaming or setup-required state.

## Required Scope

- Preserve the accepted three-panel shell, left project/session navigation, and
  Browser / Files / Terminal right-panel tab contract.
- Preserve verified TASK-011 and TASK-011A Terminal behavior.
- Preserve provider/model, session ownership, runtime adapter, Files, Browser,
  artifact preview, and scoped frontend update boundaries.
- Keep this as a session transition and run-state polish task before TASK-012;
  do not implement Settings IA, extension records, runtime extension
  invocation, concurrency, restart/resume, memory, audit hardening, or broad
  approval-policy changes.
- Do not renumber frozen MVP tasks. This progress item is an inserted follow-up
  between TASK-011A and TASK-012.

## Acceptance

- Starting from the new-session composer, submitting a prompt creates a new chat
  item in the project/session navigation and selects it.
- The main pane transitions into the new chat transcript without showing stale
  prior chat content.
- The first visible turn in the transcript is the submitted user prompt.
- The assistant side enters a distinct working, streaming, completed, failed,
  or provider-setup-required state instead of looking like an inert or
  accidental transcript card.
- If the selected provider/model cannot run because setup is incomplete, the UI
  handles that as an explicit setup-required state before or during the new
  transcript transition.
- Regression coverage verifies the new-session-to-chat-session transition and
  protects existing follow-up composer context behavior.

## Expected Implementation Paths

- `frontend/`
- `tests/frontend-task-005*.test.js` or a focused
  `tests/frontend-task-011B*.test.js`
- `backend/` and `tests/server/` only if the setup-required state needs a
  backend-owned field that is already part of the existing session/runtime
  contract.
- `.agents/development/progress/`

## Verification

- `node --test --test-concurrency=1 tests/frontend-task-011B.test.js` passed:
  12 tests cover immediate new chat creation/selection, stale transcript
  prevention, first user turn visibility, realtime runtime activity before
  final response, completed work-log expansion after backend session
  reconciliation, agent command terminal session isolation, newest-first chat
  session navigation, active agent command running-state rendering before the
  prompt response resolves, late prior-session terminal response isolation,
  existing-thread stale terminal snapshot protection, multi-command prompt
  normalization, Terminal tool selection for new-session explicit command
  prompts, fenced markdown code block rendering, working state, and explicit
  provider-setup-required state.
- `node --test --test-concurrency=1 tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js tests/frontend-task-007.test.js
  tests/frontend-task-011.test.js tests/frontend-task-011B.test.js` passed:
  37 tests cover TASK-005, TASK-005A, TASK-007, TASK-011, and TASK-011B
  regressions, including preserved Terminal behavior.
- `node --test --test-concurrency=1 tests/server/task-005-openrouter-chat.test.js
  tests/server/task-011-terminal.test.js` passed: 8 tests, including async
  native prompt execution for realtime events, newest-first backend project
  session listing, and clean zsh startup for Terminal PTYs.
- `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1` passed:
  40 backend tests.
- Manual QA followed `.agents/workflows/manual-qa.md` with
  `npm run backend:build` and an isolated native debug launch equivalent to
  `npm run backend:run:qa` using
  `C4OS_SESSION_STORE=/tmp/c4os-task-011-qa-sessions-fixed.json cargo run
  --manifest-path backend/Cargo.toml`. The route sequence reached the start
  screen, opened `/Users/cblanquera/server/projects/cblanquera/c4os` through
  the native folder picker, reached `#new-session` for workspace `c4os`,
  submitted `run git status`, then submitted `run git status and ls` as an
  existing-thread follow-up. The visible app transcript showed the submitted
  user prompt first, the left navigation selected the new `run git status`
  chat item, and the Agent command terminal showed `$ git status` followed by
  `$ git status && ls` for the follow-up with no stale `$ pwd` output. The
  isolated session record stored both terminal actions with
  `terminalKind: "agent"` while the user terminal remained unchanged. Backend
  stdout/stderr showed no runtime errors. `screencapture` was unavailable in
  this environment (`could not create image from display`), so evidence came
  from macOS accessibility inspection plus the isolated session store. The
  generated `.c4os/workspace.json` trusted-folder descriptor side effect was
  removed after QA.
- Provider/API setup was not available in the manual QA run, so the assistant
  response correctly used the setup/fallback text
  `Configure a provider key to run the selected model.` while the explicit
  command bridge still completed through the Agent command terminal.
- `cargo fmt --manifest-path backend/Cargo.toml -- --check` passed.
- `git diff --check` passed.

## Outputs

- `frontend/data.js` now creates/selects a provisional chat session and pending
  turn before awaiting backend `create_session`, then reconciles the backend
  session id without replacing the active pending turn.
- `frontend/data.js` renders setup-required connector failures as an explicit
  `Provider setup required.` assistant state with setup detail in the work log.
- `tests/frontend-task-011B.test.js` adds focused regression coverage for the
  new-session-to-chat transition and setup-required state.
- Follow-up polish fixed backend session reconciliation so the completed
  `Worked for...` work log remains expandable, protected new-session runtime
  activity streaming before final responses return, and started zsh-backed
  Terminal PTYs in fast mode before interactive mode so user startup-file
  errors do not print ahead of the prompt.
- Second follow-up made native `send_prompt` run on a blocking worker so Tauri
  runtime events can reach the frontend during execution, reset new chat
  Terminal state instead of inheriting another session's agent output, and
  rendered fenced markdown blocks as `<pre><code>` content.
- Third follow-up made the new-session Terminal tab render an empty Agent
  command terminal, sorted backend project session records newest-first for the
  left navigation, and stages explicit agent commands as `$ command` plus
  `Running...` before the command bridge returns final output.
- Fourth follow-up starts the explicit agent command bridge before awaiting the
  assistant response, so a prompt like `run git status` replaces previous Agent
  terminal output immediately and can stream command results independently of
  the chat response lifecycle.
- Fifth follow-up makes backend-owned Agent command terminal results
  latest-command-only instead of append-only, while preserving append behavior
  for the user terminal transcript. This prevents older commands like `$ pwd`
  from reappearing when a later explicit command such as `run git status`
  persists or reconciles.
- Sixth follow-up guards frontend terminal command responses by the session id
  captured when the command started, so a late response from a previously
  selected chat cannot overwrite the currently selected chat's Agent terminal.
- Seventh follow-up preserves the locally active Agent terminal while an
  explicit command prompt reconciles a stale session snapshot in an existing
  thread, and normalizes simple multi-command prompts such as
  `run pwd and git status` to `pwd && git status` before both frontend command
  bridge execution and backend persistence. This parser behavior was later
  superseded by the WO-006/runtime-gateway correction below; do not restore it.
- Eighth follow-up selects the Terminal tool for new-session explicit command
  prompts before the chat-session shell renders, so command output is visible
  instead of hidden behind the Browser tab. This prompt-text auto-selection was
  later superseded by the WO-006/runtime-gateway correction below; do not
  restore it. Existing chat sessions keep the user's current right-panel tool
  selection.
- Ninth follow-up makes `run_terminal_command` the only backend path that
  mutates Agent command terminal output. `send_prompt` / `append_prompt` now
  persist chat runtime state only, preventing explicit-command chat responses
  from carrying stale Agent terminal snapshots such as `$ pwd`.
- Tenth follow-up fixes the real Tauri command request contract by accepting
  frontend camelCase `terminalKind` / `sessionId` fields in
  `TerminalCommandRequest`. Native manual QA caught that the frontend command
  bridge was otherwise defaulting to `terminalKind: "user"` even when tests
  using internal snake_case paths passed.
- Post-verification correction: the explicit prompt command parser must not be
  treated as accepted behavior. Chat prompts are sent to the runtime, and the
  runtime/tool-gateway path is responsible for requesting `terminal.run` or any
  other tool. C4OS executes approved tools through product-owned authority
  boundaries. The reusable contract should map per-session tool config by tool
  identity such as `terminal.run`, `files.read`, `files.write`, or
  `browser.open`, not by lifecycle event name. Tool implementations can define
  default and maximum approval levels; session config can narrow authority but
  must not silently widen it. This is documented in
  `.agents/context/technical-specs.md`,
  `.agents/references/context/technical-specs/runtime-adapter.md`, and
  `.agents/context/work-orders.md`, and should be handled before or during
  TASK-016 rather than added casually to TASK-012.

## Next Step

After TASK-011B is accepted, continue with `.agents/specs/mvp/tasks.md`
`TASK-012` without renumbering frozen MVP tasks. Do not restore or expand the
removed explicit command parser; command planning belongs to the runtime
tool-gateway path.
