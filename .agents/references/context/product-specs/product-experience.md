# Product Experience

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Imported from human planning sources.

## Summary

The C4OS experience is folder-first, session-centered, and Codex-like. A user should be able to enter a trusted project workspace, start or resume work, choose provider/model/approval settings, watch agent prework and tool activity, approve sensitive actions, inspect outputs, and return later with state intact.

## First Run

- The first launch shows a folder-first empty state.
- Primary actions are open existing folder, create a new local folder-backed workspace, clone a repository, and open a recent trusted workspace or folder.
- The prompt composer is disabled until a project folder exists and is trusted.
- Trust should be explained through UI state and permissions rather than long instructional text.

## Main Workspace

- The first usable screen after trusting a folder is the active workspace, not a dashboard or marketing page.
- The shell uses a Codex-like desktop layout with project/session navigation on the left, session transcript and composer in the center, and contextual tool surfaces on the right.
- Status areas should keep active runs, pending approvals, model/provider state, trusted-root state, and runtime health visible.
- Right or secondary surfaces can include files, artifacts, approvals, provider settings, runtime state, Browser, Terminal, and inspectors.

## Session Flow

1. User selects a trusted project and starts or resumes a chat/session.
2. User chooses provider/model, branch or worktree preference where applicable, approval mode, attachments, and tags.
3. User submits a prompt.
4. Session shows agent prework, thinking/status, tool activity, and pending approvals before final response.
5. Sensitive actions route to approval controls.
6. Generated artifacts, file references, terminal output, Browser results, and runtime events appear in context.
7. User can continue, inspect, edit, export, or resume later.

## Prework And Response Rendering

- The transcript should show when the agent is thinking and what prework it is doing before final response.
- Prework should be grouped and collapsed after final response.
- Progress should be visible without turning the transcript into noisy logs.
- Response rendering should feel polished and Codex-like, including typing or graceful text reveal where appropriate.

## Prompt Composer

The composer should support:

- disabled state until a trusted folder is available
- text prompt entry
- file attachments
- branch selection or new branch creation
- approval mode selection
- speech-to-text
- inline tagging for skills, plugins, agents, models, MCP servers, and related scope-aware entities
- clear visibility into the project, model, branch, approval profile, and scopes that apply to the run

## Related Context

- `.agents/context/product-brief.md`
- `.agents/context/technical-specs.md`
- `.agents/context/product-specs.md`
- `.agents/context/creative-specs.md`
