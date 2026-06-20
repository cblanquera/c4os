# Extension Enablements And Prompt Tags

Status: accepted

C4OS includes plugin, skill, and MCP install/connect flows in MVP, but extensions can affect agent execution only after explicit per-extension enablement. Skills and plugins may be invoked implicitly through enabled "use when" routing or explicitly with `$skill` and `@plugin` tags; MCP servers require explicit `^mcp` invocation. This keeps extension discovery and connection in the viable product while preserving a clear runtime-impact gate.

## Consequences

- Extension records must show provenance, scopes, workspace/project scope, data shared, runtime/tool access, enabled state, and action audit.
- MCP invocation is explicit only.
- Prompt tag parsing is part of the MVP interaction model.
