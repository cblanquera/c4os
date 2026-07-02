# Shell Plugin Architecture Refactor Evidence

Status: proposed

## Primary Source Routing

- `.agents/references/research/final-implementation-import/adhoc-goals.md`
  Purpose: Original user-stated final implementation goals and task inventory.
  Load when: checking the original requested task inventory or whether scope was imported correctly.
  Skip when: current context and spec records already answer the task.

- `.agents/references/research/final-implementation-import/accepted-instructions.md`
  Purpose: Accepted replay and workflow instructions, including source-of-truth and ADR placement constraints.
  Load when: checking replay instructions, workflow constraints, source-of-truth placement, or migration rules.
  Skip when: current context and spec records already answer the task.

- `.agents/references/research/final-implementation-source-inventory.md`
  Purpose: Inventory of imported final-implementation sources, grill counts, and provenance status.
  Load when: auditing source coverage, imported grill data, or provenance completeness.
  Skip when: current context and spec records already answer the task.

- `.agents/context/product-brief.md`
  Purpose: Shared product identity, document map, goals, users, and vocabulary.
  Load when: checking product identity, user, goal, or document-routing context.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/technical-specs.md`
  Purpose: Shared technical boundaries for runtime, tools, plugins, approvals, persistence, and execution surfaces.
  Load when: checking architecture, runtime, tool, plugin, approval, storage, or execution boundaries.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/work-orders.md`
  Purpose: Shared sequencing, guardrails, accepted decisions, and implementation-readiness routing.
  Load when: checking accepted sequencing, guardrails, status, validation needs, or whether work may become active.
  Skip when: the task is unrelated to this source boundary.

- `.agents/references/research/final-implementation-import/grill-session/001-c4os-grill-question-001-plugin-packaging-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/003-c4os-grill-question-003-tool-view-selection.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/004-c4os-grill-question-004-plugin-instance-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/017-c4os-grill-question-017-c4os-plugin-manifest.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/050-c4os-grill-question-050-plugin-migration-failure-handling.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/001-c4os-grill-question-001-plugin-packaging-boundary.json`
  Purpose: Exact accepted grill answer for 001: C4OS Grill Question 001 - Plugin Packaging Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 001.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`
  Purpose: Exact accepted grill answer for 002: C4OS Grill Question 002 - Plugin Backend Authority.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 002.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`
  Purpose: Exact accepted grill answer for 002A: C4OS Grill Question 002A - Tauri Tool Authority Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 002A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`
  Purpose: Exact accepted grill answer for 002B: C4OS Grill Question 002B - Runtime Tool Discovery And Invocation.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 002B.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/003-c4os-grill-question-003-tool-view-selection.json`
  Purpose: Exact accepted grill answer for 003: C4OS Grill Question 003 - Tool View Selection.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 003.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`
  Purpose: Exact accepted grill answer for 003A: C4OS Grill Question 003A - Tool Event Fanout.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 003A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/004-c4os-grill-question-004-plugin-instance-scope.json`
  Purpose: Exact accepted grill answer for 004: C4OS Grill Question 004 - Plugin Instance Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 004.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/017-c4os-grill-question-017-c4os-plugin-manifest.json`
  Purpose: Exact accepted grill answer for 017: C4OS Grill Question 017 - C4OS Plugin Manifest.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 017.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`
  Purpose: Exact accepted grill answer for 038: C4OS Grill Question 038 - Codex Plugin Compatibility Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 038.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`
  Purpose: Exact accepted grill answer for 039: C4OS Grill Question 039 - Plugin Marketplace And Lifecycle.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 039.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
  Purpose: Exact accepted grill answer for 040: C4OS Grill Question 040 - Plugin Backend Registration Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 040.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/050-c4os-grill-question-050-plugin-migration-failure-handling.json`
  Purpose: Exact accepted grill answer for 050: C4OS Grill Question 050 - Plugin Migration Failure Handling.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 050.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
