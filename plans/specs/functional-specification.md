# Functional Specification

This document describes the expected product behavior.

## Project Management

Users can add a project by selecting a local folder. A project stores display name, root path, trust state, default model, default agent, active sessions, known worktrees, MCP config, plugin config, and artifact index.

Functional behavior:
 - Detect Git repositories and show branch, dirty state, remotes, and worktrees.
 - Detect `AGENTS.md` files in root and nested paths.
 - Detect project skills under supported skill paths.
 - Warn before granting external-directory access.
 - Allow projects that are not code repositories.

## Sessions

A session is a durable conversation plus runtime execution state. A project can contain multiple active and archived sessions.

Functional behavior:
 - Create, resume, rename, pin, archive, and delete sessions.
 - Persist transcript, model choices, agent choices, tool calls, approvals, generated files, and summaries.
 - Support child sessions for subagents or delegated work.
 - Support session handoff between local-root mode and worktree mode.

## Agents

Agents are configurable runtime personas with model, prompt, permissions, tools, skills, and handoff behavior.

Functional behavior:
 - Provide built-in default agents: Ask, Build, Research, Write, Review, Operations.
 - Allow project-level custom agents.
 - Allow subagents for concurrent tasks.
 - Allow agent switching within a session when the runtime supports it.

## Tools

All executable capabilities appear as tools in a unified tool catalog.

Tool categories:
 - Files: read, write, edit, apply patch, list, glob, grep.
 - Shell: command execution through PTY.
 - Git: status, diff, branch, worktree, commit helper.
 - MCP: server-provided tools.
 - Browser: page open, inspect, screenshot, content fetch.
 - Artifact: create, render, export.
 - Session: summarize, compact, fork, handoff.

Functional behavior:
 - Every tool call is logged.
 - High-risk calls require approval unless policy allows them.
 - Tool results can create artifacts.
 - Tool calls show status, stdout/stderr, affected files, duration, and approval decision.

## Git And Worktrees

Functional behavior:
 - Show Git status and diffs.
 - Support creating isolated worktrees per session.
 - Associate each worktree with parent project, branch, session, and cleanup policy.
 - Allow user review before merging, committing, or deleting worktrees.
 - Never run destructive Git operations without explicit approval.

## MCP Integration

Functional behavior:
 - Configure stdio and remote MCP servers.
 - Show server health, exposed tools, resources, prompts, and required credentials.
 - Apply per-server and per-tool permissions.
 - Support MCP roots by exposing only approved project roots.
 - Require approval for tool calls and sampling requests based on policy.

## Skills

Functional behavior:
 - Discover skills from global, project, and plugin directories.
 - Read `SKILL.md` frontmatter for catalog display.
 - Load skill body only when invoked or selected.
 - Preserve folder structure for `scripts/`, `references/`, and `assets/`.
 - Allow explicit invocation from the composer.

## Plugins

Functional behavior:
 - Install plugins from local folders and marketplace entries.
 - Validate required `.codex-plugin/plugin.json`.
 - Discover bundled skills, MCP config, assets, scripts, app metadata, and marketplace UI metadata.
 - Show plugin requested permissions before enablement.
 - Allow enable/disable per project and globally.

## Artifacts

Functional behavior:
 - Generate artifact records for created files, rendered previews, charts, documents, images, web pages, logs, diffs, and structured outputs.
 - Show artifact provenance from session and tool call.
 - Support open, reveal in file browser, export, duplicate, and delete.

## Browser And Web Viewing

Functional behavior:
 - Open local app URLs, generated HTML files, remote pages, and documentation pages.
 - Support screenshots and DOM/text extraction through an approved browser tool.
 - Keep browser content isolated from backend permissions.

## Model Providers

Functional behavior:
 - Use OpenRouter as the primary provider gateway.
 - Support OpenRouter credits and BYOK provider keys.
 - Support provider/model presets per project and per agent.
 - Show model context limits, tool support, streaming support, pricing source, and provider route when available.

## Approvals

Functional behavior:
 - Show approval prompts with command/tool name, arguments, target resources, risk, policy source, and expected effect.
 - Support one-time allow, session allow, project allow, deny, and always deny.
 - Keep an approval audit trail.
