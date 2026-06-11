# Sprint Plan

Sprint groupings are planning conveniences. Build order should continue to
follow dependency and risk.

## Sprint 1: Architecture Spine

Goal: prove the local app foundation and highest-risk runtime/provider path
without enabling state-changing tools.

Tasks:

- TASK-001: Scaffold Tauri App And Backend Boundary.
- TASK-002: Implement SQLite Schema And Migrations.
- TASK-003: Implement Keychain Credential Store.
- TASK-004: Build OpenRouter Provider Setup.
- TASK-005: Enforce Active-Session Credential Reference Rules.
- TASK-006: Implement Project Registration And Git Status.
- TASK-007: Implement Project-Root Path Resolver.
- TASK-009: Build Hardened OpenCode Runtime Launcher.
- TASK-010: Implement Runtime Event Stream Normalizer.
- TASK-011: Implement Instruction Preflight And Disclosure.
- TASK-012: Implement Runtime Stop And Crash Interruption Mapping.

Exit criteria:

- App launches.
- Provider key is stored by reference.
- Project can be registered.
- OpenCode session can start through hardened adapter.
- Instruction sources are disclosed.
- Assistant output streams into app-owned records.
- Stop preserves records.

## Sprint 2: Policy-Controlled Local Actions

Goal: safely enable protected local actions under the backend Approval Gateway.

Tasks:

- TASK-008: Build Read-Only Project Browser And AGENTS Display.
- TASK-013: Implement Protected Action Classifier.
- TASK-014: Implement Approval Prompt State And Ledger.
- TASK-015: Implement Session Allow And Structured Denial Results.
- TASK-016: Implement File Tool Policy Execution.
- TASK-017: Implement Shell Executor And Environment Filter.
- TASK-018: Implement Shell Output Summary Policy.
- TASK-019: Implement Tool Timeline And Ledger UI.

Exit criteria:

- File writes, shell commands, and Git state changes wait for approval.
- Denied and blocked actions return structured denial.
- Shell execution uses filtered environment.
- Persisted shell summaries are bounded and safe.
- Tool activity and approval records are visible.

## Sprint 3: User Workflow And Release Gates

Goal: complete the user-facing MVP workflow and validate the app build.

Tasks:

- TASK-020: Build Session Transcript And Runtime State UI.
- TASK-021: Build Approval Prompt UI.
- TASK-022: Implement Stop, Retry, And Restart Recovery UX.
- TASK-023: Build Changed Files And Diff Viewer.
- TASK-024: Implement Text-Like Artifact Records And Viewer.
- TASK-025: Run MVP Technical Test Suite.
- TASK-026: Run App-Build And Product QA Gates.

Exit criteria:

- User can configure provider, register project, run one session, approve or
  deny actions, inspect changes, resume, and handle failure.
- MVP technical tests pass.
- macOS Apple Silicon app-build validation passes.
- Release gates are documented, including Windows 11 x64 validation before
  public MVP release.

## Sprint 6: V1 Session Catalog Foundation

Goal: start the documented V1 path for multiple sessions per project without
changing the MVP one-active-run safety invariant.

Tasks:

- TASK-027: Build Project Session Catalog Foundation.
- TASK-028: Build Project-Scoped Session Selection.

Exit criteria:

- A project can list multiple durable sessions.
- The latest session remains identifiable.
- One active run remains enforced globally.
- Session catalog records expose enough state for a future UI selector without
  adding transcript search, archive, delete, or concurrent-agent behavior.
- Selecting a session validates project ownership before opening transcript
  state.

## Sprint 7: V1 Session Metadata Management

Goal: add safe session-management metadata for the documented V1 session
workflow without introducing hard delete before retention semantics are defined.

Tasks:

- TASK-029: Build Session Rename Pin And Archive Foundation.

Exit criteria:

- Sessions can be renamed without mutating transcript records.
- Sessions can be pinned or unpinned.
- Sessions can be archived or unarchived through metadata rather than record
  deletion.
- Delete remains deferred until retention/delete semantics are explicitly
  specified.

## Sprint 8: Project Selector Foundation

Goal: close the documented project-management acceptance gap for listing
registered local Git projects and selecting exactly one active project without
promoting full project-management or worktree workflows.

Tasks:

- TASK-030: Build Project Selector State Foundation.

Exit criteria:

- Registered projects can be listed for selector UI.
- Selecting a project persists exactly one active project.
- Missing project selections fail closed.
- Selector state does not expose search, grouping, archive, delete, favorites,
  metadata editing, cross-project views, non-Git project workflows, or
  worktree management.

## Sprint 9: V1 Worktree Scope Decision

Status: complete as a defer decision.

Goal: decide whether V1 worktree creation and cleanup can be implemented after
the single-active-project and multi-session foundations, or whether the roadmap
must defer worktrees until parallel-session demand is validated.

Tasks:

- TASK-031: Resolve V1 Worktree Lifecycle Scope.

Decision:

- Worktree creation and cleanup are deferred beyond current V1.
- Current V1 continues with multiple sessions, project selection, session
  metadata, and other documented roadmap items that do not require worktree
  lifecycle behavior.
- Worktrees may be reconsidered after recurring parallel-session or
  isolated-edit demand is validated and a fresh product, architecture, and
  acceptance decision defines the lifecycle.

Exit criteria:

- Product decision defers worktrees beyond current V1.
- Roadmap, non-goals, readiness gaps, and Git acceptance agree that worktree
  management remains out of scope.
- No code implementation is required for Sprint 9.

## Sprint 10: Multi-Project Status Surface

Goal: expose the accepted project-selector capability state through the app
status surface without adding full project-management behavior.

Tasks:

- TASK-032: Expose Project Selector Capability Status.

Exit criteria:

- App status reports that the project selector can list registered projects and
  select exactly one active project.
- App status reports postponed controls as unavailable: multiple active
  projects, search, grouping, archive, delete, favorites, metadata editing,
  cross-project views, non-Git projects, and worktree management.
- The app shell surfaces the one-active-project selector state.

## Sprint 11: V1 Nested AGENTS Scope Decision

Status: complete.

Goal: implement the approved current V1 nested `AGENTS.md` slice as ordered
display guidance beyond the existing MVP instruction-source disclosure
inventory.

Tasks:

- TASK-033: Resolve V1 Nested AGENTS Resolution Scope.

Decision:

- Current V1 supports `display_guidance_order_only`.
- Root and nested `AGENTS.md` files are disclosed in an ordered guidance stack.
- Root guidance appears first, then nested files by path depth and path name.
- Nested entries disclose the path subtree they apply to.
- Conflict diagnostics are limited to source order; the app does not parse,
  semantically merge, rewrite, execute, export, or round-trip guidance.
- `AGENTS.md` files have no permission effect and are not added to app-owned
  model context unless explicitly read under normal project-root rules.

Exit criteria:

- Product decision states current V1 supports ordered display guidance only.
- Standards decision defines the supported AGENTS.md conformance tier.
- UX behavior defines ordered-source conflict diagnostics.
- Acceptance criteria exist and implementation records the ordered guidance
  stack in instruction preflight disclosure.

Verification:

- `cargo test --manifest-path src-tauri/Cargo.toml instruction_preflight`
  passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 19: V1 Retention And Session Delete Scope

Status: verified.

Goal: decide whether V1 should add narrow retention/delete controls now that
session archive, raster artifact preview, and project JSON export are verified.

Tasks:

- TASK-041: Resolve V1 Retention And Session Delete Scope.

Decision:

- `archived_session_delete_only` accepted on 2026-06-12.
- Active, latest, running, pending-approval, and pinned sessions remain
  protected.
- Automatic cleanup, quotas, message-level delete/redaction, transcript
  rewrite, long-term memory, import, and round-trip compatibility remain
  deferred.

## Sprint 20: V1 Archived Session Delete

Status: verified.

Goal: implement the accepted `archived_session_delete_only` tier.

Tasks:

- TASK-042: Build Archived Session Delete.

Exit criteria:

- Archived, unpinned sessions can be deleted explicitly.
- Protected sessions cannot be deleted.
- App-managed artifact files linked only to the deleted session are removed.
- Unsupported cleanup, memory, import, and round-trip paths remain unavailable.

Verification:

- `cargo test --manifest-path src-tauri/Cargo.toml archived_session` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml deletes_archived_unpinned_session_records_only` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 21: V1 Long-Term Memory Scope

Status: verified.

Goal: decide whether V1 should add any durable long-term memory after
retention/delete controls are defined.

Tasks:

- TASK-043: Resolve V1 Long-Term Memory Scope.

Decision:

- `no_durable_memory_v1`: no cross-session memory, learned preferences,
  automatic summaries, embeddings, memory write prompts, or memory
  inspect/edit/delete UI.
- Existing session persistence, project JSON export, and archived-session
  delete remain the only durable history/portability controls.

Verification:

- User accepted `no_durable_memory_v1` on 2026-06-12.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_durable_memory_tier` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 22: V1 Broader Compatibility Claims Scope

Status: verified.

Goal: decide whether V1 should make any broader compatibility claims now that
skills, local stdio MCP, raster artifact previews, project JSON export,
archived-session delete, and no durable memory have been scoped.

Tasks:

- TASK-044: Resolve V1 Broader Compatibility Claims Scope.

Accepted tier:

- `no_broader_compatibility_claims_v1`: keep V1 compatibility claims narrow
  and feature-level.

Exit criteria:

- User accepts the tier.
- V1 allowed compatibility claims are explicit.
- Full AGENTS.md, Agent Skills, MCP, Codex plugin, OpenCode config, import,
  round-trip, browser, and durable-memory compatibility claims remain deferred
  unless separately accepted.

Verification:

- User accepted `no_broader_compatibility_claims_v1` on 2026-06-12.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_broader_compatibility_claims_tier` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 23: V1 Provider Expansion Scope

Status: verified.

Goal: decide whether V1 should expand beyond the existing OpenRouter-only
provider path into direct providers, local model providers, or fallback
routing.

Tasks:

- TASK-045: Resolve V1 Provider Expansion Scope.

Accepted tier:

- `openrouter_only_v1_no_direct_or_local_provider_expansion`: keep V1
  OpenRouter-only without direct providers, local models, offline fallback,
  provider fallback routing, automatic switching, BYOK provider
  subconfiguration, or multi-provider settings.

Exit criteria:

- User accepts the proposed tier.
- OpenRouter-only remains accepted.
- Direct providers, local models, offline fallback, provider fallback routing,
  automatic switching, and multi-provider settings are deferred.

Verification:

- User accepted `openrouter_only_v1_no_direct_or_local_provider_expansion` on
  2026-06-12.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_openrouter_only_provider_expansion_tier` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 24: V1 Browser And Web Viewing Scope

Status: verified.

Goal: decide whether V1 should add browser or web-viewing features now that
raster artifact previews, provider scope, and compatibility claims have been
bounded.

Tasks:

- TASK-046: Resolve V1 Browser And Web Viewing Scope.

Accepted tier:

- `no_browser_or_web_viewing_v1`: keep browser panels, remote URL viewing, DOM
  extraction, screenshots, browser automation/testing, Chromium-backed
  rendering, generated HTML execution, and browser-content model-context
  ingestion deferred.

Exit criteria:

- User accepts the tier.
- Browser panels, remote URL viewing, DOM extraction, screenshots, browser
  automation/testing, and browser-content ingestion are deferred.
- Chromium-backed rendering and generated HTML execution are deferred.

Verification:

- User accepted `no_browser_or_web_viewing_v1` on 2026-06-12.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_browser_or_web_viewing_tier` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 25: V1 Plugin And Marketplace Scope

Status: verified.

Goal: decide whether V1 should add plugin or marketplace workflows now that
skills, local stdio MCP, compatibility claims, provider scope, and browser
scope have been bounded.

Tasks:

- TASK-047: Resolve V1 Plugin And Marketplace Scope.

Accepted tier:

- `no_plugin_or_marketplace_v1`: keep plugin installation, enablement,
  execution, scripts/hooks, permission grants, trusted plugin assets,
  plugin-provided MCP servers, marketplace browsing, remote marketplace
  manifests, plugin search/install/update, ratings, reviews, advisories, and
  curation workflows deferred.

Exit criteria:

- User accepts the tier.
- Plugin install, enable, execution, scripts, hooks, permissions, trusted
  assets, and plugin-provided MCP servers are explicitly accepted or deferred.
- Marketplace browsing, remote manifests, plugin search/install/update, and
  curation workflows are explicitly accepted or deferred.

Verification:

- User accepted `no_plugin_or_marketplace_v1` on 2026-06-12.
- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_plugin_or_marketplace_tier` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 12: V1 Agent Skills Scope Decision

Status: complete.

Goal: decide the narrow V1 Agent Skills conformance tier before exposing any
skill discovery or invocation behavior.

Tasks:

- TASK-034: Resolve V1 Agent Skills Discovery And Invocation Scope.

Decision:

- Current V1 may support explicit project-local skill discovery and explicit
  user-selected invocation only.
- The app may inventory `SKILL.md` metadata and display skill name,
  description, source path, and unsupported-capability warnings.
- Skills are never auto-invoked.
- Skill scripts are not executed.
- Skill references and assets are not loaded as trusted content.
- Skills have no permission effect and do not alter file, shell, Git, model,
  or approval policy.
- Skill text is not added to app-owned model context unless explicitly read
  under normal project-root file-read rules.

Exit criteria:

- Product decision states the V1 skill conformance tier.
- Standards decision defines discover/display/invoke boundaries for `SKILL.md`.
- Acceptance criteria define discovery, invocation, context, permissions,
  scripts, references, assets, failure, and out-of-scope behavior.
- Implementation may proceed only within the accepted tier.

Verification:

- User accepted `explicit_discovery_and_invocation_only` on 2026-06-12.

## Sprint 13: V1 Agent Skills Discovery And Invocation

Status: complete.

Goal: implement the accepted explicit-only project-local Agent Skills surface
without adding auto-invocation, script execution, trusted asset/reference
loading, permission effects, or full Agent Skills compatibility claims.

Tasks:

- TASK-035: Build Explicit Project-Local Skill Discovery And Invocation.

Exit criteria:

- Project-local `SKILL.md` files can be discovered and displayed with support
  warnings.
- Users must explicitly select a discovered skill before its text can be
  included in the current session.
- Discovery and invocation obey project-root and secret-deny rules.
- Skill scripts, references, assets, global catalogs, plugin-provided skills,
  and auto-invocation remain unavailable.
- Automated tests cover accepted behavior and excluded behavior.

Verification:

- `cargo test --manifest-path src-tauri/Cargo.toml skills` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml invoking_project_skill_appends_explicit_session_context`
  passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 14: V1 Local Stdio MCP Scope Decision

Status: complete.

Goal: decide the narrow V1 local stdio MCP scope before exposing any MCP server
configuration, launch, tool, resource, prompt, root, sampling, or elicitation
behavior.

Tasks:

- TASK-036: Resolve V1 Local Stdio MCP Scope.

Decision:

- Current V1 supports local stdio MCP only; remote MCP remains unavailable.
- The MCP baseline is the official Model Context Protocol specification version
  `2025-11-25`, verified on 2026-06-12.
- MCP servers are never discovered, configured, launched, or connected
  automatically from project files.
- Local stdio MCP servers may be added only through explicit user action.
- Starting a local stdio MCP server is treated as an approval-gated local shell
  execution path using the existing backend Approval Gateway, filtered
  environment, and project-root containment rules.
- MCP roots are limited to the selected project root unless a later accepted
  scope defines narrower roots.
- Remote transports, authorization flows, server-provided roots outside the
  project, automatic resource context ingestion, automatic prompt use,
  unapproved tool invocation, and sampling without explicit approval remain
  unavailable.

Exit criteria:

- Product decision states the V1 local stdio MCP conformance tier.
- Acceptance criteria define server registration, launch, roots, tools,
  resources, prompts, sampling, elicitation, approvals, failure states, and
  out-of-scope behavior.
- Implementation may proceed only within the accepted tier.

Verification:

- User accepted `local_stdio_explicit_approval_only` on 2026-06-12.

## Sprint 15: V1 Local Stdio MCP Explicit Approval Surface

Status: verified.

Goal: implement the accepted local stdio MCP setup surface without starting
servers automatically, connecting to remote endpoints, bypassing shell/process
approval, or adding full MCP compatibility claims.

Tasks:

- TASK-037: Build Local Stdio MCP Explicit Approval Surface.

Exit criteria:

- App status reports the accepted local stdio MCP tier and unavailable remote
  and automatic behaviors.
- Users can register a local stdio MCP server definition explicitly.
- Registered server launch requests produce an approval-gated local command
  proposal using selected-project root scope and filtered-environment
  disclosure.
- Project MCP config files are not auto-imported, auto-discovered, or
  auto-started.
- Resources and prompts are not added to model context automatically.
- Sampling and elicitation do not trigger automatic model calls or user-data
  disclosure.
- Automated tests cover accepted behavior and excluded behavior.

Verification:

- `cargo test --manifest-path src-tauri/Cargo.toml mcp` passed.
- `npm test -- tests/backend-command-boundary.test.mjs tests/ui-state.test.mjs`
  passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 16: V1 Rich Artifact Preview Scope

Status: verified.

Goal: choose and document whether V1 should add a narrow richer artifact
preview tier, without silently promoting active renderers, browser-content
ingestion, Chromium requirements, or automatic model-context behavior.

Tasks:

- TASK-038: Scope V1 Rich Artifact Preview Support.

Exit criteria:

- Candidate preview types are ranked by user value and risk.
- Active HTML and executable artifact policy is explicit.
- Renderer sandboxing and WebView/Chromium requirements are explicit.
- Browser-content ingestion and model-context behavior are explicit.
- Artifact provenance, storage, and unsupported behavior are explicit.
- A follow-on implementation item exists only if a narrow tier is accepted.

Proposed tier:

- `raster_image_preview_only`: passive local PNG, JPEG, WebP, and GIF previews
  only.

Verification:

- User accepted `raster_image_preview_only` on 2026-06-12.

## Sprint 17: V1 Passive Raster Image Artifact Previews

Status: verified.

Goal: implement the accepted `raster_image_preview_only` artifact preview tier
without adding active renderers, browser-content ingestion, Chromium
requirements, export/search workflows, OCR/image analysis, or automatic
model-context behavior.

Tasks:

- TASK-039: Build Passive Raster Image Artifact Previews.

Exit criteria:

- Passive local PNG, JPEG, WebP, and GIF artifact previews are supported.
- SVG, HTML, PDF, documents, spreadsheets, browser rendering, remote URL
  previews, execution, export, duplicate workflows, search, OCR, image
  analysis, and automatic model-context ingestion remain unavailable.
- App status reports the accepted tier and unsupported preview types.
- Automated tests cover accepted behavior and excluded behavior.

Verification:

- `cargo test --manifest-path src-tauri/Cargo.toml artifact` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.

## Sprint 18: V1 Session And Artifact Export Import Scope

Status: verified.

Goal: implement accepted `project_json_export_only` behavior without promoting
import, raw secrets, raw shell stdout/stderr, provider keys, absolute local
paths, artifact binary payloads, or round-trip compatibility claims.

Tasks:

- TASK-040: Scope V1 Session And Artifact Export Import.

Exit criteria:

- Export format is a selected-project JSON manifest.
- Absolute local paths are excluded from exported payloads.
- Secret exclusion and provider-secret exclusion are explicit.
- Artifact metadata is exported; artifact binary payloads are not.
- Raw shell stdout/stderr export remains excluded.
- Import and round-trip compatibility remain deferred.

Verification:

- User accepted import deferred while export is scoped on 2026-06-12.
- Accepted tier is `project_json_export_only`.
- `cargo test --manifest-path src-tauri/Cargo.toml export` passed.
- `npm test -- tests/backend-command-boundary.test.mjs` passed.
- `npm test` passed.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `npm run tauri -- build` passed.
