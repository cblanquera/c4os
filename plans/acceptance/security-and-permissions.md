# Overview

Security and Permissions define the MVP minimum safety baseline for local project access, approvals, secrets, and trust boundaries.

# Success Criteria

Requirement: Local-risk actions are approval-gated.
Expected Result: File writes, shell commands, and Git state-changing actions require user approval.

Requirement: Secrets are not stored in plaintext app metadata.
Expected Result: Provider keys are stored only in approved secret storage.

# Functional Acceptance Criteria

Given a file write, shell command, or Git state-changing action is proposed
When approval is required
Then the action waits for user decision before execution.

Given a user denies an action
When the session continues
Then the action is not executed and the denial is recorded.

Given an action is approved for session
When the same approved action pattern recurs in the same session
Then the app applies only the session-scoped approval.

# Security Acceptance Criteria

Given the app stores provider credentials
When local metadata is inspected
Then raw credentials are absent.

Given a file action targets outside the project root
When the path is resolved
Then the app blocks or rejects the action under MVP policy.

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
 - Denied action executes.
 - Provider secret appears in plaintext logs or metadata.
 - Excluded capability grants permissions.

# Out Of Scope

 - Enterprise RBAC.
 - Compliance-grade audit.
 - Data-flow-aware policy.
 - Persistent always-allow permissions.
 - Remote MCP or plugin permissions.

