//! C4OS-owned model and route capability truth.
//!
//! Provider and runtime declarations are evidence, not authority. This module
//! keeps declared, adapter-normalized, observed, and effective layers explicit
//! and derives a narrow per-run snapshot without silently upgrading unknowns.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CAPABILITY_SCHEMA_VERSION: u16 = 1;
const MAX_IDENTIFIER_BYTES: usize = 160;
const MAX_REASON_BYTES: usize = 2_048;
const MAX_EVIDENCE_FIELDS: usize = 256;
const MAX_ATTACHMENTS: usize = 32;
const MAX_ATTACHMENT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityLayer {
    Declared,
    AdapterNormalized,
    Observed,
    Effective,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityState {
    Supported,
    Unsupported,
    Unknown,
    Degraded,
}

impl CapabilityState {
    pub fn usable(self) -> bool {
        matches!(self, Self::Supported | Self::Degraded)
    }

    fn restrictive_rank(self) -> u8 {
        match self {
            Self::Supported => 0,
            Self::Degraded => 1,
            Self::Unknown => 2,
            Self::Unsupported => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityKey {
    InputText,
    InputImage,
    InputAudio,
    InputVideo,
    InputPdf,
    OutputText,
    OutputImage,
    OutputAudio,
    OutputVideo,
    Streaming,
    Reasoning,
    ReasoningSummary,
    ToolCalling,
    ParallelToolCalling,
    StrictToolSchema,
    StreamedToolArguments,
    StructuredJson,
    StructuredJsonSchema,
    Temperature,
    TopP,
    TopK,
    StopSequences,
    Seed,
    Verbosity,
    PromptCaching,
    SessionAffinity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumericCapabilityKey {
    ContextTokens,
    InputTokens,
    OutputTokens,
    AttachmentBytes,
    AttachmentCount,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RouteIdentity {
    pub provider_id: String,
    pub endpoint_id: String,
    pub provider_model_id: String,
    pub model_revision: String,
    pub adapter_kind: String,
    pub adapter_version: String,
    pub runtime_kind: String,
    pub native_runtime_version: String,
    pub session_configuration_sha256: String,
}

impl RouteIdentity {
    pub fn validate(&self) -> Result<(), CapabilityError> {
        validate_snapshot_identifier(&self.provider_id)?;
        validate_snapshot_identifier(&self.endpoint_id)?;
        for value in [
            &self.adapter_kind,
            &self.adapter_version,
            &self.runtime_kind,
            &self.native_runtime_version,
        ] {
            validate_identifier(value)?;
        }
        validate_provider_route_value(&self.provider_model_id)?;
        validate_provider_route_value(&self.model_revision)?;
        validate_digest(&self.session_configuration_sha256)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityEvidence {
    pub state: CapabilityState,
    pub layer: CapabilityLayer,
    pub source: String,
    pub checked_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub constraints: Vec<String>,
    pub allowed_values: Vec<String>,
    pub reason: Option<String>,
}

impl CapabilityEvidence {
    pub fn validate(&self) -> Result<(), CapabilityError> {
        validate_identifier(&self.source)?;
        if self.checked_at_ms == 0
            || self
                .expires_at_ms
                .is_some_and(|expires| expires <= self.checked_at_ms)
            || self.constraints.len() > 64
            || self.allowed_values.len() > 64
            || self
                .constraints
                .iter()
                .chain(self.allowed_values.iter())
                .any(|value| !bounded_text(value, MAX_REASON_BYTES))
            || self
                .reason
                .as_deref()
                .is_some_and(|reason| !bounded_text(reason, MAX_REASON_BYTES))
            || matches!(
                self.state,
                CapabilityState::Unsupported | CapabilityState::Unknown | CapabilityState::Degraded
            ) && self.reason.is_none()
        {
            return Err(CapabilityError::InvalidEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumericCapabilityEvidence {
    pub evidence: CapabilityEvidence,
    pub maximum: Option<u64>,
    pub confidence: LimitConfidence,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LimitConfidence {
    Confirmed,
    Estimated,
    Unknown,
}

impl NumericCapabilityEvidence {
    pub fn validate(&self) -> Result<(), CapabilityError> {
        self.evidence.validate()?;
        if self.evidence.state == CapabilityState::Supported && self.maximum.is_none() {
            return Err(CapabilityError::InvalidEvidence);
        }
        if self.maximum == Some(0) {
            return Err(CapabilityError::InvalidEvidence);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityDescriptor {
    pub schema_version: u16,
    pub layer: CapabilityLayer,
    pub route: RouteIdentity,
    pub lifecycle: ModelLifecycle,
    pub features: BTreeMap<CapabilityKey, CapabilityEvidence>,
    pub numeric_limits: BTreeMap<NumericCapabilityKey, NumericCapabilityEvidence>,
    pub raw_evidence_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelLifecycle {
    Active,
    Preview,
    Deprecated,
    Unavailable,
}

impl CapabilityDescriptor {
    pub fn validate(&self) -> Result<(), CapabilityError> {
        if self.schema_version != CAPABILITY_SCHEMA_VERSION
            || self.features.len() > MAX_EVIDENCE_FIELDS
            || self.numeric_limits.len() > MAX_EVIDENCE_FIELDS
        {
            return Err(CapabilityError::InvalidDescriptor);
        }
        self.route.validate()?;
        validate_digest(&self.raw_evidence_sha256)?;
        for evidence in self.features.values() {
            evidence.validate()?;
            if evidence.layer != self.layer {
                return Err(CapabilityError::InvalidDescriptor);
            }
        }
        for numeric in self.numeric_limits.values() {
            numeric.validate()?;
            if numeric.evidence.layer != self.layer {
                return Err(CapabilityError::InvalidDescriptor);
            }
        }
        Ok(())
    }

    pub fn feature_state(&self, key: CapabilityKey) -> CapabilityState {
        self.features
            .get(&key)
            .map_or(CapabilityState::Unknown, |evidence| evidence.state)
    }

    pub fn numeric_maximum(&self, key: NumericCapabilityKey) -> Option<u64> {
        self.numeric_limits
            .get(&key)
            .filter(|evidence| evidence.evidence.state.usable())
            .and_then(|evidence| evidence.maximum)
    }
}

pub fn effective_intersection(
    layers: &[CapabilityDescriptor],
    now_ms: u64,
) -> Result<CapabilityDescriptor, CapabilityError> {
    let actual_layers = layers.iter().map(|layer| layer.layer).collect::<Vec<_>>();
    if layers.len() != 3
        || actual_layers
            .iter()
            .filter(|layer| **layer == CapabilityLayer::Declared)
            .count()
            != 1
        || actual_layers
            .iter()
            .filter(|layer| **layer == CapabilityLayer::AdapterNormalized)
            .count()
            != 1
        || actual_layers
            .iter()
            .filter(|layer| **layer == CapabilityLayer::Observed)
            .count()
            != 1
    {
        return Err(CapabilityError::MissingLayers);
    }
    let first = layers.first().ok_or(CapabilityError::MissingLayers)?;
    first.validate()?;
    if layers.iter().any(|layer| {
        layer.validate().is_err()
            || layer.route != first.route
            || layer
                .features
                .values()
                .chain(
                    layer
                        .numeric_limits
                        .values()
                        .map(|numeric| &numeric.evidence),
                )
                .any(|evidence| {
                    evidence
                        .expires_at_ms
                        .is_some_and(|expires| now_ms >= expires)
                })
    }) {
        return Err(CapabilityError::RouteOrEvidenceMismatch);
    }

    let feature_keys = layers
        .iter()
        .flat_map(|layer| layer.features.keys().copied())
        .collect::<BTreeSet<_>>();
    let mut features = BTreeMap::new();
    for key in feature_keys {
        let evidence = layers
            .iter()
            .map(|layer| layer.features.get(&key))
            .collect::<Vec<_>>();
        let state = evidence
            .iter()
            .flatten()
            .map(|value| value.state)
            .chain(
                (evidence.iter().any(|value| value.is_none())).then_some(CapabilityState::Unknown),
            )
            .max_by_key(|state| state.restrictive_rank())
            .unwrap_or(CapabilityState::Unknown);
        features.insert(key, derived_evidence(state, key, evidence, now_ms));
    }

    let numeric_keys = layers
        .iter()
        .flat_map(|layer| layer.numeric_limits.keys().copied())
        .collect::<BTreeSet<_>>();
    let mut numeric_limits = BTreeMap::new();
    for key in numeric_keys {
        let evidence = layers
            .iter()
            .map(|layer| layer.numeric_limits.get(&key))
            .collect::<Vec<_>>();
        let state = evidence
            .iter()
            .flatten()
            .map(|value| value.evidence.state)
            .chain(
                (evidence.iter().any(|value| value.is_none())).then_some(CapabilityState::Unknown),
            )
            .max_by_key(|state| state.restrictive_rank())
            .unwrap_or(CapabilityState::Unknown);
        let maximum = evidence
            .iter()
            .flatten()
            .filter(|value| value.evidence.state.usable())
            .filter_map(|value| value.maximum)
            .min();
        let confidence = if evidence
            .iter()
            .flatten()
            .all(|value| value.confidence == LimitConfidence::Confirmed)
        {
            LimitConfidence::Confirmed
        } else if maximum.is_some() {
            LimitConfidence::Estimated
        } else {
            LimitConfidence::Unknown
        };
        let sources = evidence
            .iter()
            .flatten()
            .map(|value| Some(&value.evidence))
            .collect::<Vec<_>>();
        numeric_limits.insert(
            key,
            NumericCapabilityEvidence {
                evidence: derived_evidence(state, key, sources, now_ms),
                maximum,
                confidence,
            },
        );
    }

    let lifecycle = layers
        .iter()
        .map(|layer| layer.lifecycle)
        .max_by_key(|lifecycle| match lifecycle {
            ModelLifecycle::Active => 0,
            ModelLifecycle::Preview => 1,
            ModelLifecycle::Deprecated => 2,
            ModelLifecycle::Unavailable => 3,
        })
        .unwrap_or(ModelLifecycle::Unavailable);
    let raw_evidence_sha256 = digest_descriptors(layers)?;
    Ok(CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Effective,
        route: first.route.clone(),
        lifecycle,
        features,
        numeric_limits,
        raw_evidence_sha256,
    })
}

fn derived_evidence<K: std::fmt::Debug>(
    state: CapabilityState,
    key: K,
    sources: Vec<Option<&CapabilityEvidence>>,
    now_ms: u64,
) -> CapabilityEvidence {
    let mut constraints = sources
        .iter()
        .flatten()
        .flat_map(|source| source.constraints.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    constraints.truncate(64);
    let allowed_sets = sources
        .iter()
        .flatten()
        .map(|source| {
            source
                .allowed_values
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .filter(|values| !values.is_empty())
        .collect::<Vec<_>>();
    let allowed_values = allowed_sets
        .first()
        .map(|first| {
            allowed_sets
                .iter()
                .skip(1)
                .fold(first.clone(), |values, next| {
                    values.intersection(next).cloned().collect()
                })
        })
        .unwrap_or_default()
        .into_iter()
        .collect();
    let reason = (!matches!(state, CapabilityState::Supported)).then(|| {
        let reasons = sources
            .iter()
            .flatten()
            .filter_map(|source| source.reason.as_deref())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join("; ");
        if reasons.is_empty() {
            format!("{key:?} is not confirmed by every required layer")
        } else {
            reasons.chars().take(MAX_REASON_BYTES).collect()
        }
    });
    CapabilityEvidence {
        state,
        layer: CapabilityLayer::Effective,
        source: "c4os.effective-intersection".into(),
        checked_at_ms: now_ms,
        expires_at_ms: sources
            .iter()
            .flatten()
            .filter_map(|source| source.expires_at_ms)
            .min(),
        constraints,
        allowed_values,
        reason,
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DraftRequirements {
    pub attachments: Vec<AttachmentRequirement>,
    pub reasoning_mode: Option<String>,
    pub requires_tools: bool,
    pub requires_json_schema: bool,
    pub prefers_streaming: bool,
    pub estimated_input_tokens: u64,
    pub requested_output_tokens: u64,
    pub installed_resources: InstalledResourcePreflight,
    pub policy: PolicyPreflight,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstalledResourcePreflight {
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub tool_ids: BTreeSet<String>,
    pub attachment_converters: BTreeSet<AttachmentMediaType>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyPreflight {
    pub snapshot_id: String,
    pub version: u64,
    pub tool_use_allowed: bool,
    pub attachment_conversion_allowed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttachmentRequirement {
    pub attachment_id: String,
    pub media_type: AttachmentMediaType,
    pub mime_type: String,
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentMediaType {
    Image,
    Audio,
    Video,
    Pdf,
    OtherFile,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum PreflightOutcome {
    Ready { warnings: Vec<PreflightIssue> },
    Blocked { issues: Vec<PreflightIssue> },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreflightIssue {
    pub code: String,
    pub subject_id: Option<String>,
    pub message: String,
    pub resolutions: Vec<PreflightResolution>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreflightResolution {
    ChangeModel,
    Convert,
    Remove,
    ChangeControl,
    ReduceContext,
    Cancel,
}

pub fn preflight(
    descriptor: &CapabilityDescriptor,
    draft: &DraftRequirements,
) -> Result<PreflightOutcome, CapabilityError> {
    descriptor.validate()?;
    if draft.attachments.len() > MAX_ATTACHMENTS
        || draft.requested_output_tokens == 0
        || draft.policy.version == 0
        || validate_snapshot_identifier(&draft.installed_resources.snapshot_id).is_err()
        || validate_digest(&draft.installed_resources.snapshot_sha256).is_err()
        || validate_identifier(&draft.policy.snapshot_id).is_err()
        || draft
            .installed_resources
            .tool_ids
            .iter()
            .any(|tool| validate_identifier(tool).is_err())
        || draft
            .attachments
            .iter()
            .any(|attachment| attachment.bytes == 0 || attachment.bytes > MAX_ATTACHMENT_BYTES)
    {
        return Err(CapabilityError::InvalidDraft);
    }
    let mut blocking = Vec::new();
    let mut warnings = Vec::new();
    if descriptor.lifecycle == ModelLifecycle::Unavailable {
        blocking.push(issue(
            "model-unavailable",
            None,
            "The selected model route is currently unavailable.",
            vec![
                PreflightResolution::ChangeModel,
                PreflightResolution::Cancel,
            ],
        ));
    }

    let total_attachment_bytes = draft
        .attachments
        .iter()
        .try_fold(0_u64, |total, attachment| {
            total.checked_add(attachment.bytes)
        })
        .ok_or(CapabilityError::InvalidDraft)?;
    let attachment_count =
        u64::try_from(draft.attachments.len()).map_err(|_| CapabilityError::InvalidDraft)?;
    let attachment_limits_exceeded = descriptor
        .numeric_maximum(NumericCapabilityKey::AttachmentBytes)
        .is_some_and(|maximum| total_attachment_bytes > maximum)
        || descriptor
            .numeric_maximum(NumericCapabilityKey::AttachmentCount)
            .is_some_and(|maximum| attachment_count > maximum);
    if attachment_limits_exceeded {
        blocking.push(issue(
            "attachment-limit",
            None,
            "The draft exceeds the selected route's confirmed attachment limit.",
            vec![
                PreflightResolution::Remove,
                PreflightResolution::ChangeModel,
                PreflightResolution::Cancel,
            ],
        ));
    }
    for attachment in &draft.attachments {
        validate_identifier(&attachment.attachment_id)?;
        if !bounded_text(&attachment.mime_type, 256) {
            return Err(CapabilityError::InvalidDraft);
        }
        let key = match attachment.media_type {
            AttachmentMediaType::Image => CapabilityKey::InputImage,
            AttachmentMediaType::Audio => CapabilityKey::InputAudio,
            AttachmentMediaType::Video => CapabilityKey::InputVideo,
            AttachmentMediaType::Pdf => CapabilityKey::InputPdf,
            AttachmentMediaType::OtherFile => CapabilityKey::InputText,
        };
        let state = descriptor.feature_state(key);
        let mut resolutions = vec![PreflightResolution::ChangeModel];
        if draft.policy.attachment_conversion_allowed
            && draft
                .installed_resources
                .attachment_converters
                .contains(&attachment.media_type)
        {
            resolutions.push(PreflightResolution::Convert);
        }
        resolutions.extend([PreflightResolution::Remove, PreflightResolution::Cancel]);
        push_state_issue(
            state,
            "attachment-incompatible",
            Some(&attachment.attachment_id),
            "The selected model route cannot accept this attachment as-is.",
            resolutions,
            &mut blocking,
            &mut warnings,
        );
    }
    if let Some(reasoning) = draft.reasoning_mode.as_deref() {
        if !bounded_text(reasoning, 160) {
            return Err(CapabilityError::InvalidDraft);
        }
        let evidence = descriptor.features.get(&CapabilityKey::Reasoning);
        let state = evidence.map_or(CapabilityState::Unknown, |value| value.state);
        let allowed = evidence.is_some_and(|value| {
            value.allowed_values.is_empty()
                || value.allowed_values.iter().any(|value| value == reasoning)
        });
        if !allowed || !state.usable() {
            blocking.push(issue(
                "reasoning-incompatible",
                None,
                "The selected reasoning control is not valid for this model route.",
                vec![
                    PreflightResolution::ChangeControl,
                    PreflightResolution::ChangeModel,
                    PreflightResolution::Cancel,
                ],
            ));
        }
    }
    if draft.requires_tools {
        if draft.installed_resources.tool_ids.is_empty() {
            blocking.push(issue(
                "tools-not-installed",
                None,
                "This draft requires a C4OS tool, but no eligible tool resource is installed.",
                vec![PreflightResolution::Cancel],
            ));
        }
        if !draft.policy.tool_use_allowed {
            blocking.push(issue(
                "tools-policy-blocked",
                None,
                "The current policy snapshot does not allow tool use for this draft.",
                vec![
                    PreflightResolution::ChangeControl,
                    PreflightResolution::Cancel,
                ],
            ));
        }
        push_state_issue(
            descriptor.feature_state(CapabilityKey::ToolCalling),
            "tools-incompatible",
            None,
            "This draft requires tool calling, which is not confirmed for the selected route.",
            vec![
                PreflightResolution::ChangeModel,
                PreflightResolution::Cancel,
            ],
            &mut blocking,
            &mut warnings,
        );
    }
    if draft.requires_json_schema {
        push_state_issue(
            descriptor.feature_state(CapabilityKey::StructuredJsonSchema),
            "schema-incompatible",
            None,
            "Schema-constrained output is unavailable for the selected route.",
            vec![
                PreflightResolution::ChangeModel,
                PreflightResolution::ChangeControl,
                PreflightResolution::Cancel,
            ],
            &mut blocking,
            &mut warnings,
        );
    }
    if draft.prefers_streaming
        && descriptor.feature_state(CapabilityKey::Streaming) != CapabilityState::Supported
    {
        warnings.push(issue(
            "streaming-unavailable",
            None,
            "This route will use a visibly complete-response path.",
            vec![PreflightResolution::ChangeModel],
        ));
    }
    let input_limit = descriptor
        .numeric_maximum(NumericCapabilityKey::InputTokens)
        .or_else(|| descriptor.numeric_maximum(NumericCapabilityKey::ContextTokens));
    let output_limit = descriptor.numeric_maximum(NumericCapabilityKey::OutputTokens);
    if input_limit.is_none_or(|limit| draft.estimated_input_tokens > limit)
        || output_limit.is_none_or(|limit| draft.requested_output_tokens > limit)
    {
        blocking.push(issue(
            "token-limit",
            None,
            "The draft or requested output exceeds a confirmed route limit.",
            vec![
                PreflightResolution::ReduceContext,
                PreflightResolution::ChangeModel,
                PreflightResolution::Cancel,
            ],
        ));
    }
    Ok(if blocking.is_empty() {
        PreflightOutcome::Ready { warnings }
    } else {
        PreflightOutcome::Blocked { issues: blocking }
    })
}

fn push_state_issue(
    state: CapabilityState,
    code: &str,
    subject_id: Option<&str>,
    message: &str,
    resolutions: Vec<PreflightResolution>,
    blocking: &mut Vec<PreflightIssue>,
    warnings: &mut Vec<PreflightIssue>,
) {
    match state {
        CapabilityState::Supported => {}
        CapabilityState::Degraded => warnings.push(issue(code, subject_id, message, resolutions)),
        CapabilityState::Unknown | CapabilityState::Unsupported => {
            blocking.push(issue(code, subject_id, message, resolutions))
        }
    }
}

fn issue(
    code: &str,
    subject_id: Option<&str>,
    message: &str,
    resolutions: Vec<PreflightResolution>,
) -> PreflightIssue {
    PreflightIssue {
        code: code.into(),
        subject_id: subject_id.map(str::to_owned),
        message: message.into(),
        resolutions,
    }
}

fn digest_descriptors(layers: &[CapabilityDescriptor]) -> Result<String, CapabilityError> {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;

    let bytes = serde_json::to_vec(layers).map_err(|_| CapabilityError::Serialization)?;
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    Ok(encoded)
}

fn validate_identifier(value: &str) -> Result<(), CapabilityError> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
    {
        return Err(CapabilityError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_snapshot_identifier(value: &str) -> Result<(), CapabilityError> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@' | b':')
        })
    {
        return Err(CapabilityError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_provider_route_value(value: &str) -> Result<(), CapabilityError> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || value.contains("://")
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'_' | b'.' | b'@' | b'/' | b':' | b'+')
        })
    {
        return Err(CapabilityError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), CapabilityError> {
    if value.strip_prefix("sha256:").is_none_or(|digest| {
        digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    }) {
        return Err(CapabilityError::InvalidDigest);
    }
    Ok(())
}

fn bounded_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum CapabilityError {
    #[error("capability descriptor is invalid")]
    InvalidDescriptor,
    #[error("capability evidence is invalid")]
    InvalidEvidence,
    #[error("capability identifier is invalid")]
    InvalidIdentifier,
    #[error("capability digest is invalid")]
    InvalidDigest,
    #[error("capability layers are missing")]
    MissingLayers,
    #[error("capability route or evidence does not match")]
    RouteOrEvidenceMismatch,
    #[error("draft requirements are invalid")]
    InvalidDraft,
    #[error("capability serialization failed")]
    Serialization,
}
