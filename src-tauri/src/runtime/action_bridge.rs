//! The sealed production path between runtime action intents and the C4OS
//! Action Gateway.
//!
//! Adapters may describe a requested effect, but they cannot authorize or
//! execute it. This bridge binds the adapter's immutable intent identity to a
//! complete C4OS classification and canonical action. A worker callback can be
//! reached only from [`RuntimeActionBridge::execute`], after the Action Gateway
//! has consumed the exact single-use authorization and durably recorded the
//! pre-effect transition.

use serde_json::Value;
use thiserror::Error;

use crate::runtime::opencode;
use crate::runtime::pi::PiEventEnvelope;
use crate::security::authorization::{
    ApprovalAnswer, AuthorizationToken, CanonicalAction, LiveAuthorityState,
};
use crate::security::gateway::{
    ActionGateway, ActionGatewayError, ApprovalResponse, GatewayProposal, NormalizedActionResult,
};
use crate::security::policy::{ActionFacts, PolicyResolution};

const MAX_BINDING_BYTES: usize = 512 * 1024;
const INTENT_BINDING_ARGUMENT: &str = "c4osRuntimeIntentSha256";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeIntentIdentity {
    pub binding_sha256: String,
    pub workspace_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub run_id: String,
    pub correlation_id: String,
    pub runtime_id: String,
    pub process_generation: u64,
    pub native_request_id: String,
    pub native_tool: String,
}

impl RuntimeIntentIdentity {
    pub fn from_opencode(intent: &opencode::ActionIntent) -> Result<Self, RuntimeBridgeError> {
        Ok(Self {
            binding_sha256: intent
                .binding_sha256()
                .map_err(|_| RuntimeBridgeError::InvalidIntent)?,
            workspace_id: intent.workspace_id.clone(),
            session_id: intent.c4os_session_id.clone(),
            turn_id: intent.c4os_turn_id.clone(),
            run_id: intent.c4os_run_id.clone(),
            correlation_id: intent.correlation_id.clone(),
            runtime_id: intent.runtime_id.clone(),
            process_generation: intent.process_generation,
            native_request_id: intent.native_request_id.clone(),
            native_tool: intent.native_tool.clone(),
        })
    }

    pub fn from_pi(event: &PiEventEnvelope) -> Result<Self, RuntimeBridgeError> {
        if event.category != "tool.action_intent" || event.runtime != "pi" {
            return Err(RuntimeBridgeError::InvalidIntent);
        }
        let native_request_id = event
            .tool_call_id
            .clone()
            .ok_or(RuntimeBridgeError::InvalidIntent)?;
        let native_tool = event
            .payload
            .get("tool")
            .and_then(Value::as_str)
            .ok_or(RuntimeBridgeError::InvalidIntent)?
            .to_owned();
        if event.payload.get("authority").and_then(Value::as_str)
            != Some("c4os-action-gateway-required")
        {
            return Err(RuntimeBridgeError::InvalidIntent);
        }
        let encoded = serde_json::to_vec(event).map_err(|_| RuntimeBridgeError::InvalidIntent)?;
        if encoded.len() > MAX_BINDING_BYTES {
            return Err(RuntimeBridgeError::InvalidIntent);
        }
        Ok(Self {
            binding_sha256: sha256(&encoded),
            workspace_id: event.workspace_id.clone(),
            session_id: event.session_id.clone(),
            turn_id: event.turn_id.clone(),
            run_id: event.run_id.clone(),
            correlation_id: event.correlation_id.clone(),
            runtime_id: "pi".into(),
            process_generation: event.process_generation,
            native_request_id,
            native_tool,
        })
    }

    fn validate(&self) -> Result<(), RuntimeBridgeError> {
        if !is_sha256(&self.binding_sha256)
            || self.process_generation == 0
            || [
                &self.workspace_id,
                &self.session_id,
                &self.turn_id,
                &self.run_id,
                &self.correlation_id,
                &self.runtime_id,
                &self.native_request_id,
                &self.native_tool,
            ]
            .iter()
            .any(|value| !safe_identifier(value))
        {
            return Err(RuntimeBridgeError::InvalidIntent);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeActionProposal {
    pub intent: RuntimeIntentIdentity,
    pub facts: ActionFacts,
    pub action: CanonicalAction,
}

impl RuntimeActionProposal {
    pub fn new(
        intent: RuntimeIntentIdentity,
        facts: ActionFacts,
        mut action: CanonicalAction,
    ) -> Result<Self, RuntimeBridgeError> {
        intent.validate()?;
        let arguments = action
            .arguments
            .as_object_mut()
            .ok_or(RuntimeBridgeError::InvalidProposal)?;
        if arguments.contains_key(INTENT_BINDING_ARGUMENT) {
            return Err(RuntimeBridgeError::InvalidProposal);
        }
        arguments.insert(
            INTENT_BINDING_ARGUMENT.into(),
            Value::String(intent.binding_sha256.clone()),
        );
        action
            .validate()
            .map_err(|_| RuntimeBridgeError::InvalidProposal)?;
        if facts.workspace_id != intent.workspace_id
            || facts.session_id != intent.session_id
            || facts.runtime_id != intent.runtime_id
            || facts.native_tool != intent.native_tool
            || action.workspace_id != intent.workspace_id
            || action.session_id != intent.session_id
            || action.run_id != intent.run_id
            || action.runtime_id != intent.runtime_id
            || action.process_generation != intent.process_generation
            || action.tool_call_id != intent.native_request_id
            || action.tool != intent.native_tool
            || action.canonical_target != facts.canonical_target
            || action.environment_id != facts.environment_id
        {
            return Err(RuntimeBridgeError::BindingMismatch);
        }
        Ok(Self {
            intent,
            facts,
            action,
        })
    }
}

pub enum RuntimeGatewayDecision {
    Denied {
        resolution: PolicyResolution,
    },
    PendingApproval {
        prompt_id: String,
        resolution: PolicyResolution,
    },
    Authorized(Box<RuntimeAuthorization>),
}

/// Non-cloneable authorization retained inside the core. Its token and exact
/// action are deliberately private so a renderer or worker cannot construct,
/// extract, replay, or mutate the grant.
pub struct RuntimeAuthorization {
    token: AuthorizationToken,
    action: CanonicalAction,
    approval_prompt_id: Option<String>,
    intent_binding_sha256: String,
}

impl RuntimeAuthorization {
    pub(crate) fn active_run_identity(&self) -> (&str, &str, &str, u64) {
        (
            &self.action.session_id,
            &self.action.run_id,
            &self.action.runtime_id,
            self.action.process_generation,
        )
    }

    pub(crate) fn action_id(&self) -> &str {
        &self.action.action_id
    }
}

/// Non-constructible proof that the Action Gateway consumed the exact runtime
/// authorization and completed the C4OS worker path. Adapters may use this to
/// deliver a result, but cannot mint a successful tool resolution themselves.
pub struct RuntimeExecutionReceipt {
    runtime_id: String,
    session_id: String,
    run_id: String,
    native_request_id: String,
    process_generation: u64,
    intent_binding_sha256: String,
    result: NormalizedActionResult,
}

impl RuntimeExecutionReceipt {
    pub fn result(&self) -> &NormalizedActionResult {
        &self.result
    }

    pub(crate) fn matches_runtime_tool(
        &self,
        runtime_id: &str,
        session_id: &str,
        run_id: &str,
        native_request_id: &str,
        process_generation: u64,
    ) -> bool {
        self.runtime_id == runtime_id
            && self.session_id == session_id
            && self.run_id == run_id
            && self.native_request_id == native_request_id
            && self.process_generation == process_generation
            && is_sha256(&self.intent_binding_sha256)
    }

    pub(crate) fn normalized_result(&self) -> &NormalizedActionResult {
        &self.result
    }
}

impl std::fmt::Debug for RuntimeAuthorization {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeAuthorization")
            .field("action_id", &self.action.action_id)
            .field("intent_binding_sha256", &self.intent_binding_sha256)
            .field("approval_prompt_id", &self.approval_prompt_id)
            .field("token", &"<redacted>")
            .finish()
    }
}

pub enum RuntimeApprovalDecision {
    Denied,
    Authorized(Box<RuntimeAuthorization>),
}

pub struct RuntimeActionBridge<'a> {
    gateway: &'a mut ActionGateway,
}

impl<'a> RuntimeActionBridge<'a> {
    pub fn new(gateway: &'a mut ActionGateway) -> Self {
        Self { gateway }
    }

    pub fn propose(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, RuntimeBridgeError> {
        let RuntimeActionProposal {
            intent,
            facts,
            action,
        } = proposal;
        match self.gateway.propose(&facts, action.clone(), now_ms)? {
            GatewayProposal::Denied { resolution } => {
                Ok(RuntimeGatewayDecision::Denied { resolution })
            }
            GatewayProposal::PendingApproval { prompt, resolution } => {
                Ok(RuntimeGatewayDecision::PendingApproval {
                    prompt_id: prompt.prompt_id.clone(),
                    resolution,
                })
            }
            GatewayProposal::Authorized { token, .. } => Ok(RuntimeGatewayDecision::Authorized(
                Box::new(RuntimeAuthorization {
                    token,
                    action,
                    approval_prompt_id: None,
                    intent_binding_sha256: intent.binding_sha256,
                }),
            )),
        }
    }

    pub fn answer_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<RuntimeApprovalDecision, RuntimeBridgeError> {
        match self.gateway.answer_approval(prompt_id, answer, now_ms)? {
            ApprovalResponse::Denied { .. } => Ok(RuntimeApprovalDecision::Denied),
            ApprovalResponse::Authorized { prompt, token } => {
                let intent_binding_sha256 = prompt
                    .action
                    .arguments
                    .get(INTENT_BINDING_ARGUMENT)
                    .and_then(Value::as_str)
                    .filter(|value| is_sha256(value))
                    .ok_or(RuntimeBridgeError::BindingMismatch)?
                    .to_owned();
                Ok(RuntimeApprovalDecision::Authorized(Box::new(
                    RuntimeAuthorization {
                        token,
                        action: prompt.action,
                        approval_prompt_id: Some(prompt_id.to_owned()),
                        intent_binding_sha256,
                    },
                )))
            }
        }
    }

    pub fn execute(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        worker_effect: impl FnOnce(crate::security::gateway::ExecutionPermit) -> NormalizedActionResult,
    ) -> Result<RuntimeExecutionReceipt, RuntimeBridgeError> {
        let bound_action = authorization
            .action
            .arguments
            .get(INTENT_BINDING_ARGUMENT)
            .and_then(Value::as_str);
        if bound_action != Some(authorization.intent_binding_sha256.as_str()) {
            return Err(RuntimeBridgeError::BindingMismatch);
        }
        let receipt_identity = (
            authorization.action.runtime_id.clone(),
            authorization.action.session_id.clone(),
            authorization.action.run_id.clone(),
            authorization.action.tool_call_id.clone(),
            authorization.action.process_generation,
            authorization.intent_binding_sha256.clone(),
        );
        let result = self.gateway.execute(
            &authorization.token,
            &authorization.action,
            live,
            authorization.approval_prompt_id.as_deref(),
            now_ms,
            worker_effect,
        )?;
        Ok(RuntimeExecutionReceipt {
            runtime_id: receipt_identity.0,
            session_id: receipt_identity.1,
            run_id: receipt_identity.2,
            native_request_id: receipt_identity.3,
            process_generation: receipt_identity.4,
            intent_binding_sha256: receipt_identity.5,
            result,
        })
    }
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[derive(Debug, Error)]
pub enum RuntimeBridgeError {
    #[error("runtime action intent is invalid")]
    InvalidIntent,
    #[error("runtime action proposal is invalid")]
    InvalidProposal,
    #[error("runtime action identity does not match its canonical binding")]
    BindingMismatch,
    #[error(transparent)]
    Gateway(#[from] ActionGatewayError),
}
