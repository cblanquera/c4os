# Progress Manifest

Status: ready-for-task-012
Updated: 2026-06-26

## Active Stream

MVP implementation queue from frozen spec `.agents/specs/mvp/status.md`.

## Last Verified Item

- `.agents/development/progress/items/TASK-001-r04-frontend.md`
- `.agents/development/progress/items/TASK-002-mock-server-connection.md`
- `.agents/development/progress/items/TASK-003-backend-mock-parity.md`
- `.agents/development/progress/items/TASK-004-first-user-flow.md`
- `.agents/development/progress/items/TASK-005-openrouter-chat-session.md`
- `.agents/development/progress/items/TASK-005A-scoped-frontend-state.md`
- `.agents/development/progress/items/TASK-006-provider-model-management.md`
- `.agents/development/progress/items/TASK-007-runtime-adapter-persistent-sessions.md`
- `.agents/development/progress/items/TASK-008-files-slice.md`
- `.agents/development/progress/items/TASK-009-artifact-safe-html-preview.md`
- `.agents/development/progress/items/TASK-010-browser-slice.md`
- `.agents/development/progress/items/TASK-010A-browser-address-bar-local-target-ui.md`
- `.agents/development/progress/items/TASK-010B-native-browser-webview-or-external-open-fallback.md`
- `.agents/development/progress/items/TASK-010C-artifact-preview-type-rendering.md`
- `.agents/development/progress/items/TASK-011-terminal-slice.md`
- `.agents/development/progress/items/TASK-011A-agent-command-terminal-bridge.md`
- `.agents/development/progress/items/TASK-011B-chat-session-transition-polish.md`

## Scope Rules

- Progress items execute the frozen MVP contract; they do not redefine product
  architecture or MVP scope.
- Production implementation paths are `backend/`, `frontend/`, and
  `tests/server/`.
- TASK-001 is the frontend-foundation exception: use `frontend/`,
  `tests/frontend-*.test.*`, `tests/support/`, minimal package scripts, and the
  active progress item only. Do not create `tests/server/` content until
  TASK-002 or later.
- Do not create or use `src-tauri/`.
- Proposed task records remain in `.agents/specs/mvp/tasks.md`; active work
  lives in `.agents/development/progress/items/`.
- Mock-backed phases must state exactly what is mocked before acceptance.
- TASK-001 is a parity-preserving production frontend task. Preserve r04 route
  structures, working interactions, and settings screen shapes, then apply a
  production desktop-app visual treatment. Do not add invented UI, stop at a
  grayscale wireframe copy, or accept source-string-only verification.
- After TASK-001, implementation may switch static fixtures and static
  component behavior to dynamic, backend-backed, or working behavior behind the
  accepted UI. Do not add new visible UI components, controls, panels, menus,
  cards, settings abstractions, or route structures unless explicitly accepted
  and documented before implementation.

## Next Step

TASK-011B is verified. Continue with `TASK-012` from
`.agents/specs/mvp/tasks.md` without renumbering frozen MVP tasks. Preserve the
verified TASK-011 Terminal slice: the existing right-panel Terminal tab renders
an `@xterm/xterm` transcript backed by C4OS-owned `portable-pty` Tauri
commands/events, while user terminal and agent command terminal state remain
product-owned and distinct. Also preserve verified TASK-011A behavior: explicit
command prompts such as `run ls` execute through backend-owned trusted
workspace command execution and write command, cwd, status, exit code, output,
session id, and `terminalKind: "agent"` into the bottom read-only Agent command
terminal. Preserve verified TASK-011B behavior: new-session prompt submission
immediately creates/selects the new chat item, switches into the transcript,
shows the submitted user turn first, and renders a clear working/streaming or
provider-setup-required assistant state. New-session Terminal must not inherit
another chat's Agent command terminal output, project chat navigation is
newest-first, and explicit agent commands should show their active running
state and start the command bridge before the assistant response resolves.
Agent command terminal output is latest-command-only; user terminal output
remains transcript-based. Late terminal command responses must be ignored when
their captured session id no longer matches the active chat session. Explicit
command prompt reconciliation must preserve the locally active Agent terminal
when a stale session snapshot returns, and simple prompts like
`run pwd and git status` should execute as `pwd && git status`. New-session
explicit command prompts should select the Terminal tool before the
chat-session shell renders; existing chat sessions preserve the user's current
right-panel tool. `run_terminal_command` is the sole backend owner of Agent
command terminal mutations; `send_prompt` / `append_prompt` must not mutate or
return explicit-command Agent terminal output. The native Tauri
`run_terminal_command` request must accept the frontend's camelCase
`sessionId` and `terminalKind` fields so explicit command prompts are recorded
as Agent terminal actions, not user terminal actions. The explicit prompt
command parser is transitional; do not expand it into natural-language command
planning. Future command/tool planning should move to a formal runtime tool
gateway where runtime events request stable tool identities, per-session tool
config controls enabled state/access/approval, and C4OS owns execution and
authority. Broad approval-policy hardening remains TASK-016.
