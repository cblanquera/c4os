//! Authenticated, loopback-only OpenCode runtime adapter boundary.
//!
//! This module deliberately owns no policy authority. It constructs a narrow
//! OpenCode transport surface, normalizes native responses, and turns native
//! C4OS broker-tool requests into action intents. OpenCode's permission UX is
//! not a security boundary: native tools are denied, and even a completed C4OS
//! effect is acknowledged to OpenCode with a native rejection so OpenCode can
//! never perform the effect itself.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};

use crate::runtime::adapter::{
    ADAPTER_CONTRACT_SCHEMA_VERSION, AdapterAuthority, AdapterConformanceDescriptor,
    PeerCapabilityClaims, peer_capabilities,
};
use crate::runtime::capability::CapabilityState;
use crate::runtime::opencode_credential::{
    ProviderCredentialAuthorizationReceipt, ProviderCredentialRequest,
    opencode_message_id_for_operation,
};
use crate::runtime::provider::ProviderProfile;
use crate::runtime::supervisor::{RUNTIME_PROTOCOL_VERSION, RuntimeKind};

pub const OPENCODE_NATIVE_VERSION: &str = "1.18.3";
pub const OPENCODE_SDK_VERSION: &str = "1.18.3";
pub const OPENCODE_ADAPTER_PROTOCOL_VERSION: u16 = 1;
pub const OPENCODE_MAX_ATTACHMENTS: usize = 32;

const MAX_IDENTIFIER_BYTES: usize = 192;
const MAX_TITLE_BYTES: usize = 1_024;
const MAX_PROMPT_BYTES: usize = 512 * 1_024;
const MAX_JSON_BYTES: usize = 512 * 1_024;
const MAX_RESPONSE_BYTES: usize = 2 * 1_024 * 1_024;
const MAX_SSE_FRAME_BYTES: usize = 512 * 1_024;
const MAX_DELTA_BYTES: usize = 128 * 1_024;
pub const OPENCODE_MAX_INLINE_ATTACHMENT_BYTES: u64 = 256 * 1_024;
const MAX_PROVIDERS: usize = 256;
const MAX_MODELS: usize = 8_192;
const MAX_RESOURCES: usize = 256;
const MAX_SEEN_EVENT_IDS: usize = 16_384;
const MAX_PENDING_INTENTS: usize = 1_024;
pub const C4OS_ACTION_PROPOSAL_TOOL: &str = "c4os_propose_action";
pub const C4OS_RESOURCE_READ_TOOL: &str = "c4os_read_resource";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidManifest,
    IncompatibleVersion,
    InvalidEndpoint,
    InvalidStateNamespace,
    InvalidSecretReference,
    InvalidLaunchPlan,
    InvalidRequest,
    AuthorityOverrideRejected,
    NotStarted,
    NotReady,
    AlreadyStarted,
    ActiveRun,
    UnknownSession,
    StaleCorrelation,
    LateEvent,
    DuplicateEvent,
    BackpressureExceeded,
    UnexpectedStatus(u16),
    InvalidHealthPayload,
    InvalidSessionPayload,
    InvalidModelPayload,
    InvalidEventPayload,
    ResponseTooLarge,
    CommandFailed(CommandFailureCode),
    TransportFailed(TransportFailureCode),
    RestartGenerationMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandFailureCode {
    SpawnRejected,
    SecretChannelUnavailable,
    ProviderCredentialUnavailable,
    TerminationFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportFailureCode {
    Unavailable,
    Timeout,
    AuthenticationRejected,
    Protocol,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeCompatibilityManifest {
    pub native_version: String,
    pub sdk_version: String,
    pub binary_sha256: String,
    pub adapter_protocol_version: u16,
}

impl OpenCodeCompatibilityManifest {
    pub fn pinned(binary_sha256: impl Into<String>) -> Result<Self, AdapterError> {
        let manifest = Self {
            native_version: OPENCODE_NATIVE_VERSION.into(),
            sdk_version: OPENCODE_SDK_VERSION.into(),
            binary_sha256: binary_sha256.into(),
            adapter_protocol_version: OPENCODE_ADAPTER_PROTOCOL_VERSION,
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), AdapterError> {
        if self.native_version != OPENCODE_NATIVE_VERSION
            || self.sdk_version != OPENCODE_SDK_VERSION
            || self.adapter_protocol_version != OPENCODE_ADAPTER_PROTOCOL_VERSION
        {
            return Err(AdapterError::IncompatibleVersion);
        }
        if !is_sha256(&self.binary_sha256) {
            return Err(AdapterError::InvalidManifest);
        }
        Ok(())
    }

    pub fn conformance_descriptor(
        &self,
        process_generation: u64,
    ) -> Result<AdapterConformanceDescriptor, AdapterError> {
        self.validate()?;
        let descriptor = AdapterConformanceDescriptor {
            schema_version: ADAPTER_CONTRACT_SCHEMA_VERSION,
            runtime_kind: RuntimeKind::OpenCode,
            adapter_version: "1.0.0".into(),
            native_version: self.native_version.clone(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
            process_generation,
            authority: AdapterAuthority::C4osActionGatewayOnly,
            capabilities: peer_capabilities(PeerCapabilityClaims {
                health: CapabilityState::Supported,
                session_create: CapabilityState::Supported,
                session_resume: CapabilityState::Degraded,
                model_discovery: CapabilityState::Supported,
                streaming: CapabilityState::Supported,
                // The security path is complete, but OpenCode remains degraded
                // until the C4OS-owned broker tools are materialized for this
                // exact native installation. Native tools stay denied.
                action_intents: CapabilityState::Degraded,
                credential_channel: CapabilityState::Supported,
                cancellation: CapabilityState::Supported,
                // Production recovery replaces the entire app-owned peer so a
                // fresh listener reservation, server secret, provider channel,
                // and process generation are composed atomically. Reusing one
                // driver cannot satisfy those boundaries and is not published
                // as a supported native capability.
                restart: CapabilityState::Degraded,
            }),
        };
        descriptor
            .validate()
            .map_err(|_| AdapterError::InvalidManifest)?;
        Ok(descriptor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopbackEndpoint {
    address: IpAddr,
    port: u16,
}

impl LoopbackEndpoint {
    pub fn new(address: IpAddr, port: u16) -> Result<Self, AdapterError> {
        if !address.is_loopback() || port == 0 {
            return Err(AdapterError::InvalidEndpoint);
        }
        Ok(Self { address, port })
    }

    pub fn address(&self) -> IpAddr {
        self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn base_url(&self) -> String {
        match self.address {
            IpAddr::V4(address) => format!("http://{address}:{}", self.port),
            IpAddr::V6(address) => format!("http://[{address}]:{}", self.port),
        }
    }
}

/// Opaque reference to a supervisor-generated random password.
///
/// The referenced value is delivered through a private inherited descriptor.
/// It is structurally absent from command arguments, environment variables,
/// transport request values, persisted adapter state, and Debug output.
#[derive(Clone, Eq, PartialEq)]
pub struct RandomSecretReference {
    reference_id: String,
    entropy_bits: u16,
}

impl RandomSecretReference {
    pub fn new(reference_id: impl Into<String>, entropy_bits: u16) -> Result<Self, AdapterError> {
        let reference_id = reference_id.into();
        if !is_safe_identifier(&reference_id) || entropy_bits < 256 {
            return Err(AdapterError::InvalidSecretReference);
        }
        Ok(Self {
            reference_id,
            entropy_bits,
        })
    }

    pub fn reference_id(&self) -> &str {
        &self.reference_id
    }
}

impl fmt::Debug for RandomSecretReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RandomSecretReference")
            .field("reference_id", &"<opaque>")
            .field("secret", &"<redacted>")
            .field("entropy_bits", &self.entropy_bits)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateNamespace {
    root: PathBuf,
    config_home: PathBuf,
    data_home: PathBuf,
    cache_home: PathBuf,
    temp_home: PathBuf,
    workspace_id: String,
    process_generation: u64,
    launch_id: String,
}

impl StateNamespace {
    pub fn new(
        c4os_home: &Path,
        workspace_id: impl Into<String>,
        process_generation: u64,
        launch_id: impl Into<String>,
    ) -> Result<Self, AdapterError> {
        let workspace_id = workspace_id.into();
        let launch_id = launch_id.into();
        if !c4os_home.is_absolute()
            || c4os_home
                .components()
                .any(|component| matches!(component, Component::ParentDir))
            || !is_safe_identifier(&workspace_id)
            || !is_safe_identifier(&launch_id)
            || process_generation == 0
        {
            return Err(AdapterError::InvalidStateNamespace);
        }
        let root = c4os_home
            .join("runtimes")
            .join("opencode")
            .join(OPENCODE_NATIVE_VERSION)
            .join("workspaces")
            .join(&workspace_id)
            .join("generations")
            .join(process_generation.to_string())
            .join(&launch_id);
        Ok(Self {
            config_home: root.join("config"),
            data_home: root.join("data"),
            cache_home: root.join("cache"),
            temp_home: root.join("tmp"),
            root,
            workspace_id,
            process_generation,
            launch_id,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn config_home(&self) -> &Path {
        &self.config_home
    }

    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    pub fn process_generation(&self) -> u64 {
        self.process_generation
    }

    pub fn launch_id(&self) -> &str {
        &self.launch_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretChannel {
    pub inherited_fd: u32,
    pub reference: RandomSecretReference,
}

/// Startup configuration that denies OpenCode's native effectful tools and
/// enables only digest-pinned C4OS broker tools. OpenCode permissions are a UX
/// mechanism, not C4OS policy or containment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAuthorityPolicy {
    configuration_sha256: String,
    allowed_tool_proposals: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePermissionDefault {
    Deny,
}

impl NativeAuthorityPolicy {
    pub fn new<I, S>(
        configuration_sha256: impl Into<String>,
        allowed_tool_proposals: I,
    ) -> Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let configuration_sha256 = configuration_sha256.into();
        let allowed_tool_proposals = allowed_tool_proposals
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<_>>();
        if !is_sha256(&configuration_sha256)
            || allowed_tool_proposals.is_empty()
            || allowed_tool_proposals.len() > MAX_RESOURCES
            || allowed_tool_proposals.iter().any(|tool| {
                !matches!(
                    tool.as_str(),
                    C4OS_ACTION_PROPOSAL_TOOL | C4OS_RESOURCE_READ_TOOL
                )
            })
        {
            return Err(AdapterError::InvalidLaunchPlan);
        }
        Ok(Self {
            configuration_sha256,
            allowed_tool_proposals,
        })
    }

    pub fn configuration_sha256(&self) -> &str {
        &self.configuration_sha256
    }

    pub fn permission_default(&self) -> NativePermissionDefault {
        NativePermissionDefault::Deny
    }

    pub fn remember_native_decisions(&self) -> bool {
        false
    }

    pub fn allowed_tool_proposals(&self) -> &BTreeSet<String> {
        &self.allowed_tool_proposals
    }

    pub fn allows(&self, native_tool: &str) -> bool {
        self.allowed_tool_proposals.contains(native_tool)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchCommand {
    pub executable: PathBuf,
    pub expected_binary_sha256: String,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub working_directory: PathBuf,
    pub secret_channel: SecretChannel,
    pub authority_policy: NativeAuthorityPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeLaunchPlan {
    pub manifest: OpenCodeCompatibilityManifest,
    pub endpoint: LoopbackEndpoint,
    pub namespace: StateNamespace,
    pub executable: PathBuf,
    pub workspace_root: PathBuf,
    pub basic_auth_username: String,
    pub password_reference: RandomSecretReference,
    pub secret_channel_fd: u32,
    pub authority_policy: NativeAuthorityPolicy,
}

impl OpenCodeLaunchPlan {
    pub fn validate(&self) -> Result<(), AdapterError> {
        self.manifest.validate()?;
        if !self.executable.is_absolute()
            || !self.workspace_root.is_absolute()
            || path_text(&self.executable).is_err()
            || path_text(&self.workspace_root).is_err()
            || self.namespace.process_generation == 0
            || self.namespace.workspace_id.is_empty()
            || self.basic_auth_username != "opencode"
            || self.secret_channel_fd < 3
        {
            return Err(AdapterError::InvalidLaunchPlan);
        }
        Ok(())
    }

    pub fn command(&self) -> Result<LaunchCommand, AdapterError> {
        self.validate()?;
        let mut environment = BTreeMap::new();
        environment.insert(
            "XDG_CONFIG_HOME".into(),
            path_text(&self.namespace.config_home)?,
        );
        environment.insert(
            "XDG_DATA_HOME".into(),
            path_text(&self.namespace.data_home)?,
        );
        environment.insert(
            "XDG_CACHE_HOME".into(),
            path_text(&self.namespace.cache_home)?,
        );
        environment.insert("TMPDIR".into(), path_text(&self.namespace.temp_home)?);
        environment.insert("NO_PROXY".into(), "127.0.0.1,::1,localhost".into());
        Ok(LaunchCommand {
            executable: self.executable.clone(),
            expected_binary_sha256: self.manifest.binary_sha256.clone(),
            arguments: vec![
                "serve".into(),
                "--hostname".into(),
                self.endpoint.address().to_string(),
                "--port".into(),
                self.endpoint.port().to_string(),
            ],
            environment,
            working_directory: self.workspace_root.clone(),
            secret_channel: SecretChannel {
                inherited_fd: self.secret_channel_fd,
                reference: self.password_reference.clone(),
            },
            authority_policy: self.authority_policy.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessHandle {
    pub process_id: u32,
    pub process_generation: u64,
}

pub trait CommandDriver {
    fn spawn(&mut self, command: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode>;

    fn terminate_process_group(
        &mut self,
        process: &ProcessHandle,
    ) -> Result<(), CommandFailureCode>;

    /// Authorizes the exact active attempt that may request bounded one-use
    /// provider leases from the private descriptor. No secret is delivered by
    /// this call.
    fn authorize_provider_credential_attempt(
        &mut self,
        _request: ProviderCredentialRequest,
    ) -> Result<Option<ProviderCredentialAuthorizationReceipt>, CommandFailureCode> {
        Ok(None)
    }

    fn revoke_provider_credential_attempt(
        &mut self,
        _request: &ProviderCredentialRequest,
    ) -> Result<(), CommandFailureCode> {
        Ok(())
    }

    fn revoke_all_provider_credential_attempts(&mut self) -> Result<(), CommandFailureCode> {
        Ok(())
    }

    fn register_provider_credential_route(
        &mut self,
        _profile: &ProviderProfile,
    ) -> Result<(), CommandFailureCode> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportAuth {
    Basic {
        username: String,
        password_reference: RandomSecretReference,
    },
}

#[derive(Clone, Eq, PartialEq)]
pub struct TransportRequest {
    pub method: HttpMethod,
    pub base_url: String,
    pub path: String,
    pub body: Option<Vec<u8>>,
    pub auth: TransportAuth,
    pub maximum_response_bytes: usize,
}

impl fmt::Debug for TransportRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportRequest")
            .field("method", &self.method)
            .field("base_url", &self.base_url)
            .field("path", &self.path)
            .field("body", &self.body.as_ref().map(|body| body.len()))
            .field("auth", &"Basic <redacted>")
            .field("maximum_response_bytes", &self.maximum_response_bytes)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TransportResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

impl fmt::Debug for TransportResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_bytes", &self.body.len())
            .finish()
    }
}

pub trait OpenCodeTransport {
    /// The transport resolves the opaque password reference just in time and
    /// injects the HTTP Basic Authorization header without returning the raw
    /// password to the adapter.
    fn execute(
        &mut self,
        request: TransportRequest,
    ) -> Result<TransportResponse, TransportFailureCode>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Stopped,
    Starting,
    Ready,
    Degraded,
    Restarting,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshot {
    pub healthy: bool,
    pub native_version: String,
    pub process_generation: u64,
    pub checked_at_ms: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeHealth {
    healthy: bool,
    version: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    Idle,
    Running,
    Cancelled,
    Completed,
    Interrupted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionBinding {
    pub workspace_id: String,
    pub c4os_session_id: String,
    pub native_session_id: String,
    pub process_generation: u64,
    pub state: SessionState,
    pub active_run_id: Option<String>,
    pub last_run_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NativeSession {
    id: String,
    #[serde(rename = "projectID")]
    project_id: String,
    directory: String,
    title: String,
    version: String,
    time: NativeSessionTime,
}

#[derive(Deserialize)]
struct NativeSessionTime {
    created: u64,
    updated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedModel {
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub lifecycle: ModelLifecycle,
    pub context_limit: u64,
    pub output_limit: u64,
    pub input_modalities: BTreeSet<String>,
    pub output_modalities: BTreeSet<String>,
    pub reasoning: bool,
    pub tool_calling: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelLifecycle {
    Alpha,
    Beta,
    Active,
    Deprecated,
    Unknown,
}

#[derive(Deserialize)]
struct NativeProviderInventory {
    providers: Vec<NativeProvider>,
    #[serde(default)]
    default: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct NativeProvider {
    id: String,
    #[allow(dead_code)]
    name: String,
    models: BTreeMap<String, NativeModel>,
}

#[derive(Deserialize)]
struct NativeModel {
    id: String,
    #[serde(rename = "providerID")]
    provider_id: String,
    name: String,
    #[serde(default)]
    capabilities: NativeModelCapabilities,
    limit: NativeModelLimits,
    #[serde(default)]
    status: String,
}

#[derive(Default, Deserialize)]
struct NativeModelCapabilities {
    #[serde(default)]
    reasoning: bool,
    #[serde(default)]
    toolcall: bool,
    #[serde(default)]
    input: BTreeMap<String, bool>,
    #[serde(default)]
    output: BTreeMap<String, bool>,
}

#[derive(Deserialize)]
struct NativeModelLimits {
    context: u64,
    output: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoute {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PromptDispatch {
    pub correlation: EventCorrelation,
    pub model: ModelRoute,
    pub text: String,
    /// Core-verified subset of the fixed C4OS broker tools exposed for this
    /// request. Native overrides cannot add to this set.
    pub eligible_tool_ids: BTreeSet<String>,
    /// Exact C4OS durable attachment metadata and verified bounded bytes. The
    /// native request receives an inline data URL, never a renderer-selected
    /// path, opaque C4OS URL, or ambient filesystem authority.
    pub attachments: Vec<PromptAttachment>,
    /// Narrow, non-authority-bearing OpenCode options. `tools`, permission,
    /// shell, and command surfaces are always rejected.
    pub native_overrides: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAttachment {
    pub attachment_id: String,
    pub stable_reference: String,
    pub display_name: String,
    pub media_type: String,
    pub byte_length: u64,
    pub content_sha256: String,
    pub snapshot_version: u64,
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventCorrelation {
    pub workspace_id: String,
    pub c4os_session_id: String,
    pub c4os_turn_id: String,
    pub c4os_run_id: String,
    pub correlation_id: String,
    pub native_session_id: String,
    pub process_generation: u64,
}

#[derive(Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionIntent {
    pub workspace_id: String,
    pub c4os_session_id: String,
    pub c4os_turn_id: String,
    pub c4os_run_id: String,
    pub correlation_id: String,
    pub runtime_id: String,
    pub process_generation: u64,
    pub native_session_id: String,
    pub native_request_id: String,
    pub native_tool: String,
    pub native_arguments: Value,
    pub resources: Vec<String>,
}

impl ActionIntent {
    pub fn binding_sha256(&self) -> Result<String, AdapterError> {
        let encoded = serde_json::to_vec(self).map_err(|_| AdapterError::InvalidEventPayload)?;
        if encoded.len() > MAX_JSON_BYTES {
            return Err(AdapterError::InvalidEventPayload);
        }
        Ok(sha256(&encoded))
    }
}

impl fmt::Debug for ActionIntent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActionIntent")
            .field("workspace_id", &self.workspace_id)
            .field("c4os_session_id", &self.c4os_session_id)
            .field("c4os_turn_id", &self.c4os_turn_id)
            .field("c4os_run_id", &self.c4os_run_id)
            .field("correlation_id", &self.correlation_id)
            .field("runtime_id", &self.runtime_id)
            .field("process_generation", &self.process_generation)
            .field("native_session_id", &self.native_session_id)
            .field("native_request_id", &self.native_request_id)
            .field("native_tool", &self.native_tool)
            .field("native_arguments", &"<redacted>")
            .field("resources", &self.resources)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "category", content = "detail")]
pub enum NormalizedEventCategory {
    Lifecycle {
        state: String,
    },
    TextDelta {
        delta: String,
    },
    ThinkingDelta {
        delta: String,
    },
    ActionIntent(Box<ActionIntent>),
    ToolProgress {
        tool_call_id: String,
        summary: String,
    },
    ToolResult {
        tool_call_id: String,
        succeeded: bool,
    },
    Usage {
        input_tokens: u64,
        output_tokens: u64,
    },
    Completed,
    Cancelled,
    Error {
        code: String,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedEvent {
    pub runtime_id: String,
    pub native_version: String,
    pub workspace_id: String,
    pub c4os_session_id: String,
    pub c4os_turn_id: String,
    pub c4os_run_id: String,
    pub correlation_id: String,
    pub native_session_id: String,
    pub native_event_id: String,
    pub native_event_type: String,
    pub process_generation: u64,
    pub adapter_sequence: u64,
    pub received_at_ms: u64,
    pub replayed: bool,
    pub authenticated_assistant_message: Option<AuthenticatedAssistantMessage>,
    pub category: NormalizedEventCategory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedAssistantMessage {
    pub native_message_id: String,
    pub parent_native_message_id: String,
    pub role: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct C4osDecisionReceipt {
    pub correlation: EventCorrelation,
    pub native_request_id: String,
    pub action_binding_sha256: String,
    pub decision: C4osPermissionDecision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum C4osPermissionDecision {
    C4osEffectCompleted,
    Deny,
}

#[derive(Deserialize, Serialize)]
struct NativeEventEnvelope {
    #[serde(rename = "type")]
    native_type: String,
    #[serde(default)]
    properties: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingNativePermission {
    correlation: EventCorrelation,
    action_binding_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestartTicket {
    pub prior_generation: u64,
    pub next_generation: u64,
    pub issued_at_ms: u64,
}

pub struct OpenCodeAdapter<T, C> {
    transport: T,
    command_driver: C,
    launch_plan: OpenCodeLaunchPlan,
    lifecycle: LifecycleState,
    process: Option<ProcessHandle>,
    health: Option<HealthSnapshot>,
    sessions: BTreeMap<String, SessionBinding>,
    adapter_sequence: u64,
    seen_native_event_ids: BTreeSet<String>,
    seen_native_event_order: VecDeque<String>,
    pending_native_permissions: BTreeMap<String, PendingNativePermission>,
    active_credential_attempts: BTreeMap<String, ProviderCredentialRequest>,
}

impl<T: OpenCodeTransport, C: CommandDriver> OpenCodeAdapter<T, C> {
    pub fn new(
        launch_plan: OpenCodeLaunchPlan,
        transport: T,
        command_driver: C,
    ) -> Result<Self, AdapterError> {
        launch_plan.validate()?;
        Ok(Self {
            transport,
            command_driver,
            launch_plan,
            lifecycle: LifecycleState::Stopped,
            process: None,
            health: None,
            sessions: BTreeMap::new(),
            adapter_sequence: 0,
            seen_native_event_ids: BTreeSet::new(),
            seen_native_event_order: VecDeque::new(),
            pending_native_permissions: BTreeMap::new(),
            active_credential_attempts: BTreeMap::new(),
        })
    }

    pub fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }

    /// Returns the exact supervised native process ID for host-level cleanup
    /// and process-surface verification. This value is not an IPC capability.
    pub fn process_id(&self) -> Option<u32> {
        self.process.as_ref().map(|process| process.process_id)
    }

    pub fn health(&self) -> Option<&HealthSnapshot> {
        self.health.as_ref()
    }

    pub fn session(&self, c4os_session_id: &str) -> Option<&SessionBinding> {
        self.sessions.get(c4os_session_id)
    }

    pub fn register_provider_credential_route(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<(), AdapterError> {
        self.command_driver
            .register_provider_credential_route(profile)
            .map_err(AdapterError::CommandFailed)
    }

    pub fn start(&mut self, checked_at_ms: u64) -> Result<HealthSnapshot, AdapterError> {
        if self.lifecycle != LifecycleState::Stopped {
            return Err(AdapterError::AlreadyStarted);
        }
        self.lifecycle = LifecycleState::Starting;
        let command = self.launch_plan.command()?;
        let process = self
            .command_driver
            .spawn(&command)
            .map_err(AdapterError::CommandFailed)?;
        if process.process_id == 0
            || process.process_generation != self.launch_plan.namespace.process_generation
        {
            if process.process_id != 0 {
                self.command_driver
                    .terminate_process_group(&process)
                    .map_err(AdapterError::CommandFailed)?;
            }
            self.lifecycle = LifecycleState::Degraded;
            return Err(AdapterError::InvalidLaunchPlan);
        }
        self.process = Some(process);
        match self.probe(checked_at_ms) {
            Ok(health) => Ok(health),
            Err(error) => {
                if let Some(process) = self.process.take() {
                    self.command_driver
                        .terminate_process_group(&process)
                        .map_err(AdapterError::CommandFailed)?;
                }
                self.health = None;
                self.lifecycle = LifecycleState::Degraded;
                Err(error)
            }
        }
    }

    pub fn probe(&mut self, checked_at_ms: u64) -> Result<HealthSnapshot, AdapterError> {
        self.require_process()?;
        let response = self.request(HttpMethod::Get, "/global/health".into(), None)?;
        if response.status != 200 {
            self.lifecycle = LifecycleState::Degraded;
            return Err(AdapterError::UnexpectedStatus(response.status));
        }
        let health: NativeHealth = decode_bounded(
            &response.body,
            MAX_RESPONSE_BYTES,
            AdapterError::InvalidHealthPayload,
        )?;
        if !health.healthy || health.version != OPENCODE_NATIVE_VERSION {
            self.lifecycle = LifecycleState::Degraded;
            return Err(if health.version != OPENCODE_NATIVE_VERSION {
                AdapterError::IncompatibleVersion
            } else {
                AdapterError::InvalidHealthPayload
            });
        }
        let snapshot = HealthSnapshot {
            healthy: true,
            native_version: health.version,
            process_generation: self.launch_plan.namespace.process_generation,
            checked_at_ms,
        };
        self.health = Some(snapshot.clone());
        self.lifecycle = LifecycleState::Ready;
        Ok(snapshot)
    }

    pub fn list_models(&mut self) -> Result<Vec<NormalizedModel>, AdapterError> {
        self.require_ready()?;
        let response = self.request(HttpMethod::Get, "/config/providers".into(), None)?;
        if response.status != 200 {
            return Err(AdapterError::UnexpectedStatus(response.status));
        }
        let inventory: NativeProviderInventory = decode_bounded(
            &response.body,
            MAX_RESPONSE_BYTES,
            AdapterError::InvalidModelPayload,
        )?;
        if inventory.providers.len() > MAX_PROVIDERS || inventory.default.len() > MAX_PROVIDERS {
            return Err(AdapterError::InvalidModelPayload);
        }
        let mut models = Vec::new();
        for provider in inventory.providers {
            if !is_safe_route_identifier(&provider.id) || provider.models.len() > MAX_MODELS {
                return Err(AdapterError::InvalidModelPayload);
            }
            for (model_key, model) in provider.models {
                if models.len() == MAX_MODELS
                    || !is_safe_route_identifier(&model_key)
                    || !is_safe_route_identifier(&model.id)
                    || !is_safe_route_identifier(&model.provider_id)
                    || model.provider_id != provider.id
                    || model.id != model_key
                    || !is_bounded_text(&model.name, MAX_TITLE_BYTES)
                    || model.limit.context == 0
                    || model.limit.output == 0
                {
                    return Err(AdapterError::InvalidModelPayload);
                }
                models.push(NormalizedModel {
                    provider_id: provider.id.clone(),
                    model_id: model.id,
                    display_name: model.name,
                    lifecycle: match model.status.as_str() {
                        "alpha" => ModelLifecycle::Alpha,
                        "beta" => ModelLifecycle::Beta,
                        "active" => ModelLifecycle::Active,
                        "deprecated" => ModelLifecycle::Deprecated,
                        _ => ModelLifecycle::Unknown,
                    },
                    context_limit: model.limit.context,
                    output_limit: model.limit.output,
                    input_modalities: enabled_keys(model.capabilities.input),
                    output_modalities: enabled_keys(model.capabilities.output),
                    reasoning: model.capabilities.reasoning,
                    tool_calling: model.capabilities.toolcall,
                });
            }
        }
        models.sort_by(|left, right| {
            (&left.provider_id, &left.model_id).cmp(&(&right.provider_id, &right.model_id))
        });
        Ok(models)
    }

    pub fn create_session(
        &mut self,
        c4os_session_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Result<SessionBinding, AdapterError> {
        self.require_ready()?;
        let c4os_session_id = c4os_session_id.into();
        let title = title.into();
        if !is_safe_identifier(&c4os_session_id)
            || !is_bounded_text(&title, MAX_TITLE_BYTES)
            || self.sessions.contains_key(&c4os_session_id)
        {
            return Err(AdapterError::InvalidRequest);
        }
        let body = encode_bounded(&json!({ "title": title }))?;
        let response = self.request(HttpMethod::Post, "/session".into(), Some(body))?;
        if response.status != 200 {
            return Err(AdapterError::UnexpectedStatus(response.status));
        }
        let native: NativeSession = decode_bounded(
            &response.body,
            MAX_RESPONSE_BYTES,
            AdapterError::InvalidSessionPayload,
        )?;
        if !is_safe_identifier(&native.id)
            || !is_bounded_text(&native.project_id, MAX_IDENTIFIER_BYTES)
            || !is_bounded_text(&native.title, MAX_TITLE_BYTES)
            || native.version != OPENCODE_NATIVE_VERSION
            || native.directory != workspace_identity_text(&self.launch_plan.workspace_root)?
            || native.time.created == 0
            || native.time.updated < native.time.created
        {
            return Err(AdapterError::InvalidSessionPayload);
        }
        let binding = SessionBinding {
            workspace_id: self.launch_plan.namespace.workspace_id.clone(),
            c4os_session_id: c4os_session_id.clone(),
            native_session_id: native.id,
            process_generation: self.launch_plan.namespace.process_generation,
            state: SessionState::Idle,
            active_run_id: None,
            last_run_id: None,
        };
        self.sessions.insert(c4os_session_id, binding.clone());
        Ok(binding)
    }

    pub fn send(&mut self, dispatch: PromptDispatch) -> Result<(), AdapterError> {
        self.send_inner(dispatch, false)
    }

    /// Production dispatch path. Process/session/provider/operation binding is
    /// derived inside the adapter; callers cannot supply a vault reference or
    /// credential lease identity.
    pub fn send_with_provider_credential(
        &mut self,
        dispatch: PromptDispatch,
    ) -> Result<(), AdapterError> {
        self.send_inner(dispatch, true)
    }

    fn send_inner(
        &mut self,
        dispatch: PromptDispatch,
        authorize_provider_credential: bool,
    ) -> Result<(), AdapterError> {
        self.require_ready()?;
        validate_prompt_dispatch(&dispatch)?;
        self.validate_correlation(&dispatch.correlation, false)?;
        let binding = self
            .sessions
            .get(&dispatch.correlation.c4os_session_id)
            .ok_or(AdapterError::UnknownSession)?;
        if binding.state == SessionState::Running {
            return Err(AdapterError::ActiveRun);
        }
        let native_message_id =
            opencode_message_id_for_operation(&dispatch.correlation.correlation_id).map_err(
                |_| AdapterError::CommandFailed(CommandFailureCode::ProviderCredentialUnavailable),
            )?;
        let mut credential_request =
            authorize_provider_credential.then(|| ProviderCredentialRequest {
                process_generation: dispatch.correlation.process_generation,
                native_session_id: dispatch.correlation.native_session_id.clone(),
                provider_id: dispatch.model.provider_id.clone(),
                model_id: dispatch.model.model_id.clone(),
                operation_id: dispatch.correlation.correlation_id.clone(),
            });
        let credential_receipt = if let Some(request) = credential_request.as_ref() {
            self.command_driver
                .authorize_provider_credential_attempt(request.clone())
                .map_err(AdapterError::CommandFailed)?
        } else {
            None
        };
        if credential_receipt
            .as_ref()
            .is_some_and(|receipt| !receipt.credential_required)
        {
            credential_request = None;
        }
        let native_provider_id = credential_receipt
            .as_ref()
            .map_or(dispatch.model.provider_id.as_str(), |receipt| {
                receipt.native_provider_id.as_str()
            });
        let fallback_native_model_id = dispatch
            .model
            .model_id
            .strip_prefix(native_provider_id)
            .and_then(|value| value.strip_prefix('/'))
            .unwrap_or(dispatch.model.model_id.as_str());
        let native_model_id = credential_receipt
            .as_ref()
            .map_or(fallback_native_model_id, |receipt| {
                receipt.native_model_id.as_str()
            });

        let mut body = serde_json::Map::new();
        body.insert("messageID".into(), Value::String(native_message_id));
        body.insert(
            "model".into(),
            json!({
                "providerID": native_provider_id,
                "modelID": native_model_id,
            }),
        );
        let mut parts = Vec::with_capacity(dispatch.attachments.len() + 1);
        if !dispatch.text.trim().is_empty() {
            parts.push(json!({ "type": "text", "text": dispatch.text }));
        }
        parts.extend(dispatch.attachments.iter().map(|attachment| {
            let encoded = BASE64_STANDARD.encode(&attachment.content);
            json!({
                "type": "file",
                "mime": attachment.media_type,
                "filename": attachment.display_name,
                "url": format!("data:{};base64,{encoded}", attachment.media_type),
            })
        }));
        body.insert("parts".into(), Value::Array(parts));
        body.insert(
            "tools".into(),
            Value::Object(
                dispatch
                    .eligible_tool_ids
                    .iter()
                    .map(|tool| (tool.clone(), Value::Bool(true)))
                    .collect(),
            ),
        );
        if let Value::Object(overrides) = dispatch.native_overrides {
            for (key, value) in overrides {
                body.insert(key, value);
            }
        }
        let payload = match encode_bounded(&Value::Object(body)) {
            Ok(payload) => payload,
            Err(error) => {
                if let Some(request) = credential_request.as_ref() {
                    self.command_driver
                        .revoke_provider_credential_attempt(request)
                        .map_err(AdapterError::CommandFailed)?;
                }
                return Err(error);
            }
        };
        let path = format!(
            "/session/{}/prompt_async",
            dispatch.correlation.native_session_id
        );
        let response = match self.request(HttpMethod::Post, path, Some(payload)) {
            Ok(response) => response,
            Err(error) => {
                if let Some(request) = credential_request.as_ref() {
                    self.command_driver
                        .revoke_provider_credential_attempt(request)
                        .map_err(AdapterError::CommandFailed)?;
                }
                return Err(error);
            }
        };
        if response.status != 204 && response.status != 200 {
            if let Some(request) = credential_request.as_ref() {
                self.command_driver
                    .revoke_provider_credential_attempt(request)
                    .map_err(AdapterError::CommandFailed)?;
            }
            return Err(AdapterError::UnexpectedStatus(response.status));
        }
        let binding = self
            .sessions
            .get_mut(&dispatch.correlation.c4os_session_id)
            .ok_or(AdapterError::UnknownSession)?;
        binding.state = SessionState::Running;
        binding.active_run_id = Some(dispatch.correlation.c4os_run_id.clone());
        binding.last_run_id = Some(dispatch.correlation.c4os_run_id);
        if let Some(request) = credential_request {
            self.active_credential_attempts
                .insert(dispatch.correlation.c4os_session_id, request);
        }
        Ok(())
    }

    /// Idempotently aborts one native session. Once cancelled, all late events
    /// for the run are rejected instead of resurrecting UI state.
    pub fn cancel(&mut self, correlation: &EventCorrelation) -> Result<bool, AdapterError> {
        self.require_ready()?;
        self.validate_correlation(correlation, true)?;
        let binding = self
            .sessions
            .get(correlation.c4os_session_id.as_str())
            .ok_or(AdapterError::UnknownSession)?;
        if binding.state != SessionState::Running {
            return Ok(false);
        }
        let path = format!("/session/{}/abort", correlation.native_session_id);
        let response = self.request(HttpMethod::Post, path, Some(b"{}".to_vec()))?;
        if response.status != 200 {
            return Err(AdapterError::UnexpectedStatus(response.status));
        }
        let accepted: bool = decode_bounded(
            &response.body,
            MAX_RESPONSE_BYTES,
            AdapterError::InvalidSessionPayload,
        )?;
        if let Some(request) = self
            .active_credential_attempts
            .get(&correlation.c4os_session_id)
            .cloned()
        {
            self.command_driver
                .revoke_provider_credential_attempt(&request)
                .map_err(AdapterError::CommandFailed)?;
            self.active_credential_attempts
                .remove(&correlation.c4os_session_id);
        }
        let binding = self
            .sessions
            .get_mut(correlation.c4os_session_id.as_str())
            .ok_or(AdapterError::UnknownSession)?;
        binding.state = SessionState::Cancelled;
        binding.active_run_id = None;
        self.pending_native_permissions
            .retain(|_, pending| pending.correlation != *correlation);
        Ok(accepted)
    }

    /// Closes a native request only from an exact C4OS decision receipt. Both a
    /// denial and a C4OS-completed effect produce native `reject`: the native
    /// worker never receives authority to perform the side effect itself.
    pub fn respond_to_permission(
        &mut self,
        receipt: &C4osDecisionReceipt,
    ) -> Result<(), AdapterError> {
        self.require_ready()?;
        self.validate_correlation(&receipt.correlation, true)?;
        if !is_safe_identifier(&receipt.native_request_id)
            || !is_sha256(&receipt.action_binding_sha256)
        {
            return Err(AdapterError::InvalidRequest);
        }
        let pending = self
            .pending_native_permissions
            .get(&receipt.native_request_id)
            .ok_or(AdapterError::StaleCorrelation)?;
        if pending.correlation != receipt.correlation
            || pending.action_binding_sha256 != receipt.action_binding_sha256
        {
            return Err(AdapterError::StaleCorrelation);
        }
        let reply = match receipt.decision {
            C4osPermissionDecision::C4osEffectCompleted | C4osPermissionDecision::Deny => "reject",
        };
        let path = format!(
            "/session/{}/permissions/{}",
            receipt.correlation.native_session_id, receipt.native_request_id
        );
        let response = self.request(
            HttpMethod::Post,
            path,
            Some(encode_bounded(&json!({ "response": reply }))?),
        )?;
        if response.status != 200 {
            return Err(AdapterError::UnexpectedStatus(response.status));
        }
        self.pending_native_permissions
            .remove(&receipt.native_request_id);
        Ok(())
    }

    pub fn normalize_sse(
        &mut self,
        correlation: &EventCorrelation,
        frame: &[u8],
        received_at_ms: u64,
    ) -> Result<NormalizedEvent, AdapterError> {
        self.require_ready()?;
        self.validate_correlation(correlation, true)?;
        let binding = self
            .sessions
            .get(&correlation.c4os_session_id)
            .ok_or(AdapterError::UnknownSession)?;
        if binding.state == SessionState::Cancelled {
            return Err(AdapterError::LateEvent);
        }
        if frame.len() > MAX_SSE_FRAME_BYTES {
            return Err(AdapterError::ResponseTooLarge);
        }
        let parsed = parse_sse_frame(frame)?;
        let native: NativeEventEnvelope = decode_bounded(
            parsed.data,
            MAX_SSE_FRAME_BYTES,
            AdapterError::InvalidEventPayload,
        )?;
        if !is_bounded_text(&native.native_type, MAX_IDENTIFIER_BYTES) {
            return Err(AdapterError::InvalidEventPayload);
        }
        let native_event_id = deterministic_native_event_id(parsed.id, &native)?;
        if self.seen_native_event_ids.contains(&native_event_id) {
            return Err(AdapterError::DuplicateEvent);
        }
        let event_session_id = native_event_session_id(&native);
        if event_session_id.is_some_and(|id| id != correlation.native_session_id)
            || (event_session_id.is_none() && !is_global_native_event(&native.native_type))
        {
            return Err(AdapterError::StaleCorrelation);
        }

        let authenticated_assistant_message = authenticated_assistant_message(&native)?;
        let category = normalize_native_category(correlation, &native, &native_event_id)?;
        if let NormalizedEventCategory::ActionIntent(intent) = &category
            && !self
                .launch_plan
                .authority_policy
                .allows(&intent.native_tool)
        {
            return Err(AdapterError::AuthorityOverrideRejected);
        }
        if native.native_type == "permission.v2.asked" || native.native_type == "permission.asked" {
            let NormalizedEventCategory::ActionIntent(intent) = &category else {
                return Err(AdapterError::InvalidEventPayload);
            };
            if self.pending_native_permissions.len() == MAX_PENDING_INTENTS
                || self
                    .pending_native_permissions
                    .contains_key(&intent.native_request_id)
            {
                return Err(AdapterError::BackpressureExceeded);
            }
            self.pending_native_permissions.insert(
                intent.native_request_id.clone(),
                PendingNativePermission {
                    correlation: correlation.clone(),
                    action_binding_sha256: intent.binding_sha256()?,
                },
            );
        }
        self.remember_native_event_id(native_event_id.clone());
        self.adapter_sequence = self
            .adapter_sequence
            .checked_add(1)
            .ok_or(AdapterError::BackpressureExceeded)?;
        let terminal = matches!(
            category,
            NormalizedEventCategory::Completed
                | NormalizedEventCategory::Cancelled
                | NormalizedEventCategory::Error { .. }
        );
        let event = NormalizedEvent {
            runtime_id: "opencode".into(),
            native_version: OPENCODE_NATIVE_VERSION.into(),
            workspace_id: correlation.workspace_id.clone(),
            c4os_session_id: correlation.c4os_session_id.clone(),
            c4os_turn_id: correlation.c4os_turn_id.clone(),
            c4os_run_id: correlation.c4os_run_id.clone(),
            correlation_id: correlation.correlation_id.clone(),
            native_session_id: correlation.native_session_id.clone(),
            native_event_id,
            native_event_type: native.native_type,
            process_generation: correlation.process_generation,
            adapter_sequence: self.adapter_sequence,
            received_at_ms,
            replayed: false,
            authenticated_assistant_message,
            category,
        };
        if terminal {
            if let Some(request) = self
                .active_credential_attempts
                .get(&correlation.c4os_session_id)
                .cloned()
            {
                self.command_driver
                    .revoke_provider_credential_attempt(&request)
                    .map_err(AdapterError::CommandFailed)?;
                self.active_credential_attempts
                    .remove(&correlation.c4os_session_id);
            }
            let binding = self
                .sessions
                .get_mut(&correlation.c4os_session_id)
                .ok_or(AdapterError::UnknownSession)?;
            binding.state = match event.category {
                NormalizedEventCategory::Completed => SessionState::Completed,
                NormalizedEventCategory::Cancelled => SessionState::Cancelled,
                NormalizedEventCategory::Error { .. } => SessionState::Interrupted,
                _ => unreachable!(),
            };
            binding.active_run_id = None;
            self.pending_native_permissions
                .retain(|_, pending| pending.correlation != *correlation);
        }
        Ok(event)
    }

    fn remember_native_event_id(&mut self, event_id: String) {
        if self.seen_native_event_ids.len() == MAX_SEEN_EVENT_IDS
            && let Some(oldest) = self.seen_native_event_order.pop_front()
        {
            self.seen_native_event_ids.remove(&oldest);
        }
        self.seen_native_event_ids.insert(event_id.clone());
        self.seen_native_event_order.push_back(event_id);
    }

    pub fn prepare_restart(&self, issued_at_ms: u64) -> Result<RestartTicket, AdapterError> {
        self.require_process()?;
        if self
            .sessions
            .values()
            .any(|session| session.state == SessionState::Running)
        {
            return Err(AdapterError::ActiveRun);
        }
        let prior_generation = self.launch_plan.namespace.process_generation;
        let next_generation = prior_generation
            .checked_add(1)
            .ok_or(AdapterError::RestartGenerationMismatch)?;
        Ok(RestartTicket {
            prior_generation,
            next_generation,
            issued_at_ms,
        })
    }

    pub fn restart(
        &mut self,
        ticket: &RestartTicket,
        next_plan: OpenCodeLaunchPlan,
        checked_at_ms: u64,
    ) -> Result<HealthSnapshot, AdapterError> {
        let expected = self.prepare_restart(ticket.issued_at_ms)?;
        next_plan.validate()?;
        if *ticket != expected
            || next_plan.namespace.process_generation != ticket.next_generation
            || next_plan.namespace.workspace_id != self.launch_plan.namespace.workspace_id
            || next_plan.namespace.root == self.launch_plan.namespace.root
            || next_plan.password_reference == self.launch_plan.password_reference
        {
            return Err(AdapterError::RestartGenerationMismatch);
        }
        self.lifecycle = LifecycleState::Restarting;
        let process = self.process.as_ref().ok_or(AdapterError::NotStarted)?;
        self.command_driver
            .terminate_process_group(process)
            .map_err(AdapterError::CommandFailed)?;
        self.process = None;
        self.health = None;
        self.launch_plan = next_plan;
        self.seen_native_event_ids.clear();
        self.seen_native_event_order.clear();
        self.pending_native_permissions.clear();
        self.active_credential_attempts.clear();
        self.command_driver
            .revoke_all_provider_credential_attempts()
            .map_err(AdapterError::CommandFailed)?;
        // State is isolated per process generation, so previous native session
        // identifiers are not silently rebound to a new OpenCode database.
        self.sessions.clear();
        let command = self.launch_plan.command()?;
        let process = self
            .command_driver
            .spawn(&command)
            .map_err(AdapterError::CommandFailed)?;
        if process.process_generation != ticket.next_generation || process.process_id == 0 {
            if process.process_id != 0 {
                self.command_driver
                    .terminate_process_group(&process)
                    .map_err(AdapterError::CommandFailed)?;
            }
            self.lifecycle = LifecycleState::Degraded;
            return Err(AdapterError::InvalidLaunchPlan);
        }
        self.process = Some(process);
        match self.probe(checked_at_ms) {
            Ok(health) => Ok(health),
            Err(error) => {
                if let Some(process) = self.process.take() {
                    self.command_driver
                        .terminate_process_group(&process)
                        .map_err(AdapterError::CommandFailed)?;
                }
                self.health = None;
                self.lifecycle = LifecycleState::Degraded;
                Err(error)
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), AdapterError> {
        let termination = if let Some(process) = self.process.as_ref() {
            self.command_driver
                .terminate_process_group(process)
                .map_err(AdapterError::CommandFailed)
        } else {
            Ok(())
        };
        let revocation = self
            .command_driver
            .revoke_all_provider_credential_attempts()
            .map_err(AdapterError::CommandFailed);
        termination?;
        self.process = None;
        self.health = None;
        self.lifecycle = LifecycleState::Stopped;
        self.pending_native_permissions.clear();
        self.active_credential_attempts.clear();
        for session in self.sessions.values_mut() {
            if session.state == SessionState::Running {
                session.state = SessionState::Interrupted;
            }
            session.active_run_id = None;
        }
        revocation
    }

    pub fn into_parts(self) -> (T, C) {
        (self.transport, self.command_driver)
    }

    #[doc(hidden)]
    pub fn test_transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    pub(crate) fn event_stream_workspace_root(&self) -> &Path {
        &self.launch_plan.workspace_root
    }

    pub(crate) fn event_stream_auth_identity(&self) -> (&str, &RandomSecretReference) {
        (
            &self.launch_plan.basic_auth_username,
            &self.launch_plan.password_reference,
        )
    }

    pub(crate) fn event_stream_transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    #[doc(hidden)]
    pub fn driver_for_test(&mut self) -> &mut C {
        &mut self.command_driver
    }

    fn request(
        &mut self,
        method: HttpMethod,
        path: String,
        body: Option<Vec<u8>>,
    ) -> Result<TransportResponse, AdapterError> {
        if !path.starts_with('/')
            || path.contains("..")
            || path.bytes().any(|byte| byte.is_ascii_control())
            || body
                .as_ref()
                .is_some_and(|value| value.len() > MAX_JSON_BYTES)
        {
            return Err(AdapterError::InvalidRequest);
        }
        let request = TransportRequest {
            method,
            base_url: self.launch_plan.endpoint.base_url(),
            path,
            body,
            auth: TransportAuth::Basic {
                username: self.launch_plan.basic_auth_username.clone(),
                password_reference: self.launch_plan.password_reference.clone(),
            },
            maximum_response_bytes: MAX_RESPONSE_BYTES,
        };
        let response = self
            .transport
            .execute(request)
            .map_err(AdapterError::TransportFailed)?;
        if response.body.len() > MAX_RESPONSE_BYTES {
            return Err(AdapterError::ResponseTooLarge);
        }
        Ok(response)
    }

    fn require_process(&self) -> Result<(), AdapterError> {
        if self.process.is_none() {
            Err(AdapterError::NotStarted)
        } else {
            Ok(())
        }
    }

    fn require_ready(&self) -> Result<(), AdapterError> {
        if self.lifecycle == LifecycleState::Ready {
            Ok(())
        } else {
            Err(AdapterError::NotReady)
        }
    }

    fn validate_correlation(
        &self,
        correlation: &EventCorrelation,
        require_run: bool,
    ) -> Result<(), AdapterError> {
        if !is_safe_identifier(&correlation.workspace_id)
            || !is_safe_identifier(&correlation.c4os_session_id)
            || !is_safe_identifier(&correlation.c4os_turn_id)
            || !is_safe_identifier(&correlation.c4os_run_id)
            || !is_safe_identifier(&correlation.correlation_id)
            || !is_safe_identifier(&correlation.native_session_id)
            || correlation.process_generation != self.launch_plan.namespace.process_generation
            || correlation.workspace_id != self.launch_plan.namespace.workspace_id
        {
            return Err(AdapterError::StaleCorrelation);
        }
        let binding = self
            .sessions
            .get(&correlation.c4os_session_id)
            .ok_or(AdapterError::UnknownSession)?;
        if binding.native_session_id != correlation.native_session_id
            || binding.process_generation != correlation.process_generation
            || binding.workspace_id != correlation.workspace_id
        {
            return Err(AdapterError::StaleCorrelation);
        }
        if require_run {
            let correlated_run = binding
                .active_run_id
                .as_ref()
                .or(binding.last_run_id.as_ref());
            if correlated_run.is_none_or(|run_id| run_id != &correlation.c4os_run_id) {
                return Err(AdapterError::StaleCorrelation);
            }
        } else if binding.last_run_id.as_ref() == Some(&correlation.c4os_run_id) {
            return Err(AdapterError::StaleCorrelation);
        }
        Ok(())
    }
}

fn validate_prompt_dispatch(dispatch: &PromptDispatch) -> Result<(), AdapterError> {
    let total_attachment_bytes = dispatch
        .attachments
        .iter()
        .try_fold(0_u64, |total, attachment| {
            total.checked_add(attachment.byte_length)
        })
        .ok_or(AdapterError::InvalidRequest)?;
    if !is_safe_route_identifier(&dispatch.model.provider_id)
        || !is_safe_route_identifier(&dispatch.model.model_id)
        || dispatch.text.len() > MAX_PROMPT_BYTES
        || dispatch.text.contains('\0')
        || dispatch.text.trim().is_empty() && dispatch.attachments.is_empty()
        || dispatch.attachments.len() > OPENCODE_MAX_ATTACHMENTS
        || total_attachment_bytes > OPENCODE_MAX_INLINE_ATTACHMENT_BYTES
        || dispatch.attachments.iter().any(|attachment| {
            !is_safe_identifier(&attachment.attachment_id)
                || !is_safe_route_identifier(&attachment.stable_reference)
                || !is_bounded_text(&attachment.display_name, 512)
                || !is_bounded_text(&attachment.media_type, 256)
                || attachment.byte_length == 0
                || attachment.byte_length > OPENCODE_MAX_INLINE_ATTACHMENT_BYTES
                || !is_sha256(&attachment.content_sha256)
                || attachment.snapshot_version == 0
                || attachment.content.len() as u64 != attachment.byte_length
                || sha256(&attachment.content) != attachment.content_sha256
        })
        || dispatch.eligible_tool_ids.iter().any(|tool_id| {
            !is_safe_identifier(tool_id)
                || !matches!(
                    tool_id.as_str(),
                    C4OS_ACTION_PROPOSAL_TOOL | C4OS_RESOURCE_READ_TOOL
                )
        })
    {
        return Err(AdapterError::InvalidRequest);
    }
    let overrides = dispatch
        .native_overrides
        .as_object()
        .ok_or(AdapterError::InvalidRequest)?;
    for (key, value) in overrides {
        match key.as_str() {
            "system" | "format" => {
                if serde_json::to_vec(value).map_or(true, |encoded| encoded.len() > MAX_JSON_BYTES)
                {
                    return Err(AdapterError::InvalidRequest);
                }
            }
            "agent" | "tools" | "tool" | "permission" | "permissions" | "command" | "shell" => {
                return Err(AdapterError::AuthorityOverrideRejected);
            }
            _ => return Err(AdapterError::InvalidRequest),
        }
    }
    Ok(())
}

fn normalize_native_category(
    correlation: &EventCorrelation,
    native: &NativeEventEnvelope,
    native_event_id: &str,
) -> Result<NormalizedEventCategory, AdapterError> {
    let properties = native
        .properties
        .as_object()
        .ok_or(AdapterError::InvalidEventPayload)?;
    match native.native_type.as_str() {
        "server.connected" | "session.created" | "session.updated" => {
            Ok(NormalizedEventCategory::Lifecycle {
                state: native.native_type.clone(),
            })
        }
        "message.part.updated" => normalize_message_part_update(properties, native_event_id),
        "message.part.delta" | "session.next.text.delta" => {
            let delta = required_bounded_string(properties, "delta", MAX_DELTA_BYTES)?;
            let field = properties
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("text");
            if field == "reasoning" || field == "thinking" {
                Ok(NormalizedEventCategory::ThinkingDelta { delta })
            } else {
                Ok(NormalizedEventCategory::TextDelta { delta })
            }
        }
        "session.next.reasoning.delta" => Ok(NormalizedEventCategory::ThinkingDelta {
            delta: required_bounded_string(properties, "delta", MAX_DELTA_BYTES)?,
        }),
        "permission.v2.asked" | "permission.asked" | "session.next.tool.called" => {
            Ok(NormalizedEventCategory::ActionIntent(Box::new(
                action_intent_from_native(correlation, native, native_event_id)?,
            )))
        }
        "session.next.tool.progress" => Ok(NormalizedEventCategory::ToolProgress {
            tool_call_id: optional_safe_string(properties, &["callID", "toolCallID", "id"])
                .unwrap_or_else(|| native_event_id.to_owned()),
            summary: properties
                .get("title")
                .or_else(|| properties.get("summary"))
                .and_then(Value::as_str)
                .filter(|summary| is_bounded_text(summary, MAX_TITLE_BYTES))
                .unwrap_or("Tool progress")
                .to_string(),
        }),
        "session.next.tool.success" | "session.next.tool.failed" => {
            Ok(NormalizedEventCategory::ToolResult {
                tool_call_id: optional_safe_string(properties, &["callID", "toolCallID", "id"])
                    .unwrap_or_else(|| native_event_id.to_owned()),
                succeeded: native.native_type.ends_with("success"),
            })
        }
        "session.usage" | "message.usage" => Ok(NormalizedEventCategory::Usage {
            input_tokens: bounded_u64(properties, "inputTokens")?,
            output_tokens: bounded_u64(properties, "outputTokens")?,
        }),
        "session.idle" | "session.completed" => Ok(NormalizedEventCategory::Completed),
        "session.next.step.ended" => Ok(NormalizedEventCategory::Lifecycle {
            state: "provider-step-ended".into(),
        }),
        "session.cancelled" | "session.aborted" => Ok(NormalizedEventCategory::Cancelled),
        "session.error" | "session.next.step.failed" => Ok(NormalizedEventCategory::Error {
            code: "native-session-error".into(),
        }),
        "session.status" => {
            let state = properties
                .get("status")
                .and_then(|status| {
                    status
                        .as_str()
                        .or_else(|| status.get("type").and_then(Value::as_str))
                })
                .filter(|status| is_bounded_text(status, MAX_IDENTIFIER_BYTES))
                .unwrap_or("unknown")
                .to_string();
            Ok(NormalizedEventCategory::Lifecycle { state })
        }
        _ => Ok(NormalizedEventCategory::Unknown),
    }
}

fn normalize_message_part_update(
    properties: &serde_json::Map<String, Value>,
    native_event_id: &str,
) -> Result<NormalizedEventCategory, AdapterError> {
    let part = properties
        .get("part")
        .and_then(Value::as_object)
        .ok_or(AdapterError::InvalidEventPayload)?;
    let part_id = optional_safe_string(part, &["id"]).unwrap_or_else(|| native_event_id.to_owned());
    let part_type = part
        .get("type")
        .and_then(Value::as_str)
        .filter(|value| is_safe_identifier(value))
        .ok_or(AdapterError::InvalidEventPayload)?;
    match part_type {
        "text" | "reasoning" => {
            let Some(delta) = properties.get("delta").and_then(Value::as_str) else {
                return Ok(NormalizedEventCategory::Unknown);
            };
            if delta.len() > MAX_DELTA_BYTES || delta.chars().any(|character| character == '\0') {
                return Err(AdapterError::InvalidEventPayload);
            }
            if part_type == "reasoning" {
                Ok(NormalizedEventCategory::ThinkingDelta {
                    delta: delta.to_owned(),
                })
            } else {
                Ok(NormalizedEventCategory::TextDelta {
                    delta: delta.to_owned(),
                })
            }
        }
        "tool" => {
            let call_id = optional_safe_string(part, &["callID"]).unwrap_or(part_id);
            let state = part
                .get("state")
                .and_then(Value::as_object)
                .ok_or(AdapterError::InvalidEventPayload)?;
            let status = state
                .get("status")
                .and_then(Value::as_str)
                .ok_or(AdapterError::InvalidEventPayload)?;
            match status {
                "pending" | "running" => {
                    let summary = state
                        .get("title")
                        .and_then(Value::as_str)
                        .filter(|value| is_bounded_text(value, MAX_TITLE_BYTES))
                        .unwrap_or("Tool progress")
                        .to_owned();
                    Ok(NormalizedEventCategory::ToolProgress {
                        tool_call_id: call_id,
                        summary,
                    })
                }
                "completed" | "error" => Ok(NormalizedEventCategory::ToolResult {
                    tool_call_id: call_id,
                    succeeded: status == "completed",
                }),
                _ => Err(AdapterError::InvalidEventPayload),
            }
        }
        _ => Ok(NormalizedEventCategory::Unknown),
    }
}

fn native_event_session_id(native: &NativeEventEnvelope) -> Option<&str> {
    let properties = native.properties.as_object()?;
    properties
        .get("sessionID")
        .and_then(Value::as_str)
        .or_else(|| {
            properties
                .get("part")
                .and_then(Value::as_object)
                .and_then(|part| part.get("sessionID"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            properties
                .get("info")
                .and_then(Value::as_object)
                .and_then(|info| info.get("sessionID"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            matches!(
                native.native_type.as_str(),
                "session.created" | "session.updated" | "session.deleted"
            )
            .then(|| {
                properties
                    .get("info")
                    .and_then(Value::as_object)
                    .and_then(|info| info.get("id"))
                    .and_then(Value::as_str)
            })
            .flatten()
        })
}

fn authenticated_assistant_message(
    native: &NativeEventEnvelope,
) -> Result<Option<AuthenticatedAssistantMessage>, AdapterError> {
    if native.native_type != "message.updated" {
        return Ok(None);
    }
    let info = native
        .properties
        .as_object()
        .and_then(|properties| properties.get("info"))
        .and_then(Value::as_object)
        .ok_or(AdapterError::InvalidEventPayload)?;
    let role = info
        .get("role")
        .and_then(Value::as_str)
        .ok_or(AdapterError::InvalidEventPayload)?;
    if role != "assistant" {
        return Ok(None);
    }
    let native_message_id = info
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| is_safe_identifier(value))
        .ok_or(AdapterError::InvalidEventPayload)?;
    let parent_native_message_id = info
        .get("parentID")
        .and_then(Value::as_str)
        .filter(|value| is_safe_identifier(value))
        .ok_or(AdapterError::InvalidEventPayload)?;
    Ok(Some(AuthenticatedAssistantMessage {
        native_message_id: native_message_id.to_owned(),
        parent_native_message_id: parent_native_message_id.to_owned(),
        role: role.to_owned(),
    }))
}

fn is_global_native_event(native_type: &str) -> bool {
    matches!(
        native_type,
        "server.connected"
            | "installation.updated"
            | "installation.update-available"
            | "global.disposed"
            | "server.instance.disposed"
    )
}

fn action_intent_from_native(
    correlation: &EventCorrelation,
    native: &NativeEventEnvelope,
    native_event_id: &str,
) -> Result<ActionIntent, AdapterError> {
    let properties = native
        .properties
        .as_object()
        .ok_or(AdapterError::InvalidEventPayload)?;
    let native_request_id = optional_safe_string(properties, &["id", "requestID"])
        .unwrap_or_else(|| native_event_id.to_owned());
    let native_tool = optional_safe_string(
        properties,
        &["action", "permission", "tool", "toolID", "name"],
    )
    .ok_or(AdapterError::InvalidEventPayload)?;
    let native_arguments = properties
        .get("metadata")
        .or_else(|| properties.get("input"))
        .or_else(|| properties.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    if serde_json::to_vec(&native_arguments).map_or(true, |encoded| encoded.len() > MAX_JSON_BYTES)
    {
        return Err(AdapterError::InvalidEventPayload);
    }
    let resources = properties
        .get("resources")
        .or_else(|| properties.get("patterns"))
        .and_then(Value::as_array)
        .map(|resources| {
            if resources.len() > MAX_RESOURCES {
                return Err(AdapterError::InvalidEventPayload);
            }
            resources
                .iter()
                .map(|resource| {
                    let resource = resource.as_str().ok_or(AdapterError::InvalidEventPayload)?;
                    if !is_bounded_text(resource, MAX_TITLE_BYTES) {
                        return Err(AdapterError::InvalidEventPayload);
                    }
                    Ok(resource.to_string())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(ActionIntent {
        workspace_id: correlation.workspace_id.clone(),
        c4os_session_id: correlation.c4os_session_id.clone(),
        c4os_turn_id: correlation.c4os_turn_id.clone(),
        c4os_run_id: correlation.c4os_run_id.clone(),
        correlation_id: correlation.correlation_id.clone(),
        runtime_id: "opencode".into(),
        process_generation: correlation.process_generation,
        native_session_id: correlation.native_session_id.clone(),
        native_request_id,
        native_tool,
        native_arguments,
        resources,
    })
}

struct ParsedSseFrame<'a> {
    data: &'a [u8],
    id: Option<&'a str>,
}

fn parse_sse_frame(frame: &[u8]) -> Result<ParsedSseFrame<'_>, AdapterError> {
    let text = std::str::from_utf8(frame).map_err(|_| AdapterError::InvalidEventPayload)?;
    let mut data = None;
    let mut id = None;
    for line in text.lines() {
        if line.starts_with(':') || line.starts_with("event:") {
            continue;
        }
        if let Some(value) = line.strip_prefix("id:") {
            let value = value.trim_start();
            if id.replace(value).is_some()
                || value.len() > MAX_IDENTIFIER_BYTES
                || value.chars().any(char::is_control)
            {
                return Err(AdapterError::InvalidEventPayload);
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("data:") {
            if data.is_some() {
                return Err(AdapterError::InvalidEventPayload);
            }
            data = Some(value.trim_start().as_bytes());
        } else if !line.trim().is_empty() {
            return Err(AdapterError::InvalidEventPayload);
        }
    }
    Ok(ParsedSseFrame {
        data: data.ok_or(AdapterError::InvalidEventPayload)?,
        id,
    })
}

fn deterministic_native_event_id(
    sse_id: Option<&str>,
    native: &NativeEventEnvelope,
) -> Result<String, AdapterError> {
    let mut material = Vec::new();
    if let Some(sse_id) = sse_id {
        material.extend_from_slice(sse_id.as_bytes());
    }
    material.push(0);
    serde_json::to_writer(&mut material, native).map_err(|_| AdapterError::InvalidEventPayload)?;
    if material.len() > MAX_SSE_FRAME_BYTES + MAX_IDENTIFIER_BYTES + 1 {
        return Err(AdapterError::ResponseTooLarge);
    }
    let digest = sha256(&material);
    Ok(format!("event-{}", digest.trim_start_matches("sha256:")))
}

fn encode_bounded(value: &Value) -> Result<Vec<u8>, AdapterError> {
    let encoded = serde_json::to_vec(value).map_err(|_| AdapterError::InvalidRequest)?;
    if encoded.len() > MAX_JSON_BYTES {
        return Err(AdapterError::ResponseTooLarge);
    }
    Ok(encoded)
}

fn decode_bounded<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    maximum: usize,
    invalid: AdapterError,
) -> Result<T, AdapterError> {
    if bytes.len() > maximum {
        return Err(AdapterError::ResponseTooLarge);
    }
    serde_json::from_slice(bytes).map_err(|_| invalid)
}

fn enabled_keys(values: BTreeMap<String, bool>) -> BTreeSet<String> {
    values
        .into_iter()
        .filter_map(|(key, enabled)| enabled.then_some(key))
        .collect()
}

fn required_bounded_string(
    properties: &serde_json::Map<String, Value>,
    key: &str,
    maximum: usize,
) -> Result<String, AdapterError> {
    let value = properties
        .get(key)
        .and_then(Value::as_str)
        .ok_or(AdapterError::InvalidEventPayload)?;
    if !is_bounded_text(value, maximum) {
        return Err(AdapterError::InvalidEventPayload);
    }
    Ok(value.to_string())
}

fn optional_safe_string(
    properties: &serde_json::Map<String, Value>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|key| {
        properties
            .get(*key)
            .and_then(Value::as_str)
            .filter(|value| is_safe_route_identifier(value))
            .map(ToString::to_string)
    })
}

fn bounded_u64(
    properties: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<u64, AdapterError> {
    properties
        .get(key)
        .and_then(Value::as_u64)
        .filter(|value| *value <= u64::MAX / 2)
        .ok_or(AdapterError::InvalidEventPayload)
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

fn is_safe_route_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@' | b'/' | b':')
        })
        && !value.contains("..")
}

fn is_bounded_text(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value.chars().any(|character| character == '\0')
}

fn path_text(path: &Path) -> Result<String, AdapterError> {
    path.to_str()
        .filter(|value| !value.chars().any(char::is_control))
        .map(ToString::to_string)
        .ok_or(AdapterError::InvalidLaunchPlan)
}

fn workspace_identity_text(path: &Path) -> Result<String, AdapterError> {
    match path.canonicalize() {
        Ok(canonical) => path_text(&canonical),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => path_text(path),
        Err(_) => Err(AdapterError::InvalidLaunchPlan),
    }
}

fn sha256(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}
