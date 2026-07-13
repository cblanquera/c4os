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
| REQ-007 | AC-003 | Architect resolution | Windows ConPTY/fallback and macOS/Linux PTY behavior |
| REQ-008 | AC-004 | Architect resolution, EVD-WF-001 | Output bounds, backpressure, scrollback, cleanup, and memory caps remain spec behavior |
| REQ-009 | AC-005 | Architect resolution, EVD-WF-001 | Typed terminal lifecycle and user PTY event separation |
| REQ-010 | AC-001, AC-005 | 047, EVD-WF-001 | Runtime terminal tool events are separate from user PTY panel events |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#terminal-user-pty` | REQ-001, REQ-005, REQ-010 | AC-001, AC-005 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-002 through REQ-010 | AC-001 through AC-005 |

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
