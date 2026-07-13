# File Editor Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 018, 019, EVD-WF-003 | IDE requires FS dependency and dependency enforcement |
| REQ-002 | AC-001, AC-007 | 015A, 016, EVD-WF-003 | Plugin panel placement and UI preferences through settings |
| REQ-003 | AC-002 | 032, EVD-WF-001, EVD-WF-003 | Explorer context menu Copy Path and Add to chat |
| REQ-004 | AC-002 | 025, 032, EVD-WF-001, EVD-WF-003 | Inline @ file tag reference behavior, resolved inline display, and serialized runtime reference behavior |
| REQ-005 | AC-001, AC-007 | 033, EVD-WF-003 | File icon theme and hidden-file visibility |
| REQ-006 | AC-003, AC-004, AC-005, AC-006 | 046, EVD-WF-003 | Save/revert, create/rename/delete, trash, and conflict handling |
| REQ-007 | AC-004 | 046, EVD-WF-003 | Backend file service and policy gate authority for file operations |
| REQ-008 | AC-005 | 046, EVD-WF-003 | Platform-aware trash/recycle, path casing, icon lookup, and external-change adapters |
| REQ-009 | AC-006 | 032, 046, EVD-WF-001, EVD-WF-003 | Typed editor/file-operation events for prompt, Chat Debug, and plugin hydration |
| REQ-010 | AC-007 | EVD-WF-003 | Non-code text, docs, config, and operations-file empty states |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#prompt-suggestions` | REQ-003, REQ-004, REQ-009 | AC-002, AC-006 |
| `wireframes/r05-final-implementation/index.html#attachment-states` | REQ-004, REQ-009 | AC-002, AC-006 |
| `wireframes/r05-final-implementation/index.html#blocked-suggestion-repair` | REQ-001, REQ-007, REQ-009 | AC-001, AC-006 |
| `wireframes/r05-final-implementation/index.html#files-left-panel` | REQ-001, REQ-002, REQ-005 | AC-001, AC-007 |
| `wireframes/r05-final-implementation/index.html#files-right-panel` | REQ-002 | AC-007 |
| `wireframes/r05-final-implementation/index.html#file-editor` | REQ-001, REQ-006, REQ-007 | AC-001, AC-003 |
| `wireframes/r05-final-implementation/index.html#file-context-menu` | REQ-003, REQ-004, REQ-009 | AC-002, AC-006 |
| `wireframes/r05-final-implementation/index.html#file-operations` | REQ-006, REQ-007, REQ-008 | AC-004, AC-005, AC-006 |
| `wireframes/r05-final-implementation/index.html#file-editor-dirty` | REQ-006, REQ-007, REQ-009 | AC-003, AC-006 |
| `wireframes/r05-final-implementation/index.html#file-external-conflict` | REQ-006, REQ-007, REQ-008 | AC-003, AC-006 |
| `wireframes/r05-final-implementation/index.html#file-empty-states` | REQ-010 | AC-007 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010 | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007 |

## Source Rule

This spec derives project-wide truth from `.agents/context/` and detailed evidence from `.agents/resources/research/`, `.agents/resources/grill/`, and `.agents/resources/history/`. It must not depend on sibling specs for project-wide truth.

## Reference Routing

- `.agents/resources/history/context/work-orders.md`
  Purpose: Shared accepted sequencing, guardrails, and pending-spec routing.
  Load when: checking whether this spec is proposed, frozen, or ready for execution conversion.
  Skip when: only reading local requirement coverage.

- `.agents/resources/grill/final-implementation/answers/`
  Purpose: Exact grill Q&A JSON source records.
  Load when: verifying a QID, answer wording, note, or superseded question.
  Skip when: local decisions and context already answer the planning question.
