# Product Context

Status: active
Created: 2026-06-18
Updated: 2026-06-21
Source Note: Imported from human planning sources.

## Summary

C4OS is a local-first desktop command center for AI-assisted work. It lets users open trusted project folders, run persistent agent sessions, manage providers and approvals, inspect artifacts, edit files, browse with project-scoped Browser profiles, and use separate user and agent-owned terminal surfaces from an app-owned workspace.

The product combines the useful parts of a desktop IDE, local-first agent runtime, and Codex-like session interface. It should feel like a workspace for real work, not a prompt box or runtime wrapper.

## Product Thesis

AI agents are useful across coding, writing, research, analysis, operations, and documentation, but current products tend to split into two weak models:

- cloud-first chat tools that do not safely own local project context, files, approvals, credentials, or workspace state
- developer-first CLIs and runtimes that expose too much runtime detail, require terminal fluency, and fragment persistence, safety, artifact handling, and multi-project workflows

C4OS should bridge those models by owning the local workspace, safety model, session state, artifact model, provider configuration, and extension boundaries while keeping runtime backends behind adapters.

## Target Users

- Technical power users who need stronger local project context than cloud chat and safer workflows than unconstrained scripts.
- Developers and small technical teams that use repositories, instruction files, skills, MCP, scripts, branches, and review artifacts.
- Knowledge workers with folder-based local projects who need persistent sessions, artifacts, provider control, and local memory.
- Future secondary users include team leads, administrators, plugin authors, MCP authors, and skill authors.

### Primary User Needs

- Technical users need a stronger interface than chat and a safer interface than unconstrained local agent scripts.
- Developers and small teams need concurrent sessions, visible approvals, file inspection, Git awareness, and reproducible local state that respects repository conventions.
- Folder-based knowledge workers need persistent sessions, artifacts, provider control, and local memory even when they do not heavily use Git or shell.
- Team leads and administrators need future policy, auditability, provider restriction, credential, and extension trust support without forcing a cloud admin console into the MVP.
- Plugin, MCP, and skill authors need provenance, permissions, enablement, and scoping to be visible before runtime use.

## Product Problems

- Local context is hard to trust: the app must make trusted folders explicit and contain local operations to those roots.
- Agent work is hard to resume: transcript, runtime events, artifacts, approvals, selected models, and local memory need durable session identity.
- Approvals are too runtime-specific: C4OS must own the approval gateway rather than delegating safety only to OpenCode, Pi, MCP, shell, or another runtime.
- Model and provider setup is fragmented: users need OpenAI-compatible provider profiles, secure BYOK storage, model discovery or manual entry, and per-session model selection.
- Generated outputs need a workspace: code, Markdown, diffs, images, PDFs, HTML previews, command logs, and research notes need artifact identity and safe viewers.
- Power features need safety boundaries: browser, terminal, file editing, extensions, skills, MCP, and plugins must be designed around trust, containment, provenance, and auditability.

## Product Goals

- Provide a folder-first desktop AI workspace for local projects.
- Keep project files, session state, approvals, artifacts, memory, and provider settings under app-owned local control.
- Support multiple trusted project folders and multiple resumable sessions per project.
- Allow agent runs to operate concurrently without mixing approvals, outputs, working directories, or artifacts.
- Route sensitive actions through app-owned approval and audit boundaries.
- Support OpenAI-compatible provider profiles with bring-your-own-key storage.
- Preserve compatibility with useful ecosystem conventions such as `AGENTS.md`, `SKILL.md`, MCP, OpenCode configuration, and OpenCode plugins where practical.
- Provide safe artifact, browser, terminal, and file-explorer surfaces without exposing privileged app APIs to untrusted content.

## Success Measures

- A new user can open and trust a project folder without external docs.
- A user can start a session, choose a model, submit a prompt, approve an action, and inspect the result in one flow.
- A user can leave and return to a session with transcript, artifacts, selected model, and approval history intact.
- Sensitive file, shell, MCP, credential, and destructive actions are blocked or approval-gated before execution.
- Raw provider keys never appear in workspace files or normal app data.
- Untrusted generated HTML cannot access privileged app APIs.
- File and terminal actions cannot escape trusted roots through traversal.
- Runtime lifecycle can start, stop, recover, and report errors reliably.
- Provider profiles can connect to at least one real OpenAI-compatible provider.
- Artifact storage and viewers work across frozen MVP artifact types.
- Sessions can run concurrently without cross-run approval or output leakage.
- Users can complete meaningful local project work without leaving the app for basic file inspection, approvals, artifacts, provider setup, and transcript continuity.

## Core Workflow

1. Open, create, clone, or select a local project folder.
2. Trust the folder before local file, Git, shell, instruction, skill, or approval-scoped workflows are available.
3. Start or resume a session from the project workspace.
4. Select provider and model settings.
5. Submit prompts through the composer.
6. Watch agent status, tool activity, approvals, artifacts, and runtime events in context.
7. Approve or deny sensitive actions.
8. Inspect files, artifacts, browser previews, and terminal output inside product safety boundaries.
9. Resume later with workspace, session, artifact, approval, provider, and memory state intact.

## Product Principles

- Local-first by default: store project metadata, session metadata, approvals, artifacts, audit logs, local memory, and workspace state locally by default.
- Project-first and folder-trusted: local file, Git, shell, instruction, skill, and approval-scoped workflows require a trusted project folder.
- App-owned product model: users interact with C4OS workspaces, sessions, approvals, artifacts, providers, files, Browser, Terminal, and settings rather than raw runtime concepts.
- Explicit trust before execution: shell commands, file writes/deletes, MCP mutation, credential use, networked tools, worktree mutation, share/export, and destructive operations pass through app-owned policy.
- Standards-compatible, not proprietary-first: respect `AGENTS.md`, `AGENTS.override.md`, `SKILL.md`, MCP, OpenCode/Pi config, and OpenCode/Pi-compatible plugins where practical.
- Safe surfaces over raw power: browser, terminal, artifact preview, and file editing are useful only when privileged app APIs and arbitrary local authority are not exposed to untrusted content.
- Durable work with lightweight files: `.c4os` workspace files are portable descriptors; transcripts, artifacts, audit logs, secrets, and operational state stay in app-owned local storage unless explicitly exported.

## Product Implications

- The product should feel like a workspace, not a prompt box.
- Underlying runtimes should remain implementation details behind an app-owned adapter.
- Trust, approvals, provider settings, artifacts, and persistence are product features, not runtime leftovers.
- Future specs should start from the product context, experience, feature surface, interface, and model records before reading detailed research records.
