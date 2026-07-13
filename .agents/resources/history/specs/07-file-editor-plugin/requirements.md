# File Editor Plugin Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Require the FS plugin before IDE can be enabled. |
| REQ-002 | Provide file explorer and editor panel configurable left or right. |
| REQ-003 | Right-click file context menu supports Copy Path and Add to chat. |
| REQ-004 | Add to chat inserts inline prompt tag reference, not hidden attachment or full file contents. |
| REQ-005 | Support full VS Code-like file icon theme with hidden files visible except .git. |
| REQ-006 | Own save/revert UI, active OS menu save commands, create, rename, guarded delete to trash/recycle, and external-change conflict handling. |
| REQ-007 | Execute all file operations through C4OS backend file services and policy gates, not direct renderer filesystem authority. |
| REQ-008 | Route delete-to-trash/recycle, path casing, hidden-file behavior, icon lookup, and external-change detection through platform-aware adapters. |
| REQ-009 | Emit typed editor/file-operation events for save, revert, conflict, create, rename, delete, and Add to chat. |
| REQ-010 | Keep editor UX useful for non-code text and operations files, not only source code. |
