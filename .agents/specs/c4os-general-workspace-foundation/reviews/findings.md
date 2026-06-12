# Readiness Findings

## Finding Counts

| Severity | Open | Resolved |
| --- | ---: | ---: |
| BLOCKER | 0 | 4 |
| HIGH | 0 | 2 |
| MEDIUM | 0 | 2 |
| LOW | 0 | 0 |
| QUESTION | 4 | 0 |

## BLOCKER-001: Non-Git Folder Scope Is Undecided

Status: resolved
Related:
- Q-001
- CAP-003
- TASK-003

Research, writing, documentation, and analysis workflows often happen outside
Git repositories, but the current project selector explicitly reports
`non_git_projects_allowed: false`. Promoting non-Git folders would affect file
boundaries, project registration, Git status assumptions, exports, and UX.

Recommendation: defer non-Git folders for the first foundation slice unless
the user explicitly wants that as the primary unlock.

Resolution: user accepted deferral on 2026-06-12.

## BLOCKER-002: Organization Primitive Is Undecided

Status: resolved
Related:
- Q-002
- CAP-002
- TASK-002

The spec does not yet decide whether the first foundation slice should use
workflow labels, editable metadata, templates, or only existing title/pin/
archive primitives. Implementation cannot be scoped until this is chosen.

Recommendation: use lightweight workflow-purpose labels first, with explicit
no-context-effect semantics.

Resolution: user accepted lightweight workflow-purpose labels on 2026-06-12.

## BLOCKER-003: Project Navigation Scope Is Undecided

Status: resolved
Related:
- Q-003
- CAP-003
- TASK-003

The project selector has several disabled controls. The feature spec must pick
a narrow first set instead of enabling search, grouping, favorites, metadata
editing, cross-project views, and archive all at once.

Recommendation: promote search and workflow-purpose filtering first; defer
cross-project views, archive/delete, favorites, and non-Git support.

Resolution: user accepted project search and workflow-purpose filtering first
on 2026-06-12.

## BLOCKER-004: Session Navigation Scope Is Undecided

Status: resolved
Related:
- Q-004
- CAP-004
- TASK-004

The session catalog has disabled search/archive/delete controls and no workflow
grouping. The feature spec must choose a narrow first set without implying
concurrent active sessions.

Recommendation: promote session search/filtering and workflow-purpose grouping
first; keep delete behavior limited to the already accepted archived-session
delete tier.

Resolution: user accepted session search/filtering and workflow-purpose
grouping first on 2026-06-12.

## HIGH-001: Brownfield Data Model Review Required

Status: resolved
Related:
- RISK-004
- TASK-006

Workflow labels or metadata could affect projects, sessions, exports,
archived-session delete, future import, and status contracts. A data-model
review is needed before implementation tasks are frozen.

Resolution: item-050 completed the brownfield data-model review. Workflow
labels should be nullable bounded metadata on projects and sessions, exported
as safe metadata, deleted with session rows when session-scoped, and exposed in
status flags with no hidden context effect.

## HIGH-002: Workflow-Specific Acceptance Examples Missing

Status: resolved
Related:
- RISK-003
- AC-001
- TASK-001

The spec names research, writing, documentation, and analysis, but does not yet
define concrete examples such as research project resume, writing draft
iteration, documentation review, or analysis session comparison.

Resolution: item-051 defined examples for research project resume, writing
draft iteration, documentation review, and analysis session comparison.

## MEDIUM-001: Workspace Home Shape Is Undefined

Status: resolved
Related:
- CAP-001
- TASK-001

The spec does not yet decide whether the first screen is a project/session
overview, selected-project dashboard, activity feed, or task-oriented launcher.

Resolution: item-051 accepted a work-focused overview with project
search/filtering, workflow-purpose filtering, latest/resumable session state,
and compact capability status.

## MEDIUM-002: Status Surface Needs Product Copy Boundary

Status: resolved
Related:
- CAP-005
- TASK-005

The feature should expose available/deferred capability status without turning
the UI into a planning document or making broad compatibility claims.

Resolution: item-051 accepted capability-level status copy with no broad
compatibility, browser, durable memory, import/round-trip, or hidden-context
claims.

## Questions

- Q-001: Resolved 2026-06-12; defer non-Git folders.
- Q-002: Resolved 2026-06-12; lightweight workflow-purpose labels.
- Q-003: Resolved 2026-06-12; project search and workflow-purpose filtering.
- Q-004: Resolved 2026-06-12; session search/filtering and workflow-purpose
  grouping.
