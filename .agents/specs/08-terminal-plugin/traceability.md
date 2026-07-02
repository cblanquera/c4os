# Terminal Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 047 | Terminal plugin owns user PTY panel only |
| REQ-002 | AC-001 | 034 | Exactly one user terminal per chat session |
| REQ-003 | AC-002 | 034 | No-project cwd uses user home directory |
| REQ-004 | AC-001 | 034 | Remove chat terminates user terminal and state |
| REQ-005 | AC-001 | 023, 047 | Runtime terminal tools remain gateway-owned |
| REQ-006 | AC-002 | 041, 047 | config.toml versus plugin UI preference split |

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
