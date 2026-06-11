# Overview

Agent Execution covers running one OpenCode-backed agent session and surfacing model and tool activity.

# Success Criteria

Requirement: The app can start one OpenCode-backed session.
Expected Result: The user sees assistant output and tool activity for the active project.

Requirement: MVP uses one default coding agent.
Expected Result: The UI does not expose project default agent controls, custom agents, persona controls, agent switching, subagents, or handoff controls.

Requirement: Runtime actions are visible before and after execution.
Expected Result: File activity, shell commands, Git inspection, and runtime-proposed Git state changes appear in the activity timeline and ledger.

Requirement: OpenCode config compatibility is not claimed in MVP.
Expected Result: The app stores only app-owned launch settings and adapter references required to run OpenCode.

# Functional Acceptance Criteria

Given provider credentials and an active project
When the user sends a prompt
Then the runtime starts and assistant output appears in the session.

Given the app starts an OpenCode-backed session
When the session has a selected OpenRouter model
Then the runtime effective model matches the app-owned selected model.

Given a session has transcript history and no active run
When the project default model changes
Then the existing session keeps its original selected model.

Given a session is running
When the project default model changes
Then the running session keeps its original selected model and credential/model reference.

Given a session is running
When the project default model changes
Then the app does not restart the run, hot-swap the model, reconfigure the runtime, or migrate the in-flight model call.

Given a failed assistant/run record exists
When the user retries
Then the retry uses the session's original selected model.

Given the user views model information for a session
When model metadata is available
Then the UI may show selected model, provider route, metadata source, and metadata freshness.

Given the user views model information for a session
When the MVP is in scope
Then the UI does not show per-call token counts, cost estimates, spend history, budget meters, or budget enforcement.

Given the user configures OpenRouter
When credentials are present and the selected model route is viable
Then MVP can start model-backed sessions without showing account billing information.

Given the user views OpenRouter settings
When the MVP is in scope
Then the UI does not show credit balance, billing links, top-up flows, invoice links, spend warnings, or account diagnostics.

Given the app starts an OpenCode-backed session
When the runtime captures provider settings
Then the session keeps its starting OpenRouter credential reference until stopped or complete.

Given the MVP build
When the user starts or views a session
Then the session uses the default coding agent without exposing persona management.

Given OpenCode requires an internal agent reference
When the app stores it
Then it is persisted as adapter metadata and not shown as user-facing agent/persona configuration.

Given a session is running or has transcript history
When the user views project settings
Then no project default agent, retry-with-different-agent, per-message persona, or agent migration controls are available.

Given the MVP build
When the user views project or session settings
Then no editable system prompt, project prompt editor, instruction composer, prompt template picker, or hidden app instruction control is available.

Given the product roadmap includes skill creation
When MVP scope is evaluated
Then skill creator workflows for explicit, reviewable instruction artifacts remain post-MVP.

Given the runtime proposes a file write, shell command, or Git state change
When the action requires approval
Then the action is not executed until the user approves it.

Given the runtime emits a tool result
When the result is received
Then the app records the tool name, status, approval decision, bounded redacted output summary, and affected files.

Given a shell command emits stdout or stderr
When the output is persisted after execution
Then the stored record contains a bounded redacted/truncated summary, not unlimited raw stdout/stderr.

Given a shell command emits stdout or stderr
When a safe redacted/truncated summary cannot be produced
Then the stored record contains command metadata and an explicit output-omitted marker, not raw stdout/stderr.

Given a shell tool result is truncated, redacted, or omitted
When the stored tool record includes reason metadata
Then the metadata uses safe labels and does not include redacted substrings or reconstruction details.

Given a shell output summary is visible in the activity timeline
When the user selects visible text manually
Then normal OS text copy is available without a dedicated copy raw output or shell output export control.

Given a protected action approval prompt is answered
When the runtime emits the resulting tool status
Then the app records structured prompt metadata, decision, timestamp, resulting action status, and bounded redacted prompt summary or diff reference in the activity ledger.

Given a model call fails because OpenRouter or network access is unavailable
When the session returns to the user
Then runtime state is failed or recoverable, and transcript, tool history, approvals, and artifacts remain available.

Given a user message has been submitted
When the model call fails because OpenRouter or network access is unavailable
Then the user message remains appended, the failed assistant/run status is persisted, and no automatic resend occurs.

Given a provider or network failure interrupted an assistant response
When connectivity or provider access recovers
Then the user must explicitly retry or send a follow-up before a new model call is made.

Given a provider or network failure created a failed assistant/run record
When the user retries
Then the app appends a retry action/status and starts a new model call under normal session rules.

Given a provider or network failure created a failed assistant/run record
When the user retries
Then the app does not replace the failed assistant record, regenerate it in place, branch from it, or mutate prior transcript messages.

Given the agent is streaming or running tools
When the user selects stop
Then the active runtime/model stream is canceled, app-supervised child processes are terminated, and transcript, tool history, approvals, and artifacts remain available.

Given the agent is streaming or running tools
When the app window is minimized
Then the run may continue while the app process remains running.

Given the agent is streaming or running tools
When the app is closed or quit
Then the active run is stopped and transcript, tool history, approvals, and artifacts remain available.

Given the agent is waiting for approval
When the app is closed or quit
Then the pending approval is discarded and no action executes from that prompt.

Given the agent is streaming or running tools
When the app crashes or is force-quit
Then the next launch marks the run interrupted/crashed and preserves the last persisted transcript and tool records.

Given the agent is waiting for approval
When the app crashes or is force-quit
Then the next launch does not replay the stale approval prompt and no action executes from it.

Given the app is relaunched after a crash or force-quit during agent execution
When the latest session is opened
Then the app does not automatically continue generation, reconnect to the old stream, reattach to prior runtime processes, replay recovery, or resend unsent prompts.

Given assistant output is partially streamed
When the user selects stop
Then the partial assistant message is persisted as stopped or interrupted, not complete.

Given the session already contains user or assistant messages
When runtime activity continues
Then prior messages are not edited or deleted, except for status updates on runtime-generated records.

Given OpenCode requires launch settings or runtime references
When the app stores those values
Then they are treated as app-owned adapter settings, not imported or mirrored OpenCode config.

# Security Acceptance Criteria

Given a runtime action requires local access
When the action targets file writes, shell commands, or Git state changes
Then the action is checked by the app approval policy before execution.

Given provider credentials are configured
When the runtime starts
Then credentials are not displayed in the transcript or tool activity.

Given existing OpenCode config affects runtime behavior
When Phase 0 evaluates runtime integration
Then the behavior is discovered and disclosed before implementation continues.

Given existing OpenCode config can override provider or model routing
When the app cannot detect or prevent the override
Then the OpenCode-backed runtime path is blocked for MVP.

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
 - The app claims OpenCode config import, mirror, edit, export, or round-trip compatibility in MVP.
 - Existing OpenCode config affects runtime behavior without disclosure.
 - Existing OpenCode config overrides the selected OpenRouter model without detection or prevention.
 - An existing session changes selected model after creation.
 - A running session changes model because the project default model changed.
 - A project default model change restarts, reconfigures, hot-swaps, or migrates a running session.
 - Retry allows selecting a different model in the same session.
 - A per-message model override appears in MVP.
 - The UI shows per-call token counts, cost estimates, spend history, budget meters, or budget enforcement in MVP.
 - The UI shows OpenRouter credit balance, billing links, top-up flows, invoice links, spend warnings, or account diagnostics in MVP.
 - A running session changes OpenRouter credential references after key update or revoke.
 - Provider failure loses transcript, tool history, approvals, or artifacts.
 - Provider failure deletes or rewrites an already-submitted user message.
 - Provider failure triggers a hidden retry loop or silent prompt resend.
 - Provider retry replaces, deletes, regenerates in place, or branches from a failed assistant/run record.
 - Provider failure causes synthetic assistant continuation.
 - The user cannot stop a running session.
 - Stop deletes transcript, tool history, approvals, or artifacts.
 - Stop deletes, auto-regenerates, or silently completes a partial assistant response.
 - Partial stopped assistant output has no visible stopped or interrupted status.
 - Closing or quitting the app leaves model streaming or app-supervised child processes running.
 - A pending approval survives app close, crash, force-quit, or restart as an approvable prompt.
 - A stale approval prompt executes after restart.
 - Crash or force-quit recovery loses the last persisted transcript or tool records.
 - Crash or force-quit recovery automatically continues generation, reconnects, reattaches to prior runtime processes, replays recovery, or resends unsent prompts.
 - A prior user or assistant message is edited, deleted, pruned, redacted through UI, branched, or rewritten.
 - Stop leaves app-supervised child processes running.
 - Stop kills arbitrary external processes outside the app-supervised runtime/process tree.
 - Custom agent controls appear in MVP.
 - Project default agent controls appear in MVP.
 - Persona controls or per-message persona overrides appear in MVP.
 - Retry allows selecting a different agent in the same session.
 - A session changes agent/persona after transcript history exists or while running.
 - Editable system prompt, project prompt editor, instruction composer, prompt template picker, or hidden app instruction layer appears in MVP.
 - Skill creator workflow is required for MVP session execution.
 - Agent switching controls appear in MVP.

# Out Of Scope

 - Multiple concurrent agents.
 - Runtime abstraction across multiple engines.
 - Custom agents.
 - Custom prompts/personas.
 - Agent switching.
 - Subagents.
 - Agent handoffs.
 - Message edit/delete.
 - Transcript pruning.
 - Message redaction UI.
 - Branch-from-message.
 - Conversation rewrite.
 - Partial-response editing.
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
 - Killing arbitrary external processes.
 - Offline model fallback.
 - Hot key rotation.
 - Per-session credential switching.
 - Mid-call credential retry.
 - Queued background resend after provider failure.
 - OpenCode config import, mirror, edit, export, or round-trip compatibility.
