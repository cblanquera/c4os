# Objective

Determine whether direct OpenCode integration can satisfy the MVP runtime-control gate without UI-only approvals, terminal scraping, post-execution audit, hidden model/provider overrides, or invisible instruction loading.

# Context

The MVP architecture depends on `AI GUI -> OpenCode Runtime -> OpenRouter`. The final readiness review marks this as the primary blocker because the app's trust model requires structured runtime events, pre-execution interception, reliable stop behavior, app-owned session mapping, provider/model control, and instruction-loading observability.

The Tauri/Rust backend is intended to own filesystem access, process supervision, secret storage, approvals, and persistence. OpenCode can remain the execution engine only if protected actions cannot bypass the backend Approval Gateway.

# Related Findings

 - FINDING-001 OpenCode Runtime Control Is An Unproven MVP Gate.
 - FINDING-002 Backend Approval Gateway Depends On Runtime Cooperation.
 - FINDING-004 OpenRouter-Only Model Routing Needs Runtime-Level Verification.
 - FINDING-005 Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope.
 - FINDING-014 What Is The Accepted Runtime Path If Direct OpenCode Fails?

# Questions To Answer

 - What supported OpenCode integration surface is available for MVP: library API, server protocol, CLI subprocess protocol, filesystem state, fork, or another path?
 - Can OpenCode emit structured events for sessions, assistant output, model calls, tool proposals, tool results, approvals, denials, errors, and stop/interruption state?
 - Can file writes, shell commands, and runtime-proposed Git state changes be intercepted before execution?
 - Can denied or blocked actions be returned to OpenCode as structured denial results without being reported as success or silent failure?
 - Can OpenCode be launched with app-owned session, project, provider, model, and credential-reference settings that override ambient user config?
 - Can the runtime effective model and provider path be observed at session start and during active runs?
 - Can existing OpenCode config override provider, model, tool, permission, instruction, or session behavior without detection?
 - Can OpenCode stop active model streams and terminate app-supervised child processes while preserving app-owned records and partial assistant output status?
 - How are OpenCode-native session IDs, logs, and persistence files mapped to app-owned canonical session records?
 - Does OpenCode automatically load root or nested `AGENTS.md` files or other instruction sources?
 - If runtime-native instruction loading exists, can it be disabled, observed, or clearly disclosed?

# Assumptions Being Validated

 - OpenCode can be treated as a subordinate runtime behind the app-owned Approval Gateway.
 - Direct OpenCode integration can provide structured events without fragile terminal scraping.
 - Protected local actions can be blocked until the backend approves them.
 - OpenCode-native config and instruction behavior can be constrained or surfaced enough for MVP honesty.
 - The app can own canonical session, message, tool, approval, artifact, provider, and model records while treating OpenCode state as adapter metadata.

# Investigation Plan

 - Review OpenCode documentation and source for stable headless, event, tool, permission, provider, config, stop, and session-control surfaces.
 - Review Odysseus as an external proof-of-concept reference for an OpenCode-backed self-hosted AI workspace, strictly to identify possible integration patterns and not as a product or architecture template.
 - Produce a runtime-control matrix covering each MVP-required event, action class, enforcement point, and stop/recovery behavior.
 - Run minimal throwaway probes only where documentation is insufficient to confirm event ordering, pre-execution interception, denial handling, stop semantics, effective model selection, and instruction-loading behavior.
 - Identify every path where OpenCode can execute file writes, shell commands, Git state changes, provider calls, or instruction loading outside app-owned control.
 - Record whether each control is supported, unsupported, unstable, undocumented, or requires a wrapper, proxy, fork, runtime replacement, or MVP scope reduction.
 - Document evidence with command transcripts, config snippets, observed event payloads, or source references sufficient for architecture review.

# External Proof-Of-Concept References

## Odysseus

Repository: https://github.com/pewdiepie-archdaemon/odysseus

Purpose in this spike: use Odysseus only as a research reference showing that another self-hosted AI workspace has attempted an OpenCode-backed agent integration with local tools.

What it may help validate:

 - Whether OpenCode has been embedded or wrapped by another web/workspace-style product.
 - What integration seams, tool dispatch patterns, or security boundaries that project used.
 - Which OpenCode runtime assumptions are worth inspecting first.
 - Which failure modes are likely when mixing OpenCode, shell, files, MCP, skills, memory, and a web UI.

What it does not prove:

 - It does not prove that OpenCode satisfies this product's pre-execution Approval Gateway requirement.
 - It does not prove that file writes, shell commands, or Git state changes can be intercepted before execution.
 - It does not prove that structured events are available without terminal scraping or post-execution inference.
 - It does not prove that OpenRouter-only routing, credential references, or instruction loading can be constrained for this MVP.
 - It does not define this product's architecture, user experience, permission model, deployment model, or security posture.

Reason for boundary:

Odysseus describes itself as a self-hosted AI workspace with an agent built on OpenCode, MCP, web, files, shell, skills, and memory. Its threat model frames privileged local access as intentional for trusted/admin users. This is materially different from this MVP, where the app-owned backend Approval Gateway must be authoritative before protected actions execute.

Research handling:

 - Treat Odysseus as an external PoC only.
 - Do not copy its architecture into this MVP.
 - Do not use its existence as evidence that FINDING-001 is resolved.
 - Use it to generate inspection questions and possible probe targets for the OpenCode runtime-control matrix.
 - If Odysseus contains a concrete pre-execution tool-control mechanism, record that mechanism as evidence to investigate directly against OpenCode, not as inherited proof.

# Success Criteria

 - The runtime-control matrix has explicit pass/fail evidence for all mandatory MVP gates.
 - File writes, shell commands, and runtime-proposed Git state changes are proven to wait for backend approval before execution, or direct OpenCode is rejected.
 - Structured runtime events are available without terminal scraping for user-visible activity and app-owned ledger records.
 - Stop behavior is proven to cancel active runtime/model work, terminate app-supervised child processes, and preserve app-owned records.
 - Existing OpenCode config cannot silently override app-owned provider/model/tool/instruction behavior, or the override path is detected and blocked.
 - Runtime-native instruction loading is disabled, observable, or disclosed in a way that does not contradict the MVP `AGENTS.md` display-only boundary.
 - The team has enough evidence to keep direct OpenCode, require an adapter/proxy/fork, replace the runtime, or reduce MVP scope.

# Deliverables

 - Runtime-control matrix with evidence links or excerpts.
 - OpenCode integration-surface recommendation.
 - Pre-execution interception and denial-behavior evidence.
 - Stop/recovery behavior evidence.
 - App-owned record mapping notes.
 - Config, provider/model, and instruction-loading behavior notes.
 - Go/no-go recommendation for direct OpenCode as the MVP runtime.

# ADRs Impacted

 - ADR-003 Agent Runtime Strategy.
 - ADR-004 Policy Enforcement Authority.
 - ADR-008 Unified Tool Invocation And Ledger.
 - ADR-009 Permission And Approval Model.
 - ADR-019 Model Provider Strategy.

# Decisions Unlocked

 - Whether direct OpenCode is viable for MVP.
 - Whether the backend Approval Gateway can remain authoritative with OpenCode.
 - Whether an adapter, proxy, fork, runtime replacement, or MVP scope reduction is required.
 - Whether Phase 1 implementation planning can begin.

# Failure Conditions

 - Any protected local action can execute before backend approval.
 - Required runtime activity is visible only through terminal scraping, log tailing, or post-execution inference.
 - Denied or blocked actions cannot be represented as structured runtime results.
 - Stop cannot reliably cancel active work or leaves app-supervised child processes running.
 - Existing OpenCode config can silently change provider, model, tool, permission, session, or instruction behavior.
 - Runtime-native instruction loading is invisible and cannot be disabled or disclosed.
 - Evidence is insufficient to distinguish supported integration from unsupported coupling.
