# MVP Scope

This document defines the smallest useful product that can be shipped and validated with constrained engineering resources.

## Product Thesis

Technical users will prefer a desktop AI workspace over a terminal-only agent flow if it gives them a clearer control center for one local coding project: persistent sessions, visible tool activity, explicit approvals, and reviewable file changes.

The MVP should validate that thesis only. It should not attempt to validate a general-purpose AI workspace, a plugin ecosystem, a marketplace, multi-agent orchestration, remote MCP, or enterprise governance.

## MVP Audience

Primary audience:

 - Individual developers and technical power users.
 - Users working in local Git repositories.
 - Users comfortable with API keys, local files, Git diffs, and shell commands.

Not the MVP audience:

 - Non-technical knowledge workers.
 - Enterprise administrators.
 - Plugin authors.
 - Teams coordinating shared AI work.
 - Regulated customers requiring compliance-grade audit.

## MVP Architecture Required

The MVP architecture should be the smallest architecture that can validate the thesis:

 - Desktop app shell.
 - Local backend boundary for filesystem, process supervision, secrets, approvals, and persistence.
 - One OpenCode runtime integration path.
 - OpenRouter-only model provider setup.
 - Local SQLite database for projects, sessions, messages, approvals, and basic tool records.
 - OS keychain or encrypted local vault for provider credentials.
 - Project-root scoped file access.
 - Approval gateway for file writes, shell commands, and runtime-proposed Git state changes.
 - Basic Git status and diff integration.

Architecture explicitly not required for MVP:

 - Multiple runtime adapters.
 - Direct provider integrations.
 - MCP server manager.
 - Plugin manager.
 - Marketplace.
 - Worktree manager.
 - Browser automation.
 - Team sync.
 - Enterprise policy system.
 - Compliance-grade audit log.

## Feature Classification

### MVP

Each MVP feature must directly validate trust, persistence, controllability, or local coding usefulness.

 - Desktop shell: required to test whether a desktop control center is valuable.
 - OpenRouter credential setup: required to run model-backed sessions through the preferred provider path.
 - Register local Git projects and choose one selected project at a time through a minimal selector/list: required to test project-scoped local work without making multi-project workflows part of MVP.
 - Show project root and Git status: required to establish the user's working boundary.
 - Start one active agent session: required to validate the core loop.
 - Persist and resume the latest session: required to test continuity.
 - Conversation transcript: required for interaction and session review.
 - OpenCode runtime adapter: required to test reuse of an existing runtime.
 - Project-root file read/write through approved runtime tools: required for useful coding tasks.
 - Shell command approval: required for tests/builds, but only with explicit approval.
 - Git diff viewer: required for trust and review.
 - Changed-file list: required to understand agent impact.
 - One-time allow, narrow shell session allow, and deny: required to validate approval UX without permanent risky grants.
 - Tool activity timeline: required to see what the agent did.
 - Basic tool ledger: required to persist tool name, status, approval, redacted/truncated output summary, and affected files.
 - Root `AGENTS.md` display: required to validate the simplest project-instruction convention.
 - Basic file browser: required to orient users inside the project.
 - Basic artifact capture for text, logs, diffs, and generated files: required to preserve outputs from the session.
 - Stop running session: required for user control.
 - Minimized-window execution while the app process remains running: required for practical desktop use.
 - Crash or force-quit recovery that marks the previous run interrupted/crashed and preserves last persisted records: required for honest restart behavior.
 - Basic failure states for runtime, provider, and tool failures: required for usable validation.
 - Submitted-prompt retention after provider/network failure with explicit retry only: required for append-only transcript integrity.
 - Retry as a new appended action/status, not failed-response replacement: required for append-only transcript integrity.

### V1

Useful after the MVP validates the core loop.

 - Multiple sessions per project.
 - Polished multi-project workflows, cross-project navigation, and project management.
 - Project search, grouping, archive, delete, favorites, metadata editing, and cross-project views.
 - Session rename, archive, pin, and delete.
 - Global search across transcripts, tool logs, artifacts, projects, or files.
 - Full-text transcript search, tool-log search, artifact search, and project-wide file content search UI.
 - System tray daemon, background agent service, OS notifications, scheduled runs, and wake/resume automation.
 - Crash recovery replay, automatic process reattachment, automatic continuation after crash, and unsent prompt resend.
 - Durable pending approvals, approve-after-restart, and stale approval prompt replay.
 - Approval record export, copy-all, support bundle, JSON download, or share workflow.
 - Dedicated approval-record copy button.
 - Approval decision editing, approval decision deletion, full prompt replay blobs, raw command output, and raw secret values in approval records.
 - Dedicated raw shell output copy button, shell output export, and raw stdout/stderr persistence.
 - Retained live terminal buffers after completion, navigation away, reload, app close, or session restore.
 - Raw shell output fallback when redaction, truncation, or safe summary generation fails.
 - Sending live raw terminal buffers, omitted raw shell output, redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction metadata into model context.
 - Redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction hints in shell summary metadata.
 - Prompt deletion, transcript rewrite, hidden retry loops, or silent prompt resend after provider/network failure.
 - Retry-in-place, response replacement, branch-from-failure, and hidden transcript mutation.
 - Per-call token counts, cost estimates, model-call accounting, spend history, budget meters, and budget enforcement.
 - OpenRouter credit balance, billing link management, top-up flows, invoice links, spend warnings, and account diagnostics.
 - Worktree creation and cleanup.
 - Nested `AGENTS.md` resolution.
 - Explicit skill discovery and invocation.
 - Skill creator workflows for explicit, reviewable instruction artifacts.
 - Editable system prompts, project prompt editors, instruction composers, prompt templates, and hidden app-authored instruction layers.
 - Local stdio MCP server support.
 - Richer artifact previews for images, JSON, CSV, HTML, and PDFs.
 - Export/import for sessions and artifacts.
 - Storage quotas and cleanup controls.
 - Keyboard command palette.
 - Accessibility acceptance pass.
 - Direct provider fallback if OpenRouter-only setup fails validation.

### V2

Important, but too broad or risky for initial validation.

 - Multiple concurrent agents.
 - Child sessions and delegated subagents.
 - Project default agent UI, custom persona controls, retry-with-different-agent, per-message persona, and agent migration.
 - Runtime abstraction across OpenCode, Codex, Claude Code, or custom runtimes.
 - Plugin installation.
 - Plugin permission review.
 - Remote MCP.
 - Browser panel, screenshots, DOM extraction, and web content ingestion.
 - Document, spreadsheet, and slide workflows.
 - Long-term memory beyond session persistence.
 - Tamper-evident audit log.
 - Data-flow-aware policy engine.
 - Advanced model routing and budget controls.
 - Detailed model-call accounting, spend history, and cost dashboards.
 - OpenRouter account and billing management.
 - Hot model swap, run restart for model change, in-flight model migration, idle-session model switching, retry-with-different-model, and per-message model overrides.
 - Full editor, merge UI, or multi-file review workflow beyond approval prompts, changed-file list, and diff viewer.

 - Giant scrollback approval prompts, unbounded generated diffs, and approve-by-summary-only for oversized file-write batches.
 - Checkbox-per-file partial approval, automatic subset execution, hidden batch rewriting, blanket approve-all-future-writes, project-wide write approval, and hidden partial execution for file-write batches.

 - User-configurable approval thresholds, per-project approval caps, and advanced safety settings.

### Future

Strategic, but not needed to validate the initial product.

 - Public marketplace.
 - Signed plugin registry.
 - Enterprise admin console.
 - Team collaboration.
 - Cloud sync.
 - Organization policy management.
 - Shared team memory.
 - Regulated compliance mode.
 - Mobile clients.
 - Workflow automation designer.

### Remove

These should not be promised or designed into MVP.

 - General-purpose workspace positioning for MVP.
 - Non-code user onboarding in MVP.
 - Marketplace in MVP.
 - Plugin scripts or hooks in MVP.
 - Remote MCP in MVP.
 - Global search in MVP.
 - Closed-app background execution in MVP.
 - Browser content automatically entering model context.
 - Always-allow approvals in MVP.
 - Project-wide permanent approvals in MVP.
 - Multi-agent concurrency in MVP.
 - Worktrees as a prerequisite for MVP.
 - Compliance-grade audit claims in MVP.

## Included Features

The MVP includes only:

 - Single-user local desktop app.
 - Multiple registered local Git projects, with one selected project active at a time.
 - One active agent session at a time.
 - OpenRouter credentials and model selection.
 - OpenCode-backed session execution.
 - Project-root file access.
 - Approval prompts for file writes, shell commands, and runtime-proposed Git state changes.
 - Bounded diff or summary in file-write approval prompts when available.
 - Explicit atomic batch approval for multiple file writes when every target and per-file preview state is visible and fixed readable caps are not exceeded.
 - Tool activity timeline and persisted basic tool ledger.
 - Local-ledger-visible answered approval decision records with structured metadata, decision, timestamp, resulting action status, and bounded redacted summary or diff reference.
 - Git status, changed files, and diffs.
 - Manual recovery support through reviewable logs, changed-file lists, diffs, and stop controls.
 - Root `AGENTS.md` display.
 - Basic text/log/diff/generated-file artifacts.
 - Session resume after app restart.
 - Narrow compatibility claims: OpenRouter-backed model access, local Git project support, root `AGENTS.md` display, and app-owned text-like artifact records.

## Excluded Features

The MVP excludes:

 - Plugins.
 - Marketplace.
 - MCP.
 - Browser/web panel.
 - Worktrees.
 - Commit, branch, pull request, merge, rebase, tag, and push workflows.
 - Automatic rollback, snapshots, restore points, and undo stacks.
 - Multiple concurrent sessions.
 - Multiple concurrent agents.
 - Non-code workflows.
 - Enterprise controls.
 - Cloud sync.
 - Rich document/spreadsheet previews.
 - Long-term memory.
 - Direct provider fallback.
 - Compliance audit.
 - Full AGENTS.md, Agent Skills, MCP, Codex plugin, OpenCode config, import/export, or round-trip compatibility claims.
 - Generic prompt editor or app-authored hidden instruction layer.

## Technical Risks

 - OpenCode may not expose pre-execution tool interception.
 - OpenCode may not expose structured events suitable for a desktop UI.
 - A backend approval gateway may be hard to enforce if runtime tools execute internally.
 - Shell commands may be unsafe without stronger sandboxing.
 - Project-root scoping can fail if symlinks and path canonicalization are weak.
 - OpenRouter-only setup may not satisfy users who require direct provider keys.
 - SQLite is likely enough for one active session but may hide future concurrency issues.
 - Tauri may be sufficient for MVP but may not support later browser-heavy features.

## Assumptions Being Tested

 - A coding-first wedge is the smallest credible path to validate the broader workspace concept.
 - Users value a desktop control center enough to switch from terminal-only agent usage.
 - Tool visibility and approvals increase trust.
 - Session resume is a meaningful differentiator.
 - OpenCode can be integrated without building a custom runtime.
 - OpenRouter-only provider setup is acceptable for early technical users.
 - One active session is enough to validate product value.
