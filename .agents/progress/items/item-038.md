# item-038: TASK-037 Build Local Stdio MCP Explicit Approval Surface

## Status

verified

## Objective

Implement the accepted `local_stdio_explicit_approval_only` MCP setup surface
for explicit local stdio server registration and approval-gated launch
proposal.

## Dependencies

- item-037

## Scope

- Local stdio MCP only; remote MCP remains unavailable.
- Register local stdio MCP server definitions only through explicit user
  action.
- Do not auto-import, auto-discover, auto-launch, or connect MCP servers from
  project files.
- Launch requests must produce an approval-gated local command proposal; they
  must not start a process directly.
- Use selected-project root scope and disclose filtered environment behavior.
- MCP roots are limited to the selected project root.
- MCP resources and prompts are not automatically added to app-owned model
  context.
- MCP sampling and elicitation do not trigger automatic model calls or
  user-data disclosure.
- Remote transports, authorization flows, unapproved tool invocation,
  marketplace workflows, export, import, and round-trip compatibility remain
  unsupported.

## Inputs

- `.agents/archive/planning-corpus/acceptance/mcp-integration.md`
- `.agents/archive/planning-corpus/adr/011-mcp-integration-strategy.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-004-mcp-scope-and-threat-model.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/progress/items/item-037.md`

## Deliverables

- App status fields for accepted and unavailable MCP behavior.
- Backend explicit local stdio MCP server registration.
- Backend launch proposal for approval-gated local stdio MCP command review.
- Root scope projection limited to selected project root.
- Tests for registration, launch proposal, project-root scoping, remote
  rejection, no auto-import, no automatic context ingestion, and no automatic
  sampling or elicitation.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml mcp`.
- `npm test -- tests/backend-command-boundary.test.mjs tests/ui-state.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

## Handoff

Implemented the accepted `local_stdio_explicit_approval_only` tier. The next
roadmap candidate should be scoped separately before implementation.

## Result

- Added app status projection for accepted local stdio MCP support and excluded
  remote, automatic, and full-compatibility behavior.
- Added explicit local stdio MCP server registration under the selected project
  root.
- Added launch proposal generation that requires approval and does not start a
  process.
- Kept project MCP files passive; no auto-import, auto-discovery, or auto-start
  path was added.
- Kept resources, prompts, sampling, and elicitation out of automatic model
  context and automatic model-call paths.
