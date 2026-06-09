# ADR-009: Permission And Approval Model

Status: Provisional with unresolved enforcement details.

## Context

The research recommends layered permissions: workspace trust, project policy, agent policy, plugin/MCP policy, per-tool risk classification, and per-call approval. The security spec defines approval scopes including once, session, project, always allow, deny once, and always deny.

The review warns that broad approval scopes and per-tool-only checks may be unsafe, especially for team-managed environments and multi-tool exfiltration chains.

## Decision

Adopt a layered permission model with explicit approval prompts for risky actions.

This decision is provisional until default-deny rules, approval scopes, and the authoritative enforcement layer are decided.

## Alternatives Considered

 - Runtime-only prompts.
 - Static config only.
 - Layered policy plus per-call approval.

## Alternatives That Should Be Considered

 - Default-deny for high-risk operations.
 - Separate policy dimensions for read, write, execute, network, credentials, publish, and delete.
 - Narrow MVP approval scopes only: once and session.
 - Information-flow policy for sequences of tool calls.

## Tradeoffs

Layered policy supports safe defaults, admin control, and productivity.

Too many layers can confuse users and make policy debugging difficult.

Broad "always allow" decisions can create persistent risk.

## Consequences

 - Approval UX must show what will run, what it reads, what it changes, policy source, and approval scope.
 - Policies must be explainable and debuggable.
 - Team or enterprise modes may require different approval scopes than personal local use.

## Follow-Up Questions

 - What are the default-deny rules?
 - Which approval scopes are allowed in MVP?
 - How are destructive commands classified?
 - Are approvals inherited by child agents?
 - How are denied actions represented to the model?

## ADR Recommendation

Keep this ADR high priority and resolve alongside ADR-004 and ADR-010.

