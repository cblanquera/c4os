//! Durable MCP lifecycle and authority composition.

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::security::gateway::ExecutionPermit;

use super::transport::{
    McpCancellation, McpConnection, McpRawResult, McpTransportFactory, ResolvedMcpLaunch,
};
use super::{
    MAX_MCP_TURN_CATALOG_BYTES, MAX_MCP_TURN_TOOLS, MCP_STATE_SCHEMA_VERSION, McpAuditEvent,
    McpCapabilitySnapshot, McpDefinitionSource, McpError, McpInvocationSnapshot,
    McpInvocationStatus, McpLifecycle, McpPendingTrustApproval, McpResourceReadInput,
    McpResourceSnapshot, McpServerDefinitionInput, McpServerMutationInput,
    McpServerRevocationInput, McpServerSnapshot, McpServiceSnapshot, McpToolCallInput,
    McpToolSnapshot, McpTrustApprovalState, McpTrustState, McpTurnSnapshot, McpTurnToolSnapshot,
    mcp_authority_identity, server_definition_sha256, validate_definition_input,
    validate_identifier, validate_mcp_input_schema, validate_mcp_name, validate_text,
};

/// The persistence implementation must atomically compare the complete
/// document generation and append the supplied audit event before returning.
pub trait McpRepository: Send + Sync {
    fn load(&self) -> Result<Option<McpServiceSnapshot>, McpError>;

    fn compare_and_swap(
        &self,
        expected_generation: Option<u64>,
        replacement: &McpServiceSnapshot,
        event: &McpAuditEvent,
    ) -> Result<(), McpError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum McpAuthorityRequest<'a> {
    Tool {
        server: &'a McpServerSnapshot,
        input: &'a McpToolCallInput,
        advertised: &'a McpToolSnapshot,
    },
    Resource {
        server: &'a McpServerSnapshot,
        input: &'a McpResourceReadInput,
        advertised: &'a McpResourceSnapshot,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum McpAuthorityEffectStatus {
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    OutputLimitExceeded,
}

/// Core-owned authority hooks. Implementations compose credential leases,
/// capability preflight, PolicyService, ActionGateway, audit correlation, and
/// renderer-safe redaction. A transport is never consulted before an effect
/// lease has been durably started.
pub trait McpAuthority: Send + Sync {
    type EffectLease: Send;

    fn resolve_launch(&self, server: &McpServerSnapshot) -> Result<ResolvedMcpLaunch, McpError>;

    /// Returns only after capability/policy evaluation, approval/authorization
    /// consumption, and durable ActionGateway effect-start have succeeded.
    fn begin_authorized_effect(
        &self,
        request: &McpAuthorityRequest<'_>,
        now_ms: u64,
    ) -> Result<Self::EffectLease, McpError>;

    fn redact_result(
        &self,
        request: &McpAuthorityRequest<'_>,
        untrusted: Value,
    ) -> Result<Value, McpError>;

    /// Makes the redacted normalized outcome and ActionGateway effect-finished
    /// record durable. The opaque lease cannot be forged by the transport.
    fn complete_authorized_effect(
        &self,
        lease: Self::EffectLease,
        status: McpAuthorityEffectStatus,
        redacted_result: &Value,
        now_ms: u64,
    ) -> Result<(), McpError>;

    /// Closes an effect-start lease when no trusted normalized result exists.
    fn fail_authorized_effect(
        &self,
        lease: Self::EffectLease,
        status: McpAuthorityEffectStatus,
        now_ms: u64,
    ) -> Result<(), McpError>;

    fn revoke_server(
        &self,
        server: &McpServerSnapshot,
        reason: &str,
        now_ms: u64,
    ) -> Result<(), McpError>;
}

struct WorkerSlot {
    /// `None` is a deliberate in-flight placeholder. Production broker I/O
    /// temporarily owns the connection so the service mutex is never held
    /// across an untrusted transport await, while lifecycle commands can still
    /// observe and atomically quiesce the exact worker generation.
    connection: Option<Box<dyn McpConnection>>,
    observed_notification_epoch: u64,
}

pub(crate) struct McpPreparedToolInvocation {
    server: McpServerSnapshot,
    input: McpToolCallInput,
    advertised: McpToolSnapshot,
    connection: Box<dyn McpConnection>,
    started_event_id: u64,
}

impl McpPreparedToolInvocation {
    pub(crate) async fn execute(
        &self,
        cancellation: McpCancellation,
    ) -> Result<McpRawResult, McpError> {
        self.connection
            .call_tool(
                &self.input.tool_name,
                self.input.arguments.clone(),
                self.server.timeout_ms,
                self.server.max_output_bytes,
                cancellation,
            )
            .await
    }

    pub(crate) fn server_id(&self) -> &str {
        &self.server.server_id
    }
}

pub struct McpService<R, A, F>
where
    R: McpRepository,
    A: McpAuthority,
    F: McpTransportFactory,
{
    repository: Arc<R>,
    authority: Arc<A>,
    factory: Arc<F>,
    snapshot: McpServiceSnapshot,
    workers: BTreeMap<String, WorkerSlot>,
}

impl<R, A, F> McpService<R, A, F>
where
    R: McpRepository,
    A: McpAuthority,
    F: McpTransportFactory,
{
    pub fn restore(
        repository: Arc<R>,
        authority: Arc<A>,
        factory: Arc<F>,
        now_ms: u64,
    ) -> Result<Self, McpError> {
        let snapshot = match repository.load()? {
            Some(snapshot) => {
                let mut normalized = normalize_after_restart(snapshot.clone())?;
                normalized.validate()?;
                if normalized != snapshot {
                    normalized.generation = snapshot
                        .generation
                        .checked_add(1)
                        .ok_or(McpError::InvalidState)?;
                    normalized.last_event_id = snapshot
                        .last_event_id
                        .checked_add(1)
                        .ok_or(McpError::InvalidState)?;
                    let event = McpAuditEvent {
                        schema_version: MCP_STATE_SCHEMA_VERSION,
                        event_id: normalized.last_event_id,
                        generation: normalized.generation,
                        lifecycle_generation: 0,
                        operation_id: operation_id(
                            "mcp-service",
                            "restart-recovery",
                            normalized.last_event_id,
                        ),
                        server_id: "mcp-service".into(),
                        event_kind: "service.restart_recovery".into(),
                        target: None,
                        result: "recovered".into(),
                        detail: Some(
                            "active MCP workers require explicit recovery after restart".into(),
                        ),
                        occurred_at_ms: now_ms,
                    };
                    normalized.validate()?;
                    repository.compare_and_swap(Some(snapshot.generation), &normalized, &event)?;
                }
                normalized
            }
            None => {
                let snapshot = McpServiceSnapshot {
                    schema_version: MCP_STATE_SCHEMA_VERSION,
                    generation: 1,
                    servers: Vec::new(),
                    active_workers: 0,
                    last_event_id: 1,
                };
                let event = McpAuditEvent {
                    schema_version: MCP_STATE_SCHEMA_VERSION,
                    event_id: 1,
                    generation: 1,
                    lifecycle_generation: 0,
                    operation_id: "mcp-service-initialize".into(),
                    server_id: "mcp-service".into(),
                    event_kind: "service.initialized".into(),
                    target: None,
                    result: "succeeded".into(),
                    detail: None,
                    occurred_at_ms: now_ms,
                };
                repository.compare_and_swap(None, &snapshot, &event)?;
                snapshot
            }
        };
        Ok(Self {
            repository,
            authority,
            factory,
            snapshot,
            workers: BTreeMap::new(),
        })
    }

    pub fn snapshot(&self) -> McpServiceSnapshot {
        self.snapshot.clone()
    }

    /// Captures the exact bounded MCP tool set visible to one Chat turn. The
    /// route is minted from live Rust-owned lifecycle/trust truth and is never
    /// accepted from the renderer or MCP peer.
    pub fn turn_snapshot(
        &self,
        workspace_id: &str,
        project_id: &str,
        session_id: &str,
        captured_at_ms: u64,
    ) -> Result<McpTurnSnapshot, McpError> {
        validate_identifier(workspace_id)?;
        validate_identifier(project_id)?;
        validate_identifier(session_id)?;
        if captured_at_ms == 0 {
            return Err(McpError::InvalidInput);
        }
        let mut eligible = Vec::new();
        for server in &self.snapshot.servers {
            if server.lifecycle != McpLifecycle::Ready
                || !self.workers.contains_key(&server.server_id)
                || !has_current_trust_binding(server)?
                || server.protocol_version.as_deref() != Some(super::MCP_PROTOCOL_VERSION)
                || !server.capabilities.tools
                || scope_matches(server, workspace_id, project_id, session_id).is_err()
            {
                continue;
            }
            let definition_sha256 = server_definition_sha256(server)?;
            for advertised in &server.tools {
                let route_digest = sha256_value(&json!({
                    "serverId": server.server_id,
                    "source": server.source,
                    "lifecycleGeneration": server.lifecycle_generation,
                    "definitionSha256": definition_sha256,
                    "toolName": advertised.name,
                    "inputSchemaSha256": advertised.input_schema_sha256,
                    "outputSchemaSha256": advertised.output_schema_sha256,
                    "workspaceId": workspace_id,
                    "projectId": project_id,
                    "sessionId": session_id,
                }))?;
                eligible.push(McpTurnToolSnapshot {
                    target_id: format!("mcp-tool:{}", route_digest.trim_start_matches("sha256:")),
                    server_id: server.server_id.clone(),
                    source: server.source.clone(),
                    lifecycle_generation: server.lifecycle_generation,
                    definition_sha256: definition_sha256.clone(),
                    transport_kind: server.transport.kind(),
                    tool_name: advertised.name.clone(),
                    title: advertised.title.clone(),
                    description: advertised.description.clone(),
                    input_schema: advertised.input_schema.clone(),
                    input_schema_sha256: advertised.input_schema_sha256.clone(),
                    output_schema_sha256: advertised.output_schema_sha256.clone(),
                });
            }
        }
        eligible.sort_by(|left, right| left.target_id.cmp(&right.target_id));
        let eligible_count = eligible.len();
        let mut tools = Vec::new();
        for tool in eligible {
            if tools.len() >= MAX_MCP_TURN_TOOLS {
                break;
            }
            let mut candidate = tools.clone();
            candidate.push(tool.clone());
            if serde_json::to_vec(&candidate)
                .map_err(|_| McpError::InvalidState)?
                .len()
                > MAX_MCP_TURN_CATALOG_BYTES / 2
            {
                break;
            }
            tools.push(tool);
        }
        let omitted_tool_count = u32::try_from(eligible_count.saturating_sub(tools.len()))
            .map_err(|_| McpError::BoundExceeded)?;
        let digest_payload = json!({
            "serviceGeneration": self.snapshot.generation,
            "capturedAtMs": captured_at_ms,
            "workspaceId": workspace_id,
            "projectId": project_id,
            "sessionId": session_id,
            "tools": &tools,
            "truncated": omitted_tool_count > 0,
            "omittedToolCount": omitted_tool_count,
        });
        let sha256 = sha256_value(&digest_payload)?;
        let snapshot = McpTurnSnapshot {
            snapshot_id: format!("mcp-turn:{}", sha256.trim_start_matches("sha256:")),
            service_generation: self.snapshot.generation,
            captured_at_ms,
            workspace_id: workspace_id.to_owned(),
            project_id: project_id.to_owned(),
            session_id: session_id.to_owned(),
            tools,
            truncated: omitted_tool_count > 0,
            omitted_tool_count,
            sha256,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// Refreshes peer-declared list changes immediately before freezing one
    /// Chat turn. Existing turns retain their immutable catalog; only this new
    /// turn observes a successfully persisted notification refresh.
    pub async fn prepare_turn_snapshot(
        &mut self,
        workspace_id: &str,
        project_id: &str,
        session_id: &str,
        captured_at_ms: u64,
    ) -> Result<McpTurnSnapshot, McpError> {
        let refresh = self
            .snapshot
            .servers
            .iter()
            .filter(|server| {
                server.lifecycle == McpLifecycle::Ready
                    && scope_matches(server, workspace_id, project_id, session_id).is_ok()
            })
            .filter_map(|server| {
                self.workers.get(&server.server_id).and_then(|worker| {
                    worker.connection.as_ref().and_then(|connection| {
                        (connection.notification_epoch() > worker.observed_notification_epoch)
                            .then(|| server.server_id.clone())
                    })
                })
            })
            .collect::<Vec<_>>();
        for server_id in refresh {
            let input = McpServerMutationInput {
                expected_generation: self.snapshot.generation,
                server_id,
            };
            self.refresh_notifications(&input, captured_at_ms).await?;
        }
        self.turn_snapshot(workspace_id, project_id, session_id, captured_at_ms)
    }

    pub fn upsert_server(
        &mut self,
        input: McpServerDefinitionInput,
        source: McpDefinitionSource,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        validate_definition_input(&input)?;
        self.require_generation(input.expected_generation)?;
        if let McpDefinitionSource::Plugin {
            package_id,
            declaration_id,
        } = &source
        {
            validate_identifier(package_id)?;
            validate_identifier(declaration_id)?;
        }

        let mut replacement = self.snapshot.clone();
        let server = if let Some(existing) = replacement
            .servers
            .iter_mut()
            .find(|server| server.server_id == input.server_id)
        {
            if self.workers.contains_key(&input.server_id)
                || !matches!(
                    existing.lifecycle,
                    McpLifecycle::Disabled | McpLifecycle::Failed
                )
                || matches!(existing.trust, McpTrustState::Revoked)
            {
                return Err(McpError::InvalidState);
            }
            existing.display_name = input.display_name;
            existing.source = source;
            existing.scope = input.scope;
            existing.transport = input.transport;
            self.authority
                .revoke_server(existing, "definition-updated", now_ms)?;
            existing.trust = McpTrustState::Pending;
            existing.trusted_definition_sha256 = None;
            existing.pending_trust_approval = None;
            existing.lifecycle = McpLifecycle::Disabled;
            existing.timeout_ms = input.timeout_ms;
            existing.max_output_bytes = input.max_output_bytes;
            existing.restart_attempts = 0;
            existing.next_restart_at_ms = None;
            existing.protocol_version = None;
            existing.server_name = None;
            existing.server_version = None;
            existing.instructions_present = false;
            existing.capabilities = McpCapabilitySnapshot::default();
            existing.tools.clear();
            existing.resources.clear();
            existing.active_requests = 0;
            existing.last_failure_code = None;
            existing.last_failure_detail = None;
            existing.lifecycle_generation = existing
                .lifecycle_generation
                .checked_add(1)
                .ok_or(McpError::InvalidState)?;
            existing
        } else {
            if replacement.servers.len() >= super::MAX_MCP_SERVERS {
                return Err(McpError::BoundExceeded);
            }
            replacement.servers.push(McpServerSnapshot {
                server_id: input.server_id.clone(),
                display_name: input.display_name,
                source,
                scope: input.scope,
                transport: input.transport,
                trust: McpTrustState::Pending,
                trusted_definition_sha256: None,
                pending_trust_approval: None,
                lifecycle: McpLifecycle::Disabled,
                timeout_ms: input.timeout_ms,
                max_output_bytes: input.max_output_bytes,
                lifecycle_generation: 1,
                restart_attempts: 0,
                next_restart_at_ms: None,
                protocol_version: None,
                server_name: None,
                server_version: None,
                instructions_present: false,
                capabilities: McpCapabilitySnapshot::default(),
                tools: Vec::new(),
                resources: Vec::new(),
                active_requests: 0,
                last_connected_at_ms: None,
                last_failure_code: None,
                last_failure_detail: None,
                last_event_id: 0,
            });
            replacement
                .servers
                .last_mut()
                .ok_or(McpError::InvalidState)?
        };
        let server_id = server.server_id.clone();
        let lifecycle_generation = server.lifecycle_generation;
        replacement
            .servers
            .sort_by(|left, right| left.server_id.cmp(&right.server_id));
        self.commit(
            replacement,
            &server_id,
            lifecycle_generation,
            "definition.saved",
            None,
            "succeeded",
            None,
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    /// Commits trust for one exact, already-persisted definition. The caller
    /// must first consume an ActionGateway authorization bound to the returned
    /// definition digest; renderer input can never mint this transition.
    pub fn trust_server(
        &mut self,
        input: &McpServerMutationInput,
        prompt_id: &str,
        expected_action_binding_sha256: &str,
        expected_definition_sha256: &str,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        validate_identifier(&input.server_id)?;
        let mut replacement = self.snapshot.clone();
        let server = replacement
            .servers
            .iter_mut()
            .find(|server| server.server_id == input.server_id)
            .ok_or(McpError::InvalidInput)?;
        let approval = server
            .pending_trust_approval
            .as_ref()
            .ok_or(McpError::Conflict)?;
        if server.trust != McpTrustState::Pending
            || server.lifecycle != McpLifecycle::Disabled
            || self.workers.contains_key(&input.server_id)
            || approval.prompt_id != prompt_id
            || approval.action_binding_sha256 != expected_action_binding_sha256
            || approval.definition_sha256 != expected_definition_sha256
            || definition_sha256(server)? != expected_definition_sha256
        {
            return Err(McpError::Conflict);
        }
        server.trust = McpTrustState::Trusted;
        server.trusted_definition_sha256 = Some(expected_definition_sha256.to_owned());
        server.pending_trust_approval = None;
        let lifecycle_generation = server.lifecycle_generation;
        self.commit(
            replacement,
            &input.server_id,
            lifecycle_generation,
            "definition.trusted",
            Some(expected_definition_sha256),
            "succeeded",
            None,
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub fn record_trust_approval(
        &mut self,
        input: &McpServerMutationInput,
        approval: McpPendingTrustApproval,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        validate_identifier(&input.server_id)?;
        validate_identifier(&approval.prompt_id)?;
        super::validate_digest(&approval.definition_sha256)?;
        super::validate_digest(&approval.action_binding_sha256)?;
        if approval.action_configuration_version != input.expected_generation
            || approval.requested_at_ms == 0
            || approval.requested_at_ms >= approval.expires_at_ms
        {
            return Err(McpError::InvalidInput);
        }
        let mut replacement = self.snapshot.clone();
        let server = server_mut(&mut replacement, &input.server_id)?;
        if server.trust != McpTrustState::Pending
            || server.lifecycle != McpLifecycle::Disabled
            || self.workers.contains_key(&input.server_id)
            || definition_sha256(server)? != approval.definition_sha256
        {
            return Err(McpError::Conflict);
        }
        let lifecycle_generation = server.lifecycle_generation;
        let definition_sha256 = approval.definition_sha256.clone();
        server.pending_trust_approval = Some(approval);
        self.commit(
            replacement,
            &input.server_id,
            lifecycle_generation,
            "definition.trust_requested",
            Some(&definition_sha256),
            "pending_approval",
            None,
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub fn clear_trust_approval(
        &mut self,
        input: &McpServerMutationInput,
        prompt_id: &str,
        result: &str,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        validate_identifier(&input.server_id)?;
        validate_identifier(prompt_id)?;
        validate_identifier(result)?;
        let mut replacement = self.snapshot.clone();
        let server = server_mut(&mut replacement, &input.server_id)?;
        if server
            .pending_trust_approval
            .as_ref()
            .is_none_or(|approval| approval.prompt_id != prompt_id)
        {
            return Err(McpError::Conflict);
        }
        server.pending_trust_approval = None;
        let lifecycle_generation = server.lifecycle_generation;
        self.commit(
            replacement,
            &input.server_id,
            lifecycle_generation,
            "definition.trust_settled",
            None,
            result,
            None,
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub fn definition_sha256(&self, server_id: &str) -> Result<String, McpError> {
        validate_identifier(server_id)?;
        definition_sha256(self.server(server_id)?)
    }

    /// Tests a server without enabling it or retaining a live worker.
    pub async fn test_server(
        &mut self,
        input: &McpServerMutationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        let server =
            self.preflight_connect(input, &[McpLifecycle::Disabled, McpLifecycle::Failed])?;
        self.commit_lifecycle(
            &server.server_id,
            McpLifecycle::Testing,
            "lifecycle.testing",
            now_ms,
        )?;
        match self.connect_and_discover(&server).await {
            Ok((connection, handshake, tools, resources, _notification_epoch)) => {
                let mut replacement = self.snapshot.clone();
                let target = server_mut(&mut replacement, &server.server_id)?;
                apply_discovery(target, &handshake, tools, resources, now_ms);
                target.lifecycle = McpLifecycle::Disabled;
                let lifecycle_generation = target.lifecycle_generation;
                self.commit(
                    replacement,
                    &server.server_id,
                    lifecycle_generation,
                    "lifecycle.tested",
                    None,
                    "succeeded",
                    None,
                    now_ms,
                )?;
                connection.close(server.timeout_ms).await?;
                Ok(self.snapshot())
            }
            Err(error) => {
                self.record_failure(&server.server_id, &error, now_ms)?;
                Err(error)
            }
        }
    }

    pub async fn enable_server(
        &mut self,
        input: &McpServerMutationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.connect_server(input, &[McpLifecycle::Disabled], now_ms)
            .await
    }

    async fn connect_server(
        &mut self,
        input: &McpServerMutationInput,
        allowed: &[McpLifecycle],
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        let server = self.preflight_connect(input, allowed)?;
        self.commit_lifecycle(
            &server.server_id,
            McpLifecycle::Connecting,
            "lifecycle.connecting",
            now_ms,
        )?;
        match self.connect_and_discover(&server).await {
            Ok((connection, handshake, tools, resources, notification_epoch)) => {
                let mut replacement = self.snapshot.clone();
                let target = server_mut(&mut replacement, &server.server_id)?;
                target.lifecycle_generation = target
                    .lifecycle_generation
                    .checked_add(1)
                    .ok_or(McpError::InvalidState)?;
                apply_discovery(target, &handshake, tools, resources, now_ms);
                target.lifecycle = McpLifecycle::Ready;
                let lifecycle_generation = target.lifecycle_generation;
                replacement.active_workers =
                    u16::try_from(self.workers.len() + 1).map_err(|_| McpError::BoundExceeded)?;
                if let Err(error) = self.commit(
                    replacement,
                    &server.server_id,
                    lifecycle_generation,
                    "lifecycle.enabled",
                    None,
                    "succeeded",
                    None,
                    now_ms,
                ) {
                    let _ = connection.close(server.timeout_ms).await;
                    return Err(error);
                }
                self.workers.insert(
                    server.server_id,
                    WorkerSlot {
                        connection: Some(connection),
                        observed_notification_epoch: notification_epoch,
                    },
                );
                Ok(self.snapshot())
            }
            Err(error) => {
                self.record_failure(&server.server_id, &error, now_ms)?;
                Err(error)
            }
        }
    }

    pub async fn disable_server(
        &mut self,
        input: &McpServerMutationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        let server = self.server(&input.server_id)?.clone();
        if matches!(server.trust, McpTrustState::Revoked) {
            return Err(McpError::Revoked);
        }
        if !matches!(
            server.lifecycle,
            McpLifecycle::Ready
                | McpLifecycle::Executing
                | McpLifecycle::Failed
                | McpLifecycle::Restarting
                | McpLifecycle::Connecting
        ) {
            return Err(McpError::InvalidState);
        }
        self.authority
            .revoke_server(&server, "server-disabled", now_ms)?;
        let mut replacement = self.snapshot.clone();
        let target = server_mut(&mut replacement, &server.server_id)?;
        target.lifecycle = McpLifecycle::Disabled;
        target.active_requests = 0;
        target.next_restart_at_ms = None;
        let lifecycle_generation = target.lifecycle_generation;
        replacement.active_workers = u16::try_from(self.workers.len().saturating_sub(1))
            .map_err(|_| McpError::BoundExceeded)?;
        if let Err(error) = self.commit(
            replacement,
            &server.server_id,
            lifecycle_generation,
            "lifecycle.disabled",
            None,
            "succeeded",
            None,
            now_ms,
        ) {
            self.fail_closed_worker_after_commit_failure(&server).await;
            return Err(error);
        }
        if let Some(worker) = self.workers.remove(&server.server_id)
            && let Some(connection) = worker.connection
        {
            connection.close(server.timeout_ms).await?;
        }
        Ok(self.snapshot())
    }

    pub fn delete_server(
        &mut self,
        input: &McpServerMutationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        validate_identifier(&input.server_id)?;
        let server = self.server(&input.server_id)?.clone();
        if self.workers.contains_key(&server.server_id)
            || !matches!(
                server.lifecycle,
                McpLifecycle::Disabled | McpLifecycle::Failed
            )
        {
            return Err(McpError::InvalidState);
        }
        let mut replacement = self.snapshot.clone();
        replacement
            .servers
            .retain(|candidate| candidate.server_id != server.server_id);
        self.commit(
            replacement,
            &server.server_id,
            server.lifecycle_generation,
            "definition.deleted",
            None,
            "succeeded",
            None,
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub async fn revoke_server(
        &mut self,
        input: &McpServerRevocationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        validate_identifier(&input.server_id)?;
        validate_text(&input.reason)?;
        let server = self.server(&input.server_id)?.clone();
        self.authority
            .revoke_server(&server, &input.reason, now_ms)?;
        let mut replacement = self.snapshot.clone();
        let target = server_mut(&mut replacement, &server.server_id)?;
        target.trust = McpTrustState::Revoked;
        target.trusted_definition_sha256 = None;
        target.lifecycle = McpLifecycle::Revoked;
        target.active_requests = 0;
        target.next_restart_at_ms = None;
        target.last_failure_code = Some("revoked".into());
        target.last_failure_detail = Some("Server authority was revoked".into());
        let lifecycle_generation = target.lifecycle_generation;
        replacement.active_workers = u16::try_from(self.workers.len().saturating_sub(1))
            .map_err(|_| McpError::BoundExceeded)?;
        if let Err(error) = self.commit(
            replacement,
            &server.server_id,
            lifecycle_generation,
            "lifecycle.revoked",
            None,
            "succeeded",
            Some("server and outstanding authority revoked"),
            now_ms,
        ) {
            self.fail_closed_worker_after_commit_failure(&server).await;
            return Err(error);
        }
        if let Some(worker) = self.workers.remove(&server.server_id)
            && let Some(connection) = worker.connection
        {
            let _ = connection.close(server.timeout_ms).await;
        }
        Ok(self.snapshot())
    }

    async fn fail_closed_worker_after_commit_failure(&mut self, server: &McpServerSnapshot) {
        if let Some(worker) = self.workers.remove(&server.server_id)
            && let Some(connection) = worker.connection
        {
            let _ = connection.close(server.timeout_ms).await;
        }
        self.snapshot.active_workers = u16::try_from(self.workers.len()).unwrap_or(u16::MAX);
        if let Some(target) = self
            .snapshot
            .servers
            .iter_mut()
            .find(|candidate| candidate.server_id == server.server_id)
        {
            target.lifecycle = McpLifecycle::Failed;
            target.active_requests = 0;
            target.last_failure_code = Some("persistence_commit_failed".into());
            target.last_failure_detail =
                Some("MCP worker stopped after a persistence failure".into());
        }
    }

    pub async fn recover_server(
        &mut self,
        input: &McpServerMutationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        let server = self.server(&input.server_id)?.clone();
        if !has_current_trust_binding(&server)? {
            return Err(if server.trust == McpTrustState::Revoked {
                McpError::Revoked
            } else {
                McpError::Untrusted
            });
        }
        if server.lifecycle != McpLifecycle::Failed
            || server
                .next_restart_at_ms
                .is_some_and(|restart_at| now_ms < restart_at)
        {
            return Err(McpError::InvalidState);
        }
        self.commit_lifecycle(
            &server.server_id,
            McpLifecycle::Restarting,
            "lifecycle.restarting",
            now_ms,
        )?;
        let expected_generation = self.snapshot.generation;
        self.connect_server(
            &McpServerMutationInput {
                expected_generation,
                server_id: server.server_id,
            },
            &[McpLifecycle::Restarting],
            now_ms,
        )
        .await
    }

    /// Invalidates every definition bound to one opaque vault reference after
    /// the vault durably replaces or removes it. Workers are dropped without a
    /// graceful HTTP DELETE so a revoked bearer is never transmitted again.
    pub async fn invalidate_credential_reference(
        &mut self,
        credential_reference: &str,
        reason: &str,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        validate_identifier(credential_reference)?;
        validate_text(reason)?;
        let affected = self
            .snapshot
            .servers
            .iter()
            .filter(|server| server_references_credential(server, credential_reference))
            .cloned()
            .collect::<Vec<_>>();
        if affected.is_empty() {
            return Ok(self.snapshot());
        }

        for server in &affected {
            // Invalidate Action Gateway approvals/authorizations before the
            // worker or local projection can be reused.
            self.authority.revoke_server(server, reason, now_ms)?;
            self.workers.remove(&server.server_id);
        }
        self.snapshot.active_workers = u16::try_from(self.workers.len()).unwrap_or(u16::MAX);

        for server in affected {
            let mut replacement = self.snapshot.clone();
            let target = server_mut(&mut replacement, &server.server_id)?;
            target.lifecycle_generation = target
                .lifecycle_generation
                .checked_add(1)
                .ok_or(McpError::InvalidState)?;
            target.lifecycle = McpLifecycle::Failed;
            target.active_requests = 0;
            target.next_restart_at_ms = None;
            target.last_failure_code = Some(reason.to_owned());
            target.last_failure_detail =
                Some("An MCP credential changed; review and restart the server".into());
            let lifecycle_generation = target.lifecycle_generation;
            replacement.active_workers = u16::try_from(self.workers.len()).unwrap_or(u16::MAX);
            if let Err(error) = self.commit(
                replacement,
                &server.server_id,
                lifecycle_generation,
                "credential.invalidated",
                None,
                "revoked",
                None,
                now_ms,
            ) {
                if let Some(target) = self
                    .snapshot
                    .servers
                    .iter_mut()
                    .find(|candidate| candidate.server_id == server.server_id)
                {
                    target.lifecycle = McpLifecycle::Failed;
                    target.active_requests = 0;
                    target.last_failure_code = Some("persistence_commit_failed".into());
                    target.last_failure_detail =
                        Some("MCP worker stopped after a persistence failure".into());
                }
                return Err(error);
            }
        }
        Ok(self.snapshot())
    }

    pub async fn refresh_notifications(
        &mut self,
        input: &McpServerMutationInput,
        now_ms: u64,
    ) -> Result<McpServiceSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        let server = self.server(&input.server_id)?.clone();
        let worker = self
            .workers
            .get(&server.server_id)
            .ok_or(McpError::NotReady)?;
        let connection = worker.connection.as_ref().ok_or(McpError::NotReady)?;
        let current_epoch = connection.notification_epoch();
        if current_epoch <= worker.observed_notification_epoch {
            return Ok(self.snapshot());
        }
        let tools = if server.capabilities.tools {
            connection.list_tools(server.timeout_ms).await?
        } else {
            Vec::new()
        };
        let resources = if server.capabilities.resources {
            connection.list_resources(server.timeout_ms).await?
        } else {
            Vec::new()
        };
        let tools = self.redact_discovered_tools(&server, tools)?;
        let mut replacement = self.snapshot.clone();
        let target = server_mut(&mut replacement, &server.server_id)?;
        target.tools = tools;
        target.resources = resources;
        let lifecycle_generation = target.lifecycle_generation;
        self.commit(
            replacement,
            &server.server_id,
            lifecycle_generation,
            "capabilities.refreshed",
            None,
            "succeeded",
            None,
            now_ms,
        )?;
        if let Some(worker) = self.workers.get_mut(&server.server_id) {
            worker.observed_notification_epoch = current_epoch;
        }
        Ok(self.snapshot())
    }

    pub async fn call_tool(
        &mut self,
        input: &McpToolCallInput,
        cancellation: McpCancellation,
        now_ms: u64,
    ) -> Result<McpInvocationSnapshot, McpError> {
        validate_tool_call(input)?;
        self.require_generation(input.expected_generation)?;
        let server = self.preflight_execution(
            &input.server_id,
            input.lifecycle_generation,
            &input.workspace_id,
            &input.project_id,
            &input.session_id,
        )?;
        let advertised = server
            .tools
            .iter()
            .find(|tool| tool.name == input.tool_name)
            .cloned()
            .ok_or(McpError::Denied)?;
        validate_mcp_input_schema(&advertised.input_schema, Some(&input.arguments))?;
        let request = McpAuthorityRequest::Tool {
            server: &server,
            input,
            advertised: &advertised,
        };
        let lease = match self.authority.begin_authorized_effect(&request, now_ms) {
            Ok(lease) => lease,
            Err(error) => {
                self.record_denial(&server, &input.tool_name, &error, now_ms)?;
                return Err(error);
            }
        };
        if let Err(error) = self.mark_executing(&server, &input.tool_name, now_ms) {
            let _ = self.authority.fail_authorized_effect(
                lease,
                McpAuthorityEffectStatus::Failed,
                now_ms,
            );
            return Err(error);
        }
        let raw = self
            .workers
            .get(&server.server_id)
            .ok_or(McpError::NotReady)?
            .connection
            .as_ref()
            .ok_or(McpError::NotReady)?
            .call_tool(
                &input.tool_name,
                input.arguments.clone(),
                server.timeout_ms,
                server.max_output_bytes,
                cancellation,
            )
            .await;
        self.finish_invocation(&server, request, Some(lease), &input.tool_name, raw, now_ms)
            .await
    }

    /// Executes one MCP tool under an authorization already consumed by the
    /// production runtime broker. The permit is an unforgeable core value; the
    /// method revalidates its exact turn/tool binding against current trust and
    /// lifecycle truth but deliberately does not enter ActionGateway again.
    pub(crate) fn begin_tool_pre_authorized(
        &mut self,
        permit: &ExecutionPermit,
        now_ms: u64,
    ) -> Result<McpPreparedToolInvocation, McpError> {
        let action = permit.action();
        if action.tool != "c4os_propose_action" {
            return Err(McpError::Denied);
        }
        let broker = action
            .arguments
            .get("broker")
            .and_then(Value::as_object)
            .ok_or(McpError::Denied)?;
        if broker.get("operation").and_then(Value::as_str) != Some("mcp.call-tool")
            || broker.get("target").and_then(Value::as_str)
                != Some(action.canonical_target.as_str())
        {
            return Err(McpError::Denied);
        }
        let arguments = broker
            .get("arguments")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or(McpError::Denied)?;
        let resolved = action
            .arguments
            .get("resolved")
            .and_then(Value::as_object)
            .ok_or(McpError::Denied)?;
        let server_id = resolved_text(resolved, "serverId")?;
        let tool_name = resolved_text(resolved, "toolName")?;
        let project_id = resolved_text(resolved, "projectId")?;
        let turn_id = resolved_text(resolved, "turnId")?;
        let definition_binding = resolved_text(resolved, "definitionSha256")?;
        let schema_binding = resolved_text(resolved, "inputSchemaSha256")?;
        let lifecycle_generation = resolved
            .get("lifecycleGeneration")
            .and_then(Value::as_u64)
            .filter(|generation| *generation > 0)
            .ok_or(McpError::Denied)?;
        let input = McpToolCallInput {
            expected_generation: self.snapshot.generation,
            server_id: server_id.to_owned(),
            lifecycle_generation,
            tool_name: tool_name.to_owned(),
            arguments,
            run_id: action.run_id.clone(),
            workspace_id: action.workspace_id.clone(),
            project_id: project_id.to_owned(),
            session_id: action.session_id.clone(),
            turn_id: turn_id.to_owned(),
        };
        validate_tool_call(&input)?;
        let server = self.preflight_execution(
            &input.server_id,
            lifecycle_generation,
            &input.workspace_id,
            &input.project_id,
            &input.session_id,
        )?;
        if server_definition_sha256(&server)? != definition_binding {
            return Err(McpError::Conflict);
        }
        let advertised = server
            .tools
            .iter()
            .find(|tool| tool.name == input.tool_name)
            .cloned()
            .ok_or(McpError::Denied)?;
        validate_mcp_input_schema(&advertised.input_schema, Some(&input.arguments))?;
        let authority_id = mcp_authority_identity(&server.server_id, &server.source)?;
        if advertised.input_schema_sha256 != schema_binding
            || action.target_version
                != sha256_value(&json!({
                    "definitionSha256": definition_binding,
                    "lifecycleGeneration": lifecycle_generation,
                    "toolName": tool_name,
                    "inputSchemaSha256": schema_binding,
                    "outputSchemaSha256": advertised.output_schema_sha256,
                }))?
            || action.plugin_or_mcp_id.as_deref() != Some(authority_id.as_str())
        {
            return Err(McpError::Conflict);
        }
        let connection = self
            .workers
            .get_mut(&server.server_id)
            .ok_or(McpError::NotReady)?
            .connection
            .take()
            .ok_or(McpError::NotReady)?;
        if let Err(error) = self.mark_executing(&server, &input.tool_name, now_ms) {
            if let Some(worker) = self.workers.get_mut(&server.server_id) {
                worker.connection = Some(connection);
            }
            return Err(error);
        }
        Ok(McpPreparedToolInvocation {
            server,
            input,
            advertised,
            connection,
            started_event_id: self.snapshot.last_event_id,
        })
    }

    pub(crate) async fn finish_tool_pre_authorized(
        &mut self,
        prepared: McpPreparedToolInvocation,
        raw: Result<McpRawResult, McpError>,
        now_ms: u64,
    ) -> Result<McpInvocationSnapshot, McpError> {
        let McpPreparedToolInvocation {
            server,
            input,
            advertised,
            connection,
            started_event_id,
        } = prepared;
        let may_finalize = self
            .snapshot
            .servers
            .iter()
            .find(|candidate| candidate.server_id == server.server_id)
            .is_some_and(|candidate| {
                candidate.lifecycle == McpLifecycle::Executing
                    && candidate.lifecycle_generation == server.lifecycle_generation
                    && candidate.active_requests > 0
                    && candidate.trust == McpTrustState::Trusted
            })
            && self
                .workers
                .get(&server.server_id)
                .is_some_and(|worker| worker.connection.is_none());
        if !may_finalize {
            let _ = connection.close(server.timeout_ms).await;
            let completed_at_ms = wall_clock_ms().max(now_ms);
            let redacted = safe_error_value("cancelled_by_lifecycle_change");
            return Ok(McpInvocationSnapshot {
                operation_id: operation_id(&server.server_id, &input.tool_name, started_event_id),
                server_id: server.server_id,
                lifecycle_generation: server.lifecycle_generation,
                target: input.tool_name,
                status: McpInvocationStatus::Cancelled,
                is_error: true,
                output_sha256: sha256_value(&redacted)?,
                redacted_content: redacted,
                output_bytes: 0,
                truncated: false,
                started_at_ms: now_ms,
                completed_at_ms,
                audit_event_id: self.snapshot.last_event_id,
            });
        }
        self.workers
            .get_mut(&server.server_id)
            .ok_or(McpError::NotReady)?
            .connection = Some(connection);
        let request = McpAuthorityRequest::Tool {
            server: &server,
            input: &input,
            advertised: &advertised,
        };
        self.finish_invocation(&server, request, None, &input.tool_name, raw, now_ms)
            .await
    }

    pub async fn read_resource(
        &mut self,
        input: &McpResourceReadInput,
        cancellation: McpCancellation,
        now_ms: u64,
    ) -> Result<McpInvocationSnapshot, McpError> {
        validate_resource_read(input)?;
        self.require_generation(input.expected_generation)?;
        let server = self.preflight_execution(
            &input.server_id,
            input.lifecycle_generation,
            &input.workspace_id,
            &input.project_id,
            &input.session_id,
        )?;
        let advertised = server
            .resources
            .iter()
            .find(|resource| resource.uri == input.uri)
            .cloned()
            .ok_or(McpError::Denied)?;
        let request = McpAuthorityRequest::Resource {
            server: &server,
            input,
            advertised: &advertised,
        };
        let lease = match self.authority.begin_authorized_effect(&request, now_ms) {
            Ok(lease) => lease,
            Err(error) => {
                self.record_denial(&server, &input.uri, &error, now_ms)?;
                return Err(error);
            }
        };
        if let Err(error) = self.mark_executing(&server, &input.uri, now_ms) {
            let _ = self.authority.fail_authorized_effect(
                lease,
                McpAuthorityEffectStatus::Failed,
                now_ms,
            );
            return Err(error);
        }
        let raw = self
            .workers
            .get(&server.server_id)
            .ok_or(McpError::NotReady)?
            .connection
            .as_ref()
            .ok_or(McpError::NotReady)?
            .read_resource(
                &input.uri,
                server.timeout_ms,
                server.max_output_bytes,
                cancellation,
            )
            .await;
        self.finish_invocation(&server, request, Some(lease), &input.uri, raw, now_ms)
            .await
    }

    async fn finish_invocation(
        &mut self,
        server: &McpServerSnapshot,
        request: McpAuthorityRequest<'_>,
        lease: Option<A::EffectLease>,
        target: &str,
        raw: Result<McpRawResult, McpError>,
        started_at_ms: u64,
    ) -> Result<McpInvocationSnapshot, McpError> {
        let completed_at_ms = wall_clock_ms().max(started_at_ms);
        let is_tool = matches!(&request, McpAuthorityRequest::Tool { .. });
        let raw = match (raw, &request) {
            (Ok(raw), McpAuthorityRequest::Tool { advertised, .. })
                if !raw.is_error && advertised.output_schema.is_some() =>
            {
                let schema = advertised
                    .output_schema
                    .as_ref()
                    .ok_or(McpError::InvalidState)?;
                match raw
                    .value
                    .get("structuredContent")
                    .ok_or(McpError::InvalidState)
                    .and_then(|structured| validate_mcp_input_schema(schema, Some(structured)))
                {
                    Ok(()) => Ok(raw),
                    Err(_) => Err(McpError::Transport("output_schema_mismatch".into())),
                }
            }
            (raw, _) => raw,
        };
        let (
            mut status,
            mut is_error,
            mut redacted,
            mut output_bytes,
            mut terminal_failure,
            complete,
        ) = match raw {
            Ok(raw) => match self.authority.redact_result(&request, raw.value) {
                Ok(redacted) => {
                    let bytes =
                        serde_json::to_vec(&redacted).map_err(|_| McpError::InvalidState)?;
                    let redacted_bytes =
                        u64::try_from(bytes.len()).map_err(|_| McpError::BoundExceeded)?;
                    if redacted_bytes > server.max_output_bytes {
                        (
                            McpInvocationStatus::OutputLimitExceeded,
                            true,
                            safe_error_value("output_limit_exceeded"),
                            0,
                            false,
                            false,
                        )
                    } else {
                        (
                            if raw.is_error {
                                McpInvocationStatus::Failed
                            } else {
                                McpInvocationStatus::Succeeded
                            },
                            raw.is_error,
                            redacted,
                            raw.output_bytes,
                            false,
                            true,
                        )
                    }
                }
                Err(_) => (
                    McpInvocationStatus::Failed,
                    true,
                    safe_error_value("redaction_failed"),
                    0,
                    true,
                    false,
                ),
            },
            Err(McpError::TimedOut) => (
                McpInvocationStatus::TimedOut,
                true,
                safe_error_value("timed_out"),
                0,
                true,
                false,
            ),
            Err(McpError::Cancelled) => (
                McpInvocationStatus::Cancelled,
                true,
                safe_error_value("cancelled"),
                0,
                true,
                false,
            ),
            Err(McpError::BoundExceeded) => (
                McpInvocationStatus::OutputLimitExceeded,
                true,
                safe_error_value("output_limit_exceeded"),
                0,
                false,
                false,
            ),
            Err(McpError::Conflict) => (
                McpInvocationStatus::Stale,
                true,
                safe_error_value("stale"),
                0,
                false,
                false,
            ),
            Err(error) => (
                McpInvocationStatus::Failed,
                true,
                safe_error_value(error_code(&error)),
                0,
                true,
                false,
            ),
        };
        let authority_status = authority_effect_status(status);
        let authority_terminal = match lease {
            Some(lease) if complete => self.authority.complete_authorized_effect(
                lease,
                authority_status,
                &redacted,
                completed_at_ms,
            ),
            Some(lease) => {
                self.authority
                    .fail_authorized_effect(lease, authority_status, completed_at_ms)
            }
            None => Ok(()),
        };
        if authority_terminal.is_err() {
            status = McpInvocationStatus::Failed;
            is_error = true;
            redacted = safe_error_value("authority_finalization_failed");
            output_bytes = 0;
            terminal_failure = true;
        }
        let output_sha256 = sha256_value(&redacted)?;
        let operation_id = operation_id(&server.server_id, target, self.snapshot.last_event_id + 1);
        let mut replacement = self.snapshot.clone();
        let lifecycle_generation = {
            let target_server = server_mut(&mut replacement, &server.server_id)?;
            target_server.active_requests = target_server.active_requests.saturating_sub(1);
            target_server.lifecycle = if terminal_failure {
                McpLifecycle::Failed
            } else {
                McpLifecycle::Ready
            };
            if terminal_failure {
                apply_failure_backoff(target_server, "transport", completed_at_ms);
            }
            target_server.lifecycle_generation
        };
        if terminal_failure {
            replacement.active_workers = u16::try_from(self.workers.len().saturating_sub(1))
                .map_err(|_| McpError::BoundExceeded)?;
        }
        let event_kind = if is_tool {
            "tool.completed"
        } else {
            "resource.completed"
        };
        if let Err(error) = self.commit(
            replacement,
            &server.server_id,
            lifecycle_generation,
            event_kind,
            Some(target),
            invocation_result(status),
            None,
            completed_at_ms,
        ) {
            // The terminal transport result cannot remain attached to an
            // Executing projection when its durable CAS failed. Stop the
            // worker and publish an explicit in-memory recovery state even
            // though the failed database transition cannot be trusted.
            self.fail_closed_worker_after_commit_failure(server).await;
            return Err(error);
        }
        if terminal_failure
            && let Some(worker) = self.workers.remove(&server.server_id)
            && let Some(connection) = worker.connection
        {
            let _ = connection.close(server.timeout_ms).await;
        }
        Ok(McpInvocationSnapshot {
            operation_id,
            server_id: server.server_id.clone(),
            lifecycle_generation,
            target: target.to_owned(),
            status,
            is_error,
            redacted_content: redacted,
            output_sha256,
            output_bytes,
            truncated: false,
            started_at_ms,
            completed_at_ms,
            audit_event_id: self.snapshot.last_event_id,
        })
    }

    fn preflight_connect(
        &self,
        input: &McpServerMutationInput,
        allowed: &[McpLifecycle],
    ) -> Result<McpServerSnapshot, McpError> {
        self.require_generation(input.expected_generation)?;
        validate_identifier(&input.server_id)?;
        let server = self.server(&input.server_id)?;
        if !has_current_trust_binding(server)? {
            return Err(if server.trust == McpTrustState::Revoked {
                McpError::Revoked
            } else {
                McpError::Untrusted
            });
        }
        if !allowed.contains(&server.lifecycle) || self.workers.contains_key(&server.server_id) {
            return Err(McpError::InvalidState);
        }
        Ok(server.clone())
    }

    fn preflight_execution(
        &self,
        server_id: &str,
        lifecycle_generation: u64,
        workspace_id: &str,
        project_id: &str,
        session_id: &str,
    ) -> Result<McpServerSnapshot, McpError> {
        validate_identifier(server_id)?;
        validate_identifier(workspace_id)?;
        validate_identifier(project_id)?;
        validate_identifier(session_id)?;
        let server = self.server(server_id)?;
        if !has_current_trust_binding(server)? {
            return Err(if server.trust == McpTrustState::Revoked {
                McpError::Revoked
            } else {
                McpError::Untrusted
            });
        }
        if server.lifecycle_generation != lifecycle_generation {
            return Err(McpError::Conflict);
        }
        if server.lifecycle != McpLifecycle::Ready || !self.workers.contains_key(server_id) {
            return Err(McpError::NotReady);
        }
        scope_matches(server, workspace_id, project_id, session_id)?;
        Ok(server.clone())
    }

    async fn connect_and_discover(
        &self,
        server: &McpServerSnapshot,
    ) -> Result<
        (
            Box<dyn McpConnection>,
            super::transport::McpHandshakeSnapshot,
            Vec<McpToolSnapshot>,
            Vec<McpResourceSnapshot>,
            u64,
        ),
        McpError,
    > {
        let launch = self.authority.resolve_launch(server)?;
        let connection = self
            .factory
            .connect(&server.transport, launch, server.timeout_ms)
            .await?;
        let handshake = connection.handshake().clone();
        if handshake.protocol_version != super::MCP_PROTOCOL_VERSION {
            let _ = connection.close(server.timeout_ms).await;
            return Err(McpError::UnsupportedProtocol);
        }
        // Sample before discovery so a list-change notification racing either
        // list remains pending for the next turn refresh rather than being
        // incorrectly marked observed with a stale catalog.
        let notification_epoch = connection.notification_epoch();
        let tools = if handshake.capabilities.tools {
            connection.list_tools(server.timeout_ms).await
        } else {
            Ok(Vec::new())
        };
        let tools = match tools {
            Ok(tools) => tools,
            Err(error) => {
                let _ = connection.close(server.timeout_ms).await;
                return Err(error);
            }
        };
        let tools = match self.redact_discovered_tools(server, tools) {
            Ok(tools) => tools,
            Err(error) => {
                let _ = connection.close(server.timeout_ms).await;
                return Err(error);
            }
        };
        let resources = if handshake.capabilities.resources {
            connection.list_resources(server.timeout_ms).await
        } else {
            Ok(Vec::new())
        };
        let resources = match resources {
            Ok(resources) => resources,
            Err(error) => {
                let _ = connection.close(server.timeout_ms).await;
                return Err(error);
            }
        };
        Ok((connection, handshake, tools, resources, notification_epoch))
    }

    fn redact_discovered_tools(
        &self,
        server: &McpServerSnapshot,
        tools: Vec<McpToolSnapshot>,
    ) -> Result<Vec<McpToolSnapshot>, McpError> {
        let mut redacted_tools = Vec::with_capacity(tools.len());
        for mut advertised in tools {
            let input = McpToolCallInput {
                expected_generation: self.snapshot.generation,
                server_id: server.server_id.clone(),
                lifecycle_generation: server.lifecycle_generation,
                tool_name: advertised.name.clone(),
                arguments: json!({}),
                run_id: "mcp-discovery".into(),
                workspace_id: "mcp-discovery".into(),
                project_id: "mcp-discovery".into(),
                session_id: "mcp-discovery".into(),
                turn_id: "mcp-discovery".into(),
            };
            let request = McpAuthorityRequest::Tool {
                server,
                input: &input,
                advertised: &advertised,
            };
            let redacted = self.authority.redact_result(
                &request,
                json!({
                    "title": advertised.title.clone(),
                    "description": advertised.description.clone(),
                    "inputSchema": advertised.input_schema.clone(),
                    "outputSchema": advertised.output_schema.clone(),
                }),
            )?;
            let object = redacted.as_object().ok_or(McpError::InvalidState)?;
            advertised.title = object
                .get("title")
                .and_then(Value::as_str)
                .map(str::to_owned);
            advertised.description = object
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_owned);
            advertised.input_schema = object
                .get("inputSchema")
                .filter(|value| value.is_object())
                .cloned()
                .ok_or(McpError::InvalidState)?;
            advertised.input_schema_sha256 = sha256_value(&advertised.input_schema)?;
            advertised.output_schema = match object.get("outputSchema") {
                Some(Value::Object(schema)) => Some(Value::Object(schema.clone())),
                Some(Value::Null) | None => None,
                Some(_) => return Err(McpError::InvalidState),
            };
            advertised.output_schema_sha256 = advertised
                .output_schema
                .as_ref()
                .map(sha256_value)
                .transpose()?;
            redacted_tools.push(advertised);
        }
        Ok(redacted_tools)
    }

    fn mark_executing(
        &mut self,
        server: &McpServerSnapshot,
        target: &str,
        now_ms: u64,
    ) -> Result<(), McpError> {
        let mut replacement = self.snapshot.clone();
        let target_server = server_mut(&mut replacement, &server.server_id)?;
        target_server.lifecycle = McpLifecycle::Executing;
        target_server.active_requests = target_server
            .active_requests
            .checked_add(1)
            .ok_or(McpError::BoundExceeded)?;
        let lifecycle_generation = target_server.lifecycle_generation;
        self.commit(
            replacement,
            &server.server_id,
            lifecycle_generation,
            "operation.started",
            Some(target),
            "started",
            None,
            now_ms,
        )
    }

    fn record_denial(
        &mut self,
        server: &McpServerSnapshot,
        target: &str,
        error: &McpError,
        now_ms: u64,
    ) -> Result<(), McpError> {
        self.commit(
            self.snapshot.clone(),
            &server.server_id,
            server.lifecycle_generation,
            "operation.denied",
            Some(target),
            "denied",
            Some(error_code(error)),
            now_ms,
        )
    }

    fn commit_lifecycle(
        &mut self,
        server_id: &str,
        lifecycle: McpLifecycle,
        event_kind: &str,
        now_ms: u64,
    ) -> Result<(), McpError> {
        let mut replacement = self.snapshot.clone();
        let server = server_mut(&mut replacement, server_id)?;
        server.lifecycle = lifecycle;
        let lifecycle_generation = server.lifecycle_generation;
        self.commit(
            replacement,
            server_id,
            lifecycle_generation,
            event_kind,
            None,
            "started",
            None,
            now_ms,
        )
    }

    fn record_failure(
        &mut self,
        server_id: &str,
        error: &McpError,
        now_ms: u64,
    ) -> Result<(), McpError> {
        let mut replacement = self.snapshot.clone();
        let server = server_mut(&mut replacement, server_id)?;
        server.lifecycle = McpLifecycle::Failed;
        server.active_requests = 0;
        let code = error_code(error);
        apply_failure_backoff(server, code, now_ms);
        let lifecycle_generation = server.lifecycle_generation;
        replacement.active_workers =
            u16::try_from(self.workers.len()).map_err(|_| McpError::BoundExceeded)?;
        self.commit(
            replacement,
            server_id,
            lifecycle_generation,
            "lifecycle.failed",
            None,
            "failed",
            Some(code),
            now_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn commit(
        &mut self,
        mut replacement: McpServiceSnapshot,
        server_id: &str,
        lifecycle_generation: u64,
        event_kind: &str,
        target: Option<&str>,
        result: &str,
        detail: Option<&str>,
        now_ms: u64,
    ) -> Result<(), McpError> {
        let expected_generation = self.snapshot.generation;
        replacement.generation = expected_generation
            .checked_add(1)
            .ok_or(McpError::InvalidState)?;
        replacement.last_event_id = self
            .snapshot
            .last_event_id
            .checked_add(1)
            .ok_or(McpError::InvalidState)?;
        if let Some(server) = replacement
            .servers
            .iter_mut()
            .find(|server| server.server_id == server_id)
        {
            server.last_event_id = replacement.last_event_id;
        }
        replacement.validate()?;
        let event = McpAuditEvent {
            schema_version: MCP_STATE_SCHEMA_VERSION,
            event_id: replacement.last_event_id,
            generation: replacement.generation,
            lifecycle_generation,
            operation_id: operation_id(
                server_id,
                target.unwrap_or(event_kind),
                replacement.last_event_id,
            ),
            server_id: server_id.to_owned(),
            event_kind: event_kind.to_owned(),
            target: target.map(str::to_owned),
            result: result.to_owned(),
            detail: detail.map(str::to_owned),
            occurred_at_ms: now_ms,
        };
        self.repository
            .compare_and_swap(Some(expected_generation), &replacement, &event)?;
        self.snapshot = replacement;
        Ok(())
    }

    fn require_generation(&self, expected: u64) -> Result<(), McpError> {
        if self.snapshot.generation != expected {
            return Err(McpError::Conflict);
        }
        Ok(())
    }

    fn server(&self, server_id: &str) -> Result<&McpServerSnapshot, McpError> {
        self.snapshot
            .servers
            .iter()
            .find(|server| server.server_id == server_id)
            .ok_or(McpError::InvalidInput)
    }
}

fn normalize_after_restart(
    mut snapshot: McpServiceSnapshot,
) -> Result<McpServiceSnapshot, McpError> {
    snapshot.active_workers = 0;
    for server in &mut snapshot.servers {
        server.active_requests = 0;
        let current_definition_sha256 = server_definition_sha256(server)?;
        if server.trust == McpTrustState::Trusted
            && server.trusted_definition_sha256.as_deref()
                != Some(current_definition_sha256.as_str())
        {
            server.trust = McpTrustState::Pending;
            server.trusted_definition_sha256 = None;
            server.pending_trust_approval = None;
            server.lifecycle = McpLifecycle::Disabled;
            server.lifecycle_generation = server
                .lifecycle_generation
                .checked_add(1)
                .ok_or(McpError::InvalidState)?;
            server.protocol_version = None;
            server.server_name = None;
            server.server_version = None;
            server.instructions_present = false;
            server.capabilities = McpCapabilitySnapshot::default();
            server.tools.clear();
            server.resources.clear();
            server.restart_attempts = 0;
            server.next_restart_at_ms = None;
            server.last_failure_code = Some("trust_binding_required".into());
            server.last_failure_detail =
                Some("The persisted definition requires a new trust review".into());
        } else if server.trust != McpTrustState::Trusted {
            server.trusted_definition_sha256 = None;
        }
        if let Some(approval) = server.pending_trust_approval.as_mut() {
            approval.state = McpTrustApprovalState::Interrupted;
        }
        if matches!(
            server.lifecycle,
            McpLifecycle::Testing
                | McpLifecycle::Connecting
                | McpLifecycle::Ready
                | McpLifecycle::Executing
                | McpLifecycle::Restarting
        ) {
            server.lifecycle = McpLifecycle::Failed;
            server.last_failure_code = Some("restart_recovery_required".into());
            server.last_failure_detail =
                Some("MCP worker did not survive application restart".into());
            server.restart_attempts = server.restart_attempts.saturating_add(1);
            server.next_restart_at_ms = None;
        }
    }
    Ok(snapshot)
}

fn has_current_trust_binding(server: &McpServerSnapshot) -> Result<bool, McpError> {
    let current_definition_sha256 = server_definition_sha256(server)?;
    Ok(server.trust == McpTrustState::Trusted
        && server.trusted_definition_sha256.as_deref() == Some(current_definition_sha256.as_str()))
}

fn server_mut<'a>(
    snapshot: &'a mut McpServiceSnapshot,
    server_id: &str,
) -> Result<&'a mut McpServerSnapshot, McpError> {
    snapshot
        .servers
        .iter_mut()
        .find(|server| server.server_id == server_id)
        .ok_or(McpError::InvalidInput)
}

fn apply_discovery(
    server: &mut McpServerSnapshot,
    handshake: &super::transport::McpHandshakeSnapshot,
    tools: Vec<McpToolSnapshot>,
    resources: Vec<McpResourceSnapshot>,
    now_ms: u64,
) {
    server.protocol_version = Some(handshake.protocol_version.clone());
    server.server_name = Some(handshake.server_name.clone());
    server.server_version = Some(handshake.server_version.clone());
    server.instructions_present = handshake.instructions_present;
    server.capabilities = handshake.capabilities.clone();
    server.tools = tools;
    server.resources = resources;
    server.restart_attempts = 0;
    server.next_restart_at_ms = None;
    server.last_connected_at_ms = Some(now_ms);
    server.last_failure_code = None;
    server.last_failure_detail = None;
}

fn apply_failure_backoff(server: &mut McpServerSnapshot, code: &str, now_ms: u64) {
    server.restart_attempts = server.restart_attempts.saturating_add(1).min(16);
    let exponent = u32::from(server.restart_attempts.saturating_sub(1).min(10));
    let delay_ms = 1_000_u64
        .saturating_mul(2_u64.saturating_pow(exponent))
        .min(300_000);
    server.next_restart_at_ms = now_ms.checked_add(delay_ms);
    server.last_failure_code = Some(code.to_owned());
    server.last_failure_detail = Some("MCP worker failed; bounded recovery is available".into());
}

fn validate_tool_call(input: &McpToolCallInput) -> Result<(), McpError> {
    validate_identifier(&input.server_id)?;
    validate_mcp_name(&input.tool_name)?;
    validate_identifier(&input.run_id)?;
    validate_identifier(&input.workspace_id)?;
    validate_identifier(&input.project_id)?;
    validate_identifier(&input.session_id)?;
    validate_identifier(&input.turn_id)?;
    if !input.arguments.is_object() || input.lifecycle_generation == 0 {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

fn validate_resource_read(input: &McpResourceReadInput) -> Result<(), McpError> {
    validate_identifier(&input.server_id)?;
    validate_text(&input.uri)?;
    validate_identifier(&input.run_id)?;
    validate_identifier(&input.workspace_id)?;
    validate_identifier(&input.project_id)?;
    validate_identifier(&input.session_id)?;
    validate_identifier(&input.turn_id)?;
    if input.lifecycle_generation == 0 {
        return Err(McpError::InvalidInput);
    }
    Ok(())
}

fn scope_matches(
    server: &McpServerSnapshot,
    workspace_id: &str,
    project_id: &str,
    session_id: &str,
) -> Result<(), McpError> {
    let matches = match &server.scope {
        super::McpScope::Application => true,
        super::McpScope::Workspace {
            workspace_id: expected,
        } => expected == workspace_id,
        super::McpScope::Project {
            workspace_id: expected_workspace,
            project_id: expected_project,
        } => expected_workspace == workspace_id && expected_project == project_id,
        super::McpScope::Chat {
            workspace_id: expected_workspace,
            project_id: expected_project,
            session_id: expected_session,
        } => {
            expected_workspace == workspace_id
                && expected_project == project_id
                && expected_session == session_id
        }
    };
    if matches {
        Ok(())
    } else {
        Err(McpError::Denied)
    }
}

fn server_references_credential(server: &McpServerSnapshot, reference: &str) -> bool {
    match &server.transport {
        super::McpTransportDefinition::Stdio { environment, .. } => {
            environment.iter().any(|binding| {
                matches!(
                    &binding.source,
                    super::McpEnvironmentSource::Secret {
                        reference: super::McpSecretReference::Vault {
                            credential_reference,
                        },
                    } if credential_reference == reference
                )
            })
        }
        super::McpTransportDefinition::StreamableHttp {
            bearer, headers, ..
        } => {
            bearer.as_ref().is_some_and(|secret| {
                matches!(
                    secret,
                    super::McpSecretReference::Vault {
                        credential_reference,
                    } if credential_reference == reference
                )
            }) || headers.iter().any(|header| {
                matches!(
                    &header.source,
                    super::McpHeaderSource::Secret {
                        reference: super::McpSecretReference::Vault {
                            credential_reference,
                        },
                    } if credential_reference == reference
                )
            })
        }
    }
}

fn safe_error_value(code: &str) -> Value {
    json!({ "error": { "code": code } })
}

fn definition_sha256(server: &McpServerSnapshot) -> Result<String, McpError> {
    server_definition_sha256(server)
}

fn sha256_value(value: &Value) -> Result<String, McpError> {
    let bytes = serde_json::to_vec(value).map_err(|_| McpError::InvalidState)?;
    Ok(format!("sha256:{}", hex_digest(&Sha256::digest(bytes))))
}

fn resolved_text<'a>(
    resolved: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str, McpError> {
    resolved
        .get(key)
        .and_then(Value::as_str)
        .ok_or(McpError::Denied)
}

fn operation_id(server_id: &str, target: &str, event_id: u64) -> String {
    let mut digest = Sha256::new();
    digest.update(server_id.as_bytes());
    digest.update([0]);
    digest.update(target.as_bytes());
    digest.update([0]);
    digest.update(event_id.to_be_bytes());
    format!("mcp-{}", hex_digest(&digest.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn error_code(error: &McpError) -> &'static str {
    match error {
        McpError::InvalidInput => "invalid_input",
        McpError::InvalidState => "invalid_state",
        McpError::BoundExceeded => "bound_exceeded",
        McpError::Conflict => "conflict",
        McpError::Untrusted => "untrusted",
        McpError::Revoked => "revoked",
        McpError::Disabled => "disabled",
        McpError::NotReady => "not_ready",
        McpError::UnsupportedProtocol => "unsupported_protocol",
        McpError::Transport(_) => "transport",
        McpError::TimedOut => "timed_out",
        McpError::Cancelled => "cancelled",
        McpError::Denied => "denied",
        McpError::Persistence(_) => "persistence",
        McpError::Credential => "credential",
        McpError::StateUnavailable => "state_unavailable",
    }
}

fn invocation_result(status: McpInvocationStatus) -> &'static str {
    match status {
        McpInvocationStatus::Succeeded => "succeeded",
        McpInvocationStatus::Failed => "failed",
        McpInvocationStatus::Denied => "denied",
        McpInvocationStatus::Cancelled => "cancelled",
        McpInvocationStatus::TimedOut => "timed_out",
        McpInvocationStatus::OutputLimitExceeded => "output_limit_exceeded",
        McpInvocationStatus::Stale => "stale",
    }
}

fn authority_effect_status(status: McpInvocationStatus) -> McpAuthorityEffectStatus {
    match status {
        McpInvocationStatus::Succeeded => McpAuthorityEffectStatus::Succeeded,
        McpInvocationStatus::Cancelled => McpAuthorityEffectStatus::Cancelled,
        McpInvocationStatus::TimedOut => McpAuthorityEffectStatus::TimedOut,
        McpInvocationStatus::OutputLimitExceeded => McpAuthorityEffectStatus::OutputLimitExceeded,
        McpInvocationStatus::Failed | McpInvocationStatus::Denied | McpInvocationStatus::Stale => {
            McpAuthorityEffectStatus::Failed
        }
    }
}

fn wall_clock_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}
