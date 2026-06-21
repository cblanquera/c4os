# MVP Freeze Review

Status: accepted
Updated: 2026-06-21

## Target

Freeze target: `.agents/specs/mvp/`

Source records:

- `.agents/context/product-brief.md`
- `.agents/context/product-specs.md`
- `.agents/context/technical-specs.md`
- `.agents/context/creative-specs.md`
- `.agents/context/work-orders.md`
- `.agents/specs/mvp/brief.md`
- `.agents/specs/mvp/requirements.md`
- `.agents/specs/mvp/acceptance.md`
- `.agents/specs/mvp/decisions.md`
- `.agents/specs/mvp/risks.md`
- `.agents/specs/mvp/evidence.md`
- `.agents/specs/mvp/mvp-viability-gaps.md`
- `.agents/specs/mvp/tasks.md`
- `.agents/specs/mvp/traceability.md`

## Decision

The MVP spec is frozen for implementation. The accepted contract is the full
documented/r04 desktop MVP scope, with checkpoints used only as implementation
progress gates.

## Checks

- Requirements link to acceptance criteria through
  `.agents/specs/mvp/traceability.md`.
- Important decisions, risks, evidence, and viability gaps are recorded.
- Browser, Terminal, extension, concurrency, restart/resume, Files, settings,
  approvals, local memory, action records, audit records, and artifact preview
  scope remain represented.
- Implementation paths are `backend/`, `frontend/`, and `tests/server/`.
- `src-tauri/` is explicitly out of path.
- r04 wireframes and UI handoff remain required inputs for frontend work.
- Mock-backed phases must state exactly what is mocked before user acceptance.

## Accepted Risks

The viability gaps in `.agents/specs/mvp/mvp-viability-gaps.md` are accepted as
implementation-time risks. They do not block freeze, but progress items must
keep them visible when a task touches Browser, Terminal, extensions,
concurrency, resume, descriptor safety, or replacement of mocked behavior.

## Next Step

Use `.agents/workflows/progress.md` to execute the ready progress item for
`TASK-001`.
