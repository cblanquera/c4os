# Architecture Review

This review challenges the generated specifications from the perspective of a principal engineer, architect, security reviewer, product manager, and skeptical CTO. It does not propose implementation work. It identifies decisions that need stronger evidence before architecture freeze.

## Executive Assessment

The specification set is directionally strong: it chooses existing standards, correctly avoids a greenfield agent runtime for MVP, recognizes local-first security requirements, and separates projects, sessions, tools, plugins, MCP, skills, and artifacts.

The weak point is that several recommendations assume compatibility and composability will be straightforward. They will not be. The hardest parts are not UI layout or data persistence; they are runtime control, approval enforcement, process isolation, plugin trust, model/provider portability, and long-term migration away from whichever runtime becomes the first dependency.

The architecture should not be considered ready for implementation until the team answers the runtime ownership, security boundary, and product-scope questions in this review.

## Highest-Risk Assumptions

### 1. OpenCode Runtime Can Serve As A Headless Product Runtime

The specs assume OpenCode can be wrapped cleanly by a Tauri GUI. That is not established. A CLI/TUI-oriented runtime may not expose stable APIs for:

 - event streaming
 - pre-tool approval interception
 - session resume
 - child sessions
 - worktree mapping
 - custom policy engines
 - plugin loading
 - artifact capture
 - provider routing visibility

If the GUI has to scrape terminal output, monkey-patch config files, or infer state from filesystem side effects, the product will inherit brittle behavior and poor security guarantees.

Hard question: What exact OpenCode interface is the product betting on: library API, JSON RPC server, CLI subprocess protocol, filesystem state, or a fork?

Decision gate: no architecture freeze until a runtime-control matrix proves each required event and control point is available without fragile scraping.

### 2. Policy Enforcement Ownership Is Ambiguous

The specs say the Rust backend owns policy enforcement while OpenCode performs permission-aware tool use. That creates a split-brain risk. If both layers can approve, deny, transform, or execute tools, then auditability and security depend on perfect synchronization.

Failure modes:

 - OpenCode executes a tool before the GUI sees the proposal.
 - GUI marks a command denied, but runtime has already spawned a process.
 - Runtime reads files through a path not visible to the GUI policy engine.
 - MCP tool calls bypass the app ledger.
 - A plugin script executes through shell permissions instead of plugin permissions.

Hard question: Is the app the mandatory policy enforcement point, or is OpenCode? If both, which layer is authoritative on conflict?

Decision gate: define one mandatory policy enforcement point for every tool family.

### 3. "General-Purpose AI Workspace" Is Under-Specified

The specs state that the product is not limited to software engineering, but most requirements still inherit coding-agent assumptions: Git, worktrees, shell, file patches, AGENTS.md, OpenCode, and diffs.

Non-coding workflows need different primitives:

 - research sources and citation traceability
 - document lifecycle and review workflows
 - spreadsheet provenance and cell-level formulas
 - long-running operations dashboards
 - knowledge-base ingestion
 - task/project management integrations
 - data-analysis notebooks or tabular pipelines
 - approval flows for external business actions

Hard question: Is the MVP actually a coding workspace that can be extended later, or a general workspace from day one?

Decision gate: define the primary first audience and name which non-coding workflows are first-class in MVP versus plugin-delivered later.

### 4. Standards Compatibility Is Treated Too Optimistically

The specs recommend AGENTS.md, Agent Skills, MCP, Codex-compatible plugins, OpenCode config, and OpenRouter. That is good directionally, but "support" can mean several incompatible things:

 - parse only
 - display only
 - execute partially
 - execute fully
 - round-trip without data loss
 - export to another tool
 - import from another tool

Hard question: What conformance level is required for each standard?

Decision gate: create a compatibility matrix for AGENTS.md, SKILL.md, MCP, Codex plugins, OpenCode config, and OpenRouter before committing UI or data model.

## Security Review

### Critical Security Concerns

The current security spec identifies the right threat categories, but it is not strict enough about enforcement mechanics.

Major gaps:

 - No explicit sandboxing model for shell commands.
 - No statement on whether agents run as the user's OS account.
 - No network egress policy beyond approval language.
 - No default-deny policy for remote MCP servers.
 - No plugin signing or trust-tier plan beyond "future".
 - No secret redaction model for logs, transcripts, tool outputs, screenshots, or artifacts.
 - No prompt-injection containment strategy for browser content, documents, emails, or MCP resources.
 - No clear separation between instructions as data and instructions as policy.
 - No model-provider data retention disclosure requirements.

Hard question: If a malicious webpage instructs an agent to read `.env` and exfiltrate it via an allowed MCP tool, which layer prevents the complete chain?

### Tool Composition Risk

Per-tool approval is necessary but insufficient. A sequence of individually allowed actions can be unsafe:

1. read confidential file
2. summarize it
3. send summary to remote issue tracker
4. write "done" to local log

The specs need a data-flow or information-flow view, not only per-tool gates.

Hard question: Does the policy engine understand data movement between tools, or only tool invocation?

### Plugin Supply Chain Risk

The plugin spec says indexing must not execute code. Good. But a malicious plugin can still:

 - write dangerous instructions in `SKILL.md`
 - define misleading MCP tool descriptions
 - request broad shell permissions
 - bundle obfuscated scripts
 - include assets that exploit preview renderers
 - use dependency install steps as code execution

Hard question: Will the product lint, score, quarantine, or warn on suspicious plugin contents before enablement?

### Browser Risk

The browser panel is described as isolated, but not enough. Browser pages are among the most likely prompt-injection sources. Generated local HTML can also be hostile if it includes script content from the model.

Hard question: Are browser pages allowed to become model context automatically, or must the user explicitly select page content? Are screenshots treated as less risky than DOM text?

### Audit Log Integrity

The tool ledger is valuable only if it is tamper-evident enough for trust and debugging. A local SQLite table is easy for local malware or scripts to modify.

Hard question: Is the audit ledger for user convenience, compliance, or security? If compliance/security, it needs stronger integrity guarantees.

## Product Requirements Gaps

### Missing User And Market Definition

The specs do not define:

 - target user persona
 - buyer persona
 - single-player versus team product
 - consumer versus enterprise posture
 - expected technical sophistication
 - first five workflows the app must excel at
 - workflows intentionally excluded

Without this, "general-purpose" becomes a scope trap.

### Missing Success Metrics

No product metrics are defined. Examples:

 - time to first successful task
 - percentage of tool calls requiring approval
 - approval regret rate
 - session recovery success rate
 - plugin install success rate
 - task completion rate
 - artifact reopen/export success
 - crash recovery rate

Hard question: How will the team know the architecture is making users more effective rather than merely exposing more controls?

### Missing Onboarding Requirements

The app assumes users understand projects, sessions, agents, skills, plugins, MCP, models, providers, BYOK, and worktrees. That is too much conceptual load.

Hard question: What is the first-run path for a non-expert user who only wants help with a folder of documents?

### Missing Collaboration Boundary

Team and marketplace concepts appear, but team collaboration is out of MVP. That tension needs resolution. Marketplace trust, team policy, shared skills, and admin controls are much harder to retrofit if local-only assumptions leak into the data model.

Hard question: Is this a local personal app that may later sync, or a team-governed app with local execution?

## Scaling Concerns

### Concurrent Agent Scaling

Multiple agents mean multiple runtime processes, MCP servers, shells, file watchers, browser instances, and model streams. The specs do not define limits, scheduling, backpressure, or resource accounting.

Risks:

 - CPU and memory exhaustion.
 - Too many file watchers on large monorepos.
 - MCP server process leaks.
 - SQLite write contention.
 - UI event stream overload.
 - Model spend surprises.

Hard question: What is the maximum supported number of concurrent sessions, child agents, MCP servers, and worktrees on a typical laptop?

### Large Repository And Large Folder Handling

The specs do not address:

 - monorepo indexing scale
 - symlink traversal
 - generated/vendor directory exclusion
 - binary file handling
 - network-mounted folders
 - permission changes while sessions are active
 - file rename/move provenance

Hard question: What happens when a user adds a 20 GB monorepo or a synced cloud-drive folder?

### Artifact Growth

Artifacts can become the largest local storage consumer. The specs mention artifact storage but not quotas, garbage collection, deduplication, retention, or backup behavior.

Hard question: Who owns cleanup when 50 sessions generate screenshots, PDFs, logs, HTML previews, and build artifacts?

## Alternative Architectures

### Alternative A: Tauri GUI With OpenCode Runtime

This is the current preferred path.

Strengths:

 - Fastest route if OpenCode exposes stable controls.
 - Aligns with local-first and Rust backend goals.
 - Avoids custom agent loop.

Weaknesses:

 - High dependency on OpenCode internals.
 - Potential mismatch between GUI product needs and CLI runtime assumptions.
 - Policy split-brain risk.

Best if: OpenCode has a stable embeddable protocol and tool-call interception.

### Alternative B: Electron GUI With OpenCode Runtime

Strengths:

 - Stronger Chromium consistency for browser and artifact rendering.
 - Easier integration with Node-based agent tooling and extension ecosystems.
 - Larger pool of desktop web engineers.

Weaknesses:

 - Larger footprint.
 - Larger attack surface if IPC and Node integration are mishandled.
 - Weaker alignment with Rust-first backend goals.

Best if: browser automation, extension compatibility, or Node runtime integration becomes critical.

### Alternative C: Tauri GUI With Runtime-Abstraction Layer And Pluggable Engines

Strengths:

 - Reduces lock-in to OpenCode.
 - Allows future Codex, Claude Code, custom runtime, or local runtime adapters.
 - Forces clean event/tool/session interfaces.

Weaknesses:

 - More upfront architecture work.
 - Lowest-common-denominator risk.
 - May delay MVP.

Best if: long-term product strategy depends on runtime independence.

### Alternative D: Custom Runtime From Day One

Strengths:

 - Full control over policy, persistence, tools, and UX.
 - Cleanest security model if done well.

Weaknesses:

 - Highest engineering cost.
 - Highest correctness risk.
 - Duplicates fast-moving ecosystem work.

Best if: OpenCode cannot satisfy approval interception, session control, or security boundaries.

### Alternative E: Web App Plus Local Daemon

Strengths:

 - Easier updates and cross-device access.
 - Local daemon can own filesystem/shell.
 - Enterprise administration is easier later.

Weaknesses:

 - Weaker local-first story.
 - More complex auth and trust boundary.
 - Browser security model complicates local control.

Best if: team collaboration and centralized policy become more important than native desktop posture.

## Lock-In Risks

### OpenCode Lock-In

OpenCode may shape:

 - config format
 - session model
 - permission semantics
 - agent definitions
 - tool names
 - plugin support
 - provider assumptions

Migration risk: if OpenCode changes or cannot support a required feature, the app may need a runtime replacement and data migration.

Mitigation requirement: define app-owned canonical models for sessions, tool calls, artifacts, policies, and plugins, with OpenCode as an adapter, not the source of truth.

### OpenRouter Lock-In

OpenRouter is valuable, but routing, pricing, BYOK fee structure, model slugs, and provider behavior are external dependencies.

Migration risk: direct provider support later may require remapping model IDs, credentials, pricing, telemetry, and tool support.

Mitigation requirement: store provider-neutral model capability records and keep OpenRouter-specific fields isolated.

### Codex Plugin Compatibility Lock-In

Codex-compatible plugins are useful, but Codex plugin fields may not be a stable external standard. The specs rely on current scaffold behavior.

Migration risk: marketplace manifests and plugin metadata may drift from Codex or conflict with app-specific needs.

Mitigation requirement: define conformance level and namespaced extension policy.

### Tauri Lock-In

Tauri binds the app to Rust, system WebViews, Tauri capabilities, and plugin ecosystem.

Migration risk: if browser/artifact requirements need full Chromium or robust automation, migration to Electron could be expensive.

Mitigation requirement: keep frontend framework and backend service boundaries portable.

## Future Migration Risks

 - SQLite schema may not support team sync, conflict resolution, or remote collaboration.
 - Local absolute paths in session records will break across machines.
 - Worktree records may not map to cloud or remote runners.
 - Artifacts stored as local files need export/import and retention semantics.
 - Plugin enablement policies may not map to enterprise RBAC later.
 - Approval scopes like "always allow" may be unsafe in team-managed environments.
 - Skill and plugin indexes may need signed provenance later.
 - Sessions tied to one runtime may be hard to replay in another.

Hard question: Which data is portable by design, and which is explicitly machine-local?

## Missing Requirements To Add Before Architecture Freeze

 - Runtime control and conformance requirements.
 - Policy enforcement ownership requirements.
 - Explicit sandbox model for shell and tool subprocesses.
 - Network egress controls.
 - Secret redaction and leakage handling.
 - Prompt-injection handling for browser, documents, MCP, and plugins.
 - Plugin trust tiers and verification requirements.
 - Resource limits for concurrent agents and MCP servers.
 - Storage quotas and artifact retention.
 - Backup/export/import semantics.
 - Error recovery and crash consistency.
 - Model spend controls and budget alerts.
 - Accessibility acceptance criteria.
 - Cross-platform support matrix.
 - Non-code workflow acceptance criteria.
 - Compatibility matrix for standards and formats.
 - Telemetry/privacy policy.

## Difficult Questions

1. If OpenCode cannot provide pre-execution tool interception, is OpenCode still acceptable?
2. If the app must fork OpenCode to enforce policy, is that still cheaper than a custom runtime?
3. What is the minimum viable security model users can trust with private repositories, documents, and credentials?
4. Should remote MCP servers be disabled in MVP?
5. Should plugin scripts be disabled in MVP?
6. Is a general-purpose workspace credible without native document, spreadsheet, and research workflows?
7. Are worktrees meaningful for non-code projects, or is that a coding-specific concept leaking into the product model?
8. How will the app explain the difference between skills, plugins, MCP servers, agents, and instructions to normal users?
9. What data leaves the machine for every provider path?
10. Can users inspect and delete all memory, summaries, artifacts, and logs?
11. What happens when two agents edit the same non-Git folder?
12. Who is responsible when a marketplace plugin causes data loss?
13. What are the default-deny rules?
14. What is the rollback story after a bad agent action?
15. How does the product remain valuable if OpenCode adds its own GUI?
16. How does the product remain valuable if Codex opens its plugin marketplace and general-purpose workflows expand?
17. What part of the product is defensible beyond being a wrapper around OpenCode and OpenRouter?
18. Can the app support regulated customers without enterprise policy, audit export, and deterministic data boundaries?
19. How are model costs surfaced before and during long-running multi-agent work?
20. What is the support burden when users install arbitrary local plugins and MCP servers?

## CTO Gate Recommendation

Do not proceed directly from these specs into implementation. The documents are a good first pass, but the architecture has unresolved make-or-break assumptions.

Before implementation planning, require these decisions:

 - Pick the authoritative policy enforcement layer.
 - Prove or reject OpenCode as a controllable headless runtime.
 - Define MVP audience and first-class workflows.
 - Define standard conformance levels.
 - Decide plugin and remote MCP trust posture for MVP.
 - Define sandbox, network, and secret-handling requirements.
 - Define portability boundaries for local versus shareable data.

If those questions are answered cleanly, the recommended stack may be viable. If not, the product risks becoming a visually polished wrapper over unstable runtime assumptions with a security model that users cannot reason about.
