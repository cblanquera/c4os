//! C4OS-owned Model Context Protocol definition, lifecycle, and projection
//! contracts.
//!
//! MCP peers are supervised, hostile workers. They may describe tools and
//! resources, but they never become C4OS policy, credential, or execution
//! authorities. Durable records contain only opaque secret references.

use std::{collections::BTreeSet, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ts_rs::TS;

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
pub const MCP_STATE_SCHEMA_VERSION: u16 = 1;
pub const MAX_MCP_SERVERS: usize = 256;
pub const MAX_MCP_ARGUMENTS: usize = 128;
pub const MAX_MCP_BINDINGS: usize = 128;
pub const MAX_MCP_TOOLS: usize = 4_096;
pub const MAX_MCP_RESOURCES: usize = 4_096;
pub const MAX_MCP_TEXT_BYTES: usize = 16_384;
pub const MAX_MCP_SCHEMA_BYTES: usize = 64 * 1_024;
pub const MAX_MCP_TURN_TOOLS: usize = 128;
pub const MAX_MCP_TURN_CATALOG_BYTES: usize = 128 * 1_024;
pub const MIN_MCP_TIMEOUT_MS: u64 = 250;
pub const MAX_MCP_TIMEOUT_MS: u64 = 300_000;
pub const MIN_MCP_OUTPUT_BYTES: u64 = 1_024;
pub const MAX_MCP_OUTPUT_BYTES: u64 = 16 * 1_024 * 1_024;

pub mod authority;
pub mod database;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) mod production_sampling;
pub mod service;
pub mod transport;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpTransportKind {
    Stdio,
    StreamableHttp,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpTrustState {
    Pending,
    Trusted,
    Revoked,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpLifecycle {
    Disabled,
    Testing,
    Connecting,
    Ready,
    Executing,
    Restarting,
    Failed,
    Revoked,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpDefinitionSource {
    User,
    Plugin {
        package_id: String,
        declaration_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpScope {
    Application,
    Workspace {
        workspace_id: String,
    },
    Project {
        workspace_id: String,
        project_id: String,
    },
    Chat {
        workspace_id: String,
        project_id: String,
        session_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpWorkingDirectory {
    C4osHome,
    ActiveProject,
    TrustedRoot { path: String },
}

/// An opaque reference to secret material. The referenced value is resolved
/// only for one operation and never serialized into an MCP definition.
#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpSecretReference {
    Vault { credential_reference: String },
    Environment { variable: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpEnvironmentSource {
    Literal { value: String },
    Passthrough,
    Secret { reference: McpSecretReference },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpEnvironmentBinding {
    pub name: String,
    pub source: McpEnvironmentSource,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpHeaderSource {
    Literal { value: String },
    Environment { variable: String },
    Secret { reference: McpSecretReference },
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpHeaderBinding {
    pub name: String,
    pub source: McpHeaderSource,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum McpTransportDefinition {
    Stdio {
        command: String,
        #[serde(default)]
        arguments: Vec<String>,
        #[serde(default)]
        environment: Vec<McpEnvironmentBinding>,
        working_directory: McpWorkingDirectory,
        executable_sha256: Option<String>,
    },
    StreamableHttp {
        url: String,
        bearer: Option<McpSecretReference>,
        #[serde(default)]
        headers: Vec<McpHeaderBinding>,
    },
}

impl McpTransportDefinition {
    pub const fn kind(&self) -> McpTransportKind {
        match self {
            Self::Stdio { .. } => McpTransportKind::Stdio,
            Self::StreamableHttp { .. } => McpTransportKind::StreamableHttp,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpServerDefinitionInput {
    pub expected_generation: u64,
    pub server_id: String,
    pub display_name: String,
    pub scope: McpScope,
    pub transport: McpTransportDefinition,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpTrustApprovalAnswer {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpTrustRequestInput {
    pub expected_generation: u64,
    pub server_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpTrustApprovalInput {
    pub expected_generation: u64,
    pub server_id: String,
    pub prompt_id: String,
    pub answer: McpTrustApprovalAnswer,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpTrustRequestStatus {
    Trusted,
    PendingApproval,
    Denied,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpTrustApprovalState {
    Pending,
    Interrupted,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpPendingTrustApproval {
    pub prompt_id: String,
    pub definition_sha256: String,
    pub action_binding_sha256: String,
    pub action_configuration_version: u64,
    pub requested_at_ms: u64,
    pub expires_at_ms: u64,
    pub state: McpTrustApprovalState,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpTrustResponse {
    pub snapshot: McpServiceSnapshot,
    pub status: McpTrustRequestStatus,
    pub server_id: String,
    pub definition_sha256: String,
    pub prompt_id: Option<String>,
    pub prompt_expires_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpServerMutationInput {
    pub expected_generation: u64,
    pub server_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpServerRevocationInput {
    pub expected_generation: u64,
    pub server_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpToolCallInput {
    pub expected_generation: u64,
    pub server_id: String,
    pub lifecycle_generation: u64,
    pub tool_name: String,
    #[ts(type = "unknown")]
    pub arguments: serde_json::Value,
    pub run_id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub turn_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpResourceReadInput {
    pub expected_generation: u64,
    pub server_id: String,
    pub lifecycle_generation: u64,
    pub uri: String,
    pub run_id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub turn_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpCapabilitySnapshot {
    pub tools: bool,
    pub tool_list_changed: bool,
    pub resources: bool,
    pub resource_list_changed: bool,
    pub resource_subscribe: bool,
    pub prompts: bool,
    pub logging: bool,
    pub completions: bool,
    pub tasks: bool,
    pub experimental_keys: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpToolSnapshot {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[ts(type = "unknown")]
    pub input_schema: serde_json::Value,
    pub input_schema_sha256: String,
    /// Retained only inside the live Rust service for hostile-result
    /// validation. It is never persisted or projected to the renderer.
    #[serde(skip, default)]
    #[ts(skip)]
    pub output_schema: Option<serde_json::Value>,
    pub output_schema_sha256: Option<String>,
}

/// Rust-owned, attempt-scoped projection of one MCP tool that was eligible at
/// turn start. It is deliberately not exported over Tauri; the renderer cannot
/// add routes or alter the authority-bearing server/lifecycle binding.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpTurnToolSnapshot {
    pub target_id: String,
    pub server_id: String,
    pub source: McpDefinitionSource,
    pub lifecycle_generation: u64,
    pub definition_sha256: String,
    pub transport_kind: McpTransportKind,
    pub tool_name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub input_schema_sha256: String,
    pub output_schema_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpTurnSnapshot {
    pub snapshot_id: String,
    pub service_generation: u64,
    pub captured_at_ms: u64,
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub tools: Vec<McpTurnToolSnapshot>,
    pub truncated: bool,
    pub omitted_tool_count: u32,
    pub sha256: String,
}

impl McpTurnSnapshot {
    pub fn validate(&self) -> Result<(), McpError> {
        validate_identifier(&self.snapshot_id)?;
        validate_identifier(&self.workspace_id)?;
        validate_identifier(&self.project_id)?;
        validate_identifier(&self.session_id)?;
        validate_digest(&self.sha256)?;
        if self.service_generation == 0
            || self.captured_at_ms == 0
            || self.tools.len() > MAX_MCP_TURN_TOOLS
            || self.truncated != (self.omitted_tool_count > 0)
        {
            return Err(McpError::BoundExceeded);
        }
        let mut routes = BTreeSet::new();
        for tool in &self.tools {
            validate_identifier(&tool.target_id)?;
            validate_identifier(&tool.server_id)?;
            validate_mcp_name(&tool.tool_name)?;
            validate_optional_text(tool.title.as_deref())?;
            validate_optional_text(tool.description.as_deref())?;
            validate_digest(&tool.definition_sha256)?;
            validate_digest(&tool.input_schema_sha256)?;
            if let Some(digest) = &tool.output_schema_sha256 {
                validate_digest(digest)?;
            }
            if tool.lifecycle_generation == 0
                || !routes.insert(tool.target_id.as_str())
                || !tool.input_schema.is_object()
            {
                return Err(McpError::InvalidState);
            }
            let schema_bytes =
                serde_json::to_vec(&tool.input_schema).map_err(|_| McpError::InvalidState)?;
            let schema_sha256 = format!("sha256:{}", hex_digest(&Sha256::digest(&schema_bytes)),);
            validate_mcp_input_schema(&tool.input_schema, None)
                .map_err(|_| McpError::InvalidState)?;
            let route_bytes = serde_json::to_vec(&serde_json::json!({
                "serverId": tool.server_id,
                "source": tool.source,
                "lifecycleGeneration": tool.lifecycle_generation,
                "definitionSha256": tool.definition_sha256,
                "toolName": tool.tool_name,
                "inputSchemaSha256": tool.input_schema_sha256,
                "outputSchemaSha256": tool.output_schema_sha256,
                "workspaceId": self.workspace_id,
                "projectId": self.project_id,
                "sessionId": self.session_id,
            }))
            .map_err(|_| McpError::InvalidState)?;
            let expected_target =
                format!("mcp-tool:{}", hex_digest(&Sha256::digest(&route_bytes)),);
            if schema_bytes.len() > MAX_MCP_SCHEMA_BYTES
                || schema_sha256 != tool.input_schema_sha256
                || expected_target != tool.target_id
            {
                return Err(McpError::InvalidState);
            }
        }
        let encoded = serde_json::to_vec(&serde_json::json!({
            "serviceGeneration": self.service_generation,
            "capturedAtMs": self.captured_at_ms,
            "workspaceId": &self.workspace_id,
            "projectId": &self.project_id,
            "sessionId": &self.session_id,
            "tools": &self.tools,
            "truncated": self.truncated,
            "omittedToolCount": self.omitted_tool_count,
        }))
        .map_err(|_| McpError::InvalidState)?;
        let expected_sha256 = format!("sha256:{}", hex_digest(&Sha256::digest(&encoded)));
        if encoded.len() > MAX_MCP_TURN_CATALOG_BYTES
            || expected_sha256 != self.sha256
            || self.snapshot_id
                != format!("mcp-turn:{}", expected_sha256.trim_start_matches("sha256:"))
        {
            return Err(McpError::InvalidState);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpResourceSnapshot {
    pub uri: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpServerSnapshot {
    pub server_id: String,
    pub display_name: String,
    pub source: McpDefinitionSource,
    pub scope: McpScope,
    pub transport: McpTransportDefinition,
    pub trust: McpTrustState,
    #[serde(default)]
    pub trusted_definition_sha256: Option<String>,
    pub pending_trust_approval: Option<McpPendingTrustApproval>,
    pub lifecycle: McpLifecycle,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub lifecycle_generation: u64,
    pub restart_attempts: u16,
    pub next_restart_at_ms: Option<u64>,
    pub protocol_version: Option<String>,
    pub server_name: Option<String>,
    pub server_version: Option<String>,
    pub instructions_present: bool,
    pub capabilities: McpCapabilitySnapshot,
    pub tools: Vec<McpToolSnapshot>,
    pub resources: Vec<McpResourceSnapshot>,
    pub active_requests: u16,
    pub last_connected_at_ms: Option<u64>,
    pub last_failure_code: Option<String>,
    pub last_failure_detail: Option<String>,
    pub last_event_id: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpServiceSnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub servers: Vec<McpServerSnapshot>,
    pub active_workers: u16,
    pub last_event_id: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum McpInvocationStatus {
    Succeeded,
    Failed,
    Denied,
    Cancelled,
    TimedOut,
    OutputLimitExceeded,
    Stale,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpInvocationSnapshot {
    pub operation_id: String,
    pub server_id: String,
    pub lifecycle_generation: u64,
    pub target: String,
    pub status: McpInvocationStatus,
    pub is_error: bool,
    #[ts(type = "unknown")]
    pub redacted_content: serde_json::Value,
    pub output_sha256: String,
    pub output_bytes: u64,
    pub truncated: bool,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
    pub audit_event_id: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct McpAuditEvent {
    pub schema_version: u16,
    pub event_id: u64,
    pub generation: u64,
    pub lifecycle_generation: u64,
    pub operation_id: String,
    pub server_id: String,
    pub event_kind: String,
    pub target: Option<String>,
    pub result: String,
    pub detail: Option<String>,
    pub occurred_at_ms: u64,
}

impl McpServiceSnapshot {
    pub fn validate(&self) -> Result<(), McpError> {
        if self.schema_version != MCP_STATE_SCHEMA_VERSION || self.generation == 0 {
            return Err(McpError::InvalidState);
        }
        if self.servers.len() > MAX_MCP_SERVERS {
            return Err(McpError::BoundExceeded);
        }
        let mut ids = BTreeSet::new();
        for server in &self.servers {
            validate_identifier(&server.server_id)?;
            validate_text(&server.display_name)?;
            validate_scope(&server.scope)?;
            validate_transport(&server.transport)?;
            validate_limits(server.timeout_ms, server.max_output_bytes)?;
            let trust_binding_is_valid =
                match (server.trust, server.trusted_definition_sha256.as_deref()) {
                    (McpTrustState::Trusted, Some(binding)) => {
                        validate_digest(binding).is_ok()
                            && server_definition_sha256(server)
                                .is_ok_and(|current| current == binding)
                    }
                    (McpTrustState::Trusted, None) => false,
                    (_, None) => true,
                    (_, Some(_)) => false,
                };
            if server.lifecycle_generation == 0
                || server.tools.len() > MAX_MCP_TOOLS
                || server.resources.len() > MAX_MCP_RESOURCES
                || !ids.insert(server.server_id.as_str())
                || server.active_requests as usize > MAX_MCP_ARGUMENTS
                || (server.trust == McpTrustState::Revoked
                    && server.lifecycle != McpLifecycle::Revoked)
                || (server.trust != McpTrustState::Pending
                    && server.pending_trust_approval.is_some())
                || !trust_binding_is_valid
                || (server.lifecycle == McpLifecycle::Ready
                    && server.protocol_version.as_deref() != Some(MCP_PROTOCOL_VERSION))
            {
                return Err(McpError::InvalidState);
            }
            if let Some(approval) = &server.pending_trust_approval
                && (validate_identifier(&approval.prompt_id).is_err()
                    || validate_digest(&approval.definition_sha256).is_err()
                    || validate_digest(&approval.action_binding_sha256).is_err()
                    || approval.action_configuration_version == 0
                    || approval.requested_at_ms == 0
                    || approval.requested_at_ms >= approval.expires_at_ms)
            {
                return Err(McpError::InvalidState);
            }
            for tool in &server.tools {
                validate_mcp_name(&tool.name)?;
                validate_optional_text(tool.title.as_deref())?;
                validate_optional_text(tool.description.as_deref())?;
                validate_digest(&tool.input_schema_sha256)?;
                let schema_bytes =
                    serde_json::to_vec(&tool.input_schema).map_err(|_| McpError::InvalidState)?;
                if schema_bytes.len() > MAX_MCP_SCHEMA_BYTES
                    || !tool.input_schema.is_object()
                    || format!("sha256:{}", hex_digest(&Sha256::digest(schema_bytes)))
                        != tool.input_schema_sha256
                {
                    return Err(McpError::InvalidState);
                }
                validate_mcp_input_schema(&tool.input_schema, None)
                    .map_err(|_| McpError::InvalidState)?;
                if let Some(digest) = tool.output_schema_sha256.as_deref() {
                    validate_digest(digest)?;
                }
                match (&tool.output_schema, &tool.output_schema_sha256) {
                    (Some(schema), Some(digest)) => {
                        let encoded =
                            serde_json::to_vec(schema).map_err(|_| McpError::InvalidState)?;
                        if encoded.len() > MAX_MCP_SCHEMA_BYTES
                            || !schema.is_object()
                            || format!("sha256:{}", hex_digest(&Sha256::digest(encoded))) != *digest
                        {
                            return Err(McpError::InvalidState);
                        }
                        validate_mcp_input_schema(schema, None)
                            .map_err(|_| McpError::InvalidState)?;
                    }
                    (None, None | Some(_)) => {}
                    (Some(_), None) => return Err(McpError::InvalidState),
                }
            }
            for resource in &server.resources {
                validate_text(&resource.uri)?;
                validate_text(&resource.name)?;
                validate_optional_text(resource.title.as_deref())?;
                validate_optional_text(resource.description.as_deref())?;
                validate_optional_text(resource.mime_type.as_deref())?;
            }
        }
        Ok(())
    }
}

pub(crate) fn server_definition_sha256(server: &McpServerSnapshot) -> Result<String, McpError> {
    let value = serde_json::json!({
        "serverId": server.server_id,
        "source": server.source,
        "scope": server.scope,
        "transport": server.transport,
        "timeoutMs": server.timeout_ms,
        "maxOutputBytes": server.max_output_bytes,
    });
    let bytes = serde_json::to_vec(&value).map_err(|_| McpError::InvalidState)?;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    Ok(format!("sha256:{output}"))
}

pub(crate) fn validate_definition_input(input: &McpServerDefinitionInput) -> Result<(), McpError> {
    validate_identifier(&input.server_id)?;
    validate_text(&input.display_name)?;
    validate_scope(&input.scope)?;
    validate_transport(&input.transport)?;
    validate_limits(input.timeout_ms, input.max_output_bytes)
}

pub(crate) fn validate_identifier(value: &str) -> Result<(), McpError> {
    if value.is_empty()
        || value.len() > 255
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
    {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

/// One stable Action Gateway identity for every authority-bearing operation
/// owned by an exact MCP definition source. Direct calls, runtime-broker calls,
/// disablement, and revocation must all use this same value.
pub(crate) fn mcp_authority_identity(
    server_id: &str,
    source: &McpDefinitionSource,
) -> Result<String, McpError> {
    validate_identifier(server_id)?;
    let encoded = serde_json::to_vec(&serde_json::json!({
        "serverId": server_id,
        "source": source,
    }))
    .map_err(|_| McpError::InvalidState)?;
    Ok(format!(
        "mcp:{}",
        hex_digest(&Sha256::digest(encoded.as_slice()))
    ))
}

pub(crate) fn validate_text(value: &str) -> Result<(), McpError> {
    if value.is_empty()
        || value.len() > MAX_MCP_TEXT_BYTES
        || value
            .chars()
            .any(|character| character.is_control() && character != '\n')
    {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_mcp_name(value: &str) -> Result<(), McpError> {
    if value.is_empty() || value.len() > 255 || value.chars().any(char::is_control) {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>) -> Result<(), McpError> {
    match value {
        Some(value) => validate_text(value),
        None => Ok(()),
    }
}

fn validate_limits(timeout_ms: u64, max_output_bytes: u64) -> Result<(), McpError> {
    if !(MIN_MCP_TIMEOUT_MS..=MAX_MCP_TIMEOUT_MS).contains(&timeout_ms)
        || !(MIN_MCP_OUTPUT_BYTES..=MAX_MCP_OUTPUT_BYTES).contains(&max_output_bytes)
    {
        return Err(McpError::BoundExceeded);
    }
    Ok(())
}

fn validate_scope(scope: &McpScope) -> Result<(), McpError> {
    match scope {
        McpScope::Application => Ok(()),
        McpScope::Workspace { workspace_id } => validate_identifier(workspace_id),
        McpScope::Project {
            workspace_id,
            project_id,
        } => {
            validate_identifier(workspace_id)?;
            validate_identifier(project_id)
        }
        McpScope::Chat {
            workspace_id,
            project_id,
            session_id,
        } => {
            validate_identifier(workspace_id)?;
            validate_identifier(project_id)?;
            validate_identifier(session_id)
        }
    }
}

fn validate_transport(transport: &McpTransportDefinition) -> Result<(), McpError> {
    match transport {
        McpTransportDefinition::Stdio {
            command,
            arguments,
            environment,
            working_directory,
            executable_sha256,
        } => {
            validate_text(command)?;
            if !Path::new(command).is_absolute() {
                return Err(McpError::InvalidInput);
            }
            if arguments.len() > MAX_MCP_ARGUMENTS || environment.len() > MAX_MCP_BINDINGS {
                return Err(McpError::BoundExceeded);
            }
            for argument in arguments {
                validate_text(argument)?;
            }
            let mut names = BTreeSet::new();
            for binding in environment {
                validate_environment_name(&binding.name)?;
                if !names.insert(binding.name.as_str()) {
                    return Err(McpError::InvalidInput);
                }
                match &binding.source {
                    McpEnvironmentSource::Literal { value } => validate_text(value)?,
                    McpEnvironmentSource::Passthrough => {}
                    McpEnvironmentSource::Secret { reference } => {
                        validate_secret_reference(reference)?
                    }
                }
            }
            match working_directory {
                McpWorkingDirectory::C4osHome | McpWorkingDirectory::ActiveProject => {}
                McpWorkingDirectory::TrustedRoot { path } => validate_text(path)?,
            }
            validate_digest(executable_sha256.as_deref().ok_or(McpError::InvalidInput)?)?;
        }
        McpTransportDefinition::StreamableHttp {
            url,
            bearer,
            headers,
        } => {
            validate_text(url)?;
            if headers.len() > MAX_MCP_BINDINGS {
                return Err(McpError::BoundExceeded);
            }
            validate_secret_reference(bearer.as_ref().ok_or(McpError::InvalidInput)?)?;
            let mut names = BTreeSet::new();
            for header in headers {
                validate_header_name(&header.name)?;
                let normalized = header.name.to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "authorization"
                        | "cookie"
                        | "host"
                        | "accept"
                        | "content-type"
                        | "content-length"
                        | "mcp-session-id"
                        | "mcp-protocol-version"
                        | "last-event-id"
                ) || !names.insert(normalized)
                {
                    return Err(McpError::InvalidInput);
                }
                match &header.source {
                    McpHeaderSource::Literal { value } => validate_text(value)?,
                    McpHeaderSource::Environment { variable } => {
                        validate_environment_name(variable)?
                    }
                    McpHeaderSource::Secret { reference } => validate_secret_reference(reference)?,
                }
            }
        }
    }
    Ok(())
}

fn validate_secret_reference(reference: &McpSecretReference) -> Result<(), McpError> {
    match reference {
        McpSecretReference::Vault {
            credential_reference,
        } => validate_identifier(credential_reference),
        McpSecretReference::Environment { variable } => validate_environment_name(variable),
    }
}

fn validate_environment_name(value: &str) -> Result<(), McpError> {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return Err(McpError::InvalidInput);
    };
    let normalized = value.to_ascii_uppercase();
    if matches!(normalized.as_str(), "HOME" | "PATH" | "SHELL" | "TMPDIR")
        || normalized.starts_with("DYLD_")
        || normalized.starts_with("LD_")
        || !(first.is_ascii_alphabetic() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || value.len() > 255
    {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

fn validate_header_name(value: &str) -> Result<(), McpError> {
    if value.is_empty()
        || value.len() > 255
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
    {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), McpError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(McpError::InvalidInput);
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

/// Compiles one bounded MCP input schema without external retrieval and, when
/// supplied, validates the exact model arguments against Draft 2020-12 before
/// any policy proposal or transport effect can occur.
pub(crate) fn validate_mcp_input_schema(
    schema: &serde_json::Value,
    instance: Option<&serde_json::Value>,
) -> Result<(), McpError> {
    let encoded = serde_json::to_vec(schema).map_err(|_| McpError::InvalidInput)?;
    if encoded.len() > MAX_MCP_SCHEMA_BYTES || !schema.is_object() {
        return Err(McpError::InvalidInput);
    }
    let validator = jsonschema::draft202012::new(schema).map_err(|_| McpError::InvalidInput)?;
    if instance.is_some_and(|instance| !validator.is_valid(instance)) {
        return Err(McpError::Denied);
    }
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[derive(Debug, Error)]
pub enum McpError {
    #[error("MCP input is invalid")]
    InvalidInput,
    #[error("MCP durable state is invalid")]
    InvalidState,
    #[error("MCP input or output exceeded a bounded limit")]
    BoundExceeded,
    #[error("MCP generation is stale")]
    Conflict,
    #[error("MCP server is not trusted")]
    Untrusted,
    #[error("MCP server or authority has been revoked")]
    Revoked,
    #[error("MCP server is disabled")]
    Disabled,
    #[error("MCP server is not ready")]
    NotReady,
    #[error("MCP selected an unsupported protocol version")]
    UnsupportedProtocol,
    #[error("MCP transport failed: {0}")]
    Transport(String),
    #[error("MCP request timed out")]
    TimedOut,
    #[error("MCP request was cancelled")]
    Cancelled,
    #[error("MCP policy denied the operation")]
    Denied,
    #[error("MCP persistence failed: {0}")]
    Persistence(String),
    #[error("MCP credential delivery failed")]
    Credential,
    #[error("MCP service state is unavailable")]
    StateUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sha256_value(value: &serde_json::Value) -> String {
        let encoded = serde_json::to_vec(value).expect("test JSON");
        format!("sha256:{}", hex_digest(&Sha256::digest(encoded)))
    }

    fn turn_snapshot() -> McpTurnSnapshot {
        let input_schema = json!({
            "type": "object",
            "properties": { "value": { "type": "string" } },
            "required": ["value"],
            "additionalProperties": false,
        });
        let input_schema_sha256 = sha256_value(&input_schema);
        let definition_sha256 = format!("sha256:{}", "a".repeat(64));
        let route = json!({
            "serverId": "fixture",
            "source": McpDefinitionSource::User,
            "lifecycleGeneration": 3,
            "definitionSha256": definition_sha256,
            "toolName": "echo",
            "inputSchemaSha256": input_schema_sha256,
            "outputSchemaSha256": null,
            "workspaceId": "workspace-1",
            "projectId": "project-1",
            "sessionId": "session-1",
        });
        let target_id = format!(
            "mcp-tool:{}",
            sha256_value(&route).trim_start_matches("sha256:")
        );
        let tool = McpTurnToolSnapshot {
            target_id,
            server_id: "fixture".into(),
            source: McpDefinitionSource::User,
            lifecycle_generation: 3,
            definition_sha256,
            transport_kind: McpTransportKind::Stdio,
            tool_name: "echo".into(),
            title: Some("Echo".into()),
            description: None,
            input_schema,
            input_schema_sha256,
            output_schema_sha256: None,
        };
        let catalog = json!({
            "serviceGeneration": 7,
            "capturedAtMs": 10,
            "workspaceId": "workspace-1",
            "projectId": "project-1",
            "sessionId": "session-1",
            "tools": [&tool],
            "truncated": false,
            "omittedToolCount": 0,
        });
        let sha256 = sha256_value(&catalog);
        McpTurnSnapshot {
            snapshot_id: format!("mcp-turn:{}", sha256.trim_start_matches("sha256:")),
            service_generation: 7,
            captured_at_ms: 10,
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            tools: vec![tool],
            truncated: false,
            omitted_tool_count: 0,
            sha256,
        }
    }

    #[test]
    fn turn_snapshot_recomputes_schema_route_and_catalog_bindings() {
        let snapshot = turn_snapshot();
        snapshot.validate().expect("exact snapshot");

        let mut target_tampered = snapshot.clone();
        target_tampered.tools[0].target_id = "mcp-tool:substituted".into();
        assert!(matches!(
            target_tampered.validate(),
            Err(McpError::InvalidState)
        ));

        let mut schema_tampered = snapshot.clone();
        schema_tampered.tools[0].input_schema = json!({ "type": "object" });
        assert!(matches!(
            schema_tampered.validate(),
            Err(McpError::InvalidState)
        ));
    }

    #[test]
    fn draft_2020_12_arguments_are_checked_before_effect_dispatch() {
        let snapshot = turn_snapshot();
        let schema = &snapshot.tools[0].input_schema;
        assert!(validate_mcp_input_schema(schema, Some(&json!({ "value": "ok" }))).is_ok());
        assert!(matches!(
            validate_mcp_input_schema(schema, Some(&json!({ "value": 7 }))),
            Err(McpError::Denied)
        ));
        assert!(matches!(
            validate_mcp_input_schema(&json!({ "type": "not-a-json-schema-type" }), None),
            Err(McpError::InvalidInput)
        ));
    }

    #[test]
    fn stdio_definition_rejects_relative_executables_and_reserved_environment_names() {
        let input =
            |command: &str, environment: Vec<McpEnvironmentBinding>| McpServerDefinitionInput {
                expected_generation: 1,
                server_id: "fixture".into(),
                display_name: "Fixture".into(),
                scope: McpScope::Application,
                transport: McpTransportDefinition::Stdio {
                    command: command.into(),
                    arguments: Vec::new(),
                    environment,
                    working_directory: McpWorkingDirectory::C4osHome,
                    executable_sha256: Some(format!("sha256:{}", "0".repeat(64))),
                },
                timeout_ms: 1_000,
                max_output_bytes: 1_024,
            };
        assert!(matches!(
            validate_definition_input(&input("fixture-mcp", Vec::new())),
            Err(McpError::InvalidInput)
        ));
        assert!(matches!(
            validate_definition_input(&input(
                "/usr/bin/false",
                vec![McpEnvironmentBinding {
                    name: "PATH".into(),
                    source: McpEnvironmentSource::Passthrough,
                }]
            )),
            Err(McpError::InvalidInput)
        ));
    }

    #[test]
    fn http_definition_rejects_transport_owned_headers() {
        for name in [
            "Authorization",
            "Accept",
            "Content-Type",
            "Content-Length",
            "MCP-Session-Id",
            "MCP-Protocol-Version",
            "Last-Event-ID",
        ] {
            let input = McpServerDefinitionInput {
                expected_generation: 1,
                server_id: "fixture".into(),
                display_name: "Fixture".into(),
                scope: McpScope::Application,
                transport: McpTransportDefinition::StreamableHttp {
                    url: "https://mcp.example.test".into(),
                    bearer: Some(McpSecretReference::Environment {
                        variable: "MCP_TOKEN".into(),
                    }),
                    headers: vec![McpHeaderBinding {
                        name: name.into(),
                        source: McpHeaderSource::Literal { value: "x".into() },
                    }],
                },
                timeout_ms: 1_000,
                max_output_bytes: 1_024,
            };
            assert!(
                matches!(
                    validate_definition_input(&input),
                    Err(McpError::InvalidInput)
                ),
                "accepted transport-owned header {name}"
            );
        }
    }
}
