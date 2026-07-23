#![allow(deprecated)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

use rmcp::model::{CreateMessageRequestParams, CreateMessageResult, SamplingMessage};
use tempfile::TempDir;

use super::{
    FirstRuntimeDispatchIntent, RuntimeApplicationService, answer_runtime_sampling_approval,
    current_time_ms,
    mcp::{
        McpError,
        production_sampling::{McpSamplingParentGuard, ProductionMcpSamplingBroker},
        transport::{McpCancellation, McpSamplingContext, broker_sampling_request},
    },
    production_broker_authoritative_resources,
    protocol::{CorrelationId, ProtocolError, ProtocolErrorCode},
    runtime::{
        adapter::{
            ADAPTER_CONTRACT_SCHEMA_VERSION, AdapterAuthority, AdapterConformanceDescriptor,
            PeerCapabilityClaims, peer_capabilities,
        },
        capability::{
            CapabilityKey, CapabilityState, DraftRequirements, InstalledResourcePreflight,
            LimitConfidence, ModelLifecycle, NumericCapabilityKey, PolicyPreflight, RouteIdentity,
        },
        capability_evidence::{
            CapabilityRouteEpoch, FeatureClaim, NumericClaim, ProviderDeclaredCatalogClaim,
            RuntimeObservationOutcome, RuntimeRouteObservation, pi_adapter_evidence,
            pi_observed_evidence, provider_declared_evidence,
        },
        dispatch::{
            AttachmentPreflightResolution, DispatchIdentity, FirstDispatchOptions,
            PeerDispatchError, PeerDispatchEvent, PeerDispatchRequest, PeerSamplingRequest,
            PeerSamplingResult, RuntimeDispatchPeer, RuntimePeerRegistration,
        },
        dispatch_authority::{
            AuthoritativeConfiguration, AuthorityMintIntent, ConfigurationFieldValue,
            authoritative_configuration_sha256, authoritative_resources_sha256,
        },
        pi::{PI_NATIVE_VERSION, PI_PROTOCOL, PiCapabilityState, PiHealth, PiModelRoute},
        provider::{
            ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, PROVIDER_SCHEMA_VERSION,
            ProviderAuthentication, ProviderConnectionEvidence, ProviderDiscovery,
            ProviderEndpoint, ProviderKind, ProviderModelDeclaration, ProviderProbe,
            ProviderProbeFailure, ProviderProfile, RouteAvailability,
        },
        supervisor::{
            HealthState, RUNTIME_PROTOCOL_VERSION, RuntimeInstallation, RuntimeKind, sha256_file,
        },
    },
    runtime_sampling_approval_summary,
    security::{
        authorization::{
            ApprovalAnswer, CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk,
        },
        credentials::CredentialVault,
    },
};
use crate::core::database::{
    ChatRecord, DatabaseActor, DatabaseDescriptor, LifecycleState, ProjectPathState, ProjectRecord,
    SnapshotQuery, WorkspaceRecord,
};

const SERVER_ID: &str = "docs";
const AUTHORITY_ID: &str = "mcp-docs";
const PRIVATE_CANARY: &str = "private prompt canary task12";

struct FixtureProbe(ProviderDiscovery);

impl ProviderProbe for FixtureProbe {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let mut discovery = self.0.clone();
        discovery.connection_evidence = Some(ProviderConnectionEvidence::from_tested_profile(
            profile,
            discovery.checked_at_ms,
            digest('9'),
        )?);
        Ok(discovery)
    }
}

struct SamplingPeer {
    registration: RuntimePeerRegistration,
    sampling_calls: Arc<AtomicUsize>,
    sampled: Arc<Mutex<Vec<PeerSamplingRequest>>>,
    wait_for_cancellation: Arc<AtomicBool>,
    post_cancel_delay_ms: Arc<AtomicU64>,
}

impl RuntimeDispatchPeer for SamplingPeer {
    fn registration(&self) -> &RuntimePeerRegistration {
        &self.registration
    }

    fn readiness(&self) -> Result<(), PeerDispatchError> {
        Ok(())
    }

    fn create_session(&mut self, _request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
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
        Ok(true)
    }

    fn sample(
        &mut self,
        request: &PeerSamplingRequest,
    ) -> Result<PeerSamplingResult, PeerDispatchError> {
        self.sampling_calls.fetch_add(1, Ordering::SeqCst);
        self.sampled.lock().unwrap().push(request.clone());
        if self.wait_for_cancellation.load(Ordering::SeqCst) {
            while !request.cancelled.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(2));
            }
            std::thread::sleep(Duration::from_millis(
                self.post_cancel_delay_ms.load(Ordering::SeqCst),
            ));
            return Err(PeerDispatchError::Cancellation);
        }
        Ok(PeerSamplingResult {
            text: "bounded production sampling response".into(),
            model_id: request.model.model_id.clone(),
            stop_reason: CreateMessageResult::STOP_REASON_END_TURN.into(),
        })
    }
}

struct SamplingFixture {
    _temporary: TempDir,
    database: Arc<DatabaseActor>,
    service: Arc<RuntimeApplicationService>,
    broker: Arc<ProductionMcpSamplingBroker>,
    _parent: McpSamplingParentGuard,
    context: McpSamplingContext,
    sampling_calls: Arc<AtomicUsize>,
    sampled: Arc<Mutex<Vec<PeerSamplingRequest>>>,
    wait_for_cancellation: Arc<AtomicBool>,
    post_cancel_delay_ms: Arc<AtomicU64>,
}

fn answer_sampling_through_tauri_command_core(
    fixture: &SamplingFixture,
    expected_generation: u64,
    approval: &super::mcp::production_sampling::McpSamplingApprovalSummary,
    answer: ApprovalAnswer,
) -> Result<bool, ProtocolError> {
    answer_runtime_sampling_approval(
        fixture.service.as_ref(),
        &fixture.broker.approvals(),
        CorrelationId::new("correlation-renderer-command").unwrap(),
        expected_generation,
        &approval.runtime_id,
        &approval.correlation_id,
        &approval.prompt_id,
        answer,
        current_time_ms().unwrap(),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn production_sampling_golden_path_requires_informed_approval_and_settles_once() {
    let fixture = sampling_fixture();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            McpCancellation::default(),
        )
        .await
    });

    let approval = wait_for_sampling_approval(&fixture, &mut task).await;
    assert_eq!(approval.server_id, SERVER_ID);
    assert_eq!(approval.provider_id, "provider-openai");
    assert_eq!(approval.model_id, "openai/gpt-4o-mini");
    assert_eq!(approval.max_tokens, 256);
    assert_eq!(approval.message_count, 1);
    assert!(approval.has_system_prompt);
    assert!(!format!("{approval:?}").contains(PRIVATE_CANARY));
    let rendered_projection = runtime_sampling_approval_summary(approval.clone());
    assert_eq!(rendered_projection.approval_kind, "mcp-sampling");
    assert_eq!(rendered_projection.server_id.as_deref(), Some(SERVER_ID));
    assert_eq!(
        rendered_projection.disclosure_scope.as_deref(),
        Some(
            "Private active-operation text will be disclosed to the selected model provider; credentials remain operation-scoped and hidden."
        )
    );
    assert!(
        !serde_json::to_string(&rendered_projection)
            .unwrap()
            .contains(PRIVATE_CANARY)
    );

    let generation = fixture
        .service
        .snapshot(current_time_ms().unwrap())
        .unwrap()
        .generation;
    assert!(
        answer_sampling_through_tauri_command_core(
            &fixture,
            generation,
            &approval,
            ApprovalAnswer::Allow,
        )
        .unwrap()
    );

    let result = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .expect("sampling completion timeout")
        .unwrap()
        .expect("production sampling result");
    assert_eq!(
        result.message,
        SamplingMessage::assistant_text("bounded production sampling response")
    );
    assert_eq!(result.model, "openai/gpt-4o-mini");
    assert_eq!(
        result.stop_reason,
        Some(CreateMessageResult::STOP_REASON_END_TURN.into())
    );
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 1);
    let sampled = fixture.sampled.lock().unwrap();
    assert_eq!(sampled.len(), 1);
    assert_eq!(sampled[0].identity.runtime_id, "pi-primary");
    assert_eq!(sampled[0].identity.workspace_id, "workspace-1");
    assert_eq!(sampled[0].messages[0].text, PRIVATE_CANARY);
    drop(sampled);

    let records = fixture
        .database
        .security_records(SnapshotQuery::new(100).unwrap())
        .unwrap();
    let sampling_records = records
        .iter()
        .filter(|record| record.action_id.starts_with("mcp-sampling-"))
        .collect::<Vec<_>>();
    assert_eq!(
        sampling_records
            .iter()
            .filter(|record| record.record_kind == "action-result" && record.state == "succeeded")
            .count(),
        1
    );
    assert!(sampling_records.iter().any(|record| {
        record.record_kind == "action-intent" && record.state == "effect-finished"
    }));
    assert!(
        sampling_records
            .iter()
            .all(|record| !record.canonical_document.contains(PRIVATE_CANARY))
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_approval_persistence_can_retry_the_same_exact_sampling_prompt() {
    let fixture = sampling_fixture();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            McpCancellation::default(),
        )
        .await
    });

    let approval = wait_for_sampling_approval(&fixture, &mut task).await;
    let connection = rusqlite::Connection::open(&fixture.database.descriptor().path).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER fail_sampling_approval_answer
             BEFORE INSERT ON security_events
             WHEN NEW.record_kind = 'approval-prompt' AND NEW.state = 'approved'
             BEGIN SELECT RAISE(ABORT, 'injected sampling approval failure'); END;",
        )
        .unwrap();
    let generation = fixture
        .service
        .snapshot(current_time_ms().unwrap())
        .unwrap()
        .generation;

    assert!(matches!(
        fixture.broker.approvals().answer(
            fixture.service.as_ref(),
            generation,
            &approval.runtime_id,
            &approval.correlation_id,
            &approval.prompt_id,
            ApprovalAnswer::Allow,
            current_time_ms().unwrap(),
        ),
        Err(McpError::StateUnavailable)
    ));
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 0);
    assert!(
        fixture
            .broker
            .approvals()
            .stable_summaries()
            .unwrap()
            .is_some_and(|(_, summaries)| summaries.iter().any(|summary| {
                summary.runtime_id == approval.runtime_id
                    && summary.correlation_id == approval.correlation_id
                    && summary.prompt_id == approval.prompt_id
            }))
    );

    connection
        .execute_batch("DROP TRIGGER fail_sampling_approval_answer;")
        .unwrap();
    assert!(
        fixture
            .broker
            .approvals()
            .answer(
                fixture.service.as_ref(),
                generation,
                &approval.runtime_id,
                &approval.correlation_id,
                &approval.prompt_id,
                ApprovalAnswer::Allow,
                current_time_ms().unwrap(),
            )
            .unwrap()
    );

    let result = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .expect("sampling completion timeout")
        .unwrap()
        .expect("sampling must complete after approval persistence retry");
    assert_eq!(result.model, "openai/gpt-4o-mini");
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn denying_sampling_approval_prevents_model_io_and_records_the_denial() {
    let fixture = sampling_fixture();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            McpCancellation::default(),
        )
        .await
    });

    let approval = wait_for_sampling_approval(&fixture, &mut task).await;
    let generation = fixture
        .service
        .snapshot(current_time_ms().unwrap())
        .unwrap()
        .generation;
    assert!(
        answer_sampling_through_tauri_command_core(
            &fixture,
            generation,
            &approval,
            ApprovalAnswer::Deny,
        )
        .unwrap()
    );

    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("sampling denial timeout")
            .unwrap(),
        Err(McpError::Denied)
    ));
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 0);
    assert!(wait_for_no_sampling_approvals(&fixture).await);
    assert!(
        fixture
            .database
            .security_records(SnapshotQuery::new(100).unwrap())
            .unwrap()
            .iter()
            .any(|record| {
                record.action_id.starts_with("mcp-sampling-")
                    && record.record_kind == "action-result"
                    && record.state == "denied"
            })
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stale_sampling_approval_generation_cannot_settle_or_reach_the_model() {
    let fixture = sampling_fixture();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            McpCancellation::default(),
        )
        .await
    });

    let approval = wait_for_sampling_approval(&fixture, &mut task).await;
    let generation = fixture
        .service
        .snapshot(current_time_ms().unwrap())
        .unwrap()
        .generation;
    assert!(matches!(
        answer_sampling_through_tauri_command_core(
            &fixture,
            generation.saturating_sub(1),
            &approval,
            ApprovalAnswer::Allow,
        ),
        Err(error) if error.code == ProtocolErrorCode::Unavailable
    ));
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 0);
    assert!(
        fixture
            .broker
            .approvals()
            .stable_summaries()
            .unwrap()
            .is_some_and(|(_, summaries)| summaries.iter().any(|summary| {
                summary.runtime_id == approval.runtime_id
                    && summary.correlation_id == approval.correlation_id
                    && summary.prompt_id == approval.prompt_id
            }))
    );

    assert!(
        answer_sampling_through_tauri_command_core(
            &fixture,
            generation,
            &approval,
            ApprovalAnswer::Allow,
        )
        .unwrap()
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("sampling completion timeout")
            .unwrap()
            .is_ok()
    );
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelling_a_pending_production_sampling_request_never_reaches_the_model() {
    let fixture = sampling_fixture();
    let cancellation = McpCancellation::default();
    let task_cancellation = cancellation.clone();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            task_cancellation,
        )
        .await
    });

    let _approval = wait_for_sampling_approval(&fixture, &mut task).await;
    cancellation.cancel();
    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("sampling cancellation timeout")
            .unwrap(),
        Err(McpError::Cancelled)
    ));
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 0);
    assert!(wait_for_no_sampling_approvals(&fixture).await);
    let records = fixture
        .database
        .security_records(SnapshotQuery::new(100).unwrap())
        .unwrap();
    assert!(records.iter().all(|record| {
        !(record.canonical_document.contains(PRIVATE_CANARY)
            || (record.action_id.starts_with("mcp-sampling-")
                && record.record_kind == "action-result"
                && record.state == "succeeded"))
    }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provider_drift_after_sampling_approval_is_published_fails_before_model_io() {
    let fixture = sampling_fixture();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            McpCancellation::default(),
        )
        .await
    });

    let approval = wait_for_sampling_approval(&fixture, &mut task).await;
    let now_ms = current_time_ms().unwrap();
    let snapshot = fixture.service.snapshot(now_ms).unwrap();
    let mut profile = snapshot
        .providers
        .providers
        .iter()
        .find(|provider| provider.profile.provider_id == "provider-openai")
        .unwrap()
        .profile
        .clone();
    profile.enabled = false;
    fixture
        .service
        .save_provider(
            snapshot.generation,
            profile,
            snapshot.providers.generation,
            now_ms,
        )
        .unwrap();
    let current_generation = fixture.service.snapshot(now_ms).unwrap().generation;
    assert!(
        fixture
            .broker
            .approvals()
            .answer(
                fixture.service.as_ref(),
                current_generation,
                &approval.runtime_id,
                &approval.correlation_id,
                &approval.prompt_id,
                ApprovalAnswer::Allow,
                now_ms.saturating_add(1),
            )
            .unwrap()
    );

    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .expect("sampling drift timeout")
            .unwrap(),
        Err(McpError::Conflict)
    ));
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 0);
    assert!(
        fixture
            .database
            .security_records(SnapshotQuery::new(100).unwrap())
            .unwrap()
            .iter()
            .all(|record| {
                !(record.canonical_document.contains(PRIVATE_CANARY)
                    || (record.action_id.starts_with("mcp-sampling-")
                        && record.record_kind == "action-result"
                        && record.state == "succeeded"))
            })
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pending_sampling_approval_times_out_and_releases_registry_capacity() {
    let mut fixture = sampling_fixture();
    fixture.context.timeout_ms = 25;

    assert!(matches!(
        broker_sampling_request(
            fixture.broker.as_ref(),
            &fixture.context,
            sampling_request(),
            McpCancellation::default(),
        )
        .await,
        Err(McpError::TimedOut)
    ));
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 0);
    assert!(wait_for_no_sampling_approvals(&fixture).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn in_flight_cancellation_returns_without_waiting_for_native_settlement() {
    let fixture = sampling_fixture();
    fixture.wait_for_cancellation.store(true, Ordering::SeqCst);
    fixture.post_cancel_delay_ms.store(300, Ordering::SeqCst);
    let cancellation = McpCancellation::default();
    let task_cancellation = cancellation.clone();
    let broker = Arc::clone(&fixture.broker);
    let context = fixture.context.clone();
    let mut task = tokio::spawn(async move {
        broker_sampling_request(
            broker.as_ref(),
            &context,
            sampling_request(),
            task_cancellation,
        )
        .await
    });
    let approval = wait_for_sampling_approval(&fixture, &mut task).await;
    let generation = fixture
        .service
        .snapshot(current_time_ms().unwrap())
        .unwrap()
        .generation;
    fixture
        .broker
        .approvals()
        .answer(
            fixture.service.as_ref(),
            generation,
            &approval.runtime_id,
            &approval.correlation_id,
            &approval.prompt_id,
            ApprovalAnswer::Allow,
            current_time_ms().unwrap(),
        )
        .unwrap();
    for _ in 0..200 {
        if fixture.sampling_calls.load(Ordering::SeqCst) == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert_eq!(fixture.sampling_calls.load(Ordering::SeqCst), 1);

    cancellation.cancel();
    assert!(matches!(
        tokio::time::timeout(Duration::from_millis(100), task)
            .await
            .expect("broker cancellation must not await native settlement")
            .unwrap(),
        Err(McpError::Cancelled)
    ));

    tokio::time::sleep(Duration::from_millis(500)).await;
    let records = fixture
        .database
        .security_records(SnapshotQuery::new(100).unwrap())
        .unwrap();
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record.action_id.starts_with("mcp-sampling-")
                    && record.record_kind == "action-result"
                    && record.state == "cancelled"
            })
            .count(),
        1
    );
}

async fn wait_for_sampling_approval(
    fixture: &SamplingFixture,
    task: &mut tokio::task::JoinHandle<Result<CreateMessageResult, McpError>>,
) -> super::mcp::production_sampling::McpSamplingApprovalSummary {
    for _ in 0..200 {
        if let Some((_, summaries)) = fixture.broker.approvals().stable_summaries().unwrap()
            && let Some(summary) = summaries.into_iter().next()
        {
            return summary;
        }
        if task.is_finished() {
            let outcome = task.await.expect("sampling task join");
            panic!("production sampling ended before approval publication: {outcome:?}");
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("production sampling approval was not published")
}

async fn wait_for_no_sampling_approvals(fixture: &SamplingFixture) -> bool {
    for _ in 0..200 {
        if fixture
            .broker
            .approvals()
            .stable_summaries()
            .unwrap()
            .is_some_and(|(_, summaries)| summaries.is_empty())
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    false
}

fn sampling_fixture() -> SamplingFixture {
    let temporary = TempDir::new().unwrap();
    let now_ms = current_time_ms().unwrap();
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path().join("c4os-home"))).unwrap();
    let database = Arc::new(database);
    let service =
        Arc::new(RuntimeApplicationService::restore(Arc::clone(&database), now_ms).unwrap());

    let workspace_root = temporary.path().join("workspace");
    let (workspace_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        &workspace_root,
        "workspace-1",
    ))
    .unwrap();
    let workspace_database = Arc::new(workspace_database);
    seed_workspace(&workspace_database, &workspace_root, now_ms);
    service.bind_workspace(workspace_database).unwrap();

    let configuration = pi_configuration();
    let route = pi_route(authoritative_configuration_sha256(&configuration).unwrap());
    let native_model = PiModelRoute {
        provider: "openai".into(),
        model_id: "gpt-4o-mini".into(),
        base_url: "https://api.openai.com/v1".into(),
    };
    let adapter = pi_adapter_evidence(&native_model, &route, now_ms, now_ms + 120_000).unwrap();
    let model = ModelRoute {
        model_id: "gpt-4o-mini".into(),
        display_name: "GPT-4o mini".into(),
        recommendation_rank: 0,
        availability: RouteAvailability::Available,
        checked_at_ms: now_ms + 2,
        capabilities: adapter.descriptor().clone(),
        provider_declaration: Some(ProviderModelDeclaration {
            schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
            provider_model_id: "gpt-4o-mini".into(),
            model_revision: route.model_revision.clone(),
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::new(),
            numeric_limits: BTreeMap::new(),
            raw_catalog_sha256: digest('b'),
            declared_at_ms: now_ms + 2,
            expires_at_ms: now_ms + 120_000,
        }),
    };
    let generation = service.snapshot(now_ms).unwrap().generation;
    service
        .save_provider(generation, provider_profile(), 0, now_ms + 1)
        .unwrap();
    let generation = service.snapshot(now_ms + 1).unwrap().generation;
    service
        .test_provider(
            generation,
            "provider-openai",
            1,
            now_ms + 2,
            &mut FixtureProbe(ProviderDiscovery {
                checked_at_ms: now_ms + 2,
                models: vec![model],
                recommended_model_id: Some("gpt-4o-mini".into()),
                connection_evidence: None,
            }),
        )
        .unwrap();

    let generation = service.snapshot(now_ms + 2).unwrap().generation;
    service
        .register_runtime(generation, pi_installation(temporary.path()), now_ms + 3)
        .unwrap();
    let generation = service.snapshot(now_ms + 3).unwrap().generation;
    let reserved = service
        .reserve_managed_runtime_start(generation, "pi-primary", now_ms + 4)
        .unwrap();
    let descriptor = pi_descriptor(reserved.value);
    let sampling_calls = Arc::new(AtomicUsize::new(0));
    let sampled = Arc::new(Mutex::new(Vec::new()));
    let wait_for_cancellation = Arc::new(AtomicBool::new(false));
    let post_cancel_delay_ms = Arc::new(AtomicU64::new(0));
    service
        .register_dispatch_peer(SamplingPeer {
            registration: RuntimePeerRegistration {
                runtime_id: "pi-primary".into(),
                workspace_id: "workspace-1".into(),
                descriptor: descriptor.clone(),
            },
            sampling_calls: Arc::clone(&sampling_calls),
            sampled: Arc::clone(&sampled),
            wait_for_cancellation: Arc::clone(&wait_for_cancellation),
            post_cancel_delay_ms: Arc::clone(&post_cancel_delay_ms),
        })
        .unwrap();
    let declared = provider_declared_evidence(&provider_claim(route.clone(), now_ms)).unwrap();
    let observed = pi_observed_evidence(
        &descriptor,
        &pi_health(reserved.value),
        &runtime_observation(route.clone(), reserved.value, now_ms + 5),
    )
    .unwrap();
    let generation = service.snapshot(now_ms + 5).unwrap().generation;
    service
        .attach_managed_runtime_process_with_capabilities(
            generation,
            service.capability_evidence_generation().unwrap(),
            "pi-primary",
            reserved.value,
            42_424,
            HealthState::Healthy,
            vec![
                CapabilityRouteEpoch::new(
                    "pi-primary",
                    reserved.value,
                    declared,
                    adapter,
                    observed,
                )
                .unwrap(),
            ],
            now_ms + 6,
        )
        .unwrap();

    let generation = service.snapshot(now_ms + 6).unwrap().generation;
    service
        .create_provisional(generation, "session-1", now_ms + 7)
        .unwrap();
    let resources = production_broker_authoritative_resources();
    let resources_sha256 = authoritative_resources_sha256(&resources).unwrap();
    let generation = service.snapshot(now_ms + 7).unwrap().generation;
    let dispatch = service
        .dispatch_first(
            generation,
            FirstRuntimeDispatchIntent {
                authority: AuthorityMintIntent {
                    workspace_id: "workspace-1".into(),
                    project_id: Some("project-1".into()),
                    runtime_id: "pi-primary".into(),
                    expected_authority_generation: service.dispatch_authority_generation().unwrap(),
                    expected_capability_generation: service
                        .capability_evidence_generation()
                        .unwrap(),
                },
                provider_id: "provider-openai".into(),
                model_id: "gpt-4o-mini".into(),
                session_id: "session-1".into(),
                turn_id: "turn-1".into(),
                attempt_id: "attempt-1".into(),
                authorization_scope_id: "authority-1".into(),
                correlation_id: "correlation-1".into(),
                prompt: Some("Open the production sampling fixture".into()),
                attachments: Vec::new(),
                skill_context: Vec::new(),
                mcp_turn: None,
                draft: DraftRequirements {
                    attachments: Vec::new(),
                    reasoning_mode: None,
                    requires_tools: false,
                    requires_json_schema: false,
                    prefers_streaming: true,
                    estimated_input_tokens: 100,
                    requested_output_tokens: 256,
                    installed_resources: InstalledResourcePreflight {
                        snapshot_id: format!(
                            "resources:{}",
                            resources_sha256.strip_prefix("sha256:").unwrap()
                        ),
                        snapshot_sha256: resources_sha256,
                        tool_ids: resources.records.keys().cloned().collect(),
                        attachment_converters: BTreeSet::new(),
                    },
                    policy: PolicyPreflight {
                        snapshot_id: "policy-1".into(),
                        version: 1,
                        tool_use_allowed: false,
                        attachment_conversion_allowed: false,
                    },
                },
                submitted_at_ms: now_ms + 8,
                preflight_at_ms: now_ms + 8,
            },
            FirstDispatchOptions {
                title: "Sampling parent".into(),
                credential_reference: None,
                credential_lease_id: None,
                attachment_resolution: AttachmentPreflightResolution::NotRequired,
                broker_authority: None,
            },
        )
        .unwrap();
    assert!(matches!(
        dispatch,
        super::runtime::dispatch::CoordinatedFirstDispatch::Accepted { .. }
    ));
    let definition_sha256 = digest('d');
    let parent_action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: "parent-mcp-action".into(),
        tool_call_id: "parent-mcp-tool-call".into(),
        tool: "c4os_propose_action".into(),
        arguments: serde_json::json!({
            "resolved": {
                "serverId": SERVER_ID,
                "definitionSha256": definition_sha256,
                "lifecycleGeneration": 7,
            }
        }),
        risk: CanonicalRisk::High,
        requested_authority: BTreeSet::from(["network.execute".into()]),
        canonical_target: "mcp:docs:tool:lookup".into(),
        target_version: digest('e'),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        run_id: "attempt-1".into(),
        runtime_id: "pi-primary".into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: Some(AUTHORITY_ID.into()),
        process_generation: reserved.value,
        configuration_version: 1,
        policy_version: 1,
        revocation_epoch: 1,
    };
    let broker = Arc::new(ProductionMcpSamplingBroker::new(Arc::clone(&service)));
    let parent = broker.parents().register(SERVER_ID, parent_action).unwrap();
    let context = McpSamplingContext {
        server_id: SERVER_ID.into(),
        authority_id: AUTHORITY_ID.into(),
        lifecycle_generation: 7,
        definition_sha256,
        timeout_ms: 2_000,
        max_output_bytes: 64 * 1_024,
    };

    SamplingFixture {
        _temporary: temporary,
        database,
        service,
        broker,
        _parent: parent,
        context,
        sampling_calls,
        sampled,
        wait_for_cancellation,
        post_cancel_delay_ms,
    }
}

fn sampling_request() -> CreateMessageRequestParams {
    CreateMessageRequestParams::new(vec![SamplingMessage::user_text(PRIVATE_CANARY)], 256)
        .with_system_prompt("Bounded private system context")
}

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn provider_profile() -> ProviderProfile {
    let vault = CredentialVault::session_only().unwrap();
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
        credential_reference: Some(vault.store("provider-key", b"fixture-secret").unwrap()),
        headers: BTreeMap::new(),
        enabled: true,
    }
}

fn pi_configuration() -> AuthoritativeConfiguration {
    AuthoritativeConfiguration {
        version: 1,
        fields: BTreeMap::from([
            (
                "provider-id".into(),
                ConfigurationFieldValue::Text("provider-openai".into()),
            ),
            (
                "endpoint-id".into(),
                ConfigurationFieldValue::Text("openai-api".into()),
            ),
            (
                "model-id".into(),
                ConfigurationFieldValue::Text("openai/gpt-4o-mini".into()),
            ),
            (
                "model-revision".into(),
                ConfigurationFieldValue::Text("pi-catalog-0.80.10".into()),
            ),
            (
                "adapter-kind".into(),
                ConfigurationFieldValue::Text("pi".into()),
            ),
            (
                "adapter-version".into(),
                ConfigurationFieldValue::Text("1.0.0".into()),
            ),
            (
                "runtime-kind".into(),
                ConfigurationFieldValue::Text("pi".into()),
            ),
            (
                "native-runtime-version".into(),
                ConfigurationFieldValue::Text(PI_NATIVE_VERSION.into()),
            ),
        ]),
    }
}

fn pi_route(configuration_sha256: String) -> RouteIdentity {
    RouteIdentity {
        provider_id: "provider-openai".into(),
        endpoint_id: "openai-api".into(),
        provider_model_id: "openai/gpt-4o-mini".into(),
        model_revision: "pi-catalog-0.80.10".into(),
        adapter_kind: "pi".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "pi".into(),
        native_runtime_version: PI_NATIVE_VERSION.into(),
        session_configuration_sha256: configuration_sha256,
    }
}

fn feature(state: CapabilityState) -> FeatureClaim {
    FeatureClaim {
        state,
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: None,
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

fn provider_claim(route: RouteIdentity, now_ms: u64) -> ProviderDeclaredCatalogClaim {
    ProviderDeclaredCatalogClaim {
        route,
        lifecycle: ModelLifecycle::Active,
        declared_at_ms: now_ms,
        expires_at_ms: now_ms + 120_000,
        features: BTreeMap::from([
            (
                CapabilityKey::InputText,
                feature(CapabilityState::Supported),
            ),
            (
                CapabilityKey::OutputText,
                feature(CapabilityState::Supported),
            ),
            (
                CapabilityKey::Streaming,
                feature(CapabilityState::Supported),
            ),
            (
                CapabilityKey::ToolCalling,
                feature(CapabilityState::Supported),
            ),
        ]),
        numeric_limits: BTreeMap::from([
            (NumericCapabilityKey::ContextTokens, numeric(128_000)),
            (NumericCapabilityKey::OutputTokens, numeric(16_384)),
        ]),
        raw_catalog_sha256: digest('b'),
    }
}

fn runtime_observation(
    route: RouteIdentity,
    process_generation: u64,
    checked_at_ms: u64,
) -> RuntimeRouteObservation {
    RuntimeRouteObservation {
        runtime_id: "pi-primary".into(),
        route,
        process_generation,
        health_checked_at_ms: checked_at_ms,
        observed_at_ms: checked_at_ms,
        expires_at_ms: checked_at_ms + 120_000,
        outcome: RuntimeObservationOutcome::Available,
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::from([
            (
                CapabilityKey::InputText,
                feature(CapabilityState::Supported),
            ),
            (
                CapabilityKey::OutputText,
                feature(CapabilityState::Supported),
            ),
            (
                CapabilityKey::Streaming,
                feature(CapabilityState::Supported),
            ),
            (
                CapabilityKey::ToolCalling,
                feature(CapabilityState::Supported),
            ),
        ]),
        numeric_limits: BTreeMap::from([
            (NumericCapabilityKey::ContextTokens, numeric(128_000)),
            (NumericCapabilityKey::OutputTokens, numeric(16_384)),
        ]),
        raw_observation_sha256: digest('c'),
    }
}

fn pi_descriptor(process_generation: u64) -> AdapterConformanceDescriptor {
    AdapterConformanceDescriptor {
        schema_version: ADAPTER_CONTRACT_SCHEMA_VERSION,
        runtime_kind: RuntimeKind::Pi,
        adapter_version: "1.0.0".into(),
        native_version: PI_NATIVE_VERSION.into(),
        protocol_version: RUNTIME_PROTOCOL_VERSION,
        process_generation,
        authority: AdapterAuthority::C4osActionGatewayOnly,
        capabilities: peer_capabilities(PeerCapabilityClaims {
            health: CapabilityState::Supported,
            session_create: CapabilityState::Supported,
            session_resume: CapabilityState::Degraded,
            model_discovery: CapabilityState::Unsupported,
            streaming: CapabilityState::Supported,
            action_intents: CapabilityState::Supported,
            credential_channel: CapabilityState::Supported,
            cancellation: CapabilityState::Supported,
            restart: CapabilityState::Degraded,
        }),
    }
}

fn pi_health(process_generation: u64) -> PiHealth {
    let state = |state: &str| PiCapabilityState {
        state: state.into(),
        reason: None,
    };
    PiHealth {
        status: "ready".into(),
        runtime: "pi".into(),
        transport: "c4os-node-sdk-sidecar".into(),
        protocol: PI_PROTOCOL.into(),
        process_generation,
        sessions: 1,
        stale_events_rejected: 0,
        capabilities: BTreeMap::from([
            ("streaming".into(), state("supported")),
            ("cancellation".into(), state("supported")),
            ("tools".into(), state("supported")),
            ("nativePersistence".into(), state("unsupported")),
            ("nativeExtensions".into(), state("unsupported")),
            ("nativeTools".into(), state("unsupported")),
            ("crashResume".into(), state("degraded")),
            ("providerAuthentication".into(), state("supported")),
            ("rpcTransport".into(), state("supported")),
        ]),
    }
}

fn pi_installation(root: &Path) -> RuntimeInstallation {
    let install_root = root.join("runtime-pi");
    let executable = install_root.join("bin/pi");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let executable_sha256 = sha256_file(&executable).unwrap();
    RuntimeInstallation {
        runtime_id: "pi-primary".into(),
        workspace_id: "workspace-1".into(),
        runtime_kind: RuntimeKind::Pi,
        native_version: PI_NATIVE_VERSION.into(),
        adapter_version: "1.0.0".into(),
        protocol_version: RUNTIME_PROTOCOL_VERSION,
        install_root: install_root.clone(),
        asset_tree_sha256: executable_sha256.clone(),
        executable,
        executable_sha256,
        state_namespace: install_root.join("state"),
        arguments: Vec::new(),
        sanitized_environment: BTreeMap::new(),
    }
}

fn seed_workspace(database: &DatabaseActor, root: &Path, now_ms: u64) {
    let now_ms = i64::try_from(now_ms).unwrap();
    database
        .create_workspace(WorkspaceRecord {
            workspace_id: "workspace-1".into(),
            display_name: "Sampling Workspace".into(),
            created_at: now_ms,
            updated_at: now_ms,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_project(ProjectRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            display_name: "Sampling Project".into(),
            current_path: root.display().to_string(),
            last_known_path: root.display().to_string(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_chat(ChatRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            chat_id: "session-1".into(),
            title: "Sampling Chat".into(),
            created_at: now_ms,
            updated_at: now_ms,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
}
