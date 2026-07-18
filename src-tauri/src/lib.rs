pub mod core;
pub mod execution;
pub mod protocol;
pub mod security;

use protocol::{
    FoundationSnapshot, ProtocolEnvelope, ProtocolError, ProtocolErrorCode, SnapshotRequest,
    StateGeneration, WorkspaceId, WorkspaceRecentSnapshot, WorkspaceStartSnapshot,
};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

struct AppCoreState {
    database: Arc<core::database::DatabaseActor>,
    _configuration: Mutex<core::services::ManagedAppConfiguration>,
    _action_gateway: Mutex<security::gateway::ActionGateway>,
    _credentials: CredentialServiceState,
}

struct CredentialServiceState {
    _vault: Option<security::credentials::CredentialVault>,
    _requires_explicit_fallback: bool,
}

impl CredentialServiceState {
    #[cfg(target_os = "macos")]
    fn initialize(
        c4os_home: &std::path::Path,
    ) -> Result<Self, security::credentials::CredentialVaultError> {
        use security::credentials::{
            CredentialVault, CredentialVaultError, MacOsInstallationKeyStore,
        };

        let key_store = MacOsInstallationKeyStore::new("com.c4os.desktop");
        match CredentialVault::open_or_create_with_installation_key(
            c4os_home.join("vault/credentials.vault"),
            &key_store,
        ) {
            Ok(vault) => Ok(Self {
                _vault: Some(vault),
                _requires_explicit_fallback: false,
            }),
            Err(CredentialVaultError::KeychainUnavailable) => Ok(Self {
                _vault: None,
                _requires_explicit_fallback: true,
            }),
            Err(error) => Err(error),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn initialize(
        _c4os_home: &std::path::Path,
    ) -> Result<Self, security::credentials::CredentialVaultError> {
        Ok(Self {
            _vault: None,
            _requires_explicit_fallback: true,
        })
    }
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
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(std::io::Error::other)?
                .as_millis()
                .try_into()
                .map_err(|_| std::io::Error::other("system time exceeds u64 milliseconds"))?;
            let action_gateway = security::gateway::ActionGateway::restore(
                security::policy::PolicyConfiguration::default(),
                Arc::clone(&database),
                now_ms,
            )?;
            let credentials = CredentialServiceState::initialize(&c4os_home)?;
            app.manage(AppCoreState {
                database,
                _configuration: Mutex::new(configuration),
                _action_gateway: Mutex::new(action_gateway),
                _credentials: credentials,
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
