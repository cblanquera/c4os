# Overview

Shell Execution covers approved local commands used by the agent for coding tasks such as tests and builds.

# Success Criteria

Requirement: Shell commands require explicit user approval.
Expected Result: No shell command executes before approval.

Requirement: Shell commands run as the current OS user.
Expected Result: The app does not claim a stronger sandbox than the MVP provides.

Requirement: Approved shell commands may use normal network access.
Expected Result: The app does not claim network isolation, and the approval prompt makes the risk visible.

Requirement: Shell command output is visible.
Expected Result: The user can inspect command status and redacted/truncated summarized output.

Requirement: Shell output persistence is bounded.
Expected Result: The app persists redacted/truncated output summaries, not unlimited raw stdout/stderr by default.

Requirement: Unsafe shell output summaries fail closed.
Expected Result: If a safe redacted/truncated summary cannot be produced, the app persists command metadata and an explicit output-omitted marker instead of raw stdout/stderr.

Requirement: Shell summary reason labels are safe metadata.
Expected Result: Expanded shell summaries may show labels such as `truncated_by_size`, `redacted_secret_pattern`, or `output_omitted_safe_summary_failed` without exposing redacted substrings or reconstruction details.

Requirement: Visible shell output summaries can be copied through normal text selection.
Expected Result: The app allows normal OS text selection/copy of visible redacted/truncated summaries without adding dedicated raw-output copy or export controls.

Requirement: Live terminal output is not the persisted shell record.
Expected Result: Raw terminal text may be visible only while the command is running or the live terminal buffer remains open; after completion, navigation away, reload, app close, or session restore, the durable view uses only the persisted summary or output-omitted marker.

Requirement: Completed-command live drawers are temporary and labeled.
Expected Result: An open live terminal drawer may remain visible after completion in the same app session only when it is labeled live/ephemeral and not presented as persisted history.

Requirement: Shell commands use a filtered environment.
Expected Result: Approved commands do not inherit the full interactive shell environment or obvious credential variables.

Requirement: Destructive shell commands require fresh one-time approval.
Expected Result: High-risk commands are not covered by ordinary session allow.

Requirement: Session allow is narrow.
Expected Result: Session allow applies only to matching non-destructive shell commands inside the selected project root or approved project subpath.

Requirement: Recovery after approved shell commands is manual.
Expected Result: The app provides logs, status, changed files, diffs, and stop controls, but does not promise automatic rollback.

# Functional Acceptance Criteria

Given the agent proposes a shell command
When the approval prompt appears
Then the command text and working directory are shown.

Given the agent proposes an obviously destructive shell command
When ordinary shell commands have session allow
Then the app still requires a fresh one-time approval.

Given ordinary shell commands have session allow
When a matching non-destructive shell command recurs inside the selected project root or approved project subpath
Then the app may apply the session-scoped approval.

Given ordinary shell commands have session allow
When a command targets an outside-root path, a secret-deny file, a destructive pattern, or a Git state change
Then the app does not apply the shell session allow.

Given the user denies a shell command
When the session continues
Then the command does not execute, the denial is logged, and the runtime receives a structured `user_denied` result.

Given a shell command is blocked by MVP policy
When the runtime receives the result
Then the result includes a denial category such as `outside_project_root`, `secret_denied`, `destructive_requires_one_time`, or `mvp_scope_blocked`.

Given a shell command completes
When the result is displayed
Then the user sees success or failure status and a redacted/truncated output summary.

Given a shell command produces large output
When command output is persisted
Then the app stores a redacted/truncated summary sufficient for common test and build debugging.

Given a shell command produces output that cannot be safely summarized
When command output is persisted
Then the app stores command metadata, exit status, timestamps, working directory, approval decision, affected-file summary when available, and an explicit output-omitted marker.

Given a shell command produces output that cannot be safely summarized
When command output is persisted
Then raw stdout and stderr are not stored as a fallback.

Given a shell output summary was truncated, redacted, or omitted
When the user expands summary details
Then the UI shows only safe reason labels such as `truncated_by_size`, `redacted_secret_pattern`, or `output_omitted_safe_summary_failed`.

Given a shell output summary was truncated, redacted, or omitted
When the user expands summary details
Then the UI does not show redacted substrings, sensitive raw byte counts, offsets, hashes, or instructions that could reconstruct the raw output.

Given a shell command is running or its live terminal buffer remains open
When the user inspects the terminal drawer
Then live terminal output may be visible without becoming the app-owned persisted shell output record.

Given a shell command has completed and the live terminal drawer remains open
When the user inspects the drawer in the same app session
Then the drawer is labeled live/ephemeral and does not appear as persisted shell history.

Given a completed shell command is referenced by a later model call
When shell tool context is assembled
Then the model receives only the persisted redacted/truncated summary, output-omitted marker, and safe reason labels, not the live terminal buffer or reconstructable metadata.

Given a shell command has completed
When the user navigates away, reloads, closes the app, or later restores the session
Then the live terminal buffer is not retained and the shell output view uses the persisted summary or output-omitted marker.

Given a shell output summary is visible
When the user selects visible summary text manually
Then normal OS text copy works without a dedicated copy raw output or shell output export control.

Given an approved shell command causes unwanted changes
When the user reviews the session
Then the app shows command logs and changed-file/diff context for manual recovery.

# Security Acceptance Criteria

Given a shell command is proposed
When it targets a working directory
Then the working directory is the active project root or approved project subpath.

Given a shell command is approved
When it executes
Then it runs as the current OS user with the backend-filtered environment.

Given a shell command is approved
When the backend builds the command environment
Then it keeps safe basics such as `PATH`, `HOME`, `USER`, `SHELL`, `LANG`, `LC_*`, and `TERM`.

Given environment variables include likely credentials
When the backend builds the command environment
Then variables matching secret-bearing names such as `*_KEY`, `*_TOKEN`, `*_SECRET`, and `*_PASSWORD` are stripped.

Given a shell command may access the network
When the approval prompt appears
Then the user is told that approved shell commands are not network-blocked.

Given environment variables include credentials
When command output is persisted
Then provider credentials and known secret-shaped values are redacted from persisted logs.

Given a shell command produces raw stdout or stderr
When MVP persistence runs
Then unlimited raw stdout and stderr are not persisted by default.

Given raw shell stdout or stderr includes sensitive values
When command output is displayed or persisted as an MVP summary
Then known secret values and secret-shaped values are redacted before they are retained in app-owned records.

Given a command matches destructive patterns such as recursive deletion, recursive permission changes, disk writes, force Git cleanup/reset, package-manager uninstall/prune, or broad path targeting
When the approval prompt appears
Then the prompt marks the command as high-risk and offers only one-time allow or deny.

# Performance Acceptance Criteria

Requirement: Shell command status updates while the command is running.
Expected Result: The UI does not appear frozen during long-running commands.

# User Experience Acceptance Criteria

Given a command approval prompt
When displayed
Then the user can choose allow once, allow for session, or deny.

Given a command approval prompt offers allow for session
When displayed
Then it explains that session allow covers only matching non-destructive shell commands inside the selected project boundary.

Given a destructive command approval prompt
When displayed
Then the user cannot choose session allow.

Given a command approval prompt
When displayed
Then the user is told the command runs with a filtered environment.

# Failure Conditions

 - A command executes without approval.
 - A denied command executes.
 - The app claims shell commands are strongly sandboxed.
 - The app claims approved shell commands are network-isolated.
 - A command inherits the full interactive shell environment.
 - A command inherits obvious credential variables.
 - Persisted shell logs contain known secret values.
 - Unlimited raw stdout or stderr is persisted by default.
 - Live terminal buffers are retained after completion, navigation away, reload, app close, or session restore.
 - A completed-command terminal drawer remains visible without being labeled live/ephemeral.
 - Live raw terminal text, omitted raw shell output, or reconstructable shell metadata is sent into model context.
 - Raw stdout or stderr is persisted because redaction, truncation, or safe summary generation failed.
 - Shell summary metadata exposes redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction hints.
 - Raw shell output can be exported or copied through a dedicated raw-output control in MVP.
 - Output truncation hides command status or exit code.
 - A destructive command executes under ordinary session allow.
 - A destructive command prompt offers session allow.
 - Session allow covers outside-root paths, secret-deny files, destructive commands, or Git state changes.
 - A denied shell command is reported to the runtime as success.
 - A blocked shell command fails without a structured denial result.
 - The app claims it can automatically roll back arbitrary approved shell commands.
 - The user cannot inspect command failure status.

# Out Of Scope

 - Project-wide permanent command approvals.
 - Always-allow approvals.
 - Remote shell execution.
 - Full sandboxing guarantees beyond the MVP security baseline.
 - Automatic rollback, snapshots, restore points, and undo stacks.
 - Full raw shell output capture and export.
