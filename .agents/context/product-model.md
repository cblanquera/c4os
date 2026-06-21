# Product Model

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Imported from human planning sources and reconciled with promoted runtime adapter findings.

## Summary

C4OS owns the product model even when it delegates execution to OpenCode, Pi, MCP servers, shells, Browser engines, or other runtime components. Workspaces, sessions, approvals, artifacts, providers, memory, extension state, Browser state, Terminal state, and audit logs are C4OS concepts.

## Minimum Entities

- Workspace
- Workspace file
- Project folder
- Trust record
- Session
- Agent run
- Transcript message
- Runtime event
- Approval request
- Approval decision
- Saved approval rule
- Artifact
- Provider profile
- Model profile
- Credential reference
- Local memory record
- Extension inventory item
- Skill reference
- MCP server reference
- Plugin reference
- Browser state record
- Terminal session record
- Audit log entry

## Workspace And Project Registry

The app needs a local registry for:

- project display name
- local path
- trust state
- recent sessions
- runtime configuration references
- provider/model preferences
- optional Git metadata
- workspace membership

Users can add, inspect, remove, and switch between multiple local projects. Multiple trusted folders should be manageable without moving files or forcing a repository structure.

## Workspace Descriptor

- Workspace files use a `.c4os` descriptor model.
- A descriptor can reference folders, display names, workspace settings, enabled extension references, default provider/model preferences, and non-secret IDs.
- A descriptor must not contain raw secrets, full transcripts, artifact archives, or private operational state by default.
- Full state migration belongs in explicit future export/import flows.

## Sessions And Runs

Each project supports multiple sessions. A session should include:

- transcript and message index
- run status
- selected agent
- selected model/provider
- approval state and audit references
- artifact references
- local memory references
- resume metadata
- runtime references
- branch/worktree state where applicable

Concurrent agent runs must not mix approvals, tool output, working directories, artifacts, runtime events, or cancellation state. Users need active status, cancel, error recovery, and resume behavior.

## Runtime Ownership Boundary

C4OS owns:

- user-facing session identity
- workspace persistence
- artifact identity
- approval policy
- provider settings
- local memory
- runtime lifecycle supervision
- runtime recovery and error reporting

The runtime adapter hides OpenCode/Pi-specific concepts unless advanced settings or compatibility inspection require them.

## Secret Handling

Secrets are secure references, not raw values in workspace files, general app tables, transcripts, or normal app data. Raw provider keys belong only in OS keychain or the selected secure storage abstraction.

## Related Context

- `.agents/context/runtime-adapter.md`
- `.agents/context/constraints.md`
- `.agents/context/terms.md`
