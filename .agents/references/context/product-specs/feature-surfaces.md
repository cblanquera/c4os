# Feature Surfaces

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Imported from human planning sources and reconciled with accepted research decisions and POC results.

## Summary

C4OS is defined by its product surfaces: workspace/project management, sessions and agent runs, runtime orchestration, approvals, providers/models, files, artifacts, Browser, Terminal, settings/scoping, extensions, and local memory. These surfaces are product-owned even when implementation delegates to runtime adapters or native shells.

## Workspace And Projects

- Users open, create, clone, trust, inspect, remove, and switch local project folders.
- Trust unlocks local file, Git, shell, instruction, skill, approval-scoped, and runtime workflows.
- Multiple trusted folders are supported without requiring files to move or projects to adopt a proprietary layout.

## Approval And Security

The approval layer supports:

- allow, ask, and deny modes
- per-project defaults and per-session overrides
- scoped remembered rules
- pending approval queue
- clear action descriptions
- affected files, commands, tools, providers, or roots
- audit log entries for approved, denied, remembered, cancelled, and failed actions

The policy engine evaluates sensitive operations before runtime or tool execution. Runtime permission requests may feed the UI, but are not the only enforcement layer.

## Providers And Models

Provider support starts with OpenAI-compatible BYOK profiles. Built-in types include OpenAI, OpenRouter, Hugging Face router, LiteLLM proxy, and custom OpenAI-compatible endpoint.

Provider/model settings include:

- enable/disable state
- credential setup and secure key storage status
- base URL where applicable
- model discovery or manual model entry
- model enable/disable state
- model search
- per-session model selection
- tested connection status
- hidden provider inputs auto-filled for built-in types on save

## File Explorer And Editor

The file surface should feel familiar to VS Code users:

- trusted-root file tree
- recursive folder navigation
- expandable folders
- hidden files and folders visible except `.git`
- recognizable icons for known file types
- compact metadata
- Git-aware colors when reliable Git state exists
- safe degradation when Git state is unavailable

The editor supports trusted-root loading, code highlighting, Markdown/code rendering where appropriate, clickable breadcrumbs, dirty state, Save, Revert, New File, New Folder, Rename, guarded Delete with exact-path confirmation, external change detection, Reload, Keep Draft, multi-tab editing, and active-file literal search with next/previous navigation.

File editing is user-controlled. Agent-initiated mutation requires separate approval-boundary work.

## Artifacts

Artifacts are first-class outputs, not incidental chat blobs. Supported artifact types include Markdown, text, code, diffs, generated HTML, images, PDFs, downloadable files, command logs, and research or analysis outputs.

Artifact records preserve project/session association, creation time, originating run, export/reveal options, and safe viewer type. Generated or untrusted HTML renders in isolated preview without privileged app APIs. External web browsing is visually and technically separate from local artifact preview.

## Browser

The Browser surface is a user-owned desktop surface for approved local preview and web URLs. It must not expose provider credentials, arbitrary workspace files, shell state, or privileged app APIs to page content.

Browser promotion requires a designed permission model and cross-platform isolation proof. Native WebView work should not proceed until product Browser content has no privileged Wry IPC handler.

## Terminal

The Terminal surface is backend-owned, not renderer-owned. It requires trusted-root cwd validation, approval-gated launch for high-impact commands, sanitized environment, PTY spawn behind approved launch plans, backend-owned lifecycle, bounded renderer event transport and backpressure, input/output/resize/cancel/disconnect/cleanup/exit events, and audit records.

SSH, remote shells, containers, multiplexing, agent auto-run, and arbitrary renderer shell spawn are future expansion and should not enter scope accidentally.

## Settings And Scoping

Settings cover providers, models, approval modes, saved approval rules, configuration, plugins, skills, MCP servers, runtime configuration, and customizations.

Scoping must make clear whether a setting applies to app, workspace, project, session, runtime, provider, extension, or model. Runtime configuration scoping includes OpenCode and Pi.

## Standards, Extensions, And Marketplace Readiness

C4OS discovers, displays, and respects compatible instruction and skill files including `AGENTS.md`, `AGENTS.override.md`, `SKILL.md`, `.opencode/skills`, `.pi/skills`, `.claude/skills`, and `.agents/skills`.

The extension inventory covers MCP Servers, Skills, Plugins, and related scoped surfaces. It shows provenance, install source, requested permissions, trust state, enabled state, risk/approval requirements, scope, and runtime impact.

Extensions are disabled by default before they affect runtime execution, model context, tools, or app-owned state. Harmless MCP connection should land before executable plugin loading. Marketplace readiness is an architecture goal, not MVP scope.

## Local Memory

Local memory is app-owned and scoped to workspace, project, or session. It may include session summaries, pinned facts, retrieval cues, project preferences, recurring task context, and user-approved durable notes.

Memory stays separate from raw provider state and runtime-local session storage. Users should understand what is remembered and where it is scoped.
