# Finalized Decisions

This document summarizes decisions that are settled for MVP implementation
planning after the Phase 0 validation loop on 2026-06-10.

Current readiness: READY FOR FREEZE-AND-PLAN.

Implementation planning may begin. Release readiness remains gated by the built
app, code-level tests, and platform/product QA listed in
`.agents/archive/planning-corpus/decisions/unresolved-decisions.md`.

`.agents/archive/planning-corpus/reviews/finding-resolution.md` was requested as review input but is not
present in this workspace. This rollup uses the available final readiness
review, ADRs, MVP documents, spikes, and validation artifacts.

## Final MVP Decisions

### ADR-001: MVP Product Scope And Audience

Status: finalized for MVP.

Decision: MVP is a single-user, coding-first desktop control center for
technical users working in local Git repositories.

Final decisions:

- MVP validates agent-assisted local coding, not a general-purpose AI
  workspace.
- MVP audience is individual developers and technical power users.
- Non-code workflows, enterprise governance, team collaboration, plugins,
  marketplace, MCP, browser automation, worktrees, and direct providers are out
  of MVP.

### ADR-002: Desktop Application Shell

Status: finalized for MVP with macOS-first platform matrix.

Decision: Use Tauri for MVP. macOS Apple Silicon is the mandatory launch
validation target. Windows 11 x64 is required before public MVP release. macOS
Intel is optional when hardware is available. Linux is deferred.

Final decisions:

- Tauri is the selected MVP desktop shell.
- Browser-heavy and rich-preview features are post-MVP.
- Tauri UI validation remains a release or milestone gate because it requires a
  built app.

### ADR-003: Agent Runtime Strategy

Status: finalized for MVP through hardened adapter fallback.

Decision: Use a hardened OpenCode adapter with mandatory instruction/config
disclosure. Do not use unconstrained direct OpenCode.

Final decisions:

- Runtime events must be structured and adapter-observed.
- Runtime IDs, messages, parts, tool calls, permissions, logs, and persistence
  references are adapter metadata only.
- App-owned records are canonical.
- Runtime-native instructions may be used only after app preflight disclosure
  and app-owned session recording.

### ADR-004: Policy Enforcement Authority

Status: finalized for MVP with live runtime-cooperation evidence.

Decision: The Tauri/Rust backend is the authoritative Approval Gateway for MVP
file writes, shell commands, and runtime-proposed Git state changes.

Final decisions:

- Protected actions must not execute before backend approval.
- Denied or blocked actions must return structured denial results to the
  runtime.
- Approval records are app-owned ledger records.
- Runtime-native policy is defense in depth only.

### ADR-005 And ADR-006: Narrow Compatibility Claims

Status: finalized for MVP.

Decision: MVP may claim only OpenRouter-backed model access, local Git project
support, root `AGENTS.md` app display, and app-owned text-like artifact
records. MVP does not claim OpenCode config compatibility.

### ADR-007 And ADR-020: Local-First Storage, Retention, And Session Integrity

Status: finalized for MVP.

Decision: MVP uses app-owned local SQLite metadata, local artifact files, and
OS keychain or platform credential storage. MVP retains local records
indefinitely by default and uses append-only transcript semantics.

Final decisions:

- App-owned records are canonical for sessions, messages, tool calls,
  approvals, artifacts, projects, models, and settings.
- OpenCode-native records are adapter references only.
- Messages are append-only; runtime records may receive status updates.
- Stop, crash, and provider failure preserve records instead of rewriting
  history.

### ADR-008: Basic Tool Ledger

Status: finalized for MVP.

Decision: MVP includes a basic local tool ledger for user inspection and
debugging, not compliance-grade audit.

### ADR-009: MVP Approval Scope

Status: finalized for MVP.

Decision: MVP supports one-time allow, narrow shell session allow, and deny.

Final decisions:

- Session allow applies only to matching non-destructive shell commands inside
  the selected project root or approved project subpath.
- Session allow never covers destructive commands, outside-root paths,
  secret-deny files, or Git state changes.
- File writes and runtime-proposed Git state changes require explicit approval.

### ADR-010: Project-Root Boundary And Shell Baseline

Status: finalized for MVP with linked concrete policy.

Decision: Project-root file boundaries, symlink containment, secret-deny
behavior, current-user shell execution, normal network access, filtered shell
environment baseline, destructive one-time approval behavior, manual recovery,
and bounded shell output persistence are accepted for MVP.

Final decisions:

- Reads and writes outside the selected project root are blocked in MVP.
- Symlinks are resolved before boundary checks.
- Secret-deny files have no MVP approval override.
- Shell commands run as the current OS user after approval.
- Approved shell commands may use normal network access.
- Exact environment, redaction, truncation, destructive-command, and
  unclassifiable-command rules are defined in
  `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`.

### ADR-011, ADR-014, ADR-015, ADR-016: MCP, Plugins, And Marketplace

Status: finalized as excluded from MVP.

Decision: MCP, plugin execution, plugin installation, marketplace, and remote
plugin sources are out of MVP.

### ADR-013: Root AGENTS.md And Runtime Instruction Disclosure

Status: finalized for MVP with OpenCode-backed disclosure caveat.

Decision: Root `AGENTS.md` display is passive app-owned behavior. OpenCode-
backed sessions may still load runtime-native instruction sources, so the app
must inventory, disclose, and persist effective instruction sources before
session start.

Final decisions:

- The app-owned root `AGENTS.md` display is passive.
- Explicit runtime reads of root `AGENTS.md` follow normal project-root
  file-read rules.
- Approved edits to `AGENTS.md` do not automatically reload app-owned policy.
- Do not claim root or nested instructions are absent from OpenCode model
  context unless the adapter proves and records that for the session.
- Full instruction editing, precedence visualization, and compatibility
  workflows are post-MVP.

### ADR-017 And ADR-018: Artifact And Browser Boundaries

Status: finalized for MVP.

Decision: MVP artifacts are text-like records or file links only. Browser
automation and active browser/content surfaces are out of MVP.

### ADR-019: OpenRouter-Only Provider Path

Status: finalized for MVP with runtime evidence and context disclosure caveat.

Decision: MVP uses OpenRouter as the only provider gateway and one selected
OpenRouter model fixed for each session.

Final decisions:

- The OpenRouter API key is stored only in OS keychain or platform credential
  storage.
- Key update or revoke is blocked while a session is running.
- Running sessions keep their starting credential reference.
- The selected model is fixed at session creation.
- OpenRouter or network outages fail closed and require explicit retry.
- The adapter must redact sensitive `/config` values immediately.
- OpenRouter-bound context is described through bounded, disclosed
  context-source categories, including runtime-native instruction sources when
  present.

### ADR-021: One Default Coding Agent

Status: finalized for MVP.

Decision: MVP uses one default coding agent/runtime persona.

### ADR-022: MVP Roadmap Starts With Validation Gates

Status: finalized.

Decision: Phase 0 validation gates precede implementation epics, and those
architecture gates are separated from product QA acceptance.

Final decisions:

- Implementation planning must not begin until BLOCKER and HIGH Phase 0 gates
  are resolved or explicitly converted to accepted fallback/scope decisions.
- Phase 0 spikes are not implementation tasks.
- Product QA begins after implementation surfaces exist.

### ADR-023: Product Telemetry

Status: finalized for MVP.

Decision: MVP sends no product telemetry. OpenRouter model traffic is required
provider behavior and must be disclosed separately.

## Finding Resolution Matrix

### FINDING-001: OpenCode Runtime Control Is An Unproven MVP Gate

Resolved: yes for implementation planning.

How resolved: Structured events, pre-execution interception, stop behavior, and
app-owned record mapping passed. Strict config isolation failed for project
instructions, so the MVP path is the hardened OpenCode adapter with mandatory
instruction/config disclosure.

### FINDING-002: Backend Approval Gateway Depends On Runtime Cooperation

Resolved: yes for implementation planning.

How resolved: Live probes showed protected file writes, shell commands, and
runtime-proposed Git state changes did not execute before backend permission
reply. Denial produced a structured rejected permission reply and tool error.

### FINDING-003: Shell Safety Still Lacks Concrete Redaction And Classification Limits

Resolved: yes for implementation planning.

How resolved: `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md` defines
the exact environment, redaction, truncation, destructive-command, and
unclassifiable-command policies. Implementation must add code-level tests.

### FINDING-004: OpenRouter-Only Model Routing Needs Runtime-Level Verification

Resolved: yes with scope change.

How resolved: Runtime verification passed for OpenRouter provider path,
effective model match, and starting credential-reference stability. Strict
context exclusion failed as originally worded and is replaced by bounded,
disclosed context-source categories.

### FINDING-005: Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope

Resolved: yes with disclosure policy.

How resolved: OpenCode-native instruction loading is not disabled, but it is
observable and disclosable through app preflight inventory plus effective
`/config` inspection. OpenCode-backed sessions must disclose instruction
sources before session start.

### FINDING-006: Old Readiness Gap Rollup Conflicts With Later Finalized Decisions

Resolved: yes for the active rollups.

### FINDING-007: Plugin And Marketplace Specs Remain Broader Than MVP

Resolved: yes.

Resolution: Plugins, MCP, and marketplace are excluded from MVP.

### FINDING-008: Local Indefinite Retention Has No Cleanup Or Deletion Story

Resolved: yes by explicit MVP tradeoff acceptance.

### FINDING-009: Tauri Choice Is Reasonable But Still Needs Platform Validation

Resolved: partially.

How resolved: The platform matrix is finalized for planning. UI validation is
deferred until an app build exists and remains a release or milestone gate.

### FINDING-010: Append-Only Transcript Integrity Is Strong But UX Recovery Is Thin

Resolved: yes by explicit MVP tradeoff acceptance.

### FINDING-011: Acceptance Criteria Are Strong But Some Are Phase-0 Gates, Not Product QA

Resolved: yes.

How resolved: `.agents/archive/planning-corpus/validation/acceptance-gate-separation.md` separates Phase
0 architecture gates, accepted fallback/scope-change gates, deferred app-build
validation, and product QA acceptance.

### FINDING-012: Preplan Brief Still Says To Create Plans Even Though Corpus Exists

Resolved: no, low severity.

### FINDING-013: ADR Numbering And Rollup Naming Are Hard To Follow

Resolved: partially, low severity.

### FINDING-014: What Is The Accepted Runtime Path If Direct OpenCode Fails?

Resolved: yes.

How resolved: Select hardened OpenCode adapter with mandatory instruction/config
disclosure as the MVP fallback; keep shadow workspace, proxy, fork, runtime
replacement, and scope reduction as ordered fallback paths.

### FINDING-015: Which Target Platforms Must Pass Tauri Validation?

Resolved: yes for planning.

How resolved: Mandatory launch validation is macOS Apple Silicon; Windows 11
x64 is required before public MVP release; macOS Intel is optional; Linux is
deferred.

## Final Decisions That Are Safe To Use

- Coding-first, single-user, local Git repository MVP.
- Tauri desktop shell with macOS-first platform matrix.
- Hardened OpenCode adapter with mandatory instruction/config disclosure.
- App-owned backend Approval Gateway as mandatory policy authority.
- Project-root scoped file access.
- Current-user shell execution with approval, filtered environment, normal
  network access, no sandbox claim, and bounded persisted summaries.
- OpenRouter-only provider path with fixed per-session selected model.
- OS keychain or platform credential storage for OpenRouter key.
- Root `AGENTS.md` passive app display plus OpenCode-backed instruction-source
  disclosure.
- App-owned SQLite/local artifact records.
- Append-only session history and indefinite local retention by default.
- Basic tool ledger, changed-file list, and Git diff viewer.
- No plugins, marketplace, MCP, browser automation, rich artifacts, direct
  providers, worktrees, multi-agent execution, long-term memory, team sync, or
  enterprise governance in MVP.
