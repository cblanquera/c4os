# MVP Brief

Status: frozen-for-implementation
Updated: 2026-06-21
Sources:
- `.agents/specs/research/research-freeze.md`
- `.agents/specs/research/requirements.md`
- `.agents/specs/research/acceptance.md`
- `.agents/specs/research/decisions.md`
- `.agents/specs/research/viability-gaps.md`
- `wireframes/ui-handoff-spec.md`

## Purpose

This spec defines the distributable C4OS desktop MVP from the frozen research
records. It preserves the full documented/r04 scope. Checkpoints are
implementation progress gates only; they do not reduce MVP scope.

## Target Evaluator

The MVP is for a technical local-project user who needs a desktop workspace to
trust a folder, configure an OpenAI-compatible provider, run persistent coding
agent sessions, inspect and edit trusted project files, use a project-scoped
Browser, run terminal workflows through app-owned boundaries, and enable
extensions deliberately.

## Smallest Coherent Customer Workflow

1. Open the app and select, clone, open, or resume a trusted local project.
2. Configure provider/model settings with raw keys stored only in secure
   storage.
3. Start or resume a session backed by the C4OS-owned runtime adapter.
4. Submit a request with visible attachment, approval, branch, provider, and
   model context.
5. Watch structured user, agent, tool, run, and approval-waiting states.
6. Inspect or edit trusted-root files, preview artifacts, browse local/public
   pages, and use user or agent terminal surfaces under app-owned policy.
7. Install/connect plugins, skills, and MCP servers, then explicitly enable any
   extension before it affects runtime behavior.
8. Quit and restart with session transcript, runtime references, approvals,
   artifacts, and action records available for continuation.

## Implementation Boundary

Production MVP work belongs only in:

- `backend/` for Rust/Tauri backend authority, native commands, state,
  security boundaries, runtime adapter, trusted filesystem access, terminal
  execution, Browser records, extension records, approvals, persistence, and
  restart/resume behavior.
- `frontend/` for the desktop app UI loaded by the product shell.
- `tests/server/` for backend, integration, and MVP acceptance tests.

Do not create or use `src-tauri/`.

## Scope Statement

The MVP includes the full documented/r04 product slice unless a later accepted
MVP-spec decision explicitly moves an item out. Included scope covers:

- folder-first trust and workspace descriptors
- provider/model setup with secure BYOK storage
- OpenCode first behind a thin C4OS-owned runtime adapter
- persistent and resumable sessions
- concurrent sessions/runs across trusted folders, with one main run per chat
  session
- app-owned approvals, artifacts, audit logs, and local state
- Files explorer/editor inside trusted-root controls
- Browser with project-scoped profile, local file browsing, public browsing,
  request-scoped agent browsing, and recorded agent Browser actions
- separate user terminal and backend-owned agent command terminal
- deterministic agent command allowlist with approval fallback
- plugin, skill, and MCP install/connect records, explicit enablement, audit
  records, and `$skill`, `@plugin`, `^mcp` prompt tags
- r04 shell behavior: App Start, New Session, Chat Session, provider/model
  selectors, Browser, Files, Terminal, Settings surfaces, and native OS-level
  File/Edit app menu commands

## Out Of Scope

- Browser downloads.
- Final brand styling or permanent product copy beyond accepted behavioral
  handoff.
- Provider-native API integrations beyond OpenAI-compatible profiles.
- Pi as the first runtime adapter.
- Treating proof code or wireframe simulation as production behavior.
