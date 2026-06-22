# TASK-005 OpenRouter Chat Session Review Slice

Status: accepted
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-005`
- Frozen spec: `.agents/specs/mvp/status.md`
- Progress manifest: `.agents/development/progress/manifest.md`
- Accepted baseline: `.agents/development/progress/items/TASK-004-first-user-flow.md`

## Goal

Replace the TASK-004 mock chat run with a real OpenRouter-backed chat session
review path behind the accepted r04 UI. The review target is the visible chat
experience: real API request, streamed reasoning/thinking activity when the
selected reasoning model returns it, structured app/runtime activity before the
final response, and a final assistant response.

## Linked Requirements

REQ-003, REQ-004, REQ-005, REQ-013, REQ-014, REQ-016

## Linked Acceptance

AC-003, AC-004, AC-005, AC-013, AC-015, AC-016

## Expected Changed Paths

- `backend/`
- `frontend/`
- `tests/server/`
- `.agents/development/progress/`

## Required Scope

- Use a real OpenRouter API key through backend-owned credential handling or a
  review-only secure local configuration. Raw keys must not be written to
  workspace descriptors, transcripts, app-normal state, logs, or frontend
  fixtures.
- Add a backend OpenRouter client path for `POST
  https://openrouter.ai/api/v1/chat/completions` with streaming enabled.
- Send reasoning configuration for reasoning-capable models and tolerate models
  that do not emit reasoning.
- Normalize streamed OpenRouter events into C4OS-owned runtime events:
  reasoning/thinking, app/runtime activity, assistant content, completion,
  error, and usage/metadata when available.
- Render reasoning/thinking as structured activity before the final assistant
  response when OpenRouter returns `reasoning_details` or equivalent reasoning
  text.
- Keep app/runtime activity distinct from assistant content.
- Create and update C4OS-owned workspace, session, message, run, and runtime
  event records needed for the session list and active thread to stay coherent
  during the review.
- Preserve the accepted r04 chat/session layout and connector boundary. Do not
  add alternate onboarding, alternate chat structure, new settings
  abstractions, or new route structures.
- Pause for user review after the real OpenRouter chat session experience is
  working.

## Still Allowed To Remain Mocked

- Files content and mutation.
- Browser.
- Terminal.
- Extensions, plugins, skills, and MCP runtime behavior.
- Local memory.
- Artifacts.
- Broad restart/resume.
- Full provider-management polish.
- Advanced approval policy.
- Audit-log hardening.
- Non-chat feature slices.

## Verification

- `tests/server/task-005-openrouter-chat.test.js` proves the backend has an
  OpenRouter stream normalizer, uses `OPENROUTER_API_KEY`, targets
  `google/gemini-2.5-flash-lite`, requests reasoning support, emits
  `c4os://runtime-event`, and does not persist raw `sk-or-v1-` key material in
  the reviewed source paths.
- `tests/frontend-task-005.test.js` proves the accepted shell can create a
  session, receive streamed reasoning/activity events, show thinking before the
  final assistant response, and render the final response without adding a new
  chat layout.
- `cargo test --manifest-path backend/Cargo.toml`
- `npm test` passed: 43 tests, 8 suites.
- `git diff --check` passed.

## Implementation Notes

- Backend chat execution uses direct OpenRouter chat completions streaming for
  this review slice instead of routing through OpenCode.
- The review key source is the process environment variable
  `OPENROUTER_API_KEY`; raw keys are not written to descriptors, fixtures,
  normal app-visible records, or source files.
- The default review model is `google/gemini-2.5-flash-lite`.
- OpenRouter stream deltas are normalized into C4OS-owned runtime events:
  `reasoning`, `activity`, `assistant`, `complete`, and `error`.
- OpenRouter requests have a 15 second connect timeout and 90 second total
  timeout so failed network/API paths return visibly instead of leaving the
  shell waiting indefinitely.
- The frontend subscribes through the existing Tauri connector boundary and
  renders runtime thinking/activity in the existing Tool Call and Agent Run
  surfaces before the final assistant response.
- The frontend also replays returned runtime events from the final command
  payload if live desktop event delivery is unavailable or the event listener
  setup fails.
- Connector failures render actionable detail in the Agent Run card instead of
  a blank placeholder.
- The pending state is held for a short visible frame so the accepted chat
  shell visibly indicates that a request started.
- Chat turns are appended in transcript order instead of replacing the prior
  prompt/response, and each turn renders user prompt, thinking/activity, then
  assistant response.
- Newly added progress and response elements ease into the existing shell with
  a short CSS transition; no alternate chat layout or route was added.
- The old `Tool Call` and `Agent Run` cards were replaced with a compact
  Codex-style `Worked for ...` work log row. Completed work logs collapse and
  can be expanded; failed logs stay expanded so errors remain visible.
- Chat bubble top-right chevrons were removed. Message disclosure still uses
  the existing `Show More` / `Show Less` affordance.
- Assistant response text is treated as Markdown and rendered through a
  DOM-built Markdown subset instead of raw text.
- A permission prompt surface now exists above the composer for future approval
  events, but no approval/runtime permission behavior is activated in this
  slice.
- Right-panel Browser, Files, and Terminal tabs now switch local tool-panel
  state without navigating away from the active chat session.
- Right-panel tool state is keyed by the current chat-session or new-session
  surface so a chat can keep a different active right-panel tool than the new
  session screen. Tool-tab clicks replace only the right panel instead of
  rerendering the whole shell, preventing transcript fade/reload.
- Completed `Worked for ...` rows remain expandable on older turns after later
  prompts are appended.
- A live API smoke call was not run during automated verification to avoid
  exposing or spending the provided short-lived review key from shell tooling.

## Open Questions

- None for this review slice. Full provider settings, keychain-backed storage,
  OpenCode routing, deeper persistence, and durable transcript/event storage
  remain later slices.

## Acceptance

- Accepted by user on 2026-06-22 after OpenRouter live review and chat-session
  UX polish.
