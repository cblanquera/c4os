use crate::credentials::{CredentialStore, MacOsKeychainCredentialStore};
use crate::export::ProjectExportService;
use crate::mcp::{LocalStdioMcpRegistration, LocalStdioMcpRegistry, McpRegistryError};
use crate::project::{ProjectError, ProjectService};
use crate::project_selector::{ProjectSelectionError, ProjectSelector};
use crate::provider::{
    MetadataStatus, OpenRouterValidator, ProviderSetupError, ProviderSetupService,
    ProviderValidation,
};
use crate::runtime::{
    CodingRuntimeRunner, OpenCodeJsonRunner, RuntimePromptError, RuntimePromptRequest,
};
use crate::session_view::{RuntimeState, SessionView};
use crate::skills::{ProjectSkillCatalog, ProjectSkillError};
use crate::storage::{AppStore, ArchivedSessionDeleteError, NewMessage};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

pub mod action_classifier;
pub mod approval;
pub mod approval_prompt;
pub mod artifact;
pub mod credentials;
pub mod event_normalizer;
pub mod export;
pub mod file_browser;
pub mod file_tools;
pub mod instruction_preflight;
pub mod mcp;
pub mod path_resolver;
pub mod project;
pub mod project_selector;
pub mod provider;
pub mod recovery_view;
pub mod runtime;
pub mod runtime_control;
pub mod session_allow;
pub mod session_catalog;
pub mod session_view;
pub mod shell_executor;
pub mod shell_summary;
pub mod skills;
pub mod storage;
pub mod tool_timeline;

struct C4OsState {
    db_path: PathBuf,
    store: Mutex<AppStore>,
}

struct BasicOpenRouterValidator;

#[derive(Debug)]
enum SubmitPromptError {
    Message(String),
    Runtime(RuntimePromptError),
    Store(rusqlite::Error),
}

impl OpenRouterValidator for BasicOpenRouterValidator {
    fn validate_key(&self, key: &str) -> ProviderValidation {
        let key = key.trim();

        if key.is_empty() {
            return ProviderValidation::Invalid("OpenRouter key is required.".into());
        }

        if !key.starts_with("sk-or-") {
            return ProviderValidation::Invalid("OpenRouter key should start with sk-or-.".into());
        }

        ProviderValidation::Valid
    }
}

#[tauri::command]
fn get_app_status(state: tauri::State<'_, C4OsState>) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    app_status(&store).map_err(|error| error.to_string())
}

#[tauri::command]
fn configure_openrouter(
    state: tauri::State<'_, C4OsState>,
    api_key: String,
    selected_model: Option<String>,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let credential_store = MacOsKeychainCredentialStore::new("default");
    let validator = BasicOpenRouterValidator;
    let service = ProviderSetupService::new(&store, &credential_store, &validator);
    let selected_model = selected_model.as_deref();
    let provider = service
        .configure_openrouter(&api_key, selected_model)
        .map_err(provider_setup_error)?;

    Ok(json!({
        "id": provider.provider,
        "ready": provider.ready,
        "selectedModel": provider.selected_model,
        "metadataStatus": metadata_status(&provider.metadata_status),
        "disclosure": provider.disclosure,
    }))
}

#[tauri::command]
fn register_project(
    state: tauri::State<'_, C4OsState>,
    root_path: String,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let service = ProjectService::new(&store);
    let project = service
        .register_git_project(root_path)
        .map_err(project_error)?;
    let selector = ProjectSelector::select(&store, &project.id).map_err(selection_error)?;

    Ok(json!({
        "active": true,
        "id": project.id,
        "name": project.name,
        "rootPath": project.root_path,
        "branch": project.git_status.branch,
        "dirty": project.git_status.dirty,
        "changedFileCount": project.git_status.changed_files.len(),
        "hasRootAgents": project.has_root_agents,
        "registeredProjectCount": selector.projects.len(),
    }))
}

#[tauri::command]
fn list_project_skills(state: tauri::State<'_, C4OsState>) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    list_project_skills_for_store(&store).map_err(|error| error.to_string())
}

#[tauri::command]
fn invoke_project_skill(
    state: tauri::State<'_, C4OsState>,
    relative_path: String,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    invoke_project_skill_for_store(&store, &relative_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_mcp_servers(state: tauri::State<'_, C4OsState>) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    list_mcp_servers_for_store(&store).map_err(mcp_registry_error)
}

#[tauri::command]
fn register_local_stdio_mcp_server(
    state: tauri::State<'_, C4OsState>,
    name: String,
    command: String,
    args: Vec<String>,
    working_directory: String,
    remote_url: Option<String>,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    register_local_stdio_mcp_server_for_store(
        &store,
        LocalStdioMcpRegistration {
            name,
            transport: "stdio".into(),
            command,
            args,
            working_directory,
            remote_url,
        },
    )
    .map_err(mcp_registry_error)
}

#[tauri::command]
fn propose_mcp_server_launch(
    state: tauri::State<'_, C4OsState>,
    server_id: String,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    propose_mcp_server_launch_for_store(&store, &server_id).map_err(mcp_registry_error)
}

#[tauri::command]
fn export_project_json(state: tauri::State<'_, C4OsState>) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    export_project_json_for_store(&store).map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_archived_session(
    state: tauri::State<'_, C4OsState>,
    session_id: String,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let app_data_root = state
        .db_path
        .parent()
        .ok_or_else(|| "App data directory is unavailable.".to_string())?;

    delete_archived_session_for_store(&store, app_data_root, &session_id)
        .map_err(archived_session_delete_error)
}

fn list_project_skills_for_store(store: &AppStore) -> Result<Value, String> {
    let selector = ProjectSelector::list(&store).map_err(|error| error.to_string())?;
    let Some(active_project) = selector.active_project else {
        return Err("Register and select a Git project before listing skills.".into());
    };
    let catalog =
        ProjectSkillCatalog::new(active_project.root_path).map_err(project_skill_error)?;
    let skills = catalog.discover().map_err(project_skill_error)?;

    Ok(json!({
        "supportTier": "explicit_discovery_and_invocation_only",
        "discoveryAvailable": true,
        "explicitInvocationRequired": true,
        "autoInvocationAvailable": false,
        "scriptExecutionAvailable": false,
        "referencesAutoLoaded": false,
        "trustedAssetRendering": false,
        "globalCatalogAvailable": false,
        "pluginProvidedSkillsAvailable": false,
        "permissionEffect": "none",
        "modelContextEffect": "explicit_user_selected_context_only",
        "skills": skills,
    }))
}

fn invoke_project_skill_for_store(store: &AppStore, relative_path: &str) -> Result<Value, String> {
    let selector = ProjectSelector::list(store).map_err(|error| error.to_string())?;
    let Some(active_project) = selector.active_project else {
        return Err("Register and select a Git project before invoking skills.".into());
    };
    let catalog =
        ProjectSkillCatalog::new(&active_project.root_path).map_err(project_skill_error)?;
    let invocation = catalog
        .invoke(relative_path.trim())
        .map_err(project_skill_error)?;
    let session_id = SessionView::latest_for_project(store, &active_project.id)
        .map_err(|error| error.to_string())?
        .map(|session| session.session_id);

    if let Some(session_id) = session_id.as_deref() {
        store
            .append_message(NewMessage {
                id: &format!(
                    "skill-{}-{}",
                    timestamp(),
                    safe_message_suffix(&invocation.name)
                ),
                session_id,
                role: "user",
                content: &format!(
                    "Selected skill `{}` from `{}`.\n\n{}",
                    invocation.name, invocation.relative_path, invocation.content
                ),
                status: "submitted",
                metadata_json: r#"{"explicitUserSelectedSkill":true}"#,
            })
            .map_err(|error| error.to_string())?;
    }

    Ok(json!({
        "name": invocation.name,
        "description": invocation.description,
        "relativePath": invocation.relative_path,
        "content": invocation.content,
        "contextEffect": invocation.context_effect,
        "permissionEffect": invocation.permission_effect,
        "unsupportedCapabilities": invocation.unsupported_capabilities,
        "sessionId": session_id,
    }))
}

fn list_mcp_servers_for_store(store: &AppStore) -> Result<Value, McpRegistryError> {
    let selector = ProjectSelector::list(store).map_err(|_| McpRegistryError::StoreFailed)?;
    let Some(active_project) = selector.active_project else {
        return Err(McpRegistryError::MissingProject);
    };
    let registry = LocalStdioMcpRegistry::new(store, &active_project.id)?;
    let servers = registry
        .list()?
        .into_iter()
        .map(mcp_server_json)
        .collect::<Vec<_>>();

    Ok(json!({
        "supportTier": "local_stdio_explicit_approval_only",
        "servers": servers,
    }))
}

fn register_local_stdio_mcp_server_for_store(
    store: &AppStore,
    registration: LocalStdioMcpRegistration,
) -> Result<Value, McpRegistryError> {
    let selector = ProjectSelector::list(store).map_err(|_| McpRegistryError::StoreFailed)?;
    let Some(active_project) = selector.active_project else {
        return Err(McpRegistryError::MissingProject);
    };
    let registry = LocalStdioMcpRegistry::new(store, &active_project.id)?;
    let server = registry.register(registration)?;

    Ok(mcp_server_json(server))
}

fn propose_mcp_server_launch_for_store(
    store: &AppStore,
    server_id: &str,
) -> Result<Value, McpRegistryError> {
    let selector = ProjectSelector::list(store).map_err(|_| McpRegistryError::StoreFailed)?;
    let Some(active_project) = selector.active_project else {
        return Err(McpRegistryError::MissingProject);
    };
    let registry = LocalStdioMcpRegistry::new(store, &active_project.id)?;
    let proposal = registry.launch_proposal(server_id)?;

    Ok(json!({
        "serverId": proposal.server_id,
        "command": proposal.command,
        "workingDirectory": proposal.working_directory,
        "rootScope": proposal.root_scope,
        "requiresApproval": proposal.requires_approval,
        "riskLevel": proposal.risk_level,
        "approvalActionType": proposal.approval_action_type,
        "filteredEnvironment": proposal.filtered_environment,
        "startsProcess": proposal.starts_process,
        "unsupportedCapabilities": proposal.unsupported_capabilities,
    }))
}

fn mcp_server_json(server: crate::mcp::LocalStdioMcpServer) -> Value {
    json!({
        "id": server.id,
        "name": server.name,
        "transport": server.transport,
        "command": server.command,
        "args": server.args,
        "workingDirectory": server.working_directory,
        "rootScope": server.root_scope,
        "autoStart": server.auto_start,
        "remoteUrl": server.remote_url,
        "resourcesAutoContext": server.resources_auto_context,
        "promptsAutoContext": server.prompts_auto_context,
        "samplingAutoApproved": server.sampling_auto_approved,
        "elicitationAutoDisclosure": server.elicitation_auto_disclosure,
    })
}

fn export_project_json_for_store(store: &AppStore) -> Result<Value, String> {
    let selector = ProjectSelector::list(store).map_err(|error| error.to_string())?;
    let Some(active_project) = selector.active_project else {
        return Err("Register and select a Git project before exporting.".into());
    };
    let service = ProjectExportService::new(store);

    service
        .export_project_json(&active_project.id)
        .map_err(|error| error.to_string())
}

fn delete_archived_session_for_store(
    store: &AppStore,
    app_data_root: impl AsRef<std::path::Path>,
    session_id: &str,
) -> Result<Value, ArchivedSessionDeleteError> {
    let deleted = store.delete_archived_session(session_id.trim())?;
    let removed_artifact_files =
        remove_app_managed_artifact_paths(app_data_root.as_ref(), &deleted.artifact_paths)
            .map_err(|error| ArchivedSessionDeleteError::StoreFailed(error.to_string()))?;

    Ok(json!({
        "supportTier": "archived_session_delete_only",
        "deletedSessionId": deleted.session_id,
        "projectId": deleted.project_id,
        "fallbackSessionId": deleted.fallback_session_id,
        "removedArtifactFiles": removed_artifact_files,
        "automaticCleanup": false,
        "storageQuotaCleanup": false,
        "messageLevelDelete": false,
        "memoryDeleted": false,
        "importAvailable": false,
        "roundTripCompatibility": false,
    }))
}

fn remove_app_managed_artifact_paths(
    app_data_root: &std::path::Path,
    artifact_paths: &[String],
) -> std::io::Result<usize> {
    let artifact_root = app_data_root.join("artifacts");
    let mut removed = 0;

    for artifact_path in artifact_paths {
        let path = std::path::PathBuf::from(artifact_path);

        if !path.starts_with(&artifact_root) || !path.is_file() {
            continue;
        }

        fs::remove_file(&path)?;
        removed += 1;

        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }

    Ok(removed)
}

#[tauri::command]
fn submit_prompt(state: tauri::State<'_, C4OsState>, prompt: String) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let credential_store = MacOsKeychainCredentialStore::new("default");
    let run = prepare_prompt_run(&store, &credential_store, &prompt)
        .map_err(|error| error.to_string())?;
    let session = session_state(&store).map_err(|error| error.to_string())?;
    let db_path = state.db_path.clone();

    std::thread::spawn(move || {
        let Ok(store) = AppStore::open(db_path) else {
            return;
        };
        let runner = OpenCodeJsonRunner::default_from_workspace();

        finalize_prompt_run(&store, &runner, run);
    });

    Ok(session)
}

#[cfg(test)]
fn submit_prompt_with_runner(
    store: &AppStore,
    credential_store: &impl CredentialStore,
    runner: &impl CodingRuntimeRunner,
    prompt: &str,
) -> Result<Value, SubmitPromptError> {
    let run = prepare_prompt_run(store, credential_store, prompt)?;

    finalize_prompt_run(store, runner, run);

    Ok(session_state(store)?)
}

fn prepare_prompt_run(
    store: &AppStore,
    credential_store: &impl CredentialStore,
    prompt: &str,
) -> Result<PromptRun, SubmitPromptError> {
    let provider = provider_state(store)?;
    let selector = ProjectSelector::list(store)?;
    let prompt = prompt.trim();

    if !provider
        .get("ready")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err("Configure OpenRouter before starting a session.".into());
    }

    let Some(active_project) = selector.active_project else {
        return Err("Register and select a Git project before starting a session.".into());
    };

    if prompt.is_empty() {
        return Err("Enter a prompt before starting a session.".into());
    }

    let start_guard = SessionView::start_guard(store)?;

    if !start_guard.can_start {
        return Err(SubmitPromptError::Message(
            start_guard
                .reason
                .unwrap_or_else(|| "A session is already running.".into()),
        ));
    }

    let credential_reference =
        store
            .latest_openrouter_credential_reference()?
            .ok_or_else(|| {
                SubmitPromptError::Message("Missing OpenRouter credential reference.".into())
            })?;
    let openrouter_api_key = credential_store
        .read_openrouter_key(&credential_reference)
        .map_err(|error| SubmitPromptError::Message(format!("{error:?}")))?;
    let selected_model = provider
        .get("selectedModel")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            SubmitPromptError::Message("Select a model before starting a session.".into())
        })?
        .to_string();
    let session_id = format!("session-{}", timestamp());
    let message_id = format!("message-{}", timestamp());
    let title = first_line(prompt);

    store.start_openrouter_session(&session_id, &active_project.id, &title)?;
    store.append_message(NewMessage {
        id: &message_id,
        session_id: &session_id,
        role: "user",
        content: prompt,
        status: "submitted",
        metadata_json: "{}",
    })?;

    Ok(PromptRun {
        session_id,
        message_id,
        project_root: active_project.root_path.into(),
        prompt: prompt.into(),
        model_id: selected_model,
        openrouter_api_key,
    })
}

fn finalize_prompt_run(store: &AppStore, runner: &impl CodingRuntimeRunner, run: PromptRun) {
    match runner.run_prompt(
        store,
        RuntimePromptRequest {
            session_id: run.session_id.clone(),
            project_root: run.project_root,
            prompt: run.prompt,
            model_id: run.model_id,
            openrouter_api_key: run.openrouter_api_key,
        },
    ) {
        Ok(result) => {
            let _ = store.append_message(NewMessage {
                id: &format!("{}-assistant", run.message_id),
                session_id: &run.session_id,
                role: "assistant",
                content: &result.assistant_text,
                status: "complete",
                metadata_json: "{}",
            });
            let _ = store.update_session_status(&run.session_id, "complete");
        }
        Err(error) => {
            let _ = store.append_message(NewMessage {
                id: &format!("{}-assistant", run.message_id),
                session_id: &run.session_id,
                role: "assistant",
                content: &runtime_prompt_error_message(&error),
                status: "failed",
                metadata_json: "{}",
            });
            let _ = store.update_session_status(&run.session_id, "failed");
        }
    }
}

struct PromptRun {
    session_id: String,
    message_id: String,
    project_root: PathBuf,
    prompt: String,
    model_id: String,
    openrouter_api_key: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;

            fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("c4os.sqlite");
            let store = AppStore::open(&db_path)?;

            store.mark_active_sessions_interrupted()?;
            app.manage(C4OsState {
                db_path,
                store: Mutex::new(store),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            configure_openrouter,
            register_project,
            list_project_skills,
            invoke_project_skill,
            list_mcp_servers,
            register_local_stdio_mcp_server,
            propose_mcp_server_launch,
            export_project_json,
            delete_archived_session,
            submit_prompt
        ])
        .run(tauri::generate_context!())
        .expect("error while running C4OS");
}

fn app_status(store: &AppStore) -> rusqlite::Result<Value> {
    let selector = ProjectSelector::list(store)?;
    let active_project = selector.active_project.clone();
    let session = session_state(store)?;

    Ok(json!({
        "appName": "C4OS",
        "backendReady": true,
        "telemetryEnabled": false,
        "diagnostics": {
            "storageMode": "local-only",
            "productTelemetry": "disabled",
            "automaticCrashUpload": false,
            "supportBundleUpload": false,
        },
        "provider": provider_state(store)?,
        "providerExpansion": provider_expansion_status(),
        "workflowOrganization": workflow_organization_status(),
        "project": {
            "active": active_project.is_some(),
            "rootPath": active_project.as_ref().map(|project| project.root_path.clone()),
            "workflowPurpose": active_project.as_ref().and_then(|project| project.workflow_purpose.clone()),
            "branch": null,
            "dirty": false,
            "changedFileCount": 0,
            "browserMode": "read-only",
            "rootAgentsDisplay": "passive",
            "instructionResolution": {
                "nestedAgentsSupported": true,
                "supportTier": "display_guidance_order_only",
                "permissionEffect": "none",
                "modelContextEffect": "none_without_explicit_runtime_read",
                "editReloadBehavior": "next_preflight_only",
                "conflictDiagnostics": "ordered_sources_no_semantic_merge",
            },
            "selector": {
                "available": true,
                "listRegisteredProjects": true,
                "selectExactlyOneActive": true,
                "registeredProjectCount": selector.projects.len(),
                "multipleActiveProjectsAllowed": false,
                "searchAvailable": selector.search_available,
                "workflowPurposeFilterAvailable": selector.workflow_purpose_filter_available,
                "groupingAvailable": false,
                "archiveAvailable": false,
                "deleteAvailable": false,
                "favoritesAvailable": false,
                "metadataEditingAvailable": false,
                "crossProjectViewsAvailable": false,
                "nonGitProjectsAllowed": false,
                "worktreeManagementAvailable": false,
            },
        },
        "skills": skill_capability_status(),
        "mcp": mcp_capability_status(),
        "timeline": {
            "toolActivityVisible": true,
            "rawOutputExportAvailable": false,
            "liveDrawerLabel": "live/ephemeral",
        },
        "session": session,
        "retention": retention_status(),
        "memory": memory_status(),
        "compatibility": compatibility_status(),
        "browserWebViewing": browser_web_viewing_status(),
        "pluginMarketplace": plugin_marketplace_status(),
        "approvals": {
            "pendingCount": 0,
            "choices": ["allow_once", "allow_session", "deny"],
            "highRiskSessionAllow": false,
            "policyBlocksExecute": false,
        },
        "recovery": {
            "canStop": session.get("active").and_then(Value::as_bool).unwrap_or(false),
            "canRetry": false,
            "retryAppendsNewAction": true,
            "autoContinueAfterRestart": false,
            "reattachAfterRestart": false,
            "resendPendingPromptsAfterRestart": false,
        },
        "changes": {
            "changedFileCount": 0,
            "diffViewerAvailable": true,
            "gitInspectionLogged": true,
            "approvalRequiredForInspection": false,
            "refreshAfterToolExecution": true,
        },
        "artifacts": {
            "supportTier": "raster_image_preview_only",
            "visibleRecords": true,
            "searchAvailable": false,
            "activeHtmlPreview": false,
            "rasterImagePreview": true,
            "richPreview": true,
            "exportAvailable": true,
            "exportSupportTier": "project_json_export_only",
            "importAvailable": false,
            "roundTripCompatibility": false,
            "includeInModelContextByDefault": false,
            "supportedPreviewTypes": ["text", "markdown", "log", "diff", "source", "config", "png", "jpeg", "webp", "gif"],
            "unsupportedPreviewTypes": ["svg", "html", "pdf", "document", "spreadsheet", "remote_url"],
        },
        "portability": portability_status(),
    }))
}

fn skill_capability_status() -> Value {
    json!({
        "supportTier": "explicit_discovery_and_invocation_only",
        "discoveryAvailable": true,
        "explicitInvocationRequired": true,
        "autoInvocationAvailable": false,
        "scriptExecutionAvailable": false,
        "referencesAutoLoaded": false,
        "trustedAssetRendering": false,
        "globalCatalogAvailable": false,
        "pluginProvidedSkillsAvailable": false,
        "permissionEffect": "none",
        "modelContextEffect": "explicit_user_selected_context_only",
    })
}

fn mcp_capability_status() -> Value {
    let status = crate::mcp::mcp_capability_status();

    json!({
        "supportTier": status.support_tier,
        "localStdioAvailable": status.local_stdio_available,
        "remoteAvailable": status.remote_available,
        "explicitRegistrationRequired": status.explicit_registration_required,
        "autoStartFromProjectFiles": status.auto_start_from_project_files,
        "approvalRequiredForLaunch": status.approval_required_for_launch,
        "rootsLimitedToSelectedProject": status.roots_limited_to_selected_project,
        "toolsRequireApproval": status.tools_require_approval,
        "resourcesAutoContext": status.resources_auto_context,
        "promptsAutoContext": status.prompts_auto_context,
        "samplingAutoApproved": status.sampling_auto_approved,
        "elicitationAutoDisclosure": status.elicitation_auto_disclosure,
        "fullCompatibilityClaim": status.full_compatibility_claim,
    })
}

fn workflow_organization_status() -> Value {
    json!({
        "supportTier": "workflow_purpose_labels_only",
        "allowedPurposes": ["research", "writing", "documentation", "analysis"],
        "projectLabelsAvailable": true,
        "sessionLabelsAvailable": true,
        "unsetAllowed": true,
        "modelContextEffect": "none",
        "autoContextInjection": false,
        "hiddenFileIngestion": false,
        "templatesAvailable": false,
        "nonGitProjectsAllowed": false,
    })
}

fn portability_status() -> Value {
    json!({
        "supportTier": "project_json_export_only",
        "exportAvailable": true,
        "importAvailable": false,
        "roundTripCompatibility": false,
        "rawSecretsIncluded": false,
        "rawShellOutputIncluded": false,
        "providerKeysIncluded": false,
        "absoluteLocalPathsIncluded": false,
        "artifactPayloadsIncluded": false,
    })
}

fn retention_status() -> Value {
    json!({
        "supportTier": "archived_session_delete_only",
        "deleteAvailable": true,
        "explicitConfirmationRequired": true,
        "archivedUnpinnedSessionDelete": true,
        "activeSessionDelete": false,
        "latestSessionDelete": false,
        "runningSessionDelete": false,
        "pendingApprovalSessionDelete": false,
        "pinnedSessionDelete": false,
        "automaticCleanup": false,
        "storageQuotaCleanup": false,
        "messageLevelDelete": false,
        "messageRedaction": false,
        "memoryControls": false,
        "importAvailable": false,
        "roundTripCompatibility": false,
    })
}

fn memory_status() -> Value {
    json!({
        "supportTier": "no_durable_memory_v1",
        "durableMemoryAvailable": false,
        "crossSessionMemory": false,
        "learnedPreferences": false,
        "automaticSummaries": false,
        "embeddings": false,
        "backgroundIndexing": false,
        "memoryWritePrompts": false,
        "memoryInspectEditDeleteUi": false,
        "providerSideMemoryClaims": false,
        "memoryImportExport": false,
        "modelContextAutoInjection": false,
        "deletedSessionMemoryRecords": false,
    })
}

fn compatibility_status() -> Value {
    json!({
        "supportTier": "no_broader_compatibility_claims_v1",
        "claimsAreFeatureLevel": true,
        "openRouterBackedModelAccess": true,
        "localGitProjectSupport": true,
        "rootAgentsDisplay": true,
        "nestedAgentsDisplayGuidance": true,
        "explicitProjectLocalSkills": true,
        "localStdioMcpExplicitApproval": true,
        "appOwnedTextAndRasterArtifacts": true,
        "projectJsonExport": true,
        "archivedSessionDelete": true,
        "durableMemory": false,
        "fullAgentsMdCompatibility": false,
        "fullAgentSkillsCompatibility": false,
        "fullMcpCompatibility": false,
        "codexPluginCompatibility": false,
        "openCodeConfigCompatibility": false,
        "importCompatibility": false,
        "roundTripCompatibility": false,
        "browserCompatibility": false,
        "standardsCompatibleMarketingClaim": false,
    })
}

fn provider_expansion_status() -> Value {
    json!({
        "supportTier": "openrouter_only_v1_no_direct_or_local_provider_expansion",
        "openRouterOnly": true,
        "openRouterConfigurable": true,
        "directProviderSetup": false,
        "localModelProviderSetup": false,
        "offlineModelFallback": false,
        "providerFallbackRouting": false,
        "automaticModelSwitching": false,
        "byokProviderSubconfiguration": false,
        "providerSpecificSettings": false,
        "providerComparisonWorkflow": false,
        "multiProviderConfiguration": false,
        "credentialStorageRemainsKeychainOnly": true,
        "runningSessionCredentialMutation": false,
        "oneModelPerSession": true,
        "midSessionModelSwitching": false,
    })
}

fn browser_web_viewing_status() -> Value {
    json!({
        "supportTier": "no_browser_or_web_viewing_v1",
        "inAppBrowserPanel": false,
        "remoteUrlViewing": false,
        "domExtraction": false,
        "screenshots": false,
        "screenshotUnderstanding": false,
        "browserTesting": false,
        "browserAutomation": false,
        "webContentIngestion": false,
        "chromiumBackedRendering": false,
        "generatedHtmlExecution": false,
        "browserContentModelContextIngestion": false,
        "rendererIsolationSpecified": false,
        "promptInjectionMitigationSpecified": false,
    })
}

fn plugin_marketplace_status() -> Value {
    json!({
        "supportTier": "no_plugin_or_marketplace_v1",
        "pluginInstallAvailable": false,
        "pluginEnableAvailable": false,
        "pluginExecutionAvailable": false,
        "pluginScriptsHooks": false,
        "pluginPermissionGrants": false,
        "trustedPluginAssets": false,
        "pluginProvidedMcpServers": false,
        "marketplaceBrowsing": false,
        "remoteMarketplaceManifests": false,
        "pluginSearchInstallUpdate": false,
        "ratingsReviewsAdvisories": false,
        "curationWorkflows": false,
        "projectLocalPluginMetadataTrusted": false,
        "projectLocalPluginMetadataExecuted": false,
    })
}

fn provider_state(store: &AppStore) -> rusqlite::Result<Value> {
    let credential_store = MacOsKeychainCredentialStore::new("default");
    let validator = BasicOpenRouterValidator;
    let service = ProviderSetupService::new(store, &credential_store, &validator);
    let provider = service.current_state()?;

    Ok(json!({
        "id": provider.provider,
        "ready": provider.ready,
        "selectedModel": provider.selected_model,
        "metadataStatus": metadata_status(&provider.metadata_status),
        "disclosure": provider.disclosure,
    }))
}

fn session_state(store: &AppStore) -> rusqlite::Result<Value> {
    let selector = ProjectSelector::list(store)?;
    let screen = match selector.active_project {
        Some(project) => SessionView::latest_for_project(store, &project.id)?,
        None => None,
    };
    let start_guard = SessionView::start_guard(store)?;

    Ok(match screen {
        Some(screen) => json!({
            "active": matches!(
                screen.runtime_state,
                RuntimeState::Running | RuntimeState::WaitingForApproval
            ),
            "id": screen.session_id,
            "title": screen.title,
            "runtimeState": runtime_state(&screen.runtime_state),
            "runtimeStates": [
                "running",
                "waiting_for_approval",
                "stopped",
                "failed",
                "complete",
            ],
            "transcriptAppendOnly": true,
            "canStartNewRun": start_guard.can_start,
            "failureDisplay": screen.failure_display,
            "messages": screen.messages.into_iter().map(|message| {
                json!({
                    "id": message.id,
                    "role": message.role,
                    "content": message.content,
                    "status": message.status,
                })
            }).collect::<Vec<_>>(),
            "catalog": session_catalog_status(),
        }),
        None => json!({
            "active": false,
            "id": null,
            "title": null,
            "runtimeState": "stopped",
            "runtimeStates": [
                "running",
                "waiting_for_approval",
                "stopped",
                "failed",
                "complete",
            ],
            "transcriptAppendOnly": true,
            "canStartNewRun": start_guard.can_start,
            "failureDisplay": null,
            "messages": [],
            "catalog": session_catalog_status(),
        }),
    })
}

fn session_catalog_status() -> Value {
    json!({
        "searchAvailable": true,
        "workflowPurposeFilterAvailable": true,
        "workflowPurposeGroupingAvailable": true,
        "projectScopedOnly": true,
        "crossProjectSearchAvailable": false,
        "concurrentActiveSessions": false,
        "deleteSupportTier": "archived_session_delete_only",
    })
}

fn metadata_status(status: &MetadataStatus) -> &'static str {
    match status {
        MetadataStatus::Current => "current",
        MetadataStatus::Unknown => "unknown",
        MetadataStatus::Stale => "stale",
    }
}

fn runtime_state(status: &RuntimeState) -> &'static str {
    match status {
        RuntimeState::Running => "running",
        RuntimeState::WaitingForApproval => "waiting_for_approval",
        RuntimeState::Stopped => "stopped",
        RuntimeState::Failed => "failed",
        RuntimeState::Complete => "complete",
    }
}

fn provider_setup_error(error: ProviderSetupError) -> String {
    match error {
        ProviderSetupError::ActiveSession(message) => message,
        ProviderSetupError::ValidationFailed(message) => message,
        ProviderSetupError::CredentialStorage(error) => format!("{error:?}"),
    }
}

fn runtime_prompt_error_message(error: &RuntimePromptError) -> String {
    match error {
        RuntimePromptError::InvalidProjectRoot => {
            "OpenCode runtime failed because the selected project root is invalid.".into()
        }
        RuntimePromptError::MissingAssistantOutput => {
            "OpenCode completed without assistant output.".into()
        }
        RuntimePromptError::SpawnFailed(message) => {
            format!("OpenCode runtime could not start: {message}")
        }
        RuntimePromptError::ExecutionFailed(message) => {
            format!("OpenCode runtime failed: {message}")
        }
        RuntimePromptError::StoreFailed(message) => {
            format!("OpenCode runtime event storage failed: {message}")
        }
    }
}

fn project_error(error: ProjectError) -> String {
    match error {
        ProjectError::UnreadablePath(message) => message,
        ProjectError::NotGitRepository(message) => message,
        ProjectError::GitFailed(message) => message,
        ProjectError::StoreFailed(error) => error.to_string(),
    }
}

fn selection_error(error: ProjectSelectionError) -> String {
    match error {
        ProjectSelectionError::MissingProject => "Registered project was not found.".into(),
        ProjectSelectionError::StoreFailed => "Project selection failed.".into(),
    }
}

fn project_skill_error(error: ProjectSkillError) -> String {
    match error {
        ProjectSkillError::ProjectRootUnavailable => "Selected project root is unavailable.".into(),
        ProjectSkillError::DiscoveryFailed => "Project skill discovery failed.".into(),
        ProjectSkillError::TargetUnavailable => "Skill file was not found.".into(),
        ProjectSkillError::OutsideProjectRoot => {
            "Skill files must stay inside the selected project root.".into()
        }
        ProjectSkillError::SecretDenied => "Secret-deny files cannot be used as skills.".into(),
        ProjectSkillError::NotSkillFile => "Only SKILL.md files can be invoked as skills.".into(),
        ProjectSkillError::ReadFailed => "Skill file could not be read.".into(),
    }
}

fn mcp_registry_error(error: McpRegistryError) -> String {
    match error {
        McpRegistryError::MissingProject => {
            "Register and select a Git project before using MCP servers.".into()
        }
        McpRegistryError::RemoteUnavailable => {
            "Remote MCP is unavailable in V1. Register a local stdio server instead.".into()
        }
        McpRegistryError::InvalidTransport => {
            "Only local stdio MCP transport is available in V1.".into()
        }
        McpRegistryError::InvalidCommand => "MCP server command is required.".into(),
        McpRegistryError::SecretDenied => {
            "MCP server launch commands cannot include secret material.".into()
        }
        McpRegistryError::OutsideProjectRoot => {
            "MCP server working directories must stay inside the selected project root.".into()
        }
        McpRegistryError::StoreFailed => "MCP server registry storage failed.".into(),
    }
}

fn archived_session_delete_error(error: ArchivedSessionDeleteError) -> String {
    match error {
        ArchivedSessionDeleteError::MissingSession => "Session was not found.".into(),
        ArchivedSessionDeleteError::NotArchived => "Only archived sessions can be deleted.".into(),
        ArchivedSessionDeleteError::Pinned => {
            "Pinned sessions must be unpinned before deletion.".into()
        }
        ArchivedSessionDeleteError::ActiveSession => {
            "Running or approval-pending sessions cannot be deleted.".into()
        }
        ArchivedSessionDeleteError::LatestSession => {
            "The latest session cannot be deleted in this V1 tier.".into()
        }
        ArchivedSessionDeleteError::StoreFailed(message) => message,
    }
}

fn first_line(value: &str) -> String {
    value
        .lines()
        .next()
        .unwrap_or("Session")
        .chars()
        .take(80)
        .collect()
}

fn safe_message_suffix(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
        .take(32)
        .collect::<String>()
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

impl From<&str> for SubmitPromptError {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}

impl From<String> for SubmitPromptError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<rusqlite::Error> for SubmitPromptError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Store(value)
    }
}

impl From<RuntimePromptError> for SubmitPromptError {
    fn from(value: RuntimePromptError) -> Self {
        Self::Runtime(value)
    }
}

impl std::fmt::Display for SubmitPromptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => write!(formatter, "{message}"),
            Self::Runtime(error) => write!(formatter, "{}", runtime_prompt_error_message(error)),
            Self::Store(error) => write!(formatter, "{error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use crate::runtime::{RuntimePromptRequest, RuntimePromptResult};
    use crate::storage::{NewArtifact, NewProject, NewSession};
    use tempfile::tempdir;

    struct FakeRunner {
        assistant_text: String,
    }

    impl CodingRuntimeRunner for FakeRunner {
        fn run_prompt(
            &self,
            _app_store: &AppStore,
            request: RuntimePromptRequest,
        ) -> Result<RuntimePromptResult, RuntimePromptError> {
            assert_eq!(request.prompt, "Say hello");
            assert_eq!(request.model_id, "openai/gpt-4.1");
            assert_eq!(request.openrouter_api_key, "sk-or-test-secret");

            Ok(RuntimePromptResult {
                assistant_text: self.assistant_text.clone(),
            })
        }
    }

    #[test]
    fn submit_prompt_persists_runtime_assistant_output() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();
        let runner = FakeRunner {
            assistant_text: "Hello from OpenCode.".into(),
        };

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .set_selected_project("project-1")
            .expect("project selected");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");

        let session = submit_prompt_with_runner(&store, &credential_store, &runner, "Say hello")
            .expect("prompt submitted");
        let messages = session
            .get("messages")
            .and_then(Value::as_array)
            .expect("messages");

        assert_eq!(
            session.get("runtimeState").and_then(Value::as_str),
            Some("complete")
        );
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[1].get("content").and_then(Value::as_str),
            Some("Hello from OpenCode.")
        );
    }

    #[test]
    fn invoking_project_skill_appends_explicit_session_context() {
        let directory = tempdir().expect("tempdir");
        fs::create_dir_all(directory.path().join("skills/review")).expect("skill dir");
        fs::write(
            directory.path().join("skills/review/SKILL.md"),
            "---\nname: review\n---\n\nUse review guidance.",
        )
        .expect("skill written");
        let store = AppStore::open_in_memory().expect("store opens");

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .set_selected_project("project-1")
            .expect("project selected");
        store
            .create_session(crate::storage::NewSession {
                id: "session-1",
                project_id: "project-1",
                title: "Run",
                status: "complete",
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");

        let invocation = invoke_project_skill_for_store(&store, "skills/review/SKILL.md")
            .expect("skill invoked");
        let messages = store.list_messages("session-1").expect("messages");

        assert_eq!(
            invocation.get("sessionId").and_then(Value::as_str),
            Some("session-1")
        );
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].status, "submitted");
        assert!(messages[0].content.contains("Use review guidance."));
    }

    #[test]
    fn delete_archived_session_removes_only_app_managed_artifact_files() {
        let directory = tempdir().expect("tempdir");
        let app_data_root = directory.path().join("app-data");
        let artifact_dir = app_data_root.join("artifacts/project-1/artifact-1");
        fs::create_dir_all(&artifact_dir).expect("artifact dir");
        let original_path = artifact_dir.join("original");
        let preview_path = artifact_dir.join("preview");
        let outside_path = directory.path().join("outside-project-file.txt");
        fs::write(&original_path, "original").expect("original written");
        fs::write(&preview_path, "preview").expect("preview written");
        fs::write(&outside_path, "outside").expect("outside written");
        let project_root = directory.path().to_string_lossy().to_string();
        let original_path_text = original_path.to_string_lossy().to_string();
        let preview_path_text = preview_path.to_string_lossy().to_string();
        let outside_path_text = outside_path.to_string_lossy().to_string();
        let store = AppStore::open_in_memory().expect("store opens");

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: &project_root,
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .create_session(NewSession {
                id: "session-1",
                project_id: "project-1",
                title: "Archived",
                status: "complete",
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("archived session inserted");
        store
            .set_session_archived("session-1", true)
            .expect("session archived");
        store
            .record_artifact(NewArtifact {
                id: "artifact-1",
                project_id: "project-1",
                session_id: Some("session-1"),
                tool_call_id: None,
                artifact_type: "text",
                title: "App artifact",
                mime_type: Some("text/plain"),
                file_path: Some(&original_path_text),
                preview_path: Some(&preview_path_text),
                metadata_json: "{}",
            })
            .expect("artifact inserted");
        store
            .record_artifact(NewArtifact {
                id: "artifact-outside",
                project_id: "project-1",
                session_id: Some("session-1"),
                tool_call_id: None,
                artifact_type: "text",
                title: "Outside artifact",
                mime_type: Some("text/plain"),
                file_path: Some(&outside_path_text),
                preview_path: None,
                metadata_json: "{}",
            })
            .expect("outside artifact inserted");
        store
            .create_session(NewSession {
                id: "session-2",
                project_id: "project-1",
                title: "Fallback",
                status: "complete",
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("fallback session inserted");

        let deleted = delete_archived_session_for_store(&store, &app_data_root, "session-1")
            .expect("delete result");

        assert_eq!(
            deleted.get("supportTier").and_then(Value::as_str),
            Some("archived_session_delete_only")
        );
        assert_eq!(
            deleted.get("fallbackSessionId").and_then(Value::as_str),
            Some("session-2")
        );
        assert_eq!(
            deleted.get("removedArtifactFiles").and_then(Value::as_u64),
            Some(2)
        );
        assert!(!original_path.exists());
        assert!(!preview_path.exists());
        assert!(outside_path.exists());
    }

    #[test]
    fn app_status_reports_no_durable_memory_tier() {
        let store = AppStore::open_in_memory().expect("store opens");
        let status = app_status(&store).expect("status");
        let memory = status.get("memory").expect("memory status");

        assert_eq!(
            memory.get("supportTier").and_then(Value::as_str),
            Some("no_durable_memory_v1")
        );
        assert_eq!(
            memory
                .get("durableMemoryAvailable")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            memory.get("crossSessionMemory").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            memory.get("automaticSummaries").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            memory.get("embeddings").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            memory
                .get("modelContextAutoInjection")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn app_status_reports_no_broader_compatibility_claims_tier() {
        let store = AppStore::open_in_memory().expect("store opens");
        let status = app_status(&store).expect("status");
        let compatibility = status.get("compatibility").expect("compatibility status");

        assert_eq!(
            compatibility.get("supportTier").and_then(Value::as_str),
            Some("no_broader_compatibility_claims_v1")
        );
        assert_eq!(
            compatibility
                .get("claimsAreFeatureLevel")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            compatibility
                .get("fullAgentsMdCompatibility")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            compatibility
                .get("fullAgentSkillsCompatibility")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            compatibility
                .get("fullMcpCompatibility")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            compatibility
                .get("openCodeConfigCompatibility")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            compatibility
                .get("roundTripCompatibility")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            compatibility
                .get("browserCompatibility")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn app_status_reports_openrouter_only_provider_expansion_tier() {
        let store = AppStore::open_in_memory().expect("store opens");
        let status = app_status(&store).expect("status");
        let provider_expansion = status
            .get("providerExpansion")
            .expect("provider expansion status");

        assert_eq!(
            provider_expansion
                .get("supportTier")
                .and_then(Value::as_str),
            Some("openrouter_only_v1_no_direct_or_local_provider_expansion")
        );
        assert_eq!(
            provider_expansion
                .get("openRouterOnly")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            provider_expansion
                .get("directProviderSetup")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            provider_expansion
                .get("localModelProviderSetup")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            provider_expansion
                .get("providerFallbackRouting")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            provider_expansion
                .get("automaticModelSwitching")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            provider_expansion
                .get("multiProviderConfiguration")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn app_status_reports_workflow_navigation_foundation_tier() {
        let store = AppStore::open_in_memory().expect("store opens");
        let status = app_status(&store).expect("status");
        let workflow = status
            .get("workflowOrganization")
            .expect("workflow organization status");
        let project_selector = status
            .get("project")
            .and_then(|project| project.get("selector"))
            .expect("project selector status");
        let session_catalog = status
            .get("session")
            .and_then(|session| session.get("catalog"))
            .expect("session catalog status");

        assert_eq!(
            workflow.get("supportTier").and_then(Value::as_str),
            Some("workflow_purpose_labels_only")
        );
        assert_eq!(
            workflow.get("modelContextEffect").and_then(Value::as_str),
            Some("none")
        );
        assert_eq!(
            workflow
                .get("autoContextInjection")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            workflow
                .get("nonGitProjectsAllowed")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            project_selector
                .get("searchAvailable")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            project_selector
                .get("workflowPurposeFilterAvailable")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            project_selector
                .get("crossProjectViewsAvailable")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            session_catalog
                .get("searchAvailable")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            session_catalog
                .get("workflowPurposeGroupingAvailable")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            session_catalog
                .get("concurrentActiveSessions")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn app_status_reports_no_browser_or_web_viewing_tier() {
        let store = AppStore::open_in_memory().expect("store opens");
        let status = app_status(&store).expect("status");
        let browser_web = status.get("browserWebViewing").expect("browser web status");

        assert_eq!(
            browser_web.get("supportTier").and_then(Value::as_str),
            Some("no_browser_or_web_viewing_v1")
        );
        assert_eq!(
            browser_web
                .get("inAppBrowserPanel")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            browser_web.get("remoteUrlViewing").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            browser_web.get("domExtraction").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            browser_web.get("screenshots").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            browser_web
                .get("browserAutomation")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            browser_web
                .get("browserContentModelContextIngestion")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn app_status_reports_no_plugin_or_marketplace_tier() {
        let store = AppStore::open_in_memory().expect("store opens");
        let status = app_status(&store).expect("status");
        let plugin_marketplace = status
            .get("pluginMarketplace")
            .expect("plugin marketplace status");

        assert_eq!(
            plugin_marketplace
                .get("supportTier")
                .and_then(Value::as_str),
            Some("no_plugin_or_marketplace_v1")
        );
        assert_eq!(
            plugin_marketplace
                .get("pluginInstallAvailable")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            plugin_marketplace
                .get("pluginExecutionAvailable")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            plugin_marketplace
                .get("pluginPermissionGrants")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            plugin_marketplace
                .get("pluginProvidedMcpServers")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            plugin_marketplace
                .get("marketplaceBrowsing")
                .and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            plugin_marketplace
                .get("projectLocalPluginMetadataTrusted")
                .and_then(Value::as_bool),
            Some(false)
        );
    }
}
