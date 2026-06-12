# Requirements

## REQ-001: General-Purpose Workspace

Status: proposed
Confidence: proposed
MVP: no
Source: user-provided original project prompt
Related:
- CAP-001
- AC-001

### Statement

C4OS should evolve from a coding-first MVP into a general-purpose AI workspace
for coding, writing, research, analysis, operations, documentation, and other
agent-assisted workflows.

### Rationale

The original project goal was broader than software engineering, but the
verified MVP intentionally narrowed scope to validate the local coding loop.

## REQ-002: Standards-Aligned Extensibility

Status: proposed
Confidence: proposed
MVP: no
Source: user-provided original project prompt
Related:
- CAP-006
- CAP-007
- AC-006
- AC-007

### Statement

Future extensibility should prefer adopted ecosystem conventions for AGENTS.md,
Agent Skills, MCP, plugins, and portable workflows over proprietary formats
when practical.

### Rationale

Interoperability was an explicit architectural goal, but current V1 decisions
only accept narrow feature-level support and defer broad compatibility claims.

## REQ-003: Preserve Backend-Owned Safety

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `CONTEXT.md`, `.agents/specs/c4os-mvp/records/requirements.md`
Related:
- CON-001
- RISK-001
- AC-009

### Statement

Every post-V1 phase must preserve backend authority over filesystem access,
shell execution, Git state-changing policy, process supervision, secrets,
approvals, persistence, and runtime adapter control.

### Rationale

The verified MVP trust model depends on the UI reflecting enforced backend
state rather than becoming the policy authority.

## REQ-004: Explicit Promotion Of Deferred Capabilities

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/decisions.md`
Related:
- DEC-001
- RISK-002

### Statement

Capabilities deferred by V1 decisions must receive their own accepted scope,
acceptance criteria, and validation evidence before implementation work begins.

### Rationale

The current V1 record set explicitly deferred worktrees, browser/web viewing,
plugin/marketplace workflows, direct/local providers, durable memory, and broad
compatibility claims.
