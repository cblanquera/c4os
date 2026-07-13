# Shell Plugin Architecture Refactor Acceptance

Status: proposed

| ID | Acceptance Criteria |
| --- | --- |
| AC-001 | A reader can distinguish persistent shell, plugin registry, plugin view host, registered tool catalog, tool gateway, policy engine, service manager, event fanout, migration controller, and redacted audit record responsibilities. |
| AC-002 | Built-in app plugins are bundled/default marketplace entries, disabled by default, and uninstallable. |
| AC-003 | Runtime tool discovery, tool invocation, plugin views, and backend authority are separated; runtime discovery uses the C4OS Registered Tool Catalog and is not limited to visible or enabled plugin views. |
| AC-004 | Plugin migration failures are classified by the spec-local migration table: cache/reinstallable-bundle/stale-index failures may auto-recover, corrupt plugin-owned config may reset and keep enabled only when the core contract remains valid, and schema/dependency/security/native-module/manifest/source failures produce visible blocked, repair, or disabled states. |
| AC-005 | Tool event fanout runs the backend call once, records one event, updates visible compatible views, lets hidden compatible plugin instances update per-chat state/unread indicators, and proves hidden plugins do not open panels, steal focus, prompt the user, or trigger duplicate backend calls. |
| AC-006 | View-oriented tools can execute without an enabled or visible compatible plugin view when policy allows, and C4OS stores app-owned per-chat inspectable tool result state for later compatible views instead of silently dropping the result. Multiple compatible plugin views hydrate the same shared source state while keeping view-local UI state separate; shared source state can be mutated or deleted only through explicit C4OS-governed actions. |
| AC-007 | Per-chat plugin instances are lightweight view/state instances; heavy plugin services are not spawned per chat by default and are shared at the narrowest safe lifecycle scope. |
| AC-008 | Redacted execution/lifecycle/audit records can exist independently of Chat Debug, but Chat Debug remains disabled by default and may display those records only under its redaction and no-export rules. |
| AC-009 | The architecture model explicitly aligns uncertain behavior with OpenAI/Codex, Claude/Anthropic, and MCP/open standards precedence, or records a C4OS-specific deviation. |
| AC-010 | The implementation plan preserves separate authority modules, event-driven communication, memory/RAM lifecycle controls, Windows platform-adapter provisions, and worker-friendly shell/plugin responsibilities. |
