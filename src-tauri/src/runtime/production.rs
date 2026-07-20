//! Production composition for the two exact C4OS runtime peers.
//!
//! Construction verifies both packaged asset graphs and remains process-idle.
//! A native process is created only by a `prepare_*_peer` call carrying a
//! validated Workspace/runtime/process binding from the Rust-owned core.

#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::runtime::broker_worker::{
    BrokerActionApplication, BrokerActionContext, BrokerActionWorker, BrokerWorkerError,
    RuntimeBrokerApprovalAnswer, RuntimeBrokerWorkerOutcome,
};
use crate::runtime::capability::{
    CapabilityKey, ModelLifecycle, NumericCapabilityKey, RouteIdentity,
};
use crate::runtime::capability_evidence::{
    CapabilityRouteEpoch, FeatureClaim, NumericClaim, RuntimeObservationOutcome,
    RuntimeRouteObservation, opencode_adapter_evidence, opencode_observed_evidence,
    pi_adapter_evidence, pi_observed_evidence, provider_model_declared_evidence,
};
use crate::runtime::dispatch::{
    DispatchError, DispatchEvent, DispatchEventCategory, DispatchIdentity, OpenCodeDispatchPeer,
    PeerDispatchError, PiDispatchCredentialIssuer, PiDispatchPeer, RuntimeDispatchRegistry,
    RuntimePeerRegistration,
};
use crate::runtime::dispatch_authority::{
    DispatchAuthorityError, authoritative_configuration_sha256, authoritative_route_configuration,
};
use crate::runtime::opencode::{
    AdapterError as OpenCodeAdapterError, CommandDriver, CommandFailureCode, LaunchCommand,
    LoopbackEndpoint, NativeAuthorityPolicy, OpenCodeAdapter, OpenCodeCompatibilityManifest,
    OpenCodeLaunchPlan, ProcessHandle, RandomSecretReference, StateNamespace,
};
use crate::runtime::opencode_assets::{
    OpenCodeAssetError, OpenCodeProductionAssetFactory, ResolvedOpenCodeAssets,
};
use crate::runtime::opencode_broker::{
    ActiveBrokerContextResolver, BrokerContextRegistryError, FacilityRegistryError,
    InstalledBrokerClassification, InstalledBrokerFacility, InstalledBrokerFacilityRegistry,
    OpenCodeBrokerPump, OpenCodeBrokerPumpConfig, OpenCodeBrokerPumpError,
    OpenCodeBrokerPumpOutcome,
};
use crate::runtime::opencode_credential::{
    ProviderCredentialAuthorizationReceipt, ProviderCredentialRequest,
};
use crate::runtime::opencode_native::{
    LoopbackHttpTransport, NativeBoundaryError, OPENCODE_C4OS_TOOL_IDS,
    OpenCodeNativeCommandDriver, VaultCredentialResolver, opencode_authority_configuration_sha256,
};
use crate::runtime::opencode_stream::OpenCodeStreamBounds;
use crate::runtime::pi::{
    PI_NATIVE_VERSION, PiAdapter, PiAdapterError, PiHealth, PiModelRoute, PiSidecarManifest,
    PiSidecarRunner,
};
use crate::runtime::pi_process::{
    PiCredentialLeaseMetadata, PiProcessError, PiSidecarIntegrity, SpawnedPiRunner,
};
use crate::runtime::provider::{
    CurlProviderConnectivity, ModelRoute, PROVIDER_TEST_FRESHNESS_MS, ProviderDiscovery,
    ProviderError, ProviderKind, ProviderProbe, ProviderProbeFailure, ProviderProfile,
    ProviderRecord, ProviderTestStatus, VerifiedOpenCodeProviderProbe,
};
use crate::runtime::supervisor::{
    RUNTIME_PROTOCOL_VERSION, RuntimeInstallation, RuntimeKind, SupervisorError, sha256_file,
};
use crate::security::authorization::ApprovalAnswer;
use crate::security::credentials::{
    CredentialReference, CredentialVault, OperationCredentialLease, VaultProtection,
};
use crate::security::gateway::NormalizedActionStatus;
use crate::security::policy::ActionRequestOrigin;

const OPEN_CODE_SECRET_DESCRIPTOR: u32 = 198;
const OPEN_CODE_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const OPEN_CODE_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);
const OPEN_CODE_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const OPEN_CODE_BROKER_RECEIVE_TIMEOUT: Duration = Duration::from_millis(25);
const OPEN_CODE_IO_TIMEOUT: Duration = Duration::from_secs(5);
const PI_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(10);
const PI_CREDENTIAL_LEASE_TTL: Duration = Duration::from_secs(30);
const MAX_ID_BYTES: usize = 192;
const NODE_RUNTIME_METADATA_SCHEMA_VERSION: u16 = 1;
const NODE_MINIMUM_SUPPORTED_VERSION: &str = "22.19.0";
const MAX_NODE_RUNTIME_METADATA_BYTES: u64 = 16 * 1024;
const MAX_NODE_VERSION_BYTES: u64 = 128;
const NODE_VERSION_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_PENDING_PI_ACTION_INTENTS: usize = 4_096;
const MAX_TEST_TLS_TRUST_CAPABILITIES: usize = 16;
const MAX_TEST_TLS_CA_BYTES: usize = 128 * 1024;
const MAX_TEST_TLS_DESCRIPTOR_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestTlsTrustCapability {
    provider_id: String,
    opencode_native_provider_id: String,
    pi_native_provider_id: String,
    base_url: String,
    ca_pem: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionRuntimeBinding {
    runtime_id: String,
    runtime_kind: RuntimeKind,
    workspace_id: String,
    workspace_root: PathBuf,
    c4os_home: PathBuf,
    process_generation: u64,
    launch_id: String,
}

/// Crate-owned input assembled from the authoritative Workspace/runtime
/// records. This type and all of its fields are intentionally unavailable to
/// external callers.
pub struct CoreRuntimeBinding {
    pub(crate) runtime_id: String,
    pub(crate) runtime_kind: RuntimeKind,
    pub(crate) workspace_id: String,
    pub(crate) workspace_root: PathBuf,
    pub(crate) c4os_home: PathBuf,
    pub(crate) process_generation: u64,
    pub(crate) launch_id: String,
}

impl ProductionRuntimeBinding {
    pub fn from_core(input: CoreRuntimeBinding) -> Result<Self, RuntimeProductionError> {
        let CoreRuntimeBinding {
            runtime_id,
            runtime_kind,
            workspace_id,
            workspace_root,
            c4os_home,
            process_generation,
            launch_id,
        } = input;
        if !valid_id(&runtime_id)
            || !valid_id(&workspace_id)
            || !valid_id(&launch_id)
            || process_generation == 0
        {
            return Err(RuntimeProductionError::InvalidBinding);
        }
        let workspace_root = canonical_directory(&workspace_root)?;
        let c4os_home = canonical_directory(&c4os_home)?;
        Ok(Self {
            runtime_id,
            runtime_kind,
            workspace_id,
            workspace_root,
            c4os_home,
            process_generation,
            launch_id,
        })
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn runtime_kind(&self) -> RuntimeKind {
        self.runtime_kind
    }

    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn process_generation(&self) -> u64 {
        self.process_generation
    }

    pub fn launch_id(&self) -> &str {
        &self.launch_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeProductionReadiness {
    pub opencode_native_version: String,
    pub opencode_native_sha256: String,
    pub opencode_native_tree_sha256: String,
    pub pi_native_version: String,
    pub pi_dependency_tree_sha256: String,
    pub node_sha256: String,
    pub node_version: String,
    pub credential_vault_available: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NodeRuntimeMetadata {
    schema_version: u16,
    executable_relative_path: PathBuf,
    sha256: String,
    version: String,
    minimum_supported_version: String,
}

struct VerifiedNodeRuntime {
    executable: PathBuf,
    sha256: String,
    version: String,
}

const MAX_PRODUCTION_PROVIDER_MODELS: usize = 512;

/// Exact non-secret provider/model material needed to construct one native
/// route. The profile's opaque vault reference stays inside Rust; native
/// configuration receives only its profile-qualified identity, tested
/// endpoint, SDK kind, and discovered model metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductionProviderRoute {
    profile: ProviderProfile,
    selected_model_id: String,
    models: Vec<ModelRoute>,
    opencode_sdk_npm: Option<&'static str>,
}

impl ProductionProviderRoute {
    pub(crate) fn from_record(
        record: ProviderRecord,
        runtime_kind: RuntimeKind,
        checked_at_ms: u64,
    ) -> Result<Self, RuntimeProductionError> {
        let ProviderRecord {
            profile,
            test_status,
            connection_evidence,
            models,
            selected_model_id,
            ..
        } = record;
        profile.validate()?;
        let tested_at_ms = match test_status {
            ProviderTestStatus::Succeeded { checked_at_ms } => checked_at_ms,
            _ => return Err(RuntimeProductionError::ProviderRouteUnavailable),
        };
        let evidence =
            connection_evidence.ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
        evidence
            .validate_for(&profile, tested_at_ms)
            .map_err(|_| RuntimeProductionError::ProviderRouteUnavailable)?;
        let selected_model_id =
            selected_model_id.ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
        if checked_at_ms < tested_at_ms
            || checked_at_ms.saturating_sub(tested_at_ms) > PROVIDER_TEST_FRESHNESS_MS
        {
            return Err(RuntimeProductionError::ProviderRouteUnavailable);
        }
        let models = models
            .into_values()
            .filter(|model| {
                let runtime_compatible = match runtime_kind {
                    RuntimeKind::OpenCode => {
                        model.capabilities.route.runtime_kind == RuntimeKind::OpenCode.as_str()
                    }
                    RuntimeKind::Pi => model.provider_declaration.is_some(),
                };
                runtime_compatible
                    && model.validate_for(&profile).is_ok()
                    && model.is_production_ready_at(checked_at_ms)
                    && model.checked_at_ms <= checked_at_ms
                    && checked_at_ms.saturating_sub(model.checked_at_ms)
                        <= PROVIDER_TEST_FRESHNESS_MS
            })
            .collect::<Vec<_>>();
        if models.is_empty()
            || models.len() > MAX_PRODUCTION_PROVIDER_MODELS
            || !models.iter().any(|model| {
                model.model_id == selected_model_id
                    && model.is_production_ready_at(checked_at_ms)
                    && model.checked_at_ms <= checked_at_ms
                    && checked_at_ms.saturating_sub(model.checked_at_ms)
                        <= PROVIDER_TEST_FRESHNESS_MS
            })
        {
            return Err(RuntimeProductionError::ProviderRouteUnavailable);
        }
        let opencode_sdk_npm = match runtime_kind {
            RuntimeKind::OpenCode => Some(opencode_provider_npm(&profile)?),
            RuntimeKind::Pi => {
                profile
                    .pi_native_provider_id()
                    .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
                None
            }
        };
        Ok(Self {
            profile,
            selected_model_id,
            models,
            opencode_sdk_npm,
        })
    }

    pub(crate) fn profile(&self) -> &ProviderProfile {
        &self.profile
    }

    pub(crate) fn selected_model_id(&self) -> &str {
        &self.selected_model_id
    }

    pub(crate) fn models(&self) -> &[ModelRoute] {
        &self.models
    }

    pub(crate) fn opencode_sdk_npm(&self) -> Result<&'static str, RuntimeProductionError> {
        self.opencode_sdk_npm
            .ok_or(RuntimeProductionError::ProviderRouteUnavailable)
    }
}

fn exact_route(
    provider: &ProductionProviderRoute,
    model: &ModelRoute,
    runtime_kind: RuntimeKind,
) -> Result<RouteIdentity, RuntimeProductionError> {
    let native_provider_id = match runtime_kind {
        RuntimeKind::OpenCode => provider.profile().opencode_native_provider_id(),
        RuntimeKind::Pi => provider
            .profile()
            .pi_native_provider_id()
            .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?,
    };
    let mut route = RouteIdentity {
        provider_id: provider.profile().provider_id.clone(),
        endpoint_id: provider.profile().endpoint.endpoint_id.clone(),
        provider_model_id: format!("{native_provider_id}/{}", model.model_id),
        model_revision: model.capabilities.route.model_revision.clone(),
        adapter_kind: runtime_kind.as_str().into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: runtime_kind.as_str().into(),
        native_runtime_version: match runtime_kind {
            RuntimeKind::OpenCode => crate::runtime::opencode::OPENCODE_NATIVE_VERSION,
            RuntimeKind::Pi => PI_NATIVE_VERSION,
        }
        .into(),
        session_configuration_sha256: format!("sha256:{}", "0".repeat(64)),
    };
    route.session_configuration_sha256 =
        authoritative_configuration_sha256(&authoritative_route_configuration(&route)?)?;
    route
        .validate()
        .map_err(|_| RuntimeProductionError::ProviderRouteUnavailable)?;
    Ok(route)
}

fn observation_from_adapter(
    runtime_id: &str,
    process_generation: u64,
    health_checked_at_ms: u64,
    observed_at_ms: u64,
    expires_at_ms: u64,
    adapter: &crate::runtime::capability_evidence::AdapterNormalizedEvidence,
) -> RuntimeRouteObservation {
    let descriptor = adapter.descriptor();
    RuntimeRouteObservation {
        runtime_id: runtime_id.into(),
        route: descriptor.route.clone(),
        process_generation,
        health_checked_at_ms,
        observed_at_ms,
        expires_at_ms,
        outcome: if descriptor.lifecycle == ModelLifecycle::Unavailable {
            RuntimeObservationOutcome::Failed
        } else {
            RuntimeObservationOutcome::Available
        },
        lifecycle: descriptor.lifecycle,
        features: descriptor
            .features
            .iter()
            .map(|(key, evidence)| {
                (
                    *key,
                    FeatureClaim {
                        state: evidence.state,
                        constraints: evidence.constraints.clone(),
                        allowed_values: evidence.allowed_values.clone(),
                        reason: evidence.reason.clone(),
                    },
                )
            })
            .collect(),
        numeric_limits: descriptor
            .numeric_limits
            .iter()
            .map(|(key, numeric)| {
                (
                    *key,
                    NumericClaim {
                        state: numeric.evidence.state,
                        maximum: numeric.maximum,
                        confidence: numeric.confidence,
                        reason: numeric.evidence.reason.clone(),
                    },
                )
            })
            .collect(),
        raw_observation_sha256: descriptor.raw_evidence_sha256.clone(),
    }
}

fn opencode_capability_epochs(
    binding: &ProductionRuntimeBinding,
    routes: &[ProductionProviderRoute],
    conformance: &crate::runtime::adapter::AdapterConformanceDescriptor,
    health: &crate::runtime::opencode::HealthSnapshot,
    checked_at_ms: u64,
) -> Result<Vec<CapabilityRouteEpoch>, RuntimeProductionError> {
    let mut epochs = Vec::new();
    for provider in routes {
        for model in provider.models() {
            let route = exact_route(provider, model, RuntimeKind::OpenCode)?;
            let declaration = model
                .provider_declaration
                .as_ref()
                .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
            let declared = provider_model_declared_evidence(declaration, route.clone())?;
            let mut projected_model = model.clone();
            projected_model.capabilities.route = route;
            let adapter = opencode_adapter_evidence(&projected_model)?;
            let expires_at_ms = declaration.expires_at_ms;
            if expires_at_ms <= checked_at_ms {
                return Err(RuntimeProductionError::ProviderRouteUnavailable);
            }
            let observation = observation_from_adapter(
                binding.runtime_id(),
                binding.process_generation(),
                health.checked_at_ms,
                checked_at_ms,
                expires_at_ms,
                &adapter,
            );
            let observed = opencode_observed_evidence(conformance, health, &observation)?;
            epochs.push(CapabilityRouteEpoch::new(
                binding.runtime_id(),
                binding.process_generation(),
                declared,
                adapter,
                observed,
            )?);
        }
    }
    Ok(epochs)
}

fn pi_capability_epochs(
    binding: &ProductionRuntimeBinding,
    routes: &[ProductionProviderRoute],
    conformance: &crate::runtime::adapter::AdapterConformanceDescriptor,
    health: &PiHealth,
    checked_at_ms: u64,
) -> Result<Vec<CapabilityRouteEpoch>, RuntimeProductionError> {
    let mut epochs = Vec::new();
    for provider in routes {
        let native_provider_id = provider
            .profile()
            .pi_native_provider_id()
            .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
        for model in provider.models() {
            let route = exact_route(provider, model, RuntimeKind::Pi)?;
            let declaration = model
                .provider_declaration
                .as_ref()
                .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
            let declared = provider_model_declared_evidence(declaration, route.clone())?;
            let adapter = pi_adapter_evidence(
                &PiModelRoute {
                    provider: native_provider_id.into(),
                    model_id: model.model_id.clone(),
                    base_url: provider.profile().endpoint.base_url.clone(),
                },
                &route,
                model.checked_at_ms,
                declaration.expires_at_ms,
            )?;
            if declaration.expires_at_ms <= checked_at_ms {
                return Err(RuntimeProductionError::ProviderRouteUnavailable);
            }
            let observation = observation_from_adapter(
                binding.runtime_id(),
                binding.process_generation(),
                checked_at_ms,
                checked_at_ms,
                declaration.expires_at_ms,
                &adapter,
            );
            let observed = pi_observed_evidence(conformance, health, &observation)?;
            epochs.push(CapabilityRouteEpoch::new(
                binding.runtime_id(),
                binding.process_generation(),
                declared,
                adapter,
                observed,
            )?);
        }
    }
    Ok(epochs)
}

/// Process-idle production root. Asset paths are derived from one verified
/// resource root; callers cannot supply native executable paths or digests.
pub struct RuntimeProductionBootstrap {
    opencode_factory: OpenCodeProductionAssetFactory,
    opencode_assets: ResolvedOpenCodeAssets,
    pi_sidecar_root: PathBuf,
    pi_manifest: PiSidecarManifest,
    pi_dependency_tree_sha256: String,
    node_executable: PathBuf,
    node_sha256: String,
    node_version: String,
    credential_vault: Option<CredentialVault>,
    opencode_loopback_vault: CredentialVault,
    broker_contexts: ActiveBrokerContextResolver,
    broker_facilities: InstalledBrokerFacilityRegistry,
    test_tls_trust_capabilities: Vec<TestTlsTrustCapability>,
}

impl RuntimeProductionBootstrap {
    /// Verifies both complete packaged runtime graphs without starting Node,
    /// OpenCode, Pi, a broker pump, or an event-stream worker.
    pub fn new(
        resource_root: impl Into<PathBuf>,
        credential_vault: Option<CredentialVault>,
    ) -> Result<Self, RuntimeProductionError> {
        let resource_root = canonical_directory(&resource_root.into())?;
        let node = verified_node_runtime(&resource_root)?;
        let opencode_loopback_vault = CredentialVault::session_only()?;

        Self::from_verified_node(
            resource_root,
            node,
            credential_vault,
            opencode_loopback_vault,
        )
    }

    /// Native-test seam for retaining an observer clone of the session-only
    /// loopback vault. It is Rust-only and never exposed through Tauri IPC.
    #[doc(hidden)]
    pub fn new_with_loopback_vault_for_test(
        resource_root: impl Into<PathBuf>,
        credential_vault: Option<CredentialVault>,
        opencode_loopback_vault: CredentialVault,
    ) -> Result<Self, RuntimeProductionError> {
        if opencode_loopback_vault.protection() != VaultProtection::SessionOnly {
            return Err(RuntimeProductionError::CredentialVaultUnavailable);
        }
        let resource_root = canonical_directory(&resource_root.into())?;
        let node = verified_node_runtime(&resource_root)?;
        Self::from_verified_node(
            resource_root,
            node,
            credential_vault,
            opencode_loopback_vault,
        )
    }

    fn from_verified_node(
        resource_root: PathBuf,
        node: VerifiedNodeRuntime,
        credential_vault: Option<CredentialVault>,
        opencode_loopback_vault: CredentialVault,
    ) -> Result<Self, RuntimeProductionError> {
        let VerifiedNodeRuntime {
            executable: node_executable,
            sha256: node_sha256,
            version: node_version,
        } = node;

        let opencode_factory = OpenCodeProductionAssetFactory::new(&resource_root)?;
        let opencode_assets = opencode_factory.resolve()?;
        let pi_sidecar_root = canonical_directory(&resource_root.join("sidecars/pi"))?;
        if !pi_sidecar_root.starts_with(&resource_root) {
            return Err(RuntimeProductionError::InvalidAssetRoot);
        }
        PiSidecarIntegrity::verify(&pi_sidecar_root)?;
        let pi_manifest = PiSidecarManifest::load(&pi_sidecar_root)?;
        let pi_dependency_tree_sha256 =
            PiSidecarIntegrity::dependency_tree_sha256(&pi_sidecar_root)?;

        Ok(Self {
            opencode_factory,
            opencode_assets,
            pi_sidecar_root,
            pi_manifest,
            pi_dependency_tree_sha256,
            node_executable,
            node_sha256,
            node_version,
            credential_vault,
            opencode_loopback_vault,
            broker_contexts: ActiveBrokerContextResolver::new(),
            // This registry is deliberately not exposed. The authenticated
            // pump seals the shared installation set before its first frame;
            // until then no classifier or executor is reachable.
            broker_facilities: InstalledBrokerFacilityRegistry::new(),
            test_tls_trust_capabilities: Vec::new(),
        })
    }

    /// Installs one private, loopback-only CA capability for the ignored
    /// native production golden path. The capability is held only in this
    /// Rust bootstrap, is never accepted through Tauri IPC, and is delivered
    /// to native workers through inherited descriptors rather than argv,
    /// environment bytes, configuration files, or serialized dispatches.
    #[doc(hidden)]
    pub fn install_test_tls_trust_capability(
        &mut self,
        profile: &ProviderProfile,
        ca_pem: Vec<u8>,
    ) -> Result<(), RuntimeProductionError> {
        profile.validate()?;
        let pi_native_provider_id = profile
            .pi_native_provider_id()
            .ok_or(RuntimeProductionError::InvalidTestTlsTrustCapability)?;
        if self.test_tls_trust_capabilities.len() >= MAX_TEST_TLS_TRUST_CAPABILITIES
            || self
                .test_tls_trust_capabilities
                .iter()
                .any(|capability| capability.provider_id == profile.provider_id)
            || !valid_test_tls_loopback_url(&profile.endpoint.base_url)
            || !valid_test_tls_ca_pem(&ca_pem)
        {
            return Err(RuntimeProductionError::InvalidTestTlsTrustCapability);
        }
        self.test_tls_trust_capabilities
            .push(TestTlsTrustCapability {
                provider_id: profile.provider_id.clone(),
                opencode_native_provider_id: profile.opencode_native_provider_id().into(),
                pi_native_provider_id: pi_native_provider_id.into(),
                base_url: profile.endpoint.base_url.clone(),
                ca_pem,
            });
        Ok(())
    }

    fn test_tls_trust_descriptor(
        &self,
        provider_routes: &[ProductionProviderRoute],
        runtime_kind: RuntimeKind,
    ) -> Result<Option<Vec<u8>>, RuntimeProductionError> {
        if self.test_tls_trust_capabilities.is_empty() {
            return Ok(None);
        }
        let mut encoded_capabilities = Vec::new();
        for capability in &self.test_tls_trust_capabilities {
            let route = provider_routes
                .iter()
                .find(|route| route.profile().provider_id == capability.provider_id)
                .ok_or(RuntimeProductionError::InvalidTestTlsTrustCapability)?;
            let profile = route.profile();
            let native_provider_id = match runtime_kind {
                RuntimeKind::OpenCode => profile.opencode_native_provider_id(),
                RuntimeKind::Pi => profile
                    .pi_native_provider_id()
                    .ok_or(RuntimeProductionError::InvalidTestTlsTrustCapability)?,
            };
            let expected_native_provider_id = match runtime_kind {
                RuntimeKind::OpenCode => capability.opencode_native_provider_id.as_str(),
                RuntimeKind::Pi => capability.pi_native_provider_id.as_str(),
            };
            if profile.endpoint.base_url != capability.base_url
                || native_provider_id != expected_native_provider_id
            {
                return Err(RuntimeProductionError::InvalidTestTlsTrustCapability);
            }
            let ca_pem = std::str::from_utf8(&capability.ca_pem)
                .map_err(|_| RuntimeProductionError::InvalidTestTlsTrustCapability)?;
            encoded_capabilities.push(serde_json::json!({
                "providerId": capability.provider_id,
                "nativeProviderId": native_provider_id,
                "baseUrl": capability.base_url,
                "baseUrlSha256": sha256_prefixed(capability.base_url.as_bytes()),
                "tlsCaPem": ca_pem,
                "tlsCaSha256": sha256_prefixed(&capability.ca_pem),
            }));
        }
        let encoded = serde_json::to_vec(&serde_json::json!({
            "schemaVersion": 1,
            "capabilities": encoded_capabilities,
        }))
        .map_err(|_| RuntimeProductionError::InvalidTestTlsTrustCapability)?;
        if encoded.is_empty() || encoded.len() > MAX_TEST_TLS_DESCRIPTOR_BYTES {
            return Err(RuntimeProductionError::InvalidTestTlsTrustCapability);
        }
        Ok(Some(encoded))
    }

    pub fn readiness(&self) -> RuntimeProductionReadiness {
        RuntimeProductionReadiness {
            opencode_native_version: crate::runtime::opencode::OPENCODE_NATIVE_VERSION.into(),
            opencode_native_sha256: self.opencode_assets.native_executable_sha256().to_owned(),
            opencode_native_tree_sha256: self.opencode_assets.native_tree_sha256().to_owned(),
            pi_native_version: self.pi_manifest.native_version.clone(),
            pi_dependency_tree_sha256: self.pi_dependency_tree_sha256.clone(),
            node_sha256: self.node_sha256.clone(),
            node_version: self.node_version.clone(),
            credential_vault_available: self.credential_vault.is_some(),
        }
    }

    /// Describes the two exact verified production slots for one authoritative
    /// Workspace without starting either native process. Immutable assets,
    /// launch executables, and writable state namespaces remain distinct.
    pub fn runtime_installations(
        &self,
        workspace_id: &str,
        c4os_home: &Path,
    ) -> Result<[RuntimeInstallation; 2], RuntimeProductionError> {
        if !valid_id(workspace_id) {
            return Err(RuntimeProductionError::InvalidBinding);
        }
        let c4os_home = canonical_directory(c4os_home)?;
        let opencode_state = c4os_home
            .join("runtimes")
            .join(RuntimeKind::OpenCode.as_str())
            .join(crate::runtime::opencode::OPENCODE_NATIVE_VERSION)
            .join(workspace_id);
        let pi_state = c4os_home
            .join("runtimes")
            .join(RuntimeKind::Pi.as_str())
            .join(PI_NATIVE_VERSION)
            .join(workspace_id);
        fs::create_dir_all(&opencode_state).map_err(RuntimeProductionError::AssetIo)?;
        fs::create_dir_all(&pi_state).map_err(RuntimeProductionError::AssetIo)?;

        let opencode = RuntimeInstallation {
            runtime_id: "opencode-primary".into(),
            workspace_id: workspace_id.into(),
            runtime_kind: RuntimeKind::OpenCode,
            native_version: crate::runtime::opencode::OPENCODE_NATIVE_VERSION.into(),
            adapter_version: "1.0.0".into(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
            install_root: self.opencode_assets.native_root().to_path_buf(),
            asset_tree_sha256: self.opencode_assets.native_tree_sha256().into(),
            executable: self.opencode_assets.native_executable().to_path_buf(),
            executable_sha256: self.opencode_assets.native_executable_sha256().to_owned(),
            state_namespace: opencode_state,
            arguments: Vec::new(),
            sanitized_environment: BTreeMap::new(),
        };
        let pi = RuntimeInstallation {
            runtime_id: "pi-primary".into(),
            workspace_id: workspace_id.into(),
            runtime_kind: RuntimeKind::Pi,
            native_version: self.pi_manifest.native_version.clone(),
            adapter_version: "1.0.0".into(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
            install_root: self.pi_sidecar_root.clone(),
            asset_tree_sha256: self.pi_dependency_tree_sha256.clone(),
            executable: self.node_executable.clone(),
            executable_sha256: self.node_sha256.clone(),
            state_namespace: pi_state,
            arguments: Vec::new(),
            sanitized_environment: BTreeMap::new(),
        };
        opencode.validate()?;
        pi.validate()?;
        Ok([opencode, pi])
    }

    pub fn active_broker_contexts(&self) -> usize {
        self.broker_contexts.active_count()
    }

    /// Rust-only installation boundary for an exact resource facility. This
    /// remains process-idle and becomes immutable when the first production
    /// peer is prepared.
    pub fn install_broker_resource(
        &self,
        resource: impl Into<String>,
        selector: Option<String>,
        classification: InstalledBrokerClassification,
        facility: Box<dyn InstalledBrokerFacility>,
    ) -> Result<(), RuntimeProductionError> {
        self.broker_facilities
            .install_resource(resource, selector, classification, facility)?;
        Ok(())
    }

    /// Rust-only installation boundary for one exact effectful action route.
    /// Renderer and native workers cannot obtain or extend this registry.
    pub fn install_broker_action(
        &self,
        operation: impl Into<String>,
        target: impl Into<String>,
        expected_arguments: serde_json::Map<String, serde_json::Value>,
        classification: InstalledBrokerClassification,
        facility: Box<dyn InstalledBrokerFacility>,
    ) -> Result<(), RuntimeProductionError> {
        self.broker_facilities.install_action(
            operation,
            target,
            expected_arguments,
            classification,
            facility,
        )?;
        Ok(())
    }

    pub fn activate_broker_context(
        &self,
        context: BrokerActionContext,
        active_from_ms: u64,
        expires_at_ms: u64,
    ) -> Result<(), RuntimeProductionError> {
        self.broker_contexts
            .activate(context, active_from_ms, expires_at_ms)
            .map_err(RuntimeProductionError::BrokerContext)
    }

    pub fn refresh_broker_context(
        &self,
        context: BrokerActionContext,
        active_from_ms: u64,
        expires_at_ms: u64,
    ) -> Result<(), RuntimeProductionError> {
        self.broker_contexts
            .refresh(context, active_from_ms, expires_at_ms)
            .map_err(RuntimeProductionError::BrokerContext)
    }

    pub fn retire_broker_context(
        &self,
        context: &BrokerActionContext,
    ) -> Result<bool, RuntimeProductionError> {
        self.broker_contexts
            .retire(context)
            .map_err(RuntimeProductionError::BrokerContext)
    }

    /// Prepares the exact peer using only the binding derived by the Rust
    /// application service. Endpoint selection and loopback credentials are
    /// minted here and never accepted from a renderer or command caller.
    pub(crate) fn prepare_core_owned_peer(
        &self,
        binding: ProductionRuntimeBinding,
        provider_routes: &[ProductionProviderRoute],
        checked_at_ms: u64,
    ) -> Result<PreparedCoreProductionPeer, RuntimeProductionError> {
        match binding.runtime_kind() {
            RuntimeKind::OpenCode => {
                let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
                    .map_err(RuntimeProductionError::LoopbackReservation)?;
                let endpoint = LoopbackEndpoint::new(
                    listener
                        .local_addr()
                        .map_err(RuntimeProductionError::LoopbackReservation)?
                        .ip(),
                    listener
                        .local_addr()
                        .map_err(RuntimeProductionError::LoopbackReservation)?
                        .port(),
                )
                .map_err(RuntimeProductionError::OpenCodeAdapter)?;
                let password_reference = RandomSecretReference::new(
                    format!(
                        "opencode-password-{}-{}",
                        binding.launch_id(),
                        binding.process_generation()
                    ),
                    256,
                )
                .map_err(RuntimeProductionError::OpenCodeAdapter)?;
                let prepared = self.prepare_opencode_peer_with_profiles(
                    binding,
                    listener,
                    endpoint,
                    password_reference,
                    provider_routes,
                    checked_at_ms,
                )?;
                Ok(PreparedCoreProductionPeer::OpenCode(Box::new(prepared)))
            }
            RuntimeKind::Pi => {
                if provider_routes
                    .iter()
                    .any(|route| route.profile().pi_native_provider_id().is_none())
                {
                    return Err(RuntimeProductionError::ProviderRouteUnavailable);
                }
                let mut prepared =
                    self.prepare_pi_peer_with_profiles(binding, provider_routes, checked_at_ms)?;
                prepared.register_provider_credential_routes(provider_routes)?;
                Ok(PreparedCoreProductionPeer::Pi(Box::new(prepared)))
            }
        }
    }

    fn prepare_opencode_peer_with_profiles(
        &self,
        binding: ProductionRuntimeBinding,
        listener: TcpListener,
        endpoint: LoopbackEndpoint,
        password_reference: RandomSecretReference,
        provider_routes: &[ProductionProviderRoute],
        checked_at_ms: u64,
    ) -> Result<PreparedOpenCodeProductionPeer, RuntimeProductionError> {
        require_kind(&binding, RuntimeKind::OpenCode)?;
        if checked_at_ms == 0 {
            return Err(RuntimeProductionError::InvalidBinding);
        }
        let credential_vault = self
            .credential_vault
            .clone()
            .ok_or(RuntimeProductionError::CredentialVaultUnavailable)?;
        let loopback_credential = OwnedLoopbackCredential::random(
            self.opencode_loopback_vault.clone(),
            password_reference.clone(),
        )?;
        let credentials = loopback_credential.resolver();
        let prepared_driver = self.opencode_factory.construct_driver_with_provider_vault(
            &self.node_executable,
            &self.node_sha256,
            credentials.clone(),
            credential_vault.clone(),
            binding.process_generation,
            OPEN_CODE_STARTUP_TIMEOUT,
            OPEN_CODE_SHUTDOWN_TIMEOUT,
        )?;
        if prepared_driver.assets() != &self.opencode_assets {
            return Err(RuntimeProductionError::AssetChanged);
        }
        let (assets, mut driver) = prepared_driver.into_parts();
        if let Some(descriptor) =
            self.test_tls_trust_descriptor(provider_routes, RuntimeKind::OpenCode)?
        {
            driver
                .install_test_tls_trust_descriptor(descriptor)
                .map_err(RuntimeProductionError::NativeBoundary)?;
        }
        driver
            .install_loopback_reservation(listener)
            .map_err(RuntimeProductionError::NativeBoundary)?;
        let manifest =
            OpenCodeCompatibilityManifest::pinned(assets.native_executable_sha256().to_owned())
                .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        let descriptor = manifest
            .conformance_descriptor(binding.process_generation)
            .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        let namespace = StateNamespace::new(
            &binding.c4os_home,
            &binding.workspace_id,
            binding.process_generation,
            &binding.launch_id,
        )
        .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        materialize_opencode_provider_configuration(&namespace, provider_routes)?;
        let authority_policy = NativeAuthorityPolicy::new(
            opencode_authority_configuration_sha256(),
            OPENCODE_C4OS_TOOL_IDS,
        )
        .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        let launch_plan = OpenCodeLaunchPlan {
            manifest,
            endpoint: endpoint.clone(),
            namespace,
            executable: assets.native_executable().to_path_buf(),
            workspace_root: binding.workspace_root.clone(),
            basic_auth_username: "opencode".into(),
            password_reference,
            secret_channel_fd: OPEN_CODE_SECRET_DESCRIPTOR,
            authority_policy,
        };
        let transport = LoopbackHttpTransport::new(
            endpoint,
            credentials,
            OPEN_CODE_CONNECT_TIMEOUT,
            OPEN_CODE_IO_TIMEOUT,
        )
        .map_err(RuntimeProductionError::NativeBoundary)?;
        let shared_driver = SharedOpenCodeDriver::new(driver);
        let mut adapter = OpenCodeAdapter::new(launch_plan, transport, shared_driver.clone())
            .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        for route in provider_routes {
            adapter
                .register_provider_credential_route(route.profile())
                .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        }
        let health = adapter
            .start(checked_at_ms)
            .map_err(RuntimeProductionError::OpenCodeAdapter)?;
        let capability_epochs = opencode_capability_epochs(
            &binding,
            provider_routes,
            &descriptor,
            &health,
            checked_at_ms,
        )?;
        let broker_pump = shared_driver.with_driver(|driver| {
            OpenCodeBrokerPump::take_from_started_driver(
                driver,
                self.broker_facilities.clone(),
                OpenCodeBrokerPumpConfig {
                    receive_timeout: OPEN_CODE_BROKER_RECEIVE_TIMEOUT,
                    ..OpenCodeBrokerPumpConfig::default()
                },
            )
        })??;
        let registration = RuntimePeerRegistration {
            runtime_id: binding.runtime_id.clone(),
            workspace_id: binding.workspace_id.clone(),
            descriptor,
        };
        registration.validate()?;

        Ok(PreparedOpenCodeProductionPeer {
            binding,
            registration,
            adapter,
            broker_pump,
            broker_contexts: self.broker_contexts.clone(),
            credential_vault,
            capability_epochs,
            loopback_credential,
        })
    }

    /// Starts and health-checks the pinned Pi sidecar only for a bound runtime.
    /// Integrity and the Node digest are rechecked again inside `spawn`.
    pub fn prepare_pi_peer(
        &self,
        binding: ProductionRuntimeBinding,
    ) -> Result<PreparedPiProductionPeer, RuntimeProductionError> {
        self.prepare_pi_peer_with_profiles(binding, &[], 1)
    }

    fn prepare_pi_peer_with_profiles(
        &self,
        binding: ProductionRuntimeBinding,
        provider_routes: &[ProductionProviderRoute],
        checked_at_ms: u64,
    ) -> Result<PreparedPiProductionPeer, RuntimeProductionError> {
        require_kind(&binding, RuntimeKind::Pi)?;
        if checked_at_ms == 0 {
            return Err(RuntimeProductionError::InvalidBinding);
        }
        let credential_vault = self
            .credential_vault
            .clone()
            .ok_or(RuntimeProductionError::CredentialVaultUnavailable)?;
        PiSidecarIntegrity::verify(&self.pi_sidecar_root)?;
        let current_manifest = PiSidecarManifest::load(&self.pi_sidecar_root)?;
        if current_manifest != self.pi_manifest
            || PiSidecarIntegrity::dependency_tree_sha256(&self.pi_sidecar_root)?
                != self.pi_dependency_tree_sha256
        {
            return Err(RuntimeProductionError::AssetChanged);
        }
        let runner = match self.test_tls_trust_descriptor(provider_routes, RuntimeKind::Pi)? {
            Some(descriptor) => SpawnedPiRunner::spawn_with_test_tls_trust(
                &self.node_executable,
                &self.node_sha256,
                &self.pi_sidecar_root,
                &self.pi_manifest,
                binding.process_generation,
                PI_EXCHANGE_TIMEOUT,
                descriptor,
            )?,
            None => SpawnedPiRunner::spawn(
                &self.node_executable,
                &self.node_sha256,
                &self.pi_sidecar_root,
                &self.pi_manifest,
                binding.process_generation,
                PI_EXCHANGE_TIMEOUT,
            )?,
        };
        let shared_runner = SharedPiRunner::new(runner);
        let descriptor = self
            .pi_manifest
            .conformance_descriptor(binding.process_generation, true)?;
        let mut adapter = PiAdapter::new(
            self.pi_manifest.clone(),
            shared_runner.clone(),
            binding.process_generation,
        )?;
        let (health, _) = adapter.start()?;
        for provider in provider_routes {
            let native_provider_id = provider
                .profile()
                .pi_native_provider_id()
                .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
            for model in provider.models() {
                let route = PiModelRoute {
                    provider: native_provider_id.into(),
                    model_id: model.model_id.clone(),
                    base_url: provider.profile().endpoint.base_url.clone(),
                };
                if adapter.preflight_model(&route) != Ok(true) {
                    return Err(RuntimeProductionError::ProviderRouteUnavailable);
                }
            }
        }
        let registration = RuntimePeerRegistration {
            runtime_id: binding.runtime_id.clone(),
            workspace_id: binding.workspace_id.clone(),
            descriptor,
        };
        registration.validate()?;
        let capability_epochs = pi_capability_epochs(
            &binding,
            provider_routes,
            &registration.descriptor,
            &health,
            checked_at_ms,
        )?;
        let peer = PiDispatchPeer::new(registration, adapter)?;
        Ok(PreparedPiProductionPeer {
            binding,
            peer,
            runner: shared_runner,
            credential_vault,
            broker_facilities: self.broker_facilities.clone(),
            capability_epochs,
        })
    }
}

impl fmt::Debug for RuntimeProductionBootstrap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeProductionBootstrap")
            .field("opencode", &self.opencode_assets)
            .field("pi_sidecar_root", &self.pi_sidecar_root)
            .field("pi_native_version", &self.pi_manifest.native_version)
            .field("node_executable", &self.node_executable)
            .field("node_version", &self.node_version)
            .field(
                "credential_vault",
                &self.credential_vault.as_ref().map(|_| "<Rust-owned>"),
            )
            .field("broker_contexts", &self.broker_contexts)
            .field("broker_facilities", &"<closed installation set>")
            .finish()
    }
}

#[derive(Clone)]
struct SharedOpenCodeDriver {
    inner: Arc<Mutex<OpenCodeNativeCommandDriver>>,
}

impl SharedOpenCodeDriver {
    fn new(driver: OpenCodeNativeCommandDriver) -> Self {
        Self {
            inner: Arc::new(Mutex::new(driver)),
        }
    }

    fn with_driver<T>(
        &self,
        operation: impl FnOnce(&mut OpenCodeNativeCommandDriver) -> T,
    ) -> Result<T, RuntimeProductionError> {
        self.inner
            .lock()
            .map(|mut driver| operation(&mut driver))
            .map_err(|_| RuntimeProductionError::DriverUnavailable)
    }
}

impl CommandDriver for SharedOpenCodeDriver {
    fn spawn(&mut self, command: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode> {
        self.inner
            .lock()
            .map_err(|_| CommandFailureCode::SpawnRejected)?
            .spawn(command)
    }

    fn terminate_process_group(
        &mut self,
        process: &ProcessHandle,
    ) -> Result<(), CommandFailureCode> {
        self.inner
            .lock()
            .map_err(|_| CommandFailureCode::TerminationFailed)?
            .terminate_process_group(process)
    }

    fn authorize_provider_credential_attempt(
        &mut self,
        request: ProviderCredentialRequest,
    ) -> Result<Option<ProviderCredentialAuthorizationReceipt>, CommandFailureCode> {
        self.inner
            .lock()
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)?
            .authorize_provider_credential_attempt(request)
    }

    fn revoke_provider_credential_attempt(
        &mut self,
        request: &ProviderCredentialRequest,
    ) -> Result<(), CommandFailureCode> {
        self.inner
            .lock()
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)?
            .revoke_provider_credential_attempt(request)
    }

    fn revoke_all_provider_credential_attempts(&mut self) -> Result<(), CommandFailureCode> {
        self.inner
            .lock()
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)?
            .revoke_all_provider_credential_attempts()
    }

    fn register_provider_credential_route(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<(), CommandFailureCode> {
        self.inner
            .lock()
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)?
            .register_provider_credential_route(profile)
    }
}

#[derive(Clone)]
struct SharedPiRunner {
    inner: Arc<Mutex<SpawnedPiRunner>>,
}

impl SharedPiRunner {
    fn new(runner: SpawnedPiRunner) -> Self {
        Self {
            inner: Arc::new(Mutex::new(runner)),
        }
    }

    fn with_runner<T>(
        &self,
        operation: impl FnOnce(&mut SpawnedPiRunner) -> T,
    ) -> Result<T, RuntimeProductionError> {
        self.inner
            .lock()
            .map(|mut runner| operation(&mut runner))
            .map_err(|_| RuntimeProductionError::RunnerUnavailable)
    }
}

impl PiSidecarRunner for SharedPiRunner {
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String> {
        self.inner
            .lock()
            .map_err(|_| "Pi production runner is unavailable".to_owned())?
            .exchange(request_line)
    }

    fn poll(&mut self) -> Result<Vec<String>, String> {
        self.inner
            .lock()
            .map_err(|_| "Pi production runner is unavailable".to_owned())?
            .poll()
    }

    fn terminate(&mut self) -> Result<(), String> {
        self.inner
            .lock()
            .map_err(|_| "Pi production runner is unavailable".to_owned())?
            .terminate()
    }
}

struct OwnedLoopbackCredential {
    vault: CredentialVault,
    resolver: VaultCredentialResolver,
    random_reference: RandomSecretReference,
    credential_reference: Option<CredentialReference>,
}

impl OwnedLoopbackCredential {
    fn random(
        vault: CredentialVault,
        random_reference: RandomSecretReference,
    ) -> Result<Self, RuntimeProductionError> {
        if vault.protection() != VaultProtection::SessionOnly {
            return Err(RuntimeProductionError::CredentialVaultUnavailable);
        }
        let credential_reference = vault.store_random_hex("opencode-loopback-password", 32)?;
        let registered_reference = credential_reference.clone();
        let resolver = VaultCredentialResolver::new(vault.clone());
        let mut owned = Self {
            vault,
            resolver,
            random_reference,
            credential_reference: Some(credential_reference),
        };
        if let Err(error) = owned
            .resolver
            .register(&owned.random_reference, registered_reference)
        {
            owned.revoke();
            return Err(RuntimeProductionError::NativeBoundary(error));
        }
        Ok(owned)
    }

    fn resolver(&self) -> VaultCredentialResolver {
        self.resolver.clone()
    }

    fn revoke(&mut self) {
        let Some(credential_reference) = self.credential_reference.as_ref() else {
            return;
        };
        let _ = self
            .resolver
            .unregister(&self.random_reference, credential_reference);
        if self.vault.remove(credential_reference).is_ok() {
            self.credential_reference = None;
        }
    }
}

impl Drop for OwnedLoopbackCredential {
    fn drop(&mut self) {
        self.revoke();
    }
}

pub struct PreparedOpenCodeProductionPeer {
    binding: ProductionRuntimeBinding,
    registration: RuntimePeerRegistration,
    adapter: OpenCodeAdapter<LoopbackHttpTransport, SharedOpenCodeDriver>,
    broker_pump: OpenCodeBrokerPump,
    broker_contexts: ActiveBrokerContextResolver,
    credential_vault: CredentialVault,
    capability_epochs: Vec<CapabilityRouteEpoch>,
    loopback_credential: OwnedLoopbackCredential,
}

pub(crate) enum PreparedCoreProductionPeer {
    OpenCode(Box<PreparedOpenCodeProductionPeer>),
    Pi(Box<PreparedPiProductionPeer>),
}

impl PreparedCoreProductionPeer {
    pub(crate) fn binding(&self) -> &ProductionRuntimeBinding {
        match self {
            Self::OpenCode(prepared) => &prepared.binding,
            Self::Pi(prepared) => &prepared.binding,
        }
    }

    pub(crate) fn native_process_id(&self) -> Result<u32, RuntimeProductionError> {
        match self {
            Self::OpenCode(prepared) => prepared
                .native_process_id()
                .ok_or(RuntimeProductionError::MissingNativeProcess),
            Self::Pi(prepared) => prepared.native_process_id(),
        }
    }

    pub(crate) fn capability_epochs(&self) -> Vec<CapabilityRouteEpoch> {
        match self {
            Self::OpenCode(prepared) => prepared.capability_epochs.clone(),
            Self::Pi(prepared) => prepared.capability_epochs.clone(),
        }
    }

    pub(crate) fn register(
        self,
        registry: &mut RuntimeDispatchRegistry,
    ) -> Result<CoreProductionWorker, RuntimeProductionError> {
        match self {
            Self::OpenCode(prepared) => Ok(CoreProductionWorker::OpenCode(
                (*prepared).register(registry, OpenCodeStreamBounds::default())?,
            )),
            Self::Pi(prepared) => Ok(CoreProductionWorker::Pi((*prepared).register(registry)?)),
        }
    }
}

pub(crate) enum CoreProductionWorker {
    OpenCode(OpenCodeProductionWorker),
    Pi(PiProductionWorker),
}

impl CoreProductionWorker {
    pub(crate) fn binding(&self) -> &ProductionRuntimeBinding {
        match self {
            Self::OpenCode(worker) => worker.binding(),
            Self::Pi(worker) => worker.binding(),
        }
    }

    pub(crate) fn revoke_transient_credentials(&mut self) {
        if let Self::OpenCode(worker) = self {
            worker.revoke_transient_credentials();
        }
    }
}

impl PreparedOpenCodeProductionPeer {
    pub fn native_process_id(&self) -> Option<u32> {
        self.adapter.process_id()
    }

    pub fn register_provider_credential_route(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<(), RuntimeProductionError> {
        self.adapter
            .register_provider_credential_route(profile)
            .map_err(RuntimeProductionError::OpenCodeAdapter)
    }

    /// Executes the production provider test while the exact started adapter is
    /// still core-owned and before it moves into the dispatch registry.
    pub fn test_provider(
        &mut self,
        profile: &ProviderProfile,
        checked_at_ms: u64,
        session_configuration_sha256: impl Into<String>,
    ) -> Result<ProviderDiscovery, RuntimeProductionError> {
        let mut connectivity = CurlProviderConnectivity::macos_system()?;
        let mut probe = VerifiedOpenCodeProviderProbe::new(
            &mut self.adapter,
            &self.credential_vault,
            &mut connectivity,
            checked_at_ms,
            session_configuration_sha256,
        )?;
        probe
            .test_and_discover(profile)
            .map_err(RuntimeProductionError::ProviderProbe)
    }

    pub fn register(
        self,
        registry: &mut RuntimeDispatchRegistry,
        stream_bounds: OpenCodeStreamBounds,
    ) -> Result<OpenCodeProductionWorker, RuntimeProductionError> {
        stream_bounds
            .validate()
            .map_err(|_| RuntimeProductionError::InvalidStreamBounds)?;
        let mut peer = OpenCodeDispatchPeer::new(self.registration, self.adapter)?;
        peer.attach_broker_context_resolver(self.broker_contexts.clone())?;
        peer.attach_production_event_stream(stream_bounds)?;
        registry.register(peer)?;
        Ok(OpenCodeProductionWorker {
            binding: self.binding,
            broker_pump: self.broker_pump,
            broker_contexts: self.broker_contexts,
            loopback_credential: self.loopback_credential,
        })
    }
}

pub struct OpenCodeProductionWorker {
    binding: ProductionRuntimeBinding,
    broker_pump: OpenCodeBrokerPump,
    broker_contexts: ActiveBrokerContextResolver,
    loopback_credential: OwnedLoopbackCredential,
}

impl OpenCodeProductionWorker {
    pub fn binding(&self) -> &ProductionRuntimeBinding {
        &self.binding
    }

    pub(crate) fn revoke_transient_credentials(&mut self) {
        self.loopback_credential.revoke();
    }

    pub fn pending_approvals(&self) -> usize {
        self.broker_pump.pending_count()
    }

    pub fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
        self.broker_pump.pending_approval_descriptors()
    }

    pub fn pump_one<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        now_ms: u64,
    ) -> Result<OpenCodeBrokerPumpOutcome, OpenCodeBrokerPumpError> {
        self.broker_pump
            .pump_one(application, &mut self.broker_contexts, now_ms)
    }

    pub fn answer_approval<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        correlation_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<OpenCodeBrokerPumpOutcome, OpenCodeBrokerPumpError> {
        self.broker_pump.answer_approval(
            application,
            &mut self.broker_contexts,
            correlation_id,
            prompt_id,
            answer,
            now_ms,
        )
    }
}

pub struct PreparedPiProductionPeer {
    binding: ProductionRuntimeBinding,
    peer: PiDispatchPeer<SharedPiRunner>,
    runner: SharedPiRunner,
    credential_vault: CredentialVault,
    broker_facilities: InstalledBrokerFacilityRegistry,
    capability_epochs: Vec<CapabilityRouteEpoch>,
}

impl PreparedPiProductionPeer {
    pub fn native_process_id(&self) -> Result<u32, RuntimeProductionError> {
        self.runner.with_runner(|runner| runner.process_id())
    }

    pub(crate) fn register_provider_credential_routes(
        &mut self,
        routes: &[ProductionProviderRoute],
    ) -> Result<(), RuntimeProductionError> {
        let mut credentials = BTreeMap::new();
        for route in routes {
            let profile = route.profile();
            profile.validate()?;
            let native_provider_id = profile
                .pi_native_provider_id()
                .ok_or(RuntimeProductionError::ProviderRouteUnavailable)?;
            if credentials
                .insert(
                    profile.provider_id.clone(),
                    (
                        native_provider_id.to_owned(),
                        profile.endpoint.base_url.clone(),
                        profile.credential_reference.clone(),
                    ),
                )
                .is_some()
            {
                return Err(RuntimeProductionError::ProviderRouteUnavailable);
            }
        }
        self.peer
            .attach_credential_issuer(ProductionPiCredentialIssuer {
                binding: self.binding.clone(),
                credentials,
                delivery: Box::new(self.runner.clone()),
                credential_vault: self.credential_vault.clone(),
            });
        Ok(())
    }

    pub fn register(
        self,
        registry: &mut RuntimeDispatchRegistry,
    ) -> Result<PiProductionWorker, RuntimeProductionError> {
        let gateway =
            PiProductionGatewayWorker::attach(self.binding.clone(), self.broker_facilities)?;
        registry.register(self.peer)?;
        Ok(PiProductionWorker {
            binding: self.binding,
            runner: self.runner,
            credential_vault: self.credential_vault,
            gateway,
            pending_actions: PendingPiProductionActions::default(),
        })
    }
}

struct ProductionPiCredentialIssuer {
    binding: ProductionRuntimeBinding,
    credentials: BTreeMap<String, (String, String, CredentialReference)>,
    delivery: Box<dyn PiCredentialLeaseDelivery>,
    credential_vault: CredentialVault,
}

trait PiCredentialLeaseDelivery: Send {
    fn deliver(
        &mut self,
        metadata: &PiCredentialLeaseMetadata,
        lease: &OperationCredentialLease,
    ) -> Result<(), PeerDispatchError>;
}

impl PiCredentialLeaseDelivery for SharedPiRunner {
    fn deliver(
        &mut self,
        metadata: &PiCredentialLeaseMetadata,
        lease: &OperationCredentialLease,
    ) -> Result<(), PeerDispatchError> {
        self.with_runner(|runner| runner.deliver_credential_lease(metadata, lease))
            .map_err(|_| PeerDispatchError::Credential)?
            .map_err(|_| PeerDispatchError::Credential)
    }
}

impl PiDispatchCredentialIssuer for ProductionPiCredentialIssuer {
    fn native_provider_id(&self, provider_id: &str) -> Result<String, PeerDispatchError> {
        self.credentials
            .get(provider_id)
            .map(|(native_provider_id, _, _)| native_provider_id.clone())
            .ok_or(PeerDispatchError::Credential)
    }

    fn base_url(&self, provider_id: &str) -> Result<String, PeerDispatchError> {
        self.credentials
            .get(provider_id)
            .map(|(_, base_url, _)| base_url.clone())
            .ok_or(PeerDispatchError::Credential)
    }

    fn deliver_for_dispatch(
        &mut self,
        identity: &DispatchIdentity,
        provider_id: &str,
    ) -> Result<(), PeerDispatchError> {
        if identity.runtime_id != self.binding.runtime_id
            || identity.workspace_id != self.binding.workspace_id
            || identity.process_generation != self.binding.process_generation
        {
            return Err(PeerDispatchError::Credential);
        }
        let (native_provider_id, _, credential_reference) = self
            .credentials
            .get(provider_id)
            .ok_or(PeerDispatchError::Credential)?;
        let operation = pi_credential_operation(identity, provider_id, native_provider_id);
        let now_ms: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| PeerDispatchError::Credential)?
            .as_millis()
            .try_into()
            .map_err(|_| PeerDispatchError::Credential)?;
        let metadata = PiCredentialLeaseMetadata {
            lease_id: operation.clone(),
            provider: native_provider_id.clone(),
            expires_at_ms: now_ms.saturating_add(
                u64::try_from(PI_CREDENTIAL_LEASE_TTL.as_millis())
                    .map_err(|_| PeerDispatchError::Credential)?,
            ),
        };
        let lease = self
            .credential_vault
            .lease_for_operation(
                credential_reference,
                operation.clone(),
                PI_CREDENTIAL_LEASE_TTL,
            )
            .map_err(|_| PeerDispatchError::Credential)?;
        self.delivery.deliver(&metadata, &lease)
    }
}

fn pi_credential_operation(
    identity: &DispatchIdentity,
    provider_id: &str,
    native_provider_id: &str,
) -> String {
    let generation = identity.process_generation.to_string();
    let mut digest = Sha256::new();
    for field in [
        identity.runtime_id.as_str(),
        identity.workspace_id.as_str(),
        identity.session_id.as_str(),
        identity.turn_id.as_str(),
        identity.attempt_id.as_str(),
        identity.correlation_id.as_str(),
        generation.as_str(),
        provider_id,
        native_provider_id,
    ] {
        digest.update((field.len() as u64).to_be_bytes());
        digest.update(field.as_bytes());
    }
    let bytes = digest.finalize();
    let mut operation = String::with_capacity(76);
    operation.push_str("pi-provider:");
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(operation, "{byte:02x}");
    }
    operation
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiGatewayAuthorityContext {
    pub request_origin: ActionRequestOrigin,
    pub configuration_version: u64,
    pub policy_version: u64,
    pub revocation_epoch: u64,
    pub eligible_tool_ids: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PiProductionGatewayOutcome {
    PendingApproval {
        native_request_id: String,
        prompt_id: String,
    },
    Completed {
        native_request_id: String,
        result_code: String,
    },
    Denied {
        native_request_id: String,
        reason_code: String,
    },
}

/// Result of the coordinator/gateway phase of one Pi tool continuation. The
/// registered peer is resolved only in a later registry phase so production
/// pumping never holds the dispatch lock while entering application authority.
pub(crate) struct PiProductionGatewaySettlement {
    identity: DispatchIdentity,
    outcome: RuntimeBrokerWorkerOutcome,
}

/// Core-owned gateway continuation for one production Pi peer. The worker
/// accepts only the private Pi action material created by `PiDispatchPeer`
/// after active-run and process-generation validation. It owns every pending
/// approval until an exact completed or denied resolution is written back to
/// that same registered peer.
struct PiProductionGatewayWorker {
    binding: ProductionRuntimeBinding,
    worker: BrokerActionWorker<InstalledBrokerFacilityRegistry>,
    executor: InstalledBrokerFacilityRegistry,
    pending: BTreeMap<String, DispatchIdentity>,
}

impl PiProductionGatewayWorker {
    fn attach(
        binding: ProductionRuntimeBinding,
        facilities: InstalledBrokerFacilityRegistry,
    ) -> Result<Self, RuntimeProductionError> {
        require_kind(&binding, RuntimeKind::Pi)?;
        facilities.seal()?;
        Ok(Self {
            binding,
            worker: BrokerActionWorker::new(facilities.clone()),
            executor: facilities,
            pending: BTreeMap::new(),
        })
    }

    fn pending_count(&self) -> usize {
        self.pending.len()
    }

    fn pending_identity(&self, correlation_id: &str) -> Option<DispatchIdentity> {
        self.pending
            .values()
            .find(|identity| identity.correlation_id == correlation_id)
            .cloned()
    }

    fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
        self.pending
            .iter()
            .filter_map(|(native_request_id, identity)| {
                self.worker
                    .pending_runtime_approval_prompt(native_request_id)
                    .map(|prompt_id| (identity.correlation_id.clone(), prompt_id.to_owned()))
            })
            .collect()
    }

    fn evaluate_action_intent<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        event: &DispatchEvent,
        authority: PiGatewayAuthorityContext,
        now_ms: u64,
    ) -> Result<PiProductionGatewaySettlement, RuntimeProductionError> {
        let DispatchEventCategory::PiActionIntent(action_intent) = &event.peer.category else {
            return Err(RuntimeProductionError::InvalidPiActionIntent);
        };
        self.validate_event_binding(event, action_intent.identity())?;
        let context = pi_broker_context(
            &event.peer.identity,
            action_intent.identity().native_request_id.as_str(),
            authority,
        )?;
        let outcome = self.worker.accept_runtime_intent(
            application,
            &mut self.executor,
            action_intent.identity().clone(),
            action_intent.arguments().clone(),
            context,
            now_ms,
        )?;
        Ok(PiProductionGatewaySettlement {
            identity: event.peer.identity.clone(),
            outcome,
        })
    }

    fn evaluate_approval<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        native_request_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        authority: PiGatewayAuthorityContext,
        now_ms: u64,
    ) -> Result<PiProductionGatewaySettlement, RuntimeProductionError> {
        let identity = self
            .pending
            .get(native_request_id)
            .cloned()
            .ok_or(RuntimeProductionError::UnknownPiApproval)?;
        let context = pi_broker_context(&identity, native_request_id, authority)?;
        let outcome = self.worker.answer_runtime_intent_approval(
            application,
            &mut self.executor,
            RuntimeBrokerApprovalAnswer {
                native_request_id,
                prompt_id,
                answer,
                current_context: &context,
                now_ms,
            },
        )?;
        Ok(PiProductionGatewaySettlement { identity, outcome })
    }

    fn validate_event_binding(
        &self,
        event: &DispatchEvent,
        intent: &crate::runtime::action_bridge::RuntimeIntentIdentity,
    ) -> Result<(), RuntimeProductionError> {
        if event.sequence == 0
            || event.peer.identity.runtime_kind != RuntimeKind::Pi
            || event.peer.identity.runtime_id != self.binding.runtime_id
            || event.peer.identity.workspace_id != self.binding.workspace_id
            || event.peer.identity.process_generation != self.binding.process_generation
            || intent.runtime_id != event.peer.identity.runtime_id
            || intent.workspace_id != event.peer.identity.workspace_id
            || intent.session_id != event.peer.identity.session_id
            || intent.turn_id != event.peer.identity.turn_id
            || intent.run_id != event.peer.identity.attempt_id
            || intent.correlation_id != event.peer.identity.correlation_id
            || intent.process_generation != event.peer.identity.process_generation
        {
            return Err(RuntimeProductionError::InvalidPiActionIntent);
        }
        Ok(())
    }

    fn settle(
        &mut self,
        registry: &mut RuntimeDispatchRegistry,
        settlement: &PiProductionGatewaySettlement,
    ) -> Result<PiProductionGatewayOutcome, RuntimeProductionError> {
        let PiProductionGatewaySettlement { identity, outcome } = settlement;
        match outcome {
            RuntimeBrokerWorkerOutcome::PendingApproval {
                native_request_id,
                prompt_id,
            } => {
                match self.pending.entry(native_request_id.clone()) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(identity.clone());
                    }
                    std::collections::btree_map::Entry::Occupied(_) => {
                        return Err(RuntimeProductionError::PiApprovalBindingMismatch);
                    }
                }
                Ok(PiProductionGatewayOutcome::PendingApproval {
                    native_request_id: native_request_id.clone(),
                    prompt_id: prompt_id.clone(),
                })
            }
            RuntimeBrokerWorkerOutcome::Denied {
                native_request_id,
                reason_code,
            } => {
                resolve_pi_or_cancel(
                    registry,
                    &mut self.pending,
                    identity,
                    native_request_id,
                    |registry| registry.resolve_pi_denied(identity, native_request_id, reason_code),
                )?;
                Ok(PiProductionGatewayOutcome::Denied {
                    native_request_id: native_request_id.clone(),
                    reason_code: reason_code.clone(),
                })
            }
            RuntimeBrokerWorkerOutcome::Executed {
                native_request_id,
                receipt,
            } => {
                let result_code = receipt.result().result_code.clone();
                if receipt.result().status == NormalizedActionStatus::Succeeded {
                    resolve_pi_or_cancel(
                        registry,
                        &mut self.pending,
                        identity,
                        native_request_id,
                        |registry| {
                            registry.resolve_pi_completed(identity, native_request_id, receipt)
                        },
                    )?;
                    Ok(PiProductionGatewayOutcome::Completed {
                        native_request_id: native_request_id.clone(),
                        result_code,
                    })
                } else {
                    resolve_pi_or_cancel(
                        registry,
                        &mut self.pending,
                        identity,
                        native_request_id,
                        |registry| {
                            registry.resolve_pi_denied(identity, native_request_id, &result_code)
                        },
                    )?;
                    Ok(PiProductionGatewayOutcome::Denied {
                        native_request_id: native_request_id.clone(),
                        reason_code: result_code,
                    })
                }
            }
        }
    }
}

fn pi_broker_context(
    dispatch: &DispatchIdentity,
    native_request_id: &str,
    authority: PiGatewayAuthorityContext,
) -> Result<BrokerActionContext, RuntimeProductionError> {
    if dispatch.runtime_kind != RuntimeKind::Pi
        || authority.request_origin == ActionRequestOrigin::Unknown
        || authority.configuration_version == 0
        || authority.policy_version == 0
        || authority.eligible_tool_ids.len() > OPENCODE_C4OS_TOOL_IDS.len()
        || authority
            .eligible_tool_ids
            .iter()
            .any(|tool| !OPENCODE_C4OS_TOOL_IDS.contains(&tool.as_str()))
        || !valid_id(native_request_id)
    {
        return Err(RuntimeProductionError::InvalidPiActionIntent);
    }
    Ok(BrokerActionContext {
        dispatch: dispatch.clone(),
        native_session_id: dispatch.session_id.clone(),
        native_message_id: native_request_id.to_owned(),
        eligible_tool_ids: authority.eligible_tool_ids,
        request_origin: authority.request_origin,
        configuration_version: authority.configuration_version,
        policy_version: authority.policy_version,
        revocation_epoch: authority.revocation_epoch,
    })
}

fn resolve_pi_or_cancel(
    registry: &mut RuntimeDispatchRegistry,
    pending: &mut BTreeMap<String, DispatchIdentity>,
    identity: &DispatchIdentity,
    native_request_id: &str,
    resolve: impl FnOnce(&mut RuntimeDispatchRegistry) -> Result<(), DispatchError>,
) -> Result<(), RuntimeProductionError> {
    if pending
        .get(native_request_id)
        .is_some_and(|pending_identity| pending_identity != identity)
    {
        return Err(RuntimeProductionError::PiApprovalBindingMismatch);
    }
    if resolve(registry).is_ok() {
        pending.remove(native_request_id);
        return Ok(());
    }
    if matches!(registry.cancel(identity), Ok(true)) {
        pending.remove(native_request_id);
        return Err(RuntimeProductionError::PiToolResolutionFailed);
    }
    // Neither exact tool settlement nor run cancellation reached the native
    // process. Retain the identity so this process generation can be
    // quarantined without losing the unresolved approval/tool binding.
    pending
        .entry(native_request_id.to_owned())
        .or_insert_with(|| identity.clone());
    Err(RuntimeProductionError::PiToolSettlementGenerationFatal)
}

pub struct PiProductionWorker {
    binding: ProductionRuntimeBinding,
    runner: SharedPiRunner,
    credential_vault: CredentialVault,
    gateway: PiProductionGatewayWorker,
    pending_actions: PendingPiProductionActions,
}

enum PendingPiProductionAction {
    AwaitingEvaluation(DispatchEvent),
    AwaitingSettlement(PiProductionGatewaySettlement),
}

#[derive(Default)]
struct PendingPiProductionActions {
    items: VecDeque<PendingPiProductionAction>,
}

impl PendingPiProductionActions {
    fn enqueue(
        &mut self,
        binding: &ProductionRuntimeBinding,
        events: Vec<DispatchEvent>,
    ) -> Result<(), RuntimeProductionError> {
        if self
            .items
            .len()
            .checked_add(events.len())
            .is_none_or(|total| total > MAX_PENDING_PI_ACTION_INTENTS)
            || events.iter().any(|event| {
                !matches!(
                    event.peer.category,
                    DispatchEventCategory::PiActionIntent(_)
                ) || event.peer.identity.runtime_id != binding.runtime_id
                    || event.peer.identity.workspace_id != binding.workspace_id
                    || event.peer.identity.process_generation != binding.process_generation
            })
        {
            return Err(RuntimeProductionError::InvalidPiActionIntent);
        }
        self.items.extend(
            events
                .into_iter()
                .map(PendingPiProductionAction::AwaitingEvaluation),
        );
        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn front_identity(&self) -> Option<DispatchIdentity> {
        self.items.front().map(|pending| match pending {
            PendingPiProductionAction::AwaitingEvaluation(event) => event.peer.identity.clone(),
            PendingPiProductionAction::AwaitingSettlement(settlement) => {
                settlement.identity.clone()
            }
        })
    }

    fn front_event(&self) -> Option<&DispatchEvent> {
        match self.items.front() {
            Some(PendingPiProductionAction::AwaitingEvaluation(event)) => Some(event),
            _ => None,
        }
    }

    fn store_settlement(
        &mut self,
        settlement: PiProductionGatewaySettlement,
    ) -> Result<(), RuntimeProductionError> {
        let Some(pending) = self.items.front_mut() else {
            return Err(RuntimeProductionError::InvalidPiActionIntent);
        };
        if !matches!(pending, PendingPiProductionAction::AwaitingEvaluation(_)) {
            return Err(RuntimeProductionError::InvalidPiActionIntent);
        }
        *pending = PendingPiProductionAction::AwaitingSettlement(settlement);
        Ok(())
    }

    fn front_settlement(&self) -> Option<&PiProductionGatewaySettlement> {
        match self.items.front() {
            Some(PendingPiProductionAction::AwaitingSettlement(settlement)) => Some(settlement),
            _ => None,
        }
    }

    fn complete_front(&mut self) {
        self.items.pop_front();
    }
}

impl PiProductionWorker {
    pub fn binding(&self) -> &ProductionRuntimeBinding {
        &self.binding
    }

    pub fn native_process_id(&self) -> Result<u32, RuntimeProductionError> {
        self.runner.with_runner(|runner| runner.process_id())
    }

    pub fn pending_action_approvals(&self) -> usize {
        self.gateway.pending_count()
    }

    pub fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
        self.gateway.pending_approval_descriptors()
    }

    pub(crate) fn pending_approval_identity(
        &self,
        correlation_id: &str,
    ) -> Option<DispatchIdentity> {
        self.gateway.pending_identity(correlation_id)
    }

    pub(crate) fn enqueue_action_intents(
        &mut self,
        events: Vec<DispatchEvent>,
    ) -> Result<(), RuntimeProductionError> {
        self.pending_actions.enqueue(&self.binding, events)
    }

    pub(crate) fn has_pending_action_intents(&self) -> bool {
        !self.pending_actions.is_empty()
    }

    pub(crate) fn next_pending_action_identity(&self) -> Option<DispatchIdentity> {
        self.pending_actions.front_identity()
    }

    pub(crate) fn evaluate_next_action_intent<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        authority: PiGatewayAuthorityContext,
        now_ms: u64,
    ) -> Result<(), RuntimeProductionError> {
        let Some(event) = self.pending_actions.front_event() else {
            return if self.pending_actions.front_settlement().is_some() {
                Ok(())
            } else {
                Err(RuntimeProductionError::InvalidPiActionIntent)
            };
        };
        let settlement =
            self.gateway
                .evaluate_action_intent(application, event, authority, now_ms)?;
        self.pending_actions.store_settlement(settlement)
    }

    pub(crate) fn settle_next_action_intent(
        &mut self,
        registry: &mut RuntimeDispatchRegistry,
    ) -> Result<PiProductionGatewayOutcome, RuntimeProductionError> {
        let settlement = self
            .pending_actions
            .front_settlement()
            .ok_or(RuntimeProductionError::InvalidPiActionIntent)?;
        let result = self.gateway.settle(registry, settlement);
        if result.is_ok() || matches!(result, Err(RuntimeProductionError::PiToolResolutionFailed)) {
            self.pending_actions.complete_front();
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn evaluate_action_approval<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        native_request_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        authority: PiGatewayAuthorityContext,
        now_ms: u64,
    ) -> Result<PiProductionGatewaySettlement, RuntimeProductionError> {
        self.gateway.evaluate_approval(
            application,
            native_request_id,
            prompt_id,
            answer,
            authority,
            now_ms,
        )
    }

    pub(crate) fn evaluate_action_approval_by_correlation<A: BrokerActionApplication>(
        &mut self,
        application: &mut A,
        correlation_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        authority: PiGatewayAuthorityContext,
        now_ms: u64,
    ) -> Result<PiProductionGatewaySettlement, RuntimeProductionError> {
        let native_request_id = self
            .gateway
            .pending
            .iter()
            .find(|(_, identity)| identity.correlation_id == correlation_id)
            .map(|(native_request_id, _)| native_request_id.clone())
            .ok_or(RuntimeProductionError::UnknownPiApproval)?;
        self.evaluate_action_approval(
            application,
            &native_request_id,
            prompt_id,
            answer,
            authority,
            now_ms,
        )
    }

    pub(crate) fn settle_action_approval(
        &mut self,
        registry: &mut RuntimeDispatchRegistry,
        settlement: PiProductionGatewaySettlement,
    ) -> Result<PiProductionGatewayOutcome, RuntimeProductionError> {
        self.gateway.settle(registry, &settlement)
    }

    /// Delivers a provider credential only through Pi's inherited anonymous
    /// descriptor. The caller supplies opaque identities; this worker obtains
    /// and consumes the operation lease from the Rust-owned vault.
    pub fn deliver_provider_credential(
        &self,
        credential_reference: &CredentialReference,
        metadata: &PiCredentialLeaseMetadata,
    ) -> Result<(), RuntimeProductionError> {
        let operation = format!("pi-provider-{}-{}", metadata.provider, metadata.lease_id);
        if !valid_id(&metadata.provider) || !valid_id(&metadata.lease_id) {
            return Err(RuntimeProductionError::InvalidCredentialBinding);
        }
        let lease = self.credential_vault.lease_for_operation(
            credential_reference,
            operation,
            PI_CREDENTIAL_LEASE_TTL,
        )?;
        self.runner
            .with_runner(|runner| runner.deliver_credential_lease(metadata, &lease))??;
        Ok(())
    }
}

fn materialize_opencode_provider_configuration(
    namespace: &StateNamespace,
    routes: &[ProductionProviderRoute],
) -> Result<(), RuntimeProductionError> {
    let mut providers = serde_json::Map::new();
    for route in routes {
        let profile = route.profile();
        profile.validate()?;
        let native_provider_id = profile.opencode_native_provider_id();
        if providers.contains_key(native_provider_id) {
            return Err(RuntimeProductionError::ProviderRouteUnavailable);
        }
        let mut models = serde_json::Map::new();
        for model in route.models().iter().filter(|model| model.is_usable()) {
            let input_modalities = opencode_modalities(
                &model.capabilities,
                &[
                    (CapabilityKey::InputText, "text"),
                    (CapabilityKey::InputAudio, "audio"),
                    (CapabilityKey::InputImage, "image"),
                    (CapabilityKey::InputVideo, "video"),
                    (CapabilityKey::InputPdf, "pdf"),
                ],
            );
            let output_modalities = opencode_modalities(
                &model.capabilities,
                &[
                    (CapabilityKey::OutputText, "text"),
                    (CapabilityKey::OutputAudio, "audio"),
                    (CapabilityKey::OutputImage, "image"),
                    (CapabilityKey::OutputVideo, "video"),
                ],
            );
            let attachment = input_modalities.iter().any(|modality| *modality != "text");
            let mut configuration = serde_json::Map::new();
            configuration.insert("id".into(), model.model_id.clone().into());
            configuration.insert("name".into(), model.display_name.clone().into());
            configuration.insert("attachment".into(), attachment.into());
            configuration.insert(
                "reasoning".into(),
                model
                    .capabilities
                    .feature_state(CapabilityKey::Reasoning)
                    .usable()
                    .into(),
            );
            configuration.insert(
                "temperature".into(),
                model
                    .capabilities
                    .feature_state(CapabilityKey::Temperature)
                    .usable()
                    .into(),
            );
            configuration.insert(
                "tool_call".into(),
                model
                    .capabilities
                    .feature_state(CapabilityKey::ToolCalling)
                    .usable()
                    .into(),
            );
            configuration.insert(
                "modalities".into(),
                serde_json::json!({
                    "input": input_modalities,
                    "output": output_modalities,
                }),
            );
            configuration.insert(
                "status".into(),
                match model.capabilities.lifecycle {
                    ModelLifecycle::Active => "active",
                    ModelLifecycle::Preview => "beta",
                    ModelLifecycle::Deprecated => "deprecated",
                    ModelLifecycle::Unavailable => continue,
                }
                .into(),
            );
            if let (Some(context), Some(output)) = (
                model
                    .capabilities
                    .numeric_maximum(NumericCapabilityKey::ContextTokens),
                model
                    .capabilities
                    .numeric_maximum(NumericCapabilityKey::OutputTokens),
            ) {
                configuration.insert(
                    "limit".into(),
                    serde_json::json!({ "context": context, "output": output }),
                );
            }
            if models
                .insert(
                    model.model_id.clone(),
                    serde_json::Value::Object(configuration),
                )
                .is_some()
            {
                return Err(RuntimeProductionError::ProviderRouteUnavailable);
            }
        }
        if !models.contains_key(route.selected_model_id()) {
            return Err(RuntimeProductionError::ProviderRouteUnavailable);
        }
        let mut configuration = serde_json::Map::new();
        configuration.insert(
            "id".into(),
            serde_json::Value::String(native_provider_id.into()),
        );
        configuration.insert(
            "name".into(),
            serde_json::Value::String(profile.display_name.clone()),
        );
        configuration.insert("models".into(), serde_json::Value::Object(models));
        configuration.insert(
            "options".into(),
            serde_json::json!({ "baseURL": profile.endpoint.base_url.clone() }),
        );
        configuration.insert(
            "npm".into(),
            serde_json::Value::String(route.opencode_sdk_npm()?.into()),
        );
        providers.insert(
            native_provider_id.into(),
            serde_json::Value::Object(configuration),
        );
    }
    if providers.is_empty() {
        return Ok(());
    }

    let opencode_directory = namespace.config_home().join("opencode");
    fs::create_dir_all(&opencode_directory).map_err(RuntimeProductionError::AssetIo)?;
    fs::set_permissions(&opencode_directory, fs::Permissions::from_mode(0o700))
        .map_err(RuntimeProductionError::AssetIo)?;
    let canonical_directory = opencode_directory
        .canonicalize()
        .map_err(RuntimeProductionError::AssetIo)?;
    let canonical_config_home = namespace
        .config_home()
        .canonicalize()
        .map_err(RuntimeProductionError::AssetIo)?;
    if !canonical_directory.starts_with(&canonical_config_home) {
        return Err(RuntimeProductionError::InvalidAssetRoot);
    }
    let encoded = serde_json::to_vec(&serde_json::json!({ "provider": providers }))
        .map_err(|_| RuntimeProductionError::ProviderRouteUnavailable)?;
    let path = canonical_directory.join("opencode.json");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(RuntimeProductionError::AssetIo)?;
    file.write_all(&encoded)
        .and_then(|_| file.sync_all())
        .map_err(RuntimeProductionError::AssetIo)
}

fn opencode_modalities(
    capabilities: &crate::runtime::capability::CapabilityDescriptor,
    candidates: &[(CapabilityKey, &'static str)],
) -> Vec<&'static str> {
    candidates
        .iter()
        .filter_map(|(key, modality)| {
            capabilities
                .feature_state(*key)
                .usable()
                .then_some(*modality)
        })
        .collect()
}

fn require_kind(
    binding: &ProductionRuntimeBinding,
    expected: RuntimeKind,
) -> Result<(), RuntimeProductionError> {
    if binding.runtime_kind != expected {
        return Err(RuntimeProductionError::RuntimeKindMismatch);
    }
    Ok(())
}

fn opencode_provider_npm(
    profile: &ProviderProfile,
) -> Result<&'static str, RuntimeProductionError> {
    match profile.kind {
        ProviderKind::OpenAi => Ok("@ai-sdk/openai"),
        ProviderKind::Anthropic => Ok("@ai-sdk/anthropic"),
        ProviderKind::Gemini => Ok("@ai-sdk/google"),
        ProviderKind::OpenRouter => Ok("@openrouter/ai-sdk-provider"),
        ProviderKind::Custom if profile.endpoint.api_kind == "openai-compatible" => {
            Ok("@ai-sdk/openai-compatible")
        }
        ProviderKind::Custom => Err(RuntimeProductionError::ProviderRouteUnavailable),
    }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, RuntimeProductionError> {
    let canonical = path
        .canonicalize()
        .map_err(RuntimeProductionError::AssetIo)?;
    if !canonical.is_dir() {
        return Err(RuntimeProductionError::InvalidAssetRoot);
    }
    Ok(canonical)
}

fn canonical_executable(path: &Path) -> Result<PathBuf, RuntimeProductionError> {
    let unresolved = fs::symlink_metadata(path).map_err(RuntimeProductionError::AssetIo)?;
    if !unresolved.file_type().is_file() || unresolved.file_type().is_symlink() {
        return Err(RuntimeProductionError::InvalidNodeExecutable);
    }
    let canonical = path
        .canonicalize()
        .map_err(RuntimeProductionError::AssetIo)?;
    let metadata = canonical
        .metadata()
        .map_err(RuntimeProductionError::AssetIo)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(RuntimeProductionError::InvalidNodeExecutable);
    }
    Ok(canonical)
}

fn verified_node_runtime(
    resource_root: &Path,
) -> Result<VerifiedNodeRuntime, RuntimeProductionError> {
    let metadata_path = resource_root.join("sidecars/node-runtime.json");
    let unresolved =
        fs::symlink_metadata(&metadata_path).map_err(RuntimeProductionError::AssetIo)?;
    if !unresolved.file_type().is_file()
        || unresolved.file_type().is_symlink()
        || unresolved.len() == 0
        || unresolved.len() > MAX_NODE_RUNTIME_METADATA_BYTES
    {
        return Err(RuntimeProductionError::InvalidNodeMetadata);
    }
    let metadata_path = metadata_path
        .canonicalize()
        .map_err(RuntimeProductionError::AssetIo)?;
    if !metadata_path.starts_with(resource_root) {
        return Err(RuntimeProductionError::InvalidNodeMetadata);
    }
    let metadata: NodeRuntimeMetadata = serde_json::from_reader(
        fs::File::open(metadata_path).map_err(RuntimeProductionError::AssetIo)?,
    )
    .map_err(|_| RuntimeProductionError::InvalidNodeMetadata)?;
    if metadata.schema_version != NODE_RUNTIME_METADATA_SCHEMA_VERSION
        || metadata.minimum_supported_version != NODE_MINIMUM_SUPPORTED_VERSION
        || parse_semver(&metadata.version).is_none()
        || parse_semver(&metadata.minimum_supported_version).is_none()
        || parse_semver(&metadata.version) < parse_semver(NODE_MINIMUM_SUPPORTED_VERSION)
        || !valid_sha256(&metadata.sha256)
        || metadata.executable_relative_path.is_absolute()
        || metadata
            .executable_relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(RuntimeProductionError::InvalidNodeMetadata);
    }
    let executable = canonical_executable(&resource_root.join(&metadata.executable_relative_path))?;
    if !executable.starts_with(resource_root)
        || sha256_file(&executable)? != metadata.sha256
        || !verify_node_version(&executable, &metadata.version)?
    {
        return Err(RuntimeProductionError::InvalidNodeMetadata);
    }
    Ok(VerifiedNodeRuntime {
        executable,
        sha256: metadata.sha256,
        version: metadata.version,
    })
}

fn verify_node_version(
    executable: &Path,
    expected_version: &str,
) -> Result<bool, RuntimeProductionError> {
    let mut child = Command::new(executable)
        .arg("--version")
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(RuntimeProductionError::AssetIo)?;
    let process_group_id = child.id();
    let deadline = Instant::now() + NODE_VERSION_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().map_err(RuntimeProductionError::AssetIo)? {
            if !status.success() {
                return Ok(false);
            }
            let mut output = Vec::new();
            child
                .stdout
                .take()
                .ok_or(RuntimeProductionError::InvalidNodeMetadata)?
                .take(MAX_NODE_VERSION_BYTES + 1)
                .read_to_end(&mut output)
                .map_err(RuntimeProductionError::AssetIo)?;
            return Ok(output.len() <= MAX_NODE_VERSION_BYTES as usize
                && String::from_utf8(output).ok().is_some_and(|version| {
                    version.trim().trim_start_matches('v') == expected_version
                }));
        }
        if Instant::now() >= deadline {
            if let Ok(process_group_id) = i32::try_from(process_group_id) {
                // SAFETY: the child owns this fresh process group and the
                // constant signal does not retain or dereference a pointer.
                unsafe { libc::kill(-process_group_id, libc::SIGKILL) };
            }
            let _ = child.wait();
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn parse_semver(value: &str) -> Option<(u64, u64, u64)> {
    let mut parts = value.split('.');
    let version = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(version)
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn valid_test_tls_loopback_url(value: &str) -> bool {
    let Some(authority_and_path) = value.strip_prefix("https://127.0.0.1:") else {
        return false;
    };
    let Some((port, path)) = authority_and_path.split_once('/') else {
        return false;
    };
    port.parse::<u16>().is_ok_and(|port| port != 0) && path == "v1"
}

fn valid_test_tls_ca_pem(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > MAX_TEST_TLS_CA_BYTES || bytes.contains(&0) {
        return false;
    }
    let Ok(pem) = std::str::from_utf8(bytes) else {
        return false;
    };
    pem.matches("-----BEGIN CERTIFICATE-----").count() == 1
        && pem.matches("-----END CERTIFICATE-----").count() == 1
        && pem.trim_start().starts_with("-----BEGIN CERTIFICATE-----")
        && pem.trim_end().ends_with("-----END CERTIFICATE-----")
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

#[derive(Debug, Error)]
pub enum RuntimeProductionError {
    #[error("production runtime binding is invalid")]
    InvalidBinding,
    #[error("production runtime kind does not match the requested peer")]
    RuntimeKindMismatch,
    #[error("production runtime asset root is invalid")]
    InvalidAssetRoot,
    #[error("production Node executable is invalid")]
    InvalidNodeExecutable,
    #[error("production Node runtime metadata is invalid")]
    InvalidNodeMetadata,
    #[error("production runtime assets changed after bootstrap")]
    AssetChanged,
    #[error("production OpenCode driver is unavailable")]
    DriverUnavailable,
    #[error("production Pi runner is unavailable")]
    RunnerUnavailable,
    #[error("production loopback endpoint could not be reserved")]
    LoopbackReservation(#[source] std::io::Error),
    #[error("production peer did not expose a native process identifier")]
    MissingNativeProcess,
    #[error("production peer has no selected provider credential route")]
    ProviderRouteUnavailable,
    #[error("production native-test TLS trust capability is invalid")]
    InvalidTestTlsTrustCapability,
    #[error("production Pi credential binding is invalid")]
    InvalidCredentialBinding,
    #[error("production Pi action intent does not match the active runtime binding")]
    InvalidPiActionIntent,
    #[error("production Pi approval does not match a pending action intent")]
    UnknownPiApproval,
    #[error("production Pi approval continuation changed its action binding")]
    PiApprovalBindingMismatch,
    #[error("production Pi tool resolution failed; the active run was cancelled")]
    PiToolResolutionFailed,
    #[error(
        "production Pi tool resolution and cancellation both failed; quarantine the process generation"
    )]
    PiToolSettlementGenerationFatal,
    #[error("production credential vault is unavailable; explicit fallback is required")]
    CredentialVaultUnavailable,
    #[error("production OpenCode stream bounds are invalid")]
    InvalidStreamBounds,
    #[error("production runtime asset I/O failed")]
    AssetIo(#[source] std::io::Error),
    #[error(transparent)]
    OpenCodeAssets(#[from] OpenCodeAssetError),
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),
    #[error(transparent)]
    Credential(#[from] crate::security::credentials::CredentialVaultError),
    #[error("OpenCode adapter composition failed: {0:?}")]
    OpenCodeAdapter(OpenCodeAdapterError),
    #[error("OpenCode native boundary composition failed: {0:?}")]
    NativeBoundary(NativeBoundaryError),
    #[error(transparent)]
    PiProcess(#[from] PiProcessError),
    #[error(transparent)]
    PiAdapter(#[from] PiAdapterError),
    #[error(transparent)]
    Dispatch(#[from] DispatchError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    CapabilityEvidence(#[from] crate::runtime::capability_evidence::CapabilityEvidenceError),
    #[error(transparent)]
    DispatchAuthority(#[from] DispatchAuthorityError),
    #[error("production provider probe failed: {0:?}")]
    ProviderProbe(ProviderProbeFailure),
    #[error("production broker context failed: {0}")]
    BrokerContext(BrokerContextRegistryError),
    #[error(transparent)]
    BrokerFacility(#[from] FacilityRegistryError),
    #[error(transparent)]
    BrokerWorker(#[from] BrokerWorkerError),
    #[error(transparent)]
    BrokerPump(#[from] OpenCodeBrokerPumpError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde_json::{Map, json};
    use tempfile::TempDir;

    use crate::core::database::{DatabaseActor, DatabaseDescriptor};
    use crate::runtime::action_bridge::{
        RuntimeActionBridge, RuntimeActionProposal, RuntimeApprovalDecision, RuntimeAuthorization,
        RuntimeExecutionReceipt, RuntimeGatewayDecision,
    };
    use crate::runtime::dispatch::{
        PeerDispatchError, PeerDispatchEvent, PeerDispatchRequest, RuntimeDispatchPeer,
        normalize_pi_event,
    };
    use crate::runtime::opencode::{C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL};
    use crate::runtime::opencode_broker::{InstalledBrokerClassification, InstalledBrokerFacility};
    use crate::runtime::pi::{PI_PROTOCOL_SCHEMA_VERSION, PiEventEnvelope, PiSidecarManifest};
    use crate::security::authorization::LiveAuthorityState;
    use crate::security::gateway::{ActionGateway, ExecutionPermit, NormalizedActionResult};
    use crate::security::policy::{
        ActionEffect, ActionReversibility, ActionScope, ActionSensitivity, ActionSurface,
        PolicyConfiguration, RepositoryState,
    };

    #[test]
    fn test_tls_capability_accepts_only_one_bounded_loopback_root() {
        let valid = b"-----BEGIN CERTIFICATE-----\nY2VydGlmaWNhdGU=\n-----END CERTIFICATE-----\n";
        assert!(valid_test_tls_loopback_url("https://127.0.0.1:44321/v1"));
        assert!(!valid_test_tls_loopback_url("https://localhost:44321/v1"));
        assert!(!valid_test_tls_loopback_url("https://127.0.0.1:44321/v2"));
        assert!(valid_test_tls_ca_pem(valid));
        assert!(!valid_test_tls_ca_pem(
            b"-----BEGIN CERTIFICATE-----\na\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\nb\n-----END CERTIFICATE-----\n"
        ));
        assert!(!valid_test_tls_ca_pem(&vec![
            b'a';
            MAX_TEST_TLS_CA_BYTES + 1
        ]));
    }

    #[test]
    fn production_opencode_broker_poll_is_bounded_to_the_app_driver_cadence() {
        assert_eq!(OPEN_CODE_BROKER_RECEIVE_TIMEOUT, Duration::from_millis(25));
        assert!(
            OPEN_CODE_BROKER_RECEIVE_TIMEOUT < OpenCodeBrokerPumpConfig::default().receive_timeout
        );
    }

    #[test]
    fn owned_loopback_credential_is_session_only_and_removed_on_revoke_and_drop() {
        let vault = CredentialVault::session_only().expect("session-only loopback vault");
        let reference = RandomSecretReference::new("loopback-test-reference", 256)
            .expect("random secret reference");
        let mut credential =
            OwnedLoopbackCredential::random(vault.clone(), reference).expect("loopback secret");
        assert_eq!(vault.metadata().unwrap().len(), 1);
        credential.revoke();
        credential.revoke();
        assert!(vault.metadata().unwrap().is_empty());

        let reference = RandomSecretReference::new("loopback-drop-reference", 256)
            .expect("random secret reference");
        let credential =
            OwnedLoopbackCredential::random(vault.clone(), reference).expect("loopback secret");
        assert_eq!(vault.metadata().unwrap().len(), 1);
        drop(credential);
        assert!(vault.metadata().unwrap().is_empty());
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct PiCredentialDeliveryObservation {
        metadata: PiCredentialLeaseMetadata,
        secret_sha256: [u8; 32],
    }

    struct CapturingPiCredentialDelivery {
        observations: Arc<Mutex<Vec<PiCredentialDeliveryObservation>>>,
    }

    impl PiCredentialLeaseDelivery for CapturingPiCredentialDelivery {
        fn deliver(
            &mut self,
            metadata: &PiCredentialLeaseMetadata,
            lease: &OperationCredentialLease,
        ) -> Result<(), PeerDispatchError> {
            let mut secret = Vec::new();
            lease
                .deliver_to(&mut secret)
                .map_err(|_| PeerDispatchError::Credential)?;
            let secret_sha256 = Sha256::digest(&secret).into();
            secret.fill(0);
            self.observations
                .lock()
                .unwrap()
                .push(PiCredentialDeliveryObservation {
                    metadata: metadata.clone(),
                    secret_sha256,
                });
            Ok(())
        }
    }

    #[test]
    fn only_core_input_can_mint_a_validated_runtime_binding() {
        let temporary = TempDir::new().expect("temporary binding root");
        let workspace = temporary.path().join("workspace");
        let c4os_home = temporary.path().join("c4os-home");
        fs::create_dir(&workspace).expect("workspace root");
        fs::create_dir(&c4os_home).expect("C4OS home");

        let binding = ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
            runtime_id: "opencode-production".into(),
            runtime_kind: RuntimeKind::OpenCode,
            workspace_id: "workspace-1".into(),
            workspace_root: workspace.clone(),
            c4os_home: c4os_home.clone(),
            process_generation: 41,
            launch_id: "launch-1".into(),
        })
        .expect("validated core binding");
        assert_eq!(binding.runtime_id(), "opencode-production");
        assert_eq!(binding.runtime_kind(), RuntimeKind::OpenCode);
        assert_eq!(binding.workspace_id(), "workspace-1");
        assert_eq!(binding.workspace_root(), workspace.canonicalize().unwrap());
        assert_eq!(binding.process_generation(), 41);

        assert!(matches!(
            ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
                runtime_id: "../runtime".into(),
                runtime_kind: RuntimeKind::OpenCode,
                workspace_id: "workspace-1".into(),
                workspace_root: workspace,
                c4os_home,
                process_generation: 0,
                launch_id: "launch-1".into(),
            }),
            Err(RuntimeProductionError::InvalidBinding)
        ));
    }

    #[test]
    fn pi_pending_actions_are_atomic_fifo_and_retain_evaluated_settlement() {
        let temporary = TempDir::new().expect("temporary production roots");
        let workspace = temporary.path().join("workspace");
        let c4os_home = temporary.path().join("c4os-home");
        fs::create_dir(&workspace).expect("workspace root");
        fs::create_dir(&c4os_home).expect("C4OS home");
        let binding = ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
            runtime_id: "pi-production".into(),
            runtime_kind: RuntimeKind::Pi,
            workspace_id: "workspace-1".into(),
            workspace_root: workspace,
            c4os_home,
            process_generation: 4,
            launch_id: "launch-1".into(),
        })
        .expect("Pi production binding");
        let identity = pi_dispatch_identity();
        let first = pi_action_event(identity.clone(), 1, "pi-tool-1", "workspace.summary");
        let second = pi_action_event(identity.clone(), 2, "pi-tool-2", "workspace.summary");
        let mut queue = PendingPiProductionActions::default();

        queue
            .enqueue(&binding, vec![first, second])
            .expect("enqueue complete durable batch");
        assert_eq!(queue.items.len(), 2);
        assert_eq!(queue.front_identity(), Some(identity.clone()));

        queue
            .store_settlement(PiProductionGatewaySettlement {
                identity: identity.clone(),
                outcome: RuntimeBrokerWorkerOutcome::Denied {
                    native_request_id: "pi-tool-1".into(),
                    reason_code: "policy-denied".into(),
                },
            })
            .expect("retain evaluated settlement");
        let Some(PiProductionGatewaySettlement {
            outcome:
                RuntimeBrokerWorkerOutcome::Denied {
                    native_request_id, ..
                },
            ..
        }) = queue.front_settlement()
        else {
            panic!("evaluated settlement must remain at the FIFO head")
        };
        assert_eq!(native_request_id, "pi-tool-1");
        assert!(matches!(
            queue.store_settlement(PiProductionGatewaySettlement {
                identity: identity.clone(),
                outcome: RuntimeBrokerWorkerOutcome::Denied {
                    native_request_id: "pi-tool-1".into(),
                    reason_code: "duplicate-evaluation".into(),
                },
            }),
            Err(RuntimeProductionError::InvalidPiActionIntent)
        ));
        assert_eq!(queue.items.len(), 2);

        queue.complete_front();
        let Some(next) = queue.front_event() else {
            panic!("second persisted intent must remain queued")
        };
        let DispatchEventCategory::PiActionIntent(next_intent) = &next.peer.category else {
            panic!("second queued event must retain its Pi action identity")
        };
        assert_eq!(next_intent.identity().native_request_id, "pi-tool-2");

        let mut invalid = next.clone();
        invalid.peer.category = DispatchEventCategory::Other;
        assert!(matches!(
            queue.enqueue(&binding, vec![invalid]),
            Err(RuntimeProductionError::InvalidPiActionIntent)
        ));
        assert_eq!(queue.items.len(), 1);
    }

    #[test]
    fn production_pi_gateway_completes_and_denies_through_the_registered_peer() {
        let temporary = TempDir::new().expect("temporary production roots");
        let workspace = temporary.path().join("workspace");
        let c4os_home = temporary.path().join("c4os-home");
        fs::create_dir(&workspace).expect("workspace root");
        fs::create_dir(&c4os_home).expect("C4OS home");
        let binding = ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
            runtime_id: "pi-production".into(),
            runtime_kind: RuntimeKind::Pi,
            workspace_id: "workspace-1".into(),
            workspace_root: workspace,
            c4os_home,
            process_generation: 4,
            launch_id: "launch-1".into(),
        })
        .expect("Pi production binding");
        let identity = pi_dispatch_identity();
        let event = pi_action_event(identity.clone(), 1, "pi-tool-1", "workspace.summary");

        let facilities = InstalledBrokerFacilityRegistry::new();
        let executions = Arc::new(AtomicUsize::new(0));
        facilities
            .install_resource(
                "workspace.summary",
                None,
                InstalledBrokerClassification {
                    surface: ActionSurface::C4os,
                    effects: BTreeSet::from([ActionEffect::Read]),
                    scope: ActionScope::Workspace,
                    sensitivity: ActionSensitivity::Ordinary,
                    reversibility: ActionReversibility::Reversible,
                    repository_state: RepositoryState::NotApplicable,
                    inside_active_project: true,
                    canonical_target: "c4os:workspace-summary".into(),
                    normalized_arguments: Map::new(),
                    trusted_root: true,
                    explicit_scope_grant: false,
                    sandbox_allows: true,
                    declaration_exceeded: false,
                },
                Box::new(TestFacility {
                    executions: Arc::clone(&executions),
                }),
            )
            .expect("installed read facility");
        let mut production =
            PiProductionGatewayWorker::attach(binding.clone(), facilities).expect("Pi gateway");
        let control = Arc::new(Mutex::new(TestPiPeerControl::default()));
        let mut registry = RuntimeDispatchRegistry::new();
        registry
            .register(TestPiPeer::new(Arc::clone(&control)))
            .expect("registered Pi peer");
        let mut application = TestGatewayApplication::new();

        let completed = production
            .evaluate_action_intent(&mut application, &event, pi_authority(), 10)
            .and_then(|settlement| production.settle(&mut registry, &settlement))
            .expect("completed Pi action");
        assert_eq!(
            completed,
            PiProductionGatewayOutcome::Completed {
                native_request_id: "pi-tool-1".into(),
                result_code: "facility-completed".into(),
            }
        );
        assert_eq!(executions.load(Ordering::SeqCst), 1);
        assert_eq!(control.lock().unwrap().completed, 1);
        assert_eq!(control.lock().unwrap().denied, 0);

        let mut restricted_authority = pi_authority();
        restricted_authority.eligible_tool_ids =
            BTreeSet::from([C4OS_ACTION_PROPOSAL_TOOL.to_owned()]);
        let excluded_event =
            pi_action_event(identity.clone(), 2, "pi-tool-excluded", "workspace.summary");
        let excluded = production
            .evaluate_action_intent(&mut application, &excluded_event, restricted_authority, 15)
            .expect("ineligible Pi tool settles as a denial");
        assert!(matches!(
            excluded.outcome,
            RuntimeBrokerWorkerOutcome::Denied {
                reason_code,
                ..
            } if reason_code == "unsupported-broker-tool"
        ));
        assert_eq!(
            executions.load(Ordering::SeqCst),
            1,
            "a tool omitted from the durable run snapshot must not execute"
        );

        let denied_binding = binding;
        let denied_control = Arc::new(Mutex::new(TestPiPeerControl::default()));
        let mut denied_registry = RuntimeDispatchRegistry::new();
        denied_registry
            .register(TestPiPeer::new(Arc::clone(&denied_control)))
            .expect("registered denied Pi peer");
        let mut denied_production = PiProductionGatewayWorker::attach(
            denied_binding,
            InstalledBrokerFacilityRegistry::new(),
        )
        .expect("fail-closed Pi gateway");
        let denied_event = pi_action_event(identity, 2, "pi-tool-2", "workspace.missing");
        let denied = denied_production
            .evaluate_action_intent(&mut application, &denied_event, pi_authority(), 20)
            .and_then(|settlement| denied_production.settle(&mut denied_registry, &settlement))
            .expect("unsupported Pi route resolved as denial");
        assert_eq!(
            denied,
            PiProductionGatewayOutcome::Denied {
                native_request_id: "pi-tool-2".into(),
                reason_code: "unsupported-broker-operation".into(),
            }
        );
        assert_eq!(denied_control.lock().unwrap().denied, 1);
        assert_eq!(denied_production.pending_count(), 0);
    }

    #[test]
    fn pi_credential_descriptor_key_is_bound_to_the_full_dispatch_operation() {
        assert_eq!(
            pi_credential_operation(&pi_dispatch_identity(), "provider-openai", "openai"),
            "pi-provider:01d169892782dbb9ee5101c7d006daff42630eabe2602db71e95ca16be186b6b"
        );
    }

    #[test]
    fn production_pi_credential_delivery_keeps_same_native_profiles_isolated() {
        let temporary = TempDir::new().expect("temporary production roots");
        let workspace = temporary.path().join("workspace");
        let c4os_home = temporary.path().join("c4os-home");
        fs::create_dir(&workspace).expect("workspace root");
        fs::create_dir(&c4os_home).expect("C4OS home");
        let binding = ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
            runtime_id: "pi-production".into(),
            runtime_kind: RuntimeKind::Pi,
            workspace_id: "workspace-1".into(),
            workspace_root: workspace,
            c4os_home,
            process_generation: 4,
            launch_id: "launch-1".into(),
        })
        .expect("Pi production binding");
        let vault = CredentialVault::session_only().expect("session-only credential vault");
        let secret_a = b"team-a-secret";
        let secret_b = b"team-b-secret";
        let credential_a = vault.store("provider-a", secret_a).unwrap();
        let credential_b = vault.store("provider-b", secret_b).unwrap();
        let observations = Arc::new(Mutex::new(Vec::new()));
        let mut issuer = ProductionPiCredentialIssuer {
            binding,
            credentials: BTreeMap::from([
                (
                    "openai-team-a".into(),
                    (
                        "openai".into(),
                        "https://api.openai.com/v1".into(),
                        credential_a.clone(),
                    ),
                ),
                (
                    "openai-team-b".into(),
                    (
                        "openai".into(),
                        "https://api.openai.com/v1".into(),
                        credential_b.clone(),
                    ),
                ),
            ]),
            delivery: Box::new(CapturingPiCredentialDelivery {
                observations: Arc::clone(&observations),
            }),
            credential_vault: vault,
        };
        let identity_a = pi_dispatch_identity();
        let mut identity_b = identity_a.clone();
        identity_b.session_id = "session-2".into();
        identity_b.turn_id = "turn-2".into();
        identity_b.attempt_id = "run-2".into();
        identity_b.correlation_id = "run-correlation-2".into();

        assert_eq!(
            issuer.native_provider_id("openai-team-a").unwrap(),
            "openai"
        );
        assert_eq!(
            issuer.native_provider_id("openai-team-b").unwrap(),
            "openai"
        );
        assert_eq!(
            issuer.base_url("openai-team-a").unwrap(),
            "https://api.openai.com/v1"
        );
        issuer
            .deliver_for_dispatch(&identity_a, "openai-team-a")
            .unwrap();
        issuer
            .deliver_for_dispatch(&identity_b, "openai-team-b")
            .unwrap();
        assert!(matches!(
            issuer.deliver_for_dispatch(&identity_b, "openai-team-missing"),
            Err(PeerDispatchError::Credential)
        ));

        let observations = observations.lock().unwrap();
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].metadata.provider, "openai");
        assert_eq!(observations[1].metadata.provider, "openai");
        assert_ne!(
            observations[0].metadata.lease_id,
            observations[1].metadata.lease_id
        );
        let expected_a: [u8; 32] = Sha256::digest(secret_a).into();
        let expected_b: [u8; 32] = Sha256::digest(secret_b).into();
        assert_eq!(observations[0].secret_sha256, expected_a);
        assert_eq!(observations[1].secret_sha256, expected_b);
        let debug = format!("{observations:?}");
        assert!(!debug.contains(credential_a.as_str()));
        assert!(!debug.contains(credential_b.as_str()));
        assert!(!debug.contains("team-a-secret"));
        assert!(!debug.contains("team-b-secret"));
    }

    #[test]
    fn production_pi_approval_allow_and_deny_settle_the_exact_native_tool() {
        let temporary = TempDir::new().expect("temporary production roots");
        let workspace = temporary.path().join("workspace");
        let c4os_home = temporary.path().join("c4os-home");
        fs::create_dir(&workspace).expect("workspace root");
        fs::create_dir(&c4os_home).expect("C4OS home");
        let binding = ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
            runtime_id: "pi-production".into(),
            runtime_kind: RuntimeKind::Pi,
            workspace_id: "workspace-1".into(),
            workspace_root: workspace,
            c4os_home,
            process_generation: 4,
            launch_id: "launch-1".into(),
        })
        .expect("Pi production binding");
        let identity = pi_dispatch_identity();
        let executions = Arc::new(AtomicUsize::new(0));

        for (tool_call_id, answer) in [
            ("pi-tool-allow", ApprovalAnswer::Allow),
            ("pi-tool-deny", ApprovalAnswer::Deny),
            (
                "pi-tool-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                ApprovalAnswer::Allow,
            ),
        ] {
            let facilities = InstalledBrokerFacilityRegistry::new();
            facilities
                .install_action(
                    "window.focus",
                    "main",
                    Map::new(),
                    InstalledBrokerClassification {
                        surface: ActionSurface::Desktop,
                        effects: BTreeSet::from([ActionEffect::Control]),
                        scope: ActionScope::Workspace,
                        sensitivity: ActionSensitivity::Ordinary,
                        reversibility: ActionReversibility::Reversible,
                        repository_state: RepositoryState::NotApplicable,
                        inside_active_project: true,
                        canonical_target: "desktop:main-window".into(),
                        normalized_arguments: Map::new(),
                        trusted_root: true,
                        explicit_scope_grant: false,
                        sandbox_allows: true,
                        declaration_exceeded: false,
                    },
                    Box::new(TestFacility {
                        executions: Arc::clone(&executions),
                    }),
                )
                .expect("installed action facility");
            let mut production =
                PiProductionGatewayWorker::attach(binding.clone(), facilities).expect("Pi gateway");
            let control = Arc::new(Mutex::new(TestPiPeerControl::default()));
            let mut registry = RuntimeDispatchRegistry::new();
            registry
                .register(TestPiPeer::new(Arc::clone(&control)))
                .expect("registered Pi peer");
            let mut application = TestGatewayApplication::new();
            let event = pi_action_proposal_event(identity.clone(), 1, tool_call_id);

            let pending = production
                .evaluate_action_intent(&mut application, &event, pi_authority(), 10)
                .and_then(|settlement| production.settle(&mut registry, &settlement))
                .expect("pending Pi approval");
            let PiProductionGatewayOutcome::PendingApproval {
                native_request_id,
                prompt_id,
            } = pending
            else {
                panic!("effectful Pi action must pause for approval")
            };
            assert_eq!(native_request_id, tool_call_id);
            assert_eq!(production.pending_count(), 1);

            let outcome = production
                .evaluate_approval(
                    &mut application,
                    &native_request_id,
                    &prompt_id,
                    answer,
                    pi_authority(),
                    20,
                )
                .and_then(|settlement| production.settle(&mut registry, &settlement))
                .expect("Pi approval settlement");
            assert_eq!(production.pending_count(), 0);
            match answer {
                ApprovalAnswer::Allow => {
                    assert!(matches!(
                        outcome,
                        PiProductionGatewayOutcome::Completed { .. }
                    ));
                    assert_eq!(control.lock().unwrap().completed, 1);
                }
                ApprovalAnswer::Deny => {
                    assert!(matches!(outcome, PiProductionGatewayOutcome::Denied { .. }));
                    assert_eq!(control.lock().unwrap().denied, 1);
                }
            }
        }
        assert_eq!(executions.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn pi_approval_binding_survives_resolution_failure_until_cancel_is_confirmed() {
        let temporary = TempDir::new().expect("temporary production roots");
        let workspace = temporary.path().join("workspace");
        let c4os_home = temporary.path().join("c4os-home");
        fs::create_dir(&workspace).expect("workspace root");
        fs::create_dir(&c4os_home).expect("C4OS home");
        let binding = ProductionRuntimeBinding::from_core(CoreRuntimeBinding {
            runtime_id: "pi-production".into(),
            runtime_kind: RuntimeKind::Pi,
            workspace_id: "workspace-1".into(),
            workspace_root: workspace,
            c4os_home,
            process_generation: 4,
            launch_id: "launch-1".into(),
        })
        .expect("Pi production binding");
        let identity = pi_dispatch_identity();
        let native_request_id = "pi-tool-pending".to_owned();
        let settlement = PiProductionGatewaySettlement {
            identity: identity.clone(),
            outcome: RuntimeBrokerWorkerOutcome::Denied {
                native_request_id: native_request_id.clone(),
                reason_code: "approval-denied".into(),
            },
        };
        let mut production = PiProductionGatewayWorker::attach(
            binding.clone(),
            InstalledBrokerFacilityRegistry::new(),
        )
        .expect("Pi gateway");
        production
            .pending
            .insert(native_request_id.clone(), identity.clone());
        let control = Arc::new(Mutex::new(TestPiPeerControl {
            fail_denied: true,
            ..TestPiPeerControl::default()
        }));
        let mut registry = RuntimeDispatchRegistry::new();
        registry
            .register(TestPiPeer::new(Arc::clone(&control)))
            .expect("registered Pi peer");

        assert!(matches!(
            production.settle(&mut registry, &settlement),
            Err(RuntimeProductionError::PiToolResolutionFailed)
        ));
        assert_eq!(production.pending_count(), 0);
        assert_eq!(control.lock().unwrap().cancellations, 1);

        let fatal_settlement = PiProductionGatewaySettlement {
            identity: identity.clone(),
            outcome: RuntimeBrokerWorkerOutcome::Denied {
                native_request_id: native_request_id.clone(),
                reason_code: "approval-denied".into(),
            },
        };
        let mut fatal_production = PiProductionGatewayWorker::attach(
            binding.clone(),
            InstalledBrokerFacilityRegistry::new(),
        )
        .expect("Pi gateway");
        fatal_production
            .pending
            .insert(native_request_id.clone(), identity.clone());
        let fatal_control = Arc::new(Mutex::new(TestPiPeerControl {
            fail_denied: true,
            fail_cancel: true,
            ..TestPiPeerControl::default()
        }));
        let mut fatal_registry = RuntimeDispatchRegistry::new();
        fatal_registry
            .register(TestPiPeer::new(Arc::clone(&fatal_control)))
            .expect("registered Pi peer");

        assert!(matches!(
            fatal_production.settle(&mut fatal_registry, &fatal_settlement),
            Err(RuntimeProductionError::PiToolSettlementGenerationFatal)
        ));
        assert_eq!(fatal_production.pending_count(), 1);
        assert_eq!(fatal_control.lock().unwrap().cancellations, 1);

        let false_cancel_settlement = PiProductionGatewaySettlement {
            identity: identity.clone(),
            outcome: RuntimeBrokerWorkerOutcome::Denied {
                native_request_id: native_request_id.clone(),
                reason_code: "approval-denied".into(),
            },
        };
        let mut false_cancel_production =
            PiProductionGatewayWorker::attach(binding, InstalledBrokerFacilityRegistry::new())
                .expect("Pi gateway");
        false_cancel_production
            .pending
            .insert(native_request_id.clone(), identity);
        let false_cancel_control = Arc::new(Mutex::new(TestPiPeerControl {
            fail_denied: true,
            cancel_returns_false: true,
            ..TestPiPeerControl::default()
        }));
        let mut false_cancel_registry = RuntimeDispatchRegistry::new();
        false_cancel_registry
            .register(TestPiPeer::new(Arc::clone(&false_cancel_control)))
            .expect("registered Pi peer");

        assert!(matches!(
            false_cancel_production.settle(&mut false_cancel_registry, &false_cancel_settlement,),
            Err(RuntimeProductionError::PiToolSettlementGenerationFatal)
        ));
        assert_eq!(false_cancel_production.pending_count(), 1);
        assert_eq!(false_cancel_control.lock().unwrap().cancellations, 1);
    }

    fn pi_dispatch_identity() -> DispatchIdentity {
        DispatchIdentity {
            workspace_id: "workspace-1".into(),
            environment_id: "local".into(),
            session_id: "session-1".into(),
            turn_id: "turn-1".into(),
            attempt_id: "run-1".into(),
            correlation_id: "run-correlation-1".into(),
            runtime_id: "pi-production".into(),
            runtime_kind: RuntimeKind::Pi,
            adapter_version: "1.0.0".into(),
            native_version: crate::runtime::pi::PI_NATIVE_VERSION.into(),
            process_generation: 4,
        }
    }

    fn pi_action_event(
        identity: DispatchIdentity,
        sequence: u64,
        tool_call_id: &str,
        resource: &str,
    ) -> DispatchEvent {
        let peer = normalize_pi_event(
            identity.clone(),
            PiEventEnvelope {
                schema_version: PI_PROTOCOL_SCHEMA_VERSION,
                kind: "event".into(),
                event_id: format!("pi-event-{sequence}"),
                correlation_id: identity.correlation_id.clone(),
                process_generation: identity.process_generation,
                sequence,
                runtime: "pi".into(),
                workspace_id: identity.workspace_id.clone(),
                session_id: identity.session_id.clone(),
                turn_id: identity.turn_id.clone(),
                run_id: identity.attempt_id.clone(),
                category: "tool.action_intent".into(),
                native_type: "beforeToolCall".into(),
                tool_call_id: Some(tool_call_id.into()),
                payload: json!({
                    "tool": C4OS_RESOURCE_READ_TOOL,
                    "arguments": { "resource": resource },
                    "authority": "c4os-action-gateway-required",
                }),
            },
            10,
        )
        .expect("normalized Pi action event");
        DispatchEvent { sequence, peer }
    }

    fn pi_action_proposal_event(
        identity: DispatchIdentity,
        sequence: u64,
        tool_call_id: &str,
    ) -> DispatchEvent {
        let peer = normalize_pi_event(
            identity.clone(),
            PiEventEnvelope {
                schema_version: PI_PROTOCOL_SCHEMA_VERSION,
                kind: "event".into(),
                event_id: format!("pi-action-event-{sequence}"),
                correlation_id: identity.correlation_id.clone(),
                process_generation: identity.process_generation,
                sequence,
                runtime: "pi".into(),
                workspace_id: identity.workspace_id.clone(),
                session_id: identity.session_id.clone(),
                turn_id: identity.turn_id.clone(),
                run_id: identity.attempt_id.clone(),
                category: "tool.action_intent".into(),
                native_type: "beforeToolCall".into(),
                tool_call_id: Some(tool_call_id.into()),
                payload: json!({
                    "tool": C4OS_ACTION_PROPOSAL_TOOL,
                    "arguments": { "operation": "window.focus", "target": "main" },
                    "authority": "c4os-action-gateway-required",
                }),
            },
            10,
        )
        .expect("normalized Pi action proposal event");
        DispatchEvent { sequence, peer }
    }

    fn pi_authority() -> PiGatewayAuthorityContext {
        PiGatewayAuthorityContext {
            request_origin: ActionRequestOrigin::NaturalLanguageChat,
            configuration_version: 7,
            policy_version: 9,
            revocation_epoch: 2,
            eligible_tool_ids: OPENCODE_C4OS_TOOL_IDS
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }

    struct TestGatewayApplication {
        _temporary: TempDir,
        gateway: ActionGateway,
    }

    impl TestGatewayApplication {
        fn new() -> Self {
            let temporary = TempDir::new().expect("temporary gateway database");
            let (database, _) =
                DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("database");
            Self {
                _temporary: temporary,
                gateway: ActionGateway::new(PolicyConfiguration::default(), Arc::new(database)),
            }
        }
    }

    impl BrokerActionApplication for TestGatewayApplication {
        type Error = ();

        fn propose_runtime_action(
            &mut self,
            proposal: RuntimeActionProposal,
            now_ms: u64,
        ) -> Result<RuntimeGatewayDecision, Self::Error> {
            RuntimeActionBridge::new(&mut self.gateway)
                .propose(proposal, now_ms)
                .map_err(|_| ())
        }

        fn answer_runtime_approval(
            &mut self,
            prompt_id: &str,
            answer: ApprovalAnswer,
            now_ms: u64,
        ) -> Result<RuntimeApprovalDecision, Self::Error> {
            RuntimeActionBridge::new(&mut self.gateway)
                .answer_approval(prompt_id, answer, now_ms)
                .map_err(|_| ())
        }

        fn execute_runtime_action<F>(
            &mut self,
            authorization: RuntimeAuthorization,
            live: LiveAuthorityState,
            now_ms: u64,
            effect: F,
        ) -> Result<RuntimeExecutionReceipt, Self::Error>
        where
            F: FnOnce(ExecutionPermit) -> NormalizedActionResult,
        {
            RuntimeActionBridge::new(&mut self.gateway)
                .execute(authorization, live, now_ms, effect)
                .map_err(|_| ())
        }

        fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
            self.gateway
                .cancel_run(run_id, now_ms)
                .map(|_| ())
                .map_err(|_| ())
        }
    }

    struct TestFacility {
        executions: Arc<AtomicUsize>,
    }

    impl InstalledBrokerFacility for TestFacility {
        fn current_target_version(&mut self) -> Option<String> {
            Some("workspace-generation:1".into())
        }

        fn execute(&mut self, _permit: ExecutionPermit) -> NormalizedActionResult {
            self.executions.fetch_add(1, Ordering::SeqCst);
            NormalizedActionResult {
                status: NormalizedActionStatus::Succeeded,
                result_code: "facility-completed".into(),
                exit_code: None,
                changed_targets: Vec::new(),
                output_sha256: Some(format!("sha256:{:064x}", 1)),
                completed_at_ms: 21,
            }
        }
    }

    #[derive(Default)]
    struct TestPiPeerControl {
        completed: usize,
        denied: usize,
        cancellations: usize,
        fail_completed: bool,
        fail_denied: bool,
        fail_cancel: bool,
        cancel_returns_false: bool,
    }

    struct TestPiPeer {
        registration: RuntimePeerRegistration,
        control: Arc<Mutex<TestPiPeerControl>>,
    }

    impl TestPiPeer {
        fn new(control: Arc<Mutex<TestPiPeerControl>>) -> Self {
            let sidecar_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
            let manifest = PiSidecarManifest::load(&sidecar_root).expect("Pi manifest");
            let descriptor = manifest
                .conformance_descriptor(4, true)
                .expect("Pi descriptor");
            Self {
                registration: RuntimePeerRegistration {
                    runtime_id: "pi-production".into(),
                    workspace_id: "workspace-1".into(),
                    descriptor,
                },
                control,
            }
        }
    }

    impl RuntimeDispatchPeer for TestPiPeer {
        fn registration(&self) -> &RuntimePeerRegistration {
            &self.registration
        }

        fn readiness(&self) -> Result<(), PeerDispatchError> {
            Ok(())
        }

        fn create_session(
            &mut self,
            _request: &PeerDispatchRequest,
        ) -> Result<(), PeerDispatchError> {
            Ok(())
        }

        fn dispatch(&mut self, _request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
            Ok(())
        }

        fn poll_events(
            &mut self,
            _recorded_at_ms: u64,
        ) -> Result<Vec<PeerDispatchEvent>, PeerDispatchError> {
            Ok(Vec::new())
        }

        fn cancel(&mut self, _identity: &DispatchIdentity) -> Result<bool, PeerDispatchError> {
            let mut control = self.control.lock().unwrap();
            control.cancellations += 1;
            if control.fail_cancel {
                Err(PeerDispatchError::Cancellation)
            } else {
                Ok(!control.cancel_returns_false)
            }
        }

        fn resolve_pi_denied(
            &mut self,
            _identity: &DispatchIdentity,
            _native_request_id: &str,
            _reason_code: &str,
        ) -> Result<(), PeerDispatchError> {
            let mut control = self.control.lock().unwrap();
            control.denied += 1;
            if control.fail_denied {
                Err(PeerDispatchError::UnsupportedToolResolution)
            } else {
                Ok(())
            }
        }

        fn resolve_pi_completed(
            &mut self,
            _identity: &DispatchIdentity,
            _native_request_id: &str,
            receipt: &RuntimeExecutionReceipt,
        ) -> Result<(), PeerDispatchError> {
            assert_eq!(receipt.result().status, NormalizedActionStatus::Succeeded);
            let mut control = self.control.lock().unwrap();
            control.completed += 1;
            if control.fail_completed {
                Err(PeerDispatchError::UnsupportedToolResolution)
            } else {
                Ok(())
            }
        }
    }
}
