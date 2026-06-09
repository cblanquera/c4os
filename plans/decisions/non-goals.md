# Non-Goals

This document defines what the project will not do for the MVP and, where appropriate, what the project should avoid permanently. It exists to prevent scope creep and protect the current architecture and validation plan.

The MVP product thesis is narrow: technical users will prefer a desktop AI workspace over a terminal-only agent flow if it provides a clearer control center for one local coding project with persistent sessions, visible tool activity, explicit approvals, and reviewable file changes.

## 1. General-Purpose Workspace Positioning For MVP

# Item

Classification: Not MVP.

The MVP will not claim to be a general-purpose AI workspace for writing, research, operations, documentation, spreadsheets, slides, or non-code knowledge work.

# Why It Is Not Included

The current smallest useful product is coding-first. General-purpose workflows require different primitives, artifact types, onboarding, integrations, and security assumptions.

# Risks Of Including It

 - Dilutes the MVP validation thesis.
 - Creates user expectations the product cannot satisfy.
 - Forces premature document, spreadsheet, browser, and research workflows.
 - Increases UX complexity for technical validation users.

# Conditions Under Which It May Be Reconsidered

After the coding-first MVP proves users value the desktop control center and after separate acceptance criteria exist for non-code workflows.

## 2. Custom Agent Runtime

# Item

Classification: Research Required.

The MVP will not start by building a custom agent runtime.

# Why It Is Not Included

The preferred direction is to validate an existing runtime integration before taking on the cost and risk of building an agent runtime from scratch.

# Risks Of Including It

 - Major engineering cost before product value is validated.
 - Rebuilds tool orchestration, permissions, sessions, model integration, and runtime state.
 - Delays learning about whether users want the desktop workspace at all.

# Conditions Under Which It May Be Reconsidered

If OpenCode cannot provide structured events, session control, and pre-execution tool interception, or if security requirements cannot be met through an adapter.

## 3. Multiple Runtime Adapters

# Item

Classification: Not MVP.

The MVP will not support multiple runtime engines such as OpenCode, Codex, Claude Code, local runtimes, and custom runtimes.

# Why It Is Not Included

The MVP only needs one runtime path to validate the desktop workspace thesis.

# Risks Of Including It

 - Turns the MVP into a runtime abstraction project.
 - Forces lowest-common-denominator tool and session models.
 - Multiplies testing across runtime behaviors.

# Conditions Under Which It May Be Reconsidered

After the OpenCode path validates or fails and the product needs runtime independence for strategic reasons.

## 4. Direct Model Provider Layer

# Item

Classification: Not MVP.

The MVP will not implement direct integrations with OpenAI, Anthropic, Google, local model servers, or other providers outside OpenRouter.

# Why It Is Not Included

OpenRouter-only setup is sufficient to test early provider acceptance and keeps model/provider scope small.

# Risks Of Including It

 - Adds credential, pricing, routing, capability, and error-handling complexity.
 - Distracts from runtime, approvals, and session validation.
 - Increases privacy disclosure scope.

# Conditions Under Which It May Be Reconsidered

If MVP users reject OpenRouter-only setup or if validation shows direct provider support is required for adoption.

## 5. Proprietary Skill Format

# Item

Classification: Never.

The project will not invent a proprietary skill format while portable Agent Skills remain viable.

# Why It Is Not Included

The project explicitly favors existing conventions and interoperability over proprietary workflow formats.

# Risks Of Including It

 - Creates ecosystem lock-in.
 - Prevents users from sharing skills with compatible tools.
 - Adds documentation and migration burden.

# Conditions Under Which It May Be Reconsidered

Only if existing skill conventions cannot represent required behavior and a namespaced extension is insufficient.

## 6. Proprietary MCP Protocol

# Item

Classification: Never.

The project will not create a custom replacement for MCP as the primary external tool/context protocol.

# Why It Is Not Included

MCP is the identified interoperability standard for tools and context. A proprietary replacement would undermine the standards-first direction.

# Risks Of Including It

 - Fragments integrations.
 - Requires custom server ecosystem work.
 - Increases vendor lock-in and support burden.

# Conditions Under Which It May Be Reconsidered

Do not reconsider as a replacement. App-specific extensions may exist behind adapters, but they should not replace MCP compatibility where MCP is appropriate.

## 7. MCP Integration In MVP

# Item

Classification: Not MVP.

The MVP will not include local or remote MCP server configuration.

# Why It Is Not Included

MCP expands the execution and data-exposure surface before policy authority and runtime control are proven.

# Risks Of Including It

 - Introduces exfiltration risk.
 - Requires roots, prompts, resources, sampling, auth, and tool approval handling.
 - Makes debugging and support harder.

# Conditions Under Which It May Be Reconsidered

After the policy enforcement model is resolved. Local stdio MCP should be reconsidered before remote MCP.

## 8. Remote MCP

# Item

Classification: Future Consideration.

Remote MCP will not be part of MVP or the first extension phase unless a threat model and network egress policy exist.

# Why It Is Not Included

Remote MCP has materially higher privacy, auth, and data exfiltration risk than local-only integrations.

# Risks Of Including It

 - Sends local context to external servers.
 - Complicates authorization and consent.
 - Increases prompt-injection surface.

# Conditions Under Which It May Be Reconsidered

After local MCP is validated, data-flow controls exist, and remote server trust levels are defined.

## 9. Plugin System In MVP

# Item

Classification: Not MVP.

The MVP will not install, enable, or execute plugins.

# Why It Is Not Included

Plugins are not required to validate the core desktop workspace thesis and introduce supply-chain risk.

# Risks Of Including It

 - Enables untrusted scripts, hooks, assets, or MCP servers.
 - Requires permission review, package validation, and source integrity.
 - Increases support burden before core value is proven.

# Conditions Under Which It May Be Reconsidered

After plugin format compatibility, trust tiers, sandboxing, and source integrity are specified.

## 10. Plugin Scripts And Hooks

# Item

Classification: Future Consideration.

Plugin scripts and hooks are not allowed in MVP.

# Why It Is Not Included

Executable plugin content is high risk and not needed for first validation.

# Risks Of Including It

 - Arbitrary local code execution.
 - Credential leakage.
 - Data loss.
 - Hard-to-debug side effects.

# Conditions Under Which It May Be Reconsidered

Only after plugin signing, sandboxing, permission review, and explicit user trust flows exist.

## 11. Marketplace

# Item

Classification: Future Consideration.

The MVP will not include plugin marketplace browsing, installation, updates, or remote marketplace manifests.

# Why It Is Not Included

Marketplace implies trust, curation, liability, update verification, and vulnerability response. It is unnecessary while plugins are excluded.

# Risks Of Including It

 - Supply-chain exposure.
 - Support burden for third-party content.
 - Legal and trust ambiguity.
 - Premature distribution of unsafe extensions.

# Conditions Under Which It May Be Reconsidered

After plugin execution is safe and marketplace operational responsibilities are defined.

## 12. Team Collaboration

# Item

Classification: Future Consideration.

The MVP will not include shared projects, shared sessions, multiplayer collaboration, or team coordination.

# Why It Is Not Included

The MVP validates a single-user local workflow. Team features require identity, sharing, policy, conflict handling, and sync.

# Risks Of Including It

 - Forces account system and permission models.
 - Creates data sharing and privacy complexity.
 - Complicates local-first storage.

# Conditions Under Which It May Be Reconsidered

After single-user value is validated and team use cases are specified.

## 13. Cloud Synchronization

# Item

Classification: Future Consideration.

The MVP will not sync projects, sessions, artifacts, settings, or memory across devices.

# Why It Is Not Included

Local-first validation does not require cloud sync.

# Risks Of Including It

 - Requires conflict resolution.
 - Complicates secret handling.
 - Turns local artifacts and absolute paths into portability problems.
 - Adds backend operations.

# Conditions Under Which It May Be Reconsidered

After local storage boundaries, export semantics, and team/cross-device needs are validated.

## 14. Multi-Tenant SaaS

# Item

Classification: Never for MVP; Future Consideration only as a separate product strategy.

The MVP will not be a hosted multi-tenant SaaS product.

# Why It Is Not Included

The project is currently a local desktop workspace. Multi-tenant SaaS requires a different architecture, threat model, data model, and operating model.

# Risks Of Including It

 - Invalidates local-first assumptions.
 - Requires tenant isolation, billing, account management, cloud execution, and compliance.
 - Distracts from validating local workflow value.

# Conditions Under Which It May Be Reconsidered

Only after a deliberate product strategy change, not as incremental MVP scope.

## 15. Enterprise Admin Console

# Item

Classification: Future Consideration.

The MVP will not include enterprise policy management, admin dashboards, RBAC, or organization controls.

# Why It Is Not Included

The MVP targets individual technical users, not enterprise buyers.

# Risks Of Including It

 - Forces policy and identity complexity.
 - Requires support and compliance commitments.
 - Slows validation of core local workflow.

# Conditions Under Which It May Be Reconsidered

After individual value is proven and enterprise requirements are gathered.

## 16. Compliance-Grade Audit

# Item

Classification: Future Consideration.

The MVP will not claim compliance-grade, tamper-evident, or regulated audit capabilities.

# Why It Is Not Included

The MVP tool ledger is for user inspection and debugging, not legal or compliance assurance.

# Risks Of Including It

 - Requires stronger integrity, retention, export, and access controls.
 - Creates regulatory expectations.
 - Expands scope far beyond MVP validation.

# Conditions Under Which It May Be Reconsidered

After regulated customer requirements are explicitly targeted.

## 17. Data-Flow-Aware Policy Engine

# Item

Classification: Future Consideration.

The MVP will not implement information-flow tracking across tool chains.

# Why It Is Not Included

MVP validates basic approval and visibility. Data-flow policy is a more advanced security model.

# Risks Of Including It

 - Adds complex policy semantics.
 - Requires classifying data sources, sinks, and transformations.
 - Slows MVP significantly.

# Conditions Under Which It May Be Reconsidered

After basic approval flows are validated and external integrations create stronger exfiltration risk.

## 18. Always-Allow Or Permanent Project-Wide Approvals

# Item

Classification: Not MVP.

The MVP will not support always-allow or permanent project-wide approval scopes.

# Why It Is Not Included

Persistent broad approvals are risky before trust and policy semantics are proven.

# Risks Of Including It

 - Users may accidentally grant dangerous long-lived capabilities.
 - Increases blast radius of malicious prompts or runtime errors.
 - Hard to explain and revoke safely.

# Conditions Under Which It May Be Reconsidered

After approval comprehension and default-deny rules are validated.

## 19. Multiple Concurrent Agents

# Item

Classification: Not MVP.

The MVP will not support multiple concurrent agents or delegated subagents.

# Why It Is Not Included

Concurrency adds runtime, resource, session, and conflict complexity. One active session is enough to validate the product thesis.

# Risks Of Including It

 - File edit conflicts.
 - Resource exhaustion.
 - Confusing UI state.
 - More complicated approvals and audit.

# Conditions Under Which It May Be Reconsidered

After single-session value is proven and resource limits are measured.

## 20. Multiple Active Sessions

# Item

Classification: Not MVP.

The MVP will not support multiple active sessions per project.

# Why It Is Not Included

The smallest validation loop needs only one active session.

# Risks Of Including It

 - Confusing project state.
 - Concurrent file changes.
 - More complicated persistence and runtime supervision.

# Conditions Under Which It May Be Reconsidered

After users demonstrate recurring need to run or compare multiple sessions.

## 21. Worktree Manager

# Item

Classification: Not MVP.

The MVP will not create or manage Git worktrees.

# Why It Is Not Included

Worktrees are useful for parallel coding work but not required for validating one agent session in one local project.

# Risks Of Including It

 - Branch cleanup complexity.
 - Dirty worktree confusion.
 - Git edge cases around submodules, LFS, nested repos, and conflicts.

# Conditions Under Which It May Be Reconsidered

After users need parallel sessions or safer isolated edits.

## 22. Non-Git Project Workflows

# Item

Classification: Not MVP.

The MVP will not support non-Git project workflows as first-class.

# Why It Is Not Included

The MVP uses Git status and diffs as the trust and review mechanism.

# Risks Of Including It

 - Requires alternative rollback and change review.
 - Weakens coding-first validation.
 - Adds file conflict and provenance complexity.

# Conditions Under Which It May Be Reconsidered

After the coding-first MVP validates and non-code workflows are explicitly scoped.

## 23. Browser Panel And Web Automation

# Item

Classification: Future Consideration.

The MVP will not include an in-app browser, DOM extraction, screenshots, browser testing, or web content ingestion.

# Why It Is Not Included

Browser content is a prompt-injection and sandboxing risk. Browser-heavy features may also affect the Tauri versus Electron decision.

# Risks Of Including It

 - Remote prompt injection.
 - Active content execution.
 - Browser engine compatibility issues.
 - Higher memory and security complexity.

# Conditions Under Which It May Be Reconsidered

After artifact/browser security research and desktop shell validation.

## 24. Rich Document, Spreadsheet, And Slide Workflows

# Item

Classification: Future Consideration.

The MVP will not provide first-class document, spreadsheet, slide, or PDF workflows.

# Why It Is Not Included

These workflows are part of broader general-purpose ambitions, not the coding-first MVP.

# Risks Of Including It

 - Adds many artifact renderers and editors.
 - Introduces file-format security risks.
 - Distracts from validating local coding sessions.

# Conditions Under Which It May Be Reconsidered

After MVP validates core workspace value and non-code workflow acceptance criteria are written.

## 25. Rich Artifact Preview Suite

# Item

Classification: Not MVP.

The MVP will not preview all artifact types. It will keep basic text, Markdown, logs, diffs, and generated files.

# Why It Is Not Included

Rich renderers add security and maintenance cost.

# Risks Of Including It

 - Active content execution.
 - Renderer vulnerabilities.
 - Large storage and performance burden.

# Conditions Under Which It May Be Reconsidered

After artifact preview security and retention policies are defined.

## 26. Long-Term Memory

# Item

Classification: Not MVP.

The MVP will not store durable cross-session memory beyond session persistence and basic tool records.

# Why It Is Not Included

Memory creates privacy, correctness, and deletion complexity.

# Risks Of Including It

 - Stale or false summaries influence future sessions.
 - Sensitive data persists unexpectedly.
 - Requires inspect, edit, and delete controls.

# Conditions Under Which It May Be Reconsidered

After session retention and privacy controls are defined.

## 27. Automatic Browser Or Document Context Ingestion

# Item

Classification: Never for MVP.

The MVP will not automatically ingest browser pages, documents, or external content into model context.

# Why It Is Not Included

Automatic ingestion is a prompt-injection and data-leakage risk.

# Risks Of Including It

 - Untrusted content can steer the model.
 - Sensitive data may be sent to providers unintentionally.
 - Users may not understand what context was used.

# Conditions Under Which It May Be Reconsidered

Only with explicit user selection, context provenance, and prompt-injection controls.

## 28. Mobile Clients

# Item

Classification: Future Consideration.

The MVP will not include mobile clients.

# Why It Is Not Included

The product validates local desktop project workflows.

# Risks Of Including It

 - Requires sync and remote execution assumptions.
 - Adds platform-specific UX and security work.
 - Does not validate local desktop thesis.

# Conditions Under Which It May Be Reconsidered

After desktop usage is proven and mobile workflows are identified.

## 29. Workflow Automation Designer

# Item

Classification: Future Consideration.

The MVP will not include a visual or complex workflow automation builder.

# Why It Is Not Included

The MVP validates interactive agent sessions, not automation design.

# Risks Of Including It

 - Expands product into automation platform.
 - Requires scheduling, permissions, retry semantics, and observability.
 - Increases user complexity.

# Conditions Under Which It May Be Reconsidered

After repeated workflows emerge from user validation.

## 30. Proprietary Workspace Standards

# Item

Classification: Never.

The project will not invent proprietary replacements for widely adopted workspace conventions such as AGENTS.md, Agent Skills, MCP, or Codex-style plugin metadata where those conventions are viable.

# Why It Is Not Included

The architectural direction favors interoperability.

# Risks Of Including It

 - Locks users into this app.
 - Reduces ecosystem compatibility.
 - Creates migration burden.

# Conditions Under Which It May Be Reconsidered

Only as namespaced extensions when existing standards cannot express required app-specific metadata.

## 31. Full OpenCode Config Ownership In MVP

# Item

Classification: Research Required.

The MVP will not assume full ownership of OpenCode configuration semantics beyond what is needed for the runtime adapter.

# Why It Is Not Included

OpenCode config compatibility and ownership are unresolved.

# Risks Of Including It

 - Couples the app to OpenCode internals.
 - Creates migration risk if OpenCode config changes.
 - Confuses source of truth between app settings and runtime settings.

# Conditions Under Which It May Be Reconsidered

After runtime control and standards conformance research defines which config is imported, mirrored, or owned.

## 32. Public Growth Features

# Item

Classification: Not MVP.

The MVP will not include sharing, public galleries, community templates, ratings, reviews, or social distribution.

# Why It Is Not Included

The MVP is a validation product for local technical workflows.

# Risks Of Including It

 - Distracts from core trust and runtime questions.
 - Adds moderation and support obligations.
 - Encourages marketplace scope prematurely.

# Conditions Under Which It May Be Reconsidered

After core product value and extension trust are validated.

