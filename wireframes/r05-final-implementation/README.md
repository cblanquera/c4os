# C4OS r05 Final Implementation Shell Foundation Wireframe

Draft stage: wireframe review, Batch 1.

This revision is a new major wireframe revision because the final implementation shell model changes the r04 layout contract. It keeps r04 as reference history only and does not copy the full r04 route set.

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
| `./index.html#coverage` | r05 route/state coverage matrix for specs 01 and 02. |

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
| `#settings-configuration` | Deferred | Configuration details are outside Batch 1 shell foundation. |
| `#settings-plugins` | Partially carried forward through `#settings` and `#repair-state` | Batch 1 only needs plugin configuration placement and repair entry. |
| `#settings-skills` | Deferred | Skills settings are outside Batch 1 shell foundation. |
| `#settings-mcp` | Deferred | MCP settings are outside Batch 1 shell foundation. |

## Simulation Boundary

This is static HTML/CSS/JS for wireframe review only. Plugin state, tool fanout, panel restore, resize collision, Settings restore, Chat Debug history, and repair state are simulated to make the shell behavior reviewable before implementation.

Safe to delete: yes. This is a review artifact, not product code.
