# ADR-011: MCP Integration Strategy

Status: Provisional, remote MCP unresolved.

## Context

The research recommends MCP as the default protocol for external tools, local services, and organization connectors. Functional requirements include configuring stdio and remote MCP servers, displaying server health, exposed tools, resources, prompts, and credentials, and requiring approvals for tool calls and sampling.

The review asks whether remote MCP should be disabled in MVP because of data exfiltration, authorization, and prompt-injection risk.

## Decision

Support MCP as the primary integration protocol, with final MVP scope for remote MCP unresolved.

## Alternatives Considered

 - MCP default integration protocol.
 - Custom plugin API only.
 - Direct SDK integrations only.

## Alternatives That Should Be Considered

 - Local stdio MCP only for MVP.
 - Remote MCP disabled by default.
 - MCP proxy/gateway controlled by the app policy engine.
 - Direct native connectors for a small number of critical integrations.

## Tradeoffs

MCP maximizes interoperability and reduces custom connector work.

Remote MCP introduces stronger security and privacy risks than local stdio servers.

Direct SDK integrations can provide better UX and permission clarity for high-value systems but scale poorly.

## Consequences

 - MCP tools, resources, prompts, roots, and sampling must be mapped to app policy.
 - Tool annotations and descriptions must be treated as untrusted unless the server is trusted.
 - Remote MCP auth and network boundaries must be explicit before broad use.

## Follow-Up Questions

 - Which MCP spec version is the baseline?
 - Are remote MCP servers allowed in MVP?
 - Are MCP resources automatically eligible for model context?
 - How are sampling requests approved?
 - How are roots scoped and displayed?

## ADR Recommendation

Keep this ADR high priority and resolve before plugin marketplace work.

