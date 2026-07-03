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

- `proofs/approval-ui-flow/`
  Purpose: Runnable proof for typed approval decisions, remember duration
    summaries, thread/Chat Debug summary display, and Settings > Configuration
    review/edit/revoke routing.
  Load when: checking approval UI POC evidence, event shape, remembered-rule
    summaries, or Settings policy routing before freeze.
  Skip when: local decisions and POC results already answer the question.

- `proofs/prompt-tag-resolution/`
  Purpose: Runnable proof for `$` Skills, `@` enabled resources/files, `/`
    runtime/tool-gateway command routing, and disabled-resource hiding.
  Load when: checking prompt resolver POC evidence or frontend/backend parsing
    boundaries before freeze.
  Skip when: local decisions and POC results already answer the question.

- `proofs/attachment-compatibility/`
  Purpose: Runnable proof for prompt-level C4OS attachment records, Browser
    screenshot and annotation handoff, OpenAI-compatible adaptation, visible
    degradation, redacted logs, and clearing active annotations after send.
  Load when: checking prompt attachment POC evidence or compatibility handoff
    before freeze.
  Skip when: local decisions and POC results already answer the question.

- `proofs/model-attachment-adapter/`
  Purpose: Prior runtime/tool-policy proof for provider adapter translation and
    degradation on the OpenAI-compatible provider path.
  Load when: checking the provider-adapter decision that
    `proofs/attachment-compatibility/` depends on.
  Skip when: only prompt-level attachment collection is in scope.

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

## Approved Wireframe Evidence

| Evidence | Source | Result |
| --- | --- | --- |
| EVD-WF-001 | `wireframes/r05-final-implementation/index.html#prompt-suggestions`; `wireframes/r05-final-implementation/review-round-13.md`; `wireframes/r05-final-implementation/qa/notes.md` | Approved on 2026-07-03 as Batch 3 prompt-reference wireframe evidence. The approved UI uses trigger-scoped `$`, `@`, and `/` typeahead above the fixed prompt, filters an active query by caret context, supports up/down/enter selection, keeps selected and exact-match references blue inline without bubble styling, and separates displayed resolved inline references from the serialized prompt sent to runtime. |
| EVD-WF-002 | `wireframes/r05-final-implementation/index.html#approval-dialog`; `wireframes/r05-final-implementation/index.html#remembered-rule-summary`; `wireframes/r05-final-implementation/review-round-19.md`; `wireframes/r05-final-implementation/qa/notes.md` | Approved on 2026-07-03 as Batch 3 approval-flow wireframe evidence. The approved UI shows readable approval copy, Deny, Deny and wait, Allow once, and Allow and remember actions, an Advanced accordion for tool/action/scope metadata and remember duration, and a remembered-rule summary that routes user-global review/edit/revoke to Settings > Configuration. |
| EVD-WF-003 | `wireframes/r05-final-implementation/index.html#blocked-suggestion-repair`; `wireframes/r05-final-implementation/index.html#safe-fallback`; `wireframes/r05-final-implementation/index.html#coverage` | Approved on 2026-07-03 as Batch 3 repair and fallback evidence. The approved UI shows disabled/dependency-blocked suggestions only with a repair route and presents safe fallback messaging for unsupported attachment or provider states. |
| EVD-WF-004 | `wireframes/r05-final-implementation/index.html#branch-popover`; `wireframes/r05-final-implementation/index.html#attachment-states`; `wireframes/r05-final-implementation/index.html#coverage` | Approved on 2026-07-03 as Batch 3 branch and attachment evidence. The approved UI covers branch choose/create, file attachment chips, Browser screenshot attachments, Browser annotation bundle attachments, unsupported attachment warning, and coverage mapping across specs 05, 04, 07, 10, and 11. |
