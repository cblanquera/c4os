# TASK-001 r04 Frontend

Status: ready
Updated: 2026-06-21

## Source

- Task: `.agents/specs/mvp/tasks.md` `TASK-001`
- Frozen spec: `.agents/specs/mvp/status.md`
- Traceability: `.agents/specs/mvp/traceability.md`
- Wireframes: `wireframes/r04-single-page-app/` and
  `wireframes/ui-handoff-spec.md`
- Context: `.agents/context/product-specs.md`,
  `.agents/context/technical-specs.md`, `.agents/context/creative-specs.md`,
  `.agents/context/work-orders.md`

## Goal

Build the full r04 frontend in `frontend/` as the actual product shell
frontend, ready to connect to mocked server behavior in `TASK-002`.

## Linked Requirements

REQ-001, REQ-003, REQ-008, REQ-010, REQ-011, REQ-013, REQ-014, REQ-015,
REQ-016

## Linked Acceptance

AC-001, AC-013, AC-014, AC-015, AC-016, AC-017, AC-018, AC-019, AC-020, AC-021

## Expected Files And Folders

- `frontend/`
- `.agents/development/progress/items/TASK-001-r04-frontend.md`

No backend activation is expected in this item. Do not edit `backend/` or
`tests/server/` unless the implementation uncovers a narrowly required
scaffold note for the next task.

## Work Instructions

- Build from r04 wireframes and `wireframes/ui-handoff-spec.md`.
- Keep the frontend aligned with the accepted creative and product context.
- Use visible states for trust, provider/model setup, files, structured thread,
  Browser, Terminal, settings IA, extensions, concurrency, resume, local memory,
  action records, audit records, and artifact preview where those surfaces are
  part of r04 or required by acceptance.
- Keep data mocked or local to the frontend only as needed for visual and state
  completeness; do not claim real backend, runtime, filesystem, Browser,
  Terminal, extension, security, approval, or persistence behavior in this item.
- Do not pause for user review during this item. Continue until the frontend is
  coherent, maintainable, and ready for `TASK-002`.

## Non-Goals

- Real backend behavior.
- Real provider or runtime integration.
- Real filesystem, Browser, Terminal, extension, security, approval, or
  persistence behavior.
- Mock server harness.
- `src-tauri/`.

## Verification

Run these before marking this item `review`, `done`, or `verified`:

- `git diff --check`
- `npm run dev` for local browser review when a frontend dev surface exists.
- `npm test` if frontend work touches existing test-covered behavior.
- `npm run acceptance:mvp` once the acceptance script can evaluate the frontend
  surface.

Record skipped commands with the reason in this file before marking the item
`review`, `done`, or `verified`.

## Next Step

When active implementation starts, change this item to `in_progress` and keep
status, outputs, verification notes, and blockers current.
