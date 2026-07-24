use std::collections::BTreeMap;

use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, ModelLifecycle, RouteIdentity,
};
use c4os_lib::runtime::provider::{
    ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, PROVIDER_SCHEMA_VERSION,
    ProviderAuthentication, ProviderConnectionEvidence, ProviderDiscovery, ProviderEndpoint,
    ProviderError, ProviderFeatureClaim, ProviderFieldKey, ProviderKind, ProviderModelDeclaration,
    ProviderNumericClaim, ProviderProbe, ProviderProbeFailure, ProviderProfile, ProviderService,
    ProviderSnapshot, ProviderTestStatus, RouteAvailability,
};
use c4os_lib::security::credentials::CredentialVault;

const NOW: u64 = 1_721_300_000_000;

struct FixtureProbe(Result<ProviderDiscovery, ProviderProbeFailure>);

impl ProviderProbe for FixtureProbe {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let mut result = self.0.clone();
        if let Ok(discovery) = &mut result {
            discovery.connection_evidence = Some(ProviderConnectionEvidence::from_tested_profile(
                profile,
                discovery.checked_at_ms,
                digest('c'),
            )?);
        }
        result
    }
}

fn digest(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}

fn profile() -> ProviderProfile {
    let vault = CredentialVault::session_only().unwrap();
    let credential_reference = vault
        .store("openrouter-api-key", b"fixture-secret")
        .unwrap();
    ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: "provider-openrouter".into(),
        kind: ProviderKind::OpenRouter,
        display_name: "OpenRouter".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "openrouter-chat".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            api_kind: "openai-compatible".into(),
        },
        authentication: ProviderAuthentication::Bearer,
        credential_reference: Some(credential_reference),
        headers: BTreeMap::new(),
        enabled: true,
    }
}

fn tested_service() -> (ProviderService, ProviderProfile, u64) {
    let candidate = profile();
    let mut service = ProviderService::new();
    service.save_profile(candidate.clone(), 0).unwrap();
    let report = service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("model-a", 1, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();
    (service, candidate, report.generation)
}

fn assert_profile_edit_invalidates_latest_test(edit: impl FnOnce(&mut ProviderProfile)) {
    let (mut service, mut candidate, tested_generation) = tested_service();
    edit(&mut candidate);

    service
        .save_profile(candidate, tested_generation)
        .expect("a valid relevant edit should save");
    let snapshot = service.snapshot();
    let record = &snapshot.providers[0];
    assert!(matches!(record.test_status, ProviderTestStatus::Untested));
    assert!(record.connection_evidence.is_none());
    assert!(record.models.is_empty());
    assert!(record.selected_model_id.is_none());
    assert!(!snapshot.onboarding_ready_at(NOW));
}

fn route(model_id: &str, rank: u32, availability: RouteAvailability) -> ModelRoute {
    let profile = profile();
    let mut features = BTreeMap::new();
    features.insert(
        CapabilityKey::InputText,
        CapabilityEvidence {
            state: CapabilityState::Supported,
            layer: CapabilityLayer::AdapterNormalized,
            source: "fixture.opencode-adapter".into(),
            checked_at_ms: NOW,
            expires_at_ms: Some(NOW + 60_000),
            constraints: vec![],
            allowed_values: vec![],
            reason: None,
        },
    );
    ModelRoute {
        model_id: model_id.into(),
        display_name: model_id.replace('-', " "),
        recommendation_rank: rank,
        availability,
        checked_at_ms: NOW,
        capabilities: CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::AdapterNormalized,
            route: RouteIdentity {
                provider_id: profile.provider_id,
                endpoint_id: profile.endpoint.endpoint_id,
                provider_model_id: format!("anthropic/{model_id}"),
                model_revision: "2026-07-18".into(),
                adapter_kind: "opencode".into(),
                adapter_version: "1.0.0".into(),
                runtime_kind: "opencode".into(),
                native_runtime_version: "1.18.3".into(),
                session_configuration_sha256: digest('a'),
            },
            lifecycle: ModelLifecycle::Active,
            features,
            numeric_limits: BTreeMap::new(),
            raw_evidence_sha256: digest('b'),
        },
        provider_declaration: Some(ProviderModelDeclaration {
            schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
            provider_model_id: model_id.into(),
            model_revision: "provider-catalog-2026-07-18".into(),
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::from([
                (
                    CapabilityKey::InputText,
                    ProviderFeatureClaim {
                        state: CapabilityState::Supported,
                        constraints: vec![],
                        allowed_values: vec![],
                        reason: None,
                    },
                ),
                (
                    CapabilityKey::InputImage,
                    ProviderFeatureClaim {
                        state: CapabilityState::Supported,
                        constraints: vec![],
                        allowed_values: vec!["image/png".into()],
                        reason: None,
                    },
                ),
            ]),
            numeric_limits: BTreeMap::from([(
                c4os_lib::runtime::capability::NumericCapabilityKey::ContextTokens,
                ProviderNumericClaim {
                    state: CapabilityState::Supported,
                    maximum: Some(200_000),
                    confidence: c4os_lib::runtime::capability::LimitConfidence::Confirmed,
                    reason: None,
                },
            )]),
            raw_catalog_sha256: digest('d'),
            declared_at_ms: NOW,
            expires_at_ms: NOW + 60_000,
        }),
    }
}

fn discovery(models: Vec<ModelRoute>, recommended: Option<&str>) -> ProviderDiscovery {
    ProviderDiscovery {
        checked_at_ms: NOW,
        models,
        recommended_model_id: recommended.map(str::to_owned),
        connection_evidence: None,
    }
}

#[test]
fn provider_fields_are_kind_specific_and_mark_secrets() {
    let built_in = ProviderKind::Anthropic.fields();
    assert_eq!(built_in.len(), 1);
    assert_eq!(built_in[0].key, ProviderFieldKey::Credential);
    assert!(built_in[0].required && built_in[0].secret);

    let custom = ProviderKind::Custom.fields();
    assert_eq!(custom.len(), 2);
    assert_eq!(custom[0].key, ProviderFieldKey::Endpoint);
    assert!(!custom[0].secret);
    assert!(custom[1].secret);
}

#[test]
fn zero_model_success_is_not_onboarding_ready() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    let report = service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(vec![], None))),
        )
        .unwrap();

    assert_eq!(report.discovered_models, 0);
    assert!(matches!(
        report.status,
        ProviderTestStatus::SucceededNoUsableModels { .. }
    ));
    assert!(!service.snapshot().onboarding_ready_at(NOW));
}

#[test]
fn catalog_only_discovery_cannot_mark_a_provider_successful() {
    struct UnverifiedCatalogProbe(ProviderDiscovery);
    impl ProviderProbe for UnverifiedCatalogProbe {
        fn test_and_discover(
            &mut self,
            _profile: &ProviderProfile,
        ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
            Ok(self.0.clone())
        }
    }

    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    let result = service.test_provider(
        "provider-openrouter",
        1,
        NOW,
        &mut UnverifiedCatalogProbe(discovery(
            vec![route("model-a", 1, RouteAvailability::Available)],
            None,
        )),
    );
    assert!(matches!(
        result,
        Err(c4os_lib::runtime::provider::ProviderError::MissingConnectionProof)
    ));
    let snapshot = service.snapshot();
    assert!(matches!(
        snapshot.providers[0].test_status,
        ProviderTestStatus::Untested
    ));
    assert!(!snapshot.onboarding_ready_at(NOW));
}

#[test]
fn one_usable_model_is_selected_and_explicitly_confirms_readiness() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    let report = service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("claude-sonnet", 10, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();

    assert_eq!(report.selected_model_id.as_deref(), Some("claude-sonnet"));
    assert!(service.snapshot().onboarding_ready_at(NOW));
}

#[test]
fn onboarding_readiness_is_bound_to_the_requested_provider_and_model() {
    let (service, _candidate, _tested_generation) = tested_service();
    let snapshot = service.snapshot();

    assert!(snapshot.onboarding_ready_at(NOW));
    assert!(snapshot.provider_model_ready_at("provider-openrouter", "model-a", NOW));
    assert!(!snapshot.provider_model_ready_at("provider-missing", "model-a", NOW));
    assert!(!snapshot.provider_model_ready_at("provider-openrouter", "model-missing", NOW));
}

#[test]
fn launch_requires_explicit_onboarding_completion() {
    let (mut service, _candidate, tested_generation) = tested_service();
    let tested = service.snapshot();
    assert!(tested.onboarding_ready_at(NOW));
    assert!(!tested.launch_ready());
    assert_eq!(tested.onboarding_completed_at_ms, None);

    let completed_generation = service.complete_onboarding(tested_generation, NOW).unwrap();
    let completed = service.snapshot();
    assert_eq!(completed.generation, completed_generation);
    assert_eq!(completed.onboarding_completed_at_ms, Some(NOW));
    assert!(completed.launch_ready());
}

#[test]
fn completed_onboarding_persists_after_test_freshness_expires() {
    let (mut service, _candidate, tested_generation) = tested_service();
    service.complete_onboarding(tested_generation, NOW).unwrap();

    let stale_at = NOW + 5 * 60 * 1_000 + 1;
    let completed = service.snapshot();
    assert!(!completed.onboarding_ready_at(stale_at));
    assert!(completed.launch_ready());

    let restored = ProviderService::restore(completed).unwrap();
    assert!(restored.snapshot().launch_ready());
}

#[test]
fn deleting_the_last_provider_clears_onboarding_completion() {
    let (mut service, _candidate, tested_generation) = tested_service();
    let completed_generation = service.complete_onboarding(tested_generation, NOW).unwrap();
    assert!(service.snapshot().launch_ready());

    service
        .delete_provider("provider-openrouter", completed_generation)
        .unwrap();
    let deleted = service.snapshot();
    assert!(deleted.providers.is_empty());
    assert_eq!(deleted.onboarding_completed_at_ms, None);
    assert!(!deleted.launch_ready());
}

#[test]
fn relevant_auth_header_endpoint_and_credential_edits_invalidate_latest_test() {
    assert_profile_edit_invalidates_latest_test(|candidate| {
        candidate.authentication = ProviderAuthentication::ApiKeyHeader {
            header_name: "x-provider-key".into(),
        };
    });
    assert_profile_edit_invalidates_latest_test(|candidate| {
        candidate
            .headers
            .insert("x-tenant-id".into(), "tenant-b".into());
    });
    assert_profile_edit_invalidates_latest_test(|candidate| {
        candidate.kind = ProviderKind::Custom;
        candidate.endpoint.base_url = "https://openrouter.example/api/v1".into();
        candidate.endpoint.api_kind = "openai-compatible".into();
    });
    assert_profile_edit_invalidates_latest_test(|candidate| {
        let vault = CredentialVault::session_only().unwrap();
        candidate.credential_reference = Some(
            vault
                .store("replacement-openrouter-key", b"replacement-secret")
                .unwrap(),
        );
    });
}

#[test]
fn custom_http_endpoints_accept_only_numeric_loopback_hosts() {
    let mut candidate = profile();
    candidate.kind = ProviderKind::Custom;
    candidate.endpoint.api_kind = "openai-compatible".into();

    for endpoint in [
        "http://127.0.0.1/v1",
        "http://127.0.0.1:8080/v1",
        "http://[::1]/v1",
        "http://[::1]:8080/v1",
    ] {
        candidate.endpoint.base_url = endpoint.into();
        assert!(
            candidate.validate().is_ok(),
            "expected {endpoint} to be valid"
        );
    }

    for endpoint in [
        "http://localhost:8080/v1",
        "http://127.0.0.2:8080/v1",
        "http://[::2]:8080/v1",
        "http://provider.example/v1",
    ] {
        candidate.endpoint.base_url = endpoint.into();
        assert!(matches!(
            candidate.validate(),
            Err(ProviderError::InvalidEndpoint)
        ));
    }

    candidate.kind = ProviderKind::OpenAi;
    candidate.endpoint.base_url = "http://127.0.0.1:8080/v1".into();
    assert!(matches!(
        candidate.validate(),
        Err(ProviderError::InvalidEndpoint)
    ));
}

#[test]
fn authentication_and_credential_presence_must_agree() {
    let mut candidate = profile();
    candidate.authentication = ProviderAuthentication::None;
    assert!(matches!(
        candidate.validate(),
        Err(ProviderError::InvalidProfile)
    ));

    candidate.credential_reference = None;
    assert!(candidate.validate().is_ok());

    candidate.authentication = ProviderAuthentication::Bearer;
    assert!(matches!(
        candidate.validate(),
        Err(ProviderError::InvalidProfile)
    ));
}

#[test]
fn many_models_honor_a_valid_upstream_recommendation_and_allow_revision() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    let report = service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![
                    route("model-a", 1, RouteAvailability::Available),
                    route("model-b", 20, RouteAvailability::Available),
                    route("model-c", 0, RouteAvailability::Unavailable),
                ],
                Some("model-b"),
            ))),
        )
        .unwrap();
    assert_eq!(report.selected_model_id.as_deref(), Some("model-b"));

    let generation = service
        .select_model("provider-openrouter", "model-a", report.generation)
        .unwrap();
    let snapshot = service.snapshot();
    assert_eq!(snapshot.generation, generation);
    assert_eq!(
        snapshot.providers[0].selected_model_id.as_deref(),
        Some("model-a")
    );
    assert!(
        service
            .select_model("provider-openrouter", "model-c", generation)
            .is_err()
    );
}

#[test]
fn failed_retest_clears_selection_and_downgrades_cached_routes() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    let first = service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("model-a", 1, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();
    let failed = service
        .test_provider(
            "provider-openrouter",
            first.generation,
            NOW + 1,
            &mut FixtureProbe(Err(ProviderProbeFailure::Authentication)),
        )
        .unwrap();
    assert!(matches!(failed.status, ProviderTestStatus::Failed { .. }));
    let snapshot = service.snapshot();
    assert_eq!(snapshot.providers[0].selected_model_id, None);
    assert_eq!(
        snapshot.providers[0].models["model-a"].availability,
        RouteAvailability::Unknown
    );
    assert!(!snapshot.onboarding_ready_at(NOW + 1));
}

#[test]
fn stale_updates_and_credential_bearing_urls_fail_closed() {
    let mut service = ProviderService::new();
    let mut candidate = profile();
    service.save_profile(candidate.clone(), 0).unwrap();
    assert!(service.save_profile(candidate.clone(), 0).is_err());

    candidate.endpoint.base_url = "https://secret@example.com/api".into();
    assert!(candidate.validate().is_err());
    candidate.endpoint.base_url = "http://example.com/api".into();
    assert!(candidate.validate().is_err());
    candidate.endpoint.base_url = "https://example.com/api?key=secret".into();
    assert!(candidate.validate().is_err());
}

#[test]
fn serialized_product_state_contains_only_the_opaque_reference() {
    let candidate = profile();
    let serialized = serde_json::to_string(&candidate).unwrap();
    assert!(serialized.contains("credential:"));
    assert!(!serialized.contains("fixture-secret"));
}

#[test]
fn model_declaration_round_trips_with_runtime_neutral_provider_claims() {
    let candidate = route("model-a", 1, RouteAvailability::Available);
    let serialized = serde_json::to_string(&candidate).unwrap();
    assert!(serialized.contains("providerDeclaration"));
    let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    let declaration = &value["providerDeclaration"];
    assert!(declaration.get("adapterKind").is_none());
    assert!(declaration.get("runtimeKind").is_none());
    assert!(declaration.get("nativeRuntimeVersion").is_none());

    let restored: ModelRoute = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored, candidate);
    assert!(restored.is_production_ready());
}

#[test]
fn legacy_route_without_declaration_restores_but_requires_retest_for_readiness() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("model-a", 1, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();

    let mut serialized = serde_json::to_value(service.snapshot()).unwrap();
    serialized["providers"][0]["models"]["model-a"]
        .as_object_mut()
        .unwrap()
        .remove("providerDeclaration");
    let legacy_snapshot: ProviderSnapshot = serde_json::from_value(serialized).unwrap();
    assert!(legacy_snapshot.providers[0].models["model-a"].is_usable());
    assert!(!legacy_snapshot.providers[0].models["model-a"].is_production_ready());

    let generation = legacy_snapshot.generation;
    let mut restored = ProviderService::restore(legacy_snapshot).unwrap();
    assert!(!restored.snapshot().onboarding_ready_at(NOW));
    restored
        .test_provider(
            "provider-openrouter",
            generation,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("model-a", 1, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();
    assert!(restored.snapshot().onboarding_ready_at(NOW));
}

#[test]
fn provider_declaration_remains_distinct_from_opencode_adapter_evidence() {
    let route = route("model-a", 1, RouteAvailability::Available);
    let declaration = route.provider_declaration.as_ref().unwrap();
    assert_eq!(
        declaration.features[&CapabilityKey::InputImage].state,
        CapabilityState::Supported
    );
    assert_eq!(
        route.capabilities.feature_state(CapabilityKey::InputImage),
        CapabilityState::Unknown
    );
    assert_eq!(route.capabilities.layer, CapabilityLayer::AdapterNormalized);
    assert_ne!(
        declaration.raw_catalog_sha256,
        route.capabilities.raw_evidence_sha256
    );
}

#[test]
fn onboarding_requires_a_fresh_test_and_reenable_requires_retest() {
    let mut service = ProviderService::new();
    let mut candidate = profile();
    service.save_profile(candidate.clone(), 0).unwrap();
    let report = service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("model-a", 1, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();
    assert!(service.snapshot().onboarding_ready_at(NOW + 1));
    assert!(
        !service
            .snapshot()
            .onboarding_ready_at(NOW + 5 * 60 * 1_000 + 1)
    );

    candidate.enabled = false;
    let disabled_generation = service
        .save_profile(candidate.clone(), report.generation)
        .unwrap();
    candidate.enabled = true;
    service
        .save_profile(candidate, disabled_generation)
        .unwrap();
    let restored = service.snapshot();
    assert!(matches!(
        restored.providers[0].test_status,
        ProviderTestStatus::Untested
    ));
    assert!(restored.providers[0].models.is_empty());
    assert!(!restored.onboarding_ready_at(NOW + 1));
}

#[test]
fn provider_display_names_are_unique_case_insensitively() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    let mut duplicate = profile();
    duplicate.provider_id = "provider-second".into();
    duplicate.endpoint.endpoint_id = "second-endpoint".into();
    duplicate.display_name = "  openrouter  ".into();
    assert!(matches!(
        service.save_profile(duplicate, 1),
        Err(c4os_lib::runtime::provider::ProviderError::DuplicateDisplayName)
    ));
}

#[test]
fn runtime_provider_identity_is_profile_qualified_for_opencode_and_kind_bound_for_pi() {
    let mut first = profile();
    first.provider_id = "openai-team-a".into();
    first.kind = ProviderKind::OpenAi;
    first.endpoint.base_url = "https://api.openai.com/v1".into();
    first.endpoint.api_kind = "openai".into();
    let mut second = first.clone();
    second.provider_id = "openai-team-b".into();

    assert_eq!(first.opencode_native_provider_id(), "openai-team-a");
    assert_eq!(second.opencode_native_provider_id(), "openai-team-b");
    assert_eq!(first.pi_native_provider_id(), Some("openai"));
    assert_eq!(second.pi_native_provider_id(), Some("openai"));

    for (kind, base_url, api_kind, expected) in [
        (
            ProviderKind::Anthropic,
            "https://api.anthropic.com",
            "anthropic",
            "anthropic",
        ),
        (
            ProviderKind::Gemini,
            "https://generativelanguage.googleapis.com/v1beta",
            "google",
            "google",
        ),
        (
            ProviderKind::OpenRouter,
            "https://openrouter.ai/api/v1",
            "openai-compatible",
            "openrouter",
        ),
    ] {
        let mut standard = profile();
        standard.kind = kind;
        standard.endpoint.base_url = base_url.into();
        standard.endpoint.api_kind = api_kind.into();
        assert_eq!(standard.pi_native_provider_id(), Some(expected));
    }

    let mut endpoint_substitution = second.clone();
    endpoint_substitution.endpoint.base_url = "https://proxy.example/v1".into();
    assert_eq!(endpoint_substitution.pi_native_provider_id(), None);
    let mut api_substitution = second.clone();
    api_substitution.endpoint.api_kind = "openai-compatible".into();
    assert_eq!(api_substitution.pi_native_provider_id(), None);

    let mut custom = first;
    custom.kind = ProviderKind::Custom;
    custom.endpoint.base_url = "https://proxy.example/v1".into();
    assert_eq!(custom.pi_native_provider_id(), None);
    custom.endpoint.api_kind = "openai-compatible".into();
    assert_eq!(custom.pi_native_provider_id(), Some("openai"));
}

#[test]
fn restored_success_requires_connection_evidence_for_the_exact_saved_profile() {
    let mut service = ProviderService::new();
    service.save_profile(profile(), 0).unwrap();
    service
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(Ok(discovery(
                vec![route("model-a", 1, RouteAvailability::Available)],
                None,
            ))),
        )
        .unwrap();

    let mut endpoint_substitution = service.snapshot();
    endpoint_substitution.providers[0].profile.endpoint.base_url =
        "https://substitute.example/api/v1".into();
    assert!(matches!(
        ProviderService::restore(endpoint_substitution),
        Err(c4os_lib::runtime::provider::ProviderError::InvalidSnapshot)
    ));

    let mut credential_substitution = service.snapshot();
    credential_substitution.providers[0]
        .profile
        .credential_reference = profile().credential_reference;
    assert!(matches!(
        ProviderService::restore(credential_substitution),
        Err(c4os_lib::runtime::provider::ProviderError::InvalidSnapshot)
    ));
}
