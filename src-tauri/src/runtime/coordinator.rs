//! Production composition boundary for provider, runtime, capability, session,
//! and action execution state.
//!
//! The individual runtime modules deliberately own narrow invariants. This
//! facade is the place where those invariants become one dispatch decision: a
//! selected and freshly tested provider route, the exact effective capability
//! intersection, and a live supervised process generation must all agree
//! before a Chat can submit or retry a run.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runtime::action_bridge::{
    RuntimeActionBridge, RuntimeActionProposal, RuntimeApprovalDecision, RuntimeAuthorization,
    RuntimeBridgeError, RuntimeExecutionReceipt, RuntimeGatewayDecision, RuntimeIntentIdentity,
};
use crate::runtime::capability::{
    CapabilityDescriptor, CapabilityError, DraftRequirements, PreflightOutcome,
    effective_intersection, preflight,
};
use crate::runtime::provider::{
    PROVIDER_TEST_FRESHNESS_MS, ProviderError, ProviderProbe, ProviderProfile, ProviderService,
    ProviderSnapshot, ProviderTestReport, ProviderTestStatus,
};
use crate::runtime::session::{
    AttemptIdentity, FirstSubmission, RetryRequest, RunEventRecord, SessionError, SessionRecord,
    SessionRepository, SessionService, SideEffectState, TerminalAttemptOutcome, TurnSubmission,
};
use crate::runtime::supervisor::{
    CompatibilityState, HealthState, RuntimeInstallation, RuntimeLifecycle, RuntimeSupervisor,
    SupervisorError, SupervisorSnapshot,
};
use crate::security::authorization::{
    ApprovalAnswer, AuthorizationToken, CanonicalAction, LiveAuthorityState,
};
use crate::security::gateway::{
    ActionGateway, ActionGatewayError, ApprovalResponse, ExecutionPermit, GatewayProposal,
    NormalizedActionResult,
};
use crate::security::policy::ActionFacts;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeCoordinatorSnapshot {
    pub generation: u64,
    pub providers: ProviderSnapshot,
    pub runtimes: SupervisorSnapshot,
    pub onboarding_ready: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoordinatorOperation<T> {
    pub coordinator_generation: u64,
    pub value: T,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelPreflight {
    pub provider_id: String,
    pub model_id: String,
    pub effective_capabilities: CapabilityDescriptor,
    pub outcome: PreflightOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoordinatedFirstSubmission {
    pub submission: FirstSubmission,
    /// Provider-service lookup identity. This can differ from the provider's
    /// native model identifier stored in the capability route snapshot.
    pub provider_id: String,
    pub selected_model_id: String,
    pub capability_layers: [CapabilityDescriptor; 3],
    pub draft: DraftRequirements,
    pub preflight_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoordinatedRetry {
    pub request: RetryRequest,
    pub provider_id: String,
    pub selected_model_id: String,
    pub capability_layers: [CapabilityDescriptor; 3],
    pub draft: DraftRequirements,
    pub preflight_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoordinatedTurn {
    pub submission: TurnSubmission,
    pub provider_id: String,
    pub selected_model_id: String,
    pub capability_layers: [CapabilityDescriptor; 3],
    pub draft: DraftRequirements,
    pub preflight_at_ms: u64,
}

/// Long-lived app-owned runtime services. No inner mutable service is exposed,
/// so every cross-domain mutation advances the single coordinator generation.
pub struct RuntimeCoordinator<R> {
    generation: u64,
    providers: ProviderService,
    supervisor: RuntimeSupervisor,
    sessions: SessionService<R>,
    action_gateway: ActionGateway,
}

impl<R: SessionRepository> RuntimeCoordinator<R> {
    pub fn new(
        providers: ProviderService,
        supervisor: RuntimeSupervisor,
        sessions: SessionService<R>,
        action_gateway: ActionGateway,
    ) -> Self {
        Self {
            generation: 0,
            providers,
            supervisor,
            sessions,
            action_gateway,
        }
    }

    pub fn restore(
        generation: u64,
        providers: ProviderService,
        supervisor: RuntimeSupervisor,
        sessions: SessionService<R>,
        action_gateway: ActionGateway,
    ) -> Self {
        Self {
            generation,
            providers,
            supervisor,
            sessions,
            action_gateway,
        }
    }

    pub fn snapshot(&self, now_ms: u64) -> RuntimeCoordinatorSnapshot {
        let providers = self.providers.snapshot();
        RuntimeCoordinatorSnapshot {
            generation: self.generation,
            onboarding_ready: providers.onboarding_ready_at(now_ms),
            providers,
            runtimes: self.supervisor.snapshot(),
        }
    }

    pub fn session(&self, session_id: &str) -> Result<SessionRecord, CoordinatorError> {
        Ok(self.sessions.session(session_id)?)
    }

    pub fn durable_sessions(&self) -> Result<Vec<SessionRecord>, CoordinatorError> {
        Ok(self.sessions.durable_sessions()?)
    }

    pub fn save_provider(
        &mut self,
        profile: ProviderProfile,
        expected_provider_generation: u64,
    ) -> Result<CoordinatorOperation<u64>, CoordinatorError> {
        let generation = self
            .providers
            .save_profile(profile, expected_provider_generation)?;
        self.operation(generation)
    }

    pub fn test_provider(
        &mut self,
        provider_id: &str,
        expected_provider_generation: u64,
        attempted_at_ms: u64,
        probe: &mut impl ProviderProbe,
    ) -> Result<CoordinatorOperation<ProviderTestReport>, CoordinatorError> {
        let report = self.providers.test_provider(
            provider_id,
            expected_provider_generation,
            attempted_at_ms,
            probe,
        )?;
        self.operation(report)
    }

    pub fn select_model(
        &mut self,
        provider_id: &str,
        model_id: &str,
        expected_provider_generation: u64,
    ) -> Result<CoordinatorOperation<u64>, CoordinatorError> {
        let generation =
            self.providers
                .select_model(provider_id, model_id, expected_provider_generation)?;
        self.operation(generation)
    }

    pub fn register_runtime(
        &mut self,
        installation: RuntimeInstallation,
        checked_at_ms: u64,
    ) -> Result<CoordinatorOperation<CompatibilityState>, CoordinatorError> {
        let compatibility = self
            .supervisor
            .register_installation(installation, checked_at_ms)?;
        self.operation(compatibility)
    }

    pub fn replace_workspace_runtime_installations(
        &mut self,
        workspace_id: &str,
        installations: Vec<RuntimeInstallation>,
        checked_at_ms: u64,
    ) -> Result<
        CoordinatorOperation<std::collections::BTreeMap<String, CompatibilityState>>,
        CoordinatorError,
    > {
        let compatibility = self.supervisor.replace_workspace_installations(
            workspace_id,
            installations,
            checked_at_ms,
        )?;
        self.operation(compatibility)
    }

    pub fn clear_runtime_installations(
        &mut self,
    ) -> Result<CoordinatorOperation<()>, CoordinatorError> {
        self.supervisor.clear_installations()?;
        self.operation(())
    }

    pub fn start_runtime(
        &mut self,
        runtime_id: &str,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<u64>, CoordinatorError> {
        let process_generation = self.supervisor.start(runtime_id, at_ms)?;
        self.operation(process_generation)
    }

    /// Reserves one generation for a production adapter that owns the native
    /// process handle and protocol-aware shutdown path.
    pub fn reserve_managed_runtime_start(
        &mut self,
        runtime_id: &str,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<u64>, CoordinatorError> {
        let process_generation = self.supervisor.reserve_managed_start(runtime_id, at_ms)?;
        self.operation(process_generation)
    }

    pub fn attach_managed_runtime_process(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        process_id: u32,
        health: HealthState,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, CoordinatorError> {
        self.supervisor.attach_managed_process(
            runtime_id,
            process_generation,
            process_id,
            health,
            at_ms,
        )?;
        self.operation(())
    }

    pub fn abort_managed_runtime_start(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, CoordinatorError> {
        self.supervisor
            .abort_managed_start(runtime_id, process_generation, at_ms)?;
        self.operation(())
    }

    pub fn finish_managed_runtime_shutdown(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, CoordinatorError> {
        self.supervisor
            .finish_managed_shutdown(runtime_id, process_generation, at_ms)?;
        self.operation(())
    }

    pub fn record_runtime_health(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        health: HealthState,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, CoordinatorError> {
        self.supervisor
            .record_health(runtime_id, process_generation, health, at_ms)?;
        self.operation(())
    }

    /// Advances the application-wide authority after a separately typed core
    /// service, such as capability evidence, commits a state change. The
    /// renderer still observes one monotonic generation across domains.
    pub(crate) fn note_core_authority_change(&mut self) -> Result<u64, CoordinatorError> {
        Ok(self.operation(())?.coordinator_generation)
    }

    pub(crate) fn discard_all_provisionals(&mut self) -> Result<usize, CoordinatorError> {
        Ok(self.sessions.discard_all_provisionals()?)
    }

    pub fn create_provisional(
        &mut self,
        session_id: impl Into<String>,
        created_at_ms: u64,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let record = self
            .sessions
            .create_provisional(session_id, created_at_ms)?;
        self.operation(record)
    }

    pub fn discard_provisional(
        &mut self,
        session_id: &str,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let record = self.sessions.discard_provisional(session_id)?;
        self.operation(record)
    }

    /// Resolve the exact selected route. This is read-only; callers may show a
    /// blocked outcome without mutating or silently rewriting the draft.
    pub fn model_preflight(
        &self,
        provider_id: &str,
        model_id: &str,
        layers: &[CapabilityDescriptor; 3],
        draft: &DraftRequirements,
        now_ms: u64,
    ) -> Result<ModelPreflight, CoordinatorError> {
        let provider = self.fresh_selected_provider(provider_id, model_id, now_ms)?;
        let route = provider
            .models
            .get(model_id)
            .ok_or(CoordinatorError::ModelRouteUnavailable)?;
        let effective = effective_intersection(layers, now_ms)?;
        if effective.route.provider_id != route.capabilities.route.provider_id
            || effective.route.endpoint_id != route.capabilities.route.endpoint_id
            || effective.route.provider_model_id.rsplit('/').next()
                != route
                    .capabilities
                    .route
                    .provider_model_id
                    .rsplit('/')
                    .next()
            || effective.route.model_revision != route.capabilities.route.model_revision
        {
            return Err(CoordinatorError::BindingMismatch);
        }
        let outcome = preflight(&effective, draft)?;
        Ok(ModelPreflight {
            provider_id: provider_id.to_owned(),
            model_id: model_id.to_owned(),
            effective_capabilities: effective,
            outcome,
        })
    }

    pub fn submit_first(
        &mut self,
        request: CoordinatedFirstSubmission,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let preflight = self.model_preflight(
            &request.provider_id,
            &request.selected_model_id,
            &request.capability_layers,
            &request.draft,
            request.preflight_at_ms,
        )?;
        require_ready(&preflight.outcome)?;
        self.ensure_binding_matches_preflight(&request.submission, &preflight)?;
        self.ensure_runtime_ready(
            &request.submission.binding.runtime_id,
            request.submission.process_generation,
            &request.submission.binding.adapter.native_version,
            &request.submission.binding.adapter.adapter_version,
        )?;
        let record = self.sessions.submit_first(request.submission)?;
        self.operation(record)
    }

    pub fn submit_turn(
        &mut self,
        request: CoordinatedTurn,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let resolved = self.model_preflight(
            &request.provider_id,
            &request.selected_model_id,
            &request.capability_layers,
            &request.draft,
            request.preflight_at_ms,
        )?;
        require_ready(&resolved.outcome)?;
        let route = &request.submission.context.model_route;
        if route.provider_id != resolved.provider_id
            || route.model_id != resolved.effective_capabilities.route.provider_model_id
            || route.endpoint_id != resolved.effective_capabilities.route.endpoint_id
            || route.model_revision != resolved.effective_capabilities.route.model_revision
        {
            return Err(CoordinatorError::BindingMismatch);
        }
        self.ensure_runtime_ready(
            &request.submission.context.runtime_id,
            request.submission.process_generation,
            &request.submission.context.adapter.native_version,
            &request.submission.context.adapter.adapter_version,
        )?;
        let record = self.sessions.submit_turn(request.submission)?;
        self.operation(record)
    }

    pub fn append_normalized_event(
        &mut self,
        session_id: &str,
        identity: &AttemptIdentity,
        event: RunEventRecord,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        self.ensure_runtime_ready(&event.runtime_id, event.process_generation, "", "")?;
        let record = self.sessions.append_event(session_id, identity, event)?;
        self.operation(record)
    }

    pub fn request_cancellation(
        &mut self,
        session_id: &str,
        identity: &AttemptIdentity,
        requested_at_ms: u64,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let record = self
            .sessions
            .request_cancellation(session_id, identity, requested_at_ms)?;
        self.operation(record)
    }

    pub fn finish_attempt(
        &mut self,
        session_id: &str,
        identity: &AttemptIdentity,
        outcome: TerminalAttemptOutcome,
        finished_at_ms: u64,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let record = self
            .sessions
            .finish_attempt(session_id, identity, outcome, finished_at_ms)?;
        self.operation(record)
    }

    pub fn retry(
        &mut self,
        request: CoordinatedRetry,
    ) -> Result<CoordinatorOperation<SessionRecord>, CoordinatorError> {
        let resolved = self.model_preflight(
            &request.provider_id,
            &request.selected_model_id,
            &request.capability_layers,
            &request.draft,
            request.preflight_at_ms,
        )?;
        require_ready(&resolved.outcome)?;
        let route = &request.request.context.model_route;
        if route.provider_id != resolved.provider_id
            || route.model_id != resolved.effective_capabilities.route.provider_model_id
            || route.endpoint_id != resolved.effective_capabilities.route.endpoint_id
            || route.model_revision != resolved.effective_capabilities.route.model_revision
        {
            return Err(CoordinatorError::BindingMismatch);
        }
        self.ensure_runtime_ready(
            &request.request.context.runtime_id,
            request.request.process_generation,
            &request.request.context.adapter.native_version,
            &request.request.context.adapter.adapter_version,
        )?;
        let record = self.sessions.retry(request.request)?;
        self.operation(record)
    }

    pub fn recover_interrupted(
        &mut self,
        recovered_at_ms: u64,
    ) -> Result<CoordinatorOperation<Vec<SessionRecord>>, CoordinatorError> {
        let recovered = self.sessions.recover_all_interrupted(recovered_at_ms)?;
        self.operation(recovered)
    }

    pub fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<RuntimeGatewayDecision>, CoordinatorError> {
        self.ensure_active_intent(&proposal.intent)?;
        self.ensure_runtime_ready(
            &proposal.intent.runtime_id,
            proposal.intent.process_generation,
            "",
            "",
        )?;
        let decision =
            RuntimeActionBridge::new(&mut self.action_gateway).propose(proposal, now_ms)?;
        self.operation(decision)
    }

    /// Routes a core-owned direct user action through the same durable gateway
    /// without pretending it originated from a runtime tool attempt.
    pub(crate) fn propose_direct_action(
        &mut self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<GatewayProposal>, CoordinatorError> {
        let proposal = self.action_gateway.propose(facts, action, now_ms)?;
        self.operation(proposal)
    }

    pub(crate) fn requeue_interrupted_direct_approval(
        &mut self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<GatewayProposal>, CoordinatorError> {
        let proposal = self
            .action_gateway
            .requeue_interrupted_approval(facts, action, now_ms)?;
        self.operation(proposal)
    }

    pub(crate) fn answer_direct_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<ApprovalResponse>, CoordinatorError> {
        let response = self
            .action_gateway
            .answer_approval(prompt_id, answer, now_ms)?;
        self.operation(response)
    }

    pub(crate) fn cancel_direct_action_run(
        &mut self,
        run_id: &str,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<usize>, CoordinatorError> {
        let cancelled = self.action_gateway.cancel_run(run_id, now_ms)?;
        self.operation(cancelled)
    }

    pub(crate) fn execute_direct_action(
        &mut self,
        token: &AuthorizationToken,
        action: &CanonicalAction,
        live: LiveAuthorityState,
        approval_prompt_id: Option<&str>,
        now_ms: u64,
        effect: impl FnOnce(ExecutionPermit) -> NormalizedActionResult,
    ) -> Result<CoordinatorOperation<NormalizedActionResult>, CoordinatorError> {
        let result =
            self.action_gateway
                .execute(token, action, live, approval_prompt_id, now_ms, effect)?;
        self.operation(result)
    }

    pub fn answer_runtime_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<RuntimeApprovalDecision>, CoordinatorError> {
        let decision = RuntimeActionBridge::new(&mut self.action_gateway)
            .answer_approval(prompt_id, answer, now_ms)?;
        self.operation(decision)
    }

    pub fn execute_runtime_action(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        effect: impl FnOnce(ExecutionPermit) -> NormalizedActionResult,
    ) -> Result<CoordinatorOperation<RuntimeExecutionReceipt>, CoordinatorError> {
        let (session_id, run_id, runtime_id, process_generation) = {
            let (session_id, run_id, runtime_id, process_generation) =
                authorization.active_run_identity();
            (
                session_id.to_owned(),
                run_id.to_owned(),
                runtime_id.to_owned(),
                process_generation,
            )
        };
        let action_id = authorization.action_id().to_owned();
        let session = self.sessions.session(&session_id)?;
        let binding = session.binding().ok_or(CoordinatorError::BindingMismatch)?;
        let attempt = session
            .attempt(&run_id)
            .ok_or(CoordinatorError::BindingMismatch)?;
        if session.active_attempt_id.as_deref() != Some(run_id.as_str())
            || binding.runtime_id != runtime_id
            || attempt.process_generation != process_generation
        {
            return Err(CoordinatorError::BindingMismatch);
        }
        self.sessions.set_side_effect_state(
            &session_id,
            &AttemptIdentity {
                attempt_id: run_id.clone(),
                correlation_id: attempt.correlation_id.clone(),
                process_generation,
            },
            SideEffectState::Started {
                action_id: action_id.clone(),
                idempotent: false,
            },
            now_ms,
        )?;
        let receipt = RuntimeActionBridge::new(&mut self.action_gateway).execute(
            authorization,
            live,
            now_ms,
            effect,
        )?;
        let terminal_effect = match receipt.result().status {
            crate::security::gateway::NormalizedActionStatus::Succeeded => {
                SideEffectState::Completed { action_id }
            }
            crate::security::gateway::NormalizedActionStatus::UnknownAfterInterruption => {
                SideEffectState::Unknown { action_id }
            }
            crate::security::gateway::NormalizedActionStatus::Failed
            | crate::security::gateway::NormalizedActionStatus::Cancelled
            | crate::security::gateway::NormalizedActionStatus::Denied => {
                SideEffectState::ProvenNotCompleted {
                    action_id,
                    idempotent: false,
                }
            }
        };
        self.sessions.set_side_effect_state(
            &session_id,
            &AttemptIdentity {
                attempt_id: run_id,
                correlation_id: attempt.correlation_id.clone(),
                process_generation,
            },
            terminal_effect,
            now_ms.saturating_add(1),
        )?;
        self.operation(receipt)
    }

    /// Invalidates every still-open Action Gateway authorization and approval
    /// for one durable Run Attempt. Runtime workers call this when their
    /// native request is cancelled or its trusted identity becomes stale.
    pub fn cancel_runtime_actions(
        &mut self,
        run_id: &str,
        now_ms: u64,
    ) -> Result<CoordinatorOperation<usize>, CoordinatorError> {
        let cancelled = self
            .action_gateway
            .cancel_run(run_id, now_ms)
            .map_err(RuntimeBridgeError::Gateway)?;
        self.operation(cancelled)
    }

    fn fresh_selected_provider(
        &self,
        provider_id: &str,
        model_id: &str,
        now_ms: u64,
    ) -> Result<crate::runtime::provider::ProviderRecord, CoordinatorError> {
        let record = self
            .providers
            .snapshot()
            .providers
            .into_iter()
            .find(|record| record.profile.provider_id == provider_id)
            .ok_or(CoordinatorError::ModelRouteUnavailable)?;
        let tested_at_ms = match &record.test_status {
            ProviderTestStatus::Succeeded { checked_at_ms } => *checked_at_ms,
            _ => return Err(CoordinatorError::ProviderNotReady),
        };
        let route = record
            .models
            .get(model_id)
            .ok_or(CoordinatorError::ModelRouteUnavailable)?;
        if !record.profile.enabled
            || !route.is_usable()
            || record.connection_evidence.as_ref().is_none_or(|evidence| {
                evidence.tested_at_ms != tested_at_ms
                    || evidence.tested_at_ms > now_ms
                    || now_ms.saturating_sub(evidence.tested_at_ms) > PROVIDER_TEST_FRESHNESS_MS
            })
            || tested_at_ms > now_ms
            || route.checked_at_ms > now_ms
            || now_ms.saturating_sub(tested_at_ms) > PROVIDER_TEST_FRESHNESS_MS
            || now_ms.saturating_sub(route.checked_at_ms) > PROVIDER_TEST_FRESHNESS_MS
            || route.capabilities.features.values().any(expired_at(now_ms))
            || route
                .capabilities
                .numeric_limits
                .values()
                .map(|value| &value.evidence)
                .any(expired_at(now_ms))
        {
            return Err(CoordinatorError::ProviderNotReady);
        }
        Ok(record)
    }

    fn ensure_binding_matches_preflight(
        &self,
        submission: &FirstSubmission,
        resolved: &ModelPreflight,
    ) -> Result<(), CoordinatorError> {
        let route = &submission.binding.initial_model_route;
        if route.provider_id != resolved.provider_id
            || route.model_id != resolved.effective_capabilities.route.provider_model_id
            || route.endpoint_id != resolved.effective_capabilities.route.endpoint_id
            || route.model_revision != resolved.effective_capabilities.route.model_revision
        {
            return Err(CoordinatorError::BindingMismatch);
        }
        Ok(())
    }

    fn ensure_active_intent(&self, intent: &RuntimeIntentIdentity) -> Result<(), CoordinatorError> {
        let session = self.sessions.session(&intent.session_id)?;
        let binding = session.binding().ok_or(CoordinatorError::BindingMismatch)?;
        let attempt = session
            .attempt(&intent.run_id)
            .ok_or(CoordinatorError::BindingMismatch)?;
        if binding.workspace_id != intent.workspace_id
            || binding.runtime_id != intent.runtime_id
            || session.active_attempt_id.as_deref() != Some(intent.run_id.as_str())
            || attempt.turn_id != intent.turn_id
            || attempt.correlation_id != intent.correlation_id
            || attempt.process_generation != intent.process_generation
        {
            return Err(CoordinatorError::BindingMismatch);
        }
        Ok(())
    }

    fn ensure_runtime_ready(
        &self,
        runtime_id: &str,
        process_generation: u64,
        native_version: &str,
        adapter_version: &str,
    ) -> Result<(), CoordinatorError> {
        let snapshot = self.supervisor.snapshot();
        let record = snapshot
            .records
            .iter()
            .find(|record| record.installation.runtime_id == runtime_id)
            .ok_or(CoordinatorError::RuntimeNotReady)?;
        if record.compatibility != CompatibilityState::Compatible
            || !matches!(
                record.lifecycle,
                RuntimeLifecycle::Ready | RuntimeLifecycle::Degraded
            )
            || record.process_id.is_none()
            || record.process_generation != process_generation
            || (!native_version.is_empty() && record.installation.native_version != native_version)
            || (!adapter_version.is_empty()
                && record.installation.adapter_version != adapter_version)
        {
            return Err(CoordinatorError::RuntimeNotReady);
        }
        Ok(())
    }

    fn operation<T>(&mut self, value: T) -> Result<CoordinatorOperation<T>, CoordinatorError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(CoordinatorError::GenerationExhausted)?;
        Ok(CoordinatorOperation {
            coordinator_generation: self.generation,
            value,
        })
    }
}

fn require_ready(outcome: &PreflightOutcome) -> Result<(), CoordinatorError> {
    if matches!(outcome, PreflightOutcome::Ready { .. }) {
        Ok(())
    } else {
        Err(CoordinatorError::PreflightBlocked)
    }
}

fn expired_at(now_ms: u64) -> impl FnMut(&crate::runtime::capability::CapabilityEvidence) -> bool {
    move |evidence| {
        evidence.checked_at_ms > now_ms
            || evidence
                .expires_at_ms
                .is_some_and(|expires_at_ms| now_ms >= expires_at_ms)
    }
}

#[derive(Debug, Error)]
pub enum CoordinatorError {
    #[error("provider route has not passed a fresh successful test")]
    ProviderNotReady,
    #[error("selected model route is unavailable")]
    ModelRouteUnavailable,
    #[error("runtime process is not ready for the exact requested generation")]
    RuntimeNotReady,
    #[error("model, runtime, or session binding does not match resolved authority")]
    BindingMismatch,
    #[error("draft preflight is blocked")]
    PreflightBlocked,
    #[error("runtime coordinator generation was exhausted")]
    GenerationExhausted,
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),
    #[error(transparent)]
    Capability(#[from] CapabilityError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error(transparent)]
    RuntimeBridge(#[from] RuntimeBridgeError),
    #[error(transparent)]
    ActionGateway(#[from] ActionGatewayError),
}
