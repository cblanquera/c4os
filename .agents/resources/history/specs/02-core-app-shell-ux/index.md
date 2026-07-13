# Core App Shell UX

Status: proposed
Created: 2026-07-02

## Purpose

Define the final persistent shell layout, panel behavior, Settings route, and plugin icon interactions.

## Source Boundaries

This spec derives from shared context, references, original goal inputs, and the exact grill answer JSON. It must not depend on sibling specs for project-wide truth.

## Source Routing

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

- `.agents/context/product-specs.md`
  Purpose: Shared product behavior, feature surfaces, and customer workflow truth.
  Load when: checking product behavior, workflow, or feature-surface expectations.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/creative-specs.md`
  Purpose: Shared UI, interaction, layout, accessibility, and visual direction.
  Load when: checking shell layout, UI behavior, visual direction, accessibility, or interaction constraints.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/work-orders.md`
  Purpose: Shared sequencing, guardrails, accepted decisions, and implementation-readiness routing.
  Load when: checking accepted sequencing, guardrails, status, validation needs, or whether work may become active.
  Skip when: the task is unrelated to this source boundary.

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

- `.agents/references/research/final-implementation-import/grill-session/010-c4os-grill-question-010-unassigned-chat-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/011-c4os-grill-question-011-panel-coexistence.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/012-c4os-grill-question-012-panel-visibility-persistence.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/013-c4os-grill-question-013-center-pane-and-panel-resize.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/014-c4os-grill-question-014-settings-placement.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/015-c4os-grill-question-015-plugin-icon-reordering.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/015a-c4os-grill-question-015a-plugin-configuration-entry.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

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
