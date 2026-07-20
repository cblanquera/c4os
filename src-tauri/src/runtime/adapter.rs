//! C4OS-owned peer adapter contract shared by OpenCode and Pi.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::runtime::capability::CapabilityState;
use crate::runtime::supervisor::{RUNTIME_PROTOCOL_VERSION, RuntimeKind};

pub const ADAPTER_CONTRACT_SCHEMA_VERSION: u16 = 1;
const MAX_ID_BYTES: usize = 160;
const REQUIRED_CAPABILITIES: [&str; 9] = [
    "health",
    "session-create",
    "session-resume",
    "model-discovery",
    "streaming",
    "action-intents",
    "credential-channel",
    "cancellation",
    "restart",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterAuthority {
    C4osActionGatewayOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterConformanceDescriptor {
    pub schema_version: u16,
    pub runtime_kind: RuntimeKind,
    pub adapter_version: String,
    pub native_version: String,
    pub protocol_version: u16,
    pub process_generation: u64,
    pub authority: AdapterAuthority,
    pub capabilities: BTreeMap<String, CapabilityState>,
}

impl AdapterConformanceDescriptor {
    pub fn validate(&self) -> Result<(), AdapterContractError> {
        if self.schema_version != ADAPTER_CONTRACT_SCHEMA_VERSION
            || self.protocol_version != RUNTIME_PROTOCOL_VERSION
            || self.process_generation == 0
        {
            return Err(AdapterContractError::InvalidDescriptor);
        }
        validate_id(&self.adapter_version)?;
        validate_id(&self.native_version)?;
        if self.capabilities.len() != REQUIRED_CAPABILITIES.len()
            || REQUIRED_CAPABILITIES
                .iter()
                .any(|key| !self.capabilities.contains_key(*key))
        {
            return Err(AdapterContractError::IncompleteCapabilities);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterEventIdentity {
    pub workspace_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub run_id: String,
    pub correlation_id: String,
    pub runtime_kind: RuntimeKind,
    pub process_generation: u64,
    pub sequence: u64,
}

impl AdapterEventIdentity {
    pub fn validate_against(
        &self,
        descriptor: &AdapterConformanceDescriptor,
    ) -> Result<(), AdapterContractError> {
        descriptor.validate()?;
        for value in [
            &self.workspace_id,
            &self.session_id,
            &self.turn_id,
            &self.run_id,
            &self.correlation_id,
        ] {
            validate_id(value)?;
        }
        if self.runtime_kind != descriptor.runtime_kind
            || self.process_generation != descriptor.process_generation
            || self.sequence == 0
        {
            return Err(AdapterContractError::StaleIdentity);
        }
        Ok(())
    }
}

/// Behavior states must be supplied explicitly by each peer. The shared
/// contract owns the keys, but never upgrades an adapter capability merely
/// because another peer implements it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerCapabilityClaims {
    pub health: CapabilityState,
    pub session_create: CapabilityState,
    pub session_resume: CapabilityState,
    pub model_discovery: CapabilityState,
    pub streaming: CapabilityState,
    pub action_intents: CapabilityState,
    pub credential_channel: CapabilityState,
    pub cancellation: CapabilityState,
    pub restart: CapabilityState,
}

pub fn peer_capabilities(claims: PeerCapabilityClaims) -> BTreeMap<String, CapabilityState> {
    BTreeMap::from([
        ("health".into(), claims.health),
        ("session-create".into(), claims.session_create),
        ("session-resume".into(), claims.session_resume),
        ("model-discovery".into(), claims.model_discovery),
        ("streaming".into(), claims.streaming),
        ("action-intents".into(), claims.action_intents),
        ("credential-channel".into(), claims.credential_channel),
        ("cancellation".into(), claims.cancellation),
        ("restart".into(), claims.restart),
    ])
}

fn validate_id(value: &str) -> Result<(), AdapterContractError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@' | b'/')
        })
    {
        return Err(AdapterContractError::InvalidDescriptor);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AdapterContractError {
    #[error("adapter conformance descriptor is invalid")]
    InvalidDescriptor,
    #[error("adapter capability evidence is incomplete")]
    IncompleteCapabilities,
    #[error("adapter event identity is stale or mismatched")]
    StaleIdentity,
}
