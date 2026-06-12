# item-050: Foundation Brownfield Data Model Review

## Status

verified

## Objective

Review current project/session/export/archive/delete persistence before
freezing implementation tasks for the general workspace foundation feature.

## Accepted Decisions

- Non-Git folders are deferred for the first foundation slice.
- Organization uses lightweight workflow-purpose labels with explicit
  no-context-effect semantics.
- Project navigation promotes search and workflow-purpose filtering first.
- Session navigation promotes search/filtering and workflow-purpose grouping
  first.

## Dependencies

- item-049
- `.agents/specs/c4os-general-workspace-foundation/`

## Inputs

- `src-tauri/src/storage.rs`
- `src-tauri/src/project_selector.rs`
- `src-tauri/src/session_catalog.rs`
- `src-tauri/src/export.rs`
- `src-tauri/src/artifact.rs`
- `tests/backend-command-boundary.test.mjs`
- `.agents/specs/c4os-general-workspace-foundation/records/`

## Deliverables

- Current-state notes for project/session/workflow metadata implications.
- Updated evidence, risks, acceptance, and tasks in
  `.agents/specs/c4os-general-workspace-foundation/`.
- Recommendation for whether workflow-purpose labels belong on projects,
  sessions, both, or a separate table.
- Updated readiness findings and status.

## Acceptance Criteria

- Project table and session table impacts are reviewed.
- Export behavior is reviewed for workflow labels.
- Archived-session delete behavior is reviewed for workflow labels.
- Status contract impacts are reviewed.
- No implementation task is frozen until the data-model recommendation is
  recorded.

## Verification

- Review changed foundation spec records and indexes for traceability.
- Confirm `reviews/findings.md` and `status.md` reflect the post-review
  readiness state.

## Verification Evidence

- Reviewed project and session schema, export service, archived-session delete,
  project selector, session catalog, backend status contract, and frontend
  command registry.
- Updated foundation evidence, decisions, risks, acceptance, tasks, findings,
  status, and indexes.
- Confirmed next recommended action is workflow-specific acceptance examples
  and workspace home shape.
