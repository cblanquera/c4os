# File System Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001 | 006, 007, 008, 008A, 009, 009A, 010, EVD-WF-001 | User-level config/state, workspace files, project identity, and chats |
| REQ-002 | AC-001 | 007, 009, 009A, EVD-WF-001 | Workspace file load/save and folder grouping semantics |
| REQ-003 | AC-001 | 008, 008A, 009A, EVD-WF-001 | Canonical project path identity and shared chats |
| REQ-004 | AC-001, AC-005 | 008, 008A, EVD-WF-002 | Missing project UI, read-only sessions, and relocation |
| REQ-005 | AC-002, AC-003 | 029, 030, 031, EVD-WF-001, EVD-WF-002 | Workspace/project actions, clone, reorder, remove, rename, reveal/copy |
| REQ-006 | AC-001, AC-007 | 031, EVD-WF-003 | Project/chat search takeover behavior |
| REQ-007 | AC-001 | 010, 024 | Browser activation must not create project chat rows |
| REQ-008 | AC-004 | EVD-WF-004 | Platform path, reveal/copy path, watch, and trash/recycle behavior remains adapter-owned |
| REQ-009 | AC-005 | 008A, EVD-WF-002 | Missing project metadata supports explicit relink/migration before writable chats resume |
| REQ-010 | AC-006 | EVD-WF-003 | Non-Git workspaces remain first-class and hide irrelevant Git-only controls |
| REQ-011 | AC-007 | EVD-WF-001, EVD-WF-003 | Workspace/project lifecycle state must hydrate shell and plugin UI without DOM text scraping |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#workspace-start` | REQ-001, REQ-002, REQ-003, REQ-005 | AC-001, AC-007 |
| `wireframes/r05-final-implementation/index.html#workspace-loaded` | REQ-001, REQ-002, REQ-003, REQ-005, REQ-011 | AC-001, AC-007 |
| `wireframes/r05-final-implementation/index.html#workspace-missing-project` | REQ-004, REQ-009 | AC-005 |
| `wireframes/r05-final-implementation/index.html#workspace-search` | REQ-006 | AC-007 |
| `wireframes/r05-final-implementation/index.html#workspace-non-git` | REQ-010 | AC-006 |
| `wireframes/r05-final-implementation/index.html#file-operations` | REQ-008 | AC-004 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-008, REQ-009, REQ-010, REQ-011 | AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-007 |

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
