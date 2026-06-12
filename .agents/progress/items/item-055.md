# item-055: Foundation Session Navigation Search Filtering And Grouping

## Status

verified

## Type

implementation

## Objective

Implement session catalog support for search/filtering and workflow-purpose
grouping.

## Dependencies

- item-053

## Spec Links

- CAP-004
- AC-005
- DEC-007
- DEC-008

## Scope

- Session catalog can expose and apply search/filtering within a selected
  project.
- Session catalog can group or filter sessions by workflow-purpose labels.
- One-active-run behavior remains unchanged.
- Delete remains limited to the archived-session-delete tier.

## Acceptance Criteria

- Cross-project session selection remains rejected.
- Search/filtering does not imply cross-session memory or content indexing.
- Concurrent active sessions remain unavailable.

## Verification

- Rust session catalog tests passed.
- Existing retention/delete tests passed.
- Full `cargo test`, full `npm test`, and `npm run build` passed.
