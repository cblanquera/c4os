# MVP Traceability

Status: frozen-for-implementation
Updated: 2026-06-22

## Purpose

This file maps the MVP contract across requirements, acceptance criteria,
accepted task records, evidence, and the normalized context owners. Spec task
records remain planning records; active work lives in
`.agents/development/progress/`.

## Requirement Traceability

| Requirement | Acceptance | Proposed Tasks | Evidence | Context Owner |
| --- | --- | --- | --- | --- |
| REQ-001 Trusted Folder Gate | AC-001, AC-014 | TASK-001, TASK-002, TASK-004, TASK-016 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-002 Workspace Descriptor Safety | AC-002 | TASK-002, TASK-003, TASK-004, TASK-016, TASK-017 | EVD-001, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-003 Provider And Model Setup | AC-003, AC-015, AC-018 | TASK-001, TASK-002, TASK-003, TASK-005, TASK-006, TASK-016, TASK-017 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-004 C4OS-Owned Runtime Adapter | AC-004 | TASK-003, TASK-004, TASK-005, TASK-007, TASK-017 | EVD-004, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-005 Persistent Resumable Sessions | AC-005 | TASK-002, TASK-003, TASK-004, TASK-005, TASK-007, TASK-013, TASK-014, TASK-017 | EVD-001, EVD-004, EVD-009 | `product-specs.md`, `technical-specs.md` |
| REQ-006 Concurrent Session Isolation | AC-006 | TASK-002, TASK-003, TASK-013, TASK-015, TASK-017 | EVD-001, EVD-009 | `product-specs.md`, `technical-specs.md`, `work-orders.md` |
| REQ-007 App-Owned Approval Policy | AC-007, AC-011, AC-012, AC-019 | TASK-002, TASK-003, TASK-011, TASK-014, TASK-016, TASK-017 | EVD-001, EVD-005, EVD-006, EVD-008, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-008 Trusted-Root Files | AC-008, AC-017 | TASK-001, TASK-002, TASK-003, TASK-008, TASK-016, TASK-017 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-009 Artifact And HTML Preview Safety | AC-009 | TASK-002, TASK-003, TASK-009, TASK-016, TASK-017 | EVD-001, EVD-005, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md` |
| REQ-010 Browser Surface | AC-010 | TASK-001, TASK-002, TASK-003, TASK-010, TASK-016, TASK-017 | EVD-002, EVD-005, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-011 Backend-Owned Terminal Surfaces | AC-011 | TASK-001, TASK-002, TASK-003, TASK-011, TASK-016, TASK-017 | EVD-002, EVD-006, EVD-007, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-012 Deterministic Command Allowlist | AC-012 | TASK-002, TASK-003, TASK-011, TASK-016, TASK-017 | EVD-006, EVD-009 | `technical-specs.md`, `work-orders.md` |
| REQ-013 r04 Desktop Shell | AC-013, AC-014, AC-018, AC-021, AC-022, AC-023, AC-024, AC-025, AC-026 | TASK-001, TASK-002, TASK-003, TASK-004, TASK-005 through TASK-017 | EVD-002, EVD-003, EVD-007, EVD-009 | `creative-specs.md`, `product-specs.md` |
| REQ-014 Structured Prompt And Thread States | AC-015, AC-016, AC-022, AC-024, AC-025 | TASK-001, TASK-002, TASK-004, TASK-005, TASK-007, TASK-017 | EVD-001, EVD-002, EVD-007, EVD-009 | `product-specs.md`, `creative-specs.md` |
| REQ-015 Extension Install, Enablement, And Invocation | AC-019, AC-020 | TASK-001, TASK-002, TASK-003, TASK-012, TASK-014, TASK-016, TASK-017 | EVD-001, EVD-008, EVD-009 | `product-specs.md`, `technical-specs.md`, `creative-specs.md` |
| REQ-016 Implementation Paths | AC-001 through AC-026 | TASK-001 through TASK-017 | EVD-009 | `technical-specs.md`, `work-orders.md` |

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
| AC-022 r04 Functional Parity | EVD-002, EVD-003, EVD-007, EVD-009 | r04 route, structure, and interaction baseline. |
| AC-023 No Invented Frontend Surfaces | EVD-002, EVD-007, EVD-009 | Prevents unapproved controls or abstractions during frontend foundation. |
| AC-024 Behavior-Based Frontend Verification | EVD-007, EVD-009 | Requires rendered interaction checks beyond source-string assertions. |
| AC-025 Production Frontend Treatment | EVD-002, EVD-003, EVD-007, EVD-009 | Requires production visual treatment over the r04 parity baseline. |
| AC-026 Native Desktop App Menu | EVD-002, EVD-007, EVD-009 | TASK-003 must expose OS-level File/Edit commands without duplicating them as in-app UI. |

## Task Coverage

| Task | Primary Coverage | Notes |
| --- | --- | --- |
| TASK-001 Build Full r04 Frontend | REQ-001, REQ-003, REQ-008, REQ-010, REQ-011, REQ-013, REQ-014, REQ-015, REQ-016 | Builds a production-quality frontend foundation from r04 with functional parity first, production visual treatment second, and no invented controls or source-string-only acceptance. |
| TASK-002 Connect Frontend To Mock Server Harness | REQ-001 through REQ-016 | Mock-backed coverage must state every fake surface listed in `TASK-002` while preserving TASK-001 r04 parity. |
| TASK-003 Build Backend Mock Parity | REQ-002 through REQ-013, REQ-015, REQ-016 | Creates real backend surface and native app menu while still returning mock behavior without changing accepted frontend structure. |
| TASK-004 Activate First User Flow | REQ-001 through REQ-005, REQ-008, REQ-013, REQ-014, REQ-016 | First real end-to-end flow through accepted r04 screens from App Start to useful session state. |
| TASK-005 OpenRouter Chat Session Review Slice | REQ-003, REQ-004, REQ-005, REQ-013, REQ-014, REQ-016 | Replaces the mock chat run with real OpenRouter streaming, reasoning/thinking activity when available, structured runtime activity, and final response behind the accepted r04 chat shell. |
| TASK-005A Scoped Frontend State And DOM Updates | REQ-013, REQ-014, REQ-016 | Consolidates app-owned frontend state and scoped DOM updates before provider/model work, preserving accepted UI while reducing full-shell rerender tech debt. |
| TASK-006 Provider And Model Management Slice | REQ-003, REQ-013, REQ-016 | Replaces provider/model mock records behind accepted Settings and composer surfaces. |
| TASK-007 Runtime Adapter And Persistent Sessions Slice | REQ-004, REQ-005, REQ-014, REQ-016 | Replaces remaining mock session/runtime behavior with C4OS-owned adapter, session, run, message, and runtime-reference records. |
| TASK-008 Files Slice | REQ-008, REQ-013, REQ-016 | Replaces Files mock behavior with trusted-root browsing, editing, saving, reverting, and guarded mutation. |
| TASK-009 Artifact And Safe HTML Preview Slice | REQ-009, REQ-016 | Replaces artifact/preview mocks with product-owned artifact records and safe generated/untrusted HTML rendering. |
| TASK-010 Browser Slice | REQ-010, REQ-013, REQ-016 | Replaces Browser mock behavior with project-scoped profile state, local/public browsing, request-scoped agent browsing, and recorded agent Browser actions. |
| TASK-010A Browser Address Bar And Local Target UI | REQ-010, REQ-013, REQ-016 | Exposes TASK-010 Browser capabilities through the accepted Browser address bar and trusted local-target UI without adding new routes, panels, tabs, or downloads. |
| TASK-010B Native Browser Webview Or External-Open Fallback | REQ-010, REQ-013, REQ-016 | Handles public websites that block iframe embedding while preserving Browser/Files/Terminal tabs, artifact preview isolation, and user-vs-agent browsing authority. |
| TASK-010C Artifact Preview Type Rendering | REQ-009, REQ-010, REQ-013, REQ-016 | Extends product-owned artifact previews by MIME type or file extension while keeping artifacts distinct from general Browser navigation. |
| TASK-011 Terminal Slice | REQ-011, REQ-012, REQ-013, REQ-016 | Replaces Terminal mock behavior with backend-owned terminal lifecycle, cwd validation, output streaming, cancellation, and action records. |
| TASK-012 Settings IA And Extension Records Slice | REQ-015, REQ-013, REQ-016 | Replaces settings and extension mock records behind accepted Settings IA while keeping runtime impact gated. |
| TASK-013 Concurrency And Restart/Resume Slice | REQ-005, REQ-006, REQ-016 | Implements concurrent run isolation and restart/resume behavior for MVP continuation state. |
| TASK-014 Local Memory, Action Records, And Audit Records Slice | REQ-005, REQ-007, REQ-015, REQ-016 | Implements app-owned memory, action, and audit records separate from descriptors and raw runtime/provider state. |
| TASK-015 Pause At Feature Complete | REQ-006, REQ-013, REQ-016 | User acceptance gate before final security and approval hardening. |
| TASK-016 Implement Security And Approval Policies | REQ-001, REQ-002, REQ-003, REQ-007 through REQ-012, REQ-015, REQ-016 | Hardens trusted roots, descriptors, keys, preview isolation, Browser, Terminal, extensions, and audit logs. |
| TASK-017 Complete Remaining Integration And Release Readiness | REQ-001 through REQ-016 | Final integration, packaging, QA, accessibility, restart/resume, failure states, and MVP acceptance verification. |

## Checkpoint Mapping

| Checkpoint | MVP Coverage |
| --- | --- |
| Frontend foundation | TASK-001; r04 functional parity and production visual treatment completed and behavior-verified before user review. |
| Mock acceptance gate | TASK-002; frontend connected to fake processing and dummy data without changing r04 parity, then paused for user acceptance. |
| Backend mock parity | TASK-003; backend returns the same mock behavior and exposes the native app menu contract without claiming real completion or changing accepted frontend structure. |
| First user-flow activation | TASK-004; first real end-to-end journey through accepted r04 screens, then paused for user acceptance. |
| OpenRouter chat review | TASK-005; real streaming chat experience with reasoning/thinking activity and final response behind accepted UI. |
| Scoped frontend state cleanup | TASK-005A; app-owned state and scoped DOM update cleanup before provider/model feature work. |
| Feature unlock sequence | TASK-006 through TASK-014; one feature slice at a time behind accepted UI until feature complete. |
| Feature-complete acceptance | TASK-015; pause before final policy hardening. |
| Security and approval hardening | TASK-016; policies implemented against the feature-complete surface. |
| Release readiness | TASK-017; final integration and acceptance coverage. |

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
