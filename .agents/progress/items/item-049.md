# item-049: TASK-000 Roadmap Standards Refresh

## Status

verified

## Objective

Refresh current standards and product-surface evidence before accepting broad
post-V1 compatibility, ecosystem, plugin, MCP, browser, provider, worktree,
memory, or workspace implementation phases.

## Accepted Decisions

- Next phase: `TASK-000` standards refresh first.
- First general-workspace audience priorities: research, writing,
  documentation, and analysis.
- Roadmap remains the cross-phase index.
- High-risk areas should split into feature specs before implementation:
  browser/web artifacts, provider expansion, worktrees/concurrent agents,
  plugin/marketplace, and durable memory.

## Dependencies

- item-048
- `.agents/specs/c4os-general-workspace-roadmap/`

## Inputs

- `CONTEXT.md`
- `.agents/specs/c4os-mvp/`
- `.agents/specs/c4os-general-workspace-roadmap/`
- `.agents/archive/planning-corpus/`
- Current primary sources for AGENTS.md, Agent Skills, MCP, Codex plugins,
  Codex worktrees, providers, browser/web surfaces, artifact previews, memory,
  permissions, and local workspace/session patterns.

## Deliverables

- Dated evidence records for each refreshed standard or product surface.
- Updated risks, questions, decisions, acceptance, and tasks in
  `.agents/specs/c4os-general-workspace-roadmap/`.
- Recommendation on which feature specs should be created next.
- Updated readiness findings and status.

## Acceptance Criteria

- AGENTS.md and nested instruction conventions are refreshed from primary
  sources.
- Agent Skills conventions are refreshed from primary sources.
- MCP and MCP security/approval guidance are refreshed from primary sources.
- Codex plugin, worktree, permission, browser/app, skills, and MCP surfaces are
  refreshed from primary sources where available.
- Provider expansion, browser/web, artifact, memory, and local workspace/session
  patterns have dated source notes or explicit evidence gaps.
- The roadmap records clearly distinguish accepted scope, proposed scope,
  deferred scope, and evidence gaps.

## Verification

- Review changed `.agents/specs/c4os-general-workspace-roadmap/` records and
  indexes for traceability.
- Confirm `reviews/findings.md` and `status.md` reflect the post-refresh
  readiness state.

## Verification Evidence

- Added dated evidence records `EVD-012` through `EVD-022`.
- Updated decisions, risks, acceptance, findings, status, and indexes.
- Confirmed roadmap status routes next work to a feature spec for `TASK-001`.
