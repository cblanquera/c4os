# Constraints

## CON-001: Backend-Owned Safety Boundary

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/requirements.md`
Related:
- REQ-002
- RISK-001

### Statement

The foundation slice must preserve backend authority over filesystem access,
shell execution, Git state-changing policy, process supervision, secrets,
approvals, persistence, and runtime adapter control.

## CON-002: Deferred Scope Remains Deferred

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-general-workspace-roadmap/records/decisions.md`
Related:
- RISK-002

### Statement

Browser/web artifacts, provider expansion, worktrees/concurrent agents,
plugin/marketplace, and durable memory require separate feature specs before
implementation.

## CON-003: No Broad Compatibility Claim

Status: proposed
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/decisions.md`
Related:
- RISK-003

### Statement

This foundation slice must not claim full compatibility with AGENTS.md, Agent
Skills, MCP, Codex plugins, OpenCode config, import/export, browser workflows,
or durable memory.
