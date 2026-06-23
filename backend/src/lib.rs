pub mod commands;
pub mod files;
pub mod menu;
pub mod mock_data;
pub mod openrouter;
pub mod provider_models;
pub mod runtime_adapter;
pub mod runtime_sessions;
pub mod workspace;

pub fn run() {
    tauri::Builder::default()
        .enable_macos_default_menu(false)
        .menu(menu::build_app_menu)
        .on_menu_event(menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            commands::load_workspace,
            commands::send_prompt,
            commands::open_workspace,
            commands::create_session,
            commands::load_session,
            commands::read_file,
            commands::save_file,
            commands::create_artifact_preview,
            commands::open_browser,
            commands::run_terminal_command,
            commands::open_browser_preview,
            commands::list_extensions,
            commands::list_provider_profiles,
            commands::save_provider_profile,
            commands::delete_provider_profile,
            commands::list_provider_models,
            commands::set_model_enabled,
            commands::set_provider_enabled,
            commands::select_session_model,
            commands::native_menu_contract,
            commands::native_menu_state
        ])
        .run(tauri::generate_context!())
        .expect("failed to run C4OS Tauri backend");
}
