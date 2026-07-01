# Shell Plugin Architecture Refactor

Status: proposed
Created: 2026-07-02

## Purpose

Define the persistent chat shell and app-plugin architecture boundary without starting implementation.

## Source Boundaries

This spec derives from shared context, references, original goal inputs, and the exact grill answer JSON. It must not depend on sibling specs for project-wide truth.

## Sources

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

## Package Files

- `status.md`
- `requirements.md`
- `acceptance.md`
- `decisions.md`
- `risks.md`
- `evidence.md`
- `tasks.md`
- `traceability.md`
- `poc/index.md`
