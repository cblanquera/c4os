# ADR-004: Policy Enforcement Authority

Status: Unresolved.

## Context

The architecture states that the Tauri/Rust backend owns filesystem access, process management, secret storage, plugin validation, and policy enforcement. It also states that OpenCode Runtime should perform permission-aware tool use where available.

The review identifies this as a split-brain risk. If both layers can approve, deny, transform, or execute tools, auditability and safety depend on perfect synchronization.

## Decision

No final decision has been made.

The current documents imply layered enforcement, but they do not identify the authoritative enforcement point.

## Alternatives Considered

 - Backend-owned policy enforcement.
 - Runtime-owned policy enforcement.
 - Layered backend and runtime enforcement.

## Alternatives That Should Be Considered

 - Backend as mandatory policy gateway for all tool families.
 - Runtime-native policy with backend limited to UX and audit.
 - A tool proxy that all local, MCP, browser, plugin, shell, Git, and artifact tools must traverse.
 - Deny any runtime mode that can execute outside the app policy envelope.

## Tradeoffs

Backend authority provides consistent product-level security but may require deep runtime integration.

Runtime authority reduces integration complexity but weakens app-owned guarantees and may not cover browser/plugin/artifact surfaces.

Layered policy can provide defense in depth, but only if one layer is authoritative and conflicts are deterministic.

## Consequences

 - This decision blocks security architecture.
 - It determines whether OpenCode is an adapter behind app policy or a co-equal policy engine.
 - It affects audit completeness, approval UX, plugin execution, MCP tools, shell commands, and file access.

## Follow-Up Questions

 - Which layer is authoritative on conflict?
 - Can OpenCode execute any tool before backend approval?
 - Can MCP calls bypass the backend ledger?
 - Are approval decisions stored by the app, runtime, or both?
 - Can a plugin script execute through shell permissions instead of plugin permissions?

## ADR Recommendation

Create and resolve this ADR before architecture freeze.

