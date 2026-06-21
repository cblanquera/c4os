# Progress Workflow

Use this for active execution packets, batches, logs, and handoffs.

Use progress only after an MVP or bounded feature spec is frozen for implementation.

## Rules

- A `TASK` record is proposed work, not active execution.
- Active implementation belongs in `.agents/development/progress/`.
- Progress items must link to frozen spec tasks, requirements, acceptance criteria, context, wireframes, or validation evidence.
- MVP progress items must link to `.agents/specs/mvp/`.
- Progress files execute scope; they do not define product architecture or MVP boundaries.
- Raw feedback, QA notes, and polish requests must be validated or reconciled before they become progress items.
- Only mark an item `verified` after the stated verification was actually run.

## Implementation Paths

For distributable MVP work, progress items may edit:

- `backend/`
- `frontend/`
- `tests/server/`
- `.agents/development/progress/` for execution state
- `.agents/specs/mvp/` only when implementation discovers a required spec correction

Do not create or use `src-tauri/`.

POC code belongs in `proofs/<proof-name>/`, not product implementation paths.

## Start Gate

Before creating or starting a progress item:

1. Read `.agents/specs/mvp/status.md` for MVP work.
2. Confirm status is `frozen-for-implementation`.
3. Link the item to task IDs from `.agents/specs/mvp/tasks.md`.
4. Link each item to MVP requirement and acceptance IDs.
5. Name the files or folders expected to change.
6. Name the verification command.

## Item Statuses

Use `planned`, `ready`, `in_progress`, `blocked`, `review`, `done`, and `verified`.

## Stop

Stop when the active item has current status, outputs, verification notes, blockers, and a recommended next step.
