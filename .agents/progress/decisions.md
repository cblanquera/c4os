# Decisions

## DEC-001: Start With TASK-001

Date: 2026-06-11
Status: active

Decision:

Start implementation with TASK-001, the Tauri app scaffold and backend command
boundary.

Reason:

`.agents/archive/planning-corpus/implementation/recommended-build-order.md` identifies the local
foundation as the first dependency layer, and TASK-001 has no dependencies.

Applies To:

- item-001

## DEC-002: Track Progress In Agent Progress Bank And CSV Sheet

Date: 2026-06-11
Status: active

Decision:

Use `.agents/progress/manifest.md` as the routing dashboard and
`.agents/progress/progress.csv` as the spreadsheet-style progress sheet.

Reason:

The implementation spans many dependent tasks and needs durable status tracking
that future sessions can update without loading the full planning corpus.

Applies To:

- all items

## DEC-003: Nested AGENTS.md V1 Tier

Date: 2026-06-11
Status: active

Decision:

Current V1 supports nested `AGENTS.md` at
`display_guidance_order_only` tier. The app discloses an ordered guidance stack
with root guidance first and nested files ordered by path depth and name.

Reason:

This resolves TASK-033 without promoting full AGENTS.md compatibility. It gives
users visible instruction-source context while preserving the established
permission and model-context boundaries.

Applies To:

- item-034

## DEC-004: `.agents` Replaces `.task-bank`

Date: 2026-06-11
Status: active

Decision:

Use `.agents/progress/` for active execution state and
`.agents/specs/c4os-mvp/` for durable planning records. Do not add new updates
to `.task-bank/`.

Reason:

The project is moving to the ChrisAI agent spec and progress layout while
preserving `.task-bank/` as a deprecated historical source until deletion is
confirmed.

Applies To:

- all items

