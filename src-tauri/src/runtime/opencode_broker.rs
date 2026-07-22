//! Rust-owned pump between the authenticated OpenCode SDK descriptor and the
//! C4OS Action Gateway.
//!
//! The native process contributes only a bounded broker proposal. Exact
//! facility classification, approval continuation, version revalidation, and
//! effect execution remain in this core-owned surface. Nothing in this module
//! is exposed through Tauri or the renderer.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Map, Value};
use sha2::Digest as _;
use thiserror::Error;

use crate::mcp::{
    McpTransportKind, McpTurnToolSnapshot, mcp_authority_identity, validate_mcp_input_schema,
};
use crate::runtime::action_bridge::RuntimeEffectResult;
use crate::runtime::broker_worker::{
    AuthenticatedBrokerEvent, AuthenticatedBrokerEventKind, AuthenticatedBrokerMetadata,
    BrokerActionApplication, BrokerActionClassifier, BrokerActionContext, BrokerActionWorker,
    BrokerApprovalAnswer, BrokerClassificationError, BrokerDeferredStart, BrokerDeferredTicket,
    BrokerEffectExecutor, BrokerWorkerError, BrokerWorkerOutcome, ResolvedBrokerAction,
};
use crate::runtime::opencode::{C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL};
use crate::runtime::opencode_native::{NativeBrokerError, OpenCodeNativeCommandDriver};
use crate::runtime::opencode_sdk::{BrokerDecision, OpenCodeSdkBroker, OpenCodeSdkError};
use crate::security::authorization::ApprovalAnswer;
use crate::security::gateway::{ExecutionPermit, NormalizedActionResult};
use crate::security::policy::{
    ActionEffect, ActionReversibility, ActionScope, ActionSensitivity, ActionSurface,
    RepositoryState,
};

const MAX_BROKER_FACILITIES: usize = 128;
const MAX_ROUTE_TEXT_BYTES: usize = 512;
const MAX_ARGUMENT_BYTES: usize = 256 * 1024;
const MAX_IO_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_ACTIVE_BROKER_CONTEXTS: usize = 4_096;

/// Core-owned classification installed for one exact facility route.
///
/// Target versions are intentionally absent. They are obtained from the
/// installed facility at classification time and re-read immediately before
/// the authorized effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledBrokerClassification {
    pub surface: ActionSurface,
    pub effects: BTreeSet<ActionEffect>,
    pub scope: ActionScope,
    pub sensitivity: ActionSensitivity,
    pub reversibility: ActionReversibility,
    pub repository_state: RepositoryState,
    pub inside_active_project: bool,
    pub canonical_target: String,
    pub normalized_arguments: Map<String, Value>,
    pub trusted_root: bool,
    pub explicit_scope_grant: bool,
    pub sandbox_allows: bool,
    pub declaration_exceeded: bool,
}

/// Executable facility installed by the Rust core. Broker frames cannot
/// implement this trait, obtain an `ExecutionPermit`, or manufacture a result.
pub trait InstalledBrokerFacility: Send {
    /// Returns the current stable target generation/version. `None` means the
    /// facility or target is presently unavailable.
    fn current_target_version(&mut self) -> Option<String>;

    /// Runs only after the Action Gateway consumed the exact authorization.
    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult;
}

/// Single core-installed deferred namespace used by long-running broker
/// facilities such as MCP. The registry remains sealed; only the route data in
/// a consumed ExecutionPermit can select work after startup.
pub trait InstalledDeferredBrokerFacility: Send {
    fn start(&mut self, permit: ExecutionPermit) -> BrokerDeferredStart;
    fn poll(&mut self, ticket: &BrokerDeferredTicket) -> Option<RuntimeEffectResult>;
    fn cancel(&mut self, ticket: &BrokerDeferredTicket) -> bool;
    /// Cancels and removes a deferred operation when its outer gateway lease
    /// is being settled independently during runtime teardown.
    fn abandon(&mut self, ticket: &BrokerDeferredTicket) -> bool;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum FacilityKey {
    Resource {
        resource: String,
        selector: Option<String>,
    },
    Action {
        operation: String,
        target: String,
    },
}

struct FacilityEntry {
    classification: InstalledBrokerClassification,
    expected_arguments: Map<String, Value>,
    facility: Box<dyn InstalledBrokerFacility>,
}

#[derive(Default)]
struct FacilityRegistryState {
    entries: BTreeMap<FacilityKey, FacilityEntry>,
    deferred: Option<Box<dyn InstalledDeferredBrokerFacility>>,
    sealed: bool,
}

/// Bounded, sealable registry used by both classification and execution.
/// Clones share the same sealed core-owned installation set.
#[derive(Clone, Default)]
pub struct InstalledBrokerFacilityRegistry {
    state: Arc<Mutex<FacilityRegistryState>>,
}

impl InstalledBrokerFacilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install_resource(
        &self,
        resource: impl Into<String>,
        selector: Option<String>,
        classification: InstalledBrokerClassification,
        facility: Box<dyn InstalledBrokerFacility>,
    ) -> Result<(), FacilityRegistryError> {
        let resource = resource.into();
        validate_route_text(&resource)?;
        if let Some(selector) = &selector {
            validate_route_text(selector)?;
        }
        validate_classification(&classification, C4OS_RESOURCE_READ_TOOL)?;
        self.install(
            FacilityKey::Resource { resource, selector },
            Map::new(),
            classification,
            facility,
        )
    }

    pub fn install_action(
        &self,
        operation: impl Into<String>,
        target: impl Into<String>,
        expected_arguments: Map<String, Value>,
        classification: InstalledBrokerClassification,
        facility: Box<dyn InstalledBrokerFacility>,
    ) -> Result<(), FacilityRegistryError> {
        let operation = operation.into();
        let target = target.into();
        validate_route_text(&operation)?;
        validate_route_text(&target)?;
        validate_arguments(&expected_arguments)?;
        validate_classification(&classification, C4OS_ACTION_PROPOSAL_TOOL)?;
        self.install(
            FacilityKey::Action { operation, target },
            expected_arguments,
            classification,
            facility,
        )
    }

    pub fn install_deferred(
        &self,
        facility: Box<dyn InstalledDeferredBrokerFacility>,
    ) -> Result<(), FacilityRegistryError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| FacilityRegistryError::Unavailable)?;
        if state.sealed {
            return Err(FacilityRegistryError::Sealed);
        }
        if state.deferred.is_some() {
            return Err(FacilityRegistryError::Conflict);
        }
        state.deferred = Some(facility);
        Ok(())
    }

    fn install(
        &self,
        key: FacilityKey,
        expected_arguments: Map<String, Value>,
        classification: InstalledBrokerClassification,
        facility: Box<dyn InstalledBrokerFacility>,
    ) -> Result<(), FacilityRegistryError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| FacilityRegistryError::Unavailable)?;
        if state.sealed {
            return Err(FacilityRegistryError::Sealed);
        }
        if state.entries.len() >= MAX_BROKER_FACILITIES {
            return Err(FacilityRegistryError::Capacity);
        }
        if state.entries.contains_key(&key) {
            return Err(FacilityRegistryError::Conflict);
        }
        state.entries.insert(
            key,
            FacilityEntry {
                classification,
                expected_arguments,
                facility,
            },
        );
        Ok(())
    }

    pub(crate) fn seal(&self) -> Result<(), FacilityRegistryError> {
        self.state
            .lock()
            .map_err(|_| FacilityRegistryError::Unavailable)?
            .sealed = true;
        Ok(())
    }

    fn resolve(
        &self,
        key: &FacilityKey,
        arguments: &Map<String, Value>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| BrokerClassificationError::Unavailable)?;
        if !state.sealed {
            return Err(BrokerClassificationError::Unavailable);
        }
        let entry = state
            .entries
            .get_mut(key)
            .ok_or(BrokerClassificationError::Unsupported)?;
        if &entry.expected_arguments != arguments {
            return Err(BrokerClassificationError::Unsupported);
        }
        let target_version = entry
            .facility
            .current_target_version()
            .filter(|version| valid_version(version))
            .ok_or(BrokerClassificationError::Unavailable)?;
        Ok(resolved(&entry.classification, target_version))
    }

    fn execute_permit(&self, permit: ExecutionPermit) -> NormalizedActionResult {
        let completed_at_ms = nonzero_time(permit.consumed_at_ms());
        let Some((key, arguments)) = route_from_permit(&permit) else {
            return NormalizedActionResult::denied(
                "broker-permit-binding-mismatch",
                completed_at_ms,
            );
        };
        let Ok(mut state) = self.state.lock() else {
            return NormalizedActionResult::denied("broker-facility-unavailable", completed_at_ms);
        };
        if !state.sealed {
            return NormalizedActionResult::denied("broker-registry-unsealed", completed_at_ms);
        }
        let Some(entry) = state.entries.get_mut(&key) else {
            return NormalizedActionResult::denied("broker-facility-uninstalled", completed_at_ms);
        };
        if entry.expected_arguments != arguments
            || permit.action().canonical_target != entry.classification.canonical_target
            || permit.action().arguments.get("resolved")
                != Some(&Value::Object(
                    entry.classification.normalized_arguments.clone(),
                ))
        {
            return NormalizedActionResult::denied(
                "broker-permit-binding-mismatch",
                completed_at_ms,
            );
        }
        let Some(current_version) = entry
            .facility
            .current_target_version()
            .filter(|version| valid_version(version))
        else {
            return NormalizedActionResult::denied("broker-target-unavailable", completed_at_ms);
        };
        if current_version != permit.action().target_version {
            return NormalizedActionResult::denied(
                "broker-target-version-changed",
                completed_at_ms,
            );
        }
        entry.facility.execute(permit)
    }
}

impl fmt::Debug for InstalledBrokerFacilityRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (installed, sealed) = self
            .state
            .lock()
            .map(|state| (state.entries.len(), state.sealed))
            .unwrap_or((0, true));
        formatter
            .debug_struct("InstalledBrokerFacilityRegistry")
            .field("installed", &installed)
            .field("sealed", &sealed)
            .finish()
    }
}

impl BrokerActionClassifier for InstalledBrokerFacilityRegistry {
    fn resolve_resource(
        &mut self,
        _context: &BrokerActionContext,
        resource: &str,
        selector: Option<&str>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        self.resolve(
            &FacilityKey::Resource {
                resource: resource.to_owned(),
                selector: selector.map(str::to_owned),
            },
            &Map::new(),
        )
    }

    fn resolve_action(
        &mut self,
        context: &BrokerActionContext,
        operation: &str,
        target: &str,
        arguments: &Map<String, Value>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        if operation == "mcp.call-tool" {
            return resolve_mcp_action(context, target, arguments);
        }
        self.resolve(
            &FacilityKey::Action {
                operation: operation.to_owned(),
                target: target.to_owned(),
            },
            arguments,
        )
    }
}

fn resolve_mcp_action(
    context: &BrokerActionContext,
    target: &str,
    arguments: &Map<String, Value>,
) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
    validate_arguments(arguments).map_err(|_| BrokerClassificationError::Ambiguous)?;
    let snapshot = context
        .mcp_turn
        .as_ref()
        .ok_or(BrokerClassificationError::Unsupported)?;
    let tool = snapshot
        .tools
        .iter()
        .find(|tool| tool.target_id == target)
        .ok_or(BrokerClassificationError::Unsupported)?;
    validate_mcp_input_schema(&tool.input_schema, Some(&Value::Object(arguments.clone())))
        .map_err(|_| BrokerClassificationError::Ambiguous)?;
    if snapshot.workspace_id != context.dispatch.workspace_id
        || snapshot.session_id != context.dispatch.session_id
    {
        return Err(BrokerClassificationError::Unavailable);
    }
    let target_version = mcp_target_version(tool)?;
    let authority_id = mcp_authority_id(tool)?;
    Ok(ResolvedBrokerAction {
        surface: match tool.transport_kind {
            McpTransportKind::Stdio => ActionSurface::Process,
            McpTransportKind::StreamableHttp => ActionSurface::Network,
        },
        effects: BTreeSet::from([ActionEffect::Execute]),
        scope: match tool.transport_kind {
            McpTransportKind::Stdio => ActionScope::ExternalLocal,
            McpTransportKind::StreamableHttp => ActionScope::Remote,
        },
        sensitivity: match tool.transport_kind {
            McpTransportKind::Stdio => ActionSensitivity::Ordinary,
            McpTransportKind::StreamableHttp => ActionSensitivity::Authenticated,
        },
        reversibility: ActionReversibility::Destructive,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: false,
        canonical_target: tool.target_id.clone(),
        target_version,
        normalized_arguments: serde_json::Map::from_iter([
            ("serverId".into(), Value::String(tool.server_id.clone())),
            (
                "lifecycleGeneration".into(),
                Value::Number(tool.lifecycle_generation.into()),
            ),
            (
                "definitionSha256".into(),
                Value::String(tool.definition_sha256.clone()),
            ),
            ("toolName".into(), Value::String(tool.tool_name.clone())),
            (
                "projectId".into(),
                Value::String(snapshot.project_id.clone()),
            ),
            (
                "turnId".into(),
                Value::String(context.dispatch.turn_id.clone()),
            ),
            (
                "inputSchemaSha256".into(),
                Value::String(tool.input_schema_sha256.clone()),
            ),
            (
                "turnSnapshotSha256".into(),
                Value::String(snapshot.sha256.clone()),
            ),
        ]),
        trusted_root: false,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
        confidence: crate::security::policy::ClassificationConfidence::Ambiguous,
        risk: crate::security::authorization::CanonicalRisk::High,
        plugin_or_mcp_id: Some(authority_id),
    })
}

fn mcp_target_version(tool: &McpTurnToolSnapshot) -> Result<String, BrokerClassificationError> {
    hash_mcp_binding(&serde_json::json!({
        "definitionSha256": tool.definition_sha256,
        "lifecycleGeneration": tool.lifecycle_generation,
        "toolName": tool.tool_name,
        "inputSchemaSha256": tool.input_schema_sha256,
        "outputSchemaSha256": tool.output_schema_sha256,
    }))
}

fn mcp_authority_id(tool: &McpTurnToolSnapshot) -> Result<String, BrokerClassificationError> {
    mcp_authority_identity(&tool.server_id, &tool.source)
        .map_err(|_| BrokerClassificationError::Ambiguous)
}

fn hash_mcp_binding(value: &Value) -> Result<String, BrokerClassificationError> {
    let encoded = serde_json::to_vec(value).map_err(|_| BrokerClassificationError::Ambiguous)?;
    let digest = sha2::Sha256::digest(encoded);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").map_err(|_| BrokerClassificationError::Ambiguous)?;
    }
    Ok(output)
}

impl BrokerEffectExecutor for InstalledBrokerFacilityRegistry {
    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult {
        self.execute_permit(permit)
    }

    fn start_deferred(&mut self, permit: ExecutionPermit) -> BrokerDeferredStart {
        let completed_at_ms = nonzero_time(permit.consumed_at_ms());
        let is_mcp = route_from_permit(&permit).is_some_and(|(key, _)| {
            matches!(key, FacilityKey::Action { operation, .. } if operation == "mcp.call-tool")
        }) && permit
            .action()
            .plugin_or_mcp_id
            .as_deref()
            .is_some_and(|identity| identity.starts_with("mcp:"));
        if !is_mcp {
            return BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                "deferred-broker-permit-binding-mismatch",
                completed_at_ms,
            ));
        }
        let Ok(mut state) = self.state.lock() else {
            return BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                "deferred-broker-facility-unavailable",
                completed_at_ms,
            ));
        };
        if !state.sealed {
            return BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                "broker-registry-unsealed",
                completed_at_ms,
            ));
        }
        match state.deferred.as_mut() {
            Some(facility) => facility.start(permit),
            None => BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                "deferred-broker-facility-uninstalled",
                completed_at_ms,
            )),
        }
    }

    fn poll_deferred(&mut self, ticket: &BrokerDeferredTicket) -> Option<RuntimeEffectResult> {
        self.state.lock().ok()?.deferred.as_mut()?.poll(ticket)
    }

    fn cancel_deferred(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        self.state
            .lock()
            .ok()
            .and_then(|mut state| {
                state
                    .deferred
                    .as_mut()
                    .map(|facility| facility.cancel(ticket))
            })
            .unwrap_or(false)
    }

    fn abandon_deferred(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        self.state
            .lock()
            .ok()
            .and_then(|mut state| {
                state
                    .deferred
                    .as_mut()
                    .map(|facility| facility.abandon(ticket))
            })
            .unwrap_or(false)
    }
}

mod context_resolver_sealed {
    pub trait Sealed {}
}

/// Resolves descriptor-authenticated native metadata to the one active C4OS
/// dispatch and its current live authority versions. A pump caller cannot
/// choose a context for an individual frame. The trait is sealed so only a
/// resolver owned by this Rust core can participate in the authority boundary.
pub trait BrokerActionContextResolver: context_resolver_sealed::Sealed {
    fn resolve_authenticated(
        &mut self,
        metadata: &AuthenticatedBrokerMetadata,
        now_ms: u64,
    ) -> Result<BrokerActionContext, BrokerContextResolutionError>;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BrokerContextKey {
    native_session_id: String,
    native_message_id: String,
    process_generation: u64,
}

impl BrokerContextKey {
    fn from_context(context: &BrokerActionContext) -> Self {
        Self {
            native_session_id: context.native_session_id.clone(),
            native_message_id: context.native_message_id.clone(),
            process_generation: context.dispatch.process_generation,
        }
    }

    fn from_metadata(metadata: &AuthenticatedBrokerMetadata) -> Self {
        Self {
            native_session_id: metadata.native_session_id().to_owned(),
            native_message_id: metadata.native_message_id().to_owned(),
            process_generation: metadata.process_generation(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveBrokerContext {
    context: BrokerActionContext,
    active_from_ms: u64,
    expires_at_ms: u64,
}

/// Authenticated `message.updated` identity facts obtained from OpenCode's
/// core-owned event stream. The resolver still validates every field against
/// the active C4OS attempt seed before publishing an assistant-message alias.
/// Broker/plugin payloads must never be used to construct this evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedAssistantMessageEvidence {
    pub native_session_id: String,
    pub native_message_id: String,
    pub role: String,
    pub parent_native_message_id: Option<String>,
    pub process_generation: u64,
}

#[derive(Default)]
struct BrokerContextRegistryState {
    attempt_seeds: BTreeMap<BrokerContextKey, ActiveBrokerContext>,
    assistant_aliases: BTreeMap<BrokerContextKey, BrokerContextKey>,
}

impl BrokerContextRegistryState {
    fn binding_count(&self) -> usize {
        self.attempt_seeds.len() + self.assistant_aliases.len()
    }
}

/// Core-owned active context map. Dispatch/event integration installs an exact
/// native session/message/process binding and refreshes it when live
/// configuration, policy, or revocation versions change. Approval resume goes
/// through the same map, so a changed binding cannot reuse the old grant.
#[derive(Clone, Default)]
pub struct ActiveBrokerContextResolver {
    state: Arc<Mutex<BrokerContextRegistryState>>,
}

impl ActiveBrokerContextResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(
        &self,
        context: BrokerActionContext,
        active_from_ms: u64,
        expires_at_ms: u64,
    ) -> Result<(), BrokerContextRegistryError> {
        validate_context_lifetime(&context, active_from_ms, expires_at_ms)?;
        let key = BrokerContextKey::from_context(&context);
        let mut state = self
            .state
            .lock()
            .map_err(|_| BrokerContextRegistryError::Unavailable)?;
        if state.binding_count() >= MAX_ACTIVE_BROKER_CONTEXTS {
            return Err(BrokerContextRegistryError::Capacity);
        }
        if state.attempt_seeds.contains_key(&key) || state.assistant_aliases.contains_key(&key) {
            return Err(BrokerContextRegistryError::Conflict);
        }
        state.attempt_seeds.insert(
            key,
            ActiveBrokerContext {
                context,
                active_from_ms,
                expires_at_ms,
            },
        );
        Ok(())
    }

    /// Publishes one assistant-message alias only after authenticated OpenCode
    /// evidence binds it to the exact core-created user message for an active
    /// attempt. The attempt seed remains unchanged and is the sole authority
    /// source for every alias and approval continuation.
    pub fn publish_authenticated_assistant_alias(
        &self,
        attempt_seed: &BrokerActionContext,
        evidence: AuthenticatedAssistantMessageEvidence,
        now_ms: u64,
    ) -> Result<(), BrokerContextRegistryError> {
        attempt_seed
            .validate()
            .map_err(|_| BrokerContextRegistryError::InvalidContext)?;
        validate_assistant_evidence(&evidence)?;

        let seed_key = BrokerContextKey::from_context(attempt_seed);
        let alias_key = BrokerContextKey {
            native_session_id: evidence.native_session_id.clone(),
            native_message_id: evidence.native_message_id.clone(),
            process_generation: evidence.process_generation,
        };
        let mut state = self
            .state
            .lock()
            .map_err(|_| BrokerContextRegistryError::Unavailable)?;
        let active = state
            .attempt_seeds
            .get(&seed_key)
            .ok_or(BrokerContextRegistryError::Unmapped)?;
        if !same_attempt_seed(&active.context, attempt_seed) {
            return Err(BrokerContextRegistryError::InvalidContext);
        }
        if now_ms == 0 || now_ms < active.active_from_ms || now_ms > active.expires_at_ms {
            return Err(BrokerContextRegistryError::Stale);
        }
        if evidence.native_session_id != active.context.native_session_id
            || evidence.process_generation != active.context.dispatch.process_generation
            || evidence.parent_native_message_id.as_deref()
                != Some(active.context.native_message_id.as_str())
            || evidence.native_message_id == active.context.native_message_id
        {
            return Err(BrokerContextRegistryError::EvidenceBindingMismatch);
        }
        if state.attempt_seeds.contains_key(&alias_key) {
            return Err(BrokerContextRegistryError::Conflict);
        }
        if let Some(existing_seed) = state.assistant_aliases.get(&alias_key) {
            return if existing_seed == &seed_key {
                Ok(())
            } else {
                Err(BrokerContextRegistryError::Conflict)
            };
        }
        if state.binding_count() >= MAX_ACTIVE_BROKER_CONTEXTS {
            return Err(BrokerContextRegistryError::Capacity);
        }
        state.assistant_aliases.insert(alias_key, seed_key);
        Ok(())
    }

    /// Replaces only an already-active exact native binding. This is the
    /// authority-version refresh path; it cannot create a new mapping.
    pub fn refresh(
        &self,
        context: BrokerActionContext,
        active_from_ms: u64,
        expires_at_ms: u64,
    ) -> Result<(), BrokerContextRegistryError> {
        validate_context_lifetime(&context, active_from_ms, expires_at_ms)?;
        let key = BrokerContextKey::from_context(&context);
        let mut state = self
            .state
            .lock()
            .map_err(|_| BrokerContextRegistryError::Unavailable)?;
        let current = state
            .attempt_seeds
            .get_mut(&key)
            .ok_or(BrokerContextRegistryError::Unmapped)?;
        if !same_attempt_seed(&current.context, &context) {
            return Err(BrokerContextRegistryError::InvalidContext);
        }
        *current = ActiveBrokerContext {
            context,
            active_from_ms,
            expires_at_ms,
        };
        Ok(())
    }

    pub fn retire(
        &self,
        context: &BrokerActionContext,
    ) -> Result<bool, BrokerContextRegistryError> {
        context
            .validate()
            .map_err(|_| BrokerContextRegistryError::InvalidContext)?;
        let key = BrokerContextKey::from_context(context);
        let mut state = self
            .state
            .lock()
            .map_err(|_| BrokerContextRegistryError::Unavailable)?;
        let Some(active) = state.attempt_seeds.get(&key) else {
            return Ok(false);
        };
        if !same_attempt_seed(&active.context, context) {
            return Err(BrokerContextRegistryError::InvalidContext);
        }
        state.attempt_seeds.remove(&key);
        state
            .assistant_aliases
            .retain(|_, attempt_seed| attempt_seed != &key);
        Ok(true)
    }

    pub fn active_count(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.attempt_seeds.len())
            .unwrap_or(0)
    }

    pub fn assistant_alias_count(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.assistant_aliases.len())
            .unwrap_or(0)
    }
}

impl fmt::Debug for ActiveBrokerContextResolver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActiveBrokerContextResolver")
            .field("active", &self.active_count())
            .field("assistant_aliases", &self.assistant_alias_count())
            .finish()
    }
}

impl context_resolver_sealed::Sealed for ActiveBrokerContextResolver {}

impl BrokerActionContextResolver for ActiveBrokerContextResolver {
    fn resolve_authenticated(
        &mut self,
        metadata: &AuthenticatedBrokerMetadata,
        now_ms: u64,
    ) -> Result<BrokerActionContext, BrokerContextResolutionError> {
        if now_ms == 0 {
            return Err(BrokerContextResolutionError::Stale);
        }
        let state = self
            .state
            .lock()
            .map_err(|_| BrokerContextResolutionError::Unavailable)?;
        let requested_key = BrokerContextKey::from_metadata(metadata);
        let seed_key = if state.attempt_seeds.contains_key(&requested_key) {
            &requested_key
        } else {
            state
                .assistant_aliases
                .get(&requested_key)
                .ok_or(BrokerContextResolutionError::Unmapped)?
        };
        let active = state
            .attempt_seeds
            .get(seed_key)
            .ok_or(BrokerContextResolutionError::Unmapped)?;
        if now_ms < active.active_from_ms || now_ms > active.expires_at_ms {
            return Err(BrokerContextResolutionError::Stale);
        }
        active
            .context
            .validate()
            .map_err(|_| BrokerContextResolutionError::InvalidContext)?;
        if requested_key.native_session_id != active.context.native_session_id
            || requested_key.process_generation != active.context.dispatch.process_generation
        {
            return Err(BrokerContextResolutionError::InvalidContext);
        }
        let mut resolved = active.context.clone();
        resolved.native_message_id = requested_key.native_message_id;
        resolved
            .validate()
            .map_err(|_| BrokerContextResolutionError::InvalidContext)?;
        Ok(resolved)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenCodeBrokerPumpConfig {
    pub receive_timeout: Duration,
    pub response_timeout: Duration,
    pub max_pending_approvals: usize,
    pub max_terminal_requests: usize,
}

impl Default for OpenCodeBrokerPumpConfig {
    fn default() -> Self {
        Self {
            receive_timeout: Duration::from_secs(2),
            response_timeout: Duration::from_secs(2),
            max_pending_approvals: 256,
            max_terminal_requests: 4_096,
        }
    }
}

impl OpenCodeBrokerPumpConfig {
    fn validate(self) -> Result<Self, OpenCodeBrokerPumpError> {
        if self.receive_timeout.is_zero()
            || self.receive_timeout > MAX_IO_TIMEOUT
            || self.response_timeout.is_zero()
            || self.response_timeout > MAX_IO_TIMEOUT
            || self.max_pending_approvals == 0
            || self.max_terminal_requests < self.max_pending_approvals
        {
            return Err(OpenCodeBrokerPumpError::InvalidConfiguration);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpenCodeBrokerPumpOutcome {
    PendingApproval {
        correlation_id: String,
        prompt_id: String,
    },
    Responded {
        correlation_id: String,
        decision: BrokerDecision,
    },
    ObservedCancellation {
        correlation_id: String,
        decision: BrokerDecision,
    },
    EffectRunning {
        correlation_id: String,
    },
    SettledAfterCancellation {
        correlation_id: String,
        decision: BrokerDecision,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingPumpApproval {
    prompt_id: String,
    metadata: AuthenticatedBrokerMetadata,
}

/// Single owner of the authenticated broker endpoint and its approval state.
pub struct OpenCodeBrokerPump {
    broker: OpenCodeSdkBroker,
    worker: BrokerActionWorker<InstalledBrokerFacilityRegistry>,
    executor: InstalledBrokerFacilityRegistry,
    pending: BTreeMap<String, PendingPumpApproval>,
    completed: BTreeSet<String>,
    config: OpenCodeBrokerPumpConfig,
    sealed: bool,
}

impl OpenCodeBrokerPump {
    /// Attaches a broker endpoint that came from the SDK inherited-descriptor
    /// pair. This constructor is useful for a core supervisor and deterministic
    /// transport tests; it does not accept decoded broker events.
    pub fn attach_authenticated(
        broker: OpenCodeSdkBroker,
        facilities: InstalledBrokerFacilityRegistry,
        config: OpenCodeBrokerPumpConfig,
    ) -> Result<Self, OpenCodeBrokerPumpError> {
        let config = config.validate()?;
        facilities.seal()?;
        let worker = BrokerActionWorker::with_capacity(
            facilities.clone(),
            config.max_pending_approvals,
            config.max_terminal_requests,
        )?;
        Ok(Self {
            broker,
            worker,
            executor: facilities,
            pending: BTreeMap::new(),
            completed: BTreeSet::new(),
            config,
            sealed: false,
        })
    }

    /// Moves the only parent-side descriptor out of a successfully started
    /// native driver. The driver keeps process lifecycle ownership; this pump
    /// becomes the sole broker reader/writer.
    pub fn take_from_started_driver(
        driver: &mut OpenCodeNativeCommandDriver,
        facilities: InstalledBrokerFacilityRegistry,
        config: OpenCodeBrokerPumpConfig,
    ) -> Result<Self, OpenCodeBrokerPumpError> {
        let config = config.validate()?;
        facilities.seal()?;
        let worker = BrokerActionWorker::with_capacity(
            facilities.clone(),
            config.max_pending_approvals,
            config.max_terminal_requests,
        )?;
        let broker = driver.take_broker_for_pump()?;
        Ok(Self {
            broker,
            worker,
            executor: facilities,
            pending: BTreeMap::new(),
            completed: BTreeSet::new(),
            config,
            sealed: false,
        })
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
        self.pending
            .iter()
            .map(|(correlation_id, pending)| (correlation_id.clone(), pending.prompt_id.clone()))
            .collect()
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn drain_deferred_unknown<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        now_ms: u64,
    ) -> Result<usize, OpenCodeBrokerPumpError> {
        self.worker
            .drain_deferred_unknown(application, &mut self.executor, now_ms)
            .map_err(|error| {
                self.sealed = true;
                OpenCodeBrokerPumpError::Worker(error)
            })
    }

    /// Receives and processes exactly one authenticated descriptor frame.
    pub fn pump_one<A, R>(
        &mut self,
        application: &mut A,
        resolver: &mut R,
        now_ms: u64,
    ) -> Result<OpenCodeBrokerPumpOutcome, OpenCodeBrokerPumpError>
    where
        A: BrokerActionApplication,
        R: BrokerActionContextResolver,
    {
        self.require_open()?;
        if self.completed.len() >= self.config.max_terminal_requests {
            self.sealed = true;
            return Err(OpenCodeBrokerPumpError::Capacity);
        }
        let event = match AuthenticatedBrokerEvent::receive(
            &mut self.broker,
            self.config.receive_timeout,
        ) {
            Ok(event) => event,
            Err(error) if transient_channel_error(&error) => {
                if let Some(outcome) = self
                    .worker
                    .poll_deferred(application, &mut self.executor, now_ms)
                    .map_err(|worker_error| {
                        self.sealed = true;
                        OpenCodeBrokerPumpError::Worker(worker_error)
                    })?
                {
                    return self.settle(outcome, None);
                }
                return Err(OpenCodeBrokerPumpError::Channel(error));
            }
            Err(error) => {
                self.sealed = true;
                return Err(OpenCodeBrokerPumpError::Channel(error));
            }
        };
        let metadata = event.metadata();
        if self.completed.contains(metadata.correlation_id()) {
            self.sealed = true;
            return Err(OpenCodeBrokerPumpError::AlreadySettled);
        }
        let trusted_context = match resolver.resolve_authenticated(&metadata, now_ms) {
            Ok(context) => context,
            Err(_error) if metadata.kind() == AuthenticatedBrokerEventKind::Cancellation => {
                let outcome = self
                    .worker
                    .accept_authenticated_cancellation(
                        application,
                        &mut self.executor,
                        event,
                        now_ms,
                    )
                    .map_err(|worker_error| {
                        self.sealed = true;
                        OpenCodeBrokerPumpError::Worker(worker_error)
                    })?;
                return self.settle(outcome, Some(metadata));
            }
            Err(error) => return self.deny_unresolved_context(&metadata, error),
        };
        let outcome = self
            .worker
            .accept_authenticated_event(
                application,
                &mut self.executor,
                event,
                trusted_context,
                now_ms,
            )
            .map_err(|error| {
                self.sealed = true;
                OpenCodeBrokerPumpError::Worker(error)
            })?;
        self.settle(outcome, Some(metadata))
    }

    /// Resumes the exact pending prompt/correlation pair. No response is
    /// retried after an uncertain descriptor write.
    pub fn answer_approval<A, R>(
        &mut self,
        application: &mut A,
        resolver: &mut R,
        correlation_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<OpenCodeBrokerPumpOutcome, OpenCodeBrokerPumpError>
    where
        A: BrokerActionApplication,
        R: BrokerActionContextResolver,
    {
        self.require_open()?;
        if self.completed.contains(correlation_id) {
            return Err(OpenCodeBrokerPumpError::AlreadySettled);
        }
        let pending = self
            .pending
            .get(correlation_id)
            .cloned()
            .ok_or(OpenCodeBrokerPumpError::ApprovalBindingMismatch)?;
        if pending.prompt_id != prompt_id {
            return Err(OpenCodeBrokerPumpError::ApprovalBindingMismatch);
        }
        let current_context = match resolver.resolve_authenticated(&pending.metadata, now_ms) {
            Ok(context) => context,
            Err(_) => {
                let outcome = self
                    .worker
                    .invalidate_pending_approval(application, correlation_id, prompt_id, now_ms)
                    .map_err(|error| {
                        self.sealed = true;
                        OpenCodeBrokerPumpError::Worker(error)
                    })?;
                return self.settle(outcome, None);
            }
        };
        let outcome = self
            .worker
            .answer_approval(
                application,
                &mut self.executor,
                BrokerApprovalAnswer {
                    correlation_id,
                    prompt_id,
                    answer,
                    current_context: &current_context,
                    now_ms,
                },
            )
            .map_err(|error| {
                self.sealed = true;
                OpenCodeBrokerPumpError::Worker(error)
            })?;
        self.settle(outcome, None)
    }

    fn settle(
        &mut self,
        outcome: BrokerWorkerOutcome,
        authenticated_metadata: Option<AuthenticatedBrokerMetadata>,
    ) -> Result<OpenCodeBrokerPumpOutcome, OpenCodeBrokerPumpError> {
        match outcome {
            BrokerWorkerOutcome::PendingApproval {
                correlation_id,
                prompt_id,
            } => {
                if self.pending.len() >= self.config.max_pending_approvals
                    || self.pending.contains_key(&correlation_id)
                    || self.completed.contains(&correlation_id)
                {
                    self.sealed = true;
                    return Err(OpenCodeBrokerPumpError::Capacity);
                }
                let metadata = authenticated_metadata.ok_or_else(|| {
                    self.sealed = true;
                    OpenCodeBrokerPumpError::ApprovalBindingMismatch
                })?;
                if metadata.correlation_id() != correlation_id {
                    self.sealed = true;
                    return Err(OpenCodeBrokerPumpError::ApprovalBindingMismatch);
                }
                self.pending.insert(
                    correlation_id.clone(),
                    PendingPumpApproval {
                        prompt_id: prompt_id.clone(),
                        metadata,
                    },
                );
                Ok(OpenCodeBrokerPumpOutcome::PendingApproval {
                    correlation_id,
                    prompt_id,
                })
            }
            BrokerWorkerOutcome::Respond {
                correlation_id,
                decision,
            } => {
                if self.completed.contains(&correlation_id) {
                    self.sealed = true;
                    return Err(OpenCodeBrokerPumpError::AlreadySettled);
                }
                // A write may have been partial. Mark the request terminal and
                // seal on error; never retry an uncertain response.
                self.completed.insert(correlation_id.clone());
                self.pending.remove(&correlation_id);
                if let Err(error) = self.broker.respond(
                    &correlation_id,
                    decision.clone(),
                    self.config.response_timeout,
                ) {
                    self.sealed = true;
                    return Err(OpenCodeBrokerPumpError::ResponseStatusUnknown(error));
                }
                Ok(OpenCodeBrokerPumpOutcome::Responded {
                    correlation_id,
                    decision,
                })
            }
            BrokerWorkerOutcome::EffectRunning { correlation_id } => {
                self.pending.remove(&correlation_id);
                Ok(OpenCodeBrokerPumpOutcome::EffectRunning { correlation_id })
            }
            BrokerWorkerOutcome::SettledAfterCancellation {
                correlation_id,
                decision,
            } => Ok(OpenCodeBrokerPumpOutcome::SettledAfterCancellation {
                correlation_id,
                decision,
            }),
            BrokerWorkerOutcome::ObservedCancellation {
                correlation_id,
                decision,
            } => {
                // `OpenCodeSdkBroker::receive` already emitted the cancellation
                // result. Recording it here prevents approval replay without a
                // second descriptor write.
                self.completed.insert(correlation_id.clone());
                self.pending.remove(&correlation_id);
                Ok(OpenCodeBrokerPumpOutcome::ObservedCancellation {
                    correlation_id,
                    decision,
                })
            }
        }
    }

    fn deny_unresolved_context(
        &mut self,
        metadata: &AuthenticatedBrokerMetadata,
        error: BrokerContextResolutionError,
    ) -> Result<OpenCodeBrokerPumpOutcome, OpenCodeBrokerPumpError> {
        let correlation_id = metadata.correlation_id().to_owned();
        let decision = BrokerDecision::Denied {
            reason_code: error.reason_code().to_owned(),
        };
        self.completed.insert(correlation_id.clone());
        if let Err(channel_error) = self.broker.respond(
            &correlation_id,
            decision.clone(),
            self.config.response_timeout,
        ) {
            self.sealed = true;
            return Err(OpenCodeBrokerPumpError::ResponseStatusUnknown(
                channel_error,
            ));
        }
        Ok(OpenCodeBrokerPumpOutcome::Responded {
            correlation_id,
            decision,
        })
    }

    fn require_open(&self) -> Result<(), OpenCodeBrokerPumpError> {
        if self.sealed {
            Err(OpenCodeBrokerPumpError::Sealed)
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for OpenCodeBrokerPump {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenCodeBrokerPump")
            .field("pending", &self.pending.len())
            .field("completed", &self.completed.len())
            .field("sealed", &self.sealed)
            .finish()
    }
}

fn resolved(
    classification: &InstalledBrokerClassification,
    target_version: String,
) -> ResolvedBrokerAction {
    ResolvedBrokerAction {
        surface: classification.surface.clone(),
        effects: classification.effects.clone(),
        scope: classification.scope,
        sensitivity: classification.sensitivity,
        reversibility: classification.reversibility,
        repository_state: classification.repository_state,
        inside_active_project: classification.inside_active_project,
        canonical_target: classification.canonical_target.clone(),
        target_version,
        normalized_arguments: classification.normalized_arguments.clone(),
        trusted_root: classification.trusted_root,
        explicit_scope_grant: classification.explicit_scope_grant,
        sandbox_allows: classification.sandbox_allows,
        declaration_exceeded: classification.declaration_exceeded,
        confidence: crate::security::policy::ClassificationConfidence::Known,
        risk: canonical_risk_for_installed(classification),
        plugin_or_mcp_id: None,
    }
}

fn canonical_risk_for_installed(
    classification: &InstalledBrokerClassification,
) -> crate::security::authorization::CanonicalRisk {
    use crate::security::authorization::CanonicalRisk;
    if classification.sensitivity == ActionSensitivity::Credential
        || classification.reversibility == ActionReversibility::Destructive
        || classification.effects.iter().any(|effect| {
            matches!(
                effect,
                ActionEffect::Execute
                    | ActionEffect::Publish
                    | ActionEffect::Reveal
                    | ActionEffect::Listen
            )
        })
    {
        CanonicalRisk::High
    } else if classification
        .effects
        .iter()
        .any(|effect| effect.is_mutating())
    {
        CanonicalRisk::Medium
    } else {
        CanonicalRisk::Low
    }
}

fn route_from_permit(permit: &ExecutionPermit) -> Option<(FacilityKey, Map<String, Value>)> {
    let action = permit.action();
    let broker = action.arguments.get("broker")?.as_object()?;
    match action.tool.as_str() {
        C4OS_RESOURCE_READ_TOOL => {
            let resource = broker.get("resource")?.as_str()?.to_owned();
            let selector = match broker.get("selector") {
                None | Some(Value::Null) => None,
                Some(Value::String(value)) => Some(value.clone()),
                _ => return None,
            };
            Some((FacilityKey::Resource { resource, selector }, Map::new()))
        }
        C4OS_ACTION_PROPOSAL_TOOL => {
            let operation = broker.get("operation")?.as_str()?.to_owned();
            let target = broker.get("target")?.as_str()?.to_owned();
            let arguments = broker.get("arguments")?.as_object()?.clone();
            Some((FacilityKey::Action { operation, target }, arguments))
        }
        _ => None,
    }
}

fn validate_classification(
    classification: &InstalledBrokerClassification,
    tool: &str,
) -> Result<(), FacilityRegistryError> {
    validate_route_text(&classification.canonical_target)?;
    validate_arguments(&classification.normalized_arguments)?;
    if classification.effects.is_empty()
        || matches!(classification.surface, ActionSurface::Unknown(_))
        || classification.effects.contains(&ActionEffect::Unknown)
        || classification.scope == ActionScope::Unknown
        || classification.sensitivity == ActionSensitivity::Unknown
        || classification.reversibility == ActionReversibility::Unknown
        || classification.repository_state == RepositoryState::Unknown
    {
        return Err(FacilityRegistryError::InvalidDefinition);
    }
    let effect_shape_matches = match tool {
        C4OS_RESOURCE_READ_TOOL => {
            classification
                .effects
                .iter()
                .all(|effect| effect.is_read_only())
                && classification.sensitivity != ActionSensitivity::Credential
                && classification.surface != ActionSurface::Credential
        }
        C4OS_ACTION_PROPOSAL_TOOL => classification
            .effects
            .iter()
            .any(|effect| !effect.is_read_only()),
        _ => false,
    };
    if !effect_shape_matches {
        return Err(FacilityRegistryError::InvalidDefinition);
    }
    Ok(())
}

fn validate_route_text(value: &str) -> Result<(), FacilityRegistryError> {
    if value.trim().is_empty()
        || value.len() > MAX_ROUTE_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
        Err(FacilityRegistryError::InvalidDefinition)
    } else {
        Ok(())
    }
}

fn validate_arguments(arguments: &Map<String, Value>) -> Result<(), FacilityRegistryError> {
    if serde_json::to_vec(arguments).map_or(true, |encoded| encoded.len() > MAX_ARGUMENT_BYTES) {
        Err(FacilityRegistryError::InvalidDefinition)
    } else {
        Ok(())
    }
}

fn valid_version(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= MAX_ROUTE_TEXT_BYTES
        && !value.chars().any(char::is_control)
}

fn nonzero_time(value: u64) -> u64 {
    if value == 0 { 1 } else { value }
}

fn transient_channel_error(error: &OpenCodeSdkError) -> bool {
    matches!(
        error,
        OpenCodeSdkError::Io(source)
            if matches!(
                source.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
            )
    )
}

fn validate_context_lifetime(
    context: &BrokerActionContext,
    active_from_ms: u64,
    expires_at_ms: u64,
) -> Result<(), BrokerContextRegistryError> {
    context
        .validate()
        .map_err(|_| BrokerContextRegistryError::InvalidContext)?;
    if active_from_ms == 0 || expires_at_ms < active_from_ms {
        return Err(BrokerContextRegistryError::InvalidLifetime);
    }
    Ok(())
}

fn validate_assistant_evidence(
    evidence: &AuthenticatedAssistantMessageEvidence,
) -> Result<(), BrokerContextRegistryError> {
    if evidence.role != "assistant"
        || !safe_broker_identifier(&evidence.native_session_id)
        || !safe_broker_identifier(&evidence.native_message_id)
        || evidence
            .parent_native_message_id
            .as_deref()
            .is_none_or(|parent| !safe_broker_identifier(parent))
        || evidence.process_generation == 0
    {
        return Err(BrokerContextRegistryError::InvalidEvidence);
    }
    Ok(())
}

fn safe_broker_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

fn same_attempt_seed(left: &BrokerActionContext, right: &BrokerActionContext) -> bool {
    left.dispatch == right.dispatch
        && left.native_session_id == right.native_session_id
        && left.native_message_id == right.native_message_id
        && left.request_origin == right.request_origin
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum BrokerContextResolutionError {
    #[error("authenticated broker metadata is not mapped to an active dispatch")]
    Unmapped,
    #[error("authenticated broker context is stale")]
    Stale,
    #[error("authenticated broker context is invalid")]
    InvalidContext,
    #[error("authenticated broker context resolver is unavailable")]
    Unavailable,
}

impl BrokerContextResolutionError {
    fn reason_code(self) -> &'static str {
        match self {
            Self::Unmapped => "unmapped-broker-context",
            Self::Stale => "stale-runtime-identity",
            Self::InvalidContext => "invalid-trusted-context",
            Self::Unavailable => "broker-context-resolver-unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum BrokerContextRegistryError {
    #[error("broker context is invalid")]
    InvalidContext,
    #[error("broker context lifetime is invalid")]
    InvalidLifetime,
    #[error("authenticated assistant-message evidence is invalid")]
    InvalidEvidence,
    #[error("authenticated assistant-message evidence does not bind to the attempt seed")]
    EvidenceBindingMismatch,
    #[error("broker context binding is stale")]
    Stale,
    #[error("broker context registry is at capacity")]
    Capacity,
    #[error("broker context binding conflicts with an active binding")]
    Conflict,
    #[error("broker context binding is not mapped")]
    Unmapped,
    #[error("broker context registry is unavailable")]
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum FacilityRegistryError {
    #[error("broker facility definition is invalid")]
    InvalidDefinition,
    #[error("broker facility registry is at capacity")]
    Capacity,
    #[error("broker facility route conflicts with an installed route")]
    Conflict,
    #[error("broker facility registry is sealed")]
    Sealed,
    #[error("broker facility registry is unavailable")]
    Unavailable,
}

#[derive(Debug, Error)]
pub enum OpenCodeBrokerPumpError {
    #[error("OpenCode broker pump configuration is invalid")]
    InvalidConfiguration,
    #[error("OpenCode broker pump is sealed")]
    Sealed,
    #[error("OpenCode broker pump capacity was exhausted")]
    Capacity,
    #[error("OpenCode broker approval binding did not match")]
    ApprovalBindingMismatch,
    #[error("OpenCode broker request was already settled")]
    AlreadySettled,
    #[error("OpenCode native driver broker is unavailable")]
    Native(#[from] NativeBrokerError),
    #[error("OpenCode broker channel failed")]
    Channel(#[source] OpenCodeSdkError),
    #[error("OpenCode broker response status is unknown")]
    ResponseStatusUnknown(#[source] OpenCodeSdkError),
    #[error("OpenCode broker worker failed")]
    Worker(#[from] BrokerWorkerError),
    #[error("OpenCode broker facility registry failed")]
    Registry(#[from] FacilityRegistryError),
}

impl OpenCodeBrokerPumpError {
    /// Once a broker result write becomes uncertain, this native generation
    /// cannot safely accept another request or approval continuation. The host
    /// must retire the generation rather than treating the descriptor failure
    /// as a transient empty poll.
    pub fn is_generation_fatal(&self) -> bool {
        matches!(self, Self::ResponseStatusUnknown(_))
            || matches!(self, Self::Channel(error) if !transient_channel_error(error))
    }
}
