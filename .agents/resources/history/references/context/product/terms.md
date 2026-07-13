# Terms

Status: active
Created: 2026-06-18
Updated: 2026-06-18
Source Note: Imported from human planning sources.

## Terms

- Workspace: An app-owned grouping that can reference project folders, settings, provider preferences, and extension references.
- Workspace file: A portable `.c4os` descriptor that references non-secret workspace metadata.
- Project folder: A local folder opened, created, cloned, or selected by the user.
- Trust record: The user's explicit decision to allow project-scoped local operations.
- Project browser profile: Browser session state, including login/cookie state, remembered per trusted project and shared by sessions in that project.
- Agent authority: The operations an agent may perform on behalf of the user, scoped by active project, current request, and any explicit permissions granted in chat. Inside the active project, file reads and writes are allowed when implied by the request. Outside-project file access requires explicit permission.
- Agent action record: An audit entry for agent file reads, file writes, outside-project permission grants, public websites visited, and logged-in sites used.
- Agent-owned command terminal: Backend-owned terminal execution channel used by agents, separate from the user's terminal. Read-only allowlisted commands may run without prompting; mutating, non-allowlisted, or outside-project commands require approval.
- Command allowlist: App-bundled default config, overrideable by workspace config, containing exact command patterns that the agent-owned command terminal may run without prompting.
- Session: A persistent conversation/workstream associated with a project.
- Agent run: A runtime execution inside a session.
- Approval request: A sensitive action pending user or policy decision.
- Concurrent agent runs: Multiple active agent runs across different trusted project folders, with one main run per chat session and isolated approval, output, artifact, runtime event, and cancellation state.
- Artifact: A generated output associated with a project, session, and originating run.
- Provider profile: A configured OpenAI-compatible provider connection.
- Model profile: A selectable model from a provider profile or manual entry.
- Local memory record: App-owned scoped memory, separate from raw provider state or runtime-local storage.
- Extension inventory item: A discovered MCP server, skill, plugin, or related scoped surface.
- Extension enablement: The explicit per-extension decision that allows a connected plugin, skill, or MCP server to affect agent execution, model context, tools, or app-owned state.
- Extension prompt tag: Chat syntax for explicit extension invocation: `$` for skills, `@` for plugins, and `^` for MCP servers.
