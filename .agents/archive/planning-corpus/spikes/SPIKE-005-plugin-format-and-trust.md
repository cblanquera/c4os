# Objective

Validate whether Codex-compatible plugins should be supported, and define the safe MVP posture for plugin scripts, hooks, assets, and manifests.

# Context

Specs prefer Codex-compatible plugin layout, but reviews warn that the fields may not be stable and that executable plugin content increases supply-chain risk.

# Questions To Answer

 - Which Codex plugin manifest fields are stable enough to parse?
 - What is the conformance level: display, parse, execute, round-trip, or export?
 - Should plugin scripts be disabled in MVP?
 - Should plugin hooks be disabled in MVP?
 - How are plugin assets previewed safely?
 - Are dependency install steps ever run automatically?
 - What plugin linting or quarantine behavior is needed?

# Hypothesis

MVP should not execute plugin scripts or hooks. At most, it should parse and display plugin metadata until sandboxing and source integrity are proven.

# Investigation Plan

 - Review current Codex plugin scaffold and manifest conventions.
 - Identify required versus optional fields.
 - Classify plugin surfaces by risk: skills, MCP, scripts, hooks, assets, app metadata.
 - Evaluate static validation rules.
 - Define safe compatibility tiers.
 - Document which plugin capabilities are excluded from MVP.

# Success Criteria

 - Plugin conformance matrix is produced.
 - MVP plugin execution posture is recommended.
 - Script, hook, asset, and dependency-install risks are documented.
 - The team can decide whether plugins are MVP, V1, or later.

# Decisions Unlocked

 - ADR-014: Plugin System Compatibility.
 - ADR-016: Plugin Trust And Execution.
 - ADR-015: Marketplace Model.

# Estimated Effort

2 to 4 engineering days.

