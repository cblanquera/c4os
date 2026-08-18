//! Production capability evidence producers and the C4OS-owned layer registry.
//!
//! Provider declarations, adapter normalization, and runtime observation enter
//! through distinct typed producers. The registry cannot relabel a descriptor
//! from one layer as another, and it returns exactly the three inputs required
//! by `capability::effective_intersection`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::runtime::{
    adapter::AdapterConformanceDescriptor,
    capability::{
        CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
        CapabilityLayer, CapabilityState, LimitConfidence, ModelLifecycle,
        NumericCapabilityEvidence, NumericCapabilityKey, RouteIdentity, effective_intersection,
    },
    opencode::{
        HealthSnapshot as OpenCodeHealth, OPENCODE_MAX_ATTACHMENTS,
        OPENCODE_MAX_INLINE_ATTACHMENT_BYTES, OPENCODE_NATIVE_VERSION,
    },
    pi::{
        PI_MAX_ATTACHMENTS, PI_MAX_IMAGE_BYTES, PI_NATIVE_VERSION, PI_PROTOCOL, PiHealth,
        PiModelRoute,
    },
    provider::{ModelRoute, ProviderModelDeclaration},
    supervisor::RuntimeKind,
};

pub const MAX_EVIDENCE_ROUTES: usize = 512;
pub const MAX_HISTORICAL_EVIDENCE_ROUTES: usize = 32;
const MAX_RESTORABLE_HISTORICAL_EVIDENCE_ROUTES: usize = 4_096;
const MAX_EVIDENCE_VALUES: usize = 256;
const MAX_HEALTH_OBSERVATION_AGE_MS: u64 = 60_000;

const FEATURE_KEYS: [CapabilityKey; 26] = [
    CapabilityKey::InputText,
    CapabilityKey::InputImage,
    CapabilityKey::InputAudio,
    CapabilityKey::InputVideo,
    CapabilityKey::InputPdf,
    CapabilityKey::OutputText,
    CapabilityKey::OutputImage,
    CapabilityKey::OutputAudio,
    CapabilityKey::OutputVideo,
    CapabilityKey::Streaming,
    CapabilityKey::Reasoning,
    CapabilityKey::ReasoningSummary,
    CapabilityKey::ToolCalling,
    CapabilityKey::ParallelToolCalling,
    CapabilityKey::StrictToolSchema,
    CapabilityKey::StreamedToolArguments,
    CapabilityKey::StructuredJson,
    CapabilityKey::StructuredJsonSchema,
    CapabilityKey::Temperature,
    CapabilityKey::TopP,
    CapabilityKey::TopK,
    CapabilityKey::StopSequences,
    CapabilityKey::Seed,
    CapabilityKey::Verbosity,
    CapabilityKey::PromptCaching,
    CapabilityKey::SessionAffinity,
];

const NUMERIC_KEYS: [NumericCapabilityKey; 5] = [
    NumericCapabilityKey::ContextTokens,
    NumericCapabilityKey::InputTokens,
    NumericCapabilityKey::OutputTokens,
    NumericCapabilityKey::AttachmentBytes,
    NumericCapabilityKey::AttachmentCount,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderDeclaredCatalogClaim {
    pub route: RouteIdentity,
    pub lifecycle: ModelLifecycle,
    pub declared_at_ms: u64,
    pub expires_at_ms: u64,
    pub features: BTreeMap<CapabilityKey, FeatureClaim>,
    pub numeric_limits: BTreeMap<NumericCapabilityKey, NumericClaim>,
    pub raw_catalog_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FeatureClaim {
    pub state: CapabilityState,
    pub constraints: Vec<String>,
    pub allowed_values: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumericClaim {
    pub state: CapabilityState,
    pub maximum: Option<u64>,
    pub confidence: LimitConfidence,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRouteObservation {
    pub runtime_id: String,
    pub route: RouteIdentity,
    pub process_generation: u64,
    pub health_checked_at_ms: u64,
    pub observed_at_ms: u64,
    pub expires_at_ms: u64,
    pub outcome: RuntimeObservationOutcome,
    pub lifecycle: ModelLifecycle,
    pub features: BTreeMap<CapabilityKey, FeatureClaim>,
    pub numeric_limits: BTreeMap<NumericCapabilityKey, NumericClaim>,
    pub raw_observation_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeObservationOutcome {
    Available,
    Degraded,
    Failed,
}

#[derive(Clone, Debug)]
pub struct DeclaredEvidence(CapabilityDescriptor);

#[derive(Clone, Debug)]
pub struct AdapterNormalizedEvidence(CapabilityDescriptor);

#[derive(Clone, Debug)]
pub struct ObservedEvidence {
    runtime_id: String,
    process_generation: u64,
    descriptor: CapabilityDescriptor,
}

impl DeclaredEvidence {
    pub fn descriptor(&self) -> &CapabilityDescriptor {
        &self.0
    }
}

impl AdapterNormalizedEvidence {
    pub fn descriptor(&self) -> &CapabilityDescriptor {
        &self.0
    }
}

impl ObservedEvidence {
    pub fn descriptor(&self) -> &CapabilityDescriptor {
        &self.descriptor
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn process_generation(&self) -> u64 {
        self.process_generation
    }
}

/// Produces provider-declared evidence from the catalog artifact captured at
/// provider ingestion. Missing fields become explicit Unknown evidence; they
/// are never inferred from adapter or runtime behavior.
pub fn provider_declared_evidence(
    claim: &ProviderDeclaredCatalogClaim,
) -> Result<DeclaredEvidence, CapabilityEvidenceError> {
    validate_claim_envelope(
        &claim.route,
        claim.declared_at_ms,
        claim.expires_at_ms,
        &claim.raw_catalog_sha256,
        claim.features.len(),
        claim.numeric_limits.len(),
    )?;
    let mut features = BTreeMap::new();
    let source = "provider.catalog";
    for key in FEATURE_KEYS {
        features.insert(
            key,
            claim.features.get(&key).map_or_else(
                || {
                    unknown_evidence(
                        CapabilityLayer::Declared,
                        source,
                        claim.declared_at_ms,
                        claim.expires_at_ms,
                        "provider catalog did not declare this feature",
                    )
                },
                |feature| {
                    evidence_from_claim(
                        CapabilityLayer::Declared,
                        source,
                        claim.declared_at_ms,
                        claim.expires_at_ms,
                        feature,
                        "provider catalog reports this feature without support",
                    )
                },
            ),
        );
    }
    let mut numeric_limits = BTreeMap::new();
    for key in NUMERIC_KEYS {
        numeric_limits.insert(
            key,
            claim.numeric_limits.get(&key).map_or_else(
                || {
                    unknown_numeric(
                        CapabilityLayer::Declared,
                        source,
                        claim.declared_at_ms,
                        claim.expires_at_ms,
                        "provider catalog did not declare this limit",
                    )
                },
                |numeric| {
                    numeric_from_claim(
                        CapabilityLayer::Declared,
                        source,
                        claim.declared_at_ms,
                        claim.expires_at_ms,
                        numeric,
                        "provider catalog reports this limit without confirmation",
                    )
                },
            ),
        );
    }
    let descriptor = CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Declared,
        route: claim.route.clone(),
        lifecycle: claim.lifecycle,
        features,
        numeric_limits,
        raw_evidence_sha256: claim.raw_catalog_sha256.clone(),
    };
    validate_descriptor(&descriptor, CapabilityLayer::Declared)?;
    Ok(DeclaredEvidence(descriptor))
}

/// Projects one runtime-neutral provider declaration onto an exact adapter
/// route without relabelling adapter or runtime observations as provider
/// truth.
pub fn provider_model_declared_evidence(
    declaration: &ProviderModelDeclaration,
    route: RouteIdentity,
) -> Result<DeclaredEvidence, CapabilityEvidenceError> {
    if route_model_id(&route) != Some(declaration.provider_model_id.as_str()) {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    let features = declaration
        .features
        .iter()
        .map(|(key, claim)| {
            (
                *key,
                FeatureClaim {
                    state: claim.state,
                    constraints: claim.constraints.clone(),
                    allowed_values: claim.allowed_values.clone(),
                    reason: claim.reason.clone(),
                },
            )
        })
        .collect();
    let numeric_limits = declaration
        .numeric_limits
        .iter()
        .map(|(key, claim)| {
            (
                *key,
                NumericClaim {
                    state: claim.state,
                    maximum: claim.maximum,
                    confidence: claim.confidence,
                    reason: claim.reason.clone(),
                },
            )
        })
        .collect();
    provider_declared_evidence(&ProviderDeclaredCatalogClaim {
        route,
        lifecycle: declaration.lifecycle,
        declared_at_ms: declaration.declared_at_ms,
        expires_at_ms: declaration.expires_at_ms,
        features,
        numeric_limits,
        raw_catalog_sha256: declaration.raw_catalog_sha256.clone(),
    })
}

/// Produces adapter-normalized evidence from the real OpenCode provider route.
/// Existing adapter evidence remains adapter-normalized; omitted fields are
/// explicit Unknowns owned by C4OS rather than copied provider declarations.
pub fn opencode_adapter_evidence(
    model_route: &ModelRoute,
) -> Result<AdapterNormalizedEvidence, CapabilityEvidenceError> {
    let descriptor = &model_route.capabilities;
    validate_descriptor(descriptor, CapabilityLayer::AdapterNormalized)?;
    if descriptor.route.adapter_kind != "opencode"
        || descriptor.route.adapter_version != "1.0.0"
        || descriptor.route.runtime_kind != "opencode"
        || descriptor.route.native_runtime_version != OPENCODE_NATIVE_VERSION
        || model_route.checked_at_ms == 0
        || descriptor
            .features
            .values()
            .any(|evidence| evidence.checked_at_ms != model_route.checked_at_ms)
        || descriptor
            .numeric_limits
            .values()
            .any(|numeric| numeric.evidence.checked_at_ms != model_route.checked_at_ms)
        || route_model_id(&descriptor.route) != Some(model_route.model_id.as_str())
    {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    let expires_at_ms = earliest_expiration(descriptor)?;
    let mut normalized = descriptor.clone();
    // C4OS verifies image and PDF bytes before serializing exact native file
    // parts. This narrows an already-usable OpenCode model claim; it never
    // upgrades an Unknown or Unsupported model route.
    for (key, allowed_value) in [
        (CapabilityKey::InputImage, "image/png"),
        (CapabilityKey::InputPdf, "application/pdf"),
    ] {
        if let Some(evidence) = normalized.features.get_mut(&key)
            && evidence.state.usable()
        {
            evidence.source = "c4os.opencode.verified-file-part".into();
            evidence.constraints = vec![format!(
                "Total attachment content is limited to {OPENCODE_MAX_INLINE_ATTACHMENT_BYTES} bytes"
            )];
            evidence.allowed_values = vec![allowed_value.into()];
        }
    }
    // Audio and video file ingestion have no executable boundary evidence in
    // this adapter revision and therefore remain explicitly unavailable.
    for key in [CapabilityKey::InputAudio, CapabilityKey::InputVideo] {
        normalized.features.insert(
            key,
            CapabilityEvidence {
                state: CapabilityState::Unsupported,
                layer: CapabilityLayer::AdapterNormalized,
                source: "c4os.opencode.unverified-file-modality".into(),
                checked_at_ms: model_route.checked_at_ms,
                expires_at_ms: Some(expires_at_ms),
                constraints: Vec::new(),
                allowed_values: Vec::new(),
                reason: Some(
                    "This OpenCode adapter revision has no verified audio or video file-part path"
                        .into(),
                ),
            },
        );
    }
    normalized.numeric_limits.insert(
        NumericCapabilityKey::AttachmentBytes,
        confirmed_adapter_limit(
            "c4os.opencode.verified-file-part",
            model_route.checked_at_ms,
            expires_at_ms,
            OPENCODE_MAX_INLINE_ATTACHMENT_BYTES,
        ),
    );
    normalized.numeric_limits.insert(
        NumericCapabilityKey::AttachmentCount,
        confirmed_adapter_limit(
            "c4os.opencode.verified-file-part",
            model_route.checked_at_ms,
            expires_at_ms,
            u64::try_from(OPENCODE_MAX_ATTACHMENTS)
                .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?,
        ),
    );
    fill_unknown_features(
        &mut normalized,
        CapabilityLayer::AdapterNormalized,
        "c4os.adapter-omission",
        model_route.checked_at_ms,
        expires_at_ms,
        "OpenCode adapter did not normalize this feature",
    );
    fill_unknown_numeric(
        &mut normalized,
        CapabilityLayer::AdapterNormalized,
        "c4os.adapter-omission",
        model_route.checked_at_ms,
        expires_at_ms,
        "OpenCode adapter did not normalize this limit",
    );
    normalized.raw_evidence_sha256 = digest_json(&(model_route, descriptor))?;
    validate_descriptor(&normalized, CapabilityLayer::AdapterNormalized)?;
    Ok(AdapterNormalizedEvidence(normalized))
}

/// Normalizes the exact Pi 0.80.10 agent contract and the subset of its pinned
/// model catalog that C4OS has verified. Provider-declared model truth remains
/// a separate, required layer; unknown catalog routes stay fail-closed for
/// numeric preflight limits.
pub fn pi_adapter_evidence(
    model_route: &PiModelRoute,
    route: &RouteIdentity,
    checked_at_ms: u64,
    expires_at_ms: u64,
) -> Result<AdapterNormalizedEvidence, CapabilityEvidenceError> {
    validate_claim_envelope(
        route,
        checked_at_ms,
        expires_at_ms,
        &digest_json(model_route)?,
        0,
        0,
    )?;
    if route.adapter_kind != "pi"
        || route.adapter_version != "1.0.0"
        || route.runtime_kind != "pi"
        || route.native_runtime_version != PI_NATIVE_VERSION
        || route.provider_model_id.split_once('/')
            != Some((model_route.provider.as_str(), model_route.model_id.as_str()))
    {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    let mut descriptor = CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::AdapterNormalized,
        route: route.clone(),
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::new(),
        numeric_limits: BTreeMap::new(),
        raw_evidence_sha256: digest_json(model_route)?,
    };
    for key in [
        CapabilityKey::InputText,
        CapabilityKey::OutputText,
        CapabilityKey::Streaming,
        CapabilityKey::ToolCalling,
    ] {
        descriptor.features.insert(
            key,
            CapabilityEvidence {
                state: CapabilityState::Supported,
                layer: CapabilityLayer::AdapterNormalized,
                source: "c4os.pi-0.80.10.agent-contract".into(),
                checked_at_ms,
                expires_at_ms: Some(expires_at_ms),
                constraints: vec![
                    "Only C4OS-installed broker tools are exposed to the Pi agent".into(),
                ],
                allowed_values: Vec::new(),
                reason: None,
            },
        );
    }
    descriptor.features.insert(
        CapabilityKey::InputImage,
        CapabilityEvidence {
            state: CapabilityState::Supported,
            layer: CapabilityLayer::AdapterNormalized,
            source: "c4os.pi.verified-image-input".into(),
            checked_at_ms,
            expires_at_ms: Some(expires_at_ms),
            constraints: vec![format!(
                "Aggregate image content is limited to {PI_MAX_IMAGE_BYTES} bytes"
            )],
            allowed_values: ["image/jpeg", "image/png", "image/gif", "image/webp"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            reason: None,
        },
    );
    for (key, reason) in [
        (
            CapabilityKey::InputAudio,
            "Pi 0.80.10 exact attachment input accepts images, not audio",
        ),
        (
            CapabilityKey::InputVideo,
            "Pi 0.80.10 exact attachment input accepts images, not video",
        ),
        (
            CapabilityKey::InputPdf,
            "Pi 0.80.10 exact attachment input accepts images, not PDF",
        ),
    ] {
        descriptor.features.insert(
            key,
            CapabilityEvidence {
                state: CapabilityState::Unsupported,
                layer: CapabilityLayer::AdapterNormalized,
                source: "c4os.pi.image-only-input".into(),
                checked_at_ms,
                expires_at_ms: Some(expires_at_ms),
                constraints: Vec::new(),
                allowed_values: Vec::new(),
                reason: Some(reason.into()),
            },
        );
    }
    descriptor.numeric_limits.insert(
        NumericCapabilityKey::AttachmentBytes,
        confirmed_adapter_limit(
            "c4os.pi.verified-image-input",
            checked_at_ms,
            expires_at_ms,
            PI_MAX_IMAGE_BYTES,
        ),
    );
    descriptor.numeric_limits.insert(
        NumericCapabilityKey::AttachmentCount,
        confirmed_adapter_limit(
            "c4os.pi.verified-image-input",
            checked_at_ms,
            expires_at_ms,
            u64::try_from(PI_MAX_ATTACHMENTS)
                .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?,
        ),
    );
    if model_route.provider == "openai" && model_route.model_id == "gpt-4o-mini" {
        for (key, maximum) in [
            (NumericCapabilityKey::ContextTokens, 128_000),
            (NumericCapabilityKey::OutputTokens, 16_384),
        ] {
            descriptor.numeric_limits.insert(
                key,
                confirmed_adapter_limit(
                    "c4os.pi-0.80.10.catalog.openai.gpt-4o-mini",
                    checked_at_ms,
                    expires_at_ms,
                    maximum,
                ),
            );
        }
    }
    fill_unknown_features(
        &mut descriptor,
        CapabilityLayer::AdapterNormalized,
        "pi.route-unknown",
        checked_at_ms,
        expires_at_ms,
        "Pi route metadata does not declare model feature support",
    );
    fill_unknown_numeric(
        &mut descriptor,
        CapabilityLayer::AdapterNormalized,
        "pi.route-unknown",
        checked_at_ms,
        expires_at_ms,
        "Pi route metadata does not declare model limits",
    );
    validate_descriptor(&descriptor, CapabilityLayer::AdapterNormalized)?;
    Ok(AdapterNormalizedEvidence(descriptor))
}

pub fn opencode_observed_evidence(
    conformance: &AdapterConformanceDescriptor,
    health: &OpenCodeHealth,
    observation: &RuntimeRouteObservation,
) -> Result<ObservedEvidence, CapabilityEvidenceError> {
    validate_observation_identity(
        RuntimeKind::OpenCode,
        conformance,
        &observation.route,
        observation,
    )?;
    if !health.healthy
        || health.native_version != conformance.native_version
        || health.process_generation != conformance.process_generation
        || health.checked_at_ms != observation.health_checked_at_ms
        || health.checked_at_ms > observation.observed_at_ms
        || observation
            .observed_at_ms
            .saturating_sub(health.checked_at_ms)
            > MAX_HEALTH_OBSERVATION_AGE_MS
    {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    observed_from_artifacts(
        conformance,
        observation,
        CapabilityState::Supported,
        &BTreeMap::new(),
        "opencode.runtime-observation",
        digest_json(&(conformance, health, observation))?,
    )
}

pub fn pi_observed_evidence(
    conformance: &AdapterConformanceDescriptor,
    health: &PiHealth,
    observation: &RuntimeRouteObservation,
) -> Result<ObservedEvidence, CapabilityEvidenceError> {
    validate_observation_identity(
        RuntimeKind::Pi,
        conformance,
        &observation.route,
        observation,
    )?;
    if health.runtime != "pi"
        || health.transport != "c4os-node-sdk-sidecar"
        || health.protocol != PI_PROTOCOL
        || health.process_generation != conformance.process_generation
        || !matches!(health.status.as_str(), "ready" | "degraded")
        || observation.health_checked_at_ms != observation.observed_at_ms
    {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    let health_gates = pi_health_gates(health)?;
    let health_state = if health.status == "ready" {
        CapabilityState::Supported
    } else {
        CapabilityState::Degraded
    };
    observed_from_artifacts(
        conformance,
        observation,
        health_state,
        &health_gates,
        "pi.runtime-observation",
        digest_json(&(conformance, health, observation))?,
    )
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityRouteEpoch {
    runtime_id: String,
    process_generation: u64,
    route: RouteIdentity,
    declared: CapabilityDescriptor,
    adapter_normalized: CapabilityDescriptor,
    observed: CapabilityDescriptor,
}

impl CapabilityRouteEpoch {
    pub fn new(
        runtime_id: impl Into<String>,
        process_generation: u64,
        declared: DeclaredEvidence,
        adapter_normalized: AdapterNormalizedEvidence,
        observed: ObservedEvidence,
    ) -> Result<Self, CapabilityEvidenceError> {
        let runtime_id = runtime_id.into();
        let route = declared.0.route.clone();
        let epoch = Self {
            runtime_id,
            process_generation,
            route,
            declared: declared.0,
            adapter_normalized: adapter_normalized.0,
            observed: observed.descriptor,
        };
        epoch.validate(Some((&observed.runtime_id, observed.process_generation)))?;
        Ok(epoch)
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn process_generation(&self) -> u64 {
        self.process_generation
    }

    pub fn route(&self) -> &RouteIdentity {
        &self.route
    }

    pub fn layers(&self) -> [CapabilityDescriptor; 3] {
        [
            self.declared.clone(),
            self.adapter_normalized.clone(),
            self.observed.clone(),
        ]
    }

    fn validate(
        &self,
        observed_identity: Option<(&str, u64)>,
    ) -> Result<(), CapabilityEvidenceError> {
        if !valid_runtime_id(&self.runtime_id) || self.process_generation == 0 {
            return Err(CapabilityEvidenceError::InvalidProcessIdentity);
        }
        self.route
            .validate()
            .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?;
        for (descriptor, layer) in [
            (&self.declared, CapabilityLayer::Declared),
            (&self.adapter_normalized, CapabilityLayer::AdapterNormalized),
            (&self.observed, CapabilityLayer::Observed),
        ] {
            validate_descriptor(descriptor, layer)?;
            if descriptor.route != self.route {
                return Err(CapabilityEvidenceError::ArtifactMismatch);
            }
        }
        if observed_identity.is_some_and(|(runtime_id, process_generation)| {
            runtime_id != self.runtime_id || process_generation != self.process_generation
        }) {
            return Err(CapabilityEvidenceError::ArtifactMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityEvidenceSnapshot {
    pub generation: u64,
    pub active_routes: BTreeMap<String, CapabilityRouteEpoch>,
    pub active_processes: BTreeMap<String, u64>,
    pub historical_routes: Vec<CapabilityRouteEpoch>,
    pub historical_routes_dropped: u64,
}

#[derive(Clone, Default)]
pub struct CapabilityEvidenceRegistry {
    generation: u64,
    routes: BTreeMap<String, CapabilityRouteEpoch>,
    processes: BTreeMap<String, u64>,
    history: Vec<CapabilityRouteEpoch>,
    history_dropped: u64,
}

impl CapabilityEvidenceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn process_generation(&self, runtime_id: &str) -> Option<u64> {
        self.processes.get(runtime_id).copied()
    }

    pub fn has_active_processes(&self) -> bool {
        !self.processes.is_empty() || !self.routes.is_empty()
    }

    pub fn snapshot(&self) -> CapabilityEvidenceSnapshot {
        CapabilityEvidenceSnapshot {
            generation: self.generation,
            active_routes: self.routes.clone(),
            active_processes: self.processes.clone(),
            historical_routes: self.history.clone(),
            historical_routes_dropped: self.history_dropped,
        }
    }

    /// Restores durable evidence but never restores process authority. Any
    /// routes that were active at the crash boundary become historical and a
    /// fresh generation is minted so retained renderer/run CAS values fail.
    pub fn restore(snapshot: CapabilityEvidenceSnapshot) -> Result<Self, CapabilityEvidenceError> {
        if snapshot.active_routes.len() > MAX_EVIDENCE_ROUTES
            || snapshot.historical_routes.len() > MAX_RESTORABLE_HISTORICAL_EVIDENCE_ROUTES
        {
            return Err(CapabilityEvidenceError::CapacityExceeded);
        }
        for (key, epoch) in &snapshot.active_routes {
            epoch.validate(None)?;
            if route_key(&epoch.route)? != *key
                || snapshot.active_processes.get(&epoch.runtime_id)
                    != Some(&epoch.process_generation)
            {
                return Err(CapabilityEvidenceError::ArtifactMismatch);
            }
        }
        for (runtime_id, process_generation) in &snapshot.active_processes {
            if !valid_runtime_id(runtime_id) || *process_generation == 0 {
                return Err(CapabilityEvidenceError::InvalidProcessIdentity);
            }
        }
        for epoch in &snapshot.historical_routes {
            epoch.validate(None)?;
        }
        let had_active =
            !snapshot.active_processes.is_empty() || !snapshot.active_routes.is_empty();
        let mut historical_routes = snapshot.historical_routes;
        let compacted_routes = historical_routes
            .len()
            .saturating_sub(MAX_HISTORICAL_EVIDENCE_ROUTES);
        if compacted_routes > 0 {
            historical_routes.drain(..compacted_routes);
        }
        let mut registry = Self {
            generation: snapshot.generation,
            routes: BTreeMap::new(),
            processes: BTreeMap::new(),
            history: historical_routes,
            history_dropped: snapshot
                .historical_routes_dropped
                .saturating_add(compacted_routes as u64),
        };
        for epoch in snapshot.active_routes.into_values() {
            registry.archive(epoch);
        }
        if had_active {
            registry.generation = registry.next_generation()?;
        }
        Ok(registry)
    }

    #[cfg(test)]
    pub(crate) fn advance_generation_for_atomic_publication_test(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }

    pub fn replace_process_routes(
        &mut self,
        expected_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        epochs: Vec<CapabilityRouteEpoch>,
    ) -> Result<u64, CapabilityEvidenceError> {
        self.require_generation(expected_generation)?;
        if !valid_runtime_id(runtime_id) || process_generation == 0 {
            return Err(CapabilityEvidenceError::InvalidProcessIdentity);
        }
        if epochs.len() > MAX_EVIDENCE_ROUTES {
            return Err(CapabilityEvidenceError::CapacityExceeded);
        }
        if epochs.iter().any(|epoch| {
            epoch.runtime_id != runtime_id || epoch.process_generation != process_generation
        }) {
            return Err(CapabilityEvidenceError::ArtifactMismatch);
        }
        if self
            .processes
            .get(runtime_id)
            .is_some_and(|generation| *generation != process_generation)
        {
            return Err(CapabilityEvidenceError::ProcessConflict);
        }

        let mut incoming = BTreeMap::new();
        for epoch in epochs {
            epoch.validate(None)?;
            let key = route_key(&epoch.route)?;
            if let Some(current) = self.routes.get(&key)
                && current.runtime_id == runtime_id
                && current.process_generation == process_generation
                && !epoch.is_monotonic_after(current)
            {
                return Err(CapabilityEvidenceError::NonMonotonicEvidence);
            }
            if incoming.insert(key, epoch).is_some() {
                return Err(CapabilityEvidenceError::RouteConflict);
            }
        }

        let mut candidate_routes = self.routes.clone();
        candidate_routes.retain(|_, epoch| {
            epoch.runtime_id != runtime_id || epoch.process_generation != process_generation
        });
        for (key, epoch) in incoming {
            if candidate_routes.contains_key(&key) {
                return Err(CapabilityEvidenceError::RouteConflict);
            }
            candidate_routes.insert(key, epoch);
        }
        if candidate_routes.len() > MAX_EVIDENCE_ROUTES {
            return Err(CapabilityEvidenceError::CapacityExceeded);
        }
        let next_generation = self.next_generation()?;
        let retired = self
            .routes
            .values()
            .filter(|epoch| {
                epoch.runtime_id == runtime_id && epoch.process_generation == process_generation
            })
            .cloned()
            .collect::<Vec<_>>();
        for epoch in retired {
            self.archive(epoch);
        }
        self.routes = candidate_routes;
        self.processes
            .insert(runtime_id.to_owned(), process_generation);
        self.generation = next_generation;
        Ok(next_generation)
    }

    pub fn invalidate_process(
        &mut self,
        expected_generation: u64,
        runtime_id: &str,
        process_generation: u64,
    ) -> Result<u64, CapabilityEvidenceError> {
        self.require_generation(expected_generation)?;
        if self.processes.get(runtime_id) != Some(&process_generation) {
            return Err(CapabilityEvidenceError::ProcessNotFound);
        }
        let next_generation = self.next_generation()?;
        let retired = self
            .routes
            .values()
            .filter(|epoch| {
                epoch.runtime_id == runtime_id && epoch.process_generation == process_generation
            })
            .cloned()
            .collect::<Vec<_>>();
        for epoch in retired {
            self.archive(epoch);
        }
        self.routes.retain(|_, epoch| {
            epoch.runtime_id != runtime_id || epoch.process_generation != process_generation
        });
        self.processes.remove(runtime_id);
        self.generation = next_generation;
        Ok(next_generation)
    }

    pub fn layers(
        &self,
        route: &RouteIdentity,
        now_ms: u64,
    ) -> Result<[CapabilityDescriptor; 3], CapabilityEvidenceError> {
        route
            .validate()
            .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?;
        if now_ms == 0 {
            return Err(CapabilityEvidenceError::InvalidTimestamp);
        }
        let key = route_key(route)?;
        let epoch = self
            .routes
            .get(&key)
            .ok_or(CapabilityEvidenceError::RouteNotFound)?;
        epoch.validate(None)?;
        if &epoch.route != route
            || self.processes.get(&epoch.runtime_id) != Some(&epoch.process_generation)
        {
            return Err(CapabilityEvidenceError::ArtifactMismatch);
        }
        let layers = epoch.layers();
        for descriptor in &layers {
            if descriptor
                .features
                .values()
                .chain(
                    descriptor
                        .numeric_limits
                        .values()
                        .map(|numeric| &numeric.evidence),
                )
                .any(|evidence| {
                    evidence
                        .expires_at_ms
                        .is_some_and(|expiry| now_ms >= expiry)
                })
            {
                return Err(CapabilityEvidenceError::Expired);
            }
        }
        Ok(layers)
    }

    pub fn route_for_runtime_model(
        &self,
        runtime_id: &str,
        provider_id: &str,
        endpoint_id: &str,
        model_id: &str,
        now_ms: u64,
    ) -> Result<RouteIdentity, CapabilityEvidenceError> {
        if !valid_runtime_id(runtime_id) || now_ms == 0 {
            return Err(CapabilityEvidenceError::InvalidProcessIdentity);
        }
        let mut matches = self.routes.values().filter(|epoch| {
            epoch.runtime_id == runtime_id
                && self.processes.get(runtime_id) == Some(&epoch.process_generation)
                && epoch.route.provider_id == provider_id
                && epoch.route.endpoint_id == endpoint_id
                && route_model_id(&epoch.route) == Some(model_id)
        });
        let route = matches
            .next()
            .ok_or(CapabilityEvidenceError::RouteNotFound)?
            .route
            .clone();
        if matches.next().is_some() {
            return Err(CapabilityEvidenceError::RouteConflict);
        }
        self.layers(&route, now_ms)?;
        Ok(route)
    }

    /// Resolves effective preflight truth only from registry-owned layers.
    /// Callers supply an exact selected route identity, never descriptors.
    pub fn effective_for_route(
        &self,
        route: &RouteIdentity,
        now_ms: u64,
    ) -> Result<CapabilityDescriptor, CapabilityEvidenceError> {
        let layers = self.layers(route, now_ms)?;
        effective_intersection(&layers, now_ms)
            .map_err(|_| CapabilityEvidenceError::IntersectionFailed)
    }

    fn require_generation(&self, expected: u64) -> Result<(), CapabilityEvidenceError> {
        if expected == self.generation {
            Ok(())
        } else {
            Err(CapabilityEvidenceError::StaleGeneration {
                expected,
                current: self.generation,
            })
        }
    }

    fn next_generation(&self) -> Result<u64, CapabilityEvidenceError> {
        self.generation
            .checked_add(1)
            .ok_or(CapabilityEvidenceError::GenerationExhausted)
    }

    fn archive(&mut self, epoch: CapabilityRouteEpoch) {
        while self.history.len() >= MAX_HISTORICAL_EVIDENCE_ROUTES {
            self.history.remove(0);
            self.history_dropped = self.history_dropped.saturating_add(1);
        }
        self.history.push(epoch);
    }
}

impl CapabilityRouteEpoch {
    fn is_monotonic_after(&self, current: &Self) -> bool {
        if self.runtime_id != current.runtime_id
            || self.process_generation != current.process_generation
            || self.route != current.route
        {
            return false;
        }
        [
            (&self.declared, &current.declared),
            (&self.adapter_normalized, &current.adapter_normalized),
            (&self.observed, &current.observed),
        ]
        .into_iter()
        .all(|(incoming, previous)| {
            let incoming_checked = descriptor_checked_at(incoming);
            let previous_checked = descriptor_checked_at(previous);
            incoming_checked > previous_checked
                || (incoming_checked == previous_checked && incoming == previous)
        })
    }
}

fn descriptor_checked_at(descriptor: &CapabilityDescriptor) -> u64 {
    descriptor
        .features
        .values()
        .map(|evidence| evidence.checked_at_ms)
        .chain(
            descriptor
                .numeric_limits
                .values()
                .map(|numeric| numeric.evidence.checked_at_ms),
        )
        .max()
        .unwrap_or(0)
}

fn observed_from_artifacts(
    conformance: &AdapterConformanceDescriptor,
    observation: &RuntimeRouteObservation,
    health_state: CapabilityState,
    health_feature_gates: &BTreeMap<CapabilityKey, CapabilityState>,
    source: &str,
    raw_evidence_sha256: String,
) -> Result<ObservedEvidence, CapabilityEvidenceError> {
    let outcome_state = match observation.outcome {
        RuntimeObservationOutcome::Available => CapabilityState::Supported,
        RuntimeObservationOutcome::Degraded => CapabilityState::Degraded,
        RuntimeObservationOutcome::Failed => CapabilityState::Unknown,
    };
    let conformance_health = conformance
        .capabilities
        .get("health")
        .copied()
        .unwrap_or(CapabilityState::Unknown);
    let overall_state = restrictive_state(
        restrictive_state(health_state, outcome_state),
        conformance_health,
    );
    let mut features = BTreeMap::new();
    for key in FEATURE_KEYS {
        let Some(claim) = observation.features.get(&key) else {
            features.insert(
                key,
                unknown_evidence(
                    CapabilityLayer::Observed,
                    source,
                    observation.observed_at_ms,
                    observation.expires_at_ms,
                    "runtime observation did not exercise this feature",
                ),
            );
            continue;
        };
        let adapter_gate = conformance_gate(conformance, key);
        let health_gate = health_feature_gates
            .get(&key)
            .copied()
            .unwrap_or(CapabilityState::Supported);
        let state = restrictive_state(
            restrictive_state(restrictive_state(claim.state, overall_state), adapter_gate),
            health_gate,
        );
        let mut effective_claim = claim.clone();
        effective_claim.state = state;
        if state != CapabilityState::Supported && effective_claim.reason.is_none() {
            effective_claim.reason = Some(observation_reason(state, key));
        }
        features.insert(
            key,
            evidence_from_claim(
                CapabilityLayer::Observed,
                source,
                observation.observed_at_ms,
                observation.expires_at_ms,
                &effective_claim,
                "runtime observation did not prove this feature",
            ),
        );
    }
    let mut numeric_limits = BTreeMap::new();
    for key in NUMERIC_KEYS {
        let Some(claim) = observation.numeric_limits.get(&key) else {
            numeric_limits.insert(
                key,
                unknown_numeric(
                    CapabilityLayer::Observed,
                    source,
                    observation.observed_at_ms,
                    observation.expires_at_ms,
                    "runtime observation did not exercise this limit",
                ),
            );
            continue;
        };
        let mut effective_claim = claim.clone();
        effective_claim.state = restrictive_state(claim.state, overall_state);
        if effective_claim.state != CapabilityState::Supported && effective_claim.reason.is_none() {
            effective_claim.reason =
                Some("runtime health or behavior degraded this limit evidence".into());
        }
        numeric_limits.insert(
            key,
            numeric_from_claim(
                CapabilityLayer::Observed,
                source,
                observation.observed_at_ms,
                observation.expires_at_ms,
                &effective_claim,
                "runtime observation did not prove this limit",
            ),
        );
    }
    let descriptor = CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Observed,
        route: observation.route.clone(),
        lifecycle: if observation.outcome == RuntimeObservationOutcome::Failed {
            ModelLifecycle::Unavailable
        } else {
            observation.lifecycle
        },
        features,
        numeric_limits,
        raw_evidence_sha256,
    };
    validate_descriptor(&descriptor, CapabilityLayer::Observed)?;
    Ok(ObservedEvidence {
        runtime_id: observation.runtime_id.clone(),
        process_generation: observation.process_generation,
        descriptor,
    })
}

fn validate_observation_identity(
    runtime_kind: RuntimeKind,
    conformance: &AdapterConformanceDescriptor,
    route: &RouteIdentity,
    observation: &RuntimeRouteObservation,
) -> Result<(), CapabilityEvidenceError> {
    conformance
        .validate()
        .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?;
    validate_claim_envelope(
        route,
        observation.observed_at_ms,
        observation.expires_at_ms,
        &observation.raw_observation_sha256,
        observation.features.len(),
        observation.numeric_limits.len(),
    )?;
    if !valid_runtime_id(&observation.runtime_id)
        || &observation.route != route
        || conformance.runtime_kind != runtime_kind
        || route.runtime_kind != runtime_kind.as_str()
        || route.adapter_kind != runtime_kind.as_str()
        || route.adapter_version != conformance.adapter_version
        || route.native_runtime_version != conformance.native_version
        || observation.process_generation != conformance.process_generation
        || observation.health_checked_at_ms == 0
        || observation.health_checked_at_ms > observation.observed_at_ms
        || match runtime_kind {
            RuntimeKind::OpenCode => conformance.native_version != OPENCODE_NATIVE_VERSION,
            RuntimeKind::Pi => conformance.native_version != PI_NATIVE_VERSION,
        }
    {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    Ok(())
}

fn conformance_gate(
    conformance: &AdapterConformanceDescriptor,
    key: CapabilityKey,
) -> CapabilityState {
    let capability = match key {
        CapabilityKey::Streaming | CapabilityKey::StreamedToolArguments => Some("streaming"),
        CapabilityKey::ToolCalling
        | CapabilityKey::ParallelToolCalling
        | CapabilityKey::StrictToolSchema => Some("action-intents"),
        CapabilityKey::SessionAffinity => Some("session-resume"),
        _ => None,
    };
    capability.map_or(CapabilityState::Supported, |capability| {
        conformance
            .capabilities
            .get(capability)
            .copied()
            .unwrap_or(CapabilityState::Unknown)
    })
}

fn pi_health_gates(
    health: &PiHealth,
) -> Result<BTreeMap<CapabilityKey, CapabilityState>, CapabilityEvidenceError> {
    let mut native_boundary_valid = true;
    for key in [
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
        let capability = health
            .capabilities
            .get(key)
            .ok_or(CapabilityEvidenceError::InvalidArtifact)?;
        let _ = capability_state(&capability.state)?;
        if matches!(
            key,
            "nativePersistence" | "nativeExtensions" | "nativeTools"
        ) && capability.state != "unsupported"
        {
            native_boundary_valid = false;
        }
    }
    if !native_boundary_valid {
        return Err(CapabilityEvidenceError::ArtifactMismatch);
    }
    let streaming = capability_state(&health.capabilities["streaming"].state)?;
    let tools = capability_state(&health.capabilities["tools"].state)?;
    let resume = capability_state(&health.capabilities["crashResume"].state)?;
    Ok(BTreeMap::from([
        (CapabilityKey::Streaming, streaming),
        (CapabilityKey::StreamedToolArguments, streaming),
        (CapabilityKey::ToolCalling, tools),
        (CapabilityKey::ParallelToolCalling, tools),
        (CapabilityKey::StrictToolSchema, tools),
        (CapabilityKey::SessionAffinity, resume),
    ]))
}

fn capability_state(value: &str) -> Result<CapabilityState, CapabilityEvidenceError> {
    match value {
        "supported" => Ok(CapabilityState::Supported),
        "degraded" => Ok(CapabilityState::Degraded),
        "unsupported" => Ok(CapabilityState::Unsupported),
        "unknown" => Ok(CapabilityState::Unknown),
        _ => Err(CapabilityEvidenceError::InvalidArtifact),
    }
}

fn observation_reason(state: CapabilityState, key: CapabilityKey) -> String {
    match state {
        CapabilityState::Degraded => {
            format!("runtime or adapter conformance degrades {key:?}").to_lowercase()
        }
        CapabilityState::Unsupported => {
            format!("runtime or adapter conformance does not support {key:?}").to_lowercase()
        }
        CapabilityState::Unknown => {
            format!("runtime observation could not prove {key:?}").to_lowercase()
        }
        CapabilityState::Supported => String::new(),
    }
}

fn validate_claim_envelope(
    route: &RouteIdentity,
    checked_at_ms: u64,
    expires_at_ms: u64,
    digest: &str,
    features: usize,
    numeric_limits: usize,
) -> Result<(), CapabilityEvidenceError> {
    route
        .validate()
        .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?;
    if checked_at_ms == 0
        || expires_at_ms <= checked_at_ms
        || features > MAX_EVIDENCE_VALUES
        || numeric_limits > MAX_EVIDENCE_VALUES
        || !valid_digest(digest)
    {
        return Err(CapabilityEvidenceError::InvalidArtifact);
    }
    Ok(())
}

fn validate_descriptor(
    descriptor: &CapabilityDescriptor,
    required_layer: CapabilityLayer,
) -> Result<(), CapabilityEvidenceError> {
    descriptor
        .validate()
        .map_err(|_| CapabilityEvidenceError::InvalidArtifact)?;
    if required_layer == CapabilityLayer::Effective || descriptor.layer != required_layer {
        return Err(CapabilityEvidenceError::LayerRelabeling);
    }
    Ok(())
}

fn fill_unknown_features(
    descriptor: &mut CapabilityDescriptor,
    layer: CapabilityLayer,
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    reason: &str,
) {
    for key in FEATURE_KEYS {
        descriptor.features.entry(key).or_insert_with(|| {
            unknown_evidence(layer, source, checked_at_ms, expires_at_ms, reason)
        });
    }
}

fn fill_unknown_numeric(
    descriptor: &mut CapabilityDescriptor,
    layer: CapabilityLayer,
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    reason: &str,
) {
    for key in NUMERIC_KEYS {
        descriptor.numeric_limits.entry(key).or_insert_with(|| {
            unknown_numeric(layer, source, checked_at_ms, expires_at_ms, reason)
        });
    }
}

fn evidence_from_claim(
    layer: CapabilityLayer,
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    claim: &FeatureClaim,
    fallback_reason: &str,
) -> CapabilityEvidence {
    CapabilityEvidence {
        state: claim.state,
        layer,
        source: source.into(),
        checked_at_ms,
        expires_at_ms: Some(expires_at_ms),
        constraints: claim.constraints.clone(),
        allowed_values: claim.allowed_values.clone(),
        reason: if claim.state == CapabilityState::Supported {
            None
        } else {
            claim
                .reason
                .clone()
                .or_else(|| Some(fallback_reason.into()))
        },
    }
}

fn numeric_from_claim(
    layer: CapabilityLayer,
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    claim: &NumericClaim,
    fallback_reason: &str,
) -> NumericCapabilityEvidence {
    NumericCapabilityEvidence {
        evidence: CapabilityEvidence {
            state: claim.state,
            layer,
            source: source.into(),
            checked_at_ms,
            expires_at_ms: Some(expires_at_ms),
            constraints: Vec::new(),
            allowed_values: Vec::new(),
            reason: if claim.state == CapabilityState::Supported {
                None
            } else {
                claim
                    .reason
                    .clone()
                    .or_else(|| Some(fallback_reason.into()))
            },
        },
        maximum: claim.maximum,
        confidence: claim.confidence,
    }
}

fn confirmed_adapter_limit(
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    maximum: u64,
) -> NumericCapabilityEvidence {
    NumericCapabilityEvidence {
        evidence: CapabilityEvidence {
            state: CapabilityState::Supported,
            layer: CapabilityLayer::AdapterNormalized,
            source: source.into(),
            checked_at_ms,
            expires_at_ms: Some(expires_at_ms),
            constraints: Vec::new(),
            allowed_values: Vec::new(),
            reason: None,
        },
        maximum: Some(maximum),
        confidence: LimitConfidence::Confirmed,
    }
}

fn unknown_evidence(
    layer: CapabilityLayer,
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    reason: &str,
) -> CapabilityEvidence {
    CapabilityEvidence {
        state: CapabilityState::Unknown,
        layer,
        source: source.into(),
        checked_at_ms,
        expires_at_ms: Some(expires_at_ms),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: Some(reason.into()),
    }
}

fn unknown_numeric(
    layer: CapabilityLayer,
    source: &str,
    checked_at_ms: u64,
    expires_at_ms: u64,
    reason: &str,
) -> NumericCapabilityEvidence {
    NumericCapabilityEvidence {
        evidence: unknown_evidence(layer, source, checked_at_ms, expires_at_ms, reason),
        maximum: None,
        confidence: LimitConfidence::Unknown,
    }
}

fn earliest_expiration(descriptor: &CapabilityDescriptor) -> Result<u64, CapabilityEvidenceError> {
    descriptor
        .features
        .values()
        .chain(
            descriptor
                .numeric_limits
                .values()
                .map(|numeric| &numeric.evidence),
        )
        .filter_map(|evidence| evidence.expires_at_ms)
        .min()
        .ok_or(CapabilityEvidenceError::InvalidArtifact)
}

fn restrictive_state(left: CapabilityState, right: CapabilityState) -> CapabilityState {
    let rank = |state| match state {
        CapabilityState::Supported => 0,
        CapabilityState::Degraded => 1,
        CapabilityState::Unknown => 2,
        CapabilityState::Unsupported => 3,
    };
    if rank(left) >= rank(right) {
        left
    } else {
        right
    }
}

fn route_key(route: &RouteIdentity) -> Result<String, CapabilityEvidenceError> {
    serde_json::to_string(route).map_err(|_| CapabilityEvidenceError::Serialization)
}

pub(crate) fn route_model_id(route: &RouteIdentity) -> Option<&str> {
    route
        .provider_model_id
        .split_once('/')
        .map(|(_, model_id)| model_id)
        .filter(|model_id| !model_id.is_empty())
}

fn digest_json(value: &impl Serialize) -> Result<String, CapabilityEvidenceError> {
    let bytes = serde_json::to_vec(value).map_err(|_| CapabilityEvidenceError::Serialization)?;
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    use std::fmt::Write as _;
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    Ok(encoded)
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn valid_runtime_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 192
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum CapabilityEvidenceError {
    #[error("capability evidence artifact is invalid")]
    InvalidArtifact,
    #[error("capability evidence artifact identities do not match")]
    ArtifactMismatch,
    #[error("capability evidence cannot be relabeled across layers")]
    LayerRelabeling,
    #[error("capability evidence timestamp is invalid")]
    InvalidTimestamp,
    #[error("capability evidence expired")]
    Expired,
    #[error("capability evidence route was not found")]
    RouteNotFound,
    #[error("capability evidence process identity is invalid")]
    InvalidProcessIdentity,
    #[error("capability evidence process generation conflicts with the active epoch")]
    ProcessConflict,
    #[error("capability evidence process was not found")]
    ProcessNotFound,
    #[error("capability evidence route is already owned by another process epoch")]
    RouteConflict,
    #[error("capability evidence layer is missing")]
    MissingLayer,
    #[error("capability evidence registry reached its route bound")]
    CapacityExceeded,
    #[error("capability evidence is not newer than the recorded layer")]
    NonMonotonicEvidence,
    #[error("capability evidence generation is stale: expected {expected}, current {current}")]
    StaleGeneration { expected: u64, current: u64 },
    #[error("capability evidence generation was exhausted")]
    GenerationExhausted,
    #[error("capability evidence serialization failed")]
    Serialization,
    #[error("stored capability evidence could not produce effective truth")]
    IntersectionFailed,
}
