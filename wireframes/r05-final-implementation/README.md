# C4OS r05 Final Implementation Shell Foundation Wireframe

Approval stage: Batches 1, 2, 3, 4, and 5 approved.

This revision is the continuing r05 final-implementation wireframe because Batch
1 was approved and not rejected. It keeps the approved r05 shell model,
Settings structure, prompt conventions, and workspace/files behavior. Batch 5
now keeps only visible Terminal, Browser, and Chat Debug panel surfaces as
review routes. Backend lifecycle, cleanup, hydration, retention, and policy
requirements are tracked in coverage and notes instead of standalone screens.

## Scope

- Single global header.
- Left and right plugin icon groups.
- No default right panel.
- Primary plugin icon click toggles a plugin panel.
- One visible panel per side.
- Same-side plugin icon clicks replace the visible panel.
- Per-chat panel restore.
- Settings as a center route that closes panels and restores them after return.
- Resize/collision behavior preserving a 640px center pane.
- Hidden plugin unread/activity indicators.
- Invalid shell layout repair state.
- Settings > Plugins list, detail/settings renderer, marketplace source/install
  flow, dependency/incompatibility/pending-restart/repair states, uninstall
  data prompt, sensitive setting redaction, icon fallback, and panel/icon-order
  configuration.
- Settings > Configuration per-server-tool policy rows, remembered approval
  rule review/edit/revoke controls, and config parse-error/last-valid fallback.
- Settings > Skills list, metadata-first detail, bundled customization copy,
  and invalid skill states hidden from `$` suggestions.
- Prompt suggestions for `$` skills, `@` files/plugin resources, and `/`
  runtime commands with backend-authoritative resolution boundary.
- Trigger-based prompt reference behavior with active query, typeahead menu,
  resolved inline reference, unresolved token, and serialized prompt states;
  resolved references render as inline blue text rather than chips.
- Approval dialog/popover states for Deny, Deny and wait, Allow once, Allow
  and remember, session-only versus user-global duration, remembered-rule
  summary, and Settings > Configuration routing.
- Disabled/dependency-blocked suggestion repair path.
- Branch choose/create popover shown only for Git-backed projects.
- File attachment chips, Browser screenshot attachments, unsupported
  attachment warning, and safe fallback messaging.
- Batch 3 prompt routes use the chat-session shape: Chats panel, thread list,
  work log, agent response, permission prompt, and composer dock.
- Workspace create/load/save, folder/repository opening controls, sidebar
  project reorder/collapse/rename/remove/reveal/copy
  path affordances, missing project muted strike-through with relocate and
  read-only behavior, center search takeover, and non-Git workspace state.
- File Explorer/File Editor plugin panel placement on either side, file context
  menu, Add to chat reference insertion, create/rename/delete-to-trash
  confirmation, save/revert dirty state, external-change conflict, icon theme
  behavior, hidden-file behavior, and non-code empty states.
- Terminal plugin user terminal panel derived from the r04 terminal output pane
  with the agent debug/results pane removed.
- Terminal lifecycle, chat deletion cleanup, UI preferences, config.toml
  shell/env/tool policy split, bounded output, backpressure, and scrollback are
  treated as spec/coverage behavior rather than standalone wireframe screens.
- Browser plugin visible navigation/actions, screenshot attach control,
  toolbar context menu, page right-click context menu, and PDF/document preview
  host state.
- Browser annotation behavior is captured in `./browser-annotations.md`
  instead of a rendered route.
- Browser clear-after-send, invisible runtime action hydration, local-file
  access, profile isolation, and security boundary behavior are treated as
  spec/coverage behavior unless a visible product state is later requested.
- Chat Debug active chat event timeline, current/historical run selector,
  event detail view, redacted sensitive fields, and no export controls.
- Chat Debug disabled-by-default plugin visibility, retention/limit, and
  cleanup behavior are treated as spec/settings/coverage behavior rather than
  standalone wireframe screens.

## Review Routes

| Route | Purpose |
| --- | --- |
| `./index.html#shell-foundation` | Default shell with no default right panel. |
| `./index.html#same-side-replacement` | Same-side replacement and active icon close behavior. |
| `./index.html#per-chat-restore` | Per-chat panel visibility restoration. |
| `./index.html#resize-collision` | Center minimum width and collision behavior. |
| `./index.html#hidden-activity` | Hidden compatible plugin activity indicator behavior. |
| `./index.html#debug` | Chat Debug command, tool call, result, and approval history. |
| `./index.html#terminal-user-pty` | r04 Terminal output pane with the agent debug/results pane removed. |
| `./index.html#browser-navigation` | Browser navigation/actions and screenshot attach control. |
| `./index.html#browser-menu` | Browser toolbar menu with zoom, reload, find, device toolbar, and settings actions. |
| `./index.html#browser-page-context-menu` | Browser page right-click menu with annotate, back/forward/reload, and inspect actions. |
| `./index.html#browser-preview-host` | Browser-native PDF preview and document-family rendered-output hosting. |
| `./index.html#debug-timeline` | Current/historical run selector with selected-run events. |
| `./index.html#debug-event-detail` | Event detail view with sensitive field redaction. |
| `./index.html#repair-state` | Invalid shell layout repair/disable state. |
| `./index.html#settings` | Settings center route and panel restore contract. |
| `./index.html#settings-plugins` | Settings > Plugins list with status, source, panel, icon order, fallback icon, and flow links. |
| `./index.html#settings-plugin-detail` | Plugin detail/settings renderer with input, number, switch, select, default-only, and redacted sensitive values. |
| `./index.html#settings-plugin-marketplace` | Plugins page with clickable marketplace/add source/install review flow. |
| `./index.html#settings-plugin-states` | Dependency-blocked, incompatible, pending-restart, repairable, and icon-fallback states. |
| `./index.html#settings-plugin-uninstall` | Uninstall prompt with plugin-owned data choice. |
| `./index.html#settings-configuration` | Per-server-tool policy rows with edit/revoke icon actions. |
| `./index.html#settings-config-error` | config.toml parse-error and last-valid fallback state. |
| `./index.html#settings-skills` | Clickable Skills list with source/status/suggestion boundaries. |
| `./index.html#settings-skill-detail` | Metadata-first skill detail. |
| `./index.html#settings-skill-customize` | Bundled read-only skill customization copy flow. |
| `./index.html#settings-skill-invalid` | Invalid skill states and `$` suggestion filtering. |
| `./index.html#prompt-suggestions` | Interactive `$`, `@`, and `/` trigger typeahead above the fixed composer, with inline blue resolved references. |
| `./index.html#approval-dialog` | Approval request with Deny, Deny and wait, Allow once, Allow and remember, Advanced metadata, and duration choices. |
| `./index.html#remembered-rule-summary` | Applied remembered-rule summary plus Settings > Configuration route. |
| `./index.html#blocked-suggestion-repair` | Disabled/dependency-blocked suggestion repair path. |
| `./index.html#branch-popover` | Branch choose/create popover and read-only current chat branch. |
| `./index.html#attachment-states` | File and Browser screenshot attachment records. |
| `./index.html#safe-fallback` | Unsupported attachment warning and safe provider fallback. |
| `./index.html#workspace-start` | Create, load, and save workspace file behavior without project-root metadata. |
| `./index.html#workspace-loaded` | Normal loaded workspace state with FS project list and center new-chat prompt. |
| `./index.html#workspace-missing-project` | Missing project muted strike-through, last-known path, read-only sessions, and Relocate/Copy path/Rename/Remove actions. |
| `./index.html#workspace-search` | Center-screen project/chat search takeover with close action. |
| `./index.html#workspace-non-git` | Non-Git workspace state with repository-only controls hidden. |
| `./index.html#files-left-panel` | File Editor plugin explorer mounted on the left side. |
| `./index.html#files-right-panel` | File Editor plugin explorer mounted on the right side. |
| `./index.html#file-editor` | File Editor plugin code view reached by clicking an explorer file. |
| `./index.html#file-context-menu` | File context menu with Add to chat, Copy Path, reveal, rename, and delete actions. |
| `./index.html#file-operations` | Create, rename, and guarded delete-to-trash confirmation behavior. |
| `./index.html#file-editor-dirty` | Editor dirty state with save/revert controls. |
| `./index.html#file-external-conflict` | External-change conflict requiring user choice before overwrite. |
| `./index.html#file-empty-states` | Non-code file and empty editor state. |
| `./index.html#coverage` | Batch 2 through Batch 5 route/state coverage matrix for specs 03 through 11. |

## r04 Route Carry-Forward Decision

| r04 route | r05 decision | Reason |
| --- | --- | --- |
| `#app-start` | Intentionally not copied forward | Spec 02 removes the r04 start-screen shell assumption for final shell review. |
| `#new-session` | Superseded by `#shell-foundation` | The new center chat route is reviewed without fixed r04 side tabs. |
| `#chat-session` | Carried forward as center chat concept only | Batch 1 needs chat continuity but not the full r04 message thread. |
| `#providers-popover` | Deferred | Prompt/model picker behavior belongs to later prompt interaction review. |
| `#models-popover` | Deferred | Prompt/model picker behavior belongs to later prompt interaction review. |
| `#file-explorer` | Carried forward as File Editor plugin explorer states | The r04 explorer density and click-to-editor behavior remain relevant, but the surface is now the File Editor plugin. |
| `#file-editor` | Carried forward as File Editor plugin code view | The r04 code-view/breadcrumb pattern remains relevant inside the File Editor plugin panel. |
| `#terminal` | Superseded as fixed right tab; Batch 5 keeps the user terminal output pane only | The r04 agent command preview/results pane moved out of Terminal review and belongs to Chat Debug/runtime output surfaces. |
| `#settings-providers` | Deferred | Provider settings are outside Batch 1 shell foundation. |
| `#settings-add-provider` | Deferred | Provider form details are outside Batch 1 shell foundation. |
| `#settings-models` | Deferred | Model settings are outside Batch 1 shell foundation. |
| `#settings-runtimes` | Deferred | Runtime settings are outside Batch 1 shell foundation. |
| `#settings-configuration` | Focused Batch 2 route added | Covers spec 04 server-tool policy, remembered approvals, and config fallback only. |
| `#settings-plugins` | Focused Batch 2 route added | Covers spec 03 plugin settings/configuration states without recreating all r04 catalog behavior. |
| `#settings-skills` | Focused Batch 2 route added | Covers spec 11 skills list/detail/customize/invalid behavior only. |
| `#settings-mcp` | Deferred | MCP settings are outside Batch 1 shell foundation. |

## Simulation Boundary

This is static HTML/CSS/JS for wireframe review only. Plugin state, tool fanout,
panel restore, resize collision, Settings restore, Chat Debug history, plugin
marketplace install, policy editing, config fallback, skill customization,
prompt resolution, approval decisions, remembered policy application, branch
creation, attachment records, provider fallback, workspace load/save, folder
selection, clone registration, project relocation, file operations, editor
conflict resolution, Browser navigation, Browser preview hosting, Chat Debug
timeline, redaction, and repair states are simulated to make the visible
behavior reviewable before implementation. Terminal lifecycle, cleanup, output
limits, Browser annotation behavior, Browser hydration/security, and Chat Debug
retention are tracked as markdown/coverage/spec behavior rather than visible
wireframe screens.

Safe to delete: yes. This is a review artifact, not product code.
