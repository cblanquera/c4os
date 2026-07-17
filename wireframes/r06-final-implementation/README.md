# C4OS r06 Final Implementation Wireframe

Review stage: Batches 1 through 5 ready for acceptance.

This revision started as an exact copy of `../r04-single-page-app/`, then
integrated the approved r05 final-implementation feature routes into the r06
single-page app. r04-only routes remain available where r05 did not supersede
them. Approved r05 routes are rendered through the latest r05 feature renderer
inside this folder so the Browser, Terminal, Chat Debug, Settings, prompt, and
workspace corrections stay intact.

## Scope

- r04 App Start, provider/model popovers, provider/model/runtime/MCP settings,
  and legacy carry-forward routes remain reachable.
- Batch 1 shell foundation is available through the approved global header,
  left/right plugin icon groups, no default right panel, side replacement,
  per-chat restore, resize collision, hidden activity, repair, Settings, and
  Chat Debug routes.
- Batch 2 adds Settings > Plugins, plugin marketplace/detail/state/uninstall,
  Settings > Configuration policy/error states, and Settings > Skills
  list/detail/customize/invalid states.
- Batch 3 adds prompt suggestions, approval dialog, remembered rule summary,
  blocked suggestion repair, branch popover, attachments, and safe fallback.
- Batch 4 adds workspace create/load/save states, project list actions,
  missing-project behavior, workspace search, non-Git state, File Editor panel
  placement, file context menu, file operations, dirty editor, external
  conflict, and file empty states.
- Batch 5 adds only the visible runtime plugin panels approved in r05:
  Terminal user PTY, Browser navigation/menu/page-menu/preview host, and Chat
  Debug current run/timeline/event detail. Browser annotation behavior remains
  markdown-only in `./browser-annotations.md`.

## Review Routes

| Route | Purpose |
| --- | --- |
| `./index.html#app-start` | r04 folder-first entry state. |
| `./index.html#new-session` | r04 new-session center flow inside r06. |
| `./index.html#chat-session` | r04 chat-thread carry-forward state inside r06. |
| `./index.html#providers-popover` | r04 provider popover. |
| `./index.html#models-popover` | r04 model popover. |
| `./index.html#file-explorer` | r04 file explorer carry-forward state. |
| `./index.html#terminal` | r04 terminal carry-forward state. |
| `./index.html#settings-providers` | r04 provider settings. |
| `./index.html#settings-add-provider` | r04 add-provider form. |
| `./index.html#settings-models` | r04 model settings. |
| `./index.html#settings-runtimes` | r04 runtime settings. |
| `./index.html#settings-mcp` | r04 MCP settings. |
| `./index.html#shell-foundation` | Batch 1 final shell baseline with no default right panel. |
| `./index.html#same-side-replacement` | Batch 1 same-side panel replacement. |
| `./index.html#per-chat-restore` | Batch 1 per-chat panel restoration. |
| `./index.html#resize-collision` | Batch 1 center minimum/collision state. |
| `./index.html#hidden-activity` | Batch 1 hidden-plugin activity indicator. |
| `./index.html#debug` | Batch 1/5 Chat Debug current run panel. |
| `./index.html#repair-state` | Batch 1 invalid plugin layout repair state. |
| `./index.html#settings` | Batch 1 Settings center route. |
| `./index.html#settings-plugins` | Batch 2 plugin settings list. |
| `./index.html#settings-plugin-detail` | Batch 2 plugin advanced settings renderer. |
| `./index.html#settings-plugin-marketplace` | Batch 2 marketplace/source/install flow. |
| `./index.html#settings-plugin-states` | Batch 2 dependency, incompatibility, pending restart, repair, and icon fallback states. |
| `./index.html#settings-plugin-uninstall` | Batch 2 plugin-owned data uninstall prompt. |
| `./index.html#settings-configuration` | Batch 2 per-server-tool policy rows and remembered approvals. |
| `./index.html#settings-config-error` | Batch 2 config parse-error and last-valid fallback. |
| `./index.html#settings-skills` | Batch 2 Skills list with source/status/suggestion boundaries. |
| `./index.html#settings-skill-detail` | Batch 2 metadata-first skill detail. |
| `./index.html#settings-skill-customize` | Batch 2 bundled skill customization copy flow. |
| `./index.html#settings-skill-invalid` | Batch 2 invalid skill state and `$` suggestion filtering. |
| `./index.html#prompt-suggestions` | Batch 3 `$`, `@`, and `/` prompt typeahead behavior. |
| `./index.html#approval-dialog` | Batch 3 approval request and duration choices. |
| `./index.html#remembered-rule-summary` | Batch 3 remembered-rule summary and Settings route. |
| `./index.html#blocked-suggestion-repair` | Batch 3 disabled/dependency-blocked suggestion repair path. |
| `./index.html#branch-popover` | Batch 3 Git branch choose/create popover. |
| `./index.html#attachment-states` | Batch 3 file and Browser screenshot attachment records. |
| `./index.html#safe-fallback` | Batch 3 unsupported attachment warning and safe provider fallback. |
| `./index.html#workspace-start` | Batch 4 create/load/save workspace entry state. |
| `./index.html#workspace-loaded` | Batch 4 normal loaded workspace state. |
| `./index.html#workspace-missing-project` | Batch 4 missing-project muted/read-only and action-menu state. |
| `./index.html#workspace-search` | Batch 4 center-screen workspace search takeover. |
| `./index.html#workspace-non-git` | Batch 4 non-Git workspace state. |
| `./index.html#files-left-panel` | Batch 4 File Editor explorer mounted on the left. |
| `./index.html#files-right-panel` | Batch 4 File Editor explorer mounted on the right. |
| `./index.html#file-editor` | Batch 4 File Editor code view. |
| `./index.html#file-context-menu` | Batch 4 file context menu. |
| `./index.html#file-operations` | Batch 4 create/rename/delete-to-trash confirmation. |
| `./index.html#file-editor-dirty` | Batch 4 dirty editor save/revert state. |
| `./index.html#file-external-conflict` | Batch 4 external-change conflict state. |
| `./index.html#file-empty-states` | Batch 4 non-code and empty editor states. |
| `./index.html#terminal-user-pty` | Batch 5 r04-derived user Terminal panel without agent debug/results. |
| `./index.html#browser-navigation` | Batch 5 Browser navigation/actions and screenshot control. |
| `./index.html#browser-menu` | Batch 5 Browser toolbar menu. |
| `./index.html#browser-page-context-menu` | Batch 5 Browser page right-click menu. |
| `./index.html#browser-preview-host` | Batch 5 Browser PDF/document preview host. |
| `./index.html#debug-timeline` | Batch 5 Chat Debug current/historical run selector. |
| `./index.html#debug-event-detail` | Batch 5 Chat Debug event detail with redaction. |
| `./index.html#coverage` | Batch 2 through Batch 5 route/state coverage matrix. |

## Simulation Boundary

This is static HTML/CSS/JS for wireframe review only. Plugin state, file
operations, prompt resolution, approvals, Terminal transport, Browser
navigation, Chat Debug history, policy editing, workspace persistence, and
repair flows are simulated. Review context and non-rendered Browser annotation
behavior live in Markdown files in this folder, not in visible shell UI.

Safe to delete: yes. This is a review artifact, not product code.
