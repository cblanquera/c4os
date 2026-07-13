# C4OS Agent Workspace Migration

Status: complete
Started: 2026-07-13
Legacy snapshot: Git commit `7f57cfe220be43c06668f53850bf598b2c563477`

## Purpose

Track the clean-room replay from the legacy C4OS Agent Workspace into the managed `chrisai-agents` contract without treating archived instructions, planning state, or historical execution records as current truth.

## Raw Source

- [Legacy C4OS Agent Workspace operating contract](resources/history/AGENTS.md): load when auditing the prior folder contract, workflow routing, source boundaries, or record rules.
- [Legacy context product gate](resources/history/context/product-brief.md): load when checking whether accepted reusable product truth was promoted accurately.
- [Legacy spec manifest](resources/history/specs/manifest.md): load when checking historical and proposed spec coverage.
- [Legacy completed MVP execution manifest](resources/history/development/mvp/manifest.md): load when checking completed implementation history or whether an old task was already accepted.

The retained legacy source is organized by purpose rather than kept as a second Agent Workspace:

- `resources/grill/`: 134 exact JSON grill answers and schemas.
- `resources/research/`: 11 exact research and intake Markdown records.
- `resources/history/`: 199 exact historical context, spec, execution, supporting-context, and operating-contract records.

These 344 files remain Raw Source and must not be followed as current instructions. The 18 superseded legacy workflow files were intentionally retired after C4OS-specific behavior was reconciled into the active workflow surface.

## Coverage Ledger

| Legacy surface | Files | Disposition |
| --- | ---: | --- |
| `context/` | 5 | Preserve under `resources/history/context/`; rebuild accepted reusable truth behind the active `context/index.md`. |
| `specs/` | 147 | Preserve under `resources/history/specs/`; rebuild completed MVP history plus the 11 proposed spec packages under the managed SDD lifecycle. |
| `development/` | 26 | Preserve under `resources/history/development/`; do not restore the legacy future-execution model. |
| `references/` | 165 | Route 134 JSON records to `resources/grill/`, 11 research records to `resources/research/`, and 20 historical context records to `resources/history/references/context/`. |
| `workflows/` | 18 | Retire after restoring only C4OS-specific behavior not owned by the managed workflows. |
| `AGENTS.md` | 1 | Preserve under `resources/history/AGENTS.md`; supersede it with the managed operating contract plus C4OS Project Rules. |

## Replay Rules

1. Context is rebuilt before specs.
2. Accepted reusable truth is promoted without shortening away constraints, examples, or stakeholder intent.
3. Exact user answers remain preserved in Raw Source and are normalized into decisions only without changing meaning.
4. Completed MVP work remains historical and must not become open implementation work.
5. Proposed final-implementation specs remain Proposed until the managed Freeze criteria are satisfied.
6. New implementation planning uses spec-local `tasks/`; the legacy `.agents/development/<spec-id>/` model is not restored.
7. Every promoted Reference File must be flat, numbered, descriptively linked, and owned by another Agent File.

## Current Phase

- Managed baseline: complete and validated.
- Legacy preservation: complete.
- Context replay: complete and validated.
- Historical MVP replay: complete as Frozen history.
- Proposed spec replay: complete as Proposed managed packages.
- C4OS workflow reconciliation: complete.
- Legacy resource reorganization: complete; 344 retained files were routed by purpose and 18 superseded workflows were retired.
- External proof and primary wireframe link reconciliation: complete; historical r05 review notes retain a documented legacy-ID mapping.
- Final deterministic validation: passed without warnings after resource retirement.

## Closeout Evidence

- Managed installer dry run: no changes needed.
- Deterministic validator: passed without warnings after resource retirement.
- Retained resource integrity: all 344 retained files matched their legacy-source SHA-256 hashes before retirement.
- Diff whitespace check: passed.
- Implementation authorization: none; specs `00002` through `00012` remain Proposed.
