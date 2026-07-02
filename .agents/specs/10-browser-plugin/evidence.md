# Browser Plugin Evidence

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

- `.agents/context/technical-specs.md`
  Purpose: Shared technical boundaries for runtime, tools, plugins, approvals, persistence, and execution surfaces.
  Load when: checking architecture, runtime, tool, plugin, approval, storage, or execution boundaries.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/creative-specs.md`
  Purpose: Shared UI, interaction, layout, accessibility, and visual direction.
  Load when: checking shell layout, UI behavior, visual direction, accessibility, or interaction constraints.
  Skip when: the task is unrelated to this source boundary.

- `.agents/references/research/final-implementation-import/research/plugin-shell-research-pass-2-2026-07-01.md`
  Purpose: Supporting source or evidence for this spec.
  Load when: checking supporting evidence or provenance for this spec.
  Skip when: current context and spec records already answer the task.

- `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/027-c4os-grill-question-027-attachments-first-pass.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/036-c4os-grill-question-036-browser-document-preview.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/036a-c4os-grill-question-036a-document-preview-plugin-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/042-c4os-grill-question-042-browser-capture-annotation-and-state.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/042a-c4os-grill-question-042a-browser-annotation-attachment-model.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`
  Purpose: Exact accepted grill answer for 024: C4OS Grill Question 024 - Browser Tool Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 024.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/027-c4os-grill-question-027-attachments-first-pass.json`
  Purpose: Exact accepted grill answer for 027: C4OS Grill Question 027 - Attachments First Pass.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 027.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/036-c4os-grill-question-036-browser-document-preview.json`
  Purpose: Exact accepted grill answer for 036: C4OS Grill Question 036 - Browser Document Preview.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 036.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/036a-c4os-grill-question-036a-document-preview-plugin-boundary.json`
  Purpose: Exact accepted grill answer for 036A: C4OS Grill Question 036A - Document Preview Plugin Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 036A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/042-c4os-grill-question-042-browser-capture-annotation-and-state.json`
  Purpose: Exact accepted grill answer for 042: C4OS Grill Question 042 - Browser Capture, Annotation, And State.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 042.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/042a-c4os-grill-question-042a-browser-annotation-attachment-model.json`
  Purpose: Exact accepted grill answer for 042A: C4OS Grill Question 042A - Browser Annotation Attachment Model.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 042A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
