# Shell Plugin Architecture Refactor Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 001, 010 | Persistent shell packaging and unassigned chat scope from exact grill answers |
| REQ-002 | AC-002 | 001, 038 | C4OS app plugin packaging and Codex compatibility boundary |
| REQ-003 | AC-002 | 038, 039 | Marketplace source, cache, uninstall, and reinstall decisions |
| REQ-004 | AC-003 | 002, 002A, 002B, 040 | Runtime discovery and app-owned backend gateway boundary |
| REQ-005 | AC-003, AC-005, AC-006 | 003, 003A, 004, 2026-07-02 grill refinements | Tool event fanout, per-chat plugin instance scope, hidden plugin state updates, and inspectable view-oriented tool state |
| REQ-006 | AC-004 | 050 | Plugin migration failure handling |
| REQ-007 | AC-007 | 004, 040, 2026-07-02 grill refinement | Per-chat plugin instances are lightweight view/state instances; heavy services are shared and lifecycle-managed outside per-chat view instances |

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

## POC Coverage

| Proof | Requirements | Evidence |
| --- | --- | --- |
| `proofs/tool-event-fanout/` | REQ-005, REQ-007 | EVD-POC-001 |
| `proofs/bundled-plugin-lifecycle/` | REQ-002, REQ-003 | EVD-POC-002 |
| `proofs/plugin-migration-failure-handling/` | REQ-008 | EVD-POC-003 |
