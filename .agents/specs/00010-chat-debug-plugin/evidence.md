# Chat Debug Plugin Evidence

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

- `.agents/resources/grill/final-implementation/answers/035-c4os-grill-question-035-chat-debug-visibility.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/resources/grill/final-implementation/answers/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/resources/grill/final-implementation/answers/035-c4os-grill-question-035-chat-debug-visibility.json`
  Purpose: Exact accepted grill answer for 035: C4OS Grill Question 035 - Chat Debug Visibility.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 035.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/043-c4os-grill-question-043-chat-debug-redaction-and-history.json`
  Purpose: Exact accepted grill answer for 043: C4OS Grill Question 043 - Chat Debug Redaction And History.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 043.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/resources/grill/final-implementation/answers/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
  Purpose: Exact accepted grill answer for 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 047.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## POC Evidence

- `proofs/chat-debug-redaction-history/`
  Purpose: Runnable proof for typed Chat Debug history, approval visibility,
  redaction-before-persistence, bounded retention, and no export.
  Load when: verifying the Chat Debug POC result, proof harness, or evidence
  README.
  Skip when: the spec-local POC result and decision are enough.

## Approved Wireframe Evidence

| Evidence | Source | Result |
| --- | --- | --- |
| EVD-WF-001 | `wireframes/r05-final-implementation/index.html#debug`; `wireframes/r05-final-implementation/index.html#debug-timeline`; `wireframes/r05-final-implementation/index.html#debug-event-detail`; `wireframes/r05-final-implementation/index.html#coverage`; `wireframes/r05-final-implementation/review-round-44.md`; `wireframes/r05-final-implementation/review-round-45.md`; `wireframes/r05-final-implementation/review-round-46.md`; `wireframes/r05-final-implementation/review-round-47.md`; `wireframes/r05-final-implementation/review-round-50.md`; `wireframes/r05-final-implementation/qa/notes.md` | Approved on 2026-07-06 as Batch 5 Chat Debug plugin evidence. The approved UI shows realistic CLI command/result and tool call/result records, an active-chat current/historical run selector, selected-run event rows, structured event detail with redacted sensitive fields, and no export control. Disabled-by-default visibility, retention/limit, and deletion cleanup remain settings/spec behavior instead of standalone panel routes. |
