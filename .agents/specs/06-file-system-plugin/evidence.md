# File System Plugin Evidence

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

- `.agents/references/research/final-implementation-import/grill-session/006-c4os-grill-question-006-user-config-location.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/007-c4os-grill-question-007-workspace-descriptor-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/008-c4os-grill-question-008-project-identity.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/008a-c4os-grill-question-008a-project-relocation-identity.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/009-c4os-grill-question-009-workspace-registry-layout.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/009a-c4os-grill-question-009a-workspace-chat-identity.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/010-c4os-grill-question-010-unassigned-chat-scope.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/021a-c4os-grill-question-021a-read-approval-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/022-c4os-grill-question-022-mutating-tool-defaults.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/028-c4os-grill-question-028-branch-selection.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/029-c4os-grill-question-029-remove-project-semantics.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/030-c4os-grill-question-030-clone-repository-registration.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/031-c4os-grill-question-031-project-search.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/006-c4os-grill-question-006-user-config-location.json`
  Purpose: Exact accepted grill answer for 006: C4OS Grill Question 006 - User Config Location.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 006.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/007-c4os-grill-question-007-workspace-descriptor-boundary.json`
  Purpose: Exact accepted grill answer for 007: C4OS Grill Question 007 - Workspace Descriptor Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 007.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/008-c4os-grill-question-008-project-identity.json`
  Purpose: Exact accepted grill answer for 008: C4OS Grill Question 008 - Project Identity.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 008.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/008a-c4os-grill-question-008a-project-relocation-identity.json`
  Purpose: Exact accepted grill answer for 008A: C4OS Grill Question 008A - Project Relocation Identity.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 008A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/009-c4os-grill-question-009-workspace-registry-layout.json`
  Purpose: Exact accepted grill answer for 009: C4OS Grill Question 009 - Workspace Registry Layout.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 009.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/009a-c4os-grill-question-009a-workspace-chat-identity.json`
  Purpose: Exact accepted grill answer for 009A: C4OS Grill Question 009A - Workspace Chat Identity.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 009A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/010-c4os-grill-question-010-unassigned-chat-scope.json`
  Purpose: Exact accepted grill answer for 010: C4OS Grill Question 010 - Unassigned Chat Scope.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 010.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/021a-c4os-grill-question-021a-read-approval-boundary.json`
  Purpose: Exact accepted grill answer for 021A: C4OS Grill Question 021A - Read Approval Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 021A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/022-c4os-grill-question-022-mutating-tool-defaults.json`
  Purpose: Exact accepted grill answer for 022: C4OS Grill Question 022 - Mutating Tool Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 022.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/028-c4os-grill-question-028-branch-selection.json`
  Purpose: Exact accepted grill answer for 028: C4OS Grill Question 028 - Branch Selection.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 028.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/029-c4os-grill-question-029-remove-project-semantics.json`
  Purpose: Exact accepted grill answer for 029: C4OS Grill Question 029 - Remove Project Semantics.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 029.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/030-c4os-grill-question-030-clone-repository-registration.json`
  Purpose: Exact accepted grill answer for 030: C4OS Grill Question 030 - Clone Repository Registration.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 030.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/031-c4os-grill-question-031-project-search.json`
  Purpose: Exact accepted grill answer for 031: C4OS Grill Question 031 - Project Search.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 031.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## POC Evidence Routing

- `proofs/fs-workspace-file-and-relink/`
  Purpose: Executable proof for workspace file load/save, user-level chat ownership, missing read-only state, and explicit relink migration.
  Load when: verifying workspace descriptor boundaries, project identity relocation, or missing project behavior.
  Skip when: the spec-local POC result already answers the question.
- `proofs/project-chat-sharing-across-workspaces/`
  Purpose: Executable proof that the same canonical project folder shares chats across multiple workspace files.
  Load when: checking workspace/chat ownership separation or cross-workspace chat hydration.
  Skip when: the spec-local POC result already answers the question.
- `proofs/project-and-chat-removal-semantics/`
  Purpose: Executable proof separating Remove Chat deletion from Remove Project membership removal.
  Load when: checking project removal, chat deletion, or preserved project-history semantics.
  Skip when: the spec-local POC result already answers the question.
