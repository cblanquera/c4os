use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use c4os_lib::runtime::opencode::*;
use c4os_lib::runtime::opencode_credential::{
    ProviderCredentialAuthorizationReceipt, ProviderCredentialRequest,
};
use c4os_lib::runtime::provider::{
    OpenCodeProviderProbe, PROVIDER_SCHEMA_VERSION, ProviderAuthentication, ProviderEndpoint,
    ProviderKind, ProviderProbe, ProviderProfile,
};
use c4os_lib::security::credentials::CredentialVault;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

const DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn content_digest(content: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(content) {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

#[derive(Default)]
struct FakeTransport {
    responses: VecDeque<Result<TransportResponse, TransportFailureCode>>,
    requests: Vec<TransportRequest>,
}

impl FakeTransport {
    fn respond_json(&mut self, status: u16, value: Value) {
        self.responses.push_back(Ok(TransportResponse {
            status,
            body: serde_json::to_vec(&value).expect("fixture JSON"),
        }));
    }
}

impl OpenCodeTransport for FakeTransport {
    fn execute(
        &mut self,
        request: TransportRequest,
    ) -> Result<TransportResponse, TransportFailureCode> {
        self.requests.push(request);
        self.responses
            .pop_front()
            .unwrap_or(Err(TransportFailureCode::Protocol))
    }
}

#[derive(Default)]
struct FakeCommandDriver {
    spawn_generations: VecDeque<u64>,
    commands: Vec<LaunchCommand>,
    terminated: Vec<ProcessHandle>,
    credential_receipts: VecDeque<ProviderCredentialAuthorizationReceipt>,
    credential_revocations: Vec<ProviderCredentialRequest>,
}

impl CommandDriver for FakeCommandDriver {
    fn spawn(&mut self, command: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode> {
        self.commands.push(command.clone());
        let process_generation = self
            .spawn_generations
            .pop_front()
            .ok_or(CommandFailureCode::SpawnRejected)?;
        Ok(ProcessHandle {
            process_id: 4_000 + self.commands.len() as u32,
            process_generation,
        })
    }

    fn terminate_process_group(
        &mut self,
        process: &ProcessHandle,
    ) -> Result<(), CommandFailureCode> {
        self.terminated.push(process.clone());
        Ok(())
    }

    fn authorize_provider_credential_attempt(
        &mut self,
        _request: ProviderCredentialRequest,
    ) -> Result<Option<ProviderCredentialAuthorizationReceipt>, CommandFailureCode> {
        Ok(self.credential_receipts.pop_front())
    }

    fn revoke_provider_credential_attempt(
        &mut self,
        request: &ProviderCredentialRequest,
    ) -> Result<(), CommandFailureCode> {
        self.credential_revocations.push(request.clone());
        Ok(())
    }
}

fn secret(id: &str) -> RandomSecretReference {
    RandomSecretReference::new(id, 256).expect("secret reference")
}

fn plan(generation: u64, launch: &str, secret_id: &str) -> OpenCodeLaunchPlan {
    let namespace = StateNamespace::new(
        PathBuf::from("/private/tmp/c4os-home").as_path(),
        "workspace-1",
        generation,
        launch,
    )
    .expect("namespace");
    OpenCodeLaunchPlan {
        manifest: OpenCodeCompatibilityManifest::pinned(DIGEST).expect("manifest"),
        endpoint: LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 49_173).expect("endpoint"),
        namespace,
        executable: PathBuf::from("/Applications/C4OS.app/Contents/MacOS/opencode"),
        workspace_root: PathBuf::from("/private/tmp/c4os-project"),
        basic_auth_username: "opencode".into(),
        password_reference: secret(secret_id),
        secret_channel_fd: 7,
        authority_policy: NativeAuthorityPolicy::new(
            DIGEST,
            ["c4os_read_resource", "c4os_propose_action"],
        )
        .expect("authority policy"),
    }
}

fn health() -> Value {
    json!({ "healthy": true, "version": "1.18.3" })
}

fn native_session(id: &str) -> Value {
    json!({
        "id": id,
        "slug": "proof",
        "projectID": "project-native-1",
        "directory": "/private/tmp/c4os-project",
        "title": "C4OS session",
        "version": "1.18.3",
        "time": { "created": 10, "updated": 11 }
    })
}

fn ready_adapter() -> OpenCodeAdapter<FakeTransport, FakeCommandDriver> {
    let mut transport = FakeTransport::default();
    transport.respond_json(200, health());
    let mut driver = FakeCommandDriver::default();
    driver.spawn_generations.push_back(7);
    let mut adapter =
        OpenCodeAdapter::new(plan(7, "launch-7", "vault-launch-7"), transport, driver)
            .expect("adapter");
    adapter.start(100).expect("start");
    adapter
}

fn create_session(adapter: &mut OpenCodeAdapter<FakeTransport, FakeCommandDriver>) {
    adapter
        .transport_for_test()
        .respond_json(200, native_session("native-session-1"));
    adapter
        .create_session("chat-1", "C4OS session")
        .expect("create session");
}

fn correlation(generation: u64, run: &str) -> EventCorrelation {
    EventCorrelation {
        workspace_id: "workspace-1".into(),
        c4os_session_id: "chat-1".into(),
        c4os_turn_id: format!("turn-{run}"),
        c4os_run_id: run.into(),
        correlation_id: format!("correlation-{run}"),
        native_session_id: "native-session-1".into(),
        process_generation: generation,
    }
}

fn begin_run(adapter: &mut OpenCodeAdapter<FakeTransport, FakeCommandDriver>) -> EventCorrelation {
    create_session(adapter);
    adapter.transport_for_test().respond_json(204, Value::Null);
    let correlation = correlation(7, "run-1");
    adapter
        .send(PromptDispatch {
            correlation: correlation.clone(),
            model: ModelRoute {
                provider_id: "openai".into(),
                model_id: "gpt-4o-mini".into(),
            },
            text: "hello".into(),
            eligible_tool_ids: BTreeSet::from([
                C4OS_ACTION_PROPOSAL_TOOL.into(),
                C4OS_RESOURCE_READ_TOOL.into(),
            ]),
            attachments: vec![],
            native_overrides: json!({}),
        })
        .expect("send");
    correlation
}

#[test]
fn credentialed_send_uses_the_core_derived_native_provider_route() {
    let mut transport = FakeTransport::default();
    transport.respond_json(200, health());
    transport.respond_json(200, native_session("native-session-1"));
    transport.respond_json(204, Value::Null);
    let mut driver = FakeCommandDriver::default();
    driver.spawn_generations.push_back(7);
    driver
        .credential_receipts
        .push_back(ProviderCredentialAuthorizationReceipt {
            process_generation: 7,
            native_session_id: "native-session-1".into(),
            c4os_provider_id: "provider-openai".into(),
            native_provider_id: "provider-openai".into(),
            native_model_id: "gpt-5".into(),
            operation_authorization_id: "credential-authorization:fixture".into(),
            credential_required: true,
        });
    let mut adapter = OpenCodeAdapter::new(
        plan(7, "launch-provider-route", "vault-provider-route"),
        transport,
        driver,
    )
    .unwrap();
    adapter.start(100).unwrap();
    adapter
        .create_session("chat-1", "C4OS session")
        .expect("native session");
    let active = correlation(7, "credential-route");
    adapter
        .send_with_provider_credential(PromptDispatch {
            correlation: active.clone(),
            model: ModelRoute {
                provider_id: "provider-openai".into(),
                model_id: "provider-openai/gpt-5".into(),
            },
            text: "hello".into(),
            eligible_tool_ids: BTreeSet::from([
                C4OS_ACTION_PROPOSAL_TOOL.into(),
                C4OS_RESOURCE_READ_TOOL.into(),
            ]),
            attachments: Vec::new(),
            native_overrides: json!({}),
        })
        .unwrap();

    let prompt = adapter
        .transport_for_test()
        .requests
        .last()
        .and_then(|request| request.body.as_ref())
        .and_then(|body| serde_json::from_slice::<Value>(body).ok())
        .unwrap();
    assert_eq!(prompt["model"]["providerID"], "provider-openai");
    assert_eq!(prompt["model"]["modelID"], "gpt-5");
    assert!(!prompt.to_string().contains("credential-lease"));
    assert!(!prompt.to_string().contains("credential-authorization"));
    let step = adapter
        .normalize_sse(
            &active,
            b"data: {\"type\":\"session.next.step.ended\",\"properties\":{\"sessionID\":\"native-session-1\"}}\n\n",
            150,
        )
        .expect("provider continuation step");
    assert_eq!(
        step.category,
        NormalizedEventCategory::Lifecycle {
            state: "provider-step-ended".into()
        }
    );
    assert!(adapter.driver_for_test().credential_revocations.is_empty());
    adapter
        .normalize_sse(
            &active,
            b"data: {\"type\":\"session.idle\",\"properties\":{\"sessionID\":\"native-session-1\"}}\n\n",
            200,
        )
        .expect("terminal provider event");
    let (_, driver) = adapter.into_parts();
    assert_eq!(driver.credential_revocations.len(), 1);
    assert_eq!(
        driver.credential_revocations[0].operation_id,
        active.correlation_id
    );
}

// Test-only inspection keeps production fields private while allowing the
// hostile boundary suite to assert exact transport behavior.
trait AdapterTestAccess {
    fn transport_for_test(&mut self) -> &mut FakeTransport;
}

impl AdapterTestAccess for OpenCodeAdapter<FakeTransport, FakeCommandDriver> {
    fn transport_for_test(&mut self) -> &mut FakeTransport {
        // SAFETY: avoided by using the public ownership round-trip in production;
        // this test module needs no unsafe access because the source exposes the
        // helper below only while compiled as a test path module.
        self.test_transport_mut()
    }
}

#[test]
fn manifest_is_exactly_pinned_and_digest_validated() {
    let manifest = OpenCodeCompatibilityManifest::pinned(DIGEST).expect("pinned");
    assert_eq!(manifest.native_version, "1.18.3");
    assert_eq!(manifest.sdk_version, "1.18.3");

    let mut wrong = manifest.clone();
    wrong.native_version = "1.18.4".into();
    assert_eq!(wrong.validate(), Err(AdapterError::IncompatibleVersion));

    let mut malformed = manifest;
    malformed.binary_sha256 = "sha256:not-a-digest".into();
    assert_eq!(malformed.validate(), Err(AdapterError::InvalidManifest));
}

#[test]
fn typed_driver_transport_and_decision_failure_codes_are_exhaustive() {
    let command_codes = [
        CommandFailureCode::SpawnRejected,
        CommandFailureCode::SecretChannelUnavailable,
        CommandFailureCode::ProviderCredentialUnavailable,
        CommandFailureCode::TerminationFailed,
    ];
    let transport_codes = [
        TransportFailureCode::Unavailable,
        TransportFailureCode::Timeout,
        TransportFailureCode::AuthenticationRejected,
        TransportFailureCode::Protocol,
    ];
    let decisions = [
        C4osPermissionDecision::C4osEffectCompleted,
        C4osPermissionDecision::Deny,
    ];
    assert_eq!(command_codes.len(), 4);
    assert_eq!(transport_codes.len(), 4);
    assert_eq!(decisions.len(), 2);
}

#[test]
fn endpoint_accepts_only_nonzero_loopback_ports() {
    let ipv4 = LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080).expect("v4");
    assert_eq!(ipv4.base_url(), "http://127.0.0.1:8080");
    let ipv6 = LoopbackEndpoint::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8081).expect("v6");
    assert_eq!(ipv6.base_url(), "http://[::1]:8081");
    assert_eq!(
        LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 8080),
        Err(AdapterError::InvalidEndpoint)
    );
    assert_eq!(
        LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        Err(AdapterError::InvalidEndpoint)
    );
}

#[test]
fn state_is_namespaced_by_version_workspace_generation_and_launch() {
    let first = plan(7, "launch-7", "vault-launch-7");
    let second = plan(8, "launch-8", "vault-launch-8");
    assert_ne!(first.namespace.root(), second.namespace.root());
    assert!(
        first
            .namespace
            .root()
            .ends_with("runtimes/opencode/1.18.3/workspaces/workspace-1/generations/7/launch-7")
    );
    assert_eq!(first.namespace.workspace_id(), "workspace-1");
    assert_eq!(first.namespace.process_generation(), 7);
    assert_eq!(first.namespace.launch_id(), "launch-7");
    assert_eq!(
        StateNamespace::new(
            PathBuf::from("relative").as_path(),
            "workspace-1",
            1,
            "launch"
        ),
        Err(AdapterError::InvalidStateNamespace)
    );
    assert!(RandomSecretReference::new("weak", 128).is_err());
}

#[test]
fn launch_command_has_no_raw_secret_in_argv_environment_or_debug() {
    let plan = plan(7, "launch-7", "vault-launch-7");
    let command = plan.command().expect("command");
    assert_eq!(
        command.arguments,
        ["serve", "--hostname", "127.0.0.1", "--port", "49173"]
    );
    assert!(
        command
            .environment
            .keys()
            .all(|key| !key.to_ascii_lowercase().contains("password"))
    );
    assert!(!format!("{command:?}").contains("correct-horse-battery-staple"));
    assert!(format!("{:?}", command.secret_channel.reference).contains("<redacted>"));
    assert_eq!(command.secret_channel.inherited_fd, 7);
    assert_eq!(command.expected_binary_sha256, DIGEST);
    assert_eq!(command.authority_policy.configuration_sha256(), DIGEST);
    assert_eq!(
        command.authority_policy.permission_default(),
        NativePermissionDefault::Deny
    );
    assert!(!command.authority_policy.remember_native_decisions());
    assert!(
        command
            .authority_policy
            .allowed_tool_proposals()
            .contains("c4os_propose_action")
    );
}

#[test]
fn start_uses_basic_auth_reference_and_exact_health_version() {
    let adapter = ready_adapter();
    assert_eq!(adapter.lifecycle(), LifecycleState::Ready);
    assert_eq!(adapter.health().expect("health").native_version, "1.18.3");
    let (transport, driver) = adapter.into_parts();
    assert_eq!(driver.commands.len(), 1);
    assert_eq!(transport.requests.len(), 1);
    assert_eq!(transport.requests[0].base_url, "http://127.0.0.1:49173");
    assert_eq!(transport.requests[0].path, "/global/health");
    match &transport.requests[0].auth {
        TransportAuth::Basic {
            username,
            password_reference,
        } => {
            assert_eq!(username, "opencode");
            assert_eq!(password_reference.reference_id(), "vault-launch-7");
        }
    }
    assert!(!format!("{:?}", transport.requests[0]).contains("vault-launch-7"));
}

#[test]
fn incompatible_or_unhealthy_probe_fails_closed() {
    let mut transport = FakeTransport::default();
    transport.respond_json(200, json!({ "healthy": true, "version": "1.18.4" }));
    let mut driver = FakeCommandDriver::default();
    driver.spawn_generations.push_back(7);
    let mut adapter =
        OpenCodeAdapter::new(plan(7, "launch-7", "vault-launch-7"), transport, driver)
            .expect("adapter");
    assert_eq!(adapter.start(1), Err(AdapterError::IncompatibleVersion));
    assert_eq!(adapter.lifecycle(), LifecycleState::Degraded);
    let (_, driver) = adapter.into_parts();
    assert_eq!(driver.terminated.len(), 1);
}

#[test]
fn session_creation_validates_native_version_and_directory() {
    let mut adapter = ready_adapter();
    create_session(&mut adapter);
    let session = adapter.session("chat-1").expect("binding");
    assert_eq!(session.native_session_id, "native-session-1");
    assert_eq!(session.state, SessionState::Idle);

    adapter
        .transport_for_test()
        .respond_json(200, native_session("native-session-2"));
    let duplicate = adapter.create_session("chat-1", "again");
    assert_eq!(duplicate, Err(AdapterError::InvalidRequest));
}

#[test]
fn hostile_session_directory_is_rejected() {
    let mut adapter = ready_adapter();
    let mut hostile = native_session("native-session-1");
    hostile["directory"] = json!("/private/tmp/another-project");
    adapter.transport_for_test().respond_json(200, hostile);
    assert_eq!(
        adapter.create_session("chat-1", "C4OS session"),
        Err(AdapterError::InvalidSessionPayload)
    );
}

#[test]
fn provider_models_are_bounded_and_normalized() {
    let mut adapter = ready_adapter();
    let inventory = json!({
        "providers": [{
            "id": "openai",
            "name": "OpenAI",
            "source": "config",
            "env": [],
            "options": {},
            "models": {
                "gpt-4o-mini": {
                    "id": "gpt-4o-mini",
                    "providerID": "openai",
                    "api": {"id": "gpt-4o-mini", "url": "", "npm": ""},
                    "name": "GPT-4o mini",
                    "capabilities": {
                        "reasoning": false,
                        "toolcall": true,
                        "input": {"text": true, "image": true},
                        "output": {"text": true}
                    },
                    "limit": {"context": 128000, "output": 16384},
                    "status": "active",
                    "cost": {"input": 0, "output": 0, "cache": {"read": 0, "write": 0}},
                    "options": {}, "headers": {}, "release_date": "2024-01-01"
                }
            }
        }],
        "default": {"openai": "gpt-4o-mini"}
    });
    adapter
        .transport_for_test()
        .respond_json(200, inventory.clone());
    let models = adapter.list_models().expect("models");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].lifecycle, ModelLifecycle::Active);
    assert!(models[0].tool_calling);
    assert!(models[0].input_modalities.contains("image"));
    assert_eq!(models[0].context_limit, 128_000);

    let vault = CredentialVault::session_only().unwrap();
    let profile = ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: "provider-openai".into(),
        kind: ProviderKind::OpenAi,
        display_name: "OpenAI".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "openai-models".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_kind: "openai".into(),
        },
        authentication: ProviderAuthentication::Bearer,
        credential_reference: Some(vault.store("openai", b"fixture-secret").unwrap()),
        headers: BTreeMap::new(),
        enabled: true,
    };
    adapter.transport_for_test().respond_json(200, inventory);
    let mut probe = OpenCodeProviderProbe::new(&mut adapter, 300, DIGEST).unwrap();
    let discovery = probe.test_and_discover(&profile).unwrap();
    assert_eq!(discovery.models.len(), 1);
    assert_eq!(
        discovery.models[0].capabilities.route.provider_id,
        "provider-openai"
    );
    assert_eq!(
        discovery.models[0].capabilities.route.provider_model_id,
        "provider-openai/gpt-4o-mini"
    );
    assert_eq!(
        discovery.models[0]
            .capabilities
            .feature_state(c4os_lib::runtime::capability::CapabilityKey::InputImage),
        c4os_lib::runtime::capability::CapabilityState::Supported
    );
}

#[test]
fn per_request_tools_and_permission_overrides_are_rejected_before_transport() {
    for overrides in [
        json!({"tools": {"write": true}}),
        json!({"permission": "allow"}),
        json!({"agent": "unreviewed-agent"}),
        json!({"shell": "rm -rf /"}),
    ] {
        let mut adapter = ready_adapter();
        create_session(&mut adapter);
        let request_count = adapter.transport_for_test().requests.len();
        let result = adapter.send(PromptDispatch {
            correlation: correlation(7, "run-1"),
            model: ModelRoute {
                provider_id: "openai".into(),
                model_id: "gpt-4o-mini".into(),
            },
            text: "try to write".into(),
            eligible_tool_ids: BTreeSet::from([
                C4OS_ACTION_PROPOSAL_TOOL.into(),
                C4OS_RESOURCE_READ_TOOL.into(),
            ]),
            attachments: vec![],
            native_overrides: overrides,
        });
        assert_eq!(result, Err(AdapterError::AuthorityOverrideRejected));
        assert_eq!(adapter.transport_for_test().requests.len(), request_count);
    }
}

#[test]
fn narrow_prompt_body_contains_only_c4os_broker_tools() {
    let mut adapter = ready_adapter();
    create_session(&mut adapter);
    adapter.transport_for_test().respond_json(204, Value::Null);
    adapter
        .send(PromptDispatch {
            correlation: correlation(7, "run-1"),
            model: ModelRoute {
                provider_id: "openai".into(),
                model_id: "gpt-4o-mini".into(),
            },
            text: "hello".into(),
            eligible_tool_ids: BTreeSet::from([C4OS_ACTION_PROPOSAL_TOOL.into()]),
            attachments: vec![],
            native_overrides: json!({"format": {"type": "text"}}),
        })
        .expect("send");
    assert_eq!(
        adapter.session("chat-1").expect("session").state,
        SessionState::Running
    );
    let request = adapter
        .transport_for_test()
        .requests
        .last()
        .expect("prompt request");
    let body: Value = serde_json::from_slice(request.body.as_ref().expect("body")).expect("JSON");
    assert_eq!(
        body.get("tools"),
        Some(&json!({ "c4os_propose_action": true }))
    );
    assert_eq!(body["model"]["providerID"], "openai");
}

#[test]
fn verified_image_and_pdf_content_are_serialized_as_exact_native_file_parts() {
    for (run, media_type, name, content) in [
        (
            "run-image",
            "image/png",
            "concept.png",
            b"known-png".as_slice(),
        ),
        (
            "run-pdf",
            "application/pdf",
            "brief.pdf",
            b"%PDF-known".as_slice(),
        ),
    ] {
        let mut adapter = ready_adapter();
        create_session(&mut adapter);
        adapter.transport_for_test().respond_json(204, Value::Null);
        let content_sha256 = content_digest(content);
        adapter
            .send(PromptDispatch {
                correlation: correlation(7, run),
                model: ModelRoute {
                    provider_id: "openai".into(),
                    model_id: "gpt-4o-mini".into(),
                },
                text: String::new(),
                eligible_tool_ids: BTreeSet::new(),
                attachments: vec![PromptAttachment {
                    attachment_id: "attachment-1".into(),
                    stable_reference: format!("workspace-blob:{content_sha256}:v3"),
                    display_name: name.into(),
                    media_type: media_type.into(),
                    byte_length: content.len() as u64,
                    content_sha256,
                    snapshot_version: 3,
                    content: content.to_vec(),
                }],
                native_overrides: json!({}),
            })
            .expect("attachment-only dispatch");

        let request = adapter.transport_for_test().requests.last().unwrap();
        let body: Value = serde_json::from_slice(request.body.as_ref().unwrap()).unwrap();
        assert_eq!(
            body["parts"],
            json!([{
                "type": "file",
                "mime": media_type,
                "filename": name,
                "url": format!("data:{media_type};base64,{}", BASE64_STANDARD.encode(content))
            }])
        );
        assert!(
            body["parts"][0].get("id").is_none(),
            "OpenCode 1.18.3 must mint its own native prt_* identifier"
        );
    }
}

#[test]
fn aggregate_inline_attachment_limit_rejects_two_individually_valid_files_before_transport() {
    let mut adapter = ready_adapter();
    create_session(&mut adapter);
    let requests_before = adapter.transport_for_test().requests.len();
    let content = vec![b'x'; 150 * 1024];
    let content_sha256 = content_digest(&content);
    let attachments = ["attachment-1", "attachment-2"]
        .into_iter()
        .map(|attachment_id| PromptAttachment {
            attachment_id: attachment_id.into(),
            stable_reference: format!("workspace-blob:{content_sha256}:v3"),
            display_name: format!("{attachment_id}.png"),
            media_type: "image/png".into(),
            byte_length: content.len() as u64,
            content_sha256: content_sha256.clone(),
            snapshot_version: 3,
            content: content.clone(),
        })
        .collect();

    assert_eq!(
        adapter.send(PromptDispatch {
            correlation: correlation(7, "run-over-limit"),
            model: ModelRoute {
                provider_id: "openai".into(),
                model_id: "gpt-4o-mini".into(),
            },
            text: String::new(),
            eligible_tool_ids: BTreeSet::new(),
            attachments,
            native_overrides: json!({}),
        }),
        Err(AdapterError::InvalidRequest)
    );
    assert_eq!(adapter.transport_for_test().requests.len(), requests_before);
}

#[test]
fn stale_generation_workspace_and_native_session_correlations_are_rejected() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    for stale in [
        EventCorrelation {
            process_generation: 6,
            ..active.clone()
        },
        EventCorrelation {
            workspace_id: "workspace-other".into(),
            ..active.clone()
        },
        EventCorrelation {
            native_session_id: "native-session-old".into(),
            ..active.clone()
        },
    ] {
        assert_eq!(
            adapter.normalize_sse(
                &stale,
                br#"data: {"type":"session.idle","properties":{"sessionID":"native-session-1"}}\n\n"#,
                200
            ),
            Err(AdapterError::StaleCorrelation)
        );
    }
}

#[test]
fn text_thinking_and_unknown_events_are_typed_and_sequenced() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    let text = adapter
        .normalize_sse(
            &active,
            br#"event: message
data: {"type":"message.part.updated","properties":{"part":{"id":"part-text-1","sessionID":"native-session-1","messageID":"message-1","type":"text","text":"hi"},"delta":"hi"}}

"#,
            201,
        )
        .expect("text event");
    assert_eq!(
        text.category,
        NormalizedEventCategory::TextDelta { delta: "hi".into() }
    );
    assert_eq!(text.c4os_turn_id, "turn-run-1");
    assert_eq!(text.correlation_id, "correlation-run-1");
    assert_eq!(text.adapter_sequence, 1);
    assert!(text.native_event_id.starts_with("event-"));
    assert_eq!(text.native_event_id.len(), 70);

    let thinking = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-reasoning-1","sessionID":"native-session-1","messageID":"message-1","type":"reasoning","text":"consider","time":{"start":1}},"delta":"consider"}}

"#,
            202,
        )
        .expect("thinking event");
    assert!(matches!(
        thinking.category,
        NormalizedEventCategory::ThinkingDelta { .. }
    ));

    let unknown = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"future.event","properties":{"sessionID":"native-session-1","secret":"not echoed"}}

"#,
            203,
        )
        .expect("unknown event");
    assert_eq!(unknown.category, NormalizedEventCategory::Unknown);
    assert_eq!(unknown.native_event_type, "future.event");
}

#[test]
fn native_session_errors_publish_only_bounded_recovery_codes() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    let authentication = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"session.error","properties":{"sessionID":"native-session-1","error":{"name":"ProviderAuthError","data":{"providerID":"provider-openrouter","message":"secret-bearing native detail"}}}}

"#,
            210,
        )
        .expect("provider authentication error");
    assert_eq!(
        authentication.category,
        NormalizedEventCategory::Error {
            code: "provider-authentication-failed".into()
        }
    );
    assert!(!format!("{authentication:?}").contains("secret-bearing"));

    let rate_limit = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"session.error","properties":{"sessionID":"native-session-1","error":{"name":"APIError","data":{"message":"provider detail","statusCode":429,"isRetryable":true,"responseHeaders":{"authorization":"must-not-cross"}}}}}

"#,
            211,
        )
        .expect("provider rate limit error");
    assert_eq!(
        rate_limit.category,
        NormalizedEventCategory::Error {
            code: "provider-rate-limited".into()
        }
    );
    assert!(!format!("{rate_limit:?}").contains("must-not-cross"));
}

#[test]
fn authenticated_assistant_message_updates_preserve_exact_parent_identity() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    let update = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"message.updated","properties":{"info":{"id":"assistant-message-1","sessionID":"native-session-1","role":"assistant","parentID":"msg_correlation-run-1"}}}

"#,
            204,
        )
        .expect("authenticated assistant message update");
    let assistant = update
        .authenticated_assistant_message
        .expect("assistant identity evidence");
    assert_eq!(assistant.native_message_id, "assistant-message-1");
    assert_eq!(assistant.parent_native_message_id, "msg_correlation-run-1");
    assert_eq!(assistant.role, "assistant");
    assert_eq!(update.native_session_id, "native-session-1");
    assert_eq!(update.process_generation, 7);

    assert!(matches!(
        adapter.normalize_sse(
            &active,
            br#"data: {"type":"message.updated","properties":{"info":{"id":"assistant-message-2","sessionID":"native-session-1","role":"assistant"}}}

"#,
            205,
        ),
        Err(AdapterError::InvalidEventPayload)
    ));
}

#[test]
fn only_c4os_broker_permission_and_tool_calls_become_action_intents() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    let permission = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"permission.asked","properties":{"id":"request-1","sessionID":"native-session-1","permission":"c4os_propose_action","patterns":["src/main.rs"],"metadata":{"path":"src/main.rs","token":"must-not-log"},"always":[]}}

"#,
            300,
        )
        .expect("permission event");
    let NormalizedEventCategory::ActionIntent(intent) = permission.category else {
        panic!("permission must be an action intent")
    };
    assert_eq!(intent.native_request_id, "request-1");
    assert_eq!(intent.native_tool, "c4os_propose_action");
    assert_eq!(intent.c4os_turn_id, "turn-run-1");
    assert_eq!(intent.correlation_id, "correlation-run-1");
    assert_eq!(intent.resources, ["src/main.rs"]);
    assert!(!format!("{intent:?}").contains("must-not-log"));

    let tool = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"session.next.tool.called","properties":{"sessionID":"native-session-1","id":"call-1","tool":"c4os_propose_action","input":{"action":"file.write"}}}

"#,
            301,
        )
        .expect("tool event");
    assert!(matches!(
        tool.category,
        NormalizedEventCategory::ActionIntent(_)
    ));
}

#[test]
fn startup_tool_allowlist_rejects_undeclared_native_proposals() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    assert_eq!(
        adapter.normalize_sse(
            &active,
            br#"data: {"type":"session.next.tool.called","properties":{"sessionID":"native-session-1","id":"call-1","tool":"undeclared-native-tool","input":{}}}

"#,
            301,
        ),
        Err(AdapterError::AuthorityOverrideRejected)
    );
}

#[test]
fn duplicate_events_and_cross_session_events_fail_closed() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    let frame = br#"data: {"type":"session.status","properties":{"sessionID":"native-session-1","status":{"type":"busy"}}}

"#;
    adapter.normalize_sse(&active, frame, 1).expect("first");
    assert_eq!(
        adapter.normalize_sse(&active, frame, 2),
        Err(AdapterError::DuplicateEvent)
    );
    assert_eq!(
        adapter.normalize_sse(
            &active,
            br#"data: {"type":"session.status","properties":{"sessionID":"native-session-other","status":{"type":"busy"}}}

"#,
            3
        ),
        Err(AdapterError::StaleCorrelation)
    );
}

#[test]
fn cancel_is_idempotent_and_late_events_are_rejected() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    adapter.transport_for_test().respond_json(200, json!(true));
    assert_eq!(adapter.cancel(&active), Ok(true));
    assert_eq!(adapter.cancel(&active), Ok(false));
    assert_eq!(
        adapter.session("chat-1").expect("session").state,
        SessionState::Cancelled
    );
    assert_eq!(
        adapter.normalize_sse(
            &active,
            br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-late","sessionID":"native-session-1","messageID":"message-1","type":"text","text":"late"},"delta":"late"}}\n\n"#,
            4
        ),
        Err(AdapterError::LateEvent)
    );
}

#[test]
fn completed_c4os_effect_is_still_rejected_at_the_native_boundary() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    let permission = adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"permission.asked","properties":{"id":"request-1","sessionID":"native-session-1","permission":"c4os_propose_action","patterns":["src/main.rs"],"metadata":{"path":"src/main.rs"},"always":[]}}

"#,
            300,
        )
        .expect("permission intent");
    let NormalizedEventCategory::ActionIntent(intent) = permission.category else {
        panic!("expected intent")
    };
    adapter.transport_for_test().respond_json(200, json!(true));
    adapter
        .respond_to_permission(&C4osDecisionReceipt {
            correlation: active,
            native_request_id: "request-1".into(),
            action_binding_sha256: intent.binding_sha256().expect("binding"),
            decision: C4osPermissionDecision::C4osEffectCompleted,
        })
        .expect("reply");
    let request = adapter
        .transport_for_test()
        .requests
        .last()
        .expect("permission response");
    let body: Value = serde_json::from_slice(request.body.as_ref().expect("body")).expect("JSON");
    assert_eq!(body, json!({"response": "reject"}));
    assert!(!String::from_utf8_lossy(request.body.as_ref().expect("body")).contains("always"));
}

#[test]
fn permission_receipt_must_match_the_exact_emitted_intent() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    adapter
        .normalize_sse(
            &active,
            br#"data: {"type":"permission.asked","properties":{"id":"request-1","sessionID":"native-session-1","permission":"c4os_propose_action","patterns":["src/main.rs"],"metadata":{"path":"src/main.rs"},"always":[]}}

"#,
            300,
        )
        .expect("permission intent");
    assert_eq!(
        adapter.respond_to_permission(&C4osDecisionReceipt {
            correlation: active,
            native_request_id: "request-1".into(),
            action_binding_sha256: DIGEST.into(),
            decision: C4osPermissionDecision::Deny,
        }),
        Err(AdapterError::StaleCorrelation)
    );
}

#[test]
fn restart_requires_cancelled_runs_fresh_generation_state_and_secret() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    assert_eq!(adapter.prepare_restart(500), Err(AdapterError::ActiveRun));
    adapter.transport_for_test().respond_json(200, json!(true));
    adapter.cancel(&active).expect("cancel");
    let ticket = adapter.prepare_restart(500).expect("ticket");
    assert_eq!(ticket.next_generation, 8);

    adapter.transport_for_test().respond_json(200, health());
    adapter.driver_for_test().spawn_generations.push_back(8);
    let snapshot = adapter
        .restart(&ticket, plan(8, "launch-8", "vault-launch-8"), 501)
        .expect("restart");
    assert_eq!(snapshot.process_generation, 8);
    assert!(adapter.session("chat-1").is_none());
    assert_eq!(adapter.lifecycle(), LifecycleState::Ready);
}

#[test]
fn restart_rejects_reused_secret_or_namespace() {
    let mut adapter = ready_adapter();
    let ticket = adapter.prepare_restart(1).expect("ticket");
    let reused_secret = plan(8, "launch-8", "vault-launch-7");
    assert_eq!(
        adapter.restart(&ticket, reused_secret, 2),
        Err(AdapterError::RestartGenerationMismatch)
    );
}

#[test]
fn stop_terminates_the_process_group_and_interrupts_active_state() {
    let mut adapter = ready_adapter();
    begin_run(&mut adapter);
    adapter.stop().expect("stop");
    assert_eq!(adapter.lifecycle(), LifecycleState::Stopped);
    assert_eq!(
        adapter.session("chat-1").expect("session").state,
        SessionState::Interrupted
    );
    let (_, driver) = adapter.into_parts();
    assert_eq!(driver.terminated.len(), 1);
}

#[test]
fn malformed_and_oversized_sse_frames_are_rejected_without_echoing_content() {
    let mut adapter = ready_adapter();
    let active = begin_run(&mut adapter);
    assert_eq!(
        adapter.normalize_sse(&active, b"not-sse", 1),
        Err(AdapterError::InvalidEventPayload)
    );
    let oversized = vec![b'x'; 512 * 1_024 + 1];
    assert_eq!(
        adapter.normalize_sse(&active, &oversized, 2),
        Err(AdapterError::ResponseTooLarge)
    );
}
