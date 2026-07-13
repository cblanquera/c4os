# Core App Shell UX Evidence

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

- `.agents/context/creative-specs.md`
  Purpose: Shared UI, interaction, layout, accessibility, and visual direction.
  Load when: checking shell layout, UI behavior, visual direction, accessibility, or interaction constraints.
  Skip when: the task is unrelated to this source boundary.

- `.agents/resources/history/context/work-orders.md`
  Purpose: Shared sequencing, guardrails, accepted decisions, and implementation-readiness routing.
  Load when: checking accepted sequencing, guardrails, status, validation needs, or whether work may become active.
  Skip when: the task is unrelated to this source boundary.

- `proofs/shell-panel-resize-and-restore/`
  Purpose: Runnable proof for panel toggle, same-side replacement,
    per-session restore, Settings route close/restore, and resize collision
    preserving a 640px center pane.
  Load when: checking shell-panel POC evidence, proof commands, or state-machine
    behavior before freeze.
  Skip when: local decisions and POC results already answer the question.

- `.agents/resources/grill/final-implementation/answers/003-c4os-grill-question-003-tool-view-selection.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/003a-c4os-grill-question-003a-tool-event-fanout.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/004-c4os-grill-question-004-plugin-instance-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/010-c4os-grill-question-010-unassigned-chat-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/011-c4os-grill-question-011-panel-coexistence.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/012-c4os-grill-question-012-panel-visibility-persistence.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/013-c4os-grill-question-013-center-pane-and-panel-resize.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/014-c4os-grill-question-014-settings-placement.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/015-c4os-grill-question-015-plugin-icon-reordering.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/015a-c4os-grill-question-015a-plugin-configuration-entry.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/resources/grill/final-implementation/answers/003-c4os-grill-question-003-tool-view-selection.json`
  Purpose: Exact accepted grill answer for 003: C4OS Grill Question 003 - Tool View Selection.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 003.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/003a-c4os-grill-question-003a-tool-event-fanout.json`
  Purpose: Exact accepted grill answer for 003A: C4OS Grill Question 003A - Tool Event Fanout.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 003A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/004-c4os-grill-question-004-plugin-instance-scope.json`
  Purpose: Exact accepted grill answer for 004: C4OS Grill Question 004 - Plugin Instance Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 004.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/010-c4os-grill-question-010-unassigned-chat-scope.json`
  Purpose: Exact accepted grill answer for 010: C4OS Grill Question 010 - Unassigned Chat Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 010.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/011-c4os-grill-question-011-panel-coexistence.json`
  Purpose: Exact accepted grill answer for 011: C4OS Grill Question 011 - Panel Coexistence.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 011.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/012-c4os-grill-question-012-panel-visibility-persistence.json`
  Purpose: Exact accepted grill answer for 012: C4OS Grill Question 012 - Panel Visibility Persistence.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 012.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/013-c4os-grill-question-013-center-pane-and-panel-resize.json`
  Purpose: Exact accepted grill answer for 013: C4OS Grill Question 013 - Center Pane And Panel Resize.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 013.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/014-c4os-grill-question-014-settings-placement.json`
  Purpose: Exact accepted grill answer for 014: C4OS Grill Question 014 - Settings Placement.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 014.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/015-c4os-grill-question-015-plugin-icon-reordering.json`
  Purpose: Exact accepted grill answer for 015: C4OS Grill Question 015 - Plugin Icon Reordering.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 015.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/015a-c4os-grill-question-015a-plugin-configuration-entry.json`
  Purpose: Exact accepted grill answer for 015A: C4OS Grill Question 015A - Plugin Configuration Entry.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 015A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/044-c4os-grill-question-044-plugin-svg-icon-constraints.json`
  Purpose: Exact accepted grill answer for 044: C4OS Grill Question 044 - Plugin SVG Icon Constraints.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 044.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Wireframe Evidence

| Evidence | Source | Result |
| --- | --- | --- |
| EVD-WF-001 | `wireframes/r05-final-implementation/` | Approved r05 Batch 1 shell-foundation wireframes. Covers single global header, left/right plugin icons, no default right panel, plugin panel toggle/close via icon, one visible panel per side, same-side replacement, per-chat panel restore, Settings center route, resize/collision behavior, hidden plugin unread/activity indicators, and invalid shell layout repair state. |
| EVD-WF-002 | `wireframes/r05-final-implementation/index.html#coverage` | Approved route/state coverage matrix maps r05 shell-foundation states to specs 01 and 02. |
| EVD-WF-003 | `wireframes/r05-final-implementation/README.md` | Records r04 route carry-forward decisions for this final-implementation shell revision, including superseded, deferred, and intentionally not copied routes. |
