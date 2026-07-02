# Skills Settings Evidence

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

- `.agents/context/work-orders.md`
  Purpose: Shared sequencing, guardrails, accepted decisions, and implementation-readiness routing.
  Load when: checking accepted sequencing, guardrails, status, validation needs, or whether work may become active.
  Skip when: the task is unrelated to this source boundary.

- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/037-c4os-grill-question-037-skills-settings-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/048-c4os-grill-question-048-skills-customization-and-invalid-states.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
  Purpose: Exact accepted grill answer for 025: C4OS Grill Question 025 - Prompt Tag Routing.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 025.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/026-c4os-grill-question-026-disabled-plugin-prompt-tags.json`
  Purpose: Exact accepted grill answer for 026: C4OS Grill Question 026 - Disabled Plugin Prompt Tags.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 026.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/037-c4os-grill-question-037-skills-settings-scope.json`
  Purpose: Exact accepted grill answer for 037: C4OS Grill Question 037 - Skills Settings Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 037.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/048-c4os-grill-question-048-skills-customization-and-invalid-states.json`
  Purpose: Exact accepted grill answer for 048: C4OS Grill Question 048 - Skills Customization And Invalid States.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 048.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## POC Evidence

- `proofs/skills-settings-invalid-states/`
  Purpose: Runnable proof for metadata-first skill discovery, source
  precedence, customization copy, invalid-state repair visibility, and `$`
  suggestion filtering.
  Load when: verifying the Skills Settings invalid-state proof.
  Skip when: the spec-local POC result and decision are enough.
