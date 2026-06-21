# Technical Specs

Status: active
Created: 2026-06-21
Updated: 2026-06-21
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

## Security And Trust

- Project-local operations require explicit trusted-root containment.
- High-impact actions require app-owned approval policy before runtime execution.
- High-impact categories include shell commands, file writes/deletes, MCP mutation, credential use, networked tools, worktree mutation, share/export, Browser agent actions, extension runtime impact, and destructive actions.
- Raw API keys must be stored only in OS keychain or an equivalent secure storage abstraction.
- Secrets are secure references, not raw values in workspace files, general app tables, transcripts, or normal app data.
- Generated or untrusted HTML must render without provider credentials, arbitrary workspace file access, shell state, or privileged app APIs.
- Extensions are disabled by default before they affect runtime execution, model context, tools, or app-owned state.

## Browser Constraints

Browser is a user-owned desktop surface with project-scoped profile. Agent Browser use is request-scoped, and agent file/Browser actions must be recorded.

The implementation must not ignore the failed direct Tauri `WebviewWindow` proof: page content observed `window.__TAURI_INTERNALS__` in that POC. Local-file browsing, logged-in use, privileged bridge exposure, and agent authority require explicit handling before Browser claims are trusted.

## Terminal Constraints

Terminal sessions are backend-owned; renderer code must not spawn arbitrary shells directly. Terminal implementation requires trusted-root cwd validation, deterministic command allowlist, approval policy, sanitized environment, backend-owned lifecycle, bounded renderer event transport, backpressure handling, audit persistence, and cross-platform PTY/ConPTY confirmation.

## Implementation Locations

Production implementation belongs in `backend/`, `frontend/`, and `tests/server/`. Do not create or use `src-tauri/`. POC implementation artifacts belong under `proofs/<proof-name>/`.
