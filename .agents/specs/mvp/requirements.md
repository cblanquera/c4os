# MVP Requirements

Status: ready-for-review
Updated: 2026-06-21

## Requirements

### REQ-001: Trusted Folder Gate

Source: `.agents/specs/research/requirements.md`, `wireframes/ui-handoff-spec.md`

Local file, Git, shell, instruction, skill, extension, and approval-scoped
workflows remain unavailable until a local project folder is selected and
trusted.

### REQ-002: Workspace Descriptor Safety

Source: `.agents/specs/research/requirements.md`

Workspace descriptors may store non-secret project references and preferences,
but not raw secrets, full transcripts, artifact archives, or private operational
state by default.

### REQ-003: Provider And Model Setup

Source: `.agents/specs/research/requirements.md`, `.agents/context/technical-specs.md`

Users can configure OpenAI-compatible providers, store raw keys only in secure
storage, manage models, and select provider/model context per session.

### REQ-004: C4OS-Owned Runtime Adapter

Source: `.agents/specs/research/decisions.md`, `.agents/specs/research/evidence.md`

OpenCode is the first runtime backend behind a thin C4OS-owned adapter.
Workspace, session, run, approval, artifact, provider, extension, and UI records
remain product-owned.

### REQ-005: Persistent Resumable Sessions

Source: `.agents/specs/research/requirements.md`, `.agents/context/product-brief.md`

Sessions preserve transcript, message index, run status, selected agent,
provider/model, approval, artifact, memory, runtime, branch/worktree, Browser,
terminal, extension, and action-record references needed for restart/resume.

### REQ-006: Concurrent Session Isolation

Source: `.agents/specs/research/research-freeze.md`, `.agents/specs/research/viability-gaps.md`

The MVP supports concurrent sessions/runs across trusted project folders, with
one main active run per chat session and isolated approvals, outputs, working
directories, artifacts, runtime events, and cancellation state.

### REQ-007: App-Owned Approval Policy

Source: `.agents/specs/research/requirements.md`

Shell commands, file writes/deletes, MCP mutation, credential use, networked
tools, worktree mutation, share/export, Browser agent actions, extension
runtime impact, and destructive actions are evaluated by app-owned policy before
execution.

### REQ-008: Trusted-Root Files

Source: `.agents/specs/research/requirements.md`, `.agents/context/product-brief.md`

File browsing, loading, editing, saving, reverting, creating, renaming, and
deleting stay inside trusted roots and reject traversal or casual `.git`
mutation.

### REQ-009: Artifact And HTML Preview Safety

Source: `.agents/specs/research/requirements.md`, `wireframes/ui-handoff-spec.md`

Artifacts are product records with safe viewers. Generated or untrusted HTML
renders without provider credentials, arbitrary workspace file access, shell
state, or privileged app APIs.

### REQ-010: Browser Surface

Source: `docs/adr/0001-browser-and-agent-authority.md`, `.agents/specs/research/decisions.md`

Browser is a user-owned desktop surface with project-scoped profile, local file
browsing, public web browsing, request-scoped agent browsing, logged-in use when
clearly requested, and audit records for agent Browser actions. Downloads are
excluded.

### REQ-011: Backend-Owned Terminal Surfaces

Source: `.agents/specs/research/requirements.md`, `.agents/specs/research/decisions.md`

Terminal includes a user terminal and an agent-owned command terminal. Backend
state owns process lifecycle, cwd validation, sanitized environment, transport,
output streaming, cancellation, failure reporting, cleanup, and audit records.

### REQ-012: Deterministic Command Allowlist

Source: `.agents/specs/research/requirements.md`, `.agents/specs/research/decisions.md`

Agent-owned read-only command execution uses exact pattern matching against an
app-bundled default allowlist, overrideable by workspace config. Non-allowlisted
or outside-project commands require approval.

### REQ-013: r04 Desktop Shell

Source: `wireframes/ui-handoff-spec.md`, `wireframes/screens.md`

The app implements the accepted r04 desktop shell: App Start, New Session, Chat
Session, provider/model selectors, three-panel layout, right Browser/Files/
Terminal tabs, and Settings surfaces.

### REQ-014: Structured Prompt And Thread States

Source: `.agents/specs/research/requirements.md`, `wireframes/ui-handoff-spec.md`

The composer exposes attachment, approval policy, branch, provider, and model
context before submission. The thread distinguishes user messages, agent
messages, tool activity, run activity, approval waits, and follow-up composer
state.

### REQ-015: Extension Install, Enablement, And Invocation

Source: `docs/adr/0002-extension-enablements-and-prompt-tags.md`, `.agents/specs/research/requirements.md`

The MVP supports plugin, skill, and MCP install/connect records with provenance,
scopes, workspace/project scope, shared data, runtime/tool access, enabled
state, and audit log. Runtime impact requires explicit per-extension
enablement. Invocation tags are `$skill`, `@plugin`, and `^mcp`; MCP invocation
is explicit only.

### REQ-016: Implementation Paths

Source: `.agents/AGENTS.md`, `.agents/specs/research/research-freeze.md`

Production implementation belongs in `backend/`, `frontend/`, and
`tests/server/`. `src-tauri/` is not used.

