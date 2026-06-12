# Source Documents

| Source | Role | Notes |
| --- | --- | --- |
| `.agents/specs/c4os-general-workspace-roadmap/` | Parent roadmap | Defines TASK-001, audience priorities, standards refresh, and feature-spec split decision. |
| `.agents/specs/c4os-mvp/` | Current MVP/V1 scope | Defines accepted safety, project, session, provider, artifact, skills, MCP, retention, and compatibility boundaries. |
| `.agents/progress/manifest.md` | Verified work state | Shows MVP/V1 items through `item-049` verified. |
| `src-tauri/src/project_selector.rs` | Current project selector state | Lists registered projects and explicitly disables search, grouping, archive, delete, favorites, metadata editing, cross-project views, non-Git projects, and worktree management. |
| `src-tauri/src/session_catalog.rs` | Current session catalog state | Lists project sessions and latest session, while disabling search, archive, delete, and concurrent active sessions in catalog state. |
| `src/main.js` | Current UI shell | Shows first-run provider/project/session workflow and status tiles. |
| `tests/backend-command-boundary.test.mjs` | Current status contract | Exposes current support tiers and deferred capability flags. |
