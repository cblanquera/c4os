# Plugin System And Settings Management Requirements

Status: proposed

| ID | Requirement |
| --- | --- |
| REQ-001 | Support full Codex plugin compatibility while adding C4OS app-shell metadata in agents/c4os.yaml. |
| REQ-002 | Store plugin enablement, panel, icon order, marketplaces, and user config at user-global scope. |
| REQ-003 | Render plugin settings from simple field schema and reserved shell keys. |
| REQ-004 | Declare typed dependencies in agents/c4os.yaml for app plugins, C4OS/preinstalled native modules, heavy services, and required C4OS capabilities/tool identities; enforce manual plugin enablement, cascading disable, and visible blocked states. |
| REQ-005 | Support marketplace sources: bundled/default, repo, personal, local root, GitHub, Git URL, ref, and sparse paths. |
| REQ-006 | Constrain plugin SVG icons to installed bundle relative paths and sanitized static SVG. |
| REQ-007 | Treat per-chat plugin instances as lightweight view/state instances, not backend service/process instances; heavy plugin services use the narrowest safe shared lifecycle scope and lazy startup. |
| REQ-008 | Store plugin settings marked `sensitive` in C4OS-managed secure secret storage/keychain with only references or redacted placeholders in config files, and expose raw secret use only through governed C4OS calls. |
