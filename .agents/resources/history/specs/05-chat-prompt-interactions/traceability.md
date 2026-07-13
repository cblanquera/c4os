# Chat Prompt Interactions Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001, AC-004, AC-005, AC-008 | 020, 021, 021A, 022, 023, 024, 2026-07-02 grill refinement, proofs/approval-ui-flow, EVD-WF-002 | Approval categories, Deny/Deny and wait/Allow once/Allow and remember actions, Advanced metadata, remembered-rule summaries, duration choices, per-tool policy display, and Settings > Configuration review/revoke routing |
| REQ-002 | AC-002, AC-006 | 025, 026, 049, proofs/prompt-tag-resolution, EVD-WF-001 | Prompt trigger/typeahead targets, caret-scoped active queries, keyboard selection, backend-authoritative resolver events, serialized prompt generation, and gateway command routing |
| REQ-003 | AC-002, AC-006 | 018, 019, 026, proofs/prompt-tag-resolution, EVD-WF-003 | Dependency-blocked plugin resources, tag suggestion behavior, and visible repair routing |
| REQ-004 | AC-002 | 028, EVD-WF-004 | Git branch control visibility, choose/create behavior, and read-only thread branch |
| REQ-005 | AC-003, AC-007 | 027, 045, proofs/attachment-compatibility, proofs/model-attachment-adapter, EVD-WF-003, EVD-WF-004 | C4OS attachment records, model adapter fallback, unsupported-state warning, and safe fallback messaging |
| REQ-006 | AC-003, AC-007 | 027, 032, 042A, 045, proofs/attachment-compatibility, EVD-WF-004 | File references, screenshots, and many annotation attachments |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#prompt-suggestions` | REQ-002, REQ-007 | AC-002, AC-006 |
| `wireframes/r05-final-implementation/index.html#approval-dialog` | REQ-001, REQ-009 | AC-001, AC-004, AC-008 |
| `wireframes/r05-final-implementation/index.html#remembered-rule-summary` | REQ-001 | AC-004, AC-005 |
| `wireframes/r05-final-implementation/index.html#blocked-suggestion-repair` | REQ-003 | AC-006 |
| `wireframes/r05-final-implementation/index.html#branch-popover` | REQ-004 | AC-002 |
| `wireframes/r05-final-implementation/index.html#attachment-states` | REQ-005, REQ-006, REQ-008 | AC-003, AC-007 |
| `wireframes/r05-final-implementation/index.html#safe-fallback` | REQ-005, REQ-008 | AC-003, AC-007 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001 through REQ-009 | AC-001 through AC-008 |

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
