# Browser Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-003 | 024 | Browser navigation/actions allowed by default |
| REQ-002 | AC-001 | 024, 027, 042A, 045 | Screenshots and annotations attach as prompt evidence |
| REQ-003 | AC-001 | 042A | Many Codex-style annotation attachments |
| REQ-004 | AC-001 | 042A | Persist sent attachments and clear active annotations |
| REQ-005 | AC-002 | 036, 036A | Document-family preview boundary and corrected Browser role |
| REQ-006 | AC-003 | 010, 024 | Browser activation does not create chat item |
| REQ-007 | AC-004 | 002B, 024, 2026-07-02 grill refinements | Browser-oriented runtime tool state can exist before a visible Browser view and hydrate later |

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
