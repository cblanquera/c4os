# Deferred Decisions

This document summarizes ADR decisions that are intentionally deferred beyond MVP or moved to later validation.

## ADR-006: Worktrees And Multi-Session Workspace Model

Summary: Worktrees, multiple sessions per project, and non-Git isolation are deferred.

Status: Deferred to V1 or later.

Rationale: MVP validates one active session in one local Git project. Worktrees and multi-session behavior add lifecycle complexity without being required for initial validation.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-011-large-workspace-and-concurrency.md`

Next action: Revisit after the MVP proves users value the single-session workspace.

## ADR-011: MCP Integration

Summary: MCP is deferred, with local stdio MCP as a likely first reconsideration point.

Status: Deferred to V1 or later.

Rationale: MCP is important for extensibility but not necessary for MVP validation and creates security and policy complexity.

Affected documents:

 - `plans/adr/011-mcp-integration-strategy.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-004-mcp-scope-and-threat-model.md`

Next action: Decide V1 MCP scope after policy authority is resolved.

## ADR-012: Agent Skills Support

Summary: Skill discovery and explicit skill invocation are deferred.

Status: Deferred to V1.

Rationale: Skills are useful, but MVP can validate the core workspace without skill loading, script execution, or version conflict handling.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`

Next action: Define skill conformance and trust model before V1.

## ADR-013: Nested AGENTS.md Resolution

Summary: Nested AGENTS.md precedence and conflict diagnostics are deferred.

Status: Deferred to V1.

Rationale: Root AGENTS.md display is enough for MVP. Nested resolution increases complexity and can confuse users without clear effective-instruction UI.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`

Next action: Revisit with standards conformance and instruction-stack UI design.

## ADR-015: Marketplace Model

Summary: Marketplace distribution is deferred.

Status: Deferred to Future.

Rationale: Marketplace requires trust, review, source integrity, update verification, and support operations. MVP excludes plugins and therefore does not need marketplace.

Affected documents:

 - `plans/adr/015-marketplace-model.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-015-marketplace-operational-trust.md`

Next action: Revisit only after plugin execution and plugin trust are solved.

## ADR-016: Plugin Scripts And Hooks

Summary: Plugin scripts and hooks are deferred.

Status: Deferred to V2 or Future.

Rationale: Executable plugin content is high-risk and not needed for MVP validation.

Affected documents:

 - `plans/adr/016-plugin-trust-and-execution.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-005-plugin-format-and-trust.md`

Next action: Define plugin signing, sandboxing, and permission review before reconsidering.

## ADR-017: Rich Artifact Preview Model

Summary: Rich artifact previews are deferred; MVP keeps plain text, Markdown, logs, diffs, and generated source or config files only.

Status: Deferred to V1/V2.

Rationale: PDFs, active HTML, images, spreadsheets, documents, Chromium-backed previews, artifact execution, and export/duplicate workflows introduce renderer and security risks. They are not needed for the core coding-first validation.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-013-artifact-browser-preview-security.md`

Next action: Validate artifact preview security before adding active or complex renderers.

## ADR-018: Browser/Web Viewing

Summary: Browser panel, screenshots, DOM extraction, and automatic browser-content ingestion are deferred.

Status: Deferred to V2.

Rationale: Browser integration is a prompt-injection and sandboxing risk and is not required for validating local coding project workflows.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-013-artifact-browser-preview-security.md`

Next action: Revisit after artifact/browser security research.

## ADR-019: Direct Provider Fallback

Summary: Direct provider fallback and local model providers are deferred.

Status: Deferred to V2.

Rationale: MVP uses OpenRouter only to reduce integration scope. Direct providers are valuable for lock-in reduction but not necessary for first validation.

Affected documents:

 - `plans/adr/019-model-provider-strategy.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-009-provider-privacy-cost.md`

Next action: Revisit if validation users reject OpenRouter-only setup.

## ADR-020: Long-Term Memory

Summary: Long-term memory and cross-session memory are deferred.

Status: Deferred to V2.

Rationale: Memory increases privacy and correctness risk. MVP only needs session persistence.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-014-memory-session-retention.md`

Next action: Define inspect/delete/review controls before adding durable memory.

## ADR-021: Broader UX Modes

Summary: Non-code workspace modes and broad concept education are deferred.

Status: Deferred to V1 or later.

Rationale: MVP targets technical users and should hide non-MVP concepts such as plugins, MCP, skills, worktrees, and marketplace.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-user-journeys.md`
 - `plans/spikes/SPIKE-012-ux-onboarding-concept-load.md`

Next action: Validate MVP onboarding comprehension before expanding UI concepts.
