# ADR-009: Permission And Approval Model

Status: Finalized for MVP approval scopes. Broader post-MVP policy remains unresolved.

## Context

The research recommends layered permissions: workspace trust, project policy, agent policy, plugin/MCP policy, per-tool risk classification, and per-call approval. Earlier specs considered approval scopes including once, session, project, always allow, deny once, and always deny.

The review warns that broad approval scopes and per-tool-only checks may be unsafe, especially for team-managed environments and multi-tool exfiltration chains.

## Decision

For MVP, use explicit approval prompts with only these scopes:

 - One-time allow.
 - Session allow for matching non-destructive shell commands inside the selected project root or approved project subpath.
 - Deny.

Session allow never covers destructive shell commands, outside-root paths, secret-deny files, or Git state-changing actions. Always-allow, project-wide approvals, enterprise policy scopes, and plugin/MCP permissions are post-MVP.

Denied or blocked MVP actions return a structured denial result to the runtime. The app records the denial in the approval/tool ledger and sends a user-safe denial category and message back to the model/runtime. Denied actions must not be represented as success, and they must not fail silently.

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

 - Approval UX must show what will run, what it reads, what it changes, approval source, and approval scope.
 - Session allow must be explained as a narrow shell-only convenience, not a broad local automation grant.
 - The runtime receives enough denial context to choose another path without exposing raw policy internals.
 - Team or enterprise modes may require different approval scopes than personal local use.

## Follow-Up Questions

 - What exact denial payload shape does the OpenCode adapter require?

## ADR Recommendation

Finalize for MVP. Revisit for post-MVP policy engine, MCP, plugins, child sessions, and enterprise controls.
