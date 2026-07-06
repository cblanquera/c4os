# Runtime And Tool Policy Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 002, 002A, 002B, 040, EVD-WF-004 | Tool gateway, discovery, invocation, and plugin backend authority |
| REQ-002 | AC-001, AC-004 | 002B, 003, 003A, 2026-07-02 grill refinements, EVD-WF-004 | Runtime can invoke tools without plugin view, events fan out to views, and view-oriented tools leave app-owned per-chat inspectable state |
| REQ-003 | AC-001, AC-006 | 020, 021, 021A, 022, 023, 024, 2026-07-02 grill refinement, EVD-WF-002 | Per-tool approval defaults, risk boundaries, dialog decision actions, Advanced metadata, and remembered-rule key/duration behavior |
| REQ-004 | AC-001 | 041 | config.toml source behavior and last-valid fallback |
| REQ-005 | AC-002 | 025, 026, 049 | Prompt command routing through runtime/tool gateway |
| REQ-006 | AC-002 | 049 | Pi proof-critical capabilities |
| REQ-007 | AC-004, AC-005 | 004, 040, 2026-07-02 grill refinements, EVD-WF-004 | C4OS-owned execution/state, plugin view hydration, and heavy service lifecycle scope |
| REQ-008 | AC-007 | 041, 2026-07-02 grill refinement | Settings > Configuration owns per-server-tool policy items with explanations and review/edit/revoke controls |
| REQ-009 | AC-009 | EVD-WF-004 | Tool lifecycle events feed shell views, plugin views, Chat Debug, runtime adapters, audit records, and provider result streaming |
| REQ-010 | AC-010 | EVD-WF-004 | Tool-call records include request/trace ids, caller, scope, caps, approval, redaction, result, and attachment metadata |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#approval-dialog` | REQ-003, REQ-010 | AC-001, AC-006, AC-009 |
| `wireframes/r05-final-implementation/index.html#remembered-rule-summary` | REQ-003, REQ-008 | AC-006, AC-007 |
| `wireframes/r05-final-implementation/index.html#safe-fallback` | REQ-009, REQ-010 | AC-009, AC-010, AC-011 |
| `wireframes/r05-final-implementation/index.html#terminal-user-pty` | REQ-001, REQ-007, REQ-009, REQ-010 | AC-002, AC-005, AC-009 |
| `wireframes/r05-final-implementation/index.html#browser-navigation` | REQ-002, REQ-007, REQ-009, REQ-010 | AC-004, AC-009, AC-010 |
| `wireframes/r05-final-implementation/index.html#browser-menu` | REQ-002, REQ-007, REQ-009 | AC-004, AC-009 |
| `wireframes/r05-final-implementation/index.html#browser-page-context-menu` | REQ-002, REQ-007, REQ-009, REQ-010 | AC-004, AC-009, AC-010 |
| `wireframes/r05-final-implementation/index.html#browser-preview-host` | REQ-002, REQ-007, REQ-009, REQ-010 | AC-004, AC-009, AC-010 |
| `wireframes/r05-final-implementation/index.html#debug` | REQ-001, REQ-009, REQ-010 | AC-002, AC-009, AC-010 |
| `wireframes/r05-final-implementation/index.html#debug-timeline` | REQ-009, REQ-010 | AC-009, AC-010 |
| `wireframes/r05-final-implementation/index.html#debug-event-detail` | REQ-009, REQ-010 | AC-009, AC-010 |
| `wireframes/r05-final-implementation/browser-annotations.md` | REQ-002, REQ-009, REQ-010 | AC-004, AC-009, AC-010 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-002, REQ-003, REQ-005, REQ-007 through REQ-010 | AC-004 through AC-011 |

## Source Rule

This spec derives project-wide truth from `.agents/context/` and detailed evidence from `.agents/references/`. It must not depend on sibling specs for project-wide truth.

## Reference Routing

- `.agents/context/work-orders.md`
  Purpose: Shared accepted sequencing, guardrails, and pending-spec routing.
  Load when: checking whether this spec is proposed, frozen, or ready for execution conversion.
  Skip when: only reading local requirement coverage.

- `.agents/references/research/final-implementation-import/grill-session/`
  Purpose: Exact grill Q&A JSON source records.
  Load when: verifying a QID, answer wording, note, or superseded question.
  Skip when: local decisions and context already answer the planning question.

## POC Coverage

| Proof | Requirements | Evidence |
| --- | --- | --- |
| `proofs/runtime-tool-discovery-without-plugin-view/` | REQ-001, REQ-002, REQ-007, REQ-009, REQ-010 | EVD-POC-001 |
| `proofs/user-directed-file-access-policy/` | REQ-003, REQ-010 | EVD-POC-002 |
| `proofs/approval-remember-policy/` | REQ-003, REQ-008 | EVD-POC-003 |
| `proofs/pi-runtime-app-layer-proof/` | REQ-005, REQ-006 | EVD-POC-004 |
| `proofs/model-attachment-adapter/` | REQ-009, REQ-010 | EVD-POC-005 |
