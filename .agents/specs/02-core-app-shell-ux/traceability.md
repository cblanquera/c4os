# Core App Shell UX Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 011, 014 | No fixed right-panel tab model; Settings center route behavior |
| REQ-002 | AC-004 | 015, 015A, 016, 044 | Header icon ordering, settings entry, schema keys, SVG constraints |
| REQ-003 | AC-001 | 015A | Primary click toggles panel; config in Settings > Plugins |
| REQ-004 | AC-002 | 011 | One visible panel per side and same-side replacement |
| REQ-005 | AC-002, AC-005, AC-006 | 012, 003A, 2026-07-02 grill refinements, proofs/shell-panel-resize-and-restore | Per-chat visible panel persistence, hidden-compatible state update behavior, and shared hydration of app-owned tool result state |
| REQ-006 | AC-003 | 013, proofs/shell-panel-resize-and-restore | 640px center-pane minimum and collision rule |
| REQ-007 | AC-002 | 014, proofs/shell-panel-resize-and-restore | Settings center route closes and restores panels |

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

## Wireframe Coverage

| Wireframe State | Requirements | Acceptance | Evidence |
| --- | --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#shell-foundation` | REQ-001, REQ-002 | AC-001, AC-004 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#same-side-replacement` | REQ-003, REQ-004 | AC-002 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#per-chat-restore` | REQ-005 | AC-002 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#settings` | REQ-007 | AC-002, AC-007 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#resize-collision` | REQ-006 | AC-003 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#hidden-activity` | REQ-008 | AC-005, AC-006, AC-008 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#repair-state` | REQ-009 | AC-007, AC-009 | EVD-WF-001 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009 | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007, AC-008, AC-009 | EVD-WF-002 |
