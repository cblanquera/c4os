# Questions

## Q-001: Next Accepted Phase

Status: accepted
Confidence: evidence-backed
MVP: no
Source: roadmap planning, validation pass, user decision 2026-06-12
Related:
- TASK-000
- TASK-001

### Statement

Should the next accepted phase be general workspace foundation, portability and
import, or a standards refresh before feature planning?

### Validation

Recommended answer: accept a scoped standards refresh first, then choose the
first implementation phase. This still needs explicit user acceptance because
it changes active roadmap order.

### Decision

User accepted the recommendation to run `TASK-000` standards refresh first on
2026-06-12.

## Q-002: Spec Split Boundaries

Status: accepted
Confidence: evidence-backed
MVP: no
Source: roadmap planning, validation pass, standards refresh 2026-06-12
Related:
- TASK-000

### Statement

Should all post-V1 phases stay under this roadmap spec until accepted, or
should high-risk areas such as plugin marketplace, browser/web, providers, and
memory become separate feature specs after readiness review?

### Validation

Recommended answer: keep this roadmap as the cross-phase index, then split
browser/web artifacts, providers, worktrees/concurrent agents, plugins/
marketplace, and memory into feature specs before implementation.

### Decision

Accepted for roadmap sequencing in `DEC-005` and `DEC-007`.

## Q-003: Target User Priority

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user-provided original project prompt, validation pass, user decision 2026-06-12
Related:
- REQ-001

### Statement

Which non-coding workflow should shape the first general workspace phase:
writing, research, analysis, operations, documentation, or another workflow?

### Validation

Manual user decision required. Repository evidence cannot determine the target
workflow priority from the original prompt because the prompt intentionally
lists several workflow classes.

### Decision

User selected research, writing, documentation, and analysis as the first
general-workspace audience priorities on 2026-06-12.

## Q-004: Standards Refresh Scope

Status: accepted
Confidence: evidence-backed
MVP: no
Source: readiness review, validation pass, item-049 standards refresh
Related:
- TASK-000
- RISK-005

### Statement

Which standards and product surfaces must be refreshed before phase acceptance:
AGENTS.md, Agent Skills, MCP, Codex plugins, worktrees, providers, browser/web,
memory, or all of them?

### Validation

Recommended answer: refresh all listed surfaces at a shallow dated level first,
then do deeper source review inside each feature spec. The initial refresh
should include AGENTS.md, Agent Skills, MCP, Codex plugins, Codex worktrees,
provider models, browser/web surfaces, artifact previews, memory, permissions,
and local workspace/session patterns.

### Result

Resolved for roadmap sequencing by `item-049`. Deeper source review remains
required inside feature specs.
