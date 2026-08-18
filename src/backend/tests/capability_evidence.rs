use std::{collections::BTreeMap, path::PathBuf};

use c4os_lib::runtime::{
    capability::{
        CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
        CapabilityLayer, CapabilityState, LimitConfidence, ModelLifecycle,
        NumericCapabilityEvidence, NumericCapabilityKey, RouteIdentity, effective_intersection,
    },
    capability_evidence::{
        CapabilityEvidenceError, CapabilityEvidenceRegistry, CapabilityEvidenceSnapshot,
        CapabilityRouteEpoch, FeatureClaim, MAX_EVIDENCE_ROUTES, MAX_HISTORICAL_EVIDENCE_ROUTES,
        NumericClaim, ProviderDeclaredCatalogClaim, RuntimeObservationOutcome,
        RuntimeRouteObservation, opencode_adapter_evidence, opencode_observed_evidence,
        pi_adapter_evidence, pi_observed_evidence, provider_declared_evidence,
        provider_model_declared_evidence,
    },
    opencode::{HealthSnapshot, OpenCodeCompatibilityManifest},
    pi::{PiCapabilityState, PiHealth, PiModelRoute, PiSidecarManifest},
    provider::{
        ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, ProviderModelDeclaration,
        RouteAvailability,
    },
};

const NOW: u64 = 1_721_300_000_000;

fn digest(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}

fn open_code_route() -> RouteIdentity {
    RouteIdentity {
        provider_id: "provider-openai".into(),
        endpoint_id: "openai-api".into(),
        provider_model_id: "openai/gpt-4o-mini".into(),
        model_revision: "2026-07-19".into(),
        adapter_kind: "opencode".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "opencode".into(),
        native_runtime_version: "1.18.3".into(),
        session_configuration_sha256: digest('a'),
    }
}

fn feature(state: CapabilityState, reason: Option<&str>) -> FeatureClaim {
    FeatureClaim {
        state,
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: reason.map(str::to_owned),
    }
}

fn numeric(maximum: u64) -> NumericClaim {
    NumericClaim {
        state: CapabilityState::Supported,
        maximum: Some(maximum),
        confidence: LimitConfidence::Confirmed,
        reason: None,
    }
}

fn provider_claim(route: RouteIdentity, checked_at_ms: u64) -> ProviderDeclaredCatalogClaim {
    ProviderDeclaredCatalogClaim {
        route,
        lifecycle: ModelLifecycle::Active,
        declared_at_ms: checked_at_ms,
        expires_at_ms: checked_at_ms + 60_000,
        features: BTreeMap::from([
            (
                CapabilityKey::InputText,
                feature(CapabilityState::Supported, None),
            ),
            (
                CapabilityKey::InputImage,
                feature(CapabilityState::Supported, None),
            ),
            (
                CapabilityKey::ToolCalling,
                feature(CapabilityState::Supported, None),
            ),
            (
                CapabilityKey::Streaming,
                feature(CapabilityState::Supported, None),
            ),
        ]),
        numeric_limits: BTreeMap::from([(NumericCapabilityKey::ContextTokens, numeric(200_000))]),
        raw_catalog_sha256: digest('b'),
    }
}

fn normalized_evidence(state: CapabilityState, checked_at_ms: u64) -> CapabilityEvidence {
    CapabilityEvidence {
        state,
        layer: CapabilityLayer::AdapterNormalized,
        source: "opencode.1.18.3".into(),
        checked_at_ms,
        expires_at_ms: Some(checked_at_ms + 60_000),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: (state != CapabilityState::Supported)
            .then(|| "OpenCode inventory reports this capability unavailable".into()),
    }
}

fn open_code_model_route(route: RouteIdentity, checked_at_ms: u64) -> ModelRoute {
    ModelRoute {
        model_id: "gpt-4o-mini".into(),
        display_name: "GPT-4o mini".into(),
        recommendation_rank: 0,
        availability: RouteAvailability::Available,
        checked_at_ms,
        capabilities: CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::AdapterNormalized,
            route,
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::from([
                (
                    CapabilityKey::InputText,
                    normalized_evidence(CapabilityState::Supported, checked_at_ms),
                ),
                (
                    CapabilityKey::InputImage,
                    normalized_evidence(CapabilityState::Unsupported, checked_at_ms),
                ),
                (
                    CapabilityKey::ToolCalling,
                    normalized_evidence(CapabilityState::Supported, checked_at_ms),
                ),
            ]),
            numeric_limits: BTreeMap::from([(
                NumericCapabilityKey::ContextTokens,
                NumericCapabilityEvidence {
                    evidence: normalized_evidence(CapabilityState::Supported, checked_at_ms),
                    maximum: Some(128_000),
                    confidence: LimitConfidence::Confirmed,
                },
            )]),
            raw_evidence_sha256: digest('c'),
        },
        provider_declaration: None,
    }
}

#[test]
fn provider_qualified_routes_preserve_model_ids_with_slashes() {
    let mut route = open_code_route();
    route.provider_id = "provider-openrouter".into();
    route.endpoint_id = "openrouter-api".into();
    route.provider_model_id = "provider:openrouter/openrouter/auto-beta".into();
    let declaration = ProviderModelDeclaration {
        schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
        provider_model_id: "openrouter/auto-beta".into(),
        model_revision: route.model_revision.clone(),
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::new(),
        numeric_limits: BTreeMap::new(),
        raw_catalog_sha256: digest('d'),
        declared_at_ms: NOW - 900,
        expires_at_ms: NOW + 60_000,
    };
    let mut model = open_code_model_route(route.clone(), NOW - 900);
    model.model_id = "openrouter/auto-beta".into();

    let declared = provider_model_declared_evidence(&declaration, route.clone()).unwrap();
    let adapter = opencode_adapter_evidence(&model).unwrap();

    assert_eq!(declared.descriptor().route, route);
    assert_eq!(adapter.descriptor().route, route);

    let mut pi_route = route;
    pi_route.provider_model_id = "openrouter/openrouter/auto-beta".into();
    pi_route.adapter_kind = "pi".into();
    pi_route.runtime_kind = "pi".into();
    pi_route.native_runtime_version = "0.80.10".into();
    let pi_model = PiModelRoute {
        provider: "openrouter".into(),
        model_id: "openrouter/auto-beta".into(),
        base_url: "https://openrouter.ai/api/v1".into(),
    };
    assert!(pi_adapter_evidence(&pi_model, &pi_route, NOW - 900, NOW + 60_000).is_ok());
}

fn observation(route: RouteIdentity, checked_at_ms: u64) -> RuntimeRouteObservation {
    RuntimeRouteObservation {
        runtime_id: "opencode-primary".into(),
        route,
        process_generation: 7,
        health_checked_at_ms: checked_at_ms - 1,
        observed_at_ms: checked_at_ms,
        expires_at_ms: checked_at_ms + 30_000,
        outcome: RuntimeObservationOutcome::Available,
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::from([
            (
                CapabilityKey::InputText,
                feature(CapabilityState::Supported, None),
            ),
            (
                CapabilityKey::InputImage,
                feature(CapabilityState::Supported, None),
            ),
            (
                CapabilityKey::ToolCalling,
                feature(CapabilityState::Supported, None),
            ),
            (
                CapabilityKey::Streaming,
                feature(CapabilityState::Supported, None),
            ),
        ]),
        numeric_limits: BTreeMap::from([(NumericCapabilityKey::ContextTokens, numeric(100_000))]),
        raw_observation_sha256: digest('d'),
    }
}

fn open_code_artifacts(
    checked_at_ms: u64,
) -> (
    c4os_lib::runtime::adapter::AdapterConformanceDescriptor,
    HealthSnapshot,
) {
    let conformance = OpenCodeCompatibilityManifest::pinned(digest('e'))
        .unwrap()
        .conformance_descriptor(7)
        .unwrap();
    let health = HealthSnapshot {
        healthy: true,
        native_version: "1.18.3".into(),
        process_generation: 7,
        checked_at_ms: checked_at_ms - 1,
    };
    (conformance, health)
}

fn open_code_epoch(route: RouteIdentity) -> CapabilityRouteEpoch {
    open_code_epoch_for_runtime(route, "opencode-primary")
}

fn open_code_epoch_for_runtime(route: RouteIdentity, runtime_id: &str) -> CapabilityRouteEpoch {
    open_code_epoch_for_runtime_at(route, runtime_id, NOW - 1_000, NOW - 900, NOW - 800)
}

fn open_code_epoch_for_runtime_at(
    route: RouteIdentity,
    runtime_id: &str,
    declared_at_ms: u64,
    normalized_at_ms: u64,
    observed_at_ms: u64,
) -> CapabilityRouteEpoch {
    let declared =
        provider_declared_evidence(&provider_claim(route.clone(), declared_at_ms)).unwrap();
    let normalized =
        opencode_adapter_evidence(&open_code_model_route(route.clone(), normalized_at_ms)).unwrap();
    let (conformance, health) = open_code_artifacts(observed_at_ms);
    let mut observation = observation(route, observed_at_ms);
    observation.runtime_id = runtime_id.into();
    let observed = opencode_observed_evidence(&conformance, &health, &observation).unwrap();
    CapabilityRouteEpoch::new(runtime_id, 7, declared, normalized, observed).unwrap()
}

#[test]
fn live_registry_path_produces_exact_layers_and_restrictive_effective_truth() {
    let route = open_code_route();
    let declared = provider_declared_evidence(&provider_claim(route.clone(), NOW - 1_000)).unwrap();
    let normalized =
        opencode_adapter_evidence(&open_code_model_route(route.clone(), NOW - 900)).unwrap();
    let (conformance, health) = open_code_artifacts(NOW - 800);
    let observed = opencode_observed_evidence(
        &conformance,
        &health,
        &observation(route.clone(), NOW - 800),
    )
    .unwrap();

    let epoch =
        CapabilityRouteEpoch::new("opencode-primary", 7, declared, normalized, observed).unwrap();
    let mut registry = CapabilityEvidenceRegistry::new();
    assert_eq!(
        registry
            .replace_process_routes(0, "opencode-primary", 7, vec![epoch])
            .unwrap(),
        1
    );

    let layers = registry.layers(&route, NOW).unwrap();
    assert_eq!(layers[0].layer, CapabilityLayer::Declared);
    assert_eq!(layers[1].layer, CapabilityLayer::AdapterNormalized);
    assert_eq!(layers[2].layer, CapabilityLayer::Observed);
    assert!(layers.iter().all(|layer| layer.route == route));
    assert!(layers.iter().all(|layer| layer.features.len() == 26));

    let effective = registry.effective_for_route(&route, NOW).unwrap();
    assert_eq!(
        effective.feature_state(CapabilityKey::InputImage),
        CapabilityState::Unsupported
    );
    assert_eq!(
        effective.feature_state(CapabilityKey::ToolCalling),
        CapabilityState::Degraded,
        "OpenCode action-intent conformance remains degraded"
    );
    assert_eq!(
        effective.numeric_maximum(NumericCapabilityKey::ContextTokens),
        Some(100_000)
    );
}

#[test]
fn omitted_upstream_fields_stay_unknown_in_their_own_layer() {
    let route = open_code_route();
    let mut claim = provider_claim(route.clone(), NOW - 1_000);
    claim.features.clear();
    claim.numeric_limits.clear();
    let declared = provider_declared_evidence(&claim).unwrap();
    assert_eq!(
        declared
            .descriptor()
            .feature_state(CapabilityKey::ToolCalling),
        CapabilityState::Unknown
    );

    let normalized = opencode_adapter_evidence(&open_code_model_route(route, NOW - 900)).unwrap();
    assert_eq!(
        normalized
            .descriptor()
            .feature_state(CapabilityKey::Streaming),
        CapabilityState::Unknown,
        "provider declarations must not be copied into adapter omissions"
    );
}

#[test]
fn opencode_file_modalities_are_narrowed_to_verified_native_parts_and_inline_limit() {
    let route = open_code_route();
    let mut model = open_code_model_route(route, NOW - 900);
    for key in [CapabilityKey::InputImage, CapabilityKey::InputPdf] {
        model.capabilities.features.insert(
            key,
            normalized_evidence(CapabilityState::Supported, NOW - 900),
        );
    }

    let normalized = opencode_adapter_evidence(&model).unwrap();
    let image = normalized
        .descriptor()
        .features
        .get(&CapabilityKey::InputImage)
        .unwrap();
    assert_eq!(image.state, CapabilityState::Supported);
    assert_eq!(image.source, "c4os.opencode.verified-file-part");
    assert_eq!(image.allowed_values, ["image/png"]);
    let pdf = normalized
        .descriptor()
        .features
        .get(&CapabilityKey::InputPdf)
        .unwrap();
    assert_eq!(pdf.state, CapabilityState::Supported);
    assert_eq!(pdf.allowed_values, ["application/pdf"]);
    assert_eq!(
        normalized
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::AttachmentBytes),
        Some(256 * 1024)
    );
    assert_eq!(
        normalized
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::AttachmentCount),
        Some(32)
    );
    assert_eq!(
        normalized
            .descriptor()
            .feature_state(CapabilityKey::InputAudio),
        CapabilityState::Unsupported
    );
    assert_eq!(
        normalized
            .descriptor()
            .feature_state(CapabilityKey::InputVideo),
        CapabilityState::Unsupported
    );
}

#[test]
fn opencode_verified_file_transport_does_not_upgrade_unknown_or_unsupported_model_truth() {
    let normalized =
        opencode_adapter_evidence(&open_code_model_route(open_code_route(), NOW - 900)).unwrap();
    assert_eq!(
        normalized
            .descriptor()
            .feature_state(CapabilityKey::InputImage),
        CapabilityState::Unsupported
    );
    assert_eq!(
        normalized
            .descriptor()
            .feature_state(CapabilityKey::InputPdf),
        CapabilityState::Unknown
    );
}

#[test]
fn expiry_and_route_version_digest_time_substitutions_fail_closed() {
    let route = open_code_route();
    let (conformance, health) = open_code_artifacts(NOW - 800);
    let mut mismatched = observation(route.clone(), NOW - 800);
    mismatched.route.native_runtime_version = "1.18.4".into();
    assert!(matches!(
        opencode_observed_evidence(&conformance, &health, &mismatched),
        Err(CapabilityEvidenceError::ArtifactMismatch)
    ));

    let mut invalid_digest = provider_claim(route.clone(), NOW - 1_000);
    invalid_digest.raw_catalog_sha256 = "sha256:ABC".into();
    assert!(provider_declared_evidence(&invalid_digest).is_err());

    let mut bad_time = provider_claim(route.clone(), NOW - 1_000);
    bad_time.expires_at_ms = bad_time.declared_at_ms;
    assert!(provider_declared_evidence(&bad_time).is_err());

    let declared = provider_declared_evidence(&provider_claim(route.clone(), NOW - 1_000)).unwrap();
    let normalized =
        opencode_adapter_evidence(&open_code_model_route(route.clone(), NOW - 900)).unwrap();
    let observed = opencode_observed_evidence(
        &conformance,
        &health,
        &observation(route.clone(), NOW - 800),
    )
    .unwrap();
    let mut registry = CapabilityEvidenceRegistry::new();
    registry
        .replace_process_routes(
            0,
            "opencode-primary",
            7,
            vec![
                CapabilityRouteEpoch::new("opencode-primary", 7, declared, normalized, observed)
                    .unwrap(),
            ],
        )
        .unwrap();
    assert!(matches!(
        registry.layers(&route, NOW + 30_000),
        Err(CapabilityEvidenceError::Expired)
    ));

    let mut substituted_route = route;
    substituted_route.endpoint_id = "substituted-endpoint".into();
    assert!(matches!(
        registry.layers(&substituted_route, NOW),
        Err(CapabilityEvidenceError::RouteNotFound)
    ));
}

#[test]
fn complete_process_epoch_is_one_generation_and_invalidates_atomically() {
    let route = open_code_route();
    let mut registry = CapabilityEvidenceRegistry::new();
    assert_eq!(
        registry
            .replace_process_routes(
                0,
                "opencode-primary",
                7,
                vec![open_code_epoch(route.clone())]
            )
            .unwrap(),
        1
    );
    assert!(matches!(
        registry.replace_process_routes(
            0,
            "opencode-primary",
            7,
            vec![open_code_epoch(route.clone())],
        ),
        Err(CapabilityEvidenceError::StaleGeneration { .. })
    ));
    assert_eq!(registry.generation(), 1);
    assert_eq!(
        registry
            .invalidate_process(1, "opencode-primary", 7)
            .unwrap(),
        2
    );
    assert!(matches!(
        registry.layers(&route, NOW),
        Err(CapabilityEvidenceError::RouteNotFound)
    ));
    assert!(matches!(
        registry.invalidate_process(2, "opencode-primary", 7),
        Err(CapabilityEvidenceError::ProcessNotFound)
    ));
}

#[test]
fn older_replacement_is_rejected_and_restore_archives_active_truth() {
    let route = open_code_route();
    let current = open_code_epoch_for_runtime_at(
        route.clone(),
        "opencode-primary",
        NOW - 1_000,
        NOW - 900,
        NOW - 800,
    );
    let older = open_code_epoch_for_runtime_at(
        route.clone(),
        "opencode-primary",
        NOW - 2_000,
        NOW - 1_900,
        NOW - 1_800,
    );
    let mut registry = CapabilityEvidenceRegistry::new();
    registry
        .replace_process_routes(0, "opencode-primary", 7, vec![current.clone()])
        .unwrap();
    assert!(matches!(
        registry.replace_process_routes(1, "opencode-primary", 7, vec![older]),
        Err(CapabilityEvidenceError::NonMonotonicEvidence)
    ));
    assert_eq!(registry.generation(), 1);
    assert_eq!(registry.layers(&route, NOW).unwrap(), current.layers());

    let restored = CapabilityEvidenceRegistry::restore(registry.snapshot()).unwrap();
    let snapshot = restored.snapshot();
    assert_eq!(snapshot.generation, 2);
    assert!(snapshot.active_routes.is_empty());
    assert!(snapshot.active_processes.is_empty());
    assert_eq!(snapshot.historical_routes, vec![current]);
    assert_eq!(snapshot.historical_routes_dropped, 0);
    assert!(matches!(
        restored.layers(&route, NOW),
        Err(CapabilityEvidenceError::RouteNotFound)
    ));
}

#[test]
fn restore_compacts_legacy_history_to_the_newest_bounded_epochs() {
    let historical_routes = (0..MAX_HISTORICAL_EVIDENCE_ROUTES + 5)
        .map(|index| {
            let mut route = open_code_route();
            route.endpoint_id = format!("legacy-endpoint-{index}");
            open_code_epoch(route)
        })
        .collect::<Vec<_>>();
    let restored = CapabilityEvidenceRegistry::restore(CapabilityEvidenceSnapshot {
        generation: 7,
        active_routes: BTreeMap::new(),
        active_processes: BTreeMap::new(),
        historical_routes,
        historical_routes_dropped: 3,
    })
    .unwrap();

    let snapshot = restored.snapshot();
    assert_eq!(
        snapshot.historical_routes.len(),
        MAX_HISTORICAL_EVIDENCE_ROUTES
    );
    assert_eq!(snapshot.historical_routes_dropped, 8);
    assert_eq!(
        snapshot.historical_routes[0].route().endpoint_id,
        "legacy-endpoint-5"
    );
    assert_eq!(
        snapshot
            .historical_routes
            .last()
            .unwrap()
            .route()
            .endpoint_id,
        format!("legacy-endpoint-{}", MAX_HISTORICAL_EVIDENCE_ROUTES + 4)
    );
}

#[test]
fn invalid_batch_and_zero_route_processes_never_publish_partial_truth() {
    let route = open_code_route();
    let mut other_route = route.clone();
    other_route.endpoint_id = "other-endpoint".into();
    let mut registry = CapabilityEvidenceRegistry::new();
    assert!(matches!(
        registry.replace_process_routes(
            0,
            "opencode-primary",
            7,
            vec![
                open_code_epoch(route.clone()),
                open_code_epoch_for_runtime(other_route, "opencode-other"),
            ],
        ),
        Err(CapabilityEvidenceError::ArtifactMismatch)
    ));
    assert_eq!(registry.generation(), 0);
    assert!(matches!(
        registry.layers(&route, NOW),
        Err(CapabilityEvidenceError::RouteNotFound)
    ));

    assert_eq!(
        registry
            .replace_process_routes(0, "opencode-primary", 7, vec![])
            .unwrap(),
        1,
        "a healthy zero-provider process still owns an explicit empty epoch"
    );
    assert_eq!(
        registry
            .invalidate_process(1, "opencode-primary", 7)
            .unwrap(),
        2
    );
}

#[test]
fn registry_route_count_is_strictly_bounded() {
    let mut registry = CapabilityEvidenceRegistry::new();
    let mut epochs = Vec::new();
    for index in 0..MAX_EVIDENCE_ROUTES {
        let mut route = open_code_route();
        route.endpoint_id = format!("endpoint-{index}");
        epochs.push(open_code_epoch(route));
    }
    registry
        .replace_process_routes(0, "opencode-primary", 7, epochs)
        .unwrap();
    let mut overflow_route = open_code_route();
    overflow_route.endpoint_id = "endpoint-overflow".into();
    let mut overflow = (0..MAX_EVIDENCE_ROUTES)
        .map(|index| {
            let mut route = open_code_route();
            route.endpoint_id = format!("replacement-endpoint-{index}");
            open_code_epoch(route)
        })
        .collect::<Vec<_>>();
    overflow.push(open_code_epoch(overflow_route));
    assert!(matches!(
        registry.replace_process_routes(1, "opencode-primary", 7, overflow),
        Err(CapabilityEvidenceError::CapacityExceeded)
    ));
    assert_eq!(registry.generation(), 1);
}

#[test]
fn pi_artifacts_preserve_supported_degraded_and_unknown_differences() {
    let route = RouteIdentity {
        provider_id: "anthropic".into(),
        endpoint_id: "anthropic-api".into(),
        provider_model_id: "anthropic/claude-sonnet".into(),
        model_revision: "2026-07-19".into(),
        adapter_kind: "pi".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "pi".into(),
        native_runtime_version: "0.80.10".into(),
        session_configuration_sha256: digest('f'),
    };
    let model = PiModelRoute {
        provider: "anthropic".into(),
        model_id: "claude-sonnet".into(),
        base_url: "https://api.anthropic.com".into(),
    };
    let adapter = pi_adapter_evidence(&model, &route, NOW - 900, NOW + 60_000).unwrap();
    let image = adapter
        .descriptor()
        .features
        .get(&CapabilityKey::InputImage)
        .unwrap();
    assert_eq!(image.state, CapabilityState::Supported);
    assert_eq!(
        image.allowed_values,
        ["image/jpeg", "image/png", "image/gif", "image/webp"]
    );
    assert_eq!(
        adapter.descriptor().feature_state(CapabilityKey::InputPdf),
        CapabilityState::Unsupported
    );
    assert_eq!(
        adapter
            .descriptor()
            .feature_state(CapabilityKey::InputAudio),
        CapabilityState::Unsupported
    );
    assert_eq!(
        adapter
            .descriptor()
            .feature_state(CapabilityKey::InputVideo),
        CapabilityState::Unsupported
    );
    assert_eq!(
        adapter
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::AttachmentBytes),
        Some(128 * 1024)
    );
    assert_eq!(
        adapter
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::AttachmentCount),
        Some(32)
    );
    for key in [
        CapabilityKey::InputText,
        CapabilityKey::OutputText,
        CapabilityKey::Streaming,
        CapabilityKey::ToolCalling,
    ] {
        assert_eq!(
            adapter.descriptor().feature_state(key),
            CapabilityState::Supported
        );
    }
    assert_eq!(
        adapter
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::ContextTokens),
        None,
        "unverified Pi catalog routes must retain fail-closed token limits"
    );

    let openai_route = RouteIdentity {
        provider_id: "openai".into(),
        endpoint_id: "openai-api".into(),
        provider_model_id: "openai/gpt-4o-mini".into(),
        model_revision: "pi-catalog-0.80.10".into(),
        adapter_kind: "pi".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "pi".into(),
        native_runtime_version: "0.80.10".into(),
        session_configuration_sha256: digest('e'),
    };
    let openai_model = PiModelRoute {
        provider: "openai".into(),
        model_id: "gpt-4o-mini".into(),
        base_url: "https://api.openai.com/v1".into(),
    };
    let openai_adapter =
        pi_adapter_evidence(&openai_model, &openai_route, NOW - 900, NOW + 60_000).unwrap();
    assert_eq!(
        openai_adapter
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::ContextTokens),
        Some(128_000)
    );
    assert_eq!(
        openai_adapter
            .descriptor()
            .numeric_maximum(NumericCapabilityKey::OutputTokens),
        Some(16_384)
    );

    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi");
    let conformance = PiSidecarManifest::load(&sidecar_root)
        .unwrap()
        .conformance_descriptor(7, true)
        .unwrap();
    let supported = |state: &str| PiCapabilityState {
        state: state.into(),
        reason: None,
    };
    let health = PiHealth {
        status: "ready".into(),
        runtime: "pi".into(),
        transport: "c4os-node-sdk-sidecar".into(),
        protocol: "c4os.pi.ndjson.v1".into(),
        process_generation: 7,
        sessions: 1,
        stale_events_rejected: 0,
        capabilities: BTreeMap::from([
            ("streaming".into(), supported("supported")),
            ("cancellation".into(), supported("supported")),
            ("tools".into(), supported("supported")),
            ("nativePersistence".into(), supported("unsupported")),
            ("nativeExtensions".into(), supported("unsupported")),
            ("nativeTools".into(), supported("unsupported")),
            ("crashResume".into(), supported("degraded")),
            ("providerAuthentication".into(), supported("supported")),
            ("rpcTransport".into(), supported("supported")),
        ]),
    };
    let mut observation = observation(route.clone(), NOW - 800);
    observation.runtime_id = "pi-primary".into();
    observation.health_checked_at_ms = observation.observed_at_ms;
    observation.features.remove(&CapabilityKey::InputImage);
    observation.features.insert(
        CapabilityKey::SessionAffinity,
        feature(CapabilityState::Supported, None),
    );
    let observed = pi_observed_evidence(&conformance, &health, &observation).unwrap();
    let mut declaration = provider_claim(route, NOW - 1_000);
    declaration.features.remove(&CapabilityKey::InputImage);
    let declared = provider_declared_evidence(&declaration).unwrap();
    let effective = effective_intersection(
        &[
            declared.descriptor().clone(),
            adapter.descriptor().clone(),
            observed.descriptor().clone(),
        ],
        NOW,
    )
    .unwrap();
    assert_eq!(
        effective.feature_state(CapabilityKey::InputImage),
        CapabilityState::Unknown,
        "verified Pi transport must not upgrade unknown provider and runtime model truth"
    );
    assert_eq!(
        observed
            .descriptor()
            .feature_state(CapabilityKey::ToolCalling),
        CapabilityState::Supported
    );
    assert_eq!(
        observed
            .descriptor()
            .feature_state(CapabilityKey::SessionAffinity),
        CapabilityState::Degraded
    );
    assert_eq!(
        observed
            .descriptor()
            .feature_state(CapabilityKey::OutputAudio),
        CapabilityState::Unknown
    );
}

#[test]
fn pi_gpt_4o_mini_limits_match_the_exact_vendored_catalog() {
    let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../sidecars/pi/node_modules/@earendil-works/pi-ai/dist/providers/openai.models.js",
    );
    let catalog = std::fs::read_to_string(catalog_path).expect("vendored Pi OpenAI catalog");
    let model = catalog
        .split_once("\"gpt-4o-mini\": {")
        .expect("gpt-4o-mini catalog entry")
        .1
        .split_once("\n    },")
        .expect("bounded gpt-4o-mini catalog entry")
        .0;
    for exact in [
        "api: \"openai-responses\"",
        "provider: \"openai\"",
        "input: [\"text\", \"image\"]",
        "contextWindow: 128000",
        "maxTokens: 16384",
    ] {
        assert!(model.contains(exact), "Pi catalog drifted at {exact}");
    }
}
