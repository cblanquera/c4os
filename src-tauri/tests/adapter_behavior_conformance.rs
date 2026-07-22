use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use c4os_lib::runtime::capability::CapabilityState;
use c4os_lib::runtime::opencode::{
    AdapterError as OpenCodeError, C4osDecisionReceipt, C4osPermissionDecision, CommandDriver,
    CommandFailureCode, EventCorrelation, LaunchCommand, LifecycleState, LoopbackEndpoint,
    ModelRoute as OpenCodeModelRoute, NativeAuthorityPolicy, NormalizedEventCategory,
    OpenCodeAdapter, OpenCodeCompatibilityManifest, OpenCodeLaunchPlan, OpenCodeTransport,
    ProcessHandle, PromptDispatch, RandomSecretReference, StateNamespace, TransportFailureCode,
    TransportRequest, TransportResponse,
};
use c4os_lib::runtime::pi::{
    PI_MAX_LINE_BYTES, PI_NATIVE_VERSION, PiAdapter, PiAdapterState, PiModelRoute,
    PiSidecarManifest, PiSidecarRunner, PiToolDecision,
};
use serde_json::{Value, json};

const GENERATION: u64 = 7;
const DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const SHUTDOWN_BOUND: Duration = Duration::from_millis(250);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BehavioralState {
    Ready,
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HealthObservation {
    native_version: String,
    process_generation: u64,
    state: BehavioralState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EventObservation {
    workspace_id: String,
    session_id: String,
    turn_id: Option<String>,
    run_id: String,
    correlation_id: Option<String>,
    process_generation: u64,
    category: String,
    delta: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IntentObservation {
    workspace_id: String,
    session_id: String,
    turn_id: String,
    run_id: String,
    correlation_id: String,
    native_request_id: String,
    native_tool: String,
    native_effect_before_denial: bool,
    denial_granted_native_execution: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CapabilityObservation {
    model_discovery: CapabilityState,
    action_intents: CapabilityState,
    restart: CapabilityState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShutdownObservation {
    elapsed: Duration,
    stopped: bool,
    termination_count: usize,
}

#[derive(Clone, Copy)]
struct PeerExpectation {
    native_version: &'static str,
    state: BehavioralState,
    capabilities: CapabilityObservation,
}

/// The same behavior sequence is compiled and executed for both actual
/// adapters. Implementations may translate their native protocol, but cannot
/// omit or replace an assertion with a descriptor claim.
trait BehavioralPeer {
    fn start_and_health(&mut self) -> HealthObservation;
    fn capabilities(&self) -> CapabilityObservation;
    fn create_session(&mut self);
    fn dispatch_streaming_event(&mut self) -> EventObservation;
    fn cancel_twice(&mut self) -> (bool, bool);
    fn reject_late_and_stale_events(&mut self) -> usize;
    fn create_broker_session(&mut self);
    fn dispatch_broker_intent_and_deny(&mut self) -> IntentObservation;
    fn shutdown_twice(&mut self) -> ShutdownObservation;
}

fn assert_shared_behavior<P: BehavioralPeer>(mut peer: P, expected: PeerExpectation) {
    let health = peer.start_and_health();
    assert_eq!(health.native_version, expected.native_version);
    assert_eq!(health.process_generation, GENERATION);
    assert_eq!(health.state, expected.state);

    // Claims remain explicit evidence labels. The behavior checks below still
    // execute even when a peer honestly reports a degraded capability.
    assert_eq!(peer.capabilities(), expected.capabilities);

    peer.create_session();
    let event = peer.dispatch_streaming_event();
    assert_eq!(event.workspace_id, "workspace-1");
    assert_eq!(event.session_id, "session-1");
    assert_eq!(event.run_id, "run-stream");
    assert_eq!(event.process_generation, GENERATION);
    assert_eq!(event.category, "text-delta");
    assert_eq!(event.delta, "hello");
    assert_eq!(event.turn_id.as_deref(), Some("turn-stream"));
    assert_eq!(event.correlation_id.as_deref(), Some("correlation-stream"));

    assert_eq!(peer.cancel_twice(), (true, false));
    assert_eq!(peer.reject_late_and_stale_events(), 2);

    // A cancelled native Chat is terminal. Use a second actual native session
    // for the broker path instead of assuming either peer permits resurrection.
    peer.create_broker_session();
    let intent = peer.dispatch_broker_intent_and_deny();
    assert_eq!(intent.workspace_id, "workspace-1");
    assert_eq!(intent.session_id, "session-broker");
    assert_eq!(intent.turn_id, "turn-broker");
    assert_eq!(intent.run_id, "run-broker");
    assert_eq!(intent.correlation_id, "correlation-broker");
    assert!(!intent.native_request_id.is_empty());
    assert!(matches!(
        intent.native_tool.as_str(),
        "c4os_propose_action" | "c4os_read_resource"
    ));
    assert!(!intent.native_effect_before_denial);
    assert!(!intent.denial_granted_native_execution);

    let shutdown = peer.shutdown_twice();
    assert!(shutdown.elapsed <= SHUTDOWN_BOUND);
    assert!(shutdown.stopped);
    assert_eq!(shutdown.termination_count, 1);
}

#[test]
fn actual_peer_adapters_pass_one_shared_behavioral_conformance_sequence() {
    assert_shared_behavior(
        OpenCodePeer::new(),
        PeerExpectation {
            native_version: "1.18.3",
            state: BehavioralState::Ready,
            capabilities: CapabilityObservation {
                model_discovery: CapabilityState::Supported,
                action_intents: CapabilityState::Degraded,
                restart: CapabilityState::Degraded,
            },
        },
    );
    assert_shared_behavior(
        PiPeer::new(),
        PeerExpectation {
            native_version: "0.80.10",
            state: BehavioralState::Degraded,
            capabilities: CapabilityObservation {
                model_discovery: CapabilityState::Unsupported,
                action_intents: CapabilityState::Supported,
                restart: CapabilityState::Degraded,
            },
        },
    );
}

#[derive(Default)]
struct OpenCodeFakeTransport {
    responses: VecDeque<Result<TransportResponse, TransportFailureCode>>,
    requests: Vec<TransportRequest>,
}

impl OpenCodeFakeTransport {
    fn respond_json(&mut self, status: u16, value: Value) {
        self.responses.push_back(Ok(TransportResponse {
            status,
            body: serde_json::to_vec(&value).unwrap(),
        }));
    }
}

impl OpenCodeTransport for OpenCodeFakeTransport {
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
struct OpenCodeFakeDriver {
    spawn_generations: VecDeque<u64>,
    terminations: usize,
}

impl CommandDriver for OpenCodeFakeDriver {
    fn spawn(&mut self, _command: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode> {
        Ok(ProcessHandle {
            process_id: 4_001,
            process_generation: self
                .spawn_generations
                .pop_front()
                .ok_or(CommandFailureCode::SpawnRejected)?,
        })
    }

    fn terminate_process_group(
        &mut self,
        _process: &ProcessHandle,
    ) -> Result<(), CommandFailureCode> {
        self.terminations += 1;
        Ok(())
    }
}

type ActualOpenCodeAdapter = OpenCodeAdapter<OpenCodeFakeTransport, OpenCodeFakeDriver>;

struct OpenCodePeer {
    adapter: ActualOpenCodeAdapter,
    stream: EventCorrelation,
    broker: EventCorrelation,
}

impl OpenCodePeer {
    fn new() -> Self {
        let namespace = StateNamespace::new(
            PathBuf::from("/private/tmp/c4os-conformance").as_path(),
            "workspace-1",
            GENERATION,
            "behavioral-harness",
        )
        .unwrap();
        let launch_plan = OpenCodeLaunchPlan {
            manifest: OpenCodeCompatibilityManifest::pinned(DIGEST).unwrap(),
            endpoint: LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 49_174).unwrap(),
            namespace,
            executable: PathBuf::from("/Applications/C4OS.app/Contents/MacOS/opencode"),
            workspace_root: PathBuf::from("/private/tmp/c4os-behavior-workspace"),
            basic_auth_username: "opencode".into(),
            password_reference: RandomSecretReference::new("behavior-secret", 256).unwrap(),
            secret_channel_fd: 7,
            authority_policy: NativeAuthorityPolicy::new(
                DIGEST,
                ["c4os_read_resource", "c4os_propose_action"],
            )
            .unwrap(),
        };
        let mut transport = OpenCodeFakeTransport::default();
        transport.respond_json(200, json!({ "healthy": true, "version": "1.18.3" }));
        let mut driver = OpenCodeFakeDriver::default();
        driver.spawn_generations.push_back(GENERATION);
        Self {
            adapter: OpenCodeAdapter::new(launch_plan, transport, driver).unwrap(),
            stream: opencode_correlation("run-stream"),
            broker: EventCorrelation {
                workspace_id: "workspace-1".into(),
                c4os_session_id: "session-broker".into(),
                c4os_turn_id: "turn-broker".into(),
                c4os_run_id: "run-broker".into(),
                correlation_id: "correlation-broker".into(),
                native_session_id: "native-session-2".into(),
                process_generation: GENERATION,
            },
        }
    }

    fn dispatch(&mut self, correlation: EventCorrelation, text: &str) {
        self.adapter
            .test_transport_mut()
            .respond_json(204, Value::Null);
        self.adapter
            .send(PromptDispatch {
                correlation,
                model: OpenCodeModelRoute {
                    provider_id: "openai".into(),
                    model_id: "gpt-4o-mini".into(),
                },
                text: text.into(),
                eligible_tool_ids: BTreeSet::from(["c4os_propose_action".into()]),
                attachments: vec![],
                native_overrides: json!({}),
            })
            .unwrap();
    }
}

impl BehavioralPeer for OpenCodePeer {
    fn start_and_health(&mut self) -> HealthObservation {
        let health = self.adapter.start(100).unwrap();
        HealthObservation {
            native_version: health.native_version,
            process_generation: health.process_generation,
            state: match self.adapter.lifecycle() {
                LifecycleState::Ready => BehavioralState::Ready,
                LifecycleState::Degraded => BehavioralState::Degraded,
                other => panic!("unexpected OpenCode state: {other:?}"),
            },
        }
    }

    fn capabilities(&self) -> CapabilityObservation {
        let descriptor = OpenCodeCompatibilityManifest::pinned(DIGEST)
            .unwrap()
            .conformance_descriptor(GENERATION)
            .unwrap();
        CapabilityObservation {
            model_discovery: descriptor.capabilities["model-discovery"],
            action_intents: descriptor.capabilities["action-intents"],
            restart: descriptor.capabilities["restart"],
        }
    }

    fn create_session(&mut self) {
        self.adapter.test_transport_mut().respond_json(
            200,
            json!({
                "id": "native-session-1",
                "slug": "behavior",
                "projectID": "native-project-1",
                "directory": "/private/tmp/c4os-behavior-workspace",
                "title": "Behavior session",
                "version": "1.18.3",
                "time": { "created": 10, "updated": 11 }
            }),
        );
        let binding = self
            .adapter
            .create_session("session-1", "Behavior session")
            .unwrap();
        assert_eq!(binding.workspace_id, "workspace-1");
        assert_eq!(binding.c4os_session_id, "session-1");
        assert_eq!(binding.process_generation, GENERATION);
    }

    fn dispatch_streaming_event(&mut self) -> EventObservation {
        self.dispatch(self.stream.clone(), "stream");
        let event = self
            .adapter
            .normalize_sse(
                &self.stream,
                br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-stream-1","sessionID":"native-session-1","messageID":"message-stream-1","type":"text","text":"hello"},"delta":"hello"}}

"#,
                101,
            )
            .unwrap();
        let NormalizedEventCategory::TextDelta { delta } = event.category else {
            panic!("OpenCode did not normalize the streaming delta");
        };
        EventObservation {
            workspace_id: event.workspace_id,
            session_id: event.c4os_session_id,
            turn_id: Some(event.c4os_turn_id),
            run_id: event.c4os_run_id,
            correlation_id: Some(event.correlation_id),
            process_generation: event.process_generation,
            category: "text-delta".into(),
            delta,
        }
    }

    fn cancel_twice(&mut self) -> (bool, bool) {
        self.adapter
            .test_transport_mut()
            .respond_json(200, json!(true));
        (
            self.adapter.cancel(&self.stream).unwrap(),
            self.adapter.cancel(&self.stream).unwrap(),
        )
    }

    fn reject_late_and_stale_events(&mut self) -> usize {
        let late = self.adapter.normalize_sse(
            &self.stream,
            br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-late-1","sessionID":"native-session-1","messageID":"message-stream-1","type":"text","text":"late"},"delta":"late"}}

"#,
            102,
        );
        let stale = self.adapter.normalize_sse(
            &EventCorrelation {
                process_generation: GENERATION - 1,
                ..self.stream.clone()
            },
            br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-stale-1","sessionID":"native-session-1","messageID":"message-stream-1","type":"text","text":"stale"},"delta":"stale"}}

"#,
            103,
        );
        usize::from(matches!(late, Err(OpenCodeError::LateEvent)))
            + usize::from(matches!(stale, Err(OpenCodeError::StaleCorrelation)))
    }

    fn create_broker_session(&mut self) {
        self.adapter.test_transport_mut().respond_json(
            200,
            json!({
                "id": "native-session-2",
                "slug": "broker",
                "projectID": "native-project-1",
                "directory": "/private/tmp/c4os-behavior-workspace",
                "title": "Broker session",
                "version": "1.18.3",
                "time": { "created": 12, "updated": 13 }
            }),
        );
        self.adapter
            .create_session("session-broker", "Broker session")
            .unwrap();
    }

    fn dispatch_broker_intent_and_deny(&mut self) -> IntentObservation {
        self.dispatch(self.broker.clone(), "broker");
        let permission_requests_before = self
            .adapter
            .test_transport_mut()
            .requests
            .iter()
            .filter(|request| request.path.contains("/permissions/"))
            .count();
        let event = self
            .adapter
            .normalize_sse(
                &self.broker,
                br#"data: {"type":"permission.asked","properties":{"id":"broker-call-1","sessionID":"native-session-2","permission":"c4os_propose_action","patterns":["README.md"],"metadata":{"path":"README.md"},"always":[]}}

"#,
                104,
            )
            .unwrap();
        let NormalizedEventCategory::ActionIntent(intent) = event.category else {
            panic!("OpenCode did not isolate the broker intent");
        };
        let binding = intent.binding_sha256().unwrap();
        self.adapter
            .test_transport_mut()
            .respond_json(200, json!(true));
        self.adapter
            .respond_to_permission(&C4osDecisionReceipt {
                correlation: self.broker.clone(),
                native_request_id: intent.native_request_id.clone(),
                action_binding_sha256: binding,
                decision: C4osPermissionDecision::Deny,
            })
            .unwrap();
        let denial = self.adapter.test_transport_mut().requests.last().unwrap();
        let denial_body: Value = serde_json::from_slice(denial.body.as_ref().unwrap()).unwrap();
        IntentObservation {
            workspace_id: intent.workspace_id,
            session_id: intent.c4os_session_id,
            turn_id: intent.c4os_turn_id,
            run_id: intent.c4os_run_id,
            correlation_id: intent.correlation_id,
            native_request_id: intent.native_request_id,
            native_tool: intent.native_tool,
            native_effect_before_denial: permission_requests_before != 0,
            denial_granted_native_execution: denial_body != json!({ "response": "reject" }),
        }
    }

    fn shutdown_twice(&mut self) -> ShutdownObservation {
        let started = Instant::now();
        self.adapter.stop().unwrap();
        self.adapter.stop().unwrap();
        ShutdownObservation {
            elapsed: started.elapsed(),
            stopped: self.adapter.lifecycle() == LifecycleState::Stopped,
            termination_count: self.adapter.driver_for_test().terminations,
        }
    }
}

fn opencode_correlation(run_id: &str) -> EventCorrelation {
    EventCorrelation {
        workspace_id: "workspace-1".into(),
        c4os_session_id: "session-1".into(),
        c4os_turn_id: "turn-stream".into(),
        c4os_run_id: run_id.into(),
        correlation_id: "correlation-stream".into(),
        native_session_id: "native-session-1".into(),
        process_generation: GENERATION,
    }
}

#[derive(Default)]
struct PiFakeRunner {
    requests: Vec<Value>,
    poll_lines: VecDeque<String>,
    next_sequence: u64,
    terminations: usize,
}

impl PiSidecarRunner for PiFakeRunner {
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String> {
        let request: Value =
            serde_json::from_str(request_line).map_err(|error| error.to_string())?;
        self.requests.push(request.clone());
        let operation = request["operation"].as_str().unwrap();
        let payload = match operation {
            "health" => pi_health(),
            "version" => json!({
                "adapter": "PIAdapter",
                "adapterVersion": "0.1.0",
                "nativePackage": "@earendil-works/pi-coding-agent",
                "nativeVersion": PI_NATIVE_VERSION,
                "protocol": "c4os.pi.ndjson.v1"
            }),
            "session.create" => json!({
                "sessionId": request["sessionId"],
                "nativeSessionId": "native-session-1",
                "persistence": "c4os-authoritative"
            }),
            "dispatch" => json!({ "accepted": true, "runId": request["runId"] }),
            "tool.resolve" => json!({ "resolved": true, "executedBySidecar": false }),
            "cancel" => json!({ "cancelled": true, "alreadyTerminal": false }),
            "shutdown" => json!({ "stopped": true }),
            _ => json!({}),
        };
        let mut lines = Vec::new();
        if operation == "dispatch" {
            self.next_sequence += 1;
            let input = request["payload"]["input"].as_str().unwrap();
            if input == "stream" {
                lines.push(pi_event_from_request(
                    &request,
                    self.next_sequence,
                    "content.delta",
                    None,
                    json!({ "delta": "hello" }),
                ));
            } else if input == "broker" {
                lines.push(pi_event_from_request(
                    &request,
                    self.next_sequence,
                    "tool.action_intent",
                    Some("broker-call-1"),
                    json!({
                        "tool": "c4os_read_resource",
                        "arguments": { "target": "workspace:/README.md" },
                        "authority": "c4os-action-gateway-required"
                    }),
                ));
            }
        }
        lines.push(
            serde_json::to_string(&json!({
                "schemaVersion": 1,
                "kind": "response",
                "requestId": request["requestId"],
                "correlationId": request["correlationId"],
                "processGeneration": GENERATION,
                "status": "ok",
                "payload": payload
            }))
            .unwrap(),
        );
        Ok(lines)
    }

    fn poll(&mut self) -> Result<Vec<String>, String> {
        Ok(self.poll_lines.drain(..).collect())
    }

    fn terminate(&mut self) -> Result<(), String> {
        self.terminations += 1;
        Ok(())
    }
}

struct PiPeer {
    adapter: PiAdapter<PiFakeRunner>,
}

impl PiPeer {
    fn new() -> Self {
        let sidecar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
        Self {
            adapter: PiAdapter::new(
                PiSidecarManifest::load(&sidecar).unwrap(),
                PiFakeRunner::default(),
                GENERATION,
            )
            .unwrap(),
        }
    }
}

impl BehavioralPeer for PiPeer {
    fn start_and_health(&mut self) -> HealthObservation {
        let (health, version) = self.adapter.start().unwrap();
        HealthObservation {
            native_version: version.native_version,
            process_generation: health.process_generation,
            state: match self.adapter.state() {
                PiAdapterState::Ready => BehavioralState::Ready,
                PiAdapterState::Degraded => BehavioralState::Degraded,
                other => panic!("unexpected Pi state: {other:?}"),
            },
        }
    }

    fn capabilities(&self) -> CapabilityObservation {
        let descriptor = self
            .adapter
            .manifest()
            .conformance_descriptor(GENERATION, true)
            .unwrap();
        CapabilityObservation {
            model_discovery: descriptor.capabilities["model-discovery"],
            action_intents: descriptor.capabilities["action-intents"],
            restart: descriptor.capabilities["restart"],
        }
    }

    fn create_session(&mut self) {
        self.adapter
            .create_session(
                "workspace-1",
                "session-1",
                PiModelRoute {
                    provider: "openai".into(),
                    model_id: "gpt-4o-mini".into(),
                    base_url: "https://api.openai.com/v1".into(),
                },
                &BTreeSet::from(["c4os_propose_action".into()]),
            )
            .unwrap();
    }

    fn dispatch_streaming_event(&mut self) -> EventObservation {
        self.adapter
            .dispatch(
                "workspace-1",
                "session-1",
                "turn-stream",
                "run-stream",
                "correlation-stream",
                "stream",
            )
            .unwrap();
        let event = self.adapter.take_events().pop().unwrap();
        EventObservation {
            workspace_id: event.workspace_id,
            session_id: event.session_id,
            turn_id: Some(event.turn_id),
            run_id: event.run_id,
            correlation_id: Some(event.correlation_id),
            process_generation: event.process_generation,
            category: if event.category == "content.delta" {
                "text-delta".into()
            } else {
                event.category
            },
            delta: event.payload["delta"].as_str().unwrap().into(),
        }
    }

    fn cancel_twice(&mut self) -> (bool, bool) {
        (
            self.adapter
                .cancel(
                    "workspace-1",
                    "session-1",
                    "turn-stream",
                    "run-stream",
                    "correlation-stream",
                )
                .unwrap(),
            self.adapter
                .cancel(
                    "workspace-1",
                    "session-1",
                    "turn-stream",
                    "run-stream",
                    "correlation-stream",
                )
                .unwrap(),
        )
    }

    fn reject_late_and_stale_events(&mut self) -> usize {
        let before = self.adapter.stale_events_rejected();
        self.adapter
            .runner_mut_ref()
            .poll_lines
            .push_back(pi_event_line(GENERATION, 2, "late-1", "run-stream"));
        self.adapter
            .runner_mut_ref()
            .poll_lines
            .push_back(pi_event_line(GENERATION - 1, 3, "stale-1", "run-stream"));
        assert!(self.adapter.poll_events().unwrap().is_empty());
        usize::try_from(self.adapter.stale_events_rejected() - before).unwrap()
    }

    fn create_broker_session(&mut self) {
        self.adapter
            .create_session(
                "workspace-1",
                "session-broker",
                PiModelRoute {
                    provider: "openai".into(),
                    model_id: "gpt-4o-mini".into(),
                    base_url: "https://api.openai.com/v1".into(),
                },
                &BTreeSet::from(["c4os_propose_action".into()]),
            )
            .unwrap();
    }

    fn dispatch_broker_intent_and_deny(&mut self) -> IntentObservation {
        self.adapter
            .dispatch(
                "workspace-1",
                "session-broker",
                "turn-broker",
                "run-broker",
                "correlation-broker",
                "broker",
            )
            .unwrap();
        let event = self.adapter.take_events().pop().unwrap();
        let resolutions_before = self
            .adapter
            .runner_ref()
            .requests
            .iter()
            .filter(|request| request["operation"] == "tool.resolve")
            .count();
        self.adapter
            .resolve_tool(
                "session-broker",
                "run-broker",
                "correlation-broker",
                event.tool_call_id.as_deref().unwrap(),
                PiToolDecision::Denied {
                    reason: "policy-denied".into(),
                },
            )
            .unwrap();
        let resolution = self.adapter.runner_ref().requests.last().unwrap();
        IntentObservation {
            workspace_id: event.workspace_id,
            session_id: event.session_id,
            turn_id: event.turn_id,
            run_id: event.run_id,
            correlation_id: event.correlation_id,
            native_request_id: event.tool_call_id.unwrap(),
            native_tool: event.payload["tool"].as_str().unwrap().into(),
            native_effect_before_denial: resolutions_before != 0,
            denial_granted_native_execution: resolution["payload"]["decision"] != "denied",
        }
    }

    fn shutdown_twice(&mut self) -> ShutdownObservation {
        let started = Instant::now();
        self.adapter.shutdown().unwrap();
        self.adapter.shutdown().unwrap();
        ShutdownObservation {
            elapsed: started.elapsed(),
            stopped: self.adapter.state() == &PiAdapterState::Stopped,
            termination_count: self.adapter.runner_ref().terminations,
        }
    }
}

fn pi_health() -> Value {
    let mut capabilities = BTreeMap::new();
    capabilities.insert("streaming", json!({ "state": "supported" }));
    capabilities.insert("cancellation", json!({ "state": "supported" }));
    capabilities.insert("tools", json!({ "state": "supported" }));
    capabilities.insert("nativePersistence", json!({ "state": "unsupported" }));
    capabilities.insert("nativeExtensions", json!({ "state": "unsupported" }));
    capabilities.insert("nativeTools", json!({ "state": "unsupported" }));
    capabilities.insert(
        "crashResume",
        json!({ "state": "degraded", "reason": "C4OS replay required" }),
    );
    capabilities.insert(
        "providerAuthentication",
        json!({ "state": "degraded", "reason": "fixture channel" }),
    );
    capabilities.insert("rpcTransport", json!({ "state": "unsupported" }));
    json!({
        "status": "ready",
        "runtime": "pi",
        "transport": "c4os-node-sdk-sidecar",
        "protocol": "c4os.pi.ndjson.v1",
        "processGeneration": GENERATION,
        "sessions": 0,
        "staleEventsRejected": 0,
        "capabilities": capabilities
    })
}

fn pi_event_from_request(
    request: &Value,
    sequence: u64,
    category: &str,
    tool_call_id: Option<&str>,
    payload: Value,
) -> String {
    let mut event = json!({
        "schemaVersion": 1,
        "kind": "event",
        "eventId": format!("pi-event-{sequence}"),
        "correlationId": request["correlationId"],
        "processGeneration": GENERATION,
        "sequence": sequence,
        "runtime": "pi",
        "workspaceId": request["workspaceId"],
        "sessionId": request["sessionId"],
        "turnId": request["turnId"],
        "runId": request["runId"],
        "category": category,
        "nativeType": "fixture.native.event",
        "payload": payload
    });
    if let Some(tool_call_id) = tool_call_id {
        event["toolCallId"] = Value::String(tool_call_id.into());
    }
    let encoded = serde_json::to_string(&event).unwrap();
    assert!(encoded.len() < PI_MAX_LINE_BYTES);
    encoded
}

fn pi_event_line(generation: u64, sequence: u64, event_id: &str, run_id: &str) -> String {
    serde_json::to_string(&json!({
        "schemaVersion": 1,
        "kind": "event",
        "eventId": event_id,
        "correlationId": "correlation-stream",
        "processGeneration": generation,
        "sequence": sequence,
        "runtime": "pi",
        "workspaceId": "workspace-1",
        "sessionId": "session-1",
        "turnId": "turn-stream",
        "runId": run_id,
        "category": "content.delta",
        "nativeType": "fixture.native.event",
        "payload": { "delta": "late" }
    }))
    .unwrap()
}
