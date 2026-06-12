# Risks

## RISK-001: Safety Boundary Regression

Status: mitigated
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/requirements.md`
Related:
- CON-001

### Statement

General workspace UX could imply broader file, shell, Git, or context access
than the backend actually permits.

### Mitigation

Keep status surfaces explicit and route any new action through existing
backend-owned policy.

## RISK-002: Deferred Scope Creep

Status: proposed
Confidence: evidence-backed
MVP: no
Source: roadmap DEC-005 and DEC-007
Related:
- CON-002

### Statement

Research and writing workflows naturally pull toward browser, document
preview, durable memory, and import/export scope.

### Mitigation

Record those needs as follow-on specs instead of adding them to this
foundation slice.

## RISK-003: Generic Workspace UX

Status: mitigated
Confidence: evidence-backed
MVP: no
Source: readiness review
Related:
- REQ-001

### Statement

If research, writing, documentation, and analysis workflows are not translated
into concrete navigation and session needs, the feature may become a generic
dashboard with weak acceptance criteria.

### Mitigation

Require workflow-specific acceptance examples before implementation packets.

### Result

Workflow examples were defined in `AC-007`.

## RISK-004: Data Model Drift

Status: proposed
Confidence: inferred
MVP: no
Source: current code inspection
Related:
- ASM-002

### Statement

Adding workspace metadata without a data-model review could conflict with
existing project/session persistence, export, archive/delete, and future import
semantics.

### Mitigation

Run brownfield data-model review before freezing implementation tasks.

### Result

Item-050 recommends nullable bounded workflow-purpose labels on projects and
sessions, with export/status contract updates and no message/artifact storage.

## RISK-005: Non-Git Scope Expansion

Status: proposed
Confidence: evidence-backed
MVP: no
Source: readiness review
Related:
- Q-001

### Statement

Allowing non-Git folders in the first general workspace slice would require
revisiting project registration, Git status assumptions, file boundaries,
exports, and status copy.

### Mitigation

Defer non-Git folders unless explicitly accepted as first-slice scope.

### Result

User accepted deferral on 2026-06-12.
