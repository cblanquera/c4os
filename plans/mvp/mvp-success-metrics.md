# MVP Success Metrics

This document defines measurable success criteria for the smallest useful validation product.

## Validation Cohort

Target cohort:

 - 10 to 20 technical users.
 - Users who already work in local Git repositories.
 - Users willing to test on non-critical coding tasks.

Validation window:

 - 4 to 6 weeks.

## Activation Metrics

Success criteria:

 - 80% of users enter provider credentials without support.
 - 80% of users register a local Git project without support.
 - 70% of users start a first session within 10 minutes of opening the app.
 - 70% of users can identify the selected project root after setup.

Failure signal:

 - Users cannot get to a first runnable session without guided help.

## Core Workflow Metrics

Success criteria:

 - 60% of users complete at least one useful coding task.
 - 70% of successful tasks include at least one visible tool action.
 - 70% of users inspect changed files or diffs after an agent action.
 - 50% of users resume a prior session at least once.

Failure signal:

 - Users treat the app as slower or less clear than terminal-only usage.

## Trust And Safety Metrics

Success criteria:

 - 70% of users say tool activity visibility improved confidence.
 - 60% of users correctly explain an approval prompt before acting on it.
 - Fewer than 20% of users say approval prompts blocked normal progress.
 - 100% of file writes during validation stay inside the selected project root unless explicitly approved.
 - 100% of shell commands require approval before execution.
 - 0 known cases of denied actions executing.

Failure signal:

 - Approval prompts are misunderstood or users approve without knowing the consequence.

## Runtime And Persistence Metrics

Success criteria:

 - 95% of sessions start the runtime successfully.
 - 95% of runtime tool events appear in the activity timeline.
 - 100% of completed sessions reopen with transcript and tool history intact.
 - Runtime crash or cancellation does not corrupt project or session metadata.
 - App remains responsive during one active runtime session.

Failure signal:

 - Runtime state is incomplete, misleading, or not resumable.

## Performance Metrics

Success criteria:

 - Cold start under 5 seconds on validation machines.
 - Project registration under 10 seconds for typical repositories under 1 GB.
 - Approval prompt appears before execution with no observable bypass.
 - Tool activity updates within 1 second of event receipt.

Failure signal:

 - UI latency makes the control-center model feel worse than terminal usage.

## Learning Metrics

The MVP must answer:

 - Do users want a desktop control center for agent-assisted local coding?
 - Does visibility into tools and diffs increase trust?
 - Is one active session enough for an initial useful product?
 - Is OpenCode controllable enough for this product?
 - Is OpenRouter-only setup acceptable for early users?
 - Which excluded feature is requested first: worktrees, multiple sessions, skills, MCP, plugins, browser, or direct providers?

## Expansion Triggers

Consider V1 expansion only if:

 - Core workflow completion is at or above target.
 - Runtime integration is reliable.
 - Users report trust gains from tool visibility.
 - Approval prompts are understood.
 - No local safety boundary failures occur.
 - The top requested V1 feature is clear.

## Invalidation Criteria

The MVP thesis is invalid or needs major re-scope if:

 - OpenCode cannot provide structured events and pre-execution tool interception.
 - Users do not complete useful local coding tasks.
 - Users do not value the desktop shell over terminal-only usage.
 - Approval prompts do not improve trust.
 - Project-root scoping cannot be enforced.
 - OpenRouter-only setup blocks a material portion of validation users.
 - Users primarily want non-code workflows instead of coding workflows.

## Metrics Explicitly Out Of Scope

Do not measure MVP success by:

 - Marketplace installs.
 - Plugin adoption.
 - MCP usage.
 - Multi-agent throughput.
 - Enterprise policy usage.
 - Cross-device sync.
 - Non-code workflow completion.
 - Public growth metrics.

