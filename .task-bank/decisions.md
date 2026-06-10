# Decisions

## DEC-001: Start With TASK-001

Date: 2026-06-11
Status: active

Decision:

Start implementation with TASK-001, the Tauri app scaffold and backend command
boundary.

Reason:

`plans/implementation/recommended-build-order.md` identifies the local
foundation as the first dependency layer, and TASK-001 has no dependencies.

Applies To:

- item-001

## DEC-002: Track Progress In Task Bank And CSV Sheet

Date: 2026-06-11
Status: active

Decision:

Use `.task-bank/manifest.md` as the routing dashboard and
`.task-bank/progress.csv` as the spreadsheet-style progress sheet.

Reason:

The implementation spans many dependent tasks and needs durable status tracking
that future sessions can update without loading the full planning corpus.

Applies To:

- all items
