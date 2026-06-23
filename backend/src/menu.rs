use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tauri::{
    menu::{MenuBuilder, MenuItem as TauriMenuItem, SubmenuBuilder},
    AppHandle, Emitter, Manager, Runtime,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuContract {
    pub kind: String,
    pub menus: Vec<Menu>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Menu {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_when: Option<String>,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_when: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusState {
    pub editable: bool,
    pub file_editor_open: bool,
    pub file_can_save: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuState {
    pub menus: BTreeMap<String, EnabledState>,
    pub commands: BTreeMap<String, EnabledState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnabledState {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

pub fn menu_contract() -> MenuContract {
    MenuContract {
        kind: "native-desktop-menu".into(),
        menus: vec![
            Menu {
                id: "file".into(),
                label: "File".into(),
                enabled_when: None,
                items: vec![
                    MenuItem {
                        id: "file.openWorkspace".into(),
                        label: "Open Workspace".into(),
                        enabled_when: None,
                        handler: Some("open_workspace".into()),
                        role: None,
                    },
                    MenuItem {
                        id: "file.saveWorkspace".into(),
                        label: "Save Workspace".into(),
                        enabled_when: None,
                        handler: Some("save_workspace".into()),
                        role: None,
                    },
                    MenuItem {
                        id: "file.saveFile".into(),
                        label: "Save File".into(),
                        enabled_when: Some("file_editor_open".into()),
                        handler: Some("save_file".into()),
                        role: None,
                    },
                    MenuItem {
                        id: "file.revertFile".into(),
                        label: "Revert File".into(),
                        enabled_when: Some("file_editor_open".into()),
                        handler: Some("revert_file".into()),
                        role: None,
                    },
                ],
            },
            Menu {
                id: "edit".into(),
                label: "Edit".into(),
                enabled_when: Some("editable".into()),
                items: vec![
                    edit_item("edit.undo", "Undo", "undo"),
                    edit_item("edit.redo", "Redo", "redo"),
                    edit_item("edit.selectAll", "Select All", "select_all"),
                    edit_item("edit.cut", "Cut", "cut"),
                    edit_item("edit.copy", "Copy", "copy"),
                    edit_item("edit.paste", "Paste", "paste"),
                ],
            },
        ],
    }
}

pub fn build_app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let open_workspace = TauriMenuItem::with_id(
        app,
        "file.openWorkspace",
        "Open Workspace",
        true,
        Some("CmdOrCtrl+O"),
    )?;
    let save_workspace = TauriMenuItem::with_id(
        app,
        "file.saveWorkspace",
        "Save Workspace",
        true,
        Some("CmdOrCtrl+Shift+S"),
    )?;
    let save_file = TauriMenuItem::with_id(
        app,
        "file.saveFile",
        "Save File",
        false,
        Some("CmdOrCtrl+S"),
    )?;
    let revert_file = TauriMenuItem::with_id(
        app,
        "file.revertFile",
        "Revert File",
        false,
        Some("CmdOrCtrl+R"),
    )?;

    let file = SubmenuBuilder::with_id(app, "file", "File")
        .item(&open_workspace)
        .item(&save_workspace)
        .separator()
        .item(&save_file)
        .item(&revert_file)
        .build()?;

    let edit = SubmenuBuilder::with_id(app, "edit", "Edit")
        .undo()
        .redo()
        .separator()
        .select_all()
        .cut()
        .copy()
        .paste()
        .build()?;

    MenuBuilder::new(app).item(&file).item(&edit).build()
}

pub fn apply_native_menu_state<R: Runtime>(
    app: &AppHandle<R>,
    focus_state: &FocusState,
) -> tauri::Result<MenuState> {
    let state = evaluate_menu_state(focus_state);

    if let Some(menu) = app.menu() {
        if let Some(file) = menu.get("file").and_then(|item| item.as_submenu().cloned()) {
            if let Some(save_file) = file
                .get("file.saveFile")
                .and_then(|item| item.as_menuitem().cloned())
            {
                save_file.set_enabled(state.commands["file.saveFile"].enabled)?;
            }
            if let Some(revert_file) = file
                .get("file.revertFile")
                .and_then(|item| item.as_menuitem().cloned())
            {
                revert_file.set_enabled(state.commands["file.revertFile"].enabled)?;
            }
        }

        if let Some(edit) = menu.get("edit").and_then(|item| item.as_submenu().cloned()) {
            edit.set_enabled(state.menus["Edit"].enabled)?;
        }
    }

    Ok(state)
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    let id = event.id().0.as_str();
    if matches!(
        id,
        "file.openWorkspace" | "file.saveWorkspace" | "file.saveFile" | "file.revertFile"
    ) {
        let _ = app.emit("c4os://native-menu", id);
        if matches!(id, "file.saveFile" | "file.revertFile") {
            if let Ok(serialized_id) = serde_json::to_string(id) {
                let script = format!(
                    "globalThis.__c4osNativeMenuCommand && globalThis.__c4osNativeMenuCommand({serialized_id});"
                );
                for webview in app.webview_windows().values() {
                    let _ = webview.eval(script.clone());
                }
            }
        }
    }
}

pub fn evaluate_menu_state(focus_state: &FocusState) -> MenuState {
    let file_editor_open = focus_state.file_editor_open;
    let mut menus = BTreeMap::new();
    menus.insert(
        "File".into(),
        EnabledState {
            enabled: true,
            label: None,
        },
    );
    menus.insert(
        "Edit".into(),
        EnabledState {
            enabled: focus_state.editable,
            label: None,
        },
    );

    let mut commands = BTreeMap::new();
    for menu in menu_contract().menus {
        for item in menu.items {
            let is_edit_command = item.id.starts_with("edit.");
            let enabled = if item.id == "file.saveFile" || item.id == "file.revertFile" {
                file_editor_open
            } else {
                !is_edit_command || focus_state.editable
            };
            commands.insert(
                item.id,
                EnabledState {
                    enabled,
                    label: Some(item.label),
                },
            );
        }
    }

    MenuState { menus, commands }
}

fn edit_item(id: &str, label: &str, role: &str) -> MenuItem {
    MenuItem {
        id: id.into(),
        label: label.into(),
        enabled_when: Some("editable".into()),
        handler: None,
        role: Some(role.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_required_native_menu_items() {
        let contract = menu_contract();

        assert_eq!(contract.menus[0].label, "File");
        assert_eq!(
            contract.menus[0]
                .items
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Open Workspace",
                "Save Workspace",
                "Save File",
                "Revert File"
            ]
        );
        assert_eq!(
            contract.menus[1]
                .items
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            vec!["Undo", "Redo", "Select All", "Cut", "Copy", "Paste"]
        );
    }

    #[test]
    fn enables_file_editor_commands_when_editor_is_open() {
        let without_editor = evaluate_menu_state(&FocusState {
            editable: true,
            file_editor_open: false,
            file_can_save: true,
        });
        let with_editor = evaluate_menu_state(&FocusState {
            editable: false,
            file_editor_open: true,
            file_can_save: false,
        });

        assert!(!without_editor.commands["file.saveFile"].enabled);
        assert!(!without_editor.commands["file.revertFile"].enabled);
        assert!(with_editor.commands["file.saveFile"].enabled);
        assert!(with_editor.commands["file.revertFile"].enabled);
    }

    #[test]
    fn enables_edit_menu_for_editable_focus_contexts() {
        let browsing = evaluate_menu_state(&FocusState::default());
        let editing = evaluate_menu_state(&FocusState {
            editable: true,
            file_editor_open: false,
            file_can_save: false,
        });

        assert!(!browsing.menus["Edit"].enabled);
        assert!(editing.menus["Edit"].enabled);
        for command in [
            "edit.undo",
            "edit.redo",
            "edit.selectAll",
            "edit.cut",
            "edit.copy",
            "edit.paste",
        ] {
            assert!(editing.commands[command].enabled);
        }
    }
}
