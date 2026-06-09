# ADR-003: Agent Runtime Strategy

Status: Provisional.

## Context

The preferred architecture is `AI GUI -> OpenCode Runtime -> OpenRouter (BYOK)`. The requirements explicitly say the MVP should avoid building a custom runtime if OpenCode Runtime can satisfy the core requirements. The architecture recommends using OpenCode through a narrow adapter.

The review identifies this as a make-or-break assumption. A CLI/TUI-oriented runtime may not expose stable interfaces for event streaming, pre-tool approval interception, session resume, child sessions, worktree mapping, custom policy engines, plugin loading, artifact capture, or provider routing visibility.

## Decision

Use OpenCode Runtime as the first runtime target through a narrow adapter, but do not treat the decision as final until headless control and policy hooks are proven.

## Alternatives Considered

 - OpenCode Runtime.
 - Custom runtime.
 - Direct OpenRouter calls.
 - Codex CLI/runtime dependency.
 - Multiple runtimes immediately.

## Alternatives That Should Be Considered

 - Runtime abstraction layer with OpenCode as the first adapter.
 - OpenCode fork if required hooks are unavailable.
 - App-owned local daemon API that can wrap OpenCode or another runtime later.
 - Direct provider runtime only for a smaller non-agent MVP.

## Tradeoffs

OpenCode accelerates MVP and aligns with agents, tools, permissions, MCP, providers, and sessions. It also creates dependency risk if the runtime is not designed for GUI embedding.

A custom runtime offers full control over policy, persistence, tools, and UX, but has the highest cost and correctness risk.

A runtime abstraction layer reduces lock-in but adds upfront architecture and can collapse into lowest-common-denominator design.

## Consequences

 - Runtime validation must precede UI buildout.
 - The app should own canonical records for sessions, tool calls, artifacts, policies, and plugins rather than treating OpenCode storage as the only source of truth.
 - If pre-execution tool interception is unavailable, the current security model must be revised.

## Follow-Up Questions

 - What exact OpenCode interface will be used: library API, JSON RPC server, CLI subprocess protocol, filesystem state, or fork?
 - Can every tool call be intercepted before execution?
 - Can OpenCode stream structured events without terminal scraping?
 - How does OpenCode persist sessions?
 - Can multiple runtime instances run safely in parallel?

## ADR Recommendation

Keep this ADR open until a runtime-control matrix proves or rejects OpenCode as a controllable headless runtime.

