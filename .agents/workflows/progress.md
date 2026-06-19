# Progress Workflow

Use this for active execution packets, batches, logs, and handoffs.

## Rules

- A `TASK` record is proposed work, not active execution.
- Active implementation belongs in `.agents/development/progress/`.
- Progress items must link to specs, acceptance criteria, context, wireframes, or validation evidence.
- Only mark an item `verified` after the stated verification was actually run.

## Item Statuses

Use `planned`, `ready`, `in_progress`, `blocked`, `review`, `done`, and `verified`.

## Stop

Stop when the active item has current status, outputs, verification notes, blockers, and a recommended next step.
