# Chat Prompt Interactions Evidence

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

- `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/027-c4os-grill-question-027-attachments-first-pass.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/028-c4os-grill-question-028-branch-selection.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/032-c4os-grill-question-032-file-explorer-add-to-chat.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/042a-c4os-grill-question-042a-browser-annotation-attachment-model.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`
  Purpose: Exact accepted grill answer for 020: C4OS Grill Question 020 - App Tool Approval Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 020.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
  Purpose: Exact accepted grill answer for 025: C4OS Grill Question 025 - Prompt Tag Routing.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 025.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`
  Purpose: Exact accepted grill answer for 026: C4OS Grill Question 026 - Disabled Plugin Prompt Tags.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 026.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/027-c4os-grill-question-027-attachments-first-pass.json`
  Purpose: Exact accepted grill answer for 027: C4OS Grill Question 027 - Attachments First Pass.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 027.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/028-c4os-grill-question-028-branch-selection.json`
  Purpose: Exact accepted grill answer for 028: C4OS Grill Question 028 - Branch Selection.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 028.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/032-c4os-grill-question-032-file-explorer-add-to-chat.json`
  Purpose: Exact accepted grill answer for 032: C4OS Grill Question 032 - File Explorer Add To Chat.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 032.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/042a-c4os-grill-question-042a-browser-annotation-attachment-model.json`
  Purpose: Exact accepted grill answer for 042A: C4OS Grill Question 042A - Browser Annotation Attachment Model.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 042A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`
  Purpose: Exact accepted grill answer for 043: C4OS Grill Question 043 - Chat Debug Redaction And History.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 043.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`
  Purpose: Exact accepted grill answer for 045: C4OS Grill Question 045 - Model Attachment Compatibility.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 045.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`
  Purpose: Exact accepted grill answer for 049: C4OS Grill Question 049 - App Tool Taxonomy And Pi Proof.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 049.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
