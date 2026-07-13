# Runtime And Tool Policy Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Expose runtime tool discovery and invocation through the C4OS tool gateway. |
| REQ-002 | Allow runtime discovery/invocation of registered tools even without app-plugin views, subject to approval policy. |
| REQ-003 | Define per-tool default policy and maximum authority using ask-by-default policy plus deny, deny-and-wait, allow-once, and allow-and-remember approval decisions, including remembered rules keyed by tool id, action/risk category, normalized target scope where applicable, and plugin id for plugin-contributed tools. |
| REQ-004 | Implement config.toml as source config written by Settings with last-valid fallback on parse errors. |
| REQ-005 | Route / prompt commands through runtime/tool gateway command handling with C4OS and plugin command definitions. |
| REQ-006 | Plan Pi proof for prompt execution, streaming, tool-call interception, approval denial, and resume. |
| REQ-007 | Keep C4OS as the owner of tool execution and app-level/per-chat tool result state while plugin views hydrate from C4OS-owned state and heavy plugin services use shared lifecycle scopes. |
| REQ-008 | Surface server-tool policy in Settings > Configuration as one policy item per registered server tool, including a user-readable explanation and review/edit/revoke controls for user-global remembered rules. |
| REQ-009 | Define the MCP-shaped C4OS tool descriptor, request envelope, lifecycle events, structured result envelope, and structured error taxonomy. |
| REQ-010 | Include request IDs, trace IDs, caller identity, target scope, approval decision, timeout, cancellation, memory/output caps, and redaction metadata in tool-call records. |
| REQ-011 | Resolve uncertain tool semantics against OpenAI/Codex tool/MCP patterns first, Claude/Anthropic conventions second, and MCP/open standards third, with explicit local deviations. |
