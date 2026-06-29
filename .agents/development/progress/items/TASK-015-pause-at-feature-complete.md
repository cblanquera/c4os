# TASK-015 Pause At Feature Complete

Status: accepted
Updated: 2026-06-30

## Source

- Frozen task basis: `.agents/specs/mvp/tasks.md` `TASK-015`
- MVP traceability: `.agents/specs/mvp/traceability.md`
- Progress manifest: `.agents/development/progress/manifest.md`
- Accepted predecessors:
  - `.agents/development/progress/items/TASK-001-r04-frontend.md`
  - `.agents/development/progress/items/TASK-002-mock-server-connection.md`
  - `.agents/development/progress/items/TASK-003-backend-mock-parity.md`
  - `.agents/development/progress/items/TASK-004-first-user-flow.md`
  - `.agents/development/progress/items/TASK-005-openrouter-chat-session.md`
  - `.agents/development/progress/items/TASK-005A-scoped-frontend-state.md`
  - `.agents/development/progress/items/TASK-006-provider-model-management.md`
  - `.agents/development/progress/items/TASK-007-runtime-adapter-persistent-sessions.md`
  - `.agents/development/progress/items/TASK-008-files-slice.md`
  - `.agents/development/progress/items/TASK-009-artifact-safe-html-preview.md`
  - `.agents/development/progress/items/TASK-010-browser-slice.md`
  - `.agents/development/progress/items/TASK-010A-browser-address-bar-local-target-ui.md`
  - `.agents/development/progress/items/TASK-010B-native-browser-webview-or-external-open-fallback.md`
  - `.agents/development/progress/items/TASK-010C-artifact-preview-type-rendering.md`
  - `.agents/development/progress/items/TASK-011-terminal-slice.md`
  - `.agents/development/progress/items/TASK-011A-agent-command-terminal-bridge.md`
  - `.agents/development/progress/items/TASK-011B-chat-session-transition-polish.md`
  - `.agents/development/progress/items/TASK-012-settings-ia-extension-records.md`
  - `.agents/development/progress/items/TASK-013-concurrency-restart-resume.md`
  - `.agents/development/progress/items/TASK-013A-desktop-qa-bootstrap-workspace-provider-persistence.md`
  - `.agents/development/progress/items/TASK-014-local-memory-action-audit-records.md`

## Goal

Pause at the feature-complete MVP checkpoint before `TASK-016` security and
approval hardening. This package summarizes the unlocked feature surface,
accepted evidence, remaining mocks or shortcuts, and the explicit boundary for
stakeholder acceptance.

No product implementation was performed for this item.

## Completed MVP Feature List

- Production r04 desktop shell with App Start, New Session, Chat Session,
  Browser, Files, Terminal, Providers, Models, Runtimes, Configuration,
  Plugins, Skills, and MCP Servers surfaces.
- Connector-backed frontend state, mock server harness, Rust/Tauri backend
  command surface, and native desktop File/Edit menu contract.
- Trusted-folder first journey from App Start through descriptor-backed
  workspace activation into the accepted session shell.
- Real OpenRouter chat review path with streaming runtime activity,
  reasoning/thinking support when available, Markdown assistant rendering, and
  explicit provider setup state when a model cannot run.
- Scoped frontend state and DOM update boundaries for chat turns, work logs,
  right-panel tool switching, composer state, and session-owned surfaces.
- Provider/model management for OpenAI-compatible providers, non-secret
  provider/model records, model discovery/manual entry, enablement, per-session
  model selection, and process/key-store-backed review credential handling.
- C4OS-owned runtime/session records with persistent workspace, session, run,
  message, runtime-event, selected-model, Browser, Files, and Terminal state.
- Trusted-root Files browsing, file open/code view, dirty editing, save,
  revert, guarded traversal and `.git` rejection, native Save/Revert menu
  routing, and session restore.
- Product-owned artifact records and safe Browser/Preview rendering for
  generated/untrusted HTML plus typed previews for Markdown, image, PDF, plain
  text, JSON, and code.
- Browser surface for public URLs, trusted project-local file targets, editable
  address bar, bounded errors, native Wry public Browser host, local-file
  preview handling, session restore, and Browser action records.
- Backend-owned Terminal surface with distinct top user PTY and bottom
  read-only Agent command terminal, explicit prompt command bridge, command
  action records, and session isolation.
- Chat-session transition polish that immediately creates/selects a new chat,
  shows the submitted user turn, protects against stale transcripts, and
  renders working/completed/failed/setup-required states.
- App-owned Settings IA extension records for plugins, skills, and MCP servers
  with provenance, scope, runtime/tool access, enabled state, audit detail, and
  inert runtime impact.
- Concurrent trusted-folder isolation and restart/resume for sessions,
  provider/model context, Browser, Files, Terminal, artifacts, and extension
  records.
- Deterministic desktop QA bootstrap through explicit `--workspace` and
  `--workspace-file` launch flags, portable non-secret workspace file
  open/save, recent workspace display, and provider/model persistence.
- App-owned local memory, broad action record, and audit record stores separate
  from workspace descriptors, provider stores, and runtime-local storage.

## Acceptance Evidence By Task

| Task | Status | Evidence |
| --- | --- | --- |
| TASK-001 | verified | r04 route parity, interaction coverage, screenshots, `npm test`, `node --check`, `git diff --check`; `npm run acceptance:mvp` unavailable because `scripts/mvp-acceptance.js` is absent. |
| TASK-002 | accepted | Mock server connector harness, rendered loading/success/failure integration tests, connector contract tests, and final `npm test` pass. |
| TASK-003 | accepted | Rust/Tauri backend scaffold, native menu implementation, backend scaffold tests, Rust tests, `npm run backend:build`, and `npm test` pass. |
| TASK-004 | accepted | Descriptor-backed workspace activation, first prompt session creation, frontend/backend tests, Rust tests, `npm test`, and `git diff --check`. |
| TASK-005 | accepted | OpenRouter stream normalizer and frontend streaming tests, Rust tests, `npm test` with 43 tests, and live review; automated live API smoke was intentionally skipped. |
| TASK-005A | verified | Scoped DOM/state tests for right-panel, work-log, streaming/progress, file-panel, composer, sidebar, shell layout, and `npm test` with 51 tests. |
| TASK-006 | verified | Provider/model Rust, server, frontend, full-suite, formatting, diff, and secret-material scan coverage; final approved visual polish. |
| TASK-007 | approved | Runtime/session persistence tests, frontend restore tests, connector tests, Rust tests, `npm test` with 75 tests, formatting, and diff checks. |
| TASK-008 | accepted | Trusted Files frontend/server tests, native menu tests, full frontend regression, Rust tests, `npm test` with 83 tests, formatting, and diff checks. |
| TASK-009 | accepted | Artifact preview Rust/frontend coverage, broad frontend/server regression, `npm test` with 93 tests, formatting, and diff checks. |
| TASK-010 | verified | Browser Rust/frontend coverage, frontend/server regressions, `npm test` with 97 tests, Rust full suite, formatting, and diff checks. |
| TASK-010A | verified | Editable address/local target frontend tests, Browser/File backend tests, full frontend regression, `npm test` with 102 tests, Rust full suite, formatting, and diff checks. |
| TASK-010B | verified | Native Browser frontend/server/Rust tests, user visual approval for geometry, `npm test` with 107 tests, Rust full suite, backend build, formatting, and diff checks. |
| TASK-010C | verified | Typed artifact preview frontend/Rust tests, Browser regression, `npm test` with 111 tests, backend build, formatting, and diff checks. |
| TASK-011 | verified | Terminal frontend/server/connectors/Rust tests, multiple user-review follow-ups, built-app manual QA for real folder `ls`, `npm test` with 121 tests, and diff checks. |
| TASK-011A | verified | Explicit prompt command bridge Rust/frontend regressions, `npm test` with 122 tests, formatting, and diff checks after user-reviewed UI/scroll polish. |
| TASK-011B | verified | New-session transition frontend/server/Rust tests, manual QA with deterministic store, provider-setup-required state, formatting, and diff checks. |
| TASK-012 | approved | Settings IA/extension frontend and server tests, Rust tests, connector tests, provider/model and Terminal regressions. |
| TASK-013 | verified | Concurrent trusted-folder Rust/server/frontend coverage, full backend and `npm test` with 146 tests, built-app manual QA for same-name folders, restart/resume, OpenRouter model context, and Terminal contract. |
| TASK-013A | accepted | Workspace bootstrap/open-save/provider persistence tests, built-app manual QA for menu save/open, multi-project workspace files, recents, relative root fixture, formatting, and diff checks. |
| TASK-014 | accepted | Record ownership red/green tests, Rust task coverage, TASK-013/TASK-013A and Browser/Terminal regressions, built-app QA with `tests/projects/workspace.c4os.json`, and stakeholder acceptance after Browser regression follow-up. |

## Surface Evidence Summary

- Shell and navigation: verified through TASK-001 parity coverage and preserved
  by every later slice.
- Workspace trust and descriptors: accepted in TASK-004, hardened for QA and
  portable workspace files in TASK-013A.
- Chat/runtime: accepted through TASK-005, TASK-007, TASK-011B, and TASK-013
  with real OpenRouter review, persistent sessions, and restart/resume.
- Provider/model setup: verified in TASK-006 and persisted in TASK-013A without
  raw key material in normal records.
- Files: accepted in TASK-008 with trusted-root open/save/revert behavior and
  preserved through TASK-013/TASK-014 regressions.
- Browser/artifacts: verified across TASK-009, TASK-010, TASK-010A, TASK-010B,
  TASK-010C, and TASK-014 Browser follow-up.
- Terminal: verified across TASK-011, TASK-011A, TASK-011B, and TASK-013 manual
  QA with top user PTY and bottom Agent command terminal preserved.
- Settings/extensions: approved in TASK-012 as app-owned inert records behind
  the accepted Settings IA.
- Memory/action/audit records: accepted in TASK-014 as backend-owned record
  stores and mirrors for current Browser, Terminal, and file-save actions.

## Remaining Mocks, Shortcuts, And Non-Release-Ready Areas

- `npm run acceptance:mvp` remains unavailable because
  `scripts/mvp-acceptance.js` is absent.
- TASK-016 security and approval policy hardening has not started.
- Approval UI and approval-policy semantics are not final; the existing
  permission prompt surface and action records are pre-hardening support.
- Provider credential storage is not release-hardened OS keychain behavior.
  TASK-006/TASK-013A protect raw keys from normal records, but final key policy
  remains TASK-016.
- Browser authority and profile hardening remain incomplete beyond the accepted
  feature boundaries for public URLs, trusted local files, artifact previews,
  and native Wry hosting.
- Terminal command policy and deterministic allowlist enforcement remain
  pre-hardening; explicit command prompts are a transitional bridge, not the
  durable runtime tool gateway.
- Extension records are app-owned and visible but runtime impact, invocation,
  plugin/skill loading, MCP execution, and prompt tags remain inert.
- Runtime tool gateway work is documented as needed before or during security
  hardening and is not implemented as a general tool-call system yet.
- Local memory save is backend-command-only; no new Memory UI route, panel, or
  settings category was accepted or added.
- File create, folder create, rename, and delete remain deferred because the
  accepted Files surface has no visible mutation controls for them.
- Downloads remain out of Browser scope.
- Release packaging, accessibility closeout, migration handling, final
  failure-state sweep, and full MVP acceptance verification remain TASK-017.

## Boundary Before TASK-016

- Stop here for stakeholder acceptance.
- Do not start `TASK-016` until this feature-complete acceptance package is
  accepted.
- `TASK-016` may harden the feature-complete surface, but it must not use
  security work as a reason to redesign the accepted shell, add new routes,
  introduce new panels, widen Browser/Terminal/Files authority, or silently
  enable extension runtime impact.
- Security and approval hardening must target the existing accepted surfaces:
  trusted roots, descriptors, provider keys, HTML/artifact preview isolation,
  Browser authority, terminal command policy, extension enablement gates, action
  records, and audit logs.

## Recommended Next Step

Stakeholder accepted this TASK-015 package on 2026-06-30.

Before `TASK-016`, proceed to
`.agents/development/progress/items/WO-006-runtime-tool-gateway-refactor.md`.
The runtime tool gateway refactor should formalize tool execution behind the
C4OS-owned runtime adapter before broad security and approval hardening starts.
