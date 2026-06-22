# Backend Rust/Tauri Backend Surface

Status: TASK-004 review target.

`backend/` is the Rust/Tauri backend authority for the accepted r04 desktop
shell. The Tauri app lives under this root directory, not `src-tauri/`.
TASK-004 activates the first real user flow by letting `open_workspace` create
or load a non-secret `.c4os/workspace.json` descriptor for a local project
folder and return that real workspace identity to the accepted r04 shell.

This does not claim real provider, runtime, filesystem browsing/editing,
Browser, terminal, extension, security, approval, action, artifact, memory, or
deep persistence behavior.

## Commands

- `load_workspace` still returns the TASK-002 workspace payload shape.
- `open_workspace` canonicalizes a local folder, creates or loads
  `.c4os/workspace.json`, and returns a shell payload with the real workspace
  identity.
- `send_prompt` returns the fake successful agent transition payload.
- `create_session`, `read_file`, `save_file`, `run_terminal_command`,
  `open_browser_preview`, and `list_extensions` are Rust mock handlers matching
  the TASK-002 connector method inventory.

## Workspace Descriptor

The descriptor is stored at `.c4os/workspace.json` inside the selected folder.
It contains only non-secret identity and reference fields:

- schema version
- stable workspace id
- display name
- canonical root path
- trust flag
- created/updated timestamps

It does not store provider keys, transcripts, artifacts, local memory, terminal
history, Browser state, approvals, or runtime output.

## Package Scripts

- `npm run backend:build` builds the Rust/Tauri backend crate.
- `npm run backend:run` runs the Rust/Tauri backend app.
- `npm run backend:test` runs the Rust backend tests.

The frontend auto-selects the Tauri connector when the native `invoke` bridge is
available. Browser review can still force the TASK-002 HTTP mock connector with
`?connector=server&api=<origin>`.

## Native Desktop Menu

The desktop menu is installed as a Tauri app-wide OS menu in
`backend/src/lib.rs` through `menu::build_app_menu`. It is not an in-app toolbar,
not a duplicate settings panel, and not a replacement for accepted r04 controls.

Required menu items:

- `File > Open Workspace`
- `File > Save Workspace`
- `File > Save File`
- `Edit > Undo`
- `Edit > Redo`
- `Edit > Select All`
- `Edit > Cut`
- `Edit > Copy`
- `Edit > Paste`

`File > Save File` is enabled only when the file editor surface is open and the
current file can be saved. The menu starts with `Save File` disabled, and
`native_menu_state` applies focus-state updates to the real Tauri menu item.
`Edit` is enabled when focus is in an editable context such as the chat prompt,
Browser address bar, file editor, provider/settings input fields, or another
text-entry control.

## Still Mocked

- `load_workspace` boot state before opening a folder.
- Security and policy hardening around trust beyond the first descriptor-backed
  activation.
- Provider/model records and settings IA records.
- Session creation, structured thread/run events, and agent processing.
- Files, file editor content, artifacts/previews, Browser state, and Terminal
  output.
- Plugin, skill, MCP, extension, approval, local memory, action record,
  artifact, audit, and deeper persistence state.
