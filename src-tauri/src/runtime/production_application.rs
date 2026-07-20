//! App-lifetime production runtime composition.

#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use thiserror::Error;

use crate::RuntimeApplicationService;
use crate::runtime::capability_evidence::CapabilityRouteEpoch;
use crate::runtime::dispatch::{DispatchError, DispatchEventCategory, RuntimeDispatchRegistry};
use crate::runtime::production::{
    CoreProductionWorker, PreparedCoreProductionPeer, ProductionProviderRoute,
    ProductionRuntimeBinding, RuntimeProductionBootstrap,
};
use crate::runtime::supervisor::HealthState;
use crate::security::authorization::ApprovalAnswer;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivatedProductionRuntime {
    pub coordinator_generation: u64,
    pub capability_generation: u64,
    pub authority_generation: u64,
    pub runtime_id: String,
    pub process_generation: u64,
    pub process_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingProductionRuntimeApproval {
    pub runtime_id: String,
    pub correlation_id: String,
    pub prompt_id: String,
}

const MAX_PUBLISHED_RUNTIME_APPROVALS: usize = 4_096;

#[derive(Debug, Error)]
pub enum RuntimeProductionApplicationError {
    #[error("production runtime application state is unavailable")]
    Unavailable,
    #[error("production runtime is already active")]
    AlreadyActive,
    #[error("production runtime is not active")]
    NotActive,
    #[error("production runtime preparation was rejected")]
    PreparationRejected,
    #[error("prepared production peer changed its core-owned binding")]
    BindingMismatch,
    #[error(transparent)]
    Application(#[from] crate::RuntimeApplicationError),
    #[error(transparent)]
    Dispatch(#[from] DispatchError),
    #[error(transparent)]
    Production(#[from] crate::runtime::production::RuntimeProductionError),
    #[error(transparent)]
    OpenCodeBrokerPump(#[from] crate::runtime::opencode_broker::OpenCodeBrokerPumpError),
    #[error("OpenCode broker response status is unknown; the native generation is fatal")]
    OpenCodeBrokerGenerationFatal(
        #[source] crate::runtime::opencode_broker::OpenCodeBrokerPumpError,
    ),
    #[error(
        "OpenCode broker response status is unknown; the native generation is quarantined because cleanup failed: {cleanup}"
    )]
    OpenCodeBrokerGenerationQuarantined {
        #[source]
        source: crate::runtime::opencode_broker::OpenCodeBrokerPumpError,
        cleanup: String,
    },
    #[error("Pi tool settlement is unresolved; the native generation is fatal")]
    PiToolSettlementGenerationFatal,
    #[error(
        "Pi tool settlement is unresolved; the native generation is quarantined because cleanup failed: {cleanup}"
    )]
    PiToolSettlementGenerationQuarantined { cleanup: String },
}

pub trait ProductionRuntimeBackend: Send + Sync {
    fn prepare(
        &self,
        binding: ProductionRuntimeBinding,
        provider_routes: &[ProductionProviderRoute],
        checked_at_ms: u64,
    ) -> Result<Box<dyn PreparedProductionRuntimePeer>, RuntimeProductionApplicationError>;
}

pub trait PreparedProductionRuntimePeer: Send {
    fn binding(&self) -> &ProductionRuntimeBinding;
    fn native_process_id(&self) -> Result<u32, RuntimeProductionApplicationError>;
    fn capability_epochs(
        &self,
    ) -> Result<Vec<CapabilityRouteEpoch>, RuntimeProductionApplicationError> {
        Ok(Vec::new())
    }
    fn install(
        self: Box<Self>,
        registry: &mut RuntimeDispatchRegistry,
    ) -> Result<Box<dyn ProductionRuntimeWorker>, RuntimeProductionApplicationError>;
}

pub trait ProductionRuntimeWorker: Send {
    fn binding(&self) -> &ProductionRuntimeBinding;

    fn revoke_transient_credentials(&mut self) {}

    fn pump(
        &mut self,
        application: &Arc<RuntimeApplicationService>,
        now_ms: u64,
    ) -> Result<usize, RuntimeProductionApplicationError>;

    fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn answer_approval(
        &mut self,
        _application: &Arc<RuntimeApplicationService>,
        _expected_coordinator_generation: u64,
        _correlation_id: &str,
        _prompt_id: &str,
        _answer: ApprovalAnswer,
        _answered_at_ms: u64,
    ) -> Result<(), RuntimeProductionApplicationError> {
        Err(RuntimeProductionApplicationError::PreparationRejected)
    }
}

struct ActiveProductionRuntime {
    worker: Box<dyn ProductionRuntimeWorker>,
    attached: bool,
    quarantined: bool,
    native_stopped: bool,
}

/// App-owned composition that binds verified production peers to the durable
/// supervisor and dispatch registry. `B` is the exact packaged bootstrap in
/// production and a deterministic process fixture in unit tests.
pub struct RuntimeProductionApplication<B> {
    application: Arc<RuntimeApplicationService>,
    backend: B,
    active: Mutex<BTreeMap<String, ActiveProductionRuntime>>,
}

impl<B: ProductionRuntimeBackend> RuntimeProductionApplication<B> {
    /// Constructs the app-owned host around an alternate Rust backend.
    ///
    /// Production uses [`RuntimeProductionBootstrap`]. The generic constructor
    /// remains renderer-inaccessible and exists so native integration tests can
    /// wrap the exact bootstrap to force generation races after process spawn.
    pub fn with_backend(application: Arc<RuntimeApplicationService>, backend: B) -> Self {
        Self {
            application,
            backend,
            active: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn active_runtime_count(&self) -> Result<usize, RuntimeProductionApplicationError> {
        self.active
            .lock()
            .map(|active| {
                active
                    .values()
                    .filter(|runtime| !runtime.quarantined)
                    .count()
            })
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)
    }

    pub fn activate_runtime(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        checked_at_ms: u64,
    ) -> Result<ActivatedProductionRuntime, RuntimeProductionApplicationError> {
        if self
            .active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?
            .contains_key(runtime_id)
        {
            return Err(RuntimeProductionApplicationError::AlreadyActive);
        }

        let reserved = self.application.reserve_managed_runtime_start(
            expected_coordinator_generation,
            runtime_id,
            checked_at_ms,
        )?;
        let process_generation = reserved.value;
        let (binding, provider_routes) = match self.application.production_runtime_preparation(
            runtime_id,
            process_generation,
            checked_at_ms,
        ) {
            Ok(binding) => binding,
            Err(error) => {
                self.abort_reservation(
                    reserved.coordinator_generation,
                    runtime_id,
                    process_generation,
                    checked_at_ms,
                );
                return Err(error.into());
            }
        };
        let prepared = match self
            .backend
            .prepare(binding.clone(), &provider_routes, checked_at_ms)
        {
            Ok(prepared) => prepared,
            Err(error) => {
                self.abort_reservation(
                    reserved.coordinator_generation,
                    runtime_id,
                    process_generation,
                    checked_at_ms,
                );
                return Err(error);
            }
        };
        if prepared.binding() != &binding {
            self.abort_reservation(
                reserved.coordinator_generation,
                runtime_id,
                process_generation,
                checked_at_ms,
            );
            return Err(RuntimeProductionApplicationError::BindingMismatch);
        }
        let process_id = match prepared.native_process_id() {
            Ok(process_id) if process_id != 0 => process_id,
            Ok(_) | Err(_) => {
                self.abort_reservation(
                    reserved.coordinator_generation,
                    runtime_id,
                    process_generation,
                    checked_at_ms,
                );
                return Err(RuntimeProductionApplicationError::PreparationRejected);
            }
        };
        let capability_epochs = match prepared.capability_epochs() {
            Ok(epochs) => epochs,
            Err(error) => {
                self.abort_reservation(
                    reserved.coordinator_generation,
                    runtime_id,
                    process_generation,
                    checked_at_ms,
                );
                return Err(error);
            }
        };
        let mut worker = {
            let mut registry = self.application.dispatch()?;
            match prepared.install(&mut registry) {
                Ok(worker) => worker,
                Err(error) => {
                    drop(registry);
                    self.abort_reservation(
                        reserved.coordinator_generation,
                        runtime_id,
                        process_generation,
                        checked_at_ms,
                    );
                    return Err(error);
                }
            }
        };
        let expected_capability_generation = self.application.capability_evidence_generation()?;
        let (attached, capability_generation, authority_generation) = match self
            .application
            .attach_managed_runtime_process_with_capabilities(
                reserved.coordinator_generation,
                expected_capability_generation,
                runtime_id,
                process_generation,
                process_id,
                HealthState::Healthy,
                capability_epochs,
                checked_at_ms,
            ) {
            Ok(attached) => attached,
            Err(error) => {
                let cleanup = self.application.dispatch().and_then(|mut registry| {
                    registry
                        .shutdown_and_unregister(runtime_id, process_generation)
                        .map_err(Into::into)
                });
                if let Err(cleanup_error) = cleanup {
                    worker.revoke_transient_credentials();
                    self.active
                        .lock()
                        .map_err(|_| RuntimeProductionApplicationError::Unavailable)?
                        .insert(
                            runtime_id.into(),
                            ActiveProductionRuntime {
                                worker,
                                attached: false,
                                quarantined: true,
                                native_stopped: false,
                            },
                        );
                    return Err(cleanup_error.into());
                }
                self.abort_reservation(
                    reserved.coordinator_generation,
                    runtime_id,
                    process_generation,
                    checked_at_ms,
                );
                return Err(error.into());
            }
        };
        if worker.binding() != &binding {
            if let Err(cleanup_error) = self.application.dispatch().and_then(|mut registry| {
                registry
                    .shutdown_and_unregister(runtime_id, process_generation)
                    .map_err(Into::into)
            }) {
                worker.revoke_transient_credentials();
                self.active
                    .lock()
                    .map_err(|_| RuntimeProductionApplicationError::Unavailable)?
                    .insert(
                        runtime_id.into(),
                        ActiveProductionRuntime {
                            worker,
                            attached: true,
                            quarantined: true,
                            native_stopped: false,
                        },
                    );
                return Err(cleanup_error.into());
            }
            let current_generation = self.application.snapshot(checked_at_ms)?.generation;
            self.application.finish_managed_runtime_shutdown(
                current_generation,
                runtime_id,
                process_generation,
                checked_at_ms,
            )?;
            return Err(RuntimeProductionApplicationError::BindingMismatch);
        }
        self.active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?
            .insert(
                runtime_id.into(),
                ActiveProductionRuntime {
                    worker,
                    attached: true,
                    quarantined: false,
                    native_stopped: false,
                },
            );
        Ok(ActivatedProductionRuntime {
            coordinator_generation: attached.coordinator_generation,
            capability_generation,
            authority_generation,
            runtime_id: runtime_id.into(),
            process_generation,
            process_id,
        })
    }

    pub fn pump_runtime_once(
        &self,
        runtime_id: &str,
        now_ms: u64,
    ) -> Result<usize, RuntimeProductionApplicationError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?;
        let runtime = active
            .get_mut(runtime_id)
            .ok_or(RuntimeProductionApplicationError::NotActive)?;
        if runtime.quarantined {
            return Err(RuntimeProductionApplicationError::NotActive);
        }
        match runtime.worker.pump(&self.application, now_ms) {
            Err(RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(source)) => {
                runtime.quarantined = true;
                let cleanup =
                    self.terminate_quarantined_generation(&mut active, runtime_id, now_ms);
                match cleanup {
                    Ok(()) => Err(
                        RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(source),
                    ),
                    Err(cleanup) => Err(
                        RuntimeProductionApplicationError::OpenCodeBrokerGenerationQuarantined {
                            source,
                            cleanup: cleanup.to_string(),
                        },
                    ),
                }
            }
            Err(RuntimeProductionApplicationError::PiToolSettlementGenerationFatal) => {
                runtime.quarantined = true;
                let cleanup =
                    self.terminate_quarantined_generation(&mut active, runtime_id, now_ms);
                match cleanup {
                    Ok(()) => {
                        Err(RuntimeProductionApplicationError::PiToolSettlementGenerationFatal)
                    }
                    Err(cleanup) => Err(
                        RuntimeProductionApplicationError::PiToolSettlementGenerationQuarantined {
                            cleanup: cleanup.to_string(),
                        },
                    ),
                }
            }
            result => result,
        }
    }

    pub fn pump_runtime_once_expected(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        now_ms: u64,
    ) -> Result<(u64, usize), RuntimeProductionApplicationError> {
        let reserved_generation = self
            .application
            .reserve_runtime_operation(expected_coordinator_generation, now_ms)?;
        let pumped = self.pump_runtime_once(runtime_id, now_ms)?;
        let coordinator_generation = self.application.snapshot(now_ms)?.generation;
        debug_assert!(coordinator_generation >= reserved_generation);
        Ok((coordinator_generation, pumped))
    }

    pub fn pump_all_once(&self, now_ms: u64) -> Result<usize, RuntimeProductionApplicationError> {
        let runtime_ids = self
            .active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?
            .iter()
            .filter_map(|(runtime_id, runtime)| {
                (!runtime.quarantined).then_some(runtime_id.clone())
            })
            .collect::<Vec<_>>();
        let mut total = 0_usize;
        let mut first_error = None;
        for runtime_id in runtime_ids {
            match self.pump_runtime_once(&runtime_id, now_ms) {
                Ok(pumped) => total = total.saturating_add(pumped),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }
        first_error.map_or(Ok(total), Err)
    }

    pub fn pending_approvals(
        &self,
    ) -> Result<Vec<PendingProductionRuntimeApproval>, RuntimeProductionApplicationError> {
        let active = self
            .active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?;
        let mut approvals = Vec::new();
        for (runtime_id, runtime) in active.iter().filter(|(_, runtime)| !runtime.quarantined) {
            for (correlation_id, prompt_id) in runtime.worker.pending_approval_descriptors() {
                if approvals.len() >= MAX_PUBLISHED_RUNTIME_APPROVALS {
                    return Err(RuntimeProductionApplicationError::Unavailable);
                }
                approvals.push(PendingProductionRuntimeApproval {
                    runtime_id: runtime_id.clone(),
                    correlation_id,
                    prompt_id,
                });
            }
        }
        Ok(approvals)
    }

    pub fn shutdown_runtime(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        stopped_at_ms: u64,
    ) -> Result<u64, RuntimeProductionApplicationError> {
        let (process_generation, attached, native_stopped) = {
            let active = self
                .active
                .lock()
                .map_err(|_| RuntimeProductionApplicationError::Unavailable)?;
            let active = active
                .get(runtime_id)
                .ok_or(RuntimeProductionApplicationError::NotActive)?;
            (
                active.worker.binding().process_generation(),
                active.attached,
                active.native_stopped,
            )
        };
        let reserved_generation = self
            .application
            .reserve_runtime_operation(expected_coordinator_generation, stopped_at_ms)?;
        if !native_stopped {
            let cleanup = self.application.dispatch().and_then(|mut registry| {
                registry
                    .shutdown_and_unregister(runtime_id, process_generation)
                    .map_err(Into::into)
            });
            let mut active = self
                .active
                .lock()
                .map_err(|_| RuntimeProductionApplicationError::Unavailable)?;
            let runtime = active
                .get_mut(runtime_id)
                .ok_or(RuntimeProductionApplicationError::NotActive)?;
            if let Err(error) = cleanup {
                runtime.quarantined = true;
                runtime.worker.revoke_transient_credentials();
                return Err(error.into());
            }
            runtime.revoke_after_native_stop();
        }
        let stopped = if attached {
            self.application.finish_managed_runtime_shutdown(
                reserved_generation,
                runtime_id,
                process_generation,
                stopped_at_ms,
            )?
        } else {
            self.application.abort_managed_runtime_start(
                reserved_generation,
                runtime_id,
                process_generation,
                stopped_at_ms,
            )?
        };
        self.active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?
            .remove(runtime_id);
        Ok(stopped.coordinator_generation)
    }

    pub fn answer_runtime_approval(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        correlation_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        answered_at_ms: u64,
    ) -> Result<(), RuntimeProductionApplicationError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| RuntimeProductionApplicationError::Unavailable)?;
        let runtime = active
            .get_mut(runtime_id)
            .ok_or(RuntimeProductionApplicationError::NotActive)?;
        if runtime.quarantined {
            return Err(RuntimeProductionApplicationError::NotActive);
        }
        let result = runtime.worker.answer_approval(
            &self.application,
            expected_coordinator_generation,
            correlation_id,
            prompt_id,
            answer,
            answered_at_ms,
        );
        match result {
            Err(RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(source)) => {
                runtime.quarantined = true;
                let cleanup =
                    self.terminate_quarantined_generation(&mut active, runtime_id, answered_at_ms);
                match cleanup {
                    Ok(()) => Err(
                        RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(source),
                    ),
                    Err(cleanup) => Err(
                        RuntimeProductionApplicationError::OpenCodeBrokerGenerationQuarantined {
                            source,
                            cleanup: cleanup.to_string(),
                        },
                    ),
                }
            }
            Err(RuntimeProductionApplicationError::PiToolSettlementGenerationFatal) => {
                runtime.quarantined = true;
                let cleanup =
                    self.terminate_quarantined_generation(&mut active, runtime_id, answered_at_ms);
                match cleanup {
                    Ok(()) => {
                        Err(RuntimeProductionApplicationError::PiToolSettlementGenerationFatal)
                    }
                    Err(cleanup) => Err(
                        RuntimeProductionApplicationError::PiToolSettlementGenerationQuarantined {
                            cleanup: cleanup.to_string(),
                        },
                    ),
                }
            }
            result => result,
        }
    }

    fn terminate_quarantined_generation(
        &self,
        active: &mut BTreeMap<String, ActiveProductionRuntime>,
        runtime_id: &str,
        stopped_at_ms: u64,
    ) -> Result<(), RuntimeProductionApplicationError> {
        let runtime = active
            .get_mut(runtime_id)
            .ok_or(RuntimeProductionApplicationError::NotActive)?;
        let process_generation = runtime.worker.binding().process_generation();
        let attached = runtime.attached;
        let native_stopped = runtime.native_stopped;
        // A fatal generation can no longer be trusted to authenticate any
        // request, even when descendant cleanup subsequently fails. Revoke
        // its session-only credential before attempting native teardown.
        runtime.worker.revoke_transient_credentials();
        let snapshot = self.application.snapshot(stopped_at_ms)?;
        let reserved_generation = self
            .application
            .reserve_runtime_operation(snapshot.generation, stopped_at_ms)?;
        if !native_stopped {
            self.application
                .dispatch()?
                .shutdown_and_unregister(runtime_id, process_generation)?;
            active
                .get_mut(runtime_id)
                .ok_or(RuntimeProductionApplicationError::NotActive)?
                .native_stopped = true;
        }
        if attached {
            self.application.finish_managed_runtime_shutdown(
                reserved_generation,
                runtime_id,
                process_generation,
                stopped_at_ms,
            )?;
        } else {
            self.application.abort_managed_runtime_start(
                reserved_generation,
                runtime_id,
                process_generation,
                stopped_at_ms,
            )?;
        }
        active.remove(runtime_id);
        Ok(())
    }

    fn abort_reservation(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) {
        if self
            .application
            .abort_managed_runtime_start(
                expected_coordinator_generation,
                runtime_id,
                process_generation,
                at_ms,
            )
            .is_err()
            && let Ok(snapshot) = self.application.snapshot(at_ms)
        {
            let _ = self.application.abort_managed_runtime_start(
                snapshot.generation,
                runtime_id,
                process_generation,
                at_ms,
            );
        }
    }
}

impl ActiveProductionRuntime {
    fn revoke_after_native_stop(&mut self) {
        self.worker.revoke_transient_credentials();
        self.native_stopped = true;
    }
}

impl<B> Drop for RuntimeProductionApplication<B> {
    fn drop(&mut self) {
        let stopped_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|duration| duration.as_millis().try_into().ok())
            .unwrap_or(1);
        let Ok(active) = self.active.get_mut() else {
            return;
        };
        let runtimes = std::mem::take(active);
        for (runtime_id, active) in runtimes {
            let process_generation = active.worker.binding().process_generation();
            let cleanup_succeeded = active.native_stopped
                || self
                    .application
                    .dispatch()
                    .and_then(|mut registry| {
                        registry
                            .shutdown_and_unregister(&runtime_id, process_generation)
                            .map_err(Into::into)
                    })
                    .is_ok();
            if cleanup_succeeded && let Ok(snapshot) = self.application.snapshot(stopped_at_ms) {
                if active.attached {
                    let _ = self.application.finish_managed_runtime_shutdown(
                        snapshot.generation,
                        &runtime_id,
                        process_generation,
                        stopped_at_ms,
                    );
                } else {
                    let _ = self.application.abort_managed_runtime_start(
                        snapshot.generation,
                        &runtime_id,
                        process_generation,
                        stopped_at_ms,
                    );
                }
            }
        }
    }
}

impl RuntimeProductionApplication<RuntimeProductionBootstrap> {
    pub fn new(
        application: Arc<RuntimeApplicationService>,
        bootstrap: RuntimeProductionBootstrap,
    ) -> Self {
        Self::with_backend(application, bootstrap)
    }
}

impl ProductionRuntimeBackend for RuntimeProductionBootstrap {
    fn prepare(
        &self,
        binding: ProductionRuntimeBinding,
        provider_routes: &[ProductionProviderRoute],
        checked_at_ms: u64,
    ) -> Result<Box<dyn PreparedProductionRuntimePeer>, RuntimeProductionApplicationError> {
        Ok(Box::new(self.prepare_core_owned_peer(
            binding,
            provider_routes,
            checked_at_ms,
        )?))
    }
}

impl PreparedProductionRuntimePeer for PreparedCoreProductionPeer {
    fn binding(&self) -> &ProductionRuntimeBinding {
        self.binding()
    }

    fn native_process_id(&self) -> Result<u32, RuntimeProductionApplicationError> {
        Ok(self.native_process_id()?)
    }

    fn capability_epochs(
        &self,
    ) -> Result<Vec<CapabilityRouteEpoch>, RuntimeProductionApplicationError> {
        Ok(self.capability_epochs())
    }

    fn install(
        self: Box<Self>,
        registry: &mut RuntimeDispatchRegistry,
    ) -> Result<Box<dyn ProductionRuntimeWorker>, RuntimeProductionApplicationError> {
        Ok(Box::new((*self).register(registry)?))
    }
}

impl ProductionRuntimeWorker for CoreProductionWorker {
    fn binding(&self) -> &ProductionRuntimeBinding {
        self.binding()
    }

    fn revoke_transient_credentials(&mut self) {
        CoreProductionWorker::revoke_transient_credentials(self);
    }

    fn pump(
        &mut self,
        application: &Arc<RuntimeApplicationService>,
        now_ms: u64,
    ) -> Result<usize, RuntimeProductionApplicationError> {
        // The production event worker already bounds each drain by its
        // configured queue limit. Persist that normalized SSE batch first,
        // then process at most one broker descriptor frame so either source
        // can make progress without monopolizing the app-wide pump loop.
        match self {
            Self::OpenCode(worker) => {
                let expected_generation = application.snapshot(now_ms)?.generation;
                let events = application.poll_runtime_events(
                    expected_generation,
                    worker.binding().runtime_id(),
                    now_ms,
                )?;
                let processed = events.len();
                let mut application = Arc::clone(application);
                match worker.pump_one(&mut application, now_ms) {
                    Ok(_) => Ok(processed.saturating_add(1)),
                    Err(error) if error.is_generation_fatal() => {
                        Err(RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(error))
                    }
                    Err(crate::runtime::opencode_broker::OpenCodeBrokerPumpError::Channel(_)) => {
                        Ok(processed)
                    }
                    Err(error) => {
                        Err(crate::runtime::production::RuntimeProductionError::from(error).into())
                    }
                }
            }
            Self::Pi(worker) => {
                let mut processed = 0_usize;
                if !worker.has_pending_action_intents() {
                    let expected_generation = application
                        .snapshot(now_ms)
                        .map_err(|_| {
                            RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                        })?
                        .generation;
                    let events = application
                        .poll_runtime_events(
                            expected_generation,
                            worker.binding().runtime_id(),
                            now_ms,
                        )
                        .map_err(|_| {
                            RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                        })?;
                    let mut action_intents = Vec::new();
                    for applied in events {
                        if matches!(
                            applied.event.peer.category,
                            DispatchEventCategory::PiActionIntent(_)
                        ) {
                            action_intents.push(applied.event);
                        } else {
                            processed = processed.saturating_add(1);
                        }
                    }
                    worker.enqueue_action_intents(action_intents).map_err(|_| {
                        RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                    })?;
                }
                let Some(identity) = worker.next_pending_action_identity() else {
                    return Ok(processed);
                };
                let expected_generation = application
                    .snapshot(now_ms)
                    .map_err(|_| {
                        RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                    })?
                    .generation;
                let mut transaction = match application
                    .begin_runtime_broker_transaction(expected_generation, now_ms)
                {
                    Ok(transaction) => transaction,
                    Err(crate::RuntimeApplicationError::Generation { .. }) => {
                        return Ok(processed);
                    }
                    Err(_) => {
                        return Err(
                            RuntimeProductionApplicationError::PiToolSettlementGenerationFatal,
                        );
                    }
                };
                let authority = transaction
                    .pi_gateway_authority_for_identity(&identity)
                    .map_err(|_| {
                        RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                    })?;
                worker
                    .evaluate_next_action_intent(&mut transaction, authority, now_ms)
                    .map_err(|_| {
                        RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                    })?;
                let mut registry = transaction.dispatch().map_err(|_| {
                    RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                })?;
                match worker.settle_next_action_intent(&mut registry) {
                    Ok(_) => Ok(processed.saturating_add(1)),
                    Err(
                        crate::runtime::production::RuntimeProductionError::PiToolResolutionFailed,
                    ) => {
                        drop(registry);
                        transaction
                            .record_pi_fallback_cancellation(&identity, now_ms)
                            .map_err(|_| {
                                RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                            })?;
                        Err(crate::runtime::production::RuntimeProductionError::PiToolResolutionFailed.into())
                    }
                    Err(_) => {
                        Err(RuntimeProductionApplicationError::PiToolSettlementGenerationFatal)
                    }
                }
            }
        }
    }

    fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
        match self {
            Self::OpenCode(worker) => worker.pending_approval_descriptors(),
            Self::Pi(worker) => worker.pending_approval_descriptors(),
        }
    }

    fn answer_approval(
        &mut self,
        application: &Arc<RuntimeApplicationService>,
        expected_coordinator_generation: u64,
        correlation_id: &str,
        prompt_id: &str,
        answer: ApprovalAnswer,
        answered_at_ms: u64,
    ) -> Result<(), RuntimeProductionApplicationError> {
        match self {
            Self::OpenCode(worker) => {
                let mut application = application.begin_runtime_broker_transaction(
                    expected_coordinator_generation,
                    answered_at_ms,
                )?;
                if let Err(error) = worker.answer_approval(
                    &mut application,
                    correlation_id,
                    prompt_id,
                    answer,
                    answered_at_ms,
                ) {
                    if error.is_generation_fatal() {
                        return Err(
                            RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(error),
                        );
                    }
                    return Err(
                        crate::runtime::production::RuntimeProductionError::from(error).into(),
                    );
                }
            }
            Self::Pi(worker) => {
                let identity = worker
                    .pending_approval_identity(correlation_id)
                    .ok_or(RuntimeProductionApplicationError::BindingMismatch)?;
                let mut application = application.begin_runtime_broker_transaction(
                    expected_coordinator_generation,
                    answered_at_ms,
                )?;
                let authority = application
                    .pi_gateway_authority_for_identity(&identity)
                    .map_err(|_| {
                        RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                    })?;
                let settlement = worker
                    .evaluate_action_approval_by_correlation(
                        &mut application,
                        correlation_id,
                        prompt_id,
                        answer,
                        authority,
                        answered_at_ms,
                    )
                    .map_err(|_| {
                        RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                    })?;
                let mut registry = application.dispatch().map_err(|_| {
                    RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                })?;
                match worker.settle_action_approval(&mut registry, settlement) {
                    Ok(_) => {}
                    Err(
                        crate::runtime::production::RuntimeProductionError::PiToolResolutionFailed,
                    ) => {
                        drop(registry);
                        application
                            .record_pi_fallback_cancellation(&identity, answered_at_ms)
                            .map_err(|_| {
                                RuntimeProductionApplicationError::PiToolSettlementGenerationFatal
                            })?;
                        return Err(crate::runtime::production::RuntimeProductionError::PiToolResolutionFailed.into());
                    }
                    Err(_) => {
                        return Err(
                            RuntimeProductionApplicationError::PiToolSettlementGenerationFatal,
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use tempfile::TempDir;

    use crate::RuntimeApplicationService;
    use crate::core::database::{DatabaseActor, DatabaseDescriptor};
    use crate::runtime::adapter::{
        ADAPTER_CONTRACT_SCHEMA_VERSION, AdapterAuthority, AdapterConformanceDescriptor,
        PeerCapabilityClaims, peer_capabilities,
    };
    use crate::runtime::capability::{
        CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
        CapabilityLayer, CapabilityState, ModelLifecycle, RouteIdentity,
    };
    use crate::runtime::dispatch::{
        DispatchIdentity, PeerDispatchError, PeerDispatchEvent, PeerDispatchRequest,
        RuntimeDispatchPeer, RuntimeDispatchRegistry, RuntimePeerRegistration,
    };
    use crate::runtime::production::ProductionRuntimeBinding;
    use crate::runtime::provider::{
        ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, PROVIDER_SCHEMA_VERSION,
        ProviderConnectionEvidence, ProviderDiscovery, ProviderEndpoint, ProviderKind,
        ProviderModelDeclaration, ProviderProbe, ProviderProbeFailure, ProviderProfile,
        RouteAvailability,
    };
    use crate::runtime::supervisor::{
        PI_NATIVE_VERSION, RUNTIME_PROTOCOL_VERSION, RuntimeInstallation, RuntimeKind,
        RuntimeLifecycle, sha256_file,
    };
    use crate::security::credentials::{CredentialReference, CredentialVault};

    const NOW: u64 = 1_721_300_000_000;

    fn digest(seed: char) -> String {
        format!("sha256:{}", seed.to_string().repeat(64))
    }

    struct FixtureProviderProbe(ProviderDiscovery);

    impl ProviderProbe for FixtureProviderProbe {
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

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct ProviderRouteObservation {
        provider_id: String,
        endpoint_id: String,
        native_provider_id: String,
        base_url: String,
        selected_model_id: String,
        credential_reference: CredentialReference,
    }

    struct RouteCapturingBackend {
        captured: Arc<Mutex<Vec<ProviderRouteObservation>>>,
    }

    impl ProductionRuntimeBackend for RouteCapturingBackend {
        fn prepare(
            &self,
            binding: ProductionRuntimeBinding,
            provider_routes: &[ProductionProviderRoute],
            _checked_at_ms: u64,
        ) -> Result<Box<dyn PreparedProductionRuntimePeer>, RuntimeProductionApplicationError>
        {
            let observations = provider_routes
                .iter()
                .map(|route| {
                    let profile = route.profile();
                    Ok(ProviderRouteObservation {
                        provider_id: profile.provider_id.clone(),
                        endpoint_id: profile.endpoint.endpoint_id.clone(),
                        native_provider_id: profile
                            .pi_native_provider_id()
                            .ok_or(RuntimeProductionApplicationError::PreparationRejected)?
                            .into(),
                        base_url: profile.endpoint.base_url.clone(),
                        selected_model_id: route.selected_model_id().into(),
                        credential_reference: profile.credential_reference.clone(),
                    })
                })
                .collect::<Result<Vec<_>, RuntimeProductionApplicationError>>()?;
            *self.captured.lock().unwrap() = observations;
            Ok(Box::new(FakePreparedPeer {
                binding,
                process_id: 42_424,
                pumps: Arc::new(Mutex::new(0)),
                shutdowns: Arc::new(Mutex::new(0)),
                revocations: None,
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            }))
        }
    }

    struct FakePeer {
        registration: RuntimePeerRegistration,
        shutdowns: Arc<Mutex<usize>>,
    }

    impl RuntimeDispatchPeer for FakePeer {
        fn registration(&self) -> &RuntimePeerRegistration {
            &self.registration
        }

        fn readiness(&self) -> Result<(), PeerDispatchError> {
            Ok(())
        }

        fn create_session(
            &mut self,
            _request: &PeerDispatchRequest,
        ) -> Result<(), PeerDispatchError> {
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

        fn shutdown(&mut self) -> Result<(), PeerDispatchError> {
            *self.shutdowns.lock().unwrap() += 1;
            if self.registration.runtime_id == "pi-cleanup-fail" {
                Err(PeerDispatchError::Shutdown)
            } else {
                Ok(())
            }
        }
    }

    struct FakePreparedPeer {
        binding: ProductionRuntimeBinding,
        process_id: u32,
        pumps: Arc<Mutex<usize>>,
        shutdowns: Arc<Mutex<usize>>,
        revocations: Option<Arc<Mutex<usize>>>,
        invalidate_attach: Option<Arc<RuntimeApplicationService>>,
        fail_pump_runtime: Option<String>,
        fatal_pump_runtime: Option<String>,
    }

    impl PreparedProductionRuntimePeer for FakePreparedPeer {
        fn binding(&self) -> &ProductionRuntimeBinding {
            &self.binding
        }

        fn native_process_id(&self) -> Result<u32, RuntimeProductionApplicationError> {
            Ok(self.process_id)
        }

        fn install(
            self: Box<Self>,
            registry: &mut RuntimeDispatchRegistry,
        ) -> Result<Box<dyn ProductionRuntimeWorker>, RuntimeProductionApplicationError> {
            registry.register(FakePeer {
                registration: RuntimePeerRegistration {
                    runtime_id: self.binding.runtime_id().into(),
                    workspace_id: self.binding.workspace_id().into(),
                    descriptor: descriptor(self.binding.process_generation()),
                },
                shutdowns: Arc::clone(&self.shutdowns),
            })?;
            if let Some(application) = &self.invalidate_attach {
                let generation = application.snapshot(NOW + 2)?.generation;
                application.publish_runtime_policy_authority(generation, 1, 2, 0, NOW + 2)?;
            }
            Ok(Box::new(FakeWorker {
                binding: self.binding.clone(),
                pumps: Arc::clone(&self.pumps),
                revocations: self.revocations.clone(),
                fail_pump_runtime: self.fail_pump_runtime.clone(),
                fatal_pump_runtime: self.fatal_pump_runtime.clone(),
            }))
        }
    }

    struct FakeWorker {
        binding: ProductionRuntimeBinding,
        pumps: Arc<Mutex<usize>>,
        revocations: Option<Arc<Mutex<usize>>>,
        fail_pump_runtime: Option<String>,
        fatal_pump_runtime: Option<String>,
    }

    impl ProductionRuntimeWorker for FakeWorker {
        fn binding(&self) -> &ProductionRuntimeBinding {
            &self.binding
        }

        fn revoke_transient_credentials(&mut self) {
            if let Some(revocations) = &self.revocations {
                *revocations.lock().unwrap() += 1;
            }
        }

        fn pump(
            &mut self,
            application: &Arc<RuntimeApplicationService>,
            now_ms: u64,
        ) -> Result<usize, RuntimeProductionApplicationError> {
            let _ = application.snapshot(now_ms)?;
            let mut pumps = self.pumps.lock().unwrap();
            *pumps += 1;
            if self.fail_pump_runtime.as_deref() == Some(self.binding.runtime_id()) {
                return Err(RuntimeProductionApplicationError::PreparationRejected);
            }
            if self.fatal_pump_runtime.as_deref() == Some(self.binding.runtime_id()) {
                return Err(
                    RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(
                        crate::runtime::opencode_broker::OpenCodeBrokerPumpError::ResponseStatusUnknown(
                            crate::runtime::opencode_sdk::OpenCodeSdkError::UnknownCorrelation,
                        ),
                    ),
                );
            }
            Ok(1)
        }

        fn pending_approval_descriptors(&self) -> Vec<(String, String)> {
            vec![(
                format!("correlation:{}", self.binding.runtime_id()),
                format!("approval:{}", self.binding.runtime_id()),
            )]
        }

        fn answer_approval(
            &mut self,
            _application: &Arc<RuntimeApplicationService>,
            _expected_coordinator_generation: u64,
            _correlation_id: &str,
            _prompt_id: &str,
            _answer: ApprovalAnswer,
            _answered_at_ms: u64,
        ) -> Result<(), RuntimeProductionApplicationError> {
            if self.fatal_pump_runtime.as_deref() == Some(self.binding.runtime_id()) {
                return Err(
                    RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(
                        crate::runtime::opencode_broker::OpenCodeBrokerPumpError::ResponseStatusUnknown(
                            crate::runtime::opencode_sdk::OpenCodeSdkError::UnknownCorrelation,
                        ),
                    ),
                );
            }
            Err(RuntimeProductionApplicationError::PreparationRejected)
        }
    }

    struct FakeBackend {
        captured: Arc<Mutex<Vec<ProductionRuntimeBinding>>>,
        pumps: Arc<Mutex<usize>>,
        reject: bool,
        shutdowns: Arc<Mutex<usize>>,
        revocations: Option<Arc<Mutex<usize>>>,
        invalidate_attach: Option<Arc<RuntimeApplicationService>>,
        fail_pump_runtime: Option<String>,
        fatal_pump_runtime: Option<String>,
    }

    impl ProductionRuntimeBackend for FakeBackend {
        fn prepare(
            &self,
            binding: ProductionRuntimeBinding,
            _provider_routes: &[ProductionProviderRoute],
            _checked_at_ms: u64,
        ) -> Result<Box<dyn PreparedProductionRuntimePeer>, RuntimeProductionApplicationError>
        {
            self.captured.lock().unwrap().push(binding.clone());
            if self.reject {
                return Err(RuntimeProductionApplicationError::PreparationRejected);
            }
            Ok(Box::new(FakePreparedPeer {
                binding,
                process_id: 42_424,
                pumps: Arc::clone(&self.pumps),
                shutdowns: Arc::clone(&self.shutdowns),
                revocations: self.revocations.clone(),
                invalidate_attach: self.invalidate_attach.clone(),
                fail_pump_runtime: self.fail_pump_runtime.clone(),
                fatal_pump_runtime: self.fatal_pump_runtime.clone(),
            }))
        }
    }

    fn descriptor(process_generation: u64) -> AdapterConformanceDescriptor {
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
                session_resume: CapabilityState::Supported,
                model_discovery: CapabilityState::Supported,
                streaming: CapabilityState::Supported,
                action_intents: CapabilityState::Supported,
                credential_channel: CapabilityState::Supported,
                cancellation: CapabilityState::Supported,
                restart: CapabilityState::Supported,
            }),
        }
    }

    fn setup_service(temporary: &TempDir) -> Arc<RuntimeApplicationService> {
        let c4os_home = temporary.path().join("c4os-home");
        let (app_database, _) = DatabaseActor::start(DatabaseDescriptor::app(&c4os_home)).unwrap();
        let service =
            Arc::new(RuntimeApplicationService::restore(Arc::new(app_database), NOW).unwrap());
        let workspace_root = temporary.path().join("workspace-active");
        let (workspace_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
            &workspace_root,
            "workspace-1",
        ))
        .unwrap();
        service
            .bind_workspace(Arc::new(workspace_database))
            .unwrap();
        service
            .register_runtime(1, pi_installation(temporary.path()), NOW + 1)
            .unwrap();
        service
    }

    fn pi_installation(root: &Path) -> RuntimeInstallation {
        pi_installation_named(root, "pi-primary")
    }

    fn pi_installation_named(root: &Path, runtime_id: &str) -> RuntimeInstallation {
        let install_root = root.join(format!("runtime-{runtime_id}"));
        let executable = install_root.join("bin/pi");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"#!/bin/sh\n/bin/sleep 30\n").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        RuntimeInstallation {
            runtime_id: runtime_id.into(),
            workspace_id: "workspace-1".into(),
            runtime_kind: RuntimeKind::Pi,
            native_version: PI_NATIVE_VERSION.into(),
            adapter_version: "1.0.0".into(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
            install_root: install_root.clone(),
            asset_tree_sha256: sha256_file(&executable).unwrap(),
            executable_sha256: sha256_file(&executable).unwrap(),
            executable,
            state_namespace: install_root.join("state"),
            arguments: Vec::new(),
            sanitized_environment: BTreeMap::new(),
        }
    }

    fn tested_pi_profile(
        provider_id: &str,
        endpoint_id: &str,
        credential_reference: CredentialReference,
    ) -> ProviderProfile {
        ProviderProfile {
            schema_version: PROVIDER_SCHEMA_VERSION,
            provider_id: provider_id.into(),
            kind: ProviderKind::OpenAi,
            display_name: format!("OpenAI {provider_id}"),
            endpoint: ProviderEndpoint {
                endpoint_id: endpoint_id.into(),
                base_url: "https://api.openai.com/v1".into(),
                api_kind: "openai".into(),
            },
            credential_reference,
            enabled: true,
        }
    }

    fn tested_pi_model(profile: &ProviderProfile, checked_at_ms: u64) -> ModelRoute {
        let supported = || CapabilityEvidence {
            state: CapabilityState::Supported,
            layer: CapabilityLayer::AdapterNormalized,
            source: "pi.0.80.10".into(),
            checked_at_ms,
            expires_at_ms: Some(checked_at_ms + 60_000),
            constraints: Vec::new(),
            allowed_values: Vec::new(),
            reason: None,
        };
        ModelRoute {
            model_id: "gpt-4o-mini".into(),
            display_name: "GPT-4o mini".into(),
            recommendation_rank: 0,
            availability: RouteAvailability::Available,
            checked_at_ms,
            capabilities: CapabilityDescriptor {
                schema_version: CAPABILITY_SCHEMA_VERSION,
                layer: CapabilityLayer::AdapterNormalized,
                route: RouteIdentity {
                    provider_id: profile.provider_id.clone(),
                    endpoint_id: profile.endpoint.endpoint_id.clone(),
                    provider_model_id: "openai/gpt-4o-mini".into(),
                    model_revision: "2026-07-19".into(),
                    adapter_kind: "pi".into(),
                    adapter_version: "1.0.0".into(),
                    runtime_kind: "pi".into(),
                    native_runtime_version: PI_NATIVE_VERSION.into(),
                    session_configuration_sha256: digest('a'),
                },
                lifecycle: ModelLifecycle::Active,
                features: BTreeMap::from([
                    (CapabilityKey::InputText, supported()),
                    (CapabilityKey::OutputText, supported()),
                    (CapabilityKey::Streaming, supported()),
                ]),
                numeric_limits: BTreeMap::new(),
                raw_evidence_sha256: digest('b'),
            },
            provider_declaration: Some(ProviderModelDeclaration {
                schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
                provider_model_id: "gpt-4o-mini".into(),
                model_revision: "2026-07-19".into(),
                lifecycle: ModelLifecycle::Active,
                features: BTreeMap::new(),
                numeric_limits: BTreeMap::new(),
                raw_catalog_sha256: digest('c'),
                declared_at_ms: checked_at_ms,
                expires_at_ms: checked_at_ms + 60_000,
            }),
        }
    }

    fn seed_tested_pi_provider(
        application: &RuntimeApplicationService,
        profile: ProviderProfile,
        saved_at_ms: u64,
        tested_at_ms: u64,
    ) {
        let before_save = application.snapshot(saved_at_ms).unwrap();
        application
            .save_provider(
                before_save.generation,
                profile.clone(),
                before_save.providers.generation,
                saved_at_ms,
            )
            .unwrap();
        let before_test = application.snapshot(tested_at_ms).unwrap();
        application
            .test_provider(
                before_test.generation,
                &profile.provider_id,
                before_test.providers.generation,
                tested_at_ms,
                &mut FixtureProviderProbe(ProviderDiscovery {
                    checked_at_ms: tested_at_ms,
                    models: vec![tested_pi_model(&profile, tested_at_ms)],
                    recommended_model_id: Some("gpt-4o-mini".into()),
                    connection_evidence: None,
                }),
            )
            .unwrap();
    }

    #[test]
    fn one_tested_custom_pi_profile_reaches_the_backend_with_its_exact_https_route() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let vault = CredentialVault::session_only().unwrap();
        let credential_reference = vault.store("provider-key", b"team-a-secret").unwrap();
        let mut profile = tested_pi_profile(
            "openai-team-a",
            "openai-endpoint-a",
            credential_reference.clone(),
        );
        profile.kind = ProviderKind::Custom;
        profile.endpoint.base_url = "https://proxy.example/v1".into();
        profile.endpoint.api_kind = "openai-compatible".into();
        seed_tested_pi_provider(&application, profile, NOW + 2, NOW + 3);
        let captured = Arc::new(Mutex::new(Vec::new()));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            RouteCapturingBackend {
                captured: Arc::clone(&captured),
            },
        );
        let before_activation = application.snapshot(NOW + 4).unwrap();

        let activated = host
            .activate_runtime(before_activation.generation, "pi-primary", NOW + 4)
            .unwrap();
        assert_eq!(
            *captured.lock().unwrap(),
            vec![ProviderRouteObservation {
                provider_id: "openai-team-a".into(),
                endpoint_id: "openai-endpoint-a".into(),
                native_provider_id: "openai".into(),
                base_url: "https://proxy.example/v1".into(),
                selected_model_id: "gpt-4o-mini".into(),
                credential_reference,
            }]
        );
        assert!(!format!("{:?}", captured.lock().unwrap()).contains("team-a-secret"));
        host.shutdown_runtime(activated.coordinator_generation, "pi-primary", NOW + 5)
            .unwrap();
    }

    #[test]
    fn two_pi_profiles_sharing_openai_retain_distinct_routes_and_credentials() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let vault = CredentialVault::session_only().unwrap();
        let credential_a = vault.store("provider-key-a", b"team-a-secret").unwrap();
        let credential_b = vault.store("provider-key-b", b"team-b-secret").unwrap();
        seed_tested_pi_provider(
            &application,
            tested_pi_profile("openai-team-a", "openai-endpoint-a", credential_a.clone()),
            NOW + 2,
            NOW + 3,
        );
        seed_tested_pi_provider(
            &application,
            tested_pi_profile("openai-team-b", "openai-endpoint-b", credential_b.clone()),
            NOW + 4,
            NOW + 5,
        );
        let captured = Arc::new(Mutex::new(Vec::new()));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            RouteCapturingBackend {
                captured: Arc::clone(&captured),
            },
        );
        let before_activation = application.snapshot(NOW + 6).unwrap();

        let activated = host
            .activate_runtime(before_activation.generation, "pi-primary", NOW + 6)
            .unwrap();
        assert_eq!(
            *captured.lock().unwrap(),
            vec![
                ProviderRouteObservation {
                    provider_id: "openai-team-a".into(),
                    endpoint_id: "openai-endpoint-a".into(),
                    native_provider_id: "openai".into(),
                    base_url: "https://api.openai.com/v1".into(),
                    selected_model_id: "gpt-4o-mini".into(),
                    credential_reference: credential_a,
                },
                ProviderRouteObservation {
                    provider_id: "openai-team-b".into(),
                    endpoint_id: "openai-endpoint-b".into(),
                    native_provider_id: "openai".into(),
                    base_url: "https://api.openai.com/v1".into(),
                    selected_model_id: "gpt-4o-mini".into(),
                    credential_reference: credential_b,
                },
            ]
        );
        let debug = format!("{:?}", captured.lock().unwrap());
        assert!(!debug.contains("team-a-secret"));
        assert!(!debug.contains("team-b-secret"));
        host.shutdown_runtime(activated.coordinator_generation, "pi-primary", NOW + 7)
            .unwrap();
    }

    #[test]
    fn core_owned_host_derives_registers_attaches_pumps_and_shuts_down_a_runtime() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let captured = Arc::new(Mutex::new(Vec::new()));
        let pumps = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::clone(&captured),
                pumps: Arc::clone(&pumps),
                reject: false,
                shutdowns: Arc::new(Mutex::new(0)),
                revocations: None,
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );

        let activated = host.activate_runtime(2, "pi-primary", NOW + 2).unwrap();
        assert_eq!(activated.process_generation, 1);
        assert_eq!(activated.process_id, 42_424);
        assert_eq!(host.active_runtime_count().unwrap(), 1);
        assert_eq!(
            host.pending_approvals().unwrap(),
            vec![PendingProductionRuntimeApproval {
                runtime_id: "pi-primary".into(),
                correlation_id: "correlation:pi-primary".into(),
                prompt_id: "approval:pi-primary".into(),
            }]
        );
        let binding = captured.lock().unwrap().first().unwrap().clone();
        assert_eq!(binding.runtime_id(), "pi-primary");
        assert_eq!(binding.workspace_id(), "workspace-1");
        assert_eq!(
            binding.workspace_root(),
            temporary
                .path()
                .join("workspace-active")
                .canonicalize()
                .unwrap()
        );
        assert_eq!(
            binding.launch_id(),
            format!("launch-pi-primary-1-{}", NOW + 2)
        );
        let ready = application.snapshot(NOW + 2).unwrap();
        let record = ready
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == "pi-primary")
            .unwrap();
        assert_eq!(record.lifecycle, RuntimeLifecycle::Ready);
        assert_eq!(record.process_id, Some(42_424));

        assert_eq!(host.pump_runtime_once("pi-primary", NOW + 3).unwrap(), 1);
        assert_eq!(*pumps.lock().unwrap(), 1);

        host.shutdown_runtime(ready.generation, "pi-primary", NOW + 4)
            .unwrap();
        assert_eq!(host.active_runtime_count().unwrap(), 0);
        let stopped = application.snapshot(NOW + 4).unwrap();
        let record = stopped
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == "pi-primary")
            .unwrap();
        assert_eq!(record.lifecycle, RuntimeLifecycle::Stopped);
        assert_eq!(record.process_id, None);
    }

    #[test]
    fn production_recovery_replaces_the_peer_with_a_fresh_process_generation() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let captured = Arc::new(Mutex::new(Vec::new()));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::clone(&captured),
                pumps: Arc::new(Mutex::new(0)),
                reject: false,
                shutdowns: Arc::new(Mutex::new(0)),
                revocations: None,
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );

        let first = host.activate_runtime(2, "pi-primary", NOW + 2).unwrap();
        let after_first_shutdown = host
            .shutdown_runtime(first.coordinator_generation, "pi-primary", NOW + 3)
            .unwrap();
        let second = host
            .activate_runtime(after_first_shutdown, "pi-primary", NOW + 4)
            .unwrap();

        assert_eq!(first.process_generation, 1);
        assert_eq!(second.process_generation, 2);
        let bindings = captured.lock().unwrap();
        assert_eq!(bindings.len(), 2);
        assert_ne!(bindings[0].launch_id(), bindings[1].launch_id());
        drop(bindings);

        host.shutdown_runtime(second.coordinator_generation, "pi-primary", NOW + 5)
            .unwrap();
    }

    #[test]
    fn failed_native_preparation_durably_aborts_the_reserved_generation() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::new(Mutex::new(0)),
                reject: true,
                shutdowns: Arc::new(Mutex::new(0)),
                revocations: None,
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );

        assert!(matches!(
            host.activate_runtime(2, "pi-primary", NOW + 2),
            Err(RuntimeProductionApplicationError::PreparationRejected)
        ));
        assert_eq!(host.active_runtime_count().unwrap(), 0);
        let snapshot = application.snapshot(NOW + 2).unwrap();
        let record = snapshot
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == "pi-primary")
            .unwrap();
        assert_eq!(record.process_generation, 1);
        assert_eq!(record.lifecycle, RuntimeLifecycle::Stopped);
        assert_eq!(record.process_id, None);
    }

    #[test]
    fn attach_cas_failure_shuts_down_and_unregisters_the_spawned_peer_before_abort() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let shutdowns = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::new(Mutex::new(0)),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: None,
                invalidate_attach: Some(Arc::clone(&application)),
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );

        assert!(matches!(
            host.activate_runtime(2, "pi-primary", NOW + 2),
            Err(RuntimeProductionApplicationError::Application(
                crate::RuntimeApplicationError::Generation { .. }
            ))
        ));
        assert_eq!(*shutdowns.lock().unwrap(), 1);
        assert_eq!(host.active_runtime_count().unwrap(), 0);
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
    fn attach_cas_cleanup_failure_quarantines_the_unattached_worker() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        application
            .register_runtime(
                2,
                pi_installation_named(temporary.path(), "pi-cleanup-fail"),
                NOW + 2,
            )
            .unwrap();
        let shutdowns = Arc::new(Mutex::new(0));
        let pumps = Arc::new(Mutex::new(0));
        let revocations = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::clone(&pumps),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: Some(Arc::clone(&revocations)),
                invalidate_attach: Some(Arc::clone(&application)),
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );

        assert!(
            host.activate_runtime(3, "pi-cleanup-fail", NOW + 3)
                .is_err()
        );
        assert_eq!(*shutdowns.lock().unwrap(), 1);
        assert_eq!(*revocations.lock().unwrap(), 1);
        assert_eq!(host.active_runtime_count().unwrap(), 0);
        assert_eq!(*pumps.lock().unwrap(), 0);
        assert!(matches!(
            host.pump_runtime_once("pi-cleanup-fail", NOW + 4),
            Err(RuntimeProductionApplicationError::NotActive)
        ));
        assert_eq!(*pumps.lock().unwrap(), 0);
    }

    #[test]
    fn shutdown_cleanup_failure_revokes_transient_credentials_before_quarantine() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        application
            .register_runtime(
                2,
                pi_installation_named(temporary.path(), "pi-cleanup-fail"),
                NOW + 2,
            )
            .unwrap();
        let shutdowns = Arc::new(Mutex::new(0));
        let revocations = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::new(Mutex::new(0)),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: Some(Arc::clone(&revocations)),
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );
        let activated = host
            .activate_runtime(3, "pi-cleanup-fail", NOW + 3)
            .unwrap();

        assert!(
            host.shutdown_runtime(activated.coordinator_generation, "pi-cleanup-fail", NOW + 4,)
                .is_err()
        );
        assert_eq!(*shutdowns.lock().unwrap(), 1);
        assert_eq!(*revocations.lock().unwrap(), 1);
        assert_eq!(host.active_runtime_count().unwrap(), 0);
        assert!(matches!(
            host.pump_runtime_once("pi-cleanup-fail", NOW + 5),
            Err(RuntimeProductionApplicationError::NotActive)
        ));
    }

    #[test]
    fn fatal_cleanup_failure_revokes_transient_credentials_before_quarantine() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        application
            .register_runtime(
                2,
                pi_installation_named(temporary.path(), "pi-cleanup-fail"),
                NOW + 2,
            )
            .unwrap();
        let shutdowns = Arc::new(Mutex::new(0));
        let revocations = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::new(Mutex::new(0)),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: Some(Arc::clone(&revocations)),
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: Some("pi-cleanup-fail".into()),
            },
        );
        host.activate_runtime(3, "pi-cleanup-fail", NOW + 3)
            .unwrap();

        assert!(matches!(
            host.pump_runtime_once("pi-cleanup-fail", NOW + 4),
            Err(RuntimeProductionApplicationError::OpenCodeBrokerGenerationQuarantined { .. })
        ));
        assert_eq!(*shutdowns.lock().unwrap(), 1);
        assert_eq!(*revocations.lock().unwrap(), 1);
        assert_eq!(host.active_runtime_count().unwrap(), 0);
    }

    #[test]
    fn stale_shutdown_generation_has_no_native_side_effect() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let shutdowns = Arc::new(Mutex::new(0));
        let pumps = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::clone(&pumps),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: None,
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: None,
            },
        );
        let activated = host.activate_runtime(2, "pi-primary", NOW + 2).unwrap();
        application
            .publish_runtime_policy_authority(activated.coordinator_generation, 1, 2, 0, NOW + 3)
            .unwrap();

        assert!(matches!(
            host.shutdown_runtime(activated.coordinator_generation, "pi-primary", NOW + 4),
            Err(RuntimeProductionApplicationError::Application(
                crate::RuntimeApplicationError::Generation { .. }
            ))
        ));
        assert_eq!(*shutdowns.lock().unwrap(), 0);
        assert_eq!(host.pump_runtime_once("pi-primary", NOW + 5).unwrap(), 1);
        assert_eq!(*pumps.lock().unwrap(), 1);
        let ready = application.snapshot(NOW + 5).unwrap();
        assert_eq!(ready.runtimes.records[0].lifecycle, RuntimeLifecycle::Ready);

        host.shutdown_runtime(ready.generation, "pi-primary", NOW + 6)
            .unwrap();
        assert_eq!(*shutdowns.lock().unwrap(), 1);
    }

    #[test]
    fn generation_fatal_broker_write_terminates_and_deactivates_the_worker() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let shutdowns = Arc::new(Mutex::new(0));
        let revocations = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::new(Mutex::new(0)),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: Some(Arc::clone(&revocations)),
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: Some("pi-primary".into()),
            },
        );
        host.activate_runtime(2, "pi-primary", NOW + 2).unwrap();

        assert!(matches!(
            host.pump_runtime_once("pi-primary", NOW + 3),
            Err(RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(_))
        ));
        assert_eq!(*shutdowns.lock().unwrap(), 1);
        assert_eq!(*revocations.lock().unwrap(), 1);
        assert_eq!(host.active_runtime_count().unwrap(), 0);
        assert!(matches!(
            host.pump_runtime_once("pi-primary", NOW + 4),
            Err(RuntimeProductionApplicationError::NotActive)
        ));
        let stopped = application.snapshot(NOW + 4).unwrap();
        assert_eq!(
            stopped.runtimes.records[0].lifecycle,
            RuntimeLifecycle::Stopped
        );
        assert_eq!(stopped.runtimes.records[0].process_id, None);
    }

    #[test]
    fn generation_fatal_approval_write_also_terminates_and_deactivates_the_worker() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        let shutdowns = Arc::new(Mutex::new(0));
        let revocations = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::new(Mutex::new(0)),
                reject: false,
                shutdowns: Arc::clone(&shutdowns),
                revocations: Some(Arc::clone(&revocations)),
                invalidate_attach: None,
                fail_pump_runtime: None,
                fatal_pump_runtime: Some("pi-primary".into()),
            },
        );
        let activated = host.activate_runtime(2, "pi-primary", NOW + 2).unwrap();

        assert!(matches!(
            host.answer_runtime_approval(
                activated.coordinator_generation,
                "pi-primary",
                "request-1",
                "prompt-1",
                ApprovalAnswer::Deny,
                NOW + 3,
            ),
            Err(RuntimeProductionApplicationError::OpenCodeBrokerGenerationFatal(_))
        ));
        assert_eq!(*shutdowns.lock().unwrap(), 1);
        assert_eq!(*revocations.lock().unwrap(), 1);
        assert_eq!(host.active_runtime_count().unwrap(), 0);
        let stopped = application.snapshot(NOW + 4).unwrap();
        assert_eq!(
            stopped.runtimes.records[0].lifecycle,
            RuntimeLifecycle::Stopped
        );
    }

    #[test]
    fn a_failing_runtime_does_not_starve_later_runtime_pumps() {
        let temporary = TempDir::new().unwrap();
        let application = setup_service(&temporary);
        application
            .register_runtime(
                2,
                pi_installation_named(temporary.path(), "pi-secondary"),
                NOW + 2,
            )
            .unwrap();
        let pumps = Arc::new(Mutex::new(0));
        let host = RuntimeProductionApplication::with_backend(
            Arc::clone(&application),
            FakeBackend {
                captured: Arc::new(Mutex::new(Vec::new())),
                pumps: Arc::clone(&pumps),
                reject: false,
                shutdowns: Arc::new(Mutex::new(0)),
                revocations: None,
                invalidate_attach: None,
                fail_pump_runtime: Some("pi-primary".into()),
                fatal_pump_runtime: None,
            },
        );
        host.activate_runtime(3, "pi-primary", NOW + 3).unwrap();
        let first_ready = application.snapshot(NOW + 3).unwrap();
        host.activate_runtime(first_ready.generation, "pi-secondary", NOW + 4)
            .unwrap();

        assert!(matches!(
            host.pump_all_once(NOW + 5),
            Err(RuntimeProductionApplicationError::PreparationRejected)
        ));
        assert_eq!(
            *pumps.lock().unwrap(),
            2,
            "the second runtime must still be pumped after the first error"
        );

        let generation = application.snapshot(NOW + 6).unwrap().generation;
        host.shutdown_runtime(generation, "pi-primary", NOW + 6)
            .unwrap();
        let generation = application.snapshot(NOW + 7).unwrap().generation;
        host.shutdown_runtime(generation, "pi-secondary", NOW + 7)
            .unwrap();
    }
}
