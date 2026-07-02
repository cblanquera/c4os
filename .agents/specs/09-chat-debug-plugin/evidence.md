# Chat Debug Plugin Evidence

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

- `.agents/references/research/final-implementation-import/grill-session/035-c4os-grill-question-035-chat-debug-visibility.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/035-c4os-grill-question-035-chat-debug-visibility.json`
  Purpose: Exact accepted grill answer for 035: C4OS Grill Question 035 - Chat Debug Visibility.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 035.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`
  Purpose: Exact accepted grill answer for 043: C4OS Grill Question 043 - Chat Debug Redaction And History.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 043.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
  Purpose: Exact accepted grill answer for 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 047.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
