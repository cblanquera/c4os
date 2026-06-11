# Compatibility Claims Acceptance Criteria

## Current Decision

Status: accepted on 2026-06-12.

Accepted tier: `no_broader_compatibility_claims_v1`.

V1 may claim only the narrow compatibility already supported by implemented
surfaces: OpenRouter-backed model access, local Git project support, root and
nested `AGENTS.md` display guidance, explicit project-local Agent Skills
discovery/invocation, explicit approval-gated local stdio MCP registration,
app-owned text/raster artifact records, project JSON export, archived-session
delete, and no durable memory.

V1 must not claim full AGENTS.md compatibility, full Agent Skills
compatibility, full MCP compatibility, Codex plugin compatibility, OpenCode
config compatibility, import compatibility, export/import round-trip
compatibility, broader artifact compatibility, browser compatibility, or
durable-memory compatibility.

## Acceptance Criteria

- User explicitly accepts, revises, or rejects
  `no_broader_compatibility_claims_v1`.
- Product-facing status and documentation continue to use narrow, feature-level
  compatibility claims rather than broad standards-compatible language.
- Existing accepted features keep their own explicit support tiers.
- Full compatibility claims remain deferred until standards behavior,
  conformance tests, import/export round trips, runtime config behavior, and
  trust boundaries are specified and verified.

## Out Of Scope

- New compatibility modes.
- Import or round-trip support.
- Codex plugin compatibility.
- Full OpenCode config ownership.
- Full AGENTS.md, Agent Skills, or MCP compatibility.
- Browser or rich document compatibility claims.
