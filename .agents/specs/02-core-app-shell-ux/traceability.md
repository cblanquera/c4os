# Core App Shell UX Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 011, 014 | No fixed right-panel tab model; Settings center route behavior |
| REQ-002 | AC-004 | 015, 015A, 016, 044 | Header icon ordering, settings entry, schema keys, SVG constraints |
| REQ-003 | AC-001 | 015A | Primary click toggles panel; config in Settings > Plugins |
| REQ-004 | AC-002 | 011 | One visible panel per side and same-side replacement |
| REQ-005 | AC-002, AC-005, AC-006 | 012, 003A, 2026-07-02 grill refinements | Per-chat visible panel persistence, hidden-compatible state update behavior, and shared hydration of app-owned tool result state |
| REQ-006 | AC-003 | 013 | 640px center-pane minimum and collision rule |
| REQ-007 | AC-002 | 014 | Settings center route closes and restores panels |

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
