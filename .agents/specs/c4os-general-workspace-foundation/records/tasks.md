# Tasks

## TASK-000: Foundation Readiness Review

Status: accepted
Confidence: evidence-backed
MVP: no
Source: feature planning
Related:
- AC-001

### Statement

Review this feature spec for blockers, high-risk findings, missing evidence,
and scope ambiguity before creating implementation progress packets.

### Result

Completed; blockers were resolved by user decisions and follow-on planning.

## TASK-001: Define Workspace Home Slice

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-001
Related:
- AC-001

### Statement

Define the workspace-level home or overview for research, writing,
documentation, and analysis workflows.

### Result

Use a work-focused overview with project search/filtering, workflow-purpose
filtering, latest/resumable session state, and compact capability status.

## TASK-002: Define Workflow Organization Primitive

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-002
Related:
- AC-003

### Statement

Decide whether first-slice organization uses workflow labels, editable
metadata, session templates, or existing title/pin/archive primitives.

### Result

Use nullable bounded workflow-purpose labels on projects and sessions with no
automatic model-context effect.

## TASK-003: Define Project Navigation Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-003
Related:
- AC-004

### Statement

Choose which disabled project selector controls to promote for the first
foundation implementation.

### Result

Promote project search and workflow-purpose filtering first.

## TASK-004: Define Session Navigation Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-004
Related:
- AC-005

### Statement

Choose which disabled session catalog controls to promote for research,
writing, documentation, and analysis workflows.

### Result

Promote session search/filtering and workflow-purpose grouping first.

## TASK-005: Define General Workspace Status Surface

Status: accepted
Confidence: evidence-backed
MVP: no
Source: CAP-005
Related:
- AC-006

### Statement

Define user-visible status for accepted, deferred, and unsupported general
workspace capabilities.

### Result

Use capability-level copy only; do not claim broad compatibility, browser,
durable memory, import/round-trip, or hidden context behavior.

## TASK-006: Brownfield Data Model Review

Status: accepted
Confidence: evidence-backed
MVP: no
Source: ASM-002, RISK-004
Related:
- AC-003
- AC-004
- AC-005

### Statement

Review project/session/export/archive/delete persistence before freezing any
workspace metadata implementation.

### Result

Workflow-purpose labels should be nullable bounded metadata on projects and
sessions. Export may include labels as safe metadata; archived-session delete
removes session labels with the deleted session row; status needs explicit
capability flags.

## Frozen Progress Items

| Progress Item | Scope | Source Tasks |
| --- | --- | --- |
| item-053 | Workflow label data model, backend status, and safe export metadata | TASK-002, TASK-005, TASK-006 |
| item-054 | Project navigation search and workflow-purpose filtering | TASK-003 |
| item-055 | Session navigation search, filtering, and workflow-purpose grouping | TASK-004 |
| item-056 | Workspace overview UI and capability-level status copy | TASK-001, TASK-005 |
| item-057 | Foundation verification and release gate | TASK-000 |
