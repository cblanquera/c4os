# ADR-005: Standards-First Interoperability

Status: Provisional.

## Context

The research recommends composing existing standards instead of inventing proprietary formats. The supported standards include AGENTS.md, Agent Skills, MCP, Codex-compatible plugin and marketplace conventions, OpenCode-compatible configuration, and OpenRouter model/provider conventions.

The review warns that "support" is ambiguous. It can mean display-only, parse-only, partial execution, full execution, round-trip compatibility, import, or export.

## Decision

Use standards-first interoperability as a product principle, but define conformance levels per standard before committing data model or UX promises.

## Alternatives Considered

 - Standards-first formats.
 - Proprietary app-native formats.
 - Import/export compatibility only.

## Alternatives That Should Be Considered

 - App-owned canonical data model with adapters for standards.
 - Tiered conformance: display, parse, execute, round-trip, export.
 - Explicit "best effort compatibility" labels for unstable external conventions.

## Tradeoffs

Standards-first design improves portability, ecosystem leverage, and user trust.

Proprietary formats provide stronger control and clearer invariants but create lock-in.

Canonical app-owned models with adapters reduce future migration risk but require more upfront mapping work.

## Consequences

 - Compatibility claims must be precise.
 - App-specific extensions should be namespaced.
 - Round-trip support should not be implied unless tested.
 - Unstable external conventions should not become unversioned internal contracts.

## Follow-Up Questions

 - What conformance level is required for AGENTS.md?
 - What conformance level is required for SKILL.md?
 - What MCP spec version is the baseline?
 - Which Codex plugin fields are stable enough to rely on?
 - Is OpenCode config imported, mirrored, or owned by the app?

## ADR Recommendation

Keep this ADR high priority and use it to produce a standards compatibility matrix.

