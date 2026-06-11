# Implementation Tasks

## TASK-001: Scaffold Tauri App And Backend Boundary

### Epic

Epic 1: App Shell And Local Foundation.

### Objective

Create the minimal Tauri application and backend command boundary used by all
MVP features.

### Inputs

- `plans/mvp/mvp-freeze.md`
- `plans/adr/002-desktop-application-shell.md`

### Deliverables

- Tauri app scaffold.
- Backend command registration pattern.
- Basic app layout shell.
- Local diagnostics initialized with no telemetry upload path.

### Acceptance Criteria

- `plans/acceptance/telemetry-and-diagnostics.md`
- `plans/acceptance/settings-and-configuration.md`

### Dependencies

- None.

### Complexity

Medium.

### Completion Criteria

- App launches locally.
- Backend command can be invoked from the UI.
- No product telemetry request path exists.

### Verification

- App build/dev command succeeds.
- Unit or smoke test covers backend command invocation.

## TASK-002: Implement SQLite Schema And Migrations

### Epic

Epic 1: App Shell And Local Foundation.

### Objective

Create app-owned persistence for projects, sessions, messages, tool calls,
approvals, artifacts, models, settings, diagnostics, and adapter references.

### Inputs

- `plans/specs/data-model-specification.md`
- `plans/mvp/mvp-freeze.md`

### Deliverables

- SQLite migration system.
- MVP tables.
- Repository/store APIs.
- Restart persistence smoke tests.

### Acceptance Criteria

- `plans/acceptance/sessions.md`
- `plans/acceptance/security-and-permissions.md`

### Dependencies

- TASK-001.

### Complexity

Medium.

### Completion Criteria

- Tables initialize from a clean profile.
- Records survive app restart.
- Runtime IDs are stored only as adapter metadata.

### Verification

- Migration tests.
- Persistence restart test.

## TASK-003: Implement Keychain Credential Store

### Epic

Epic 1: App Shell And Local Foundation.

### Objective

Store OpenRouter keys only in OS keychain or platform credential storage and
persist only references in app metadata.

### Inputs

- `plans/acceptance/openrouter-integration.md`
- `plans/specs/security-specification.md`

### Deliverables

- Credential store interface.
- macOS keychain implementation.
- Failure handling when credential storage is unavailable.
- Tests proving raw keys are absent from SQLite and logs.

### Acceptance Criteria

- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/telemetry-and-diagnostics.md`

### Dependencies

- TASK-002.

### Complexity

Medium.

### Completion Criteria

- Provider key save returns a stable credential reference.
- Raw key is absent from app-owned database and diagnostics.

### Verification

- Unit tests with fake credential store.
- Local macOS keychain smoke test.

## TASK-004: Build OpenRouter Provider Setup

### Epic

Epic 3: Provider And Model Setup.

### Objective

Let users configure OpenRouter, select an MVP model, and see provider-ready
state with off-device context disclosure.

### Inputs

- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/model-providers.md`

### Deliverables

- Provider setup UI.
- Provider readiness state.
- Selected model storage.
- Standing disclosure copy.
- Provider authentication failure state.

### Acceptance Criteria

- `plans/acceptance/openrouter-integration.md`

### Dependencies

- TASK-003.

### Complexity

Medium.

### Completion Criteria

- Valid credential can produce provider-ready state.
- Invalid or storage-failed setup is actionable and fail-closed.
- Missing/stale metadata does not falsely downgrade readiness.

### Verification

- Provider setup unit tests with fake OpenRouter endpoint.
- UI smoke test for disclosure and readiness states.

## TASK-005: Enforce Active-Session Credential Reference Rules

### Epic

Epic 3: Provider And Model Setup.

### Objective

Capture the OpenRouter credential reference and model at session start, and
block key update/revoke while a session is running.

### Inputs

- `plans/validation/FINDING-004-openrouter-runtime-verification.md`
- `plans/acceptance/openrouter-integration.md`

### Deliverables

- Session credential-reference capture.
- Running-session key update/revoke guard.
- Tests for stable credential reference behavior.

### Acceptance Criteria

- `plans/acceptance/openrouter-integration.md`

### Dependencies

- TASK-004.
- TASK-002.

### Complexity

Medium.

### Completion Criteria

- Running sessions use their starting credential reference.
- Update/revoke attempts are blocked until the session stops or completes.

### Verification

- Integration test with fake session and fake credential store.

## TASK-006: Implement Project Registration And Git Status

### Epic

Epic 2: Project And Git Boundary.

### Objective

Register local Git projects and show root, branch, dirty state, and changed
files.

### Inputs

- `plans/acceptance/project-management.md`
- `plans/acceptance/git-integration.md`

### Deliverables

- Folder selection backend contract.
- Git project validation.
- Project record persistence.
- Branch/dirty/changed-file status.

### Acceptance Criteria

- `plans/acceptance/project-management.md`
- `plans/acceptance/git-integration.md`

### Dependencies

- TASK-002.

### Complexity

Medium.

### Completion Criteria

- Git projects can be registered and reopened.
- Non-Git or unreadable paths fail clearly.
- Git status for typical repos under 1 GB loads within accepted limits.

### Verification

- Unit tests with temporary Git repositories.
- UI smoke test for project registration.

## TASK-007: Implement Project-Root Path Resolver

### Epic

Epic 2: Project And Git Boundary.

### Objective

Centralize project-root containment, symlink resolution, traversal handling,
and secret-deny file blocking.

### Inputs

- `plans/acceptance/file-access.md`
- `plans/specs/security-specification.md`

### Deliverables

- Path resolver service.
- Secret-deny matcher.
- Boundary-check test fixtures.

### Acceptance Criteria

- `plans/acceptance/file-access.md`
- `plans/acceptance/security-and-permissions.md`

### Dependencies

- TASK-006.

### Complexity

High.

### Completion Criteria

- Outside-root paths are blocked.
- Symlinks are resolved before access decisions.
- Secret-deny files have no approval override.

### Verification

- Unit tests for traversal, inside symlink, outside symlink, and secret-deny
  patterns.

## TASK-008: Build Read-Only Project Browser And AGENTS Display

### Epic

Epic 2: Project And Git Boundary.

### Objective

Display project files read-only and show root `AGENTS.md` as passive app-owned
guidance.

### Inputs

- `plans/acceptance/file-access.md`
- `plans/acceptance/skills.md`

### Deliverables

- Read-only file browser.
- Root `AGENTS.md` panel.
- No project-wide content search UI.

### Acceptance Criteria

- `plans/acceptance/file-access.md`
- `plans/acceptance/skills.md`

### Dependencies

- TASK-007.

### Complexity

Medium.

### Completion Criteria

- Read-only file opens are scoped by resolver.
- Secret-deny contents are not previewed.
- Root `AGENTS.md` display does not modify model context or policy.

### Verification

- UI tests for allowed file, outside-root block, secret-deny preview block, and
  root `AGENTS.md` display.

## TASK-009: Build Hardened OpenCode Runtime Launcher

### Epic

Epic 4: Hardened OpenCode Adapter.

### Objective

Launch OpenCode sessions through app-owned config, selected model, credential
reference, and project root.

### Inputs

- `plans/validation/FINDING-001-opencode-integration-surface.md`
- `plans/adr/003-agent-runtime-strategy.md`

### Deliverables

- Runtime launcher.
- Session lifecycle state.
- Runtime process supervision.
- Adapter reference storage.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/openrouter-integration.md`

### Dependencies

- TASK-005.
- TASK-007.

### Complexity

High.

### Completion Criteria

- Runtime can start for a registered project and provider-ready session.
- Launch settings are app-owned adapter settings, not OpenCode config
  compatibility.

### Verification

- Integration test with local/fake OpenCode-compatible endpoint where possible.
- Launch failure test.

## TASK-010: Implement Runtime Event Stream Normalizer

### Epic

Epic 4: Hardened OpenCode Adapter.

### Objective

Map OpenCode structured events into app-owned session, message, model, tool,
approval, denial, error, stop, and idle records.

### Inputs

- `plans/validation/FINDING-001-structured-events.md`
- `plans/validation/FINDING-001-app-owned-record-mapping.md`

### Deliverables

- SSE/event client.
- Event normalization layer.
- App-owned record writers.
- Regression fixtures for observed event payloads.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

### Dependencies

- TASK-009.
- TASK-002.

### Complexity

High.

### Completion Criteria

- Runtime records are never the source of truth.
- Tool activity appears within one second of event receipt.

### Verification

- Fixture-based event mapping tests.
- Integration smoke test with live OpenCode when available.

## TASK-011: Implement Instruction Preflight And Disclosure

### Epic

Epic 4: Hardened OpenCode Adapter.

### Objective

Inventory runtime-native instruction sources, redact `/config`, disclose
effective sources before session start, and block undisclosable sources.

### Inputs

- `plans/validation/FINDING-001-instruction-loading-observability.md`
- `plans/validation/FINDING-001-runtime-config-isolation.md`
- `plans/validation/FINDING-014-runtime-fallback-decision.md`

### Deliverables

- Instruction-source scanner.
- Effective `/config` capture with secret redaction.
- Session disclosure record.
- Block-on-undisclosable-source behavior.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/skills.md`
- `plans/acceptance/openrouter-integration.md`

### Dependencies

- TASK-009.
- TASK-008.

### Complexity

High.

### Completion Criteria

- OpenCode-backed sessions disclose runtime-native instruction sources before
  session start.
- Sensitive `/config` values are not persisted raw.
- Session start blocks if required source inventory cannot be produced.

### Verification

- Tests with root `AGENTS.md`, nested `AGENTS.md`, project `opencode.json`, and
  redacted secret-like config values.

## TASK-012: Implement Runtime Stop And Crash Interruption Mapping

### Epic

Epic 4: Hardened OpenCode Adapter.

### Objective

Map user stop, runtime abort, process termination, app close, crash, and
force-quit into durable app-owned states.

### Inputs

- `plans/validation/FINDING-001-stop-behavior.md`
- `plans/acceptance/agent-execution.md`

### Deliverables

- Stop command path.
- Runtime abort mapping.
- App-supervised child-process termination hook.
- Interrupted/crashed restart marking.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

### Dependencies

- TASK-010.

### Complexity

High.

### Completion Criteria

- Stop preserves partial assistant output and tool records.
- Relaunch after crash marks prior active run interrupted/crashed.
- Pending approvals are not replayed.

### Verification

- Process supervision tests.
- Restart recovery tests.

## TASK-013: Implement Protected Action Classifier

### Epic

Epic 5: Approval Gateway And Policy Engine.

### Objective

Classify runtime-proposed file writes, shell commands, Git state changes,
read-only Git inspection, file reads, blocked policy actions, and MVP-scope
blocks.

### Inputs

- `plans/acceptance/security-and-permissions.md`
- `plans/validation/FINDING-003-shell-security-policy.md`

### Deliverables

- Action classifier.
- Risk category model.
- Denial category model.
- Tests for protected and policy-allowed actions.

### Acceptance Criteria

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/git-integration.md`

### Dependencies

- TASK-007.
- TASK-010.

### Complexity

High.

### Completion Criteria

- File writes, shell commands, and Git state changes require approval.
- Read-only project-root file reads and Git inspection are logged without
  approval.

### Verification

- Unit tests for each action category and boundary condition.

## TASK-014: Implement Approval Prompt State And Ledger

### Epic

Epic 5: Approval Gateway And Policy Engine.

### Objective

Create pending approval state, user decisions, durable answered approval
records, and non-durable pending approval discard behavior.

### Inputs

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/file-access.md`

### Deliverables

- Pending approval model.
- Approval UI API.
- Durable answered approval ledger records.
- Restart discard behavior for pending approvals.

### Acceptance Criteria

- `plans/acceptance/security-and-permissions.md`

### Dependencies

- TASK-013.

### Complexity

High.

### Completion Criteria

- Answered decisions are durable.
- Pending prompts are not approvable after close/crash/restart.
- Approval records do not store raw command output, raw secrets, or full prompt
  replay blobs.

### Verification

- Unit tests for approval persistence.
- Restart tests for pending prompt discard.

## TASK-015: Implement Session Allow And Structured Denial Results

### Epic

Epic 5: Approval Gateway And Policy Engine.

### Objective

Apply narrow shell session allow and return structured denials for user-denied
or policy-blocked actions.

### Inputs

- `plans/validation/FINDING-001-pre-execution-interception.md`
- `plans/acceptance/shell-execution.md`

### Deliverables

- Session allow matcher.
- Denial result mapper.
- Tool ledger denial display.

### Acceptance Criteria

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/shell-execution.md`

### Dependencies

- TASK-014.

### Complexity

High.

### Completion Criteria

- Matching non-destructive shell commands can use session allow.
- Destructive commands, outside-root targets, secret-deny files, and Git state
  changes always require fresh approval or block.
- Runtime receives structured denial, not success or silence.

### Verification

- Unit tests for session allow matching and denial categories.
- Integration test for denied file write and denied shell command.

## TASK-016: Implement File Tool Policy Execution

### Epic

Epic 6: File, Shell, And Tool Execution.

### Objective

Execute project-root file reads and approved writes under the path resolver and
Approval Gateway.

### Inputs

- `plans/acceptance/file-access.md`
- `plans/validation/FINDING-001-pre-execution-interception.md`

### Deliverables

- File read executor.
- File write proposal/preview flow.
- Batch write caps.
- Denied/blocked write behavior.

### Acceptance Criteria

- `plans/acceptance/file-access.md`
- `plans/acceptance/security-and-permissions.md`

### Dependencies

- TASK-015.
- TASK-007.

### Complexity

High.

### Completion Criteria

- Writes do not execute before approval.
- Denied writes leave target files unchanged.
- Oversized or unsafe batches are blocked as a whole.

### Verification

- Temporary filesystem integration tests.
- Batch approval tests.

## TASK-017: Implement Shell Executor And Environment Filter

### Epic

Epic 6: File, Shell, And Tool Execution.

### Objective

Run approved shell commands as the current OS user in the project boundary with
a backend-filtered environment.

### Inputs

- `plans/validation/FINDING-003-shell-security-policy.md`
- `plans/acceptance/shell-execution.md`

### Deliverables

- Shell executor.
- Working-directory enforcement.
- Environment keep/strip/conditional policy.
- Destructive and unclassifiable command handling.

### Acceptance Criteria

- `plans/acceptance/shell-execution.md`

### Dependencies

- TASK-015.

### Complexity

High.

### Completion Criteria

- Commands wait for approval.
- Credential-shaped env vars are stripped.
- Destructive commands require one-time approval and never show session allow.
- Opaque commands are conservatively high-risk or blocked.

### Verification

- Unit tests for environment filtering.
- Integration tests for approval-before-execution.
- Classification tests from shell policy examples.

## TASK-018: Implement Shell Output Summary Policy

### Epic

Epic 6: File, Shell, And Tool Execution.

### Objective

Persist bounded redacted/truncated shell summaries and omit output when a safe
summary cannot be produced.

### Inputs

- `plans/validation/FINDING-003-shell-security-policy.md`
- `plans/acceptance/shell-execution.md`

### Deliverables

- Redaction matcher set.
- Summary byte/line/line-length caps.
- Safe reason labels.
- Output-omitted marker.
- No raw stdout/stderr fallback.

### Acceptance Criteria

- `plans/acceptance/shell-execution.md`
- `plans/acceptance/openrouter-integration.md`

### Dependencies

- TASK-017.

### Complexity

High.

### Completion Criteria

- Persisted records include status, exit code, timestamps, metadata, and safe
  summary or omission marker.
- Redacted substrings, sensitive raw byte counts, offsets, hashes, and
  reconstruction metadata are not persisted or sent to model context.

### Verification

- Unit tests with known secrets, secret-shaped values, large output, binary-like
  output, and redaction failure.

## TASK-019: Implement Tool Timeline And Ledger UI

### Epic

Epic 6: File, Shell, And Tool Execution.

### Objective

Show runtime tool activity, approvals, results, affected files, denial
categories, and bounded summaries in the session timeline.

### Inputs

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/shell-execution.md`

### Deliverables

- Tool activity timeline.
- Tool ledger views.
- Approval and denial result display.
- Live/ephemeral shell drawer label.

### Acceptance Criteria

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/shell-execution.md`

### Dependencies

- TASK-010.
- TASK-014.
- TASK-018.

### Complexity

Medium.

### Completion Criteria

- Tool activity is visible from the main session workflow.
- Completed live terminal drawers are labeled live/ephemeral.
- No dedicated raw shell output export/copy control exists.

### Verification

- UI tests for pending, approved, denied, failed, and completed tool calls.

## TASK-020: Build Session Transcript And Runtime State UI

### Epic

Epic 7: Session UI And User Control.

### Objective

Render append-only transcript messages, runtime state, failure states, and one
active session flow.

### Inputs

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

### Deliverables

- Session screen.
- Transcript rendering.
- Runtime state indicator.
- Failure state display.
- One-active-session guard.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

### Dependencies

- TASK-010.
- TASK-002.

### Complexity

Medium.

### Completion Criteria

- Prior messages are not edited or deleted except runtime status updates.
- Runtime states are clear: running, waiting for approval, stopped, failed, or
  complete.

### Verification

- UI tests for message append, failure, stopped, and complete states.

## TASK-021: Build Approval Prompt UI

### Epic

Epic 7: Session UI And User Control.

### Objective

Display action type, target, risk category, preview/summary state, and allowed
approval choices.

### Inputs

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/file-access.md`
- `plans/acceptance/shell-execution.md`

### Deliverables

- Approval prompt component.
- File write preview states.
- Shell command prompt states.
- Destructive command prompt with no session allow.

### Acceptance Criteria

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/file-access.md`
- `plans/acceptance/shell-execution.md`

### Dependencies

- TASK-014.
- TASK-019.

### Complexity

Medium.

### Completion Criteria

- Prompts appear before execution.
- Users can choose allow once, allow for session when allowed, or deny.
- High-risk/destructive prompts are clearly marked.

### Verification

- UI tests for file write, batch write, shell, destructive shell, Git state
  change, deny, and policy block.

## TASK-022: Implement Stop, Retry, And Restart Recovery UX

### Epic

Epic 7: Session UI And User Control.

### Objective

Expose stop control, retry as a new appended action/status, and restart
recovery states without transcript mutation or pending approval replay.

### Inputs

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

### Deliverables

- Stop button behavior.
- Retry-as-new-action behavior.
- Restart recovery banners/states.
- Minimized-window execution messaging where applicable.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`

### Dependencies

- TASK-012.
- TASK-020.

### Complexity

Medium.

### Completion Criteria

- Stop cancels runtime/model streaming and supervised child processes.
- Retry appends; it does not replace failed messages.
- Relaunch after crash does not auto-continue, reattach, or resend prompts.

### Verification

- UI/integration tests for stop, provider failure retry, crash marker, and
  pending approval discard.

## TASK-023: Build Changed Files And Diff Viewer

### Epic

Epic 8: Review Surfaces And Artifacts.

### Objective

Show changed files and file-level diffs tied to the active project and session
activity.

### Inputs

- `plans/acceptance/git-integration.md`
- `plans/mvp/mvp-user-journeys.md`

### Deliverables

- Changed-file list.
- Diff viewer.
- Refresh behavior after tool execution.
- Tool-to-change context where available.

### Acceptance Criteria

- `plans/acceptance/git-integration.md`

### Dependencies

- TASK-006.
- TASK-019.

### Complexity

Medium.

### Completion Criteria

- Users can identify changed files and inspect diffs.
- Git inspection is logged but not approval-gated.
- Git state-changing actions still require approval.

### Verification

- Temporary Git repository tests.
- UI tests for modified, added, deleted, and untracked files where MVP supports
  display.

## TASK-024: Implement Text-Like Artifact Records And Viewer

### Epic

Epic 8: Review Surfaces And Artifacts.

### Objective

Capture and reopen MVP artifacts for text, logs, diffs, and generated source or
config files without rich rendering.

### Inputs

- `plans/acceptance/artifacts.md`
- `plans/specs/data-model-specification.md`

### Deliverables

- Artifact record model.
- Artifact file storage layout.
- Text-like artifact viewer.
- Artifact link from session/tool context.

### Acceptance Criteria

- `plans/acceptance/artifacts.md`

### Dependencies

- TASK-002.
- TASK-019.

### Complexity

Medium.

### Completion Criteria

- Text-like artifacts persist and reopen after restart.
- Rich previews, active HTML rendering, export, sharing, and execution are not
  present.

### Verification

- Persistence tests.
- UI tests for text/log/diff/generated-file artifacts.

## TASK-025: Run MVP Technical Test Suite

### Epic

Epic 9: MVP QA And Release Gates.

### Objective

Add and run the code-level tests required by deferred implementation-time
validation.

### Inputs

- `plans/decisions/unresolved-decisions.md`
- `plans/validation/evidence-matrix.md`

### Deliverables

- Adapter tests.
- Approval and policy tests.
- Shell filtering/redaction/truncation tests.
- Credential-reference tests.
- Persistence/restart tests.

### Acceptance Criteria

- All MVP acceptance documents listed in `plans/mvp/mvp-freeze.md`.

### Dependencies

- TASK-001 through TASK-024.

### Complexity

High.

### Completion Criteria

- Deferred implementation-time validation items are covered by tests.
- Regression tests exist for Phase 0 probe findings.

### Verification

- Full automated test suite passes locally.

## TASK-026: Run App-Build And Product QA Gates

### Epic

Epic 9: MVP QA And Release Gates.

### Objective

Validate the built app against MVP user journeys, Tauri UI behavior, text-like
artifact previews, and release platform expectations.

### Inputs

- `plans/mvp/mvp-validation-plan.md`
- `plans/mvp/mvp-user-journeys.md`
- `plans/validation/FINDING-009-015-tauri-platform-matrix.md`

### Deliverables

- macOS Apple Silicon app-build validation.
- End-to-end product QA checklist.
- Text-like artifact preview report.
- Windows 11 x64 compatibility report before public MVP release.

### Acceptance Criteria

- `plans/mvp/mvp-validation-plan.md`

### Dependencies

- TASK-025.

### Complexity

High.

### Completion Criteria

- MVP journeys pass on mandatory launch target.
- No public MVP release proceeds before Windows 11 x64 compatibility validation
  is complete or explicitly re-scoped.

### Verification

- Manual and automated QA results recorded in `plans/validation/` or a release
  validation report.

## TASK-027: Build Project Session Catalog Foundation

### Epic

V1: Multiple Sessions Per Project.

### Objective

Provide a backend projection for listing multiple durable sessions attached to
one project while preserving the one-active-run invariant.

### Inputs

- `plans/roadmap/implementation-roadmap.md`
- `plans/acceptance/sessions.md`
- `CONTEXT.md`

### Deliverables

- Project-scoped session list API.
- Session catalog projection with latest-session marker.
- Active-run guard carried into catalog state.
- Tests proving multiple sessions can be listed without allowing multiple
  active runs.

### Acceptance Criteria

- `plans/acceptance/sessions.md`

### Dependencies

- TASK-020.
- TASK-025.

### Complexity

Medium.

### Completion Criteria

- Multiple sessions for one project are returned in newest-first order.
- Exactly one latest session is identifiable when sessions exist.
- A running or waiting session blocks starting another active run.
- The catalog does not introduce search, archive, delete, or concurrent active
  session behavior.

### Verification

- Unit tests for multi-session listing, latest marker, active-run guard, and
  empty project state.

## TASK-028: Build Project-Scoped Session Selection

### Epic

V1: Multiple Sessions Per Project.

### Objective

Allow a caller to open a selected session only when it belongs to the selected
project, returning transcript state through the existing session screen
projection.

### Inputs

- `plans/acceptance/sessions.md`
- `plans/specs/functional-specification.md`
- `CONTEXT.md`

### Deliverables

- Project-scoped session selection API.
- Cross-project session rejection.
- Latest-session fallback when no explicit session is selected.
- Tests for owned selection, cross-project rejection, and missing sessions.

### Acceptance Criteria

- `plans/acceptance/sessions.md`

### Dependencies

- TASK-027.
- TASK-020.

### Complexity

Medium.

### Completion Criteria

- Opening a session validates the session's project ownership.
- Missing sessions fail closed.
- Omitted session selection opens the latest session for the selected project.
- Selection does not create archive, delete, search, or concurrent session
  behavior.

### Verification

- Unit tests for selected session ownership, latest fallback, missing session,
  and cross-project rejection.

## TASK-029: Build Session Rename Pin And Archive Foundation

### Epic

V1: Session Rename, Archive, Pin, And Delete.

### Objective

Add durable session metadata controls for rename, pin, and archive while
preserving transcript append-only semantics and deferring hard delete until
retention/delete semantics are specified.

### Inputs

- `plans/roadmap/implementation-roadmap.md`
- `plans/specs/data-model-specification.md`
- `plans/decisions/non-goals.md`

### Deliverables

- Migration for session pin/archive metadata.
- Rename session title API.
- Pin/unpin session API.
- Archive/unarchive session API.
- Tests proving metadata changes do not edit/delete messages.

### Acceptance Criteria

- `plans/acceptance/sessions.md`

### Dependencies

- TASK-027.

### Complexity

Medium.

### Completion Criteria

- Existing databases migrate to session pin/archive metadata.
- Session title updates change only session metadata.
- Pin/archive updates change only session metadata.
- Message append-only triggers remain intact.
- Hard delete is not implemented in this task.

### Verification

- Migration tests.
- Rename/pin/archive tests.
- Regression test that message records remain append-only after session
  metadata updates.

## TASK-030: Build Project Selector State Foundation

### Epic

Epic 1: Local Foundation.

### Objective

Provide a backend projection for listing registered local Git projects and
persisting exactly one selected active project.

### Inputs

- `plans/acceptance/project-management.md`
- `plans/specs/functional-specification.md`
- `CONTEXT.md`

### Deliverables

- Registered-project list API.
- Selected-project persistence API.
- Project selector projection with exactly one active project marker.
- Tests proving postponed project-management controls stay unavailable.

### Acceptance Criteria

- `plans/acceptance/project-management.md`

### Dependencies

- TASK-006.
- TASK-025.

### Complexity

Medium.

### Completion Criteria

- Registered projects are returned for selector UI.
- Selecting a known project marks exactly one active project.
- Missing project selection fails closed.
- Selection does not introduce multiple active projects, project search,
  grouping, archive, delete, favorites, metadata editing, cross-project views,
  non-Git project workflows, or worktree management.

### Verification

- Unit tests for registered-project listing, selected-project persistence,
  missing-project rejection, and excluded project-management controls.

## TASK-031: Resolve V1 Worktree Lifecycle Scope

### Epic

V1: Worktree Creation And Cleanup.

### Objective

Resolve the missing product, architecture, cleanup, and acceptance decisions
before implementing Git worktree creation or cleanup.

### Inputs

- `plans/roadmap/implementation-roadmap.md`
- `plans/decisions/deferred-decisions.md`
- `plans/decisions/implementation-readiness-gaps.md`
- `plans/decisions/non-goals.md`
- `plans/acceptance/git-integration.md`

### Deliverables

- Product decision deferring worktrees beyond current V1.
- Roadmap update moving worktree creation and cleanup out of current V1.
- Decision and acceptance updates confirming worktree management remains out of
  scope.

### Acceptance Criteria

No code implementation proceeds for worktrees in current V1. Git acceptance
criteria explicitly keep worktree management out of scope until a future product
and architecture decision defines lifecycle behavior.

### Dependencies

- TASK-027.
- TASK-028.
- TASK-030.

### Complexity

High.

### Completion Criteria

- Worktree lifecycle scope is no longer contradictory or missing.
- Cleanup and dirty-state behavior are deferred beyond current V1 instead of
  inferred by implementation.
- Edge cases for branch reuse, submodules, Git LFS, nested repos, and failed
  cleanup are explicitly out of scope until a future worktree lifecycle
  decision.
- Current V1 implementation can proceed without inventing worktree product
  behavior from code.

### Verification

- Planning review against roadmap, decisions, non-goals, and Git acceptance.

### Status

Complete as defer decision.

## TASK-032: Expose Project Selector Capability Status

### Epic

V1: Polished Multi-Project Workflows.

### Objective

Expose project-selector capability state through the app status surface so the
UI can distinguish the accepted one-active-project workflow from postponed full
project-management behavior.

### Inputs

- `plans/acceptance/project-management.md`
- `plans/implementation/sprint-plan.md`
- `CONTEXT.md`

### Deliverables

- App status fields for project-selector availability.
- Status fields for one-active-project behavior.
- Status fields proving postponed project-management controls remain
  unavailable.
- Shell display of the selector state.

### Acceptance Criteria

- `plans/acceptance/project-management.md`

### Dependencies

- TASK-030.
- TASK-031.

### Complexity

Low.

### Completion Criteria

- Status reports registered-project listing and exactly-one-active-project
  selection as available.
- Status reports multiple active projects, search, grouping, archive, delete,
  favorites, metadata editing, cross-project views, non-Git projects, and
  worktree management as unavailable.
- App shell shows the selector state without adding full project-management UI.

### Verification

- JS status command tests.
- Static build.

## TASK-033: Resolve V1 Nested AGENTS Resolution Scope

### Epic

V1: Nested AGENTS.md Resolution.

### Objective

Resolve the standards, product, permission, and UX decisions required before
implementing nested `AGENTS.md` resolution beyond MVP instruction-source
disclosure.

### Inputs

- `plans/decisions/deferred-decisions.md`
- `plans/decisions/implementation-readiness-gaps.md`
- `plans/spikes/SPIKE-006-standards-conformance-matrix.md`
- `plans/validation/FINDING-001-instruction-loading-observability.md`
- `plans/acceptance/file-access.md`

### Deliverables

- Product decision for whether nested `AGENTS.md` resolution is current V1,
  later V1, or deferred.
- Standards conformance decision for AGENTS.md support tier.
- UX decision for effective-instruction display and conflict diagnostics.
- Acceptance criteria for resolution, precedence, disclosure, permissions, and
  out-of-scope behavior.

### Acceptance Criteria

New or updated nested `AGENTS.md` acceptance criteria are required before code
implementation.

### Dependencies

- TASK-011.
- TASK-032.

### Complexity

High.

### Completion Criteria

- Nested `AGENTS.md` scope is no longer missing or ambiguous.
- AGENTS.md conformance tier is defined.
- Effective-instruction UI and conflict diagnostics are defined.
- Implementation can proceed without inventing instruction precedence or
  permission behavior from code.

### Verification

- Planning review against deferred decisions, readiness gaps, standards
  conformance spike, instruction-loading validation, and file-access
  acceptance.

### Status

Blocked pending standards and UX decisions.
