use std::{
    collections::VecDeque,
    fs,
    io::{self, Write},
    net::{IpAddr, Ipv4Addr},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use c4os_lib::{
    runtime::{
        opencode::{
            CommandDriver, CommandFailureCode, LaunchCommand, LoopbackEndpoint,
            NativeAuthorityPolicy, OpenCodeAdapter, OpenCodeCompatibilityManifest,
            OpenCodeLaunchPlan, OpenCodeTransport, ProcessHandle, RandomSecretReference,
            StateNamespace, TransportFailureCode, TransportRequest, TransportResponse,
        },
        provider::{
            CurlProviderConnectivity, PROVIDER_SCHEMA_VERSION, ProviderConnectionObservation,
            ProviderConnectionRequest, ProviderConnectivity, ProviderEndpoint, ProviderKind,
            ProviderProbeFailure, ProviderProfile, ProviderService, ProviderTestStatus,
            VerifiedOpenCodeProviderProbe,
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
            api_kind: "openai-compatible".into(),
        },
        credential_reference: vault.store("openai", b"test-provider-secret").unwrap(),
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
        credential_lease: &OperationCredentialLease,
    ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
        self.requests.push(request.clone());
        if let ConnectivityOutcome::Failure(failure) = self.outcome {
            return Err(failure);
        }
        let mut channel = CountingCredentialChannel(0);
        credential_lease
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
            _credential_lease: &OperationCredentialLease,
        ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
            Ok(ProviderConnectionObservation {
                endpoint_sha256: request.endpoint_sha256.clone(),
                profile_binding_sha256: request.profile_binding_sha256.clone(),
                response_sha256: RESPONSE_DIGEST.into(),
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
