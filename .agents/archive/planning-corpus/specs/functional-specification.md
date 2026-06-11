# Functional Specification

This document describes the expected product behavior.

## Project Management

Users can add a project by selecting a local Git folder. The MVP can remember multiple registered projects and expose a minimal Project Selector, but only one selected project is visible and active at a time. A project stores display name, root path, default model, latest session, artifact index, and runtime adapter metadata when required.

Functional behavior:
 - List registered projects and allow choosing one selected project.
 - Detect Git repositories and show branch, dirty state, and remotes.
 - Detect and display root `AGENTS.md` as user-visible guidance only.
 - Allow root `AGENTS.md` to be explicitly read during a session under normal project-root file-read rules.
 - Allow agent-proposed edits to root `AGENTS.md` under normal project-root file-write approval rules.
 - Block file access outside the selected project root.
 - Exclude non-Git folders from MVP.
 - Exclude automatic root `AGENTS.md` injection, automatic root `AGENTS.md` reload after write, nested `AGENTS.md` resolution, `AGENTS.md` permission authority, instruction precedence handling, editable system prompts, project prompt editors, instruction composers, prompt templates, hidden app-authored instruction layers, project search, grouping, archive, delete, favorites, metadata editing, cross-project views, and multi-project operations from MVP.

## Sessions

A session is a durable conversation and tool-activity record attached to one selected project. It may be resumed after app restart, but only one session can be active or running at a time.

Functional behavior:
 - Create the current session for the selected project.
 - Resume the latest session after app restart.
 - Stop the active run by canceling the current runtime/model stream and supervised child processes owned by the app.
 - Allow an active run to continue while the app window is minimized, as long as the app process remains running.
 - Stop the active run when the app is closed or quit, using normal stop-active-run behavior.
 - On next launch after an app crash or force-quit during an active run, mark the previous run interrupted/crashed and preserve the last persisted transcript and tool records.
 - Discard pending approval prompts on close, crash, force-quit, or restart; require the agent to repropose the action later.
 - Persist partial assistant output after stop as an interrupted or stopped assistant message.
 - Preserve transcript, tool history, approvals, and artifacts after stop so the user can continue or retry later.
 - Treat the session transcript as append-only, except for runtime status updates on generated records.
 - Treat retry as a new appended user action/status, not as in-place regeneration or replacement of the failed assistant record.
 - Persist transcript, selected model, runtime adapter metadata, tool calls, approvals, and text-like generated-file artifact records.
 - Exclude durable pending approvals, approve-after-restart, stale approval prompt replay, closed-app background execution, crash recovery replay, automatic process reattachment, automatic continuation after crash, unsent prompt resend, system tray daemon, background agent service, OS notifications, scheduled runs, wake/resume automation, global search, cross-session search, transcript full-text search, tool-log search, user/assistant message edit, message delete, transcript pruning, message redaction UI, branch-from-message, branch-from-failure, conversation rewrite, regenerate-in-place, response replacement, partial-response editing, branch/regenerate controls, automatic continuation after stop, pause/resume stream, partial-response regeneration controls, arbitrary external process killing, session rename, pin, archive, delete, child sessions, handoff, and multiple active sessions from MVP.

## Agents

MVP uses one default coding agent/runtime persona.

Functional behavior:
 - Use one default coding agent for MVP sessions.
 - Persist the runtime agent reference if OpenCode requires one, but only as adapter metadata.
 - Do not expose project default agent UI or persona controls.
 - Exclude changing project default agent while a session is running or after transcript history exists, retry-with-different-agent, per-message persona, project-level custom agents, custom prompts/personas, editable system prompts, project prompt editors, instruction composers, prompt templates, subagents, handoffs, and agent switching from MVP.

## Tools

All executable capabilities appear as tools in a unified tool catalog.

Tool categories:
 - Files: read, list, glob, grep, and approval-gated runtime write/edit/apply-patch actions.
 - Shell: command execution through PTY.
 - Git: unprompted status, branch display, changed-file list, diff inspection, and approval-gated state-changing actions.
 - Artifact: capture plain text, Markdown, logs, diffs, and generated source or config files.

Functional behavior:
 - Every tool call is logged.
 - Runtime file search tools such as glob and grep stay scoped to the selected project and normal file-access rules.
 - File writes, shell commands, and Git state-changing actions route through the Approval Gateway.
 - Tool results can create artifacts.
 - Tool calls show status, redacted/truncated output summary, affected files, duration, and approval decision.
 - Denied or blocked tool calls return a structured denial result to the runtime.

## Git

Functional behavior:
 - Show Git status and diffs.
 - Show changed files for the selected project.
 - Allow Git inspection without approval prompts.
 - Log runtime-performed Git inspection in tool activity.
 - Require approval for Git actions that change local or remote repository state.
 - Never run destructive Git operations without explicit approval.
 - Exclude worktree creation and cleanup from MVP.

## Artifacts

Functional behavior:
 - Generate artifact records for plain text, Markdown, logs, diffs, and generated source or config files.
 - Show artifact provenance from session and tool call.
 - Reveal or link non-text generated files without rich previewing them.
 - Exclude artifact search, active HTML rendering, rich previews, browser/web viewing, duplicate, and export workflows from MVP.

## Model Providers

Functional behavior:
 - Use OpenRouter as the only MVP provider gateway.
 - Support OpenRouter credential setup and model selection.
 - Allow OpenRouter credential update or revoke only when no session is running.
 - Keep a running session on the credential reference it started with until stopped or complete.
 - Use one selected OpenRouter model per session, fixed at session creation.
 - Apply model changes only to future sessions, not to existing idle sessions with transcript history.
 - Allow project default model changes while a session is running, but do not affect the running session or runtime.
 - Ensure the runtime effective model matches the app-owned selected OpenRouter model.
 - Send only the active session transcript, selected model/routing metadata, user-approved or policy-allowed tool results/summaries, and explicitly read in-project file contents as model context.
 - For shell command tool results sent into model context, include only the persisted bounded redacted/truncated summary, output-omitted marker, and safe output summary reason labels.
 - Never send live raw terminal buffers, omitted raw shell output, redacted substrings, sensitive raw byte counts, offsets, hashes, or reconstruction metadata into model context.
 - Provide standing provider setup disclosure that prompts and bounded context leave through OpenRouter.
 - Record bounded context-source summaries for model calls in session activity.
 - Fail new model calls closed with clear provider or network errors when OpenRouter or network access is unavailable.
 - Keep already-submitted user messages appended when a provider or network failure prevents the assistant response.
 - Persist failed assistant/run status after provider or network failure, and require explicit user retry or follow-up.
 - If a retry command is offered after provider or network failure, record it as a new appended user action/status.
 - Preserve transcript, tool history, approvals, and artifacts so the user can retry after provider or connectivity recovery.
 - Validate OpenRouter credential presence and selected model route viability.
 - Show model context limits, tool support, streaming support, pricing source, provider route, metadata source, and freshness timestamp when available.
 - Label pricing, capability, or route metadata as stale or unknown when unavailable.
 - Do not block sessions because model metadata is stale or unavailable when provider credentials and the selected model route are viable.
 - Exclude OpenRouter credit balance, billing link management, top-up flows, invoice links, spend warnings, account diagnostics, per-call token counts, cost estimates, model-call accounting, spend history, budget meters, budget enforcement, prompt deletion after failed model call, retry-in-place, response replacement, branch-from-failure, hidden retry loops, silent prompt resend, hot key rotation, per-session credential switching, mid-call credential retry, offline model fallback, direct provider fallback, automatic model switching, queued background resend, synthetic assistant continuation, per-call model-context approval, raw prompt export, token-by-token context inspection, editable context composers, direct provider integrations, local model providers, app-owned live model catalog sync, cache invalidation workflows, hot model swap, run restart for model change, in-flight model migration, runtime reconfiguration from default-model change, mid-session model switching, idle-session model switching, retry-with-different-model, per-message model override, per-agent model overrides, fallback routing, cost dashboards, budget controls, and provider-specific app settings from MVP.

## Approvals

Functional behavior:
 - Show approval prompts with command/tool name, arguments, target resources, risk, approval source, and expected effect.
 - Support one-time allow, narrow shell session allow, and deny.
 - Treat pending approval prompts as non-durable runtime state, not persisted approval decisions.
 - Persist answered approval decisions as durable ledger records with structured metadata, decision, timestamp, resulting action status, and bounded redacted prompt summary or diff reference.
 - Show approval records only in the local activity ledger.
 - Allow normal OS text selection/copy of visible approval ledger text.
 - Exclude dedicated approval copy buttons, approval record export, copy-all, support bundle, JSON download, sharing, full prompt text replay blobs, raw command output, raw secret values, approval decision editing, approval decision deletion, and durable pending approval state from MVP.
 - Return structured denial results for denied or blocked actions, using categories such as `user_denied`, `outside_project_root`, `secret_denied`, `destructive_requires_one_time`, or `mvp_scope_blocked`.
 - Keep an approval audit trail.
 - Allow live terminal output to be visible while a shell command is running or while the live terminal buffer remains open.
 - Allow an open live terminal drawer to remain temporarily available after command completion in the same app session only when labeled live/ephemeral.
 - Do not retain live terminal buffers after command completion, navigation away, reload, app close, or session restore; durable views use the persisted summary or output-omitted marker.
 - Persist shell command output as bounded redacted/truncated summaries, not unlimited raw stdout/stderr.
 - If a safe shell output summary cannot be produced, persist command metadata and an explicit output-omitted marker instead of raw stdout/stderr.
 - Allow shell output summaries to show safe reason labels such as `truncated_by_size`, `redacted_secret_pattern`, or `output_omitted_safe_summary_failed`.
 - Exclude redacted substrings, sensitive raw byte counts, and reconstruction paths to raw shell output from shell summary metadata.
 - Use only the persisted shell output summary, output-omitted marker, and safe reason labels when shell tool results are included in model context.
 - Allow normal OS text selection/copy of visible redacted/truncated shell summaries.
 - Exclude dedicated raw shell output copy buttons, shell output export, and raw stdout/stderr persistence from MVP.
 - For file-write approvals, show target path, action type, and a bounded diff or summary when available.
 - If a safe diff cannot be produced for a file-write approval, state that clearly and require explicit approval before execution.
 - Allow multiple file writes in one explicit atomic batch approval only when every target path, action type, and per-file diff or summary state is visible.
 - Cap batch file-write approvals to a fixed MVP product threshold, initially 10 files plus bounded total preview size.
 - If a batch exceeds file-count or preview-size caps, block the batch and require the agent to propose smaller batches.
 - If any file in a batch is blocked by policy, including outside-root or secret-deny targets, block the whole batch and require the agent to propose a smaller valid batch.
 - If any file in a batch lacks safe preview, make that visible before approval; the batch still remains approve-or-deny as proposed.
 - Exclude user-configurable approval thresholds, per-project approval caps, advanced safety settings, giant scrollback approval prompts, unbounded generated diffs, approve-by-summary-only for oversized batches, checkbox-per-file partial approval, automatic subset execution, hidden batch rewriting, blanket approve-all-future-writes, and project-wide write approval from MVP.

## Post-MVP Functional Areas

The following functional areas are intentionally excluded from MVP:

 - Multiple active sessions.
 - Full project management beyond a minimal registered-project selector.
 - Session rename, pin, archive, and delete.
 - Child sessions and handoff.
 - Worktrees.
 - MCP integration.
 - Skill discovery and invocation.
 - Skill creator workflows for explicit, reviewable instruction artifacts.
 - Plugin installation and execution.
 - Marketplace support.
 - Browser and web viewing.
 - Rich artifact previews and export workflows.
 - Full AGENTS.md compatibility.
 - Agent Skills compatibility.
 - Generic prompt editor or hidden app-authored instruction layer.
 - MCP compatibility.
 - Codex plugin compatibility.
 - OpenCode config compatibility.
 - Import/export and round-trip compatibility.
