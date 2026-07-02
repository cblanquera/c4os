# Skills Settings Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 037, 048 | Bundled skill creator and customization rule |
| REQ-002 | AC-001 | 037 | Bundled and user-global skill sources |
| REQ-003 | AC-001 | 037 | Plugin-provided skills follow parent plugin enablement |
| REQ-004 | AC-001 | 025, 026, 037 | $ tag visibility requirements |
| REQ-005 | AC-002 | 048 | Bundled read-only and user-global copy customization |
| REQ-006 | AC-002 | 048 | Invalid skill state taxonomy |
| REQ-007 | AC-002 | 048 | Project-local skills require FS if promoted |
| REQ-008 | AC-002 | 048 | Invalid skills hidden from suggestions with repair actions |
| REQ-001 through REQ-012 | AC-001 through AC-006 | POC `proofs/skills-settings-invalid-states/` | Metadata-first source, customization, invalid-state, and suggestion-filtering proof |

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
