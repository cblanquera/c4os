# MVP Traceability

Status: ready-for-review
Updated: 2026-06-21

## Requirement To Acceptance

| Requirement | Acceptance |
| --- | --- |
| REQ-001 Trusted Folder Gate | AC-001, AC-014 |
| REQ-002 Workspace Descriptor Safety | AC-002 |
| REQ-003 Provider And Model Setup | AC-003, AC-015, AC-018 |
| REQ-004 C4OS-Owned Runtime Adapter | AC-004 |
| REQ-005 Persistent Resumable Sessions | AC-005 |
| REQ-006 Concurrent Session Isolation | AC-006 |
| REQ-007 App-Owned Approval Policy | AC-007, AC-011, AC-012, AC-019 |
| REQ-008 Trusted-Root Files | AC-008, AC-017 |
| REQ-009 Artifact And HTML Preview Safety | AC-009 |
| REQ-010 Browser Surface | AC-010 |
| REQ-011 Backend-Owned Terminal Surfaces | AC-011 |
| REQ-012 Deterministic Command Allowlist | AC-012 |
| REQ-013 r04 Desktop Shell | AC-013, AC-014, AC-018, AC-021 |
| REQ-014 Structured Prompt And Thread States | AC-015, AC-016 |
| REQ-015 Extension Install, Enablement, And Invocation | AC-019, AC-020 |
| REQ-016 Implementation Paths | AC-001 through AC-021, TASK-011 |

## Requirement To Tasks

| Requirement | Proposed Tasks |
| --- | --- |
| REQ-001, REQ-013, REQ-014 | TASK-001, TASK-002, TASK-004 |
| REQ-002, REQ-003, REQ-004, REQ-005 | TASK-002, TASK-003, TASK-004, TASK-005 |
| REQ-006 | TASK-005, TASK-006, TASK-008 |
| REQ-007 | TASK-002, TASK-003, TASK-007, TASK-008 |
| REQ-008, REQ-009 | TASK-002, TASK-003, TASK-005, TASK-007 |
| REQ-010, REQ-011, REQ-012 | TASK-002, TASK-003, TASK-005, TASK-007 |
| REQ-015 | TASK-002, TASK-003, TASK-005, TASK-007 |
| REQ-016 | TASK-001 through TASK-008 |

## Acceptance To Evidence

| Acceptance | Evidence |
| --- | --- |
| AC-001, AC-014 | EVD-001, EVD-002, EVD-007 |
| AC-002, AC-003 | EVD-001, EVD-009 |
| AC-004, AC-005 | EVD-004, EVD-009 |
| AC-006 | EVD-009 |
| AC-007 | EVD-001, EVD-005, EVD-006, EVD-008 |
| AC-008, AC-009 | EVD-001, EVD-005, EVD-007 |
| AC-010 | EVD-005 |
| AC-011, AC-012 | EVD-006 |
| AC-013, AC-015, AC-016, AC-017, AC-018, AC-021 | EVD-002, EVD-003, EVD-007 |
| AC-019, AC-020 | EVD-008 |

## Checkpoint Mapping

| Checkpoint | MVP Coverage |
| --- | --- |
| Frontend foundation | TASK-001; r04 frontend completed before user review |
| Mock acceptance gate | TASK-002; frontend connected to fake processing and dummy data, then paused for user acceptance |
| Backend mock parity | TASK-003; backend returns the same mock behavior without claiming real completion |
| First user-flow activation | TASK-004; first real end-to-end journey, then paused for user acceptance |
| Feature unlock sequence | TASK-005; one feature slice at a time until feature complete |
| Feature-complete acceptance | TASK-006; pause before policy hardening |
| Security and approval hardening | TASK-007; policies implemented after feature behavior exists |
| Release readiness | TASK-008; final integration and acceptance coverage |

## Source Map

| Source | MVP Records |
| --- | --- |
| `.agents/specs/research/research-freeze.md` | brief, status, requirements, decisions, viability gaps |
| `.agents/specs/research/requirements.md` | REQ-001 through REQ-016 |
| `.agents/specs/research/acceptance.md` | AC-001 through AC-021 |
| `.agents/specs/research/decisions.md` | DEC-001 through DEC-012 |
| `.agents/specs/research/evidence.md` | EVD-001 through EVD-009 |
| `.agents/specs/research/viability-gaps.md` | mvp-viability-gaps.md |
| `.agents/specs/research/implementation-checkpoint-plan.md` | checkpoint mapping and task sequencing |
| `wireframes/ui-handoff-spec.md` | shell, composer, thread, tools, settings, placeholder guard |
| `docs/adr/0001-browser-and-agent-authority.md` | Browser scope and authority |
| `docs/adr/0002-extension-enablements-and-prompt-tags.md` | extension enablement and prompt tags |
