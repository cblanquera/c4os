# Requirements

## REQ-001: General Workspace Foundation Slice

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-general-workspace-roadmap/records/tasks.md`, user decision 2026-06-12
Related:
- CAP-001
- AC-001

### Statement

The first general workspace foundation should support research, writing,
documentation, and analysis workflows without requiring browser, memory,
plugin, provider expansion, worktree, or concurrent-agent scope.

### Current State

The app is a coding-first MVP shell with project registration, selected Git
project, prompt submission, latest transcript, status tiles, and backend-owned
safety controls.

### Intended State

The app provides workspace navigation and session organization that can be used
for research, writing, documentation, and analysis projects.

### Gap

The current UI and data model expose coding-first project/session concepts but
do not yet model workflow purpose, research/writing/docs/analysis organization,
or general workspace home navigation.

## REQ-002: Preserve Single Active Run Safety

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/requirements.md`, `src-tauri/src/session_catalog.rs`
Related:
- CON-001
- AC-002

### Statement

The foundation slice must preserve one active run at a time unless a later
concurrent-agent feature spec accepts and implements a different model.

### Current State

`SessionCatalog` exposes `concurrent_active_sessions: false`, and active
sessions block new runs.

### Intended State

General workspace navigation can show multiple projects and sessions, but
execution remains single-active-run.

### Gap

None for safety; future UX must avoid implying concurrent execution.

## REQ-003: Workflow Organization Without Hidden Context

Status: proposed
Confidence: inferred
MVP: no
Source: roadmap audience decision, `CONTEXT.md`
Related:
- CAP-002
- AC-003

### Statement

Workflow labels or organization for research, writing, documentation, and
analysis must not automatically inject files, artifacts, memory, browser
content, or hidden project context into model calls.

### Current State

Model context is limited to transcript, selected model/routing metadata,
approved tool results, and explicitly read project-root file contents.

### Intended State

General workspace organization improves navigation and intent without changing
model-context rules.

### Gap

Any future workflow metadata needs explicit context-effect semantics.
