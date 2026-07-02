# Chat Debug Plugin Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Chat Debug is disabled by default everywhere and developer-oriented. |
| REQ-002 | Show CLI commands/results, tool use, tool events, and approvals for active chat. |
| REQ-003 | Show current and historical runs for the active chat session. |
| REQ-004 | Always redact secrets, tokens, raw credentials, provider keys, auth headers, cookies, sensitive params, and plugin settings marked `sensitive`. |
| REQ-005 | Do not provide export. |
| REQ-006 | Define a typed debug event schema for runtime, tool, approval, plugin lifecycle, terminal-tool, attachment, structured error, and audit-summary events. |
| REQ-007 | Redact sensitive data before debug display and before debug persistence. |
| REQ-008 | Bound debug history by chat, count/size limit, and chat deletion cleanup. |
| REQ-009 | Keep debug copy understandable for support/operations/admin diagnosis, not only developers. |
