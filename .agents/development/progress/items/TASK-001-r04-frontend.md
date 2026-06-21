# TASK-001 r04 Frontend

Status: ready
Updated: 2026-06-22

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

Additional frontend foundation gates: AC-022, AC-023, AC-024

## Expected Files And Folders

- `frontend/`
- `.agents/development/progress/items/TASK-001-r04-frontend.md`

No backend activation is expected in this item. Do not edit `backend/` or
`tests/server/` unless the implementation uncovers a narrowly required
scaffold note for the next task.

## Work Instructions

- Build from r04 wireframes and `wireframes/ui-handoff-spec.md`.
- Treat `wireframes/r04-single-page-app/` as the functional baseline. Port r04
  screen structure and working interactions first; do not redesign from
  scratch.
- Keep the frontend aligned with the accepted creative and product context.
- Use visible states for trust, provider/model setup, files, structured thread,
  Browser, Terminal, settings IA, extensions, concurrency, resume, local memory,
  action records, audit records, and artifact preview where those surfaces are
  part of r04 or required by acceptance.
- Preserve every r04 route and screen-specific settings layout. Providers, Add
  Provider, Models, Runtimes, Configuration, Plugins, Skills, and MCP Servers
  must not be replaced by generic cards, generic lists, or invented extension
  metadata surfaces.
- Preserve working r04 interactions: left/right panel collapse, left/right
  panel resize, Terminal bottom-panel resize, attachment preview, approval and
  branch popovers, provider/model popovers, message show/hide, plugin
  marketplace and connection dialogs, skill detail dialog, MCP add-server
  dialog, and MCP transport switching.
- Do not add new visible controls or extra surfaces that were not in r04 or the
  frozen MVP contract. Record proposed UI changes for review instead of
  silently including them in TASK-001.
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
- Production redesign beyond r04 parity.
- New controls, filters, metadata cards, settings abstractions, or alternate
  route structures not present in r04 or the frozen MVP contract.

## Verification

Run these before marking this item `review`, `done`, or `verified`:

- `git diff --check`
- `npm run dev` for local browser review when a frontend dev surface exists.
- Browser automation that renders every r04 route and verifies structure,
  control placement, and settings screen parity against
  `wireframes/r04-single-page-app/`.
- Browser automation that clicks every collapse control, opens every popover
  and dialog listed in Work Instructions, toggles message/detail states, and
  drags every resize handle far enough to prove dimensions change.
- `npm test` with rendered behavior coverage. Source-string checks alone are
  not sufficient.
- `npm run acceptance:mvp` once the acceptance script can evaluate the frontend
  surface and includes behavior-based checks for AC-022, AC-023, and AC-024.

Record skipped commands with the reason in this file before marking the item
`review`, `done`, or `verified`.

## Next Step

When active implementation starts, change this item to `in_progress` and keep
status, outputs, verification notes, and blockers current.
