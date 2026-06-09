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
 - Approval gateway for file writes, shell commands, and Git actions.
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
 - Register one local Git project: required to test project-scoped local work.
 - Show project root and Git status: required to establish the user's working boundary.
 - Start one active agent session: required to validate the core loop.
 - Persist and resume the latest session: required to test continuity.
 - Conversation transcript: required for interaction and session review.
 - OpenCode runtime adapter: required to test reuse of an existing runtime.
 - Project-root file read/write through approved runtime tools: required for useful coding tasks.
 - Shell command approval: required for tests/builds, but only with explicit approval.
 - Git diff viewer: required for trust and review.
 - Changed-file list: required to understand agent impact.
 - One-time allow, session allow, and deny: required to validate approval UX without permanent risky grants.
 - Tool activity timeline: required to see what the agent did.
 - Basic tool ledger: required to persist tool name, status, approval, output summary, and affected files.
 - Root `AGENTS.md` display: required to validate the simplest project-instruction convention.
 - Basic file browser: required to orient users inside the project.
 - Basic artifact capture for text, logs, diffs, and generated files: required to preserve outputs from the session.
 - Stop running session: required for user control.
 - Basic failure states for runtime, provider, and tool failures: required for usable validation.

### V1

Useful after the MVP validates the core loop.

 - Multiple sessions per project.
 - Multiple registered projects with smoother navigation.
 - Session rename, archive, pin, and delete.
 - Worktree creation and cleanup.
 - Nested `AGENTS.md` resolution.
 - Explicit skill discovery and invocation.
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
 - Browser content automatically entering model context.
 - Always-allow approvals in MVP.
 - Project-wide permanent approvals in MVP.
 - Multi-agent concurrency in MVP.
 - Worktrees as a prerequisite for MVP.
 - Compliance-grade audit claims in MVP.

## Included Features

The MVP includes only:

 - Single-user local desktop app.
 - One local Git project active at a time.
 - One active agent session at a time.
 - OpenRouter credentials and model selection.
 - OpenCode-backed session execution.
 - Project-root file access.
 - Approval prompts for file writes, shell commands, and Git actions.
 - Tool activity timeline and persisted basic tool ledger.
 - Git status, changed files, and diffs.
 - Root `AGENTS.md` display.
 - Basic text/log/diff/generated-file artifacts.
 - Session resume after app restart.

## Excluded Features

The MVP excludes:

 - Plugins.
 - Marketplace.
 - MCP.
 - Browser/web panel.
 - Worktrees.
 - Multiple concurrent sessions.
 - Multiple concurrent agents.
 - Non-code workflows.
 - Enterprise controls.
 - Cloud sync.
 - Rich document/spreadsheet previews.
 - Long-term memory.
 - Direct provider fallback.
 - Compliance audit.

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

