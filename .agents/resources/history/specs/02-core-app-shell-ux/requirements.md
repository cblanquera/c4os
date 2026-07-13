# Core App Shell UX Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Remove start screen, right-panel tabs, and right-panel collapse icon from the final shell. |
| REQ-002 | Provide one header with plugin icons configured left or right and reordered in the header. |
| REQ-003 | Make primary plugin icon click toggle that plugin panel; configuration lives in Settings > Plugins. |
| REQ-004 | Allow one visible plugin panel per side, left and right together, with same-side replacement. |
| REQ-005 | Persist visible plugin panels per chat session and restore them on session switch. |
| REQ-006 | Enforce 640px center-pane minimum and collision behavior. |
| REQ-007 | Open Settings as a center route that closes panels and restores per-chat panels when leaving. |
| REQ-008 | Define shell layout as a documented event-driven state machine for icon click, panel replacement, Settings route, resize collision, session switch, and hidden plugin event delivery. |
| REQ-009 | Keep visible shell language and repair/empty states worker-friendly and not coding-only. |
