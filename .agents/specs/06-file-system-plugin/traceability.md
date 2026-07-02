# File System Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 006, 007, 008, 008A, 009, 009A, 010 | User-level config/state, workspace files, project identity, and chats |
| REQ-002 | AC-001 | 007, 009, 009A | Workspace file load/save and folder grouping semantics |
| REQ-003 | AC-001 | 008, 008A, 009A | Canonical project path identity and shared chats |
| REQ-004 | AC-001 | 008, 008A | Missing project UI, read-only sessions, and relocation |
| REQ-005 | AC-002, AC-003 | 029, 030, 031 | Workspace/project actions, clone, reorder, remove, rename, reveal/copy |
| REQ-006 | AC-001 | 031 | Project/chat search takeover behavior |
| REQ-007 | AC-001 | 010, 024 | Browser activation must not create project chat rows |

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
