# Shell Plugin Architecture Refactor Evidence

Status: proposed

## Primary Sources

- `.agents/references/research/final-implementation-import/adhoc-goals.md`
- `.agents/references/research/final-implementation-import/accepted-instructions.md`
- `.agents/references/research/final-implementation-source-inventory.md`
- `.agents/context/product-brief.md`
- `.agents/context/technical-specs.md`
- `.agents/context/work-orders.md`
- `.agents/references/research/final-implementation-source-inventory.md`
- `.agents/references/research/final-implementation-import/grill-session/001-c4os-grill-question-001-plugin-packaging-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`
- `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`
- `.agents/references/research/final-implementation-import/grill-session/003-c4os-grill-question-003-tool-view-selection.json`
- `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`
- `.agents/references/research/final-implementation-import/grill-session/004-c4os-grill-question-004-plugin-instance-scope.json`
- `.agents/references/research/final-implementation-import/grill-session/017-c4os-grill-question-017-c4os-plugin-manifest.json`
- `.agents/references/research/final-implementation-import/grill-session/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`
- `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/050-c4os-grill-question-050-plugin-migration-failure-handling.json`

## Grill QIDs

- 001: C4OS Grill Question 001 - Plugin Packaging Boundary (`.agents/references/research/final-implementation-import/grill-session/001-c4os-grill-question-001-plugin-packaging-boundary.json`)
- 002: C4OS Grill Question 002 - Plugin Backend Authority (`.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`)
- 002A: C4OS Grill Question 002A - Tauri Tool Authority Boundary (`.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`)
- 002B: C4OS Grill Question 002B - Runtime Tool Discovery And Invocation (`.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`)
- 003: C4OS Grill Question 003 - Tool View Selection (`.agents/references/research/final-implementation-import/grill-session/003-c4os-grill-question-003-tool-view-selection.json`)
- 003A: C4OS Grill Question 003A - Tool Event Fanout (`.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`)
- 004: C4OS Grill Question 004 - Plugin Instance Scope (`.agents/references/research/final-implementation-import/grill-session/004-c4os-grill-question-004-plugin-instance-scope.json`)
- 017: C4OS Grill Question 017 - C4OS Plugin Manifest (`.agents/references/research/final-implementation-import/grill-session/017-c4os-grill-question-017-c4os-plugin-manifest.json`)
- 038: C4OS Grill Question 038 - Codex Plugin Compatibility Boundary (`.agents/references/research/final-implementation-import/grill-session/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`)
- 039: C4OS Grill Question 039 - Plugin Marketplace And Lifecycle (`.agents/references/research/final-implementation-import/grill-session/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`)
- 040: C4OS Grill Question 040 - Plugin Backend Registration Boundary (`.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`)
- 050: C4OS Grill Question 050 - Plugin Migration Failure Handling (`.agents/references/research/final-implementation-import/grill-session/050-c4os-grill-question-050-plugin-migration-failure-handling.json`)
