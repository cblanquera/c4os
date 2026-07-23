//! Rust-owned OpenCode broker processing and Action Gateway integration.
//!
//! The SDK worker can propose one of two effectless broker tools. It cannot
//! construct policy facts, canonical actions, authorizations, or successful
//! results. This state machine accepts frames only after the authenticated
//! broker transport validated them, binds them to an active Rust dispatch,
//! and retains a bounded approval pause until the sealed Action Gateway path
//! returns a receipt.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::mcp::McpTurnSnapshot;
use crate::runtime::action_bridge::{
    RuntimeActionEffectLease, RuntimeActionProposal, RuntimeApprovalDecision, RuntimeAuthorization,
    RuntimeEffectCompletionCertainty, RuntimeEffectResult, RuntimeExecutionReceipt,
    RuntimeGatewayDecision, RuntimeIntentIdentity,
};
use crate::runtime::dispatch::DispatchIdentity;
use crate::runtime::opencode::{
    C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL, OPENCODE_NATIVE_VERSION,
};
use crate::runtime::opencode_sdk::{
    BrokerDecision, BrokerEvent, BrokerProposal, OpenCodeSdkBroker, OpenCodeSdkError,
};
use crate::runtime::pi::PI_NATIVE_VERSION;
use crate::runtime::supervisor::RuntimeKind;
use crate::security::authorization::{
    ApprovalAnswer, CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk,
    LiveAuthorityState,
};
use crate::security::gateway::{ExecutionPermit, NormalizedActionResult, NormalizedActionStatus};
use crate::security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ClassificationConfidence, RepositoryState,
};

pub const DEFAULT_MAX_PENDING_BROKER_APPROVALS: usize = 256;
pub const DEFAULT_MAX_TERMINAL_BROKER_REQUESTS: usize = 4_096;

const MAX_TEXT_BYTES: usize = 512;
const MAX_CANONICAL_ARGUMENT_BYTES: usize = 256 * 1024;
const BROKER_ADAPTER_VERSION: &str = "1.0.0";

/// A broker event that crossed the inherited-descriptor authentication and
/// frame-validation boundary in [`crate::runtime::opencode_sdk`].
///
/// There is no constructor from a caller-supplied `BrokerEvent`, so renderer
/// commands and external callers cannot label arbitrary JSON as authenticated.
pub struct AuthenticatedBrokerEvent(BrokerEvent);

impl AuthenticatedBrokerEvent {
    /// Reads directly from the descriptor-authenticated broker. There is no
    /// public constructor from a caller-supplied `BrokerEvent`.
    pub fn receive(
        broker: &mut OpenCodeSdkBroker,
        timeout: Duration,
    ) -> Result<Self, OpenCodeSdkError> {
        broker.receive(timeout).map(Self)
    }

    /// Returns only descriptor-authenticated routing metadata. The broker
    /// payload remains opaque until a core context resolver has mapped this
    /// exact native session/message/process tuple to an active dispatch.
    pub fn metadata(&self) -> AuthenticatedBrokerMetadata {
        let (kind, proposal) = match &self.0 {
            BrokerEvent::Proposal(proposal) => (AuthenticatedBrokerEventKind::Proposal, proposal),
            BrokerEvent::Cancelled(proposal) => {
                (AuthenticatedBrokerEventKind::Cancellation, proposal)
            }
        };
        AuthenticatedBrokerMetadata {
            kind,
            correlation_id: proposal.correlation_id.clone(),
            tool: proposal.tool.clone(),
            native_session_id: proposal.session_id.clone(),
            native_message_id: proposal.message_id.clone(),
            process_generation: proposal.process_generation,
        }
    }

    fn into_inner(self) -> BrokerEvent {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthenticatedBrokerEventKind {
    Proposal,
    Cancellation,
}

/// Metadata materialized only from an event accepted by
/// `OpenCodeSdkBroker::receive`. Fields are private so arbitrary callers cannot
/// construct an authenticated routing identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedBrokerMetadata {
    kind: AuthenticatedBrokerEventKind,
    correlation_id: String,
    tool: String,
    native_session_id: String,
    native_message_id: String,
    process_generation: u64,
}

impl AuthenticatedBrokerMetadata {
    pub fn kind(&self) -> AuthenticatedBrokerEventKind {
        self.kind
    }

    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }

    pub fn tool(&self) -> &str {
        &self.tool
    }

    pub fn native_session_id(&self) -> &str {
        &self.native_session_id
    }

    pub fn native_message_id(&self) -> &str {
        &self.native_message_id
    }

    pub fn process_generation(&self) -> u64 {
        self.process_generation
    }
}

/// Rust-owned binding between one native OpenCode message and the active C4OS
/// attempt. None of these fields come from the broker proposal payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerActionContext {
    pub dispatch: DispatchIdentity,
    pub native_session_id: String,
    pub native_message_id: String,
    pub eligible_tool_ids: BTreeSet<String>,
    pub request_origin: ActionRequestOrigin,
    pub configuration_version: u64,
    pub policy_version: u64,
    pub revocation_epoch: u64,
    pub mcp_turn: Option<McpTurnSnapshot>,
}

impl BrokerActionContext {
    pub(crate) fn validate(&self) -> Result<(), BrokerWorkerError> {
        self.dispatch
            .validate()
            .map_err(|_| BrokerWorkerError::InvalidTrustedContext)?;
        let exact_runtime = match self.dispatch.runtime_kind {
            RuntimeKind::OpenCode => self.dispatch.native_version == OPENCODE_NATIVE_VERSION,
            RuntimeKind::Pi => self.dispatch.native_version == PI_NATIVE_VERSION,
        };
        if !exact_runtime
            || self.dispatch.adapter_version != BROKER_ADAPTER_VERSION
            || !safe_identifier(&self.native_session_id)
            || !safe_identifier(&self.native_message_id)
            || self.eligible_tool_ids.len() > 2
            || self.eligible_tool_ids.iter().any(|tool_id| {
                !matches!(
                    tool_id.as_str(),
                    C4OS_ACTION_PROPOSAL_TOOL | C4OS_RESOURCE_READ_TOOL
                )
            })
            || self.configuration_version == 0
            || self.policy_version == 0
            || self.request_origin == ActionRequestOrigin::Unknown
        {
            return Err(BrokerWorkerError::InvalidTrustedContext);
        }
        if let Some(mcp_turn) = &self.mcp_turn {
            mcp_turn
                .validate()
                .map_err(|_| BrokerWorkerError::InvalidTrustedContext)?;
        }
        Ok(())
    }

    fn live_authority(&self) -> LiveAuthorityState {
        LiveAuthorityState {
            process_generation: self.dispatch.process_generation,
            configuration_version: self.configuration_version,
            policy_version: self.policy_version,
            revocation_epoch: self.revocation_epoch,
        }
    }
}

/// Target and execution classification returned by a Rust core facility.
///
/// This is deliberately not an `ActionFacts` or `CanonicalAction`. The broker
/// worker derives both complete structures, including authority and risk, so a
/// facility cannot accidentally omit the original untrusted payload or runtime
/// identity from the exact Action Gateway binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedBrokerAction {
    pub surface: ActionSurface,
    pub effects: BTreeSet<ActionEffect>,
    pub scope: ActionScope,
    pub sensitivity: ActionSensitivity,
    pub reversibility: ActionReversibility,
    pub repository_state: RepositoryState,
    pub inside_active_project: bool,
    pub canonical_target: String,
    pub target_version: String,
    pub normalized_arguments: Map<String, Value>,
    pub trusted_root: bool,
    pub explicit_scope_grant: bool,
    pub sandbox_allows: bool,
    pub declaration_exceeded: bool,
    pub confidence: ClassificationConfidence,
    pub risk: CanonicalRisk,
    pub plugin_or_mcp_id: Option<String>,
}

impl ResolvedBrokerAction {
    fn validate(&self, tool: &str) -> Result<(), BrokerClassificationError> {
        if self.effects.is_empty()
            || matches!(self.surface, ActionSurface::Unknown(_))
            || self.effects.contains(&ActionEffect::Unknown)
            || self.scope == ActionScope::Unknown
            || self.sensitivity == ActionSensitivity::Unknown
            || self.reversibility == ActionReversibility::Unknown
            || self.repository_state == RepositoryState::Unknown
            || self.risk == CanonicalRisk::Unknown
            || self.canonical_target.trim().is_empty()
            || self.canonical_target.chars().any(char::is_control)
            || self.target_version.trim().is_empty()
            || self.target_version.chars().any(char::is_control)
            || serde_json::to_vec(&self.normalized_arguments)
                .map_or(true, |encoded| encoded.len() > MAX_CANONICAL_ARGUMENT_BYTES)
        {
            return Err(BrokerClassificationError::Ambiguous);
        }

        match tool {
            C4OS_RESOURCE_READ_TOOL
                if self.effects.iter().all(|effect| effect.is_read_only())
                    && self.sensitivity != ActionSensitivity::Credential
                    && self.surface != ActionSurface::Credential =>
            {
                Ok(())
            }
            C4OS_ACTION_PROPOSAL_TOOL
                if self.effects.iter().any(|effect| !effect.is_read_only()) =>
            {
                Ok(())
            }
            C4OS_RESOURCE_READ_TOOL | C4OS_ACTION_PROPOSAL_TOOL => {
                Err(BrokerClassificationError::Ambiguous)
            }
            _ => Err(BrokerClassificationError::Unsupported),
        }
    }
}

/// The only component allowed to translate a broker target hint into a live,
/// canonical C4OS resource or facility target.
pub trait BrokerActionClassifier {
    fn resolve_resource(
        &mut self,
        context: &BrokerActionContext,
        resource: &str,
        selector: Option<&str>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError>;

    fn resolve_action(
        &mut self,
        context: &BrokerActionContext,
        operation: &str,
        target: &str,
        arguments: &Map<String, Value>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum BrokerClassificationError {
    #[error("broker tool or operation is unsupported")]
    Unsupported,
    #[error("broker action classification is ambiguous")]
    Ambiguous,
    #[error("broker target could not be resolved")]
    Unresolved,
    #[error("broker target classification is unavailable")]
    Unavailable,
}

/// Small integration surface for `RuntimeApplicationService` (or the
/// coordinator it owns). The sealed authorization can only be passed back to
/// `execute_runtime_action`; workers receive an `ExecutionPermit` afterward.
pub trait BrokerActionApplication {
    type Error;

    fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, Self::Error>;

    fn answer_runtime_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<RuntimeApprovalDecision, Self::Error>;

    fn execute_runtime_action<F>(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        effect: F,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>
    where
        F: FnOnce(ExecutionPermit) -> NormalizedActionResult;

    fn begin_runtime_action_effect(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<RuntimeActionEffectLease, Self::Error>;

    fn complete_runtime_action_effect(
        &mut self,
        lease: RuntimeActionEffectLease,
        result: RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>;

    fn complete_runtime_action_effect_retryable(
        &mut self,
        lease: &mut RuntimeActionEffectLease,
        result: RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>;

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error>;
}

/// Effect implementation selected by the Rust core for the classified action.
/// This callback is unreachable until the Action Gateway consumes an exact,
/// single-use authorization.
pub trait BrokerEffectExecutor {
    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult;

    fn start_deferred(&mut self, _permit: ExecutionPermit) -> BrokerDeferredStart {
        BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
            "deferred-broker-facility-unavailable",
            1,
        ))
    }

    fn poll_deferred(&mut self, _ticket: &BrokerDeferredTicket) -> Option<RuntimeEffectResult> {
        None
    }

    fn cancel_deferred(&mut self, _ticket: &BrokerDeferredTicket) -> bool {
        false
    }

    fn abandon_deferred(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        self.cancel_deferred(ticket)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BrokerDeferredTicket(String);

impl BrokerDeferredTicket {
    /// Creates the opaque ticket returned by a core-installed deferred
    /// facility. The worker still validates and owns the ticket lifecycle.
    pub fn new(value: String) -> Result<Self, BrokerWorkerError> {
        if !safe_identifier(&value) {
            return Err(BrokerWorkerError::InvalidDeferredTicket);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub enum BrokerDeferredStart {
    Started(BrokerDeferredTicket),
    Rejected(NormalizedActionResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrokerWorkerOutcome {
    PendingApproval {
        correlation_id: String,
        prompt_id: String,
    },
    Respond {
        correlation_id: String,
        decision: BrokerDecision,
    },
    EffectRunning {
        correlation_id: String,
    },
    SettledAfterCancellation {
        correlation_id: String,
        decision: BrokerDecision,
    },
    /// `OpenCodeSdkBroker::receive` already returned the cancellation response
    /// before exposing this event; the decision is retained for core audit and
    /// must not be written to the descriptor a second time.
    ObservedCancellation {
        correlation_id: String,
        decision: BrokerDecision,
    },
}

/// Resolution retained for runtime transports, such as Pi, that must return
/// the exact sealed gateway receipt to their pending native tool call.
/// OpenCode continues to receive the deliberately reduced `BrokerDecision`
/// transport summary through [`BrokerWorkerOutcome`].
pub enum RuntimeBrokerWorkerOutcome {
    PendingApproval {
        native_request_id: String,
        prompt_id: String,
    },
    Denied {
        native_request_id: String,
        reason_code: String,
    },
    Executed {
        native_request_id: String,
        receipt: RuntimeExecutionReceipt,
    },
    EffectRunning {
        native_request_id: String,
    },
    SettledAfterCancellation {
        native_request_id: String,
        receipt: RuntimeExecutionReceipt,
    },
}

/// One approval continuation for a runtime intent already accepted from a
/// core-validated dispatch event. The current context must still match every
/// identity and authority version captured before the prompt was shown.
pub struct RuntimeBrokerApprovalAnswer<'a> {
    pub native_request_id: &'a str,
    pub prompt_id: &'a str,
    pub answer: ApprovalAnswer,
    pub current_context: &'a BrokerActionContext,
    pub now_ms: u64,
}

/// One core-owned approval continuation. Grouping the identity, answer, live
/// context, and clock makes it harder for an integration caller to substitute
/// only one authority-bearing field.
pub struct BrokerApprovalAnswer<'a> {
    pub correlation_id: &'a str,
    pub prompt_id: &'a str,
    pub answer: ApprovalAnswer,
    pub current_context: &'a BrokerActionContext,
    pub now_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingBrokerApproval {
    prompt_id: String,
    context: BrokerActionContext,
    proposal: BrokerProposal,
    intent: RuntimeIntentIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingRuntimeApproval {
    prompt_id: String,
    context: BrokerActionContext,
    intent: RuntimeIntentIdentity,
}

struct PendingBrokerEffect {
    ticket: BrokerDeferredTicket,
    lease: RuntimeActionEffectLease,
    intent: RuntimeIntentIdentity,
    context: BrokerActionContext,
    proposal: BrokerProposal,
    response_cancelled: bool,
}

struct PendingRuntimeEffect {
    ticket: BrokerDeferredTicket,
    lease: RuntimeActionEffectLease,
    intent: RuntimeIntentIdentity,
    run_terminal: bool,
}

pub struct BrokerActionWorker<C> {
    classifier: C,
    pending: BTreeMap<String, PendingBrokerApproval>,
    pending_runtime: BTreeMap<String, PendingRuntimeApproval>,
    pending_effects: BTreeMap<String, PendingBrokerEffect>,
    pending_runtime_effects: BTreeMap<String, PendingRuntimeEffect>,
    terminal: BTreeSet<String>,
    max_pending: usize,
    max_terminal: usize,
    sealed_at_capacity: bool,
}

impl<C: BrokerActionClassifier> BrokerActionWorker<C> {
    pub fn new(classifier: C) -> Self {
        Self::with_capacity(
            classifier,
            DEFAULT_MAX_PENDING_BROKER_APPROVALS,
            DEFAULT_MAX_TERMINAL_BROKER_REQUESTS,
        )
        .expect("default broker worker capacities are valid")
    }

    pub fn with_capacity(
        classifier: C,
        max_pending: usize,
        max_terminal: usize,
    ) -> Result<Self, BrokerWorkerError> {
        if max_pending == 0 || max_terminal < max_pending {
            return Err(BrokerWorkerError::InvalidConfiguration);
        }
        Ok(Self {
            classifier,
            pending: BTreeMap::new(),
            pending_runtime: BTreeMap::new(),
            pending_effects: BTreeMap::new(),
            pending_runtime_effects: BTreeMap::new(),
            terminal: BTreeSet::new(),
            max_pending,
            max_terminal,
            sealed_at_capacity: false,
        })
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
            + self.pending_runtime.len()
            + self.pending_effects.len()
            + self.pending_runtime_effects.len()
    }

    pub(crate) fn pending_runtime_approval_prompt(&self, native_request_id: &str) -> Option<&str> {
        self.pending_runtime
            .get(native_request_id)
            .map(|pending| pending.prompt_id.as_str())
    }

    /// Accepts one action intent that was already authenticated by a
    /// core-owned runtime dispatch peer. The supplied binding digest remains
    /// the exact Pi event digest; it is never replaced with caller metadata.
    pub fn accept_runtime_intent<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        intent: RuntimeIntentIdentity,
        payload: Value,
        context: BrokerActionContext,
        now_ms: u64,
    ) -> Result<RuntimeBrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        context.validate()?;
        let native_request_id = intent.native_request_id.clone();
        if self.pending_runtime.contains_key(&native_request_id)
            || self
                .pending_runtime_effects
                .contains_key(&native_request_id)
            || self.terminal.contains(&native_request_id)
        {
            return Ok(runtime_denied(
                &native_request_id,
                "broker-request-replayed",
            ));
        }
        if self.sealed_at_capacity || self.terminal.len() >= self.max_terminal {
            self.sealed_at_capacity = true;
            return Ok(runtime_denied(&native_request_id, "broker-worker-capacity"));
        }
        if self.pending_count() >= self.max_pending {
            self.record_terminal(&native_request_id);
            return Ok(runtime_denied(
                &native_request_id,
                "pending-approval-capacity",
            ));
        }
        if validate_runtime_intent_binding(&intent, &context).is_err() {
            self.record_terminal(&native_request_id);
            return Ok(runtime_denied(&native_request_id, "stale-runtime-identity"));
        }

        let runtime_proposal = match self.classify_runtime_intent(&intent, payload, &context) {
            Ok(proposal) => proposal,
            Err(error) => {
                self.record_terminal(&native_request_id);
                return Ok(runtime_denied(&native_request_id, error.reason_code()));
            }
        };
        let decision = match application.propose_runtime_action(runtime_proposal, now_ms) {
            Ok(decision) => decision,
            Err(_) => {
                self.record_terminal(&native_request_id);
                return Ok(runtime_denied(
                    &native_request_id,
                    "action-gateway-unavailable",
                ));
            }
        };
        match decision {
            RuntimeGatewayDecision::Denied { .. } => {
                self.record_terminal(&native_request_id);
                Ok(runtime_denied(&native_request_id, "policy-denied"))
            }
            RuntimeGatewayDecision::PendingApproval { prompt_id, .. } => {
                self.pending_runtime.insert(
                    native_request_id.clone(),
                    PendingRuntimeApproval {
                        prompt_id: prompt_id.clone(),
                        context,
                        intent,
                    },
                );
                Ok(RuntimeBrokerWorkerOutcome::PendingApproval {
                    native_request_id,
                    prompt_id,
                })
            }
            RuntimeGatewayDecision::Authorized(authorization) => self.execute_runtime_authorized(
                application,
                executor,
                intent,
                *authorization,
                &context,
                now_ms,
            ),
        }
    }

    /// Resumes one exact Pi approval. A changed dispatch or live authority
    /// burns the prompt and returns a denial that the production worker can
    /// immediately deliver to the blocked sidecar tool call.
    pub fn answer_runtime_intent_approval<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        request: RuntimeBrokerApprovalAnswer<'_>,
    ) -> Result<RuntimeBrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let RuntimeBrokerApprovalAnswer {
            native_request_id,
            prompt_id,
            answer,
            current_context,
            now_ms,
        } = request;
        current_context.validate()?;
        if self.terminal.contains(native_request_id) {
            return Ok(runtime_denied(native_request_id, "broker-request-replayed"));
        }
        let pending = self
            .pending_runtime
            .get(native_request_id)
            .cloned()
            .ok_or(BrokerWorkerError::UnknownApproval)?;
        if pending.prompt_id != prompt_id {
            return Err(BrokerWorkerError::ApprovalBindingMismatch);
        }
        if pending.context != *current_context {
            let cancellation =
                application.cancel_runtime_run(&pending.context.dispatch.attempt_id, now_ms);
            self.pending_runtime.remove(native_request_id);
            self.record_terminal(native_request_id);
            cancellation.map_err(|_| BrokerWorkerError::Application)?;
            return Ok(runtime_denied(native_request_id, "stale-runtime-identity"));
        }

        let decision = application
            .answer_runtime_approval(prompt_id, answer, now_ms)
            .map_err(|_| BrokerWorkerError::Application)?;
        match decision {
            RuntimeApprovalDecision::Denied => {
                self.pending_runtime.remove(native_request_id);
                self.record_terminal(native_request_id);
                Ok(runtime_denied(native_request_id, "user-denied"))
            }
            RuntimeApprovalDecision::Authorized(authorization) => {
                self.pending_runtime.remove(native_request_id);
                self.execute_runtime_authorized(
                    application,
                    executor,
                    pending.intent,
                    *authorization,
                    &pending.context,
                    now_ms,
                )
            }
        }
    }

    pub fn accept_authenticated_event<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        event: AuthenticatedBrokerEvent,
        context: BrokerActionContext,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        match event.into_inner() {
            BrokerEvent::Proposal(proposal) => {
                context.validate()?;
                self.accept_proposal(application, executor, proposal, context, now_ms)
            }
            BrokerEvent::Cancelled(proposal) => {
                self.accept_cancellation(application, executor, proposal, now_ms)
            }
        }
    }

    pub fn answer_approval<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        request: BrokerApprovalAnswer<'_>,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let BrokerApprovalAnswer {
            correlation_id,
            prompt_id,
            answer,
            current_context,
            now_ms,
        } = request;
        current_context.validate()?;
        if self.terminal.contains(correlation_id) {
            return Ok(respond_denied(correlation_id, "broker-request-replayed"));
        }
        let pending = self
            .pending
            .get(correlation_id)
            .cloned()
            .ok_or(BrokerWorkerError::UnknownApproval)?;
        if pending.prompt_id != prompt_id {
            return Err(BrokerWorkerError::ApprovalBindingMismatch);
        }
        if pending.context != *current_context {
            let cancellation =
                application.cancel_runtime_run(&pending.context.dispatch.attempt_id, now_ms);
            self.pending.remove(correlation_id);
            self.record_terminal(correlation_id);
            cancellation.map_err(|_| BrokerWorkerError::Application)?;
            return Ok(respond_denied(correlation_id, "stale-runtime-identity"));
        }

        let decision = application
            .answer_runtime_approval(prompt_id, answer, now_ms)
            .map_err(|_| BrokerWorkerError::Application)?;
        match decision {
            RuntimeApprovalDecision::Denied => {
                self.pending.remove(correlation_id);
                self.record_terminal(correlation_id);
                Ok(respond_denied(correlation_id, "user-denied"))
            }
            RuntimeApprovalDecision::Authorized(authorization) => {
                self.pending.remove(correlation_id);
                self.execute_authorized(
                    application,
                    executor,
                    pending.intent,
                    *authorization,
                    &pending.context,
                    Some(pending.proposal),
                    now_ms,
                )
            }
        }
    }

    /// Burns a pending approval when the core can no longer resolve its
    /// authenticated native metadata to the same active dispatch/live state.
    /// This path never receives replacement action facts or an authorization.
    pub fn invalidate_pending_approval<A>(
        &mut self,
        application: &mut A,
        correlation_id: &str,
        prompt_id: &str,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
    {
        if self.terminal.contains(correlation_id) {
            return Ok(respond_denied(correlation_id, "broker-request-replayed"));
        }
        let pending = self
            .pending
            .get(correlation_id)
            .cloned()
            .ok_or(BrokerWorkerError::UnknownApproval)?;
        if pending.prompt_id != prompt_id {
            return Err(BrokerWorkerError::ApprovalBindingMismatch);
        }
        let cancellation =
            application.cancel_runtime_run(&pending.context.dispatch.attempt_id, now_ms);
        self.pending.remove(correlation_id);
        self.record_terminal(correlation_id);
        cancellation.map_err(|_| BrokerWorkerError::Application)?;
        Ok(respond_denied(correlation_id, "stale-runtime-identity"))
    }

    /// Processes an authenticated descriptor cancellation even if the active
    /// dispatch mapping was concurrently retired. The broker transport already
    /// wrote the cancellation result, and this method can only burn pending
    /// core state; it cannot authorize or execute an effect.
    pub fn accept_authenticated_cancellation<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        event: AuthenticatedBrokerEvent,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        match event.into_inner() {
            BrokerEvent::Cancelled(proposal) => {
                self.accept_cancellation(application, executor, proposal, now_ms)
            }
            BrokerEvent::Proposal(_) => Err(BrokerWorkerError::ExpectedCancellation),
        }
    }

    fn accept_proposal<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        proposal: BrokerProposal,
        context: BrokerActionContext,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        if self.pending.contains_key(&proposal.correlation_id)
            || self.pending_effects.contains_key(&proposal.correlation_id)
            || self.terminal.contains(&proposal.correlation_id)
        {
            return Ok(respond_denied(
                &proposal.correlation_id,
                "broker-request-replayed",
            ));
        }
        if self.sealed_at_capacity || self.terminal.len() >= self.max_terminal {
            self.sealed_at_capacity = true;
            return Ok(respond_denied(
                &proposal.correlation_id,
                "broker-worker-capacity",
            ));
        }
        if let Err(error) = validate_envelope_binding(&proposal, &context) {
            self.record_terminal(&proposal.correlation_id);
            return Ok(respond_denied(
                &proposal.correlation_id,
                error.reason_code(),
            ));
        }
        if self.pending_count() >= self.max_pending {
            self.record_terminal(&proposal.correlation_id);
            return Ok(respond_denied(
                &proposal.correlation_id,
                "pending-approval-capacity",
            ));
        }

        let runtime_proposal = match self.classify_and_bind(&proposal, &context) {
            Ok(proposal) => proposal,
            Err(error) => {
                self.record_terminal(&proposal.correlation_id);
                return Ok(respond_denied(
                    &proposal.correlation_id,
                    error.reason_code(),
                ));
            }
        };
        let intent = runtime_proposal.intent.clone();
        let decision = match application.propose_runtime_action(runtime_proposal, now_ms) {
            Ok(decision) => decision,
            Err(_) => {
                self.record_terminal(&proposal.correlation_id);
                return Ok(respond_denied(
                    &proposal.correlation_id,
                    "action-gateway-unavailable",
                ));
            }
        };

        match decision {
            RuntimeGatewayDecision::Denied { .. } => {
                self.record_terminal(&proposal.correlation_id);
                Ok(respond_denied(&proposal.correlation_id, "policy-denied"))
            }
            RuntimeGatewayDecision::PendingApproval { prompt_id, .. } => {
                let correlation_id = proposal.correlation_id.clone();
                self.pending.insert(
                    correlation_id.clone(),
                    PendingBrokerApproval {
                        prompt_id: prompt_id.clone(),
                        context,
                        proposal,
                        intent,
                    },
                );
                Ok(BrokerWorkerOutcome::PendingApproval {
                    correlation_id,
                    prompt_id,
                })
            }
            RuntimeGatewayDecision::Authorized(authorization) => self.execute_authorized(
                application,
                executor,
                intent,
                *authorization,
                &context,
                Some(proposal),
                now_ms,
            ),
        }
    }

    fn accept_cancellation<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        proposal: BrokerProposal,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        if let Some(pending) = self.pending_effects.get_mut(&proposal.correlation_id) {
            if pending.proposal.tool != proposal.tool
                || pending.proposal.session_id != proposal.session_id
                || pending.proposal.message_id != proposal.message_id
                || pending.proposal.process_generation != proposal.process_generation
            {
                return Ok(BrokerWorkerOutcome::ObservedCancellation {
                    correlation_id: proposal.correlation_id,
                    decision: denied("stale-runtime-identity"),
                });
            }
            pending.response_cancelled = true;
            let _ = executor.cancel_deferred(&pending.ticket);
            let cancellation =
                application.cancel_runtime_run(&pending.context.dispatch.attempt_id, now_ms);
            cancellation.map_err(|_| BrokerWorkerError::Application)?;
            return Ok(BrokerWorkerOutcome::ObservedCancellation {
                correlation_id: proposal.correlation_id,
                decision: BrokerDecision::Cancelled,
            });
        }
        let Some(pending) = self.pending.get(&proposal.correlation_id).cloned() else {
            let decision = if self.terminal.contains(&proposal.correlation_id) {
                denied("broker-request-replayed")
            } else {
                denied("unknown-broker-request")
            };
            return Ok(BrokerWorkerOutcome::ObservedCancellation {
                correlation_id: proposal.correlation_id,
                decision,
            });
        };
        if pending.proposal.tool != proposal.tool
            || pending.proposal.session_id != proposal.session_id
            || pending.proposal.message_id != proposal.message_id
            || pending.proposal.process_generation != proposal.process_generation
        {
            return Ok(BrokerWorkerOutcome::ObservedCancellation {
                correlation_id: proposal.correlation_id,
                decision: denied("stale-runtime-identity"),
            });
        }

        let cancellation =
            application.cancel_runtime_run(&pending.context.dispatch.attempt_id, now_ms);
        self.pending.remove(&proposal.correlation_id);
        self.record_terminal(&proposal.correlation_id);
        cancellation.map_err(|_| BrokerWorkerError::Application)?;
        Ok(BrokerWorkerOutcome::ObservedCancellation {
            correlation_id: proposal.correlation_id,
            decision: BrokerDecision::Cancelled,
        })
    }

    fn classify_runtime_intent(
        &mut self,
        intent: &RuntimeIntentIdentity,
        payload: Value,
        context: &BrokerActionContext,
    ) -> Result<RuntimeActionProposal, ProposalRejection> {
        if !context.eligible_tool_ids.contains(&intent.native_tool) {
            return Err(ProposalRejection::UnsupportedTool);
        }
        let (action_kind, broker_arguments, resolved) = match intent.native_tool.as_str() {
            C4OS_RESOURCE_READ_TOOL => {
                let request: ReadResourceRequest = serde_json::from_value(payload)
                    .map_err(|_| ProposalRejection::InvalidPayload)?;
                request.validate()?;
                let resolved = self
                    .classifier
                    .resolve_resource(context, &request.resource, request.selector.as_deref())
                    .map_err(ProposalRejection::Classification)?;
                (
                    "resource.read".to_owned(),
                    json!({
                        "resource": request.resource,
                        "selector": request.selector,
                    }),
                    resolved,
                )
            }
            C4OS_ACTION_PROPOSAL_TOOL => {
                let request: ProposeActionRequest = serde_json::from_value(payload)
                    .map_err(|_| ProposalRejection::InvalidPayload)?;
                request.validate()?;
                let resolved = self
                    .classifier
                    .resolve_action(
                        context,
                        &request.operation,
                        &request.target,
                        &request.arguments,
                    )
                    .map_err(ProposalRejection::Classification)?;
                (
                    request.operation.clone(),
                    json!({
                        "operation": request.operation,
                        "target": request.target,
                        "arguments": request.arguments,
                    }),
                    resolved,
                )
            }
            _ => return Err(ProposalRejection::UnsupportedTool),
        };
        resolved
            .validate(&intent.native_tool)
            .map_err(ProposalRejection::Classification)?;

        let arguments = json!({
            "broker": broker_arguments,
            "resolved": resolved.normalized_arguments,
        });
        let facts = ActionFacts {
            action_kind: action_kind.clone(),
            native_tool: intent.native_tool.clone(),
            surface: resolved.surface.clone(),
            effects: resolved.effects.clone(),
            scope: resolved.scope,
            initiator: ActionInitiator::Runtime,
            sensitivity: resolved.sensitivity,
            reversibility: resolved.reversibility,
            confidence: resolved.confidence,
            request_origin: context.request_origin,
            repository_state: resolved.repository_state,
            inside_active_project: resolved.inside_active_project,
            canonical_target: resolved.canonical_target.clone(),
            workspace_id: context.dispatch.workspace_id.clone(),
            session_id: context.dispatch.session_id.clone(),
            runtime_id: context.dispatch.runtime_id.clone(),
            environment_id: context.dispatch.environment_id.clone(),
            plugin_or_mcp_id: resolved.plugin_or_mcp_id.clone(),
            target_resolved: true,
            authenticated: true,
            trusted_root: resolved.trusted_root,
            explicit_scope_grant: resolved.explicit_scope_grant,
            sandbox_allows: resolved.sandbox_allows,
            declaration_exceeded: resolved.declaration_exceeded,
        };
        let action = CanonicalAction {
            schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
            action_id: action_id(&intent.binding_sha256),
            tool_call_id: intent.native_request_id.clone(),
            tool: intent.native_tool.clone(),
            arguments,
            risk: resolved.risk,
            requested_authority: requested_authority(&resolved),
            canonical_target: resolved.canonical_target,
            target_version: resolved.target_version,
            workspace_id: context.dispatch.workspace_id.clone(),
            session_id: context.dispatch.session_id.clone(),
            run_id: context.dispatch.attempt_id.clone(),
            runtime_id: context.dispatch.runtime_id.clone(),
            environment_id: context.dispatch.environment_id.clone(),
            plugin_or_mcp_id: resolved.plugin_or_mcp_id.clone(),
            process_generation: context.dispatch.process_generation,
            configuration_version: context.configuration_version,
            policy_version: context.policy_version,
            revocation_epoch: context.revocation_epoch,
        };
        RuntimeActionProposal::new(intent.clone(), facts, action)
            .map_err(|_| ProposalRejection::Binding)
    }

    fn classify_and_bind(
        &mut self,
        proposal: &BrokerProposal,
        context: &BrokerActionContext,
    ) -> Result<RuntimeActionProposal, ProposalRejection> {
        if !context.eligible_tool_ids.contains(&proposal.tool) {
            return Err(ProposalRejection::UnsupportedTool);
        }
        let (action_kind, broker_arguments, resolved) = match proposal.tool.as_str() {
            C4OS_RESOURCE_READ_TOOL => {
                let request: ReadResourceRequest = serde_json::from_value(proposal.payload.clone())
                    .map_err(|_| ProposalRejection::InvalidPayload)?;
                request.validate()?;
                let resolved = self
                    .classifier
                    .resolve_resource(context, &request.resource, request.selector.as_deref())
                    .map_err(ProposalRejection::Classification)?;
                (
                    "resource.read".to_owned(),
                    json!({
                        "resource": request.resource,
                        "selector": request.selector,
                    }),
                    resolved,
                )
            }
            C4OS_ACTION_PROPOSAL_TOOL => {
                let request: ProposeActionRequest =
                    serde_json::from_value(proposal.payload.clone())
                        .map_err(|_| ProposalRejection::InvalidPayload)?;
                request.validate()?;
                let resolved = self
                    .classifier
                    .resolve_action(
                        context,
                        &request.operation,
                        &request.target,
                        &request.arguments,
                    )
                    .map_err(ProposalRejection::Classification)?;
                (
                    request.operation.clone(),
                    json!({
                        "operation": request.operation,
                        "target": request.target,
                        "arguments": request.arguments,
                    }),
                    resolved,
                )
            }
            _ => return Err(ProposalRejection::UnsupportedTool),
        };
        resolved
            .validate(&proposal.tool)
            .map_err(ProposalRejection::Classification)?;

        let binding_sha256 = proposal_binding_sha256(proposal, context, &broker_arguments)?;
        let intent = RuntimeIntentIdentity {
            binding_sha256,
            workspace_id: context.dispatch.workspace_id.clone(),
            session_id: context.dispatch.session_id.clone(),
            turn_id: context.dispatch.turn_id.clone(),
            run_id: context.dispatch.attempt_id.clone(),
            correlation_id: context.dispatch.correlation_id.clone(),
            runtime_id: context.dispatch.runtime_id.clone(),
            process_generation: context.dispatch.process_generation,
            native_request_id: proposal.correlation_id.clone(),
            native_tool: proposal.tool.clone(),
        };
        let arguments = json!({
            "broker": broker_arguments,
            "resolved": resolved.normalized_arguments,
        });
        let facts = ActionFacts {
            action_kind: action_kind.clone(),
            native_tool: proposal.tool.clone(),
            surface: resolved.surface.clone(),
            effects: resolved.effects.clone(),
            scope: resolved.scope,
            initiator: ActionInitiator::Runtime,
            sensitivity: resolved.sensitivity,
            reversibility: resolved.reversibility,
            confidence: resolved.confidence,
            request_origin: context.request_origin,
            repository_state: resolved.repository_state,
            inside_active_project: resolved.inside_active_project,
            canonical_target: resolved.canonical_target.clone(),
            workspace_id: context.dispatch.workspace_id.clone(),
            session_id: context.dispatch.session_id.clone(),
            runtime_id: context.dispatch.runtime_id.clone(),
            environment_id: context.dispatch.environment_id.clone(),
            plugin_or_mcp_id: resolved.plugin_or_mcp_id.clone(),
            target_resolved: true,
            authenticated: true,
            trusted_root: resolved.trusted_root,
            explicit_scope_grant: resolved.explicit_scope_grant,
            sandbox_allows: resolved.sandbox_allows,
            declaration_exceeded: resolved.declaration_exceeded,
        };
        let action = CanonicalAction {
            schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
            action_id: action_id(&intent.binding_sha256),
            tool_call_id: proposal.correlation_id.clone(),
            tool: proposal.tool.clone(),
            arguments,
            risk: resolved.risk,
            requested_authority: requested_authority(&resolved),
            canonical_target: resolved.canonical_target,
            target_version: resolved.target_version,
            workspace_id: context.dispatch.workspace_id.clone(),
            session_id: context.dispatch.session_id.clone(),
            run_id: context.dispatch.attempt_id.clone(),
            runtime_id: context.dispatch.runtime_id.clone(),
            environment_id: context.dispatch.environment_id.clone(),
            plugin_or_mcp_id: resolved.plugin_or_mcp_id.clone(),
            process_generation: context.dispatch.process_generation,
            configuration_version: context.configuration_version,
            policy_version: context.policy_version,
            revocation_epoch: context.revocation_epoch,
        };
        let runtime_proposal = RuntimeActionProposal::new(intent, facts, action)
            .map_err(|_| ProposalRejection::Binding)?;
        validate_complete_binding(&runtime_proposal, context, proposal)
            .map_err(|_| ProposalRejection::Binding)?;
        Ok(runtime_proposal)
    }

    fn execute_authorized<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        intent: RuntimeIntentIdentity,
        authorization: RuntimeAuthorization,
        context: &BrokerActionContext,
        proposal: Option<BrokerProposal>,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        if authorization.is_mcp() {
            return self.start_deferred_authorized(
                application,
                executor,
                intent,
                authorization,
                context,
                proposal.ok_or(BrokerWorkerError::ReceiptBindingMismatch)?,
                now_ms,
            );
        }
        let correlation_id = intent.native_request_id.clone();
        let receipt = application
            .execute_runtime_action(authorization, context.live_authority(), now_ms, |permit| {
                executor.execute(permit)
            })
            .map_err(|_| {
                self.record_terminal(&correlation_id);
                BrokerWorkerError::EffectStatusUnknown
            })?;
        if !receipt.matches_runtime_tool(
            &intent.runtime_id,
            &intent.session_id,
            &intent.run_id,
            &intent.native_request_id,
            intent.process_generation,
        ) {
            self.record_terminal(&correlation_id);
            return Err(BrokerWorkerError::ReceiptBindingMismatch);
        }
        let decision = decision_from_receipt(&receipt);
        self.record_terminal(&correlation_id);
        Ok(BrokerWorkerOutcome::Respond {
            correlation_id,
            decision,
        })
    }

    // The deferred lease keeps every exact authority binding explicit.
    #[allow(clippy::too_many_arguments)]
    fn start_deferred_authorized<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        intent: RuntimeIntentIdentity,
        authorization: RuntimeAuthorization,
        context: &BrokerActionContext,
        proposal: BrokerProposal,
        now_ms: u64,
    ) -> Result<BrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let correlation_id = intent.native_request_id.clone();
        if self.pending_count() >= self.max_pending {
            self.record_terminal(&correlation_id);
            return Ok(BrokerWorkerOutcome::Respond {
                correlation_id,
                decision: denied("pending-effect-capacity"),
            });
        }
        let mut lease = application
            .begin_runtime_action_effect(authorization, context.live_authority(), now_ms)
            .map_err(|_| BrokerWorkerError::Application)?;
        let permit = lease
            .take_execution_permit()
            .map_err(|_| BrokerWorkerError::EffectStatusUnknown)?;
        match executor.start_deferred(permit) {
            BrokerDeferredStart::Started(ticket) => {
                self.pending_effects.insert(
                    correlation_id.clone(),
                    PendingBrokerEffect {
                        ticket,
                        lease,
                        intent,
                        context: context.clone(),
                        proposal,
                        response_cancelled: false,
                    },
                );
                Ok(BrokerWorkerOutcome::EffectRunning { correlation_id })
            }
            BrokerDeferredStart::Rejected(normalized) => {
                let receipt = application
                    .complete_runtime_action_effect(
                        lease,
                        RuntimeEffectResult::normalized(normalized),
                        now_ms.saturating_add(1),
                    )
                    .map_err(|_| BrokerWorkerError::EffectStatusUnknown)?;
                if !receipt.matches_runtime_tool(
                    &intent.runtime_id,
                    &intent.session_id,
                    &intent.run_id,
                    &intent.native_request_id,
                    intent.process_generation,
                ) {
                    return Err(BrokerWorkerError::ReceiptBindingMismatch);
                }
                let decision = decision_from_receipt(&receipt);
                self.record_terminal(&correlation_id);
                Ok(BrokerWorkerOutcome::Respond {
                    correlation_id,
                    decision,
                })
            }
        }
    }

    pub fn poll_deferred<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        now_ms: u64,
    ) -> Result<Option<BrokerWorkerOutcome>, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let Some(correlation_id) = self.pending_effects.keys().next().cloned() else {
            return Ok(None);
        };
        let Some(result) = self
            .pending_effects
            .get(&correlation_id)
            .and_then(|pending| executor.poll_deferred(&pending.ticket))
        else {
            return Ok(None);
        };
        let pending = self
            .pending_effects
            .remove(&correlation_id)
            .ok_or(BrokerWorkerError::EffectStatusUnknown)?;
        let receipt = application
            .complete_runtime_action_effect(pending.lease, result, now_ms)
            .map_err(|_| BrokerWorkerError::EffectStatusUnknown)?;
        if !receipt.matches_runtime_tool(
            &pending.intent.runtime_id,
            &pending.intent.session_id,
            &pending.intent.run_id,
            &pending.intent.native_request_id,
            pending.intent.process_generation,
        ) {
            return Err(BrokerWorkerError::ReceiptBindingMismatch);
        }
        let decision = decision_from_receipt(&receipt);
        self.record_terminal(&correlation_id);
        Ok(Some(if pending.response_cancelled {
            BrokerWorkerOutcome::SettledAfterCancellation {
                correlation_id,
                decision,
            }
        } else {
            BrokerWorkerOutcome::Respond {
                correlation_id,
                decision,
            }
        }))
    }

    fn execute_runtime_authorized<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        intent: RuntimeIntentIdentity,
        authorization: RuntimeAuthorization,
        context: &BrokerActionContext,
        now_ms: u64,
    ) -> Result<RuntimeBrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        if authorization.is_mcp() {
            return self.start_runtime_deferred_authorized(
                application,
                executor,
                intent,
                authorization,
                context,
                now_ms,
            );
        }
        let native_request_id = intent.native_request_id.clone();
        let receipt = application
            .execute_runtime_action(authorization, context.live_authority(), now_ms, |permit| {
                executor.execute(permit)
            })
            .map_err(|_| {
                self.record_terminal(&native_request_id);
                BrokerWorkerError::EffectStatusUnknown
            })?;
        if !receipt.matches_runtime_tool(
            &intent.runtime_id,
            &intent.session_id,
            &intent.run_id,
            &intent.native_request_id,
            intent.process_generation,
        ) {
            self.record_terminal(&native_request_id);
            return Err(BrokerWorkerError::ReceiptBindingMismatch);
        }
        self.record_terminal(&native_request_id);
        Ok(RuntimeBrokerWorkerOutcome::Executed {
            native_request_id,
            receipt,
        })
    }

    fn start_runtime_deferred_authorized<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        intent: RuntimeIntentIdentity,
        authorization: RuntimeAuthorization,
        context: &BrokerActionContext,
        now_ms: u64,
    ) -> Result<RuntimeBrokerWorkerOutcome, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let native_request_id = intent.native_request_id.clone();
        if self.pending_count() >= self.max_pending {
            self.record_terminal(&native_request_id);
            return Ok(runtime_denied(
                &native_request_id,
                "pending-effect-capacity",
            ));
        }
        let mut lease = application
            .begin_runtime_action_effect(authorization, context.live_authority(), now_ms)
            .map_err(|_| BrokerWorkerError::Application)?;
        let permit = lease
            .take_execution_permit()
            .map_err(|_| BrokerWorkerError::EffectStatusUnknown)?;
        match executor.start_deferred(permit) {
            BrokerDeferredStart::Started(ticket) => {
                self.pending_runtime_effects.insert(
                    native_request_id.clone(),
                    PendingRuntimeEffect {
                        ticket,
                        lease,
                        intent,
                        run_terminal: false,
                    },
                );
                Ok(RuntimeBrokerWorkerOutcome::EffectRunning { native_request_id })
            }
            BrokerDeferredStart::Rejected(normalized) => {
                let receipt = application
                    .complete_runtime_action_effect(
                        lease,
                        RuntimeEffectResult::normalized(normalized),
                        now_ms.saturating_add(1),
                    )
                    .map_err(|_| BrokerWorkerError::EffectStatusUnknown)?;
                Ok(RuntimeBrokerWorkerOutcome::Executed {
                    native_request_id,
                    receipt,
                })
            }
        }
    }

    pub fn poll_runtime_deferred<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        now_ms: u64,
    ) -> Result<Option<RuntimeBrokerWorkerOutcome>, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let Some(native_request_id) = self.pending_runtime_effects.keys().next().cloned() else {
            return Ok(None);
        };
        let Some(result) = self
            .pending_runtime_effects
            .get(&native_request_id)
            .and_then(|pending| executor.poll_deferred(&pending.ticket))
        else {
            return Ok(None);
        };
        let pending = self
            .pending_runtime_effects
            .remove(&native_request_id)
            .ok_or(BrokerWorkerError::EffectStatusUnknown)?;
        let receipt = application
            .complete_runtime_action_effect(pending.lease, result, now_ms)
            .map_err(|_| BrokerWorkerError::EffectStatusUnknown)?;
        if !receipt.matches_runtime_tool(
            &pending.intent.runtime_id,
            &pending.intent.session_id,
            &pending.intent.run_id,
            &pending.intent.native_request_id,
            pending.intent.process_generation,
        ) {
            return Err(BrokerWorkerError::ReceiptBindingMismatch);
        }
        self.record_terminal(&native_request_id);
        Ok(Some(if pending.run_terminal {
            RuntimeBrokerWorkerOutcome::SettledAfterCancellation {
                native_request_id,
                receipt,
            }
        } else {
            RuntimeBrokerWorkerOutcome::Executed {
                native_request_id,
                receipt,
            }
        }))
    }

    /// Signals every deferred Pi effect owned by one exact Run Attempt while
    /// retaining its lease for late durable settlement. The native run may
    /// already be terminal, so completion must never be delivered back to Pi.
    pub fn cancel_runtime_deferred_for_run<E>(&mut self, executor: &mut E, run_id: &str) -> usize
    where
        E: BrokerEffectExecutor,
    {
        let mut signalled = 0_usize;
        for pending in self
            .pending_runtime_effects
            .values_mut()
            .filter(|pending| pending.intent.run_id == run_id)
        {
            pending.run_terminal = true;
            if executor.cancel_deferred(&pending.ticket) {
                signalled = signalled.saturating_add(1);
            }
        }
        signalled
    }

    /// Cancels every outstanding deferred facility operation and consumes its
    /// Action Gateway effect lease as an explicit unknown result. Production
    /// runtime shutdown and quarantine call this before dropping the broker
    /// worker so an `effect-started` record is never abandoned in memory.
    pub fn drain_deferred_unknown<A, E>(
        &mut self,
        application: &mut A,
        executor: &mut E,
        now_ms: u64,
    ) -> Result<usize, BrokerWorkerError>
    where
        A: BrokerActionApplication,
        E: BrokerEffectExecutor,
    {
        let mut drained = 0_usize;
        let mut failure = None;

        let correlation_ids = self.pending_effects.keys().cloned().collect::<Vec<_>>();
        for correlation_id in correlation_ids {
            let Some(mut pending) = self.pending_effects.remove(&correlation_id) else {
                continue;
            };
            let _ = executor.cancel_deferred(&pending.ticket);
            match application.complete_runtime_action_effect_retryable(
                &mut pending.lease,
                interrupted_effect_result(now_ms),
                now_ms,
            ) {
                Ok(receipt)
                    if receipt.matches_runtime_tool(
                        &pending.intent.runtime_id,
                        &pending.intent.session_id,
                        &pending.intent.run_id,
                        &pending.intent.native_request_id,
                        pending.intent.process_generation,
                    ) =>
                {
                    let _ = executor.abandon_deferred(&pending.ticket);
                    self.record_terminal(&correlation_id);
                    drained = drained.saturating_add(1);
                }
                Ok(_) => {
                    failure.get_or_insert(BrokerWorkerError::ReceiptBindingMismatch);
                    let _ = executor.abandon_deferred(&pending.ticket);
                    self.record_terminal(&correlation_id);
                    drained = drained.saturating_add(1);
                }
                Err(_) => {
                    failure.get_or_insert(BrokerWorkerError::EffectStatusUnknown);
                    self.pending_effects.insert(correlation_id, pending);
                }
            };
        }

        let native_request_ids = self
            .pending_runtime_effects
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for native_request_id in native_request_ids {
            let Some(mut pending) = self.pending_runtime_effects.remove(&native_request_id) else {
                continue;
            };
            let _ = executor.cancel_deferred(&pending.ticket);
            match application.complete_runtime_action_effect_retryable(
                &mut pending.lease,
                interrupted_effect_result(now_ms),
                now_ms,
            ) {
                Ok(receipt)
                    if receipt.matches_runtime_tool(
                        &pending.intent.runtime_id,
                        &pending.intent.session_id,
                        &pending.intent.run_id,
                        &pending.intent.native_request_id,
                        pending.intent.process_generation,
                    ) =>
                {
                    let _ = executor.abandon_deferred(&pending.ticket);
                    self.record_terminal(&native_request_id);
                    drained = drained.saturating_add(1);
                }
                Ok(_) => {
                    failure.get_or_insert(BrokerWorkerError::ReceiptBindingMismatch);
                    let _ = executor.abandon_deferred(&pending.ticket);
                    self.record_terminal(&native_request_id);
                    drained = drained.saturating_add(1);
                }
                Err(_) => {
                    failure.get_or_insert(BrokerWorkerError::EffectStatusUnknown);
                    self.pending_runtime_effects
                        .insert(native_request_id, pending);
                }
            };
        }

        failure.map_or(Ok(drained), Err)
    }

    fn record_terminal(&mut self, correlation_id: &str) {
        if self.terminal.len() < self.max_terminal {
            self.terminal.insert(correlation_id.to_owned());
        } else {
            self.sealed_at_capacity = true;
        }
    }
}

fn interrupted_effect_result(now_ms: u64) -> RuntimeEffectResult {
    RuntimeEffectResult::with_certainty(
        NormalizedActionResult {
            status: NormalizedActionStatus::UnknownAfterInterruption,
            result_code: "runtime-effect-status-unknown".into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms: now_ms.max(1),
        },
        None,
        RuntimeEffectCompletionCertainty::Unknown,
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadResourceRequest {
    resource: String,
    #[serde(default)]
    selector: Option<String>,
}

impl ReadResourceRequest {
    fn validate(&self) -> Result<(), ProposalRejection> {
        validate_short_text(&self.resource)?;
        if let Some(selector) = &self.selector {
            validate_short_text(selector)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProposeActionRequest {
    operation: String,
    target: String,
    #[serde(default)]
    arguments: Map<String, Value>,
}

impl ProposeActionRequest {
    fn validate(&self) -> Result<(), ProposalRejection> {
        validate_short_text(&self.operation)?;
        validate_short_text(&self.target)?;
        let encoded =
            serde_json::to_vec(&self.arguments).map_err(|_| ProposalRejection::InvalidPayload)?;
        if encoded.len() > MAX_CANONICAL_ARGUMENT_BYTES {
            return Err(ProposalRejection::InvalidPayload);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnvelopeBindingError {
    NativeSession,
    NativeMessage,
    ProcessGeneration,
    Tool,
}

impl EnvelopeBindingError {
    fn reason_code(self) -> &'static str {
        match self {
            Self::NativeSession => "native-session-mismatch",
            Self::NativeMessage => "native-message-mismatch",
            Self::ProcessGeneration => "process-generation-mismatch",
            Self::Tool => "unsupported-broker-tool",
        }
    }
}

fn validate_envelope_binding(
    proposal: &BrokerProposal,
    context: &BrokerActionContext,
) -> Result<(), EnvelopeBindingError> {
    if proposal.session_id != context.native_session_id {
        return Err(EnvelopeBindingError::NativeSession);
    }
    if proposal.message_id != context.native_message_id {
        return Err(EnvelopeBindingError::NativeMessage);
    }
    if proposal.process_generation != context.dispatch.process_generation {
        return Err(EnvelopeBindingError::ProcessGeneration);
    }
    if !matches!(
        proposal.tool.as_str(),
        C4OS_ACTION_PROPOSAL_TOOL | C4OS_RESOURCE_READ_TOOL
    ) {
        return Err(EnvelopeBindingError::Tool);
    }
    Ok(())
}

fn validate_complete_binding(
    proposal: &RuntimeActionProposal,
    context: &BrokerActionContext,
    broker: &BrokerProposal,
) -> Result<(), ()> {
    let intent = &proposal.intent;
    let facts = &proposal.facts;
    let action = &proposal.action;
    if intent.workspace_id != context.dispatch.workspace_id
        || intent.session_id != context.dispatch.session_id
        || intent.turn_id != context.dispatch.turn_id
        || intent.run_id != context.dispatch.attempt_id
        || intent.correlation_id != context.dispatch.correlation_id
        || intent.runtime_id != context.dispatch.runtime_id
        || intent.process_generation != context.dispatch.process_generation
        || intent.native_request_id != broker.correlation_id
        || intent.native_tool != broker.tool
        || facts.workspace_id != context.dispatch.workspace_id
        || facts.session_id != context.dispatch.session_id
        || facts.runtime_id != context.dispatch.runtime_id
        || facts.environment_id != context.dispatch.environment_id
        || facts.native_tool != broker.tool
        || action.workspace_id != context.dispatch.workspace_id
        || action.session_id != context.dispatch.session_id
        || action.run_id != context.dispatch.attempt_id
        || action.runtime_id != context.dispatch.runtime_id
        || action.environment_id != context.dispatch.environment_id
        || action.process_generation != context.dispatch.process_generation
        || action.configuration_version != context.configuration_version
        || action.policy_version != context.policy_version
        || action.revocation_epoch != context.revocation_epoch
        || action.tool_call_id != broker.correlation_id
        || action.tool != broker.tool
    {
        Err(())
    } else {
        Ok(())
    }
}

fn validate_runtime_intent_binding(
    intent: &RuntimeIntentIdentity,
    context: &BrokerActionContext,
) -> Result<(), ()> {
    if context.dispatch.runtime_kind != RuntimeKind::Pi
        || intent.workspace_id != context.dispatch.workspace_id
        || intent.session_id != context.dispatch.session_id
        || intent.turn_id != context.dispatch.turn_id
        || intent.run_id != context.dispatch.attempt_id
        || intent.correlation_id != context.dispatch.correlation_id
        || intent.runtime_id != context.dispatch.runtime_id
        || intent.process_generation != context.dispatch.process_generation
        || intent.native_request_id != context.native_message_id
        || !matches!(
            intent.native_tool.as_str(),
            C4OS_ACTION_PROPOSAL_TOOL | C4OS_RESOURCE_READ_TOOL
        )
    {
        Err(())
    } else {
        Ok(())
    }
}

fn proposal_binding_sha256(
    proposal: &BrokerProposal,
    context: &BrokerActionContext,
    broker_arguments: &Value,
) -> Result<String, ProposalRejection> {
    let binding = json!({
        "workspaceId": context.dispatch.workspace_id,
        "environmentId": context.dispatch.environment_id,
        "sessionId": context.dispatch.session_id,
        "turnId": context.dispatch.turn_id,
        "runId": context.dispatch.attempt_id,
        "runCorrelationId": context.dispatch.correlation_id,
        "runtimeId": context.dispatch.runtime_id,
        "adapterVersion": context.dispatch.adapter_version,
        "nativeVersion": context.dispatch.native_version,
        "processGeneration": context.dispatch.process_generation,
        "nativeSessionId": proposal.session_id,
        "nativeMessageId": proposal.message_id,
        "nativeRequestId": proposal.correlation_id,
        "nativeTool": proposal.tool,
        "brokerArguments": broker_arguments,
        "configurationVersion": context.configuration_version,
        "policyVersion": context.policy_version,
        "revocationEpoch": context.revocation_epoch,
    });
    let encoded = serde_json::to_vec(&binding).map_err(|_| ProposalRejection::Binding)?;
    if encoded.len() > MAX_CANONICAL_ARGUMENT_BYTES {
        return Err(ProposalRejection::InvalidPayload);
    }
    Ok(sha256_prefixed(&encoded))
}

fn requested_authority(resolved: &ResolvedBrokerAction) -> BTreeSet<String> {
    let group = match resolved.surface {
        ActionSurface::File => "workspace-files",
        ActionSurface::Terminal | ActionSurface::Process => "commands-processes",
        ActionSurface::Git => "version-control",
        ActionSurface::Network => "network-sharing",
        ActionSurface::Browser | ActionSurface::Desktop => "browser-desktop",
        ActionSurface::Credential => "credentials",
        ActionSurface::C4os | ActionSurface::Unknown(_) => "extensions-c4os",
    };
    resolved
        .effects
        .iter()
        .map(|effect| {
            let effect = match effect {
                ActionEffect::Read => "read",
                ActionEffect::Capture => "capture",
                ActionEffect::Create => "create",
                ActionEffect::Modify => "modify",
                ActionEffect::Delete => "delete",
                ActionEffect::Execute => "execute",
                ActionEffect::Control => "control",
                ActionEffect::Publish => "publish",
                ActionEffect::Upload => "upload",
                ActionEffect::Reveal => "reveal",
                ActionEffect::Listen => "listen",
                ActionEffect::Unknown => "unknown",
            };
            format!("{group}.{effect}")
        })
        .collect()
}

fn action_id(binding_sha256: &str) -> String {
    let digest = binding_sha256
        .strip_prefix("sha256:")
        .expect("binding digests are constructed locally");
    format!("broker-{}", &digest[..40])
}

fn decision_from_receipt(receipt: &RuntimeExecutionReceipt) -> BrokerDecision {
    let result = receipt.normalized_result();
    match result.status {
        NormalizedActionStatus::Cancelled => BrokerDecision::Cancelled,
        NormalizedActionStatus::Denied => denied(&result.result_code),
        NormalizedActionStatus::Succeeded
        | NormalizedActionStatus::Failed
        | NormalizedActionStatus::UnknownAfterInterruption => {
            BrokerDecision::Result(receipt.model_payload().cloned().unwrap_or_else(|| {
                json!({
                    "status": result.status,
                    "resultCode": result.result_code,
                    "exitCode": result.exit_code,
                    "changedTargetCount": result.changed_targets.len(),
                    "outputSha256": result.output_sha256,
                    "completedAtMs": result.completed_at_ms,
                })
            }))
        }
    }
}

fn respond_denied(correlation_id: &str, reason_code: &str) -> BrokerWorkerOutcome {
    BrokerWorkerOutcome::Respond {
        correlation_id: correlation_id.to_owned(),
        decision: denied(reason_code),
    }
}

fn runtime_denied(native_request_id: &str, reason_code: &str) -> RuntimeBrokerWorkerOutcome {
    RuntimeBrokerWorkerOutcome::Denied {
        native_request_id: native_request_id.to_owned(),
        reason_code: reason_code.to_owned(),
    }
}

fn denied(reason_code: &str) -> BrokerDecision {
    BrokerDecision::Denied {
        reason_code: reason_code.to_owned(),
    }
}

fn validate_short_text(value: &str) -> Result<(), ProposalRejection> {
    if value.trim().is_empty()
        || value.len() > MAX_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
        Err(ProposalRejection::InvalidPayload)
    } else {
        Ok(())
    }
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

#[derive(Debug)]
enum ProposalRejection {
    InvalidPayload,
    UnsupportedTool,
    Classification(BrokerClassificationError),
    Binding,
}

impl ProposalRejection {
    fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidPayload => "invalid-broker-payload",
            Self::UnsupportedTool => "unsupported-broker-tool",
            Self::Classification(BrokerClassificationError::Unsupported) => {
                "unsupported-broker-operation"
            }
            Self::Classification(BrokerClassificationError::Ambiguous) => {
                "ambiguous-broker-operation"
            }
            Self::Classification(BrokerClassificationError::Unresolved) => {
                "unresolved-broker-target"
            }
            Self::Classification(BrokerClassificationError::Unavailable) => {
                "broker-classifier-unavailable"
            }
            Self::Binding => "broker-binding-mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum BrokerWorkerError {
    #[error("broker worker capacity configuration is invalid")]
    InvalidConfiguration,
    #[error("trusted broker action context is invalid")]
    InvalidTrustedContext,
    #[error("broker approval is unknown")]
    UnknownApproval,
    #[error("broker approval identity does not match")]
    ApprovalBindingMismatch,
    #[error("runtime application service failed")]
    Application,
    #[error("authorized broker effect did not return a trustworthy terminal receipt")]
    EffectStatusUnknown,
    #[error("runtime receipt did not match the broker action")]
    ReceiptBindingMismatch,
    #[error("authenticated broker event was not a cancellation")]
    ExpectedCancellation,
    #[error("deferred broker ticket is invalid")]
    InvalidDeferredTicket,
}
