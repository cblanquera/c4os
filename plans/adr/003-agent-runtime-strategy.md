# ADR-003: Agent Runtime Strategy

Status: Provisional.

## Context

The preferred architecture is `AI GUI -> OpenCode Runtime -> OpenRouter (BYOK)`. The requirements explicitly say the MVP should avoid building a custom runtime if OpenCode Runtime can satisfy the core requirements. The architecture recommends using OpenCode through a narrow adapter.

The review identifies this as a make-or-break assumption. A CLI/TUI-oriented runtime may not expose stable interfaces for event streaming, pre-tool approval interception, session resume, child sessions, worktree mapping, custom policy engines, plugin loading, artifact capture, or provider routing visibility.

## Decision

Use OpenCode Runtime as the first runtime target through a narrow adapter, but
do not use unconstrained direct OpenCode as the MVP runtime path.

Phase 0 validation selected a hardened OpenCode adapter fallback after strict
runtime config isolation failed. The MVP adapter must launch OpenCode with
app-owned runtime settings, isolated local runtime directories, and mandatory
instruction/config disclosure before session start. OpenCode-native project
instructions may be used only when the app has enumerated, disclosed, and
persisted the effective instruction sources in app-owned session records.

MVP stop behavior requires canceling the active runtime/model stream and terminating app-supervised child processes while preserving app-owned transcript, tool history, approvals, and artifacts. Partial assistant output is persisted as stopped or interrupted and is not auto-regenerated, auto-deleted, or silently treated as complete. Stop is not session deletion and does not include partial-response editing, branch/regenerate controls, automatic continuation, pause/resume stream, background continuation, partial-response regeneration controls, or killing arbitrary external processes.

## Alternatives Considered

 - OpenCode Runtime.
 - Custom runtime.
 - Direct OpenRouter calls.
 - Codex CLI/runtime dependency.
 - Multiple runtimes immediately.

## Alternatives That Should Be Considered

 - Runtime abstraction layer with OpenCode as the first adapter.
 - OpenCode fork if required hooks are unavailable.
 - App-owned local daemon API that can wrap OpenCode or another runtime later.
 - Direct provider runtime only for a smaller non-agent MVP.

## Tradeoffs

OpenCode accelerates MVP and aligns with agents, tools, permissions, MCP, providers, and sessions. It also creates dependency risk if the runtime is not designed for GUI embedding.

A custom runtime offers full control over policy, persistence, tools, and UX, but has the highest cost and correctness risk.

A runtime abstraction layer reduces lock-in but adds upfront architecture and can collapse into lowest-common-denominator design.

## Consequences

 - Runtime validation must precede UI buildout.
 - The app owns canonical MVP records for sessions, messages, tool calls, approvals, artifacts, projects, models, and settings rather than treating OpenCode storage as the source of truth.
 - OpenCode-native session IDs, logs, and persistence files are adapter references only.
 - If reliable pre-execution interception for file writes, shell commands, and Git state changes is unavailable, OpenCode is not viable as the direct MVP runtime.
 - If OpenCode cannot cancel active runs and terminate app-supervised child processes without deleting app-owned session history or losing partial assistant output state, OpenCode is not viable as the direct MVP runtime.
 - If OpenCode invisibly loads root or nested instruction files and the app cannot observe, disclose, or disable that behavior, OpenCode is not viable as the direct MVP runtime.
 - If OpenCode loads root or nested instruction files, the MVP may proceed only
   through the hardened adapter fallback: preflight instruction inventory,
   effective config capture, user-visible disclosure, and app-owned
   context-source persistence.
 - UI-only approval prompts, post-execution audit logs, terminal scraping, or best-effort observation do not satisfy the MVP security model.
 - A wrapper, proxy, fork, runtime replacement, or MVP scope reduction must be chosen before Phase 1 if the direct OpenCode path fails this gate.

## Follow-Up Questions

 - What exact OpenCode interface will be used: library API, JSON RPC server, CLI subprocess protocol, filesystem state, or fork?
 - Can every tool call be intercepted before execution?
 - Can OpenCode stream structured events without terminal scraping?
 - Can OpenCode cancel active model streams and terminate app-supervised child processes while preserving app-owned session records and partial assistant output status?
 - How does OpenCode persist sessions, and how are runtime references mapped to app-owned session records?
 - Which additional OpenCode instruction-bearing config fields must the
   hardened adapter disclose before session start?
 - Can multiple runtime instances run safely in parallel?

## ADR Recommendation

Keep this ADR open until the remaining pre-implementation gates pass. The
runtime-control evidence no longer supports unconstrained direct OpenCode, but
it does support a hardened OpenCode adapter with mandatory instruction/config
disclosure as the selected MVP fallback path.
