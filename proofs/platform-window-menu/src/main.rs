use tauri::menu::{
    Menu, MenuItemBuilder, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .menu(|app| {
            let settings = MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("CmdOrCtrl+,")
                .build(app)?;
            let app_menu = Submenu::with_items(
                app,
                app.package_info().name.clone(),
                true,
                &[
                    &PredefinedMenuItem::about(app, None, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &settings,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::services(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::hide_others(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;
            let file_menu = Submenu::with_items(
                app,
                "File",
                true,
                &[&PredefinedMenuItem::close_window(app, None)?],
            )?;
            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;
            let view_menu = Submenu::with_items(
                app,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(app, None)?],
            )?;
            let window_menu = Submenu::with_id_and_items(
                app,
                WINDOW_SUBMENU_ID,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, None)?,
                    &PredefinedMenuItem::maximize(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                ],
            )?;
            let help_menu = Submenu::with_id_and_items(
                app,
                HELP_SUBMENU_ID,
                "Help",
                true,
                &[],
            )?;
            Menu::with_items(
                app,
                &[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu, &help_menu],
            )
        })
        .on_menu_event(|app, event| {
            eprintln!("native menu event: {}", event.id().as_ref());
            if event.id().as_ref() == "settings" {
                if let Some(window) = app.get_webview_window("window-menu") {
                    let _ = window.eval("window.__C4OS_OPEN_SETTINGS__?.('native-menu')");
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running platform window menu proof");
}
