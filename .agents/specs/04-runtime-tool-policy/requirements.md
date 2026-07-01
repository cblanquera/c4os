# Runtime And Tool Policy Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Expose runtime tool discovery and invocation through the C4OS tool gateway. |
| REQ-002 | Allow runtime discovery/invocation of registered tools even without app-plugin views, subject to approval policy. |
| REQ-003 | Define per-tool default policy and maximum authority using allow, ask, deny, and remember. |
| REQ-004 | Implement config.toml as source config written by Settings with last-valid fallback on parse errors. |
| REQ-005 | Route / prompt commands through runtime/tool gateway command handling with C4OS and plugin command definitions. |
| REQ-006 | Plan Pi proof for prompt execution, streaming, tool-call interception, approval denial, and resume. |
