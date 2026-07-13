# Core App Shell UX Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | No default right panel appears without an enabled/toggled plugin. |
| AC-002 | Panel behavior matches one visible panel per side and per-chat restoration. |
| AC-003 | Resize rules preserve at least 640px for the chat center pane. |
| AC-004 | Plugin icons render in safe fixed-size slots with sanitized SVG/fallback behavior. |
| AC-005 | Hidden compatible plugin instances can update per-chat state and unread/activity indicators after fanned-out tool events, but do not open panels, steal focus, prompt the user, or trigger another backend call. |
| AC-006 | Multiple compatible plugin views can hydrate the same app-owned per-chat tool result state without claiming it, forking the source state, opening extra panels, or mutating/deleting the shared state outside explicit C4OS-governed actions. |
| AC-007 | Shell-reserved settings keys that affect layout and header behavior, including `panel`, `enabled`, and `iconOrder`, are validated before the shell applies them; invalid declarations show visible repair/disable reasons instead of corrupting shell layout. |
| AC-008 | The spec names each shell state transition and the C4OS event or persisted field that drives it, with no direct plugin-to-plugin UI coupling. |
| AC-009 | Shell empty states, repair states, and Settings labels avoid coding-only assumptions and are understandable for general worker workflows. |
