# Research Freeze

Status: frozen-for-mvp-specification
Updated: 2026-06-21
Source:
- `.agents/references/research/grill-session.md`
- `.agents/specs/research/requirements.md`
- `.agents/specs/research/acceptance.md`
- `wireframes/ui-handoff-spec.md`

## Purpose

This file closes the research round. It captures accepted research findings,
candidate MVP scope, accepted POC learning, and viability gaps that must feed a
separate MVP specification.

This file is not an implementation contract. It does not authorize active
progress items by itself.

## Candidate MVP Scope

The research-supported candidate MVP is the full documented/r04 product slice
unless explicitly moved out by a later accepted MVP-spec decision.

Candidate scope includes:

- folder-first trusted workspace entry
- provider setup with OpenAI-compatible provider profiles and secure key
  storage
- OpenCode first behind a thin C4OS-owned runtime adapter; Pi remains a later
  adapter target
- persistent sessions with visible agent status, response history, runtime
  state, artifacts, approvals, and resumability after app restart
- concurrent sessions/runs across trusted project folders, with one main run
  per chat session
- agent authority scoped by active project and user request
- Browser as a user-owned desktop surface with project-scoped profile, local
  file browsing, public web browsing, request-scoped agent browsing, logged-in
  use when clearly requested, and recorded agent Browser actions
- Browser downloads excluded from MVP
- Files explorer/editor for trusted project file inspection and editing
- separate user terminal and agent-owned command terminal, both backend-owned
- agent-owned terminal read-only allowlist using exact command patterns from
  app-bundled defaults, overrideable by workspace config
- extension install/connect for plugins, skills, and MCP servers, with
  explicit per-extension enablement before runtime impact
- chat prompt extension tags: `$skill`, `@plugin`, and `^mcp`; MCP invocation
  explicit only
- r04 shell behavior: App Start, New Session, Chat Session, provider/model
  selectors, Browser, Files, Terminal, and Settings surfaces

## Research Acceptance Bar

The MVP specification must turn the candidate scope into a customer-usable
workflow and objective acceptance criteria. At minimum, the MVP spec must cover:

- trusted folder opened
- provider configured
- OpenCode session created through the C4OS adapter boundary
- agent status/thinking visible
- agent response persisted
- trusted-root file read/write
- backend-owned, policy-gated agent command terminal
- allowlisted read-only commands
- approval for sensitive or outside-project commands
- Browser preview with project-scoped profile, local artifact/file opening,
  public browsing, request-scoped agent browsing, and recorded Browser actions
- Browser download exclusion
- plugin, skill, and MCP install/connect records with provenance, scopes,
  shared data, runtime/tool access, enabled state, and audit records
- explicit per-extension enablement before runtime impact
- `$skill`, `@plugin`, and `^mcp` prompt tags, with MCP explicit only
- concurrent sessions/runs across trusted project folders
- one main active run per chat session
- session resume after app restart with transcript, runtime references,
  approvals, artifacts, and file/terminal/Browser/extension action records

## Required Next Step

Create or repair `.agents/specs/mvp/` before implementation.

The MVP spec must define the actual distributable desktop MVP, including
requirements, acceptance, risks, decisions, evidence, viability gaps,
traceability, and proposed tasks. Only a frozen MVP spec can be converted into
progress items.

For this repository, the MVP spec must state that production implementation
belongs in:

- `backend/` for Rust/Tauri backend authority, native commands, state,
  security, runtime, filesystem, terminal, Browser, extension, approval, and
  persistence behavior
- `frontend/` for the desktop app UI
- `tests/server/` for backend, integration, and MVP acceptance tests

Do not create or use `src-tauri/`.
