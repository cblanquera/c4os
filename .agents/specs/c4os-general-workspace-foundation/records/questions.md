# Questions

## Q-001: Non-Git Project Support

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/project_selector.rs`, roadmap audience decision, user decision 2026-06-12
Related:
- CAP-003
- TASK-003

### Statement

Should research, writing, documentation, and analysis work allow non-Git local
folders in this foundation slice, or should non-Git project support remain
deferred?

### Decision

User accepted deferring non-Git local folders for the first foundation slice on
2026-06-12.

## Q-002: Organization Primitive

Status: accepted
Confidence: evidence-backed
MVP: no
Source: feature planning, user decision 2026-06-12
Related:
- CAP-002
- TASK-002

### Statement

Should workspace organization use workflow labels, user-editable metadata,
session templates, or only existing titles, pinned state, and archive state in
the first slice?

### Decision

User accepted lightweight workflow-purpose labels with explicit
no-context-effect semantics on 2026-06-12.

## Q-003: Project Navigation Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/project_selector.rs`, user decision 2026-06-12
Related:
- CAP-003
- TASK-003

### Statement

Which currently disabled selector controls should be promoted first: search,
grouping, favorites, metadata editing, cross-project views, or archive?

### Decision

User accepted project search and workflow-purpose filtering first on
2026-06-12. Cross-project views, archive/delete, favorites, and non-Git support
remain deferred.

## Q-004: Session Navigation Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/session_catalog.rs`, user decision 2026-06-12
Related:
- CAP-004
- TASK-004

### Statement

Which currently disabled session controls should be promoted first: search,
archive visibility, delete visibility, filters, or workflow-purpose grouping?

### Decision

User accepted session search/filtering and workflow-purpose grouping first on
2026-06-12. Delete behavior remains limited to the accepted
archived-session-delete tier.
