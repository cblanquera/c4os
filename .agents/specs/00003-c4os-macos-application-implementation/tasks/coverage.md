# Feature Coverage Ledger

All 50 normative IDs from [Feature coverage](../feature-coverage.md) are mapped below. State is `open` until the primary and supporting tasks are verified, their Agent Acceptance passes, and the coordinator inspects the integrated evidence; only then may it become `closed`.

| ID | Primary task | Supporting tasks | State | Evidence |
| --- | --- | --- | --- | --- |
| UX-001 | 00006 | 00001, 00007, 00013 | open | Not yet recorded |
| UX-002 | 00015 | 00006, 00007, 00013, 00015B | open | Not yet recorded |
| UX-003 | 00005 | 00006, 00015B | open | Not yet recorded |
| UX-004 | 00006 | 00001, 00015 | open | Not yet recorded |
| UX-005 | 00007 | 00006, 00008, 00013 | open | Not yet recorded |
| UX-006 | 00007 | 00006, 00008 | open | Not yet recorded |
| UX-007 | 00007 | 00004, 00008, 00009, 00010 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) production streaming, cancellation, immutable turn/attempt, stale-correlation, and recovery support passed; conversation and facility integration remain open. |
| UX-008 | 00008 | 00006, 00007, 00009, 00010 | open | Not yet recorded |
| UX-009 | 00015 | 00005, 00006, 00007, 00008, 00013, 00015B | open | Not yet recorded |
| UX-010 | 00003 | 00002, 00004, 00008 through 00014 | open | [Task 00003](00003-policy-credentials-action-gateway.md) production policy, credential, journal, and executor boundary passed. [Task 00004](00004-runtime-provider-capability-lifecycle.md) added exact OpenCode/Pi provider dispatch, private credentials, attachments, approvals, persistence, and cleanup; facilities remain open. |
| UX-011 | 00006 | 00007, 00009, 00010, 00015 | open | Not yet recorded |
| UX-012 | 00003 | 00004, 00007, 00013 | open | [Task 00003](00003-policy-credentials-action-gateway.md) policy/activity acceptance passed. [Task 00004](00004-runtime-provider-capability-lifecycle.md) added atomic route capability evidence, attachment preflight, dependent controls, and native approval continuation; Chat and Settings integration remain open. |
| UX-013 | 00002 | 00003, 00011, 00012, 00014 | open | [Task 00002](00002-durable-core-configuration-workspace.md) persistence/configuration/archive acceptance passed; supporting security and extension lifecycle remain open. |
| UX-014 | 00002 | 00007, 00011, 00012 | open | [Task 00002](00002-durable-core-configuration-workspace.md) exact inactivation/no-delete acceptance passed; integrated scoped-process and extension behavior remain open. |
| UX-015 | 00003 | 00004, 00007, 00011, 00012, 00015A | open | [Task 00003](00003-policy-credentials-action-gateway.md) one-use authorization and approval lifecycle passed. [Task 00004](00004-runtime-provider-capability-lifecycle.md) proved OpenCode/Pi allow and deny continuations through the same Rust-owned Action Gateway; extension and final security audit remain open. |
| UI-001 | 00005 | 00006, 00015B | open | Not yet recorded |
| UI-002 | 00005 | 00006, 00007, 00008, 00013, 00015B | open | Not yet recorded |
| UI-003 | 00005 | 00006, 00013, 00015B | open | Not yet recorded |
| UI-004 | 00005 | 00006, 00007, 00008, 00013, 00015B | open | Not yet recorded |
| UI-005 | 00005 | 00006, 00007, 00008, 00013, 00015B | open | Not yet recorded |
| CHAT-001 | 00007 | 00006 | open | Not yet recorded |
| CHAT-002 | 00007 | 00002, 00005, 00015 | open | Not yet recorded |
| CHAT-003 | 00007 | 00002, 00004 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) provisional promotion and atomic first-submit runtime/route/environment/capability binding passed; production conversation promotion remains with Task 00007. |
| CHAT-004 | 00007 | 00004, 00006 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) immutable turns, streamed events, busy/completion, cancellation, retry ancestry, and durable attempt history passed; the full renderer/editor remains open. |
| CHAT-005 | 00007 | 00004, 00005 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) descriptor-rooted exact attachment materialization, bounds, native image/PDF parts, Pi image support, and incompatible/preflight states passed; native picker/drop composition remains open. |
| CHAT-006 | 00007 | 00004, 00013 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) provider/model route identity, atomic effective capabilities, reasoning/attachment controls, and stale-generation rejection passed; Chat information and Settings composition remain open. |
| CHAT-007 | 00007 | 00003, 00004, 00008, 00009, 00010 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) proved AI dispatch stays provider/runtime routed and all proposed effects stay C4OS brokered; composer mode and direct-facility surfaces remain open. |
| CHAT-008 | 00007 | 00006, 00008, 00009, 00010 | open | Not yet recorded |
| CHAT-009 | 00003 | 00002, 00004, 00007, 00008, 00015A | open | [Task 00003](00003-policy-credentials-action-gateway.md) repository-sensitive policy passed. [Task 00004](00004-runtime-provider-capability-lifecycle.md) proved runtime tool proposals cannot bypass Action Gateway approval or denial; Chat activity, changed-file artifacts, and final audit remain open. |
| CHAT-010 | 00003 | 00002, 00005, 00007, 00015A | open | [Task 00003](00003-policy-credentials-action-gateway.md) Git-only visibility/scope, safe dirty switch, conflict paths, unchanged-worktree, and no automatic stash/commit/reset/discard acceptance passed; native composer integration remains open. |
| ART-001 | 00008 | 00006, 00007, 00009, 00010 | open | Not yet recorded |
| ART-002 | 00010 | 00003, 00005, 00008, 00015A, 00015B | open | Not yet recorded |
| ART-003 | 00008 | 00002, 00003, 00007, 00015A | open | Not yet recorded |
| ART-004 | 00008 | 00002, 00003, 00007 | open | Not yet recorded |
| ART-005 | 00009 | 00003, 00004, 00007, 00008, 00015A | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) supplies supervised process generations, stale-event rejection, broker-only effects, and descendant cleanup required by Terminal work; the Terminal facility remains open. |
| ART-006 | 00007 | 00004, 00006, 00008, 00015B | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) supplies immutable attempt/stream/provenance state and recovery transitions; the accepted work-activity artifact and final accessibility audit remain open. |
| ART-007 | 00008 | 00003, 00004, 00007, 00015A | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) supplies descriptor-rooted exact attachment bytes, capability preflight, and safe runtime delivery; Files UI and final security integration remain open. |
| SET-001 | 00013 | 00003, 00004, 00005 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) provider-gated readiness, connectivity invalidation, zero/one/many model discovery, explicit route choice, opaque credentials, and zero-provider fail-closed dispatch passed; onboarding and native shell integration remain open. |
| SET-002 | 00002 | 00005, 00006, 00013 | open | [Task 00002](00002-durable-core-configuration-workspace.md) Start/recents/reconstruction acceptance passed; native picker, shell, and Settings integration remain open. |
| SET-003 | 00005 | 00006, 00013, 00015B | open | Not yet recorded |
| SET-004 | 00013 | 00003, 00004 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) provider profile identity, conditional native mapping, secret-reference preservation, availability, connectivity, and dependent-route invalidation passed; Settings CRUD dialogs remain open. |
| SET-005 | 00013 | 00004, 00006 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) route details, declared/normalized/observed/effective capability evidence, availability, generation, refresh, and responsive QA states passed; Models Settings composition remains open. |
| SET-006 | 00013 | 00004, 00007 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) peer OpenCode/Pi exact pins, dirty draft, save, restart generation, existing-binding stability, and new-Chat default behavior passed; Settings activation remains open. |
| SET-007 | 00011 | 00003, 00006, 00013, 00014, 00015A | open | Not yet recorded |
| SET-008 | 00011 | 00003, 00004, 00006, 00013, 00015A | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) establishes the runtime capability/authority boundary that future Skills must intersect without peer authority; Skill lifecycle and UI remain open. |
| SET-009 | 00012 | 00003, 00004, 00006, 00013, 00014, 00015A | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) establishes runtime capability evidence, broker-only tools, and stale-generation failure needed by MCP integration; MCP lifecycle, diagnostics, UI, and final audit remain open. |
| SET-010 | 00010 | 00002, 00003, 00005, 00013, 00015A | open | Not yet recorded |
| SET-011 | 00003 | 00006, 00013, 00015A | open | [Task 00003](00003-policy-credentials-action-gateway.md) production-built and native-reviewed seven-group rules, search, exceptions, dirty/revert/save, guardrails, ceilings, and revocation acceptance passed; Settings-shell integration remains open. |
| QA-001 | 00015 | 00001, 00002, 00004, 00006, 00013 | open | [Task 00004](00004-runtime-provider-capability-lifecycle.md) contributed deterministic provider/preflight/runtime/recovery states, strict Tauri envelopes, browser tests, native Computer Use evidence, and golden-path evidence; the integrated deterministic QA adapter remains with Task 00015. |
| QA-002 | 00015 | Every implementation task and 00015A through 00015C | open | Not yet recorded |

Count check: 15 UX + 5 UI + 10 CHAT + 7 ART + 11 SET + 2 QA = 50.
