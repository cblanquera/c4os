# Overview

Agent Execution covers running one OpenCode-backed agent session and surfacing model and tool activity.

# Success Criteria

Requirement: The app can start one OpenCode-backed session.
Expected Result: The user sees assistant output and tool activity for the active project.

Requirement: Runtime actions are visible before and after execution.
Expected Result: File, shell, and Git actions appear in the activity timeline and ledger.

# Functional Acceptance Criteria

Given provider credentials and an active project
When the user sends a prompt
Then the runtime starts and assistant output appears in the session.

Given the runtime proposes a file write, shell command, or Git action
When the action requires approval
Then the action is not executed until the user approves it.

Given the runtime emits a tool result
When the result is received
Then the app records the tool name, status, approval decision, output summary, and affected files.

# Security Acceptance Criteria

Given a runtime action requires local access
When the action targets files, shell, or Git
Then the action is checked by the app approval policy before execution.

Given provider credentials are configured
When the runtime starts
Then credentials are not displayed in the transcript or tool activity.

# Performance Acceptance Criteria

Requirement: 95% of sessions start successfully during validation.
Expected Result: Runtime launch is reliable enough to test the product thesis.

Requirement: Tool activity appears within 1 second of event receipt.
Expected Result: Users can follow agent work in near real time.

# User Experience Acceptance Criteria

Given the agent is working
When a user views the session
Then runtime state is clear: running, waiting for approval, stopped, failed, or complete.

# Failure Conditions

 - A local action executes before required approval.
 - Runtime events are missing from the activity timeline.
 - The user cannot stop a running session.

# Out Of Scope

 - Multiple concurrent agents.
 - Runtime abstraction across multiple engines.
 - Custom agents.
 - Agent handoffs.

