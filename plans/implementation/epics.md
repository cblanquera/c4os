# Implementation Epics

## Epic 1: App Shell And Local Foundation

### MVP Requirements

- Tauri desktop shell.
- Local backend boundary.
- App-owned SQLite persistence.
- OS keychain or platform credential access.
- No product telemetry.

### Acceptance Criteria

- `plans/acceptance/settings-and-configuration.md`
- `plans/acceptance/telemetry-and-diagnostics.md`
- `plans/acceptance/sessions.md`

### Dependencies

- None.

### Deliverables

- Minimal Tauri app.
- Backend command boundary.
- SQLite schema and migrations.
- Local diagnostics store with redaction rules.
- Keychain abstraction.

### Risks

- Platform credential APIs vary.
- Schema shortcuts can make later session/tool records hard to preserve.

### Completion Criteria

- App launches on macOS Apple Silicon.
- Database initializes and survives restart.
- Raw provider keys are not stored in SQLite or logs.
- No product telemetry requests exist.

## Epic 2: Project And Git Boundary

### MVP Requirements

- Register local Git projects.
- One active project at a time.
- Show project root, branch, dirty state, changed files, diffs, and root
  `AGENTS.md` display.
- Read-only file browser scoped to project root.

### Acceptance Criteria

- `plans/acceptance/project-management.md`
- `plans/acceptance/file-access.md`
- `plans/acceptance/git-integration.md`
- `plans/acceptance/skills.md`

### Dependencies

- Epic 1.

### Deliverables

- Project registration flow.
- Git detection/status/diff backend.
- Project-root path resolver.
- Read-only file browser.
- Root `AGENTS.md` display.

### Risks

- Symlink and traversal handling must be correct before runtime tools use it.
- Git status performance may degrade on large repos.

### Completion Criteria

- Non-Git folders are rejected or clearly not ready.
- Outside-root and secret-deny reads are blocked.
- Git status and diffs are visible.

## Epic 3: Provider And Model Setup

### MVP Requirements

- OpenRouter-only provider path.
- Keychain-backed credential storage.
- One selected model fixed per session.
- Standing disclosure for off-device model calls.
- Active-session credential update/revoke blocking.

### Acceptance Criteria

- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/model-providers.md`
- `plans/acceptance/telemetry-and-diagnostics.md`

### Dependencies

- Epic 1.

### Deliverables

- Provider setup UI.
- Keychain reference persistence.
- Provider readiness state.
- Model selection state.
- Active-session credential lock rules.

### Risks

- Provider metadata can be stale or unavailable.
- Credential handling errors are high impact.

### Completion Criteria

- Provider-ready state works without storing raw keys in app metadata.
- Active sessions keep their starting credential reference.
- Provider errors fail closed with session state preserved.

## Epic 4: Hardened OpenCode Adapter

### MVP Requirements

- OpenCode-backed runtime execution.
- Structured event ingestion.
- Permission ask/reply routing.
- Stop/abort mapping.
- App-owned runtime reference persistence.
- Instruction-source inventory and disclosure.
- Sensitive `/config` redaction.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/skills.md`

### Dependencies

- Epic 1.
- Epic 3.

### Deliverables

- Runtime launcher.
- Event stream client.
- Adapter event normalizer.
- Runtime config capture/redaction.
- Instruction preflight scanner.
- Session-start disclosure record.
- Stop handler.

### Risks

- OpenCode API behavior can drift.
- Runtime-native instruction loading is not disabled, only disclosed.

### Completion Criteria

- Session start creates app-owned session records.
- Assistant, model, tool, approval, denial, error, stop, and idle events map to
  app-owned records.
- Session start blocks if instruction-bearing sources cannot be enumerated or
  disclosed.

## Epic 5: Approval Gateway And Policy Engine

### MVP Requirements

- Backend Approval Gateway for file writes, shell commands, and
  runtime-proposed Git state changes.
- One-time allow, narrow shell session allow, and deny.
- Structured denial results.
- Durable answered approval ledger.
- Pending approvals are discarded on close/crash/restart.

### Acceptance Criteria

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/file-access.md`
- `plans/acceptance/shell-execution.md`
- `plans/acceptance/git-integration.md`

### Dependencies

- Epic 2.
- Epic 4.

### Deliverables

- Protected-action classifier.
- Approval prompt API/state.
- Decision ledger records.
- Session allow matcher.
- Structured denial mapper.

### Risks

- Any protected action executing before approval fails the MVP safety model.
- Session allow can become too broad if command matching is loose.

### Completion Criteria

- Protected actions wait before execution.
- Denials and policy blocks are visible in the ledger and returned to runtime.
- Session allow never covers destructive commands, outside-root paths,
  secret-deny files, or Git state changes.

## Epic 6: File, Shell, And Tool Execution

### MVP Requirements

- Project-root file reads and approved writes.
- Shell commands run as current OS user after approval with filtered
  environment.
- Bounded shell summaries and safe omission.
- Tool activity timeline and durable tool ledger.

### Acceptance Criteria

- `plans/acceptance/file-access.md`
- `plans/acceptance/shell-execution.md`
- `plans/acceptance/security-and-permissions.md`

### Dependencies

- Epic 2.
- Epic 5.

### Deliverables

- File tool policy executor.
- Shell executor and process supervisor.
- Environment filter.
- Shell output redaction/truncation/omission.
- Tool call storage and timeline API.

### Risks

- Shell redaction cannot be perfect, so failure must omit output.
- Long-running child processes must stop reliably.

### Completion Criteria

- File writes cannot occur without approval.
- Shell commands cannot execute without approval.
- Persisted shell records never fall back to raw stdout/stderr.
- Stop terminates app-supervised command processes.

## Epic 7: Session UI And User Control

### MVP Requirements

- Conversation transcript.
- One active session.
- Runtime state display.
- Approval prompts.
- Stop, retry-as-new-action, failure states.
- Minimized-window execution while app process remains running.

### Acceptance Criteria

- `plans/acceptance/agent-execution.md`
- `plans/acceptance/sessions.md`
- `plans/acceptance/security-and-permissions.md`

### Dependencies

- Epic 4.
- Epic 5.
- Epic 6.

### Deliverables

- Session screen.
- Transcript rendering.
- Runtime state indicators.
- Approval prompt UI.
- Stop and retry controls.
- Crash/restart interrupted-state handling.

### Risks

- UI can accidentally imply pending approvals are durable.
- Retry must not mutate prior transcript records.

### Completion Criteria

- User can run a small coding task and see assistant/tool activity.
- Stop preserves partial records.
- Restart restores latest session without replaying pending approvals.

## Epic 8: Review Surfaces And Artifacts

### MVP Requirements

- Changed-file list.
- Git diff viewer.
- Tool-to-change context.
- Basic text/log/diff/generated-file artifact records.

### Acceptance Criteria

- `plans/acceptance/git-integration.md`
- `plans/acceptance/artifacts.md`
- `plans/acceptance/sessions.md`

### Dependencies

- Epic 2.
- Epic 6.
- Epic 7.

### Deliverables

- Changed-file panel.
- Diff viewer.
- Artifact record capture.
- Text-like artifact viewer.

### Risks

- Rich preview pressure can leak post-MVP scope into MVP.
- Diff state must reflect the actual filesystem.

### Completion Criteria

- Users can identify changed files and inspect file-level diffs.
- Text-like artifacts persist and reopen after restart.

## Epic 9: MVP QA And Release Gates

### MVP Requirements

- App-build validation.
- Code-level adapter, provider, shell, approval, persistence, and disclosure
  tests.
- End-to-end product QA journeys.
- Windows validation before public MVP release.

### Acceptance Criteria

- `plans/mvp/mvp-validation-plan.md`
- All MVP acceptance documents listed in `plans/mvp/mvp-freeze.md`.

### Dependencies

- Epics 1 through 8.

### Deliverables

- Automated test suite.
- macOS Apple Silicon app validation.
- Product QA checklist execution.
- Windows 11 x64 compatibility report before public MVP release.

### Risks

- Tauri UI validation may reveal platform-specific layout or WebView issues.
- Missing adapter regression tests can hide runtime API drift.

### Completion Criteria

- MVP journeys pass on the mandatory launch target.
- Deferred validation gates from `plans/decisions/unresolved-decisions.md` are
  closed or explicitly deferred from public release.
