# Overview

Shell Execution covers approved local commands used by the agent for coding tasks such as tests and builds.

# Success Criteria

Requirement: Shell commands require explicit user approval.
Expected Result: No shell command executes before approval.

Requirement: Shell command output is visible.
Expected Result: The user can inspect command status and summarized output.

# Functional Acceptance Criteria

Given the agent proposes a shell command
When the approval prompt appears
Then the command text and working directory are shown.

Given the user denies a shell command
When the session continues
Then the command does not execute and the denial is logged.

Given a shell command completes
When the result is displayed
Then the user sees success or failure status and output summary.

# Security Acceptance Criteria

Given a shell command is proposed
When it targets a working directory
Then the working directory is the active project root or approved project subpath.

Given environment variables include credentials
When command output is persisted
Then provider credentials are not stored in plaintext logs.

# Performance Acceptance Criteria

Requirement: Shell command status updates while the command is running.
Expected Result: The UI does not appear frozen during long-running commands.

# User Experience Acceptance Criteria

Given a command approval prompt
When displayed
Then the user can choose allow once, allow for session, or deny.

# Failure Conditions

 - A command executes without approval.
 - A denied command executes.
 - The user cannot inspect command failure status.

# Out Of Scope

 - Project-wide permanent command approvals.
 - Always-allow approvals.
 - Remote shell execution.
 - Full sandboxing guarantees beyond the MVP security baseline.

