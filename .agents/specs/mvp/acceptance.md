# MVP Acceptance

Status: frozen-for-implementation
Updated: 2026-06-21

## Acceptance Criteria

### AC-001: Trust Gate

Given no trusted folder exists, when the app opens, then prompt submission and
local agent actions are unavailable until the user selects and trusts a folder.

### AC-002: Workspace Descriptor Safety

Given a workspace descriptor is written, then it contains only non-secret
references and excludes raw keys, full transcripts, artifact archives, and
private operational state by default.

### AC-003: Provider Key Storage

Given a provider key is saved, then raw key material is stored only in secure
storage and never appears in workspace descriptors or normal app data.

### AC-004: Runtime Adapter Boundary

Given a session starts, then OpenCode is reached through the C4OS adapter and
the user-facing records remain C4OS workspace/session/run records.

### AC-005: Session Resume

Given the user restarts the app, then transcript, runtime references, provider
and model context, approvals, artifacts, Browser/file/terminal/extension action
records, and run state needed for continuation are available.

### AC-006: Concurrent Runs

Given multiple trusted project folders or sessions exist, when runs are active,
then each run has isolated cwd, approvals, outputs, artifacts, runtime events,
and cancellation state, and each chat has at most one main active run.

### AC-007: Sensitive Action Approval

Given a sensitive action is requested, then app-owned policy allows, asks, or
denies before execution and records the decision.

### AC-008: Trusted-Root File Access

Given a file operation is requested, then traversal outside trusted roots and
casual `.git` mutation are rejected before read or write behavior occurs.

### AC-009: Safe Artifact Preview

Given generated or untrusted HTML is previewed, then it cannot access provider
credentials, arbitrary workspace files, shell state, or privileged app APIs.

### AC-010: Browser Scope

Given the Browser tab is used, then it supports project-scoped profile state,
local file opening, public browsing, request-scoped agent browsing, logged-in
use when clearly requested, and agent Browser action records, while downloads
remain unavailable.

### AC-011: Terminal Boundary

Given a terminal session starts, then backend process state owns lifecycle,
transport is bounded, output is auditable, and user terminal and agent command
terminal surfaces remain distinct.

### AC-012: Command Allowlist

Given the agent requests a read-only allowlisted command, then it may run
without prompting. Given a non-allowlisted or outside-project command is
requested, then explicit approval is required before execution.

### AC-013: Three-Panel Shell

Given the main app is open, then the layout has left project/session
navigation, central session content, and right tool tabs in Browser, Files,
Terminal order.

### AC-014: App Start Entry Points

Given no trusted project is active, then App Start presents folder, clone,
workspace-file, and recent-workspace entry points and does not expose prompt
submission.

### AC-015: Composer Context

Given the user can submit a prompt, then attachment, approval policy, branch,
provider, and model context are visible before submission and cannot silently
change in active follow-up context.

### AC-016: Structured Thread

Given a session has messages, tool calls, run events, or approval waits, then
the thread renders them as distinct structured surfaces with clear ownership.

### AC-017: Files Surface

Given the Files tab is active, then trusted project browsing and editing states
are available with dirty, save, revert, and guarded mutation behavior.

### AC-018: Settings IA

Given Settings is open, then navigation exposes Providers, Models, Runtimes,
Configuration, Plugins, Skills, and MCP Servers in order, with return to the
workspace shell.

### AC-019: Extension Enablement

Given a plugin, skill, or MCP server is installed or connected, then its record
shows provenance, scopes, workspace/project scope, shared data, runtime/tool
access, enabled state, and audit records, and it has no runtime effect until
explicitly enabled.

### AC-020: Extension Prompt Tags

Given enabled extensions exist, then `$skill` invokes skills, `@plugin` invokes
plugins, and `^mcp` invokes MCP servers; MCP servers are not invoked implicitly.

### AC-021: Wireframe Placeholder Guard

Given implementation uses r04 wireframes, then sample names, data, statuses,
URLs, output, copy, and grayscale styling are treated as illustrative unless
separately promoted.
