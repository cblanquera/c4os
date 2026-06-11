# Decisions

## DEC-001: Use Frozen MVP As Implementation Contract

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-001
- CON-001

### Statement

`.agents/archive/planning-corpus/mvp/mvp-freeze.md` is the implementation contract for the MVP.

## DEC-002: Keep Runtime Adapter Hardened

Status: accepted
Confidence: evidence-backed
MVP: yes
Source: `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
Related:
- REQ-004

### Statement

Phase 1 must implement the hardened OpenCode adapter path, not unconstrained
direct OpenCode.

## DEC-003: Defer Worktree Lifecycle

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/items/item-032.md`, `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
Related:
- RISK-002

### Statement

Worktree creation and cleanup are deferred beyond current V1.

## DEC-004: Promote `.agents` As Active Agent Layer

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user request, `.task-bank/DEPRECATED.md`
Related:
- CON-004

### Statement

Use `.agents/specs/` for durable planning records and `.agents/progress/` for
execution state. `.task-bank/` is deprecated and retained only as a historical
source until deletion is confirmed.

