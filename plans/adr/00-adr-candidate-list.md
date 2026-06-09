# ADR Candidate List

This document identifies significant architectural, product, security, operational, and technical decisions supported by the existing planning documents. It does not add new product decisions beyond those sources.

## Status Legend

 - Final: the documents present the decision as a settled constraint or phase rule.
 - Provisional: the documents recommend the decision, but later review identifies validation or risk gates.
 - Unresolved: the documents identify the topic as a decision gate, inconsistency, or open question.

## Inconsistencies To Track

 - The product is described as a general-purpose AI workspace, but the MVP requirements and architecture are heavily coding-oriented through OpenCode, Git, worktrees, shell, patches, AGENTS.md, and diffs.
 - The architecture says the Rust backend owns policy enforcement, while OpenCode is expected to perform permission-aware tool use. The review calls this a split-brain risk.
 - The specifications prefer Tauri, but also require browser/artifact capabilities that may favor Electron or a controlled Chromium surface.
 - The documents prefer Codex-compatible plugin and marketplace conventions, but the review warns that those fields may be implementation details rather than a stable public standard.
 - The product is local-first and team collaboration is out of MVP, but marketplace, team plugin, RBAC, and admin-policy assumptions appear in several specs.
 - The specs include remote MCP support, while the review asks whether remote MCP should be disabled in MVP.
 - Plugin scripts are supported, while the review asks whether plugin scripts should be disabled in MVP.
 - The audit ledger is specified for traceability, but its intended assurance level is unresolved.

## ADR Candidates

### ADR-001: Product Scope And MVP Audience

Status: Unresolved.

Context: The requirements define a general-purpose desktop AI workspace for coding, writing, research, analysis, operations, and documentation. The review challenges that most concrete MVP requirements are coding-agent primitives.

Alternatives considered:
 - General-purpose workspace from day one.
 - Coding workspace first, generalized through plugins later.
 - Operations/research/document workspace first.

Alternatives that should be considered:
 - Persona-specific MVP for technical power users.
 - Dual-mode product with "folder assistant" and "developer workspace" onboarding paths.
 - Plugin-first general workflows with only coding built into core.

Tradeoffs:
 - General-purpose scope supports broader market positioning but risks vague requirements.
 - Coding-first scope aligns with OpenCode, Git, worktrees, AGENTS.md, and shell tooling but contradicts broad product language.
 - Persona-specific scope improves UX and metrics but narrows initial market.

Consequences:
 - Until this is decided, UX, onboarding, security defaults, artifact priorities, and plugin strategy remain unstable.

Follow-up questions:
 - Who is the first user persona?
 - What are the first five workflows the MVP must excel at?
 - Which non-coding workflows are first-class in MVP?
 - Are worktrees meaningful outside code projects?

ADR recommended: Yes, high priority.

### ADR-002: Desktop Application Shell

Status: Provisional.

Context: The specs recommend Tauri over Electron because of smaller footprint, lower memory use, Rust backend compatibility, native integration, and local-first fit. The review notes Tauri creates WebView and migration risks.

Alternatives considered:
 - Tauri.
 - Electron.
 - Native Swift/Kotlin/WinUI.

Alternatives that should be considered:
 - Electron for artifact/browser-heavy MVP.
 - Web app plus local daemon.
 - Tauri core with external controlled Chromium for browser automation.

Tradeoffs:
 - Tauri fits security-sensitive local tooling but depends on system WebViews.
 - Electron gives consistent Chromium behavior but increases footprint and IPC security burden.
 - Native stacks offer strongest OS integration but high cross-platform cost.

Consequences:
 - The choice affects browser rendering, extension support, process control, packaging, memory footprint, and future migration cost.

Follow-up questions:
 - Which platforms are required at launch?
 - Do artifact/browser requirements require Chromium consistency?
 - Can OpenCode be controlled cleanly from a Rust backend?

ADR recommended: Yes, high priority.

### ADR-003: Agent Runtime Strategy

Status: Provisional.

Context: The requirements and architecture recommend `AI GUI -> OpenCode Runtime -> OpenRouter`, avoiding a custom runtime if OpenCode satisfies core requirements. The review says this is the highest technical risk.

Alternatives considered:
 - OpenCode Runtime.
 - Custom runtime.
 - Direct OpenRouter calls.
 - Codex CLI/runtime dependency.
 - Multiple runtimes immediately.

Alternatives that should be considered:
 - Runtime abstraction layer with OpenCode as the first adapter.
 - OpenCode fork if policy hooks are insufficient.
 - Local daemon API that can wrap multiple runtimes later.

Tradeoffs:
 - OpenCode accelerates MVP but may not expose stable GUI-grade APIs.
 - Custom runtime maximizes control but duplicates complex agent infrastructure.
 - Runtime abstraction reduces lock-in but adds upfront design cost.

Consequences:
 - If OpenCode cannot provide pre-tool interception, event streaming, and session control, core security and UX assumptions fail.

Follow-up questions:
 - What exact OpenCode interface is available?
 - Can tool calls be intercepted before execution?
 - Can sessions be resumed and child sessions modeled reliably?

ADR recommended: Yes, high priority.

### ADR-004: Policy Enforcement Authority

Status: Unresolved.

Context: The architecture says the Rust backend owns policy enforcement while OpenCode performs permission-aware tool use. The review identifies this as ambiguous and risky.

Alternatives considered:
 - Backend-owned policy.
 - Runtime-owned policy.
 - Layered policy across backend and runtime.

Alternatives that should be considered:
 - Backend as mandatory policy enforcement point for all tool families.
 - Runtime-native policy with backend as UI/audit layer only.
 - Capability proxy where all tool calls route through a single policy gateway.

Tradeoffs:
 - Backend authority improves product-level consistency but may require deep runtime hooks.
 - Runtime authority uses existing permissions but weakens app-owned security guarantees.
 - Dual enforcement can provide defense in depth but risks conflicts and bypasses.

Consequences:
 - This decision affects every security, audit, plugin, MCP, shell, and filesystem feature.

Follow-up questions:
 - Which layer is authoritative on conflict?
 - Can OpenCode execute any tool outside backend approval?
 - How are tool calls normalized across shell, filesystem, MCP, browser, and plugin scripts?

ADR recommended: Yes, high priority.

### ADR-005: Standards-First Interoperability

Status: Provisional.

Context: The research recommends composing existing standards rather than inventing proprietary formats. The review warns that "support" needs conformance levels.

Alternatives considered:
 - Standards-first.
 - Proprietary app-native formats.
 - Import/export compatibility only.

Alternatives that should be considered:
 - Tiered conformance per standard: display, parse, execute, round-trip, export.
 - App-native canonical model with adapters for standards.

Tradeoffs:
 - Standards-first improves portability and ecosystem leverage.
 - Proprietary formats improve control but create lock-in.
 - Canonical app model plus adapters protects future migrations but requires explicit mappings.

Consequences:
 - Without conformance levels, engineering may overpromise compatibility.

Follow-up questions:
 - What is the conformance level for AGENTS.md, SKILL.md, MCP, Codex plugins, OpenCode config, and OpenRouter?
 - Which formats are canonical versus imported?

ADR recommended: Yes, high priority.

### ADR-006: Project, Session, And Worktree Model

Status: Provisional.

Context: The research and functional specs recommend registered local folders as projects, durable sessions per project, and worktrees for isolated modifying tasks.

Alternatives considered:
 - One global chat history.
 - One worktree per project.
 - Project/session/worktree separation.

Alternatives that should be considered:
 - Non-Git isolation model for document folders.
 - Session-scoped snapshot/backup instead of Git worktree.
 - Runtime-managed ephemeral workspace.

Tradeoffs:
 - Worktrees are strong for code but less meaningful for non-code projects.
 - Registered folders are simple but path portability is limited.
 - Durable sessions improve continuity but complicate storage, privacy, and migration.

Consequences:
 - This model may bias the product toward software engineering despite general-purpose positioning.

Follow-up questions:
 - What happens for non-Git folders?
 - What happens when two agents edit the same non-Git folder?
 - Which paths are machine-local versus portable?

ADR recommended: Yes, medium-high priority.

### ADR-007: Local-First Storage Model

Status: Provisional.

Context: The data model recommends SQLite for metadata, local artifact files for previews and originals, and OS keychain/encrypted vault for secrets.

Alternatives considered:
 - SQLite plus artifact directory.
 - JSON files only.
 - Postgres.

Alternatives that should be considered:
 - Event log plus SQLite projections.
 - Content-addressable artifact store with retention policies.
 - Sync-ready model if team collaboration is expected soon.

Tradeoffs:
 - SQLite is local, durable, and queryable.
 - JSON is portable but weak for concurrency.
 - Postgres is heavier than needed for desktop MVP.

Consequences:
 - Team sync, audit integrity, and path portability may be harder later if not accounted for now.

Follow-up questions:
 - Is the audit ledger for convenience or compliance?
 - What data is portable versus machine-local?
 - What retention, quota, backup, and export rules apply?

ADR recommended: Yes, high priority.

### ADR-008: Unified Tool Invocation And Ledger

Status: Provisional.

Context: The research recommends modeling every runtime capability as a tool invocation with a normalized policy envelope and append-only ledger.

Alternatives considered:
 - Unified tool envelope.
 - Special-case shell/filesystem calls.
 - Separate MCP and local tool models.

Alternatives that should be considered:
 - Tool proxy/gateway that enforces policy.
 - Event-sourced tool ledger.
 - Data-flow-aware ledger that tracks information movement.

Tradeoffs:
 - Unified tools enable audit and approval UX.
 - Normalization can hide important tool-specific semantics.
 - Per-tool logs do not automatically solve unsafe multi-tool compositions.

Consequences:
 - Tool schema design becomes a core platform contract.

Follow-up questions:
 - Does the policy engine reason about data flow or only invocations?
 - How are tool results redacted?
 - Can all runtime tools be forced through the ledger?

ADR recommended: Yes, high priority.

### ADR-009: Permission And Approval Model

Status: Provisional with unresolved enforcement details.

Context: The research and security specs recommend layered permissions and approval scopes such as once, session, project, always allow, deny once, and always deny.

Alternatives considered:
 - Runtime-only prompts.
 - Static config only.
 - Layered policy plus per-call approval.

Alternatives that should be considered:
 - Default-deny high-risk operations.
 - Separate policies for read, write, execute, network, credential, and publish actions.
 - Information-flow policy for tool chains.

Tradeoffs:
 - Layered policy supports safe defaults and productivity.
 - More layers increase user confusion and debugging complexity.
 - Broad approval scopes can become dangerous in team-managed environments.

Consequences:
 - Approval design affects trust, speed, auditability, and support burden.

Follow-up questions:
 - What are default-deny rules?
 - Which approval scopes are allowed in MVP?
 - How are destructive commands classified?

ADR recommended: Yes, high priority.

### ADR-010: Filesystem And Shell Access Defaults

Status: Provisional, with sandbox model unresolved.

Context: The research and security specs recommend project-root access by default, explicit external-directory approval, shell execution defaulting to ask, and secret-file deny patterns.

Alternatives considered:
 - Full home-directory access.
 - File picker only.
 - Root-scoped access with approvals.

Alternatives that should be considered:
 - Separate OS user or sandbox account for agent commands.
 - Containerized shell execution.
 - Network-disabled command mode.

Tradeoffs:
 - Root-scoped access is usable and understandable.
 - Strong sandboxing reduces risk but increases platform complexity.
 - Running as the user preserves environment compatibility but increases damage potential.

Consequences:
 - This decision defines the minimum viable security posture for local execution.

Follow-up questions:
 - Do agents run as the user's OS account?
 - What network access is allowed during shell execution?
 - What secret patterns are blocked by default?

ADR recommended: Yes, high priority.

### ADR-011: MCP Integration Strategy

Status: Provisional, remote MCP unresolved.

Context: MCP is recommended as the default integration protocol for external tools, local services, and organization connectors. Functional specs include stdio and remote MCP servers.

Alternatives considered:
 - MCP default.
 - Custom plugin API only.
 - Direct SDK integrations only.

Alternatives that should be considered:
 - Local stdio MCP only for MVP.
 - Remote MCP disabled by default.
 - MCP proxy/gateway for policy and logging.

Tradeoffs:
 - MCP maximizes interoperability.
 - Remote MCP increases data exfiltration and auth risk.
 - Direct SDKs can provide better UX for key integrations but scale poorly.

Consequences:
 - MCP policy and root scoping must be reliable before broad connector support.

Follow-up questions:
 - Which MCP spec version is the baseline?
 - Are remote MCP servers allowed in MVP?
 - How are sampling and resources approved?

ADR recommended: Yes, high priority.

### ADR-012: Agent Skills Support

Status: Provisional.

Context: The research recommends Agent Skills as portable folders with `SKILL.md`, optional scripts, references, and assets.

Alternatives considered:
 - Prompt library only.
 - Plugin-only workflows.
 - Agent Skills folders.

Alternatives that should be considered:
 - Explicit-only skill invocation for MVP.
 - Auto-invocation with user-visible explanation.
 - Skill trust tiers.

Tradeoffs:
 - Skills are portable and lightweight.
 - Skill scripts and instructions introduce supply-chain and prompt-injection risks.
 - Auto-invocation improves usability but may surprise users.

Consequences:
 - Skill discovery and version conflict rules must be defined.

Follow-up questions:
 - Are skills auto-invoked, explicit, or both?
 - How are conflicts resolved across global, project, and plugin scopes?

ADR recommended: Yes, medium-high priority.

### ADR-013: AGENTS.md Instruction Support

Status: Provisional.

Context: The research recommends root and nested `AGENTS.md` as first-class project instructions, with closest file precedence.

Alternatives considered:
 - AGENTS.md.
 - Proprietary instruction file.
 - Tool-specific files such as `CLAUDE.md` or `.cursor/rules`.

Alternatives that should be considered:
 - Import compatibility for other instruction files.
 - UI display of effective instruction stack.
 - Instruction conflict diagnostics.

Tradeoffs:
 - AGENTS.md is portable and simple.
 - Plain Markdown has no machine-checkable policy semantics.
 - Nested precedence can be confusing without UI visibility.

Consequences:
 - The app needs to separate instructions from enforceable policy.

Follow-up questions:
 - Are AGENTS.md instructions ever allowed to modify policy?
 - How are conflicts shown to users?

ADR recommended: Yes, medium priority.

### ADR-014: Plugin System Compatibility

Status: Provisional.

Context: The plugin specs recommend Codex-compatible `.codex-plugin/plugin.json`, optional `skills/`, `hooks/`, `scripts/`, `assets/`, `.mcp.json`, `.app.json`, and namespaced app extensions.

Alternatives considered:
 - Codex-compatible plugin shape.
 - MCP manifests only.
 - Custom plugin manifest.

Alternatives that should be considered:
 - Minimal plugin MVP with skills and MCP only.
 - App-native plugin manifest with Codex import/export.
 - Disable plugin hooks until runtime semantics are clear.

Tradeoffs:
 - Codex compatibility improves ecosystem leverage.
 - Codex fields may drift or be unstable.
 - Supporting hooks/scripts increases risk.

Consequences:
 - Plugin format becomes a migration and security boundary.

Follow-up questions:
 - Which Codex fields are stable enough to depend on?
 - Should plugin hooks and scripts be disabled in MVP?

ADR recommended: Yes, high priority.

### ADR-015: Marketplace Model

Status: Provisional.

Context: The marketplace spec recommends Codex-style `marketplace.json`, local/team manifests for MVP, install separate from enablement, and public publishing out of scope.

Alternatives considered:
 - Manifest-based marketplace.
 - Central hosted marketplace only.
 - Git repository only.

Alternatives that should be considered:
 - No marketplace in MVP, local plugin install only.
 - Signed marketplace manifest requirement.
 - Trust-tiered marketplace sources.

Tradeoffs:
 - Manifest marketplaces support local/team catalogs.
 - Marketplace UX implies trust, curation, and support obligations.
 - Public marketplace can be deferred, but data model should not block it.

Consequences:
 - Marketplace trust and source integrity must be addressed before broad distribution.

Follow-up questions:
 - Who is liable for plugin-caused data loss?
 - Are unsigned remote sources allowed?
 - How are updates verified?

ADR recommended: Yes, high priority.

### ADR-016: Plugin Trust And Execution

Status: Provisional, with unresolved MVP restrictions.

Context: The plugin and security specs say plugins are untrusted, indexing must not execute code, scripts run only as approved tools, and install/enable are separate.

Alternatives considered:
 - Trust all local plugins.
 - Ban scripts in plugins.
 - Allow scripts with explicit permission.

Alternatives that should be considered:
 - Disable plugin scripts in MVP.
 - Static analysis/linting for plugin packages.
 - Quarantine mode for new plugins.

Tradeoffs:
 - Allowing scripts enables powerful workflows.
 - Script support increases supply-chain and endpoint risk.
 - Disabling scripts reduces MVP scope and risk.

Consequences:
 - Plugin trust posture affects marketplace viability and user safety.

Follow-up questions:
 - Should plugin scripts be disabled in MVP?
 - What warnings or trust scores are shown before enablement?

ADR recommended: Yes, high priority.

### ADR-017: Artifact Model

Status: Provisional.

Context: The specs recommend typed artifact records with original files, previews, provenance, and links to session/tool calls.

Alternatives considered:
 - Typed artifact records.
 - Plain file attachments.
 - Database-only blobs.

Alternatives that should be considered:
 - Content-addressable storage with quotas.
 - Artifact type plugins.
 - User-controlled retention policies.

Tradeoffs:
 - Typed artifacts provide rich UX and provenance.
 - Storage can grow quickly.
 - Renderers introduce security and maintenance risks.

Consequences:
 - Artifact lifecycle and preview security must be specified.

Follow-up questions:
 - What artifact types are required for MVP?
 - What are retention and cleanup rules?
 - Can generated HTML execute scripts?

ADR recommended: Yes, medium-high priority.

### ADR-018: Browser/Web Viewing

Status: Provisional.

Context: The UX and functional specs include browser/web content viewing, screenshots, DOM/text extraction, and isolation from privileged backend IPC.

Alternatives considered:
 - Tauri WebView browser panel.
 - External browser fallback.
 - Electron/Chromium browser surface.

Alternatives that should be considered:
 - Read-only page capture only for MVP.
 - Explicit user-selected context extraction.
 - Separate browser automation service.

Tradeoffs:
 - In-app browser improves workflow cohesion.
 - Browser content is a prompt-injection source.
 - Tauri WebView may be inconsistent across platforms.

Consequences:
 - Browser security may influence the Tauri versus Electron decision.

Follow-up questions:
 - Can browser content enter model context automatically?
 - Are screenshots considered safer than DOM extraction?

ADR recommended: Yes, medium-high priority.

### ADR-019: Model Provider Strategy

Status: Provisional.

Context: Functional specs use OpenRouter as the primary provider gateway with BYOK and model switching.

Alternatives considered:
 - OpenRouter primary.
 - Direct provider calls.
 - OpenCode provider abstraction.

Alternatives that should be considered:
 - Direct provider fallback.
 - Local model provider support.
 - Provider-neutral capability registry.

Tradeoffs:
 - OpenRouter simplifies multi-provider support and BYOK.
 - It introduces external routing, pricing, and model-ID dependency.
 - Direct provider support increases complexity but reduces lock-in.

Consequences:
 - Provider data, cost, and privacy disclosures must be clear.

Follow-up questions:
 - Should the app support direct providers if OpenRouter is unavailable?
 - Where are BYOK credentials stored and managed?
 - How are costs and fallback routes surfaced?

ADR recommended: Yes, high priority.

### ADR-020: Session And Memory Management

Status: Provisional.

Context: Research recommends durable sessions, summaries, compaction, handoff, and explicit durable memory only when configured.

Alternatives considered:
 - Explicit scoped memory.
 - Fully automatic long-term memory.
 - No memory.

Alternatives that should be considered:
 - Project-only memory.
 - Session-only summaries with no cross-session memory.
 - User-reviewable memory writes.

Tradeoffs:
 - Memory improves continuity.
 - Hidden memory creates privacy and correctness risk.
 - No memory weakens long-horizon workflows.

Consequences:
 - Users need inspect/delete controls for memory, summaries, artifacts, and logs.

Follow-up questions:
 - What memory is durable by default?
 - Can users inspect and delete all memory?

ADR recommended: Yes, medium priority.

### ADR-021: UX Layout And Interaction Model

Status: Provisional.

Context: The UX spec defines a dense desktop command center with side rail, project sidebar, main session pane, right inspector, and bottom drawer.

Alternatives considered:
 - Command-center layout.
 - Chat-only interface.
 - IDE-like workbench.

Alternatives that should be considered:
 - Persona-specific simplified mode.
 - First-run task templates.
 - Separate non-code workspace mode.

Tradeoffs:
 - Dense UI supports power users.
 - It may overwhelm non-experts.
 - Chat-only is simpler but hides tools and artifacts.

Consequences:
 - Onboarding and terminology become core product requirements.

Follow-up questions:
 - What does first-run look like for non-expert users?
 - How are skills, plugins, MCP, agents, and instructions explained?

ADR recommended: Yes, medium priority.

### ADR-022: MVP Roadmap Sequencing

Status: Provisional.

Context: The roadmap recommends Phase 0 runtime validation before UI buildout, followed by shell/persistence, runtime integration, tools/approvals, Git/worktrees, MCP/skills/instructions, plugins/marketplace, artifacts/browser, and hardening.

Alternatives considered:
 - Runtime validation first.
 - UI first.
 - Custom runtime first.

Alternatives that should be considered:
 - Security validation before plugin/marketplace work.
 - Product persona validation before broad runtime work.
 - Prototype multiple runtime options before committing.

Tradeoffs:
 - Runtime validation reduces technical risk.
 - Product validation may reveal that a different runtime scope is needed.
 - Security validation may narrow MVP features.

Consequences:
 - Incorrect sequencing could produce a polished UI on top of unviable runtime assumptions.

Follow-up questions:
 - What are Phase 0 pass/fail criteria?
 - What stops the project before implementation?

ADR recommended: Yes, medium-high priority.

### ADR-023: Telemetry And Privacy

Status: Unresolved.

Context: The review identifies telemetry/privacy policy as a missing requirement. The requirements emphasize local-first workflows.

Alternatives considered:
 - None documented.

Alternatives that should be considered:
 - No telemetry by default.
 - Local-only diagnostics export.
 - Opt-in anonymous product telemetry.
 - Enterprise-controlled telemetry policy.

Tradeoffs:
 - Telemetry helps product quality.
 - Local-first users may reject hidden data collection.
 - Diagnostics are important for support.

Consequences:
 - Provider calls already send data off-device, so privacy UX must explain both product telemetry and model traffic.

Follow-up questions:
 - What data leaves the machine?
 - Is telemetry allowed in MVP?

ADR recommended: Yes, medium-high priority.

