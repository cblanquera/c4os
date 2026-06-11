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

Status: blocked pending standards and UX decisions.

Goal: decide whether current V1 should implement nested `AGENTS.md` resolution
beyond the existing MVP instruction-source disclosure inventory.

Tasks:

- TASK-033: Resolve V1 Nested AGENTS Resolution Scope.

Blocked by:

- `plans/decisions/deferred-decisions.md` defers nested `AGENTS.md` precedence
  and conflict diagnostics to V1.
- `plans/decisions/implementation-readiness-gaps.md` still lists effective
  instruction stack UI and whether `AGENTS.md` can influence permissions as
  missing information.
- `plans/spikes/SPIKE-006-standards-conformance-matrix.md` still requires
  conformance tiers for AGENTS.md display, parse, guidance, nested resolution,
  and export before broader compatibility claims.

Exit criteria:

- Product decision states whether nested `AGENTS.md` resolution is current V1,
  later V1, or deferred.
- Standards decision defines the supported AGENTS.md conformance tier.
- UX decision defines effective-instruction display and conflict diagnostics.
- Acceptance criteria exist before implementation starts.
