# Core App Shell UX Evidence

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

## Grill Answer Routing

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
- `.agents/references/research/final-implementation-import/grill-session/010-c4os-grill-question-010-unassigned-chat-scope.json`
  Purpose: Exact accepted grill answer for 010: C4OS Grill Question 010 - Unassigned Chat Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 010.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/011-c4os-grill-question-011-panel-coexistence.json`
  Purpose: Exact accepted grill answer for 011: C4OS Grill Question 011 - Panel Coexistence.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 011.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/012-c4os-grill-question-012-panel-visibility-persistence.json`
  Purpose: Exact accepted grill answer for 012: C4OS Grill Question 012 - Panel Visibility Persistence.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 012.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/013-c4os-grill-question-013-center-pane-and-panel-resize.json`
  Purpose: Exact accepted grill answer for 013: C4OS Grill Question 013 - Center Pane And Panel Resize.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 013.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/014-c4os-grill-question-014-settings-placement.json`
  Purpose: Exact accepted grill answer for 014: C4OS Grill Question 014 - Settings Placement.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 014.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/015-c4os-grill-question-015-plugin-icon-reordering.json`
  Purpose: Exact accepted grill answer for 015: C4OS Grill Question 015 - Plugin Icon Reordering.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 015.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/015a-c4os-grill-question-015a-plugin-configuration-entry.json`
  Purpose: Exact accepted grill answer for 015A: C4OS Grill Question 015A - Plugin Configuration Entry.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 015A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`
  Purpose: Exact accepted grill answer for 044: C4OS Grill Question 044 - Plugin SVG Icon Constraints.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 044.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
