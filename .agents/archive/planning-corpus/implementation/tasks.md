# Implementation Tasks

## TASK-001: Scaffold Tauri App And Backend Boundary

### Epic

Epic 1: App Shell And Local Foundation.

### Objective

Create the minimal Tauri application and backend command boundary used by all
MVP features.

### Inputs

- `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
- `.agents/archive/planning-corpus/adr/002-desktop-application-shell.md`

### Deliverables

- Tauri app scaffold.
- Backend command registration pattern.
- Basic app layout shell.
- Local diagnostics initialized with no telemetry upload path.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/telemetry-and-diagnostics.md`
- `.agents/archive/planning-corpus/acceptance/settings-and-configuration.md`

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

- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/mvp/mvp-freeze.md`

### Deliverables

- SQLite migration system.
- MVP tables.
- Repository/store APIs.
- Restart persistence smoke tests.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`

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

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`

### Deliverables

- Credential store interface.
- macOS keychain implementation.
- Failure handling when credential storage is unavailable.
- Tests proving raw keys are absent from SQLite and logs.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/telemetry-and-diagnostics.md`

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

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/model-providers.md`

### Deliverables

- Provider setup UI.
- Provider readiness state.
- Selected model storage.
- Standing disclosure copy.
- Provider authentication failure state.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-004-openrouter-runtime-verification.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

### Deliverables

- Session credential-reference capture.
- Running-session key update/revoke guard.
- Tests for stable credential reference behavior.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

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

- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`

### Deliverables

- Folder selection backend contract.
- Git project validation.
- Project record persistence.
- Branch/dirty/changed-file status.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`

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

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`

### Deliverables

- Path resolver service.
- Secret-deny matcher.
- Boundary-check test fixtures.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`

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

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`

### Deliverables

- Read-only file browser.
- Root `AGENTS.md` panel.
- No project-wide content search UI.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-001-opencode-integration-surface.md`
- `.agents/archive/planning-corpus/adr/003-agent-runtime-strategy.md`

### Deliverables

- Runtime launcher.
- Session lifecycle state.
- Runtime process supervision.
- Adapter reference storage.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-001-structured-events.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-app-owned-record-mapping.md`

### Deliverables

- SSE/event client.
- Event normalization layer.
- App-owned record writers.
- Regression fixtures for observed event payloads.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-001-instruction-loading-observability.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-runtime-config-isolation.md`
- `.agents/archive/planning-corpus/validation/FINDING-014-runtime-fallback-decision.md`

### Deliverables

- Instruction-source scanner.
- Effective `/config` capture with secret redaction.
- Session disclosure record.
- Block-on-undisclosable-source behavior.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-001-stop-behavior.md`
- `.agents/archive/planning-corpus/acceptance/agent-execution.md`

### Deliverables

- Stop command path.
- Runtime abort mapping.
- App-supervised child-process termination hook.
- Interrupted/crashed restart marking.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`

### Deliverables

- Action classifier.
- Risk category model.
- Denial category model.
- Tests for protected and policy-allowed actions.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`

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

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`

### Deliverables

- Pending approval model.
- Approval UI API.
- Durable answered approval ledger records.
- Restart discard behavior for pending approvals.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-001-pre-execution-interception.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

### Deliverables

- Session allow matcher.
- Denial result mapper.
- Tool ledger denial display.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

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

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-pre-execution-interception.md`

### Deliverables

- File read executor.
- File write proposal/preview flow.
- Batch write caps.
- Denied/blocked write behavior.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

### Deliverables

- Shell executor.
- Working-directory enforcement.
- Environment keep/strip/conditional policy.
- Destructive and unclassifiable command handling.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

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

- `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

### Deliverables

- Redaction matcher set.
- Summary byte/line/line-length caps.
- Safe reason labels.
- Output-omitted marker.
- No raw stdout/stderr fallback.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

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

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

### Deliverables

- Tool activity timeline.
- Tool ledger views.
- Approval and denial result display.
- Live/ephemeral shell drawer label.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

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

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

### Deliverables

- Session screen.
- Transcript rendering.
- Runtime state indicator.
- Failure state display.
- One-active-session guard.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

### Deliverables

- Approval prompt component.
- File write preview states.
- Shell command prompt states.
- Destructive command prompt with no session allow.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`

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

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

### Deliverables

- Stop button behavior.
- Retry-as-new-action behavior.
- Restart recovery banners/states.
- Minimized-window execution messaging where applicable.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/acceptance/git-integration.md`
- `.agents/archive/planning-corpus/mvp/mvp-user-journeys.md`

### Deliverables

- Changed-file list.
- Diff viewer.
- Refresh behavior after tool execution.
- Tool-to-change context where available.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/git-integration.md`

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

- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`

### Deliverables

- Artifact record model.
- Artifact file storage layout.
- Text-like artifact viewer.
- Artifact link from session/tool context.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/artifacts.md`

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

- `.agents/archive/planning-corpus/decisions/unresolved-decisions.md`
- `.agents/archive/planning-corpus/validation/evidence-matrix.md`

### Deliverables

- Adapter tests.
- Approval and policy tests.
- Shell filtering/redaction/truncation tests.
- Credential-reference tests.
- Persistence/restart tests.

### Acceptance Criteria

- All MVP acceptance documents listed in `.agents/archive/planning-corpus/mvp/mvp-freeze.md`.

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

- `.agents/archive/planning-corpus/mvp/mvp-validation-plan.md`
- `.agents/archive/planning-corpus/mvp/mvp-user-journeys.md`
- `.agents/archive/planning-corpus/validation/FINDING-009-015-tauri-platform-matrix.md`

### Deliverables

- macOS Apple Silicon app-build validation.
- End-to-end product QA checklist.
- Text-like artifact preview report.
- Windows 11 x64 compatibility report before public MVP release.

### Acceptance Criteria

- `.agents/archive/planning-corpus/mvp/mvp-validation-plan.md`

### Dependencies

- TASK-025.

### Complexity

High.

### Completion Criteria

- MVP journeys pass on mandatory launch target.
- No public MVP release proceeds before Windows 11 x64 compatibility validation
  is complete or explicitly re-scoped.

### Verification

- Manual and automated QA results recorded in `.agents/archive/planning-corpus/validation/` or a release
  validation report.

## TASK-027: Build Project Session Catalog Foundation

### Epic

V1: Multiple Sessions Per Project.

### Objective

Provide a backend projection for listing multiple durable sessions attached to
one project while preserving the one-active-run invariant.

### Inputs

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `CONTEXT.md`

### Deliverables

- Project-scoped session list API.
- Session catalog projection with latest-session marker.
- Active-run guard carried into catalog state.
- Tests proving multiple sessions can be listed without allowing multiple
  active runs.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/specs/functional-specification.md`
- `CONTEXT.md`

### Deliverables

- Project-scoped session selection API.
- Cross-project session rejection.
- Latest-session fallback when no explicit session is selected.
- Tests for owned selection, cross-project rejection, and missing sessions.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`

### Deliverables

- Migration for session pin/archive metadata.
- Rename session title API.
- Pin/unpin session API.
- Archive/unarchive session API.
- Tests proving metadata changes do not edit/delete messages.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/sessions.md`

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

- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/specs/functional-specification.md`
- `CONTEXT.md`

### Deliverables

- Registered-project list API.
- Selected-project persistence API.
- Project selector projection with exactly one active project marker.
- Tests proving postponed project-management controls stay unavailable.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/project-management.md`

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

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/implementation-readiness-gaps.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`

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

- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/implementation/sprint-plan.md`
- `CONTEXT.md`

### Deliverables

- App status fields for project-selector availability.
- Status fields for one-active-project behavior.
- Status fields proving postponed project-management controls remain
  unavailable.
- Shell display of the selector state.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/project-management.md`

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

Implement the approved current V1 nested `AGENTS.md` slice as ordered display
guidance beyond MVP instruction-source disclosure.

### Inputs

- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/implementation-readiness-gaps.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-006-standards-conformance-matrix.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-instruction-loading-observability.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`

### Deliverables

- Product decision: current V1 supports ordered display guidance only.
- Standards conformance tier: `display_guidance_order_only`.
- Instruction preflight disclosure records an ordered root-to-nested guidance
  stack.
- Status surface reports the supported tier and no permission or automatic
  model-context effect.
- Acceptance criteria for resolution, disclosure, permissions, and out-of-scope
  behavior.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/project-management.md`
- `.agents/archive/planning-corpus/acceptance/file-access.md`

### Dependencies

- TASK-011.
- TASK-032.

### Complexity

High.

### Completion Criteria

- Nested `AGENTS.md` scope is no longer missing or ambiguous.
- AGENTS.md conformance tier is defined.
- Effective-instruction behavior is ordered-source disclosure only.
- Instruction preflight records ordered root and nested `AGENTS.md` guidance.
- Implementation does not give `AGENTS.md` permission effects or automatic
  app-owned model-context inclusion.

### Verification

- Rust instruction preflight tests.
- JS status command tests.
- Static build.

### Status

Verified.

## TASK-034: Resolve V1 Agent Skills Discovery And Invocation Scope

### Epic

V1: Explicit Skill Discovery And Invocation.

### Objective

Resolve the Agent Skills conformance and trust model before exposing any skill
catalog or invocation behavior in the app.

### Inputs

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-006-standards-conformance-matrix.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`

### Deliverables

- Product decision for the V1 skill conformance tier.
- Standards conformance tier for `SKILL.md`.
- Acceptance criteria for explicit skill discovery and invocation.
- Out-of-scope boundaries for auto-invocation, scripts, references, assets,
  permissions, and model-context effects.
- Progress item and handoff for the implementation task that follows acceptance.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/skills.md`

### Dependencies

- TASK-033.

### Complexity

High.

### Completion Criteria

- Skill scope is no longer missing or ambiguous.
- The app's supported skill behavior is explicit-only and testable.
- Skill discovery cannot imply script execution, plugin installation, broader
  prompt editing, or full Agent Skills compatibility.
- Skills do not affect permissions, approval decisions, file access, shell
  access, Git access, model routing, or automatic app-owned model context.
- Implementation does not begin until the proposed V1 tier is accepted.

### Verification

- Planning review against roadmap, deferred decisions, standards matrix,
  security specification, and skills acceptance criteria.

### Status

Accepted.

### Verification Evidence

- User accepted `explicit_discovery_and_invocation_only` on 2026-06-12.

## TASK-035: Build Explicit Project-Local Skill Discovery And Invocation

### Epic

V1: Explicit Skill Discovery And Invocation.

### Objective

Implement the accepted `explicit_discovery_and_invocation_only` Agent Skills
surface for project-local `SKILL.md` files.

### Inputs

- `.agents/archive/planning-corpus/acceptance/skills.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-006-standards-conformance-matrix.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/progress/items/item-035.md`

### Deliverables

- Backend discovery for project-local `SKILL.md` files under the selected
  project root.
- Skill metadata projection with name, description, source path, and
  unsupported-capability warnings.
- Explicit invocation path that can include selected skill text in the current
  session only through user action.
- Guardrails preventing auto-invocation, script execution, trusted
  asset/reference loading, permission effects, and global or plugin-provided
  skill discovery.
- UI/status surface for the accepted capability and unavailable behaviors.
- Tests for discovery, invocation, containment, secret-deny blocking, and
  excluded behavior.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/skills.md`

### Dependencies

- TASK-034.

### Complexity

High.

### Completion Criteria

- Project-local `SKILL.md` files are discoverable only inside the selected
  project root.
- Discovery displays bounded metadata and support status without executing or
  loading scripts, references, or assets.
- Invocation requires explicit user selection.
- Invoked skill text enters the current session only through explicit user
  action or normal project-root file-read behavior.
- Skills do not affect permissions, approvals, file access, shell access, Git
  access, model routing, or automatic app-owned model context.
- The app does not claim full Agent Skills compatibility.

### Verification

- Rust tests for discovery, containment, secret-deny blocking, and invocation
  policy.
- JS tests for capability/status projection and UI unavailable-behavior labels.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- Native Tauri build.

### Status

Verified.

## TASK-036: Resolve V1 Local Stdio MCP Scope

### Epic

V1: Local Stdio MCP Server Support.

### Objective

Resolve the V1 local stdio MCP conformance, transport, root, approval, and
trust model before exposing any MCP server configuration or launch behavior.

### Inputs

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/adr/011-mcp-integration-strategy.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-004-mcp-scope-and-threat-model.md`
- `.agents/archive/planning-corpus/acceptance/mcp-integration.md`
- Official MCP specification version `2025-11-25` at
  `https://modelcontextprotocol.io/specification/2025-11-25`

### Deliverables

- Product decision for the V1 local stdio MCP conformance tier.
- Acceptance criteria for local server registration, launch, roots, tools,
  resources, prompts, sampling, elicitation, approval routing, failure states,
  and out-of-scope behavior.
- Explicit deferral of remote MCP and automatic project-file MCP startup.
- Progress item and handoff for implementation if the proposed tier is
  accepted.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/mcp-integration.md`

### Dependencies

- TASK-035.

### Complexity

High.

### Completion Criteria

- MCP scope is no longer missing or ambiguous for current V1.
- Local stdio MCP behavior is explicit-only and testable.
- Local stdio server launch cannot bypass the Approval Gateway.
- MCP roots, tools, resources, prompts, sampling, and elicitation have
  documented allow/deny behavior.
- Remote MCP remains explicitly unavailable.
- Implementation does not begin until the proposed V1 tier is accepted.

### Verification

- Planning review against roadmap, deferred decisions, ADR-011, MCP threat
  model, official MCP spec, security specification, and MCP acceptance
  criteria.

### Status

Accepted.

### Verification Evidence

- User accepted `local_stdio_explicit_approval_only` on 2026-06-12.

## TASK-037: Build Local Stdio MCP Explicit Approval Surface

### Epic

V1: Local Stdio MCP Server Support.

### Objective

Implement the accepted `local_stdio_explicit_approval_only` MCP setup surface
for explicit local stdio server registration and approval-gated launch
proposal.

### Inputs

- `.agents/archive/planning-corpus/acceptance/mcp-integration.md`
- `.agents/archive/planning-corpus/adr/011-mcp-integration-strategy.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-004-mcp-scope-and-threat-model.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/progress/items/item-037.md`

### Deliverables

- App status fields for the accepted MCP tier and unavailable remote,
  automatic, and full-compatibility behaviors.
- Backend registration for explicit local stdio MCP server definitions.
- Backend launch proposal for local stdio MCP commands that routes through
  approval-gated local process semantics rather than starting a server.
- Root scope projection limited to the selected project root.
- Tests proving project files are not auto-imported or auto-started.
- Tests proving remote MCP, automatic resource/prompt context ingestion,
  unapproved tool invocation, sampling, and elicitation remain unavailable.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/mcp-integration.md`

### Dependencies

- TASK-036.

### Complexity

High.

### Completion Criteria

- Local stdio MCP support is explicit-only and testable.
- No MCP server starts automatically.
- No remote MCP connection is possible through the V1 surface.
- Launch requests produce an approval-gated command proposal, not process
  execution.
- Roots are limited to the selected project root.
- Tools, resources, prompts, sampling, and elicitation cannot bypass current
  app policy or model-context rules.
- The app does not claim full MCP compatibility.

### Verification

- `cargo test --manifest-path src-tauri/Cargo.toml mcp`.
- `npm test -- tests/backend-command-boundary.test.mjs tests/ui-state.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

### Status

Verified.

## TASK-038: Scope V1 Rich Artifact Preview Support

### Epic

V1: Artifact Preview Expansion.

### Objective

Choose and document the next accepted V1 artifact preview support tier before
implementation.

### Inputs

- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/archive/planning-corpus/specs/architecture-specification.md`
- `.agents/progress/items/item-032.md`

### Deliverables

- Accepted or deferred V1 support tier for richer artifact previews.
- Updated artifact acceptance criteria.
- Updated deferred-decision and standards-conformance notes.
- Follow-on implementation item only if a narrow tier is accepted.

### Acceptance Criteria

- Accepted tier:
  `.agents/archive/planning-corpus/acceptance/artifacts.md`
  `raster_image_preview_only`.

### Dependencies

- TASK-037.

### Complexity

High.

### Completion Criteria

- Candidate preview types are ranked by user value and risk.
- Active HTML and executable artifact policy is explicit.
- Renderer sandboxing and WebView/Chromium requirements are explicit.
- Browser-content ingestion and model-context behavior are explicit.
- Artifact provenance, storage, and unsupported behavior are explicit.
- Implementation remains blocked until a narrow tier is accepted.

### Proposed Tier

`raster_image_preview_only`: passive local PNG, JPEG, WebP, and GIF previews
only. SVG, HTML, PDF, documents, spreadsheets, browser rendering, remote URLs,
execution, export, duplicate workflows, search, and automatic model-context
ingestion remain out of scope.

### Verification

- Planning review confirms no active renderer, browser content ingestion, or
  Chromium dependency is silently promoted into V1.
- Acceptance criteria are explicit enough to drive tests.

### Status

Verified.

### Verification Evidence

- User accepted `raster_image_preview_only` on 2026-06-12.

## TASK-039: Build Passive Raster Image Artifact Previews

### Epic

V1: Artifact Preview Expansion.

### Objective

Implement the accepted `raster_image_preview_only` artifact preview tier.

### Inputs

- `.agents/progress/items/item-039.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`

### Deliverables

- Passive local PNG, JPEG, WebP, and GIF artifact preview support.
- App status fields for the accepted tier and unsupported preview types.
- Tests for accepted raster image preview behavior and excluded SVG, HTML, and
  remote URL inputs.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/artifacts.md`

### Dependencies

- TASK-038.

### Complexity

Medium.

### Completion Criteria

- Raster image artifact previews are passive and local.
- Active HTML, SVG, PDF, document, spreadsheet, browser, remote URL, execution,
  export, duplicate, search, OCR, image analysis, and automatic model-context
  behavior remain unavailable.
- Artifact provenance remains visible.

### Verification

- `cargo test --manifest-path src-tauri/Cargo.toml artifact`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

### Status

Verified.

## TASK-040: Scope V1 Session And Artifact Export Import

### Epic

V1: Portability And Retention Controls.

### Objective

Choose and document whether V1 should support any session or artifact
export/import behavior before implementation.

### Inputs

- `.agents/progress/items/item-032.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-008-storage-audit-portability.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
- `.agents/archive/planning-corpus/adr/005-standards-first-interoperability.md`
- `.agents/archive/planning-corpus/adr/007-local-first-storage-model.md`

### Deliverables

- Accepted or deferred V1 support tier for session and artifact export/import.
- Updated acceptance criteria for export/import behavior.
- Updated deferred-decision and compatibility notes.
- Follow-on implementation item only if a narrow tier is accepted.

### Acceptance Criteria

- `.agents/archive/planning-corpus/acceptance/export-import.md`

### Dependencies

- TASK-039.

### Complexity

High.

### Completion Criteria

- Export format is a selected-project JSON manifest.
- Absolute local paths are excluded from exported payloads.
- Secret exclusion and provider-secret exclusion are explicit.
- Artifact metadata is exported; artifact binary payloads are not.
- Raw shell stdout/stderr export remains excluded.
- Import and round-trip compatibility remain deferred.

### Verification

- `cargo test --manifest-path src-tauri/Cargo.toml export`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

### Status

Verified.

### Verification Evidence

- User accepted import deferred while export is scoped on 2026-06-12.
- Accepted tier is `project_json_export_only`.

## TASK-041: Resolve V1 Retention And Session Delete Scope

### Epic

V1: Portability And Retention Controls.

### Objective

Choose and document whether V1 should support narrow retention/delete controls
before implementation.

### Inputs

- `.agents/progress/items/item-030.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/reviews/final-implementation-readiness-review.md`

### Deliverables

- Accepted, revised, or deferred V1 retention/delete support tier.
- Updated acceptance criteria for the selected tier.
- Follow-on implementation item only if a narrow tier is accepted.

### Proposed Completion Criteria

- `archived_session_delete_only` is accepted or revised.
- Delete is limited to archived, unpinned sessions.
- Active, latest, running, pending-approval, and pinned sessions are protected.
- Automatic cleanup, quotas, message-level delete/redaction, memory, import,
  and round-trip compatibility remain out of scope unless separately accepted.

### Status

Verified.

### Verification Evidence

- User accepted `archived_session_delete_only` on 2026-06-12.
- Follow-on implementation is tracked as `TASK-042` / `item-043`.

## TASK-042: Build Archived Session Delete

### Epic

V1: Portability And Retention Controls.

### Objective

Implement the accepted `archived_session_delete_only` tier.

### Inputs

- `.agents/progress/items/item-043.md`
- `.agents/archive/planning-corpus/acceptance/retention-cleanup.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`

### Deliverables

- Storage-level archived-session delete operation.
- Tauri command boundary for explicit archived-session deletion.
- Browser-smoke command boundary and app status surface.
- Tests for successful delete and blocked protected sessions.

### Completion Criteria

- Archived, unpinned sessions can be deleted explicitly.
- Active, latest, running, pending-approval, and pinned sessions cannot be
  deleted.
- App-managed artifact files linked only to the deleted session are removed.
- Project files, provider credentials, automatic cleanup, quotas, memory,
  import, and round-trip compatibility remain out of scope.

### Verification

- `cargo test --manifest-path src-tauri/Cargo.toml archived_session`.
- `cargo test --manifest-path src-tauri/Cargo.toml deletes_archived_unpinned_session_records_only`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

### Status

Verified.

## TASK-043: Resolve V1 Long-Term Memory Scope

### Epic

V1: Portability And Retention Controls.

### Objective

Choose and document whether V1 should support any durable long-term memory
after retention/delete controls are defined.

### Inputs

- `.agents/progress/items/item-044.md`
- `.agents/archive/planning-corpus/acceptance/memory.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`

### Deliverables

- Accepted V1 memory support tier.
- App status surface that explicitly reports durable memory is unavailable.
- Tests proving the no-memory status surface.

### Completion Criteria

- `no_durable_memory_v1` is accepted.
- Cross-session memory behavior is explicit.
- Automatic summaries, embeddings, learned preferences, memory write prompts,
  and memory inspect/edit/delete UI are unavailable.
- Deleted sessions cannot leave separate app-owned memory records because V1
  does not create durable memory records.

### Verification

- `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_durable_memory_tier`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

### Status

Verified.

### Verification Evidence

- User accepted `no_durable_memory_v1` on 2026-06-12.
- App status exposes `no_durable_memory_v1` with durable memory, cross-session
  memory, learned preferences, automatic summaries, embeddings, memory write
  prompts, memory inspect/edit/delete UI, provider-side memory claims, memory
  import/export, and model-context auto-injection unavailable.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_durable_memory_tier`.
- Verification passed: `npm test -- tests/backend-command-boundary.test.mjs`.
- Verification passed: `npm test`.
- Verification passed: `npm run build`.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Verification passed: `npm run tauri -- build`.

## TASK-044: Resolve V1 Broader Compatibility Claims Scope

### Epic

V1: Standards And Compatibility Claims.

### Objective

Choose and document whether V1 should make any broader compatibility claims
after explicit skills, local stdio MCP, raster artifact previews, project JSON
export, archived-session delete, and no durable memory have been scoped.

### Inputs

- `CONTEXT.md`
- `.agents/progress/items/item-045.md`
- `.agents/archive/planning-corpus/acceptance/compatibility.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`
- `.agents/archive/planning-corpus/reviews/final-implementation-readiness-review.md`

### Deliverables

- Accepted, revised, or rejected V1 compatibility-claim tier.
- Updated acceptance criteria for allowed and forbidden compatibility claims.
- Follow-on implementation item only if product-facing status or docs need
  code/docs changes after acceptance.

### Completion Criteria

- `no_broader_compatibility_claims_v1` is accepted.
- V1 allowed compatibility claims are explicit and feature-level.
- Full AGENTS.md, Agent Skills, MCP, Codex plugin, OpenCode config, import,
  round-trip, browser, and durable-memory compatibility claims remain deferred
  unless separately accepted.

### Status

Verified.

### Verification Evidence

- User accepted `no_broader_compatibility_claims_v1` on 2026-06-12.
- App status exposes the accepted tier with feature-level claims only and full
  AGENTS.md, Agent Skills, MCP, Codex plugin, OpenCode config, import,
  round-trip, browser, and durable-memory compatibility claims unavailable.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_broader_compatibility_claims_tier`.
- Verification passed: `npm test -- tests/backend-command-boundary.test.mjs`.
- Verification passed: `npm test`.
- Verification passed: `npm run build`.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Verification passed: `npm run tauri -- build`.

## TASK-045: Resolve V1 Provider Expansion Scope

### Epic

V1: Provider Scope And Model Routing.

### Objective

Choose and document whether V1 should expand beyond the existing
OpenRouter-only provider path into direct providers, local model providers, or
fallback routing.

### Inputs

- `CONTEXT.md`
- `.agents/progress/items/item-046.md`
- `.agents/archive/planning-corpus/acceptance/model-providers.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`
- `.agents/archive/planning-corpus/acceptance/provider-expansion.md`
- `.agents/archive/planning-corpus/adr/019-model-provider-strategy.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-009-provider-privacy-cost.md`

### Deliverables

- Accepted, revised, or rejected V1 provider expansion tier.
- Updated acceptance criteria for allowed and forbidden provider paths.
- Follow-on implementation item only if V1 provider status or UI needs code
  changes after acceptance.

### Completion Criteria

- `openrouter_only_v1_no_direct_or_local_provider_expansion` is accepted.
- OpenRouter-only remains accepted.
- Direct providers, local models, offline fallback, provider fallback routing,
  automatic switching, and multi-provider settings are deferred.

### Status

Verified.

### Verification Evidence

- User accepted `openrouter_only_v1_no_direct_or_local_provider_expansion` on
  2026-06-12.
- App status exposes the accepted OpenRouter-only V1 provider expansion tier.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_openrouter_only_provider_expansion_tier`.
- Verification passed: `npm test -- tests/backend-command-boundary.test.mjs`.
- Verification passed: `npm test`.
- Verification passed: `npm run build`.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Verification passed: `npm run tauri -- build`.

## TASK-046: Resolve V1 Browser And Web Viewing Scope

### Epic

V1: Browser And Web Viewing Scope.

### Objective

Choose and document whether V1 should add browser or web-viewing features now
that raster artifact previews, provider scope, and compatibility claims have
been bounded.

### Inputs

- `CONTEXT.md`
- `.agents/progress/items/item-047.md`
- `.agents/archive/planning-corpus/acceptance/browser-web-viewing.md`
- `.agents/archive/planning-corpus/acceptance/artifacts.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-013-artifact-browser-preview-security.md`

### Deliverables

- Accepted, revised, or rejected V1 browser/web-viewing support tier.
- Updated acceptance criteria for browser surfaces, web content ingestion, and
  active rendering.
- Follow-on implementation item only if V1 app status or UI needs code changes
  after acceptance.

### Completion Criteria

- `no_browser_or_web_viewing_v1` is accepted.
- Browser panels, remote URL viewing, DOM extraction, screenshots, browser
  testing, browser automation, and browser-content ingestion are deferred.
- Chromium-backed rendering and generated HTML execution are deferred.

### Status

Verified.

### Verification Evidence

- User accepted `no_browser_or_web_viewing_v1` on 2026-06-12.
- App status exposes the accepted no-browser/no-web-viewing V1 tier.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_browser_or_web_viewing_tier`.
- Verification passed: `npm test -- tests/backend-command-boundary.test.mjs`.
- Verification passed: `npm test`.
- Verification passed: `npm run build`.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Verification passed: `npm run tauri -- build`.

## TASK-047: Resolve V1 Plugin And Marketplace Scope

### Epic

V1: Plugin And Marketplace Scope.

### Objective

Choose and document whether V1 should add plugin or marketplace workflows now
that skills, local stdio MCP, compatibility claims, provider scope, and browser
scope have been bounded.

### Inputs

- `CONTEXT.md`
- `.agents/progress/items/item-048.md`
- `.agents/archive/planning-corpus/acceptance/plugin-marketplace.md`
- `.agents/archive/planning-corpus/acceptance/plugin-system.md`
- `.agents/archive/planning-corpus/acceptance/marketplace.md`
- `.agents/archive/planning-corpus/decisions/deferred-decisions.md`
- `.agents/archive/planning-corpus/decisions/non-goals.md`

### Deliverables

- Accepted, revised, or rejected V1 plugin and marketplace support tier.
- Updated acceptance criteria for plugin execution, trust, permissions, and
  marketplace acquisition.
- Follow-on implementation item only if V1 app status or UI needs code changes
  after acceptance.

### Completion Criteria

- `no_plugin_or_marketplace_v1` is accepted.
- Plugin install, enable, execution, scripts, hooks, permissions, trusted
  assets, and plugin-provided MCP servers are explicitly accepted or deferred.
- Marketplace browsing, remote manifests, plugin search/install/update, and
  curation workflows are explicitly accepted or deferred.

### Status

Verified.

### Verification Evidence

- User accepted `no_plugin_or_marketplace_v1` on 2026-06-12.
- App status exposes the accepted no-plugin/no-marketplace V1 tier.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml app_status_reports_no_plugin_or_marketplace_tier`.
- Verification passed: `npm test -- tests/backend-command-boundary.test.mjs`.
- Verification passed: `npm test`.
- Verification passed: `npm run build`.
- Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml`.
- Verification passed: `npm run tauri -- build`.
