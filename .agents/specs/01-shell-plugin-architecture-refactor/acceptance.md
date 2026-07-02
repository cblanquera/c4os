# Shell Plugin Architecture Refactor Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | A reader can distinguish persistent shell responsibilities from plugin responsibilities. |
| AC-002 | Built-in app plugins are bundled/default marketplace entries, disabled by default, and uninstallable. |
| AC-003 | Runtime tool discovery, tool invocation, plugin views, and backend authority are separated. |
| AC-004 | Plugin migration failures are classified as auto-recoverable, disabled-with-reason, or reset-and-keep-enabled where accepted. |
| AC-005 | Tool event fanout runs the backend call once, records one event, updates visible compatible views, lets hidden compatible plugin instances update per-chat state/unread indicators, and proves hidden plugins do not open panels, steal focus, prompt the user, or trigger duplicate backend calls. |
| AC-006 | View-oriented tools can execute without an enabled or visible compatible plugin view when policy allows, and C4OS stores app-owned per-chat inspectable tool result state for later compatible views instead of silently dropping the result. Multiple compatible plugin views hydrate the same shared source state while keeping view-local UI state separate. |
| AC-007 | Per-chat plugin instances are lightweight view/state instances; heavy plugin services are not spawned per chat by default and are shared at the narrowest safe lifecycle scope. |
