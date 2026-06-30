pub mod commands;
pub mod extensions;
pub mod files;
pub mod menu;
pub mod mock_data;
pub mod native_browser;
pub mod openrouter;
pub mod provider_models;
pub mod records;
pub mod runtime_adapter;
pub mod runtime_sessions;
pub mod terminal_pty;
pub mod tool_gateway;
pub mod workspace;

pub fn run() {
    run_with_args(std::env::args());
}

pub fn run_with_args<I>(args: I)
where
    I: IntoIterator<Item = String>,
{
    workspace::set_workspace_bootstrap_request(parse_workspace_bootstrap_args(args));
    tauri::Builder::default()
        .enable_macos_default_menu(false)
        .menu(menu::build_app_menu)
        .on_menu_event(menu::handle_menu_event)
        .invoke_handler(tauri::generate_handler![
            commands::load_workspace,
            commands::send_prompt,
            commands::open_workspace,
            commands::open_workspace_file,
            commands::save_workspace_file,
            commands::create_session,
            commands::load_session,
            commands::read_file,
            commands::save_file,
            commands::create_artifact_preview,
            commands::open_browser,
            native_browser::sync_native_browser,
            commands::run_terminal_command,
            commands::start_terminal_session,
            commands::write_terminal_input,
            commands::read_terminal_output,
            commands::resize_terminal_session,
            commands::stop_terminal_session,
            commands::open_browser_preview,
            commands::list_local_memory_records,
            commands::save_local_memory_record,
            commands::list_action_records,
            commands::list_audit_records,
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

pub fn parse_workspace_bootstrap_args<I>(args: I) -> Option<workspace::WorkspaceBootstrapRequest>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if let Some(path) = arg.strip_prefix("--workspace=") {
            return non_empty_arg(path).map(workspace::WorkspaceBootstrapRequest::Path);
        }
        if arg == "--workspace" {
            return args
                .next()
                .and_then(|path| non_empty_arg(&path))
                .map(workspace::WorkspaceBootstrapRequest::Path);
        }
        if let Some(path) = arg.strip_prefix("--workspace-file=") {
            return non_empty_arg(path).map(workspace::WorkspaceBootstrapRequest::File);
        }
        if arg == "--workspace-file" {
            return args
                .next()
                .and_then(|path| non_empty_arg(&path))
                .map(workspace::WorkspaceBootstrapRequest::File);
        }
    }
    None
}

fn non_empty_arg(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_013a_parses_explicit_workspace_launch_flags() {
        let path = parse_workspace_bootstrap_args([
            "c4os-backend".into(),
            "--workspace".into(),
            "/tmp/c4os-agent-qa".into(),
        ])
        .expect("workspace path flag");
        let file = parse_workspace_bootstrap_args([
            "c4os-backend".into(),
            "--workspace-file".into(),
            "/tmp/review-workspace.c4os.json".into(),
        ])
        .expect("workspace file flag");

        assert_eq!(
            path,
            crate::workspace::WorkspaceBootstrapRequest::Path("/tmp/c4os-agent-qa".into())
        );
        assert_eq!(
            file,
            crate::workspace::WorkspaceBootstrapRequest::File(
                "/tmp/review-workspace.c4os.json".into()
            )
        );
    }

    #[test]
    fn task_013a_no_launch_flag_keeps_human_start_screen_default() {
        assert_eq!(
            parse_workspace_bootstrap_args(["c4os-backend".into()]),
            None
        );
    }
}
