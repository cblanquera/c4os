# Chat Debug Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 035, 043 | Disabled-by-default developer plugin |
| REQ-002 | AC-002 | 035, 043, 047 | CLI commands/results, tool use/events, approvals |
| REQ-003 | AC-001 | 043 | Current and historical runs for active chat |
| REQ-004 | AC-001 | 043, 2026-07-02 grill refinement | Hard redaction floor including plugin settings marked sensitive |
| REQ-005 | AC-002 | 043 | No export |
| REQ-002 through REQ-008 | AC-001 through AC-005 | POC `proofs/chat-debug-redaction-history/` | Redacted typed history proof with approval visibility and no export |

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
