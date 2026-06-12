# Decisions

## DEC-001: Create Separate Post-V1 Roadmap Spec

Status: proposed
Confidence: inferred
MVP: no
Source: user request, `.agents/specs/c4os-mvp/status.md`
Related:
- REQ-004

### Statement

Use `c4os-general-workspace-roadmap` as a separate planning spec for broad
post-V1 phases instead of expanding the verified `c4os-mvp` spec.

### Rationale

The MVP/V1 spec is verified and bounded. The broader original product vision
needs planning without silently changing accepted MVP/V1 scope.

## DEC-002: Roadmap Phases Are Proposed Until Reviewed

Status: proposed
Confidence: inferred
MVP: no
Source: `.agents/specs/c4os-mvp/records/decisions.md`
Related:
- TASK-000

### Statement

All phase tasks in this spec are proposed planning records until a readiness
review accepts, revises, rejects, or splits them.

## DEC-003: Standards Refresh Precedes Compatibility Claims

Status: proposed
Confidence: inferred
MVP: no
Source: user-provided original project prompt, `CON-002`
Related:
- TASK-000
- TASK-006

### Statement

Before accepting broad compatibility, plugin, marketplace, MCP, or provider
claims, refresh current standards evidence and update records with source
dates and conformance gaps.

## DEC-004: Validation Recommends Standards Refresh First

Status: accepted
Confidence: evidence-backed
MVP: no
Source: validation pass, user decision 2026-06-12
Related:
- Q-001
- TASK-000

### Statement

The validation recommendation is to accept `TASK-000` as the next phase before
creating implementation packets for general workspace foundation, portability,
browser/artifacts, providers, worktrees, plugins, memory, or advanced policy.

### Rationale

The roadmap depends on fast-moving standards and product surfaces. A scoped
refresh is lower risk than starting implementation with stale compatibility or
security assumptions.

## DEC-006: Accept First General Workspace Audience Priorities

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-003
- TASK-001

### Statement

The first general-workspace foundation phase should optimize for research,
writing, documentation, and analysis workflows.

### Rationale

These workflows best match the original prompt's non-coding product goals and
provide a coherent first audience for workspace navigation, sessions,
artifacts, browser/research surfaces, and knowledge workflows.

## DEC-005: Validation Recommends Feature Spec Split

Status: accepted
Confidence: evidence-backed
MVP: no
Source: validation pass, standards refresh 2026-06-12
Related:
- Q-002
- TASK-003
- TASK-004
- TASK-005
- TASK-006
- TASK-007

### Statement

Keep this roadmap as the cross-phase index, but create separate feature specs
before implementation for browser/web artifacts, provider expansion,
worktrees/concurrent agents, plugin/marketplace, and durable memory.

### Rationale

Each area carries enough safety, compatibility, UX, and architecture risk to
need its own accepted scope and readiness review.

## DEC-007: Standards Refresh Requires Feature-Specific Deep Dives

Status: accepted
Confidence: evidence-backed
MVP: no
Source: item-049 standards refresh
Related:
- TASK-000
- TASK-003
- TASK-004
- TASK-005
- TASK-006
- TASK-007

### Statement

The shallow standards refresh is sufficient to unblock roadmap sequencing, but
browser/web artifacts, provider expansion, worktrees/concurrent agents,
plugin/marketplace, and durable memory each require a separate feature spec
with deeper current-source review before implementation.

### Rationale

Primary-source evidence confirms these surfaces are active and security-
sensitive. A single roadmap record is not detailed enough to safely freeze
feature behavior.
