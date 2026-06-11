# MVP Freeze

Date: 2026-06-10

Status: frozen for implementation planning.

## Product Thesis

Technical users will prefer a desktop AI coding control center over a
terminal-only agent flow if it gives them clearer control over one local Git
project: persistent sessions, visible tool activity, explicit approvals, and
reviewable file changes.

## Frozen MVP Features

- Single-user local desktop app.
- Tauri shell with a macOS Apple Silicon launch-validation target.
- Multiple registered local Git projects with one selected active project.
- OpenRouter credential setup and one selected model per session.
- Hardened OpenCode adapter, not unconstrained direct OpenCode.
- One active agent session at a time.
- Conversation transcript with append-only message semantics.
- Structured runtime event ingestion.
- App-owned SQLite records for projects, sessions, messages, tool calls,
  approvals, artifacts, models, settings, and adapter references.
- OS keychain or platform credential storage for OpenRouter keys.
- Project-root scoped file access.
- Read-only MVP file browser.
- Root `AGENTS.md` passive app display.
- Runtime-native instruction-source inventory and disclosure before session
  start.
- Approval gateway for file writes, shell commands, and runtime-proposed Git
  state changes.
- One-time allow, narrow shell session allow, and deny.
- Structured denial results returned to the runtime.
- Stop running session.
- Runtime process and app-supervised child-process termination on stop.
- Tool activity timeline and basic durable tool ledger.
- Bounded redacted/truncated shell summaries and output-omitted marker when
  safe summary generation fails.
- Git branch, dirty state, changed-file list, and file-level diffs.
- Basic text/log/diff/generated-file artifact records.
- Provider, runtime, tool, and shell failure states.
- Resume latest project/session after app restart.
- Crash or force-quit recovery that marks interrupted runs and discards pending
  approvals.
- No product telemetry. Required OpenRouter model traffic is disclosed
  separately.

## Explicitly Excluded Features

- Plugins, marketplace, MCP, and plugin permissions.
- Browser/web panel, screenshots, DOM extraction, rich previews, and active
  HTML rendering.
- Worktrees.
- Commit, branch, pull request, merge, rebase, tag, and push workflows.
- Multiple concurrent sessions or agents.
- Custom agents, persona controls, subagents, handoffs, and skill workflows.
- Direct provider integrations, local models, fallback routing, hot model
  swap, per-message model override, and budget dashboards.
- Whole-repo model-context indexing and hidden background file ingestion.
- Editable system prompts, project prompt editors, instruction composers, and
  hidden app-authored instruction layers.
- Global search, transcript search, tool-log search, artifact search, and
  project-wide file content search UI.
- Session delete, export/import, cleanup, quotas, long-term memory, cloud sync,
  team collaboration, and enterprise governance.
- Durable pending approvals, approve-after-restart, always-allow approvals,
  project-wide approvals, approval export, approval copy-all, support bundles,
  and approval editing/deletion.
- Raw stdout/stderr persistence, raw shell output export, retained live
  terminal buffers after restore, and automatic rollback.

## Required Architecture

- Tauri frontend plus local backend boundary.
- Backend owns filesystem, shell, Git state-changing policy, process
  supervision, secrets, approvals, persistence, and runtime adapter control.
- Hardened OpenCode adapter owns launch, event stream ingestion, permission
  routing, stop mapping, `/config` redaction, and adapter references.
- App-owned Approval Gateway gates file writes, shell commands, and
  runtime-proposed Git state changes before execution.
- OpenRouter is the only provider path. The selected model is fixed at session
  creation.
- Running sessions use the credential reference captured at session start.
- App-owned SQLite is canonical. Runtime IDs and logs are adapter metadata.
- Model context is bounded and disclosed by source category.
- Shell execution runs as the current OS user with normal network access and a
  backend-filtered environment. The app must not claim stronger sandboxing.

## Required Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/acceptance/settings-and-configuration.md`
- `.agents/archive/planning-corpus/acceptance/telemetry-and-diagnostics.md`

Post-MVP acceptance documents for plugins, marketplace, and MCP must not enter
MVP implementation tasks.

## Success Metrics

- Users can configure provider credentials without support.
- Users can register a local Git project and understand the active project
  boundary.
- Runtime starts and streams assistant/tool activity for validation sessions.
- Protected actions wait for approval before execution.
- Denied or blocked actions do not execute and return structured denial
  results.
- Users can inspect changed files and diffs after agent work.
- Sessions, tool records, approvals, artifacts, and failure states survive
  restart.
- No project-root safety failure occurs during validation.
- Users report that the desktop app improves trust, control, or continuity over
  terminal-only agent usage.

## Assumptions Being Tested

- A coding-first desktop control center is a useful wedge.
- One active session is enough to validate value.
- Hardened OpenCode adapter integration can preserve the app's approval,
  persistence, disclosure, and stop requirements.
- OpenRouter-only setup is acceptable for early technical users.
- Tool activity, approvals, and diffs are enough to improve trust.
- Tauri supports the MVP UI and text-like artifact surface without browser-heavy
  features.

## Deferred Decisions

- Tauri UI validation waits for a built app.
- Windows 11 x64 compatibility validation is required before public MVP
  release.
- macOS Intel validation is optional when hardware is available.
- Linux validation is deferred.
- Low-severity documentation cleanup remains for `.agents/archive/planning-corpus/preplan-brief.md` and
  ADR numbering/index clarity.
- V1/V2 scope remains deferred until MVP validation data exists.

## Implementation Contract

- Anything not listed in Frozen MVP Features is out of scope for the first
  implementation pass.
- Phase 1 must implement the hardened OpenCode adapter path, not unconstrained
  direct OpenCode.
- Implementation tasks must carry the deferred code-level validation from
  `.agents/archive/planning-corpus/decisions/unresolved-decisions.md`.
- Release readiness must not be claimed until app-build validation, code-level
  adapter/shell/provider tests, and end-to-end product QA pass.
