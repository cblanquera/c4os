# MVP Status

Status: ready-for-review
Updated: 2026-06-21

## Classification

- Setup mode: MVP specification
- Source confidence: frozen research converted into MVP contract draft
- Implementation state: not started by this spec
- Freeze state: not frozen for implementation

## Readiness

`.agents/specs/mvp/` now defines the distributable desktop MVP candidate from
the frozen research records. The spec is ready for review and freeze, but it
does not authorize active progress items yet.

Implementation must not start until this status is changed to
`frozen-for-implementation` by the freeze workflow and proposed tasks are
converted into progress items.

## Required Review Before Freeze

- Confirm the full documented/r04 scope remains included.
- Confirm checkpoints are sequencing gates only.
- Confirm implementation paths remain `backend/`, `frontend/`, and
  `tests/server/`.
- Confirm no `src-tauri/` implementation path is introduced.
- Confirm Browser, Terminal, extension, concurrency, and resume acceptance
  points are covered.

## Next Step

Run review/freeze on `.agents/specs/mvp/`. If accepted, mark this spec
`frozen-for-implementation`, then convert proposed tasks into
`.agents/development/progress/` items.

