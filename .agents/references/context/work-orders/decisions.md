# Decisions

Status: active
Created: 2026-06-18
Updated: 2026-06-21
Source Note: Imported from human planning sources and reconciled with accepted research decisions and POC results.

## Accepted Restart Decisions

- C4OS is a greenfield restart that uses prior planning records only as product intent and evidence.
- The app is folder-first: prompt entry is disabled until a trusted project folder exists.
- The app owns product identity for workspaces, projects, sessions, agent runs, approvals, artifacts, providers, files, browser surfaces, terminal surfaces, and settings.
- Runtime-specific OpenCode or Pi concepts should stay behind an adapter unless exposed for advanced compatibility inspection.
- OpenCode is the first runtime backend behind the C4OS-owned adapter; Pi remains a later adapter target.
- Provider support starts with OpenAI-compatible profiles and BYOK storage.
- Built-in provider profile types include OpenAI, OpenRouter, Hugging Face router, LiteLLM proxy, and custom OpenAI-compatible endpoint.
- The interface uses a three-panel desktop shell with left navigation, central session content, and right-side Browser, Files, and Terminal tabs.
- The current interface contract is promoted into `.agents/context/creative-specs.md`.
- The first usable screen after trusting a folder is the active workspace, not a marketing dashboard.
- The MVP interface should remain neutral and utilitarian, use lucide icons, and avoid dark theme or brand styling unless a later design phase approves it.
- All documented/r04 features are MVP scope unless explicitly moved out later; checkpoint phases are implementation milestones, not MVP scope boundaries.
- Browser is MVP as a user-owned desktop Browser with project-scoped profile, local file browsing, public web browsing, and request-scoped agent browsing.
- Terminal is MVP as separate user terminal and agent-owned command terminal surfaces.
- Plugin, skill, and MCP install/connect flows are MVP; runtime impact requires explicit per-extension enablement.
- Concurrent sessions/runs across trusted project folders are MVP, with one main active run per chat session.
- The active project folder is the default agent work boundary. Inside the active project, implied file reads and writes are allowed; outside-project file access requires explicit permission.
- Public web browsing and logged-in Browser use are allowed when clearly within the user's request, and C4OS must record agent file and Browser actions.
- Browser downloads are excluded from MVP.
- Chat prompt extension tags are `$skill`, `@plugin`, and `^mcp`; MCP invocation is explicit only.
- Settings navigation from the r04 interface handoff is Providers, Models, Runtimes, Configuration, Plugins, Skills, MCP Servers.
