//! Rust-worker-owned authority for minting durable dispatch snapshots.
//!
//! Renderer intent never carries configuration, resource, capability, or
//! process snapshot material. Workers install exact bounded truth using CAS;
//! submission coordination can then mint the session records from that truth.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Mutex;

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::runtime::capability::{CapabilityDescriptor, CapabilityLayer, RouteIdentity};
use crate::runtime::session::{
    AdapterBinding, AttemptContextSnapshot, CapabilitySnapshot, ConfigurationSnapshot,
    ExecutionEnvironmentBinding, ModelRouteSnapshot, ResourceSnapshot, RuntimeKind, SessionBinding,
    canonical_capability_descriptor, capability_snapshot_sha256, capability_states_from_descriptor,
};

const MAX_AUTHORITY_ROUTES: usize = 512;
const MAX_CONFIGURATION_FIELDS: usize = 256;
const MAX_CONFIGURATION_TEXT_BYTES: usize = 4 * 1024;
const MAX_CONFIGURATION_LIST_ITEMS: usize = 128;
const MAX_RESOURCE_RECORDS: usize = 512;
const MAX_RESOURCE_KIND_BYTES: usize = 160;
const MAX_IDENTIFIER_BYTES: usize = 160;
const MAX_CONFIGURATION_CANONICAL_BYTES: usize = 1024 * 1024;
const MAX_RESOURCE_CANONICAL_BYTES: usize = 1024 * 1024;
const MAX_CANONICAL_SNAPSHOT_BYTES: usize = 8 * 1024 * 1024;

/// A bounded configuration value accepted only from an authoritative worker.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum ConfigurationFieldValue {
    Text(String),
    Boolean(bool),
    Integer(i64),
    TextList(Vec<String>),
    CredentialReference(String),
}

/// Exact configuration truth installed by a Rust worker.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoritativeConfiguration {
    pub version: u64,
    pub fields: BTreeMap<String, ConfigurationFieldValue>,
}

/// One installed non-secret resource record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoritativeResource {
    pub kind: String,
    pub version: u64,
    pub sha256: String,
}

/// Exact current resource-set truth installed by a Rust worker.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoritativeResources {
    pub version: u64,
    pub records: BTreeMap<String, AuthoritativeResource>,
}

/// Complete route authority installed atomically by a Rust worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerDispatchAuthority {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub runtime_id: String,
    pub runtime_kind: RuntimeKind,
    pub adapter: AdapterBinding,
    pub environment: ExecutionEnvironmentBinding,
    pub route: RouteIdentity,
    pub configuration: AuthoritativeConfiguration,
    pub resources: AuthoritativeResources,
    pub process_generation: u64,
}

/// Current process truth obtained from the registered dispatch peer, never the renderer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeProcessTruth {
    pub runtime_id: String,
    pub runtime_kind: RuntimeKind,
    pub adapter_version: String,
    pub native_version: String,
    pub process_generation: u64,
}

/// Narrow lookup/CAS intent. It intentionally contains no snapshot material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityMintIntent {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub runtime_id: String,
    pub expected_authority_generation: u64,
    pub expected_capability_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintedDispatchAuthority {
    pub context: AttemptContextSnapshot,
    pub process_generation: u64,
    pub authority_generation: u64,
    pub capability_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintedFirstDispatchAuthority {
    pub binding: SessionBinding,
    pub process_generation: u64,
    pub authority_generation: u64,
    pub capability_generation: u64,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AuthorityScope {
    workspace_id: String,
    project_id: Option<String>,
    runtime_id: String,
    route: AuthorityRouteScope,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AuthorityRouteScope {
    provider_id: String,
    endpoint_id: String,
    provider_model_id: String,
    model_revision: String,
    adapter_kind: String,
    adapter_version: String,
    runtime_kind: String,
    native_runtime_version: String,
}

#[derive(Default)]
struct RegistryState {
    generation: u64,
    routes: BTreeMap<AuthorityScope, WorkerDispatchAuthority>,
}

/// Bounded compare-and-swap registry for Rust-authoritative dispatch truth.
#[derive(Default)]
pub struct DispatchAuthorityRegistry {
    state: Mutex<RegistryState>,
}

impl DispatchAuthorityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generation(&self) -> Result<u64, DispatchAuthorityError> {
        Ok(self
            .state
            .lock()
            .map_err(|_| DispatchAuthorityError::Unavailable)?
            .generation)
    }

    /// Removes every route owned by a Workspace during an authoritative
    /// unbind. The registry generation advances when any route is removed.
    pub fn invalidate_workspace(&self, workspace_id: &str) -> Result<u64, DispatchAuthorityError> {
        validate_id(workspace_id)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| DispatchAuthorityError::Unavailable)?;
        let previous_len = state.routes.len();
        state
            .routes
            .retain(|scope, _| scope.workspace_id != workspace_id);
        if state.routes.len() != previous_len {
            state.generation = state
                .generation
                .checked_add(1)
                .ok_or(DispatchAuthorityError::GenerationExhausted)?;
        }
        Ok(state.generation)
    }

    /// Installs or replaces one exact worker record when the global CAS token matches.
    pub fn install_from_worker(
        &self,
        expected_generation: u64,
        authority: WorkerDispatchAuthority,
    ) -> Result<u64, DispatchAuthorityError> {
        validate_authority(&authority)?;
        let scope = scope_for(&authority);
        let configuration_digest = configuration_sha256(&authority.configuration)?;
        let resources_digest = resources_sha256(&authority.resources)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| DispatchAuthorityError::Unavailable)?;
        if state.generation != expected_generation {
            return Err(DispatchAuthorityError::StaleAuthorityGeneration {
                expected: expected_generation,
                current: state.generation,
            });
        }
        if let Some(previous) = state.routes.get(&scope) {
            let prior_configuration_sha256 = configuration_sha256(&previous.configuration)?;
            if authority.configuration.version < previous.configuration.version
                || (authority.configuration.version == previous.configuration.version
                    && configuration_digest != prior_configuration_sha256)
            {
                return Err(DispatchAuthorityError::ConfigurationDrift);
            }
            let prior_resources_sha256 = resources_sha256(&previous.resources)?;
            if authority.resources.version < previous.resources.version
                || (authority.resources.version == previous.resources.version
                    && resources_digest != prior_resources_sha256)
            {
                return Err(DispatchAuthorityError::ResourceDrift);
            }
            if authority.process_generation < previous.process_generation {
                return Err(DispatchAuthorityError::ProcessDrift);
            }
            if previous == &authority {
                return Ok(state.generation);
            }
        } else if state.routes.len() >= MAX_AUTHORITY_ROUTES {
            return Err(DispatchAuthorityError::CapacityExceeded);
        }
        let next_generation = state
            .generation
            .checked_add(1)
            .ok_or(DispatchAuthorityError::GenerationExhausted)?;
        state.routes.insert(scope, authority);
        state.generation = next_generation;
        Ok(next_generation)
    }

    /// Atomically replaces every route published by one exact native process.
    /// Many provider/model authorities can coexist for the same runtime; no
    /// partial batch is visible if any route fails validation.
    pub fn replace_runtime_from_worker(
        &self,
        expected_generation: u64,
        workspace_id: &str,
        runtime_id: &str,
        process_generation: u64,
        authorities: Vec<WorkerDispatchAuthority>,
    ) -> Result<u64, DispatchAuthorityError> {
        validate_id(workspace_id)?;
        validate_id(runtime_id)?;
        if process_generation == 0 || authorities.len() > MAX_AUTHORITY_ROUTES {
            return Err(DispatchAuthorityError::InvalidAuthority);
        }
        let mut incoming = BTreeMap::new();
        for authority in authorities {
            validate_authority(&authority)?;
            if authority.workspace_id != workspace_id
                || authority.runtime_id != runtime_id
                || authority.process_generation != process_generation
            {
                return Err(DispatchAuthorityError::ProcessDrift);
            }
            let scope = scope_for(&authority);
            if incoming.insert(scope, authority).is_some() {
                return Err(DispatchAuthorityError::RouteDrift);
            }
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| DispatchAuthorityError::Unavailable)?;
        if state.generation != expected_generation {
            return Err(DispatchAuthorityError::StaleAuthorityGeneration {
                expected: expected_generation,
                current: state.generation,
            });
        }
        if state.routes.iter().any(|(scope, authority)| {
            scope.workspace_id == workspace_id
                && scope.runtime_id == runtime_id
                && authority.process_generation > process_generation
        }) {
            return Err(DispatchAuthorityError::ProcessDrift);
        }
        for (scope, authority) in &incoming {
            if let Some(previous) = state.routes.get(scope) {
                validate_monotonic_replacement(previous, authority)?;
            }
        }
        let mut candidate = state.routes.clone();
        candidate.retain(|scope, _| {
            scope.workspace_id != workspace_id || scope.runtime_id != runtime_id
        });
        for (scope, authority) in incoming {
            if candidate.insert(scope, authority).is_some() {
                return Err(DispatchAuthorityError::RouteDrift);
            }
        }
        if candidate.len() > MAX_AUTHORITY_ROUTES {
            return Err(DispatchAuthorityError::CapacityExceeded);
        }
        let next_generation = state
            .generation
            .checked_add(1)
            .ok_or(DispatchAuthorityError::GenerationExhausted)?;
        state.routes = candidate;
        state.generation = next_generation;
        Ok(next_generation)
    }

    pub fn invalidate_runtime_process(
        &self,
        expected_generation: u64,
        workspace_id: &str,
        runtime_id: &str,
        process_generation: u64,
    ) -> Result<u64, DispatchAuthorityError> {
        validate_id(workspace_id)?;
        validate_id(runtime_id)?;
        if process_generation == 0 {
            return Err(DispatchAuthorityError::InvalidAuthority);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| DispatchAuthorityError::Unavailable)?;
        if state.generation != expected_generation {
            return Err(DispatchAuthorityError::StaleAuthorityGeneration {
                expected: expected_generation,
                current: state.generation,
            });
        }
        let previous_len = state.routes.len();
        state.routes.retain(|scope, authority| {
            scope.workspace_id != workspace_id
                || scope.runtime_id != runtime_id
                || authority.process_generation != process_generation
        });
        if state.routes.len() == previous_len {
            return Ok(state.generation);
        }
        state.generation = state
            .generation
            .checked_add(1)
            .ok_or(DispatchAuthorityError::GenerationExhausted)?;
        Ok(state.generation)
    }

    pub fn mint_first_binding(
        &self,
        intent: &AuthorityMintIntent,
        effective_capabilities: &CapabilityDescriptor,
        current_capability_generation: u64,
        process: &RuntimeProcessTruth,
        bound_at_ms: u64,
    ) -> Result<MintedFirstDispatchAuthority, DispatchAuthorityError> {
        self.mint_first_binding_with_scope(
            intent,
            effective_capabilities,
            current_capability_generation,
            process,
            bound_at_ms,
            false,
        )
    }

    /// Derives an exact Project scope from a Workspace-wide process template.
    /// The application calls this only after the bound Workspace database has
    /// confirmed that the Project is active; ordinary registry callers retain
    /// exact-scope lookup through `mint_first_binding`.
    pub(crate) fn mint_first_binding_for_active_project(
        &self,
        intent: &AuthorityMintIntent,
        effective_capabilities: &CapabilityDescriptor,
        current_capability_generation: u64,
        process: &RuntimeProcessTruth,
        bound_at_ms: u64,
    ) -> Result<MintedFirstDispatchAuthority, DispatchAuthorityError> {
        self.mint_first_binding_with_scope(
            intent,
            effective_capabilities,
            current_capability_generation,
            process,
            bound_at_ms,
            true,
        )
    }

    fn mint_first_binding_with_scope(
        &self,
        intent: &AuthorityMintIntent,
        effective_capabilities: &CapabilityDescriptor,
        current_capability_generation: u64,
        process: &RuntimeProcessTruth,
        bound_at_ms: u64,
        allow_workspace_template: bool,
    ) -> Result<MintedFirstDispatchAuthority, DispatchAuthorityError> {
        if bound_at_ms == 0 {
            return Err(DispatchAuthorityError::InvalidAuthority);
        }
        let (authority, authority_generation) =
            self.authority_for(intent, effective_capabilities, allow_workspace_template)?;
        validate_current_truth(
            &authority,
            effective_capabilities,
            intent,
            current_capability_generation,
            process,
        )?;
        let context = mint_context(
            &authority,
            effective_capabilities,
            current_capability_generation,
        )?;
        let binding = SessionBinding {
            workspace_id: context.workspace_id,
            project_id: context.project_id,
            runtime_id: context.runtime_id,
            runtime_kind: context.runtime_kind,
            adapter: context.adapter,
            environment: context.environment,
            initial_model_route: context.model_route,
            initial_configuration: context.configuration,
            initial_resources: context.resources,
            initial_capabilities: context.capabilities,
            bound_at_ms,
        };
        binding
            .validate()
            .map_err(|_| DispatchAuthorityError::InvalidAuthority)?;
        Ok(MintedFirstDispatchAuthority {
            binding,
            process_generation: authority.process_generation,
            authority_generation,
            capability_generation: current_capability_generation,
        })
    }

    pub fn mint_retry_context(
        &self,
        intent: &AuthorityMintIntent,
        binding: &SessionBinding,
        effective_capabilities: &CapabilityDescriptor,
        current_capability_generation: u64,
        process: &RuntimeProcessTruth,
    ) -> Result<MintedDispatchAuthority, DispatchAuthorityError> {
        self.mint_retry_context_with_scope(
            intent,
            binding,
            effective_capabilities,
            current_capability_generation,
            process,
            false,
        )
    }

    pub(crate) fn mint_retry_context_for_active_project(
        &self,
        intent: &AuthorityMintIntent,
        binding: &SessionBinding,
        effective_capabilities: &CapabilityDescriptor,
        current_capability_generation: u64,
        process: &RuntimeProcessTruth,
    ) -> Result<MintedDispatchAuthority, DispatchAuthorityError> {
        self.mint_retry_context_with_scope(
            intent,
            binding,
            effective_capabilities,
            current_capability_generation,
            process,
            true,
        )
    }

    fn mint_retry_context_with_scope(
        &self,
        intent: &AuthorityMintIntent,
        binding: &SessionBinding,
        effective_capabilities: &CapabilityDescriptor,
        current_capability_generation: u64,
        process: &RuntimeProcessTruth,
        allow_workspace_template: bool,
    ) -> Result<MintedDispatchAuthority, DispatchAuthorityError> {
        let (authority, authority_generation) =
            self.authority_for(intent, effective_capabilities, allow_workspace_template)?;
        if binding.workspace_id != authority.workspace_id
            || binding.project_id != authority.project_id
        {
            return Err(DispatchAuthorityError::WorkspaceDrift);
        }
        if binding.runtime_id != authority.runtime_id
            || binding.runtime_kind != authority.runtime_kind
            || binding.adapter != authority.adapter
            || binding.environment != authority.environment
        {
            return Err(DispatchAuthorityError::RetryInvariantDrift);
        }
        validate_current_truth(
            &authority,
            effective_capabilities,
            intent,
            current_capability_generation,
            process,
        )?;
        let context = mint_context(
            &authority,
            effective_capabilities,
            current_capability_generation,
        )?;
        context
            .validate_against(binding)
            .map_err(|_| DispatchAuthorityError::RetryInvariantDrift)?;
        Ok(MintedDispatchAuthority {
            context,
            process_generation: authority.process_generation,
            authority_generation,
            capability_generation: current_capability_generation,
        })
    }

    fn authority_for(
        &self,
        intent: &AuthorityMintIntent,
        effective_capabilities: &CapabilityDescriptor,
        allow_workspace_template: bool,
    ) -> Result<(WorkerDispatchAuthority, u64), DispatchAuthorityError> {
        validate_id(&intent.workspace_id)?;
        validate_optional_id(intent.project_id.as_deref())?;
        validate_id(&intent.runtime_id)?;
        if intent.expected_authority_generation == 0 || intent.expected_capability_generation == 0 {
            return Err(DispatchAuthorityError::InvalidAuthority);
        }
        let state = self
            .state
            .lock()
            .map_err(|_| DispatchAuthorityError::Unavailable)?;
        if state.generation != intent.expected_authority_generation {
            return Err(DispatchAuthorityError::StaleAuthorityGeneration {
                expected: intent.expected_authority_generation,
                current: state.generation,
            });
        }
        let scope = AuthorityScope {
            workspace_id: intent.workspace_id.clone(),
            project_id: intent.project_id.clone(),
            runtime_id: intent.runtime_id.clone(),
            route: route_scope(&effective_capabilities.route),
        };
        if let Some(authority) = state.routes.get(&scope) {
            return Ok((authority.clone(), state.generation));
        }
        if allow_workspace_template && intent.project_id.is_some() {
            let workspace_scope = AuthorityScope {
                workspace_id: intent.workspace_id.clone(),
                project_id: None,
                runtime_id: intent.runtime_id.clone(),
                route: route_scope(&effective_capabilities.route),
            };
            if let Some(authority) = state.routes.get(&workspace_scope) {
                let mut authority = authority.clone();
                authority.project_id = intent.project_id.clone();
                return Ok((authority, state.generation));
            }
        }
        if state.routes.keys().any(|candidate| {
            candidate.runtime_id == intent.runtime_id
                && candidate.workspace_id == intent.workspace_id
                && (candidate.project_id == intent.project_id
                    || (allow_workspace_template && candidate.project_id.is_none()))
        }) {
            return Err(DispatchAuthorityError::RouteDrift);
        }
        if state.routes.keys().any(|candidate| {
            candidate.runtime_id == intent.runtime_id
                && (candidate.workspace_id != intent.workspace_id
                    || candidate.project_id != intent.project_id)
        }) {
            return Err(DispatchAuthorityError::WorkspaceDrift);
        }
        Err(DispatchAuthorityError::NotFound)
    }
}

fn validate_current_truth(
    authority: &WorkerDispatchAuthority,
    effective: &CapabilityDescriptor,
    intent: &AuthorityMintIntent,
    current_capability_generation: u64,
    process: &RuntimeProcessTruth,
) -> Result<(), DispatchAuthorityError> {
    effective
        .validate()
        .map_err(|_| DispatchAuthorityError::InvalidCapabilities)?;
    if effective.layer != CapabilityLayer::Effective {
        return Err(DispatchAuthorityError::InvalidCapabilities);
    }
    if current_capability_generation != intent.expected_capability_generation {
        return Err(DispatchAuthorityError::StaleCapabilityGeneration {
            expected: intent.expected_capability_generation,
            current: current_capability_generation,
        });
    }
    let configuration_sha256 = configuration_sha256(&authority.configuration)?;
    if effective.route.session_configuration_sha256 != configuration_sha256 {
        return Err(DispatchAuthorityError::ConfigurationDrift);
    }
    if effective.route != authority.route {
        return Err(DispatchAuthorityError::RouteDrift);
    }
    validate_process(process)?;
    if process.runtime_id != authority.runtime_id
        || process.runtime_kind != authority.runtime_kind
        || process.adapter_version != authority.adapter.adapter_version
        || process.native_version != authority.adapter.native_version
        || process.process_generation != authority.process_generation
    {
        return Err(DispatchAuthorityError::ProcessDrift);
    }
    Ok(())
}

fn mint_context(
    authority: &WorkerDispatchAuthority,
    effective: &CapabilityDescriptor,
    capability_generation: u64,
) -> Result<AttemptContextSnapshot, DispatchAuthorityError> {
    let canonical_effective = canonical_capability_descriptor(effective);
    let configuration_sha256 = configuration_sha256(&authority.configuration)?;
    let resources_sha256 = resources_sha256(&authority.resources)?;
    let capability_sha256 = capability_sha256(&canonical_effective, capability_generation)?;
    let route_sha256 = digest_serializable(&authority.route)?;
    let context = AttemptContextSnapshot {
        workspace_id: authority.workspace_id.clone(),
        project_id: authority.project_id.clone(),
        runtime_id: authority.runtime_id.clone(),
        runtime_kind: authority.runtime_kind,
        adapter: authority.adapter.clone(),
        environment: authority.environment.clone(),
        model_route: ModelRouteSnapshot {
            route_id: snapshot_id("route", &route_sha256),
            provider_id: authority.route.provider_id.clone(),
            endpoint_id: authority.route.endpoint_id.clone(),
            model_id: authority.route.provider_model_id.clone(),
            model_revision: authority.route.model_revision.clone(),
        },
        configuration: ConfigurationSnapshot {
            snapshot_id: snapshot_id("configuration", &configuration_sha256),
            version: authority.configuration.version,
            sha256: configuration_sha256,
        },
        resources: ResourceSnapshot {
            snapshot_id: snapshot_id("resources", &resources_sha256),
            version: authority.resources.version,
            sha256: resources_sha256,
            resource_ids: authority.resources.records.keys().cloned().collect(),
        },
        capabilities: CapabilitySnapshot {
            snapshot_id: snapshot_id("capabilities", &capability_sha256),
            version: capability_generation,
            sha256: capability_sha256,
            capabilities: capability_states_from_descriptor(&canonical_effective)
                .map_err(|_| DispatchAuthorityError::InvalidCapabilities)?,
            effective_descriptor: Some(canonical_effective),
        },
    };
    Ok(context)
}

fn validate_authority(authority: &WorkerDispatchAuthority) -> Result<(), DispatchAuthorityError> {
    validate_id(&authority.workspace_id)?;
    validate_optional_id(authority.project_id.as_deref())?;
    validate_id(&authority.runtime_id)?;
    authority
        .route
        .validate()
        .map_err(|_| DispatchAuthorityError::InvalidAuthority)?;
    validate_configuration(&authority.configuration)?;
    validate_resources(&authority.resources)?;
    let configuration_sha256 = configuration_sha256(&authority.configuration)?;
    if authority.route.session_configuration_sha256 != configuration_sha256
        || authority.route.adapter_version != authority.adapter.adapter_version
        || authority.route.native_runtime_version != authority.adapter.native_version
        || authority.route.runtime_kind != runtime_kind_id(authority.runtime_kind)
    {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    for (key, expected) in [
        ("provider-id", authority.route.provider_id.as_str()),
        ("endpoint-id", authority.route.endpoint_id.as_str()),
        ("model-id", authority.route.provider_model_id.as_str()),
        ("model-revision", authority.route.model_revision.as_str()),
        ("adapter-kind", authority.route.adapter_kind.as_str()),
        ("adapter-version", authority.route.adapter_version.as_str()),
        ("runtime-kind", authority.route.runtime_kind.as_str()),
        (
            "native-runtime-version",
            authority.route.native_runtime_version.as_str(),
        ),
    ] {
        if !matches!(
            authority.configuration.fields.get(key),
            Some(ConfigurationFieldValue::Text(value)) if value == expected
        ) {
            return Err(DispatchAuthorityError::InvalidAuthority);
        }
    }
    validate_process(&RuntimeProcessTruth {
        runtime_id: authority.runtime_id.clone(),
        runtime_kind: authority.runtime_kind,
        adapter_version: authority.adapter.adapter_version.clone(),
        native_version: authority.adapter.native_version.clone(),
        process_generation: authority.process_generation,
    })?;
    let context = mint_context(
        authority,
        &empty_effective_descriptor(authority.route.clone()),
        1,
    )?;
    let binding = SessionBinding {
        workspace_id: context.workspace_id,
        project_id: context.project_id,
        runtime_id: context.runtime_id,
        runtime_kind: context.runtime_kind,
        adapter: context.adapter,
        environment: context.environment,
        initial_model_route: context.model_route,
        initial_configuration: context.configuration,
        initial_resources: context.resources,
        initial_capabilities: context.capabilities,
        bound_at_ms: 1,
    };
    binding
        .validate()
        .map_err(|_| DispatchAuthorityError::InvalidAuthority)
}

fn empty_effective_descriptor(route: RouteIdentity) -> CapabilityDescriptor {
    CapabilityDescriptor {
        schema_version: crate::runtime::capability::CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Effective,
        route,
        lifecycle: crate::runtime::capability::ModelLifecycle::Active,
        features: BTreeMap::new(),
        numeric_limits: BTreeMap::new(),
        raw_evidence_sha256: format!("sha256:{}", "0".repeat(64)),
    }
}

fn validate_configuration(
    configuration: &AuthoritativeConfiguration,
) -> Result<(), DispatchAuthorityError> {
    if configuration.version == 0 || configuration.fields.len() > MAX_CONFIGURATION_FIELDS {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    let mut total_text_bytes = 0_usize;
    for (key, value) in &configuration.fields {
        validate_id(key)?;
        total_text_bytes = total_text_bytes
            .checked_add(key.len())
            .ok_or(DispatchAuthorityError::InvalidAuthority)?;
        match value {
            ConfigurationFieldValue::Text(value) => {
                if value.is_empty()
                    || value.len() > MAX_CONFIGURATION_TEXT_BYTES
                    || value.contains('\0')
                {
                    return Err(DispatchAuthorityError::InvalidAuthority);
                }
                total_text_bytes = total_text_bytes
                    .checked_add(value.len())
                    .ok_or(DispatchAuthorityError::InvalidAuthority)?;
            }
            ConfigurationFieldValue::CredentialReference(value) => {
                validate_id(value)?;
                total_text_bytes = total_text_bytes
                    .checked_add(value.len())
                    .ok_or(DispatchAuthorityError::InvalidAuthority)?;
            }
            ConfigurationFieldValue::TextList(values) => {
                if values.len() > MAX_CONFIGURATION_LIST_ITEMS
                    || values.iter().any(|value| {
                        value.is_empty()
                            || value.len() > MAX_CONFIGURATION_TEXT_BYTES
                            || value.contains('\0')
                    })
                {
                    return Err(DispatchAuthorityError::InvalidAuthority);
                }
                total_text_bytes = values.iter().try_fold(total_text_bytes, |total, value| {
                    total
                        .checked_add(value.len())
                        .ok_or(DispatchAuthorityError::InvalidAuthority)
                })?;
            }
            ConfigurationFieldValue::Boolean(_) | ConfigurationFieldValue::Integer(_) => {}
        }
    }
    if total_text_bytes > MAX_CONFIGURATION_CANONICAL_BYTES {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    Ok(())
}

fn validate_resources(resources: &AuthoritativeResources) -> Result<(), DispatchAuthorityError> {
    if resources.version == 0 || resources.records.len() > MAX_RESOURCE_RECORDS {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    for (resource_id, resource) in &resources.records {
        validate_id(resource_id)?;
        if resource.version == 0
            || resource.kind.is_empty()
            || resource.kind.len() > MAX_RESOURCE_KIND_BYTES
            || !valid_sha256(&resource.sha256)
        {
            return Err(DispatchAuthorityError::InvalidAuthority);
        }
        validate_id(&resource.kind)?;
    }
    Ok(())
}

fn validate_process(process: &RuntimeProcessTruth) -> Result<(), DispatchAuthorityError> {
    validate_id(&process.runtime_id)?;
    validate_id(&process.adapter_version)?;
    validate_id(&process.native_version)?;
    if process.process_generation == 0 {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    Ok(())
}

fn configuration_sha256(
    configuration: &AuthoritativeConfiguration,
) -> Result<String, DispatchAuthorityError> {
    digest_serializable_bounded(
        &("c4os.configuration.v1", configuration),
        MAX_CONFIGURATION_CANONICAL_BYTES,
    )
}

/// Canonical digest workers bind into [`RouteIdentity`] before CAS install.
pub fn authoritative_configuration_sha256(
    configuration: &AuthoritativeConfiguration,
) -> Result<String, DispatchAuthorityError> {
    validate_configuration(configuration)?;
    configuration_sha256(configuration)
}

/// Builds the exact non-secret configuration identity bound into a route.
/// The configuration digest itself is intentionally excluded, avoiding a
/// circular hash while keeping every other native route field immutable.
pub fn authoritative_route_configuration(
    route: &RouteIdentity,
) -> Result<AuthoritativeConfiguration, DispatchAuthorityError> {
    route
        .validate()
        .map_err(|_| DispatchAuthorityError::InvalidAuthority)?;
    Ok(AuthoritativeConfiguration {
        version: 1,
        fields: BTreeMap::from([
            (
                "provider-id".into(),
                ConfigurationFieldValue::Text(route.provider_id.clone()),
            ),
            (
                "endpoint-id".into(),
                ConfigurationFieldValue::Text(route.endpoint_id.clone()),
            ),
            (
                "model-id".into(),
                ConfigurationFieldValue::Text(route.provider_model_id.clone()),
            ),
            (
                "model-revision".into(),
                ConfigurationFieldValue::Text(route.model_revision.clone()),
            ),
            (
                "adapter-kind".into(),
                ConfigurationFieldValue::Text(route.adapter_kind.clone()),
            ),
            (
                "adapter-version".into(),
                ConfigurationFieldValue::Text(route.adapter_version.clone()),
            ),
            (
                "runtime-kind".into(),
                ConfigurationFieldValue::Text(route.runtime_kind.clone()),
            ),
            (
                "native-runtime-version".into(),
                ConfigurationFieldValue::Text(route.native_runtime_version.clone()),
            ),
        ]),
    })
}

fn resources_sha256(resources: &AuthoritativeResources) -> Result<String, DispatchAuthorityError> {
    digest_serializable_bounded(
        &("c4os.resources.v1", resources),
        MAX_RESOURCE_CANONICAL_BYTES,
    )
}

/// Canonical digest of one bounded worker-owned resource set.
pub fn authoritative_resources_sha256(
    resources: &AuthoritativeResources,
) -> Result<String, DispatchAuthorityError> {
    validate_resources(resources)?;
    resources_sha256(resources)
}

fn capability_sha256(
    descriptor: &CapabilityDescriptor,
    generation: u64,
) -> Result<String, DispatchAuthorityError> {
    capability_snapshot_sha256(descriptor, generation)
        .map_err(|_| DispatchAuthorityError::InvalidCapabilities)
}

fn runtime_kind_id(kind: RuntimeKind) -> &'static str {
    match kind {
        RuntimeKind::OpenCode => "opencode",
        RuntimeKind::Pi => "pi",
    }
}

fn scope_for(authority: &WorkerDispatchAuthority) -> AuthorityScope {
    AuthorityScope {
        workspace_id: authority.workspace_id.clone(),
        project_id: authority.project_id.clone(),
        runtime_id: authority.runtime_id.clone(),
        route: route_scope(&authority.route),
    }
}

fn route_scope(route: &RouteIdentity) -> AuthorityRouteScope {
    AuthorityRouteScope {
        provider_id: route.provider_id.clone(),
        endpoint_id: route.endpoint_id.clone(),
        provider_model_id: route.provider_model_id.clone(),
        model_revision: route.model_revision.clone(),
        adapter_kind: route.adapter_kind.clone(),
        adapter_version: route.adapter_version.clone(),
        runtime_kind: route.runtime_kind.clone(),
        native_runtime_version: route.native_runtime_version.clone(),
    }
}

fn validate_monotonic_replacement(
    previous: &WorkerDispatchAuthority,
    authority: &WorkerDispatchAuthority,
) -> Result<(), DispatchAuthorityError> {
    let configuration_digest = configuration_sha256(&authority.configuration)?;
    let prior_configuration_digest = configuration_sha256(&previous.configuration)?;
    if authority.configuration.version < previous.configuration.version
        || (authority.configuration.version == previous.configuration.version
            && configuration_digest != prior_configuration_digest)
    {
        return Err(DispatchAuthorityError::ConfigurationDrift);
    }
    let resources_digest = resources_sha256(&authority.resources)?;
    let prior_resources_digest = resources_sha256(&previous.resources)?;
    if authority.resources.version < previous.resources.version
        || (authority.resources.version == previous.resources.version
            && resources_digest != prior_resources_digest)
    {
        return Err(DispatchAuthorityError::ResourceDrift);
    }
    if authority.process_generation < previous.process_generation {
        return Err(DispatchAuthorityError::ProcessDrift);
    }
    Ok(())
}

fn snapshot_id(kind: &str, sha256: &str) -> String {
    format!(
        "{kind}:{}",
        sha256.strip_prefix("sha256:").unwrap_or(sha256)
    )
}

fn digest_serializable(value: &impl Serialize) -> Result<String, DispatchAuthorityError> {
    digest_serializable_bounded(value, MAX_CANONICAL_SNAPSHOT_BYTES)
}

fn digest_serializable_bounded(
    value: &impl Serialize,
    maximum_bytes: usize,
) -> Result<String, DispatchAuthorityError> {
    let bytes = serde_json::to_vec(value).map_err(|_| DispatchAuthorityError::Serialization)?;
    if bytes.len() > maximum_bytes {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    let digest = Sha256::digest(bytes);
    let mut encoded = String::from("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    Ok(encoded)
}

fn validate_optional_id(value: Option<&str>) -> Result<(), DispatchAuthorityError> {
    if let Some(value) = value {
        validate_id(value)?;
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), DispatchAuthorityError> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@' | b'/')
        })
    {
        return Err(DispatchAuthorityError::InvalidAuthority);
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DispatchAuthorityError {
    #[error("dispatch authority is invalid")]
    InvalidAuthority,
    #[error("dispatch authority registry is unavailable")]
    Unavailable,
    #[error("dispatch authority route was not found")]
    NotFound,
    #[error("dispatch authority registry reached its route bound")]
    CapacityExceeded,
    #[error("dispatch authority generation is stale: expected {expected}, current {current}")]
    StaleAuthorityGeneration { expected: u64, current: u64 },
    #[error("capability evidence generation is stale: expected {expected}, current {current}")]
    StaleCapabilityGeneration { expected: u64, current: u64 },
    #[error("dispatch authority generation was exhausted")]
    GenerationExhausted,
    #[error("dispatch authority workspace or project changed")]
    WorkspaceDrift,
    #[error("dispatch route changed outside the authority registry")]
    RouteDrift,
    #[error("dispatch configuration changed without a new authoritative version")]
    ConfigurationDrift,
    #[error("dispatch resource set changed without a new authoritative version")]
    ResourceDrift,
    #[error("dispatch process truth changed")]
    ProcessDrift,
    #[error("retry would migrate an immutable Chat binding")]
    RetryInvariantDrift,
    #[error("effective capability truth is invalid")]
    InvalidCapabilities,
    #[error("dispatch snapshot serialization failed")]
    Serialization,
}
