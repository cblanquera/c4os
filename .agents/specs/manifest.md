# Specs Manifest

Status: active
Updated: 2026-07-02

## Specs

| Spec | Path | Status | Summary |
| --- | --- | --- | --- |
| Restart research | `research/` | frozen-for-mvp-specification | Imported research and validation records from `plans/product-brief.md`, `plans/product-interface.md`, `plans/pegs/*.png`, wireframes, and POCs. |
| Desktop MVP | `mvp/` | implementation-complete-accepted | Frozen MVP contract executed through TASK-017; no active MVP progress item remains. |
| Shell Plugin Architecture Refactor | `01-shell-plugin-architecture-refactor/` | proposed | Persistent chat shell and plugin/tool architecture boundary. |
| Core App Shell UX | `02-core-app-shell-ux/` | proposed | Single-header shell, plugin panels, Settings route, and layout behavior. |
| Plugin System And Settings Management | `03-plugin-system-settings-management/` | proposed | Codex-compatible marketplace, manifests, settings, dependencies, icons, lifecycle. |
| Runtime And Tool Policy | `04-runtime-tool-policy/` | proposed | Tool gateway, config.toml, Pi proof, approval policy, and tool taxonomy. |
| Chat Prompt Interactions | `05-chat-prompt-interactions/` | proposed | Approvals, branches, tags, attachments, and model adapter fallback. |
| File System Plugin | `06-file-system-plugin/` | proposed | Workspaces, project registry, project chats, search, clone, removal, relink. |
| File Editor Plugin | `07-file-editor-plugin/` | proposed | File explorer/editor, context menu, file operations, and icons. |
| Terminal Plugin | `08-terminal-plugin/` | proposed | User PTY panel per chat session and terminal UI preferences. |
| Chat Debug Plugin | `09-chat-debug-plugin/` | proposed | Developer debug panel, tool/CLI history, redaction, no export. |
| Browser Plugin | `10-browser-plugin/` | proposed | Browser navigation/actions, screenshots, annotations, and preview boundary. |
| Skills Settings | `11-skills-settings/` | proposed | Bundled/user skills, skill creator, invalid states, and `$` visibility. |

## Notes

- MVP implementation is accepted through
  `.agents/development/mvp/items/TASK-017-integration-release-readiness.md`.
- `.agents/development/mvp/manifest.md` is the authority for completed MVP
  progress state; its active item is `None`.
- Final-implementation planning is a new proposed stream imported from
  `.agents/references/research/final-implementation-import`. It is not active implementation and no final
  implementation spec is frozen yet.
- Imported archive material is provenance only. Reusable truth must be promoted
  into `.agents/context/`; long support belongs in `.agents/references/`; spec
  decisions belong in the affected spec's `decisions.md`.
