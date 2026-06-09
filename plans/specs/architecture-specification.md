# Architecture Specification

## Recommended Architecture

```text
Tauri Desktop GUI
  -> Workspace Core (Rust)
  -> OpenCode Runtime Adapter
  -> OpenRouter Provider Layer
  -> Model Providers via OpenRouter BYOK or OpenRouter credits
```

The GUI should be a Tauri app using a web frontend and Rust backend. The backend owns filesystem access, process management, PTY handling, SQLite persistence, secret storage, plugin validation, and policy enforcement. OpenCode Runtime should perform agent execution, tool orchestration, model-provider integration, MCP integration, and permission-aware tool use where available.

## Major Components

 - Desktop UI: navigation, session UI, file browser, diff viewer, artifact viewer, browser panel, settings.
 - Workspace Core: projects, sessions, artifacts, policies, plugin registry, skill registry, runtime process supervision.
 - Runtime Adapter: translates GUI requests into OpenCode operations and normalizes events back into the app event model.
 - Policy Engine: evaluates workspace trust, project policy, agent policy, plugin policy, MCP policy, and per-call approvals.
 - Tool Ledger: append-only record of tool calls, arguments, approvals, outputs, and artifacts.
 - Artifact Service: stores files, previews, metadata, and provenance.
 - Secret Service: stores API keys and provider credentials in OS keychain or encrypted local vault.
 - MCP Manager: registers servers, monitors health, scopes roots, exposes tool metadata.
 - Plugin Manager: validates manifests, installs bundles, resolves marketplace entries, indexes skills and MCP config.

## Tauri Versus Electron

Recommendation: use Tauri for MVP.

Alternatives considered:
 - Electron: mature, consistent Chromium runtime, broad extension ecosystem, easier Node integration.
 - Tauri: smaller footprint, Rust backend, capability system, native integration, stronger default fit for local-first tools.
 - Native desktop stack: best OS integration, too expensive across macOS, Windows, and Linux.

Tradeoffs:
 - Tauri uses system WebViews, so rendering and browser feature parity vary by OS.
 - Electron bundles Chromium, giving consistent rendering and DevTools behavior.
 - Tauri's Rust backend and capabilities match a security-sensitive local agent workspace.
 - Electron can be secured, but requires disciplined context isolation, sandboxing, disabled Node integration for remote content, and careful IPC.

Why selected: Tauri better matches the desired footprint, local-first posture, Rust process control, and security boundary requirements.

## Runtime Strategy

Recommendation: use OpenCode Runtime through a narrow adapter instead of building a custom runtime.

Alternatives considered:
 - Custom runtime: best product control, highest risk and duplicated effort.
 - Direct OpenRouter calls: easy chat MVP, insufficient for multi-agent tools, permissions, sessions, MCP, and worktrees.
 - Embed multiple runtimes immediately: maximum compatibility, too much complexity for MVP.

Tradeoffs:
 - The app depends on OpenCode runtime stability and extension points.
 - Some UX features may need adapter work if OpenCode exposes them as CLI/TUI assumptions.
 - Avoiding a custom runtime significantly reduces initial risk.

Why selected: OpenCode already models agents, tools, permissions, MCP, providers, and sessions close to the desired product.

## Process Model

 - GUI process: web frontend.
 - Tauri core process: trusted backend and IPC boundary.
 - Runtime worker processes: one or more OpenCode runtime instances per active session or project.
 - Tool subprocesses: shell commands, MCP stdio servers, Git commands, renderers.

The backend must supervise runtime processes, stream events to the UI, and terminate child processes when sessions stop.

## Data Flow

1. User sends a message in a session.
2. UI sends the request to Workspace Core.
3. Policy Engine attaches allowed tools, roots, model, agent, and approval mode.
4. Runtime Adapter invokes OpenCode.
5. OpenCode emits model deltas, tool-call proposals, tool results, file edits, and summaries.
6. Tool Ledger persists every event.
7. Artifact Service indexes generated outputs.
8. UI renders conversation, activity, diffs, and artifacts.

## Local Storage

Use SQLite for metadata, a content-addressable artifact directory for files/previews, and OS keychain for secrets.

Alternatives considered:
 - JSON files only: portable, but weak for concurrent sessions and queries.
 - Embedded document DB: flexible, but unnecessary for structured metadata.

Why selected: SQLite is reliable, local-first, inspectable, and suitable for transactional session state.

## Interoperability Boundaries

 - Instructions: `AGENTS.md` first, import other instruction files as compatibility sources.
 - Skills: Agent Skills folder structure.
 - External integrations: MCP.
 - Plugins: Codex-compatible plugin layout with app-specific namespaced extensions.
 - Providers: OpenRouter-compatible model IDs and routing.
