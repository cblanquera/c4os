# Decisions

Status: active
Created: 2026-06-18
Updated: 2026-06-18
Source Note: Imported from `plans/product-brief.md` and `plans/product-interface.md`.

## Accepted Restart Decisions

- C4OS is a greenfield restart that uses prior planning records only as product intent and evidence.
- The app is folder-first: prompt entry is disabled until a trusted project folder exists.
- The app owns product identity for workspaces, projects, sessions, agent runs, approvals, artifacts, providers, files, browser surfaces, terminal surfaces, and settings.
- Runtime-specific OpenCode or Pi concepts should stay behind an adapter unless exposed for advanced compatibility inspection.
- Provider support starts with OpenAI-compatible profiles and BYOK storage.
- The interface uses a three-panel desktop shell with left navigation, central session content, and right-side Browser, Files, and Terminal tabs.
- The first usable screen after trusting a folder is the active workspace, not a marketing dashboard.
- The MVP interface should remain neutral and utilitarian, use lucide icons, and avoid dark theme or brand styling unless a later design phase approves it.
