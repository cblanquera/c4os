# Progress Manifest

Status: ready-for-task-006
Updated: 2026-06-22

## Active Stream

MVP implementation queue from frozen spec `.agents/specs/mvp/status.md`.

## Last Verified Item

- `.agents/development/progress/items/TASK-001-r04-frontend.md`
- `.agents/development/progress/items/TASK-002-mock-server-connection.md`
- `.agents/development/progress/items/TASK-003-backend-mock-parity.md`
- `.agents/development/progress/items/TASK-004-first-user-flow.md`
- `.agents/development/progress/items/TASK-005-openrouter-chat-session.md`
- `.agents/development/progress/items/TASK-005A-scoped-frontend-state.md`

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

Start `TASK-006` provider/model management from the verified TASK-005A scoped
frontend state boundary. Preserve the accepted r04 shell and scoped update
patterns unless a later task explicitly changes the UI contract.
