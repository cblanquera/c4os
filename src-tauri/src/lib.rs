use serde::Serialize;

pub mod action_classifier;
pub mod approval;
pub mod approval_prompt;
pub mod artifact;
pub mod credentials;
pub mod event_normalizer;
pub mod file_browser;
pub mod file_tools;
pub mod instruction_preflight;
pub mod path_resolver;
pub mod project;
pub mod provider;
pub mod recovery_view;
pub mod runtime;
pub mod runtime_control;
pub mod session_allow;
pub mod session_catalog;
pub mod session_view;
pub mod shell_executor;
pub mod shell_summary;
pub mod storage;
pub mod tool_timeline;

#[derive(Serialize)]
struct DiagnosticsStatus {
    storage_mode: &'static str,
    product_telemetry: &'static str,
    automatic_crash_upload: bool,
    support_bundle_upload: bool,
}

#[derive(Serialize)]
struct AppStatus {
    app_name: &'static str,
    backend_ready: bool,
    telemetry_enabled: bool,
    diagnostics: DiagnosticsStatus,
}

#[tauri::command]
fn get_app_status() -> AppStatus {
    AppStatus {
        app_name: "C4OS",
        backend_ready: true,
        telemetry_enabled: false,
        diagnostics: DiagnosticsStatus {
            storage_mode: "local-only",
            product_telemetry: "disabled",
            automatic_crash_upload: false,
            support_bundle_upload: false,
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_status])
        .run(tauri::generate_context!())
        .expect("error while running C4OS");
}
