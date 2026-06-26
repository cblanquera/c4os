# Technical Specs

Status: active
Created: 2026-06-21
Updated: 2026-06-26
Source Note: Normalized from product-model, runtime-adapter, and constraints context. Detailed technical records are preserved under `.agents/references/context/technical-specs/`.

## Purpose

Use this as the technical gate. It summarizes product-owned system concepts, runtime boundaries, persistence, security, Browser constraints, Terminal constraints, and implementation locations before deeper technical references or specs are loaded.

## Load When

- You need architecture, product model, runtime adapter, persistence, secret handling, or implementation-location rules.
- You are working on Browser isolation, Terminal ownership, approval policy, provider credentials, extensions, or trusted-root behavior.
- You need to decide whether to open detailed technical references or POC evidence.

## Skip When

- You only need product thesis, users, goals, or vocabulary; load `product-brief.md`.
- You only need user-facing feature behavior or MVP feature inventory; load `product-specs.md`.
- You only need interface layout, interaction, visual, or accessibility guidance; load `creative-specs.md`.
- You only need sequencing, validation work, accepted decisions, or deferred work; load `work-orders.md`.

## Owns

- Product model, runtime adapter boundary, persistence, credential handling, approval/security constraints, Browser technical boundary, Terminal technical boundary, and implementation-location guardrails.

## Does Not Own

- Customer-facing feature prose, visual layout contract, accepted work-order status, active progress, or detailed spec acceptance.

## Reference Routing

| Need | Load | Why |
| --- | --- | --- |
| Entity model, workspace descriptors, sessions, runs, secrets | `.agents/references/context/technical-specs/product-model.md` | Use for data model and ownership questions. |
| OpenCode/Pi proof findings, adapter boundary, runtime implications | `.agents/references/context/technical-specs/runtime-adapter.md` | Use for runtime selection or adapter work. |
| Trust, credential, Browser, Terminal, extension, and validation constraints | `.agents/references/context/technical-specs/constraints.md` | Use for safety-sensitive implementation or review. |
| Skill, plugin, and MCP discovery/loading contract | `.agents/references/context/technical-specs/extension-loading.md` | Use before implementing extension install, discovery, loading, enablement, or invocation. |
| Source and artifact provenance | `.agents/references/context/source-provenance.md` | Use only when tracing where technical facts came from. |

## Summary

C4OS owns the product model and safety boundary even when it delegates execution to OpenCode, Pi, MCP servers, shells, Browser engines, or other runtime components. Runtime concepts should stay behind app-owned adapters unless exposed for advanced compatibility inspection.

## Product Model

Minimum product-owned entities include:

- Workspace
- Workspace file
- Project folder
- Trust record
- Session
- Agent run
- Transcript message
- Runtime event
- Approval request and decision
- Saved approval rule
- Artifact
- Provider profile and model profile
- Credential reference
- Local memory record
- Extension inventory item
- Skill, MCP server, and plugin references
- Browser state record
- Terminal session record
- Audit log entry

## Workspace And Persistence

Workspace files use a `.c4os` descriptor model. A descriptor can reference folders, display names, workspace settings, enabled extension references, default provider/model preferences, and non-secret IDs. It must not contain raw secrets, full transcripts, artifact archives, or private operational state by default.

Full state migration belongs in explicit future export/import flows.

## Runtime Boundary

OpenCode is the first implementation target behind a thin C4OS-owned runtime adapter. Pi remains a later adapter target. C4OS owns user-facing session identity, workspace persistence, artifact identity, approval policy, provider settings, local memory, runtime lifecycle supervision, runtime recovery, and error reporting.

OpenCode proof findings are preserved in `.agents/references/context/technical-specs/runtime-adapter.md`. Credentialed model-backed prompt execution and live permission-request capture still need validation because the proof avoided provider credentials and token spend.

Runtime-driven tool execution should use a formal C4OS tool gateway instead of
frontend natural-language command parsing. The target flow is: send the prompt
to the runtime, stream runtime events in real time, let the runtime request
tools through a generic tool-call contract, execute approved tools through
C4OS-owned authority boundaries, stream tool output back to the runtime and UI,
then persist the final response and run records. This is MCP-shaped, but C4OS
must remain the host/gateway that owns execution, approval, persistence, and
trusted-root enforcement.

Per-session tool configuration should map tool identities such as
`terminal.run`, `files.read`, `files.write`, `browser.open`, and
`artifact.preview` to enabled state, access level, and approval policy. Runtime
events should use lifecycle names such as `tool_call_requested`,
`tool_call_started`, `tool_output_delta`, `tool_call_completed`, and
`final_response`; event names are not tool identities. Tool implementations may
define default and maximum approval levels, and session config may narrow but
must not silently widen tool authority.

## Security And Trust

- Project-local operations require explicit trusted-root containment.
- High-impact actions require app-owned approval policy before runtime execution.
- High-impact categories include shell commands, file writes/deletes, MCP mutation, credential use, networked tools, worktree mutation, share/export, Browser agent actions, extension runtime impact, and destructive actions.
- Raw API keys must be stored only in OS keychain or an equivalent secure storage abstraction.
- Secrets are secure references, not raw values in workspace files, general app tables, transcripts, or normal app data.
- Generated or untrusted HTML must render without provider credentials, arbitrary workspace file access, shell state, or privileged app APIs.
- Extensions are disabled by default before they affect runtime execution, model context, tools, or app-owned state.
- Skill, plugin, and MCP discovery/loading should read metadata first and defer
  full instruction loading, hook execution, MCP launch, and runtime access until
  explicit enablement through app-owned records.

## Browser Constraints

Browser is a user-owned desktop surface with project-scoped profile. Agent Browser use is request-scoped, and agent file/Browser actions must be recorded.

The implementation must not ignore the failed direct Tauri `WebviewWindow` proof: page content observed `window.__TAURI_INTERNALS__` in that POC. Local-file browsing, logged-in use, privileged bridge exposure, and agent authority require explicit handling before Browser claims are trusted.

## Terminal Constraints

Terminal sessions are backend-owned; renderer code must not spawn arbitrary shells directly. Terminal implementation requires trusted-root cwd validation, deterministic command allowlist, approval policy, sanitized environment, backend-owned lifecycle, bounded renderer event transport, backpressure handling, audit persistence, and cross-platform PTY/ConPTY confirmation.

The TASK-011A/TASK-011B explicit prompt command bridge is transitional polish.
It must not become the long-term command-planning mechanism. Future command
selection should come from runtime tool-call requests and flow through the
C4OS tool gateway.

## Implementation Locations

Production implementation belongs in `backend/`, `frontend/`, and `tests/server/`. Do not create or use `src-tauri/`. POC implementation artifacts belong under `proofs/<proof-name>/`.
