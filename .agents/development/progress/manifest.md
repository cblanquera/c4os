# Progress Manifest

Status: ready
Updated: 2026-06-22

## Active Stream

MVP implementation queue from frozen spec `.agents/specs/mvp/status.md`.

## Current Item

- `.agents/development/progress/items/TASK-001-r04-frontend.md`

## Scope Rules

- Progress items execute the frozen MVP contract; they do not redefine product
  architecture or MVP scope.
- Production implementation paths are `backend/`, `frontend/`, and
  `tests/server/`.
- Do not create or use `src-tauri/`.
- Proposed task records remain in `.agents/specs/mvp/tasks.md`; active work
  lives in `.agents/development/progress/items/`.
- Mock-backed phases must state exactly what is mocked before acceptance.
- TASK-001 is a parity-first frontend task. Preserve r04 route structures,
  working interactions, and settings screen shapes before any production
  cleanup. Do not add invented UI or accept source-string-only verification.

## Next Step

Start `TASK-001` only when the user asks for active implementation.
