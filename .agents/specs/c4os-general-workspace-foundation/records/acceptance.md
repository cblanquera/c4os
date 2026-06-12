# Acceptance Criteria

## AC-001: Foundation Scope Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: REQ-001, item-051
Related:
- TASK-000

### Statement

Readiness review confirms this feature spec is limited to general workspace
foundation for research, writing, documentation, and analysis, with deferred
feature boundaries intact.

### Result

Accepted after blockers were resolved and workflow examples were defined.

## AC-002: Safety Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: REQ-002
Related:
- CON-001

### Statement

The feature preserves one active run, backend-owned approvals, scoped file
access, shell/Git controls, credential handling, and explicit model-context
rules.

### Result

Accepted. Workflow labels and navigation changes do not alter active-run,
approval, file, shell, Git, credential, or model-context rules.

## AC-003: Workflow Organization Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: REQ-003, item-050 brownfield data-model review
Related:
- TASK-002

### Statement

The feature defines the workflow organization primitive and states its
context-effect, persistence, export, and deletion behavior.

### Result

Workflow-purpose labels are nullable bounded metadata on projects and
sessions, have no model-context effect, are safe to export as metadata, and are
deleted with sessions when stored on session rows.

## AC-004: Project Navigation Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-003, user decision 2026-06-12, item-050
Related:
- TASK-003

### Statement

The feature defines which project selector controls are promoted, which remain
deferred, and how each affects selected project state.

### Result

Project search and workflow-purpose filtering are promoted first. Non-Git
folders, cross-project views, archive/delete, and favorites remain deferred.

## AC-005: Session Navigation Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-004, user decision 2026-06-12, item-050
Related:
- TASK-004

### Statement

The feature defines session navigation, filtering, grouping, and metadata
behavior without enabling concurrent active sessions.

### Result

Session search/filtering and workflow-purpose grouping are promoted first.
Concurrent active sessions remain deferred, and delete remains limited to the
accepted archived-session-delete tier.

## AC-006: Status Surface Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-005, DEC-012
Related:
- TASK-005

### Statement

The feature exposes clear user-facing status for available, deferred, and
unsupported workspace capabilities.

### Result

Accepted with capability-level copy only and no broad compatibility or hidden
context claims.

## AC-007: Workflow Example Acceptance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-051
Related:
- TASK-001

### Statement

The feature defines concrete examples for each target workflow:

- Research: resume a research project by filtering to `research`, opening the
  latest related session, and seeing linked artifacts without browser or memory
  ingestion.
- Writing: filter to `writing`, resume a draft-focused session, and inspect
  text-like artifacts and changed files.
- Documentation: filter to `documentation`, resume a docs review session, and
  use project/session status without claiming full AGENTS.md or Skills
  compatibility.
- Analysis: filter to `analysis`, compare session titles/statuses and artifacts
  within a selected project without cross-project content search.
