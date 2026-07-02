# Runtime And Tool Policy Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Expose runtime tool discovery and invocation through the C4OS tool gateway. |
| REQ-002 | Allow runtime discovery/invocation of registered tools even without app-plugin views, subject to approval policy. |
| REQ-003 | Define per-tool default policy and maximum authority using allow, ask, deny, and remember, including remembered rules keyed by tool id, action/risk category, normalized target scope where applicable, and plugin id for plugin-contributed tools. |
| REQ-004 | Implement config.toml as source config written by Settings with last-valid fallback on parse errors. |
| REQ-005 | Route / prompt commands through runtime/tool gateway command handling with C4OS and plugin command definitions. |
| REQ-006 | Plan Pi proof for prompt execution, streaming, tool-call interception, approval denial, and resume. |
| REQ-007 | Keep C4OS as the owner of tool execution and app-level/per-chat tool result state while plugin views hydrate from C4OS-owned state and heavy plugin services use shared lifecycle scopes. |
| REQ-008 | Surface server-tool policy in Settings > Configuration as one policy item per registered server tool, including a user-readable explanation and review/edit/revoke controls for user-global remembered rules. |
