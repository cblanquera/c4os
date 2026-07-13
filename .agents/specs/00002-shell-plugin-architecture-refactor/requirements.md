# Shell Plugin Architecture Refactor Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Keep the persistent shell as chat prompt, active thread, user-global Chats history, and Settings. |
| REQ-002 | Model C4OS app plugins separately from Tauri plugins and keep built-ins installed but disabled by default. |
| REQ-003 | Support Codex-compatible plugin packaging, marketplace sources, installed cache semantics, and uninstall/reinstall behavior. |
| REQ-004 | Route plugin backend capability through the C4OS tool gateway and preinstalled native modules. |
| REQ-005 | Fan out one C4OS tool event to all enabled compatible plugin views without duplicate backend invocation. |
| REQ-006 | Use a C4OS-owned Plugin Registry, Lifecycle Controller, Manifest Validator, Migration Controller, Registered Tool Catalog, Tool Gateway, Policy Engine, Service Manager, Event Fanout, and Redacted Audit Records as separate named authorities. |
| REQ-007 | Keep per-chat plugin instances as lightweight view/state instances and keep heavy plugin services shared at the narrowest safe lifecycle scope instead of spawning one backend service/process per chat. |
| REQ-008 | Classify plugin migration failures as auto-recoverable, dependency-blocked, repairable, disabled-with-reason, or reset-plugin-config-and-keep-enabled using the spec-local migration table. |
| REQ-009 | Resolve uncertain shell/plugin architecture behavior against OpenAI/Codex standards first, Claude/Anthropic conventions second, and MCP/open standards third, with explicit C4OS deviations. |
| REQ-010 | Keep shell/plugin architecture event-driven, memory-aware, Windows-ready through platform adapters, worker-friendly, and maintainable through separate authority modules. |
