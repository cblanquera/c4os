# Evidence

## EVD-001: MVP And V1 Progress Verified

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/progress/manifest.md`
Related:
- DEC-001

### Statement

The active progress manifest records `item-001` through `item-048` as verified,
with no unverified V1 roadmap item currently recorded.

## EVD-002: V1 Deferred Broad Capabilities

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `.agents/specs/c4os-mvp/records/decisions.md`
Related:
- REQ-004

### Statement

Accepted V1 decisions defer worktree lifecycle, browser/web viewing,
plugin/marketplace workflows, direct/local provider expansion, durable memory,
and broad compatibility claims.

## EVD-003: Original Prompt Requested General Workspace

Status: accepted
Confidence: proposed
MVP: no
Source: user-provided original project prompt
Related:
- REQ-001

### Statement

The original project prompt requested a general-purpose AI workspace supporting
coding, writing, research, analysis, operations, documentation, and other
agent-assisted workflows.

## EVD-004: MCP Latest Specification And Trust Model

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://modelcontextprotocol.io/specification/2025-11-25`
Related:
- TASK-000
- RISK-005

### Statement

The Model Context Protocol specification page identifies version `2025-11-25`
as latest and defines MCP around JSON-RPC, hosts, clients, servers, resources,
prompts, tools, sampling, roots, elicitation, and explicit security principles
for consent, privacy, tool safety, and sampling control.

## EVD-005: AGENTS.md Current Open Format

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://agents.md/`
Related:
- TASK-000
- RISK-005

### Statement

The AGENTS.md site describes AGENTS.md as an open Markdown format for coding
agent instructions, with common sections such as build/test commands and
security considerations, nested AGENTS.md behavior, and conflict precedence
where explicit user prompts override file instructions.

## EVD-006: OpenAI Agent Skills Current Shape

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/api/docs/guides/tools-skills`
Related:
- TASK-000
- RISK-005

### Statement

OpenAI documentation describes Agent Skills as reusable versioned bundles of
files with a `SKILL.md` manifest, compatible with the open Agent Skills
standard, and usable in local or hosted execution modes.

## EVD-007: OpenAI MCP Approval Guidance

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/api/docs/guides/tools-connectors-mcp`
Related:
- TASK-000
- RISK-001
- RISK-005

### Statement

OpenAI MCP guidance defaults to approval before data is shared with connectors
or remote MCP servers, recommends reviewing/logging shared data, and warns
about prompt injection, sensitive actions, untrusted URLs, and third-party MCP
server trust.

## EVD-008: Codex Worktree Product Pattern

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/codex/app/worktrees`
Related:
- TASK-005
- RISK-005

### Statement

OpenAI Codex app documentation describes worktrees as a way to run independent
tasks in the same Git project, with handoff between local checkout and
worktree, detached-head defaults, cleanup limits, pinned/in-progress/permanent
worktree protections, and restore behavior after deletion.

## EVD-009: Codex Surface Includes Plugins And App Features

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/codex`
Related:
- TASK-006
- RISK-005

### Statement

The OpenAI Codex documentation navigation exposes current product surfaces for
worktrees, local environments, in-app browser, Chrome extension, computer use,
permissions, hooks, AGENTS.md, MCP, plugins, skills, subagents, and agent
approvals/security.

## EVD-010: Validation Manual Decision Split

Status: accepted
Confidence: inferred
MVP: no
Source: validation pass
Related:
- Q-001
- Q-002
- Q-003
- Q-004

### Statement

Validation can recommend defaults for standards refresh scope and feature-spec
split boundaries, but it cannot infer the user's desired first non-coding
workflow or accept the next phase order without explicit user confirmation.

## EVD-011: User Accepted Next Phase And Audience Priority

Status: accepted
Confidence: evidence-backed
MVP: no
Source: user decision 2026-06-12
Related:
- Q-001
- Q-003
- DEC-004
- DEC-006
- TASK-000
- TASK-001

### Statement

The user accepted `TASK-000` standards refresh as the next phase and selected
research, writing, documentation, and analysis as the first general-workspace
audience priorities.

## EVD-012: Agent Skills Standard Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://agentskills.io/home`, checked 2026-06-12
Related:
- TASK-000
- CAP-006

### Statement

Agent Skills are currently documented as an open, lightweight folder format
centered on `SKILL.md` metadata and instructions, with optional scripts,
references, assets, and progressive disclosure stages for discovery,
activation, and execution.

## EVD-013: AGENTS.md Standard Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://agents.md/`, checked 2026-06-12
Related:
- TASK-000
- CAP-001

### Statement

AGENTS.md is currently documented as a standard Markdown instruction file for
agents, commonly covering project overview, build/test commands, code style,
testing, security, nested subproject instructions, and precedence where user
prompts override file instructions.

## EVD-014: MCP Standard Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://modelcontextprotocol.io/specification/2025-11-25`, checked 2026-06-12
Related:
- TASK-000
- CAP-006

### Statement

The current MCP specification version checked is `2025-11-25`. MCP defines a
JSON-RPC host/client/server model with resources, prompts, tools, sampling,
roots, elicitation, progress, cancellation, logging, error reporting, and
security guidance around consent, privacy, tool safety, and sampling controls.

## EVD-015: OpenAI MCP Approval Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/api/docs/guides/tools-connectors-mcp`, checked 2026-06-12
Related:
- TASK-000
- RISK-001

### Statement

OpenAI MCP guidance defaults to approvals before sharing data with connectors
or remote MCP servers, recommends reviewing and logging shared data, and calls
out prompt injection, sensitive action approvals, URL trust, and third-party
server trust as key risks.

## EVD-016: Codex Plugin Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/codex/plugins`, checked 2026-06-12
Related:
- TASK-000
- CAP-006

### Statement

Codex plugins currently bundle skills, app integrations, and MCP servers into
reusable workflows. Codex plugin surfaces include curated/shared/created
plugin directories, install/uninstall, enable/disable, explicit `@` invocation,
external app authentication, and continuing approval settings plus app-specific
terms and data-sharing policies.

## EVD-017: Codex Worktree Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://developers.openai.com/codex/app/worktrees`, checked 2026-06-12
Related:
- TASK-000
- CAP-005

### Statement

Codex worktrees currently support independent tasks in the same Git project,
local/worktree handoff, detached-head defaults, branch limitations, managed and
permanent worktrees, cleanup limits, protections for pinned/in-progress/
permanent worktrees, and snapshot/restore behavior.

## EVD-018: OpenCode Runtime Surface Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://opencode.ai/docs/`, checked 2026-06-12
Related:
- TASK-000
- RISK-004

### Statement

OpenCode is currently documented as an open source AI coding agent available as
a terminal interface, desktop app, or IDE extension. It supports provider
configuration, project initialization that creates `AGENTS.md`, Plan mode,
file references, image input, undo/redo, sharing, and customization.

## EVD-019: OpenCode Permission And MCP Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://opencode.ai/docs/permissions/`, `https://opencode.ai/docs/mcp-servers/`, checked 2026-06-12
Related:
- TASK-000
- RISK-001

### Statement

OpenCode currently documents permission actions as allow, ask, or deny, with
granular rules for tools and external directories. It also documents local and
remote MCP servers, automatic MCP tool availability after configuration, and
context-size caveats from enabled MCP tools.

## EVD-020: OpenRouter Provider Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://openrouter.ai/docs/quickstart`, `https://openrouter.ai/docs/api/reference/overview`, `https://openrouter.ai/docs/guides/routing/provider-selection`, checked 2026-06-12
Related:
- TASK-000
- CAP-004

### Statement

OpenRouter currently documents a unified API for hundreds of models, model
fallbacks, provider selection, BYOK credential APIs, models/providers
endpoints, OpenAI SDK compatibility, client SDKs, an Agent SDK, workspaces,
presets, tool calling, web search/fetch server tools, guardrails, and logging
features.

## EVD-021: Tauri Security Refresh

Status: accepted
Confidence: evidence-backed
MVP: no
Source: `https://v2.tauri.app/security/`, `https://v2.tauri.app/plugin/shell/`, checked 2026-06-12
Related:
- TASK-000
- RISK-001
- RISK-004

### Statement

Tauri v2 security guidance emphasizes the trust boundary between Rust core code
and frontend WebView code, IPC-controlled access to system resources, capability
configuration for exposed commands, operating-system WebView tradeoffs, and
shell plugin access for spawning child processes.

## EVD-022: Browser Artifact Memory Source Gaps

Status: accepted
Confidence: evidence-backed
MVP: no
Source: standards refresh 2026-06-12
Related:
- TASK-000
- TASK-003
- TASK-007

### Statement

The shallow refresh found current navigation/source signals for Codex in-app
browser, app features, memories, and OpenRouter web/PDF/multimodal surfaces,
but did not complete deep source review for browser execution isolation,
artifact preview formats, document/spreadsheet handling, or durable memory data
models. These require feature-specific specs before implementation.
