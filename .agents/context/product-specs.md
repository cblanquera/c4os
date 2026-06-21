# Product Specs

Status: active
Created: 2026-06-21
Updated: 2026-06-21
Source Note: Normalized from previous feature, experience, and MVP context documents. Detailed inventories are preserved under `.agents/references/context/product-specs/`.

## Purpose

Use this as the product-behavior gate. It summarizes what the product must do, which user workflows and surfaces exist, and which MVP exclusions matter before deeper product references or specs are loaded.

## Load When

- You need customer workflow, feature surfaces, app behavior, or MVP user-facing scope.
- You are drafting or checking requirements, acceptance criteria, feature scope, or product behavior.
- You need to decide whether to open the detailed MVP feature inventory.

## Skip When

- You only need product identity, users, goals, or terms; load `product-brief.md`.
- You only need runtime, persistence, security, Browser isolation, or Terminal ownership; load `technical-specs.md`.
- You only need UI layout, interaction, visual, or accessibility guidance; load `creative-specs.md`.
- You only need sequencing, validation work, accepted decisions, or deferred work; load `work-orders.md`.

## Owns

- Product behavior, user workflow, feature surfaces, MVP user-facing inclusions and exclusions, and product-experience routing.

## Does Not Own

- Product thesis, technical implementation authority, visual layout contract, active progress, or full traceable acceptance records.

## Reference Routing

| Need | Load | Why |
| --- | --- | --- |
| First-run, main workspace, session flow, prework, composer behavior | `.agents/references/context/product-specs/product-experience.md` | Use for experiential behavior before UI detail. |
| Workspace, approvals, providers, files, artifacts, Browser, Terminal, settings, extensions, memory | `.agents/references/context/product-specs/feature-surfaces.md` | Use for surface-level product capabilities. |
| Complete MVP feature inventory, exclusions, guardrails | `.agents/references/context/product-specs/mvp-feature-list.md` | Use before MVP implementation or detailed acceptance work. |
| Source and artifact provenance | `.agents/references/context/source-provenance.md` | Use only when tracing where product facts came from. |

## Summary

The C4OS product surface is the local-project workspace: app start, workspace trust, project/session navigation, composer, transcript, provider/model setup, approvals, artifacts, Browser, Files, Terminal, settings, extensions, runtime state, and local memory. The app owns these surfaces even when execution delegates to OpenCode, Pi, MCP servers, shells, Browser engines, or other components.

## MVP Workflow

The MVP must let a technical local-project user:

1. Open the app and select, clone, create, open, or resume a local project.
2. Trust a project folder before local file, Git, shell, instruction, skill, extension, or approval-scoped workflows become available.
3. Configure an OpenAI-compatible provider and model with raw keys stored only in secure storage.
4. Start or resume a persistent session backed by the C4OS-owned runtime adapter.
5. Submit a request with visible attachment, approval policy, branch, provider, and model context.
6. Watch structured user messages, agent messages, tool activity, run activity, approval waits, and final responses.
7. Inspect and edit trusted-root files, preview artifacts, browse local/public pages, and use user or agent terminal surfaces under app-owned policy.
8. Install or connect plugins, skills, and MCP servers, then explicitly enable any extension before it affects runtime behavior.
9. Quit and restart with transcript, runtime references, approvals, artifacts, Browser/file/terminal/extension action records, and run state available for continuation.

## Feature Surfaces

- Workspace and project management: open, create, clone, trust, inspect, remove, and switch local project folders.
- Sessions and runs: persistent transcripts, selected provider/model, approval state, artifact references, runtime references, branch/worktree context, Browser state, terminal state, extension state, action records, restart, resume, cancel, and error recovery.
- Composer and transcript: visible attachment, approval policy, branch, provider, model context, structured run activity, approval waits, and final responses.
- Providers and models: OpenAI-compatible BYOK provider profiles, built-in profile types, secure key status, connection testing, model discovery, manual model entry, model enablement, search, and per-session selection.
- Approvals and audit: allow, ask, deny, pending queue, clear action descriptions, affected resources, remembered scoped rules, and audit records.
- Files: trusted-root explorer, open file/code view, editing, saving, reverting, creating, renaming, guarded delete, external change handling, and active-file search.
- Artifacts: first-class records for Markdown, text, code, diffs, generated HTML, images, PDFs, downloadable files, command logs, research, and analysis outputs.
- Browser: user-owned desktop surface for local preview and web URLs with request-scoped agent browsing and action records.
- Terminal: user terminal plus backend-owned agent command terminal under trusted-root validation and approval policy.
- Settings and extensions: Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers, provenance, scopes, enabled state, and audit log.
- Local memory: app-owned scoped memory separate from raw provider state and runtime-local session storage.

## Product Experience

The first launch is folder-first. Prompt submission is disabled until a project folder exists and is trusted. The first usable screen after trusting a folder is the active workspace, not a dashboard. The shell uses a Codex-like desktop layout with left project/session navigation, center session content, and right workspace tools.

Agent prework should be visible without turning the transcript into noisy logs. Progress can be grouped and collapsed after final response. Response rendering should feel polished and suitable for repeated technical work.

## MVP Exclusions

- Browser downloads.
- Final brand styling or permanent product copy beyond accepted behavioral handoff.
- Provider-native integrations beyond OpenAI-compatible provider profiles.
- Pi as the first runtime adapter.
- Treating proof code, mock server behavior, or wireframe simulation as production-complete behavior.
- Browser tab strip, secondary right-panel tabs, gear tabs, or panel-management strips inside the Browser/Files/Terminal tab bar.
- Arbitrary renderer shell spawn, remote shells, SSH, containers, terminal multiplexing, and agent auto-run without deliberate future scope.
