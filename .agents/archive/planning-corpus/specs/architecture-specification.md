# Architecture Specification

## Recommended Architecture

```text
Tauri Desktop GUI
  -> Workspace Core (Rust)
  -> OpenCode Runtime Adapter
  -> OpenRouter Provider Layer
  -> Model Providers via OpenRouter
```

The GUI should be a Tauri app using a web frontend and Rust backend. For MVP, the backend owns filesystem access, process management, PTY handling, SQLite persistence, secret storage, and the Approval Gateway for file writes, shell commands, and Git state-changing actions. OpenCode Runtime should perform agent execution, tool orchestration, model-provider integration, and permission-aware tool use where available, but MVP execution must not bypass the backend Approval Gateway. UI-only approvals, post-execution audit, terminal scraping, or best-effort observation are not acceptable enforcement mechanisms for MVP.

The app owns canonical MVP records for projects, sessions, messages, tool calls, approvals, artifacts, models, and settings. OpenCode runtime state is adapter-owned execution state. If OpenCode exposes runtime session IDs, logs, or persistence files, the app may store references to them, but user-facing history and inspection come from app-owned records.

MVP OpenCode runtime settings are app-owned adapter plumbing only. The app may store runtime references and launch settings required to invoke OpenCode, but it does not import, mirror, edit, export, or round-trip OpenCode config. Any existing OpenCode config behavior that affects runtime execution must be discovered in Phase 0 and disclosed.

App-owned provider and model settings are authoritative for MVP sessions. The selected OpenRouter model is fixed at session creation and remains fixed even when the session is idle or has transcript history but no active run. Project default model changes apply only to future sessions, including when changed while a session is running. Such changes must not alter the running session's credential/model reference, restart the run, reconfigure the runtime, or migrate an in-flight model call. The Runtime Adapter must launch or configure OpenCode so the effective session model matches the session's selected OpenRouter model. If existing OpenCode config can override provider or model routing and the app cannot detect or prevent that override, OpenCode is not viable as the direct MVP runtime.

OpenRouter credential updates and revocation are allowed only when no session is running. A running session keeps the credential reference it started with until the session is stopped or complete. The Runtime Adapter must not hot-swap credentials, switch credential references per session, or retry a failed in-flight model call with a newly entered key.

## Major Components

 - Desktop UI: navigation, session UI, file browser, diff viewer, artifact viewer, settings.
 - Workspace Core: active project, active session, artifacts, approvals, runtime process supervision.
 - Runtime Adapter: translates GUI requests into OpenCode operations, stores runtime references, and normalizes events back into the app-owned event model.
 - Approval Gateway: classifies MVP file write, shell, and Git state-changing actions; requests user approval when required; records answered approval decisions; blocks denied actions.
 - Tool Ledger: append-only record of tool calls, model-call context-source summaries, arguments, answered approvals, outputs, and artifacts. Answered approvals use structured metadata plus bounded redacted summary or diff references. Pending approval prompts and full prompt replay blobs are not durable ledger records.
 - Artifact Service: stores files, previews, metadata, and provenance.
 - Secret Service: stores the OpenRouter API key in the OS keychain or platform credential store.

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
 - Runtime worker process: one OpenCode runtime instance for the active MVP session.
 - Tool subprocesses: approved shell commands, Git commands, and artifact renderers.

The backend must supervise runtime processes, stream events to the UI, and terminate child processes when sessions stop.

The MVP stop control cancels the active runtime/model stream and supervised child processes owned by the app, then records the run as stopped while preserving app-owned transcript, tool history, approvals, and artifacts. Stop is not session deletion. Pause/resume stream, background continuation, partial-response regeneration controls, and killing arbitrary external processes are post-MVP.

Minimizing the app window does not stop the active run while the app process remains alive. Closing or quitting the app stops the active run through the same stop control before shutdown and preserves app-owned session history. MVP does not include a system tray daemon, background agent service, OS notifications, scheduled runs, or wake/resume automation.

If the app closes, crashes, or is force-quit while an approval prompt is pending, the pending approval is discarded with the stopped or interrupted run state. If the app crashes or is force-quit during an active run, the next launch must mark the previous run as interrupted/crashed using the last persisted app-owned records. The MVP does not automatically reconnect to the prior runtime, reattach to processes, replay recovery, continue generation, approve after restart, replay stale approval prompts, or resend unsent prompts. Cleanup of app-owned child processes after a crash is best effort only.

## Data Flow

1. User sends a message in a session.
2. UI sends the request to Workspace Core.
3. Workspace Core attaches the project root, selected model, internal runtime agent reference if required, and MVP approval mode.
4. Runtime Adapter invokes OpenCode.
5. OpenCode emits model deltas, tool-call proposals, file edit proposals, and summaries.
6. Approval Gateway approves or blocks MVP file write, shell, and Git state-changing actions before execution.
7. Approved actions execute and return tool results.
8. Workspace Core persists canonical messages, tool calls, approvals, and status changes.
9. Tool Ledger persists every tool event.
10. Artifact Service indexes generated outputs.
11. UI renders conversation, activity, diffs, and artifacts from app-owned records.

If step 6 cannot be enforced through OpenCode directly, Phase 0 must choose a wrapper, proxy, fork, runtime replacement, or MVP scope reduction before product implementation continues.

## Model Context Boundary

MVP model calls through OpenRouter may include only:

 - Active session transcript.
 - Selected model and routing metadata.
 - User-approved or policy-allowed tool results and summaries.
 - File contents explicitly read by the runtime inside the selected project root.

Root `AGENTS.md` may enter model context only through the same explicit project-root file-read path as other allowed project files. MVP must not perform whole-repo indexing, hidden background file ingestion, automatic app-owned root `AGENTS.md` injection, app-authored system prompt injection, editable project prompt injection, or automatic artifact-content ingestion into model context. Artifact contents are sent only when referenced by the user or explicitly read by the runtime under normal file-read rules. Secret-deny file contents are never included in model context.

MVP does not require per-call user approval before sending model context through OpenRouter. Provider setup provides the standing disclosure that prompts and bounded context leave the machine through OpenRouter. Session activity records a bounded context-source summary for model calls, such as active transcript, selected model/routing metadata, explicit file reads, and tool summaries. Raw prompt export, token-by-token context inspection, and editable context composition are post-MVP.

Session records may include the selected model and best-effort metadata source/freshness when available, but MVP does not implement per-call token accounting, cost estimation, budget meters, spend history, or budget enforcement. OpenRouter route variability and metadata uncertainty are represented through source/freshness labels rather than detailed billing UI.

OpenRouter setup validates that a credential exists and that the selected model route is viable enough to start sessions. MVP does not fetch or display OpenRouter credit balance, billing links, top-up flows, invoice links, spend warnings, or account diagnostics.

If OpenRouter or network access fails during a session, new model calls fail closed with a clear provider or network error. Once Workspace Core appends a submitted user message, provider failure must not delete or rewrite it. Workspace Core persists failed assistant/run status and preserves app-owned transcript, tool history, approvals, and artifacts so the user can explicitly retry after recovery. A retry control, if present, creates a new appended retry action/status before starting a new model call; it must not regenerate in place, replace the failed assistant record, or branch from the failed response. The Runtime Adapter must not silently retry, resend prompts, switch provider routes, switch models, queue background resends, or synthesize assistant continuation after provider failure.

## Local Storage

Use SQLite for metadata, a content-addressable artifact directory for files/previews, and OS keychain for secrets.

Alternatives considered:
 - JSON files only: portable, but weak for concurrent sessions and queries.
 - Embedded document DB: flexible, but unnecessary for structured metadata.

Why selected: SQLite is reliable, local-first, inspectable, and suitable for transactional session state.

## Interoperability Boundaries

 - Instructions: root `AGENTS.md` display only for MVP. The app does not automatically inject `AGENTS.md`, app-authored system prompts, project prompts, prompt templates, or instruction composer output into model context and does not let them affect permissions. Explicit reads of root `AGENTS.md` are normal logged project-root file reads, not policy loading. Agent-proposed writes to root `AGENTS.md` are normal approval-gated project-root file writes; successful writes do not automatically reload instructions into model context or permission state. Any OpenCode-native `AGENTS.md` or instruction-file loading must be observed and disclosed, or disabled. If OpenCode invisibly loads root or nested instruction files and the app cannot observe, disclose, or disable that behavior, OpenCode is not viable as the direct MVP runtime.
 - Skills: root `AGENTS.md` display only for MVP; Agent Skills compatibility and skill creator workflows for explicit, reviewable instruction artifacts are post-MVP.
 - External integrations: MCP is post-MVP.
 - Plugins: Codex-compatible plugin layout with app-specific namespaced extensions is post-MVP.
 - Providers: OpenRouter-compatible model IDs and routing.

MVP compatibility claims are limited to OpenRouter-backed model access, local Git project support, root `AGENTS.md` display, and app-owned text-like artifact records. Full AGENTS.md compatibility, Agent Skills compatibility, MCP compatibility, Codex plugin compatibility, OpenCode config compatibility, import/export compatibility, and round-trip compatibility require the standards conformance matrix.
