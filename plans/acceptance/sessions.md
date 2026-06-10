# Overview

Sessions cover the persistent conversation and tool activity context for one active agent run.

# Success Criteria

Requirement: A user can start one session in the active project.
Expected Result: The session contains transcript, runtime status, tool activity, and approval history.

Requirement: A user can resume the latest session after restart.
Expected Result: Transcript and tool activity remain available after the app closes and reopens.

Requirement: MVP retains session records locally by default.
Expected Result: Session transcript, tool activity, approvals, local diagnostics, logs, and MVP artifacts remain on device indefinitely.

Requirement: MVP transcripts are append-only.
Expected Result: User and assistant messages cannot be edited or deleted after creation; runtime-generated status may update.

Requirement: MVP does not include transcript search.
Expected Result: Users can view the latest session transcript and activity, but cannot run full-text or cross-session transcript search.

Requirement: The user can stop the active run.
Expected Result: Active runtime/model streaming and app-supervised child processes are canceled while session records are preserved.

Requirement: Minimized app windows may continue active runs.
Expected Result: If the app process remains running, minimized execution continues and status/stop controls are visible when restored.

Requirement: Closing the app stops active runs.
Expected Result: Closing or quitting cancels active runtime/model streaming, preserves session history, and allows retry after reopening.

Requirement: Crash or force-quit recovery is explicit.
Expected Result: On next launch after a crash or force-quit during an active run, the previous run is marked interrupted/crashed, last persisted records remain available, and no automatic continuation occurs.

# Functional Acceptance Criteria

Given an active project
When the user starts a session
Then a new conversation transcript is created and associated with that project.

Given a session has messages and tool records
When the app restarts
Then the latest session reopens with the same transcript and tool activity.

Given a session has retained records
When the app runs in MVP mode
Then the app does not automatically clean up the session, tool records, logs, or MVP artifacts.

Given a session has existing user or assistant messages
When the user views the transcript
Then no edit, delete, prune, redact, or branch-from-message controls are available.

Given a session has transcript history and no active run
When the user changes the project default model
Then the existing session keeps the model selected at session creation.

Given a session is running
When the user changes the project default model
Then the running session continues with the model selected at session creation.

Given a session has transcript and tool activity
When the user views MVP session UI
Then no full-text transcript search, cross-session search, or tool-log search controls are available.

Given a user wants to correct a prior message
When the session is still usable
Then the user can send a follow-up message or start a new session.

Given a submitted user message encounters a provider or network failure
When the session returns to the user
Then the submitted message remains appended and the assistant/run record shows a failed state.

Given a provider or network failure is recoverable
When the user wants to continue
Then the user must explicitly retry or send a follow-up message.

Given a failed assistant/run record exists
When the user selects retry
Then the retry is recorded as a new appended action/status before a new model call is made.

Given a failed assistant/run record exists
When the user selects retry
Then the failed assistant/run record is not replaced, regenerated in place, deleted, or branched.

Given a session is running
When the user selects stop
Then the active runtime/model stream is canceled, app-supervised child processes are terminated, and the run status changes to stopped.

Given a session is running
When the app window is minimized
Then the active run may continue while the app process remains running.

Given a minimized app has a running session
When the user restores the app window
Then current runtime state and stop controls are visible.

Given a session is running
When the user closes or quits the app
Then the active run is stopped using normal stop-active-run behavior before shutdown.

Given the app is reopened after a close stopped a run
When the latest session is resumed
Then transcript, tool history, approvals, artifacts, and stopped state remain available.

Given a session is waiting for approval
When the user closes or quits the app
Then the pending approval prompt is discarded and cannot be approved after reopening.

Given the app crashed or was force-quit during an active run
When the app is launched again
Then the latest session shows the previous run as interrupted/crashed with the last persisted transcript and tool records.

Given the app crashed or was force-quit while waiting for approval
When the app is launched again
Then no stale approval prompt is replayed and the agent must repropose the action before it can run.

Given the app is launched after a crash or force-quit during an active run
When the latest session is resumed
Then the app does not automatically reconnect, continue generation, replay recovery, reattach to prior runtime processes, or resend unsent prompts.

Given assistant output is partially streamed
When the user selects stop
Then the partial assistant message remains in the transcript with an interrupted or stopped status marker.

Given a session has been stopped
When the user returns to the session
Then transcript, tool history, approvals, artifacts, and stopped state remain available.

Given a stopped session remains selected
When the user sends a new prompt or retries
Then the app may start a new run in the same session using the normal provider and approval rules.

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
 - Records are automatically deleted by an undocumented cleanup rule.
 - A prior user or assistant message can be edited or deleted.
 - An idle existing session changes model after transcript history exists.
 - A running session changes model because the project default model changed.
 - Retry in an existing session allows choosing a different model.
 - A provider or network failure deletes, rewrites, or silently resends an already-submitted user message.
 - A provider or network failure triggers a hidden retry loop without explicit user action.
 - Retry replaces, deletes, regenerates in place, or branches from a failed assistant/run record.
 - Transcript pruning, message redaction UI, branch-from-message, or conversation rewrite controls appear in MVP.
 - Full-text transcript search, cross-session search, or tool-log search controls appear in MVP.
 - A stopped session continues executing tools.
 - Closing or quitting the app leaves an active run executing.
 - A pending approval can be approved after app close, crash, force-quit, or restart.
 - A stale approval prompt is replayed after app restart.
 - A crash or force-quit loses the last persisted transcript or tool records.
 - A crash or force-quit recovery automatically continues, reconnects, replays, reattaches to processes, or resends an unsent prompt.
 - Minimized execution hides status or stop controls after restore.
 - Stop deletes the session transcript or tool history.
 - Stop deletes, auto-regenerates, or silently completes a partial assistant message.
 - A stopped partial assistant message lacks a visible interrupted or stopped marker.
 - Stop kills arbitrary external processes outside the app-supervised runtime/process tree.

# Out Of Scope

 - Multiple active sessions.
 - Partial-response editing.
 - Message edit/delete.
 - Full-text transcript search.
 - Cross-session search.
 - Tool-log search.
 - Transcript pruning.
 - Message redaction UI.
 - Branch-from-message.
 - Conversation rewrite.
 - Branch/regenerate controls.
 - Automatic continuation after stop.
 - Closed-app background execution.
 - System tray daemon.
 - Background agent service.
 - OS notifications.
 - Scheduled runs.
 - Wake/resume automation.
 - Pause/resume stream.
 - Background continuation after stop.
 - Partial-response regeneration controls.
 - Session delete.
 - Session archive management.
 - Automatic retention cleanup.
 - Storage quotas.
 - Export/import.
 - Cross-device sync.
 - Long-term memory.
 - Child sessions and subagents.
