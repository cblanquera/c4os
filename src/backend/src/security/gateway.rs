//! The only core path from a proposed effect to an executor grant.
//!
//! Policy resolution, approval, single-use authorization, and durable audit
//! transitions are coordinated here. An executor closure is never called until
//! the exact authorization consumption and the pre-effect transition have both
//! been durably saved.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::core::database::{DatabaseActor, DatabaseError, SecurityJournalRecord};

use super::authorization::{
    ApprovalAnswer, ApprovalPromptRecord, ApprovalPromptState, ApprovalQueue, ApprovalQueueError,
    AuthorizationConsumption, AuthorizationError, AuthorizationInvalidation, AuthorizationLedger,
    AuthorizationRecord, AuthorizationRepository, AuthorizationState, AuthorizationToken,
    CanonicalAction, GateDecision, LiveAuthorityState,
};
use super::policy::{
    ActionFacts, DecisionContribution, DecisionSource, PolicyConfiguration, PolicyDecision,
    PolicyResolution, resolve_policy,
};

pub const DEFAULT_AUTHORIZATION_TTL_MS: u64 = 60_000;
pub const DEFAULT_APPROVAL_TTL_MS: u64 = 5 * 60_000;
const MAX_RESULT_CODE_BYTES: usize = 160;
const MAX_CHANGED_TARGETS: usize = 256;
const MAX_CHANGED_TARGET_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NormalizedActionStatus {
    Succeeded,
    Failed,
    Cancelled,
    Denied,
    UnknownAfterInterruption,
}

/// Bounded post-effect data. Raw output, environment values, credentials, and
/// authorization tokens are deliberately not representable here.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedActionResult {
    pub status: NormalizedActionStatus,
    pub result_code: String,
    pub exit_code: Option<i32>,
    pub changed_targets: Vec<String>,
    pub output_sha256: Option<String>,
    pub completed_at_ms: u64,
}

/// Executor capability whose provenance is sealed inside ActionGateway. The
/// authorization state machine exposes only a non-capability consumption
/// receipt, so policy-free callers cannot mint this type.
#[derive(Debug, Eq, PartialEq)]
pub struct ExecutionPermit {
    authorization_id: String,
    action: CanonicalAction,
    consumed_at_ms: u64,
}

/// Opaque proof that one exact authorization was consumed and its durable
/// `effect-started` barrier was committed. Long-running native facilities may
/// hold this lease while the actual effect runs, then consume it exactly once
/// when recording the normalized terminal result.
#[derive(Debug)]
pub struct ActionEffectLease {
    permit: Option<ExecutionPermit>,
    action: CanonicalAction,
    approval_prompt_id: Option<String>,
    settled: bool,
}

impl ActionEffectLease {
    pub fn action(&self) -> &CanonicalAction {
        &self.action
    }

    pub fn authorization_id(&self) -> &str {
        self.permit
            .as_ref()
            .map(ExecutionPermit::authorization_id)
            .unwrap_or_default()
    }

    pub(crate) fn take_execution_permit(&mut self) -> Result<ExecutionPermit, ActionGatewayError> {
        self.permit
            .take()
            .ok_or(ActionGatewayError::MissingAuthorization)
    }
}

impl ExecutionPermit {
    fn from_consumption(consumption: AuthorizationConsumption) -> Self {
        Self {
            authorization_id: consumption.authorization_id().into(),
            action: consumption.action().clone(),
            consumed_at_ms: consumption.consumed_at_ms(),
        }
    }

    fn restored(authorization_id: String, action: CanonicalAction, consumed_at_ms: u64) -> Self {
        Self {
            authorization_id,
            action,
            consumed_at_ms,
        }
    }

    pub fn authorization_id(&self) -> &str {
        &self.authorization_id
    }

    pub fn action(&self) -> &CanonicalAction {
        &self.action
    }

    pub fn consumed_at_ms(&self) -> u64 {
        self.consumed_at_ms
    }
}

/// Exact action identity persisted without plaintext arguments or targets.
/// The full binding digest still commits to every canonical field.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RedactedActionBinding {
    schema_version: u16,
    action_id: String,
    tool_call_id: String,
    tool: String,
    arguments_sha256: String,
    risk: super::authorization::CanonicalRisk,
    requested_authority_sha256: String,
    canonical_target_sha256: String,
    target_version_sha256: String,
    workspace_id: String,
    session_id: String,
    run_id: String,
    runtime_id: String,
    environment_id: String,
    #[serde(default)]
    plugin_or_mcp_id: Option<String>,
    process_generation: u64,
    configuration_version: u64,
    policy_version: u64,
    revocation_epoch: u64,
    binding_digest: String,
}

impl RedactedActionBinding {
    fn from_action(action: &CanonicalAction) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            schema_version: action.schema_version,
            action_id: action.action_id.clone(),
            tool_call_id: action.tool_call_id.clone(),
            tool: action.tool.clone(),
            arguments_sha256: sha256_json(&action.arguments)?,
            risk: action.risk,
            requested_authority_sha256: sha256_json(&action.requested_authority)?,
            canonical_target_sha256: sha256_text(&action.canonical_target),
            target_version_sha256: sha256_text(&action.target_version),
            workspace_id: action.workspace_id.clone(),
            session_id: action.session_id.clone(),
            run_id: action.run_id.clone(),
            runtime_id: action.runtime_id.clone(),
            environment_id: action.environment_id.clone(),
            plugin_or_mcp_id: action.plugin_or_mcp_id.clone(),
            process_generation: action.process_generation,
            configuration_version: action.configuration_version,
            policy_version: action.policy_version,
            revocation_epoch: action.revocation_epoch,
            binding_digest: action
                .binding_digest()
                .map_err(|_| ActionGatewayError::Serialization)?,
        })
    }

    fn matches(&self, action: &CanonicalAction) -> Result<bool, ActionGatewayError> {
        Ok(self == &Self::from_action(action)?)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedAuthorization {
    payload_schema: String,
    authorization_id: String,
    action_binding: RedactedActionBinding,
    binding_digest: String,
    token_hash: String,
    issued_at_ms: u64,
    expires_at_ms: u64,
    state: AuthorizationState,
    #[serde(default)]
    approval_prompt_id: Option<String>,
}

impl PersistedAuthorization {
    fn from_record(
        record: &AuthorizationRecord,
        approval_prompt_id: Option<&str>,
    ) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            payload_schema: "authorization-v1".into(),
            authorization_id: record.authorization_id.clone(),
            action_binding: RedactedActionBinding::from_action(&record.action)?,
            binding_digest: record.binding_digest.clone(),
            token_hash: record.token_hash.clone(),
            issued_at_ms: record.issued_at_ms,
            expires_at_ms: record.expires_at_ms,
            state: record.state.clone(),
            approval_prompt_id: approval_prompt_id.map(str::to_owned),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedApproval {
    payload_schema: String,
    prompt_id: String,
    sequence: u64,
    action_binding: RedactedActionBinding,
    created_at_ms: u64,
    expires_at_ms: u64,
    state: ApprovalPromptState,
}

impl PersistedApproval {
    fn from_record(record: &ApprovalPromptRecord) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            payload_schema: "approval-prompt-v1".into(),
            prompt_id: record.prompt_id.clone(),
            sequence: record.sequence,
            action_binding: RedactedActionBinding::from_action(&record.action)?,
            created_at_ms: record.created_at_ms,
            expires_at_ms: record.expires_at_ms,
            state: record.state.clone(),
        })
    }
}

impl NormalizedActionResult {
    pub fn denied(code: impl Into<String>, completed_at_ms: u64) -> Self {
        Self {
            status: NormalizedActionStatus::Denied,
            result_code: code.into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        }
    }

    fn validate(&self) -> Result<(), ActionGatewayError> {
        if self.completed_at_ms == 0
            || self.result_code.is_empty()
            || self.result_code.len() > MAX_RESULT_CODE_BYTES
            || !self.result_code.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
            || self.changed_targets.len() > MAX_CHANGED_TARGETS
            || self.changed_targets.iter().any(|target| {
                target.is_empty()
                    || target.len() > MAX_CHANGED_TARGET_BYTES
                    || target.chars().any(char::is_control)
            })
            || self
                .output_sha256
                .as_deref()
                .is_some_and(|digest| !is_sha256_digest(digest))
        {
            return Err(ActionGatewayError::InvalidNormalizedResult);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum GatewayProposal {
    Denied {
        resolution: PolicyResolution,
    },
    PendingApproval {
        prompt: Box<ApprovalPromptRecord>,
        resolution: PolicyResolution,
    },
    Authorized {
        token: AuthorizationToken,
        resolution: PolicyResolution,
    },
}

#[derive(Debug)]
pub enum ApprovalResponse {
    Denied {
        prompt: Box<ApprovalPromptRecord>,
    },
    Authorized {
        prompt: Box<ApprovalPromptRecord>,
        token: AuthorizationToken,
    },
}

#[derive(Debug, Error)]
pub enum ActionGatewayError {
    #[error("the action facts do not match the canonical action")]
    BindingMismatch,
    #[error(
        "canonical arguments contain inline credential material; use an opaque credential reference"
    )]
    InlineCredentialMaterial,
    #[error("authorization failed: {0:?}")]
    Authorization(AuthorizationError),
    #[error("approval failed: {0:?}")]
    Approval(ApprovalQueueError),
    #[error("security journal serialization failed")]
    Serialization,
    #[error("security journal persistence failed: {0}")]
    Persistence(#[from] DatabaseError),
    #[error("the approved prompt no longer exists")]
    MissingApproval,
    #[error("the authorization record no longer exists")]
    MissingAuthorization,
    #[error("the normalized executor result is invalid or unbounded")]
    InvalidNormalizedResult,
}

/// SQLite adapter for authorization, approval, intent, decision, and result
/// state. DatabaseActor remains the sole writer owner.
pub struct SqliteSecurityRepository {
    database: Arc<DatabaseActor>,
}

struct JournalIndex<'a> {
    record_kind: &'a str,
    record_id: &'a str,
    run_id: &'a str,
    action_id: &'a str,
    state: &'a str,
    recorded_at_ms: u64,
}

impl SqliteSecurityRepository {
    pub fn new(database: Arc<DatabaseActor>) -> Self {
        Self { database }
    }

    fn journal_record(
        &self,
        record_kind: &str,
        record_id: &str,
        action: &CanonicalAction,
        state: &str,
        document: &impl Serialize,
        recorded_at_ms: u64,
    ) -> Result<SecurityJournalRecord, ActionGatewayError> {
        self.journal_record_indexed(
            JournalIndex {
                record_kind,
                record_id,
                run_id: &action.run_id,
                action_id: &action.action_id,
                state,
                recorded_at_ms,
            },
            document,
        )
    }

    fn journal_record_indexed(
        &self,
        index: JournalIndex<'_>,
        document: &impl Serialize,
    ) -> Result<SecurityJournalRecord, ActionGatewayError> {
        let canonical_document = serde_json::to_string(&SecurityEnvelope {
            schema_version: 1,
            record_kind: index.record_kind,
            record_id: index.record_id,
            run_id: index.run_id,
            action_id: index.action_id,
            state: index.state,
            recorded_at_ms: index.recorded_at_ms,
            payload: document,
        })
        .map_err(|_| ActionGatewayError::Serialization)?;
        Ok(SecurityJournalRecord {
            record_kind: index.record_kind.into(),
            record_id: index.record_id.into(),
            run_id: index.run_id.into(),
            action_id: index.action_id.into(),
            state: index.state.into(),
            canonical_document,
            recorded_at_ms: index.recorded_at_ms,
        })
    }

    fn save_serialized(
        &self,
        record_kind: &str,
        record_id: &str,
        action: &CanonicalAction,
        state: &str,
        document: &impl Serialize,
        recorded_at_ms: u64,
    ) -> Result<(), ActionGatewayError> {
        self.database.save_security_record(self.journal_record(
            record_kind,
            record_id,
            action,
            state,
            document,
            recorded_at_ms,
        )?)?;
        Ok(())
    }

    fn save_batch(&self, records: Vec<SecurityJournalRecord>) -> Result<(), ActionGatewayError> {
        self.database.save_security_records(records)?;
        Ok(())
    }

    fn authorization_record(
        &self,
        record: &AuthorizationRecord,
        approval_prompt_id: Option<&str>,
    ) -> Result<SecurityJournalRecord, ActionGatewayError> {
        let persisted = PersistedAuthorization::from_record(record, approval_prompt_id)?;
        self.journal_record(
            "authorization",
            &record.authorization_id,
            &record.action,
            authorization_state(&record.state),
            &persisted,
            authorization_recorded_at(record),
        )
    }

    fn persisted_authorization_record(
        &self,
        authorization: &PersistedAuthorization,
        recorded_at_ms: u64,
    ) -> Result<SecurityJournalRecord, ActionGatewayError> {
        self.journal_record_indexed(
            JournalIndex {
                record_kind: "authorization",
                record_id: &authorization.authorization_id,
                run_id: &authorization.action_binding.run_id,
                action_id: &authorization.action_binding.action_id,
                state: authorization_state(&authorization.state),
                recorded_at_ms,
            },
            authorization,
        )
    }

    fn approval_record(
        &self,
        record: &ApprovalPromptRecord,
    ) -> Result<SecurityJournalRecord, ActionGatewayError> {
        let persisted = PersistedApproval::from_record(record)?;
        self.journal_record(
            "approval-prompt",
            &record.prompt_id,
            &record.action,
            approval_state(&record.state),
            &persisted,
            approval_recorded_at(record),
        )
    }

    fn save_intent(
        &self,
        record_id: &str,
        state: &str,
        document: &impl Serialize,
        action: &CanonicalAction,
        at_ms: u64,
    ) -> Result<(), ActionGatewayError> {
        self.save_serialized("action-intent", record_id, action, state, document, at_ms)
    }

    fn save_decision(
        &self,
        record_id: &str,
        state: &str,
        document: &impl Serialize,
        action: &CanonicalAction,
        at_ms: u64,
    ) -> Result<(), ActionGatewayError> {
        self.save_serialized("action-decision", record_id, action, state, document, at_ms)
    }

    fn save_result(
        &self,
        record_id: &str,
        document: &impl Serialize,
        action: &CanonicalAction,
        result: &NormalizedActionResult,
    ) -> Result<(), ActionGatewayError> {
        self.save_serialized(
            "action-result",
            record_id,
            action,
            action_status(result.status),
            document,
            result.completed_at_ms,
        )
    }
}

impl AuthorizationRepository for SqliteSecurityRepository {
    type Error = ActionGatewayError;

    fn save_authorization(&mut self, record: &AuthorizationRecord) -> Result<(), Self::Error> {
        self.database
            .save_security_record(self.authorization_record(record, None)?)?;
        Ok(())
    }

    fn save_approval(&mut self, record: &ApprovalPromptRecord) -> Result<(), Self::Error> {
        self.database
            .save_security_record(self.approval_record(record)?)?;
        Ok(())
    }
}

pub struct ActionGateway {
    policy: PolicyConfiguration,
    authorizations: AuthorizationLedger,
    approvals: ApprovalQueue,
    restored_authorizations: BTreeMap<String, PersistedAuthorization>,
    approval_origins: BTreeMap<String, String>,
    pending_facts: BTreeMap<String, ActionFacts>,
    answered_facts: BTreeMap<String, ActionFacts>,
    repository: SqliteSecurityRepository,
    authorization_ttl_ms: u64,
    approval_ttl_ms: u64,
}

pub(crate) struct GlobalPolicyReplacement {
    policy: PolicyConfiguration,
    authorizations: AuthorizationLedger,
    restored_authorizations: BTreeMap<String, PersistedAuthorization>,
    authorization_records: Vec<SecurityJournalRecord>,
    invalidated_count: usize,
}

impl GlobalPolicyReplacement {
    pub(crate) fn authorization_records(&self) -> &[SecurityJournalRecord] {
        &self.authorization_records
    }

    pub(crate) fn invalidated_count(&self) -> usize {
        self.invalidated_count
    }
}

impl ActionGateway {
    pub fn new(policy: PolicyConfiguration, database: Arc<DatabaseActor>) -> Self {
        Self {
            policy,
            authorizations: AuthorizationLedger::default(),
            approvals: ApprovalQueue::default(),
            restored_authorizations: BTreeMap::new(),
            approval_origins: BTreeMap::new(),
            pending_facts: BTreeMap::new(),
            answered_facts: BTreeMap::new(),
            repository: SqliteSecurityRepository::new(database),
            authorization_ttl_ms: DEFAULT_AUTHORIZATION_TTL_MS,
            approval_ttl_ms: DEFAULT_APPROVAL_TTL_MS,
        }
    }

    /// Reconstructs only still-issued authorization verifiers. Open approvals
    /// are cancelled because their plaintext arguments/targets are purposely
    /// never durable; interrupted effects become explicit unknown results.
    pub fn restore(
        policy: PolicyConfiguration,
        database: Arc<DatabaseActor>,
        now_ms: u64,
    ) -> Result<Self, ActionGatewayError> {
        let recovery = database.recoverable_security_records()?;
        let mut gateway = Self::new(policy, database);
        let mut transitions = Vec::new();
        for record in recovery {
            match record.record_kind.as_str() {
                "authorization" => {
                    let mut authorization: PersistedAuthorization = decode_payload(&record)?;
                    validate_persisted_authorization(&record, &authorization)?;
                    if now_ms >= authorization.expires_at_ms {
                        authorization.state = AuthorizationState::Expired {
                            expired_at_ms: now_ms,
                        };
                        transitions.push(gateway.repository.journal_record_indexed(
                            JournalIndex {
                                record_kind: "authorization",
                                record_id: &authorization.authorization_id,
                                run_id: &authorization.action_binding.run_id,
                                action_id: &authorization.action_binding.action_id,
                                state: "expired",
                                recorded_at_ms: now_ms,
                            },
                            &authorization,
                        )?);
                    } else if authorization.approval_prompt_id.is_some() {
                        // A prompt cannot be reconstructed without its raw
                        // action detail, so its derived short-lived capability
                        // is cancelled in the same fail-closed restart pass.
                        authorization.state = AuthorizationState::Cancelled {
                            cancelled_at_ms: now_ms,
                        };
                        transitions.push(
                            gateway
                                .repository
                                .persisted_authorization_record(&authorization, now_ms)?,
                        );
                    } else {
                        gateway
                            .restored_authorizations
                            .insert(authorization.authorization_id.clone(), authorization);
                    }
                }
                "approval-prompt" => {
                    let mut approval: PersistedApproval = decode_payload(&record)?;
                    validate_persisted_approval(&record, &approval)?;
                    approval.state = ApprovalPromptState::Cancelled {
                        cancelled_at_ms: now_ms,
                    };
                    transitions.push(gateway.repository.journal_record_indexed(
                        JournalIndex {
                            record_kind: "approval-prompt",
                            record_id: &approval.prompt_id,
                            run_id: &approval.action_binding.run_id,
                            action_id: &approval.action_binding.action_id,
                            state: "cancelled",
                            recorded_at_ms: now_ms,
                        },
                        &approval,
                    )?);
                }
                "action-intent" => {
                    let mut intent: PersistedIntent = decode_payload(&record)?;
                    if intent.payload_schema != "action-intent-v1"
                        || intent.action_binding.run_id != record.run_id
                        || intent.action_binding.action_id != record.action_id
                        || intent.state != "effect-started"
                    {
                        return Err(ActionGatewayError::Serialization);
                    }
                    let result = PersistedResult::unknown(
                        intent.action_binding.clone(),
                        "interrupted-after-effect-start",
                        now_ms,
                    );
                    let result_id = format!("{}:result", record.action_id);
                    transitions.push(gateway.repository.journal_record_indexed(
                        JournalIndex {
                            record_kind: "action-result",
                            record_id: &result_id,
                            run_id: &record.run_id,
                            action_id: &record.action_id,
                            state: "unknown-after-interruption",
                            recorded_at_ms: now_ms,
                        },
                        &result,
                    )?);
                    intent.state = "interrupted".into();
                    transitions.push(gateway.repository.journal_record_indexed(
                        JournalIndex {
                            record_kind: "action-intent",
                            record_id: &record.record_id,
                            run_id: &record.run_id,
                            action_id: &record.action_id,
                            state: "interrupted",
                            recorded_at_ms: now_ms,
                        },
                        &intent,
                    )?);
                }
                _ => return Err(ActionGatewayError::Serialization),
            }
        }
        if !transitions.is_empty() {
            gateway.repository.save_batch(transitions)?;
        }
        Ok(gateway)
    }

    pub fn with_ttls(
        mut self,
        authorization_ttl_ms: u64,
        approval_ttl_ms: u64,
    ) -> Result<Self, ActionGatewayError> {
        if authorization_ttl_ms == 0 || approval_ttl_ms == 0 {
            return Err(ActionGatewayError::Authorization(
                AuthorizationError::InvalidTtl,
            ));
        }
        self.authorization_ttl_ms = authorization_ttl_ms;
        self.approval_ttl_ms = approval_ttl_ms;
        Ok(self)
    }

    pub fn policy(&self) -> &PolicyConfiguration {
        &self.policy
    }

    /// Tightening policy is immediate. The caller supplies the incremented live
    /// authority state and all stale authorizations are invalidated and saved.
    pub fn replace_policy(
        &mut self,
        policy: PolicyConfiguration,
        live: LiveAuthorityState,
        runtime_id: &str,
        environment_id: &str,
        now_ms: u64,
    ) -> Result<usize, ActionGatewayError> {
        self.policy = policy;
        let before = self.authorization_states();
        let mut count = self.authorizations.invalidate_stale_authority(
            runtime_id,
            environment_id,
            live,
            now_ms,
        );
        let mut records = self.changed_authorization_records(&before)?;
        for authorization in self.restored_authorizations.values_mut() {
            if authorization.state == AuthorizationState::Issued
                && authorization.action_binding.runtime_id == runtime_id
                && authorization.action_binding.environment_id == environment_id
                && let Some(reason) =
                    persisted_live_invalidation(&authorization.action_binding, live)
            {
                authorization.state = AuthorizationState::Invalidated {
                    invalidated_at_ms: now_ms,
                    reason,
                };
                records.push(
                    self.repository
                        .persisted_authorization_record(authorization, now_ms)?,
                );
                count += 1;
            }
        }
        if !records.is_empty() {
            self.repository.save_batch(records)?;
        }
        Ok(count)
    }

    /// Replaces the app-wide policy and invalidates every outstanding
    /// authorization. Category and exception settings are global authority;
    /// scoping invalidation to one currently installed runtime would leave a
    /// restored or temporarily stopped peer with stale permission.
    pub fn replace_policy_globally(
        &mut self,
        policy: PolicyConfiguration,
        revocation: bool,
        now_ms: u64,
    ) -> Result<usize, ActionGatewayError> {
        let replacement = self.prepare_policy_replacement(policy, revocation, now_ms)?;
        if !replacement.authorization_records.is_empty() {
            self.repository
                .save_batch(replacement.authorization_records.clone())?;
        }
        let count = replacement.invalidated_count;
        self.commit_policy_replacement(replacement);
        Ok(count)
    }

    pub(crate) fn prepare_policy_replacement(
        &self,
        policy: PolicyConfiguration,
        revocation: bool,
        now_ms: u64,
    ) -> Result<GlobalPolicyReplacement, ActionGatewayError> {
        let before = self.authorization_states();
        let reason = if revocation {
            AuthorizationInvalidation::RevocationEpoch
        } else {
            AuthorizationInvalidation::PolicyVersion
        };
        let mut authorizations = self.authorizations.clone();
        let mut count = authorizations.invalidate_all(reason, now_ms);
        let mut records = authorizations
            .records()
            .filter(|record| before.get(&record.authorization_id) != Some(&record.state))
            .map(|record| {
                self.repository.authorization_record(
                    record,
                    self.approval_origins
                        .get(&record.authorization_id)
                        .map(String::as_str),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut restored_authorizations = self.restored_authorizations.clone();
        for authorization in restored_authorizations.values_mut() {
            if authorization.state == AuthorizationState::Issued {
                authorization.state = AuthorizationState::Invalidated {
                    invalidated_at_ms: now_ms,
                    reason,
                };
                records.push(
                    self.repository
                        .persisted_authorization_record(authorization, now_ms)?,
                );
                count += 1;
            }
        }
        Ok(GlobalPolicyReplacement {
            policy,
            authorizations,
            restored_authorizations,
            authorization_records: records,
            invalidated_count: count,
        })
    }

    pub(crate) fn commit_policy_replacement(&mut self, replacement: GlobalPolicyReplacement) {
        self.policy = replacement.policy;
        self.authorizations = replacement.authorizations;
        self.restored_authorizations = replacement.restored_authorizations;
    }

    pub fn propose(
        &mut self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, ActionGatewayError> {
        validate_fact_binding(facts, &action)?;
        validate_no_inline_credentials(&action.arguments)?;
        let intent_id = format!("{}:intent", action.action_id);
        self.repository.save_intent(
            &intent_id,
            "proposed",
            &PersistedIntent::proposed(&action, facts)?,
            &action,
            now_ms,
        )?;

        let resolution = resolve_policy(facts, &self.policy, now_ms);
        let decision_id = format!("{}:decision", action.action_id);
        self.repository.save_decision(
            &decision_id,
            policy_state(resolution.decision),
            &PersistedDecision::new(&action, &resolution, now_ms)?,
            &action,
            now_ms,
        )?;

        match resolution.decision {
            PolicyDecision::Deny => {
                let result = NormalizedActionResult::denied("policy-denied", now_ms);
                self.persist_result(&action, &result)?;
                Ok(GatewayProposal::Denied { resolution })
            }
            PolicyDecision::Ask => {
                let prompt_id = format!("approval:{}", Uuid::new_v4().as_simple());
                let prompt = self
                    .approvals
                    .enqueue(prompt_id, action, now_ms, self.approval_ttl_ms)
                    .map_err(ActionGatewayError::Approval)?
                    .clone();
                self.repository.save_approval(&prompt)?;
                self.pending_facts
                    .insert(prompt.prompt_id.clone(), facts.clone());
                Ok(GatewayProposal::PendingApproval {
                    prompt: Box::new(prompt),
                    resolution,
                })
            }
            PolicyDecision::Allow => {
                let token = self
                    .authorizations
                    .issue(
                        GateDecision::Allow,
                        action,
                        now_ms,
                        self.authorization_ttl_ms,
                    )
                    .map_err(ActionGatewayError::Authorization)?;
                let record = self
                    .authorizations
                    .record(&token.authorization_id)
                    .ok_or(ActionGatewayError::MissingAuthorization)?
                    .clone();
                self.repository.save_authorization(&record)?;
                Ok(GatewayProposal::Authorized { token, resolution })
            }
        }
    }

    /// Revalidates an interrupted user-approved operation and creates a fresh
    /// prompt without ever reconstructing the cancelled prompt or silently
    /// converting it into an authorization. New deny ceilings still win; an
    /// otherwise-allowing policy is tightened to Ask for this recovery only.
    pub fn requeue_interrupted_approval(
        &mut self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, ActionGatewayError> {
        validate_fact_binding(facts, &action)?;
        validate_no_inline_credentials(&action.arguments)?;
        let intent_id = format!("{}:intent", action.action_id);
        self.repository.save_intent(
            &intent_id,
            "proposed",
            &PersistedIntent::proposed(&action, facts)?,
            &action,
            now_ms,
        )?;

        let mut resolution = resolve_policy(facts, &self.policy, now_ms);
        if resolution.decision == PolicyDecision::Allow {
            let source = DecisionSource::InterruptedApprovalRecovery;
            resolution.decision = PolicyDecision::Ask;
            resolution.controlling_sources = vec![source.clone()];
            resolution.contributions.push(DecisionContribution {
                decision: PolicyDecision::Ask,
                source,
            });
        }
        let decision_id = format!("{}:decision", action.action_id);
        self.repository.save_decision(
            &decision_id,
            policy_state(resolution.decision),
            &PersistedDecision::new(&action, &resolution, now_ms)?,
            &action,
            now_ms,
        )?;

        match resolution.decision {
            PolicyDecision::Deny => {
                let result = NormalizedActionResult::denied("policy-denied", now_ms);
                self.persist_result(&action, &result)?;
                Ok(GatewayProposal::Denied { resolution })
            }
            PolicyDecision::Ask => {
                let prompt_id = format!("approval:{}", Uuid::new_v4().as_simple());
                let prompt = self
                    .approvals
                    .enqueue(prompt_id, action, now_ms, self.approval_ttl_ms)
                    .map_err(ActionGatewayError::Approval)?
                    .clone();
                self.repository.save_approval(&prompt)?;
                self.pending_facts
                    .insert(prompt.prompt_id.clone(), facts.clone());
                Ok(GatewayProposal::PendingApproval {
                    prompt: Box::new(prompt),
                    resolution,
                })
            }
            PolicyDecision::Allow => Err(ActionGatewayError::Serialization),
        }
    }

    /// Requires an explicit user confirmation for a trust-granting action.
    /// Policy Deny remains final; an otherwise-Allow resolution is tightened
    /// to Ask with an auditable dedicated decision source.
    pub fn propose_trust_confirmation(
        &mut self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, ActionGatewayError> {
        validate_fact_binding(facts, &action)?;
        validate_no_inline_credentials(&action.arguments)?;
        let intent_id = format!("{}:intent", action.action_id);
        self.repository.save_intent(
            &intent_id,
            "proposed",
            &PersistedIntent::proposed(&action, facts)?,
            &action,
            now_ms,
        )?;

        let mut resolution = resolve_policy(facts, &self.policy, now_ms);
        if resolution.decision == PolicyDecision::Allow {
            let source = DecisionSource::ExplicitTrustConfirmation;
            resolution.decision = PolicyDecision::Ask;
            resolution.controlling_sources = vec![source.clone()];
            resolution.contributions.push(DecisionContribution {
                decision: PolicyDecision::Ask,
                source,
            });
        }
        let decision_id = format!("{}:decision", action.action_id);
        self.repository.save_decision(
            &decision_id,
            policy_state(resolution.decision),
            &PersistedDecision::new(&action, &resolution, now_ms)?,
            &action,
            now_ms,
        )?;

        match resolution.decision {
            PolicyDecision::Deny => {
                let result = NormalizedActionResult::denied("policy-denied", now_ms);
                self.persist_result(&action, &result)?;
                Ok(GatewayProposal::Denied { resolution })
            }
            PolicyDecision::Ask => {
                let prompt_id = format!("approval:{}", Uuid::new_v4().as_simple());
                let prompt = self
                    .approvals
                    .enqueue(prompt_id, action, now_ms, self.approval_ttl_ms)
                    .map_err(ActionGatewayError::Approval)?
                    .clone();
                self.repository.save_approval(&prompt)?;
                self.pending_facts
                    .insert(prompt.prompt_id.clone(), facts.clone());
                Ok(GatewayProposal::PendingApproval {
                    prompt: Box::new(prompt),
                    resolution,
                })
            }
            PolicyDecision::Allow => Err(ActionGatewayError::Serialization),
        }
    }

    /// Requires an explicit user confirmation before an MCP server may ask a
    /// configured model to generate content. Policy Deny remains final; an
    /// otherwise-Allow resolution is tightened to Ask with a dedicated audit
    /// source rather than being mislabeled as a trust grant.
    pub fn propose_sampling_confirmation(
        &mut self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, ActionGatewayError> {
        validate_fact_binding(facts, &action)?;
        validate_no_inline_credentials(&action.arguments)?;
        let intent_id = format!("{}:intent", action.action_id);
        self.repository.save_intent(
            &intent_id,
            "proposed",
            &PersistedIntent::proposed(&action, facts)?,
            &action,
            now_ms,
        )?;
        let mut resolution = resolve_policy(facts, &self.policy, now_ms);
        if resolution.decision == PolicyDecision::Allow {
            let source = DecisionSource::ExplicitSamplingConfirmation;
            resolution.decision = PolicyDecision::Ask;
            resolution.controlling_sources = vec![source.clone()];
            resolution.contributions.push(DecisionContribution {
                decision: PolicyDecision::Ask,
                source,
            });
        }
        let decision_id = format!("{}:decision", action.action_id);
        self.repository.save_decision(
            &decision_id,
            policy_state(resolution.decision),
            &PersistedDecision::new(&action, &resolution, now_ms)?,
            &action,
            now_ms,
        )?;
        match resolution.decision {
            PolicyDecision::Deny => {
                let result = NormalizedActionResult::denied("policy-denied", now_ms);
                self.persist_result(&action, &result)?;
                Ok(GatewayProposal::Denied { resolution })
            }
            PolicyDecision::Ask => {
                let prompt_id = format!("approval:{}", Uuid::new_v4().as_simple());
                let prompt = self
                    .approvals
                    .enqueue(prompt_id, action, now_ms, self.approval_ttl_ms)
                    .map_err(ActionGatewayError::Approval)?
                    .clone();
                self.repository.save_approval(&prompt)?;
                self.pending_facts
                    .insert(prompt.prompt_id.clone(), facts.clone());
                Ok(GatewayProposal::PendingApproval {
                    prompt: Box::new(prompt),
                    resolution,
                })
            }
            PolicyDecision::Allow => Err(ActionGatewayError::Serialization),
        }
    }

    pub fn answer_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<ApprovalResponse, ActionGatewayError> {
        let approvals_checkpoint = self.approvals.clone();
        let authorizations_checkpoint = self.authorizations.clone();
        let approval_origins_checkpoint = self.approval_origins.clone();
        let pending_facts_checkpoint = self.pending_facts.clone();
        let answered_facts_checkpoint = self.answered_facts.clone();
        let before = self.approval_states();
        let answered = self.approvals.answer(prompt_id, answer, now_ms);
        let prompt = match answered {
            Ok(prompt) => prompt.clone(),
            Err(error) => {
                // Expiry is a durable state transition even though the answer
                // itself fails. Other errors leave the record unchanged.
                if let Err(persistence) = self.persist_changed_approvals(&before) {
                    self.approvals = approvals_checkpoint;
                    return Err(persistence);
                }
                return Err(ActionGatewayError::Approval(error));
            }
        };
        let response = (|| match answer {
            ApprovalAnswer::Deny => {
                let result = NormalizedActionResult::denied("user-denied", now_ms);
                let mut records = self.changed_approval_records(&before)?;
                records.push(self.repository.journal_record(
                    "action-result",
                    &format!("{}:result", prompt.action.action_id),
                    &prompt.action,
                    "denied",
                    &PersistedResult::from_result(&prompt.action, &result)?,
                    now_ms,
                )?);
                self.repository.save_batch(records)?;
                Ok(ApprovalResponse::Denied {
                    prompt: Box::new(prompt),
                })
            }
            ApprovalAnswer::Allow => {
                let token = self
                    .authorizations
                    .issue(
                        GateDecision::Allow,
                        prompt.action.clone(),
                        now_ms,
                        self.authorization_ttl_ms,
                    )
                    .map_err(ActionGatewayError::Authorization)?;
                let record = self
                    .authorizations
                    .record(&token.authorization_id)
                    .ok_or(ActionGatewayError::MissingAuthorization)?
                    .clone();
                self.approval_origins
                    .insert(record.authorization_id.clone(), prompt.prompt_id.clone());
                let mut records = self.changed_approval_records(&before)?;
                records.push(
                    self.repository
                        .authorization_record(&record, Some(&prompt.prompt_id))?,
                );
                self.repository.save_batch(records)?;
                Ok(ApprovalResponse::Authorized {
                    prompt: Box::new(prompt),
                    token,
                })
            }
        })();
        if response.is_err() {
            self.approvals = approvals_checkpoint;
            self.authorizations = authorizations_checkpoint;
            self.approval_origins = approval_origins_checkpoint;
            self.pending_facts = pending_facts_checkpoint;
            self.answered_facts = answered_facts_checkpoint;
        } else if let Some(facts) = self.pending_facts.remove(prompt_id) {
            self.answered_facts.insert(prompt_id.to_owned(), facts);
        }
        response
    }

    pub(crate) fn take_answered_facts(&mut self, prompt_id: &str) -> Option<ActionFacts> {
        self.answered_facts.remove(prompt_id)
    }

    pub fn execute(
        &mut self,
        token: &AuthorizationToken,
        attempted_action: &CanonicalAction,
        live: LiveAuthorityState,
        approval_prompt_id: Option<&str>,
        now_ms: u64,
        effect: impl FnOnce(ExecutionPermit) -> NormalizedActionResult,
    ) -> Result<NormalizedActionResult, ActionGatewayError> {
        let mut lease =
            self.begin_effect(token, attempted_action, live, approval_prompt_id, now_ms)?;
        let permit = lease.take_execution_permit()?;
        let result = effect(permit);
        self.complete_effect(lease, result)
    }

    /// Consumes an exact single-use authorization and commits the durable
    /// pre-effect barrier without running the effect inline.
    pub fn begin_effect(
        &mut self,
        token: &AuthorizationToken,
        attempted_action: &CanonicalAction,
        live: LiveAuthorityState,
        approval_prompt_id: Option<&str>,
        now_ms: u64,
    ) -> Result<ActionEffectLease, ActionGatewayError> {
        match self.approval_origins.get(&token.authorization_id) {
            Some(expected_prompt_id) if approval_prompt_id != Some(expected_prompt_id.as_str()) => {
                return Err(ActionGatewayError::BindingMismatch);
            }
            None if approval_prompt_id.is_some() => {
                return Err(ActionGatewayError::BindingMismatch);
            }
            _ => {}
        }
        if let Some(prompt_id) = approval_prompt_id {
            let prompt = self
                .approvals
                .prompt(prompt_id)
                .ok_or(ActionGatewayError::MissingApproval)?;
            if prompt.action != *attempted_action {
                return Err(ActionGatewayError::BindingMismatch);
            }
        }
        let (permit, authorization_transition) = if let Some(before) = self
            .authorizations
            .record(&token.authorization_id)
            .map(|record| record.state.clone())
        {
            let consumed = self
                .authorizations
                .consume(token, attempted_action, live, now_ms);
            match consumed {
                Ok(permit) => {
                    let authorization = self
                        .authorizations
                        .record(&token.authorization_id)
                        .ok_or(ActionGatewayError::MissingAuthorization)?
                        .clone();
                    let transition = self.repository.authorization_record(
                        &authorization,
                        self.approval_origins
                            .get(&authorization.authorization_id)
                            .map(String::as_str),
                    )?;
                    (ExecutionPermit::from_consumption(permit), transition)
                }
                Err(error) => {
                    // Expiry and exact-binding failures burn the authorization.
                    // Persist only a real state transition, never a replay no-op.
                    if let Some(record) = self
                        .authorizations
                        .record(&token.authorization_id)
                        .filter(|record| record.state != before)
                        .cloned()
                    {
                        self.repository.save_authorization(&record)?;
                    }
                    return Err(ActionGatewayError::Authorization(error));
                }
            }
        } else {
            self.consume_restored(token, attempted_action, live, now_ms)?
        };
        let action = permit.action().clone();
        let intent_id = format!("{}:intent", action.action_id);
        let effect_started = self.repository.journal_record(
            "action-intent",
            &intent_id,
            &action,
            "effect-started",
            &PersistedIntent::started(&action, permit.authorization_id(), now_ms)?,
            now_ms,
        )?;
        // The single batch is the durable executor barrier: recovery can never
        // observe a consumed authorization without its effect-started intent.
        self.repository
            .save_batch(vec![authorization_transition, effect_started])?;

        Ok(ActionEffectLease {
            action,
            permit: Some(permit),
            approval_prompt_id: approval_prompt_id.map(str::to_owned),
            settled: false,
        })
    }

    /// Consumes a pre-effect lease and durably records the normalized result
    /// plus `effect-finished` (and approval completion when applicable).
    pub fn complete_effect(
        &mut self,
        mut lease: ActionEffectLease,
        result: NormalizedActionResult,
    ) -> Result<NormalizedActionResult, ActionGatewayError> {
        self.complete_effect_retryable(&mut lease, &result)?;
        Ok(result)
    }

    /// Persists completion without consuming the lease on failure. Runtime
    /// teardown uses this to retry a failed journal write while preserving the
    /// only authority handle for the already-started effect.
    pub fn complete_effect_retryable(
        &mut self,
        lease: &mut ActionEffectLease,
        result: &NormalizedActionResult,
    ) -> Result<(), ActionGatewayError> {
        if lease.settled {
            return Err(ActionGatewayError::MissingAuthorization);
        }
        self.complete_effect_records(lease.action(), lease.approval_prompt_id.as_deref(), result)?;
        lease.settled = true;
        Ok(())
    }

    fn complete_effect_records(
        &mut self,
        action: &CanonicalAction,
        approval_prompt_id: Option<&str>,
        result: &NormalizedActionResult,
    ) -> Result<(), ActionGatewayError> {
        let intent_id = format!("{}:intent", action.action_id);
        let result_id = format!("{}:result", action.action_id);
        let result_payload = PersistedResult::from_result(action, result)?;
        let mut completed_records = vec![
            self.repository.journal_record(
                "action-result",
                &result_id,
                action,
                action_status(result.status),
                &result_payload,
                result.completed_at_ms,
            )?,
            self.repository.journal_record(
                "action-intent",
                &intent_id,
                action,
                "effect-finished",
                &PersistedIntent::finished(action, result.completed_at_ms)?,
                result.completed_at_ms,
            )?,
        ];
        let completed_approval = if let Some(prompt_id) = approval_prompt_id {
            let prompt = self
                .approvals
                .completion_record(prompt_id, result.completed_at_ms)
                .map_err(ActionGatewayError::Approval)?;
            completed_records.push(self.repository.approval_record(&prompt)?);
            Some((prompt_id, prompt))
        } else {
            None
        };
        self.repository.save_batch(completed_records)?;
        if let Some((prompt_id, expected)) = completed_approval {
            let committed = self
                .approvals
                .complete(prompt_id, result.completed_at_ms)
                .map_err(ActionGatewayError::Approval)?;
            debug_assert_eq!(committed, &expected);
        }
        Ok(())
    }

    pub fn cancel_run(&mut self, run_id: &str, now_ms: u64) -> Result<usize, ActionGatewayError> {
        let authorization_states = self.authorization_states();
        let approval_states = self.approval_states();
        let mut authorizations = self.authorizations.cancel_run(run_id, now_ms);
        let approvals = self.approvals.cancel_run(run_id, now_ms);
        let mut records = self.changed_authorization_records(&authorization_states)?;
        for authorization in self.restored_authorizations.values_mut() {
            if authorization.state == AuthorizationState::Issued
                && authorization.action_binding.run_id == run_id
            {
                authorization.state = AuthorizationState::Cancelled {
                    cancelled_at_ms: now_ms,
                };
                records.push(
                    self.repository
                        .persisted_authorization_record(authorization, now_ms)?,
                );
                authorizations += 1;
            }
        }
        records.extend(self.changed_approval_records(&approval_states)?);
        if !records.is_empty() {
            self.repository.save_batch(records)?;
        }
        Ok(authorizations + approvals)
    }

    /// Revokes only authority derived from one exact Plugin identity. The
    /// transition is durably journaled before the ExtensionService terminates
    /// that Plugin's supervised hook generation.
    pub fn revoke_plugin(
        &mut self,
        plugin_id: &str,
        now_ms: u64,
    ) -> Result<usize, ActionGatewayError> {
        self.revoke_plugin_or_mcp(plugin_id, now_ms)
    }

    /// Revokes authority derived from one exact Plugin or MCP identity. The
    /// caller terminates that worker only after this durable transition.
    pub fn revoke_plugin_or_mcp(
        &mut self,
        identity: &str,
        now_ms: u64,
    ) -> Result<usize, ActionGatewayError> {
        if identity.trim().is_empty() {
            return Err(ActionGatewayError::BindingMismatch);
        }
        let authorization_states = self.authorization_states();
        let approval_states = self.approval_states();
        let mut authorizations = self.authorizations.revoke_plugin_or_mcp(identity, now_ms);
        let approvals = self.approvals.cancel_plugin_or_mcp(identity, now_ms);
        let mut records = self.changed_authorization_records(&authorization_states)?;
        for authorization in self.restored_authorizations.values_mut() {
            if authorization.state == AuthorizationState::Issued
                && authorization.action_binding.plugin_or_mcp_id.as_deref() == Some(identity)
            {
                authorization.state = AuthorizationState::Revoked {
                    revoked_at_ms: now_ms,
                };
                records.push(
                    self.repository
                        .persisted_authorization_record(authorization, now_ms)?,
                );
                authorizations += 1;
            }
        }
        records.extend(self.changed_approval_records(&approval_states)?);
        if !records.is_empty() {
            self.repository.save_batch(records)?;
        }
        Ok(authorizations + approvals)
    }

    pub fn expire_due(&mut self, now_ms: u64) -> Result<usize, ActionGatewayError> {
        let authorization_states = self.authorization_states();
        let approval_states = self.approval_states();
        let mut authorizations = self.authorizations.expire_due(now_ms);
        let approvals = self.approvals.expire_due(now_ms);
        let mut records = self.changed_authorization_records(&authorization_states)?;
        for authorization in self.restored_authorizations.values_mut() {
            if authorization.state == AuthorizationState::Issued
                && now_ms >= authorization.expires_at_ms
            {
                authorization.state = AuthorizationState::Expired {
                    expired_at_ms: now_ms,
                };
                records.push(
                    self.repository
                        .persisted_authorization_record(authorization, now_ms)?,
                );
                authorizations += 1;
            }
        }
        records.extend(self.changed_approval_records(&approval_states)?);
        if !records.is_empty() {
            self.repository.save_batch(records)?;
        }
        Ok(authorizations + approvals)
    }

    pub fn approval_queue(&self) -> &ApprovalQueue {
        &self.approvals
    }

    fn consume_restored(
        &mut self,
        token: &AuthorizationToken,
        attempted_action: &CanonicalAction,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<(ExecutionPermit, SecurityJournalRecord), ActionGatewayError> {
        let authorization = self
            .restored_authorizations
            .get_mut(&token.authorization_id)
            .ok_or(ActionGatewayError::Authorization(
                AuthorizationError::NotFound,
            ))?;
        if sha256_prefixed(token.secret_for_transport().as_bytes()) != authorization.token_hash {
            return Err(ActionGatewayError::Authorization(
                AuthorizationError::InvalidToken,
            ));
        }
        match authorization.state {
            AuthorizationState::Issued => {}
            AuthorizationState::Consumed { .. } => {
                return Err(ActionGatewayError::Authorization(
                    AuthorizationError::Replay,
                ));
            }
            AuthorizationState::Expired { .. } => {
                return Err(ActionGatewayError::Authorization(
                    AuthorizationError::Expired,
                ));
            }
            AuthorizationState::Cancelled { .. } => {
                return Err(ActionGatewayError::Authorization(
                    AuthorizationError::Cancelled,
                ));
            }
            AuthorizationState::Revoked { .. } => {
                return Err(ActionGatewayError::Authorization(
                    AuthorizationError::Revoked,
                ));
            }
            AuthorizationState::Invalidated { .. } => {
                return Err(ActionGatewayError::Authorization(
                    AuthorizationError::Invalidated,
                ));
            }
        }

        let terminal_error = if now_ms >= authorization.expires_at_ms {
            authorization.state = AuthorizationState::Expired {
                expired_at_ms: now_ms,
            };
            Some(AuthorizationError::Expired)
        } else if !authorization.action_binding.matches(attempted_action)? {
            authorization.state = AuthorizationState::Invalidated {
                invalidated_at_ms: now_ms,
                reason: AuthorizationInvalidation::Arguments,
            };
            Some(AuthorizationError::BindingMismatch(
                AuthorizationInvalidation::Arguments,
            ))
        } else if attempted_action.process_generation != live.process_generation {
            authorization.state = AuthorizationState::Invalidated {
                invalidated_at_ms: now_ms,
                reason: AuthorizationInvalidation::ProcessGeneration,
            };
            Some(AuthorizationError::BindingMismatch(
                AuthorizationInvalidation::ProcessGeneration,
            ))
        } else if attempted_action.configuration_version != live.configuration_version {
            authorization.state = AuthorizationState::Invalidated {
                invalidated_at_ms: now_ms,
                reason: AuthorizationInvalidation::ConfigurationVersion,
            };
            Some(AuthorizationError::BindingMismatch(
                AuthorizationInvalidation::ConfigurationVersion,
            ))
        } else if attempted_action.policy_version != live.policy_version {
            authorization.state = AuthorizationState::Invalidated {
                invalidated_at_ms: now_ms,
                reason: AuthorizationInvalidation::PolicyVersion,
            };
            Some(AuthorizationError::BindingMismatch(
                AuthorizationInvalidation::PolicyVersion,
            ))
        } else if attempted_action.revocation_epoch != live.revocation_epoch {
            authorization.state = AuthorizationState::Invalidated {
                invalidated_at_ms: now_ms,
                reason: AuthorizationInvalidation::RevocationEpoch,
            };
            Some(AuthorizationError::BindingMismatch(
                AuthorizationInvalidation::RevocationEpoch,
            ))
        } else {
            authorization.state = AuthorizationState::Consumed {
                consumed_at_ms: now_ms,
            };
            None
        };
        let persisted = authorization.clone();
        let state = authorization_state(&persisted.state);
        let transition = self
            .repository
            .persisted_authorization_record(&persisted, now_ms)?;
        if let Some(error) = terminal_error {
            self.repository.save_batch(vec![transition])?;
            return Err(ActionGatewayError::Authorization(error));
        }
        debug_assert_eq!(state, "consumed");
        Ok((
            ExecutionPermit::restored(persisted.authorization_id, attempted_action.clone(), now_ms),
            transition,
        ))
    }

    fn persist_result(
        &self,
        action: &CanonicalAction,
        result: &NormalizedActionResult,
    ) -> Result<(), ActionGatewayError> {
        let result_id = format!("{}:result", action.action_id);
        self.repository.save_result(
            &result_id,
            &PersistedResult::from_result(action, result)?,
            action,
            result,
        )
    }

    fn authorization_states(&self) -> BTreeMap<String, AuthorizationState> {
        self.authorizations
            .records()
            .map(|record| (record.authorization_id.clone(), record.state.clone()))
            .collect()
    }

    fn approval_states(&self) -> BTreeMap<String, ApprovalPromptState> {
        self.approvals
            .prompts()
            .map(|prompt| (prompt.prompt_id.clone(), prompt.state.clone()))
            .collect()
    }

    fn changed_authorization_records(
        &self,
        before: &BTreeMap<String, AuthorizationState>,
    ) -> Result<Vec<SecurityJournalRecord>, ActionGatewayError> {
        self.authorizations
            .records()
            .filter(|record| before.get(&record.authorization_id) != Some(&record.state))
            .map(|record| {
                self.repository.authorization_record(
                    record,
                    self.approval_origins
                        .get(&record.authorization_id)
                        .map(String::as_str),
                )
            })
            .collect()
    }

    fn persist_changed_approvals(
        &mut self,
        before: &BTreeMap<String, ApprovalPromptState>,
    ) -> Result<(), ActionGatewayError> {
        let records = self.changed_approval_records(before)?;
        if !records.is_empty() {
            self.repository.save_batch(records)?;
        }
        Ok(())
    }

    fn changed_approval_records(
        &self,
        before: &BTreeMap<String, ApprovalPromptState>,
    ) -> Result<Vec<SecurityJournalRecord>, ActionGatewayError> {
        self.approvals
            .prompts()
            .filter(|prompt| before.get(&prompt.prompt_id) != Some(&prompt.state))
            .map(|prompt| self.repository.approval_record(prompt))
            .collect()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SecurityEnvelope<'a, T> {
    schema_version: u16,
    record_kind: &'a str,
    record_id: &'a str,
    run_id: &'a str,
    action_id: &'a str,
    state: &'a str,
    recorded_at_ms: u64,
    payload: &'a T,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedIntent {
    payload_schema: String,
    state: String,
    action_binding: RedactedActionBinding,
    facts_sha256: Option<String>,
    authorization_id: Option<String>,
    occurred_at_ms: Option<u64>,
}

impl PersistedIntent {
    fn proposed(action: &CanonicalAction, facts: &ActionFacts) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            payload_schema: "action-intent-v1".into(),
            state: "proposed".into(),
            action_binding: RedactedActionBinding::from_action(action)?,
            facts_sha256: Some(sha256_json(facts)?),
            authorization_id: None,
            occurred_at_ms: None,
        })
    }

    fn started(
        action: &CanonicalAction,
        authorization_id: &str,
        at_ms: u64,
    ) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            payload_schema: "action-intent-v1".into(),
            state: "effect-started".into(),
            action_binding: RedactedActionBinding::from_action(action)?,
            facts_sha256: None,
            authorization_id: Some(authorization_id.into()),
            occurred_at_ms: Some(at_ms),
        })
    }

    fn finished(action: &CanonicalAction, at_ms: u64) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            payload_schema: "action-intent-v1".into(),
            state: "effect-finished".into(),
            action_binding: RedactedActionBinding::from_action(action)?,
            facts_sha256: None,
            authorization_id: None,
            occurred_at_ms: Some(at_ms),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedDecision {
    payload_schema: String,
    action_binding: RedactedActionBinding,
    decision: PolicyDecision,
    resolution_sha256: String,
    decided_at_ms: u64,
}

impl PersistedDecision {
    fn new(
        action: &CanonicalAction,
        resolution: &PolicyResolution,
        decided_at_ms: u64,
    ) -> Result<Self, ActionGatewayError> {
        Ok(Self {
            payload_schema: "action-decision-v1".into(),
            action_binding: RedactedActionBinding::from_action(action)?,
            decision: resolution.decision,
            resolution_sha256: sha256_json(resolution)?,
            decided_at_ms,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedResult {
    payload_schema: String,
    action_binding: RedactedActionBinding,
    status: NormalizedActionStatus,
    result_code_sha256: String,
    exit_code: Option<i32>,
    changed_target_sha256: Vec<String>,
    output_sha256: Option<String>,
    completed_at_ms: u64,
}

impl PersistedResult {
    fn from_result(
        action: &CanonicalAction,
        result: &NormalizedActionResult,
    ) -> Result<Self, ActionGatewayError> {
        result.validate()?;
        Ok(Self {
            payload_schema: "action-result-v1".into(),
            action_binding: RedactedActionBinding::from_action(action)?,
            status: result.status,
            result_code_sha256: sha256_text(&result.result_code),
            exit_code: result.exit_code,
            changed_target_sha256: result
                .changed_targets
                .iter()
                .map(|target| sha256_text(target))
                .collect(),
            output_sha256: result.output_sha256.clone(),
            completed_at_ms: result.completed_at_ms,
        })
    }

    fn unknown(action_binding: RedactedActionBinding, code: &str, completed_at_ms: u64) -> Self {
        Self {
            payload_schema: "action-result-v1".into(),
            action_binding,
            status: NormalizedActionStatus::UnknownAfterInterruption,
            result_code_sha256: sha256_text(code),
            exit_code: None,
            changed_target_sha256: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        }
    }
}

fn decode_payload<T: for<'de> Deserialize<'de>>(
    record: &SecurityJournalRecord,
) -> Result<T, ActionGatewayError> {
    let envelope: serde_json::Value = serde_json::from_str(&record.canonical_document)
        .map_err(|_| ActionGatewayError::Serialization)?;
    serde_json::from_value(
        envelope
            .get("payload")
            .cloned()
            .ok_or(ActionGatewayError::Serialization)?,
    )
    .map_err(|_| ActionGatewayError::Serialization)
}

fn validate_persisted_authorization(
    record: &SecurityJournalRecord,
    authorization: &PersistedAuthorization,
) -> Result<(), ActionGatewayError> {
    let token_digest = authorization.token_hash.strip_prefix("sha256:");
    if authorization.payload_schema != "authorization-v1"
        || authorization.authorization_id != record.record_id
        || authorization.action_binding.run_id != record.run_id
        || authorization.action_binding.action_id != record.action_id
        || authorization.binding_digest != authorization.action_binding.binding_digest
        || authorization.issued_at_ms >= authorization.expires_at_ms
        || authorization.state != AuthorizationState::Issued
        || record.state != "issued"
        || authorization
            .approval_prompt_id
            .as_deref()
            .is_some_and(|prompt_id| {
                prompt_id.strip_prefix("approval:").is_none_or(|id| {
                    id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            })
        || token_digest.is_none_or(|digest| {
            digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    {
        return Err(ActionGatewayError::Serialization);
    }
    Ok(())
}

fn validate_persisted_approval(
    record: &SecurityJournalRecord,
    approval: &PersistedApproval,
) -> Result<(), ActionGatewayError> {
    if approval.payload_schema != "approval-prompt-v1"
        || approval.prompt_id != record.record_id
        || approval.action_binding.run_id != record.run_id
        || approval.action_binding.action_id != record.action_id
        || approval.created_at_ms >= approval.expires_at_ms
        || approval_state(&approval.state) != record.state
        || !matches!(
            approval.state,
            ApprovalPromptState::Pending
                | ApprovalPromptState::Queued
                | ApprovalPromptState::Approved { .. }
        )
    {
        return Err(ActionGatewayError::Serialization);
    }
    Ok(())
}

fn sha256_json(value: &impl Serialize) -> Result<String, ActionGatewayError> {
    let bytes = serde_json::to_vec(value).map_err(|_| ActionGatewayError::Serialization)?;
    Ok(sha256_prefixed(&bytes))
}

fn sha256_text(value: &str) -> String {
    sha256_prefixed(value.as_bytes())
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(7 + 64);
    encoded.push_str("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn validate_fact_binding(
    facts: &ActionFacts,
    action: &CanonicalAction,
) -> Result<(), ActionGatewayError> {
    action
        .validate()
        .map_err(|_| ActionGatewayError::BindingMismatch)?;
    if facts.native_tool != action.tool
        || facts.canonical_target != action.canonical_target
        || facts.workspace_id != action.workspace_id
        || facts.session_id != action.session_id
        || facts.runtime_id != action.runtime_id
        || facts.environment_id != action.environment_id
        || facts.plugin_or_mcp_id != action.plugin_or_mcp_id
    {
        return Err(ActionGatewayError::BindingMismatch);
    }
    Ok(())
}

fn validate_no_inline_credentials(value: &serde_json::Value) -> Result<(), ActionGatewayError> {
    fn contains_inline(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(fields) => {
                let named_header_is_sensitive = fields
                    .iter()
                    .find(|(key, _)| matches!(key.to_ascii_lowercase().as_str(), "header" | "name"))
                    .and_then(|(_, value)| value.as_str())
                    .is_some_and(|name| {
                        let name = name.to_ascii_lowercase();
                        ["auth", "token", "secret", "api-key", "apikey", "cookie"]
                            .iter()
                            .any(|marker| name.contains(marker))
                    })
                    && fields.keys().any(|key| {
                        matches!(
                            key.to_ascii_lowercase().as_str(),
                            "value" | "payload" | "data"
                        )
                    });
                named_header_is_sensitive
                    || fields.iter().any(|(key, value)| {
                        let normalized = key
                            .chars()
                            .filter(|character| character.is_ascii_alphanumeric())
                            .flat_map(char::to_lowercase)
                            .collect::<String>();
                        let credential_field = [
                            "password",
                            "passwd",
                            "secret",
                            "token",
                            "cookie",
                            "authorization",
                            "apikey",
                            "privatekey",
                        ]
                        .iter()
                        .any(|marker| normalized.contains(marker));
                        let opaque_reference = normalized.ends_with("reference")
                            || normalized.ends_with("referenceid")
                            || normalized.ends_with("hash");
                        let numeric_token_measure = value.is_number()
                            && (normalized.ends_with("tokens")
                                || normalized.ends_with("tokencount"));
                        (credential_field && !opaque_reference && !numeric_token_measure)
                            || contains_inline(value)
                    })
            }
            serde_json::Value::Array(values) => values.iter().any(contains_inline),
            serde_json::Value::String(value) => {
                let lower = value.to_ascii_lowercase();
                let credentialed_uri = lower.contains("://")
                    && lower
                        .split_once("://")
                        .and_then(|(_, authority)| authority.split('/').next())
                        .is_some_and(|authority| {
                            authority.contains(':') && authority.contains('@')
                        });
                credentialed_uri
                    || lower.contains("-----begin private key-----")
                    || lower.contains("-----begin rsa private key-----")
                    || lower.contains("-----begin openssh private key-----")
                    || lower.contains("bearer ")
                    || lower.contains("basic ")
                    || lower.contains("password=")
                    || lower.contains("passwd=")
                    || lower.contains("secret=")
                    || lower.contains("token=")
                    || lower.contains("cookie=")
            }
            _ => false,
        }
    }

    if contains_inline(value) {
        Err(ActionGatewayError::InlineCredentialMaterial)
    } else {
        Ok(())
    }
}

fn policy_state(decision: PolicyDecision) -> &'static str {
    match decision {
        PolicyDecision::Allow => "allow",
        PolicyDecision::Ask => "ask",
        PolicyDecision::Deny => "deny",
    }
}

fn action_status(status: NormalizedActionStatus) -> &'static str {
    match status {
        NormalizedActionStatus::Succeeded => "succeeded",
        NormalizedActionStatus::Failed => "failed",
        NormalizedActionStatus::Cancelled => "cancelled",
        NormalizedActionStatus::Denied => "denied",
        NormalizedActionStatus::UnknownAfterInterruption => "unknown-after-interruption",
    }
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn persisted_live_invalidation(
    binding: &RedactedActionBinding,
    live: LiveAuthorityState,
) -> Option<AuthorizationInvalidation> {
    if binding.process_generation != live.process_generation {
        Some(AuthorizationInvalidation::ProcessGeneration)
    } else if binding.configuration_version != live.configuration_version {
        Some(AuthorizationInvalidation::ConfigurationVersion)
    } else if binding.policy_version != live.policy_version {
        Some(AuthorizationInvalidation::PolicyVersion)
    } else if binding.revocation_epoch != live.revocation_epoch {
        Some(AuthorizationInvalidation::RevocationEpoch)
    } else {
        None
    }
}

fn authorization_state(state: &super::authorization::AuthorizationState) -> &'static str {
    match state {
        AuthorizationState::Issued => "issued",
        AuthorizationState::Consumed { .. } => "consumed",
        AuthorizationState::Expired { .. } => "expired",
        AuthorizationState::Cancelled { .. } => "cancelled",
        AuthorizationState::Revoked { .. } => "revoked",
        AuthorizationState::Invalidated { .. } => "invalidated",
    }
}

fn authorization_recorded_at(record: &AuthorizationRecord) -> u64 {
    match record.state {
        AuthorizationState::Issued => record.issued_at_ms,
        AuthorizationState::Consumed { consumed_at_ms } => consumed_at_ms,
        AuthorizationState::Expired { expired_at_ms } => expired_at_ms,
        AuthorizationState::Cancelled { cancelled_at_ms } => cancelled_at_ms,
        AuthorizationState::Revoked { revoked_at_ms } => revoked_at_ms,
        AuthorizationState::Invalidated {
            invalidated_at_ms, ..
        } => invalidated_at_ms,
    }
}

fn approval_state(state: &ApprovalPromptState) -> &'static str {
    match state {
        ApprovalPromptState::Pending => "pending",
        ApprovalPromptState::Queued => "queued",
        ApprovalPromptState::Approved { .. } => "approved",
        ApprovalPromptState::Denied { .. } => "denied",
        ApprovalPromptState::Expired { .. } => "expired",
        ApprovalPromptState::Cancelled { .. } => "cancelled",
        ApprovalPromptState::Completed { .. } => "completed",
    }
}

fn approval_recorded_at(record: &ApprovalPromptRecord) -> u64 {
    match record.state {
        ApprovalPromptState::Pending | ApprovalPromptState::Queued => record.created_at_ms,
        ApprovalPromptState::Approved { answered_at_ms }
        | ApprovalPromptState::Denied { answered_at_ms } => answered_at_ms,
        ApprovalPromptState::Expired { expired_at_ms } => expired_at_ms,
        ApprovalPromptState::Cancelled { cancelled_at_ms } => cancelled_at_ms,
        ApprovalPromptState::Completed { completed_at_ms } => completed_at_ms,
    }
}
