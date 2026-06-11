# Requirements Specification

This document defines product requirements for the AI workspace before implementation.

## Product Scope

The MVP is a coding-first desktop AI workspace for technical users working in local Git repositories. It validates whether a desktop control center improves trust, persistence, inspection, and control for one selected Git project and one active agent session at a time.

The broader product may later expand into writing, research, analysis, operations, documentation, and other agent-assisted workflows, but those workflows are not MVP requirements.

## Core User Requirements

 - Users can register local Git projects and keep one selected project active at a time.
 - Users can start, resume, and stop the current session for the selected project.
 - Users can grant local file access scoped to project roots.
 - Users can run local shell commands through an approval-aware tool model.
 - Users can inspect Git status, changed files, and diffs.
 - Users can view root `AGENTS.md` project instructions.
 - Users can capture basic text, log, diff, and generated-file artifacts.
 - Users can select models through OpenRouter.
 - Users can use OpenRouter with BYOK provider credentials.
 - Users can keep workflows local-first whenever possible.
 - Users can review and approve risky actions.
 - Users can audit what agents did.

## Non-Functional Requirements

 - Interoperability: prefer root AGENTS.md, OpenCode-compatible execution, and OpenRouter-compatible model IDs for MVP.
 - Local-first: project files, session metadata, artifacts, and logs should live locally by default.
 - Security: use least privilege, explicit trust boundaries, per-tool approvals, and encrypted secret storage.
 - Durability: sessions, tool calls, artifacts, and approvals must survive app restart.
 - Explainability: every file edit, shell command, Git action, and approval should be traceable.

## MVP Requirements

 - Tauri desktop shell with project selection, main conversation, tool activity, read-only file browser, Git diff viewer, artifact viewer, and settings.
 - OpenCode Runtime integration as the execution engine.
 - OpenRouter provider configuration with BYOK-ready model selection.
 - Local SQLite metadata store.
 - Local Git project registration and selected-project root boundary.
 - Session persistence and resumability.
 - Root `AGENTS.md` discovery and display.
 - Shell/file/Git approval workflows.
 - Git status, changed-file list, and diff viewer.
 - Local artifact store for text, logs, diffs, and generated files.

## Out Of Scope For MVP

 - Building a new agent runtime from scratch.
 - Hosted multi-user collaboration.
 - Enterprise admin console.
 - Full cloud sync.
 - Public marketplace publishing workflow.
 - Plugin install from local path or marketplace manifest.
 - MCP server configuration and tool listing.
 - Skill discovery and invocation.
 - Git worktree lifecycle management.
 - Multiple active sessions or multiple concurrent agents.
 - Non-Git project workflows.
 - Complex workflow automation designer.
 - Mobile clients.

## Architectural Recommendation

Use `AI GUI -> OpenCode Runtime -> OpenRouter (BYOK)` for MVP.

Alternatives considered:
 - Custom runtime: maximum control, but high implementation and safety cost.
 - Direct model API orchestration: simpler than a full runtime, but would require rebuilding tools, permissions, sessions, and agent loops.
 - Codex CLI/runtime dependency: strong conventions, but less suitable if the product objective is model/provider portability through OpenRouter.

Why selected: OpenCode already covers agents, tools, permissions, sessions, MCP, and provider abstraction better than a greenfield runtime.
