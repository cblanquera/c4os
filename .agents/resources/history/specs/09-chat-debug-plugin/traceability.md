# Chat Debug Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 035, 043, EVD-WF-001 | Disabled-by-default developer plugin remains settings/spec behavior |
| REQ-002 | AC-002 | 035, 043, 047, EVD-WF-001 | CLI commands/results, tool use/events, approvals |
| REQ-003 | AC-001 | 043, EVD-WF-001 | Current and historical runs for active chat |
| REQ-004 | AC-001 | 043, 2026-07-02 grill refinement, EVD-WF-001 | Hard redaction floor including plugin settings marked sensitive |
| REQ-005 | AC-002 | 043, EVD-WF-001 | No export surface |
| REQ-006 | AC-003 | 043, EVD-WF-001 | Structured event detail with typed fields |
| REQ-007 | AC-001 | 043, EVD-WF-001 | Redaction before debug display |
| REQ-008 | AC-004 | 043, EVD-WF-001 | Retention and deletion cleanup remain spec behavior |
| REQ-009 | AC-002 | EVD-WF-001 | Support-friendly diagnostic records |
| REQ-002 through REQ-008 | AC-001 through AC-005 | POC `proofs/chat-debug-redaction-history/` | Redacted typed history proof with approval visibility and no export |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#debug` | REQ-002, REQ-006, REQ-009 | AC-002, AC-003 |
| `wireframes/r05-final-implementation/index.html#debug-timeline` | REQ-002, REQ-003, REQ-006 | AC-002, AC-003 |
| `wireframes/r05-final-implementation/index.html#debug-event-detail` | REQ-004, REQ-005, REQ-006, REQ-007 | AC-001, AC-003, AC-005 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001 through REQ-009 | AC-001 through AC-005 |

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
