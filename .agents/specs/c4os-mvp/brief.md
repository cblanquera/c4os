# C4OS MVP Brief

Spec ID: c4os-mvp
Type: planned-brownfield
Status: active

## Objective

Track the frozen C4OS MVP and current V1 follow-on implementation scope as
compact records and progress packets.

## Audience

Future coding agents and maintainers implementing or verifying C4OS planning
and execution work.

## Source Material

- `CONTEXT.md`
- `.agents/archive/planning-corpus/AGENTS.md`
- `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
- `.agents/archive/planning-corpus/implementation/`
- `.agents/archive/planning-corpus/acceptance/`
- `.agents/archive/planning-corpus/adr/`
- `.task-bank/`

## Scope

- MVP coding-first desktop workspace with one selected local Git project, one
  active agent session, OpenRouter model access, hardened OpenCode adapter,
  backend-owned approvals, durable session/tool records, and review surfaces.
- Current V1 progress already recorded in migrated progress items, including
  session catalog, project selector status, and ordered nested `AGENTS.md`
  display guidance.

## Non-Goals

- Do not promote excluded MVP features such as plugins, marketplace, MCP,
  browser panels, concurrent agents, worktrees, direct provider integrations,
  or global search into the MVP.
- Do not use `.task-bank/` for new progress updates.

## Current State

Implementation progress through `item-034` is migrated from `.task-bank/` to
`.agents/progress/`, with all migrated items currently marked verified in the
manifest.

## Intended State

`.agents/specs/c4os-mvp/` is the durable AI-readable specification layer, and
`.agents/progress/` is the active execution bank.

## Definition Of Ready

- New implementation work links to relevant spec records and acceptance files.
- Progress updates happen in `.agents/progress/`.
- Source retirement decisions are explicit before old planning/progress
  sources are deleted.

