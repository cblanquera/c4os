pub mod core;
pub mod protocol;

use protocol::{
    FoundationSnapshot, ProtocolEnvelope, ProtocolError, ProtocolErrorCode, SnapshotRequest,
    StateGeneration, WorkspaceId, WorkspaceRecentSnapshot, WorkspaceStartSnapshot,
};
use std::sync::{Arc, Mutex};
use tauri::Manager;

struct AppCoreState {
    database: Arc<core::database::DatabaseActor>,
    _configuration: Mutex<core::services::ManagedAppConfiguration>,
}

#[tauri::command]
fn foundation_snapshot(
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<FoundationSnapshot>, protocol::StructuredCoreError> {
    protocol::foundation_snapshot(request)
}

#[tauri::command]
fn workspace_start_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<WorkspaceStartSnapshot>, protocol::StructuredCoreError> {
    let correlation_id = request.correlation_id.clone();
    let state = core::services::load_workspace_start_state(&core.database)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let recents = state
        .recents
        .into_iter()
        .map(|recent| {
            Ok(WorkspaceRecentSnapshot {
                workspace_id: WorkspaceId::new(recent.workspace_id)?,
                display_name: recent.display_name,
                last_opened_at: recent.last_opened_at,
                is_missing: recent.is_missing,
            })
        })
        .collect::<Result<Vec<_>, ProtocolError>>()?;

    protocol::workspace_start_snapshot(
        request,
        WorkspaceStartSnapshot {
            protocol_version: protocol::PROTOCOL_VERSION,
            generation: StateGeneration(state.generation),
            authority: "rust-core".into(),
            recents,
        },
    )
}

fn workspace_state_unavailable(correlation_id: protocol::CorrelationId) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorCode::Unavailable,
        "Workspace state is unavailable",
        true,
    )
    .with_correlation(correlation_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let c4os_home = app.path().home_dir()?.join(".c4os");
            let (database, _) = core::database::DatabaseActor::start(
                core::database::DatabaseDescriptor::app(&c4os_home),
            )?;
            let database = Arc::new(database);
            let configuration = core::services::ManagedAppConfiguration::start(
                Arc::clone(&database),
                core::workspace::C4osHomeLayout::new(&c4os_home),
                core::configuration::ManagedCeilings::default(),
                core::configuration::SecurityConstraints::default(),
            )?;
            app.manage(AppCoreState {
                database,
                _configuration: Mutex::new(configuration),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            foundation_snapshot,
            workspace_start_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("C4OS application runtime failed");
}
