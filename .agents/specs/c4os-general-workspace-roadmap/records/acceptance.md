# Acceptance Criteria

## AC-001: Roadmap Readiness

Status: accepted
Confidence: evidence-backed
MVP: no
Source: roadmap planning, item-049 standards refresh
Related:
- TASK-000

### Statement

The roadmap is ready for implementation planning only after readiness review
finds no unresolved blockers and any high-risk findings are resolved, accepted,
or explicitly deferred.

### Result

Accepted for roadmap sequencing after user decisions and shallow standards
refresh. The roadmap is not fully frozen for implementation; high-risk feature
areas require separate specs.

## AC-002: General Workspace Foundation Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-001`
Related:
- TASK-001

### Statement

The phase defines target non-coding workflows, multi-project/session behavior,
project defaults, navigation, and exclusions clearly enough to create progress
items without promoting concurrency, plugins, browser, or memory implicitly.

## AC-003: Portability Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-002`
Related:
- TASK-002

### Statement

The phase defines import/export data boundaries, unsupported-feature behavior,
secret exclusion, artifact payload policy, compatibility labels, and recovery
semantics.

## AC-004: Browser And Artifact Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-003`
Related:
- TASK-003

### Statement

The phase defines passive versus active rendering, remote URL handling,
Chromium/Tauri WebView tradeoffs, content ingestion, preview isolation, and
model-context disclosure.

## AC-005: Provider Expansion Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-004`
Related:
- TASK-004

### Statement

The phase defines direct/local provider scope, credential isolation, model
metadata, switching/fallback rules, outage handling, and user-facing
disclosures.

## AC-006: Worktree And Concurrent Agent Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-005`
Related:
- TASK-005

### Statement

The phase defines worktree lifecycle, cleanup, conflict detection, process
supervision, per-agent approval queues, and UI states for concurrent work.

## AC-007: Plugin And Marketplace Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-006`
Related:
- TASK-006

### Statement

The phase defines plugin package format, trust model, permission review,
plugin-provided MCP behavior, install/update flows, marketplace metadata, and
curation/advisory behavior.

## AC-008: Memory Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-007`
Related:
- TASK-007

### Statement

The phase defines durable memory data model, provenance, retrieval behavior,
inspect/edit/delete controls, embedding policy, and model-context disclosure.

## AC-009: Advanced Policy Acceptance

Status: proposed
Confidence: proposed
MVP: no
Source: `CAP-008`
Related:
- TASK-008

### Statement

The phase defines policy scope, audit log guarantees, support-bundle redaction,
administrative controls, and migration from the existing approval gateway.
