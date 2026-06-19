# Product Goals

Status: active
Created: 2026-06-18
Updated: 2026-06-18
Source Note: Imported from `plans/product-brief.md`.

## Goals

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
