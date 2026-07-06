# Browser Plugin Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-003 | 024, EVD-WF-003 | Browser navigation/actions allowed by default |
| REQ-002 | AC-001 | 024, 027, 042A, 045, EVD-WF-001, EVD-WF-003, EVD-WF-004 | Screenshots and annotations attach as prompt evidence |
| REQ-003 | AC-001 | 042A, EVD-WF-004 | Many Codex-style annotation attachments remain spec/Markdown behavior |
| REQ-004 | AC-001 | 042A, EVD-WF-004 | Persist sent attachments and clear active annotations |
| REQ-005 | AC-002 | 036, 036A, EVD-WF-003 | Document-family preview boundary and corrected Browser role |
| REQ-006 | AC-003 | 010, 024, EVD-WF-003 | Browser activation does not create chat item |
| REQ-007 | AC-004 | 002B, 024, 2026-07-02 grill refinements, EVD-WF-003 | Browser-oriented runtime tool state can exist before a visible Browser view and hydrate later |
| REQ-008 | AC-005 | EVD-WF-003, EVD-WF-004 | Typed Browser events for navigation, actions, screenshot, annotation, prompt attachment, clear-after-send, and handoff |
| REQ-009 | AC-005 | EVD-WF-001, EVD-WF-004 | Browser attachment records carry metadata, redaction, and compatibility state |
| REQ-010 | AC-006 | EVD-WF-003 | Platform/security behavior remains spec coverage |
| REQ-011 | AC-007 | EVD-WF-003 | Browser supports general research/evidence workflows |
| REQ-002, REQ-003, REQ-004 | AC-001, AC-005 | POC `proofs/browser-annotation-attachment-model/` | Many annotation evidence bundle proof |
| REQ-005 | AC-002 | POC `proofs/browser-document-preview-boundary/` | Browser hosting and document-family ownership proof |
| REQ-007 | AC-004, AC-005 | POC `proofs/browser-state-hydration-without-visible-view/` | App-owned Browser state hydration proof |

## Approved Wireframe Route Coverage

| Route | Requirements | Acceptance |
| --- | --- | --- |
| `wireframes/r05-final-implementation/index.html#attachment-states` | REQ-002, REQ-003, REQ-004, REQ-008, REQ-009 | AC-001, AC-005 |
| `wireframes/r05-final-implementation/index.html#safe-fallback` | REQ-008, REQ-009 | AC-005 |
| `wireframes/r05-final-implementation/index.html#browser-navigation` | REQ-001, REQ-002, REQ-006, REQ-008, REQ-011 | AC-003, AC-005, AC-007 |
| `wireframes/r05-final-implementation/index.html#browser-menu` | REQ-001, REQ-008, REQ-011 | AC-003, AC-005, AC-007 |
| `wireframes/r05-final-implementation/index.html#browser-page-context-menu` | REQ-001, REQ-002, REQ-003, REQ-008, REQ-011 | AC-001, AC-003, AC-005, AC-007 |
| `wireframes/r05-final-implementation/index.html#browser-preview-host` | REQ-005 | AC-002 |
| `wireframes/r05-final-implementation/browser-annotations.md` | REQ-002, REQ-003, REQ-004, REQ-008, REQ-009 | AC-001, AC-005 |
| `wireframes/r05-final-implementation/index.html#coverage` | REQ-001 through REQ-011 | AC-001 through AC-007 |

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
