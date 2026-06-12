# Capabilities

## CAP-001: Workspace Home

Status: proposed
Confidence: proposed
MVP: no
Source: TASK-001 roadmap
Related:
- REQ-001
- AC-001
- TASK-001

### Statement

Provide a workspace-level home or overview that helps users resume research,
writing, documentation, and analysis work across registered projects and
sessions.

## CAP-002: Workflow-Purpose Organization

Status: proposed
Confidence: proposed
MVP: no
Source: user audience decision
Related:
- REQ-003
- AC-003
- TASK-002

### Statement

Allow sessions or projects to be organized by purpose such as research,
writing, documentation, and analysis, without automatic context ingestion or
hidden behavior changes.

## CAP-003: Improved Project Navigation

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/project_selector.rs`
Related:
- AC-004
- TASK-003

### Statement

Improve project navigation beyond the current one-active-project selector while
staying within accepted constraints.

### Current State

The selector can list registered projects and select exactly one active
project. Search, grouping, archive, delete, favorites, metadata editing,
cross-project views, non-Git projects, and worktree management are unavailable.

## CAP-004: Improved Session Navigation

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `src-tauri/src/session_catalog.rs`
Related:
- AC-005
- TASK-004

### Statement

Improve session navigation and organization for repeated research, writing,
documentation, and analysis work.

### Current State

The session catalog can list project sessions, identify the latest session,
reject cross-project selection, and block new runs while another session is
active. Search, archive, delete, and concurrent active sessions are unavailable
from the catalog state.

## CAP-005: General Workspace Status Surface

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `src/main.js`, `tests/backend-command-boundary.test.mjs`
Related:
- AC-006
- TASK-005

### Statement

Expose clear status for accepted and deferred general-workspace capabilities so
users can see what is available for research/writing/docs/analysis workflows.
