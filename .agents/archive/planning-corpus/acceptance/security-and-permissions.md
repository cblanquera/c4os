# Overview

Security and Permissions define the MVP minimum safety baseline for local project access, approvals, secrets, and trust boundaries.

# Success Criteria

Requirement: Local-risk actions are approval-gated.
Expected Result: File writes, shell commands, and Git state-changing actions require user approval.

Requirement: Git inspection is visible but not approval-gated.
Expected Result: Git status, branch display, changed-file list, and diffs are available without prompting, and runtime-performed Git inspection is logged.

Requirement: Project-root reads are visible but not approval-gated.
Expected Result: Runtime file reads inside the selected project root are logged without prompting the user for every read.

Requirement: Secrets are not stored in plaintext app metadata.
Expected Result: Provider keys are stored only in approved secret storage.

# Functional Acceptance Criteria

Given a file write, shell command, or Git state-changing action is proposed
When approval is required
Then the action waits for user decision before execution.

Given a user denies an action
When the session continues
Then the action is not executed, the denial is recorded, and the runtime receives a structured denial result.

Given the app blocks an action under MVP policy
When the runtime receives the tool result
Then the result includes a denial category and short user-safe message.

Given an action is denied or blocked
When the tool call is shown in the ledger
Then the denial category and approval decision are visible.

Given a user answers an approval prompt
When the tool call is shown in the ledger
Then structured prompt metadata, decision, timestamp, resulting action status, and bounded redacted prompt summary or diff reference are visible.

Given an approval decision is recorded
When local app records are inspected
Then raw secret values are not present in the approval record.

Given an approval decision is recorded
When local app records are inspected
Then full prompt text replay blobs and raw command output are not present in the approval record.

Given an answered approval decision exists
When the user views the activity ledger
Then the record is visible locally without export, copy-all, JSON download, support bundle, or share controls.

Given an answered approval decision is visible in the activity ledger
When the user uses normal OS text selection
Then the visible text can be copied without a dedicated approval-record copy/export control.

Given an approval prompt is pending
When the app closes, crashes, is force-quit, or restarts
Then no durable pending approval record remains approvable.

Given an action is approved for session
When the same approved action pattern recurs in the same session
Then the app applies the session-scoped approval only when it remains a matching non-destructive shell command inside the selected project root or approved project subpath.

Given an action is approved for session
When a destructive command, outside-root path, secret-deny file, or Git state-changing action is proposed
Then the app requires a fresh decision or blocks the action under MVP rules.

# Security Acceptance Criteria

Given the app stores provider credentials
When local metadata is inspected
Then raw credentials are absent.

Given a file action targets outside the project root
When the path is resolved
Then the app blocks or rejects the action under MVP policy.

Given a runtime file read targets inside the project root
When the session continues
Then the read is recorded in tool activity without requiring approval.

Given runtime Git inspection reads repository status or diffs
When the session continues
Then the inspection is recorded in tool activity without requiring approval.

Given browser, MCP, plugin, or marketplace capabilities are excluded from MVP
When the app runs
Then those capabilities cannot grant permissions or execute actions.

# Performance Acceptance Criteria

Requirement: Approval prompts appear before execution without user-noticeable delay after a tool proposal.
Expected Result: Users can respond before any protected action begins.

# User Experience Acceptance Criteria

Given an approval prompt
When displayed
Then it states the action type, target resource or command, risk category, and approval choices.

# Failure Conditions

 - Protected action executes before approval.
 - An answered approval decision is missing from the ledger.
 - An approval record stores a full prompt text replay blob.
 - An approval record stores raw command output.
 - An approval record includes raw secret values.
 - Approval records can be exported, copied in bulk, downloaded as JSON, included in a support bundle, or shared from MVP.
 - A dedicated approval-record copy button appears in MVP.
 - A pending approval persists as an approvable durable decision.
 - A recorded approval decision can be edited or deleted in MVP.
 - Denied action executes.
 - Runtime reads outside the project root.
 - Runtime reads are invisible to the user.
 - Read-only Git inspection repeatedly asks for approval.
 - Git state-changing action executes without approval.
 - Session approval covers destructive shell commands, outside-root paths, secret-deny files, or Git state changes.
 - A denied action is returned to the runtime as success.
 - A denied action fails silently without a structured denial result.
 - Provider secret appears in plaintext logs or metadata.
 - Excluded capability grants permissions.

# Out Of Scope

 - Enterprise RBAC.
 - Compliance-grade audit.
 - Data-flow-aware policy.
 - Persistent always-allow permissions.
 - Remote MCP or plugin permissions.
