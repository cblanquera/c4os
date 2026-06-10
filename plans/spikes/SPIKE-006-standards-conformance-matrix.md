# Objective

Define exact conformance levels for each external standard or convention the product claims to support.

# Context

The plans prefer standards-first interoperability, but reviews warn that "support" is ambiguous and can create false compatibility claims.

Accepted MVP compatibility claims are intentionally narrow: OpenRouter-backed model access, local Git project support, root `AGENTS.md` display, and app-owned text-like artifact records. Broader standards claims require this matrix.

# Questions To Answer

 - What does the app support for AGENTS.md: display, parse, execute as guidance, nested resolution, export?
 - What does the app support for SKILL.md: discover, display, invoke, run scripts, resolve assets?
 - What MCP features are supported: tools, resources, prompts, roots, sampling?
 - What Codex plugin fields are supported?
 - What OpenCode config behavior affects runtime execution and must be disclosed?
 - What OpenRouter fields are provider-canonical versus app-canonical?
 - Which claims are safe for MVP docs, UI, and marketing?

# Hypothesis

The app should use an app-owned canonical model and treat external standards as adapters with explicit conformance tiers.

# Investigation Plan

 - List every external standard mentioned in specs and ADRs.
 - Define conformance tier labels.
 - Map MVP, V1, V2, and Future support per standard.
 - Mark all non-MVP standards claims as unsupported until a tier is assigned.
 - Treat OpenCode runtime settings as app-owned adapter plumbing for MVP, not config compatibility.
 - Identify data that must round-trip versus display only.
 - Identify unsupported fields and extension namespace rules.

# Success Criteria

 - A standards conformance matrix is complete.
 - Compatibility claims are precise and testable.
 - Unsupported or partial support is explicitly labeled.
 - MVP docs do not imply full AGENTS.md, Agent Skills, MCP, Codex plugin, OpenCode config, import/export, or round-trip compatibility.

# Decisions Unlocked

 - ADR-005: Standards-First Interoperability.
 - ADR-013: AGENTS.md Instruction Support.
 - ADR-012: Agent Skills Support.
 - ADR-014: Plugin System Compatibility.

# Estimated Effort

2 to 3 architecture days.
