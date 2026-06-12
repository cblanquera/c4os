# Assumptions

## ASM-001: Workflow Labels Are Enough For First Slice

Status: accepted
Confidence: evidence-backed
MVP: no
Source: current code inspection
Related:
- Q-002
- TASK-002

### Statement

The first general workspace foundation can likely use workflow-purpose labels
and navigation changes rather than adding browser, memory, or non-Git folder
support immediately.

## ASM-002: Existing Project And Session Tables Can Extend

Status: proposed
Confidence: inferred
MVP: no
Source: `src-tauri/src/storage.rs`, `src-tauri/src/project_selector.rs`, `src-tauri/src/session_catalog.rs`
Related:
- RISK-004

### Statement

Existing project and session persistence can likely support additional
workspace metadata, but a brownfield data-model review is required before
freezing implementation tasks.

### Result

Confirmed by item-050 for nullable bounded workflow-purpose labels on projects
and sessions, subject to migration and status/export contract updates.
