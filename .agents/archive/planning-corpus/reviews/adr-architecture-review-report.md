# ADR Architecture Review Report

This report reviews all ADR candidates and draft ADRs from an adversarial principal-architect perspective. It intentionally challenges the current direction and focuses on flaws, migration risk, lock-in risk, operational risk, scaling risk, and maintenance risk.

## Overall Assessment

The ADR set correctly identifies many unresolved decisions, but it still normalizes a risky architecture: a broad "general-purpose" AI workspace built on a coding-oriented runtime, a local desktop shell, standards with uneven maturity, plugin execution, MCP integration, and OpenRouter routing. The combined risk is not any single decision. The risk is the composition of many provisional decisions that all depend on each other being correct.

The strongest architectural objection is that the current direction may become a wrapper around OpenCode, OpenRouter, Codex-style plugins, and Tauri without owning the hard product boundaries: policy authority, runtime control, data portability, plugin trust, and non-code workflow semantics.

## Cross-ADR Concerns

 - The architecture is too dependent on unresolved runtime behavior.
 - Security decisions depend on policy enforcement authority, which is explicitly unresolved.
 - General-purpose product claims are not backed by general-purpose workflow primitives.
 - Plugin, marketplace, MCP, browser, and shell features combine into a large exfiltration surface.
 - "Standards-first" risks becoming "externally shaped by several unstable ecosystems."
 - Local-first storage conflicts with future team sync, marketplace policy, and enterprise governance.
 - The product may inherit the limitations of every ecosystem it tries to support while owning support burden for all of them.

## ADR-001: Product Scope And MVP Audience

Strongest argument against it: Leaving product scope unresolved invalidates downstream architecture. A general-purpose workspace and a coding-first workspace need different runtime, UX, artifact, security, and onboarding choices.

Future migration risks:
 - A coding-first data model may not fit document, spreadsheet, research, or operations workflows.
 - Worktree/session assumptions may leak into non-code workflows and become hard to remove.
 - Repositioning from developer tool to general workspace may require rewriting onboarding and core navigation.

Lock-in risks:
 - OpenCode and Git primitives could lock the product into developer workflows.
 - Early plugin strategy may become a crutch for missing product definition.

Operational risks:
 - Support will not know what workflows are officially supported.
 - Users may judge the product against broad claims that the MVP cannot satisfy.

Scaling risks:
 - Broad workflow support multiplies artifact types, connectors, policies, and test matrices.

Maintenance risks:
 - Core abstractions may accumulate one-off exceptions for every new workflow category.

Architectural challenge: Do not let "general-purpose" survive as a slogan unless first-class non-code workflows are specified.

## ADR-002: Desktop Application Shell

Strongest argument against it: Tauri may optimize for footprint before proving the product's actual hard requirement: reliable browser, artifact, and agent-runtime integration.

Future migration risks:
 - Moving from Tauri to Electron later would affect IPC, packaging, permissions, updater, browser surfaces, and native APIs.
 - Tauri WebView inconsistencies may force a second browser stack inside the app.

Lock-in risks:
 - Tauri capability and Rust backend patterns could shape APIs in ways that are expensive to port.
 - System WebView behavior becomes an implicit product dependency.

Operational risks:
 - Platform-specific WebView bugs may appear as product bugs.
 - Support complexity increases across macOS, Windows, and Linux.

Scaling risks:
 - Multiple browser/artifact views may consume more resources than expected despite Tauri's smaller shell.

Maintenance risks:
 - Maintaining Rust backend plus web frontend plus runtime subprocesses increases skill requirements.

Architectural challenge: Footprint is not enough justification if the product behaves like a browser-heavy workbench.

## ADR-003: Agent Runtime Strategy

Strongest argument against it: Using OpenCode as the runtime may outsource the product's most important control plane to a tool not designed for this GUI's security, persistence, and UX needs.

Future migration risks:
 - Replacing OpenCode later could require rewriting sessions, tool calls, permissions, agents, MCP handling, and provider configuration.
 - A forked OpenCode path would create permanent merge debt.

Lock-in risks:
 - OpenCode tool names, config formats, permission semantics, and session behavior may become de facto internal contracts.

Operational risks:
 - Runtime upgrades may break product behavior.
 - Debugging failures across GUI, adapter, runtime, tools, and model provider will be difficult.

Scaling risks:
 - Multiple runtime subprocesses may strain laptops.
 - Runtime state may not handle many concurrent sessions cleanly.

Maintenance risks:
 - Adapter code can become a fragile compatibility layer if OpenCode lacks a stable API.

Architectural challenge: If pre-tool interception is unavailable, the current architecture should be considered rejected, not merely adjusted.

## ADR-004: Policy Enforcement Authority

Strongest argument against it: The decision is unresolved, yet nearly every other security decision assumes it will be solved cleanly. That is unsafe.

Future migration risks:
 - Changing enforcement authority later will require reworking tools, plugins, MCP, approvals, audit logs, and runtime integration.

Lock-in risks:
 - If OpenCode remains co-authoritative, the app is locked to OpenCode's permission model.
 - If the backend becomes authoritative, the app is locked to deep runtime hooks or a proxy architecture.

Operational risks:
 - Split-brain approval failures could cause real data loss or exfiltration.
 - Users and support cannot reason about why an action was allowed or denied.

Scaling risks:
 - Distributed policy checks across layers become harder as tools, plugins, and MCP servers grow.

Maintenance risks:
 - Every new tool path needs duplicated policy logic unless a single gateway exists.

Architectural challenge: No implementation should start until there is exactly one authoritative enforcement point.

## ADR-005: Standards-First Interoperability

Strongest argument against it: Standards-first can become standards-hostage. The app may inherit unstable or under-specified conventions while promising compatibility it cannot guarantee.

Future migration risks:
 - External standards may change faster than the app can adapt.
 - App data may need repeated migrations to track third-party formats.

Lock-in risks:
 - "Open" standards can still produce lock-in if one vendor's implementation becomes the practical source of truth.
 - Codex-compatible plugins and OpenCode config may become proprietary dependencies by another name.

Operational risks:
 - Users will report incompatibilities as app bugs.
 - Support must understand multiple external ecosystems.

Scaling risks:
 - Every supported standard adds a compatibility test matrix.

Maintenance risks:
 - Import/export and round-trip behavior can become a permanent maintenance burden.

Architectural challenge: Compatibility should be treated as a product surface with strict conformance levels, not a principle.

## ADR-006: Project, Session, And Worktree Model

Strongest argument against it: Worktrees are a coding-specific isolation model being generalized beyond their natural domain.

Future migration risks:
 - Non-Git project support may require a parallel isolation model.
 - Session records tied to local paths and worktrees may not port to cloud or team contexts.

Lock-in risks:
 - Git becomes structurally privileged even if the product claims broader workflows.

Operational risks:
 - Dirty worktrees, orphaned branches, and failed cleanup will create user confusion.
 - Non-Git folders have no equivalent rollback story.

Scaling risks:
 - Many worktrees over large repos consume disk and file watcher resources.

Maintenance risks:
 - Worktree lifecycle edge cases are numerous: branch conflicts, submodules, symlinks, ignored files, LFS, and nested repos.

Architectural challenge: Treat worktrees as a coding plugin capability unless non-code isolation has equal status.

## ADR-007: Local-First Storage Model

Strongest argument against it: SQLite plus local artifact directories is a practical MVP store, but it may bake in assumptions that block sync, audit integrity, collaboration, and portability.

Future migration risks:
 - Team sync may require identity, conflict resolution, and remote object storage not represented now.
 - Local absolute paths will break across devices.
 - Audit log strengthening later may require a new event model.

Lock-in risks:
 - SQLite schema becomes the app's hidden platform contract.
 - Local artifact paths become hard-coded in session history.

Operational risks:
 - Corrupted local DB or artifact directory can strand user work.
 - Backup and restore behavior is undefined.

Scaling risks:
 - Large artifacts, logs, and transcripts may bloat local storage.
 - SQLite write contention may appear with many concurrent sessions.

Maintenance risks:
 - Schema migrations for a desktop app are painful when users skip versions.

Architectural challenge: Local-first cannot mean local-only assumptions embedded everywhere.

## ADR-008: Unified Tool Invocation And Ledger

Strongest argument against it: A normalized tool ledger can create false confidence. Logging tools is not the same as controlling tools or understanding data flow.

Future migration risks:
 - Tool schemas may need major changes when new runtimes expose richer semantics.
 - Ledger records may not replay across runtime versions.

Lock-in risks:
 - If the schema mirrors OpenCode or MCP too closely, it inherits their limits.

Operational risks:
 - Logs may capture secrets.
 - Ledger volume can become expensive to store and inspect.

Scaling risks:
 - High-frequency tool streams can overwhelm UI and SQLite writes.
 - Large stdout/stderr outputs can dominate disk usage.

Maintenance risks:
 - Every new tool type needs schema, redaction, preview, approval, and audit handling.

Architectural challenge: The ledger must not be accepted as a security boundary unless it is also enforcement-complete and tamper-aware.

## ADR-009: Permission And Approval Model

Strongest argument against it: Layered permissions may be too complex for users and still too weak against multi-step misuse.

Future migration risks:
 - Personal approval scopes may not translate to enterprise RBAC.
 - "Always allow" decisions may need to be revoked or migrated later.

Lock-in risks:
 - Permission semantics may become coupled to OpenCode and MCP capabilities.

Operational risks:
 - Users may approve prompts they do not understand.
 - Support will struggle to diagnose policy interactions.

Scaling risks:
 - As tools and plugins grow, prompt fatigue increases.
 - Policy evaluation cost and complexity increase with every layer.

Maintenance risks:
 - Risk classification rules require continuous updates.

Architectural challenge: Approval UX is not a substitute for restrictive defaults and data-flow controls.

## ADR-010: Filesystem And Shell Access Defaults

Strongest argument against it: Project-root access and ask-before-shell may still be too permissive if commands run as the user's account with network access.

Future migration risks:
 - Retrofitting sandboxed execution later may break workflows and plugin assumptions.
 - Policy records may not map to containers or separate OS users.

Lock-in risks:
 - The app can become dependent on user shell environments and local toolchains.

Operational risks:
 - Shell commands can delete data, leak secrets, install malware, or alter global machine state.
 - Symlinks can bypass naive project-root checks.

Scaling risks:
 - Concurrent shells can exhaust CPU, memory, file handles, or network.

Maintenance risks:
 - Cross-platform command safety classification is difficult and never complete.

Architectural challenge: Running arbitrary commands as the user should be treated as a major product liability, not a routine tool.

## ADR-011: MCP Integration Strategy

Strongest argument against it: MCP expands the attack surface before the app has proven policy enforcement, data-flow tracking, or plugin trust.

Future migration risks:
 - MCP spec changes may alter tools, resources, prompts, roots, sampling, or auth.
 - Server-specific behavior may leak into app assumptions.

Lock-in risks:
 - The app may become dependent on MCP servers for core features rather than native product capability.

Operational risks:
 - Remote MCP servers can exfiltrate data or misrepresent tool behavior.
 - Local stdio servers can execute arbitrary code.

Scaling risks:
 - Many MCP servers mean many processes, auth flows, health checks, logs, and tool lists.

Maintenance risks:
 - MCP compatibility testing and auth support will be ongoing work.

Architectural challenge: Remote MCP should be considered hostile until the policy and network model is settled.

## ADR-012: Agent Skills Support

Strongest argument against it: Skills are prompt/code packages that can quietly alter agent behavior and introduce supply-chain risk.

Future migration risks:
 - Skill behavior may differ across runtimes even with the same `SKILL.md`.
 - Auto-invocation metadata may not port cleanly.

Lock-in risks:
 - The app may rely on Codex/Claude-style skill semantics that are not truly standardized.

Operational risks:
 - Users may not know which skill influenced an output.
 - Skills can contain stale instructions or unsafe scripts.

Scaling risks:
 - Large skill catalogs create discovery, conflict, and context-selection problems.

Maintenance risks:
 - Skill validation, versioning, conflict resolution, and trust metadata require ongoing upkeep.

Architectural challenge: Skills should not be auto-trusted just because they are lightweight.

## ADR-013: AGENTS.md Instruction Support

Strongest argument against it: AGENTS.md is plain text, not policy. Treating it as more than advisory risks instruction injection and false authority.

Future migration risks:
 - Importing other instruction formats later may create precedence conflicts.
 - Nested instruction behavior may differ across tools.

Lock-in risks:
 - The app may inherit Codex-style assumptions about AGENTS.md precedence.

Operational risks:
 - A repository can contain malicious or stale AGENTS.md guidance.
 - Users may not understand which nested file applied.

Scaling risks:
 - Large monorepos with many instruction files create expensive and confusing resolution behavior.

Maintenance risks:
 - Conflict diagnostics and effective-instruction display become necessary UI features.

Architectural challenge: AGENTS.md must remain untrusted guidance, never enforceable permission.

## ADR-014: Plugin System Compatibility

Strongest argument against it: Codex-compatible plugin layout may not be a stable open standard, so adopting it can create compatibility debt without guaranteed ecosystem payoff.

Future migration risks:
 - Codex plugin schema changes could force app migrations.
 - App-specific extensions may break compatibility anyway.

Lock-in risks:
 - The app becomes coupled to Codex packaging conventions.

Operational risks:
 - Users may expect Codex plugins to work fully when only partial behavior is supported.
 - Hooks and scripts can create dangerous runtime side effects.

Scaling risks:
 - Supporting full plugin surfaces multiplies test cases across skills, MCP, scripts, hooks, assets, and app templates.

Maintenance risks:
 - Plugin validation must track both app rules and external convention drift.

Architectural challenge: Do not claim Codex compatibility unless compatibility is testable, versioned, and intentionally limited.

## ADR-015: Marketplace Model

Strongest argument against it: Marketplace support implies trust and curation responsibilities the product is not yet prepared to own.

Future migration risks:
 - Moving from local manifests to a public marketplace later may require signing, identity, review, ratings, revocation, and legal policy.
 - Team marketplace assumptions may not fit future enterprise policy.

Lock-in risks:
 - Codex-style marketplace metadata may shape the app's distribution model.

Operational risks:
 - Broken or malicious marketplace entries become support issues.
 - Update mechanisms can become supply-chain attack paths.

Scaling risks:
 - Marketplace search, ranking, compatibility, review, and update checks become product infrastructure.

Maintenance risks:
 - Vulnerability advisories, deprecations, and plugin removals require ongoing operations.

Architectural challenge: Local plugin install may be enough until trust and review operations exist.

## ADR-016: Plugin Trust And Execution

Strongest argument against it: Treating plugins as untrusted but still allowing scripts, hooks, assets, and MCP servers is internally tense. "Untrusted but executable" is a dangerous default.

Future migration risks:
 - Disabling unsafe plugin capabilities later will break existing plugins.
 - Adding signatures later will require migration of trust state.

Lock-in risks:
 - Plugin execution semantics may become tied to the first runtime and shell model.

Operational risks:
 - Plugin scripts can cause data loss or credential leakage.
 - Malicious assets can attack previewers.
 - Dependency installation can execute arbitrary code.

Scaling risks:
 - More plugins mean more permission prompts, processes, scripts, and support burden.

Maintenance risks:
 - Static analysis and trust scoring are hard to keep accurate.

Architectural challenge: The safer MVP assumption is no plugin execution until policy, sandboxing, and source integrity are proven.

## ADR-017: Artifact Model

Strongest argument against it: Typed artifacts sound harmless but create renderer, storage, provenance, and security commitments.

Future migration risks:
 - Changing artifact storage layout can break old sessions.
 - Adding sync later requires object identity, dedupe, and conflict handling.

Lock-in risks:
 - Artifact model may become tied to local filesystem paths and Tauri renderers.

Operational risks:
 - Generated HTML, PDFs, documents, and spreadsheets can contain active or malicious content.
 - Artifacts can leak secrets in previews or exports.

Scaling risks:
 - Screenshots, logs, PDFs, and generated sites can quickly consume disk.

Maintenance risks:
 - Every artifact type needs rendering, fallback, security handling, and lifecycle tests.

Architectural challenge: Artifact previewing is a security surface, not just UX polish.

## ADR-018: Browser/Web Viewing

Strongest argument against it: Browser integration is one of the highest prompt-injection and sandboxing risks, yet it is treated as a normal viewer feature.

Future migration risks:
 - Tauri browser limitations may force Chromium/Electron migration.
 - Browser automation APIs may not map cleanly across runtimes.

Lock-in risks:
 - A chosen browser engine can shape test, screenshot, and extraction features.

Operational risks:
 - Remote pages can inject instructions into model context.
 - Generated local pages may execute model-produced malicious scripts.

Scaling risks:
 - Multiple browser views and screenshots can consume memory and disk.

Maintenance risks:
 - Web compatibility bugs and security updates become product concerns.

Architectural challenge: Browser content should not enter model context automatically under any current unresolved security model.

## ADR-019: Model Provider Strategy

Strongest argument against it: OpenRouter simplifies provider access by adding a critical intermediary dependency in the most commercially and operationally sensitive layer.

Future migration risks:
 - Direct-provider support later may require model ID remapping, credential migration, pricing changes, and telemetry changes.
 - Stored provider settings may not map cleanly outside OpenRouter.

Lock-in risks:
 - Model slugs, routing behavior, BYOK fee policy, and fallback semantics are OpenRouter-shaped.

Operational risks:
 - Provider outages or routing changes affect user sessions.
 - Unexpected fallback routes can create cost, privacy, or compliance surprises.

Scaling risks:
 - Long-running multi-agent tasks can create uncontrolled spend.

Maintenance risks:
 - Model capability metadata, pricing, and provider behavior change constantly.

Architectural challenge: OpenRouter should not be the only viable provider path if local-first and BYOK control are core claims.

## ADR-020: Session And Memory Management

Strongest argument against it: Durable sessions and memory can quietly accumulate sensitive context and stale assumptions.

Future migration risks:
 - Session replay may not work across runtime or model changes.
 - Memory semantics may need rework for team or regulated environments.

Lock-in risks:
 - Summaries and memory may encode runtime-specific tool names and assumptions.

Operational risks:
 - Users may not know what is retained.
 - Summaries can be wrong but later treated as fact.

Scaling risks:
 - Long histories, summaries, and artifacts grow without clear retention.

Maintenance risks:
 - Compaction, summarization, deletion, and export logic must be consistent across versions.

Architectural challenge: Memory must be treated as sensitive data with explicit lifecycle, not a convenience feature.

## ADR-021: UX Layout And Interaction Model

Strongest argument against it: A dense command-center UI may fit experts but fail the stated general-purpose audience.

Future migration risks:
 - Reworking from expert workbench to accessible mainstream app is expensive.
 - Terminology baked into UI can constrain product direction.

Lock-in risks:
 - UI concepts like project, agent, plugin, MCP, worktree, and artifact may overfit developer mental models.

Operational risks:
 - Support burden rises if users cannot understand permissions and workflow state.

Scaling risks:
 - More panels and inspectors can become unusable as sessions and tools multiply.

Maintenance risks:
 - Complex UI state across many panes and concurrent sessions is hard to test.

Architectural challenge: UX should not expose the architecture as the product model unless the audience is explicitly technical.

## ADR-022: MVP Roadmap Sequencing

Strongest argument against it: Runtime validation first is logical only if the product scope is already right. It is not.

Future migration risks:
 - Building around a validated runtime may still produce the wrong product for the intended audience.
 - Delaying security validation until hardening may require large rewrites.

Lock-in risks:
 - Early runtime prototype success can bias the team toward OpenCode even if product fit is weak.

Operational risks:
 - Plugin and marketplace work may begin before trust operations are understood.

Scaling risks:
 - Resource-limit testing appears too late in the roadmap.

Maintenance risks:
 - Late hardening often turns into permanent backlog.

Architectural challenge: Product scope and security gates should precede runtime commitment, not follow it.

## ADR-023: Telemetry And Privacy

Strongest argument against it: Treating telemetry/privacy as lower-priority is inconsistent with local-first claims and model-provider routing.

Future migration risks:
 - Adding privacy controls after data has been collected creates trust and compliance problems.
 - Enterprise policies may require a different telemetry architecture.

Lock-in risks:
 - Provider telemetry and OpenRouter routing may shape privacy guarantees.

Operational risks:
 - Users may not understand what data leaves the machine.
 - Support diagnostics may accidentally collect secrets.

Scaling risks:
 - Telemetry pipelines, redaction, consent, and deletion become operational systems.

Maintenance risks:
 - Privacy policies must track product behavior, providers, plugins, MCP servers, and telemetry.

Architectural challenge: Privacy is not a later product setting. It is part of the architecture.

## Most Fragile Decision Chains

### Runtime And Policy Chain

ADR-003 depends on ADR-004, ADR-008, ADR-009, and ADR-010. If OpenCode cannot be controlled before execution, then the policy model, tool ledger, shell defaults, plugin execution, and MCP approval model all weaken at once.

### Plugin And Marketplace Chain

ADR-014, ADR-015, and ADR-016 depend on ADR-004, ADR-010, and ADR-011. Plugin compatibility is unsafe without policy authority, sandboxing, and MCP boundaries.

### General-Purpose Product Chain

ADR-001 conflicts with ADR-003, ADR-006, ADR-010, ADR-013, and ADR-021. A general-purpose product cannot simply expose developer abstractions and call itself broad.

### Local-First And Provider Chain

ADR-007, ADR-019, ADR-020, and ADR-023 are in tension. Local-first storage does not mean local-first privacy if model routing, telemetry, MCP, and plugins send data off-device.

## Decisions Most Likely To Need Reversal

 - Tauri as default shell if browser/artifact requirements dominate.
 - OpenCode as central runtime if pre-tool interception is unavailable.
 - Remote MCP support in MVP if policy and network controls remain unresolved.
 - Codex-compatible plugin support if the schema is not stable enough.
 - Marketplace support in MVP if trust operations are not staffed.
 - Plugin scripts/hooks if sandboxing is not strict.
 - General-purpose positioning if MVP remains coding-heavy.

## Architecture Stop Conditions

The architecture should stop advancing if any of these are true:

 - OpenCode cannot provide structured event streaming and pre-execution tool interception.
 - No single policy enforcement authority is chosen.
 - Shell commands run as the user with broad network access and weak path controls.
 - Remote MCP is allowed before data-flow and egress controls exist.
 - Plugin scripts or hooks are enabled before sandboxing and source integrity exist.
 - Product scope remains general-purpose while MVP acceptance criteria remain coding-first.
 - Privacy and telemetry are deferred beyond provider and plugin decisions.

## Final Review Position

The current ADR set is valuable because it exposes uncertainty, but the architecture is not yet safe to treat as a product plan. The strongest criticism is that the proposed direction combines too many provisional ecosystem bets at once: OpenCode, Tauri, OpenRouter, MCP, Codex plugin conventions, Agent Skills, Git worktrees, local shell, and marketplace distribution.

The architecture needs fewer assumptions, stricter boundaries, and a narrower first product. Until then, every additional compatibility promise increases migration debt and every additional execution surface increases security risk.
