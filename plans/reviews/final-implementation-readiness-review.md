# Final Implementation Readiness Review

## Reviewed Artifacts

- `plans/preplan-brief.md`
- `plans/AGENTS.md`
- `plans/research/industry-standards-research.md`
- `plans/research/open-questions-and-research-gaps.md`
- `plans/specs/requirements-specification.md`
- `plans/specs/functional-specification.md`
- `plans/specs/architecture-specification.md`
- `plans/specs/security-specification.md`
- `plans/specs/data-model-specification.md`
- `plans/specs/ux-specification.md`
- `plans/specs/plugin-system-specification.md`
- `plans/mvp/mvp-scope.md`
- `plans/mvp/mvp-user-journeys.md`
- `plans/mvp/mvp-validation-plan.md`
- `plans/mvp/mvp-success-metrics.md`
- `plans/decisions/finalized-decisions.md`
- `plans/decisions/unresolved-decisions.md`
- `plans/decisions/deferred-decisions.md`
- `plans/decisions/implementation-readiness-gaps.md`
- `plans/decisions/non-goals.md`
- `plans/adr/*.md`
- `plans/spikes/SPIKE-001-opencode-runtime-control.md`
- `plans/spikes/SPIKE-002-policy-enforcement-authority.md`
- `plans/spikes/SPIKE-003-local-execution-sandboxing.md`
- `plans/spikes/SPIKE-006-standards-conformance-matrix.md`
- `plans/spikes/SPIKE-007-tauri-electron-browser-artifacts.md`
- `plans/spikes/SPIKE-009-provider-privacy-cost.md`
- `plans/spikes/SPIKE-013-artifact-browser-preview-security.md`
- `plans/spikes/SPIKE-014-memory-session-retention.md`
- `plans/spikes/SPIKE-015-marketplace-operational-trust.md`
- `plans/acceptance/*.md`
- `plans/reviews/architecture-review.md`
- `plans/reviews/adr-architecture-review-report.md`
- `plans/reviews/risk-analysis.md`

## Executive Readiness Assessment

Not Ready.

The corpus is substantially stronger than the original brief. The MVP has been narrowed from a general-purpose AI workspace to a coding-first, single-user desktop control center for one selected local Git project and one active OpenCode-backed session. The specs consistently exclude plugins, MCP, marketplace, browser automation, rich artifacts, multiple concurrent agents, worktrees, non-Git folders, direct provider integrations, and enterprise governance from MVP.

The remaining blocker is not product scope. It is proof. The MVP safety model depends on OpenCode exposing structured events, exact runtime model/provider control, observable instruction loading, stop semantics, and reliable pre-execution interception for file writes, shell commands, and Git state changes. The spike documents define those gates but do not record completed evidence. Implementation planning should not freeze until that runtime-control evidence exists and the selected path is accepted.

## Blockers

## FINDING-001: OpenCode Runtime Control Is An Unproven MVP Gate

Severity: BLOCKER

### Evidence

`plans/specs/architecture-specification.md` says UI-only approvals, post-execution audit, terminal scraping, and best-effort observation are not acceptable enforcement mechanisms. It also states that if protected actions cannot be enforced through OpenCode directly, Phase 0 must choose a wrapper, proxy, fork, runtime replacement, or MVP scope reduction before product implementation continues.

`plans/decisions/unresolved-decisions.md` marks ADR-003 OpenCode Runtime Viability as unresolved. `plans/spikes/SPIKE-001-opencode-runtime-control.md` is an investigation plan with success criteria, not completed evidence.

### Risk

If OpenCode executes a file write, shell command, or Git state change before the backend Approval Gateway sees it, the core safety and audit model fails. If OpenCode silently loads instructions or overrides model/provider routing, the UI can misrepresent what context, model, or policy is actually active.

### Strongest Argument Against Current Plan

The plan treats OpenCode as both a reusable runtime and a subordinate component. That only works if OpenCode has the required control surface. The corpus currently asserts the required gate but has not proven the dependency can satisfy it.

### Required Resolution

Before freeze, record the OpenCode runtime-control matrix with explicit pass/fail evidence for structured events, pre-execution interception, stop behavior, app-owned session mapping, provider/model override prevention, and instruction-loading observability. If any mandatory gate fails, choose and document a wrapper, proxy, fork, runtime replacement, or MVP scope reduction.

### Affected Artifacts

- `plans/specs/architecture-specification.md`
- `plans/specs/security-specification.md`
- `plans/decisions/unresolved-decisions.md`
- `plans/adr/003-agent-runtime-strategy.md`
- `plans/adr/004-policy-enforcement-authority.md`
- `plans/spikes/SPIKE-001-opencode-runtime-control.md`
- `plans/acceptance/agent-execution.md`

## High Findings

## FINDING-002: Backend Approval Gateway Depends On Runtime Cooperation

Severity: HIGH

### Evidence

`plans/specs/security-specification.md` makes the Tauri backend the trusted enforcement boundary. `plans/specs/architecture-specification.md` makes OpenCode the execution engine but requires it not to bypass the backend Approval Gateway. `plans/spikes/SPIKE-002-policy-enforcement-authority.md` depends on SPIKE-001 findings.

### Risk

Policy enforcement can become split-brain: the runtime may believe an action is allowed while the app believes it is pending, denied, or blocked. That would undermine approvals, denial records, and user trust.

### Strongest Argument Against Current Plan

The security model is sound only if every protected execution path is mandatory through the backend. That is an integration property, not a documentation property.

### Required Resolution

Resolve policy-enforcement authority using evidence from the runtime-control spike. The final policy decision must identify bypass paths and conflict behavior.

### Affected Artifacts

- `plans/specs/security-specification.md`
- `plans/adr/004-policy-enforcement-authority.md`
- `plans/spikes/SPIKE-002-policy-enforcement-authority.md`
- `plans/acceptance/security-and-permissions.md`

## FINDING-003: Shell Safety Still Lacks Concrete Redaction And Classification Limits

Severity: HIGH

### Evidence

`plans/specs/security-specification.md` and `plans/acceptance/shell-execution.md` correctly require filtered environments, destructive command one-time approval, bounded persisted summaries, no raw stdout/stderr fallback, and no strong sandbox claims. `plans/decisions/unresolved-decisions.md` still marks shell output redaction limits unresolved. `plans/spikes/SPIKE-003-local-execution-sandboxing.md` asks for exact redaction matchers and truncation limits.

### Risk

Current-user shell execution with normal network access can leak credentials or damage local state. Without exact summary limits, secret redaction rules, and destructive classification boundaries, implementation teams may fill gaps inconsistently.

### Strongest Argument Against Current Plan

The plan accepts a high-risk shell model for MVP because it preserves local developer workflows. That tradeoff is only defensible if the remaining mitigations are precise enough to test.

### Required Resolution

Define the minimum redaction matcher set, truncation limits, safe reason labels, destructive command examples, and known unclassifiable command cases before implementation planning freezes.

### Affected Artifacts

- `plans/specs/security-specification.md`
- `plans/acceptance/shell-execution.md`
- `plans/adr/010-filesystem-and-shell-access-defaults.md`
- `plans/spikes/SPIKE-003-local-execution-sandboxing.md`

## FINDING-004: OpenRouter-Only Model Routing Needs Runtime-Level Verification

Severity: HIGH

### Evidence

`plans/specs/architecture-specification.md` fixes the selected OpenRouter model at session creation and says OpenCode is not viable if app-selected model routing can be overridden undetectably. `plans/acceptance/agent-execution.md` has failure conditions for model override and hidden retry behavior. `plans/spikes/SPIKE-009-provider-privacy-cost.md` includes runtime effective model verification as an investigation item.

### Risk

The app may show one model/provider path while the runtime sends prompts through another route, violating user expectations, provider disclosure, cost assumptions, and privacy boundaries.

### Strongest Argument Against Current Plan

The product chooses OpenRouter-only partly to reduce scope. If OpenCode config can silently override that decision, OpenRouter-only is a UI promise rather than an enforced architecture.

### Required Resolution

Record how the runtime effective model and credential reference are verified at session start and protected during active runs.

### Affected Artifacts

- `plans/specs/architecture-specification.md`
- `plans/specs/functional-specification.md`
- `plans/adr/019-model-provider-strategy.md`
- `plans/spikes/SPIKE-009-provider-privacy-cost.md`
- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/model-providers.md`

## FINDING-005: Runtime-Native Instruction Loading Can Contradict AGENTS.md Display-Only Scope

Severity: HIGH

### Evidence

`plans/specs/architecture-specification.md` says root `AGENTS.md` is display-only for MVP and may enter model context only through normal explicit file reads. The same document says direct OpenCode use is not viable if native instruction loading is invisible, undisclosed, or undisableable. SPIKE-001 includes instruction-loading verification.

### Risk

If OpenCode automatically loads root or nested instruction files, the app may accidentally provide hidden instruction injection while claiming it does not. This can affect behavior, model context, and user trust.

### Strongest Argument Against Current Plan

The MVP deliberately avoids building instruction precedence, effective instruction UI, and permission influence. Invisible runtime-native instruction behavior reintroduces that complexity through the adapter.

### Required Resolution

Document OpenCode instruction-loading behavior and decide whether it is disabled, surfaced, or disclosed. If it cannot be controlled or observed, direct OpenCode use fails the MVP gate.

### Affected Artifacts

- `plans/specs/architecture-specification.md`
- `plans/specs/functional-specification.md`
- `plans/adr/013-root-agents-md-display-only.md`
- `plans/spikes/SPIKE-001-opencode-runtime-control.md`
- `plans/acceptance/file-access.md`

## FINDING-006: Old Readiness Gap Rollup Conflicts With Later Finalized Decisions

Severity: HIGH

### Evidence

`plans/decisions/implementation-readiness-gaps.md` still lists product scope, local execution security, privacy, MCP scope, plugin trust, standards conformance, and other areas as hard blockers. Later artifacts narrow MVP scope and finalize several of those items: `plans/decisions/finalized-decisions.md`, `plans/mvp/mvp-scope.md`, and the acceptance criteria exclude MCP, plugins, marketplace, browser panels, and non-Git workflows from MVP.

### Risk

Freeze-and-plan may operate from contradictory readiness state. One document says many topics block implementation; later documents say several are resolved, deferred, or out of MVP.

### Strongest Argument Against Current Plan

A planning corpus that cannot identify its current blocker set will produce confused implementation sequencing and review criteria.

### Required Resolution

Update the readiness gap rollup to distinguish current MVP blockers from already-finalized, deferred, and post-MVP decisions.

### Affected Artifacts

- `plans/decisions/implementation-readiness-gaps.md`
- `plans/decisions/finalized-decisions.md`
- `plans/decisions/unresolved-decisions.md`
- `plans/mvp/mvp-scope.md`

## Medium Findings

## FINDING-007: Plugin And Marketplace Specs Remain Broader Than MVP

Severity: MEDIUM

### Evidence

`plans/specs/plugin-system-specification.md` and `plans/specs/marketplace-specification.md` describe future plugin and marketplace behavior. `plans/mvp/mvp-scope.md` excludes plugin installation, plugin execution, marketplace, MCP, and marketplace publishing from MVP.

### Risk

Future-facing specs can leak into MVP implementation through data-model, settings, navigation, or acceptance creep.

### Strongest Argument Against Current Plan

The planning corpus correctly defers plugins and marketplace, but still gives them enough detail to tempt premature platform work.

### Required Resolution

During freeze, mark plugin and marketplace artifacts as post-MVP reference only and prevent their requirements from entering MVP epics.

### Affected Artifacts

- `plans/specs/plugin-system-specification.md`
- `plans/specs/marketplace-specification.md`
- `plans/mvp/mvp-scope.md`
- `plans/decisions/deferred-decisions.md`

## FINDING-008: Local Indefinite Retention Has No Cleanup Or Deletion Story

Severity: MEDIUM

### Evidence

`plans/specs/data-model-specification.md` says MVP retains sessions, messages, tool calls, approvals, diagnostics, logs, and artifacts locally and indefinitely by default. Session delete, automatic cleanup, quota management, and export/import are post-MVP.

### Risk

Indefinite retention is simple but can surprise users as transcripts, generated files, logs, and artifacts accumulate. It also complicates privacy claims, even with local-only storage.

### Strongest Argument Against Current Plan

Local-first does not remove lifecycle obligations. It only moves them onto the user's machine.

### Required Resolution

Explicitly accept indefinite local retention as an MVP validation tradeoff and reflect it in UX copy and acceptance criteria. Do not imply cleanup or portability exists.

### Affected Artifacts

- `plans/specs/data-model-specification.md`
- `plans/adr/007-local-first-storage-model.md`
- `plans/acceptance/sessions.md`
- `plans/acceptance/artifacts.md`

## FINDING-009: Tauri Choice Is Reasonable But Still Needs Platform Validation

Severity: MEDIUM

### Evidence

`plans/specs/architecture-specification.md` recommends Tauri and excludes browser-heavy surfaces from MVP. `plans/decisions/finalized-decisions.md` says Tauri is finalized for MVP but calls for WebView validation. `plans/spikes/SPIKE-007-tauri-electron-browser-artifacts.md` exists for platform validation.

### Risk

System WebView differences may affect core UI, text-like previews, file rendering, focus behavior, and accessibility. If discovered late, the project may absorb avoidable platform churn.

### Strongest Argument Against Current Plan

Tauri is selected partly because the MVP excludes browser-heavy needs. That assumption must hold on target platforms before implementation sequencing hardens.

### Required Resolution

Keep Tauri as the MVP choice but require Phase 0 validation for target OS WebViews and text-like artifact previews before broad UI buildout.

### Affected Artifacts

- `plans/specs/architecture-specification.md`
- `plans/adr/002-desktop-application-shell.md`
- `plans/spikes/SPIKE-007-tauri-electron-browser-artifacts.md`

## FINDING-010: Append-Only Transcript Integrity Is Strong But UX Recovery Is Thin

Severity: MEDIUM

### Evidence

`plans/specs/functional-specification.md`, `plans/specs/data-model-specification.md`, and `plans/acceptance/agent-execution.md` exclude message edit/delete, retry-in-place, branch-from-failure, transcript pruning, and automatic retry. Provider failures and stop actions append or update status rather than rewrite history.

### Risk

This is safer and more auditable, but users may accumulate failed or partial records with little management support. MVP can accept this, but success metrics should check whether users understand the model.

### Strongest Argument Against Current Plan

Append-only history is a technical integrity choice that creates visible product rough edges. Those rough edges are acceptable only if they are intentional validation scope.

### Required Resolution

Ensure MVP UX and success metrics explicitly validate whether stopped, failed, and retried records are understandable without edit/delete/branch controls.

### Affected Artifacts

- `plans/specs/functional-specification.md`
- `plans/specs/data-model-specification.md`
- `plans/mvp/mvp-success-metrics.md`
- `plans/acceptance/sessions.md`

## FINDING-011: Acceptance Criteria Are Strong But Some Are Phase-0 Gates, Not Product QA

Severity: MEDIUM

### Evidence

`plans/acceptance/agent-execution.md`, `plans/acceptance/security-and-permissions.md`, `plans/acceptance/file-access.md`, and `plans/acceptance/shell-execution.md` include both user-visible pass/fail behavior and architecture viability conditions such as OpenCode config override prevention and pre-execution control.

### Risk

If Phase 0 gates are mixed into ordinary product QA, implementation may start before the architecture is proven, with the team expecting tests to catch integration impossibilities later.

### Strongest Argument Against Current Plan

Some acceptance criteria are not acceptance tests for a finished feature; they are go/no-go architecture proofs.

### Required Resolution

Before freeze, label Phase 0 validation gates separately from MVP feature acceptance criteria.

### Affected Artifacts

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/file-access.md`
- `plans/acceptance/shell-execution.md`
- `plans/roadmap/implementation-roadmap.md`

## Low Findings

## FINDING-012: Preplan Brief Still Says To Create Plans Even Though Corpus Exists

Severity: LOW

### Evidence

`plans/preplan-brief.md` was added from the original project brief and says to create a `plans` folder and produce twelve documents. The corpus already exists and is more detailed than the brief.

### Risk

Future readers may treat the brief as current instructions rather than historical project framing.

### Strongest Argument Against Current Plan

The brief is useful as origin context, but it now reads like an active task prompt.

### Required Resolution

Optionally add a short note that the file is the original planning brief and later documents supersede it where they are more specific.

### Affected Artifacts

- `plans/preplan-brief.md`
- `plans/AGENTS.md`

## FINDING-013: ADR Numbering And Rollup Naming Are Hard To Follow

Severity: LOW

### Evidence

The ADR folder contains candidate and priority files plus numbered ADRs with gaps. Decision rollups refer to ADR-013, ADR-017, ADR-018, ADR-020, and ADR-021, while not all appear as standalone files in the current listing.

### Risk

Navigation friction can cause missed decisions during freeze.

### Strongest Argument Against Current Plan

The planning content is detailed enough, but the index is not yet as reliable as the decisions themselves.

### Required Resolution

Add or update an ADR index that maps rollup ADR IDs to concrete files or explicitly marks rollup-only decisions.

### Affected Artifacts

- `plans/adr/00-adr-candidate-list.md`
- `plans/adr/00-proposed-adr-priority-order.md`
- `plans/decisions/finalized-decisions.md`
- `plans/decisions/deferred-decisions.md`

## Open Questions

## FINDING-014: What Is The Accepted Runtime Path If Direct OpenCode Fails?

Severity: QUESTION

### Evidence

SPIKE-001 lists wrapper, proxy, fork, runtime replacement, and MVP scope reduction as possible outcomes if OpenCode fails direct runtime gates.

### Risk

Without a preferred fallback order, a failed spike may trigger unfocused architecture debate.

### Required Resolution

Define the decision order for fallback runtime paths before or during Phase 0.

### Affected Artifacts

- `plans/spikes/SPIKE-001-opencode-runtime-control.md`
- `plans/adr/003-agent-runtime-strategy.md`

## FINDING-015: Which Target Platforms Must Pass Tauri Validation?

Severity: QUESTION

### Evidence

The specs discuss Tauri tradeoffs but do not clearly name launch platforms.

### Risk

Platform validation can be under- or over-scoped.

### Required Resolution

Name the MVP platform matrix for Phase 0 validation.

### Affected Artifacts

- `plans/specs/architecture-specification.md`
- `plans/adr/002-desktop-application-shell.md`
- `plans/spikes/SPIKE-007-tauri-electron-browser-artifacts.md`

## ADR Challenges

- ADR-001 Product Scope: strongest objection is that the original brief asks for a general-purpose workspace, while the MVP is coding-first. The corpus resolves this by narrowing MVP scope, but product messaging must not retain general-purpose MVP claims.
- ADR-002 Desktop Shell: strongest objection is that Tauri may be premature if browser or rich preview needs return. The MVP excludes those needs, so Tauri is acceptable pending platform validation.
- ADR-003 Runtime Strategy: strongest objection is that OpenCode may not expose the control plane required by the app. This remains the blocker.
- ADR-004 Policy Authority: strongest objection is that the backend cannot be authoritative unless runtime calls are interceptable. This depends on ADR-003 evidence.
- ADR-005 Standards First: strongest objection is that compatibility can become vague marketing. The MVP narrows claims, but post-MVP conformance remains unresolved.
- ADR-007 Storage: strongest objection is that local indefinite records defer cleanup, deletion, and portability. Acceptable for MVP only if disclosed.
- ADR-009 Approval Model: strongest objection is prompt fatigue and multi-step misuse. MVP accepts narrow scopes and excludes always-allow, but data-flow-aware policy remains post-MVP.
- ADR-010 Shell Defaults: strongest objection is current-user shell execution with normal network access. This remains high risk until redaction and classification limits are concrete.
- ADR-011 MCP: strongest objection is strategic delay. Exclusion is correct for MVP because runtime policy is not yet proven.
- ADR-015/016 Marketplace And Plugins: strongest objection is premature platform detail. They should remain post-MVP reference.
- ADR-019 Provider Strategy: strongest objection is OpenRouter lock-in and route opacity. Acceptable for MVP only if runtime effective model and credential references are verifiable.

## Cross-Document Inconsistencies

- `plans/decisions/implementation-readiness-gaps.md` still lists several old hard blockers that later documents resolve, narrow, or defer.
- `plans/preplan-brief.md` frames the product as general-purpose, while MVP scope and requirements correctly narrow to coding-first validation.
- Plugin and marketplace specs are detailed despite being excluded from MVP, which can confuse freeze scope if not labeled post-MVP.
- Acceptance criteria mix Phase 0 architecture gates with user-facing product acceptance.

## Security And Permissions Risks

- The backend Approval Gateway is only as strong as the runtime interception surface.
- Current-user shell execution with network access remains high risk and must not be marketed as sandboxed.
- Secret-deny, symlink containment, filtered environment, redaction, and destructive command classification need exact Phase 0 evidence.
- Runtime-native instruction loading can undermine the display-only `AGENTS.md` boundary.
- The MVP explicitly excludes data-flow-aware policy, so multi-step misuse remains a known post-MVP risk.

## Scaling And Operational Risks

- One active session keeps MVP scaling simple, but process supervision, crash marking, and stop behavior still need runtime evidence.
- Indefinite local retention can create unbounded local growth.
- SQLite and local artifact storage are reasonable for MVP but future concurrency, sync, quotas, and exports will require new decisions.
- Tauri WebView variance needs target-platform validation before UI implementation scales.

## Lock-In And Migration Risks

- OpenCode tool semantics, session state, config behavior, and instruction loading may become implicit product contracts.
- OpenRouter-only provider setup reduces MVP scope but creates provider-path lock-in unless direct provider validation is intentionally deferred and measured.
- Codex-compatible plugin direction is future-facing and should not shape MVP storage or navigation prematurely.
- Local absolute paths in records may complicate later portability or sync.

## Required Updates Before Freeze

- Complete and record the OpenCode runtime-control matrix.
- Resolve backend policy authority using runtime evidence.
- Define shell redaction, truncation, destructive classification, and unclassifiable command limits.
- Verify runtime effective model, credential reference behavior, and OpenRouter routing visibility.
- Verify runtime-native instruction loading behavior.
- Reconcile `plans/decisions/implementation-readiness-gaps.md` with finalized and deferred decisions.
- Separate Phase 0 validation gates from product acceptance criteria.
- Label plugin and marketplace documents as post-MVP reference for freeze purposes.

Freeze-and-plan should not begin while FINDING-001 remains unresolved. If FINDING-001 is resolved and the high findings are accepted, resolved, or explicitly deferred, `chrisai-planning-greenfield-freeze-and-plan` may begin.
