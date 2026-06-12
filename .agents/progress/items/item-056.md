# item-056: Foundation Workspace Overview UI And Status Copy

## Status

verified

## Type

implementation

## Objective

Implement the first work-focused workspace overview and capability-level status
copy for the general workspace foundation slice.

## Dependencies

- item-053
- item-054
- item-055

## Spec Links

- CAP-001
- CAP-005
- AC-001
- AC-006
- AC-007
- DEC-011
- DEC-012

## Scope

- UI home/overview supports project search/filtering, workflow-purpose
  filtering, latest/resumable session state, and compact capability status.
- Include concrete flows for research, writing, documentation, and analysis.
- Capability copy describes available/deferred/unsupported behavior without
  making broad compatibility or hidden-context claims.

## Non-Goals

- Do not add global content search.
- Do not add browser ingestion, memory, import, provider expansion, plugins,
  worktrees, or concurrent agents.

## Acceptance Criteria

- Text fits existing responsive shell and does not turn the UI into a planning
  document.
- Available/deferred status matches backend capability flags.
- Workflow examples are represented in UI behavior or tests.

## Verification

- JS UI/state tests passed.
- Backend status tests passed.
- Full `npm test`, full `cargo test`, and `npm run build` passed.
- Browser smoke confirmed the workspace overview renders on desktop and mobile
  without horizontal overflow.
