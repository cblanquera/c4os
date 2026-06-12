# Risks

## RISK-001: Safety Boundary Regression

Status: mitigated
Confidence: evidence-backed
MVP: no
Source: `CONTEXT.md`, `.agents/specs/c4os-mvp/records/requirements.md`
Related:
- REQ-003
- CON-001

### Statement

Browser, plugin, MCP, provider, import, and concurrency phases could introduce
execution paths that bypass the backend-owned safety boundary.

### Mitigation

Require phase-specific threat modeling and acceptance criteria before
implementation packets are created.

## RISK-002: Deferred Feature Scope Creep

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/risks.md`
Related:
- REQ-004

### Statement

Features intentionally deferred in V1 could be bundled into a broad roadmap
without accepted scope or evidence.

### Mitigation

Keep each deferred area as a separate proposed task with explicit acceptance
criteria and readiness review.

## RISK-003: Compatibility Overclaim

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/decisions.md`
Related:
- CON-002

### Statement

C4OS could imply full compatibility with AGENTS.md, Agent Skills, MCP, Codex
plugins, OpenCode config, or import/export before conformance behavior exists.

### Mitigation

Use capability-level language until compatibility tests and unsupported-feature
labels exist.

## RISK-004: Architecture Saturation

Status: mitigated
Confidence: inferred
MVP: no
Source: `.agents/progress/manifest.md`
Related:
- ASM-001

### Statement

The MVP architecture may need deeper runtime, storage, or UI changes before it
can support rich artifacts, concurrent agents, plugins, marketplace, and
memory.

### Mitigation

Run brownfield architecture review before accepting implementation phases.

## RISK-005: Standards Freshness Gap

Status: proposed
Confidence: evidence-backed
MVP: no
Source: readiness review, EVD-004 through EVD-008
Related:
- TASK-000
- Q-004

### Statement

The roadmap references fast-moving ecosystem surfaces, including MCP, Agent
Skills, AGENTS.md, Codex plugins, worktrees, and app/browser patterns. Stale
standards assumptions could produce incorrect compatibility, safety, or
interoperability scope.

### Mitigation

Run a standards refresh with dated primary-source evidence before accepting
broad compatibility, plugin/marketplace, MCP, browser, provider, or worktree
implementation phases.

### Result

Mitigated for roadmap sequencing by `item-049`. Deeper feature-specific source
review remains required before implementation.

## RISK-006: First General Workspace Audience Ambiguity

Status: proposed
Confidence: inferred
MVP: no
Source: readiness review
Related:
- Q-003
- TASK-001

### Statement

The original prompt lists several non-coding workflows, but the first general
workspace phase does not yet identify a primary workflow to optimize. Without
that priority, UX and acceptance criteria may become generic and hard to
verify.

### Mitigation

Choose the first non-coding workflow before creating active progress packets
for `TASK-001`.

### Result

User selected research, writing, documentation, and analysis on 2026-06-12.
