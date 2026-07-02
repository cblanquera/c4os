# Runtime And Tool Policy Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 002, 002A, 002B, 040 | Tool gateway, discovery, invocation, and plugin backend authority |
| REQ-002 | AC-001 | 002B, 003, 003A | Runtime can invoke tools without plugin view; events fan out to views |
| REQ-003 | AC-001 | 020, 021, 021A, 022, 023, 024 | Per-tool approval defaults and risk boundaries |
| REQ-004 | AC-001 | 041 | config.toml source behavior and last-valid fallback |
| REQ-005 | AC-002 | 025, 026, 049 | Prompt command routing through runtime/tool gateway |
| REQ-006 | AC-002 | 049 | Pi proof-critical capabilities |

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
