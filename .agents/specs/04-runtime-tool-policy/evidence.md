# Runtime And Tool Policy Evidence

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

- `.agents/context/technical-specs.md`
  Purpose: Shared technical boundaries for runtime, tools, plugins, approvals, persistence, and execution surfaces.
  Load when: checking architecture, runtime, tool, plugin, approval, storage, or execution boundaries.
  Skip when: the task is unrelated to this source boundary.

- `.agents/context/work-orders.md`
  Purpose: Shared sequencing, guardrails, accepted decisions, and implementation-readiness routing.
  Load when: checking accepted sequencing, guardrails, status, validation needs, or whether work may become active.
  Skip when: the task is unrelated to this source boundary.

- `.agents/references/research/final-implementation-import/research/plugin-shell-research-pass-2-2026-07-01.md`
  Purpose: Supporting source or evidence for this spec.
  Load when: checking supporting evidence or provenance for this spec.
  Skip when: current context and spec records already answer the task.

- `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/021-c4os-grill-question-021-core-tool-approval-defaults.json`
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

- `.agents/references/research/final-implementation-import/grill-session/023-c4os-grill-question-023-non-file-risk-tool-defaults.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/041-c4os-grill-question-041-config-toml-and-tool-policy.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

- `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`
  Purpose: Exact accepted grill answer JSON for this spec decision set.
  Load when: verifying the exact accepted user answer, notes, or answer key for this grill question.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## Grill Answer Routing

- `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`
  Purpose: Exact accepted grill answer for 002: C4OS Grill Question 002 - Plugin Backend Authority.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 002.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`
  Purpose: Exact accepted grill answer for 002A: C4OS Grill Question 002A - Tauri Tool Authority Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 002A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`
  Purpose: Exact accepted grill answer for 002B: C4OS Grill Question 002B - Runtime Tool Discovery And Invocation.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 002B.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`
  Purpose: Exact accepted grill answer for 003A: C4OS Grill Question 003A - Tool Event Fanout.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 003A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`
  Purpose: Exact accepted grill answer for 020: C4OS Grill Question 020 - App Tool Approval Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 020.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/021-c4os-grill-question-021-core-tool-approval-defaults.json`
  Purpose: Exact accepted grill answer for 021: C4OS Grill Question 021 - Core Tool Approval Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 021.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/021a-c4os-grill-question-021a-read-approval-boundary.json`
  Purpose: Exact accepted grill answer for 021A: C4OS Grill Question 021A - Read Approval Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 021A.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/022-c4os-grill-question-022-mutating-tool-defaults.json`
  Purpose: Exact accepted grill answer for 022: C4OS Grill Question 022 - Mutating Tool Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 022.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/023-c4os-grill-question-023-non-file-risk-tool-defaults.json`
  Purpose: Exact accepted grill answer for 023: C4OS Grill Question 023 - Non-File Risk Tool Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 023.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`
  Purpose: Exact accepted grill answer for 024: C4OS Grill Question 024 - Browser Tool Defaults.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 024.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
  Purpose: Exact accepted grill answer for 025: C4OS Grill Question 025 - Prompt Tag Routing.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 025.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
  Purpose: Exact accepted grill answer for 040: C4OS Grill Question 040 - Plugin Backend Registration Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 040.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/041-c4os-grill-question-041-config-toml-and-tool-policy.json`
  Purpose: Exact accepted grill answer for 041: C4OS Grill Question 041 - Config TOML And Tool Policy.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 041.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`
  Purpose: Exact accepted grill answer for 045: C4OS Grill Question 045 - Model Attachment Compatibility.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 045.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
  Purpose: Exact accepted grill answer for 047: C4OS Grill Question 047 - Terminal Plugin Tool Boundary.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 047.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.
- `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`
  Purpose: Exact accepted grill answer for 049: C4OS Grill Question 049 - App Tool Taxonomy And Pi Proof.
  Load when: verifying answer text, notes, conflicts, or decision provenance for 049.
  Skip when: the spec-local decision already contains enough detail and the exact answer is not under audit.

## POC Execution Evidence

| Evidence | Source | Result |
| --- | --- | --- |
| EVD-POC-001 | `proofs/runtime-tool-discovery-without-plugin-view/` | `node --test proofs/runtime-tool-discovery-without-plugin-view/proof.test.mjs` passed on 2026-07-02. Supports registered tool discovery/invocation without plugin views and C4OS-owned view state hydration. |
| EVD-POC-002 | `proofs/user-directed-file-access-policy/` | `node --test proofs/user-directed-file-access-policy/proof.test.mjs` passed on 2026-07-02. Supports user-directed read/write approval boundaries. |
| EVD-POC-003 | `proofs/approval-remember-policy/` | `node --test proofs/approval-remember-policy/proof.test.mjs` passed on 2026-07-02. Supports remembered approval key shape, duration behavior, and per-tool Settings controls. |
| EVD-POC-004 | `proofs/pi-runtime-app-layer-proof/` | `node --test proofs/pi-runtime-app-layer-proof/proof.test.mjs` passed on 2026-07-02. Supports Pi app-layer streaming, tool interception, denial, and resume contract. |
| EVD-POC-005 | `proofs/model-attachment-adapter/` | `node --test proofs/model-attachment-adapter/proof.test.mjs` passed on 2026-07-02. Supports OpenAI-compatible attachment translation, degradation, and redacted logging. |
