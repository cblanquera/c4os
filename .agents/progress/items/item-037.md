# item-037: TASK-036 Resolve V1 Local Stdio MCP Scope

## Status

verified

## Objective

Resolve the V1 local stdio MCP conformance, transport, root, approval, and
trust model before exposing any MCP server configuration or launch behavior.

## Dependencies

- item-036

## Scope Decision

Current V1 supports `local_stdio_explicit_approval_only`, accepted by the user
on 2026-06-12.

- Local stdio MCP only; remote MCP remains unavailable.
- MCP baseline is official specification version `2025-11-25`, verified on
  2026-06-12 from `https://modelcontextprotocol.io/specification/2025-11-25`.
- MCP servers are never discovered, configured, launched, or connected
  automatically from project files.
- Local stdio MCP servers may be added only through explicit user action.
- Starting a local stdio MCP server is approval-gated local shell/process
  execution through the backend Approval Gateway.
- MCP roots are limited to the selected project root until a later accepted
  scope defines narrower roots.
- MCP tools must route through the app-owned tool ledger and approval policy.
- MCP resources and prompts are not automatically added to app-owned model
  context.
- MCP sampling and elicitation are denied or held for explicit approval.
- Remote transports, authorization flows, automatic project-file startup,
  automatic resource context ingestion, automatic prompt use, unapproved tool
  invocation, unapproved sampling, marketplace workflows, export, import, and
  round-trip compatibility remain unsupported.

## Evidence

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md` lists
  local stdio MCP server support after explicit skill discovery and invocation.
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md` keeps MCP
  deferred until V1 scope is decided.
- `.agents/archive/planning-corpus/adr/011-mcp-integration-strategy.md` keeps
  remote MCP unresolved and names roots, resources, prompts, tools, sampling,
  and policy routing as required decisions.
- Official MCP specification version `2025-11-25` identifies tools, resources,
  prompts, sampling, roots, and elicitation as protocol features and emphasizes
  explicit consent for data access, operations, tools, and sampling.
- `.agents/archive/planning-corpus/acceptance/mcp-integration.md` excludes MCP
  from MVP and now records proposed V1 acceptance criteria.

## Deliverables

- Product decision for the V1 local stdio MCP conformance tier.
- Acceptance criteria for local server registration, launch, roots, tools,
  resources, prompts, sampling, elicitation, approval routing, failure states,
  and out-of-scope behavior.
- Explicit deferral of remote MCP and automatic project-file MCP startup.
- Handoff for the implementation item that follows acceptance.

## Verification

- Planning review against roadmap, deferred decisions, ADR-011, MCP threat
  model, official MCP spec, security specification, and MCP acceptance
  criteria.
- User accepted the proposed scope on 2026-06-12.

## Decision

The proposed scope is accepted.

## Handoff

Continue with item-038 / TASK-037 to implement the explicit approval-gated
local stdio MCP surface.
