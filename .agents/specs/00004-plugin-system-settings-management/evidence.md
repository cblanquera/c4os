# Plugin System And Settings Management Evidence

Status: proposed

## Primary Source Routing

- `.agents/resources/research/final-implementation-import/adhoc-goals.md`
  Purpose: Original user-stated final implementation goals and task inventory.
  Load when: checking the original requested task inventory or whether scope was imported correctly.
  Skip when: current context and spec records already answer the task.

- `.agents/resources/research/final-implementation-import/accepted-instructions.md`
  Purpose: Accepted replay and workflow instructions, including source-of-truth and ADR placement constraints.
  Load when: checking replay instructions, workflow constraints, source-of-truth placement, or migration rules.
  Skip when: current context and spec records already answer the task.

- `.agents/resources/research/final-implementation-source-inventory.md`
  Purpose: Inventory of imported final-implementation sources, grill counts, and provenance status.
  Load when: auditing source coverage, imported grill data, or provenance completeness.
  Skip when: current context and spec records already answer the task.

- `.agents/context/product-specs.md`
  Purpose: Shared product behavior, feature surfaces, and customer workflow truth.
  Load when: checking product behavior, workflow, or feature-surface expectations.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/technical-specs.md`
  Purpose: Shared technical boundaries for runtime, tools, plugins, approvals, persistence, and execution surfaces.
  Load when: checking architecture, runtime, tool, plugin, approval, storage, or execution boundaries.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/creative-specs.md`
  Purpose: Shared UI, interaction, layout, accessibility, and visual direction.
  Load when: checking shell layout, UI behavior, visual direction, accessibility, or interaction constraints.
  Skip when: the task is unrelated to this source boundary.

- `.agents/resources/history/context/work-orders.md`
  Purpose: Shared sequencing, guardrails, accepted decisions, and implementation-readiness routing.
  Load when: checking accepted sequencing, guardrails, status, validation needs, or whether work may become active.
  Skip when: the task is unrelated to this source boundary.

- `.agents/resources/research/final-implementation-import/research/codex-plugin-marketplace-research-2026-07-01.md`
  Purpose: Supporting source or evidence for this spec.
  Load when: checking supporting evidence or provenance for this spec.
  Skip when: current context and spec records already answer the task.

- `.agents/resources/grill/final-implementation/answers/001-c4os-grill-question-001-plugin-packaging-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/005-c4os-grill-question-005-plugin-configuration-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/006-c4os-grill-question-006-user-config-location.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/015a-c4os-grill-question-015a-plugin-configuration-entry.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/016-c4os-grill-question-016-plugin-settings-schema.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/017-c4os-grill-question-017-c4os-plugin-manifest.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/018-c4os-grill-question-018-plugin-dependencies.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/019-c4os-grill-question-019-disabling-plugin-dependencies.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/050-c4os-grill-question-050-plugin-migration-failure-handling.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/resources/grill/final-implementation/answers/001-c4os-grill-question-001-plugin-packaging-boundary.json`
  Purpose: Exact accepted grill answer for 001: C4OS Grill Question 001 - Plugin Packaging Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 001.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/005-c4os-grill-question-005-plugin-configuration-scope.json`
  Purpose: Exact accepted grill answer for 005: C4OS Grill Question 005 - Plugin Configuration Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 005.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/006-c4os-grill-question-006-user-config-location.json`
  Purpose: Exact accepted grill answer for 006: C4OS Grill Question 006 - User Config Location.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 006.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/015a-c4os-grill-question-015a-plugin-configuration-entry.json`
  Purpose: Exact accepted grill answer for 015A: C4OS Grill Question 015A - Plugin Configuration Entry.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 015A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/016-c4os-grill-question-016-plugin-settings-schema.json`
  Purpose: Exact accepted grill answer for 016: C4OS Grill Question 016 - Plugin Settings Schema.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 016.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/017-c4os-grill-question-017-c4os-plugin-manifest.json`
  Purpose: Exact accepted grill answer for 017: C4OS Grill Question 017 - C4OS Plugin Manifest.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 017.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/018-c4os-grill-question-018-plugin-dependencies.json`
  Purpose: Exact accepted grill answer for 018: C4OS Grill Question 018 - Plugin Dependencies.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 018.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/019-c4os-grill-question-019-disabling-plugin-dependencies.json`
  Purpose: Exact accepted grill answer for 019: C4OS Grill Question 019 - Disabling Plugin Dependencies.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 019.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/038-c4os-grill-question-038-codex-plugin-compatibility-boundary.json`
  Purpose: Exact accepted grill answer for 038: C4OS Grill Question 038 - Codex Plugin Compatibility Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 038.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/039-c4os-grill-question-039-plugin-marketplace-and-lifecycle.json`
  Purpose: Exact accepted grill answer for 039: C4OS Grill Question 039 - Plugin Marketplace And Lifecycle.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 039.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
  Purpose: Exact accepted grill answer for 040: C4OS Grill Question 040 - Plugin Backend Registration Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 040.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`
  Purpose: Exact accepted grill answer for 044: C4OS Grill Question 044 - Plugin SVG Icon Constraints.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 044.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/050-c4os-grill-question-050-plugin-migration-failure-handling.json`
  Purpose: Exact accepted grill answer for 050: C4OS Grill Question 050 - Plugin Migration Failure Handling.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 050.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## POC Execution Evidence

| Evidence | Source | Result |
| --- | --- | --- |
| EVD-POC-001 | `proofs/plugin-settings-renderer/` | `node --test proofs/plugin-settings-renderer/proof.test.mjs` passed on 2026-07-02. Supports settings rendering, persistence, redaction, warnings, and reserved-key validation. |
| EVD-POC-002 | `proofs/codex-marketplace-install-cache/` | `node --test proofs/codex-marketplace-install-cache/proof.test.mjs` passed on 2026-07-02. Supports install cache, uninstall, and reinstall semantics. |
| EVD-POC-003 | `proofs/plugin-svg-sanitization/` | `node --test proofs/plugin-svg-sanitization/proof.test.mjs` passed on 2026-07-02. Supports sanitized static SVG with fallback. |
| EVD-POC-004 | `proofs/plugin-lifecycle-pending-restart-and-service-scope/` | `node --test proofs/plugin-lifecycle-pending-restart-and-service-scope/proof.test.mjs` passed on 2026-07-02. Supports live UI settings, pending-restart backend tools, dependency states, and shared service lifecycle. |

## Approved Wireframe Evidence

| Evidence | Source | Result |
| --- | --- | --- |
| EVD-WF-001 | `wireframes/r05-final-implementation/index.html#settings-plugin-marketplace`, `#settings-plugin-detail`, `#settings-plugin-states`, `#settings-plugin-uninstall`, `#repair-state`, `#coverage`; `wireframes/r05-final-implementation/review-round-07.md`; `wireframes/r05-final-implementation/qa/notes.md` | Approved on 2026-07-02 as Batch 2 Settings wireframe evidence for plugin settings and configuration behavior. The approved UI shows the Plugins page with the Built by C4OS source dropdown, the five pending-spec C4OS plugins, add-marketplace dialog, plugin connect modal, Advanced settings route to rendered plugin config, dependency-blocked/incompatible/pending-restart/repair/icon-fallback states, uninstall data prompt, sensitive-setting redaction, and panel/icon-order configuration examples. |
