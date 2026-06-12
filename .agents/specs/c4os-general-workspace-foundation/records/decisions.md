# Decisions

## DEC-001: Bound First Audience To Research Writing Documentation Analysis

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-general-workspace-roadmap/records/decisions.md`
Related:
- REQ-001

### Statement

The first general workspace foundation slice should optimize for research,
writing, documentation, and analysis workflows.

## DEC-002: Preserve Roadmap Feature Split

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-general-workspace-roadmap/records/decisions.md`
Related:
- CON-002

### Statement

This feature spec should not implement browser/web artifacts, provider
expansion, worktrees/concurrent agents, plugin/marketplace, or durable memory.

## DEC-003: Use Brownfield Current-State Records

Status: accepted
Confidence: evidence-backed
MVP: no
Source: current code inspection
Related:
- EVD-001
- EVD-002
- EVD-003

### Statement

Current code behavior is treated as current state; roadmap and user decisions
are treated as intended future state unless implementation evidence exists.

## DEC-004: Defer Non-Git Folders In First Foundation Slice

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-001
- CAP-003

### Statement

The first general workspace foundation slice keeps project registration limited
to local Git projects and defers non-Git local folders.

## DEC-005: Use Lightweight Workflow-Purpose Labels

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-002
- CAP-002

### Statement

The first organization primitive is lightweight workflow-purpose labels with
explicit no-context-effect semantics.

## DEC-006: Promote Project Search And Workflow Filtering First

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-003
- CAP-003

### Statement

The first project navigation promotion should add project search and
workflow-purpose filtering. Cross-project views, archive/delete, favorites,
and non-Git support remain deferred.

## DEC-007: Promote Session Search Filtering And Workflow Grouping First

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-004
- CAP-004

### Statement

The first session navigation promotion should add session search/filtering and
workflow-purpose grouping. Delete remains limited to the accepted
archived-session-delete tier.

## DEC-008: Store Workflow Purpose On Projects And Sessions

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-050 brownfield data-model review
Related:
- TASK-006
- CAP-002
- CAP-003
- CAP-004

### Statement

Workflow-purpose labels should be stored as nullable bounded metadata on both
projects and sessions, not in messages, artifacts, or hidden model-context
metadata.

### Rationale

Project labels support project search/filtering, session labels support
session grouping/filtering, and neither should affect transcript history,
artifact records, permissions, or model context.

## DEC-009: Export Workflow Labels As Safe Metadata Only

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-050 brownfield data-model review
Related:
- TASK-006
- CAP-005

### Statement

Project JSON export may include workflow-purpose labels as safe bounded
metadata while preserving the existing `project_json_export_only` tier:
import, round-trip compatibility, raw secrets, absolute local paths, and
artifact payloads remain excluded.

### Rationale

Workflow labels are useful organization metadata and are not secrets, but
including them must not imply import or round-trip compatibility.

## DEC-010: Workflow Labels Have No Context Effect

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12, item-050 brownfield data-model review
Related:
- REQ-003
- CAP-002

### Statement

Workflow-purpose labels are navigation and organization metadata only. They do
not automatically inject files, artifacts, memory, browser content, summaries,
or other hidden context into model calls.

## DEC-011: Use Work-Focused Workspace Overview As Home Shape

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-051 workflow examples and home-shape planning
Related:
- CAP-001
- TASK-001

### Statement

The first workspace home shape should be a work-focused overview that combines
project search/filtering, workflow-purpose filtering, latest/resumable session
state, and compact capability status.

### Rationale

This supports research, writing, documentation, and analysis workflows without
adding cross-project content search, browser ingestion, memory, or concurrent
agent behavior.

## DEC-012: Keep Status Copy Capability-Level

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-051 workflow examples and home-shape planning
Related:
- CAP-005
- TASK-005

### Statement

Status surface copy should describe available, deferred, and unsupported
capabilities at a feature level only. It must not claim standards
compatibility, browser capability, durable memory, import/round-trip support,
or hidden context behavior unless separate specs accept and verify those
capabilities.

### Rationale

The UI should guide users without becoming a planning document or implying
capabilities that remain deferred.
