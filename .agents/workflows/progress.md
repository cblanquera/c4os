# Progress Workflow

Use this for active execution packets, batches, logs, and handoffs.

Use progress only after an MVP or bounded feature spec is frozen for implementation.

## Rules

- A `TASK` record is proposed work, not active execution.
- Historical MVP execution state belongs in `.agents/development/mvp/`.
- Future active implementation belongs in `.agents/development/<spec-id>/`, where `<spec-id>` matches the frozen spec folder.
- Do not create one shared global progress namespace for unrelated specs.
- Progress items must link to frozen spec tasks, requirements, acceptance criteria, context, wireframes, or validation evidence.
- MVP progress items must link to `.agents/specs/mvp/`.
- Future spec progress items must link to their matching `.agents/specs/<spec-id>/` folder.
- Progress files execute scope; they do not define product architecture or MVP boundaries.
- Raw feedback, QA notes, and polish requests must be validated or reconciled before they become progress items.
- Only mark an item `verified` after the stated verification was actually run.

## Structure

```text
.agents/development/
  mvp/
    manifest.md
    items/
    batches/
    logs/
  <spec-id>/
    manifest.md
    items/
    batches/
    logs/
```

MVP keeps its historical `TASK-001` through `TASK-017` style IDs. Future spec work must use scoped IDs such as `FI-01-TASK-001` or another explicit per-spec prefix recorded in the spec's progress manifest. Never continue the MVP task counter for unrelated final-implementation specs.

## Implementation Paths

For distributable MVP work, progress items may edit:

- `backend/`
- `frontend/`
- `tests/server/`
- `.agents/development/mvp/` for MVP execution state
- `.agents/development/<spec-id>/` for future frozen-spec execution state
- `.agents/specs/mvp/` only when implementation discovers a required spec correction
- `.agents/specs/<spec-id>/` only when implementation discovers a required correction for the matching frozen spec

Do not create or use `src-tauri/`.

POC code belongs in `proofs/<proof-name>/`, not product implementation paths.

## Start Gate

Before creating or starting a progress item:

1. Read `.agents/specs/mvp/status.md` for MVP work, or `.agents/specs/<spec-id>/status.md` for future spec work.
2. Confirm status is `frozen-for-implementation`.
3. Link the item to task IDs from the matching spec's `tasks.md`.
4. Link each item to requirement and acceptance IDs from the same frozen spec.
5. Name the files or folders expected to change.
6. Name the verification command.
7. Use the matching development folder: `.agents/development/mvp/` for MVP, or `.agents/development/<spec-id>/` for a future frozen spec.

## Item Statuses

Use `planned`, `ready`, `in_progress`, `blocked`, `review`, `done`, and `verified`.

## Stop

Stop when the active item has current status, outputs, verification notes, blockers, and a recommended next step.
