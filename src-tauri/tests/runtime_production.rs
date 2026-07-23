#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use c4os_lib::core::database::{
    ChatRecord, DatabaseActor, DatabaseDescriptor, LifecycleState, ProjectPathState, ProjectRecord,
    SnapshotQuery, WorkspaceRecord,
};
use c4os_lib::runtime::capability::{
    AttachmentMediaType, AttachmentRequirement, CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor,
    CapabilityEvidence, CapabilityKey, CapabilityLayer, CapabilityState, DraftRequirements,
    InstalledResourcePreflight, LimitConfidence, ModelLifecycle, NumericCapabilityEvidence,
    NumericCapabilityKey, PolicyPreflight, RouteIdentity,
};
use c4os_lib::runtime::capability_evidence::CapabilityRouteEpoch;
use c4os_lib::runtime::coordinator::CoordinatorError;
use c4os_lib::runtime::dispatch::{
    AttachmentPreflightResolution, CoordinatedFirstDispatch, FirstDispatchOptions,
    RuntimeDispatchRegistry,
};
use c4os_lib::runtime::dispatch_authority::{
    AuthorityMintIntent, authoritative_configuration_sha256, authoritative_resources_sha256,
    authoritative_route_configuration,
};
use c4os_lib::runtime::opencode_native::{OPENCODE_C4OS_TOOL_IDS, sha256_bytes};
use c4os_lib::runtime::persistence::SqliteSessionRepository;
use c4os_lib::runtime::production::{
    ProductionProviderRoute, ProductionRuntimeBinding, RuntimeProductionBootstrap,
};
use c4os_lib::runtime::production_application::{
    PreparedProductionRuntimePeer, ProductionRuntimeBackend, ProductionRuntimeWorker,
    RuntimeProductionApplication, RuntimeProductionApplicationError,
};
use c4os_lib::runtime::supervisor::{RuntimeKind, RuntimeLifecycle};
use c4os_lib::runtime::{
    opencode_broker::{InstalledBrokerClassification, InstalledBrokerFacility},
    provider::{
        ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, PROVIDER_SCHEMA_VERSION,
        ProviderAuthentication, ProviderConnectionEvidence, ProviderDiscovery, ProviderEndpoint,
        ProviderFeatureClaim, ProviderKind, ProviderModelDeclaration, ProviderNumericClaim,
        ProviderProbe, ProviderProbeFailure, ProviderProfile, RouteAvailability,
    },
    session::{AttachmentSnapshot, RunAttemptStatus, RunEventKind, SessionRepository},
};
use c4os_lib::security::authorization::ApprovalAnswer;
use c4os_lib::security::credentials::CredentialVault;
use c4os_lib::security::gateway::{
    ExecutionPermit, NormalizedActionResult, NormalizedActionStatus,
};
use c4os_lib::security::policy::{
    ActionEffect, ActionReversibility, ActionScope, ActionSensitivity, ActionSurface,
    RepositoryState,
};
use c4os_lib::{
    FirstRuntimeDispatchIntent, RuntimeApplicationError, RuntimeApplicationService,
    production_broker_authoritative_resources,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use tempfile::TempDir;

const NOW: u64 = 1_721_300_000_000;
type ObservedProcessGroup = Arc<Mutex<Option<(u32, Vec<u32>)>>>;

fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_millis()
        .try_into()
        .expect("current epoch fits u64 milliseconds")
}

struct AttachInvalidatingBootstrap {
    inner: RuntimeProductionBootstrap,
    application: Arc<RuntimeApplicationService>,
    observed_process_group: ObservedProcessGroup,
}

impl ProductionRuntimeBackend for AttachInvalidatingBootstrap {
    fn prepare(
        &self,
        binding: ProductionRuntimeBinding,
        provider_routes: &[ProductionProviderRoute],
        checked_at_ms: u64,
    ) -> Result<Box<dyn PreparedProductionRuntimePeer>, RuntimeProductionApplicationError> {
        let prepared = self
            .inner
            .prepare(binding, provider_routes, checked_at_ms)?;
        let process_group_id = prepared.native_process_id()?;
        let members = observe_process_group(process_group_id);
        *self
            .observed_process_group
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)? =
            Some((process_group_id, members));
        Ok(Box::new(AttachInvalidatingPrepared {
            inner: prepared,
            application: Arc::clone(&self.application),
            checked_at_ms,
        }))
    }
}

struct AttachInvalidatingPrepared {
    inner: Box<dyn PreparedProductionRuntimePeer>,
    application: Arc<RuntimeApplicationService>,
    checked_at_ms: u64,
}

impl PreparedProductionRuntimePeer for AttachInvalidatingPrepared {
    fn binding(&self) -> &ProductionRuntimeBinding {
        self.inner.binding()
    }

    fn native_process_id(&self) -> Result<u32, RuntimeProductionApplicationError> {
        self.inner.native_process_id()
    }

    fn capability_epochs(
        &self,
    ) -> Result<Vec<CapabilityRouteEpoch>, RuntimeProductionApplicationError> {
        self.inner.capability_epochs()
    }

    fn install(
        self: Box<Self>,
        registry: &mut RuntimeDispatchRegistry,
    ) -> Result<Box<dyn ProductionRuntimeWorker>, RuntimeProductionApplicationError> {
        let worker = self.inner.install(registry)?;
        let generation = self.application.snapshot(self.checked_at_ms)?.generation;
        self.application.publish_runtime_policy_authority(
            generation,
            1,
            2,
            0,
            self.checked_at_ms.saturating_add(1),
        )?;
        Ok(worker)
    }
}

fn process_group_member_ids(process_group_id: u32) -> Vec<u32> {
    let listing = Command::new("/bin/ps")
        .args(["-ax", "-o", "pid=,pgid="])
        .output()
        .expect("list runtime process group");
    assert!(listing.status.success());
    String::from_utf8_lossy(&listing.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let process_id = fields.next()?.parse::<u32>().ok()?;
            let group_id = fields.next()?.parse::<u32>().ok()?;
            (group_id == process_group_id).then_some(process_id)
        })
        .collect()
}

fn observe_process_group(process_group_id: u32) -> Vec<u32> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let members = process_group_member_ids(process_group_id);
        if members.len() >= 2 || Instant::now() >= deadline {
            return members;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn process_group_exists(process_group_id: u32) -> bool {
    let Ok(process_group_id) = i32::try_from(process_group_id) else {
        return false;
    };
    if unsafe { libc::kill(-process_group_id, 0) } == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

fn text_dispatch_intent(
    workspace_id: &str,
    runtime_id: &str,
    submitted_at_ms: u64,
) -> FirstRuntimeDispatchIntent {
    FirstRuntimeDispatchIntent {
        authority: AuthorityMintIntent {
            workspace_id: workspace_id.into(),
            project_id: Some("project-1".into()),
            runtime_id: runtime_id.into(),
            expected_authority_generation: 0,
            expected_capability_generation: 0,
        },
        provider_id: "provider-missing".into(),
        model_id: "model-missing".into(),
        session_id: "session-zero-provider".into(),
        turn_id: "turn-zero-provider".into(),
        attempt_id: "attempt-zero-provider".into(),
        authorization_scope_id: "authority-zero-provider".into(),
        correlation_id: "correlation-zero-provider".into(),
        prompt: Some("This dispatch must remain inside C4OS.".into()),
        attachments: Vec::new(),
        skill_context: Vec::new(),
        mcp_turn: None,
        draft: DraftRequirements {
            attachments: Vec::new(),
            reasoning_mode: None,
            requires_tools: false,
            requires_json_schema: false,
            prefers_streaming: true,
            estimated_input_tokens: 16,
            requested_output_tokens: 16,
            installed_resources: InstalledResourcePreflight {
                snapshot_id: format!("resources:{}", "a".repeat(64)),
                snapshot_sha256: format!("sha256:{}", "a".repeat(64)),
                tool_ids: BTreeSet::new(),
                attachment_converters: BTreeSet::new(),
            },
            policy: PolicyPreflight {
                snapshot_id: "policy-1".into(),
                version: 1,
                tool_use_allowed: false,
                attachment_conversion_allowed: false,
            },
        },
        submitted_at_ms,
        preflight_at_ms: submitted_at_ms,
    }
}

fn text_dispatch_options() -> FirstDispatchOptions {
    FirstDispatchOptions {
        title: "Task 00004 Native Golden Path".into(),
        credential_reference: None,
        credential_lease_id: None,
        attachment_resolution: AttachmentPreflightResolution::NotRequired,
        broker_authority: None,
    }
}

struct GoldenProviderProbe {
    discovery: ProviderDiscovery,
}

impl ProviderProbe for GoldenProviderProbe {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let mut discovery = self.discovery.clone();
        discovery.connection_evidence = Some(ProviderConnectionEvidence::from_tested_profile(
            profile,
            discovery.checked_at_ms,
            sha256_bytes(b"task-00004-native-provider-fixture"),
        )?);
        Ok(discovery)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeProviderFixtureReady {
    schema_version: u16,
    base_url: String,
    process_id: u32,
}

struct NativeProviderFixture {
    child: Child,
    base_url: String,
    evidence_path: PathBuf,
    process_group_id: u32,
    stopped: bool,
}

impl NativeProviderFixture {
    fn start(node: &Path, certificate: &Path, private_key: &Path, evidence_path: PathBuf) -> Self {
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tools/task-00004-native-provider-fixture.mjs");
        let mut child = Command::new(node)
            .arg(&script)
            .arg(format!("--cert={}", certificate.display()))
            .arg(format!("--key={}", private_key.display()))
            .arg(format!("--evidence={}", evidence_path.display()))
            .env_clear()
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start trusted HTTPS provider fixture");
        let stdout = child
            .stdout
            .take()
            .expect("provider fixture readiness pipe");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("read provider fixture readiness");
        let ready: NativeProviderFixtureReady =
            serde_json::from_str(line.trim()).expect("provider fixture readiness JSON");
        assert_eq!(ready.schema_version, 1);
        assert_eq!(ready.process_id, child.id());
        assert!(ready.base_url.starts_with("https://127.0.0.1:"));
        assert!(ready.base_url.ends_with("/v1"));
        let process_group_id = child.id();
        Self {
            child,
            base_url: ready.base_url,
            evidence_path,
            process_group_id,
            stopped: false,
        }
    }

    fn stop(mut self) {
        terminate_process_group(&mut self.child, self.process_group_id);
        self.stopped = true;
        assert!(!process_group_exists(self.process_group_id));
    }
}

impl Drop for NativeProviderFixture {
    fn drop(&mut self) {
        if !self.stopped {
            terminate_process_group(&mut self.child, self.process_group_id);
        }
    }
}

struct CountingFacility {
    calls: Arc<AtomicUsize>,
}

impl InstalledBrokerFacility for CountingFacility {
    fn current_target_version(&mut self) -> Option<String> {
        Some("v1".into())
    }

    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "installed-facility-completed".into(),
            exit_code: None,
            changed_targets: vec![permit.action().canonical_target.clone()],
            output_sha256: None,
            completed_at_ms: current_epoch_ms(),
        }
    }
}

fn golden_classification(target: &str) -> InstalledBrokerClassification {
    InstalledBrokerClassification {
        surface: ActionSurface::Desktop,
        effects: BTreeSet::from([ActionEffect::Control]),
        scope: ActionScope::Workspace,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: true,
        canonical_target: format!("desktop:{target}"),
        normalized_arguments: Map::new(),
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    }
}

fn golden_model_route(profile: &ProviderProfile, checked_at_ms: u64) -> ModelRoute {
    let expires_at_ms = checked_at_ms + 5 * 60_000;
    let supported = || CapabilityEvidence {
        state: CapabilityState::Supported,
        layer: CapabilityLayer::AdapterNormalized,
        source: "task-00004.native-provider-fixture".into(),
        checked_at_ms,
        expires_at_ms: Some(expires_at_ms),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: None,
    };
    let mut route = RouteIdentity {
        provider_id: profile.provider_id.clone(),
        endpoint_id: profile.endpoint.endpoint_id.clone(),
        provider_model_id: format!("{}/gpt-4o-mini", profile.opencode_native_provider_id()),
        model_revision: "pi-catalog-0.80.10".into(),
        adapter_kind: "opencode".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "opencode".into(),
        native_runtime_version: "1.18.3".into(),
        session_configuration_sha256: format!("sha256:{}", "0".repeat(64)),
    };
    route.session_configuration_sha256 = authoritative_configuration_sha256(
        &authoritative_route_configuration(&route).expect("golden route configuration"),
    )
    .expect("golden configuration digest");
    let features = BTreeMap::from([
        (CapabilityKey::InputText, supported()),
        (CapabilityKey::InputImage, supported()),
        (CapabilityKey::OutputText, supported()),
        (CapabilityKey::Streaming, supported()),
        (CapabilityKey::ToolCalling, supported()),
    ]);
    let numeric_limits = BTreeMap::from([
        (
            NumericCapabilityKey::ContextTokens,
            NumericCapabilityEvidence {
                evidence: supported(),
                maximum: Some(128_000),
                confidence: LimitConfidence::Confirmed,
            },
        ),
        (
            NumericCapabilityKey::OutputTokens,
            NumericCapabilityEvidence {
                evidence: supported(),
                maximum: Some(16_384),
                confidence: LimitConfidence::Confirmed,
            },
        ),
    ]);
    let provider_features = features
        .iter()
        .map(|(key, evidence)| {
            (
                *key,
                ProviderFeatureClaim {
                    state: evidence.state,
                    constraints: evidence.constraints.clone(),
                    allowed_values: evidence.allowed_values.clone(),
                    reason: evidence.reason.clone(),
                },
            )
        })
        .collect();
    let provider_numeric = numeric_limits
        .iter()
        .map(|(key, numeric)| {
            (
                *key,
                ProviderNumericClaim {
                    state: numeric.evidence.state,
                    maximum: numeric.maximum,
                    confidence: numeric.confidence,
                    reason: numeric.evidence.reason.clone(),
                },
            )
        })
        .collect();
    ModelRoute {
        model_id: "gpt-4o-mini".into(),
        display_name: "GPT-4o mini native fixture".into(),
        recommendation_rank: 0,
        availability: RouteAvailability::Available,
        checked_at_ms,
        capabilities: CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::AdapterNormalized,
            route,
            lifecycle: ModelLifecycle::Active,
            features,
            numeric_limits,
            raw_evidence_sha256: sha256_bytes(b"task-00004-native-model-route"),
        },
        provider_declaration: Some(ProviderModelDeclaration {
            schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
            provider_model_id: "gpt-4o-mini".into(),
            model_revision: "pi-catalog-0.80.10".into(),
            lifecycle: ModelLifecycle::Active,
            features: provider_features,
            numeric_limits: provider_numeric,
            raw_catalog_sha256: sha256_bytes(b"pi-0.80.10-openai-gpt-4o-mini"),
            declared_at_ms: checked_at_ms,
            expires_at_ms,
        }),
    }
}

fn golden_installed_resources() -> InstalledResourcePreflight {
    let resources = production_broker_authoritative_resources();
    let snapshot_sha256 =
        authoritative_resources_sha256(&resources).expect("authoritative broker resources");
    InstalledResourcePreflight {
        snapshot_id: format!(
            "resources:{}",
            snapshot_sha256
                .strip_prefix("sha256:")
                .expect("resource digest prefix")
        ),
        snapshot_sha256,
        tool_ids: OPENCODE_C4OS_TOOL_IDS
            .into_iter()
            .map(str::to_owned)
            .collect(),
        attachment_converters: BTreeSet::new(),
    }
}

fn golden_dispatch_intent(
    runtime_id: &str,
    session_id: &str,
    flow_marker: &str,
    authority_generation: u64,
    capability_generation: u64,
    attachments: Vec<AttachmentSnapshot>,
    submitted_at_ms: u64,
) -> FirstRuntimeDispatchIntent {
    let attachment_requirements = attachments
        .iter()
        .map(|attachment| AttachmentRequirement {
            attachment_id: attachment.attachment_id.clone(),
            media_type: AttachmentMediaType::Image,
            mime_type: attachment.media_type.clone(),
            bytes: attachment.byte_length,
        })
        .collect();
    FirstRuntimeDispatchIntent {
        authority: AuthorityMintIntent {
            workspace_id: "workspace-native".into(),
            project_id: Some("project-native".into()),
            runtime_id: runtime_id.into(),
            expected_authority_generation: authority_generation,
            expected_capability_generation: capability_generation,
        },
        provider_id: "native-golden-provider".into(),
        model_id: "gpt-4o-mini".into(),
        session_id: session_id.into(),
        turn_id: format!("turn-{session_id}"),
        attempt_id: format!("attempt-{session_id}"),
        authorization_scope_id: format!("authority-{session_id}"),
        correlation_id: format!("correlation-{session_id}"),
        prompt: Some(format!(
            "Complete the C4OS native approval golden path for {runtime_id}. C4OS-NATIVE-FLOW: {flow_marker}"
        )),
        attachments,
        skill_context: Vec::new(),
        mcp_turn: None,
        draft: DraftRequirements {
            attachments: attachment_requirements,
            reasoning_mode: None,
            requires_tools: true,
            requires_json_schema: false,
            prefers_streaming: true,
            estimated_input_tokens: 128,
            requested_output_tokens: 128,
            installed_resources: golden_installed_resources(),
            policy: PolicyPreflight {
                snapshot_id: "policy-native-golden".into(),
                version: 1,
                tool_use_allowed: true,
                attachment_conversion_allowed: false,
            },
        },
        submitted_at_ms,
        preflight_at_ms: submitted_at_ms,
    }
}

fn pump_until_approval(
    host: &RuntimeProductionApplication<RuntimeProductionBootstrap>,
    repository: &SqliteSessionRepository,
    runtime_id: &str,
    session_id: &str,
    now_ms: &mut u64,
) -> (String, String) {
    for _ in 0..600 {
        *now_ms += 1;
        host.pump_runtime_once(runtime_id, *now_ms)
            .expect("pump exact production runtime to approval");
        if let Some(approval) = host
            .pending_approvals()
            .expect("inspect production approvals")
            .into_iter()
            .find(|approval| approval.runtime_id == runtime_id)
        {
            return (approval.correlation_id, approval.prompt_id);
        }
        if let Some(attempt) = repository
            .load(session_id)
            .expect("load native approval attempt")
            .and_then(|record| record.attempts.last().cloned())
            && let RunAttemptStatus::Failed { error_code, .. } = attempt.status
        {
            let events = attempt
                .events
                .iter()
                .map(|event| format!("{:?}:{}", event.kind, event.payload))
                .collect::<Vec<_>>();
            panic!("{runtime_id} failed before native approval: {error_code}; events={events:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let attempts = native_attempt_diagnostics(repository, session_id);
    panic!("{runtime_id} did not publish a native approval; attempts={attempts:?}")
}

fn pump_until_completed(
    host: &RuntimeProductionApplication<RuntimeProductionBootstrap>,
    repository: &SqliteSessionRepository,
    runtime_id: &str,
    session_id: &str,
    now_ms: &mut u64,
) -> c4os_lib::runtime::session::SessionRecord {
    for _ in 0..600 {
        *now_ms += 1;
        host.pump_runtime_once(runtime_id, *now_ms)
            .expect("pump exact production runtime to completion");
        if let Some(record) = repository
            .load(session_id)
            .expect("load durable native session")
            && record
                .attempts
                .iter()
                .any(|attempt| matches!(attempt.status, RunAttemptStatus::Completed { .. }))
        {
            return record;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let attempts = native_attempt_diagnostics(repository, session_id);
    panic!("{runtime_id} did not reach durable completion; attempts={attempts:?}")
}

fn native_attempt_diagnostics(
    repository: &SqliteSessionRepository,
    session_id: &str,
) -> Vec<String> {
    repository
        .load(session_id)
        .expect("load incomplete native session")
        .map(|record| {
            record
                .attempts
                .into_iter()
                .map(|attempt| {
                    let events = attempt
                        .events
                        .into_iter()
                        .map(|event| format!("{:?}:{}", event.kind, event.payload))
                        .collect::<Vec<_>>();
                    format!("{:?}; events={events:?}", attempt.status)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn assert_golden_session(record: &c4os_lib::runtime::session::SessionRecord, expected: &str) {
    assert!(record.active_attempt_id.is_none());
    let attempt = record.attempts.last().expect("durable native attempt");
    assert!(matches!(attempt.status, RunAttemptStatus::Completed { .. }));
    assert!(attempt.events.iter().any(|event| {
        event.kind == RunEventKind::TextDelta && event.payload.contains(expected)
    }));
    assert!(
        attempt
            .events
            .iter()
            .any(|event| event.kind == RunEventKind::Completion)
    );
}

fn terminate_process_group(child: &mut Child, process_group_id: u32) {
    let process_group_id_i32 = i32::try_from(process_group_id).expect("fixture process group");
    unsafe {
        libc::kill(-process_group_id_i32, libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if child
            .try_wait()
            .expect("wait for provider fixture")
            .is_some()
        {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    unsafe {
        libc::kill(-process_group_id_i32, libc::SIGKILL);
    }
    child.wait().expect("reap provider fixture");
}

fn process_group_surfaces(process_group_id: u32) -> String {
    let mut surfaces = String::new();
    for process_id in process_group_member_ids(process_group_id) {
        let output = Command::new("/bin/ps")
            .args(["eww", "-p", &process_id.to_string()])
            .output()
            .expect("inspect runtime process surface");
        assert!(output.status.success());
        surfaces.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    assert!(!surfaces.is_empty());
    surfaces
}

fn tree_is_absent_of(root: &Path, needle: &[u8]) -> bool {
    if needle.is_empty() {
        return false;
    }
    let Ok(metadata) = fs::symlink_metadata(root) else {
        return false;
    };
    if metadata.file_type().is_symlink() {
        return true;
    }
    if metadata.is_dir() {
        let Ok(entries) = fs::read_dir(root) else {
            return false;
        };
        return entries
            .filter_map(Result::ok)
            .all(|entry| tree_is_absent_of(&entry.path(), needle));
    }
    if !metadata.is_file() {
        return true;
    }
    let Ok(bytes) = fs::read(root) else {
        return false;
    };
    !bytes.windows(needle.len()).any(|window| window == needle)
}

fn read_fixture_evidence(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read native provider evidence"))
        .expect("native provider evidence JSON")
}

#[test]
fn bootstrap_fails_closed_without_verified_packaged_graphs() {
    let temporary = TempDir::new().expect("temporary resource root");

    assert!(RuntimeProductionBootstrap::new(temporary.path(), None).is_err());
}

#[test]
fn production_resource_authority_publishes_only_the_fixed_c4os_broker_tools() {
    let resources = production_broker_authoritative_resources();
    assert_eq!(resources.version, 1);
    assert_eq!(
        resources
            .records
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["c4os_propose_action", "c4os_read_resource"]
    );
    assert!(resources.records.values().all(|resource| {
        resource.kind == "broker-tool"
            && resource.version == 1
            && resource.sha256.starts_with("sha256:")
    }));
    let preflight = golden_installed_resources();
    assert_eq!(
        preflight.tool_ids,
        resources.records.keys().cloned().collect()
    );
    assert_eq!(
        preflight.snapshot_sha256,
        authoritative_resources_sha256(&resources).unwrap()
    );
}

#[test]
fn workspace_rebind_invalidates_stale_text_dispatch_generation() {
    let temporary = TempDir::new().expect("runtime workspace fixture");
    let c4os_home = temporary.path().join("c4os-home");
    let (app_database, _) = DatabaseActor::start(DatabaseDescriptor::app(&c4os_home)).unwrap();
    let application = RuntimeApplicationService::restore(Arc::new(app_database), NOW).unwrap();
    let (workspace_a, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        temporary.path().join("workspace-a"),
        "workspace-a",
    ))
    .unwrap();
    let (workspace_b, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        temporary.path().join("workspace-b"),
        "workspace-b",
    ))
    .unwrap();

    application.bind_workspace(Arc::new(workspace_a)).unwrap();
    let workspace_a_generation = application.snapshot(NOW).unwrap().generation;
    let provisional = application
        .create_provisional(workspace_a_generation, "session-workspace-a", NOW + 1)
        .unwrap();
    application.unbind_workspace("workspace-a").unwrap();
    application.bind_workspace(Arc::new(workspace_b)).unwrap();
    assert_eq!(
        application.bound_workspace_id().unwrap().as_deref(),
        Some("workspace-b")
    );

    let rebound = application.snapshot(NOW + 2).unwrap();
    assert!(
        rebound.generation > provisional.coordinator_generation,
        "changing the bound Workspace must invalidate renderer and worker intents minted for the prior Workspace"
    );
    assert!(matches!(
        application.create_provisional(
            provisional.coordinator_generation,
            "session-stale-workspace-a",
            NOW + 2,
        ),
        Err(RuntimeApplicationError::Generation {
            expected,
            current,
        }) if expected == provisional.coordinator_generation && current == rebound.generation
    ));
}

#[test]
#[ignore = "bundle tier: set C4OS_BUNDLED_RESOURCE_ROOT to a built C4OS.app resource tree"]
fn packaged_graphs_construct_a_ready_process_idle_bootstrap() {
    let configured = std::env::var_os("C4OS_BUNDLED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("C4OS_BUNDLED_RESOURCE_ROOT must name packaged resources");
    let resource_root = if configured.is_absolute() {
        configured
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .join(configured)
    };
    let canonical_resource_root = resource_root
        .canonicalize()
        .expect("canonical resource root");
    let bootstrap = RuntimeProductionBootstrap::new(&canonical_resource_root, None)
        .expect("verified production bootstrap");
    let readiness = bootstrap.readiness();
    let temporary = TempDir::new().expect("bundle installation fixture");
    let c4os_home = temporary.path().join("c4os-home");
    std::fs::create_dir_all(&c4os_home).expect("bundle installation home");
    let installations = bootstrap
        .runtime_installations("workspace-bundle", &c4os_home)
        .expect("verified bundle installations");
    let pi = installations
        .iter()
        .find(|installation| installation.runtime_kind == RuntimeKind::Pi)
        .expect("Pi installation");
    assert!(
        pi.executable.starts_with(&canonical_resource_root),
        "Pi must execute the bundle-contained Node runtime"
    );

    assert_eq!(readiness.opencode_native_version, "1.18.3");
    assert_eq!(readiness.pi_native_version, "0.80.10");
    assert!(readiness.opencode_native_sha256.starts_with("sha256:"));
    assert!(readiness.opencode_native_tree_sha256.starts_with("sha256:"));
    assert!(readiness.pi_dependency_tree_sha256.starts_with("sha256:"));
    assert!(readiness.node_sha256.starts_with("sha256:"));
    assert_eq!(readiness.node_version, "26.2.0");
    assert!(!readiness.credential_vault_available);
    assert_eq!(bootstrap.active_broker_contexts(), 0);
    assert!(!format!("{bootstrap:?}").contains("credential-"));
}

#[test]
#[ignore = "bundle tier: set C4OS_BUNDLED_RESOURCE_ROOT to a built C4OS.app resource tree"]
fn packaged_opencode_attach_cas_failure_cleans_the_real_process_group_and_descendant() {
    let configured = std::env::var_os("C4OS_BUNDLED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("C4OS_BUNDLED_RESOURCE_ROOT must name packaged resources");
    let resource_root = if configured.is_absolute() {
        configured
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .join(configured)
    };
    let temporary = TempDir::new().expect("attach-CAS native fixture");
    let c4os_home = temporary.path().join("c4os-home");
    let (app_database, _) = DatabaseActor::start(DatabaseDescriptor::app(&c4os_home)).unwrap();
    let application =
        Arc::new(RuntimeApplicationService::restore(Arc::new(app_database), NOW).unwrap());
    let (workspace_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        temporary.path().join("workspace"),
        "workspace-native",
    ))
    .unwrap();
    let provider_vault = CredentialVault::session_only().unwrap();
    let loopback_vault = CredentialVault::session_only().unwrap();
    let bootstrap = RuntimeProductionBootstrap::new_with_loopback_vault_for_test(
        resource_root,
        Some(provider_vault.clone()),
        loopback_vault.clone(),
    )
    .unwrap();
    let installations = bootstrap
        .runtime_installations("workspace-native", &c4os_home)
        .unwrap();
    application
        .bind_workspace_runtime_installations(
            Arc::new(workspace_database),
            installations.into_iter().collect(),
            NOW + 1,
        )
        .unwrap();
    let observed_process_group = Arc::new(Mutex::new(None));
    let host = RuntimeProductionApplication::with_backend(
        Arc::clone(&application),
        AttachInvalidatingBootstrap {
            inner: bootstrap,
            application: Arc::clone(&application),
            observed_process_group: Arc::clone(&observed_process_group),
        },
    );

    let activation = host.activate_runtime(1, "opencode-primary", NOW + 2);
    assert!(
        matches!(
            activation,
            Err(RuntimeProductionApplicationError::Application(
                RuntimeApplicationError::Generation { .. }
            ))
        ),
        "unexpected attach-CAS activation result: {activation:?}"
    );
    let (process_group_id, members) = observed_process_group
        .lock()
        .unwrap()
        .clone()
        .expect("exact OpenCode process group observed before attach invalidation");
    assert!(
        members.contains(&process_group_id)
            && members
                .iter()
                .any(|process_id| *process_id != process_group_id),
        "the exact OpenCode launcher and native descendant were not both observable"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while process_group_exists(process_group_id) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !process_group_exists(process_group_id),
        "attach-CAS rollback left the exact OpenCode process group alive"
    );
    assert_eq!(host.active_runtime_count().unwrap(), 0);
    assert!(loopback_vault.metadata().unwrap().is_empty());
    assert!(
        provider_vault
            .metadata()
            .unwrap()
            .iter()
            .all(|metadata| metadata.kind != "opencode-loopback-password")
    );
    let snapshot = application.snapshot(NOW + 3).unwrap();
    let record = snapshot
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == "opencode-primary")
        .unwrap();
    assert_eq!(record.lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(record.process_id, None);
}

#[test]
#[ignore = "bundle tier: set C4OS_BUNDLED_RESOURCE_ROOT to a built C4OS.app resource tree"]
fn packaged_pi_attach_cas_failure_cleans_the_real_process_group() {
    let configured = std::env::var_os("C4OS_BUNDLED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("C4OS_BUNDLED_RESOURCE_ROOT must name packaged resources");
    let resource_root = if configured.is_absolute() {
        configured
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .join(configured)
    };
    let temporary = TempDir::new().expect("Pi attach-CAS native fixture");
    let c4os_home = temporary.path().join("c4os-home");
    let (app_database, _) = DatabaseActor::start(DatabaseDescriptor::app(&c4os_home)).unwrap();
    let application =
        Arc::new(RuntimeApplicationService::restore(Arc::new(app_database), NOW).unwrap());
    let (workspace_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        temporary.path().join("workspace"),
        "workspace-native",
    ))
    .unwrap();
    let provider_vault = CredentialVault::session_only().unwrap();
    let loopback_vault = CredentialVault::session_only().unwrap();
    let bootstrap = RuntimeProductionBootstrap::new_with_loopback_vault_for_test(
        resource_root,
        Some(provider_vault.clone()),
        loopback_vault.clone(),
    )
    .unwrap();
    let installations = bootstrap
        .runtime_installations("workspace-native", &c4os_home)
        .unwrap();
    application
        .bind_workspace_runtime_installations(
            Arc::new(workspace_database),
            installations.into_iter().collect(),
            NOW + 1,
        )
        .unwrap();
    let observed_process_group = Arc::new(Mutex::new(None));
    let host = RuntimeProductionApplication::with_backend(
        Arc::clone(&application),
        AttachInvalidatingBootstrap {
            inner: bootstrap,
            application: Arc::clone(&application),
            observed_process_group: Arc::clone(&observed_process_group),
        },
    );

    assert!(matches!(
        host.activate_runtime(1, "pi-primary", NOW + 2),
        Err(RuntimeProductionApplicationError::Application(
            RuntimeApplicationError::Generation { .. }
        ))
    ));
    let (process_group_id, members) = observed_process_group
        .lock()
        .unwrap()
        .clone()
        .expect("exact Pi process group observed before attach invalidation");
    assert!(
        members.contains(&process_group_id),
        "the exact Pi sidecar process group leader was not observable"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while process_group_exists(process_group_id) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !process_group_exists(process_group_id),
        "attach-CAS rollback left the exact Pi process group alive"
    );
    assert_eq!(host.active_runtime_count().unwrap(), 0);
    assert!(loopback_vault.metadata().unwrap().is_empty());
    assert!(provider_vault.metadata().unwrap().is_empty());
    let snapshot = application.snapshot(NOW + 3).unwrap();
    let record = snapshot
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == "pi-primary")
        .unwrap();
    assert_eq!(record.lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(record.process_id, None);
}

#[test]
#[ignore = "native tier: requires a rebuilt C4OS.app and private test-TLS capability wrapper"]
fn packaged_production_peers_complete_the_app_owned_native_golden_paths() {
    const PI_IMAGE: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, b'I', b'H', b'D',
        b'R', 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xb5,
        0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, b'I', b'D', b'A', b'T', 0x78, 0xda, 0x63, 0x64,
        0xf8, 0x0f, 0x00, 0x01, 0x05, 0x01, 0x01, 0x27, 0x18, 0xe3, 0x66, 0x00, 0x00, 0x00, 0x00,
        b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82,
    ];
    let native_now = current_epoch_ms();

    let configured = std::env::var_os("C4OS_BUNDLED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("C4OS_BUNDLED_RESOURCE_ROOT must name final packaged resources");
    let resource_root = if configured.is_absolute() {
        configured
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .join(configured)
    };
    let certificate = PathBuf::from(
        std::env::var_os("C4OS_NATIVE_TLS_CERT").expect("trusted fixture certificate path"),
    );
    let ca_certificate = PathBuf::from(
        std::env::var_os("C4OS_NATIVE_TLS_CA").expect("fixture root certificate path"),
    );
    let private_key =
        PathBuf::from(std::env::var_os("C4OS_NATIVE_TLS_KEY").expect("fixture private-key path"));
    let credential_file = PathBuf::from(
        std::env::var_os("C4OS_NATIVE_PROVIDER_CREDENTIAL_FILE")
            .expect("fixture provider credential path"),
    );
    let evidence_path = PathBuf::from(
        std::env::var_os("C4OS_NATIVE_PROVIDER_EVIDENCE").expect("fixture evidence path"),
    );

    let temporary = TempDir::new().expect("native production golden root");
    let c4os_home = temporary.path().join("c4os-home");
    let workspace_root = temporary.path().join("workspace");
    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(&c4os_home)).expect("app database");
    let app_database = Arc::new(app_database);
    let application = Arc::new(
        RuntimeApplicationService::restore(Arc::clone(&app_database), native_now)
            .expect("production application service"),
    );
    let (workspace_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        &workspace_root,
        "workspace-native",
    ))
    .expect("workspace database");
    workspace_database
        .create_workspace(WorkspaceRecord {
            workspace_id: "workspace-native".into(),
            display_name: "Native golden Workspace".into(),
            created_at: native_now as i64,
            updated_at: native_now as i64,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("native golden Workspace record");
    workspace_database
        .add_project(ProjectRecord {
            workspace_id: "workspace-native".into(),
            project_id: "project-native".into(),
            display_name: "Native golden Project".into(),
            current_path: workspace_root.display().to_string(),
            last_known_path: workspace_root.display().to_string(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("native golden Project record");
    for chat_id in [
        "session-native-opencode-allow",
        "session-native-opencode-deny",
        "session-native-pi",
    ] {
        workspace_database
            .add_chat(ChatRecord {
                workspace_id: "workspace-native".into(),
                project_id: "project-native".into(),
                chat_id: chat_id.into(),
                title: "Native golden Chat".into(),
                created_at: native_now as i64,
                updated_at: native_now as i64,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .expect("native golden Chat record");
    }
    let workspace_database = Arc::new(workspace_database);
    let repository =
        SqliteSessionRepository::new(Arc::clone(&workspace_database)).expect("session repository");

    let vault = CredentialVault::session_only().expect("session-only native provider vault");
    let loopback_vault =
        CredentialVault::session_only().expect("session-only native loopback vault");
    let mut bootstrap = RuntimeProductionBootstrap::new_with_loopback_vault_for_test(
        &resource_root,
        Some(vault.clone()),
        loopback_vault.clone(),
    )
    .expect("verified production bootstrap");
    let installations = bootstrap
        .runtime_installations("workspace-native", &c4os_home)
        .expect("verified runtime installations");
    let node = installations
        .iter()
        .find(|installation| {
            installation.runtime_kind == c4os_lib::runtime::supervisor::RuntimeKind::Pi
        })
        .expect("packaged Node installation")
        .executable
        .clone();
    let fixture =
        NativeProviderFixture::start(&node, &certificate, &private_key, evidence_path.clone());

    let mut provider_secret = fs::read(&credential_file).expect("read provider credential");
    while provider_secret.last().is_some_and(u8::is_ascii_whitespace) {
        provider_secret.pop();
    }
    assert!(provider_secret.len() >= 32);
    fs::remove_file(&credential_file).expect("remove consumed provider credential file");
    let provider_secret_sha256 = sha256_bytes(&provider_secret);
    let provider_credential = vault
        .store("native-golden-provider-key", &provider_secret)
        .expect("store native provider credential");

    let allowed_calls = Arc::new(AtomicUsize::new(0));
    let denied_calls = Arc::new(AtomicUsize::new(0));
    bootstrap
        .install_broker_action(
            "window.focus",
            "allow-window",
            Map::new(),
            golden_classification("allow-window"),
            Box::new(CountingFacility {
                calls: Arc::clone(&allowed_calls),
            }),
        )
        .expect("install allowed native action");
    bootstrap
        .install_broker_action(
            "window.focus",
            "deny-window",
            Map::new(),
            golden_classification("deny-window"),
            Box::new(CountingFacility {
                calls: Arc::clone(&denied_calls),
            }),
        )
        .expect("install denied native action");

    application
        .bind_workspace_runtime_installations(
            Arc::clone(&workspace_database),
            installations.into_iter().collect(),
            native_now + 1,
        )
        .expect("atomic workspace/runtime binding");
    let profile = ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: "native-golden-provider".into(),
        kind: ProviderKind::Custom,
        display_name: "Native golden provider".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "native-golden-endpoint".into(),
            base_url: fixture.base_url.clone(),
            api_kind: "openai-compatible".into(),
        },
        authentication: ProviderAuthentication::Bearer,
        credential_reference: Some(provider_credential.clone()),
        headers: BTreeMap::new(),
        enabled: true,
    };
    bootstrap
        .install_test_tls_trust_capability(
            &profile,
            fs::read(&ca_certificate).expect("read private test-TLS root capability"),
        )
        .expect("install private test-TLS trust capability");
    let before_save = application
        .snapshot(native_now + 2)
        .expect("snapshot before save");
    application
        .save_provider(
            before_save.generation,
            profile.clone(),
            before_save.providers.generation,
            native_now + 2,
        )
        .expect("save native provider profile");
    let before_test = application
        .snapshot(native_now + 3)
        .expect("snapshot before test");
    application
        .test_provider(
            before_test.generation,
            &profile.provider_id,
            before_test.providers.generation,
            native_now + 3,
            &mut GoldenProviderProbe {
                discovery: ProviderDiscovery {
                    checked_at_ms: native_now + 3,
                    models: vec![golden_model_route(&profile, native_now + 3)],
                    recommended_model_id: Some("gpt-4o-mini".into()),
                    connection_evidence: None,
                },
            },
        )
        .expect("test native provider profile");

    let host = RuntimeProductionApplication::new(Arc::clone(&application), bootstrap);
    let mut now_ms = native_now + 4;
    let image_sha256 = sha256_bytes(PI_IMAGE);
    let image_digest = image_sha256
        .strip_prefix("sha256:")
        .expect("image digest prefix");
    let blob_directory = workspace_root.join("blobs/sha256");
    fs::create_dir_all(&blob_directory).expect("attachment blob directory");
    fs::write(blob_directory.join(image_digest), PI_IMAGE).expect("exact PNG attachment");
    let image = AttachmentSnapshot {
        attachment_id: "attachment-native-golden-image".into(),
        stable_reference: format!("workspace-blob:{image_sha256}:v1"),
        display_name: "native-golden.png".into(),
        media_type: "image/png".into(),
        byte_length: PI_IMAGE.len() as u64,
        content_sha256: image_sha256.clone(),
        snapshot_version: 1,
        original_reference: 1,
    };

    let before_opencode = application
        .snapshot(now_ms)
        .expect("snapshot before OpenCode activation");
    let opencode = host
        .activate_runtime(before_opencode.generation, "opencode-primary", now_ms)
        .expect("activate exact OpenCode production peer");
    let provisional = application
        .create_provisional(
            application.snapshot(now_ms).unwrap().generation,
            "session-native-opencode-allow",
            now_ms + 1,
        )
        .expect("create OpenCode allow provisional session");
    now_ms += 2;
    let opencode_dispatch = application
        .dispatch_first(
            provisional.coordinator_generation,
            golden_dispatch_intent(
                "opencode-primary",
                "session-native-opencode-allow",
                "opencode-allow",
                application
                    .dispatch_authority_generation()
                    .expect("OpenCode authority generation"),
                application
                    .capability_evidence_generation()
                    .expect("OpenCode capability generation"),
                vec![image.clone()],
                now_ms,
            ),
            text_dispatch_options(),
        )
        .expect("dispatch exact OpenCode allow golden prompt");
    if let CoordinatedFirstDispatch::Rejected { failure, .. } = &opencode_dispatch {
        panic!("unexpected OpenCode golden dispatch rejection: {failure:?}");
    }

    let (correlation_id, prompt_id) = pump_until_approval(
        &host,
        &repository,
        "opencode-primary",
        "session-native-opencode-allow",
        &mut now_ms,
    );
    host.answer_runtime_approval(
        application.snapshot(now_ms).unwrap().generation,
        "opencode-primary",
        &correlation_id,
        &prompt_id,
        ApprovalAnswer::Allow,
        now_ms + 1,
    )
    .expect("allow OpenCode native action");
    now_ms += 1;
    assert_eq!(allowed_calls.load(Ordering::SeqCst), 1);
    let opencode_allow_record = pump_until_completed(
        &host,
        &repository,
        "opencode-primary",
        "session-native-opencode-allow",
        &mut now_ms,
    );
    assert_golden_session(&opencode_allow_record, "golden-complete-opencode");

    let provisional = application
        .create_provisional(
            application.snapshot(now_ms).unwrap().generation,
            "session-native-opencode-deny",
            now_ms + 1,
        )
        .expect("create OpenCode deny provisional session");
    now_ms += 2;
    let opencode_deny_dispatch = application
        .dispatch_first(
            provisional.coordinator_generation,
            golden_dispatch_intent(
                "opencode-primary",
                "session-native-opencode-deny",
                "opencode-deny",
                application
                    .dispatch_authority_generation()
                    .expect("OpenCode deny authority generation"),
                application
                    .capability_evidence_generation()
                    .expect("OpenCode deny capability generation"),
                vec![image.clone()],
                now_ms,
            ),
            text_dispatch_options(),
        )
        .expect("dispatch exact OpenCode deny golden prompt");
    if let CoordinatedFirstDispatch::Rejected { failure, .. } = &opencode_deny_dispatch {
        panic!("unexpected OpenCode deny dispatch rejection: {failure:?}");
    }

    let (correlation_id, prompt_id) = pump_until_approval(
        &host,
        &repository,
        "opencode-primary",
        "session-native-opencode-deny",
        &mut now_ms,
    );
    host.answer_runtime_approval(
        application.snapshot(now_ms).unwrap().generation,
        "opencode-primary",
        &correlation_id,
        &prompt_id,
        ApprovalAnswer::Deny,
        now_ms + 1,
    )
    .expect("deny OpenCode native action");
    now_ms += 1;
    assert_eq!(allowed_calls.load(Ordering::SeqCst), 1);
    assert_eq!(denied_calls.load(Ordering::SeqCst), 0);
    let opencode_deny_record = pump_until_completed(
        &host,
        &repository,
        "opencode-primary",
        "session-native-opencode-deny",
        &mut now_ms,
    );
    assert_golden_session(&opencode_deny_record, "golden-complete-opencode");

    assert!(
        vault
            .metadata()
            .expect("provider credential metadata")
            .iter()
            .all(|metadata| metadata.kind != "opencode-loopback-password")
    );
    let server_reference = loopback_vault
        .metadata()
        .expect("credential metadata")
        .into_iter()
        .find(|metadata| metadata.kind == "opencode-loopback-password")
        .expect("OpenCode loopback credential")
        .credential_reference;
    let server_lease = loopback_vault
        .lease_for_operation(
            &server_reference,
            "task-00004-secret-scan",
            Duration::from_secs(5),
        )
        .expect("OpenCode loopback scan lease");
    let mut server_secret = Vec::new();
    server_lease
        .deliver_to(&mut server_secret)
        .expect("read loopback secret through one-use lease");
    let opencode_surfaces = process_group_surfaces(opencode.process_id);
    assert!(
        !opencode_surfaces
            .as_bytes()
            .windows(provider_secret.len())
            .any(|window| window == provider_secret.as_slice())
    );
    assert!(
        !opencode_surfaces
            .as_bytes()
            .windows(server_secret.len())
            .any(|window| window == server_secret.as_slice())
    );
    for forbidden in [
        "OPENCODE_SERVER_PASSWORD",
        "NODE_TLS_REJECT_UNAUTHORIZED",
        "NODE_EXTRA_CA_CERTS",
        "SSL_CERT_FILE",
        "BUN_OPTIONS",
    ] {
        assert!(!opencode_surfaces.contains(forbidden));
    }
    host.shutdown_runtime(
        application.snapshot(now_ms).unwrap().generation,
        "opencode-primary",
        now_ms + 1,
    )
    .expect("shutdown exact OpenCode production peer");
    now_ms += 1;
    assert!(!process_group_exists(opencode.process_id));
    assert!(loopback_vault.metadata().unwrap().is_empty());

    let before_pi = application
        .snapshot(now_ms)
        .expect("snapshot before Pi activation");
    let pi = host
        .activate_runtime(before_pi.generation, "pi-primary", now_ms + 1)
        .expect("activate exact Pi production peer");
    now_ms += 2;
    let provisional = application
        .create_provisional(
            application.snapshot(now_ms).unwrap().generation,
            "session-native-pi",
            now_ms + 1,
        )
        .expect("create Pi provisional session");
    now_ms += 2;
    let pi_dispatch = application
        .dispatch_first(
            provisional.coordinator_generation,
            golden_dispatch_intent(
                "pi-primary",
                "session-native-pi",
                "pi-sequential",
                application
                    .dispatch_authority_generation()
                    .expect("Pi authority generation"),
                application
                    .capability_evidence_generation()
                    .expect("Pi capability generation"),
                vec![image],
                now_ms,
            ),
            text_dispatch_options(),
        )
        .expect("dispatch exact Pi golden prompt and image");
    assert!(matches!(
        pi_dispatch,
        CoordinatedFirstDispatch::Accepted { .. }
    ));

    let (correlation_id, prompt_id) = pump_until_approval(
        &host,
        &repository,
        "pi-primary",
        "session-native-pi",
        &mut now_ms,
    );
    host.answer_runtime_approval(
        application.snapshot(now_ms).unwrap().generation,
        "pi-primary",
        &correlation_id,
        &prompt_id,
        ApprovalAnswer::Allow,
        now_ms + 1,
    )
    .expect("allow Pi native action");
    now_ms += 1;
    assert_eq!(allowed_calls.load(Ordering::SeqCst), 2);
    let (correlation_id, prompt_id) = pump_until_approval(
        &host,
        &repository,
        "pi-primary",
        "session-native-pi",
        &mut now_ms,
    );
    host.answer_runtime_approval(
        application.snapshot(now_ms).unwrap().generation,
        "pi-primary",
        &correlation_id,
        &prompt_id,
        ApprovalAnswer::Deny,
        now_ms + 1,
    )
    .expect("deny Pi native action");
    now_ms += 1;
    assert_eq!(allowed_calls.load(Ordering::SeqCst), 2);
    assert_eq!(denied_calls.load(Ordering::SeqCst), 0);
    let pi_record = pump_until_completed(
        &host,
        &repository,
        "pi-primary",
        "session-native-pi",
        &mut now_ms,
    );
    assert_golden_session(&pi_record, "golden-complete-pi");
    let pi_surfaces = process_group_surfaces(pi.process_id);
    assert!(
        !pi_surfaces
            .as_bytes()
            .windows(provider_secret.len())
            .any(|window| window == provider_secret.as_slice())
    );
    for forbidden in [
        "NODE_TLS_REJECT_UNAUTHORIZED",
        "NODE_EXTRA_CA_CERTS",
        "SSL_CERT_FILE",
        "BUN_OPTIONS",
    ] {
        assert!(!pi_surfaces.contains(forbidden));
    }
    host.shutdown_runtime(
        application.snapshot(now_ms).unwrap().generation,
        "pi-primary",
        now_ms + 1,
    )
    .expect("shutdown exact Pi production peer");
    assert!(!process_group_exists(pi.process_id));

    let fixture_evidence = read_fixture_evidence(&fixture.evidence_path);
    let requests = fixture_evidence["requests"]
        .as_array()
        .expect("fixture request evidence");
    for (path, expected_tools) in [
        (
            "/v1/chat/completions",
            serde_json::json!(["c4os_propose_action", "c4os_read_resource"]),
        ),
        (
            "/v1/responses",
            serde_json::json!(["c4os_propose_action", "c4os_read_resource"]),
        ),
    ] {
        let path_requests = requests
            .iter()
            .filter(|request| {
                request["path"] == path
                    && matches!(
                        request["stage"].as_str(),
                        Some("allow-tool" | "deny-tool" | "complete")
                    )
            })
            .collect::<Vec<_>>();
        let stages = path_requests
            .iter()
            .map(|request| request["stage"].as_str().unwrap())
            .collect::<Vec<_>>();
        if path == "/v1/chat/completions" {
            assert_eq!(stages, ["allow-tool", "complete", "deny-tool", "complete"]);
            assert_eq!(
                path_requests
                    .iter()
                    .map(|request| request["flow"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "opencode-allow",
                    "opencode-allow",
                    "opencode-deny",
                    "opencode-deny",
                ]
            );
        } else {
            assert_eq!(stages, ["allow-tool", "deny-tool", "complete"]);
        }
        for request in path_requests {
            assert_eq!(
                request["imageParts"],
                serde_json::json!([{
                    "mimeType": "image/png",
                    "sha256": image_sha256,
                }])
            );
            assert_eq!(request["providerToolNames"], expected_tools);
        }
    }
    assert!(
        requests
            .iter()
            .all(|request| request["stage"] != "rejected")
    );
    assert!(requests.iter().all(|request| {
        request["tlsEncrypted"] == true && request["credentialSha256"] == provider_secret_sha256
    }));
    let evidence_bytes = fs::read(&fixture.evidence_path).expect("fixture evidence bytes");
    assert!(
        !evidence_bytes
            .windows(provider_secret.len())
            .any(|window| window == provider_secret.as_slice())
    );

    let security_records = app_database
        .security_records(SnapshotQuery::new(100).expect("security query"))
        .expect("native Action Gateway journal");
    assert_eq!(
        security_records
            .iter()
            .filter(|record| record.record_kind == "action-result" && record.state == "succeeded")
            .count(),
        2
    );
    assert_eq!(
        security_records
            .iter()
            .filter(|record| record.record_kind == "action-result" && record.state == "denied")
            .count(),
        2
    );
    for root in [&c4os_home, &workspace_root] {
        assert!(tree_is_absent_of(root, &provider_secret));
        assert!(tree_is_absent_of(root, &server_secret));
    }

    let fixture_process_group_id = fixture.process_group_id;
    fixture.stop();
    assert!(!process_group_exists(fixture_process_group_id));
    provider_secret.fill(0);
    server_secret.fill(0);
}

#[test]
#[ignore = "bundle tier: set C4OS_BUNDLED_RESOURCE_ROOT to a built C4OS.app resource tree"]
fn packaged_pi_peer_reaches_ready_pumps_and_shuts_down_through_the_app_host() {
    let configured = std::env::var_os("C4OS_BUNDLED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("C4OS_BUNDLED_RESOURCE_ROOT must name packaged resources");
    let resource_root = if configured.is_absolute() {
        configured
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("project root")
            .join(configured)
    };
    let temporary = TempDir::new().expect("runtime host fixture");
    let c4os_home = temporary.path().join("c4os-home");
    let (app_database, _) = DatabaseActor::start(DatabaseDescriptor::app(&c4os_home)).unwrap();
    let application = Arc::new(
        RuntimeApplicationService::restore(Arc::new(app_database), 1_721_300_000_000).unwrap(),
    );
    let workspace_root = temporary.path().join("workspace");
    let (workspace_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        &workspace_root,
        "workspace-native",
    ))
    .unwrap();
    workspace_database
        .create_workspace(WorkspaceRecord {
            workspace_id: "workspace-native".into(),
            display_name: "Packaged Pi Workspace".into(),
            created_at: 1_721_300_000_000,
            updated_at: 1_721_300_000_000,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    workspace_database
        .add_project(ProjectRecord {
            workspace_id: "workspace-native".into(),
            project_id: "project-1".into(),
            display_name: "Packaged Pi Project".into(),
            current_path: workspace_root.display().to_string(),
            last_known_path: workspace_root.display().to_string(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    let bootstrap = RuntimeProductionBootstrap::new(
        resource_root,
        Some(CredentialVault::session_only().unwrap()),
    )
    .unwrap();
    let installations = bootstrap
        .runtime_installations("workspace-native", &c4os_home)
        .unwrap();
    application
        .bind_workspace_runtime_installations(
            Arc::new(workspace_database),
            installations.into_iter().collect(),
            1_721_300_000_001,
        )
        .unwrap();
    let host = RuntimeProductionApplication::new(Arc::clone(&application), bootstrap);

    let activated = host
        .activate_runtime(1, "pi-primary", 1_721_300_000_002)
        .unwrap();
    assert_ne!(activated.process_id, 0);
    assert_eq!(
        host.pump_runtime_once("pi-primary", 1_721_300_000_003)
            .unwrap(),
        0
    );
    let ready = application.snapshot(1_721_300_000_003).unwrap();
    assert_eq!(
        ready
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == "pi-primary")
            .unwrap()
            .lifecycle,
        RuntimeLifecycle::Ready
    );
    assert!(ready.providers.providers.is_empty());
    assert!(matches!(
        application.dispatch_first(
            ready.generation,
            text_dispatch_intent("workspace-native", "pi-primary", 1_721_300_000_004),
            text_dispatch_options(),
        ),
        Err(RuntimeApplicationError::Coordinator(
            CoordinatorError::ModelRouteUnavailable
        ))
    ));
    assert_eq!(
        application.snapshot(1_721_300_000_004).unwrap().generation,
        ready.generation,
        "zero-provider rejection must not create durable session or coordinator state"
    );

    let stale_generation = ready.generation.checked_sub(1).unwrap();
    assert!(matches!(
        host.shutdown_runtime(stale_generation, "pi-primary", 1_721_300_000_005),
        Err(RuntimeProductionApplicationError::Application(
            RuntimeApplicationError::Generation { expected, current }
        )) if expected == stale_generation && current == ready.generation
    ));
    assert_eq!(
        host.pump_runtime_once("pi-primary", 1_721_300_000_006)
            .expect("a stale shutdown must not terminate or unregister the active native peer"),
        0
    );
    let after_stale_shutdown = application.snapshot(1_721_300_000_006).unwrap();
    assert_eq!(
        after_stale_shutdown
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == "pi-primary")
            .unwrap()
            .lifecycle,
        RuntimeLifecycle::Ready
    );

    host.shutdown_runtime(
        after_stale_shutdown.generation,
        "pi-primary",
        1_721_300_000_007,
    )
    .unwrap();
    assert_eq!(
        application
            .snapshot(1_721_300_000_007)
            .unwrap()
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == "pi-primary")
            .unwrap()
            .lifecycle,
        RuntimeLifecycle::Stopped
    );
}
