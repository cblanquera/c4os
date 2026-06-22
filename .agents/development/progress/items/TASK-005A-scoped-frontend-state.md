# TASK-005A Scoped Frontend State And DOM Updates

Status: verified
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

## Outputs

- Added `frontend/state.js` as the small app-owned vanilla JS state boundary
  for route, workspace/session, right-panel tools, turns, connector run state,
  and composer-local state.
- Kept the accepted r04 shell and TASK-005 chat UX while replacing practical
  chat/tool full renders with scoped DOM updates:
  - right-panel tab changes replace only `.tool-panel`;
  - work-log disclosure updates only the matching work log;
  - connector progress and streaming updates target the active turn;
  - connector completion waits for a short app-owned visibility frame before
    finalizing the active turn, so pending/progress state remains observable
    without rerendering the full shell;
  - file explorer row and breadcrumb clicks update the right-panel Files view
    without changing the active chat route or screen;
  - file row and breadcrumb buttons reset native button chrome so the Files
    panel keeps its row and breadcrumb visual treatment;
  - composer model/provider picker interactions update the local composer
    instead of routing through `providers-popover` or `models-popover`;
  - sidebar project/session selection updates app-owned workspace/session
    state and scoped shell labels/active rows;
  - shell collapse, panel width, terminal height, and composer-local prompt,
    attachment, branch, approval, and picker state now have app-owned store
    slots;
  - route-owned tool screens still expose their expected tool panel even if a
    previous chat surface collapsed the right panel;
  - scoped regions guard against accidental `a[data-link]` hash navigation;
  - appended turns are keyed by stable `data-turn-id` values.
- Added `tests/frontend-task-005A.test.js` to guard DOM identity across scoped
  right-panel, work-log, streaming/progress, file-panel, file-panel visual
  styling, composer-picker, sidebar-session, shell-layout, and scoped-route-link
  updates.

## Verification Run

- `node --test tests/frontend-task-005A.test.js` passed.
- `node --test tests/frontend-task-005.test.js` passed.
- `node --test tests/frontend-task-002.test.js tests/frontend-task-005A.test.js`
  passed after the full-suite timing check.
- `node --test tests/frontend-r04.test.js tests/frontend-task-002.test.js
  tests/frontend-task-005.test.js tests/frontend-task-005A.test.js` passed
  after fixing scoped file-panel clicks.
- `node --test tests/frontend-r04.test.js tests/frontend-task-002.test.js
  tests/frontend-task-004.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js` passed after applying the audit follow-up
  scoped state changes.
- `node --test tests/frontend-r04.test.js tests/frontend-task-002.test.js
  tests/frontend-task-004.test.js tests/frontend-task-005.test.js
  tests/frontend-task-005A.test.js` passed after fixing file-panel button and
  breadcrumb styling regression.
- `npm test` passed: 51 tests, 51 pass.

## Acceptance

TASK-005A was user-approved on 2026-06-22 after verification. TASK-006 may
start from this scoped frontend state boundary.

## Still Out Of Scope

- Provider/model management.
- Durable chat transcript persistence.
- Full runtime adapter persistence.
- React/Preact or other framework migration.
- New route structures, panels, or settings abstractions.
