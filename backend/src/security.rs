use crate::records::AuditRecord;
use crate::tool_gateway::{
    tool_definition, SessionToolConfig, ToolAccess, ToolApproval, ToolDefinition, ToolPermission,
    TOOL_ARTIFACT_PREVIEW, TOOL_BROWSER_OPEN, TOOL_FILES_READ, TOOL_FILES_WRITE, TOOL_TERMINAL_RUN,
};
use crate::workspace::active_workspace_descriptor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceToolPolicy {
    pub workspace_id: String,
    pub trusted_root: String,
    pub tools: BTreeMap<String, ToolPermission>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolPolicyOverride {
    pub session_id: String,
    pub tools: BTreeMap<String, ToolPermission>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveToolPolicySnapshot {
    pub id: String,
    pub workspace_id: String,
    pub trusted_root: String,
    pub session_id: Option<String>,
    pub run_id: Option<String>,
    pub tool: String,
    pub enabled: bool,
    pub access: ToolAccess,
    pub approval: ToolApproval,
    pub source: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecureKeyStoragePolicy {
    pub provider_keys: String,
    pub raw_keys_in_workspace_files: bool,
    pub raw_keys_in_app_records: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApprovalPolicyStore {
    pub schema_version: u32,
    pub workspace: WorkspaceToolPolicy,
    #[serde(default)]
    pub session_overrides: Vec<SessionToolPolicyOverride>,
    #[serde(default)]
    pub snapshots: Vec<EffectiveToolPolicySnapshot>,
    pub key_storage: SecureKeyStoragePolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolEnforcementInput<'a> {
    pub definition: &'a ToolDefinition,
    pub session_id: Option<&'a str>,
    pub run_id: Option<&'a str>,
    pub actor: Option<&'a str>,
    pub args: &'a Value,
    pub session_config: Option<&'a SessionToolConfig>,
}

static POLICY_STORE: OnceLock<Mutex<Option<ToolApprovalPolicyStore>>> = OnceLock::new();

const STABLE_TOOL_IDENTITIES: [&str; 5] = [
    "terminal.run",
    "files.read",
    "files.write",
    "browser.open",
    "artifact.preview",
];

pub fn load_tool_approval_policy() -> ToolApprovalPolicyStore {
    if let Some(saved) = policy_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
    {
        return saved;
    }
    let store = read_policy_store_from_disk().unwrap_or_else(default_policy_store);
    *policy_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(store.clone());
    store
}

pub fn save_tool_approval_policy(
    policy: ToolApprovalPolicyStore,
) -> Result<ToolApprovalPolicyStore, String> {
    validate_workspace_policy(&policy.workspace)?;
    write_policy_store_to_disk(&policy)?;
    *policy_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(policy.clone());
    Ok(policy)
}

pub fn effective_tool_policy_snapshot(
    definition: &ToolDefinition,
    session_id: Option<&str>,
    run_id: Option<&str>,
    session_config: Option<&SessionToolConfig>,
) -> Result<EffectiveToolPolicySnapshot, String> {
    let store = load_tool_approval_policy();
    let workspace_permission = store
        .workspace
        .tools
        .get(&definition.tool)
        .cloned()
        .unwrap_or_else(|| ToolPermission {
            enabled: true,
            access: definition.access.clone(),
            approval: definition.default_approval.clone(),
        });
    validate_permission(definition, &workspace_permission)?;

    let override_permission = session_config
        .and_then(|config| config.tools.get(&definition.tool))
        .cloned()
        .or_else(|| {
            session_id.and_then(|id| {
                store
                    .session_overrides
                    .iter()
                    .find(|override_record| override_record.session_id == id)
                    .and_then(|override_record| override_record.tools.get(&definition.tool))
                    .cloned()
            })
        });
    let source = if override_permission.is_some() {
        "session-override"
    } else {
        "workspace-default"
    };
    let permission = override_permission.unwrap_or(workspace_permission.clone());
    validate_permission(definition, &permission)?;
    if approval_rank(&permission.approval) > approval_rank(&workspace_permission.approval) {
        return Err(format!(
            "C4OS tool '{}' requested wider approval than workspace policy",
            definition.tool
        ));
    }

    Ok(EffectiveToolPolicySnapshot {
        id: format!("policy-snapshot-{}", now_unix_seconds()),
        workspace_id: store.workspace.workspace_id,
        trusted_root: store.workspace.trusted_root,
        session_id: session_id.map(str::to_string),
        run_id: run_id.map(str::to_string),
        tool: definition.tool.clone(),
        enabled: permission.enabled,
        access: permission.access,
        approval: permission.approval,
        source: source.into(),
        created_at: now_unix_seconds(),
    })
}

pub fn enforce_tool_request(
    input: ToolEnforcementInput<'_>,
) -> Result<EffectiveToolPolicySnapshot, String> {
    let snapshot = effective_tool_policy_snapshot(
        input.definition,
        input.session_id,
        input.run_id,
        input.session_config,
    )?;
    let result = enforce_snapshot(&snapshot, input.actor, input.args);
    persist_effective_tool_policy_snapshot(snapshot.clone());
    match result {
        Ok(()) => {
            record_security_audit(
                input.session_id,
                "tool-approved",
                &format!("Tool '{}' allowed by effective policy", snapshot.tool),
                &snapshot.tool,
                "allowed",
            );
            Ok(snapshot)
        }
        Err(error) => {
            record_security_audit(
                input.session_id,
                "tool-rejected",
                &error,
                &snapshot.tool,
                "rejected",
            );
            Err(error)
        }
    }
}

pub fn persist_effective_tool_policy_snapshot(snapshot: EffectiveToolPolicySnapshot) {
    let mut store = load_tool_approval_policy();
    store.snapshots.push(snapshot);
    let _ = save_tool_approval_policy(store);
}

pub fn record_security_audit(
    session_id: Option<&str>,
    category: &str,
    summary: &str,
    resource: &str,
    status: &str,
) -> AuditRecord {
    crate::records::record_security_audit(
        &load_tool_approval_policy().workspace.workspace_id,
        session_id,
        category,
        summary,
        resource,
        status,
    )
}

pub fn secure_key_storage_policy() -> SecureKeyStoragePolicy {
    SecureKeyStoragePolicy {
        provider_keys: "os-keychain-or-explicit-secure-store-reference".into(),
        raw_keys_in_workspace_files: false,
        raw_keys_in_app_records: false,
    }
}

pub fn extension_runtime_enabled(enabled: bool, runtime_access: &str) -> Result<(), String> {
    if enabled || runtime_access != "disabled" {
        return Err("Extension runtime impact requires explicit enablement gates".into());
    }
    Ok(())
}

fn enforce_snapshot(
    snapshot: &EffectiveToolPolicySnapshot,
    actor: Option<&str>,
    args: &Value,
) -> Result<(), String> {
    if !snapshot.enabled {
        return Err(format!("C4OS tool '{}' is disabled", snapshot.tool));
    }
    if snapshot.approval == ToolApproval::Deny {
        return Err(format!("C4OS tool '{}' is denied", snapshot.tool));
    }
    if approval_required(&snapshot.approval) && !bool_arg(args, "approved") {
        return Err(format!(
            "C4OS tool '{}' requires explicit approval before execution",
            snapshot.tool
        ));
    }
    if snapshot.tool == TOOL_TERMINAL_RUN {
        enforce_terminal_command_policy(args)?;
    }
    if snapshot.tool == TOOL_BROWSER_OPEN
        && normalized_actor(actor) == "agent"
        && !bool_arg(args, "clearRequest")
    {
        return Err("Agent Browser access must be clearly requested for this request".into());
    }
    Ok(())
}

fn enforce_terminal_command_policy(args: &Value) -> Result<(), String> {
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    if command.is_empty() {
        return Err("Terminal command is required".into());
    }
    if bool_arg(args, "approved") {
        return Ok(());
    }
    if readonly_terminal_command(command) {
        return Ok(());
    }
    Err("Terminal command requires approval by command policy".into())
}

fn readonly_terminal_command(command: &str) -> bool {
    let disallowed = ["&&", "||", ";", "|", ">", "<", "$(", "`"];
    if disallowed.iter().any(|marker| command.contains(marker)) {
        return false;
    }
    let mut parts = command.split_whitespace();
    matches!(parts.next(), Some("pwd" | "ls"))
        || matches!((parts.next(), parts.next()), (Some("git"), Some("status")))
}

fn default_policy_store() -> ToolApprovalPolicyStore {
    let descriptor = active_workspace_descriptor();
    let workspace_id = descriptor
        .as_ref()
        .map(|descriptor| descriptor.id.clone())
        .unwrap_or_else(|| "workspace:untrusted".into());
    let trusted_root = descriptor
        .as_ref()
        .map(|descriptor| descriptor.root_path.clone())
        .unwrap_or_default();
    ToolApprovalPolicyStore {
        schema_version: 1,
        workspace: WorkspaceToolPolicy {
            workspace_id,
            trusted_root,
            tools: default_tool_permissions(),
        },
        session_overrides: Vec::new(),
        snapshots: Vec::new(),
        key_storage: secure_key_storage_policy(),
    }
}

fn default_tool_permissions() -> BTreeMap<String, ToolPermission> {
    let _stable_tool_identities = STABLE_TOOL_IDENTITIES;
    [
        (
            TOOL_TERMINAL_RUN,
            ToolAccess::WorkspaceShell,
            ToolApproval::AskEveryTime,
        ),
        (
            TOOL_FILES_READ,
            ToolAccess::WorkspaceRead,
            ToolApproval::AutoReadonly,
        ),
        (
            TOOL_FILES_WRITE,
            ToolAccess::WorkspaceWrite,
            ToolApproval::AskEveryTime,
        ),
        (
            TOOL_BROWSER_OPEN,
            ToolAccess::PublicOrTrustedLocal,
            ToolApproval::AutoReadonly,
        ),
        (
            TOOL_ARTIFACT_PREVIEW,
            ToolAccess::ArtifactPreview,
            ToolApproval::AutoReadonly,
        ),
    ]
    .into_iter()
    .map(|(tool, access, approval)| {
        (
            tool.into(),
            ToolPermission {
                enabled: true,
                access,
                approval,
            },
        )
    })
    .collect()
}

fn validate_workspace_policy(workspace: &WorkspaceToolPolicy) -> Result<(), String> {
    for (tool, permission) in &workspace.tools {
        let definition = tool_definition(tool)?;
        validate_permission(&definition, permission)?;
    }
    Ok(())
}

fn validate_permission(
    definition: &ToolDefinition,
    permission: &ToolPermission,
) -> Result<(), String> {
    if permission.access != definition.access {
        return Err(format!(
            "C4OS tool '{}' requested incompatible access",
            definition.tool
        ));
    }
    if approval_rank(&permission.approval) > approval_rank(&definition.max_approval) {
        return Err(format!(
            "C4OS tool '{}' requested wider approval than allowed",
            definition.tool
        ));
    }
    Ok(())
}

fn read_policy_store_from_disk() -> Option<ToolApprovalPolicyStore> {
    let raw = fs::read_to_string(policy_store_path()?).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_policy_store_to_disk(policy: &ToolApprovalPolicyStore) -> Result<(), String> {
    let Some(path) = policy_store_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Cannot create policy store folder: {error}"))?;
    }
    let serialized = serde_json::to_string_pretty(policy)
        .map_err(|error| format!("Cannot serialize tool policy store: {error}"))?;
    fs::write(path, format!("{serialized}\n"))
        .map_err(|error| format!("Cannot write tool policy store: {error}"))
}

fn policy_store_path() -> Option<PathBuf> {
    match std::env::var("C4OS_TOOL_POLICY_STORE") {
        Ok(path) if !path.trim().is_empty() => Some(PathBuf::from(path)),
        #[cfg(not(test))]
        _ => Some(std::env::temp_dir().join("c4os-tool-policy.json")),
        #[cfg(test)]
        _ => None,
    }
}

fn policy_store() -> &'static Mutex<Option<ToolApprovalPolicyStore>> {
    POLICY_STORE.get_or_init(|| Mutex::new(None))
}

fn bool_arg(args: &Value, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn approval_required(approval: &ToolApproval) -> bool {
    matches!(approval, ToolApproval::AskEveryTime)
}

fn normalized_actor(actor: Option<&str>) -> String {
    match actor.unwrap_or("user").trim().to_ascii_lowercase().as_str() {
        "agent" => "agent".into(),
        _ => "user".into(),
    }
}

fn approval_rank(approval: &ToolApproval) -> u8 {
    match approval {
        ToolApproval::Deny => 0,
        ToolApproval::AskEveryTime => 1,
        ToolApproval::AutoReadonly => 2,
        ToolApproval::AutoWorkspace => 3,
        ToolApproval::AutoAlways => 4,
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn reset_security_policy_for_test(name: &str) {
    let path = std::env::temp_dir().join(format!("c4os-{name}-tool-policy.json"));
    std::env::set_var("C4OS_TOOL_POLICY_STORE", path);
    *policy_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = None;
    let _ = fs::remove_file(policy_store_path().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_016_policy_store_persists_workspace_defaults_and_key_policy() {
        reset_security_policy_for_test("task-016-policy-store");

        let mut policy = load_tool_approval_policy();
        policy.workspace.workspace_id = "workspace-alpha".into();
        save_tool_approval_policy(policy).expect("save policy");
        *policy_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;

        let loaded = load_tool_approval_policy();
        assert_eq!(loaded.workspace.workspace_id, "workspace-alpha");
        assert!(loaded.workspace.tools.contains_key(TOOL_TERMINAL_RUN));
        assert!(loaded.workspace.tools.contains_key(TOOL_FILES_READ));
        assert!(loaded.workspace.tools.contains_key(TOOL_FILES_WRITE));
        assert!(loaded.workspace.tools.contains_key(TOOL_BROWSER_OPEN));
        assert!(loaded.workspace.tools.contains_key(TOOL_ARTIFACT_PREVIEW));
        assert_eq!(
            loaded.key_storage,
            SecureKeyStoragePolicy {
                provider_keys: "os-keychain-or-explicit-secure-store-reference".into(),
                raw_keys_in_workspace_files: false,
                raw_keys_in_app_records: false,
            }
        );
    }

    #[test]
    fn task_016_effective_snapshot_rejects_session_policy_widening() {
        reset_security_policy_for_test("task-016-widening");
        let definition = tool_definition(TOOL_TERMINAL_RUN).expect("definition");
        let mut tools = BTreeMap::new();
        tools.insert(
            TOOL_TERMINAL_RUN.into(),
            ToolPermission {
                enabled: true,
                access: ToolAccess::WorkspaceShell,
                approval: ToolApproval::AutoAlways,
            },
        );
        let config = SessionToolConfig { tools };

        let error = effective_tool_policy_snapshot(
            &definition,
            Some("session-one"),
            Some("run-one"),
            Some(&config),
        )
        .expect_err("widening rejected");

        assert!(error.contains("wider approval"));
    }

    #[test]
    fn task_016_unapproved_terminal_request_records_security_audit() {
        reset_security_policy_for_test("task-016-terminal-audit");
        crate::records::reset_records_store_for_test("task-016-terminal-audit");
        let definition = tool_definition(TOOL_TERMINAL_RUN).expect("definition");

        let error = enforce_tool_request(ToolEnforcementInput {
            definition: &definition,
            session_id: Some("session-one"),
            run_id: Some("run-one"),
            actor: Some("agent"),
            args: &serde_json::json!({ "command": "npm install" }),
            session_config: None,
        })
        .expect_err("approval required");

        assert!(error.contains("requires explicit approval"));
        let audits = crate::records::list_audit_records();
        assert!(audits
            .iter()
            .any(|record| record.category == "tool-rejected"
                && record.resource == TOOL_TERMINAL_RUN
                && record.status == "rejected"));
    }

    #[test]
    fn task_016_extension_runtime_impact_requires_enablement_gate() {
        assert!(extension_runtime_enabled(false, "disabled").is_ok());
        assert!(extension_runtime_enabled(true, "disabled").is_err());
        assert!(extension_runtime_enabled(false, "runtime").is_err());
    }
}
