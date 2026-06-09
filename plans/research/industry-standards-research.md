# Industry Standards Research

This document summarizes current conventions relevant to a general-purpose desktop AI workspace. The key conclusion is that the product should compose existing agent standards rather than define a new runtime format.

## Sources

 - AGENTS.md open format: https://agents.md/
 - OpenAI Codex plugins and skills: https://openai.com/academy/codex-plugins-and-skills/
 - OpenAI Codex plugin admin model: https://help.openai.com/en/articles/20001256-plugins-in-codex/
 - OpenAI Codex plugin and skill scaffolds: https://raw.githubusercontent.com/openai/codex/main/codex-rs/skills/src/assets/samples/plugin-creator/SKILL.md and https://raw.githubusercontent.com/openai/codex/main/codex-rs/skills/src/assets/samples/skill-creator/SKILL.md
 - OpenAI skills catalog: https://raw.githubusercontent.com/openai/skills/main/README.md
 - MCP 2025-06-18 specification: https://modelcontextprotocol.io/specification/2025-06-18
 - MCP tools specification: https://modelcontextprotocol.io/specification/2025-06-18/server/tools
 - OpenCode agents/tools/config docs: https://dev.opencode.ai/docs/agents/ and https://dev.opencode.ai/docs/tools/
 - OpenRouter BYOK docs: https://openrouter.ai/docs/guides/overview/auth/byok
 - Tauri capabilities/security docs: https://v2.tauri.app/security/capabilities/
 - Electron security docs: https://www.electronjs.org/docs/latest/tutorial/security
 - Codex desktop app announcement and workflow direction: https://openai.com/index/introducing-the-codex-app/

## AGENTS.md

AGENTS.md is an open Markdown convention for project-specific agent guidance. It is intentionally plain Markdown with no required fields, commonly covering setup commands, test commands, style rules, security notes, and PR guidance. The standard recommends nested AGENTS.md files for monorepos; the closest file to the edited path wins, while explicit user prompts override file guidance.

Recommendation: support root and nested `AGENTS.md` discovery exactly as a first-class convention.

Alternatives considered:
 - Proprietary `workspace.ai.md`: easier to control, but fragments interoperability.
 - Existing tool-specific files such as `CLAUDE.md` or `.cursor/rules`: useful imports, but not the primary neutral standard.

Why selected: AGENTS.md is broad, simple, shareable, and already recognized by multiple coding agents.

## SKILL.md And Agent Skills

Agent skills are folders containing a required `SKILL.md` with YAML frontmatter and Markdown instructions, plus optional `scripts/`, `references/`, and `assets/`. Codex skill guidance treats `name` and `description` as the core triggering metadata; the body loads only after the skill triggers. This progressive disclosure pattern keeps the active context small while allowing large supporting references or deterministic scripts to live beside the skill.

Recommendation: support Agent Skills as portable folders and avoid a proprietary workflow-package format for reusable procedures.

Alternatives considered:
 - Prompt library only: simple, but cannot bundle scripts, reference docs, or assets.
 - Plugin-only workflows: too heavy for simple process guidance.

Why selected: skills cleanly represent repeatable procedures across coding, writing, operations, research, and documentation workflows.

## MCP

MCP is an open JSON-RPC based protocol for connecting LLM hosts to external context and tools. Hosts contain MCP clients; servers expose resources, prompts, and tools. The 2025-06-18 spec defines client features including roots, sampling, and elicitation, and server features including resources, prompts, and tools. MCP explicitly calls for user consent, data privacy, human-in-the-loop tool safety, and sampling controls.

Recommendation: make MCP the default integration protocol for external tools, local services, and organization connectors.

Alternatives considered:
 - Custom plugin API only: more control, but worse ecosystem compatibility.
 - Direct SDK integrations only: useful for high-value native connectors, but hard to scale and share.

Why selected: MCP is the strongest current interoperability standard for agent tools and context.

## Tool Execution Models

OpenCode exposes built-in tools such as `bash`, `read`, `write`, `edit`, `apply_patch`, `grep`, `glob`, `webfetch`, `websearch`, `skill`, `task`, and LSP tools. Its permission model can set actions to `allow`, `ask`, or `deny`, with pattern-based controls for read/edit/bash-style operations. MCP tools use `tools/list` and `tools/call`, with JSON Schema input definitions and optional output schemas.

Recommendation: model every runtime capability as a tool invocation with a normalized policy envelope, regardless of whether it originates from OpenCode, MCP, plugin code, or the GUI.

Alternatives considered:
 - Special-case shell/filesystem as internal calls: faster initially, but hard to audit.
 - Treat MCP tools differently from local tools: clearer protocol boundary, but weaker unified approvals.

Why selected: a single tool ledger enables approvals, audit logs, replay, security review, and policy simulation.

## Codex Plugins And Marketplace

Codex plugins package capabilities for workflows. They may include skills, apps/connectors, app templates, MCP configuration, scripts, and assets. OpenAI help docs emphasize that plugins do not grant data access by themselves; users and admins must already have source-system access, and admins can control read/write behavior, action confirmation, RBAC, and source boundaries.

Current Codex plugin scaffold conventions include a required `.codex-plugin/plugin.json`, optional `skills/`, `hooks/`, `scripts/`, `assets/`, `.mcp.json`, and `.app.json`. Marketplace metadata uses a root `marketplace.json` with `name`, optional `interface.displayName`, ordered `plugins[]`, source metadata, category, and policy fields such as `installation` and `authentication`.

Recommendation: support Codex-compatible plugin manifests and marketplace metadata first, with additional app-specific metadata namespaced rather than replacing the core format.

Alternatives considered:
 - Build a fully custom marketplace: easier product control, worse portability.
 - Only install raw MCP servers: misses skills, assets, templates, and UX metadata.

Why selected: Codex plugin conventions already combine skills, app connectors, MCP, and marketplace policy in a way that maps well to a desktop AI workspace.

## Projects, Sessions, And Worktrees

Codex desktop positions itself as a command center for multiple agents and parallel work. OpenAI describes built-in worktree support so multiple agents can work in the same repository without conflicts. OpenAI Academy describes a Codex project as linked to a local folder and supports multiple concurrent tasks.

Recommendation: represent projects as registered local folders, sessions as durable conversations/runs under a project, and worktrees as isolated execution sandboxes for tasks that may modify files.

Alternatives considered:
 - One global chat history: simple, but weak for multi-project work.
 - One worktree per project only: safer than shared edits, but prevents parallel task isolation.

Why selected: project/session/worktree separation is proven in current agent coding tools and also generalizes to non-code folders.

## Local File Access

Modern agent tools use explicit workspace roots, ignore patterns, and external-directory gates. OpenCode exposes `external_directory` as a distinct permission. Tauri capabilities can constrain frontend access to backend/system commands by window and permission.

Recommendation: enforce root-scoped access by default, require explicit approval for external paths, and keep a per-project file-access policy separate from model context selection.

Alternatives considered:
 - Full home-directory access: convenient, too risky.
 - File picker only: safer, but too restrictive for agent workflows.

Why selected: root-scoped access balances local-first utility with understandable security boundaries.

## Permission And Approval Systems

Common patterns are `allow`, `ask`, and `deny`, plus trust levels, command patterns, write-action confirmation, and source boundaries. MCP requires hosts to provide clear consent and authorization UI, especially for tool calls and sampling. Tauri capabilities can further restrict frontend windows and remote content.

Recommendation: define a layered permission system: workspace trust, project policy, agent policy, plugin/MCP policy, per-tool risk classification, and per-call approval.

Alternatives considered:
 - Runtime-only prompts: easy to implement, poor for admin governance.
 - Static config only: auditable, but cumbersome for interactive work.

Why selected: layered policy allows safe defaults, admin control, and productive user overrides.

## Session And Memory Management

Emerging patterns include persistent sessions, automatic summaries, compaction, title generation, agent handoff, and durable project instructions. The product should separate session transcript, generated summaries, user memory, project instructions, and tool execution logs.

Recommendation: store durable memory only when explicitly configured, keep session summaries auditable, and never treat memory as a source of current truth.

Alternatives considered:
 - Fully automatic long-term memory: convenient, but privacy and correctness risks.
 - No memory: simpler, but weak for long-horizon work.

Why selected: explicit, scoped memory supports continuity without hiding state.

## Artifact Generation And Viewing

MCP tool results can return text, images, audio, structured content, resource links, and embedded resources. AI workspaces increasingly need first-class artifact viewers for documents, images, spreadsheets, slide decks, generated sites, logs, diffs, charts, and browser pages.

Recommendation: store artifacts as typed records with original files, rendered previews, provenance, and links to the session/tool call that generated them.

Alternatives considered:
 - Plain file attachments only: interoperable, but weak provenance and preview UX.
 - Database-only blobs: easier lifecycle management, worse user portability.

Why selected: typed artifact records preserve local files while enabling rich previews.

## Desktop Runtime Direction

Tauri offers a smaller Rust-centered desktop architecture with explicit capabilities and permissions. Electron offers mature Chromium/Node integration, consistent rendering, and a larger ecosystem, but larger bundles and a broader Node attack surface if misconfigured. Electron's own security guidance stresses disabling Node integration for remote content, enabling context isolation, sandboxing, and careful IPC validation.

Recommendation: choose Tauri for MVP unless OpenCode embedding or required browser automation demands Electron. Use a Rust backend with a web UI, native PTY/process management, local SQLite, and constrained command surfaces.

Alternatives considered:
 - Electron: best for Chromium consistency and Node-heavy extension ecosystems.
 - Native Swift/Kotlin/WinUI: strongest native UX, but high cross-platform cost.

Why selected: Tauri fits local-first, security-sensitive desktop tooling and aligns with Rust-based process isolation.
