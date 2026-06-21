# MVP Traceability

Status: frozen-for-implementation
Updated: 2026-06-21

## Purpose

This file maps the MVP contract across requirements, acceptance criteria,
accepted task records, evidence, and the normalized context owners. Spec task
records remain planning records; active work lives in
`.agents/development/progress/`.

## Requirement Traceability

| Requirement | Acceptance | Proposed Tasks | Evidence | Context Owner |
| --- | --- | --- | --- | --- |
| REQ-001 Trusted Folder Gate | AC-001, AC-014 | TASK-001, TASK-002, TASK-004, TASK-007 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-002 Workspace Descriptor Safety | AC-002 | TASK-002, TASK-003, TASK-004, TASK-007, TASK-008 | EVD-001, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-003 Provider And Model Setup | AC-003, AC-015, AC-018 | TASK-001, TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-004 C4OS-Owned Runtime Adapter | AC-004 | TASK-003, TASK-004, TASK-005, TASK-008 | EVD-004, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-005 Persistent Resumable Sessions | AC-005 | TASK-002, TASK-003, TASK-004, TASK-005, TASK-008 | EVD-001, EVD-004, EVD-009 | `product-specs.md`, `technical-specs.md` |
| REQ-006 Concurrent Session Isolation | AC-006 | TASK-002, TASK-003, TASK-005, TASK-006, TASK-008 | EVD-001, EVD-009 | `product-specs.md`, `technical-specs.md`, `work-orders.md` |
| REQ-007 App-Owned Approval Policy | AC-007, AC-011, AC-012, AC-019 | TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-001, EVD-005, EVD-006, EVD-008, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-008 Trusted-Root Files | AC-008, AC-017 | TASK-001, TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-009 Artifact And HTML Preview Safety | AC-009 | TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-001, EVD-005, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md` |
| REQ-010 Browser Surface | AC-010 | TASK-001, TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-002, EVD-005, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-011 Backend-Owned Terminal Surfaces | AC-011 | TASK-001, TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-002, EVD-006, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-012 Deterministic Command Allowlist | AC-012 | TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-006, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-013 r04 Desktop Shell | AC-013, AC-014, AC-018, AC-021 | TASK-001, TASK-002, TASK-004, TASK-006, TASK-008 | EVD-002, EVD-003, EVD-007, EVD-009 | `creative-specs.md`, `product-specs.md` |
| REQ-014 Structured Prompt And Thread States | AC-015, AC-016 | TASK-001, TASK-002, TASK-004, TASK-005, TASK-008 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `creative-specs.md` |
| REQ-015 Extension Install, Enablement, And Invocation | AC-019, AC-020 | TASK-001, TASK-002, TASK-003, TASK-005, TASK-007, TASK-008 | EVD-001, EVD-008, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-016 Implementation Paths | AC-001 through AC-021 | TASK-001, TASK-002, TASK-003, TASK-004, TASK-005, TASK-006, TASK-007, TASK-008 | EVD-009 | `technical-specs.md`, `work-orders.md` |

## Acceptance To Evidence

| Acceptance | Evidence | Notes |
| --- | --- | --- |
| AC-001 Trust Gate | EVD-001, EVD-002, EVD-007, EVD-009 | Product intent, r04 app start, and frozen scope. |
| AC-002 Workspace Descriptor Safety | EVD-001, EVD-009 | Product model and frozen scope. |
| AC-003 Provider Key Storage | EVD-001, EVD-009 | Product security model and frozen scope. |
| AC-004 Runtime Adapter Boundary | EVD-004, EVD-009 | Runtime POCs and research closeout. |
| AC-005 Session Resume | EVD-001, EVD-004, EVD-009 | Product state model, runtime references, and frozen scope. |
| AC-006 Concurrent Runs | EVD-001, EVD-009 | Product intent and frozen scope. |
| AC-007 Sensitive Action Approval | EVD-001, EVD-005, EVD-006, EVD-008, EVD-009 | Approval boundary spans Browser, Terminal, and extensions. |
| AC-008 Trusted-Root File Access | EVD-001, EVD-007, EVD-009 | Product trust model and UI handoff. |
| AC-009 Safe Artifact Preview | EVD-001, EVD-005, EVD-007, EVD-009 | Product safety model, Browser proof, and UI handoff. |
| AC-010 Browser Scope | EVD-005, EVD-007, EVD-009 | Browser POCs/ADR and accepted r04 surface. |
| AC-011 Terminal Boundary | EVD-006, EVD-007, EVD-009 | Terminal POCs and accepted r04 surface. |
| AC-012 Command Allowlist | EVD-006, EVD-009 | Terminal POCs and frozen scope. |
| AC-013 Three-Panel Shell | EVD-002, EVD-003, EVD-007, EVD-009 | Interface brief, pegs, and r04 handoff. |
| AC-014 App Start Entry Points | EVD-001, EVD-002, EVD-003, EVD-007, EVD-009 | Product and r04 app-start sources. |
| AC-015 Composer Context | EVD-001, EVD-002, EVD-007, EVD-009 | Product prompt context and r04 handoff. |
| AC-016 Structured Thread | EVD-001, EVD-002, EVD-007, EVD-009 | Product session model and r04 handoff. |
| AC-017 Files Surface | EVD-001, EVD-002, EVD-003, EVD-007, EVD-009 | Product file model and r04 handoff. |
| AC-018 Settings IA | EVD-002, EVD-007, EVD-009 | Interface brief and r04 handoff. |
| AC-019 Extension Enablement | EVD-001, EVD-008, EVD-009 | Product extension model and ADR. |
| AC-020 Extension Prompt Tags | EVD-008, EVD-009 | Extension ADR and frozen scope. |
| AC-021 Wireframe Placeholder Guard | EVD-002, EVD-003, EVD-007, EVD-009 | r04 handoff and visual peg sources. |

## Task Coverage

| Task | Primary Coverage | Notes |
| --- | --- | --- |
| TASK-001 Build Full r04 Frontend | REQ-001, REQ-003, REQ-008, REQ-010, REQ-011, REQ-013, REQ-014, REQ-015, REQ-016 | Builds product shell and visible surfaces from r04 before real backend activation. |
| TASK-002 Connect Frontend To Mock Server Harness | REQ-001 through REQ-016 | Mock-backed coverage must state every fake surface listed in `TASK-002`. |
| TASK-003 Build Backend Mock Parity | REQ-002 through REQ-012, REQ-015, REQ-016 | Creates real backend surface while still returning mock behavior. |
| TASK-004 Activate First User Flow | REQ-001 through REQ-005, REQ-008, REQ-013, REQ-014, REQ-016 | First real end-to-end flow from App Start to useful session state. |
| TASK-005 Unlock Feature Slices One At A Time | REQ-003 through REQ-015, REQ-016 | Replaces mock behavior with real feature slices listed in `TASK-005`; order may change, scope may not shrink. |
| TASK-006 Pause At Feature Complete | REQ-006, REQ-013, REQ-016 | User acceptance gate before final security and approval hardening. |
| TASK-007 Implement Security And Approval Policies | REQ-001, REQ-002, REQ-003, REQ-007 through REQ-012, REQ-015, REQ-016 | Hardens trusted roots, descriptors, keys, preview isolation, Browser, Terminal, extensions, and audit logs. |
| TASK-008 Complete Remaining Integration And Release Readiness | REQ-001 through REQ-016 | Final integration, packaging, QA, accessibility, restart/resume, failure states, and MVP acceptance verification. |

## Checkpoint Mapping

| Checkpoint | MVP Coverage |
| --- | --- |
| Frontend foundation | TASK-001; r04 frontend completed before user review. |
| Mock acceptance gate | TASK-002; frontend connected to fake processing and dummy data, then paused for user acceptance. |
| Backend mock parity | TASK-003; backend returns the same mock behavior without claiming real completion. |
| First user-flow activation | TASK-004; first real end-to-end journey, then paused for user acceptance. |
| Feature unlock sequence | TASK-005; one feature slice at a time until feature complete. |
| Feature-complete acceptance | TASK-006; pause before final policy hardening. |
| Security and approval hardening | TASK-007; policies implemented against the feature-complete surface. |
| Release readiness | TASK-008; final integration and acceptance coverage. |

## Source Map

| Source | MVP Records |
| --- | --- |
| `.agents/context/product-brief.md` | product frame, success measures, principles, vocabulary routing |
| `.agents/context/product-specs.md` | customer workflow, feature surfaces, MVP exclusions |
| `.agents/context/technical-specs.md` | product model, runtime boundary, persistence, security, Browser, Terminal, implementation paths |
| `.agents/context/creative-specs.md` | r04 shell, UI behavior, settings IA, visual/accessibility rules |
| `.agents/context/work-orders.md` | accepted decisions, work order sequencing, validation needs, implementation guardrails |
| `.agents/specs/research/research-freeze.md` | brief, status, requirements, decisions, viability gaps |
| `.agents/specs/research/requirements.md` | REQ-001 through REQ-016 |
| `.agents/specs/research/acceptance.md` | AC-001 through AC-021 |
| `.agents/specs/research/decisions.md` | DEC-001 through DEC-012 |
| `.agents/specs/research/evidence.md` | EVD-001 through EVD-009 |
| `.agents/specs/research/viability-gaps.md` | `mvp-viability-gaps.md` |
| `.agents/specs/research/implementation-checkpoint-plan.md` | checkpoint mapping and task sequencing |
| `wireframes/ui-handoff-spec.md` | shell, composer, thread, tools, settings, placeholder guard |
| `docs/adr/0001-browser-and-agent-authority.md` | Browser scope and authority |
| `docs/adr/0002-extension-enablements-and-prompt-tags.md` | extension enablement and prompt tags |
