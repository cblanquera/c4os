# Tasks

## TASK-000: Roadmap Standards Refresh And Readiness Review

Status: accepted
Confidence: evidence-backed
MVP: no
Source: roadmap planning, validation pass, user decision 2026-06-12
Related:
- AC-001
- DEC-003
- DEC-004
- Q-001
- Q-002

### Statement

Refresh current standards evidence for AGENTS.md, Agent Skills, MCP, plugins,
marketplace conventions, provider models, browser/artifact safety, and local
workspace patterns, then run readiness review on this roadmap.

### Dependencies

- None.

### Notes

Validation recommends this as the next accepted phase. It is blocked only on
explicit user acceptance of the phase order and the standards refresh scope.

### Progress Mapping

- `.agents/progress/items/item-049.md`

### Refresh Result

Shallow dated standards refresh completed through `item-049`. Deeper review is
still required inside feature specs for browser/web artifacts, provider
expansion, worktrees/concurrent agents, plugin/marketplace, and durable memory.

## TASK-001: Phase 2 General Workspace Foundation

Status: proposed
Confidence: evidence-backed
MVP: no
Source: roadmap planning, user decision 2026-06-12
Related:
- CAP-001
- AC-002
- Q-003

### Statement

Plan and implement workspace-level home, multi-project navigation, session
organization, project defaults, and first-class non-coding project workflows.

### Dependencies

- TASK-000 acceptance or explicit user decision to proceed before review.
- Manual selection of the first non-coding workflow audience.

### Audience Priority

Research, writing, documentation, and analysis.

### Feature Spec

- `.agents/specs/c4os-general-workspace-foundation/`

## TASK-002: Phase 3 Import Export And Portability

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-002
- AC-003

### Statement

Plan and implement safe import, partial restore, compatibility labels, and
portable workspace state building on the verified project JSON export
foundation.

### Dependencies

- TASK-001 or explicit decision to prioritize portability first.

## TASK-003: Phase 4 Browser And Rich Artifact Viewing

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-003
- AC-004

### Statement

Plan and implement safe browser/web content viewing and richer artifact
previews with explicit rendering, ingestion, and model-context boundaries.

### Dependencies

- TASK-000.
- TASK-002 recommended for artifact portability alignment.

## TASK-004: Phase 5 Provider Expansion

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-004
- AC-005

### Statement

Plan and implement direct provider and/or local provider expansion beyond the
current OpenRouter-only path.

### Dependencies

- TASK-000.

## TASK-005: Phase 6 Worktrees And Concurrent Agents

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-005
- AC-006

### Statement

Plan and implement worktree lifecycle and multiple concurrent agents with
isolated work, conflict detection, and per-agent approval/process state.

### Dependencies

- TASK-001.
- Brownfield architecture review of runtime/process/session model.

## TASK-006: Phase 7 Plugin And Marketplace System

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-006
- AC-007

### Statement

Plan and implement plugin packaging, trust, permission review, plugin-provided
MCP, install/update workflows, and marketplace discovery/curation.

### Dependencies

- TASK-000.
- TASK-003 and TASK-004 may affect plugin surfaces.

## TASK-007: Phase 8 Memory And Knowledge Layer

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-007
- AC-008

### Statement

Plan and implement user-controlled durable memory, summaries, retrieval,
provenance, and inspect/edit/delete controls.

### Dependencies

- TASK-001.
- TASK-002 recommended for portability and deletion semantics.

## TASK-008: Phase 9 Advanced Policy And Audit

Status: proposed
Confidence: proposed
MVP: no
Source: roadmap planning
Related:
- CAP-008
- AC-009

### Statement

Plan and implement data-flow-aware policy, tamper-evident audit,
support-bundle redaction, and administrative controls.

### Dependencies

- TASK-003, TASK-004, TASK-005, TASK-006, or TASK-007 should provide concrete
  policy pressure before this becomes implementation work.
