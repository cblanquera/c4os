# Chat Prompt Interactions Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Make approval popover/dialog functional for allow, ask, deny, and remember decisions, including remembered-rule summaries that show tool, action/risk category, target scope where applicable, plugin id for plugin tools, and session-only versus user-global duration. |
| REQ-002 | Support $ skills, @ plugin resources/files, and / runtime/gateway commands. |
| REQ-003 | Hide disabled/dependency-blocked plugin resources from tag suggestions unless a repair path is shown. |
| REQ-004 | Support branch choose/create popover only when git exists; chat thread branch remains read-only. |
| REQ-005 | Store C4OS attachment records and let model adapters translate or degrade safely. |
| REQ-006 | Support file references, Browser screenshots, and many Browser annotation attachments. |
| REQ-007 | Route authoritative prompt tag resolution and `/` command execution through backend/gateway services rather than frontend-only parsing. |
| REQ-008 | Include source, target metadata, memory/size limits, provider compatibility, redaction policy, and degradation status in attachment/reference records. |
| REQ-009 | Use worker-friendly prompt labels and approval text that explain impact without assuming coding-only workflows. |
