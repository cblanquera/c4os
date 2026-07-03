# File Editor Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 018, 019 | IDE requires FS dependency and dependency enforcement |
| REQ-002 | AC-001 | 015A, 016 | Plugin panel placement and UI preferences through settings |
| REQ-003 | AC-002 | 032, EVD-WF-001 | Explorer context menu Copy Path and Add to chat |
| REQ-004 | AC-002 | 025, 032, EVD-WF-001 | Inline @ file tag reference behavior, resolved inline display, and serialized runtime reference behavior |
| REQ-005 | AC-001 | 033 | File icon theme and hidden-file visibility |
| REQ-006 | AC-003 | 046 | Save/revert, create/rename/delete, trash, and conflict handling |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#prompt-suggestions` | REQ-003, REQ-004, REQ-009 | AC-002, AC-006 |
| `wireframes/r05-final-implementation/index.html#attachment-states` | REQ-004, REQ-009 | AC-002, AC-006 |
| `wireframes/r05-final-implementation/index.html#blocked-suggestion-repair` | REQ-001, REQ-007, REQ-009 | AC-001, AC-006 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001, REQ-003, REQ-004, REQ-007, REQ-009 | AC-001, AC-002, AC-006 |

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
