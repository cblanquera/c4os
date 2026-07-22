//! Exact, single-use action authorization and approval queue state machines.
//!
//! The types here are persistence-ready records. Production integration must
//! implement [`AuthorizationRepository`] and durably save every transition
//! before executing or reporting a consequential effect.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::{self, Write};
use uuid::Uuid;

pub const CANONICAL_ACTION_SCHEMA_VERSION: u16 = 1;
const MAX_PERSISTED_IDENTIFIER_BYTES: usize = 160;
const MAX_CANONICAL_VALUE_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonicalRisk {
    Low,
    Medium,
    High,
    Unknown,
}

/// The complete action identity accepted by the action gateway.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalAction {
    pub schema_version: u16,
    pub action_id: String,
    pub tool_call_id: String,
    pub tool: String,
    pub arguments: Value,
    pub risk: CanonicalRisk,
    pub requested_authority: BTreeSet<String>,
    pub canonical_target: String,
    pub target_version: String,
    pub workspace_id: String,
    pub session_id: String,
    pub run_id: String,
    pub runtime_id: String,
    pub environment_id: String,
    #[serde(default)]
    pub plugin_or_mcp_id: Option<String>,
    pub process_generation: u64,
    pub configuration_version: u64,
    pub policy_version: u64,
    pub revocation_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalActionError {
    UnsupportedSchema,
    IncompleteBinding,
}

impl CanonicalAction {
    pub fn validate(&self) -> Result<(), CanonicalActionError> {
        if self.schema_version != CANONICAL_ACTION_SCHEMA_VERSION {
            return Err(CanonicalActionError::UnsupportedSchema);
        }
        if self.requested_authority.is_empty()
            || self
                .requested_authority
                .iter()
                .any(|authority| !is_safe_persisted_identifier(authority))
            || [
                &self.action_id,
                &self.tool_call_id,
                &self.tool,
                &self.workspace_id,
                &self.session_id,
                &self.run_id,
                &self.runtime_id,
                &self.environment_id,
            ]
            .iter()
            .any(|value| !is_safe_persisted_identifier(value))
            || self
                .plugin_or_mcp_id
                .as_deref()
                .is_some_and(|value| !is_safe_persisted_identifier(value))
            || !is_bounded_canonical_value(&self.canonical_target)
            || !is_bounded_canonical_value(&self.target_version)
            || serde_json::to_vec(&self.arguments).map_or(true, |arguments| {
                arguments.len() > MAX_CANONICAL_VALUE_BYTES
            })
        {
            return Err(CanonicalActionError::IncompleteBinding);
        }
        Ok(())
    }

    /// Stable digest for persistence, correlation, and source inspection.
    /// Authorization consumption still compares the full structure exactly.
    pub fn binding_digest(&self) -> Result<String, serde_json::Error> {
        let bytes = serde_json::to_vec(self)?;
        Ok(sha256_hex(&bytes))
    }
}

fn is_safe_persisted_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PERSISTED_IDENTIFIER_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

fn is_bounded_canonical_value(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= MAX_CANONICAL_VALUE_BYTES
        && !value.chars().any(char::is_control)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateDecision {
    Allow,
    Ask,
    Deny,
}

#[derive(Clone, Eq, PartialEq)]
pub struct AuthorizationToken {
    pub authorization_id: String,
    secret: String,
}

impl AuthorizationToken {
    /// Reconstructs the transport token presented by a supervised worker after
    /// the Rust core has restored the hashed durable record.
    pub fn from_transport(
        authorization_id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Result<Self, AuthorizationError> {
        let authorization_id = authorization_id.into();
        let secret = secret.into();
        if authorization_id.trim().is_empty() || secret.is_empty() {
            return Err(AuthorizationError::InvalidToken);
        }
        Ok(Self {
            authorization_id,
            secret,
        })
    }

    pub fn secret_for_transport(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for AuthorizationToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthorizationToken")
            .field("authorization_id", &self.authorization_id)
            .field("secret", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorizationInvalidation {
    ActionIdentity,
    Arguments,
    Risk,
    RequestedAuthority,
    Target,
    TargetVersion,
    Workspace,
    Session,
    Run,
    Runtime,
    Environment,
    ProcessGeneration,
    ConfigurationVersion,
    PolicyVersion,
    RevocationEpoch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum AuthorizationState {
    Issued,
    Consumed {
        consumed_at_ms: u64,
    },
    Expired {
        expired_at_ms: u64,
    },
    Cancelled {
        cancelled_at_ms: u64,
    },
    Revoked {
        revoked_at_ms: u64,
    },
    Invalidated {
        invalidated_at_ms: u64,
        reason: AuthorizationInvalidation,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationRecord {
    pub authorization_id: String,
    pub action: CanonicalAction,
    pub binding_digest: String,
    pub token_hash: String,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
    pub state: AuthorizationState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationConsumption {
    authorization_id: String,
    action: CanonicalAction,
    consumed_at_ms: u64,
}

impl AuthorizationConsumption {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationError {
    DecisionDoesNotAllow,
    InvalidTtl,
    InvalidCanonicalAction(CanonicalActionError),
    Serialization,
    PersistedDigestMismatch,
    DuplicateAuthorization,
    NotFound,
    InvalidToken,
    Replay,
    Expired,
    Cancelled,
    Revoked,
    Invalidated,
    BindingMismatch(AuthorizationInvalidation),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveAuthorityState {
    pub process_generation: u64,
    pub configuration_version: u64,
    pub policy_version: u64,
    pub revocation_epoch: u64,
}

/// In-memory state machine over persistence-ready authorization records.
#[derive(Default)]
pub struct AuthorizationLedger {
    records: BTreeMap<String, AuthorizationRecord>,
}

impl AuthorizationLedger {
    /// Restores persistence records after validating their canonical actions,
    /// binding digests, token hashes, TTLs, and unique identifiers.
    pub fn restore(
        records: impl IntoIterator<Item = AuthorizationRecord>,
    ) -> Result<Self, AuthorizationError> {
        let mut restored = BTreeMap::new();
        for record in records {
            if record.authorization_id.trim().is_empty() {
                return Err(AuthorizationError::NotFound);
            }
            record
                .action
                .validate()
                .map_err(AuthorizationError::InvalidCanonicalAction)?;
            if record.expires_at_ms <= record.issued_at_ms {
                return Err(AuthorizationError::InvalidTtl);
            }
            let expected_digest = record
                .action
                .binding_digest()
                .map_err(|_| AuthorizationError::Serialization)?;
            if record.binding_digest != expected_digest {
                return Err(AuthorizationError::PersistedDigestMismatch);
            }
            if !is_sha256_hex(&record.token_hash) {
                return Err(AuthorizationError::InvalidToken);
            }
            let id = record.authorization_id.clone();
            if restored.insert(id, record).is_some() {
                return Err(AuthorizationError::DuplicateAuthorization);
            }
        }
        Ok(Self { records: restored })
    }

    pub fn issue(
        &mut self,
        decision: GateDecision,
        action: CanonicalAction,
        issued_at_ms: u64,
        ttl_ms: u64,
    ) -> Result<AuthorizationToken, AuthorizationError> {
        self.issue_with_material(
            decision,
            action,
            issued_at_ms,
            ttl_ms,
            Uuid::new_v4().to_string(),
            Uuid::new_v4().to_string(),
        )
    }

    /// Deterministic issuance seam for tests and durable-id coordinators.
    pub fn issue_with_material(
        &mut self,
        decision: GateDecision,
        action: CanonicalAction,
        issued_at_ms: u64,
        ttl_ms: u64,
        authorization_id: String,
        token_secret: String,
    ) -> Result<AuthorizationToken, AuthorizationError> {
        if decision != GateDecision::Allow {
            return Err(AuthorizationError::DecisionDoesNotAllow);
        }
        if ttl_ms == 0 || issued_at_ms.checked_add(ttl_ms).is_none() {
            return Err(AuthorizationError::InvalidTtl);
        }
        action
            .validate()
            .map_err(AuthorizationError::InvalidCanonicalAction)?;
        if authorization_id.trim().is_empty() || token_secret.is_empty() {
            return Err(AuthorizationError::InvalidToken);
        }
        if self.records.contains_key(&authorization_id) {
            return Err(AuthorizationError::DuplicateAuthorization);
        }
        let binding_digest = action
            .binding_digest()
            .map_err(|_| AuthorizationError::Serialization)?;
        let token_hash = hash_token(&token_secret);
        let record = AuthorizationRecord {
            authorization_id: authorization_id.clone(),
            action,
            binding_digest,
            token_hash,
            issued_at_ms,
            expires_at_ms: issued_at_ms + ttl_ms,
            state: AuthorizationState::Issued,
        };
        self.records.insert(authorization_id.clone(), record);
        Ok(AuthorizationToken {
            authorization_id,
            secret: token_secret,
        })
    }

    /// Atomically consumes an exact authorization and returns the only action
    /// that may be executed. A mismatch burns the token so a mutated request
    /// cannot be repaired and replayed.
    pub fn consume(
        &mut self,
        token: &AuthorizationToken,
        attempted_action: &CanonicalAction,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<AuthorizationConsumption, AuthorizationError> {
        let record = self
            .records
            .get_mut(&token.authorization_id)
            .ok_or(AuthorizationError::NotFound)?;
        if hash_token(&token.secret) != record.token_hash {
            return Err(AuthorizationError::InvalidToken);
        }
        match &record.state {
            AuthorizationState::Issued => {}
            AuthorizationState::Consumed { .. } => return Err(AuthorizationError::Replay),
            AuthorizationState::Expired { .. } => return Err(AuthorizationError::Expired),
            AuthorizationState::Cancelled { .. } => return Err(AuthorizationError::Cancelled),
            AuthorizationState::Revoked { .. } => return Err(AuthorizationError::Revoked),
            AuthorizationState::Invalidated { .. } => return Err(AuthorizationError::Invalidated),
        }
        if now_ms >= record.expires_at_ms {
            record.state = AuthorizationState::Expired {
                expired_at_ms: now_ms,
            };
            return Err(AuthorizationError::Expired);
        }

        let mismatch = compare_action(&record.action, attempted_action)
            .or_else(|| compare_live_authority(&record.action, live));
        if let Some(reason) = mismatch {
            record.state = AuthorizationState::Invalidated {
                invalidated_at_ms: now_ms,
                reason,
            };
            return Err(AuthorizationError::BindingMismatch(reason));
        }

        record.state = AuthorizationState::Consumed {
            consumed_at_ms: now_ms,
        };
        Ok(AuthorizationConsumption {
            authorization_id: record.authorization_id.clone(),
            action: record.action.clone(),
            consumed_at_ms: now_ms,
        })
    }

    pub fn cancel_run(&mut self, run_id: &str, now_ms: u64) -> usize {
        mutate_issued(&mut self.records, now_ms, |record| {
            (record.action.run_id == run_id).then_some(AuthorizationState::Cancelled {
                cancelled_at_ms: now_ms,
            })
        })
    }

    pub fn revoke_authorization(&mut self, authorization_id: &str, now_ms: u64) -> bool {
        let Some(record) = self.records.get_mut(authorization_id) else {
            return false;
        };
        if record.state != AuthorizationState::Issued {
            return false;
        }
        record.state = AuthorizationState::Revoked {
            revoked_at_ms: now_ms,
        };
        true
    }

    pub fn revoke_plugin(&mut self, plugin_id: &str, now_ms: u64) -> usize {
        mutate_issued(&mut self.records, now_ms, |record| {
            (record.action.plugin_or_mcp_id.as_deref() == Some(plugin_id)).then_some(
                AuthorizationState::Revoked {
                    revoked_at_ms: now_ms,
                },
            )
        })
    }

    pub fn invalidate_stale_authority(
        &mut self,
        runtime_id: &str,
        environment_id: &str,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> usize {
        mutate_issued(&mut self.records, now_ms, |record| {
            (record.action.runtime_id == runtime_id
                && record.action.environment_id == environment_id)
                .then(|| compare_live_authority(&record.action, live))
                .flatten()
                .map(|reason| AuthorizationState::Invalidated {
                    invalidated_at_ms: now_ms,
                    reason,
                })
        })
    }

    pub fn expire_due(&mut self, now_ms: u64) -> usize {
        mutate_issued(&mut self.records, now_ms, |record| {
            (now_ms >= record.expires_at_ms).then_some(AuthorizationState::Expired {
                expired_at_ms: now_ms,
            })
        })
    }

    pub fn record(&self, authorization_id: &str) -> Option<&AuthorizationRecord> {
        self.records.get(authorization_id)
    }

    pub fn records(&self) -> impl Iterator<Item = &AuthorizationRecord> {
        self.records.values()
    }
}

fn mutate_issued(
    records: &mut BTreeMap<String, AuthorizationRecord>,
    _now_ms: u64,
    mut transition: impl FnMut(&AuthorizationRecord) -> Option<AuthorizationState>,
) -> usize {
    let mut count = 0;
    for record in records.values_mut() {
        if record.state == AuthorizationState::Issued
            && let Some(next) = transition(record)
        {
            record.state = next;
            count += 1;
        }
    }
    count
}

fn compare_action(
    expected: &CanonicalAction,
    actual: &CanonicalAction,
) -> Option<AuthorizationInvalidation> {
    if expected.schema_version != actual.schema_version
        || expected.action_id != actual.action_id
        || expected.tool_call_id != actual.tool_call_id
        || expected.tool != actual.tool
    {
        return Some(AuthorizationInvalidation::ActionIdentity);
    }
    if expected.arguments != actual.arguments {
        return Some(AuthorizationInvalidation::Arguments);
    }
    if expected.risk != actual.risk {
        return Some(AuthorizationInvalidation::Risk);
    }
    if expected.requested_authority != actual.requested_authority {
        return Some(AuthorizationInvalidation::RequestedAuthority);
    }
    if expected.canonical_target != actual.canonical_target {
        return Some(AuthorizationInvalidation::Target);
    }
    if expected.target_version != actual.target_version {
        return Some(AuthorizationInvalidation::TargetVersion);
    }
    if expected.workspace_id != actual.workspace_id {
        return Some(AuthorizationInvalidation::Workspace);
    }
    if expected.session_id != actual.session_id {
        return Some(AuthorizationInvalidation::Session);
    }
    if expected.run_id != actual.run_id {
        return Some(AuthorizationInvalidation::Run);
    }
    if expected.runtime_id != actual.runtime_id {
        return Some(AuthorizationInvalidation::Runtime);
    }
    if expected.environment_id != actual.environment_id {
        return Some(AuthorizationInvalidation::Environment);
    }
    if expected.plugin_or_mcp_id != actual.plugin_or_mcp_id {
        return Some(AuthorizationInvalidation::ActionIdentity);
    }
    if expected.process_generation != actual.process_generation {
        return Some(AuthorizationInvalidation::ProcessGeneration);
    }
    if expected.configuration_version != actual.configuration_version {
        return Some(AuthorizationInvalidation::ConfigurationVersion);
    }
    if expected.policy_version != actual.policy_version {
        return Some(AuthorizationInvalidation::PolicyVersion);
    }
    if expected.revocation_epoch != actual.revocation_epoch {
        return Some(AuthorizationInvalidation::RevocationEpoch);
    }
    None
}

fn compare_live_authority(
    action: &CanonicalAction,
    live: LiveAuthorityState,
) -> Option<AuthorizationInvalidation> {
    if action.process_generation != live.process_generation {
        Some(AuthorizationInvalidation::ProcessGeneration)
    } else if action.configuration_version != live.configuration_version {
        Some(AuthorizationInvalidation::ConfigurationVersion)
    } else if action.policy_version != live.policy_version {
        Some(AuthorizationInvalidation::PolicyVersion)
    } else if action.revocation_epoch != live.revocation_epoch {
        Some(AuthorizationInvalidation::RevocationEpoch)
    } else {
        None
    }
}

fn hash_token(token: &str) -> String {
    sha256_hex(token.as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(7 + digest.len() * 2);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalAnswer {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum ApprovalPromptState {
    Pending,
    Queued,
    Approved { answered_at_ms: u64 },
    Denied { answered_at_ms: u64 },
    Expired { expired_at_ms: u64 },
    Cancelled { cancelled_at_ms: u64 },
    Completed { completed_at_ms: u64 },
}

impl ApprovalPromptState {
    fn is_open(&self) -> bool {
        matches!(self, Self::Pending | Self::Queued)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalPromptRecord {
    pub prompt_id: String,
    pub sequence: u64,
    pub action: CanonicalAction,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub state: ApprovalPromptState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalQueueError {
    InvalidTtl,
    InvalidCanonicalAction(CanonicalActionError),
    DuplicatePrompt,
    NotFound,
    NotPending,
    NotApproved,
    Expired,
    DuplicateSequence,
    InvalidPersistedQueue,
    SequenceExhausted,
}

/// Approval prompts serialize within one run while each independent run keeps
/// its own ready prompt. `visible_queue` merges all runs by stable sequence so
/// the renderer can show independent-run queue state without granting authority.
#[derive(Default)]
pub struct ApprovalQueue {
    next_sequence: u64,
    prompts: BTreeMap<String, ApprovalPromptRecord>,
    run_queues: BTreeMap<String, VecDeque<String>>,
}

impl ApprovalQueue {
    /// Restores durable prompt records and reconstructs open per-run queues.
    /// Every run must have exactly one leading Pending prompt; later open
    /// prompts must be Queued in sequence order.
    pub fn restore(
        records: impl IntoIterator<Item = ApprovalPromptRecord>,
    ) -> Result<Self, ApprovalQueueError> {
        let mut prompts = BTreeMap::new();
        let mut sequences = BTreeSet::new();
        let mut max_sequence = None::<u64>;
        for record in records {
            record
                .action
                .validate()
                .map_err(ApprovalQueueError::InvalidCanonicalAction)?;
            if record.prompt_id.trim().is_empty() {
                return Err(ApprovalQueueError::NotFound);
            }
            if record.expires_at_ms <= record.created_at_ms {
                return Err(ApprovalQueueError::InvalidTtl);
            }
            if !sequences.insert(record.sequence) {
                return Err(ApprovalQueueError::DuplicateSequence);
            }
            max_sequence =
                Some(max_sequence.map_or(record.sequence, |max| max.max(record.sequence)));
            let id = record.prompt_id.clone();
            if prompts.insert(id, record).is_some() {
                return Err(ApprovalQueueError::DuplicatePrompt);
            }
        }

        let next_sequence = match max_sequence {
            Some(sequence) => sequence
                .checked_add(1)
                .ok_or(ApprovalQueueError::SequenceExhausted)?,
            None => 0,
        };
        let mut open_by_run: BTreeMap<String, Vec<(u64, String, ApprovalPromptState)>> =
            BTreeMap::new();
        for record in prompts.values().filter(|record| record.state.is_open()) {
            open_by_run
                .entry(record.action.run_id.clone())
                .or_default()
                .push((
                    record.sequence,
                    record.prompt_id.clone(),
                    record.state.clone(),
                ));
        }

        let mut run_queues = BTreeMap::new();
        for (run_id, mut open) in open_by_run {
            open.sort_by_key(|(sequence, _, _)| *sequence);
            let valid = open
                .first()
                .is_some_and(|(_, _, state)| state == &ApprovalPromptState::Pending)
                && open
                    .iter()
                    .skip(1)
                    .all(|(_, _, state)| state == &ApprovalPromptState::Queued);
            if !valid {
                return Err(ApprovalQueueError::InvalidPersistedQueue);
            }
            run_queues.insert(
                run_id,
                open.into_iter()
                    .map(|(_, prompt_id, _)| prompt_id)
                    .collect(),
            );
        }

        Ok(Self {
            next_sequence,
            prompts,
            run_queues,
        })
    }

    pub fn enqueue(
        &mut self,
        prompt_id: impl Into<String>,
        action: CanonicalAction,
        created_at_ms: u64,
        ttl_ms: u64,
    ) -> Result<&ApprovalPromptRecord, ApprovalQueueError> {
        action
            .validate()
            .map_err(ApprovalQueueError::InvalidCanonicalAction)?;
        if ttl_ms == 0 || created_at_ms.checked_add(ttl_ms).is_none() {
            return Err(ApprovalQueueError::InvalidTtl);
        }
        let prompt_id = prompt_id.into();
        if self.prompts.contains_key(&prompt_id) {
            return Err(ApprovalQueueError::DuplicatePrompt);
        }
        let queue = self.run_queues.entry(action.run_id.clone()).or_default();
        let state = if queue.is_empty() {
            ApprovalPromptState::Pending
        } else {
            ApprovalPromptState::Queued
        };
        let record = ApprovalPromptRecord {
            prompt_id: prompt_id.clone(),
            sequence: self.next_sequence,
            action,
            created_at_ms,
            expires_at_ms: created_at_ms + ttl_ms,
            state,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        queue.push_back(prompt_id.clone());
        self.prompts.insert(prompt_id.clone(), record);
        Ok(self
            .prompts
            .get(&prompt_id)
            .expect("inserted prompt exists"))
    }

    pub fn answer(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<&ApprovalPromptRecord, ApprovalQueueError> {
        let expired_run = {
            let record = self
                .prompts
                .get(prompt_id)
                .ok_or(ApprovalQueueError::NotFound)?;
            if record.state != ApprovalPromptState::Pending {
                return Err(ApprovalQueueError::NotPending);
            }
            (now_ms >= record.expires_at_ms).then(|| record.action.run_id.clone())
        };
        if let Some(run_id) = expired_run {
            self.prompts
                .get_mut(prompt_id)
                .expect("checked prompt exists")
                .state = ApprovalPromptState::Expired {
                expired_at_ms: now_ms,
            };
            self.remove_open_and_promote(&run_id, prompt_id);
            return Err(ApprovalQueueError::Expired);
        }

        let run_id = {
            let record = self
                .prompts
                .get_mut(prompt_id)
                .expect("checked prompt exists");
            record.state = match answer {
                ApprovalAnswer::Allow => ApprovalPromptState::Approved {
                    answered_at_ms: now_ms,
                },
                ApprovalAnswer::Deny => ApprovalPromptState::Denied {
                    answered_at_ms: now_ms,
                },
            };
            record.action.run_id.clone()
        };
        self.remove_open_and_promote(&run_id, prompt_id);
        Ok(self.prompts.get(prompt_id).expect("answered prompt exists"))
    }

    /// Expires a still-open prompt when its canonical target or relevant
    /// authority version no longer matches the live proposed action.
    pub fn revalidate_open_action(
        &mut self,
        prompt_id: &str,
        current_action: &CanonicalAction,
        now_ms: u64,
    ) -> Result<bool, ApprovalQueueError> {
        let run_id = {
            let record = self
                .prompts
                .get_mut(prompt_id)
                .ok_or(ApprovalQueueError::NotFound)?;
            if !record.state.is_open() {
                return Ok(false);
            }
            if compare_action(&record.action, current_action).is_none() {
                return Ok(false);
            }
            record.state = ApprovalPromptState::Expired {
                expired_at_ms: now_ms,
            };
            record.action.run_id.clone()
        };
        self.remove_open_and_promote(&run_id, prompt_id);
        Ok(true)
    }

    pub fn complete(
        &mut self,
        prompt_id: &str,
        now_ms: u64,
    ) -> Result<&ApprovalPromptRecord, ApprovalQueueError> {
        let record = self
            .prompts
            .get_mut(prompt_id)
            .ok_or(ApprovalQueueError::NotFound)?;
        if !matches!(record.state, ApprovalPromptState::Approved { .. }) {
            return Err(ApprovalQueueError::NotApproved);
        }
        record.state = ApprovalPromptState::Completed {
            completed_at_ms: now_ms,
        };
        Ok(record)
    }

    pub fn cancel_run(&mut self, run_id: &str, now_ms: u64) -> usize {
        let ids = self.run_queues.remove(run_id).unwrap_or_default();
        let mut count = 0;
        for id in ids {
            if let Some(record) = self.prompts.get_mut(&id)
                && record.state.is_open()
            {
                record.state = ApprovalPromptState::Cancelled {
                    cancelled_at_ms: now_ms,
                };
                count += 1;
            }
        }
        count
    }

    pub fn cancel_plugin(&mut self, plugin_id: &str, now_ms: u64) -> usize {
        let matches = self
            .prompts
            .values()
            .filter(|record| {
                record.state.is_open()
                    && record.action.plugin_or_mcp_id.as_deref() == Some(plugin_id)
            })
            .map(|record| (record.prompt_id.clone(), record.action.run_id.clone()))
            .collect::<Vec<_>>();
        let mut affected_runs = BTreeSet::new();
        for (prompt_id, run_id) in &matches {
            if let Some(record) = self.prompts.get_mut(prompt_id) {
                record.state = ApprovalPromptState::Cancelled {
                    cancelled_at_ms: now_ms,
                };
            }
            if let Some(queue) = self.run_queues.get_mut(run_id) {
                queue.retain(|queued_id| queued_id != prompt_id);
            }
            affected_runs.insert(run_id.clone());
        }
        for run_id in affected_runs {
            self.promote_front(&run_id);
        }
        matches.len()
    }

    pub fn expire_due(&mut self, now_ms: u64) -> usize {
        let due: Vec<(String, String)> = self
            .prompts
            .values()
            .filter(|record| record.state.is_open() && now_ms >= record.expires_at_ms)
            .map(|record| (record.prompt_id.clone(), record.action.run_id.clone()))
            .collect();
        let mut affected_runs = BTreeSet::new();
        for (id, run_id) in &due {
            if let Some(record) = self.prompts.get_mut(id) {
                record.state = ApprovalPromptState::Expired {
                    expired_at_ms: now_ms,
                };
            }
            if let Some(queue) = self.run_queues.get_mut(run_id) {
                queue.retain(|queued_id| queued_id != id);
            }
            affected_runs.insert(run_id.clone());
        }
        for run_id in affected_runs {
            self.promote_front(&run_id);
        }
        due.len()
    }

    pub fn visible_queue(&self) -> Vec<&ApprovalPromptRecord> {
        let mut visible: Vec<_> = self
            .prompts
            .values()
            .filter(|record| record.state.is_open())
            .collect();
        visible.sort_by_key(|record| record.sequence);
        visible
    }

    pub fn prompt(&self, prompt_id: &str) -> Option<&ApprovalPromptRecord> {
        self.prompts.get(prompt_id)
    }

    /// All durable prompt records, including terminal lifecycle states.
    pub fn prompts(&self) -> impl Iterator<Item = &ApprovalPromptRecord> {
        self.prompts.values()
    }

    fn remove_open_and_promote(&mut self, run_id: &str, prompt_id: &str) {
        if let Some(queue) = self.run_queues.get_mut(run_id) {
            queue.retain(|id| id != prompt_id);
        }
        self.promote_front(run_id);
    }

    fn promote_front(&mut self, run_id: &str) {
        let next = self
            .run_queues
            .get(run_id)
            .and_then(|queue| queue.front())
            .cloned();
        if let Some(next) = next
            && let Some(record) = self.prompts.get_mut(&next)
            && record.state == ApprovalPromptState::Queued
        {
            record.state = ApprovalPromptState::Pending;
        }
    }
}

/// Persistence seam owned by the integration layer. Implementations must make
/// the authorization/approval transition and corresponding redacted audit event
/// durable before a permit crosses into an execution environment.
pub trait AuthorizationRepository {
    type Error;

    fn save_authorization(&mut self, record: &AuthorizationRecord) -> Result<(), Self::Error>;
    fn save_approval(&mut self, record: &ApprovalPromptRecord) -> Result<(), Self::Error>;
}
