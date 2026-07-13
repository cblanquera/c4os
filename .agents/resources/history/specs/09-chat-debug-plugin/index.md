# Chat Debug Plugin

Status: proposed
Created: 2026-07-02

## Purpose

Define developer-oriented debug view for CLI commands/results, tool use, tool events, approvals, and redaction.

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
