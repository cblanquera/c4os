# ADR-014: Plugin System Compatibility

Status: Provisional.

## Context

The plugin specification recommends Codex-compatible plugin manifests and layout: `.codex-plugin/plugin.json`, optional `skills/`, `hooks/`, `scripts/`, `assets/`, `.mcp.json`, and `.app.json`. It recommends namespacing non-Codex fields under `x-c4os`.

The review warns that Codex plugin fields may not be a stable public standard and that plugin hooks may not map cleanly to OpenCode or app runtime semantics.

## Decision

Use Codex-compatible plugin layout as the first plugin compatibility target, with app-specific extensions namespaced.

This decision is provisional until conformance and stability are defined.

## Alternatives Considered

 - Codex-compatible plugin shape.
 - MCP manifests only.
 - Custom plugin manifest.

## Alternatives That Should Be Considered

 - Minimal plugin MVP supporting only skills and MCP.
 - App-native manifest with import/export to Codex-style plugins.
 - Disable or ignore hooks until runtime semantics are proven.
 - Treat Codex compatibility as best-effort rather than full compatibility.

## Tradeoffs

Codex compatibility improves ecosystem leverage and aligns with existing skills/plugin conventions.

Custom manifests provide clearer product control but increase lock-in.

Supporting the full Codex-like surface, especially hooks and scripts, increases security and compatibility risk.

## Consequences

 - Plugin validation must define which fields are accepted, ignored, rejected, or namespaced.
 - Marketplace compatibility depends on this decision.
 - Security posture depends on whether scripts and hooks are allowed.

## Follow-Up Questions

 - Which Codex plugin fields are stable enough to depend on?
 - Which fields are display-only versus executable?
 - Should hooks be disabled in MVP?
 - Should scripts be disabled in MVP?

## ADR Recommendation

Keep this ADR high priority and resolve with ADR-016.

