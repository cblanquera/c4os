# Chat Prompt Interactions Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 020, 021, 021A, 022, 023, 024 | Approval categories and per-tool policy display requirements |
| REQ-002 | AC-002 | 025, 026, 049 | Prompt tag targets and gateway command routing |
| REQ-003 | AC-002 | 018, 019, 026 | Dependency-blocked plugin resources and tag suggestion behavior |
| REQ-004 | AC-002 | 028 | Git branch control visibility and read-only thread branch |
| REQ-005 | AC-003 | 027, 045 | C4OS attachment records and model adapter fallback |
| REQ-006 | AC-003 | 027, 032, 042A, 045 | File references, screenshots, and many annotation attachments |

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
