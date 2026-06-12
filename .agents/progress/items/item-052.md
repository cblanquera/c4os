# item-052: Freeze General Workspace Foundation Spec

## Status

verified

## Objective

Freeze `.agents/specs/c4os-general-workspace-foundation/` into
implementation-facing task records and `.agents/progress/` packets.

## Dependencies

- item-051
- `.agents/specs/c4os-general-workspace-foundation/`

## Inputs

- `.agents/specs/c4os-general-workspace-foundation/status.md`
- `.agents/specs/c4os-general-workspace-foundation/records/`
- `.agents/specs/c4os-general-workspace-foundation/indexes/traceability.md`
- `.agents/specs/c4os-general-workspace-foundation/reviews/findings.md`
- `.agents/progress/manifest.md`
- `.agents/progress/progress.csv`

## Deliverables

- Implementation-facing progress items for the accepted foundation slice.
- Updated progress manifest and CSV.
- Updated feature spec status and task mappings.
- Handoff identifying the first implementation item.

## Acceptance Criteria

- Frozen packets preserve accepted scope and deferred boundaries.
- Packets include data-model, backend status, project navigation, session
  navigation, UI/home, and verification work as appropriate.
- No packet promotes non-Git folders, browser/web, durable memory, provider
  expansion, plugins/marketplace, worktrees, or concurrent agents.

## Verification

- Progress manifest and item files include frozen implementation packets
  `item-053` through `item-057`.
- Feature status points to `item-053` as the first implementation item.
