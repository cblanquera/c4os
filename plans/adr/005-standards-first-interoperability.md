# ADR-005: Standards-First Interoperability

Status: Finalized for MVP compatibility claims. Broader standards conformance remains unresolved.

## Context

The research recommends composing existing standards instead of inventing proprietary formats. Candidate standards and conventions include AGENTS.md, Agent Skills, MCP, Codex-compatible plugin and marketplace conventions, OpenCode-compatible configuration, and OpenRouter model/provider conventions.

The review warns that "support" is ambiguous. It can mean display-only, parse-only, partial execution, full execution, round-trip compatibility, import, or export.

## Decision

Use standards-first interoperability as a product principle, but keep MVP compatibility claims narrow and testable.

MVP may claim only:

 - OpenRouter-backed model access.
 - Local Git project support.
 - Root `AGENTS.md` display.
 - App-owned text-like artifact records.

MVP must not claim full AGENTS.md compatibility, Agent Skills compatibility, MCP compatibility, Codex plugin compatibility, OpenCode config compatibility, import/export compatibility, or round-trip compatibility until a standards conformance matrix defines display, parse, execute, import, export, and round-trip behavior.

For MVP, OpenCode runtime settings are app-owned adapter plumbing only. The app may store runtime references and launch settings required to invoke OpenCode, but it does not import, mirror, edit, export, or round-trip OpenCode config. Existing OpenCode config behavior that affects runtime execution must be discovered during Phase 0 and disclosed.

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
 - MVP marketing, docs, and UI must avoid generic "standards-compatible" claims.
 - App-specific extensions should be namespaced.
 - Round-trip support should not be implied unless tested.
 - Unstable external conventions should not become unversioned internal contracts.

## Follow-Up Questions

 - What conformance level is required for AGENTS.md?
 - What conformance level is required for SKILL.md?
 - What MCP spec version is the baseline?
 - Which Codex plugin fields are stable enough to rely on?
 - What OpenCode config behavior affects runtime execution and must be disclosed?
 - Which compatibility claims are allowed in V1 after the matrix is complete?

## ADR Recommendation

Finalize the narrow MVP claims. Use the standards conformance matrix to unlock broader compatibility claims after MVP.
