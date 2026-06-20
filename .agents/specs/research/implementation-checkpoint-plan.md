# Implementation Checkpoint Plan

Status: active
Updated: 2026-06-21
Source:
- `.agents/specs/research/mvp-freeze.md`
- `.agents/specs/research/checkpoints.md`
- `.agents/references/research/grill-session.md`

## Rules

- Checkpoints are progress-review gates, not MVP scope boundaries.
- The MVP remains the full documented/r04 feature set.
- A checkpoint may use partial implementation, but it must not be described as
  product-complete unless the full freeze scope is complete.

## Checkpoint 1: Shell And Session Proof

Purpose: prove the first progress slice is viable enough to continue.

Must show:

- trusted folder selection
- provider setup
- OpenCode session creation through the C4OS adapter boundary
- visible agent status/thinking and final response
- persisted session record that can be resumed

Must avoid:

- calling this checkpoint the MVP
- treating mocked UI as product-complete
- moving r04 surfaces out of MVP

## Checkpoint 2: Project Work Loop

Purpose: prove the agent can perform useful project work.

Must show:

- project file read/write inside active project when request-implied
- agent-owned command terminal execution
- command allowlist behavior for read-only commands
- approval for non-allowlisted or outside-project commands
- agent file and terminal action records

## Checkpoint 3: Browser And Preview Loop

Purpose: prove the Browser can support project inspection and user-owned
browsing.

Must show:

- project-scoped Browser profile
- local file or artifact opening
- public website browsing
- request-scoped agent browsing
- logged-in Browser use when clearly requested
- agent Browser action records
- Browser downloads excluded

## Checkpoint 4: Extension Enablement Loop

Purpose: prove extension install/connect can safely affect agent behavior after
explicit enablement.

Must show:

- plugin install/connect record
- skill install/connect record
- MCP server install/connect record
- provenance, scopes, shared data, runtime/tool access, enabled state, and
  audit records
- `$skill`, `@plugin`, and `^mcp` prompt tags
- explicit MCP invocation

## Checkpoint 5: Integrated MVP

Purpose: prove the full documented/r04 MVP works as one product.

Must show:

- all freeze acceptance points
- concurrent sessions/runs across trusted project folders
- one main active run per chat session
- resumability after app restart
- traceable audit records for agent file, Browser, terminal, and extension
  activity
