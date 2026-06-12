# item-054: Foundation Project Navigation Search And Workflow Filtering

## Status

verified

## Type

implementation

## Objective

Implement project selector support for project search and workflow-purpose
filtering.

## Dependencies

- item-053

## Spec Links

- CAP-003
- AC-004
- DEC-006
- DEC-008

## Scope

- Project selector can expose and apply search over registered Git projects.
- Project selector can expose and apply workflow-purpose filtering.
- Exactly one active selected project remains the execution context.
- Non-Git folders, cross-project views, archive/delete, favorites, and
  worktree management remain deferred.

## Acceptance Criteria

- Search/filtering does not change selected project unless user explicitly
  selects a project.
- Filtering operates on bounded workflow-purpose labels only.
- Capability/status flags accurately describe promoted and deferred controls.

## Verification

- Rust project selector tests passed.
- JS backend status tests passed.
- Full `cargo test`, full `npm test`, and `npm run build` passed.
