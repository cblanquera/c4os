# TASK-005A Scoped Frontend State And DOM Updates

Status: ready
Updated: 2026-06-22

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-005A`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-005-openrouter-chat-session.md`
- Progress manifest: `.agents/development/progress/manifest.md`

## Goal

Consolidate frontend state and scoped DOM update boundaries before starting
provider/model management. This is a cleanup bridge approved after TASK-005
review, not a framework migration and not a new user-facing feature slice.

## Required Scope

- Introduce a small app-owned state store or equivalent state boundary for
  route, workspace, active session, per-session right-panel tool state, turns,
  connector run state, and composer-local state.
- Replace global `render()` calls for local chat/tool interactions with scoped
  DOM updates where practical.
- Preserve per-chat/new-session right-panel tool state.
- Keep transcript turn DOM keyed by stable turn/session identity so appending a
  turn does not replace prior turns.
- Preserve accepted r04 routes and visible shell structure.
- Do not add React, Preact, JSX, a bundler, or a new build pipeline in this
  task.

## Verification

- Tests prove right-panel tab changes do not replace the chat transcript DOM.
- Tests prove work-log expansion does not replace message bubbles.
- Tests prove streaming/progress updates affect only the active turn.
- Existing TASK-001 through TASK-005 tests continue to pass.

## Still Out Of Scope

- Provider/model management.
- Durable chat transcript persistence.
- Full runtime adapter persistence.
- React/Preact or other framework migration.
- New route structures, panels, or settings abstractions.
