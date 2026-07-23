//! Production-composed MCP sampling through the exact active Pi route.

#![allow(deprecated)]

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rmcp::model::{CreateMessageRequestParams, CreateMessageResult, SamplingMessage};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    RuntimeApplicationError, RuntimeApplicationService, RuntimeMcpSamplingIntent, current_time_ms,
    runtime::{
        dispatch::{DispatchError, PeerDispatchError, PeerSamplingResult},
        pi::PiSamplingMessage,
    },
    security::{
        authorization::{ApprovalAnswer, ApprovalQueueError, AuthorizationToken, CanonicalAction},
        gateway::{
            ActionEffectLease, ActionGatewayError, ApprovalResponse, GatewayProposal,
            NormalizedActionResult, NormalizedActionStatus,
        },
    },
};

use super::{
    McpError,
    transport::{
        McpCancellation, McpFuture, McpSamplingBroker, McpSamplingContext, sampling_text_messages,
    },
};

const MAX_PENDING_SAMPLING_APPROVALS: usize = 128;
const MAX_PENDING_SAMPLING_COMPLETIONS: usize = 128;

#[derive(Clone, Default)]
pub(crate) struct McpSamplingParentRegistry {
    active: Arc<Mutex<BTreeMap<String, CanonicalAction>>>,
}

pub(crate) struct McpSamplingParentGuard {
    registry: McpSamplingParentRegistry,
    server_id: String,
    action_id: String,
}

impl McpSamplingParentRegistry {
    pub(crate) fn register(
        &self,
        server_id: &str,
        action: CanonicalAction,
    ) -> Result<McpSamplingParentGuard, McpError> {
        let mut active = self.active.lock().map_err(|_| McpError::StateUnavailable)?;
        if active.contains_key(server_id) {
            return Err(McpError::Conflict);
        }
        let action_id = action.action_id.clone();
        active.insert(server_id.into(), action);
        Ok(McpSamplingParentGuard {
            registry: self.clone(),
            server_id: server_id.into(),
            action_id,
        })
    }

    fn parent(&self, context: &McpSamplingContext) -> Result<CanonicalAction, McpError> {
        let action = self
            .active
            .lock()
            .map_err(|_| McpError::StateUnavailable)?
            .get(&context.server_id)
            .cloned()
            .ok_or(McpError::Denied)?;
        let resolved = action
            .arguments
            .get("resolved")
            .and_then(serde_json::Value::as_object)
            .ok_or(McpError::Conflict)?;
        if action.plugin_or_mcp_id.as_deref() != Some(context.authority_id.as_str())
            || resolved.get("serverId").and_then(serde_json::Value::as_str)
                != Some(context.server_id.as_str())
            || resolved
                .get("definitionSha256")
                .and_then(serde_json::Value::as_str)
                != Some(context.definition_sha256.as_str())
            || resolved
                .get("lifecycleGeneration")
                .and_then(serde_json::Value::as_u64)
                != Some(context.lifecycle_generation)
        {
            return Err(McpError::Conflict);
        }
        Ok(action)
    }
}

impl Drop for McpSamplingParentGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = self.registry.active.lock()
            && active
                .get(&self.server_id)
                .is_some_and(|action| action.action_id == self.action_id)
        {
            active.remove(&self.server_id);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct McpSamplingApprovalSummary {
    pub runtime_id: String,
    pub correlation_id: String,
    pub prompt_id: String,
    pub server_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub max_tokens: u32,
    pub expires_at_ms: u64,
    pub message_count: usize,
    pub input_bytes: usize,
    pub has_system_prompt: bool,
    pub parent_operation: String,
}

enum SamplingApprovalDecision {
    Authorized(AuthorizationToken),
    Denied,
}

struct PendingSamplingApproval {
    runtime_id: String,
    correlation_id: String,
    prompt_id: String,
    run_id: String,
    server_id: String,
    provider_id: String,
    model_id: String,
    max_tokens: u32,
    expires_at_ms: u64,
    message_count: usize,
    input_bytes: usize,
    has_system_prompt: bool,
    parent_operation: String,
    sender: Option<oneshot::Sender<SamplingApprovalDecision>>,
    settling: bool,
    cancel_requested: bool,
}

#[derive(Default)]
struct SamplingApprovalRegistryState {
    pending: BTreeMap<String, PendingSamplingApproval>,
    reserved: usize,
    revision: u64,
    active_transitions: usize,
}

#[derive(Clone, Default)]
pub(crate) struct ProductionMcpSamplingApprovalRegistry {
    state: Arc<Mutex<SamplingApprovalRegistryState>>,
}

struct SamplingApprovalReservation {
    registry: ProductionMcpSamplingApprovalRegistry,
    active: bool,
}

struct SamplingApprovalTransition {
    registry: ProductionMcpSamplingApprovalRegistry,
    active: bool,
}

struct SamplingApprovalWaitGuard {
    registry: ProductionMcpSamplingApprovalRegistry,
    runtime: Arc<RuntimeApplicationService>,
    prompt_id: String,
    armed: bool,
}

impl ProductionMcpSamplingApprovalRegistry {
    fn lock_state(&self) -> std::sync::MutexGuard<'_, SamplingApprovalRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn reserve(&self) -> Result<SamplingApprovalReservation, McpError> {
        let mut state = self.lock_state();
        if state.pending.len().saturating_add(state.reserved) >= MAX_PENDING_SAMPLING_APPROVALS {
            return Err(McpError::StateUnavailable);
        }
        state.reserved += 1;
        Ok(SamplingApprovalReservation {
            registry: self.clone(),
            active: true,
        })
    }

    fn begin_transition(&self) -> SamplingApprovalTransition {
        let mut state = self.lock_state();
        state.active_transitions = state.active_transitions.saturating_add(1);
        state.revision = state.revision.wrapping_add(1);
        SamplingApprovalTransition {
            registry: self.clone(),
            active: true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn register(
        &self,
        reservation: &mut SamplingApprovalReservation,
        runtime_id: String,
        correlation_id: String,
        prompt_id: String,
        run_id: String,
        server_id: String,
        provider_id: String,
        model_id: String,
        max_tokens: u32,
        expires_at_ms: u64,
        message_count: usize,
        input_bytes: usize,
        has_system_prompt: bool,
        parent_operation: String,
        sender: oneshot::Sender<SamplingApprovalDecision>,
    ) -> Result<(), McpError> {
        let mut state = self.lock_state();
        if !reservation.active
            || !Arc::ptr_eq(&self.state, &reservation.registry.state)
            || state.reserved == 0
            || state.pending.len() >= MAX_PENDING_SAMPLING_APPROVALS
            || state.pending.contains_key(&prompt_id)
        {
            return Err(McpError::StateUnavailable);
        }
        state.pending.insert(
            prompt_id.clone(),
            PendingSamplingApproval {
                runtime_id,
                correlation_id,
                prompt_id,
                run_id,
                server_id,
                provider_id,
                model_id,
                max_tokens,
                expires_at_ms,
                message_count,
                input_bytes,
                has_system_prompt,
                parent_operation,
                sender: Some(sender),
                settling: false,
                cancel_requested: false,
            },
        );
        state.reserved -= 1;
        reservation.active = false;
        Ok(())
    }

    pub(crate) fn stable_summaries(
        &self,
    ) -> Result<Option<(u64, Vec<McpSamplingApprovalSummary>)>, McpError> {
        let state = self.lock_state();
        if state.active_transitions != 0 {
            return Ok(None);
        }
        let summaries = state
            .pending
            .values()
            .filter(|pending| {
                !pending.cancel_requested && !pending.settling && pending.sender.is_some()
            })
            .map(|pending| McpSamplingApprovalSummary {
                runtime_id: pending.runtime_id.clone(),
                correlation_id: pending.correlation_id.clone(),
                prompt_id: pending.prompt_id.clone(),
                server_id: pending.server_id.clone(),
                provider_id: pending.provider_id.clone(),
                model_id: pending.model_id.clone(),
                max_tokens: pending.max_tokens,
                expires_at_ms: pending.expires_at_ms,
                message_count: pending.message_count,
                input_bytes: pending.input_bytes,
                has_system_prompt: pending.has_system_prompt,
                parent_operation: pending.parent_operation.clone(),
            })
            .collect();
        Ok(Some((state.revision, summaries)))
    }

    pub(crate) fn contains_prompt(&self, prompt_id: &str) -> bool {
        self.lock_state().pending.contains_key(prompt_id)
    }

    pub(crate) fn stable_revision(&self) -> Option<u64> {
        let state = self.lock_state();
        (state.active_transitions == 0).then_some(state.revision)
    }

    // These exact identities are intentionally separate so callers cannot
    // substitute an opaque context bundle at the approval boundary.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn answer(
        &self,
        runtime: &RuntimeApplicationService,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        correlation_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<bool, McpError> {
        let _transition = self.begin_transition();
        let run_id = {
            let mut state = self.lock_state();
            let Some(pending) = state.pending.get_mut(prompt_id) else {
                return Ok(false);
            };
            if pending.runtime_id != runtime_id
                || pending.correlation_id != correlation_id
                || pending.prompt_id != prompt_id
            {
                return Err(McpError::Denied);
            }
            if pending.cancel_requested {
                return Err(McpError::Cancelled);
            }
            if pending.settling || pending.sender.is_none() {
                return Err(McpError::Conflict);
            }
            pending.settling = true;
            pending.run_id.clone()
        };
        let response = match runtime.answer_direct_approval_expected(
            expected_coordinator_generation,
            prompt_id,
            answer,
            now_ms,
        ) {
            Ok(response) => response,
            Err(RuntimeApplicationError::Generation { .. }) => {
                if let Some(pending) = self.lock_state().pending.get_mut(prompt_id) {
                    pending.settling = false;
                }
                return Err(McpError::Conflict);
            }
            Err(RuntimeApplicationError::Gateway(ActionGatewayError::Approval(
                ApprovalQueueError::Expired,
            ))) => {
                self.cancel(runtime, prompt_id, now_ms.saturating_add(1));
                return Err(McpError::TimedOut);
            }
            Err(_) => {
                if let Some(pending) = self.lock_state().pending.get_mut(prompt_id) {
                    pending.settling = false;
                }
                return Err(McpError::StateUnavailable);
            }
        };
        let decision = match response {
            ApprovalResponse::Authorized { token, .. } => {
                SamplingApprovalDecision::Authorized(token)
            }
            ApprovalResponse::Denied { .. } => SamplingApprovalDecision::Denied,
        };
        let (sender, cancel_requested) = {
            let mut state = self.lock_state();
            let Some(pending) = state.pending.get_mut(prompt_id) else {
                let _ = runtime.cancel_direct_action_run(&run_id, now_ms.saturating_add(1));
                return Ok(true);
            };
            (pending.sender.take(), pending.cancel_requested)
        };
        if cancel_requested {
            drop(sender);
            self.cancel(runtime, prompt_id, now_ms.saturating_add(1));
            return Ok(true);
        }
        let Some(sender) = sender else {
            let _ = runtime.cancel_direct_action_run(&run_id, now_ms.saturating_add(1));
            return Err(McpError::StateUnavailable);
        };
        match sender.send(decision) {
            Ok(()) => {
                self.lock_state().pending.remove(prompt_id);
            }
            Err(SamplingApprovalDecision::Authorized(_)) => {
                {
                    let mut state = self.lock_state();
                    if let Some(pending) = state.pending.get_mut(prompt_id) {
                        pending.cancel_requested = true;
                        pending.settling = false;
                    }
                }
                self.cancel(runtime, prompt_id, now_ms.saturating_add(1));
            }
            Err(SamplingApprovalDecision::Denied) => {
                self.lock_state().pending.remove(prompt_id);
            }
        }
        Ok(true)
    }

    fn cancel(&self, runtime: &RuntimeApplicationService, prompt_id: &str, now_ms: u64) -> bool {
        let _transition = self.begin_transition();
        let run_id = {
            let mut state = self.lock_state();
            let Some(pending) = state.pending.get_mut(prompt_id) else {
                return true;
            };
            pending.cancel_requested = true;
            pending.run_id.clone()
        };
        if runtime.cancel_direct_action_run(&run_id, now_ms).is_err() {
            return false;
        }
        let Some(mut pending) = self.lock_state().pending.remove(prompt_id) else {
            return true;
        };
        if let Some(sender) = pending.sender.take() {
            let _ = sender.send(SamplingApprovalDecision::Denied);
        }
        true
    }
}

impl Drop for SamplingApprovalReservation {
    fn drop(&mut self) {
        if self.active {
            let mut state = self.registry.lock_state();
            state.reserved = state.reserved.saturating_sub(1);
        }
    }
}

impl Drop for SamplingApprovalTransition {
    fn drop(&mut self) {
        if self.active {
            let mut state = self.registry.lock_state();
            state.active_transitions = state.active_transitions.saturating_sub(1);
            state.revision = state.revision.wrapping_add(1);
            self.active = false;
        }
    }
}

impl SamplingApprovalWaitGuard {
    fn new(
        registry: ProductionMcpSamplingApprovalRegistry,
        runtime: Arc<RuntimeApplicationService>,
        prompt_id: String,
    ) -> Self {
        Self {
            registry,
            runtime,
            prompt_id,
            armed: true,
        }
    }

    fn cancel(&mut self, now_ms: u64) {
        if self.armed {
            if !self.registry.cancel(&self.runtime, &self.prompt_id, now_ms) {
                schedule_sampling_approval_cancellation(
                    self.registry.clone(),
                    Arc::clone(&self.runtime),
                    self.prompt_id.clone(),
                );
            }
            self.armed = false;
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for SamplingApprovalWaitGuard {
    fn drop(&mut self) {
        if self.armed
            && !self.registry.cancel(
                &self.runtime,
                &self.prompt_id,
                current_time_ms().unwrap_or(1),
            )
        {
            schedule_sampling_approval_cancellation(
                self.registry.clone(),
                Arc::clone(&self.runtime),
                self.prompt_id.clone(),
            );
        }
    }
}

fn schedule_sampling_approval_cancellation(
    registry: ProductionMcpSamplingApprovalRegistry,
    runtime: Arc<RuntimeApplicationService>,
    prompt_id: String,
) {
    tauri::async_runtime::spawn(async move {
        let mut delay_ms = 50u64;
        loop {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            if registry.cancel(&runtime, &prompt_id, current_time_ms().unwrap_or(1)) {
                break;
            }
            delay_ms = delay_ms.saturating_mul(2).min(30_000);
        }
    });
}

struct PendingSamplingCompletion {
    lease: ActionEffectLease,
    result: NormalizedActionResult,
}

#[derive(Default)]
struct SamplingCompletionState {
    reserved: usize,
    pending: VecDeque<PendingSamplingCompletion>,
}

trait SamplingEffectCompleter: Send + Sync {
    fn complete_retryable(
        &self,
        lease: &mut ActionEffectLease,
        result: &NormalizedActionResult,
    ) -> Result<(), ()>;
}

#[derive(Clone)]
struct SamplingCompletionManager {
    completer: Arc<dyn SamplingEffectCompleter>,
    state: Arc<Mutex<SamplingCompletionState>>,
    retrying: Arc<AtomicBool>,
}

struct SamplingCompletionReservation {
    manager: SamplingCompletionManager,
    active: bool,
}

struct SamplingEffectSettlement {
    reservation: Option<SamplingCompletionReservation>,
    lease: Option<ActionEffectLease>,
}

struct SamplingCancellationSignal {
    cancelled: Arc<AtomicBool>,
    armed: bool,
}

pub(crate) struct ProductionMcpSamplingBroker {
    runtime: Arc<RuntimeApplicationService>,
    parents: McpSamplingParentRegistry,
    approvals: ProductionMcpSamplingApprovalRegistry,
    completions: SamplingCompletionManager,
}

impl ProductionMcpSamplingBroker {
    pub(crate) fn new(runtime: Arc<RuntimeApplicationService>) -> Self {
        let completer: Arc<dyn SamplingEffectCompleter> = runtime.clone();
        Self {
            completions: SamplingCompletionManager::new(completer),
            runtime,
            parents: McpSamplingParentRegistry::default(),
            approvals: ProductionMcpSamplingApprovalRegistry::default(),
        }
    }

    pub(crate) fn parents(&self) -> McpSamplingParentRegistry {
        self.parents.clone()
    }

    pub(crate) fn approvals(&self) -> ProductionMcpSamplingApprovalRegistry {
        self.approvals.clone()
    }
}

impl SamplingEffectCompleter for RuntimeApplicationService {
    fn complete_retryable(
        &self,
        lease: &mut ActionEffectLease,
        result: &NormalizedActionResult,
    ) -> Result<(), ()> {
        self.complete_direct_action_effect_retryable(lease, result)
            .map_err(|_| ())
    }
}

impl SamplingCompletionManager {
    fn new(completer: Arc<dyn SamplingEffectCompleter>) -> Self {
        Self {
            completer,
            state: Arc::new(Mutex::new(SamplingCompletionState::default())),
            retrying: Arc::new(AtomicBool::new(false)),
        }
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, SamplingCompletionState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn reserve(&self) -> Result<SamplingCompletionReservation, McpError> {
        let mut state = self.lock_state();
        if state.reserved >= MAX_PENDING_SAMPLING_COMPLETIONS {
            return Err(McpError::StateUnavailable);
        }
        state.reserved += 1;
        Ok(SamplingCompletionReservation {
            manager: self.clone(),
            active: true,
        })
    }

    fn release(&self) {
        let mut state = self.lock_state();
        state.reserved = state.reserved.saturating_sub(1);
    }

    fn retain_reserved(&self, lease: ActionEffectLease, result: NormalizedActionResult) {
        let mut state = self.lock_state();
        debug_assert!(state.reserved > state.pending.len());
        state
            .pending
            .push_back(PendingSamplingCompletion { lease, result });
        drop(state);
        self.schedule_retry();
    }

    fn pump(&self) -> Result<(), McpError> {
        loop {
            let pending = self.lock_state().pending.pop_front();
            let Some(mut pending) = pending else {
                return Ok(());
            };
            if self
                .completer
                .complete_retryable(&mut pending.lease, &pending.result)
                .is_err()
            {
                self.lock_state().pending.push_front(pending);
                self.schedule_retry();
                return Err(McpError::StateUnavailable);
            }
            self.release();
        }
    }

    fn schedule_retry(&self) {
        if self.retrying.swap(true, Ordering::AcqRel) {
            return;
        }
        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            let mut delay_ms = 50u64;
            loop {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                let _ = manager.pump();
                if manager.lock_state().pending.is_empty() {
                    break;
                }
                delay_ms = delay_ms.saturating_mul(2).min(30_000);
            }
            manager.retrying.store(false, Ordering::Release);
            if !manager.lock_state().pending.is_empty() {
                manager.schedule_retry();
            }
        });
    }
}

impl SamplingCompletionReservation {
    fn attach(mut self, lease: ActionEffectLease) -> SamplingEffectSettlement {
        self.active = false;
        SamplingEffectSettlement {
            reservation: Some(self),
            lease: Some(lease),
        }
    }
}

impl Drop for SamplingCompletionReservation {
    fn drop(&mut self) {
        if self.active {
            self.manager.release();
        }
    }
}

impl SamplingEffectSettlement {
    fn settle(mut self, result: NormalizedActionResult) -> Result<(), McpError> {
        let mut lease = self
            .lease
            .take()
            .expect("sampling settlement owns its effect lease");
        if self
            .reservation
            .as_ref()
            .expect("sampling settlement owns its capacity reservation")
            .manager
            .completer
            .complete_retryable(&mut lease, &result)
            .is_err()
        {
            let reservation = self
                .reservation
                .take()
                .expect("sampling settlement retains its reservation");
            reservation.manager.retain_reserved(lease, result);
            return Err(McpError::StateUnavailable);
        }
        let mut reservation = self
            .reservation
            .take()
            .expect("sampling settlement releases its reservation");
        reservation.active = true;
        drop(reservation);
        Ok(())
    }
}

impl Drop for SamplingEffectSettlement {
    fn drop(&mut self) {
        let (Some(lease), Some(reservation)) = (self.lease.take(), self.reservation.take()) else {
            return;
        };
        let completed_at_ms = current_time_ms().unwrap_or(1);
        let result = NormalizedActionResult {
            status: NormalizedActionStatus::UnknownAfterInterruption,
            result_code: "mcp-sampling-interrupted-after-effect-start".into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms: completed_at_ms.max(1),
        };
        reservation.manager.retain_reserved(lease, result);
    }
}

impl SamplingCancellationSignal {
    fn new(cancelled: Arc<AtomicBool>) -> Self {
        Self {
            cancelled,
            armed: true,
        }
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for SamplingCancellationSignal {
    fn drop(&mut self) {
        if self.armed {
            self.cancel();
        }
    }
}

fn sampling_execution_outcome(
    peer_result: Result<PeerSamplingResult, RuntimeApplicationError>,
    cancelled: bool,
    context: &McpSamplingContext,
    completed_at_ms: u64,
) -> (
    NormalizedActionResult,
    Result<CreateMessageResult, McpError>,
) {
    match peer_result {
        Ok(_) if cancelled => sampling_cancelled_outcome(
            "mcp-sampling-cancelled",
            McpError::Cancelled,
            completed_at_ms,
        ),
        Ok(result) => {
            let response = CreateMessageResult::new(
                SamplingMessage::assistant_text(result.text.clone()),
                result.model_id,
            )
            .with_stop_reason(result.stop_reason);
            if u64::try_from(result.text.len()).unwrap_or(u64::MAX) > context.max_output_bytes {
                return sampling_failed_outcome(
                    "mcp-sampling-output-bound-exceeded",
                    McpError::BoundExceeded,
                    completed_at_ms,
                );
            }
            if super::transport::validate_sampling_result(&response).is_err() {
                return sampling_failed_outcome(
                    "mcp-sampling-result-invalid",
                    McpError::InvalidState,
                    completed_at_ms,
                );
            }
            let output_sha256 =
                crate::runtime::opencode_native::sha256_bytes(result.text.as_bytes());
            (
                NormalizedActionResult {
                    status: NormalizedActionStatus::Succeeded,
                    result_code: "mcp-sampling-completed".into(),
                    exit_code: None,
                    changed_targets: Vec::new(),
                    output_sha256: Some(output_sha256),
                    completed_at_ms,
                },
                Ok(response),
            )
        }
        Err(RuntimeApplicationError::Dispatch(DispatchError::Peer(
            PeerDispatchError::Cancellation,
        ))) if cancelled => sampling_cancelled_outcome(
            "mcp-sampling-cancelled",
            McpError::Cancelled,
            completed_at_ms,
        ),
        Err(RuntimeApplicationError::Dispatch(DispatchError::Peer(
            PeerDispatchError::Cancellation,
        ))) => sampling_cancelled_outcome(
            "mcp-sampling-timed-out",
            McpError::TimedOut,
            completed_at_ms,
        ),
        Err(_) => sampling_unknown_outcome("mcp-sampling-runtime-ambiguous", completed_at_ms),
    }
}

fn sampling_cancelled_outcome(
    result_code: &str,
    error: McpError,
    completed_at_ms: u64,
) -> (
    NormalizedActionResult,
    Result<CreateMessageResult, McpError>,
) {
    (
        NormalizedActionResult {
            status: NormalizedActionStatus::Cancelled,
            result_code: result_code.into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        },
        Err(error),
    )
}

fn sampling_failed_outcome(
    result_code: &str,
    error: McpError,
    completed_at_ms: u64,
) -> (
    NormalizedActionResult,
    Result<CreateMessageResult, McpError>,
) {
    (
        NormalizedActionResult {
            status: NormalizedActionStatus::Failed,
            result_code: result_code.into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        },
        Err(error),
    )
}

fn sampling_unknown_outcome(
    result_code: &str,
    completed_at_ms: u64,
) -> (
    NormalizedActionResult,
    Result<CreateMessageResult, McpError>,
) {
    (
        NormalizedActionResult {
            status: NormalizedActionStatus::UnknownAfterInterruption,
            result_code: result_code.into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        },
        Err(McpError::StateUnavailable),
    )
}

impl McpSamplingBroker for ProductionMcpSamplingBroker {
    fn sample<'a>(
        &'a self,
        context: &'a McpSamplingContext,
        request: CreateMessageRequestParams,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<CreateMessageResult, McpError>> {
        Box::pin(async move {
            let operation_deadline =
                tokio::time::Instant::now() + Duration::from_millis(context.timeout_ms);
            self.completions.pump()?;
            if cancellation.is_cancelled() {
                return Err(McpError::Cancelled);
            }
            let parent_action = self.parents.parent(context)?;
            let request_bytes = serde_json::to_vec(&request).map_err(|_| McpError::InvalidInput)?;
            let request_sha256 = crate::runtime::opencode_native::sha256_bytes(&request_bytes);
            let messages = sampling_text_messages(&request)?
                .into_iter()
                .map(|(role, text)| PiSamplingMessage { role, text })
                .collect();
            let cancelled = Arc::new(AtomicBool::new(false));
            let intent = RuntimeMcpSamplingIntent {
                context: context.clone(),
                parent_action,
                messages,
                system_prompt: request.system_prompt.clone(),
                max_tokens: request.max_tokens,
                temperature: request.temperature,
                request_sha256,
                sampling_id: Uuid::new_v4().as_simple().to_string(),
                cancelled: Arc::clone(&cancelled),
            };
            let proposed_at_ms = current_time_ms().map_err(|_| McpError::StateUnavailable)?;
            let prepared = self
                .runtime
                .prepare_mcp_sampling(&intent, proposed_at_ms)
                .map_err(|_| McpError::NotReady)?;
            let mut approval_reservation = self.approvals.reserve()?;
            let mut approval_transition = Some(self.approvals.begin_transition());
            let (token, approval_prompt_id) = match self
                .runtime
                .propose_direct_sampling_confirmation(
                    &prepared.facts,
                    prepared.action.clone(),
                    proposed_at_ms,
                )
                .map_err(|_| McpError::StateUnavailable)?
            {
                GatewayProposal::Denied { .. } => return Err(McpError::Denied),
                GatewayProposal::Authorized { token, .. } => (token, None),
                GatewayProposal::PendingApproval { prompt, .. } => {
                    let prompt_id = prompt.prompt_id.clone();
                    let (sender, receiver) = oneshot::channel();
                    if let Err(error) = self.approvals.register(
                        &mut approval_reservation,
                        prepared.action.runtime_id.clone(),
                        prepared.peer.identity.correlation_id.clone(),
                        prompt_id.clone(),
                        prepared.action.run_id.clone(),
                        context.server_id.clone(),
                        prepared.peer.model.provider_id.clone(),
                        prepared.peer.model.model_id.clone(),
                        prepared.peer.max_tokens,
                        prompt.expires_at_ms,
                        prepared.peer.messages.len(),
                        prepared
                            .peer
                            .messages
                            .iter()
                            .map(|message| message.text.len())
                            .sum::<usize>()
                            .saturating_add(
                                prepared.peer.system_prompt.as_deref().map_or(0, str::len),
                            ),
                        prepared.peer.system_prompt.is_some(),
                        intent.parent_action.tool.clone(),
                        sender,
                    ) {
                        let _ = self.runtime.cancel_direct_action_run(
                            &prepared.action.run_id,
                            proposed_at_ms.saturating_add(1),
                        );
                        return Err(error);
                    }
                    drop(approval_transition.take());
                    let mut approval_guard = SamplingApprovalWaitGuard::new(
                        self.approvals.clone(),
                        Arc::clone(&self.runtime),
                        prompt_id.clone(),
                    );
                    let approval_wait_ms = prompt
                        .expires_at_ms
                        .saturating_sub(proposed_at_ms)
                        .min(context.timeout_ms);
                    let decision = tokio::select! {
                        biased;
                        _ = cancellation.cancelled() => {
                            cancelled.store(true, Ordering::SeqCst);
                            approval_guard.cancel(
                                current_time_ms().unwrap_or(proposed_at_ms.saturating_add(1)),
                            );
                            return Err(McpError::Cancelled);
                        }
                        decision = receiver => match decision {
                            Ok(decision) => decision,
                            Err(_) => {
                                approval_guard.cancel(
                                    current_time_ms()
                                        .unwrap_or(proposed_at_ms.saturating_add(1)),
                                );
                                return Err(McpError::Cancelled);
                            }
                        },
                        _ = tokio::time::sleep(Duration::from_millis(approval_wait_ms)) => {
                            cancelled.store(true, Ordering::SeqCst);
                            approval_guard.cancel(
                                current_time_ms().unwrap_or(prompt.expires_at_ms),
                            );
                            return Err(McpError::TimedOut);
                        }
                    };
                    approval_guard.disarm();
                    match decision {
                        SamplingApprovalDecision::Authorized(token) => {
                            if cancellation.is_cancelled() {
                                cancelled.store(true, Ordering::SeqCst);
                                let _ = self.runtime.cancel_direct_action_run(
                                    &prepared.action.run_id,
                                    current_time_ms().unwrap_or(proposed_at_ms.saturating_add(1)),
                                );
                                return Err(McpError::Cancelled);
                            }
                            (token, Some(prompt_id))
                        }
                        SamplingApprovalDecision::Denied => return Err(McpError::Denied),
                    }
                }
            };
            drop(approval_transition.take());
            drop(approval_reservation);

            let authorized_at_ms = current_time_ms().map_err(|_| McpError::StateUnavailable)?;
            let revalidated = self
                .runtime
                .prepare_mcp_sampling(&intent, authorized_at_ms)
                .map_err(|_| McpError::Conflict)?;
            if prepared.action != revalidated.action
                || prepared.peer.identity != revalidated.peer.identity
                || prepared.peer.model != revalidated.peer.model
            {
                let _ = self
                    .runtime
                    .cancel_direct_action_run(&prepared.action.run_id, authorized_at_ms);
                return Err(McpError::Conflict);
            }
            if cancellation.is_cancelled() {
                cancelled.store(true, Ordering::SeqCst);
                let _ = self
                    .runtime
                    .cancel_direct_action_run(&prepared.action.run_id, authorized_at_ms);
                return Err(McpError::Cancelled);
            }
            let remaining_timeout_ms = u64::try_from(
                operation_deadline
                    .saturating_duration_since(tokio::time::Instant::now())
                    .as_millis(),
            )
            .unwrap_or(u64::MAX);
            if remaining_timeout_ms == 0 {
                cancelled.store(true, Ordering::SeqCst);
                let _ = self
                    .runtime
                    .cancel_direct_action_run(&prepared.action.run_id, authorized_at_ms);
                return Err(McpError::TimedOut);
            }
            let reservation = match self.completions.reserve() {
                Ok(reservation) => reservation,
                Err(error) => {
                    let _ = self
                        .runtime
                        .cancel_direct_action_run(&prepared.action.run_id, authorized_at_ms);
                    return Err(error);
                }
            };
            let lease = self
                .runtime
                .begin_direct_action_effect(
                    &token,
                    &prepared.action,
                    revalidated.live,
                    approval_prompt_id.as_deref(),
                    authorized_at_ms,
                )
                .map_err(|_| McpError::Denied)?;
            let settlement = reservation.attach(lease);
            let runtime = Arc::clone(&self.runtime);
            let mut peer = prepared.peer.clone();
            peer.timeout_ms = remaining_timeout_ms;
            let task_context = context.clone();
            let task_cancelled = Arc::clone(&cancelled);
            let (sender, mut receiver) = oneshot::channel();
            tauri::async_runtime::spawn(async move {
                let job = tokio::task::spawn_blocking(move || runtime.execute_mcp_sampling(&peer));
                let completed_at_ms = current_time_ms()
                    .unwrap_or(authorized_at_ms.saturating_add(1))
                    .max(authorized_at_ms.saturating_add(1));
                let (normalized, response) = match job.await {
                    Ok(peer_result) => sampling_execution_outcome(
                        peer_result,
                        task_cancelled.load(Ordering::SeqCst),
                        &task_context,
                        completed_at_ms,
                    ),
                    Err(_) => sampling_unknown_outcome(
                        "mcp-sampling-runtime-interrupted",
                        completed_at_ms,
                    ),
                };
                let response = match settlement.settle(normalized) {
                    Ok(()) => response,
                    Err(error) => Err(error),
                };
                let _ = sender.send(response);
            });
            let mut cancellation_signal = SamplingCancellationSignal::new(cancelled);
            let response = tokio::select! {
                biased;
                _ = cancellation.cancelled() => {
                    cancellation_signal.cancel();
                    return Err(McpError::Cancelled);
                }
                _ = tokio::time::sleep_until(operation_deadline) => {
                    cancellation_signal.cancel();
                    return Err(McpError::TimedOut);
                }
                response = &mut receiver => response.map_err(|_| McpError::StateUnavailable)?,
            };
            cancellation_signal.disarm();
            response
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use tempfile::TempDir;

    use crate::{
        core::database::{DatabaseActor, DatabaseDescriptor},
        security::{
            authorization::{CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk},
            gateway::{ActionGateway, GatewayProposal},
            policy::{
                ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin,
                ActionReversibility, ActionScope, ActionSensitivity, ActionSurface, ApprovalPreset,
                ClassificationConfidence, PolicyConfiguration, RepositoryState,
            },
        },
    };

    use super::*;

    struct ScriptedCompleter {
        _temporary: TempDir,
        database_path: std::path::PathBuf,
        gateway: Mutex<ActionGateway>,
        failures: AtomicUsize,
        completed: Mutex<Vec<NormalizedActionResult>>,
    }

    impl SamplingEffectCompleter for ScriptedCompleter {
        fn complete_retryable(
            &self,
            lease: &mut ActionEffectLease,
            result: &NormalizedActionResult,
        ) -> Result<(), ()> {
            if self
                .failures
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |failures| {
                    failures.checked_sub(1)
                })
                .is_ok()
            {
                return Err(());
            }
            self.gateway
                .lock()
                .unwrap()
                .complete_effect_retryable(lease, result)
                .map_err(|_| ())?;
            self.completed.lock().unwrap().push(result.clone());
            Ok(())
        }
    }

    fn effect_fixture(
        failures: usize,
    ) -> (Arc<ScriptedCompleter>, ActionEffectLease, CanonicalAction) {
        let temporary = TempDir::new().unwrap();
        let descriptor = DatabaseDescriptor::app(temporary.path());
        let database_path = descriptor.path.clone();
        let (database, _) = DatabaseActor::start(descriptor).unwrap();
        let mut gateway = ActionGateway::new(
            PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                ..PolicyConfiguration::default()
            },
            Arc::new(database),
        );
        let action = CanonicalAction {
            schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
            action_id: "sampling-effect-action".into(),
            tool_call_id: "sampling-effect-call".into(),
            tool: "mcp.sampling.create-message".into(),
            arguments: serde_json::json!({ "requestSha256": format!("sha256:{}", "a".repeat(64)) }),
            risk: CanonicalRisk::Low,
            requested_authority: BTreeSet::from(["network.execute".into()]),
            canonical_target: "mcp-sampling:fixture".into(),
            target_version: format!("sha256:{}", "b".repeat(64)),
            workspace_id: "workspace-1".into(),
            session_id: "session-1".into(),
            run_id: "sampling-effect-run".into(),
            runtime_id: "pi-primary".into(),
            environment_id: "local".into(),
            plugin_or_mcp_id: Some("mcp-fixture".into()),
            process_generation: 7,
            configuration_version: 3,
            policy_version: 1,
            revocation_epoch: 1,
        };
        let facts = ActionFacts {
            action_kind: action.tool.clone(),
            native_tool: action.tool.clone(),
            surface: ActionSurface::Network,
            effects: BTreeSet::from([ActionEffect::Read]),
            scope: ActionScope::Remote,
            initiator: ActionInitiator::McpServer,
            sensitivity: ActionSensitivity::Ordinary,
            reversibility: ActionReversibility::Reversible,
            confidence: ClassificationConfidence::Known,
            request_origin: ActionRequestOrigin::McpSampling,
            repository_state: RepositoryState::NotApplicable,
            inside_active_project: true,
            canonical_target: action.canonical_target.clone(),
            workspace_id: action.workspace_id.clone(),
            session_id: action.session_id.clone(),
            runtime_id: action.runtime_id.clone(),
            environment_id: action.environment_id.clone(),
            plugin_or_mcp_id: action.plugin_or_mcp_id.clone(),
            target_resolved: true,
            authenticated: true,
            trusted_root: true,
            explicit_scope_grant: true,
            sandbox_allows: true,
            declaration_exceeded: false,
        };
        let prompt = match gateway
            .propose_sampling_confirmation(&facts, action.clone(), 10)
            .unwrap()
        {
            GatewayProposal::PendingApproval { prompt, .. } => prompt,
            proposal => panic!("fixture must require sampling approval: {proposal:?}"),
        };
        let token = match gateway
            .answer_approval(&prompt.prompt_id, ApprovalAnswer::Allow, 11)
            .unwrap()
        {
            ApprovalResponse::Authorized { token, .. } => token,
            response => panic!("fixture must authorize sampling: {response:?}"),
        };
        let lease = gateway
            .begin_effect(
                &token,
                &action,
                crate::security::authorization::LiveAuthorityState {
                    process_generation: action.process_generation,
                    configuration_version: action.configuration_version,
                    policy_version: action.policy_version,
                    revocation_epoch: action.revocation_epoch,
                },
                Some(&prompt.prompt_id),
                12,
            )
            .unwrap();
        (
            Arc::new(ScriptedCompleter {
                _temporary: temporary,
                database_path,
                gateway: Mutex::new(gateway),
                failures: AtomicUsize::new(failures),
                completed: Mutex::new(Vec::new()),
            }),
            lease,
            action,
        )
    }

    #[tokio::test]
    async fn dropped_effect_settlement_retains_unknown_and_retries_without_a_new_request() {
        let (completer, lease, _) = effect_fixture(1);
        let manager = SamplingCompletionManager::new(completer.clone());
        let settlement = manager.reserve().unwrap().attach(lease);

        drop(settlement);
        assert_eq!(manager.lock_state().reserved, 1);
        assert_eq!(manager.lock_state().pending.len(), 1);

        tokio::time::sleep(Duration::from_millis(250)).await;
        assert_eq!(manager.lock_state().reserved, 0);
        assert!(manager.lock_state().pending.is_empty());
        assert_eq!(
            completer.completed.lock().unwrap()[0].status,
            NormalizedActionStatus::UnknownAfterInterruption
        );
    }

    #[test]
    fn completion_capacity_is_reserved_before_any_effect_can_begin() {
        let (completer, _, _) = effect_fixture(0);
        let manager = SamplingCompletionManager::new(completer);
        let reservations = (0..MAX_PENDING_SAMPLING_COMPLETIONS)
            .map(|_| manager.reserve().unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(manager.reserve(), Err(McpError::StateUnavailable)));
        drop(reservations);
        assert_eq!(manager.lock_state().reserved, 0);
    }

    #[tokio::test]
    async fn failed_terminal_write_keeps_the_only_lease_until_the_retry_pump_settles_it() {
        let (completer, lease, _) = effect_fixture(1);
        let manager = SamplingCompletionManager::new(completer.clone());
        let settlement = manager.reserve().unwrap().attach(lease);
        let result = NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "mcp-sampling-completed".into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: Some(format!("sha256:{}", "c".repeat(64))),
            completed_at_ms: 12,
        };

        assert!(matches!(
            settlement.settle(result),
            Err(McpError::StateUnavailable)
        ));
        assert_eq!(manager.lock_state().reserved, 1);
        assert_eq!(manager.lock_state().pending.len(), 1);
        tokio::time::sleep(Duration::from_millis(125)).await;
        assert!(manager.lock_state().pending.is_empty());
        assert_eq!(manager.lock_state().reserved, 0);
        assert_eq!(
            completer.completed.lock().unwrap()[0].status,
            NormalizedActionStatus::Succeeded
        );
    }

    #[tokio::test]
    async fn real_storage_failure_leaves_approval_retryable_until_the_atomic_batch_commits() {
        let (completer, lease, _) = effect_fixture(0);
        let connection = rusqlite::Connection::open(&completer.database_path).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER fail_sampling_terminal
                 BEFORE INSERT ON security_events
                 WHEN NEW.record_kind = 'approval-prompt' AND NEW.state = 'completed'
                 BEGIN SELECT RAISE(ABORT, 'injected sampling terminal failure'); END;",
            )
            .unwrap();
        let manager = SamplingCompletionManager::new(completer.clone());
        let settlement = manager.reserve().unwrap().attach(lease);
        let result = NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "mcp-sampling-completed".into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: Some(format!("sha256:{}", "d".repeat(64))),
            completed_at_ms: 13,
        };

        assert!(matches!(
            settlement.settle(result),
            Err(McpError::StateUnavailable)
        ));
        assert_eq!(manager.lock_state().reserved, 1);
        assert_eq!(manager.lock_state().pending.len(), 1);
        connection
            .execute_batch("DROP TRIGGER fail_sampling_terminal;")
            .unwrap();

        tokio::time::sleep(Duration::from_millis(250)).await;
        assert!(manager.lock_state().pending.is_empty());
        assert_eq!(manager.lock_state().reserved, 0);
        assert_eq!(completer.completed.lock().unwrap().len(), 1);
    }

    #[test]
    fn approval_capacity_is_reserved_before_prompt_creation_and_summary_is_redacted() {
        let registry = ProductionMcpSamplingApprovalRegistry::default();
        let mut reservations = (0..MAX_PENDING_SAMPLING_APPROVALS)
            .map(|_| registry.reserve().unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(
            registry.reserve(),
            Err(McpError::StateUnavailable)
        ));

        let mut reservation = reservations.pop().unwrap();
        let (sender, _receiver) = oneshot::channel();
        registry
            .register(
                &mut reservation,
                "pi-primary".into(),
                "sampling-correlation".into(),
                "approval:sampling".into(),
                "sampling-run".into(),
                "server-1".into(),
                "provider-openai".into(),
                "gpt-4o-mini".into(),
                64,
                current_time_ms().unwrap().saturating_add(1_000),
                2,
                128,
                true,
                "c4os_propose_action".into(),
                sender,
            )
            .unwrap();
        let (_, summaries) = registry.stable_summaries().unwrap().unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].server_id, "server-1");
        assert_eq!(summaries[0].model_id, "gpt-4o-mini");
        assert_eq!(summaries[0].input_bytes, 128);
        assert!(!format!("{summaries:?}").contains("private prompt canary"));
    }

    #[test]
    fn approval_snapshot_is_hidden_for_the_full_coordinator_registry_transition() {
        let registry = ProductionMcpSamplingApprovalRegistry::default();
        let initial_revision = registry.stable_revision().unwrap();
        let transition = registry.begin_transition();
        assert!(registry.stable_revision().is_none());
        assert!(registry.stable_summaries().unwrap().is_none());
        drop(transition);
        assert!(registry.stable_revision().unwrap() > initial_revision);
        assert!(registry.stable_summaries().unwrap().is_some());
    }

    #[test]
    fn parent_registry_requires_the_exact_server_definition_and_lifecycle_binding() {
        let (_, _, mut action) = effect_fixture(0);
        action.plugin_or_mcp_id = Some("mcp-server-1".into());
        action.arguments = serde_json::json!({
            "resolved": {
                "serverId": "server-1",
                "definitionSha256": format!("sha256:{}", "d".repeat(64)),
                "lifecycleGeneration": 7,
            }
        });
        let registry = McpSamplingParentRegistry::default();
        let _guard = registry.register("server-1", action).unwrap();
        let exact = McpSamplingContext {
            server_id: "server-1".into(),
            authority_id: "mcp-server-1".into(),
            lifecycle_generation: 7,
            definition_sha256: format!("sha256:{}", "d".repeat(64)),
            timeout_ms: 1_000,
            max_output_bytes: 64 * 1_024,
        };
        assert!(registry.parent(&exact).is_ok());
        assert!(matches!(
            registry.parent(&McpSamplingContext {
                lifecycle_generation: 8,
                ..exact
            }),
            Err(McpError::Conflict)
        ));
    }
}
