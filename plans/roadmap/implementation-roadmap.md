# Implementation Roadmap

This roadmap is intentionally specification-level. It does not start implementation.

## Phase 0: MVP Validation Prototypes

 - Verify OpenCode Runtime can be driven headlessly from a Tauri backend.
 - Verify session event streaming, tool-call interception, and approval hooks.
 - Prove MVP file writes, shell commands, and Git state changes cannot execute before the app-owned Approval Gateway allows or denies them.
 - Verify OpenRouter provider configuration through OpenCode.
 - Verify project-root file access can be enforced for runtime-driven file operations.
 - Verify shell approval, output capture, and process termination from the backend.
 - Verify SQLite session persistence, crash recovery, and basic artifact storage.
 - Verify the MVP onboarding and approval UX does not overload technical users.

Exit criteria: no architectural blocker to the coding-first MVP runtime, approval, provider, storage, and UX loop. If reliable pre-execution interception cannot be proven, stop before Phase 1 and choose a wrapper, proxy, fork, runtime replacement, or MVP scope reduction.

## Phase 1: Desktop Shell And Persistence

 - Create Tauri app skeleton.
 - Add SQLite metadata store.
 - Add registration for local Git projects and a minimal selector/list for one selected active project.
 - Add session list and transcript shell.
 - Add settings storage and OS keychain integration.

Exit criteria: users can register projects, select one active project, and create a persistent empty session for that project.

## Phase 2: Runtime Integration

 - Add OpenCode Runtime adapter.
 - Stream messages and tool events into the UI.
 - Persist messages and tool calls.
 - Add model/provider configuration through OpenRouter.

Exit criteria: users can run an agent session and resume it after restart.

## Phase 3: Tools, Files, Shell, Git, And Approvals

 - Add MVP approval gateway for file writes, shell commands, and runtime-proposed Git state changes.
 - Add approval modal.
 - Add file browser.
 - Add shell output viewer.
 - Add Git status, changed-file list, and diff viewer.
 - Enforce project-root file boundaries with no external-directory grants.
 - Add root `AGENTS.md` discovery and display.

Exit criteria: file writes, shell commands, and runtime-proposed Git state changes are project-root scoped, auditable, approval-aware, and reviewable.

## Phase 4: Tool Ledger, Artifacts, And Alpha Hardening

 - Persist the basic tool ledger.
 - Add tool activity timeline.
 - Capture text, logs, diffs, and generated-file artifacts.
 - Add runtime, provider, and tool failure states.
 - Add stop-session behavior and process cleanup.
 - Add policy, storage migration, crash recovery, and process cleanup tests.
 - Run a security review focused on MVP file, shell, Git, secret, and approval surfaces.

Exit criteria: MVP is ready for private alpha.

## Post-MVP Roadmap

These systems remain important after MVP validation, but are not on the private-alpha critical path.

### V1

 - Multiple sessions per project.
 - Polished multi-project workflows.
 - Session rename, archive, pin, and delete.
 - Worktree creation and cleanup.
 - Nested `AGENTS.md` resolution.
 - Explicit skill discovery and invocation.
 - Local stdio MCP server support.
 - Richer artifact previews.
 - Export/import for sessions and artifacts.

### V2 And Later

 - Multiple concurrent agents.
 - Runtime abstraction across multiple agent runtimes.
 - Plugin installation and permission review.
 - Marketplace support.
 - Remote MCP.
 - Browser panel, screenshots, DOM extraction, and web content ingestion.
 - Document, spreadsheet, and slide workflows.
 - Long-term memory beyond session persistence.
 - Tamper-evident audit log.
 - Data-flow-aware policy engine.

## Sequencing Recommendation

Validate OpenCode integration before committing to detailed UI buildout.

Alternatives considered:
 - Build UI first: attractive for demos, but can hide runtime blockers.
 - Build custom runtime first: too expensive and contrary to MVP constraints.

Why selected: runtime feasibility is the highest architectural risk.
