use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io::{self, Write},
    net::{IpAddr, Ipv4Addr},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use c4os_lib::{
    runtime::{
        capability::{CapabilityKey, CapabilityState, ModelLifecycle},
        opencode::{
            CommandDriver, CommandFailureCode, LaunchCommand, LoopbackEndpoint,
            NativeAuthorityPolicy, OpenCodeAdapter, OpenCodeCompatibilityManifest,
            OpenCodeLaunchPlan, OpenCodeTransport, ProcessHandle, RandomSecretReference,
            StateNamespace, TransportFailureCode, TransportRequest, TransportResponse,
        },
        provider::{
            CurlProviderConnectivity, DirectProviderProbe, PROVIDER_SCHEMA_VERSION,
            ProviderAuthentication, ProviderCatalog, ProviderCatalogModel,
            ProviderConnectionObservation, ProviderConnectionRequest, ProviderConnectivity,
            ProviderEndpoint, ProviderKind, ProviderProbeFailure, ProviderProfile, ProviderService,
            ProviderTestStatus, RouteAvailability, VerifiedOpenCodeProviderProbe,
        },
    },
    security::credentials::{CredentialVault, OperationCredentialLease},
};
use serde_json::{Value, json};

const NOW: u64 = 1_721_300_000_000;
const DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RESPONSE_DIGEST: &str =
    "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[derive(Default)]
struct FakeTransport {
    responses: VecDeque<Result<TransportResponse, TransportFailureCode>>,
}

impl FakeTransport {
    fn respond_json(&mut self, status: u16, value: Value) {
        self.responses.push_back(Ok(TransportResponse {
            status,
            body: serde_json::to_vec(&value).unwrap(),
        }));
    }
}

impl OpenCodeTransport for FakeTransport {
    fn execute(
        &mut self,
        _request: TransportRequest,
    ) -> Result<TransportResponse, TransportFailureCode> {
        self.responses
            .pop_front()
            .unwrap_or(Err(TransportFailureCode::Protocol))
    }
}

#[derive(Default)]
struct FakeCommandDriver;

impl CommandDriver for FakeCommandDriver {
    fn spawn(&mut self, _command: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode> {
        Ok(ProcessHandle {
            process_id: 4_001,
            process_generation: 7,
        })
    }

    fn terminate_process_group(
        &mut self,
        _process: &ProcessHandle,
    ) -> Result<(), CommandFailureCode> {
        Ok(())
    }
}

fn adapter() -> OpenCodeAdapter<FakeTransport, FakeCommandDriver> {
    let namespace = StateNamespace::new(
        PathBuf::from("/private/tmp/c4os-provider-test").as_path(),
        "workspace-provider",
        7,
        "launch-provider",
    )
    .unwrap();
    let plan = OpenCodeLaunchPlan {
        manifest: OpenCodeCompatibilityManifest::pinned(DIGEST).unwrap(),
        endpoint: LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 49_176).unwrap(),
        namespace,
        executable: PathBuf::from("/Applications/C4OS.app/Contents/MacOS/opencode"),
        workspace_root: PathBuf::from("/private/tmp/c4os-provider-workspace"),
        basic_auth_username: "opencode".into(),
        password_reference: RandomSecretReference::new("provider-launch-password", 256).unwrap(),
        secret_channel_fd: 7,
        authority_policy: NativeAuthorityPolicy::new(
            DIGEST,
            ["c4os_read_resource", "c4os_propose_action"],
        )
        .unwrap(),
    };
    let mut transport = FakeTransport::default();
    transport.respond_json(200, json!({"healthy": true, "version": "1.18.3"}));
    let mut adapter = OpenCodeAdapter::new(plan, transport, FakeCommandDriver).unwrap();
    adapter.start(NOW - 1).unwrap();
    adapter
}

fn profile(vault: &CredentialVault) -> ProviderProfile {
    ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: "provider-openai".into(),
        kind: ProviderKind::OpenAi,
        display_name: "OpenAI".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "openai-api".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_kind: "openai".into(),
        },
        authentication: ProviderAuthentication::Bearer,
        credential_reference: Some(vault.store("openai", b"test-provider-secret").unwrap()),
        headers: BTreeMap::new(),
        enabled: true,
    }
}

fn model(id: &str, status: &str) -> Value {
    json!({
        "id": id,
        "providerID": "openai",
        "api": {"id": id, "url": "", "npm": ""},
        "name": id,
        "capabilities": {
            "reasoning": false,
            "toolcall": true,
            "input": {"text": true},
            "output": {"text": true}
        },
        "limit": {"context": 128000, "output": 16384},
        "status": status,
        "cost": {"input": 0, "output": 0, "cache": {"read": 0, "write": 0}},
        "options": {}, "headers": {}, "release_date": "2024-01-01"
    })
}

fn inventory(models: Vec<Value>) -> Value {
    let mapped = models
        .into_iter()
        .map(|model| (model["id"].as_str().unwrap().to_string(), model))
        .collect::<serde_json::Map<_, _>>();
    json!({
        "providers": [{
            "id": "openai", "name": "OpenAI", "source": "config", "env": [],
            "options": {}, "models": mapped
        }],
        "default": {}
    })
}

struct CountingCredentialChannel(usize);

impl Write for CountingCredentialChannel {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

enum ConnectivityOutcome {
    Success,
    EndpointSubstitution,
    Failure(ProviderProbeFailure),
}

struct RecordingConnectivity {
    outcome: ConnectivityOutcome,
    requests: Vec<ProviderConnectionRequest>,
}

impl RecordingConnectivity {
    fn success() -> Self {
        Self {
            outcome: ConnectivityOutcome::Success,
            requests: Vec::new(),
        }
    }
}

impl ProviderConnectivity for RecordingConnectivity {
    fn test_connection(
        &mut self,
        request: &ProviderConnectionRequest,
        credential_lease: Option<&OperationCredentialLease>,
    ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
        self.requests.push(request.clone());
        if let ConnectivityOutcome::Failure(failure) = self.outcome {
            return Err(failure);
        }
        let mut channel = CountingCredentialChannel(0);
        credential_lease
            .expect("credential lease")
            .deliver_to(&mut channel)
            .map_err(|_| ProviderProbeFailure::Authentication)?;
        assert!(channel.0 > 0);
        Ok(ProviderConnectionObservation {
            endpoint_sha256: if matches!(self.outcome, ConnectivityOutcome::EndpointSubstitution) {
                DIGEST.into()
            } else {
                request.endpoint_sha256.clone()
            },
            profile_binding_sha256: request.profile_binding_sha256.clone(),
            response_sha256: RESPONSE_DIGEST.into(),
            catalog: None,
        })
    }
}

fn run_verified(
    models: Vec<Value>,
    outcome: ConnectivityOutcome,
) -> (
    Result<
        c4os_lib::runtime::provider::ProviderTestReport,
        c4os_lib::runtime::provider::ProviderError,
    >,
    RecordingConnectivity,
) {
    let vault = CredentialVault::session_only().unwrap();
    let profile = profile(&vault);
    let mut adapter = adapter();
    adapter
        .test_transport_mut()
        .respond_json(200, inventory(models));
    let mut connectivity = RecordingConnectivity {
        outcome,
        requests: Vec::new(),
    };
    let mut service = ProviderService::new();
    service.save_profile(profile, 0).unwrap();
    let mut probe =
        VerifiedOpenCodeProviderProbe::new(&mut adapter, &vault, &mut connectivity, NOW, DIGEST)
            .unwrap();
    let result = service.test_provider("provider-openai", 1, NOW, &mut probe);
    drop(probe);
    (result, connectivity)
}

#[test]
fn verified_connection_covers_zero_one_and_many_catalog_results() {
    let (zero, zero_connectivity) = run_verified(vec![], ConnectivityOutcome::Success);
    let zero = zero.unwrap();
    assert!(
        matches!(
            zero.status,
            ProviderTestStatus::SucceededNoUsableModels { .. }
        ),
        "unexpected zero-model report: {zero:?}"
    );
    assert_eq!(
        zero_connectivity.requests[0].base_url,
        "https://api.openai.com/v1"
    );

    let (one, _) = run_verified(
        vec![model("gpt-4o-mini", "active")],
        ConnectivityOutcome::Success,
    );
    let one = one.unwrap();
    assert_eq!(one.discovered_models, 1);
    assert_eq!(one.selected_model_id.as_deref(), Some("gpt-4o-mini"));

    let (many, _) = run_verified(
        vec![
            model("model-a", "active"),
            model("model-b", "beta"),
            model("model-c", "deprecated"),
        ],
        ConnectivityOutcome::Success,
    );
    assert_eq!(many.unwrap().discovered_models, 3);
}

#[test]
fn authentication_and_network_failures_never_fall_back_to_cached_catalog() {
    for failure in [
        ProviderProbeFailure::Authentication,
        ProviderProbeFailure::Network,
    ] {
        let (result, connectivity) = run_verified(
            vec![model("cached-model", "active")],
            ConnectivityOutcome::Failure(failure),
        );
        let report = result.unwrap();
        assert!(matches!(report.status, ProviderTestStatus::Failed { .. }));
        assert_eq!(report.discovered_models, 0);
        assert_eq!(connectivity.requests.len(), 1);
    }
}

#[test]
fn substituted_endpoint_observation_is_rejected_after_lease_consumption() {
    let (result, connectivity) = run_verified(
        vec![model("gpt-4o-mini", "active")],
        ConnectivityOutcome::EndpointSubstitution,
    );
    let report = result.unwrap();
    assert!(matches!(
        report.status,
        ProviderTestStatus::Failed {
            code: c4os_lib::runtime::provider::ProviderFailureCode::Incompatible,
            ..
        }
    ));
    assert_eq!(connectivity.requests[0].endpoint_id, "openai-api");
}

#[test]
fn connectivity_success_without_consuming_the_credential_lease_fails_closed() {
    struct NonConsumingConnectivity;
    impl ProviderConnectivity for NonConsumingConnectivity {
        fn test_connection(
            &mut self,
            request: &ProviderConnectionRequest,
            _credential_lease: Option<&OperationCredentialLease>,
        ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
            Ok(ProviderConnectionObservation {
                endpoint_sha256: request.endpoint_sha256.clone(),
                profile_binding_sha256: request.profile_binding_sha256.clone(),
                response_sha256: RESPONSE_DIGEST.into(),
                catalog: None,
            })
        }
    }

    let vault = CredentialVault::session_only().unwrap();
    let profile = profile(&vault);
    let mut adapter = adapter();
    adapter
        .test_transport_mut()
        .respond_json(200, inventory(vec![model("cached", "active")]));
    let mut connectivity = NonConsumingConnectivity;
    let mut probe =
        VerifiedOpenCodeProviderProbe::new(&mut adapter, &vault, &mut connectivity, NOW, DIGEST)
            .unwrap();
    assert_eq!(
        c4os_lib::runtime::provider::ProviderProbe::test_and_discover(&mut probe, &profile),
        Err(ProviderProbeFailure::Incompatible)
    );
}

#[test]
fn concrete_transport_sends_endpoint_and_credential_only_through_anonymous_stdin() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("credential-check-curl");
    fs::write(
        &executable,
        r#"#!/bin/sh
payload="$(/bin/cat)"
case "$payload" in
  *'url = "https://api.openai.com/v1/models"'*'header = "Authorization: Bearer test-provider-secret"'*)
    /usr/bin/printf '{"authenticated":true}\n200'
    ;;
  *)
    /usr/bin/printf '\n401'
    ;;
esac
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&executable, permissions).unwrap();

    let vault = CredentialVault::session_only().unwrap();
    let profile = profile(&vault);
    let mut adapter = adapter();
    adapter
        .test_transport_mut()
        .respond_json(200, inventory(vec![model("gpt-4o-mini", "active")]));
    let mut connectivity = CurlProviderConnectivity::new(&executable).unwrap();
    let debug = format!("{connectivity:?}");
    assert!(!debug.contains("test-provider-secret"));

    let mut service = ProviderService::new();
    service.save_profile(profile, 0).unwrap();
    let mut probe =
        VerifiedOpenCodeProviderProbe::new(&mut adapter, &vault, &mut connectivity, NOW, DIGEST)
            .unwrap();
    let report = service
        .test_provider("provider-openai", 1, NOW, &mut probe)
        .unwrap();
    assert!(matches!(
        report.status,
        ProviderTestStatus::Succeeded { .. }
    ));
    let state = serde_json::to_string(&service.snapshot()).unwrap();
    assert!(!state.contains("test-provider-secret"));
    assert!(state.contains("connectionEvidence"));
}

#[test]
fn helper_constructor_is_used_by_successful_connectivity_fixture() {
    let connectivity = RecordingConnectivity::success();
    assert!(connectivity.requests.is_empty());
}

#[test]
fn direct_first_launch_probe_uses_the_exact_catalog_without_a_workspace_runtime() {
    struct CatalogConnectivity;
    impl ProviderConnectivity for CatalogConnectivity {
        fn test_connection(
            &mut self,
            request: &ProviderConnectionRequest,
            credential_lease: Option<&OperationCredentialLease>,
        ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
            let mut channel = CountingCredentialChannel(0);
            credential_lease
                .expect("credential lease")
                .deliver_to(&mut channel)
                .map_err(|_| ProviderProbeFailure::Authentication)?;
            Ok(ProviderConnectionObservation {
                endpoint_sha256: request.endpoint_sha256.clone(),
                profile_binding_sha256: request.profile_binding_sha256.clone(),
                response_sha256: RESPONSE_DIGEST.into(),
                catalog: Some(ProviderCatalog {
                    models: vec![ProviderCatalogModel {
                        model_id: "gpt-4o-mini".into(),
                        display_name: "GPT-4o mini".into(),
                        input_modalities: BTreeSet::from(["text".into()]),
                        output_modalities: BTreeSet::from(["text".into()]),
                        supported_parameters: BTreeSet::from([
                            "tools".into(),
                            "response_format".into(),
                        ]),
                        context_tokens: Some(128_000),
                        output_tokens: Some(16_384),
                    }],
                }),
            })
        }
    }

    let vault = CredentialVault::session_only().unwrap();
    let profile = profile(&vault);
    let mut connectivity = CatalogConnectivity;
    let mut probe = DirectProviderProbe::new(&vault, &mut connectivity, NOW).unwrap();
    let mut service = ProviderService::new();
    service.save_profile(profile, 0).unwrap();
    let report = service
        .test_provider("provider-openai", 1, NOW, &mut probe)
        .unwrap();
    assert_eq!(report.discovered_models, 1);
    assert_eq!(report.usable_models, 1);
    assert_eq!(report.selected_model_id.as_deref(), Some("gpt-4o-mini"));
    {
        let snapshot = service.snapshot();
        let route = &snapshot.providers[0].models["gpt-4o-mini"];
        assert!(route.is_production_ready());
        assert_eq!(route.availability, RouteAvailability::Available);
        assert_eq!(route.capabilities.lifecycle, ModelLifecycle::Active);
        assert_eq!(
            route.capabilities.feature_state(CapabilityKey::InputText),
            CapabilityState::Supported
        );
        assert_eq!(
            route.capabilities.feature_state(CapabilityKey::OutputText),
            CapabilityState::Supported
        );
        assert_eq!(
            route.capabilities.feature_state(CapabilityKey::Streaming),
            CapabilityState::Unknown
        );
    }
    assert!(!service.snapshot().launch_ready());
    service
        .complete_onboarding(report.generation, NOW + 1)
        .unwrap();
    assert!(service.snapshot().launch_ready());
    assert!(service.snapshot().onboarding_ready_at(NOW + 1));
}

#[test]
fn direct_first_launch_probe_accepts_sparse_official_openai_chat_models_only() {
    struct SparseCatalogConnectivity;
    impl ProviderConnectivity for SparseCatalogConnectivity {
        fn test_connection(
            &mut self,
            request: &ProviderConnectionRequest,
            credential_lease: Option<&OperationCredentialLease>,
        ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
            let mut channel = CountingCredentialChannel(0);
            credential_lease
                .expect("credential lease")
                .deliver_to(&mut channel)
                .map_err(|_| ProviderProbeFailure::Authentication)?;
            Ok(ProviderConnectionObservation {
                endpoint_sha256: request.endpoint_sha256.clone(),
                profile_binding_sha256: request.profile_binding_sha256.clone(),
                response_sha256: RESPONSE_DIGEST.into(),
                catalog: Some(ProviderCatalog {
                    models: vec![
                        ProviderCatalogModel {
                            model_id: "gpt-undisclosed".into(),
                            display_name: "Undisclosed model".into(),
                            input_modalities: BTreeSet::new(),
                            output_modalities: BTreeSet::new(),
                            supported_parameters: BTreeSet::new(),
                            context_tokens: None,
                            output_tokens: None,
                        },
                        ProviderCatalogModel {
                            model_id: "gpt-missing-output".into(),
                            display_name: "Missing output declaration".into(),
                            input_modalities: BTreeSet::from(["text".into()]),
                            output_modalities: BTreeSet::new(),
                            supported_parameters: BTreeSet::from(["tools".into()]),
                            context_tokens: Some(32_000),
                            output_tokens: None,
                        },
                    ],
                }),
            })
        }
    }

    let vault = CredentialVault::session_only().unwrap();
    let openai_profile = profile(&vault);
    let mut connectivity = SparseCatalogConnectivity;
    let mut probe = DirectProviderProbe::new(&vault, &mut connectivity, NOW).unwrap();
    let mut service = ProviderService::new();
    service.save_profile(openai_profile, 0).unwrap();

    let report = service
        .test_provider("provider-openai", 1, NOW, &mut probe)
        .unwrap();
    assert_eq!(report.discovered_models, 2);
    assert_eq!(report.usable_models, 1);
    assert_eq!(report.selected_model_id.as_deref(), Some("gpt-undisclosed"));
    assert!(matches!(
        report.status,
        ProviderTestStatus::Succeeded { .. }
    ));

    let snapshot = service.snapshot();
    let sparse_route = &snapshot.providers[0].models["gpt-undisclosed"];
    assert!(sparse_route.is_production_ready());
    assert_eq!(sparse_route.availability, RouteAvailability::Available);
    assert_eq!(sparse_route.capabilities.lifecycle, ModelLifecycle::Active);
    assert_eq!(
        sparse_route
            .capabilities
            .feature_state(CapabilityKey::InputText),
        CapabilityState::Supported
    );
    assert_eq!(
        sparse_route
            .capabilities
            .feature_state(CapabilityKey::OutputText),
        CapabilityState::Supported
    );
    assert_eq!(
        sparse_route
            .capabilities
            .feature_state(CapabilityKey::Streaming),
        CapabilityState::Unknown
    );

    let partial_route = &snapshot.providers[0].models["gpt-missing-output"];
    assert!(!partial_route.is_production_ready());
    assert_eq!(partial_route.availability, RouteAvailability::Unknown);
    assert_eq!(
        partial_route.capabilities.lifecycle,
        ModelLifecycle::Unavailable
    );
    assert_eq!(
        partial_route
            .capabilities
            .feature_state(CapabilityKey::InputText),
        CapabilityState::Supported
    );
    assert_eq!(
        partial_route
            .capabilities
            .feature_state(CapabilityKey::OutputText),
        CapabilityState::Unknown
    );
    for route in [sparse_route, partial_route] {
        assert_eq!(
            route.capabilities.feature_state(CapabilityKey::Streaming),
            CapabilityState::Unknown
        );
    }
    assert!(snapshot.onboarding_ready_at(NOW));

    let mut custom_profile = profile(&vault);
    custom_profile.provider_id = "provider-custom".into();
    custom_profile.kind = ProviderKind::Custom;
    custom_profile.display_name = "Custom OpenAI-compatible".into();
    custom_profile.endpoint = ProviderEndpoint {
        endpoint_id: "custom-api".into(),
        base_url: "https://proxy.example/v1".into(),
        api_kind: "openai-compatible".into(),
    };
    let mut custom_connectivity = SparseCatalogConnectivity;
    let mut custom_probe = DirectProviderProbe::new(&vault, &mut custom_connectivity, NOW).unwrap();
    let mut custom_service = ProviderService::new();
    custom_service.save_profile(custom_profile, 0).unwrap();

    let custom_report = custom_service
        .test_provider("provider-custom", 1, NOW, &mut custom_probe)
        .unwrap();
    assert_eq!(custom_report.discovered_models, 2);
    assert_eq!(custom_report.usable_models, 0);
    assert!(custom_report.selected_model_id.is_none());
    assert!(matches!(
        custom_report.status,
        ProviderTestStatus::SucceededNoUsableModels { .. }
    ));
    for route in custom_service.snapshot().providers[0].models.values() {
        assert!(!route.is_production_ready());
        assert_eq!(route.availability, RouteAvailability::Unknown);
        assert_eq!(route.capabilities.lifecycle, ModelLifecycle::Unavailable);
    }
}
