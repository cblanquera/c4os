# Work Orders

Status: active
Created: 2026-06-21
Updated: 2026-06-26
Source Note: Normalized from accepted decisions, constraints, MVP guardrails, and context routing. Detailed decisions are preserved under `.agents/references/context/work-orders/`.

## Purpose

Work Orders is the expandable prework surface for accepted work packages, sequencing, validation needs, deferred work, and implementation guardrails. It is not active execution state. Proposed work becomes active only when the relevant spec is frozen and converted into `.agents/development/progress/` items.

## Load When

- You need accepted decisions, work package status, sequencing, validation needs, deferred work, or implementation guardrails.
- You are deciding what should happen next before creating specs or progress records.
- You need to check whether something is proposed, accepted, blocked, active, done, or deferred.

## Skip When

- You only need product thesis, users, goals, or vocabulary; load `product-brief.md`.
- You only need product behavior, feature surfaces, or MVP user workflow; load `product-specs.md`.
- You only need runtime, security, persistence, Browser isolation, or Terminal ownership; load `technical-specs.md`.
- You only need UI layout, interaction, visual, or accessibility guidance; load `creative-specs.md`.

## Owns

- Work-order status, accepted sequencing, validation work packages, deferred work, and implementation guardrails.

## Does Not Own

- Active execution state, detailed acceptance criteria, product thesis, technical proof detail, or UI handoff detail. Active execution belongs in `.agents/development/progress/` after a frozen spec exists.

## Reference Routing

| Need | Load | Why |
| --- | --- | --- |
| Full accepted restart decisions | `.agents/references/context/work-orders/decisions.md` | Use when checking decision history. |
| Safety or validation blockers behind a work order | `.agents/references/context/technical-specs/constraints.md` | Use when work depends on trust, Browser, Terminal, credentials, or extensions. |
| MVP scope and guardrails behind a work order | `.agents/references/context/product-specs/mvp-feature-list.md` | Use before turning MVP work into tasks or progress. |
| Pre-normalization context routing | `.agents/references/context/index-legacy.md` | Use only when auditing the normalization or checking old routing. |
| Source and artifact provenance | `.agents/references/context/source-provenance.md` | Use only when tracing where work-order facts came from. |

## Status Values

Use these statuses for work-order records:

- `proposed`: recorded but not accepted for execution.
- `accepted`: approved as a work package or sequencing unit.
- `blocked`: cannot proceed without a decision, validation result, dependency, or implementation prerequisite.
- `active`: converted into progress tracking after the relevant spec is frozen.
- `done`: completed and verified.
- `deferred`: intentionally outside the current phase.

## Accepted Decisions

- C4OS is a greenfield restart that uses prior planning records as product intent and evidence.
- The app is folder-first; prompt entry is disabled until a trusted project folder exists.
- C4OS owns product identity for workspaces, projects, sessions, agent runs, approvals, artifacts, providers, files, Browser, Terminal, settings, and extensions.
- Runtime-specific OpenCode or Pi concepts stay behind an adapter unless exposed for advanced compatibility inspection.
- OpenCode is the first runtime backend; Pi remains a later adapter target.
- Provider support starts with OpenAI-compatible profiles and BYOK storage.
- The interface uses a three-panel desktop shell with left navigation, central session content, and right-side Browser, Files, and Terminal tabs.
- The MVP interface remains neutral and utilitarian unless a later design phase approves brand styling.
- All documented/r04 features are MVP scope unless explicitly moved out later. Checkpoint phases are implementation milestones, not MVP scope boundaries.
- Browser, Terminal, plugin, skill, MCP install/connect flows, and concurrent sessions/runs are MVP scope.
- Browser downloads are excluded from MVP.
- Prompt extension tags are `$skill`, `@plugin`, and `^mcp`; MCP invocation is explicit only.

## Current Work Packages

| ID | Status | Work Order | Depends On |
| --- | --- | --- | --- |
| WO-001 | accepted | Preserve five canonical prework documents in `.agents/context/` and detailed breakdowns in `.agents/references/context/`. | This document set |
| WO-002 | proposed | Repair or create `.agents/specs/mvp/` before active distributable MVP implementation. | Product Specs, Technical Specs, Creative Specs |
| WO-003 | proposed | Validate remaining Browser isolation, Terminal ownership, and credentialed OpenCode permission-request behavior before relying on those claims for freeze. | Technical Specs |
| WO-004 | deferred | Add Pi as a first implementation adapter. | Runtime adapter validation |
| WO-005 | deferred | Add Browser downloads, remote shells, SSH, containers, terminal multiplexing, or agent auto-run. | Future feature scope |
| WO-006 | accepted | Define the runtime tool gateway contract before broad approval hardening: runtime requests tools through generic events, C4OS owns authority/execution, and per-session tool config maps tool identities to enabled state, access, and approval policy. | Technical Specs, TASK-016 |
| WO-007 | proposed | Define extension discovery/loading before extension enablement or invocation: skills are `SKILL.md` folders, plugins are manifest bundles, MCP servers are explicit connections, and discovery records metadata without runtime impact. | Technical Specs, TASK-012, post-TASK-014 extension work |
| WO-008 | deferred | Make runtime/provider tool execution emit structured C4OS tool lifecycle events so Agent terminal output reflects real `terminal.run` calls instead of assistant prose or markdown. C4OS should execute those calls through the tool gateway and stream/persist outputs as tool/action/audit records. | Runtime Tool Gateway, TASK-017 |

## Implementation Guardrails

- Production implementation belongs in `backend/`, `frontend/`, and `tests/server/`.
- Do not create or use `src-tauri/`.
- Use `proofs/<proof-name>/` for POCs, spikes, throwaway harnesses, and proof evidence.
- Proposed MVP tasks are not active work until the MVP spec is frozen and converted into progress items.
- Every mock-backed phase must state exactly what is mocked.
- Do not claim product completion until acceptance passes with real behavior or explicitly accepted remaining mocks.

## Validation Needs

- Browser implementation must reconcile desktop browsing, local-file browsing, project-scoped profiles, request-scoped agent browsing, and audit records without privileged bridge exposure.
- Terminal implementation must preserve backend ownership, deterministic command allowlist, approval policy, command audit records, renderer backpressure, and cross-platform PTY behavior.
- Credentialed OpenCode prompt execution and live permission-request capture still need validation because the earlier proof avoided provider credentials and token spend.
- Runtime tool execution should be formalized before broad approval-policy work:
  prompt planning belongs to the runtime, tool execution belongs to C4OS, and
  frontend prompt-text command parsing must not be restored.
- Runtime/provider command output reflection still needs structured tool events:
  assistant prose or markdown that includes command output is not an Agent
  terminal source of truth. A future item must emit `terminal.run` lifecycle
  events, route execution through C4OS, and update the Agent terminal from
  gateway output.
