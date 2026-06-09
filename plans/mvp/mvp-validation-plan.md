# MVP Validation Plan

This document defines how to validate the smallest useful product before expanding scope.

## Validation Goal

Validate whether a single-user, coding-first desktop workspace improves trust, continuity, and control for agent-assisted local repository work.

## Product Assumptions Being Tested

 - Technical users want a desktop control center for local agent work.
 - Coding-first is the right initial wedge.
 - Tool activity, approvals, and diffs are enough to improve trust.
 - Session persistence matters in real use.
 - One active session is enough to validate value.
 - General-purpose non-code workflows can be deferred.

## Architecture Assumptions Being Tested

 - OpenCode can be integrated without a custom runtime.
 - OpenCode exposes structured events for model output, tool calls, and tool results.
 - File, shell, and Git actions can be intercepted before execution.
 - A local backend can act as the approval and persistence boundary.
 - SQLite is enough for one active session.
 - OpenRouter-only provider setup is acceptable for early users.
 - Tauri or the selected desktop shell can support the MVP UI without browser-heavy features.

## Build Boundaries For Validation

The validation build includes:

 - One local user.
 - One registered Git project active at a time.
 - One active agent session at a time.
 - OpenRouter model configuration.
 - OpenCode runtime integration.
 - Project-root file scope.
 - Approval prompts for file writes, shell commands, and Git actions.
 - Tool activity timeline.
 - Basic session persistence.
 - Git changed files and diffs.
 - Root `AGENTS.md` display.
 - Basic text/log/diff artifact capture.

The validation build excludes:

 - Plugins.
 - Marketplace.
 - MCP.
 - Browser/web panel.
 - Worktrees.
 - Multiple concurrent agents.
 - Multiple concurrent sessions.
 - Direct provider fallback.
 - Long-term memory.
 - Cloud sync.
 - Enterprise policy.

## Participant Profile

Participants should be:

 - Developers or technical power users.
 - Comfortable with local Git repositories.
 - Comfortable using AI coding agents or willing to try them.
 - Willing to use non-critical repositories or small branches for validation.

Participants should not require:

 - Enterprise controls.
 - Plugin ecosystems.
 - Remote MCP.
 - Non-code workflows.
 - Regulated data handling.

## Validation Tasks

### Task 1: Configure Provider

User enters OpenRouter credentials and selects a model.

Pass criteria:

 - User completes without support.
 - User understands that model calls leave the local machine.

### Task 2: Add Project

User registers a local Git repository.

Pass criteria:

 - Git state appears.
 - Root boundary is clear.
 - Root `AGENTS.md` appears when present.

### Task 3: Run A Small Coding Task

User asks the agent to inspect or modify a small piece of code.

Pass criteria:

 - Runtime starts.
 - Assistant output streams.
 - Tool activity is visible.
 - User can describe what the agent is doing.

### Task 4: Approve Local Actions

User receives prompts for file write, shell, or Git actions.

Pass criteria:

 - User understands the prompt.
 - Denied actions do not execute.
 - Approved actions are recorded.

### Task 5: Review Changes

User reviews changed files and diffs.

Pass criteria:

 - User can identify changed files.
 - User can inspect a diff.
 - User can connect at least one change to tool activity.

### Task 6: Resume Session

User closes and reopens the app.

Pass criteria:

 - Project remains registered.
 - Latest session reopens.
 - Transcript and tool activity remain visible.

### Task 7: Handle Failure

User encounters or simulates a provider, runtime, or tool failure.

Pass criteria:

 - Failure is visible.
 - User can stop or retry.
 - Session metadata remains intact.

## Data To Collect

Quantitative:

 - Setup completion rate.
 - Time to first session.
 - Runtime start success rate.
 - Tool event capture rate.
 - Approval allow/deny counts.
 - Approval comprehension rate.
 - Task completion rate.
 - Diff inspection rate.
 - Session resume rate.
 - Failure recovery rate.

Qualitative:

 - What increased or reduced user trust.
 - Whether the desktop app felt better than terminal-only usage.
 - Which approval prompts were unclear.
 - Which excluded feature users asked for first.
 - Whether OpenRouter-only setup was acceptable.
 - Whether one active session felt too limiting.

## Technical Validation Gates

Gate 1: Runtime control.

 - Structured runtime events are available.
 - File, shell, and Git actions are interceptable before execution.
 - Runtime can be stopped.

Gate 2: Local safety.

 - File writes stay inside project root.
 - Shell commands require approval.
 - Denied actions never execute.
 - Symlink and path boundary behavior is defined.

Gate 3: Persistence.

 - Sessions survive restart.
 - Tool records survive restart.
 - Failure does not corrupt local state.

Gate 4: Usability.

 - Users complete setup.
 - Users run one useful task.
 - Users inspect changes.
 - Users resume a session.

## Technical Risks To Monitor

 - Runtime event stream is incomplete.
 - Tool interception is unavailable or unreliable.
 - Approval gateway can be bypassed.
 - Shell command safety is weaker than expected.
 - Git diff state diverges from actual filesystem state.
 - SQLite writes block UI responsiveness.
 - Provider failures are hard to explain.

## Expansion Decision

Proceed to V1 only if:

 - MVP success metrics meet target thresholds.
 - Runtime and approval gates pass.
 - Users clearly value the desktop control center.
 - No project-root safety failure occurs.
 - The next most valuable feature is clearly identified by validation data.

Do not expand into plugins, marketplace, remote MCP, browser automation, direct providers, or multi-agent workflows until the MVP thesis is validated.

