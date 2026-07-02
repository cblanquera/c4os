# File System Plugin Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | Workspace files do not own chats; same project folder in multiple workspaces shows latest shared project chats. |
| AC-002 | Remove Chat permanently deletes chat history; Remove Project removes membership but preserves project chat history. |
| AC-003 | Clone form captures repository URL and local destination, then registers result as project. |
| AC-004 | Platform-specific path/config/reveal/watch/trash behavior has macOS, Linux, and Windows handling or a documented deferred validation note. |
| AC-005 | Missing project relink shows last known path, display name, read-only session behavior, and explicit migration result before chats are writable again. |
| AC-006 | Non-Git folders can be added, searched, opened, and used for chats without showing irrelevant Git-only controls. |
| AC-007 | Workspace/project lifecycle events are defined without requiring shell or plugin components to infer state from DOM text. |
