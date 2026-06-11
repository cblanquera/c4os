# ADR-004: Policy Enforcement Authority

Status: Finalized for MVP. Broader post-MVP policy remains unresolved.

## Context

The architecture states that the Tauri/Rust backend owns filesystem access, process management, secret storage, and the MVP Approval Gateway. It also states that OpenCode Runtime should perform permission-aware tool use where available.

The review identifies this as a split-brain risk. If both layers can approve, deny, transform, or execute tools, auditability and safety depend on perfect synchronization.

MVP scope is limited to file writes, shell commands, and runtime-proposed Git state changes. Plugin execution, MCP, browser automation, multi-agent execution, and data-flow-aware policy are excluded from MVP.

## Decision

For MVP, the Tauri/Rust backend is the authoritative Approval Gateway for file writes, shell commands, and runtime-proposed Git state changes.

OpenCode may provide runtime-native permission features as defense in depth, but it must not execute MVP-controlled actions before the backend Approval Gateway approves them. If OpenCode cannot expose pre-execution control for those action classes, the OpenCode integration is not viable for the accepted MVP without an adapter, proxy, fork, or scope reduction.

When the backend denies or blocks an MVP-controlled action, the Runtime Adapter returns a structured denial result to OpenCode. The denial is recorded in the app-owned ledger and represented to the runtime as a blocked tool result, not as success and not as silent failure.

## Alternatives Considered

 - Backend-owned approval enforcement.
 - Runtime-owned policy enforcement.
 - Layered backend and runtime enforcement.

## Alternatives That Should Be Considered

 - Backend as mandatory approval gateway for MVP file writes, shell commands, and runtime-proposed Git state changes.
 - Runtime-native policy with backend limited to UX and audit.
 - A tool proxy that all local, MCP, browser, plugin, shell, Git, and artifact tools must traverse.
 - Deny any runtime mode that can execute outside the app policy envelope.

## Tradeoffs

Backend authority provides consistent product-level security but may require deep runtime integration.

Runtime authority reduces integration complexity but weakens app-owned guarantees and may not cover browser/plugin/artifact surfaces.

Layered policy can provide defense in depth, but only if one layer is authoritative and conflicts are deterministic.

## Consequences

 - This decision unblocks MVP security architecture.
 - It makes OpenCode an execution adapter behind the app-owned Approval Gateway for MVP-controlled actions.
 - It affects audit completeness, approval UX, shell commands, runtime-proposed Git state changes, and file writes.
 - Runtime adapter work must define the exact structured denial payload expected by OpenCode.
 - Plugin execution, MCP tools, browser tools, and data-flow policy still need post-MVP policy decisions.

## Follow-Up Questions

 - Can OpenCode execute any tool before backend approval?
 - Can OpenCode expose file write, shell, and Git proposals before execution?
 - Are approval decisions stored by the app, runtime, or both?
 - What adapter, proxy, fork, or scope change is required if OpenCode cannot provide pre-execution control?
 - For post-MVP: can MCP calls, plugin scripts, browser tools, and artifact renderers bypass the backend ledger?

## ADR Recommendation

Validate this decision in the OpenCode runtime control and policy enforcement spikes before architecture freeze.
