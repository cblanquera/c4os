# Finalized Decisions

This document summarizes ADR decisions that are settled for the MVP scope. Some remain provisional for later product phases, but the MVP direction is clear enough to guide planning.

## ADR-001: MVP Product Scope And Audience

Summary: MVP is a single-user, coding-first desktop workspace for technical users working in local repositories.

Status: Finalized for MVP.

Rationale: The original general-purpose workspace claim was too broad. MVP scope narrows validation to whether a desktop control center improves trust, resume, inspection, and control for agent-assisted local coding.

Affected documents:

 - `plans/adr/001-product-scope-and-mvp-audience.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-validation-plan.md`
 - `plans/reviews/adr-architecture-review-report.md`

Next action: Keep MVP messaging coding-first and avoid general-purpose claims until non-code workflows are validated.

## ADR-002: Tauri MVP Desktop Shell

Summary: MVP uses Tauri for the desktop shell. Chromium, Electron, or controlled Chromium surfaces are deferred until post-MVP browser or rich-preview needs are validated.

Status: Finalized for MVP.

Rationale: Accepted MVP scope excludes browser panels, browser automation, screenshots, DOM extraction, active HTML rendering, and rich artifact previews. Tauri WebView is sufficient for the MVP app UI and text-like artifact previews unless Phase 0 proves otherwise. Deferring Chromium avoids premature browser-content and rich-preview security obligations.

Affected documents:

 - `plans/adr/002-desktop-application-shell.md`
 - `plans/specs/architecture-specification.md`
 - `plans/spikes/SPIKE-007-tauri-electron-browser-artifacts.md`
 - `plans/acceptance/artifacts.md`

Next action: Validate Tauri WebView for MVP UI and text-like previews; revisit Chromium after MVP if browser panels, browser automation, screenshots, DOM extraction, generated HTML previews, or rich artifact rendering become validated needs.

## ADR-007: Local-First MVP Storage And Retention

Summary: MVP uses app-owned local SQLite metadata, local artifact files, and OS keychain or platform credential storage for the OpenRouter API key.

Status: Finalized for MVP.

Rationale: SQLite and local artifact storage are sufficient for a single-user, one-active-session MVP. The app owns canonical user-facing records for sessions, messages, tool calls, approvals, artifacts, projects, models, and settings. Runtime-native OpenCode records are adapter references only. The OpenRouter API key is the only MVP provider secret and must live in the OS keychain or platform credential store; if that storage fails, provider setup is blocked. MVP retained records remain local indefinitely by default. Plaintext secret storage, project-file secret storage, env-file writes, custom encrypted vaults, secret export/import, session delete, team sync, compliance-grade audit, quotas, automatic cleanup, export/import, and cross-device portability are not MVP goals.

Affected documents:

 - `plans/adr/007-local-first-storage-model.md`
 - `plans/specs/data-model-specification.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-008-storage-audit-portability.md`

Next action: Validate SQLite persistence and crash recovery in the storage spike before detailed implementation planning.

## ADR-008: Basic Tool Ledger

Summary: MVP includes a basic tool ledger for user inspection and debugging, not compliance-grade audit.

Status: Finalized for MVP.

Rationale: The MVP needs visibility into tool activity, approvals, outputs, and affected files. Tamper-evident audit and data-flow-aware policy are deferred.

Affected documents:

 - `plans/adr/008-unified-tool-invocation-and-ledger.md`
 - `plans/specs/architecture-specification.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-success-metrics.md`

Next action: Define the minimal tool record schema during runtime and policy validation.

## ADR-009: MVP Approval Scope

Summary: MVP supports one-time allow, narrow shell session allow, and deny. Session allow applies only to matching non-destructive shell commands inside the selected project root or approved project subpath.

Status: Finalized for MVP.

Rationale: Narrow approval scopes validate user trust without introducing persistent risky grants. Project-root file reads and Git inspection are logged but not approval-gated. Session allow never covers destructive shell commands, outside-root paths, secret-deny files, or Git state-changing actions. Always-allow and project-wide approvals are excluded from MVP.

Affected documents:

 - `plans/adr/009-permission-and-approval-model.md`
 - `plans/specs/security-specification.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-user-journeys.md`

Next action: Research default-deny and destructive-command classification in the local execution spike.

## ADR-004: MVP Approval Gateway Authority

Summary: The Tauri/Rust backend is the authoritative Approval Gateway for MVP file writes, shell commands, and Git state-changing actions. Pending approval prompts are non-durable runtime state; answered approval decisions are durable ledger records. File-write approvals include target path, action type, and bounded diff or summary when available. Multiple file writes may be approved only as an explicit visible atomic batch within readable caps.

Status: Finalized for MVP. Broader post-MVP policy remains unresolved.

Rationale: The MVP needs one mandatory control point for user approval and ledger completeness. Runtime-native permissions may provide defense in depth, but OpenCode must not execute MVP-controlled state-changing actions before backend approval. Pending approval prompts are runtime state, not durable decisions; close, crash, force-quit, or restart discards them, and the agent must repropose the action later. Answered approval decisions are durable audit records visible only in the local activity ledger, with structured metadata, decision, timestamp, resulting action status, and bounded redacted prompt summary or diff reference. Approval records must not include full prompt replay blobs, raw command output, or raw secret values, and cannot be edited, deleted, exported, copied in bulk, downloaded as JSON, shared, or included in support bundles in MVP. Normal OS text selection/copy of visible ledger text is acceptable, but the app does not provide dedicated approval-record copy/export controls. This avoids approve-after-restart behavior and stale prompt replay while preserving answered decisions for local inspection without creating a leakage-prone export surface. File-write approvals need enough preview context for trust: target path, action type, and a bounded diff or summary when available. If a safe diff cannot be produced, the approval must disclose that limitation rather than hiding it. Multiple file writes can be approved together only as an explicit atomic batch where every target path, action type, and per-file preview state is visible. Batch approvals are capped to a fixed MVP product threshold, initially 10 files plus bounded total preview size; oversized batches are blocked and must be split by the agent. These caps are intentionally not user-configurable in MVP so approval behavior is predictable and testable. If any item is blocked by policy, including outside-root or secret-deny targets, the whole batch is blocked and the agent must propose a smaller valid batch. No-safe-preview items must be disclosed, but the batch remains approve-or-deny as proposed. Dedicated approval-record copy buttons, approval record export/copy/share workflows, durable pending approvals, approve-after-restart, stale approval prompt replay, full prompt replay blobs, raw command output, raw secret values in approval records, approval decision editing/deletion, user-configurable approval thresholds, per-project caps, advanced safety settings, giant scrollback approval prompts, unbounded generated diffs, approve-by-summary-only for oversized batches, checkbox-per-file partial approval, automatic subset execution, hidden batch rewriting, blanket approve-all-future-writes, project-wide write approval, hidden partial execution, full editors, merge UI, and multi-file review workflows are deferred beyond the MVP approval prompt, changed-file list, and diff viewer.

Affected documents:

 - `plans/adr/004-policy-enforcement-authority.md`
 - `plans/specs/architecture-specification.md`
 - `plans/specs/security-specification.md`
 - `plans/spikes/SPIKE-002-policy-enforcement-authority.md`

Next action: Validate that OpenCode can expose file write, shell, and Git state-changing proposals before execution; otherwise choose an adapter, proxy, fork, or scope change.

## ADR-010: MVP Project-Root File Boundary And Shell Baseline

Summary: MVP allows project-root file reads without per-read approval, requires approval for file writes, blocks all file reads and writes outside the selected project root, and runs approved shell commands as the current OS user with normal network access.

Status: Finalized for MVP file boundaries, shell execution account, and shell network baseline. Shell safety details remain unresolved.

Rationale: The selected Git project root is the MVP trust boundary. External-directory grants add a second boundary, persistence, and revocation model before the Git-backed trust loop is validated. Current-user shell execution with normal network access preserves local developer toolchain compatibility but requires explicit approval, environment filtering, redaction, process termination, and honest non-sandbox/non-network-isolation messaging. Live terminal output may be visible while a command is running or while the live terminal buffer remains open. After completion, an open drawer may remain temporarily visible in the same app session only when labeled live/ephemeral, but live buffers are not retained after navigation away, reload, app close, or session restore. Durable shell output is limited to bounded redacted/truncated summaries. If a safe summary cannot be produced, MVP persists command metadata plus an explicit output-omitted marker rather than falling back to raw stdout/stderr. Shell summaries may expose safe reason labels such as `truncated_by_size`, `redacted_secret_pattern`, or `output_omitted_safe_summary_failed`, but not redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction hints. Normal OS text selection/copy of visible shell summary text is acceptable, but retained live terminal buffers, unlabeled completed-command live drawers, dedicated raw shell output copy/export controls, raw stdout/stderr persistence, and reconstructable shell summary metadata are deferred.

Affected documents:

 - `plans/adr/010-filesystem-and-shell-access-defaults.md`
 - `plans/specs/security-specification.md`
 - `plans/acceptance/file-access.md`
 - `plans/acceptance/project-management.md`

Next action: Validate symlink canonicalization, secret-deny patterns, environment filtering, redaction, and destructive-command categories in the local execution sandboxing spike.

## ADR-011: MCP Excluded From MVP

Summary: MCP is not included in MVP.

Status: Finalized for MVP.

Rationale: MCP is strategically important, but it expands the execution and data-exposure surface before runtime policy authority is proven.

Affected documents:

 - `plans/adr/011-mcp-integration-strategy.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-004-mcp-scope-and-threat-model.md`

Next action: Validate local-only MCP scope for V1 after MVP runtime and policy gates are settled.

## ADR-013: Root AGENTS.md Display Only

Summary: MVP discovers and displays only root `AGENTS.md`; explicit reads are allowed as normal logged project-root file reads, and approved edits are normal project-root file writes. App-owned automatic model-context injection, automatic reload after writes, editable system/project prompts, hidden app instruction layers, skill creator workflows, nested resolution, permission influence, and instruction conflict handling are deferred.

Status: Finalized for MVP.

Rationale: Root `AGENTS.md` validates the project-instruction convention without taking on model-context injection, nested precedence, conflict diagnostics, or policy interactions. Displaying the file is passive; if the user or runtime explicitly reads it during a session, it follows the same scoped and logged project-root file-read path as any other project file and can enter model context only through that explicit read. If the agent proposes editing it, the edit follows the same approval-gated project-root file-write path as any other project file. Approved edits do not automatically reload instructions into model context, permission state, or instruction precedence. MVP should not include a generic prompt editor, editable system prompt, project prompt editor, prompt template system, instruction composer, or hidden app-authored instruction layer. A future skill creator can make sense as a separate workflow that produces explicit, reviewable instruction artifacts, similar to Codex skills, but that belongs after the core session and approval loop are validated. If OpenCode loads `AGENTS.md` natively, that must be disclosed as runtime behavior rather than treated as app-owned policy.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-user-journeys.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`

Next action: Include `AGENTS.md` and future skill-artifact conformance levels in the standards matrix, and verify OpenCode-native loading behavior in the runtime spike.

## ADR-005: Narrow MVP Compatibility Claims

Summary: MVP may claim only OpenRouter-backed model access, local Git project support, root `AGENTS.md` display, and app-owned text-like artifact records.

Status: Finalized for MVP. Broader standards conformance remains unresolved.

Rationale: "Support" is ambiguous across AGENTS.md, Agent Skills, MCP, Codex plugins, OpenCode config, import/export, and round-trip behavior. Narrow claims prevent false compatibility promises before conformance tiers are defined.

Affected documents:

 - `plans/adr/005-standards-first-interoperability.md`
 - `plans/spikes/SPIKE-006-standards-conformance-matrix.md`
 - `plans/specs/architecture-specification.md`
 - `plans/mvp/mvp-scope.md`

Next action: Complete the standards conformance matrix before expanding compatibility claims beyond MVP.

## ADR-006: No OpenCode Config Compatibility In MVP

Summary: MVP may store app-owned OpenCode launch settings and adapter references, but does not import, mirror, edit, export, or round-trip OpenCode config.

Status: Finalized for MVP.

Rationale: Runtime adapter plumbing is necessary to launch OpenCode, but presenting it as config compatibility would imply ownership and round-trip behavior the MVP does not validate. Any existing OpenCode config behavior that affects runtime execution must be discovered in Phase 0 and disclosed.

Affected documents:

 - `plans/adr/005-standards-first-interoperability.md`
 - `plans/specs/architecture-specification.md`
 - `plans/specs/data-model-specification.md`
 - `plans/acceptance/agent-execution.md`

Next action: During the OpenCode runtime spike, identify existing config behavior that affects runtime execution and decide whether it can be disclosed, disabled, or must become an adapter constraint.

## ADR-015: Marketplace Excluded From MVP

Summary: Marketplace support is excluded from MVP.

Status: Finalized for MVP.

Rationale: Marketplace support implies trust, curation, source integrity, update verification, and support obligations that are unnecessary for validating the core desktop workspace thesis.

Affected documents:

 - `plans/adr/015-marketplace-model.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-015-marketplace-operational-trust.md`

Next action: Revisit marketplace after plugin trust and source integrity are defined.

## ADR-016: Plugin Execution Excluded From MVP

Summary: Plugin scripts, hooks, and plugin execution are excluded from MVP.

Status: Finalized for MVP.

Rationale: Plugin execution increases supply-chain and endpoint risk. It does not directly validate the MVP thesis.

Affected documents:

 - `plans/adr/016-plugin-trust-and-execution.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-005-plugin-format-and-trust.md`

Next action: Research plugin format and trust tiers for a later phase.

## ADR-018: Browser Automation Excluded From MVP

Summary: Browser automation, DOM extraction, screenshots, and automatic browser-content ingestion are excluded from MVP.

Status: Finalized for MVP.

Rationale: Browser content is a prompt-injection and sandboxing risk. MVP can validate core coding workflow without browser automation.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-013-artifact-browser-preview-security.md`

Next action: Validate artifact/browser security separately before V1/V2 browser features.

## ADR-017: MVP Text-Like Artifact Boundary

Summary: MVP artifacts are app-owned records or file links for plain text, Markdown, logs, diffs, and generated source or config files only.

Status: Finalized for MVP.

Rationale: Text-like artifacts preserve useful session outputs without taking on active HTML rendering, Chromium/browser dependencies, rich media/document previewers, export/duplicate workflows, or artifact execution. Rich previews are a security and renderer commitment that is not required to validate the coding-first MVP.

Affected documents:

 - `plans/acceptance/artifacts.md`
 - `plans/specs/functional-specification.md`
 - `plans/specs/ux-specification.md`
 - `plans/spikes/SPIKE-013-artifact-browser-preview-security.md`

Next action: Revisit rich artifact previews post-MVP after renderer isolation, model-context ingestion, Chromium requirements, and provenance are specified.

## ADR-019: OpenRouter-Only MVP Provider Path

Summary: MVP uses OpenRouter as the only provider path and one selected OpenRouter model fixed for the whole session. The OpenRouter API key is stored only in the OS keychain or platform credential store and cannot be updated or revoked while a session is running. App-owned provider/model settings are authoritative over runtime config. OpenRouter-bound model context is limited, explicit, and covered by standing disclosure rather than per-call approval. OpenRouter or network outages fail closed with recoverable session state. Model metadata is best effort; detailed token, cost, credit, and billing accounting are excluded.

Status: Finalized for MVP.

Rationale: OpenRouter reduces provider-integration scope and lets the MVP test whether users accept the preferred routed provider direction. The selected model is fixed for the whole session after creation, including running sessions and idle sessions with transcript history; project default model changes apply only to future sessions and must not restart, reconfigure, hot-swap, or migrate the active runtime. Once a user message is submitted, provider or network failure does not delete or rewrite it; the failed assistant/run status is persisted and the user must explicitly retry or send a follow-up. A retry command may exist, but it appends a new retry action/status rather than replacing, regenerating, branching from the failed assistant record, or switching models. MVP validates credential presence and selected model route viability, not OpenRouter account or billing state. Shell tool context sent to the model is limited to the persisted bounded redacted/truncated summary, explicit output-omitted marker, and safe reason labels; live raw terminal buffers, omitted raw shell output, redacted substrings, sensitive raw byte counts, offsets, hashes, and reconstruction metadata must never be sent. Direct providers, local models, OpenRouter credit balance, billing link management, top-up flows, invoice links, spend warnings, account diagnostics, per-call token counts, cost estimates, model-call accounting, spend history, budget meters, budget enforcement, prompt deletion after failed model calls, retry-in-place, response replacement, branch-from-failure, hidden retry loops, silent prompt resend, hot key rotation, per-session credential switching, mid-call credential retry, offline model fallback, automatic model switching after outage, queued background resend, synthetic assistant continuation, per-call context approval, raw prompt export, token-by-token context inspection, editable context composers, whole-repo model-context indexing, hidden background file ingestion, app-owned automatic root `AGENTS.md` injection, implicit artifact ingestion, live raw terminal buffer context, omitted raw shell output context, reconstructable shell metadata context, app-owned live model catalog sync, cost dashboards, hot model swap, run restart for model change, in-flight model migration, runtime reconfiguration from default-model change, idle-session model switching, retry-with-different-model, per-message model override, mid-session model switching, per-agent model overrides, fallback routing, budget controls, provider-specific app settings, BYOK provider subconfiguration inside the app, custom secret vaults, and secret export/import are deferred. The UI must not show one selected OpenRouter model while OpenCode uses another provider or model, so undetectable runtime config override is a Phase 0 blocker. Secret-deny file contents must never be sent to OpenRouter. Pricing, capability, and route metadata may be stale or unavailable; MVP labels those states but does not block otherwise viable sessions because of metadata freshness.

Affected documents:

 - `plans/adr/019-model-provider-strategy.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/mvp/mvp-validation-plan.md`
 - `plans/spikes/SPIKE-009-provider-privacy-cost.md`

Next action: Validate platform keychain storage, session credential-reference behavior, OpenRouter-bound context visibility, provider/network outage handling, privacy, routing, cost disclosure, and metadata freshness labeling requirements before implementation planning.

## ADR-021: One Default Coding Agent In MVP

Summary: MVP uses one default coding agent/runtime persona. Runtime agent references may be stored only as adapter metadata. Project default agent UI, custom agents, persona management, agent switching, retry-with-different-agent, per-message persona, subagents, and handoffs are excluded.

Status: Finalized for MVP.

Rationale: One default coding agent is enough to validate the desktop control loop. If OpenCode requires an internal agent reference, the app may persist it as adapter metadata, not user-facing agent/persona configuration. Exposing project default agent controls, custom agents, persona switching, retry-with-different-agent, per-message persona, or agent migration would add prompt, permission, skill, model, and handoff complexity before the core runtime and approval loop are proven.

Affected documents:

 - `plans/specs/functional-specification.md`
 - `plans/specs/data-model-specification.md`
 - `plans/acceptance/agent-execution.md`
 - `plans/mvp/mvp-scope.md`

Next action: During runtime validation, identify whether OpenCode requires an internal agent reference and persist it as adapter metadata, not user-facing agent configuration.

## ADR-020: No Long-Term Memory In MVP

Summary: MVP persists append-only sessions, messages, tool records, approvals, logs, local diagnostics, and MVP artifacts indefinitely by default; global search, message edit/delete, session delete, and long-term memory are excluded.

Status: Finalized for MVP.

Rationale: Long-term memory increases privacy and correctness risk. MVP only needs session continuity and retained local history. User and assistant messages are append-only; runtime records may receive status updates. Corrections happen through follow-up messages or new sessions. Retry is recorded as a new appended action/status, not in-place regeneration or failed-response replacement. The user may stop the active run, which cancels runtime/model streaming and app-supervised child processes while preserving transcript, partial assistant output marked stopped or interrupted, tool history, answered approval records, and artifacts. Active runs may continue while the app window is minimized and the app process remains running. Closing or quitting the app stops active work using stop-active-run behavior and discards pending approval prompts. If the app crashes or is force-quit during an active run, the next launch marks the previous run interrupted/crashed and preserves the last persisted transcript and tool records; pending approvals are discarded and the user must retry or send a new prompt. Global search, cross-session search, full-text transcript search, tool-log search, artifact search, project-wide file content search UI, message edit/delete, transcript pruning, message redaction UI, branch-from-message, branch-from-failure, conversation rewrite, approval record export/copy/share, durable pending approvals, approve-after-restart, stale approval prompt replay, approval decision edit/delete, raw secret values in approval records, closed-app background execution, crash recovery replay, automatic process reattachment, automatic continuation after crash, unsent prompt resend, system tray daemon, background agent service, OS notifications, scheduled runs, wake/resume automation, session delete, regenerate-in-place, response replacement, partial-response editing, branch/regenerate controls, automatic continuation after stop, pause/resume stream, partial-response regeneration controls, automatic summaries, cross-session memory, durable memory writes, quotas, cleanup, and export/import are deferred. Runtime glob/grep tools may still run inside the selected project under normal file-access rules.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/mvp/mvp-scope.md`
 - `plans/spikes/SPIKE-014-memory-session-retention.md`

Next action: Document retained record classes and post-MVP delete, cleanup, quota, export/import, summary, and memory controls in the memory/session spike.

## ADR-023: No Product Telemetry In MVP

Summary: MVP sends no product telemetry and uses local-only diagnostics. OpenRouter model traffic is required product behavior and must be disclosed separately.

Status: Finalized for MVP.

Rationale: The local-first claim requires a clear privacy boundary. Local logs can support troubleshooting without automatic uploads. Provider/model calls still leave the machine through OpenRouter, so the UI must distinguish required model traffic from product telemetry.

Affected documents:

 - `plans/specs/security-specification.md`
 - `plans/acceptance/telemetry-and-diagnostics.md`
 - `plans/acceptance/openrouter-integration.md`
 - `plans/spikes/SPIKE-009-provider-privacy-cost.md`
 - `plans/spikes/SPIKE-014-memory-session-retention.md`

Next action: Verify local diagnostics redact provider credentials and known secret values; keep support bundle export out of MVP.

## ADR-022: MVP Roadmap Starts With Validation Gates

Summary: MVP planning is gated by runtime, policy, local execution, storage, provider, and UX validation.

Status: Finalized for MVP.

Rationale: Reviews identified that implementation should not begin before make-or-break assumptions are validated.

Affected documents:

 - `plans/adr/00-adr-candidate-list.md`
 - `plans/roadmap/implementation-roadmap.md`
 - `plans/mvp/mvp-validation-plan.md`
 - `plans/spikes/`

Next action: Execute research spikes before implementation plans.
