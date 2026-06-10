# Implementation Readiness Gaps

This historical document lists information that was missing before
implementation could begin. It consolidates unresolved items from the
specifications, ADRs, architecture review, and ADR architecture review report.

Status note, 2026-06-10: this file is superseded by
`plans/decisions/finalized-decisions.md`,
`plans/decisions/unresolved-decisions.md`, and the validation artifacts under
`plans/validation/`. Do not use this file as the current readiness source.

## Hard Blockers

These must be answered before implementation planning starts.

### Product Scope

Missing information:

 - Primary MVP audience.
 - Buyer persona, if different from user.
 - Whether MVP is coding-first, general-purpose from day one, or a workflow shell with plugins.
 - First five workflows the product must make excellent.
 - Explicitly excluded workflows.
 - Which non-code workflows are first-class in MVP.
 - Whether worktrees are acceptable as a core concept if the product remains general-purpose.

Why blocked: current documents describe a general-purpose AI workspace, but the concrete architecture is coding-heavy.

### Runtime Feasibility

Missing information:

 - Exact OpenCode integration surface: library API, JSON RPC server, CLI subprocess protocol, filesystem state, fork, or other.
 - Whether OpenCode can stream structured events without terminal scraping.
 - Whether every tool call can be intercepted before execution.
 - Whether OpenCode supports custom approval UI before tool execution.
 - How OpenCode persists sessions.
 - Whether the app or OpenCode owns canonical session metadata.
 - Whether multiple OpenCode runtime instances can safely run in parallel.
 - Whether child sessions and handoffs can be represented reliably.
 - Whether provider routing details can be surfaced through OpenCode.

Why blocked: if OpenCode cannot be controlled headlessly, the preferred architecture may fail.

### Policy Enforcement Authority

Missing information:

 - Single authoritative policy enforcement layer.
 - Conflict behavior if backend policy and runtime policy disagree.
 - Whether any runtime tool can execute outside backend approval.
 - Whether MCP tools can bypass app policy or ledger.
 - Whether plugin scripts route through plugin permissions or shell permissions.
 - Where approval decisions are stored.
 - Whether denied actions are visible to the model and how.

Why blocked: security, audit, approvals, plugins, MCP, shell, and filesystem access depend on this.

### Local Execution Security

Missing information:

 - Whether agent commands run as the user's OS account.
 - Sandbox model for shell commands and tool subprocesses.
 - Network egress defaults for shell, MCP, plugins, browser tools, and scripts.
 - Default secret-file denylist.
 - Symlink traversal policy.
 - Destructive command classification.
 - Environment variable inheritance policy.
 - Redaction rules for shell output, logs, tool results, transcripts, screenshots, and artifacts.
 - Rollback story after a bad local action.

Why blocked: local file and shell access are high-risk capabilities.

### MCP Scope

Missing information:

 - Minimum supported MCP spec version.
 - Whether MVP allows remote MCP servers or only local stdio servers.
 - MCP root scoping rules.
 - How MCP resources become model context, if at all.
 - How MCP prompts are displayed and approved.
 - How MCP sampling requests are represented in approval UI.
 - MCP server trust levels.
 - MCP authentication and credential storage rules.

Why blocked: MCP expands the execution and data-exposure surface.

### Plugin Trust And Compatibility

Missing information:

 - Codex plugin fields considered stable enough to support.
 - Plugin conformance level: parse, display, execute, round-trip, export.
 - Whether plugin hooks are supported in MVP.
 - Whether plugin scripts are supported in MVP.
 - Whether plugin dependency install steps are ever run automatically.
 - Plugin trust tiers.
 - Plugin linting, quarantine, warning, and scoring requirements.
 - Required checksums or signatures for remote plugin sources.
 - How plugin assets are previewed safely.

Why blocked: plugins combine instructions, scripts, MCP, assets, and marketplace trust.

### Standards Conformance

Missing information:

 - Compatibility matrix for AGENTS.md, SKILL.md, MCP, Codex plugins, OpenCode config, and OpenRouter.
 - For each standard, whether support means display, parse, execute, round-trip, import, or export.
 - Which data model is app-canonical versus adapter-owned.
 - Versioning strategy for external format changes.
 - Namespacing rules for app-specific extensions.

Why blocked: "standards-first" is too vague without conformance levels.

### Privacy And Data Movement

Missing information:

 - Complete list of data that can leave the machine.
 - Provider data-retention disclosure requirements.
 - Telemetry policy: none, opt-in, local diagnostics only, or enterprise-controlled.
 - Whether support bundles can include logs, transcripts, screenshots, artifacts, or tool outputs.
 - User controls to inspect and delete sessions, summaries, memory, artifacts, logs, and provider credentials.
 - Data-flow policy for tool chains that read local data and send output externally.

Why blocked: local-first claims are incomplete if model, MCP, plugin, telemetry, and provider traffic are undefined.

## Must Resolve Before Detailed Implementation Planning

These can follow hard-blocker decisions, but should be resolved before engineering plans are written.

### Desktop Runtime Choice

Missing information:

 - Launch platform matrix.
 - Whether Tauri WebView is sufficient for artifact and browser requirements.
 - Whether browser automation or artifact previews require Chromium consistency.
 - Whether OpenCode integration favors Rust/Tauri or Node/Electron.
 - Migration cost threshold for switching from Tauri to Electron later.

### Tool Ledger And Audit Integrity

Missing information:

 - Whether tool ledger is for convenience, debugging, compliance, or security.
 - Required integrity level: normal DB records, append-only, hash-chained, signed, or exportable audit log.
 - Tool-call schema boundaries.
 - Secret redaction strategy for tool inputs and outputs.
 - Whether ledger tracks data movement between tools.
 - Output truncation and large-log storage policy.

### Approval Model

Missing information:

 - Default approval mode by project trust level.
 - Allowed approval scopes in MVP.
 - Whether "always allow" exists in MVP.
 - How approvals apply to child agents.
 - How approval prompts describe read, write, execute, network, credential, publish, and delete actions.
 - How prompt fatigue will be managed without weakening security.

### Storage Lifecycle

Missing information:

 - Artifact retention policy.
 - Storage quotas.
 - Deduplication strategy.
 - Garbage collection rules.
 - Backup and restore behavior.
 - Export/import format.
 - How absolute local paths are represented in portable exports.
 - Crash consistency requirements.
 - Database migration policy.

### Model Provider Strategy

Missing information:

 - Whether OpenRouter is mandatory or optional.
 - Whether direct provider fallback is required.
 - Whether local model providers are in scope.
 - Where BYOK credentials are stored and managed.
 - How provider fallback routes are shown.
 - Model cost display and budget alert requirements.
 - How provider/model capability metadata is refreshed.

### Marketplace Scope

Missing information:

 - Whether marketplace support is in MVP or deferred.
 - Whether local plugin install alone is sufficient for MVP.
 - Whether team marketplaces make sense without team policy administration.
 - Update verification rules.
 - Vulnerability advisory and plugin removal process.
 - Liability/support stance for third-party plugins.

## Must Resolve Before MVP Scope Lock

These define MVP acceptance and prevent late surprises.

### Project And Workspace Model

Missing information:

 - Behavior for non-Git folders.
 - Isolation model for non-code projects.
 - What happens when multiple agents edit the same non-Git folder.
 - Handling of monorepos, nested repos, submodules, Git LFS, symlinks, generated folders, and vendor folders.
 - Behavior for network-mounted or cloud-synced folders.
 - Worktree cleanup policy.

### Artifact And Preview Model

Missing information:

 - Required MVP artifact types.
 - Preview renderers for documents, spreadsheets, PDFs, images, HTML, logs, JSON, and diffs.
 - Whether generated HTML can execute scripts.
 - Sandbox model for artifact previews.
 - Artifact provenance detail level.
 - Export behavior per artifact type.

### Browser/Web Viewing

Missing information:

 - Whether browser content can enter model context automatically.
 - Whether DOM extraction, text extraction, screenshots, and downloads have different risk levels.
 - Browser isolation model.
 - Whether remote pages, local generated pages, and localhost apps have different policies.
 - Whether external browser control is needed.

### Skills And Instructions

Missing information:

 - Whether skills are auto-invoked, explicit-only, or both.
 - Skill precedence across global, project, and plugin scopes.
 - Skill version conflict resolution.
 - Whether skill scripts are allowed separately from plugin scripts.
 - Where app-specific skill metadata lives.
 - Effective instruction stack UI for AGENTS.md and other instruction files.
 - Whether AGENTS.md can ever influence permissions.

### Session And Memory

Missing information:

 - What session data is durable by default.
 - What memory exists beyond session summaries.
 - Whether memory writes require user review.
 - How compaction summaries are generated, inspected, edited, or deleted.
 - How sessions replay or migrate across runtime/model changes.

### UX And Onboarding

Missing information:

 - First-run onboarding path.
 - How non-expert users understand projects, sessions, agents, skills, plugins, MCP, models, BYOK, and worktrees.
 - Whether there is a simplified mode.
 - Accessibility acceptance criteria.
 - Keyboard navigation requirements.
 - Error and recovery UX for failed tools, denied approvals, runtime crashes, and bad agent actions.

### Scaling And Resource Limits

Missing information:

 - Maximum supported concurrent sessions.
 - Maximum supported child agents.
 - Maximum supported MCP servers.
 - Maximum supported worktrees.
 - Resource limits for shell commands and tool subprocesses.
 - File watcher limits.
 - Model spend limits for concurrent work.
 - UI event backpressure behavior.

## External Validation Needed

Before implementation, these need evidence rather than paper decisions:

 - OpenCode structured event and pre-tool interception proof.
 - Multi-runtime concurrency proof.
 - Tauri WebView artifact/browser compatibility proof.
 - Shell sandbox feasibility on launch platforms.
 - MCP local and remote security behavior proof.
 - Plugin validation and non-execution indexing proof.
 - SQLite concurrency and crash-recovery proof for multiple active sessions.
 - Large-folder/monorepo performance proof.

## Current Stop Conditions

Implementation should not begin if any of these remain true:

 - Product scope remains general-purpose while MVP acceptance criteria remain coding-first.
 - OpenCode cannot provide structured event streaming and pre-execution tool interception.
 - No single policy enforcement authority is chosen.
 - Shell commands run as the user with broad network access and weak path controls.
 - Remote MCP is allowed before data-flow and egress controls exist.
 - Plugin scripts or hooks are enabled before sandboxing and source integrity exist.
 - Privacy and telemetry are deferred beyond provider and plugin decisions.
 - Standards compatibility is claimed without conformance levels.
