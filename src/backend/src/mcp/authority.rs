//! Production MCP composition for workspace scope, credentials, policy, and
//! the durable two-phase Action Gateway barrier.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    RuntimeApplicationService,
    core::{
        database::{LifecycleState, MAX_READ_RECORDS, ProjectPathState, SnapshotQuery},
        services::ActiveWorkspace,
    },
    security::{
        authorization::{
            CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk, LiveAuthorityState,
        },
        credentials::{CredentialReference, CredentialVault},
        gateway::{
            ActionEffectLease, GatewayProposal, NormalizedActionResult, NormalizedActionStatus,
        },
        policy::{
            ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
            ActionScope, ActionSensitivity, ActionSurface, ClassificationConfidence,
            RepositoryState,
        },
    },
};

use super::{
    MAX_MCP_TEXT_BYTES, McpDefinitionSource, McpEnvironmentSource, McpError, McpHeaderSource,
    McpScope, McpSecretReference, McpServerSnapshot, McpTransportDefinition, McpTrustState,
    mcp_authority_identity, server_definition_sha256,
    service::{McpAuthority, McpAuthorityEffectStatus, McpAuthorityRequest},
    transport::{McpSamplingContext, ResolvedMcpLaunch},
};

const MAX_REDACTION_DEPTH: usize = 32;
const MAX_REDACTION_NODES: usize = 16_384;

pub struct ProductionMcpEffectLease {
    gateway: ActionEffectLease,
}

pub struct ProductionMcpAuthority {
    c4os_home: PathBuf,
    credential_vault: Option<CredentialVault>,
    active_workspace: Arc<Mutex<Option<ActiveWorkspace>>>,
    runtime: Arc<RuntimeApplicationService>,
}

impl ProductionMcpAuthority {
    pub fn new(
        c4os_home: PathBuf,
        credential_vault: Option<CredentialVault>,
        active_workspace: Arc<Mutex<Option<ActiveWorkspace>>>,
        runtime: Arc<RuntimeApplicationService>,
    ) -> Result<Self, McpError> {
        let c4os_home = canonical_directory(&c4os_home)?;
        Ok(Self {
            c4os_home,
            credential_vault,
            active_workspace,
            runtime,
        })
    }

    fn resolve_secret(
        &self,
        reference: &McpSecretReference,
        operation: &str,
        timeout_ms: u64,
    ) -> Result<String, McpError> {
        match reference {
            McpSecretReference::Environment { variable } => {
                let value = std::env::var(variable).map_err(|_| McpError::Credential)?;
                if value.is_empty() || value.contains('\0') {
                    return Err(McpError::Credential);
                }
                Ok(value)
            }
            McpSecretReference::Vault {
                credential_reference,
            } => {
                let vault = self.credential_vault.as_ref().ok_or(McpError::Credential)?;
                let reference: CredentialReference =
                    serde_json::from_value(Value::String(credential_reference.clone()))
                        .map_err(|_| McpError::Credential)?;
                let lease = vault
                    .lease_for_operation(
                        &reference,
                        operation,
                        Duration::from_millis(timeout_ms.clamp(1, 300_000)),
                    )
                    .map_err(|_| McpError::Credential)?;
                let mut bytes = Zeroizing::new(Vec::new());
                lease
                    .deliver_to(&mut *bytes)
                    .map_err(|_| McpError::Credential)?;
                let value = String::from_utf8(std::mem::take(&mut *bytes))
                    .map_err(|_| McpError::Credential)?;
                if value.is_empty() || value.contains('\0') {
                    return Err(McpError::Credential);
                }
                Ok(value)
            }
        }
    }

    fn workspace_launch_paths(
        &self,
        server: &McpServerSnapshot,
    ) -> Result<(Option<PathBuf>, Vec<PathBuf>), McpError> {
        // Application-scoped MCP authority never implies ambient access to an
        // active Workspace or any of its Projects.
        if matches!(server.scope, McpScope::Application) {
            return Ok((None, Vec::new()));
        }
        let workspace = self
            .active_workspace
            .lock()
            .map_err(|_| McpError::StateUnavailable)?;
        let workspace = workspace.as_ref().ok_or(McpError::Denied)?;
        let workspace_id = workspace.manifest().workspace_id.to_string();
        let snapshot = workspace
            .snapshot(SnapshotQuery::new(MAX_READ_RECORDS).map_err(|_| McpError::StateUnavailable)?)
            .map_err(|_| McpError::StateUnavailable)?;

        let scoped_project_id = match &server.scope {
            McpScope::Workspace {
                workspace_id: expected,
            } if expected == &workspace_id => None,
            McpScope::Project {
                workspace_id: expected,
                project_id,
            } if expected == &workspace_id => Some(project_id.as_str()),
            McpScope::Chat {
                workspace_id: expected,
                project_id,
                session_id,
            } if expected == &workspace_id
                && snapshot.chats.iter().any(|chat| {
                    chat.chat_id == *session_id
                        && chat.project_id == *project_id
                        && chat.lifecycle_state == LifecycleState::Active
                }) =>
            {
                Some(project_id.as_str())
            }
            McpScope::Application => unreachable!("application scope returned above"),
            _ => return Err(McpError::Denied),
        };

        let active_project = scoped_project_id
            .map(|project_id| {
                snapshot
                    .projects
                    .iter()
                    .find(|project| {
                        project.project_id == project_id
                            && project.lifecycle_state == LifecycleState::Active
                            && matches!(
                                project.path_state,
                                ProjectPathState::Found | ProjectPathState::Relocated
                            )
                    })
                    .ok_or(McpError::Denied)
                    .and_then(|project| {
                        let project_id =
                            Uuid::parse_str(&project.project_id).map_err(|_| McpError::Denied)?;
                        if !workspace.is_project_trusted(project_id) {
                            return Err(McpError::Denied);
                        }
                        canonical_directory(Path::new(&project.current_path))
                    })
            })
            .transpose()?;

        let mut trusted_roots = if let Some(active_project) = active_project.as_ref() {
            vec![active_project.clone()]
        } else {
            let mut roots = Vec::new();
            for project in snapshot.projects.iter().filter(|project| {
                project.lifecycle_state == LifecycleState::Active
                    && matches!(
                        project.path_state,
                        ProjectPathState::Found | ProjectPathState::Relocated
                    )
            }) {
                let Ok(project_id) = Uuid::parse_str(&project.project_id) else {
                    continue;
                };
                if workspace.is_project_trusted(project_id) {
                    roots.push(canonical_directory(Path::new(&project.current_path))?);
                }
            }
            roots
        };
        trusted_roots.sort();
        trusted_roots.dedup();
        Ok((active_project, trusted_roots))
    }
}

impl McpAuthority for ProductionMcpAuthority {
    type EffectLease = ProductionMcpEffectLease;

    fn resolve_launch(&self, server: &McpServerSnapshot) -> Result<ResolvedMcpLaunch, McpError> {
        if server.trust != McpTrustState::Trusted {
            return Err(McpError::Untrusted);
        }
        let (active_project, trusted_roots) = self.workspace_launch_paths(server)?;
        let operation = format!(
            "mcp-launch-{}-{}",
            scratch_identity(&server.server_id),
            server.lifecycle_generation
        );
        let mut environment = BTreeMap::new();
        let mut bearer = None;
        let mut headers = BTreeMap::new();
        match &server.transport {
            McpTransportDefinition::Stdio {
                environment: bindings,
                ..
            } => {
                for binding in bindings {
                    let value = match &binding.source {
                        McpEnvironmentSource::Literal { value } => value.clone(),
                        McpEnvironmentSource::Passthrough => {
                            std::env::var(&binding.name).map_err(|_| McpError::Credential)?
                        }
                        McpEnvironmentSource::Secret { reference } => self.resolve_secret(
                            reference,
                            &format!("{operation}-env-{}", binding.name),
                            server.timeout_ms,
                        )?,
                    };
                    environment.insert(binding.name.clone(), value);
                }
            }
            McpTransportDefinition::StreamableHttp {
                bearer: definition_bearer,
                headers: bindings,
                ..
            } => {
                if let Some(reference) = definition_bearer {
                    bearer = Some(self.resolve_secret(
                        reference,
                        &format!("{operation}-bearer"),
                        server.timeout_ms,
                    )?);
                }
                for binding in bindings {
                    let value = match &binding.source {
                        McpHeaderSource::Literal { value } => value.clone(),
                        McpHeaderSource::Environment { variable } => {
                            std::env::var(variable).map_err(|_| McpError::Credential)?
                        }
                        McpHeaderSource::Secret { reference } => self.resolve_secret(
                            reference,
                            &format!("{operation}-header-{}", binding.name),
                            server.timeout_ms,
                        )?,
                    };
                    headers.insert(binding.name.clone(), value);
                }
            }
        }
        Ok(ResolvedMcpLaunch {
            sampling_context: Some(McpSamplingContext {
                server_id: server.server_id.clone(),
                authority_id: mcp_authority_identity(&server.server_id, &server.source)?,
                lifecycle_generation: server.lifecycle_generation,
                definition_sha256: server_definition_sha256(server)?,
                timeout_ms: server.timeout_ms,
                max_output_bytes: server.max_output_bytes,
            }),
            c4os_home: self.c4os_home.clone(),
            scratch_root: private_worker_scratch(
                &self.c4os_home,
                &server.server_id,
                server.lifecycle_generation,
            )?,
            active_project,
            trusted_roots,
            environment,
            bearer,
            headers,
        })
    }

    fn begin_authorized_effect(
        &self,
        request: &McpAuthorityRequest<'_>,
        now_ms: u64,
    ) -> Result<Self::EffectLease, McpError> {
        let server = match request {
            McpAuthorityRequest::Tool { server, .. }
            | McpAuthorityRequest::Resource { server, .. } => *server,
        };
        let (active_project, trusted_roots) = self.workspace_launch_paths(server)?;
        let (facts, action, live) = action_for_request(
            &self.runtime,
            request,
            active_project.as_deref(),
            &trusted_roots,
        )?;
        match self
            .runtime
            .propose_direct_action(&facts, action.clone(), now_ms)
            .map_err(|_| McpError::StateUnavailable)?
        {
            GatewayProposal::Denied { .. } | GatewayProposal::PendingApproval { .. } => {
                Err(McpError::Denied)
            }
            GatewayProposal::Authorized { token, .. } => {
                let gateway = self
                    .runtime
                    .begin_direct_action_effect(&token, &action, live, None, now_ms)
                    .map_err(|_| McpError::Denied)?;
                Ok(ProductionMcpEffectLease { gateway })
            }
        }
    }

    fn redact_result(
        &self,
        request: &McpAuthorityRequest<'_>,
        untrusted: Value,
    ) -> Result<Value, McpError> {
        let server = match request {
            McpAuthorityRequest::Tool { server, .. }
            | McpAuthorityRequest::Resource { server, .. } => *server,
        };
        redact_untrusted_result(self.credential_vault.as_ref(), &server.transport, untrusted)
    }

    fn complete_authorized_effect(
        &self,
        lease: Self::EffectLease,
        status: McpAuthorityEffectStatus,
        redacted_result: &Value,
        now_ms: u64,
    ) -> Result<(), McpError> {
        let output = serde_json::to_vec(redacted_result).map_err(|_| McpError::InvalidState)?;
        let result = normalized_result(status, Some(&output), now_ms);
        self.runtime
            .complete_direct_action_effect(lease.gateway, result)
            .map_err(|_| McpError::StateUnavailable)?;
        Ok(())
    }

    fn fail_authorized_effect(
        &self,
        lease: Self::EffectLease,
        status: McpAuthorityEffectStatus,
        now_ms: u64,
    ) -> Result<(), McpError> {
        self.runtime
            .complete_direct_action_effect(lease.gateway, normalized_result(status, None, now_ms))
            .map_err(|_| McpError::StateUnavailable)?;
        Ok(())
    }

    fn revoke_server(
        &self,
        server: &McpServerSnapshot,
        _reason: &str,
        now_ms: u64,
    ) -> Result<(), McpError> {
        let identity = mcp_authority_identity(&server.server_id, &server.source)?;
        self.runtime
            .revoke_plugin_or_mcp_authority(&identity, now_ms)
            .map_err(|_| McpError::StateUnavailable)?;
        Ok(())
    }
}

fn action_for_request(
    runtime: &RuntimeApplicationService,
    request: &McpAuthorityRequest<'_>,
    active_project: Option<&Path>,
    trusted_roots: &[PathBuf],
) -> Result<(ActionFacts, CanonicalAction, LiveAuthorityState), McpError> {
    let (
        server,
        expected_generation,
        lifecycle_generation,
        mut target,
        arguments,
        run_id,
        workspace_id,
        project_id,
        session_id,
        tool,
        effects,
        reversibility,
        confidence,
    ) = match request {
        McpAuthorityRequest::Tool {
            input,
            advertised: _,
            server,
        } => (
            *server,
            input.expected_generation,
            input.lifecycle_generation,
            format!("mcp://{}/tools/{}", input.server_id, input.tool_name),
            input.arguments.clone(),
            input.run_id.as_str(),
            input.workspace_id.as_str(),
            input.project_id.as_str(),
            input.session_id.as_str(),
            "mcp.tool",
            BTreeSet::from([ActionEffect::Execute]),
            ActionReversibility::Unknown,
            ClassificationConfidence::Ambiguous,
        ),
        McpAuthorityRequest::Resource {
            input,
            advertised: _,
            server,
        } => (
            *server,
            input.expected_generation,
            input.lifecycle_generation,
            input.uri.clone(),
            json!({ "uri": input.uri }),
            input.run_id.as_str(),
            input.workspace_id.as_str(),
            input.project_id.as_str(),
            input.session_id.as_str(),
            "mcp.resource.read",
            BTreeSet::from([ActionEffect::Read]),
            ActionReversibility::Reversible,
            ClassificationConfidence::Known,
        ),
    };
    let identity = mcp_authority_identity(&server.server_id, &server.source)?;
    let live = runtime
        .current_direct_live_authority(lifecycle_generation, expected_generation)
        .map_err(|_| McpError::StateUnavailable)?;
    let remote = matches!(
        server.transport,
        McpTransportDefinition::StreamableHttp { .. }
    );
    let transport_sandboxed = match &server.transport {
        McpTransportDefinition::Stdio {
            executable_sha256, ..
        } => executable_sha256.is_some(),
        McpTransportDefinition::StreamableHttp { bearer, .. } => bearer.is_some(),
    };
    let mut surface = if remote {
        ActionSurface::Network
    } else {
        ActionSurface::Process
    };
    let mut trusted_root = active_project
        .is_some_and(|project| trusted_roots.iter().any(|root| project.starts_with(root)));
    let mut scope = if remote {
        ActionScope::Remote
    } else if trusted_root {
        ActionScope::Workspace
    } else {
        ActionScope::ExternalLocal
    };
    if let McpAuthorityRequest::Resource { input, .. } = request {
        let classified = classify_resource_target(server, input.uri.as_str(), trusted_roots)?;
        target = classified.target;
        surface = classified.surface;
        scope = classified.scope;
        trusted_root = classified.trusted_root;
    }
    let action_id = format!("mcp-{}", Uuid::new_v4().as_simple());
    let target_version = format!("generation-{lifecycle_generation}");
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-{action_id}"),
        tool: tool.into(),
        arguments,
        risk: if tool == "mcp.tool" {
            CanonicalRisk::Unknown
        } else {
            CanonicalRisk::Low
        },
        requested_authority: BTreeSet::from([tool.replace('.', "-")]),
        canonical_target: target.clone(),
        target_version,
        workspace_id: workspace_id.into(),
        session_id: session_id.into(),
        run_id: run_id.into(),
        runtime_id: "mcp-worker".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: Some(identity.clone()),
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| McpError::InvalidInput)?;
    let facts = ActionFacts {
        action_kind: tool.into(),
        native_tool: tool.into(),
        surface,
        effects,
        scope,
        initiator: match &server.source {
            McpDefinitionSource::Plugin { .. } => ActionInitiator::Plugin,
            McpDefinitionSource::User => ActionInitiator::Runtime,
        },
        sensitivity: match &server.transport {
            McpTransportDefinition::StreamableHttp {
                bearer, headers, ..
            } if bearer.is_some()
                || headers
                    .iter()
                    .any(|header| !matches!(&header.source, McpHeaderSource::Literal { .. })) =>
            {
                ActionSensitivity::Authenticated
            }
            _ => ActionSensitivity::Ordinary,
        },
        reversibility,
        confidence,
        request_origin: ActionRequestOrigin::RuntimeTool,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: trusted_root,
        canonical_target: target,
        workspace_id: workspace_id.into(),
        session_id: session_id.into(),
        runtime_id: "mcp-worker".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: Some(identity),
        target_resolved: true,
        authenticated: matches!(
            &server.transport,
            McpTransportDefinition::StreamableHttp {
                bearer: Some(_),
                ..
            }
        ),
        trusted_root,
        explicit_scope_grant: server.trust == McpTrustState::Trusted,
        sandbox_allows: transport_sandboxed,
        declaration_exceeded: false,
    };
    let _ = project_id;
    Ok((facts, action, live))
}

fn normalized_result(
    status: McpAuthorityEffectStatus,
    output: Option<&[u8]>,
    now_ms: u64,
) -> NormalizedActionResult {
    let normalized_status = match status {
        McpAuthorityEffectStatus::Succeeded => NormalizedActionStatus::Succeeded,
        McpAuthorityEffectStatus::Cancelled => NormalizedActionStatus::Cancelled,
        McpAuthorityEffectStatus::Failed
        | McpAuthorityEffectStatus::TimedOut
        | McpAuthorityEffectStatus::OutputLimitExceeded => NormalizedActionStatus::Failed,
    };
    NormalizedActionResult {
        status: normalized_status,
        result_code: match status {
            McpAuthorityEffectStatus::Succeeded => "mcp-succeeded",
            McpAuthorityEffectStatus::Failed => "mcp-failed",
            McpAuthorityEffectStatus::Cancelled => "mcp-cancelled",
            McpAuthorityEffectStatus::TimedOut => "mcp-timed-out",
            McpAuthorityEffectStatus::OutputLimitExceeded => "mcp-output-limit-exceeded",
        }
        .into(),
        exit_code: None,
        changed_targets: Vec::new(),
        output_sha256: output.map(sha256_prefixed),
        completed_at_ms: now_ms.max(1),
    }
}

fn redact_value(value: Value, depth: usize, nodes: &mut usize) -> Result<Value, McpError> {
    *nodes = nodes.checked_add(1).ok_or(McpError::BoundExceeded)?;
    if depth > MAX_REDACTION_DEPTH || *nodes > MAX_REDACTION_NODES {
        return Err(McpError::BoundExceeded);
    }
    Ok(match value {
        Value::Object(fields) => {
            let mut output = serde_json::Map::new();
            for (key, value) in fields {
                let sensitive = ["authorization", "cookie", "password", "secret", "token"]
                    .iter()
                    .any(|needle| key.to_ascii_lowercase().contains(needle));
                output.insert(
                    key,
                    if sensitive {
                        Value::String("<redacted>".into())
                    } else {
                        redact_value(value, depth + 1, nodes)?
                    },
                );
            }
            Value::Object(output)
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|value| redact_value(value, depth + 1, nodes))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::String(value) => Value::String(value),
        scalar => scalar,
    })
}

fn sanitize_redacted_value(
    value: &mut Value,
    credential_vault: Option<&CredentialVault>,
    environment_secrets: &[String],
) -> Result<(), McpError> {
    match value {
        Value::String(text) => *text = bounded_text(text),
        Value::Array(values) => {
            for value in values {
                sanitize_redacted_value(value, credential_vault, environment_secrets)?;
            }
        }
        Value::Object(fields) => {
            let mut output = serde_json::Map::new();
            for (key, mut child) in std::mem::take(fields) {
                let mut safe_key = match credential_vault {
                    Some(vault) => vault
                        .redact_text(&key)
                        .map_err(|_| McpError::StateUnavailable)?,
                    None => key,
                };
                for secret in environment_secrets {
                    safe_key = safe_key.replace(secret, "[REDACTED]");
                }
                sanitize_redacted_value(&mut child, credential_vault, environment_secrets)?;
                output.insert(bounded_text(&safe_key), child);
            }
            *fields = output;
        }
        _ => {}
    }
    Ok(())
}

fn bounded_text(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        let character = if character.is_control() && character != '\n' && character != '\t' {
            '\u{fffd}'
        } else {
            character
        };
        if output.len() + character.len_utf8() > MAX_MCP_TEXT_BYTES {
            break;
        }
        output.push(character);
    }
    output
}

struct ResourceTargetClassification {
    target: String,
    surface: ActionSurface,
    scope: ActionScope,
    trusted_root: bool,
}

fn classify_resource_target(
    server: &McpServerSnapshot,
    value: &str,
    trusted_roots: &[PathBuf],
) -> Result<ResourceTargetClassification, McpError> {
    let uri = Url::parse(value).map_err(|_| McpError::Denied)?;
    if uri.scheme() == "file" {
        let requested = uri.to_file_path().map_err(|_| McpError::Denied)?;
        let canonical = requested.canonicalize().map_err(|_| McpError::Denied)?;
        let trusted_root = trusted_roots.iter().any(|root| canonical.starts_with(root));
        if !trusted_root {
            return Err(McpError::Denied);
        }
        return Ok(ResourceTargetClassification {
            target: canonical.to_string_lossy().into_owned(),
            surface: ActionSurface::File,
            scope: ActionScope::Workspace,
            trusted_root: true,
        });
    }
    let remote = matches!(
        server.transport,
        McpTransportDefinition::StreamableHttp { .. }
    );
    Ok(ResourceTargetClassification {
        target: format!(
            "mcp://{}/resources/{}",
            server.server_id,
            hex_digest(&Sha256::digest(value.as_bytes()))
        ),
        surface: if remote {
            ActionSurface::Network
        } else {
            ActionSurface::Process
        },
        scope: if remote {
            ActionScope::Remote
        } else {
            ActionScope::ExternalLocal
        },
        trusted_root: false,
    })
}

fn redact_untrusted_result(
    credential_vault: Option<&CredentialVault>,
    transport: &McpTransportDefinition,
    untrusted: Value,
) -> Result<Value, McpError> {
    let mut nodes = 0usize;
    let mut redacted = redact_value(untrusted, 0, &mut nodes)?;
    if let Some(vault) = credential_vault {
        vault
            .redact_diagnostic_value(&mut redacted)
            .map_err(|_| McpError::StateUnavailable)?;
    }
    let environment_secrets = environment_secret_values(transport);
    for secret in &environment_secrets {
        redact_exact_text(&mut redacted, secret);
    }
    sanitize_redacted_value(&mut redacted, credential_vault, &environment_secrets)?;
    Ok(redacted)
}

fn environment_secret_values(transport: &McpTransportDefinition) -> Vec<String> {
    let mut variables = Vec::new();
    match transport {
        McpTransportDefinition::Stdio { environment, .. } => {
            for binding in environment {
                match &binding.source {
                    McpEnvironmentSource::Passthrough => variables.push(binding.name.as_str()),
                    McpEnvironmentSource::Secret {
                        reference: McpSecretReference::Environment { variable },
                    } => variables.push(variable.as_str()),
                    _ => {}
                }
            }
        }
        McpTransportDefinition::StreamableHttp {
            bearer, headers, ..
        } => {
            if let Some(McpSecretReference::Environment { variable }) = bearer {
                variables.push(variable.as_str());
            }
            for header in headers {
                match &header.source {
                    McpHeaderSource::Environment { variable }
                    | McpHeaderSource::Secret {
                        reference: McpSecretReference::Environment { variable },
                    } => variables.push(variable.as_str()),
                    _ => {}
                }
            }
        }
    }
    variables
        .into_iter()
        .filter_map(|variable| std::env::var(variable).ok())
        .filter(|value| !value.is_empty())
        .collect()
}

fn redact_exact_text(value: &mut Value, secret: &str) {
    match value {
        Value::String(text) => *text = text.replace(secret, "[REDACTED]"),
        Value::Array(values) => {
            for value in values {
                redact_exact_text(value, secret);
            }
        }
        Value::Object(fields) => {
            for value in fields.values_mut() {
                redact_exact_text(value, secret);
            }
        }
        _ => {}
    }
}

fn private_worker_scratch(
    c4os_home: &Path,
    server_id: &str,
    lifecycle_generation: u64,
) -> Result<PathBuf, McpError> {
    let scratch = c4os_home
        .join("mcp")
        .join("scratch")
        .join(scratch_identity(server_id))
        .join(format!("generation-{lifecycle_generation}"));
    std::fs::create_dir_all(&scratch).map_err(|_| McpError::StateUnavailable)?;
    let canonical = scratch
        .canonicalize()
        .map_err(|_| McpError::StateUnavailable)?;
    if !canonical.starts_with(c4os_home) || canonical != scratch {
        return Err(McpError::Denied);
    }
    let relative = scratch
        .strip_prefix(c4os_home)
        .map_err(|_| McpError::Denied)?;
    let mut current = c4os_home.to_path_buf();
    for component in relative.components() {
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current).map_err(|_| McpError::Denied)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(McpError::Denied);
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| McpError::StateUnavailable)?;
    }
    Ok(scratch)
}

fn scratch_identity(server_id: &str) -> String {
    format!("mcp-{}", hex_digest(&Sha256::digest(server_id.as_bytes())))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", hex_digest(&Sha256::digest(bytes)))
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn canonical_directory(path: &Path) -> Result<PathBuf, McpError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| McpError::StateUnavailable)?;
    let canonical = path
        .canonicalize()
        .map_err(|_| McpError::StateUnavailable)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() || canonical != path {
        return Err(McpError::Denied);
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use serde_json::json;

    use super::*;
    use crate::mcp::{McpEnvironmentBinding, McpWorkingDirectory};
    use crate::security::credentials::SecretSurface;

    static ENVIRONMENT_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn production_redaction_removes_environment_secrets_under_innocent_keys() {
        let _guard = ENVIRONMENT_LOCK.lock().expect("environment lock");
        let variable = "C4OS_MCP_RESULT_REDACTION_CANARY";
        let canary = "mcp-canary-value-4f8567b2";
        // SAFETY: this test serializes access to its unique process variable and
        // restores it before releasing the lock.
        unsafe { std::env::set_var(variable, canary) };
        let transport = McpTransportDefinition::Stdio {
            command: "/usr/bin/false".into(),
            arguments: Vec::new(),
            environment: vec![McpEnvironmentBinding {
                name: "RESULT_REFERENCE".into(),
                source: McpEnvironmentSource::Secret {
                    reference: McpSecretReference::Environment {
                        variable: variable.into(),
                    },
                },
            }],
            working_directory: McpWorkingDirectory::C4osHome,
            executable_sha256: Some(format!("sha256:{}", "0".repeat(64))),
        };

        let redacted = redact_untrusted_result(
            None,
            &transport,
            json!({
                "message": format!("prefix-{canary}-suffix"),
                "nested": { "value": canary },
                "safe": "kept",
            }),
        )
        .expect("production redaction");
        // SAFETY: access remains serialized and the unique variable is removed
        // before another test can observe it.
        unsafe { std::env::remove_var(variable) };

        assert_eq!(redacted["message"], "prefix-[REDACTED]-suffix");
        assert_eq!(redacted["nested"]["value"], "[REDACTED]");
        assert_eq!(redacted["safe"], "kept");
        assert!(
            !serde_json::to_string(&redacted)
                .expect("serialize")
                .contains(canary)
        );
    }

    #[test]
    fn production_redaction_removes_long_vault_secrets_before_bounding() {
        let vault = CredentialVault::session_only().expect("session vault");
        let canary = format!("{}-vault-tail", "v".repeat(MAX_MCP_TEXT_BYTES));
        let leaked_prefix = "v".repeat(MAX_MCP_TEXT_BYTES);
        let reference = vault
            .store("mcp-result-test", canary.as_bytes())
            .expect("store canary");
        let transport = McpTransportDefinition::Stdio {
            command: "/usr/bin/false".into(),
            arguments: Vec::new(),
            environment: vec![McpEnvironmentBinding {
                name: "RESULT_REFERENCE".into(),
                source: McpEnvironmentSource::Secret {
                    reference: McpSecretReference::Vault {
                        credential_reference: reference.to_string(),
                    },
                },
            }],
            working_directory: McpWorkingDirectory::C4osHome,
            executable_sha256: Some(format!("sha256:{}", "0".repeat(64))),
        };

        let redacted = redact_untrusted_result(
            Some(&vault),
            &transport,
            json!({ "value": canary, "safe": "kept" }),
        )
        .expect("production redaction");
        let serialized = serde_json::to_vec(&redacted).expect("serialize");

        assert_eq!(redacted["value"], "[REDACTED]");
        assert_eq!(redacted["safe"], "kept");
        assert!(!String::from_utf8_lossy(&serialized).contains(&leaked_prefix));
        vault
            .ensure_clean(SecretSurface::RendererState, &serialized)
            .expect("renderer-safe result");
        vault
            .ensure_clean(SecretSurface::Log, &serialized)
            .expect("log-safe result");
    }
}
