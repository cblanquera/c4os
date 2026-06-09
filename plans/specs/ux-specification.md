# UX Specification

## Product Shape

The app should feel like a dense, work-focused desktop command center rather than a chat-only app or marketing surface.

## Primary Layout

 - Left side rail: Projects, Sessions, Artifacts, Plugins, Settings.
 - Project sidebar: project list, active sessions, pinned sessions, worktrees.
 - Main pane: active session transcript and composer.
 - Right inspector: tool activity, files, diffs, artifacts, approvals, model details.
 - Bottom drawer: terminal/tool output when needed.

## Project View

Project view shows:
 - Registered folder.
 - Git status.
 - Active sessions.
 - Worktrees.
 - Recent artifacts.
 - Enabled plugins and MCP servers.
 - Detected instructions and skills.

## Session View

Session view shows:
 - Conversation transcript.
 - Agent/model selector.
 - Approval mode indicator.
 - Tool activity timeline.
 - Active worktree indicator.
 - Generated artifacts.
 - Resume, stop, fork, archive, and summarize controls.

## Approval Design

Approval prompts should be compact but explicit. The user must be able to answer:
 - What will run?
 - What will it read?
 - What will it change?
 - Which policy caused the prompt?
 - Is this approval one-time or broader?

## Artifact Viewer

Artifact viewer must support:
 - Text and Markdown.
 - Images.
 - HTML pages.
 - PDFs and documents where renderer support exists.
 - Spreadsheets and CSV.
 - JSON and structured output.
 - Logs.
 - Diffs.

Each artifact shows provenance and actions: open, reveal, export, duplicate, delete.

## File Browser

The project file browser should:
 - Respect ignore and permission rules.
 - Show changed files and generated artifacts.
 - Allow opening files read-only.
 - Allow user-approved edits through the agent or external editor.

## Browser Panel

Browser/web content viewing should:
 - Open local development URLs and generated files.
 - Support screenshots and page inspection as approved tools.
 - Keep remote content isolated from privileged backend IPC.

## Model And Provider UX

Settings should separate:
 - OpenRouter account/API key.
 - BYOK provider keys and routing.
 - Project default model.
 - Agent model override.
 - Per-session temporary model switch.

Show warnings when switching to a model with weaker tool support, lower context, or different provider routing.

## Plugin Marketplace UX

Plugin cards show:
 - Name, description, category, source.
 - Included skills and MCP servers.
 - Required permissions.
 - Authentication policy.
 - Install/update/enable/disable state.

Install and enable should be separate actions.

## Accessibility And Responsiveness

 - Keyboard-first navigation.
 - Searchable command palette.
 - Screen-reader labels for core controls.
 - Clear focus states.
 - Usable at laptop widths without hiding approvals.
 - No overlapping text or controls.
