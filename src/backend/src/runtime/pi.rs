use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

use crate::runtime::action_bridge::RuntimeExecutionReceipt;
use crate::runtime::adapter::{
    ADAPTER_CONTRACT_SCHEMA_VERSION, AdapterAuthority, AdapterConformanceDescriptor,
    PeerCapabilityClaims, peer_capabilities,
};
use crate::runtime::capability::CapabilityState;
use crate::runtime::supervisor::{RUNTIME_PROTOCOL_VERSION, RuntimeKind};
use crate::security::gateway::NormalizedActionStatus;

pub const PI_NATIVE_VERSION: &str = "0.80.10";
pub const PI_NATIVE_PACKAGE: &str = "@earendil-works/pi-coding-agent";
pub const PI_PROTOCOL: &str = "c4os.pi.ndjson.v1";
pub const PI_PROTOCOL_SCHEMA_VERSION: u32 = 1;
pub const PI_MAX_LINE_BYTES: usize = 256 * 1024;
pub const PI_MAX_ATTACHMENTS: usize = 32;
const MAX_VALUE_DEPTH: usize = 12;
const MAX_COLLECTION_ITEMS: usize = 512;
const MAX_STRING_BYTES: usize = 64 * 1024;
pub const PI_MAX_IMAGE_BYTES: u64 = 128 * 1024;
const MAX_IMAGE_BASE64_BYTES: usize = (PI_MAX_IMAGE_BYTES as usize).div_ceil(3) * 4;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PiAdapterError {
    #[error("Pi sidecar manifest is invalid: {0}")]
    InvalidManifest(String),
    #[error("Pi sidecar package is incompatible: {0}")]
    IncompatiblePackage(String),
    #[error("Pi sidecar protocol rejected the message: {0}")]
    Protocol(String),
    #[error("Pi sidecar runner failed: {0}")]
    Runner(String),
    #[error("Pi sidecar rejected the request with {code}: {message}")]
    Rejected { code: String, message: String },
    #[error("Pi sidecar state rejected the operation: {0}")]
    State(String),
    #[error("Pi sidecar I/O failed: {0}")]
    Io(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiSidecarManifest {
    pub schema_version: u32,
    pub adapter: String,
    pub adapter_version: String,
    pub native_package: String,
    pub native_version: String,
    pub transport: String,
    pub protocol: String,
    pub entrypoint: String,
    pub persistence: String,
    pub tools: String,
    pub extensions: String,
    pub max_line_bytes: usize,
}

impl PiSidecarManifest {
    pub fn load(sidecar_root: &Path) -> Result<Self, PiAdapterError> {
        let manifest_path = sidecar_root.join("sidecar-manifest.json");
        let bytes = fs::read(&manifest_path).map_err(|error| {
            PiAdapterError::Io(format!(
                "could not read {}: {error}",
                manifest_path.display()
            ))
        })?;
        if bytes.len() > 64 * 1024 {
            return Err(PiAdapterError::InvalidManifest(
                "manifest exceeds 64 KiB".into(),
            ));
        }
        let manifest: Self = serde_json::from_slice(&bytes)
            .map_err(|error| PiAdapterError::InvalidManifest(error.to_string()))?;
        manifest.validate()?;
        validate_sidecar_package(sidecar_root)?;
        let root = sidecar_root
            .canonicalize()
            .map_err(|error| PiAdapterError::Io(error.to_string()))?;
        let entrypoint = sidecar_root
            .join(&manifest.entrypoint)
            .canonicalize()
            .map_err(|error| PiAdapterError::Io(error.to_string()))?;
        if !entrypoint.starts_with(&root) || !entrypoint.is_file() {
            return Err(PiAdapterError::InvalidManifest(
                "entrypoint escapes the sidecar package".into(),
            ));
        }
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), PiAdapterError> {
        let exact = [
            (self.schema_version == 1, "schemaVersion must be 1"),
            (self.adapter == "pi", "adapter must be pi"),
            (
                is_version(&self.adapter_version),
                "adapterVersion must be canonical semver",
            ),
            (
                self.native_package == PI_NATIVE_PACKAGE,
                "nativePackage is not the pinned Pi package",
            ),
            (
                self.native_version == PI_NATIVE_VERSION,
                "nativeVersion is not the pinned Pi version",
            ),
            (
                self.transport == "c4os-node-sdk-sidecar",
                "transport must be the selected SDK sidecar",
            ),
            (self.protocol == PI_PROTOCOL, "protocol is unsupported"),
            (
                self.persistence == "c4os-authoritative-in-memory-worker",
                "Pi persistence must remain disabled",
            ),
            (
                self.tools == "c4os-brokered-only",
                "Pi native tools must remain disabled",
            ),
            (
                self.extensions == "disabled",
                "Pi extensions must remain disabled",
            ),
            (
                self.max_line_bytes == PI_MAX_LINE_BYTES,
                "maxLineBytes must match the Rust protocol bound",
            ),
        ];
        if let Some((_, message)) = exact.into_iter().find(|(valid, _)| !valid) {
            return Err(PiAdapterError::InvalidManifest(message.into()));
        }
        validate_relative_entrypoint(&self.entrypoint)
    }

    pub fn conformance_descriptor(
        &self,
        process_generation: u64,
        credential_channel_available: bool,
    ) -> Result<AdapterConformanceDescriptor, PiAdapterError> {
        self.validate()?;
        let descriptor = AdapterConformanceDescriptor {
            schema_version: ADAPTER_CONTRACT_SCHEMA_VERSION,
            runtime_kind: RuntimeKind::Pi,
            adapter_version: "1.0.0".into(),
            native_version: self.native_version.clone(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
            process_generation,
            authority: AdapterAuthority::C4osActionGatewayOnly,
            capabilities: peer_capabilities(PeerCapabilityClaims {
                health: CapabilityState::Supported,
                session_create: CapabilityState::Supported,
                session_resume: CapabilityState::Degraded,
                model_discovery: CapabilityState::Unsupported,
                streaming: CapabilityState::Supported,
                action_intents: CapabilityState::Supported,
                credential_channel: if credential_channel_available {
                    CapabilityState::Supported
                } else {
                    CapabilityState::Degraded
                },
                cancellation: CapabilityState::Supported,
                restart: CapabilityState::Degraded,
            }),
        };
        descriptor.validate().map_err(|_| {
            PiAdapterError::InvalidManifest("conformance descriptor is invalid".into())
        })?;
        Ok(descriptor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiLaunchSpec {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub credential_fd: Option<u32>,
}

impl PiLaunchSpec {
    pub fn new(
        node_executable: PathBuf,
        sidecar_root: &Path,
        manifest: &PiSidecarManifest,
        process_generation: u64,
        credential_fd: Option<u32>,
    ) -> Result<Self, PiAdapterError> {
        if process_generation == 0 {
            return Err(PiAdapterError::InvalidManifest(
                "process generation must be positive".into(),
            ));
        }
        manifest.validate()?;
        if !node_executable.is_absolute() {
            return Err(PiAdapterError::InvalidManifest(
                "Node executable must be an absolute supervised path".into(),
            ));
        }
        let entrypoint = sidecar_root.join(&manifest.entrypoint);
        let mut arguments = vec![
            "--use-system-ca".into(),
            entrypoint.to_string_lossy().into_owned(),
            format!("--generation={process_generation}"),
        ];
        if let Some(fd) = credential_fd {
            if fd < 3 {
                return Err(PiAdapterError::InvalidManifest(
                    "credential channel must use a dedicated inherited descriptor".into(),
                ));
            }
            arguments.push(format!("--credential-fd={fd}"));
        }
        Ok(Self {
            executable: node_executable,
            arguments,
            environment: BTreeMap::new(),
            credential_fd,
        })
    }
}

pub trait PiSidecarRunner {
    /// Sends one bounded line and returns complete response/event lines observed for the exchange.
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String>;
    /// Returns complete event lines already available without blocking.
    fn poll(&mut self) -> Result<Vec<String>, String>;
    /// Gracefully stops and then cleans up the supervised process tree.
    fn terminate(&mut self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiAdapterState {
    Created,
    Ready,
    Degraded,
    Stopped,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiHealth {
    pub status: String,
    pub runtime: String,
    pub transport: String,
    pub protocol: String,
    pub process_generation: u64,
    pub sessions: u64,
    pub stale_events_rejected: u64,
    pub capabilities: BTreeMap<String, PiCapabilityState>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiCapabilityState {
    pub state: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiVersion {
    pub adapter: String,
    pub adapter_version: String,
    pub native_package: String,
    pub native_version: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiModelRoute {
    pub provider: String,
    pub model_id: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PiModelPreflightResponse {
    available: bool,
    provider: String,
    model_id: String,
    native_version: String,
}

impl PiModelRoute {
    fn validate(&self) -> Result<(), PiAdapterError> {
        validate_id(&self.provider, "provider")?;
        validate_id(&self.model_id, "modelId")?;
        crate::runtime::provider::validate_https_endpoint(&self.base_url).map_err(|_| {
            PiAdapterError::Protocol("model baseUrl is not a bounded HTTPS URL".into())
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiDispatchAttachment {
    pub attachment_id: String,
    pub stable_reference: String,
    pub display_name: String,
    pub media_type: String,
    pub byte_length: u64,
    pub content_sha256: String,
    pub snapshot_version: u64,
    #[serde(skip)]
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiSamplingMessage {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PiSamplingRequest {
    pub messages: Vec<PiSamplingMessage>,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiSamplingResult {
    pub text: String,
    pub model: String,
    pub stop_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiSamplingPoll {
    Pending,
    Completed(PiSamplingResult),
    Cancelled,
    Failed { code: String },
}

impl PiDispatchAttachment {
    fn validate(&self) -> Result<(), PiAdapterError> {
        validate_id(&self.attachment_id, "attachmentId")?;
        validate_id(&self.stable_reference, "stableReference")?;
        if self.display_name.trim().is_empty()
            || self.display_name.len() > 512
            || self.display_name.contains('\0')
            || self.media_type.trim().is_empty()
            || self.media_type.len() > 256
            || self.media_type.contains('\0')
            || !matches!(
                self.media_type.as_str(),
                "image/jpeg" | "image/png" | "image/gif" | "image/webp"
            )
            || self.byte_length == 0
            || self.byte_length > PI_MAX_IMAGE_BYTES
            || self
                .content_sha256
                .strip_prefix("sha256:")
                .is_none_or(|digest| {
                    digest.len() != 64
                        || !digest
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
                })
            || self.snapshot_version == 0
            || self.content.len() as u64 != self.byte_length
            || prefixed_sha256(&self.content) != self.content_sha256
        {
            return Err(PiAdapterError::Protocol(
                "direct attachment metadata is invalid".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PiEventEnvelope {
    pub schema_version: u32,
    pub kind: String,
    pub event_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
    pub sequence: u64,
    pub runtime: String,
    pub workspace_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub run_id: String,
    pub category: String,
    pub native_type: String,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone)]
struct ActiveRun {
    workspace_id: String,
    turn_id: String,
    run_id: String,
    correlation_id: String,
}

#[derive(Debug, Clone)]
struct PiSessionBinding {
    workspace_id: String,
    active_run: Option<ActiveRun>,
}

pub struct PiAdapter<R: PiSidecarRunner> {
    manifest: PiSidecarManifest,
    runner: R,
    process_generation: u64,
    state: PiAdapterState,
    next_request: u64,
    last_sequence: u64,
    sessions: BTreeMap<String, PiSessionBinding>,
    events: Vec<PiEventEnvelope>,
    stale_events_rejected: u64,
}

impl<R: PiSidecarRunner> PiAdapter<R> {
    pub fn new(
        manifest: PiSidecarManifest,
        runner: R,
        process_generation: u64,
    ) -> Result<Self, PiAdapterError> {
        manifest.validate()?;
        if process_generation == 0 {
            return Err(PiAdapterError::InvalidManifest(
                "process generation must be positive".into(),
            ));
        }
        Ok(Self {
            manifest,
            runner,
            process_generation,
            state: PiAdapterState::Created,
            next_request: 1,
            last_sequence: 0,
            sessions: BTreeMap::new(),
            events: Vec::new(),
            stale_events_rejected: 0,
        })
    }

    pub fn state(&self) -> &PiAdapterState {
        &self.state
    }

    pub fn manifest(&self) -> &PiSidecarManifest {
        &self.manifest
    }

    #[doc(hidden)]
    pub fn runner_ref(&self) -> &R {
        &self.runner
    }

    #[doc(hidden)]
    pub fn runner_mut_ref(&mut self) -> &mut R {
        &mut self.runner
    }

    pub fn start(&mut self) -> Result<(PiHealth, PiVersion), PiAdapterError> {
        if self.state != PiAdapterState::Created {
            return Err(PiAdapterError::State(
                "Pi adapter may start only once per process generation".into(),
            ));
        }
        let health_value =
            self.request("health", None, None, None, None, Value::Object(Map::new()))?;
        let health: PiHealth = serde_json::from_value(health_value)
            .map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
        validate_health(&health, self.process_generation)?;
        let version_value =
            self.request("version", None, None, None, None, Value::Object(Map::new()))?;
        let version: PiVersion = serde_json::from_value(version_value)
            .map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
        validate_version(&version)?;
        self.state = if health
            .capabilities
            .values()
            .any(|capability| capability.state == "degraded")
        {
            PiAdapterState::Degraded
        } else {
            PiAdapterState::Ready
        };
        Ok((health, version))
    }

    pub fn create_session(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        route: PiModelRoute,
        eligible_tool_ids: &BTreeSet<String>,
    ) -> Result<(), PiAdapterError> {
        self.require_running()?;
        validate_id(workspace_id, "workspaceId")?;
        validate_id(session_id, "sessionId")?;
        route.validate()?;
        if eligible_tool_ids.len() > c4os_tool_names().len()
            || eligible_tool_ids
                .iter()
                .any(|tool_id| !c4os_tool_names().contains(tool_id.as_str()))
        {
            return Err(PiAdapterError::Protocol(
                "eligible Pi tools exceed the fixed C4OS broker set".into(),
            ));
        }
        if self.sessions.contains_key(session_id) {
            return Err(PiAdapterError::State("Pi session already exists".into()));
        }
        let payload = json!({
            "modelRoute": route,
            "eligibleTools": eligible_tool_ids,
        });
        let response = self.request(
            "session.create",
            Some(workspace_id),
            Some(session_id),
            None,
            None,
            payload,
        )?;
        if response.get("sessionId").and_then(Value::as_str) != Some(session_id)
            || response.get("persistence").and_then(Value::as_str) != Some("c4os-authoritative")
        {
            return Err(PiAdapterError::Protocol(
                "session creation response changed C4OS identity or persistence authority".into(),
            ));
        }
        self.sessions.insert(
            session_id.into(),
            PiSessionBinding {
                workspace_id: workspace_id.into(),
                active_run: None,
            },
        );
        Ok(())
    }

    /// Checks the pinned Pi catalog without creating a native session or publishing capability state.
    pub fn preflight_model(&mut self, route: &PiModelRoute) -> Result<bool, PiAdapterError> {
        self.require_running()?;
        route.validate()?;
        let response = self.request(
            "model.preflight",
            None,
            None,
            None,
            None,
            json!({
                "provider": route.provider.as_str(),
                "modelId": route.model_id.as_str(),
            }),
        )?;
        let response: PiModelPreflightResponse = serde_json::from_value(response)
            .map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
        if response.provider != route.provider
            || response.model_id != route.model_id
            || response.native_version != PI_NATIVE_VERSION
        {
            return Err(PiAdapterError::Protocol(
                "model preflight response changed the native provider/model identity".into(),
            ));
        }
        Ok(response.available)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_sampling_with_credential_operation(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
        request: &PiSamplingRequest,
        credential_operation: Option<(&str, &str)>,
    ) -> Result<(), PiAdapterError> {
        self.require_running()?;
        for (value, field) in [
            (workspace_id, "workspaceId"),
            (session_id, "sessionId"),
            (turn_id, "turnId"),
            (run_id, "runId"),
            (correlation_id, "correlationId"),
        ] {
            validate_id(value, field)?;
        }
        if let Some((runtime_id, provider_id)) = credential_operation {
            validate_id(runtime_id, "runtimeId")?;
            validate_id(provider_id, "providerId")?;
        }
        validate_sampling_request(request)?;
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| PiAdapterError::State("unknown Pi session".into()))?;
        if session.workspace_id != workspace_id || session.active_run.is_some() {
            return Err(PiAdapterError::State(
                "Pi session scope mismatches or already has an active run".into(),
            ));
        }
        session.active_run = Some(ActiveRun {
            workspace_id: workspace_id.into(),
            turn_id: turn_id.into(),
            run_id: run_id.into(),
            correlation_id: correlation_id.into(),
        });
        let mut payload = Map::from_iter([
            (
                "messages".into(),
                serde_json::to_value(&request.messages)
                    .map_err(|error| PiAdapterError::Protocol(error.to_string()))?,
            ),
            (
                "maxTokens".into(),
                Value::Number(serde_json::Number::from(request.max_tokens)),
            ),
        ]);
        if let Some((runtime_id, provider_id)) = credential_operation {
            payload.insert("runtimeId".into(), Value::String(runtime_id.into()));
            payload.insert("providerId".into(), Value::String(provider_id.into()));
        }
        if let Some(system_prompt) = &request.system_prompt {
            payload.insert("systemPrompt".into(), Value::String(system_prompt.clone()));
        }
        if let Some(temperature) = request.temperature {
            let temperature = serde_json::Number::from_f64(f64::from(temperature))
                .ok_or_else(|| PiAdapterError::Protocol("temperature is not finite".into()))?;
            payload.insert("temperature".into(), Value::Number(temperature));
        }
        let response = self.request_with_correlation(
            "sampling.start",
            Some(workspace_id),
            Some(session_id),
            Some(turn_id),
            Some(run_id),
            Value::Object(payload),
            correlation_id,
        );
        if matches!(&response, Err(PiAdapterError::Rejected { .. })) {
            self.sessions
                .get_mut(session_id)
                .expect("sampling session remains bound")
                .active_run = None;
        }
        let response = response?;
        let fields = response.as_object().ok_or_else(|| {
            PiAdapterError::Protocol("sampling start response is not an object".into())
        })?;
        if fields.len() != 2
            || fields.get("accepted").and_then(Value::as_bool) != Some(true)
            || fields.get("runId").and_then(Value::as_str) != Some(run_id)
        {
            return Err(PiAdapterError::Protocol(
                "sampling response did not accept the run".into(),
            ));
        }
        Ok(())
    }

    pub fn poll_sampling(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
    ) -> Result<PiSamplingPoll, PiAdapterError> {
        self.require_sampling_run(workspace_id, session_id, turn_id, run_id, correlation_id)?;
        let response = self.request_with_correlation(
            "sampling.poll",
            Some(workspace_id),
            Some(session_id),
            Some(turn_id),
            Some(run_id),
            Value::Object(Map::new()),
            correlation_id,
        )?;
        let fields = response.as_object().ok_or_else(|| {
            PiAdapterError::Protocol("sampling poll response is not an object".into())
        })?;
        let state = fields
            .get("state")
            .and_then(Value::as_str)
            .ok_or_else(|| PiAdapterError::Protocol("sampling poll state is missing".into()))?;
        let terminal = match state {
            "pending" if fields.len() == 1 => return Ok(PiSamplingPoll::Pending),
            "completed" if fields.len() == 2 => {
                let result: PiSamplingResult =
                    serde_json::from_value(fields.get("result").cloned().ok_or_else(|| {
                        PiAdapterError::Protocol("sampling result is missing".into())
                    })?)
                    .map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
                validate_sampling_result(&result)?;
                PiSamplingPoll::Completed(result)
            }
            "cancelled" if fields.len() == 1 => PiSamplingPoll::Cancelled,
            "failed" if fields.len() == 3 => {
                let code = fields.get("code").and_then(Value::as_str).ok_or_else(|| {
                    PiAdapterError::Protocol("sampling failure code is missing".into())
                })?;
                validate_id(code, "sampling failure code")?;
                PiSamplingPoll::Failed { code: code.into() }
            }
            _ => {
                return Err(PiAdapterError::Protocol(
                    "sampling poll response is invalid".into(),
                ));
            }
        };
        self.sessions
            .get_mut(session_id)
            .expect("sampling session remains bound")
            .active_run = None;
        Ok(terminal)
    }

    pub fn cancel_sampling(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
    ) -> Result<bool, PiAdapterError> {
        self.require_sampling_run(workspace_id, session_id, turn_id, run_id, correlation_id)?;
        let response = self.request_with_correlation(
            "sampling.cancel",
            Some(workspace_id),
            Some(session_id),
            Some(turn_id),
            Some(run_id),
            Value::Object(Map::new()),
            correlation_id,
        )?;
        let fields = response.as_object().ok_or_else(|| {
            PiAdapterError::Protocol("sampling cancel response is not an object".into())
        })?;
        let cancelled = fields
            .get("cancelled")
            .and_then(Value::as_bool)
            .ok_or_else(|| PiAdapterError::Protocol("sampling cancel state is missing".into()))?;
        let already_terminal = fields
            .get("alreadyTerminal")
            .and_then(Value::as_bool)
            .ok_or_else(|| PiAdapterError::Protocol("sampling terminal state is missing".into()))?;
        if fields.len() != 2 || cancelled == already_terminal {
            return Err(PiAdapterError::Protocol(
                "sampling cancel response is contradictory".into(),
            ));
        }
        Ok(cancelled)
    }

    pub fn close_session(
        &mut self,
        workspace_id: &str,
        session_id: &str,
    ) -> Result<(), PiAdapterError> {
        self.require_running()?;
        validate_id(workspace_id, "workspaceId")?;
        validate_id(session_id, "sessionId")?;
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| PiAdapterError::State("unknown Pi session".into()))?;
        if session.workspace_id != workspace_id || session.active_run.is_some() {
            return Err(PiAdapterError::State(
                "Pi session scope mismatches or still has an active run".into(),
            ));
        }
        let response = self.request(
            "session.close",
            Some(workspace_id),
            Some(session_id),
            None,
            None,
            Value::Object(Map::new()),
        )?;
        let fields = response.as_object().ok_or_else(|| {
            PiAdapterError::Protocol("session close response is not an object".into())
        })?;
        if fields.len() != 1 || fields.get("closed").and_then(Value::as_bool) != Some(true) {
            return Err(PiAdapterError::Protocol(
                "session close did not acknowledge the exact binding".into(),
            ));
        }
        self.sessions.remove(session_id);
        Ok(())
    }

    fn require_sampling_run(
        &self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
    ) -> Result<(), PiAdapterError> {
        for (value, field) in [
            (workspace_id, "workspaceId"),
            (session_id, "sessionId"),
            (turn_id, "turnId"),
            (run_id, "runId"),
            (correlation_id, "correlationId"),
        ] {
            validate_id(value, field)?;
        }
        let active = self
            .sessions
            .get(session_id)
            .and_then(|session| session.active_run.as_ref());
        if !active.is_some_and(|active| {
            active.workspace_id == workspace_id
                && active.turn_id == turn_id
                && active.run_id == run_id
                && active.correlation_id == correlation_id
        }) {
            return Err(PiAdapterError::State(
                "sampling identity is not active".into(),
            ));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
        input: &str,
    ) -> Result<(), PiAdapterError> {
        self.dispatch_with_attachments(
            workspace_id,
            session_id,
            turn_id,
            run_id,
            correlation_id,
            input,
            &[],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn validate_dispatch_with_attachments(
        &self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
        input: &str,
        attachments: &[PiDispatchAttachment],
    ) -> Result<(), PiAdapterError> {
        self.require_running()?;
        for (value, field) in [
            (workspace_id, "workspaceId"),
            (session_id, "sessionId"),
            (turn_id, "turnId"),
            (run_id, "runId"),
            (correlation_id, "correlationId"),
        ] {
            validate_id(value, field)?;
        }
        if input.len() > MAX_STRING_BYTES
            || input.is_empty() && attachments.is_empty()
            || attachments.len() > PI_MAX_ATTACHMENTS
        {
            return Err(PiAdapterError::Protocol(
                "dispatch input and attachments are empty or exceed their bounds".into(),
            ));
        }
        let mut attachment_ids = BTreeSet::new();
        let mut total_attachment_bytes = 0_u64;
        for attachment in attachments {
            attachment.validate()?;
            total_attachment_bytes = total_attachment_bytes
                .checked_add(attachment.byte_length)
                .ok_or_else(|| PiAdapterError::Protocol("attachment size overflow".into()))?;
            if !attachment_ids.insert(attachment.attachment_id.as_str()) {
                return Err(PiAdapterError::Protocol(
                    "direct attachment identifiers are duplicated".into(),
                ));
            }
        }
        if total_attachment_bytes > PI_MAX_IMAGE_BYTES {
            return Err(PiAdapterError::Protocol(
                "direct image attachments exceed the Pi wire bound".into(),
            ));
        }
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| PiAdapterError::State("unknown Pi session".into()))?;
        if session.workspace_id != workspace_id || session.active_run.is_some() {
            return Err(PiAdapterError::State(
                "Pi session scope mismatches or already has an active run".into(),
            ));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch_with_attachments(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
        input: &str,
        attachments: &[PiDispatchAttachment],
    ) -> Result<(), PiAdapterError> {
        self.dispatch_with_credential_operation(
            workspace_id,
            session_id,
            turn_id,
            run_id,
            correlation_id,
            input,
            attachments,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dispatch_with_credential_operation(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
        input: &str,
        attachments: &[PiDispatchAttachment],
        credential_operation: Option<(&str, &str)>,
    ) -> Result<(), PiAdapterError> {
        self.validate_dispatch_with_attachments(
            workspace_id,
            session_id,
            turn_id,
            run_id,
            correlation_id,
            input,
            attachments,
        )?;
        if let Some((runtime_id, provider_id)) = credential_operation {
            validate_id(runtime_id, "runtimeId")?;
            validate_id(provider_id, "providerId")?;
        }
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| PiAdapterError::State("unknown Pi session".into()))?;
        session.active_run = Some(ActiveRun {
            workspace_id: workspace_id.into(),
            turn_id: turn_id.into(),
            run_id: run_id.into(),
            correlation_id: correlation_id.into(),
        });
        let mut payload = Map::from_iter([("input".into(), Value::String(input.into()))]);
        if !attachments.is_empty() {
            payload.insert(
                "attachments".into(),
                Value::Array(
                    attachments
                        .iter()
                        .map(|attachment| {
                            json!({
                                "attachmentId": attachment.attachment_id,
                                "stableReference": attachment.stable_reference,
                                "displayName": attachment.display_name,
                                "mediaType": attachment.media_type,
                                "byteLength": attachment.byte_length,
                                "contentSha256": attachment.content_sha256,
                                "snapshotVersion": attachment.snapshot_version,
                                "contentBase64": BASE64_STANDARD.encode(&attachment.content),
                            })
                        })
                        .collect(),
                ),
            );
        }
        if let Some((runtime_id, provider_id)) = credential_operation {
            payload.insert("runtimeId".into(), Value::String(runtime_id.into()));
            payload.insert("providerId".into(), Value::String(provider_id.into()));
        }
        let response = self.request_with_correlation(
            "dispatch",
            Some(workspace_id),
            Some(session_id),
            Some(turn_id),
            Some(run_id),
            Value::Object(payload),
            correlation_id,
        );
        if response.is_err() {
            if let Some(session) = self.sessions.get_mut(session_id) {
                session.active_run = None;
            }
            return response.map(|_| ());
        }
        if response?.get("accepted").and_then(Value::as_bool) != Some(true) {
            self.sessions
                .get_mut(session_id)
                .expect("session exists")
                .active_run = None;
            return Err(PiAdapterError::Protocol(
                "dispatch response did not accept the run".into(),
            ));
        }
        Ok(())
    }

    pub fn resolve_tool(
        &mut self,
        session_id: &str,
        run_id: &str,
        correlation_id: &str,
        tool_call_id: &str,
        decision: PiToolDecision,
    ) -> Result<(), PiAdapterError> {
        let PiToolDecision::Denied { reason } = decision;
        self.resolve_tool_payload(
            session_id,
            run_id,
            correlation_id,
            tool_call_id,
            json!({ "toolCallId": tool_call_id, "decision": "denied", "reason": reason }),
        )
    }

    pub fn resolve_completed_tool(
        &mut self,
        runtime_id: &str,
        session_id: &str,
        run_id: &str,
        correlation_id: &str,
        tool_call_id: &str,
        receipt: &RuntimeExecutionReceipt,
    ) -> Result<(), PiAdapterError> {
        if !receipt.matches_runtime_tool(
            runtime_id,
            session_id,
            run_id,
            tool_call_id,
            self.process_generation,
        ) {
            return Err(PiAdapterError::State(
                "gateway receipt does not match the active Pi tool".into(),
            ));
        }
        if receipt.normalized_result().status != NormalizedActionStatus::Succeeded {
            return Err(PiAdapterError::State(
                "only a successful C4OS effect may complete a Pi tool".into(),
            ));
        }
        let result = if let Some(payload) = receipt.model_payload() {
            payload.clone()
        } else {
            serde_json::to_value(receipt.normalized_result())
                .map_err(|error| PiAdapterError::Protocol(error.to_string()))?
        };
        validate_value(&result, "tool result", true)?;
        self.resolve_tool_payload(
            session_id,
            run_id,
            correlation_id,
            tool_call_id,
            json!({ "toolCallId": tool_call_id, "decision": "completed", "result": result }),
        )
    }

    fn resolve_tool_payload(
        &mut self,
        session_id: &str,
        run_id: &str,
        correlation_id: &str,
        tool_call_id: &str,
        payload: Value,
    ) -> Result<(), PiAdapterError> {
        self.require_running()?;
        for (value, field) in [
            (session_id, "sessionId"),
            (run_id, "runId"),
            (correlation_id, "correlationId"),
            (tool_call_id, "toolCallId"),
        ] {
            validate_id(value, field)?;
        }
        let active = self
            .sessions
            .get(session_id)
            .and_then(|binding| binding.active_run.as_ref())
            .ok_or_else(|| PiAdapterError::State("run is not active".into()))?;
        if active.run_id != run_id || active.correlation_id != correlation_id {
            return Err(PiAdapterError::State(
                "tool decision does not match the active run".into(),
            ));
        }
        validate_value(&payload, "tool resolution", true)?;
        let response = self.request_with_correlation(
            "tool.resolve",
            None,
            Some(session_id),
            None,
            Some(run_id),
            payload,
            correlation_id,
        )?;
        if response.get("executedBySidecar").and_then(Value::as_bool) != Some(false) {
            return Err(PiAdapterError::Protocol(
                "Pi sidecar claimed authority to execute a tool".into(),
            ));
        }
        Ok(())
    }

    pub fn cancel(
        &mut self,
        workspace_id: &str,
        session_id: &str,
        turn_id: &str,
        run_id: &str,
        correlation_id: &str,
    ) -> Result<bool, PiAdapterError> {
        self.require_running()?;
        for (value, field) in [
            (workspace_id, "workspaceId"),
            (session_id, "sessionId"),
            (turn_id, "turnId"),
            (run_id, "runId"),
            (correlation_id, "correlationId"),
        ] {
            validate_id(value, field)?;
        }
        let active_matches = self
            .sessions
            .get(session_id)
            .and_then(|binding| binding.active_run.as_ref())
            .is_some_and(|active| {
                active.workspace_id == workspace_id
                    && active.turn_id == turn_id
                    && active.run_id == run_id
                    && active.correlation_id == correlation_id
            });
        if !active_matches {
            return Ok(false);
        }
        let response = self.request_with_correlation(
            "cancel",
            Some(workspace_id),
            Some(session_id),
            Some(turn_id),
            Some(run_id),
            Value::Object(Map::new()),
            correlation_id,
        )?;
        let cancelled = response
            .get("cancelled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if cancelled {
            self.sessions
                .get_mut(session_id)
                .expect("session exists")
                .active_run = None;
        }
        Ok(cancelled)
    }

    pub fn poll_events(&mut self) -> Result<Vec<PiEventEnvelope>, PiAdapterError> {
        self.require_running()?;
        let lines = self.runner.poll().map_err(PiAdapterError::Runner)?;
        for line in lines {
            let envelope = parse_envelope_line(&line)?;
            match envelope {
                PiWireEnvelope::Event(event) => self.accept_event(event)?,
                PiWireEnvelope::Response(_) => {
                    return Err(PiAdapterError::Protocol(
                        "unsolicited Pi response received while polling".into(),
                    ));
                }
            }
        }
        Ok(std::mem::take(&mut self.events))
    }

    pub fn take_events(&mut self) -> Vec<PiEventEnvelope> {
        std::mem::take(&mut self.events)
    }

    pub fn stale_events_rejected(&self) -> u64 {
        self.stale_events_rejected
    }

    pub fn shutdown(&mut self) -> Result<(), PiAdapterError> {
        if self.state == PiAdapterState::Stopped {
            return Ok(());
        }
        if matches!(self.state, PiAdapterState::Ready | PiAdapterState::Degraded) {
            // A graceful acknowledgement is advisory. Forced runner cleanup
            // remains authoritative when the worker is broken or unresponsive.
            let graceful_shutdown = self.request(
                "shutdown",
                None,
                None,
                None,
                None,
                Value::Object(Map::new()),
            );
            let _graceful_acknowledged = graceful_shutdown.is_ok_and(|response| {
                response.get("stopped").and_then(Value::as_bool) == Some(true)
            });
        }
        self.runner.terminate().map_err(PiAdapterError::Runner)?;
        self.sessions.clear();
        self.state = PiAdapterState::Stopped;
        Ok(())
    }

    fn require_running(&self) -> Result<(), PiAdapterError> {
        if matches!(self.state, PiAdapterState::Ready | PiAdapterState::Degraded) {
            Ok(())
        } else {
            Err(PiAdapterError::State("Pi adapter is not running".into()))
        }
    }

    fn request(
        &mut self,
        operation: &str,
        workspace_id: Option<&str>,
        session_id: Option<&str>,
        turn_id: Option<&str>,
        run_id: Option<&str>,
        payload: Value,
    ) -> Result<Value, PiAdapterError> {
        let correlation_id = format!("pi-correlation-{}", self.next_request);
        self.request_with_correlation(
            operation,
            workspace_id,
            session_id,
            turn_id,
            run_id,
            payload,
            &correlation_id,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn request_with_correlation(
        &mut self,
        operation: &str,
        workspace_id: Option<&str>,
        session_id: Option<&str>,
        turn_id: Option<&str>,
        run_id: Option<&str>,
        payload: Value,
        correlation_id: &str,
    ) -> Result<Value, PiAdapterError> {
        validate_id(operation, "operation")?;
        validate_id(correlation_id, "correlationId")?;
        validate_value(&payload, "request payload", false)?;
        let request_id = format!("pi-request-{}", self.next_request);
        self.next_request = self
            .next_request
            .checked_add(1)
            .ok_or_else(|| PiAdapterError::State("request counter exhausted".into()))?;
        let request = PiWireRequest {
            schema_version: PI_PROTOCOL_SCHEMA_VERSION,
            kind: "request".into(),
            request_id: request_id.clone(),
            correlation_id: correlation_id.into(),
            process_generation: self.process_generation,
            operation: operation.into(),
            workspace_id: workspace_id.map(str::to_owned),
            session_id: session_id.map(str::to_owned),
            turn_id: turn_id.map(str::to_owned),
            run_id: run_id.map(str::to_owned),
            payload,
        };
        let line = encode_line(&request)?;
        let lines = self
            .runner
            .exchange(&line)
            .map_err(PiAdapterError::Runner)?;
        let mut response = None;
        for line in lines {
            match parse_envelope_line(&line)? {
                PiWireEnvelope::Response(candidate) => {
                    if response.is_some() {
                        return Err(PiAdapterError::Protocol(
                            "runner returned multiple responses".into(),
                        ));
                    }
                    response = Some(candidate);
                }
                PiWireEnvelope::Event(event) => self.accept_event(event)?,
            }
        }
        let response = response
            .ok_or_else(|| PiAdapterError::Protocol("runner did not return a response".into()))?;
        response.validate_for(&request_id, correlation_id, self.process_generation)?;
        if response.status == "error" {
            let code = response
                .payload
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("pi_rejected")
                .to_owned();
            let message = response
                .payload
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Pi sidecar rejected the operation")
                .to_owned();
            return Err(PiAdapterError::Rejected { code, message });
        }
        Ok(response.payload)
    }

    fn accept_event(&mut self, event: PiEventEnvelope) -> Result<(), PiAdapterError> {
        event.validate()?;
        if event.process_generation != self.process_generation {
            self.stale_events_rejected += 1;
            return Ok(());
        }
        let active = self
            .sessions
            .get(&event.session_id)
            .and_then(|session| session.active_run.as_ref());
        let matches = active.is_some_and(|run| {
            run.workspace_id == event.workspace_id
                && run.turn_id == event.turn_id
                && run.run_id == event.run_id
                && run.correlation_id == event.correlation_id
        });
        if !matches || event.sequence <= self.last_sequence {
            self.stale_events_rejected += 1;
            return Ok(());
        }
        if event.category == "tool.action_intent" {
            validate_action_intent(&event)?;
        }
        let terminal = matches!(
            event.category.as_str(),
            "lifecycle.settled" | "lifecycle.cancelled" | "lifecycle.error"
        );
        let session_id = event.session_id.clone();
        self.last_sequence = event.sequence;
        self.events.push(event);
        if terminal {
            self.sessions
                .get_mut(&session_id)
                .expect("accepted event session exists")
                .active_run = None;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PiToolDecision {
    Denied { reason: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PiWireRequest {
    schema_version: u32,
    kind: String,
    request_id: String,
    correlation_id: String,
    process_generation: u64,
    operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    payload: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PiWireResponse {
    schema_version: u32,
    kind: String,
    request_id: String,
    correlation_id: String,
    process_generation: u64,
    status: String,
    payload: Value,
}

impl PiWireResponse {
    fn validate_for(
        &self,
        request_id: &str,
        correlation_id: &str,
        process_generation: u64,
    ) -> Result<(), PiAdapterError> {
        if self.schema_version != PI_PROTOCOL_SCHEMA_VERSION
            || self.kind != "response"
            || self.request_id != request_id
            || self.correlation_id != correlation_id
            || self.process_generation != process_generation
            || !matches!(self.status.as_str(), "ok" | "error")
        {
            return Err(PiAdapterError::Protocol(
                "response identity, generation, kind, or status mismatch".into(),
            ));
        }
        validate_value(&self.payload, "response payload", true)
    }
}

enum PiWireEnvelope {
    Response(PiWireResponse),
    Event(PiEventEnvelope),
}

impl PiEventEnvelope {
    fn validate(&self) -> Result<(), PiAdapterError> {
        if self.schema_version != PI_PROTOCOL_SCHEMA_VERSION
            || self.kind != "event"
            || self.runtime != "pi"
            || self.sequence == 0
        {
            return Err(PiAdapterError::Protocol(
                "event schema, kind, runtime, or sequence is invalid".into(),
            ));
        }
        for (value, field) in [
            (&self.event_id, "eventId"),
            (&self.correlation_id, "correlationId"),
            (&self.workspace_id, "workspaceId"),
            (&self.session_id, "sessionId"),
            (&self.turn_id, "turnId"),
            (&self.run_id, "runId"),
            (&self.category, "category"),
            (&self.native_type, "nativeType"),
        ] {
            validate_id(value, field)?;
        }
        if let Some(tool_call_id) = &self.tool_call_id {
            validate_id(tool_call_id, "toolCallId")?;
        }
        validate_value(&self.payload, "event payload", true)
    }
}

fn parse_envelope_line(line: &str) -> Result<PiWireEnvelope, PiAdapterError> {
    validate_line(line)?;
    let value: Value =
        serde_json::from_str(line).map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| PiAdapterError::Protocol("wire envelope has no kind".into()))?;
    match kind {
        "response" => serde_json::from_value(value)
            .map(PiWireEnvelope::Response)
            .map_err(|error| PiAdapterError::Protocol(error.to_string())),
        "event" => serde_json::from_value(value)
            .map(PiWireEnvelope::Event)
            .map_err(|error| PiAdapterError::Protocol(error.to_string())),
        _ => Err(PiAdapterError::Protocol(
            "wire envelope kind is unsupported".into(),
        )),
    }
}

fn encode_line<T: Serialize>(value: &T) -> Result<String, PiAdapterError> {
    let encoded = serde_json::to_string(value)
        .map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
    validate_line(&encoded)?;
    Ok(format!("{encoded}\n"))
}

fn validate_line(line: &str) -> Result<(), PiAdapterError> {
    let content = line.strip_suffix('\n').unwrap_or(line);
    if content.is_empty()
        || content.len() > PI_MAX_LINE_BYTES
        || content.contains('\n')
        || content.contains('\r')
    {
        return Err(PiAdapterError::Protocol(
            "wire message is empty, oversized, or not one JSON line".into(),
        ));
    }
    Ok(())
}

fn validate_sidecar_package(sidecar_root: &Path) -> Result<(), PiAdapterError> {
    let package_path = sidecar_root.join("package.json");
    let bytes = fs::read(&package_path).map_err(|error| {
        PiAdapterError::Io(format!(
            "could not read {}: {error}",
            package_path.display()
        ))
    })?;
    if bytes.len() > 64 * 1024 {
        return Err(PiAdapterError::IncompatiblePackage(
            "package.json exceeds 64 KiB".into(),
        ));
    }
    let package: PackageManifest = serde_json::from_slice(&bytes)
        .map_err(|error| PiAdapterError::IncompatiblePackage(error.to_string()))?;
    if package.name != "@c4os/pi-sidecar"
        || package.version != "0.1.0"
        || !package.private
        || package.module_type != "module"
        || package
            .dependencies
            .get(PI_NATIVE_PACKAGE)
            .map(String::as_str)
            != Some(PI_NATIVE_VERSION)
        || package
            .dependencies
            .get("@earendil-works/pi-agent-core")
            .map(String::as_str)
            != Some(PI_NATIVE_VERSION)
        || package
            .dependencies
            .get("@earendil-works/pi-ai")
            .map(String::as_str)
            != Some(PI_NATIVE_VERSION)
    {
        return Err(PiAdapterError::IncompatiblePackage(
            "sidecar package identity or exact Pi pins do not match".into(),
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
struct PackageManifest {
    name: String,
    version: String,
    private: bool,
    #[serde(rename = "type")]
    module_type: String,
    dependencies: BTreeMap<String, String>,
}

fn validate_sampling_request(request: &PiSamplingRequest) -> Result<(), PiAdapterError> {
    if request.messages.is_empty()
        || request.messages.len() > 128
        || request.max_tokens == 0
        || request.max_tokens > 1_000_000
        || request
            .temperature
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        || request
            .system_prompt
            .as_deref()
            .is_some_and(|value| value.len() > MAX_STRING_BYTES || value.contains('\0'))
        || request.messages.iter().any(|message| {
            !matches!(message.role.as_str(), "user" | "assistant")
                || message.text.is_empty()
                || message.text.len() > MAX_STRING_BYTES
                || message.text.contains('\0')
        })
    {
        return Err(PiAdapterError::Protocol(
            "sampling request is invalid or exceeds its bounds".into(),
        ));
    }
    let value = serde_json::to_value(&request.messages)
        .map_err(|error| PiAdapterError::Protocol(error.to_string()))?;
    validate_value(&value, "sampling messages", false)
}

fn validate_sampling_result(result: &PiSamplingResult) -> Result<(), PiAdapterError> {
    if result.text.is_empty()
        || result.text.len() > MAX_STRING_BYTES
        || result.text.contains('\0')
        || result.model.is_empty()
        || result.model.len() > 512
        || result.model.contains('\0')
        || !matches!(result.stop_reason.as_str(), "endTurn" | "maxTokens")
    {
        return Err(PiAdapterError::Protocol(
            "sampling result is invalid or exceeds its bounds".into(),
        ));
    }
    Ok(())
}

fn validate_health(health: &PiHealth, generation: u64) -> Result<(), PiAdapterError> {
    if health.runtime != "pi"
        || health.transport != "c4os-node-sdk-sidecar"
        || health.protocol != PI_PROTOCOL
        || health.process_generation != generation
        || !matches!(health.status.as_str(), "ready" | "degraded")
    {
        return Err(PiAdapterError::Protocol(
            "Pi health identity, transport, protocol, generation, or status mismatch".into(),
        ));
    }
    for required in [
        "streaming",
        "cancellation",
        "tools",
        "nativePersistence",
        "nativeExtensions",
        "nativeTools",
        "crashResume",
        "providerAuthentication",
        "rpcTransport",
    ] {
        let capability = health.capabilities.get(required).ok_or_else(|| {
            PiAdapterError::Protocol(format!("Pi health omitted capability {required}"))
        })?;
        if !matches!(
            capability.state.as_str(),
            "supported" | "unsupported" | "unknown" | "degraded"
        ) {
            return Err(PiAdapterError::Protocol(format!(
                "Pi capability {required} has an invalid state"
            )));
        }
    }
    if health.capabilities["nativePersistence"].state != "unsupported"
        || health.capabilities["nativeExtensions"].state != "unsupported"
        || health.capabilities["nativeTools"].state != "unsupported"
    {
        return Err(PiAdapterError::Protocol(
            "Pi sidecar tried to acquire persistence, extension, or native-tool authority".into(),
        ));
    }
    Ok(())
}

fn validate_version(version: &PiVersion) -> Result<(), PiAdapterError> {
    if version.adapter != "PIAdapter"
        || version.native_package != PI_NATIVE_PACKAGE
        || version.native_version != PI_NATIVE_VERSION
        || version.protocol != PI_PROTOCOL
        || !is_version(&version.adapter_version)
    {
        return Err(PiAdapterError::IncompatiblePackage(
            "Pi sidecar version response is incompatible".into(),
        ));
    }
    Ok(())
}

fn validate_action_intent(event: &PiEventEnvelope) -> Result<(), PiAdapterError> {
    let tool_call_id = event
        .tool_call_id
        .as_deref()
        .ok_or_else(|| PiAdapterError::Protocol("Pi action intent omitted toolCallId".into()))?;
    validate_id(tool_call_id, "toolCallId")?;
    let tool = event.payload.get("tool").and_then(Value::as_str);
    if !matches!(tool, Some("c4os_read_resource" | "c4os_propose_action"))
        || event.payload.get("authority").and_then(Value::as_str)
            != Some("c4os-action-gateway-required")
    {
        return Err(PiAdapterError::Protocol(
            "Pi action intent is not a C4OS-brokered tool".into(),
        ));
    }
    Ok(())
}

fn validate_relative_entrypoint(entrypoint: &str) -> Result<(), PiAdapterError> {
    let path = Path::new(entrypoint);
    if entrypoint != "main.mjs"
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(PiAdapterError::InvalidManifest(
            "entrypoint must be exactly main.mjs".into(),
        ));
    }
    Ok(())
}

fn validate_id(value: &str, field: &str) -> Result<(), PiAdapterError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => true,
            b'.' | b'_' | b':' | b'@' | b'/' | b'-' => index > 0,
            _ => false,
        });
    if valid {
        Ok(())
    } else {
        Err(PiAdapterError::Protocol(format!(
            "{field} is not a bounded identifier"
        )))
    }
}

fn validate_value(
    value: &Value,
    path: &str,
    reject_secret_values: bool,
) -> Result<(), PiAdapterError> {
    fn walk(
        value: &Value,
        path: &str,
        depth: usize,
        reject_secret_values: bool,
    ) -> Result<(), PiAdapterError> {
        if depth > MAX_VALUE_DEPTH {
            return Err(PiAdapterError::Protocol(format!(
                "{path} is nested too deeply"
            )));
        }
        match value {
            Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
            Value::String(text) => {
                let maximum = if is_direct_image_content_path(path) {
                    MAX_IMAGE_BASE64_BYTES
                } else {
                    MAX_STRING_BYTES
                };
                if text.len() > maximum {
                    return Err(PiAdapterError::Protocol(format!(
                        "{path} string exceeds its protocol bound"
                    )));
                }
                if reject_secret_values && looks_like_secret_value(text) {
                    return Err(PiAdapterError::Protocol(format!(
                        "{path} contains credential-shaped content"
                    )));
                }
                Ok(())
            }
            Value::Array(items) => {
                if items.len() > MAX_COLLECTION_ITEMS {
                    return Err(PiAdapterError::Protocol(format!(
                        "{path} contains too many items"
                    )));
                }
                for (index, item) in items.iter().enumerate() {
                    walk(
                        item,
                        &format!("{path}[{index}]"),
                        depth + 1,
                        reject_secret_values,
                    )?;
                }
                Ok(())
            }
            Value::Object(fields) => {
                if fields.len() > MAX_COLLECTION_ITEMS {
                    return Err(PiAdapterError::Protocol(format!(
                        "{path} contains too many fields"
                    )));
                }
                for (key, item) in fields {
                    if is_secret_field(key) {
                        return Err(PiAdapterError::Protocol(format!(
                            "{path} contains a forbidden credential field"
                        )));
                    }
                    walk(
                        item,
                        &format!("{path}.{key}"),
                        depth + 1,
                        reject_secret_values,
                    )?;
                }
                Ok(())
            }
        }
    }
    walk(value, path, 0, reject_secret_values)
}

fn is_direct_image_content_path(path: &str) -> bool {
    path.strip_prefix("request payload.attachments[")
        .and_then(|path| path.strip_suffix("].contentBase64"))
        .is_some_and(|index| !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_secret_field(field: &str) -> bool {
    let normalized = field
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    let reference = normalized.ends_with("reference")
        || normalized.ends_with("referenceid")
        || normalized.ends_with("ref")
        || normalized.ends_with("refid")
        || normalized.ends_with("leaseid");
    !reference
        && [
            "apikey",
            "accesstoken",
            "refreshtoken",
            "password",
            "passwd",
            "secret",
            "authorization",
            "cookie",
            "privatekey",
        ]
        .iter()
        .any(|candidate| normalized == *candidate || normalized.ends_with(candidate))
}

fn looks_like_secret_value(value: &str) -> bool {
    let lowercase = value.to_ascii_lowercase();
    lowercase.contains("-----begin private key-----")
        || lowercase.contains("-----begin rsa private key-----")
        || lowercase
            .split_whitespace()
            .any(|part| part == "bearer" || part.starts_with("bearer:"))
        || contains_uri_credentials(value)
}

fn contains_uri_credentials(value: &str) -> bool {
    value.find("://").is_some_and(|scheme| {
        let authority = &value[scheme + 3..];
        authority
            .split(['/', ' ', '\n'])
            .next()
            .is_some_and(|host| host.contains('@') && host.contains(':'))
    })
}

fn prefixed_sha256(content: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in Sha256::digest(content) {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn is_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let valid = (0..3).all(|_| {
        parts.next().is_some_and(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'))
        })
    });
    valid && parts.next().is_none()
}

pub fn c4os_tool_names() -> BTreeSet<&'static str> {
    BTreeSet::from(["c4os_read_resource", "c4os_propose_action"])
}
