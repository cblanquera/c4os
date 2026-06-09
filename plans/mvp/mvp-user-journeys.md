# MVP User Journeys

This document describes the minimum user journeys needed to validate the MVP.

## Journey 1: Configure The App

Goal: a technical user can make the app ready for one local coding session.

Steps:

1. User opens the desktop app.
2. User enters an OpenRouter API key.
3. User selects a default model from a short supported list.
4. App confirms credentials are stored locally.

MVP features exercised:

 - Desktop shell.
 - Provider settings.
 - Local secret storage.
 - Minimal model selection.

Excluded from this journey:

 - Direct provider setup.
 - Provider routing controls.
 - Budget dashboards.
 - Team or enterprise settings.

Validation goal: users can reach a runnable state without support.

## Journey 2: Register A Local Git Project

Goal: the user can create a clear project boundary for agent work.

Steps:

1. User selects a local folder.
2. App verifies it is a Git project.
3. App shows project root, current branch, dirty state, and changed files.
4. App shows root `AGENTS.md` if present.
5. App states that file access is scoped to this project root.

MVP features exercised:

 - Project registration.
 - Git detection.
 - Root `AGENTS.md` display.
 - Basic file browser.
 - Project-root boundary display.

Excluded from this journey:

 - Non-Git project support.
 - Nested `AGENTS.md`.
 - Worktrees.
 - Monorepo indexing optimization.

Validation goal: users understand what the agent can see and change.

## Journey 3: Start One Agent Session

Goal: the user can run one useful local coding task.

Steps:

1. User opens the registered project.
2. User starts a new session.
3. User enters a small coding task.
4. App starts the OpenCode-backed runtime.
5. User sees assistant output and tool activity.

MVP features exercised:

 - Session creation.
 - Conversation transcript.
 - OpenCode runtime adapter.
 - Runtime process supervision.
 - Tool activity timeline.

Excluded from this journey:

 - Multiple active sessions.
 - Subagents.
 - Custom agents.
 - Skills.
 - MCP tools.

Validation goal: users can complete a simple coding task through the desktop flow.

## Journey 4: Approve Or Deny An Action

Goal: the user can control local-risk actions.

Steps:

1. Agent proposes a file write, shell command, or Git action.
2. App shows the action, target, risk type, and approval options.
3. User chooses allow once, allow for session, or deny.
4. App records the approval decision.
5. App shows the result after execution or denial.

MVP features exercised:

 - Approval gateway.
 - One-time allow.
 - Session allow.
 - Deny.
 - Tool ledger.
 - Tool result display.

Excluded from this journey:

 - Always allow.
 - Project-wide permanent approval.
 - Plugin approvals.
 - MCP approvals.
 - Data-flow-aware policy.

Validation goal: users understand and trust approval prompts.

## Journey 5: Inspect Agent Changes

Goal: the user can see what changed and connect changes to agent activity.

Steps:

1. Agent changes one or more files.
2. App shows changed-file list.
3. User opens a diff.
4. User sees the related tool activity.
5. User decides whether to continue, stop, or handle changes outside the app.

MVP features exercised:

 - Changed-file list.
 - Git diff viewer.
 - Tool-to-change provenance.
 - Stop session.

Excluded from this journey:

 - Commit creation.
 - Pull request creation.
 - Inline code review comments.
 - Worktree merge.

Validation goal: users can inspect output well enough to trust or reject the agent's work.

## Journey 6: Resume Context

Goal: the user can restart the app and continue understanding prior work.

Steps:

1. User closes the app.
2. User reopens the app.
3. App shows the registered project.
4. App shows the latest session transcript and tool activity.
5. User can continue the session or start over.

MVP features exercised:

 - SQLite-backed session persistence.
 - Transcript persistence.
 - Tool ledger persistence.
 - Project/session list.

Excluded from this journey:

 - Cross-device sync.
 - Session export/import.
 - Long-term memory.
 - Archive management.

Validation goal: persistence is reliable and valuable.

## Journey 7: Recover From Failure

Goal: users can handle predictable failure without losing confidence.

Steps:

1. Runtime start fails, provider request fails, shell command fails, or tool execution fails.
2. App shows a clear failure state.
3. User can inspect basic output.
4. User can retry or stop the session.

MVP features exercised:

 - Runtime error state.
 - Provider error state.
 - Failed tool state.
 - Basic logs.
 - Retry or stop control.

Excluded from this journey:

 - Automated repair.
 - Support bundle generation.
 - Advanced diagnostics.

Validation goal: failures are understandable enough for early validation users.

