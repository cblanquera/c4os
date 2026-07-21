//! Rust-owned dispatch routing between durable C4OS attempts and peer runtimes.
//!
//! A peer must match the complete Workspace/runtime/version/process binding
//! before the coordinator is allowed to promote a provisional Chat. Runtime
//! acceptance follows the durable first-submit transition; a native rejection
//! is closed into a terminal attempt instead of being reported as success.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::runtime::action_bridge::{RuntimeExecutionReceipt, RuntimeIntentIdentity};
use crate::runtime::adapter::{AdapterConformanceDescriptor, AdapterContractError};
use crate::runtime::attachment_materializer::{AttachmentContentPlan, VerifiedAttachmentContent};
use crate::runtime::broker_worker::BrokerActionContext;
use crate::runtime::capability::{DraftRequirements, PreflightOutcome};
use crate::runtime::coordinator::{
    CoordinatedFirstSubmission, CoordinatedRetry, CoordinatedTurn, CoordinatorError,
    CoordinatorOperation, RuntimeCoordinator,
};
use crate::runtime::opencode::{
    AdapterError as OpenCodeError, CommandDriver, EventCorrelation, ModelRoute, NormalizedEvent,
    NormalizedEventCategory, OPENCODE_NATIVE_VERSION, OpenCodeAdapter, OpenCodeTransport,
    PromptAttachment, PromptDispatch,
};
use crate::runtime::opencode_broker::{
    ActiveBrokerContextResolver, AuthenticatedAssistantMessageEvidence,
};
use crate::runtime::opencode_credential::opencode_message_id_for_operation;
#[cfg(unix)]
use crate::runtime::opencode_stream::{
    OpenCodeEventSubscription, OpenCodeStreamBounds, OpenCodeStreamWorker,
};
use crate::runtime::pi::{
    PI_NATIVE_VERSION, PiAdapter, PiAdapterState, PiDispatchAttachment, PiEventEnvelope,
    PiModelRoute, PiSidecarRunner, PiToolDecision,
};
use crate::runtime::session::{
    AttachmentSnapshot, AttemptIdentity, ResourceSnapshot, RunEventKind, RunEventRecord,
    SessionRecord, SessionRepository, TerminalAttemptOutcome,
};
use crate::runtime::supervisor::RuntimeKind;
use crate::security::policy::ActionRequestOrigin;

const DISPATCH_ADAPTER_VERSION: &str = "1.0.0";
const MAX_ID_BYTES: usize = 192;
const MAX_INPUT_BYTES: usize = 512 * 1024;
const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * 1024;
const MAX_PEERS: usize = 32;
const MAX_PENDING_SSE_FRAMES: usize = 4_096;
const MAX_ATTACHMENT_MATERIALIZATIONS: usize = 32;
const MAX_DIRECT_ATTACHMENT_BYTES: u64 = 64 * 1024 * 1024;
const CONVERTER_OUTPUT_MEDIA_TYPE: &str = "text/plain; charset=utf-8";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimePeerRegistration {
    pub runtime_id: String,
    pub workspace_id: String,
    pub descriptor: AdapterConformanceDescriptor,
}

impl RuntimePeerRegistration {
    pub fn validate(&self) -> Result<(), DispatchError> {
        validate_id(&self.runtime_id)?;
        validate_id(&self.workspace_id)?;
        self.descriptor.validate()?;
        let exact_native = match self.descriptor.runtime_kind {
            RuntimeKind::OpenCode => OPENCODE_NATIVE_VERSION,
            RuntimeKind::Pi => PI_NATIVE_VERSION,
        };
        if self.descriptor.adapter_version != DISPATCH_ADAPTER_VERSION
            || self.descriptor.native_version != exact_native
        {
            return Err(DispatchError::IncompatiblePeer);
        }
        for capability in [
            "session-create",
            "streaming",
            "cancellation",
            "action-intents",
        ] {
            if !self
                .descriptor
                .capabilities
                .get(capability)
                .is_some_and(|state| state.usable())
            {
                return Err(DispatchError::IncompatiblePeer);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DispatchIdentity {
    pub workspace_id: String,
    pub environment_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub attempt_id: String,
    pub correlation_id: String,
    pub runtime_id: String,
    pub runtime_kind: RuntimeKind,
    pub adapter_version: String,
    pub native_version: String,
    pub process_generation: u64,
}

impl DispatchIdentity {
    pub fn validate(&self) -> Result<(), DispatchError> {
        for value in [
            &self.workspace_id,
            &self.environment_id,
            &self.session_id,
            &self.turn_id,
            &self.attempt_id,
            &self.correlation_id,
            &self.runtime_id,
            &self.adapter_version,
            &self.native_version,
        ] {
            validate_id(value)?;
        }
        if self.process_generation == 0 {
            return Err(DispatchError::InvalidIdentity);
        }
        Ok(())
    }

    pub(crate) fn attempt_identity(&self) -> AttemptIdentity {
        AttemptIdentity {
            attempt_id: self.attempt_id.clone(),
            correlation_id: self.correlation_id.clone(),
            process_generation: self.process_generation,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchModelRoute {
    pub provider_id: String,
    pub model_id: String,
    pub credential_reference: Option<String>,
    pub credential_lease_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerDispatchAuthority {
    pub request_origin: ActionRequestOrigin,
    pub configuration_version: u64,
    pub policy_version: u64,
    pub revocation_epoch: u64,
    pub active_from_ms: u64,
    pub expires_at_ms: u64,
}

impl BrokerDispatchAuthority {
    fn validate(&self) -> Result<(), DispatchError> {
        if self.request_origin == ActionRequestOrigin::Unknown
            || self.configuration_version == 0
            || self.policy_version == 0
            || self.active_from_ms == 0
            || self.expires_at_ms <= self.active_from_ms
        {
            return Err(DispatchError::InvalidRequest);
        }
        Ok(())
    }
}

impl DispatchModelRoute {
    fn validate(&self) -> Result<(), DispatchError> {
        validate_id(&self.provider_id)?;
        validate_id(&self.model_id)?;
        for value in [
            self.credential_reference.as_deref(),
            self.credential_lease_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            validate_id(value)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerDispatchRequest {
    pub identity: DispatchIdentity,
    pub model: DispatchModelRoute,
    pub title: String,
    pub input: String,
    /// Exact core-verified broker tools eligible for this native session/run.
    /// This is derived from the authoritative installed-resource snapshot,
    /// never from an unverified renderer or peer request.
    pub eligible_tool_ids: BTreeSet<String>,
    pub broker_authority: Option<BrokerDispatchAuthority>,
    /// Exact immutable attachment metadata plus bytes verified from the bound
    /// Workspace content-addressed store. No peer resolves a renderer path or
    /// receives ambient filesystem authority.
    pub direct_attachments: Vec<VerifiedAttachmentContent>,
    /// Core-verified converter receipts bound to the durable attachment
    /// snapshots. Peers receive this provenance even though their current
    /// transport consumes only the deterministic converted input.
    pub attachments: Vec<ConvertedAttachmentMaterialization>,
}

impl PeerDispatchRequest {
    fn validate(&self) -> Result<(), DispatchError> {
        self.identity.validate()?;
        self.model.validate()?;
        if let Some(authority) = &self.broker_authority {
            authority.validate()?;
        }
        if self.title.trim().is_empty()
            || self.title.len() > 1_024
            || self.input.trim().is_empty() && self.direct_attachments.is_empty()
            || self.input.len() > MAX_INPUT_BYTES
            || self.direct_attachments.len() > MAX_ATTACHMENT_MATERIALIZATIONS
            || self.attachments.len() > MAX_ATTACHMENT_MATERIALIZATIONS
            || !self.direct_attachments.is_empty() && !self.attachments.is_empty()
            || self
                .eligible_tool_ids
                .iter()
                .any(|tool_id| validate_id(tool_id).is_err())
        {
            return Err(DispatchError::InvalidRequest);
        }
        let mut attachment_ids = BTreeSet::new();
        for attachment in &self.direct_attachments {
            validate_verified_attachment(attachment)?;
            if !attachment_ids.insert(attachment.snapshot().attachment_id.as_str()) {
                return Err(DispatchError::InvalidDirectAttachment);
            }
        }
        Ok(())
    }
}

/// Complete source, converter, installed-resource, and output binding for one
/// durable attachment. No renderer path or renderer-provided byte buffer is
/// accepted by this dispatch boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvertedAttachmentMaterialization {
    attachment_id: String,
    stable_reference: String,
    display_name: String,
    content_sha256: String,
    byte_length: u64,
    media_type: String,
    snapshot_version: u64,
    converter_id: String,
    converter_version: String,
    converter_snapshot_id: String,
    converter_snapshot_version: u64,
    converter_snapshot_sha256: String,
    resource_snapshot_id: String,
    resource_snapshot_version: u64,
    resource_snapshot_sha256: String,
    output_reference: String,
    output_sha256: String,
    output_byte_length: u64,
    output_media_type: String,
    output: String,
}

/// Untrusted converter metadata. It becomes usable authority only after its
/// exact binding matches the digest in a core-owned ResourceSnapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledAttachmentConverterDescriptor {
    pub converter_id: String,
    pub converter_version: String,
    pub snapshot_id: String,
    pub snapshot_version: u64,
    pub snapshot_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledAttachmentConverterAuthority {
    descriptor: InstalledAttachmentConverterDescriptor,
    resource_snapshot_id: String,
    resource_snapshot_version: u64,
    resource_snapshot_sha256: String,
}

impl InstalledAttachmentConverterDescriptor {
    /// Computes the digest that a core-owned resource snapshot must have
    /// sealed for this exact installed converter descriptor.
    pub fn expected_resource_snapshot_sha256(
        &self,
        resources: &ResourceSnapshot,
    ) -> Result<String, DispatchError> {
        validate_converter_descriptor(self, resources)?;
        Ok(converter_resource_binding_sha256(self, resources))
    }

    pub fn verify(
        self,
        resources: &ResourceSnapshot,
    ) -> Result<InstalledAttachmentConverterAuthority, DispatchError> {
        validate_converter_descriptor(&self, resources)?;
        if resources.sha256 != converter_resource_binding_sha256(&self, resources) {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
        Ok(InstalledAttachmentConverterAuthority {
            descriptor: self,
            resource_snapshot_id: resources.snapshot_id.clone(),
            resource_snapshot_version: resources.version,
            resource_snapshot_sha256: resources.sha256.clone(),
        })
    }
}

impl InstalledAttachmentConverterAuthority {
    /// Mints one opaque converter receipt. The authority cannot be created
    /// unless its installation metadata was sealed by the resource snapshot.
    pub fn materialize(
        &self,
        attachment: &AttachmentSnapshot,
        resources: &ResourceSnapshot,
        output_reference: impl Into<String>,
        output: impl Into<String>,
    ) -> Result<ConvertedAttachmentMaterialization, DispatchError> {
        if self.resource_snapshot_id != resources.snapshot_id
            || self.resource_snapshot_version != resources.version
            || self.resource_snapshot_sha256 != resources.sha256
            || resources.sha256 != converter_resource_binding_sha256(&self.descriptor, resources)
        {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
        let output_reference = output_reference.into();
        let output = output.into();
        validate_id(&output_reference)
            .map_err(|_| DispatchError::InvalidAttachmentMaterialization)?;
        if output.trim().is_empty() || output.contains('\0') || output.len() > MAX_INPUT_BYTES {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
        Ok(ConvertedAttachmentMaterialization {
            attachment_id: attachment.attachment_id.clone(),
            stable_reference: attachment.stable_reference.clone(),
            display_name: attachment.display_name.clone(),
            content_sha256: attachment.content_sha256.clone(),
            byte_length: attachment.byte_length,
            media_type: attachment.media_type.clone(),
            snapshot_version: attachment.snapshot_version,
            converter_id: self.descriptor.converter_id.clone(),
            converter_version: self.descriptor.converter_version.clone(),
            converter_snapshot_id: self.descriptor.snapshot_id.clone(),
            converter_snapshot_version: self.descriptor.snapshot_version,
            converter_snapshot_sha256: self.descriptor.snapshot_sha256.clone(),
            resource_snapshot_id: resources.snapshot_id.clone(),
            resource_snapshot_version: resources.version,
            resource_snapshot_sha256: resources.sha256.clone(),
            output_reference,
            output_sha256: sha256_prefixed(output.as_bytes()),
            output_byte_length: output.len() as u64,
            output_media_type: CONVERTER_OUTPUT_MEDIA_TYPE.into(),
            output,
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AttachmentMaterializationPlan {
    attachments: Vec<ConvertedAttachmentMaterialization>,
}

impl AttachmentMaterializationPlan {
    pub fn from_verified(attachments: Vec<ConvertedAttachmentMaterialization>) -> Self {
        Self { attachments }
    }

    pub fn attachments(&self) -> &[ConvertedAttachmentMaterialization] {
        &self.attachments
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum AttachmentPreflightResolution {
    #[default]
    NotRequired,
    Direct(AttachmentContentPlan),
    Materialize(AttachmentMaterializationPlan),
    Remove {
        attachment_ids: Vec<String>,
    },
    Cancel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiActionIntent {
    identity: RuntimeIntentIdentity,
    arguments: Value,
}

impl PiActionIntent {
    pub(crate) fn identity(&self) -> &RuntimeIntentIdentity {
        &self.identity
    }

    pub(crate) fn arguments(&self) -> &Value {
        &self.arguments
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchEventCategory {
    Lifecycle,
    TextDelta,
    ReasoningDelta,
    /// Provider-supplied display-safe summary normalized separately from raw
    /// thinking deltas by an app-owned adapter.
    ReasoningSummary,
    ActionIntent(Box<RuntimeIntentIdentity>),
    /// Pi action material retained only after the sidecar event crossed the
    /// generation, active-run, sequence, authority, and payload validators.
    /// Private fields prevent callers from labelling arbitrary JSON as a
    /// production Pi tool proposal.
    PiActionIntent(Box<PiActionIntent>),
    ActionProgress,
    ActionResult,
    Usage,
    Completed,
    Cancelled,
    Error {
        code: String,
    },
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerDispatchEvent {
    pub identity: DispatchIdentity,
    pub recorded_at_ms: u64,
    pub payload: String,
    pub category: DispatchEventCategory,
}

impl PeerDispatchEvent {
    fn validate(&self) -> Result<(), DispatchError> {
        self.identity.validate()?;
        if self.recorded_at_ms == 0 || self.payload.len() > MAX_EVENT_PAYLOAD_BYTES {
            return Err(DispatchError::InvalidEvent);
        }
        if let DispatchEventCategory::Error { code } = &self.category {
            validate_id(code)?;
        }
        if let DispatchEventCategory::ActionIntent(intent) = &self.category {
            validate_sha256(&intent.binding_sha256)?;
            for value in [&intent.native_request_id, &intent.native_tool] {
                validate_id(value)?;
            }
            if intent.workspace_id != self.identity.workspace_id
                || intent.session_id != self.identity.session_id
                || intent.turn_id != self.identity.turn_id
                || intent.run_id != self.identity.attempt_id
                || intent.correlation_id != self.identity.correlation_id
                || intent.runtime_id != self.identity.runtime_id
                || intent.process_generation != self.identity.process_generation
            {
                return Err(DispatchError::InvalidEvent);
            }
        }
        if let DispatchEventCategory::PiActionIntent(intent) = &self.category {
            let identity = intent.identity();
            validate_sha256(&identity.binding_sha256)?;
            for value in [&identity.native_request_id, &identity.native_tool] {
                validate_id(value)?;
            }
            if self.identity.runtime_kind != RuntimeKind::Pi
                || identity.workspace_id != self.identity.workspace_id
                || identity.session_id != self.identity.session_id
                || identity.turn_id != self.identity.turn_id
                || identity.run_id != self.identity.attempt_id
                || identity.correlation_id != self.identity.correlation_id
                || identity.runtime_id != self.identity.runtime_id
                || identity.process_generation != self.identity.process_generation
                || !intent.arguments().is_object()
                || serde_json::to_vec(intent.arguments())
                    .map_or(true, |encoded| encoded.len() > MAX_EVENT_PAYLOAD_BYTES)
            {
                return Err(DispatchError::InvalidEvent);
            }
        }
        Ok(())
    }

    fn terminal_outcome(&self) -> Option<TerminalAttemptOutcome> {
        match &self.category {
            DispatchEventCategory::Completed => Some(TerminalAttemptOutcome::Completed),
            DispatchEventCategory::Cancelled => Some(TerminalAttemptOutcome::Cancelled),
            DispatchEventCategory::Error { code } => Some(TerminalAttemptOutcome::Failed {
                error_code: code.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchEvent {
    pub sequence: u64,
    pub peer: PeerDispatchEvent,
}

impl DispatchEvent {
    fn run_event(&self) -> RunEventRecord {
        let kind = match self.peer.category {
            DispatchEventCategory::Lifecycle => RunEventKind::Status,
            DispatchEventCategory::TextDelta => RunEventKind::TextDelta,
            DispatchEventCategory::ReasoningDelta => RunEventKind::ReasoningDelta,
            DispatchEventCategory::ReasoningSummary => RunEventKind::ReasoningSummary,
            DispatchEventCategory::ActionIntent(_) | DispatchEventCategory::PiActionIntent(_) => {
                RunEventKind::ActionIntent
            }
            DispatchEventCategory::ActionProgress => RunEventKind::ActionProgress,
            DispatchEventCategory::ActionResult => RunEventKind::ActionResult,
            DispatchEventCategory::Usage => RunEventKind::Usage,
            DispatchEventCategory::Completed | DispatchEventCategory::Cancelled => {
                RunEventKind::Completion
            }
            DispatchEventCategory::Error { .. } => RunEventKind::Error,
            DispatchEventCategory::Other => RunEventKind::WorkActivity,
        };
        RunEventRecord {
            sequence: self.sequence,
            session_id: self.peer.identity.session_id.clone(),
            turn_id: self.peer.identity.turn_id.clone(),
            attempt_id: self.peer.identity.attempt_id.clone(),
            runtime_id: self.peer.identity.runtime_id.clone(),
            environment_id: self.peer.identity.environment_id.clone(),
            correlation_id: self.peer.identity.correlation_id.clone(),
            process_generation: self.peer.identity.process_generation,
            kind,
            payload: self.peer.payload.clone(),
            recorded_at_ms: self.peer.recorded_at_ms,
        }
    }
}

pub trait RuntimeDispatchPeer: Send {
    fn registration(&self) -> &RuntimePeerRegistration;
    fn readiness(&self) -> Result<(), PeerDispatchError>;
    fn create_session(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError>;
    fn activate_broker_context(
        &mut self,
        request: &PeerDispatchRequest,
    ) -> Result<(), PeerDispatchError> {
        if request.broker_authority.is_some() {
            Err(PeerDispatchError::Dispatch)
        } else {
            Ok(())
        }
    }
    fn dispatch(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError>;
    /// Poll accepted native events. `recorded_at_ms` is supplied by the
    /// Rust-owned worker clock for protocols, such as Pi RPC, that do not
    /// carry an authoritative receive timestamp.
    fn poll_events(
        &mut self,
        recorded_at_ms: u64,
    ) -> Result<Vec<PeerDispatchEvent>, PeerDispatchError>;
    fn cancel(&mut self, identity: &DispatchIdentity) -> Result<bool, PeerDispatchError>;

    /// Terminates the exact registered native peer through its protocol-aware
    /// adapter path. Generic peers fail closed unless they implement shutdown.
    fn shutdown(&mut self) -> Result<(), PeerDispatchError> {
        Err(PeerDispatchError::UnsupportedShutdown)
    }

    fn resolve_pi_denied(
        &mut self,
        _identity: &DispatchIdentity,
        _native_request_id: &str,
        _reason_code: &str,
    ) -> Result<(), PeerDispatchError> {
        Err(PeerDispatchError::UnsupportedToolResolution)
    }

    fn resolve_pi_completed(
        &mut self,
        _identity: &DispatchIdentity,
        _native_request_id: &str,
        _receipt: &RuntimeExecutionReceipt,
    ) -> Result<(), PeerDispatchError> {
        Err(PeerDispatchError::UnsupportedToolResolution)
    }

    /// Forget only Rust-owned in-memory correlation after terminal closure or
    /// recovery. This does not execute a native side effect.
    fn forget_attempt(&mut self, _identity: &DispatchIdentity) {}
}

/// Core-supplied Pi provider mapping and one-use credential-delivery boundary.
/// Implementations remain Rust-owned; native peers receive only the derived
/// provider ID and operation-scoped delivery result.
pub trait PiDispatchCredentialIssuer: Send {
    fn native_provider_id(&self, provider_id: &str) -> Result<String, PeerDispatchError>;
    fn base_url(&self, provider_id: &str) -> Result<String, PeerDispatchError>;

    fn deliver_for_dispatch(
        &mut self,
        identity: &DispatchIdentity,
        provider_id: &str,
    ) -> Result<(), PeerDispatchError>;
}

pub struct RuntimeDispatchRegistry {
    peers: BTreeMap<String, Box<dyn RuntimeDispatchPeer>>,
    event_sequences: BTreeMap<DispatchIdentity, u64>,
}

impl Default for RuntimeDispatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeDispatchRegistry {
    pub fn new() -> Self {
        Self {
            peers: BTreeMap::new(),
            event_sequences: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn register<P: RuntimeDispatchPeer + 'static>(
        &mut self,
        peer: P,
    ) -> Result<(), DispatchError> {
        let registration = peer.registration();
        registration.validate()?;
        if self.peers.len() >= MAX_PEERS || self.peers.contains_key(&registration.runtime_id) {
            return Err(DispatchError::DuplicatePeer);
        }
        self.peers
            .insert(registration.runtime_id.clone(), Box::new(peer));
        Ok(())
    }

    /// Returns the exact Rust-owned peer registration used for process-truth
    /// minting. No executable path, secret, or mutable peer handle escapes.
    pub fn registration(&self, runtime_id: &str) -> Result<RuntimePeerRegistration, DispatchError> {
        validate_id(runtime_id)?;
        self.peers
            .get(runtime_id)
            .map(|peer| peer.registration().clone())
            .ok_or(DispatchError::PeerUnavailable)
    }

    pub fn ensure_ready(&self, identity: &DispatchIdentity) -> Result<(), DispatchError> {
        identity.validate()?;
        let peer = self
            .peers
            .get(&identity.runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?;
        validate_registration(peer.registration(), identity)?;
        peer.readiness().map_err(DispatchError::Peer)
    }

    pub fn dispatch_first(&mut self, request: &PeerDispatchRequest) -> Result<(), DispatchError> {
        request.validate()?;
        self.ensure_ready(&request.identity)?;
        let peer = self
            .peers
            .get_mut(&request.identity.runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?;
        peer.create_session(request).map_err(DispatchError::Peer)?;
        peer.activate_broker_context(request)
            .map_err(DispatchError::Peer)?;
        if let Err(error) = peer.dispatch(request) {
            peer.forget_attempt(&request.identity);
            return Err(DispatchError::Peer(error));
        }
        self.event_sequences
            .entry(request.identity.clone())
            .or_insert(0);
        Ok(())
    }

    pub fn dispatch_existing(
        &mut self,
        request: &PeerDispatchRequest,
    ) -> Result<(), DispatchError> {
        request.validate()?;
        self.ensure_ready(&request.identity)?;
        let peer = self
            .peers
            .get_mut(&request.identity.runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?;
        peer.activate_broker_context(request)
            .map_err(DispatchError::Peer)?;
        if let Err(error) = peer.dispatch(request) {
            peer.forget_attempt(&request.identity);
            return Err(DispatchError::Peer(error));
        }
        self.event_sequences
            .entry(request.identity.clone())
            .or_insert(0);
        Ok(())
    }

    pub fn poll_events(
        &mut self,
        runtime_id: &str,
        recorded_at_ms: u64,
    ) -> Result<Vec<DispatchEvent>, DispatchError> {
        if recorded_at_ms == 0 {
            return Err(DispatchError::InvalidEvent);
        }
        let peer = self
            .peers
            .get_mut(runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?;
        let registration = peer.registration().clone();
        let events = peer
            .poll_events(recorded_at_ms)
            .map_err(DispatchError::Peer)?;
        let mut increments = BTreeMap::<DispatchIdentity, u64>::new();
        for event in &events {
            event.validate()?;
            validate_registration(&registration, &event.identity)?;
            let increment = increments.entry(event.identity.clone()).or_insert(0);
            *increment = increment
                .checked_add(1)
                .ok_or(DispatchError::SequenceExhausted)?;
        }
        for (identity, increment) in increments {
            self.event_sequences
                .get(&identity)
                .ok_or(DispatchError::StalePeer)?
                .checked_add(increment)
                .ok_or(DispatchError::SequenceExhausted)?;
        }
        events
            .into_iter()
            .map(|event| {
                let sequence = self
                    .event_sequences
                    .get_mut(&event.identity)
                    .ok_or(DispatchError::StalePeer)?;
                *sequence += 1;
                Ok(DispatchEvent {
                    sequence: *sequence,
                    peer: event,
                })
            })
            .collect()
    }

    pub fn cancel(&mut self, identity: &DispatchIdentity) -> Result<bool, DispatchError> {
        self.ensure_ready(identity)?;
        self.peers
            .get_mut(&identity.runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?
            .cancel(identity)
            .map_err(DispatchError::Peer)
    }

    /// Shuts down and removes one exact production peer. The generation check
    /// prevents a stale host from terminating a replacement process that
    /// reused the same runtime identity.
    pub fn shutdown_and_unregister(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
    ) -> Result<(), DispatchError> {
        validate_id(runtime_id)?;
        if process_generation == 0 {
            return Err(DispatchError::StalePeer);
        }
        let peer = self
            .peers
            .get_mut(runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?;
        if peer.registration().descriptor.process_generation != process_generation {
            return Err(DispatchError::StalePeer);
        }
        peer.shutdown().map_err(DispatchError::Peer)?;
        self.peers.remove(runtime_id);
        self.event_sequences
            .retain(|identity, _| identity.runtime_id != runtime_id);
        Ok(())
    }

    /// Returns a fail-closed decision to the exact pending Pi tool call owned
    /// by the registered peer. Other runtime kinds cannot implement this path.
    pub fn resolve_pi_denied(
        &mut self,
        identity: &DispatchIdentity,
        native_request_id: &str,
        reason_code: &str,
    ) -> Result<(), DispatchError> {
        validate_id(native_request_id)?;
        validate_id(reason_code)?;
        self.ensure_ready(identity)?;
        self.peers
            .get_mut(&identity.runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?
            .resolve_pi_denied(identity, native_request_id, reason_code)
            .map_err(DispatchError::Peer)
    }

    /// Delivers the non-forgeable Action Gateway receipt to the exact pending
    /// Pi call. `PiAdapter` performs the final runtime/run/request/generation
    /// match and accepts only a succeeded normalized result.
    pub fn resolve_pi_completed(
        &mut self,
        identity: &DispatchIdentity,
        native_request_id: &str,
        receipt: &RuntimeExecutionReceipt,
    ) -> Result<(), DispatchError> {
        validate_id(native_request_id)?;
        self.ensure_ready(identity)?;
        self.peers
            .get_mut(&identity.runtime_id)
            .ok_or(DispatchError::PeerUnavailable)?
            .resolve_pi_completed(identity, native_request_id, receipt)
            .map_err(DispatchError::Peer)
    }

    pub fn forget_attempt(&mut self, identity: &DispatchIdentity) {
        self.event_sequences.remove(identity);
        if let Some(peer) = self.peers.get_mut(&identity.runtime_id) {
            peer.forget_attempt(identity);
        }
    }
}

fn validate_registration(
    registration: &RuntimePeerRegistration,
    identity: &DispatchIdentity,
) -> Result<(), DispatchError> {
    registration.validate()?;
    if registration.runtime_id != identity.runtime_id
        || registration.workspace_id != identity.workspace_id
        || registration.descriptor.runtime_kind != identity.runtime_kind
        || registration.descriptor.adapter_version != identity.adapter_version
        || registration.descriptor.native_version != identity.native_version
        || registration.descriptor.process_generation != identity.process_generation
    {
        return Err(DispatchError::StalePeer);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstDispatchOptions {
    pub title: String,
    pub credential_reference: Option<String>,
    pub credential_lease_id: Option<String>,
    pub attachment_resolution: AttachmentPreflightResolution,
    pub broker_authority: Option<BrokerDispatchAuthority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedFirstDispatch {
    Accepted {
        coordinator_generation: u64,
        record: SessionRecord,
    },
    Rejected {
        coordinator_generation: u64,
        record: SessionRecord,
        failure: DispatchFailureCode,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryDispatchOptions {
    pub credential_reference: Option<String>,
    pub credential_lease_id: Option<String>,
    pub attachment_resolution: AttachmentPreflightResolution,
    pub broker_authority: Option<BrokerDispatchAuthority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedRetryDispatch {
    Accepted {
        coordinator_generation: u64,
        record: SessionRecord,
    },
    Rejected {
        coordinator_generation: u64,
        record: SessionRecord,
        failure: DispatchFailureCode,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnDispatchOptions {
    pub credential_reference: Option<String>,
    pub credential_lease_id: Option<String>,
    pub attachment_resolution: AttachmentPreflightResolution,
    pub broker_authority: Option<BrokerDispatchAuthority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedTurnDispatch {
    Accepted {
        coordinator_generation: u64,
        record: SessionRecord,
    },
    Rejected {
        coordinator_generation: u64,
        record: SessionRecord,
        failure: DispatchFailureCode,
    },
}

fn prepare_attachment_dispatch(
    workspace_id: &str,
    prompt: Option<&str>,
    durable: &[AttachmentSnapshot],
    resources: &ResourceSnapshot,
    draft: &DraftRequirements,
    resolution: &AttachmentPreflightResolution,
) -> Result<
    (
        String,
        Vec<VerifiedAttachmentContent>,
        Vec<ConvertedAttachmentMaterialization>,
    ),
    DispatchError,
> {
    validate_installed_resource_preflight(resources, draft)?;
    match resolution {
        AttachmentPreflightResolution::Cancel => {
            return Err(DispatchError::AttachmentDispatchCancelled);
        }
        AttachmentPreflightResolution::Remove { attachment_ids } => {
            validate_removal_preflight(durable, attachment_ids)?;
            return Err(DispatchError::AttachmentRemovalRequested);
        }
        AttachmentPreflightResolution::Materialize(_) if durable.is_empty() => {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
        AttachmentPreflightResolution::Direct(_) if durable.is_empty() => {
            return Err(DispatchError::InvalidDirectAttachment);
        }
        AttachmentPreflightResolution::NotRequired => {
            let input = prompt
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_default()
                .to_owned();
            if !durable.is_empty() {
                return Err(DispatchError::AttachmentMaterializationRequired);
            }
            if input.is_empty() {
                return Err(DispatchError::InvalidRequest);
            }
            return Ok((input, Vec::new(), Vec::new()));
        }
        AttachmentPreflightResolution::Direct(_) => {}
        AttachmentPreflightResolution::Materialize(_) => {}
    }

    if let AttachmentPreflightResolution::Direct(plan) = resolution {
        validate_direct_attachments(workspace_id, durable, draft, plan)?;
        let input = prompt
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_default()
            .to_owned();
        return Ok((input, plan.attachments().to_vec(), Vec::new()));
    }

    let AttachmentPreflightResolution::Materialize(plan) = resolution else {
        unreachable!("all non-materialization outcomes returned above")
    };
    if durable.len() > MAX_ATTACHMENT_MATERIALIZATIONS
        || plan.attachments.len() != durable.len()
        || draft.attachments.len() != durable.len()
        || !draft.policy.attachment_conversion_allowed
        || draft.installed_resources.snapshot_id != resources.snapshot_id
        || draft.installed_resources.snapshot_sha256 != resources.sha256
    {
        return Err(DispatchError::InvalidAttachmentMaterialization);
    }

    let mut by_id = BTreeMap::new();
    for materialized in &plan.attachments {
        if by_id
            .insert(materialized.attachment_id.as_str(), materialized)
            .is_some()
        {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
    }

    let mut ordered = Vec::with_capacity(durable.len());
    for attachment in durable {
        let materialized = by_id
            .get(attachment.attachment_id.as_str())
            .copied()
            .ok_or(DispatchError::InvalidAttachmentMaterialization)?;
        let requirement = draft
            .attachments
            .iter()
            .find(|requirement| requirement.attachment_id == attachment.attachment_id)
            .ok_or(DispatchError::InvalidAttachmentMaterialization)?;
        validate_converted_attachment(materialized, attachment, resources)?;
        if requirement.mime_type != attachment.media_type
            || requirement.bytes != attachment.byte_length
            || !draft
                .installed_resources
                .attachment_converters
                .contains(&requirement.media_type)
        {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
        ordered.push(materialized.clone());
    }

    let mut input = prompt
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_default()
        .to_owned();
    for attachment in &ordered {
        if !input.is_empty() {
            input.push_str("\n\n");
        }
        input.push_str("[C4OS approved converter output; attachment=");
        input.push_str(&attachment.attachment_id);
        input.push_str("; reference=");
        input.push_str(&attachment.output_reference);
        input.push_str("; digest=");
        input.push_str(&attachment.output_sha256);
        input.push_str("]\n");
        input.push_str(&attachment.output);
        if input.len() > MAX_INPUT_BYTES {
            return Err(DispatchError::InvalidAttachmentMaterialization);
        }
    }
    if input.trim().is_empty() {
        return Err(DispatchError::InvalidAttachmentMaterialization);
    }
    Ok((input, Vec::new(), ordered))
}

fn validate_installed_resource_preflight(
    resources: &ResourceSnapshot,
    draft: &DraftRequirements,
) -> Result<(), DispatchError> {
    if draft.installed_resources.snapshot_id != resources.snapshot_id
        || draft.installed_resources.snapshot_sha256 != resources.sha256
        || draft.installed_resources.tool_ids.iter().any(|tool_id| {
            !resources
                .resource_ids
                .iter()
                .any(|resource_id| resource_id == tool_id)
        })
    {
        return Err(DispatchError::InvalidResourcePreflight);
    }
    Ok(())
}

fn validate_direct_attachments(
    workspace_id: &str,
    durable: &[AttachmentSnapshot],
    draft: &DraftRequirements,
    plan: &AttachmentContentPlan,
) -> Result<(), DispatchError> {
    if plan.workspace_id() != workspace_id
        || durable.len() > MAX_ATTACHMENT_MATERIALIZATIONS
        || draft.attachments.len() != durable.len()
        || plan.attachments().len() != durable.len()
    {
        return Err(DispatchError::InvalidDirectAttachment);
    }
    let mut durable_ids = BTreeSet::new();
    let mut requirement_ids = BTreeSet::new();
    for requirement in &draft.attachments {
        if !requirement_ids.insert(requirement.attachment_id.as_str()) {
            return Err(DispatchError::InvalidDirectAttachment);
        }
    }
    for attachment in durable {
        validate_direct_attachment(attachment)?;
        if !durable_ids.insert(attachment.attachment_id.as_str()) {
            return Err(DispatchError::InvalidDirectAttachment);
        }
        let requirement = draft
            .attachments
            .iter()
            .find(|requirement| requirement.attachment_id == attachment.attachment_id)
            .ok_or(DispatchError::InvalidDirectAttachment)?;
        if requirement.mime_type != attachment.media_type
            || requirement.bytes != attachment.byte_length
        {
            return Err(DispatchError::InvalidDirectAttachment);
        }
        let verified = plan
            .attachments()
            .iter()
            .find(|verified| verified.snapshot().attachment_id == attachment.attachment_id)
            .ok_or(DispatchError::InvalidDirectAttachment)?;
        if verified.snapshot() != attachment {
            return Err(DispatchError::InvalidDirectAttachment);
        }
        validate_verified_attachment(verified)?;
    }
    Ok(())
}

fn validate_verified_attachment(
    attachment: &VerifiedAttachmentContent,
) -> Result<(), DispatchError> {
    validate_direct_attachment(attachment.snapshot())?;
    if attachment.content().len() as u64 != attachment.snapshot().byte_length
        || sha256_prefixed(attachment.content()) != attachment.snapshot().content_sha256
    {
        return Err(DispatchError::InvalidDirectAttachment);
    }
    Ok(())
}

fn validate_direct_attachment(attachment: &AttachmentSnapshot) -> Result<(), DispatchError> {
    for value in [
        attachment.attachment_id.as_str(),
        attachment.stable_reference.as_str(),
        attachment.media_type.as_str(),
    ] {
        validate_id(value).map_err(|_| DispatchError::InvalidDirectAttachment)?;
    }
    if attachment.display_name.trim().is_empty()
        || attachment.display_name.len() > 512
        || attachment.display_name.contains('\0')
        || attachment.byte_length == 0
        || attachment.byte_length > MAX_DIRECT_ATTACHMENT_BYTES
        || !is_sha256(&attachment.content_sha256)
        || attachment.snapshot_version == 0
    {
        return Err(DispatchError::InvalidDirectAttachment);
    }
    Ok(())
}

fn validate_removal_preflight(
    durable: &[AttachmentSnapshot],
    attachment_ids: &[String],
) -> Result<(), DispatchError> {
    if durable.is_empty()
        || attachment_ids.is_empty()
        || attachment_ids.len() > durable.len()
        || attachment_ids.iter().any(|id| validate_id(id).is_err())
    {
        return Err(DispatchError::InvalidAttachmentMaterialization);
    }
    let unique = attachment_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if unique.len() != attachment_ids.len()
        || !unique.iter().all(|id| {
            durable
                .iter()
                .any(|attachment| attachment.attachment_id == *id)
        })
    {
        return Err(DispatchError::InvalidAttachmentMaterialization);
    }
    Ok(())
}

fn validate_converted_attachment(
    materialized: &ConvertedAttachmentMaterialization,
    durable: &AttachmentSnapshot,
    resources: &ResourceSnapshot,
) -> Result<(), DispatchError> {
    for value in [
        materialized.attachment_id.as_str(),
        materialized.stable_reference.as_str(),
        materialized.converter_id.as_str(),
        materialized.converter_version.as_str(),
        materialized.converter_snapshot_id.as_str(),
        materialized.resource_snapshot_id.as_str(),
        materialized.output_reference.as_str(),
    ] {
        validate_id(value).map_err(|_| DispatchError::InvalidAttachmentMaterialization)?;
    }
    if materialized.attachment_id != durable.attachment_id
        || materialized.stable_reference != durable.stable_reference
        || materialized.display_name != durable.display_name
        || materialized.content_sha256 != durable.content_sha256
        || materialized.byte_length != durable.byte_length
        || materialized.media_type != durable.media_type
        || materialized.snapshot_version != durable.snapshot_version
        || materialized.resource_snapshot_id != resources.snapshot_id
        || materialized.resource_snapshot_version != resources.version
        || materialized.resource_snapshot_sha256 != resources.sha256
        || materialized.converter_snapshot_version == 0
        || !resources
            .resource_ids
            .iter()
            .any(|resource_id| resource_id == &materialized.converter_id)
        || materialized.output_media_type != CONVERTER_OUTPUT_MEDIA_TYPE
        || materialized.output.trim().is_empty()
        || materialized.output.contains('\0')
        || materialized.output.len() > MAX_INPUT_BYTES
        || materialized.output_byte_length != materialized.output.len() as u64
        || materialized.output_sha256 != sha256_prefixed(materialized.output.as_bytes())
        || !is_sha256(&materialized.content_sha256)
        || !is_sha256(&materialized.converter_snapshot_sha256)
        || !is_sha256(&materialized.resource_snapshot_sha256)
        || !is_sha256(&materialized.output_sha256)
    {
        return Err(DispatchError::InvalidAttachmentMaterialization);
    }
    Ok(())
}

fn validate_converter_descriptor(
    descriptor: &InstalledAttachmentConverterDescriptor,
    resources: &ResourceSnapshot,
) -> Result<(), DispatchError> {
    for value in [
        descriptor.converter_id.as_str(),
        descriptor.converter_version.as_str(),
        descriptor.snapshot_id.as_str(),
        resources.snapshot_id.as_str(),
    ] {
        validate_id(value).map_err(|_| DispatchError::InvalidAttachmentMaterialization)?;
    }
    if descriptor.snapshot_version == 0
        || resources.version == 0
        || !is_sha256(&descriptor.snapshot_sha256)
        || !resources
            .resource_ids
            .iter()
            .any(|resource_id| resource_id == &descriptor.converter_id)
        || resources
            .resource_ids
            .iter()
            .any(|resource_id| validate_id(resource_id).is_err())
    {
        return Err(DispatchError::InvalidAttachmentMaterialization);
    }
    Ok(())
}

fn converter_resource_binding_sha256(
    descriptor: &InstalledAttachmentConverterDescriptor,
    resources: &ResourceSnapshot,
) -> String {
    let mut hasher = Sha256::new();
    hash_binding_field(&mut hasher, b"c4os.attachment-converter.v1");
    hash_binding_field(&mut hasher, resources.snapshot_id.as_bytes());
    hash_binding_field(&mut hasher, &resources.version.to_be_bytes());
    for resource_id in &resources.resource_ids {
        hash_binding_field(&mut hasher, resource_id.as_bytes());
    }
    hash_binding_field(&mut hasher, descriptor.converter_id.as_bytes());
    hash_binding_field(&mut hasher, descriptor.converter_version.as_bytes());
    hash_binding_field(&mut hasher, descriptor.snapshot_id.as_bytes());
    hash_binding_field(&mut hasher, &descriptor.snapshot_version.to_be_bytes());
    hash_binding_field(&mut hasher, descriptor.snapshot_sha256.as_bytes());
    prefixed_digest(hasher.finalize())
}

fn hash_binding_field(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

/// Preflight and exact peer readiness are checked before the durable mutation.
/// Once first-submit commits, native create/dispatch is attempted. A rejection
/// is deterministically closed into a terminal failed attempt.
pub fn coordinate_first_dispatch<R: SessionRepository>(
    coordinator: &mut RuntimeCoordinator<R>,
    registry: &mut RuntimeDispatchRegistry,
    request: CoordinatedFirstSubmission,
    options: FirstDispatchOptions,
) -> Result<CoordinatedFirstDispatch, DispatchError> {
    let identity = identity_from_submission(&request)?;
    let preflight = coordinator.model_preflight(
        &request.provider_id,
        &request.selected_model_id,
        &request.capability_layers,
        &request.draft,
        request.preflight_at_ms,
    )?;
    if !matches!(preflight.outcome, PreflightOutcome::Ready { .. }) {
        return Err(DispatchError::PreflightBlocked);
    }
    let model = DispatchModelRoute {
        provider_id: preflight.effective_capabilities.route.provider_id.clone(),
        model_id: preflight
            .effective_capabilities
            .route
            .provider_model_id
            .clone(),
        credential_reference: options.credential_reference,
        credential_lease_id: options.credential_lease_id,
    };
    let (input, direct_attachments, attachments) = prepare_attachment_dispatch(
        &identity.workspace_id,
        request.submission.prompt.as_deref(),
        &request.submission.attachments,
        &request.submission.binding.initial_resources,
        &request.draft,
        &options.attachment_resolution,
    )?;
    let dispatch = PeerDispatchRequest {
        identity: identity.clone(),
        model,
        title: options.title,
        input,
        eligible_tool_ids: request.draft.installed_resources.tool_ids.clone(),
        broker_authority: options.broker_authority,
        direct_attachments,
        attachments,
    };
    dispatch.validate()?;
    registry.ensure_ready(&identity)?;

    let submitted = coordinator.submit_first(request)?;
    match registry.dispatch_first(&dispatch) {
        Ok(()) => Ok(CoordinatedFirstDispatch::Accepted {
            coordinator_generation: submitted.coordinator_generation,
            record: submitted.value,
        }),
        Err(error) => {
            let failure = DispatchFailureCode::from_dispatch_error(&error);
            let closed = coordinator.finish_attempt(
                &identity.session_id,
                &identity.attempt_identity(),
                TerminalAttemptOutcome::Failed {
                    error_code: failure.as_error_code().into(),
                },
                dispatch_close_time(request_time(&submitted.value)),
            )?;
            Ok(CoordinatedFirstDispatch::Rejected {
                coordinator_generation: closed.coordinator_generation,
                record: closed.value,
                failure,
            })
        }
    }
}

/// A Retry reuses the existing native session but always creates a fresh C4OS
/// attempt/correlation/authorization identity. Preflight and exact peer
/// readiness precede the durable mutation; a native rejection terminally
/// closes the newly durable retry attempt.
pub fn coordinate_retry_dispatch<R: SessionRepository>(
    coordinator: &mut RuntimeCoordinator<R>,
    registry: &mut RuntimeDispatchRegistry,
    request: CoordinatedRetry,
    options: RetryDispatchOptions,
) -> Result<CoordinatedRetryDispatch, DispatchError> {
    let record = coordinator.session(&request.request.session_id)?;
    let parent = record
        .attempt(&request.request.parent_attempt_id)
        .ok_or(DispatchError::InvalidRequest)?;
    let turn = record
        .turn(&parent.turn_id)
        .ok_or(DispatchError::InvalidRequest)?;
    let identity = identity_from_retry(&request, &parent.turn_id)?;
    let preflight = coordinator.model_preflight(
        &request.provider_id,
        &request.selected_model_id,
        &request.capability_layers,
        &request.draft,
        request.preflight_at_ms,
    )?;
    if !matches!(preflight.outcome, PreflightOutcome::Ready { .. }) {
        return Err(DispatchError::PreflightBlocked);
    }
    let model = DispatchModelRoute {
        provider_id: preflight.effective_capabilities.route.provider_id.clone(),
        model_id: preflight
            .effective_capabilities
            .route
            .provider_model_id
            .clone(),
        credential_reference: options.credential_reference,
        credential_lease_id: options.credential_lease_id,
    };
    let (input, direct_attachments, attachments) = prepare_attachment_dispatch(
        &identity.workspace_id,
        turn.prompt.as_deref(),
        &turn.attachments,
        &request.request.context.resources,
        &request.draft,
        &options.attachment_resolution,
    )?;
    let dispatch = PeerDispatchRequest {
        identity: identity.clone(),
        model,
        title: record.title.unwrap_or_else(|| "C4OS Chat".into()),
        input,
        eligible_tool_ids: request.draft.installed_resources.tool_ids.clone(),
        broker_authority: options.broker_authority,
        direct_attachments,
        attachments,
    };
    dispatch.validate()?;
    registry.ensure_ready(&identity)?;

    let retried = coordinator.retry(request)?;
    match registry.dispatch_existing(&dispatch) {
        Ok(()) => Ok(CoordinatedRetryDispatch::Accepted {
            coordinator_generation: retried.coordinator_generation,
            record: retried.value,
        }),
        Err(error) => {
            let failure = DispatchFailureCode::from_dispatch_error(&error);
            let closed = coordinator.finish_attempt(
                &identity.session_id,
                &identity.attempt_identity(),
                TerminalAttemptOutcome::Failed {
                    error_code: failure.as_error_code().into(),
                },
                dispatch_close_time(request_time(&retried.value)),
            )?;
            Ok(CoordinatedRetryDispatch::Rejected {
                coordinator_generation: closed.coordinator_generation,
                record: closed.value,
                failure,
            })
        }
    }
}

/// A follow-up turn reuses the immutable native session binding while creating
/// fresh turn, attempt, authorization, and correlation identities. The same
/// preflight-before-persistence and terminal-close-on-rejection contract used
/// for first submission and Retry applies here.
pub fn coordinate_turn_dispatch<R: SessionRepository>(
    coordinator: &mut RuntimeCoordinator<R>,
    registry: &mut RuntimeDispatchRegistry,
    request: CoordinatedTurn,
    options: TurnDispatchOptions,
) -> Result<CoordinatedTurnDispatch, DispatchError> {
    let record = coordinator.session(&request.submission.session_id)?;
    let identity = identity_from_turn(&request)?;
    let preflight = coordinator.model_preflight(
        &request.provider_id,
        &request.selected_model_id,
        &request.capability_layers,
        &request.draft,
        request.preflight_at_ms,
    )?;
    if !matches!(preflight.outcome, PreflightOutcome::Ready { .. }) {
        return Err(DispatchError::PreflightBlocked);
    }
    let model = DispatchModelRoute {
        provider_id: preflight.effective_capabilities.route.provider_id.clone(),
        model_id: preflight
            .effective_capabilities
            .route
            .provider_model_id
            .clone(),
        credential_reference: options.credential_reference,
        credential_lease_id: options.credential_lease_id,
    };
    let (mut input, direct_attachments, attachments) = prepare_attachment_dispatch(
        &identity.workspace_id,
        request.submission.prompt.as_deref(),
        &request.submission.attachments,
        &request.submission.context.resources,
        &request.draft,
        &options.attachment_resolution,
    )?;
    if let Some(reply) = &request.submission.reply_context {
        let composed = format!(
            "Reply to the immutable {kind} reference {target} ({digest}).\n\
             <reply-context>\n{excerpt}\n</reply-context>\n\
             <user-message>\n{input}\n</user-message>",
            kind = reply.target_kind,
            target = reply.target_id,
            digest = reply.source_sha256,
            excerpt = reply.source_excerpt,
        );
        if composed.len() > MAX_INPUT_BYTES {
            return Err(DispatchError::InvalidRequest);
        }
        input = composed;
    }
    let dispatch = PeerDispatchRequest {
        identity: identity.clone(),
        model,
        title: record.title.unwrap_or_else(|| "C4OS Chat".into()),
        input,
        eligible_tool_ids: request.draft.installed_resources.tool_ids.clone(),
        broker_authority: options.broker_authority,
        direct_attachments,
        attachments,
    };
    dispatch.validate()?;
    registry.ensure_ready(&identity)?;

    let submitted = coordinator.submit_turn(request)?;
    match registry.dispatch_existing(&dispatch) {
        Ok(()) => Ok(CoordinatedTurnDispatch::Accepted {
            coordinator_generation: submitted.coordinator_generation,
            record: submitted.value,
        }),
        Err(error) => {
            let failure = DispatchFailureCode::from_dispatch_error(&error);
            let closed = coordinator.finish_attempt(
                &identity.session_id,
                &identity.attempt_identity(),
                TerminalAttemptOutcome::Failed {
                    error_code: failure.as_error_code().into(),
                },
                dispatch_close_time(request_time(&submitted.value)),
            )?;
            Ok(CoordinatedTurnDispatch::Rejected {
                coordinator_generation: closed.coordinator_generation,
                record: closed.value,
                failure,
            })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedDispatchEvent {
    pub event: DispatchEvent,
    pub record: SessionRecord,
    pub coordinator_generation: u64,
}

pub fn coordinate_polled_events<R: SessionRepository>(
    coordinator: &mut RuntimeCoordinator<R>,
    registry: &mut RuntimeDispatchRegistry,
    runtime_id: &str,
    recorded_at_ms: u64,
) -> Result<Vec<AppliedDispatchEvent>, DispatchError> {
    let events = registry.poll_events(runtime_id, recorded_at_ms)?;
    let mut applied = Vec::with_capacity(events.len());
    for event in events {
        let identity = event.peer.identity.attempt_identity();
        let appended = coordinator.append_normalized_event(
            &event.peer.identity.session_id,
            &identity,
            event.run_event(),
        )?;
        let terminal = event.peer.terminal_outcome();
        let (coordinator_generation, record) = if let Some(outcome) = terminal {
            coordinator.cancel_runtime_actions(
                &event.peer.identity.attempt_id,
                event.peer.recorded_at_ms,
            )?;
            let finished = coordinator.finish_attempt(
                &event.peer.identity.session_id,
                &identity,
                outcome,
                event.peer.recorded_at_ms,
            )?;
            registry.forget_attempt(&event.peer.identity);
            (finished.coordinator_generation, finished.value)
        } else {
            (appended.coordinator_generation, appended.value)
        };
        applied.push(AppliedDispatchEvent {
            event,
            record,
            coordinator_generation,
        });
    }
    Ok(applied)
}

/// Recovery is coordinator-owned and terminalizes every interrupted durable
/// attempt before this registry forgets its in-memory event sequence. No peer
/// is asked to resume or replay a pre-restart attempt implicitly.
pub fn coordinate_recovery<R: SessionRepository>(
    coordinator: &mut RuntimeCoordinator<R>,
    registry: &mut RuntimeDispatchRegistry,
    recovered_at_ms: u64,
) -> Result<CoordinatorOperation<Vec<SessionRecord>>, DispatchError> {
    let recovered = coordinator.recover_interrupted(recovered_at_ms)?;
    let mut coordinator_generation = recovered.coordinator_generation;
    for run_id in recovered.value.iter().flat_map(|record| {
        record.attempts.iter().filter_map(|attempt| {
            matches!(
                attempt.status,
                crate::runtime::session::RunAttemptStatus::Interrupted { .. }
            )
            .then_some(attempt.attempt_id.as_str())
        })
    }) {
        coordinator_generation = coordinator
            .cancel_runtime_actions(run_id, recovered_at_ms)?
            .coordinator_generation;
    }
    let session_ids = recovered
        .value
        .iter()
        .map(|record| record.session_id.as_str())
        .collect::<BTreeSet<_>>();
    let interrupted = registry
        .event_sequences
        .keys()
        .filter(|identity| session_ids.contains(identity.session_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    for identity in interrupted {
        registry.forget_attempt(&identity);
    }
    Ok(CoordinatorOperation {
        coordinator_generation,
        value: recovered.value,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedCancellation {
    Cancelled {
        coordinator_generation: u64,
        record: SessionRecord,
    },
    Interrupted {
        coordinator_generation: u64,
        record: SessionRecord,
    },
}

pub fn coordinate_cancellation<R: SessionRepository>(
    coordinator: &mut RuntimeCoordinator<R>,
    registry: &mut RuntimeDispatchRegistry,
    identity: &DispatchIdentity,
    requested_at_ms: u64,
) -> Result<CoordinatedCancellation, DispatchError> {
    registry.ensure_ready(identity)?;
    coordinator.cancel_runtime_actions(&identity.attempt_id, requested_at_ms)?;
    coordinator.request_cancellation(
        &identity.session_id,
        &identity.attempt_identity(),
        requested_at_ms,
    )?;
    let (outcome, accepted) = match registry.cancel(identity) {
        Ok(true) => (TerminalAttemptOutcome::Cancelled, true),
        Ok(false) | Err(_) => (
            TerminalAttemptOutcome::Interrupted {
                reason_code: "runtime-cancel-not-accepted".into(),
            },
            false,
        ),
    };
    let finished = coordinator.finish_attempt(
        &identity.session_id,
        &identity.attempt_identity(),
        outcome,
        dispatch_close_time(requested_at_ms),
    )?;
    registry.forget_attempt(identity);
    if accepted {
        Ok(CoordinatedCancellation::Cancelled {
            coordinator_generation: finished.coordinator_generation,
            record: finished.value,
        })
    } else {
        Ok(CoordinatedCancellation::Interrupted {
            coordinator_generation: finished.coordinator_generation,
            record: finished.value,
        })
    }
}

fn identity_from_submission(
    request: &CoordinatedFirstSubmission,
) -> Result<DispatchIdentity, DispatchError> {
    let binding = &request.submission.binding;
    let runtime_kind = match binding.runtime_kind {
        crate::runtime::session::RuntimeKind::OpenCode => RuntimeKind::OpenCode,
        crate::runtime::session::RuntimeKind::Pi => RuntimeKind::Pi,
    };
    let identity = DispatchIdentity {
        workspace_id: binding.workspace_id.clone(),
        environment_id: binding.environment.environment_id.clone(),
        session_id: request.submission.session_id.clone(),
        turn_id: request.submission.turn_id.clone(),
        attempt_id: request.submission.attempt_id.clone(),
        correlation_id: request.submission.correlation_id.clone(),
        runtime_id: binding.runtime_id.clone(),
        runtime_kind,
        adapter_version: binding.adapter.adapter_version.clone(),
        native_version: binding.adapter.native_version.clone(),
        process_generation: request.submission.process_generation,
    };
    identity.validate()?;
    Ok(identity)
}

fn identity_from_retry(
    request: &CoordinatedRetry,
    turn_id: &str,
) -> Result<DispatchIdentity, DispatchError> {
    let context = &request.request.context;
    let runtime_kind = match context.runtime_kind {
        crate::runtime::session::RuntimeKind::OpenCode => RuntimeKind::OpenCode,
        crate::runtime::session::RuntimeKind::Pi => RuntimeKind::Pi,
    };
    let identity = DispatchIdentity {
        workspace_id: context.workspace_id.clone(),
        environment_id: context.environment.environment_id.clone(),
        session_id: request.request.session_id.clone(),
        turn_id: turn_id.into(),
        attempt_id: request.request.attempt_id.clone(),
        correlation_id: request.request.correlation_id.clone(),
        runtime_id: context.runtime_id.clone(),
        runtime_kind,
        adapter_version: context.adapter.adapter_version.clone(),
        native_version: context.adapter.native_version.clone(),
        process_generation: request.request.process_generation,
    };
    identity.validate()?;
    Ok(identity)
}

fn identity_from_turn(request: &CoordinatedTurn) -> Result<DispatchIdentity, DispatchError> {
    let context = &request.submission.context;
    let runtime_kind = match context.runtime_kind {
        crate::runtime::session::RuntimeKind::OpenCode => RuntimeKind::OpenCode,
        crate::runtime::session::RuntimeKind::Pi => RuntimeKind::Pi,
    };
    let identity = DispatchIdentity {
        workspace_id: context.workspace_id.clone(),
        environment_id: context.environment.environment_id.clone(),
        session_id: request.submission.session_id.clone(),
        turn_id: request.submission.turn_id.clone(),
        attempt_id: request.submission.attempt_id.clone(),
        correlation_id: request.submission.correlation_id.clone(),
        runtime_id: context.runtime_id.clone(),
        runtime_kind,
        adapter_version: context.adapter.adapter_version.clone(),
        native_version: context.adapter.native_version.clone(),
        process_generation: request.submission.process_generation,
    };
    identity.validate()?;
    Ok(identity)
}

fn request_time(record: &SessionRecord) -> u64 {
    record.updated_at_ms
}

fn dispatch_close_time(at_ms: u64) -> u64 {
    at_ms.saturating_add(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchFailureCode {
    PeerUnavailable,
    PeerRejected,
    StalePeer,
}

impl DispatchFailureCode {
    fn from_dispatch_error(error: &DispatchError) -> Self {
        match error {
            DispatchError::PeerUnavailable => Self::PeerUnavailable,
            DispatchError::StalePeer => Self::StalePeer,
            _ => Self::PeerRejected,
        }
    }

    fn as_error_code(self) -> &'static str {
        match self {
            Self::PeerUnavailable => "runtime-peer-unavailable",
            Self::PeerRejected => "runtime-dispatch-rejected",
            Self::StalePeer => "runtime-peer-stale",
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PeerDispatchError {
    #[error("runtime peer is not ready")]
    NotReady,
    #[error("runtime peer rejected session creation")]
    SessionCreate,
    #[error("runtime peer rejected dispatch")]
    Dispatch,
    #[error("runtime peer event stream failed")]
    EventStream,
    #[error("runtime peer rejected cancellation")]
    Cancellation,
    #[error("runtime peer does not own an OpenCode SSE event source")]
    UnsupportedEventSource,
    #[error("runtime peer identity is stale")]
    StaleIdentity,
    #[error("runtime peer does not support gateway tool resolution")]
    UnsupportedToolResolution,
    #[error("runtime peer does not support protocol-aware shutdown")]
    UnsupportedShutdown,
    #[error("runtime peer protocol-aware shutdown failed")]
    Shutdown,
    #[error("runtime peer credential delivery failed")]
    Credential,
    #[error("runtime peer session route conflicts with its immutable native binding")]
    RouteConflict,
}

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("dispatch identity is invalid")]
    InvalidIdentity,
    #[error("dispatch request is invalid")]
    InvalidRequest,
    #[error("dispatch event is invalid")]
    InvalidEvent,
    #[error("runtime dispatch peer is unavailable")]
    PeerUnavailable,
    #[error("runtime dispatch peer is duplicated")]
    DuplicatePeer,
    #[error("runtime dispatch peer is incompatible")]
    IncompatiblePeer,
    #[error("runtime dispatch peer binding is stale")]
    StalePeer,
    #[error("installed-resource preflight does not match core-owned authority")]
    InvalidResourcePreflight,
    #[error("attachment dispatch requires an explicit materialization plan")]
    AttachmentMaterializationRequired,
    #[error("attachment materialization does not match durable and installed snapshots")]
    InvalidAttachmentMaterialization,
    #[error("direct attachment metadata does not match the durable draft snapshot")]
    InvalidDirectAttachment,
    #[error("attachment removal must be completed before dispatch")]
    AttachmentRemovalRequested,
    #[error("attachment dispatch was cancelled during preflight")]
    AttachmentDispatchCancelled,
    #[error("model preflight blocked dispatch")]
    PreflightBlocked,
    #[error("dispatch event sequence was exhausted")]
    SequenceExhausted,
    #[error("OpenCode production event source is unavailable")]
    EventSourceUnavailable,
    #[error(transparent)]
    AdapterContract(#[from] AdapterContractError),
    #[error(transparent)]
    Coordinator(#[from] CoordinatorError),
    #[error(transparent)]
    Peer(#[from] PeerDispatchError),
}

struct OpenCodeSseFrame {
    session_id: String,
    frame: Vec<u8>,
    received_at_ms: u64,
}

pub struct OpenCodeDispatchPeer<T: OpenCodeTransport, C: CommandDriver> {
    registration: RuntimePeerRegistration,
    adapter: OpenCodeAdapter<T, C>,
    active: BTreeMap<String, DispatchIdentity>,
    created_sessions: BTreeSet<String>,
    native_sessions: BTreeMap<String, String>,
    pending_sse: VecDeque<OpenCodeSseFrame>,
    broker_resolver: Option<ActiveBrokerContextResolver>,
    broker_contexts: BTreeMap<String, BrokerActionContext>,
    #[cfg(unix)]
    event_stream: Option<OpenCodeStreamWorker>,
}

impl<T: OpenCodeTransport, C: CommandDriver> OpenCodeDispatchPeer<T, C> {
    pub fn new(
        registration: RuntimePeerRegistration,
        adapter: OpenCodeAdapter<T, C>,
    ) -> Result<Self, DispatchError> {
        registration.validate()?;
        if registration.descriptor.runtime_kind != RuntimeKind::OpenCode {
            return Err(DispatchError::IncompatiblePeer);
        }
        Ok(Self {
            registration,
            adapter,
            active: BTreeMap::new(),
            created_sessions: BTreeSet::new(),
            native_sessions: BTreeMap::new(),
            pending_sse: VecDeque::new(),
            broker_resolver: None,
            broker_contexts: BTreeMap::new(),
            #[cfg(unix)]
            event_stream: None,
        })
    }

    pub fn attach_broker_context_resolver(
        &mut self,
        resolver: ActiveBrokerContextResolver,
    ) -> Result<(), DispatchError> {
        if self.broker_resolver.is_some() || !self.broker_contexts.is_empty() {
            return Err(DispatchError::IncompatiblePeer);
        }
        self.broker_resolver = Some(resolver);
        Ok(())
    }

    #[cfg(unix)]
    pub fn attach_event_stream(
        &mut self,
        event_stream: OpenCodeStreamWorker,
    ) -> Result<(), DispatchError> {
        if self.event_stream.is_some() || !event_stream.is_ready() {
            return Err(DispatchError::EventSourceUnavailable);
        }
        self.event_stream = Some(event_stream);
        Ok(())
    }

    #[cfg(unix)]
    pub fn shutdown_event_stream(&mut self) -> Result<(), DispatchError> {
        let mut stream = self
            .event_stream
            .take()
            .ok_or(DispatchError::EventSourceUnavailable)?;
        stream
            .shutdown()
            .map_err(|_| DispatchError::EventSourceUnavailable)
    }

    fn correlation(
        &self,
        identity: &DispatchIdentity,
    ) -> Result<EventCorrelation, PeerDispatchError> {
        let session = self
            .adapter
            .session(&identity.session_id)
            .ok_or(PeerDispatchError::StaleIdentity)?;
        if session.workspace_id != identity.workspace_id
            || session.process_generation != identity.process_generation
        {
            return Err(PeerDispatchError::StaleIdentity);
        }
        Ok(EventCorrelation {
            workspace_id: identity.workspace_id.clone(),
            c4os_session_id: identity.session_id.clone(),
            c4os_turn_id: identity.turn_id.clone(),
            c4os_run_id: identity.attempt_id.clone(),
            correlation_id: identity.correlation_id.clone(),
            native_session_id: session.native_session_id.clone(),
            process_generation: identity.process_generation,
        })
    }
}

#[cfg(unix)]
impl<C: CommandDriver>
    OpenCodeDispatchPeer<crate::runtime::opencode_native::LoopbackHttpTransport, C>
{
    /// Materialize the production subscriber from the adapter's immutable
    /// workspace/auth binding and the transport's fixed loopback endpoint.
    /// No endpoint, path, directory, or secret bytes are caller supplied.
    pub fn attach_production_event_stream(
        &mut self,
        bounds: OpenCodeStreamBounds,
    ) -> Result<(), DispatchError> {
        let workspace_directory = self
            .adapter
            .event_stream_workspace_root()
            .to_str()
            .ok_or(DispatchError::EventSourceUnavailable)?
            .to_owned();
        let (username, password_reference) = self.adapter.event_stream_auth_identity();
        let username = username.to_owned();
        let password_reference = password_reference.clone();
        let connector = self
            .adapter
            .event_stream_transport_mut()
            .event_stream_connector(username, password_reference)
            .map_err(|_| DispatchError::EventSourceUnavailable)?;
        let worker = OpenCodeStreamWorker::start(
            connector,
            OpenCodeEventSubscription {
                workspace_directory,
                native_version: self.registration.descriptor.native_version.clone(),
                process_generation: self.registration.descriptor.process_generation,
            },
            bounds,
        )
        .map_err(|_| DispatchError::EventSourceUnavailable)?;
        self.attach_event_stream(worker)
    }
}

impl<T, C> RuntimeDispatchPeer for OpenCodeDispatchPeer<T, C>
where
    T: OpenCodeTransport + Send,
    C: CommandDriver + Send,
{
    fn registration(&self) -> &RuntimePeerRegistration {
        &self.registration
    }

    fn readiness(&self) -> Result<(), PeerDispatchError> {
        let health = self.adapter.health().ok_or(PeerDispatchError::NotReady)?;
        if !health.healthy
            || health.native_version != self.registration.descriptor.native_version
            || health.process_generation != self.registration.descriptor.process_generation
        {
            return Err(PeerDispatchError::NotReady);
        }
        #[cfg(unix)]
        if !self
            .event_stream
            .as_ref()
            .is_some_and(OpenCodeStreamWorker::is_ready)
        {
            return Err(PeerDispatchError::NotReady);
        }
        Ok(())
    }

    fn create_session(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        if self.created_sessions.contains(&request.identity.session_id) {
            return Ok(());
        }
        let binding = self
            .adapter
            .create_session(&request.identity.session_id, &request.title)
            .map_err(|_| PeerDispatchError::SessionCreate)?;
        if binding.workspace_id != request.identity.workspace_id
            || binding.process_generation != request.identity.process_generation
        {
            return Err(PeerDispatchError::StaleIdentity);
        }
        if self
            .native_sessions
            .get(&binding.native_session_id)
            .is_some_and(|session_id| session_id != &request.identity.session_id)
        {
            return Err(PeerDispatchError::StaleIdentity);
        }
        self.native_sessions.insert(
            binding.native_session_id.clone(),
            request.identity.session_id.clone(),
        );
        self.created_sessions
            .insert(request.identity.session_id.clone());
        Ok(())
    }

    fn activate_broker_context(
        &mut self,
        request: &PeerDispatchRequest,
    ) -> Result<(), PeerDispatchError> {
        let authority = request
            .broker_authority
            .as_ref()
            .ok_or(PeerDispatchError::Dispatch)?;
        let resolver = self
            .broker_resolver
            .as_ref()
            .ok_or(PeerDispatchError::Dispatch)?;
        if self
            .broker_contexts
            .contains_key(&request.identity.session_id)
        {
            return Err(PeerDispatchError::Dispatch);
        }
        let correlation = self.correlation(&request.identity)?;
        let native_message_id = opencode_message_id_for_operation(&request.identity.correlation_id)
            .map_err(|_| PeerDispatchError::Dispatch)?;
        let context = BrokerActionContext {
            dispatch: request.identity.clone(),
            native_session_id: correlation.native_session_id,
            native_message_id,
            eligible_tool_ids: request.eligible_tool_ids.clone(),
            request_origin: authority.request_origin,
            configuration_version: authority.configuration_version,
            policy_version: authority.policy_version,
            revocation_epoch: authority.revocation_epoch,
        };
        resolver
            .activate(
                context.clone(),
                authority.active_from_ms,
                authority.expires_at_ms,
            )
            .map_err(|_| PeerDispatchError::Dispatch)?;
        self.broker_contexts
            .insert(request.identity.session_id.clone(), context);
        Ok(())
    }

    fn dispatch(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        if request.model.credential_reference.is_some()
            || request.model.credential_lease_id.is_some()
        {
            return Err(PeerDispatchError::Dispatch);
        }
        let correlation = self.correlation(&request.identity)?;
        self.adapter
            .send_with_provider_credential(PromptDispatch {
                correlation,
                model: ModelRoute {
                    provider_id: request.model.provider_id.clone(),
                    model_id: request.model.model_id.clone(),
                },
                text: request.input.clone(),
                eligible_tool_ids: request.eligible_tool_ids.clone(),
                attachments: request
                    .direct_attachments
                    .iter()
                    .map(|attachment| PromptAttachment {
                        attachment_id: attachment.snapshot().attachment_id.clone(),
                        stable_reference: attachment.snapshot().stable_reference.clone(),
                        display_name: attachment.snapshot().display_name.clone(),
                        media_type: attachment.snapshot().media_type.clone(),
                        byte_length: attachment.snapshot().byte_length,
                        content_sha256: attachment.snapshot().content_sha256.clone(),
                        snapshot_version: attachment.snapshot().snapshot_version,
                        content: attachment.content().to_vec(),
                    })
                    .collect(),
                native_overrides: Value::Object(Default::default()),
            })
            .map_err(|_| PeerDispatchError::Dispatch)?;
        self.active.insert(
            request.identity.session_id.clone(),
            request.identity.clone(),
        );
        Ok(())
    }

    fn poll_events(
        &mut self,
        recorded_at_ms: u64,
    ) -> Result<Vec<PeerDispatchEvent>, PeerDispatchError> {
        #[cfg(unix)]
        {
            let frames = match self
                .event_stream
                .as_mut()
                .ok_or(PeerDispatchError::UnsupportedEventSource)?
                .drain()
            {
                Ok(frames) => frames,
                Err(_) if !self.active.is_empty() => {
                    self.pending_sse.clear();
                    return Ok(self
                        .active
                        .values()
                        .cloned()
                        .map(|identity| PeerDispatchEvent {
                            identity,
                            recorded_at_ms,
                            payload: "opencode-event-stream-failed".into(),
                            category: DispatchEventCategory::Error {
                                code: "opencode-event-stream-failed".into(),
                            },
                        })
                        .collect());
                }
                Err(_) => return Err(PeerDispatchError::EventStream),
            };
            for frame in frames {
                let Some(c4os_session_id) = active_c4os_session_for_native_frame(
                    &self.native_sessions,
                    &self.active,
                    &frame.native_session_id,
                )?
                else {
                    // OpenCode can publish authenticated bookkeeping after a
                    // completed attempt in a later stream drain. The exact
                    // app-created native session is known but inactive, so
                    // these frames cannot authorize an effect and are dropped.
                    continue;
                };
                self.queue_sse(&c4os_session_id, frame.frame, frame.received_at_ms)?;
            }
        }
        let frames = std::mem::take(&mut self.pending_sse);
        let mut events = Vec::with_capacity(frames.len());
        let mut terminal_sessions = BTreeSet::new();
        for queued in frames {
            // OpenCode can enqueue authenticated bookkeeping updates after a
            // terminal session event in the same bounded drain. The terminal
            // event closes the C4OS attempt; later frames from that same batch
            // cannot authorize effects and are dropped before active identity
            // removal. A frame arriving in a later drain still fails closed as
            // stale because the active session mapping no longer exists.
            if terminal_sessions.contains(&queued.session_id) {
                continue;
            }
            let identity = self
                .active
                .get(&queued.session_id)
                .cloned()
                .ok_or(PeerDispatchError::StaleIdentity)?;
            let correlation = self.correlation(&identity)?;
            let event =
                match self
                    .adapter
                    .normalize_sse(&correlation, &queued.frame, queued.received_at_ms)
                {
                    Ok(event) => event,
                    Err(error) => match map_opencode_event_error(error) {
                        Some(error) => return Err(error),
                        None => continue,
                    },
                };
            if let Some(assistant) = &event.authenticated_assistant_message {
                match (
                    self.broker_resolver.as_ref(),
                    self.broker_contexts.get(&queued.session_id),
                ) {
                    (Some(resolver), Some(context)) => resolver
                        .publish_authenticated_assistant_alias(
                            context,
                            AuthenticatedAssistantMessageEvidence {
                                native_session_id: event.native_session_id.clone(),
                                native_message_id: assistant.native_message_id.clone(),
                                role: assistant.role.clone(),
                                parent_native_message_id: Some(
                                    assistant.parent_native_message_id.clone(),
                                ),
                                process_generation: event.process_generation,
                            },
                            event.received_at_ms,
                        )
                        .map_err(|_| PeerDispatchError::StaleIdentity)?,
                    (None, None) => {}
                    _ => return Err(PeerDispatchError::StaleIdentity),
                }
            }
            let normalized = normalize_opencode_event(identity.clone(), event)?;
            if normalized.terminal_outcome().is_some() {
                terminal_sessions.insert(identity.session_id.clone());
            }
            events.push(normalized);
        }
        for session_id in terminal_sessions {
            self.active.remove(&session_id);
        }
        Ok(events)
    }

    fn cancel(&mut self, identity: &DispatchIdentity) -> Result<bool, PeerDispatchError> {
        let correlation = self.correlation(identity)?;
        let accepted = self
            .adapter
            .cancel(&correlation)
            .map_err(|_| PeerDispatchError::Cancellation)?;
        if accepted {
            self.active.remove(&identity.session_id);
        }
        Ok(accepted)
    }

    fn shutdown(&mut self) -> Result<(), PeerDispatchError> {
        let mut context_result = Ok(());
        if let Some(resolver) = &self.broker_resolver {
            for (_, context) in std::mem::take(&mut self.broker_contexts) {
                if resolver.retire(&context).is_err() {
                    context_result = Err(PeerDispatchError::Shutdown);
                }
            }
        }
        #[cfg(unix)]
        let stream_result = if let Some(mut stream) = self.event_stream.take() {
            stream.shutdown().map_err(|_| PeerDispatchError::Shutdown)
        } else {
            Ok(())
        };
        #[cfg(not(unix))]
        let stream_result: Result<(), PeerDispatchError> = Ok(());
        let adapter_result = self.adapter.stop().map_err(|_| PeerDispatchError::Shutdown);
        context_result.and(stream_result).and(adapter_result)
    }

    fn forget_attempt(&mut self, identity: &DispatchIdentity) {
        if let Some(context) = self.broker_contexts.remove(&identity.session_id)
            && let Some(resolver) = &self.broker_resolver
        {
            let _ = resolver.retire(&context);
        }
        self.active.remove(&identity.session_id);
        self.pending_sse
            .retain(|frame| frame.session_id != identity.session_id);
    }
}

fn active_c4os_session_for_native_frame(
    native_sessions: &BTreeMap<String, String>,
    active: &BTreeMap<String, DispatchIdentity>,
    native_session_id: &str,
) -> Result<Option<String>, PeerDispatchError> {
    let c4os_session_id = native_sessions
        .get(native_session_id)
        .ok_or(PeerDispatchError::StaleIdentity)?;
    Ok(active
        .contains_key(c4os_session_id)
        .then(|| c4os_session_id.clone()))
}

impl<T: OpenCodeTransport, C: CommandDriver> OpenCodeDispatchPeer<T, C> {
    fn queue_sse(
        &mut self,
        session_id: &str,
        frame: Vec<u8>,
        received_at_ms: u64,
    ) -> Result<(), PeerDispatchError> {
        if !self.active.contains_key(session_id)
            || frame.is_empty()
            || received_at_ms == 0
            || self.pending_sse.len() >= MAX_PENDING_SSE_FRAMES
        {
            return Err(PeerDispatchError::StaleIdentity);
        }
        self.pending_sse.push_back(OpenCodeSseFrame {
            session_id: session_id.into(),
            frame,
            received_at_ms,
        });
        Ok(())
    }
}

/// OpenCode's event stream can replay an identical SSE update. The adapter
/// remembers and rejects that duplicate before any normalized event or effect
/// is produced; the peer therefore drops it idempotently while preserving
/// fail-closed handling for stale correlation and every other protocol error.
fn map_opencode_event_error(error: OpenCodeError) -> Option<PeerDispatchError> {
    match error {
        OpenCodeError::DuplicateEvent => None,
        OpenCodeError::LateEvent | OpenCodeError::StaleCorrelation => {
            Some(PeerDispatchError::StaleIdentity)
        }
        _ => Some(PeerDispatchError::EventStream),
    }
}

#[cfg(test)]
mod opencode_event_error_tests {
    use super::*;

    fn identity(session_id: &str) -> DispatchIdentity {
        DispatchIdentity {
            workspace_id: "workspace-1".into(),
            environment_id: "local".into(),
            session_id: session_id.into(),
            turn_id: "turn-1".into(),
            attempt_id: "attempt-1".into(),
            correlation_id: "correlation-1".into(),
            runtime_id: "opencode-primary".into(),
            runtime_kind: RuntimeKind::OpenCode,
            adapter_version: DISPATCH_ADAPTER_VERSION.into(),
            native_version: "1.18.3".into(),
            process_generation: 1,
        }
    }

    #[test]
    fn duplicate_updates_are_idempotently_dropped_but_stale_events_fail_closed() {
        assert_eq!(
            map_opencode_event_error(OpenCodeError::DuplicateEvent),
            None
        );
        assert_eq!(
            map_opencode_event_error(OpenCodeError::StaleCorrelation),
            Some(PeerDispatchError::StaleIdentity)
        );
        assert_eq!(
            map_opencode_event_error(OpenCodeError::InvalidEventPayload),
            Some(PeerDispatchError::EventStream)
        );
    }

    #[test]
    fn native_frames_route_only_to_active_known_sessions_and_drop_known_terminal_tail() {
        let native_sessions = BTreeMap::from([
            ("native-active".into(), "session-active".into()),
            ("native-terminal".into(), "session-terminal".into()),
        ]);
        let active = BTreeMap::from([("session-active".into(), identity("session-active"))]);

        assert_eq!(
            active_c4os_session_for_native_frame(&native_sessions, &active, "native-active"),
            Ok(Some("session-active".into()))
        );
        assert_eq!(
            active_c4os_session_for_native_frame(&native_sessions, &active, "native-terminal"),
            Ok(None)
        );
        assert_eq!(
            active_c4os_session_for_native_frame(&native_sessions, &active, "native-unknown"),
            Err(PeerDispatchError::StaleIdentity)
        );
    }
}

pub(crate) fn normalize_opencode_event(
    identity: DispatchIdentity,
    event: NormalizedEvent,
) -> Result<PeerDispatchEvent, PeerDispatchError> {
    if event.workspace_id != identity.workspace_id
        || event.c4os_session_id != identity.session_id
        || event.c4os_turn_id != identity.turn_id
        || event.c4os_run_id != identity.attempt_id
        || event.correlation_id != identity.correlation_id
        || event.process_generation != identity.process_generation
    {
        return Err(PeerDispatchError::StaleIdentity);
    }
    let (category, payload) = match event.category {
        NormalizedEventCategory::Lifecycle { state } => (DispatchEventCategory::Lifecycle, state),
        NormalizedEventCategory::TextDelta { delta } => (DispatchEventCategory::TextDelta, delta),
        NormalizedEventCategory::ThinkingDelta { delta } => {
            (DispatchEventCategory::ReasoningDelta, delta)
        }
        NormalizedEventCategory::ActionIntent(intent) => {
            let mut action_identity = RuntimeIntentIdentity::from_opencode(&intent)
                .map_err(|_| PeerDispatchError::StaleIdentity)?;
            if action_identity.runtime_id != RuntimeKind::OpenCode.as_str() {
                return Err(PeerDispatchError::StaleIdentity);
            }
            // The native protocol identifies an adapter kind (`opencode`).
            // The trusted registry owns the concrete installation identity.
            action_identity.runtime_id = identity.runtime_id.clone();
            (
                DispatchEventCategory::ActionIntent(Box::new(action_identity)),
                "action-intent".into(),
            )
        }
        NormalizedEventCategory::ToolProgress { summary, .. } => {
            (DispatchEventCategory::ActionProgress, summary)
        }
        NormalizedEventCategory::ToolResult { succeeded, .. } => (
            DispatchEventCategory::ActionResult,
            if succeeded { "succeeded" } else { "failed" }.into(),
        ),
        NormalizedEventCategory::Usage {
            input_tokens,
            output_tokens,
        } => (
            DispatchEventCategory::Usage,
            format!("input={input_tokens};output={output_tokens}"),
        ),
        NormalizedEventCategory::Completed => {
            (DispatchEventCategory::Completed, "completed".into())
        }
        NormalizedEventCategory::Cancelled => {
            (DispatchEventCategory::Cancelled, "cancelled".into())
        }
        NormalizedEventCategory::Error { code } => {
            (DispatchEventCategory::Error { code: code.clone() }, code)
        }
        NormalizedEventCategory::Unknown => (DispatchEventCategory::Other, "unknown".into()),
    };
    Ok(PeerDispatchEvent {
        identity,
        recorded_at_ms: event.received_at_ms,
        payload,
        category,
    })
}

pub struct PiDispatchPeer<R: PiSidecarRunner> {
    registration: RuntimePeerRegistration,
    adapter: PiAdapter<R>,
    active: BTreeMap<String, DispatchIdentity>,
    created_sessions: BTreeMap<String, PiCreatedSessionRoute>,
    credential_issuer: Option<Box<dyn PiDispatchCredentialIssuer>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PiCreatedSessionRoute {
    c4os_provider_id: String,
    native_provider_id: String,
    model_id: String,
    base_url: String,
    eligible_tool_ids: BTreeSet<String>,
}

impl<R: PiSidecarRunner> PiDispatchPeer<R> {
    pub fn new(
        registration: RuntimePeerRegistration,
        adapter: PiAdapter<R>,
    ) -> Result<Self, DispatchError> {
        registration.validate()?;
        if registration.descriptor.runtime_kind != RuntimeKind::Pi
            || adapter.manifest().native_version != registration.descriptor.native_version
            || registration.descriptor.adapter_version != DISPATCH_ADAPTER_VERSION
        {
            return Err(DispatchError::IncompatiblePeer);
        }
        Ok(Self {
            registration,
            adapter,
            active: BTreeMap::new(),
            created_sessions: BTreeMap::new(),
            credential_issuer: None,
        })
    }

    pub fn attach_credential_issuer(&mut self, issuer: impl PiDispatchCredentialIssuer + 'static) {
        self.credential_issuer = Some(Box::new(issuer));
    }
}

impl<R: PiSidecarRunner + Send> RuntimeDispatchPeer for PiDispatchPeer<R> {
    fn registration(&self) -> &RuntimePeerRegistration {
        &self.registration
    }

    fn readiness(&self) -> Result<(), PeerDispatchError> {
        if !matches!(
            self.adapter.state(),
            PiAdapterState::Ready | PiAdapterState::Degraded
        ) {
            return Err(PeerDispatchError::NotReady);
        }
        Ok(())
    }

    fn create_session(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        let (provider, native_model_id, base_url) = match self.credential_issuer.as_ref() {
            Some(issuer) => {
                if request.model.credential_reference.is_some()
                    || request.model.credential_lease_id.is_some()
                {
                    return Err(PeerDispatchError::Credential);
                }
                let provider = issuer.native_provider_id(&request.model.provider_id)?;
                let base_url = issuer.base_url(&request.model.provider_id)?;
                let model_id = request
                    .model
                    .model_id
                    .strip_prefix(&format!("{provider}/"))
                    .filter(|model_id| !model_id.is_empty())
                    .ok_or(PeerDispatchError::SessionCreate)?
                    .to_owned();
                (provider, model_id, base_url)
            }
            None => (
                request.model.provider_id.clone(),
                request.model.model_id.clone(),
                "https://api.openai.com/v1".into(),
            ),
        };
        let route = PiCreatedSessionRoute {
            c4os_provider_id: request.model.provider_id.clone(),
            native_provider_id: provider.clone(),
            model_id: request.model.model_id.clone(),
            base_url: base_url.clone(),
            eligible_tool_ids: request.eligible_tool_ids.clone(),
        };
        if let Some(created) = self.created_sessions.get(&request.identity.session_id) {
            return if created == &route {
                Ok(())
            } else {
                Err(PeerDispatchError::RouteConflict)
            };
        }
        self.adapter
            .create_session(
                &request.identity.workspace_id,
                &request.identity.session_id,
                PiModelRoute {
                    provider,
                    model_id: native_model_id,
                    base_url,
                },
                &request.eligible_tool_ids,
            )
            .map_err(|_| PeerDispatchError::SessionCreate)?;
        self.created_sessions
            .insert(request.identity.session_id.clone(), route);
        Ok(())
    }

    fn dispatch(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        let attachments = request
            .direct_attachments
            .iter()
            .map(|attachment| PiDispatchAttachment {
                attachment_id: attachment.snapshot().attachment_id.clone(),
                stable_reference: attachment.snapshot().stable_reference.clone(),
                display_name: attachment.snapshot().display_name.clone(),
                media_type: attachment.snapshot().media_type.clone(),
                byte_length: attachment.snapshot().byte_length,
                content_sha256: attachment.snapshot().content_sha256.clone(),
                snapshot_version: attachment.snapshot().snapshot_version,
                content: attachment.content().to_vec(),
            })
            .collect::<Vec<_>>();
        self.adapter
            .validate_dispatch_with_attachments(
                &request.identity.workspace_id,
                &request.identity.session_id,
                &request.identity.turn_id,
                &request.identity.attempt_id,
                &request.identity.correlation_id,
                &request.input,
                &attachments,
            )
            .map_err(|_| PeerDispatchError::Dispatch)?;
        let credential_operation = match self.credential_issuer.as_mut() {
            Some(issuer) => {
                if request.model.credential_reference.is_some()
                    || request.model.credential_lease_id.is_some()
                {
                    return Err(PeerDispatchError::Credential);
                }
                issuer.deliver_for_dispatch(&request.identity, &request.model.provider_id)?;
                Some((
                    request.identity.runtime_id.as_str(),
                    request.model.provider_id.as_str(),
                ))
            }
            None => None,
        };
        self.adapter
            .dispatch_with_credential_operation(
                &request.identity.workspace_id,
                &request.identity.session_id,
                &request.identity.turn_id,
                &request.identity.attempt_id,
                &request.identity.correlation_id,
                &request.input,
                &attachments,
                credential_operation,
            )
            .map_err(|_| PeerDispatchError::Dispatch)?;
        self.active.insert(
            request.identity.session_id.clone(),
            request.identity.clone(),
        );
        Ok(())
    }

    fn poll_events(
        &mut self,
        recorded_at_ms: u64,
    ) -> Result<Vec<PeerDispatchEvent>, PeerDispatchError> {
        let events = self
            .adapter
            .poll_events()
            .map_err(|_| PeerDispatchError::EventStream)?;
        events
            .into_iter()
            .map(|event| {
                let identity = self
                    .active
                    .get(&event.session_id)
                    .cloned()
                    .ok_or(PeerDispatchError::StaleIdentity)?;
                let normalized = normalize_pi_event(identity.clone(), event, recorded_at_ms)?;
                if normalized.terminal_outcome().is_some() {
                    self.active.remove(&identity.session_id);
                }
                Ok(normalized)
            })
            .collect()
    }

    fn cancel(&mut self, identity: &DispatchIdentity) -> Result<bool, PeerDispatchError> {
        let accepted = self
            .adapter
            .cancel(
                &identity.session_id,
                &identity.attempt_id,
                &identity.correlation_id,
            )
            .map_err(|_| PeerDispatchError::Cancellation)?;
        if accepted {
            self.active.remove(&identity.session_id);
        }
        Ok(accepted)
    }

    fn shutdown(&mut self) -> Result<(), PeerDispatchError> {
        self.adapter
            .shutdown()
            .map_err(|_| PeerDispatchError::Shutdown)
    }

    fn resolve_pi_denied(
        &mut self,
        identity: &DispatchIdentity,
        native_request_id: &str,
        reason_code: &str,
    ) -> Result<(), PeerDispatchError> {
        if self.active.get(&identity.session_id) != Some(identity) {
            return Err(PeerDispatchError::StaleIdentity);
        }
        self.adapter
            .resolve_tool(
                &identity.session_id,
                &identity.attempt_id,
                &identity.correlation_id,
                native_request_id,
                PiToolDecision::Denied {
                    reason: reason_code.to_owned(),
                },
            )
            .map_err(|_| PeerDispatchError::UnsupportedToolResolution)
    }

    fn resolve_pi_completed(
        &mut self,
        identity: &DispatchIdentity,
        native_request_id: &str,
        receipt: &RuntimeExecutionReceipt,
    ) -> Result<(), PeerDispatchError> {
        if self.active.get(&identity.session_id) != Some(identity) {
            return Err(PeerDispatchError::StaleIdentity);
        }
        self.adapter
            .resolve_completed_tool(
                &identity.runtime_id,
                &identity.session_id,
                &identity.attempt_id,
                &identity.correlation_id,
                native_request_id,
                receipt,
            )
            .map_err(|_| PeerDispatchError::UnsupportedToolResolution)
    }

    fn forget_attempt(&mut self, identity: &DispatchIdentity) {
        self.active.remove(&identity.session_id);
    }
}

pub(crate) fn normalize_pi_event(
    identity: DispatchIdentity,
    event: PiEventEnvelope,
    recorded_at_ms: u64,
) -> Result<PeerDispatchEvent, PeerDispatchError> {
    if event.workspace_id != identity.workspace_id
        || event.session_id != identity.session_id
        || event.turn_id != identity.turn_id
        || event.run_id != identity.attempt_id
        || event.correlation_id != identity.correlation_id
        || event.process_generation != identity.process_generation
    {
        return Err(PeerDispatchError::StaleIdentity);
    }
    let (category, payload) = match event.category.as_str() {
        "content.delta" | "text.delta" | "assistant.text.delta" => (
            DispatchEventCategory::TextDelta,
            event
                .payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        ),
        "reasoning.delta" => (
            DispatchEventCategory::ReasoningDelta,
            event
                .payload
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        ),
        "tool.action_intent" => {
            let mut action_identity = RuntimeIntentIdentity::from_pi(&event)
                .map_err(|_| PeerDispatchError::StaleIdentity)?;
            action_identity.runtime_id = identity.runtime_id.clone();
            let arguments = event
                .payload
                .get("arguments")
                .filter(|arguments| arguments.is_object())
                .cloned()
                .ok_or(PeerDispatchError::StaleIdentity)?;
            (
                DispatchEventCategory::PiActionIntent(Box::new(PiActionIntent {
                    identity: action_identity,
                    arguments,
                })),
                "action-intent".into(),
            )
        }
        "tool.progress" => (
            DispatchEventCategory::ActionProgress,
            "tool-progress".into(),
        ),
        "tool.result" => (DispatchEventCategory::ActionResult, "tool-result".into()),
        "usage" => (DispatchEventCategory::Usage, event.payload.to_string()),
        "lifecycle.settled" => (DispatchEventCategory::Completed, "completed".into()),
        "lifecycle.cancelled" => (DispatchEventCategory::Cancelled, "cancelled".into()),
        "lifecycle.error" => {
            let code = event
                .payload
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("pi-runtime-error")
                .to_owned();
            (DispatchEventCategory::Error { code: code.clone() }, code)
        }
        category if category.starts_with("lifecycle.") => {
            (DispatchEventCategory::Lifecycle, category.into())
        }
        _ => (DispatchEventCategory::Other, event.payload.to_string()),
    };
    Ok(PeerDispatchEvent {
        identity,
        recorded_at_ms,
        payload,
        category,
    })
}

#[cfg(unix)]
pub type ProductionOpenCodeDispatchPeer = OpenCodeDispatchPeer<
    crate::runtime::opencode_native::LoopbackHttpTransport,
    crate::runtime::opencode_native::OpenCodeNativeCommandDriver,
>;

#[cfg(unix)]
pub type ProductionPiDispatchPeer = PiDispatchPeer<crate::runtime::pi_process::SpawnedPiRunner>;

fn validate_id(value: &str) -> Result<(), DispatchError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@' | b'/')
        })
    {
        return Err(DispatchError::InvalidIdentity);
    }
    Ok(())
}

fn validate_sha256(value: &str) -> Result<(), DispatchError> {
    if value.len() != 71
        || !value.starts_with("sha256:")
        || !value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(DispatchError::InvalidEvent);
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    prefixed_digest(Sha256::digest(bytes))
}

fn prefixed_digest(digest: impl AsRef<[u8]>) -> String {
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest.as_ref() {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}
