# Implementation Roadmap

This roadmap is intentionally specification-level. It does not start implementation.

## Phase 0: Validation Prototypes

 - Verify OpenCode Runtime can be driven headlessly from a Tauri backend.
 - Verify session event streaming, tool-call interception, and approval hooks.
 - Verify OpenRouter provider configuration through OpenCode.
 - Verify MCP server lifecycle management.
 - Verify Git worktree creation and cleanup from Rust.

Exit criteria: no architectural blocker to the preferred runtime stack.

## Phase 1: Desktop Shell And Persistence

 - Create Tauri app skeleton.
 - Add SQLite metadata store.
 - Add project registration.
 - Add session list and transcript shell.
 - Add settings storage and OS keychain integration.

Exit criteria: users can register projects and create persistent empty sessions.

## Phase 2: Runtime Integration

 - Add OpenCode Runtime adapter.
 - Stream messages and tool events into the UI.
 - Persist messages and tool calls.
 - Add model/provider configuration through OpenRouter.

Exit criteria: users can run an agent session and resume it after restart.

## Phase 3: Tools, Files, Shell, And Approvals

 - Add policy engine.
 - Add approval modal.
 - Add file browser.
 - Add shell output viewer.
 - Enforce project-root and external-directory policies.

Exit criteria: read/write/shell tool calls are auditable and approval-aware.

## Phase 4: Git And Worktrees

 - Add Git status and diff viewer.
 - Add worktree creation per session.
 - Add worktree cleanup policy.
 - Add commit/branch handoff helpers.

Exit criteria: concurrent sessions can safely isolate file edits.

## Phase 5: MCP, Skills, And Instructions

 - Add MCP server registry.
 - Add tool/resource/prompt display.
 - Add `AGENTS.md` discovery.
 - Add skill indexing and explicit invocation.

Exit criteria: projects can use standard instructions, skills, and MCP tools.

## Phase 6: Plugins And Marketplace

 - Add plugin validation.
 - Add local plugin install.
 - Add marketplace manifest support.
 - Add plugin enablement and permission review.

Exit criteria: Codex-compatible plugins can be installed locally and enabled per project.

## Phase 7: Artifacts And Browser

 - Add artifact store.
 - Add previews for core artifact types.
 - Add browser/web panel.
 - Link artifacts to tool calls and sessions.

Exit criteria: generated outputs are durable, previewable, and traceable.

## Phase 8: Hardening

 - Add policy tests.
 - Add migration tests.
 - Add plugin validation tests.
 - Add security review.
 - Add crash recovery and process cleanup.

Exit criteria: MVP is ready for private alpha.

## Sequencing Recommendation

Validate OpenCode integration before committing to detailed UI buildout.

Alternatives considered:
 - Build UI first: attractive for demos, but can hide runtime blockers.
 - Build custom runtime first: too expensive and contrary to MVP constraints.

Why selected: runtime feasibility is the highest architectural risk.
