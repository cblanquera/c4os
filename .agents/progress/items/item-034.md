# item-034: TASK-033 Resolve V1 Nested AGENTS Resolution Scope

## Status

verified

## Objective

Implement the approved current V1 nested `AGENTS.md` slice as ordered display
guidance beyond MVP instruction-source disclosure.

## Dependencies

- item-011
- item-033

## Scope Decision

Current V1 supports `display_guidance_order_only`.

- Root and nested `AGENTS.md` files are disclosed in an ordered guidance stack.
- Root guidance appears first, then nested files by path depth and path name.
- Nested entries disclose the path subtree they apply to.
- Conflict diagnostics are limited to ordered source disclosure.
- The app does not parse, semantically merge, rewrite, execute, export, or
  round-trip guidance.
- `AGENTS.md` files have no permission effect and are not added to app-owned
  model context unless explicitly read under normal project-root rules.

## Evidence

- `.agents/archive/planning-corpus/spikes/SPIKE-006-standards-conformance-matrix.md` now records the
  `display_guidance_order_only` tier.
- `.agents/archive/planning-corpus/acceptance/project-management.md` and
  `.agents/archive/planning-corpus/acceptance/file-access.md` now define nested `AGENTS.md` disclosure,
  no semantic merge, no permission effect, and next-preflight reload behavior.
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md` now keeps full AGENTS.md
  compatibility deferred while allowing the V1 ordered disclosure slice.

## Deliverables

- Instruction preflight records ordered nested `AGENTS.md` guidance.
- App status reports the supported tier and no permission or automatic model
  context effect.
- Planning documents and agent progress bank reflect the resolved scope.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml instruction_preflight`
  passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Handoff

TASK-033 is complete. Continue to the next documented sprint/task only after
checking the current sprint plan and task list for newly unblocked work.
