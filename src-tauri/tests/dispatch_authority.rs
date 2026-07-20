use std::collections::BTreeMap;

use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, LimitConfidence, ModelLifecycle, NumericCapabilityEvidence,
    NumericCapabilityKey, RouteIdentity,
};
use c4os_lib::runtime::dispatch_authority::{
    AuthoritativeConfiguration, AuthoritativeResource, AuthoritativeResources, AuthorityMintIntent,
    ConfigurationFieldValue, DispatchAuthorityError, DispatchAuthorityRegistry,
    RuntimeProcessTruth, WorkerDispatchAuthority, authoritative_configuration_sha256,
};
use c4os_lib::runtime::session::{AdapterBinding, ExecutionEnvironmentBinding, RuntimeKind};

const CAPABILITY_GENERATION: u64 = 17;

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn configuration(version: u64) -> AuthoritativeConfiguration {
    AuthoritativeConfiguration {
        version,
        fields: BTreeMap::from([
            (
                "provider-id".into(),
                ConfigurationFieldValue::Text("provider-anthropic".into()),
            ),
            (
                "endpoint-id".into(),
                ConfigurationFieldValue::Text("endpoint-default".into()),
            ),
            (
                "model-id".into(),
                ConfigurationFieldValue::Text("anthropic/claude-sonnet".into()),
            ),
            (
                "model-revision".into(),
                ConfigurationFieldValue::Text("2026-07-01".into()),
            ),
            (
                "adapter-kind".into(),
                ConfigurationFieldValue::Text("opencode".into()),
            ),
            (
                "adapter-version".into(),
                ConfigurationFieldValue::Text("1.0.0".into()),
            ),
            (
                "runtime-kind".into(),
                ConfigurationFieldValue::Text("opencode".into()),
            ),
            (
                "native-runtime-version".into(),
                ConfigurationFieldValue::Text("1.18.3".into()),
            ),
            (
                "credential".into(),
                ConfigurationFieldValue::CredentialReference("credential:anthropic".into()),
            ),
        ]),
    }
}

fn resources(version: u64) -> AuthoritativeResources {
    AuthoritativeResources {
        version,
        records: BTreeMap::from([
            (
                "mcp:workspace-files".into(),
                AuthoritativeResource {
                    kind: "mcp".into(),
                    version: 3,
                    sha256: digest('a'),
                },
            ),
            (
                "skill:project-review".into(),
                AuthoritativeResource {
                    kind: "skill".into(),
                    version: 5,
                    sha256: digest('b'),
                },
            ),
        ]),
    }
}

fn authority() -> WorkerDispatchAuthority {
    let configuration = configuration(4);
    WorkerDispatchAuthority {
        workspace_id: "workspace-1".into(),
        project_id: Some("project-1".into()),
        runtime_id: "opencode-primary".into(),
        runtime_kind: RuntimeKind::OpenCode,
        adapter: AdapterBinding {
            adapter_id: "adapter-opencode".into(),
            adapter_version: "1.0.0".into(),
            native_version: "1.18.3".into(),
        },
        environment: ExecutionEnvironmentBinding {
            environment_id: "local".into(),
            environment_kind: "local".into(),
            host_alias: None,
        },
        route: RouteIdentity {
            provider_id: "provider-anthropic".into(),
            endpoint_id: "endpoint-default".into(),
            provider_model_id: "anthropic/claude-sonnet".into(),
            model_revision: "2026-07-01".into(),
            adapter_kind: "opencode".into(),
            adapter_version: "1.0.0".into(),
            runtime_kind: "opencode".into(),
            native_runtime_version: "1.18.3".into(),
            session_configuration_sha256: authoritative_configuration_sha256(&configuration)
                .unwrap(),
        },
        configuration,
        resources: resources(9),
        process_generation: 12,
    }
}

fn effective(route: RouteIdentity, reverse: bool) -> CapabilityDescriptor {
    let mut constraints = vec!["policy:trusted-root".into(), "runtime:broker-only".into()];
    let mut allowed_values = vec!["medium".into(), "high".into()];
    if reverse {
        constraints.reverse();
        allowed_values.reverse();
    }
    CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Effective,
        route,
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::from([(
            CapabilityKey::Reasoning,
            CapabilityEvidence {
                state: CapabilityState::Supported,
                layer: CapabilityLayer::Effective,
                source: "c4os.effective-intersection".into(),
                checked_at_ms: 100,
                expires_at_ms: Some(1_000),
                constraints,
                allowed_values,
                reason: None,
            },
        )]),
        numeric_limits: BTreeMap::from([(
            NumericCapabilityKey::ContextTokens,
            NumericCapabilityEvidence {
                evidence: CapabilityEvidence {
                    state: CapabilityState::Supported,
                    layer: CapabilityLayer::Effective,
                    source: "c4os.effective-intersection".into(),
                    checked_at_ms: 100,
                    expires_at_ms: Some(1_000),
                    constraints: vec!["route:confirmed".into()],
                    allowed_values: Vec::new(),
                    reason: None,
                },
                maximum: Some(200_000),
                confidence: LimitConfidence::Confirmed,
            },
        )]),
        raw_evidence_sha256: digest('c'),
    }
}

fn intent(authority_generation: u64) -> AuthorityMintIntent {
    AuthorityMintIntent {
        workspace_id: "workspace-1".into(),
        project_id: Some("project-1".into()),
        runtime_id: "opencode-primary".into(),
        expected_authority_generation: authority_generation,
        expected_capability_generation: CAPABILITY_GENERATION,
    }
}

fn process() -> RuntimeProcessTruth {
    RuntimeProcessTruth {
        runtime_id: "opencode-primary".into(),
        runtime_kind: RuntimeKind::OpenCode,
        adapter_version: "1.0.0".into(),
        native_version: "1.18.3".into(),
        process_generation: 12,
    }
}

fn authority_for_model(
    provider_id: &str,
    endpoint_id: &str,
    provider_model_id: &str,
) -> WorkerDispatchAuthority {
    let mut authority = authority();
    authority.route.provider_id = provider_id.into();
    authority.route.endpoint_id = endpoint_id.into();
    authority.route.provider_model_id = provider_model_id.into();
    authority.configuration.fields.insert(
        "provider-id".into(),
        ConfigurationFieldValue::Text(provider_id.into()),
    );
    authority.configuration.fields.insert(
        "endpoint-id".into(),
        ConfigurationFieldValue::Text(endpoint_id.into()),
    );
    authority.configuration.fields.insert(
        "model-id".into(),
        ConfigurationFieldValue::Text(provider_model_id.into()),
    );
    authority.route.session_configuration_sha256 =
        authoritative_configuration_sha256(&authority.configuration).unwrap();
    authority
}

#[test]
fn deterministic_minting_canonicalizes_complete_capability_truth() {
    let registry = DispatchAuthorityRegistry::new();
    let installed = authority();
    let expected_configuration_sha256 =
        authoritative_configuration_sha256(&installed.configuration).unwrap();
    assert_eq!(
        registry.install_from_worker(0, installed.clone()).unwrap(),
        1
    );
    let first = registry
        .mint_first_binding(
            &intent(1),
            &effective(installed.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        )
        .unwrap();
    let reordered = registry
        .mint_first_binding(
            &intent(1),
            &effective(installed.route, true),
            CAPABILITY_GENERATION,
            &process(),
            500,
        )
        .unwrap();

    assert_eq!(first, reordered);
    assert_eq!(first.binding.initial_configuration.version, 4);
    assert_eq!(first.binding.initial_resources.version, 9);
    assert_eq!(
        first.binding.initial_capabilities.version,
        CAPABILITY_GENERATION
    );
    assert_eq!(first.process_generation, 12);
    assert!(
        first
            .binding
            .initial_configuration
            .snapshot_id
            .starts_with("configuration:")
    );
    assert!(
        first
            .binding
            .initial_resources
            .snapshot_id
            .starts_with("resources:")
    );
    assert!(
        first
            .binding
            .initial_capabilities
            .snapshot_id
            .starts_with("capabilities:")
    );
    assert_eq!(
        first.binding.initial_configuration.sha256,
        expected_configuration_sha256
    );
}

#[test]
fn narrow_intent_cannot_substitute_snapshot_material_or_stale_generations() {
    let registry = DispatchAuthorityRegistry::new();
    let installed = authority();
    registry.install_from_worker(0, installed.clone()).unwrap();

    assert_eq!(
        registry.mint_first_binding(
            &intent(2),
            &effective(installed.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::StaleAuthorityGeneration {
            expected: 2,
            current: 1,
        })
    );
    assert_eq!(
        registry.mint_first_binding(
            &intent(1),
            &effective(installed.route, false),
            CAPABILITY_GENERATION + 1,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::StaleCapabilityGeneration {
            expected: CAPABILITY_GENERATION,
            current: CAPABILITY_GENERATION + 1,
        })
    );
}

#[test]
fn route_configuration_workspace_and_process_drift_fail_closed() {
    let registry = DispatchAuthorityRegistry::new();
    let installed = authority();
    registry.install_from_worker(0, installed.clone()).unwrap();

    let mut route_drift = installed.route.clone();
    route_drift.provider_model_id = "anthropic/claude-opus".into();
    assert_eq!(
        registry.mint_first_binding(
            &intent(1),
            &effective(route_drift, false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::RouteDrift)
    );

    let mut configuration_drift = installed.route.clone();
    configuration_drift.session_configuration_sha256 = digest('d');
    assert_eq!(
        registry.mint_first_binding(
            &intent(1),
            &effective(configuration_drift, false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::ConfigurationDrift)
    );

    let mut wrong_workspace = intent(1);
    wrong_workspace.workspace_id = "workspace-2".into();
    assert_eq!(
        registry.mint_first_binding(
            &wrong_workspace,
            &effective(installed.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::WorkspaceDrift)
    );

    let mut stale_process = process();
    stale_process.process_generation += 1;
    assert_eq!(
        registry.mint_first_binding(
            &intent(1),
            &effective(installed.route, false),
            CAPABILITY_GENERATION,
            &stale_process,
            500,
        ),
        Err(DispatchAuthorityError::ProcessDrift)
    );
}

#[test]
fn cas_rejects_same_version_configuration_and_resource_substitution() {
    let registry = DispatchAuthorityRegistry::new();
    let installed = authority();
    registry.install_from_worker(0, installed.clone()).unwrap();

    let mut changed_configuration = installed.clone();
    changed_configuration.configuration.fields.insert(
        "reasoning-level".into(),
        ConfigurationFieldValue::Text("high".into()),
    );
    changed_configuration.route.session_configuration_sha256 =
        authoritative_configuration_sha256(&changed_configuration.configuration).unwrap();
    assert_eq!(
        registry.install_from_worker(1, changed_configuration),
        Err(DispatchAuthorityError::ConfigurationDrift)
    );

    let mut changed_resources = installed;
    changed_resources
        .resources
        .records
        .get_mut("skill:project-review")
        .unwrap()
        .sha256 = digest('f');
    assert_eq!(
        registry.install_from_worker(1, changed_resources),
        Err(DispatchAuthorityError::ResourceDrift)
    );
}

#[test]
fn batch_authority_routes_zero_one_and_many_without_partial_substitution() {
    let registry = DispatchAuthorityRegistry::new();
    let first = authority();
    let second = authority_for_model("provider-openai", "endpoint-openai", "openai/gpt-4.1");

    assert_eq!(
        registry
            .replace_runtime_from_worker(0, "workspace-1", "opencode-primary", 12, Vec::new(),)
            .unwrap(),
        1
    );
    assert_eq!(
        registry.mint_first_binding(
            &intent(1),
            &effective(first.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::NotFound)
    );

    assert_eq!(
        registry
            .replace_runtime_from_worker(
                1,
                "workspace-1",
                "opencode-primary",
                12,
                vec![first.clone()],
            )
            .unwrap(),
        2
    );
    registry
        .mint_first_binding(
            &intent(2),
            &effective(first.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        )
        .unwrap();
    assert_eq!(
        registry.mint_first_binding(
            &intent(2),
            &effective(second.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        ),
        Err(DispatchAuthorityError::RouteDrift)
    );

    assert_eq!(
        registry
            .replace_runtime_from_worker(
                2,
                "workspace-1",
                "opencode-primary",
                12,
                vec![first.clone(), second.clone()],
            )
            .unwrap(),
        3
    );
    for route in [first.route.clone(), second.route.clone()] {
        registry
            .mint_first_binding(
                &intent(3),
                &effective(route, false),
                CAPABILITY_GENERATION,
                &process(),
                500,
            )
            .unwrap();
    }

    let mut changed_configuration = first.clone();
    changed_configuration.configuration.fields.insert(
        "reasoning-level".into(),
        ConfigurationFieldValue::Text("high".into()),
    );
    changed_configuration.route.session_configuration_sha256 =
        authoritative_configuration_sha256(&changed_configuration.configuration).unwrap();
    assert_eq!(
        registry.replace_runtime_from_worker(
            3,
            "workspace-1",
            "opencode-primary",
            12,
            vec![changed_configuration, second.clone()],
        ),
        Err(DispatchAuthorityError::ConfigurationDrift)
    );
    assert_eq!(registry.generation().unwrap(), 3);

    let mut changed_resources = first.clone();
    changed_resources
        .resources
        .records
        .get_mut("skill:project-review")
        .unwrap()
        .sha256 = digest('f');
    assert_eq!(
        registry.replace_runtime_from_worker(
            3,
            "workspace-1",
            "opencode-primary",
            12,
            vec![changed_resources, second],
        ),
        Err(DispatchAuthorityError::ResourceDrift)
    );
    assert_eq!(registry.generation().unwrap(), 3);
    registry
        .mint_first_binding(
            &intent(3),
            &effective(first.route, false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        )
        .unwrap();
}

#[test]
fn retry_preserves_binding_invariants_while_minting_current_resources() {
    let registry = DispatchAuthorityRegistry::new();
    let installed = authority();
    registry.install_from_worker(0, installed.clone()).unwrap();
    let first = registry
        .mint_first_binding(
            &intent(1),
            &effective(installed.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
            500,
        )
        .unwrap();

    let mut refreshed = installed.clone();
    refreshed.resources.version = 10;
    refreshed
        .resources
        .records
        .get_mut("skill:project-review")
        .unwrap()
        .version = 6;
    assert_eq!(
        registry.install_from_worker(1, refreshed.clone()).unwrap(),
        2
    );
    assert_eq!(
        registry.mint_retry_context(
            &intent(1),
            &first.binding,
            &effective(installed.route.clone(), false),
            CAPABILITY_GENERATION,
            &process(),
        ),
        Err(DispatchAuthorityError::StaleAuthorityGeneration {
            expected: 1,
            current: 2,
        })
    );
    let retry = registry
        .mint_retry_context(
            &intent(2),
            &first.binding,
            &effective(installed.route, false),
            CAPABILITY_GENERATION,
            &process(),
        )
        .unwrap();
    assert_eq!(retry.context.workspace_id, first.binding.workspace_id);
    assert_eq!(retry.context.adapter, first.binding.adapter);
    assert_eq!(retry.context.environment, first.binding.environment);
    assert_eq!(retry.context.resources.version, 10);

    refreshed.adapter.adapter_version = "1.1.0".into();
    refreshed.process_generation = 13;
    refreshed.route.adapter_version = "1.1.0".into();
    refreshed.configuration.version = 5;
    refreshed.configuration.fields.insert(
        "adapter-version".into(),
        ConfigurationFieldValue::Text("1.1.0".into()),
    );
    refreshed.route.session_configuration_sha256 =
        authoritative_configuration_sha256(&refreshed.configuration).unwrap();
    assert_eq!(
        registry.install_from_worker(2, refreshed.clone()).unwrap(),
        3
    );
    let mut upgraded_process = process();
    upgraded_process.adapter_version = "1.1.0".into();
    upgraded_process.process_generation = 13;
    assert_eq!(
        registry.mint_retry_context(
            &intent(3),
            &first.binding,
            &effective(refreshed.route, false),
            CAPABILITY_GENERATION,
            &upgraded_process,
        ),
        Err(DispatchAuthorityError::RetryInvariantDrift)
    );
}
