# item-034: TASK-033 Resolve V1 Nested AGENTS Resolution Scope

## Status

blocked

## Objective

Resolve the standards, product, permission, and UX decisions required before
implementing nested `AGENTS.md` resolution beyond MVP instruction-source
disclosure.

## Dependencies

- item-011
- item-033

## Blocker

Nested `AGENTS.md` resolution is not safe to infer from current repository
context. The app already inventories root and nested instruction sources for
MVP disclosure, but V1 resolution requires precedence, conflict diagnostics,
effective-instruction UI, and permission behavior decisions.

## Evidence

- `plans/decisions/deferred-decisions.md` defers nested `AGENTS.md` precedence
  and conflict diagnostics to V1.
- `plans/decisions/implementation-readiness-gaps.md` lists effective
  instruction stack UI and whether `AGENTS.md` can influence permissions as
  missing information.
- `plans/spikes/SPIKE-006-standards-conformance-matrix.md` requires AGENTS.md
  conformance tiers before broader compatibility claims.
- `plans/acceptance/file-access.md` currently keeps root `AGENTS.md` reads and
  writes under normal project-root file rules, without precedence or permission
  behavior.

## Exact User Action Needed

Decide whether current V1 should implement nested `AGENTS.md` resolution. If
yes, provide or approve acceptance criteria for:

- AGENTS.md support tier: display, parse, guidance, nested resolution, export,
  or compatibility claim.
- Precedence between root, nested, OpenCode-native, and app-owned instruction
  sources.
- Effective-instruction UI and conflict diagnostics.
- Whether AGENTS.md can ever affect permissions.
- Behavior after an approved edit to root or nested `AGENTS.md`.

## Resume Prompt

Resolve TASK-033 with approved nested `AGENTS.md` resolution scope, update
acceptance criteria, then implement the narrowest Sprint 11 instruction
resolution slice.
