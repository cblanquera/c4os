# Shell Plugin Architecture Refactor Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Keep the persistent shell as chat prompt, active thread, user-global Chats history, and Settings. |
| REQ-002 | Model C4OS app plugins separately from Tauri plugins and keep built-ins installed but disabled by default. |
| REQ-003 | Support Codex-compatible plugin packaging, marketplace sources, installed cache semantics, and uninstall/reinstall behavior. |
| REQ-004 | Route plugin backend capability through the C4OS tool gateway and preinstalled native modules. |
| REQ-005 | Fan out one C4OS tool event to all enabled compatible plugin views without duplicate backend invocation. |
| REQ-006 | Define plugin migration failure handling and visible repair states. |
