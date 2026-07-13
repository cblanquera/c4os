# File System Plugin Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Store project/workspace registry and chats in user-level app state, not project-root .c4os folders. |
| REQ-002 | Support explicit workspace.c4os.json load/save as workspace name plus project folders. |
| REQ-003 | Key project chats by canonical project folder path and share them across workspace files. |
| REQ-004 | Handle missing projects with muted strike-through row, read-only sessions, and relocate action. |
| REQ-005 | Support Create New Workspace menu, add-project popover, choose folder, clone repository, reorder, collapse/expand, rename, remove, reveal/copy path. |
| REQ-006 | Search projects/chat threads with center-screen takeover and close action. |
| REQ-007 | Ensure Browser activation does not create a Browser session chat item. |
| REQ-008 | Route user config/app data paths, path normalization, file watching, reveal/copy path, and trash/recycle operations through platform adapters with Windows provisions. |
| REQ-009 | Store enough project metadata to support explicit relink/migration when canonical path identity changes. |
| REQ-010 | Keep Git/branch/clone behavior conditional; non-Git workspaces must remain first-class for operations and normal knowledge work. |
| REQ-011 | Emit typed workspace/project lifecycle events for shell and plugin hydration. |
