# Overview

Sessions cover the persistent conversation and tool activity context for one active agent run.

# Success Criteria

Requirement: A user can start one session in the active project.
Expected Result: The session contains transcript, runtime status, tool activity, and approval history.

Requirement: A user can resume the latest session after restart.
Expected Result: Transcript and tool activity remain available after the app closes and reopens.

# Functional Acceptance Criteria

Given an active project
When the user starts a session
Then a new conversation transcript is created and associated with that project.

Given a session has messages and tool records
When the app restarts
Then the latest session reopens with the same transcript and tool activity.

Given a session is running
When the user selects stop
Then the runtime is stopped and the session status changes to stopped.

# Security Acceptance Criteria

Given a session belongs to one project
When the user switches project context
Then the session does not retain authority to access another project root.

Given a transcript includes tool output
When the transcript is persisted
Then stored records exclude raw provider credentials.

# Performance Acceptance Criteria

Requirement: Reopening the latest session completes within 3 seconds for MVP-sized transcripts.
Expected Result: Resume feels immediate for validation users.

# User Experience Acceptance Criteria

Given a resumed session
When the user views the session
Then messages, tool activity, and current runtime state are distinguishable.

# Failure Conditions

 - Session transcript is lost after restart.
 - Tool activity is detached from the session.
 - A stopped session continues executing tools.

# Out Of Scope

 - Multiple active sessions.
 - Session archive management.
 - Cross-device sync.
 - Long-term memory.
 - Child sessions and subagents.

