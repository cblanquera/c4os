# UX Specification

## Product Shape

The app should feel like a dense, work-focused desktop command center rather than a chat-only app or marketing surface.

## Primary Layout

 - Left side rail: Projects, Session, Artifacts, Settings.
 - Project sidebar: minimal registered-project selector with one selected project active at a time.
 - Main pane: active session transcript and composer.
 - Right inspector: tool activity, files, diffs, artifacts, approvals, model details.
 - Bottom drawer: terminal/tool output when needed.

## Project View

Project view shows:
 - Registered project selector/list.
 - Selected Git project root.
 - Git status.
 - Current session.
 - Recent text-like artifacts.
 - Root `AGENTS.md` display when present.

MVP does not include an editable system prompt, project prompt editor, prompt template picker, instruction composer, or hidden app-authored instruction layer. Future skill creator workflows may create explicit, reviewable instruction artifacts, but they are post-MVP and separate from the MVP session composer.

MVP project UX does not include project search, grouping, archive, delete, favorites, metadata editing, cross-project views, or multi-project operations.

## Session View

Session view shows:
 - Conversation transcript.
 - Model selector.
 - Approval mode indicator.
 - Tool activity timeline.
 - Text-like generated artifacts.
 - Resume and stop controls.

MVP session UX does not include global search, cross-session search, full-text transcript search, tool-log search, artifact search, or project-wide file content search UI.

Minimized app windows may continue active runs while the app process is running. Closing or quitting the app stops active work and preserves session history. MVP does not include tray controls, OS notifications, scheduled runs, or wake/resume automation.

After an app crash or force-quit during an active run, reopening the app shows the latest session with the prior run marked interrupted/crashed. The user can retry or send a new prompt, but the MVP does not auto-continue, reconnect, replay, or resend unsent prompts.

Pending approval prompts do not survive app close, crash, force-quit, or restart. Reopening the app shows the stopped or interrupted run state; the user cannot approve stale prompts and the agent must repropose the action later.

If a provider or network failure occurs after the user submits a message, the submitted message remains in the transcript and the failed assistant/run state is visible. The user can explicitly retry or send a follow-up. A retry control may be shown, but it appends a new retry action/status instead of replacing the failed assistant record. The MVP does not delete the prompt, silently retry, regenerate in place, branch from failure, or rewrite the transcript.

## Approval Design

Approval prompts should be compact but explicit. The user must be able to answer:
 - What will run?
 - What will it read?
 - What will it change?
 - Which approval source caused the prompt?
 - Is this approval one-time or broader?

File-write approval prompts show target path, action type, and a bounded diff or summary when available, including root `AGENTS.md` edits. If a safe diff cannot be produced, the prompt states that clearly and still requires explicit approval. Multiple file writes may be approved in one explicit atomic batch only when every target path, action type, and per-file diff or summary state is visible. Batch prompts are capped to a fixed MVP product threshold, initially 10 files plus bounded total preview size; oversized batches must be split before approval. These caps are not user-configurable in MVP. If any file is blocked by policy, including outside-root or secret-deny targets, the whole batch is blocked and the agent must propose a smaller valid batch. If any file lacks safe preview, the prompt makes that visible before approval; the batch remains approve-or-deny as proposed. MVP does not include user-configurable approval thresholds, per-project approval caps, advanced safety settings, giant scrollback approval prompts, unbounded generated diffs, approve-by-summary-only for oversized batches, checkbox-per-file partial approval, automatic subset execution, hidden batch rewriting, blanket approve-all-future-writes, project-wide write approval, a full editor, merge UI, or multi-file review workflow beyond the approval prompt, changed-file list, and diff viewer.

Approval records are visible in the local activity ledger. Normal OS text selection/copy of visible ledger text is acceptable, but MVP does not expose a dedicated copy button, copy-all action, structured export, JSON download, support bundle, or share workflow for approval records.

Shell command results are visible as bounded redacted/truncated summaries. Live terminal output may be visible while a command is running or while the live terminal buffer remains open, but app-owned persisted records remain bounded redacted/truncated summaries. After completion, an open live terminal drawer may remain temporarily visible in the same app session if it is labeled live/ephemeral rather than persisted history. After navigation away, reload, app close, or session restore, the durable shell output view uses only the persisted summary or output-omitted marker. If a safe summary cannot be produced, the persisted record shows an explicit output-omitted marker with command metadata instead of raw stdout/stderr. The shell summary may expand to safe reason labels such as `truncated_by_size`, `redacted_secret_pattern`, or `output_omitted_safe_summary_failed`, but not redacted substrings, sensitive raw byte counts, or reconstruction hints. Normal OS text selection/copy of visible shell summary text is acceptable, but MVP does not expose a dedicated copy raw output button, shell output export, retained raw terminal buffer, or unredacted raw stdout/stderr persistence.

## Artifact Viewer

Artifact viewer must support:
 - Text and Markdown.
 - Logs.
 - Diffs.
 - Generated source or config files when they are text-like.

Each artifact shows provenance and actions: open text preview when safe, reveal file, and return to related tool activity.

MVP does not include active HTML rendering, images, PDFs, documents, spreadsheets, browser-based previews, Chromium-backed previews, export, duplicate, sharing, or artifact execution.

## File Browser

The project file browser should:
 - Stay scoped to the selected project root.
 - Show changed files and generated artifacts.
 - Allow opening files read-only for inspection.
 - Show secret-deny files as present but not preview their contents.
 - Avoid project-wide file content search UI in MVP.
 - Avoid built-in editing and external-editor workflows in MVP.
 - Route file changes through agent-proposed writes and the Approval Gateway.

## Post-MVP Browser Panel

Browser/web content viewing should:
 - Open local development URLs and generated files.
 - Support screenshots and page inspection as approved tools.
 - Keep remote content isolated from privileged backend IPC.

## Model And Provider UX

Settings should separate:
 - OpenRouter account/API key.
 - Project default model.
 - Selected model, provider route, and metadata source/freshness when available.

Default model changes apply to future sessions only, even if changed while a session is running. Existing sessions keep their selected model whether running, idle, or stopped. MVP does not expose OpenRouter credit balance, billing links, top-up flows, invoice links, spend warnings, account diagnostics, per-call token counts, cost estimates, model-call accounting, spend history, budget meters, budget enforcement, hot model swap, run restart for model change, in-flight model migration, mid-session model switching, idle-session model switching, retry-with-different-model, per-message model overrides, per-agent model overrides, fallback routes, or budget controls.

MVP uses one default coding agent/runtime persona. If the runtime requires an agent reference, the app may store it as adapter metadata only. MVP does not expose project default agent controls, custom persona controls, retry-with-different-agent, per-message persona overrides, subagents, or handoffs.

## Post-MVP Plugin Marketplace UX

Plugin cards show:
 - Name, description, category, source.
 - Included skills and MCP servers.
 - Required permissions.
 - Authentication policy.
 - Install/update/enable/disable state.

Install and enable should be separate actions.

## Accessibility And Responsiveness

 - Keyboard-first navigation.
 - Command palette and command search are post-MVP unless separately scoped.
 - Screen-reader labels for core controls.
 - Clear focus states.
 - Usable at laptop widths without hiding approvals.
 - No overlapping text or controls.
