# C4OS r05 Final Implementation Shell Foundation Wireframe

Draft stage: wireframe review, Batch 2.

This revision is the continuing r05 final-implementation wireframe because Batch
1 was approved and not rejected. It keeps the approved r05 shell model, uses r04
only as a still-relevant Settings structure reference, and adds only the
Settings and Configuration routes needed to review specs 03, 04, and 11.

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

## Review Routes

| Route | Purpose |
| --- | --- |
| `./index.html#shell-foundation` | Default shell with no default right panel. |
| `./index.html#same-side-replacement` | Same-side replacement and active icon close behavior. |
| `./index.html#per-chat-restore` | Per-chat panel visibility restoration. |
| `./index.html#resize-collision` | Center minimum width and collision behavior. |
| `./index.html#hidden-activity` | Hidden compatible plugin activity indicator behavior. |
| `./index.html#debug` | Chat Debug command, tool call, result, and approval history. |
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
| `./index.html#coverage` | Batch 2 route/state coverage matrix for specs 03, 04, and 11. |

## r04 Route Carry-Forward Decision

| r04 route | r05 decision | Reason |
| --- | --- | --- |
| `#app-start` | Intentionally not copied forward | Spec 02 removes the r04 start-screen shell assumption for final shell review. |
| `#new-session` | Superseded by `#shell-foundation` | The new center chat route is reviewed without fixed r04 side tabs. |
| `#chat-session` | Carried forward as center chat concept only | Batch 1 needs chat continuity but not the full r04 message thread. |
| `#providers-popover` | Deferred | Prompt/model picker behavior belongs to later prompt interaction review. |
| `#models-popover` | Deferred | Prompt/model picker behavior belongs to later prompt interaction review. |
| `#file-explorer` | Superseded as fixed right tab; deferred as plugin content | Files becomes a plugin panel contribution, not a permanent r04 right tab. |
| `#file-editor` | Superseded as fixed right tab; deferred as plugin content | File editing belongs to the File Editor plugin review. |
| `#terminal` | Superseded as fixed right tab; deferred as plugin content | Terminal becomes a plugin panel contribution. |
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
marketplace install, policy editing, config fallback, skill customization, and
repair states are simulated to make the behavior reviewable before
implementation.

Safe to delete: yes. This is a review artifact, not product code.
