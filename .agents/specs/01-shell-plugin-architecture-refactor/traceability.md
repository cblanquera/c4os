# Shell Plugin Architecture Refactor Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 001, 010 | Persistent shell packaging and unassigned chat scope from exact grill answers |
| REQ-002 | AC-002 | 001, 038 | C4OS app plugin packaging and Codex compatibility boundary |
| REQ-003 | AC-002 | 038, 039 | Marketplace source, cache, uninstall, and reinstall decisions |
| REQ-004 | AC-003 | 002, 002A, 002B, 040 | Runtime discovery and app-owned backend gateway boundary |
| REQ-005 | AC-003 | 003, 003A, 004 | Tool event fanout and per-chat plugin instance scope |
| REQ-006 | AC-004 | 050 | Plugin migration failure handling |

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
