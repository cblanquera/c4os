# Backend Rust/Tauri Mock Parity

Status: TASK-003 review target.

`backend/` is the Rust/Tauri backend authority for the accepted r04 desktop
shell. TASK-003 scaffolds the Tauri app under this root directory, not
`src-tauri/`, and registers Rust command handlers that intentionally return the
same mock workspace payload and fake run behavior proven by the TASK-002
`tests/server/` harness. It does not claim real provider, runtime, filesystem,
Browser, terminal, extension, security, approval, or persistence behavior.

## Mock Commands

- `load_workspace` returns the TASK-002 workspace payload shape.
- `send_prompt` returns the fake successful agent transition payload.
- `open_workspace`, `create_session`, `read_file`, `save_file`,
  `run_terminal_command`, `open_browser_preview`, and `list_extensions` are
  Rust mock handlers matching the TASK-002 connector method inventory.

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

- Workspace trust and trusted-root state.
- Provider/model records and settings IA records.
- Session creation, structured thread/run events, and agent processing.
- Files, file editor content, artifacts/previews, Browser state, and Terminal
  output.
- Plugin, skill, MCP, extension, approval, local memory, action record,
  workspace descriptor, and persistence state.
