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
