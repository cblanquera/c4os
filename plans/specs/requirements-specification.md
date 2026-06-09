# Requirements Specification

This document defines product requirements for the AI workspace before implementation.

## Product Scope

The product is a general-purpose desktop AI workspace for coding, writing, research, analysis, operations, documentation, and other agent-assisted workflows. The MVP should not implement a custom agent runtime if OpenCode Runtime can satisfy the core requirements.

## Core User Requirements

 - Users can register multiple local projects.
 - Users can create multiple sessions per project.
 - Users can run multiple concurrent agents.
 - Users can inspect, pause, resume, stop, and archive sessions.
 - Users can grant local file access scoped to project roots.
 - Users can run local shell commands through an approval-aware tool model.
 - Users can connect Git repositories and inspect diffs.
 - Users can isolate risky or parallel file edits in Git worktrees.
 - Users can configure MCP servers.
 - Users can install and invoke skills.
 - Users can use project instructions through `AGENTS.md`.
 - Users can install plugins from local or marketplace sources.
 - Users can generate, view, export, and reopen artifacts.
 - Users can browse local files and generated outputs.
 - Users can view web/browser content in-app.
 - Users can switch models and providers.
 - Users can use OpenRouter with BYOK provider credentials.
 - Users can keep workflows local-first whenever possible.
 - Users can review and approve risky actions.
 - Users can audit what agents did.

## Non-Functional Requirements

 - Interoperability: prefer AGENTS.md, Agent Skills, MCP, Codex-compatible plugins, and OpenCode-compatible configuration where practical.
 - Local-first: project files, session metadata, artifacts, and logs should live locally by default.
 - Security: use least privilege, explicit trust boundaries, per-tool approvals, and encrypted secret storage.
 - Durability: sessions, tool calls, artifacts, approvals, and summaries must survive app restart.
 - Explainability: every file edit, shell command, MCP call, approval, and plugin action should be traceable.
 - Extensibility: plugins should add skills, MCP servers, UI metadata, commands, assets, and app connectors without modifying core code.
 - Portability: users should be able to export skills, plugins, MCP config, sessions, and artifacts.

## MVP Requirements

 - Tauri desktop shell with side navigation, project list, session list, main conversation, tool activity, file browser, artifact viewer, and settings.
 - OpenCode Runtime integration as the execution engine.
 - OpenRouter provider configuration with BYOK-ready model selection.
 - Local SQLite metadata store.
 - Project registration and workspace root policy.
 - Session persistence and resumability.
 - `AGENTS.md` discovery and display.
 - Skill discovery from user, project, and plugin paths.
 - MCP server configuration and tool listing.
 - Shell/file/Git approval workflows.
 - Git diff viewer and worktree lifecycle management.
 - Local artifact store and preview system.
 - Plugin install from local path and marketplace manifest.

## Out Of Scope For MVP

 - Building a new agent runtime from scratch.
 - Hosted multi-user collaboration.
 - Enterprise admin console.
 - Full cloud sync.
 - Public marketplace publishing workflow.
 - Complex workflow automation designer.
 - Mobile clients.

## Architectural Recommendation

Use `AI GUI -> OpenCode Runtime -> OpenRouter (BYOK)` for MVP.

Alternatives considered:
 - Custom runtime: maximum control, but high implementation and safety cost.
 - Direct model API orchestration: simpler than a full runtime, but would require rebuilding tools, permissions, sessions, and agent loops.
 - Codex CLI/runtime dependency: strong conventions, but less suitable if the product objective is model/provider portability through OpenRouter.

Why selected: OpenCode already covers agents, tools, permissions, sessions, MCP, and provider abstraction better than a greenfield runtime.
