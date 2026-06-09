# Finalized Decisions

This document summarizes ADR decisions that are settled for the MVP scope. Some remain provisional for later product phases, but the MVP direction is clear enough to guide planning.

## ADR-001: MVP Product Scope And Audience

Summary: MVP is a single-user, coding-first desktop workspace for technical users working in local repositories.

Status: Finalized for MVP.

Rationale: The original general-purpose workspace claim was too broad. MVP scope narrows validation to whether a desktop control center improves trust, resume, inspection, and control for agent-assisted local coding.

Affected documents:

 - `plans/adr/001-product-scope-and-mvp-audience.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-validation-plan.md`
 - `plans/reviews/adr-architecture-review-report.md`

Next action: Keep MVP messaging coding-first and avoid general-purpose claims until non-code workflows are validated.

## ADR-007: Local-First MVP Storage

Summary: MVP uses local SQLite metadata, local artifact files, and OS keychain or encrypted local secret storage.

Status: Finalized for MVP.

Rationale: SQLite and local artifact storage are sufficient for a single-user, one-active-session MVP. Team sync, compliance-grade audit, and cross-device portability are not MVP goals.

Affected documents:

 - `plans/adr/007-local-first-storage-model.md`
 - `plans/specs/data-model-specification.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-008-storage-audit-portability.md`

Next action: Validate SQLite persistence and crash recovery in the storage spike before detailed implementation planning.

## ADR-008: Basic Tool Ledger

Summary: MVP includes a basic tool ledger for user inspection and debugging, not compliance-grade audit.

Status: Finalized for MVP.

Rationale: The MVP needs visibility into tool activity, approvals, outputs, and affected files. Tamper-evident audit and data-flow-aware policy are deferred.

Affected documents:

 - `plans/adr/008-unified-tool-invocation-and-ledger.md`
 - `plans/specs/architecture-specification.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-success-metrics.md`

Next action: Define the minimal tool record schema during runtime and policy validation.

## ADR-009: MVP Approval Scope

Summary: MVP supports one-time allow, session allow, and deny for file, shell, and Git actions.

Status: Finalized for MVP.

Rationale: Narrow approval scopes validate user trust without introducing persistent risky grants. Always-allow and project-wide approvals are excluded from MVP.

Affected documents:

 - `plans/adr/009-permission-and-approval-model.md`
 - `plans/specs/security-specification.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-user-journeys.md`

Next action: Research default-deny and destructive-command classification in the local execution spike.

## ADR-011: MCP Excluded From MVP

Summary: MCP is not included in MVP.

Status: Finalized for MVP.

Rationale: MCP is strategically important, but it expands the execution and data-exposure surface before runtime policy authority is proven.

Affected documents:

 - `plans/adr/011-mcp-integration-strategy.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-004-mcp-scope-and-threat-model.md`

Next action: Validate local-only MCP scope for V1 after MVP runtime and policy gates are settled.

## ADR-013: Root AGENTS.md Display Only

Summary: MVP discovers and displays only root `AGENTS.md`; nested resolution and instruction conflict handling are deferred.

Status: Finalized for MVP.

Rationale: Root `AGENTS.md` validates the project-instruction convention without taking on nested precedence, conflict diagnostics, or policy interactions.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-user-journeys.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`

Next action: Include `AGENTS.md` conformance levels in the standards matrix.

## ADR-015: Marketplace Excluded From MVP

Summary: Marketplace support is excluded from MVP.

Status: Finalized for MVP.

Rationale: Marketplace support implies trust, curation, source integrity, update verification, and support obligations that are unnecessary for validating the core desktop workspace thesis.

Affected documents:

 - `plans/adr/015-marketplace-model.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-015-marketplace-operational-trust.md`

Next action: Revisit marketplace after plugin trust and source integrity are defined.

## ADR-016: Plugin Execution Excluded From MVP

Summary: Plugin scripts, hooks, and plugin execution are excluded from MVP.

Status: Finalized for MVP.

Rationale: Plugin execution increases supply-chain and endpoint risk. It does not directly validate the MVP thesis.

Affected documents:

 - `plans/adr/016-plugin-trust-and-execution.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-005-plugin-format-and-trust.md`

Next action: Research plugin format and trust tiers for a later phase.

## ADR-018: Browser Automation Excluded From MVP

Summary: Browser automation, DOM extraction, screenshots, and automatic browser-content ingestion are excluded from MVP.

Status: Finalized for MVP.

Rationale: Browser content is a prompt-injection and sandboxing risk. MVP can validate core coding workflow without browser automation.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-013-artifact-browser-preview-security.md`

Next action: Validate artifact/browser security separately before V1/V2 browser features.

## ADR-019: OpenRouter-Only MVP Provider Path

Summary: MVP uses OpenRouter as the only provider path.

Status: Finalized for MVP.

Rationale: OpenRouter reduces provider-integration scope and lets the MVP test whether users accept the preferred BYOK/provider direction. Direct providers and local models are deferred.

Affected documents:

 - `plans/adr/019-model-provider-strategy.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-validation-plan.md`
 - `plans/spikes/SPIKE-009-provider-privacy-cost.md`

Next action: Validate provider privacy, routing, and cost disclosure requirements before implementation planning.

## ADR-020: No Long-Term Memory In MVP

Summary: MVP persists sessions and tool records only; long-term memory is excluded.

Status: Finalized for MVP.

Rationale: Long-term memory increases privacy and correctness risk. MVP only needs session continuity.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-014-memory-session-retention.md`

Next action: Define transcript, tool-record, and artifact retention behavior in the memory/session spike.

## ADR-022: MVP Roadmap Starts With Validation Gates

Summary: MVP planning is gated by runtime, policy, local execution, storage, provider, and UX validation.

Status: Finalized for MVP.

Rationale: Reviews identified that implementation should not begin before make-or-break assumptions are validated.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/roadmap/implementation-roadmap.md`
 - `plans/mvp/mvp-validation-plan.md`
 - `plans/spikes/`

Next action: Execute research spikes before implementation plans.

