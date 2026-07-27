//! Rust-owned provider profiles, route discovery, and onboarding readiness.
//!
//! Product records contain only opaque credential references. A concrete
//! `ProviderProbe` is responsible for obtaining an operation-scoped credential
//! lease and must return normalized, non-secret discovery results.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::{self, Write},
    path::{Component, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, LimitConfidence, ModelLifecycle, NumericCapabilityEvidence,
    NumericCapabilityKey, RouteIdentity,
};
use crate::runtime::opencode::{
    CommandDriver, NormalizedModel, OpenCodeAdapter, OpenCodeTransport,
};
use crate::security::credentials::{
    CredentialReference, CredentialVault, CredentialVaultError, OperationCredentialLease,
};

pub const PROVIDER_SCHEMA_VERSION: u16 = 1;
pub const PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION: u16 = 1;
pub const MAX_PROVIDER_PROFILES: usize = 128;
pub const MAX_MODELS_PER_PROVIDER: usize = 512;
pub const PROVIDER_TEST_FRESHNESS_MS: u64 = 5 * 60 * 1_000;
const PROVIDER_CREDENTIAL_LEASE_TTL: Duration = Duration::from_secs(30);
const PROVIDER_CONNECTION_EVIDENCE_SCHEMA_VERSION: u16 = 1;
const MAX_ID_BYTES: usize = 160;
const MAX_LABEL_BYTES: usize = 256;
const MAX_ENDPOINT_BYTES: usize = 2_048;
const MAX_PROVIDER_HEADERS: usize = 32;
const MAX_HEADER_NAME_BYTES: usize = 128;
const MAX_HEADER_VALUE_BYTES: usize = 2_048;
const MAX_DECLARATION_CLAIMS: usize = 256;
const MAX_CLAIM_TEXT_BYTES: usize = 2_048;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    OpenAi,
    Anthropic,
    Gemini,
    OpenRouter,
    HuggingFace,
    Custom,
}

impl ProviderKind {
    pub fn fields(self) -> Vec<ProviderField> {
        let mut fields = vec![ProviderField {
            key: ProviderFieldKey::Credential,
            label: "API key".into(),
            required: true,
            secret: true,
        }];
        if self == Self::Custom {
            fields.insert(
                0,
                ProviderField {
                    key: ProviderFieldKey::Endpoint,
                    label: "HTTPS endpoint".into(),
                    required: true,
                    secret: false,
                },
            );
        }
        fields
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", deny_unknown_fields)]
pub enum ProviderAuthentication {
    Bearer,
    ApiKeyHeader { header_name: String },
    None,
}

impl Default for ProviderAuthentication {
    fn default() -> Self {
        Self::Bearer
    }
}

impl ProviderAuthentication {
    pub fn requires_credential(&self) -> bool {
        !matches!(self, Self::None)
    }

    fn validate(&self) -> Result<(), ProviderError> {
        match self {
            Self::Bearer | Self::None => Ok(()),
            Self::ApiKeyHeader { header_name } if valid_header_name(header_name) => Ok(()),
            Self::ApiKeyHeader { .. } => Err(ProviderError::InvalidProfile),
        }
    }

    fn header_name(&self) -> Option<&str> {
        match self {
            Self::Bearer => Some("authorization"),
            Self::ApiKeyHeader { header_name } => Some(header_name),
            Self::None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderFieldKey {
    Endpoint,
    Credential,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderField {
    pub key: ProviderFieldKey,
    pub label: String,
    pub required: bool,
    pub secret: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderEndpoint {
    pub endpoint_id: String,
    pub base_url: String,
    pub api_kind: String,
}

impl ProviderEndpoint {
    fn validate_for(&self, kind: ProviderKind) -> Result<(), ProviderError> {
        validate_id(&self.endpoint_id)?;
        validate_id(&self.api_kind)?;
        validate_provider_endpoint(kind, &self.base_url)?;
        let expected_api_kind = match kind {
            ProviderKind::OpenAi => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Gemini => "google",
            ProviderKind::OpenRouter | ProviderKind::HuggingFace | ProviderKind::Custom => {
                "openai-compatible"
            }
        };
        if self.api_kind != expected_api_kind {
            return Err(ProviderError::InvalidEndpoint);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProfile {
    pub schema_version: u16,
    pub provider_id: String,
    pub kind: ProviderKind,
    pub display_name: String,
    pub endpoint: ProviderEndpoint,
    pub authentication: ProviderAuthentication,
    #[serde(default)]
    pub credential_reference: Option<CredentialReference>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub enabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderProfileWire {
    schema_version: u16,
    provider_id: String,
    kind: ProviderKind,
    display_name: String,
    endpoint: ProviderEndpoint,
    #[serde(default)]
    authentication: Option<ProviderAuthentication>,
    #[serde(default)]
    credential_reference: Option<CredentialReference>,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    enabled: bool,
}

impl<'de> Deserialize<'de> for ProviderProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProviderProfileWire::deserialize(deserializer)?;
        let authentication = wire
            .authentication
            .unwrap_or_else(|| legacy_provider_authentication(wire.kind));
        Ok(Self {
            schema_version: wire.schema_version,
            provider_id: wire.provider_id,
            kind: wire.kind,
            display_name: wire.display_name,
            endpoint: wire.endpoint,
            authentication,
            credential_reference: wire.credential_reference,
            headers: wire.headers,
            enabled: wire.enabled,
        })
    }
}

fn legacy_provider_authentication(kind: ProviderKind) -> ProviderAuthentication {
    match kind {
        ProviderKind::Anthropic => ProviderAuthentication::ApiKeyHeader {
            header_name: "x-api-key".into(),
        },
        ProviderKind::Gemini => ProviderAuthentication::ApiKeyHeader {
            header_name: "x-goog-api-key".into(),
        },
        ProviderKind::OpenAi
        | ProviderKind::OpenRouter
        | ProviderKind::HuggingFace
        | ProviderKind::Custom => ProviderAuthentication::Bearer,
    }
}

impl ProviderProfile {
    pub fn validate(&self) -> Result<(), ProviderError> {
        if self.schema_version != PROVIDER_SCHEMA_VERSION || !bounded_label(&self.display_name) {
            return Err(ProviderError::InvalidProfile);
        }
        self.authentication.validate()?;
        if self.authentication.requires_credential()
            != self
                .credential_reference
                .as_ref()
                .is_some_and(valid_credential_reference)
        {
            return Err(ProviderError::InvalidProfile);
        }
        validate_literal_headers(&self.headers)?;
        if self
            .authentication
            .header_name()
            .is_some_and(|auth_header| {
                self.headers
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case(auth_header))
            })
        {
            return Err(ProviderError::InvalidProfile);
        }
        validate_id(&self.provider_id)?;
        self.endpoint.validate_for(self.kind)
    }

    pub fn native_provider_id(&self) -> &str {
        match self.kind {
            ProviderKind::OpenAi => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Gemini => "google",
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::HuggingFace => "huggingface",
            ProviderKind::Custom => &self.provider_id,
        }
    }

    /// OpenCode configuration is profile-qualified: two saved accounts of the
    /// same provider kind may have different endpoints and credentials, so a
    /// kind-wide native identifier would collapse distinct authority routes.
    pub fn opencode_native_provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Pi's pinned SDK catalog uses provider-kind identifiers. Standard kinds
    /// are accepted only at their exact catalog endpoint. A tested Custom
    /// OpenAI-compatible profile reuses the pinned OpenAI model implementation
    /// while Rust supplies its validated HTTPS base URL per native session.
    pub fn pi_native_provider_id(&self) -> Option<&'static str> {
        match (
            self.kind,
            self.endpoint.base_url.as_str(),
            self.endpoint.api_kind.as_str(),
        ) {
            (ProviderKind::OpenAi, "https://api.openai.com/v1", "openai") => Some("openai"),
            (ProviderKind::Anthropic, "https://api.anthropic.com", "anthropic") => {
                Some("anthropic")
            }
            (
                ProviderKind::Gemini,
                "https://generativelanguage.googleapis.com/v1beta",
                "google",
            ) => Some("google"),
            (ProviderKind::OpenRouter, "https://openrouter.ai/api/v1", "openai-compatible") => {
                Some("openrouter")
            }
            (
                ProviderKind::HuggingFace,
                "https://router.huggingface.co/v1",
                "openai-compatible",
            ) => Some("openai"),
            (ProviderKind::Custom, _, "openai-compatible") => Some("openai"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteAvailability {
    Available,
    Degraded,
    Unavailable,
    Unknown,
}

/// Runtime-neutral model claims captured from the provider catalog before any
/// OpenCode or Pi adapter normalization is applied. Production composition can
/// project the same declaration onto either adapter route without relabelling
/// one adapter's evidence as provider truth.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderModelDeclaration {
    pub schema_version: u16,
    pub provider_model_id: String,
    pub model_revision: String,
    pub lifecycle: ModelLifecycle,
    pub features: BTreeMap<CapabilityKey, ProviderFeatureClaim>,
    pub numeric_limits: BTreeMap<NumericCapabilityKey, ProviderNumericClaim>,
    pub raw_catalog_sha256: String,
    pub declared_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderFeatureClaim {
    pub state: CapabilityState,
    pub constraints: Vec<String>,
    pub allowed_values: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderNumericClaim {
    pub state: CapabilityState,
    pub maximum: Option<u64>,
    pub confidence: LimitConfidence,
    pub reason: Option<String>,
}

impl ProviderModelDeclaration {
    fn validate_for(&self, route: &ModelRoute) -> Result<(), ProviderError> {
        if self.schema_version != PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION
            || self.provider_model_id != route.model_id
            || !bounded_label(&self.model_revision)
            || self.features.len() > MAX_DECLARATION_CLAIMS
            || self.numeric_limits.len() > MAX_DECLARATION_CLAIMS
            || !valid_sha256(&self.raw_catalog_sha256)
            || self.declared_at_ms == 0
            || self.declared_at_ms != route.checked_at_ms
            || self.expires_at_ms <= self.declared_at_ms
            || self.features.values().any(|claim| !claim.is_valid())
            || self.numeric_limits.values().any(|claim| !claim.is_valid())
        {
            return Err(ProviderError::InvalidDiscovery);
        }
        Ok(())
    }
}

impl ProviderFeatureClaim {
    fn is_valid(&self) -> bool {
        valid_claim_text(
            &self.constraints,
            &self.allowed_values,
            self.reason.as_deref(),
        ) && (self.state == CapabilityState::Supported || self.reason.is_some())
    }
}

impl ProviderNumericClaim {
    fn is_valid(&self) -> bool {
        self.maximum != Some(0)
            && (self.state != CapabilityState::Supported || self.maximum.is_some())
            && (self.state == CapabilityState::Supported || self.reason.is_some())
            && valid_claim_text(&[], &[], self.reason.as_deref())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelRoute {
    pub model_id: String,
    pub display_name: String,
    pub recommendation_rank: u32,
    pub availability: RouteAvailability,
    pub checked_at_ms: u64,
    pub capabilities: CapabilityDescriptor,
    /// `None` is accepted only for backwards-compatible restore. Such a route
    /// remains inspectable but is not production-ready until provider retest.
    #[serde(default)]
    pub provider_declaration: Option<ProviderModelDeclaration>,
}

impl ModelRoute {
    pub fn validate_for(&self, profile: &ProviderProfile) -> Result<(), ProviderError> {
        validate_model_id(&self.model_id)?;
        if !bounded_label(&self.display_name)
            || self.checked_at_ms == 0
            || self.capabilities.validate().is_err()
            || self.capabilities.route.provider_id != profile.provider_id
            || self.capabilities.route.endpoint_id != profile.endpoint.endpoint_id
        {
            return Err(ProviderError::InvalidDiscovery);
        }
        if let Some(declaration) = &self.provider_declaration {
            declaration.validate_for(self)?;
        }
        Ok(())
    }

    pub fn is_usable(&self) -> bool {
        matches!(
            self.availability,
            RouteAvailability::Available | RouteAvailability::Degraded
        ) && self.capabilities.lifecycle != ModelLifecycle::Unavailable
    }

    /// A restored legacy route can remain visible, but dispatch and onboarding
    /// must wait for a provider retest that captures runtime-neutral claims.
    pub fn is_production_ready(&self) -> bool {
        self.is_usable() && self.provider_declaration.is_some()
    }

    pub fn is_production_ready_at(&self, now_ms: u64) -> bool {
        self.is_production_ready()
            && self
                .provider_declaration
                .as_ref()
                .is_some_and(|declaration| {
                    declaration.declared_at_ms <= now_ms && now_ms < declaration.expires_at_ms
                })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderDiscovery {
    pub checked_at_ms: u64,
    pub models: Vec<ModelRoute>,
    pub recommended_model_id: Option<String>,
    /// A successful catalog lookup is not a provider connection test. This
    /// evidence is required before discovery can enter product state.
    pub connection_evidence: Option<ProviderConnectionEvidence>,
}

impl ProviderDiscovery {
    fn validate(&self, profile: &ProviderProfile) -> Result<(), ProviderError> {
        if self.checked_at_ms == 0 || self.models.len() > MAX_MODELS_PER_PROVIDER {
            return Err(ProviderError::InvalidDiscovery);
        }
        self.connection_evidence
            .as_ref()
            .ok_or(ProviderError::MissingConnectionProof)?
            .validate_for(profile, self.checked_at_ms)?;
        let unique = self
            .models
            .iter()
            .map(|route| &route.model_id)
            .collect::<BTreeSet<_>>();
        if unique.len() != self.models.len()
            || self
                .models
                .iter()
                .any(|route| route.checked_at_ms != self.checked_at_ms)
        {
            return Err(ProviderError::InvalidDiscovery);
        }
        if self
            .models
            .iter()
            .any(|route| route.provider_declaration.is_none())
        {
            return Err(ProviderError::MissingModelDeclaration);
        }
        self.models
            .iter()
            .try_for_each(|route| route.validate_for(profile))?;
        if self.recommended_model_id.as_deref().is_some_and(|id| {
            !self
                .models
                .iter()
                .any(|route| route.model_id == id && route.is_production_ready())
        }) {
            return Err(ProviderError::InvalidDiscovery);
        }
        Ok(())
    }
}

/// Non-secret evidence that the exact saved profile was exercised with its
/// operation-scoped credential. The evidence is persisted with provider state
/// so restore and readiness checks cannot rely on a catalog-only success.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderConnectionEvidence {
    pub schema_version: u16,
    pub provider_id: String,
    pub endpoint_id: String,
    pub endpoint_sha256: String,
    pub credential_reference_sha256: String,
    pub profile_binding_sha256: String,
    pub tested_at_ms: u64,
    pub response_sha256: String,
}

impl ProviderConnectionEvidence {
    /// Builds evidence only after a trusted connectivity boundary has consumed
    /// the provider credential lease and returned a bounded response digest.
    pub fn from_tested_profile(
        profile: &ProviderProfile,
        tested_at_ms: u64,
        response_sha256: impl Into<String>,
    ) -> Result<Self, ProviderProbeFailure> {
        let response_sha256 = response_sha256.into();
        if tested_at_ms == 0 || !valid_sha256(&response_sha256) {
            return Err(ProviderProbeFailure::Incompatible);
        }
        Ok(Self {
            schema_version: PROVIDER_CONNECTION_EVIDENCE_SCHEMA_VERSION,
            provider_id: profile.provider_id.clone(),
            endpoint_id: profile.endpoint.endpoint_id.clone(),
            endpoint_sha256: sha256_json(&profile.endpoint)?,
            credential_reference_sha256: sha256_bytes(credential_binding(profile).as_bytes()),
            profile_binding_sha256: provider_profile_binding(profile, tested_at_ms)?,
            tested_at_ms,
            response_sha256,
        })
    }

    pub(crate) fn validate_for(
        &self,
        profile: &ProviderProfile,
        tested_at_ms: u64,
    ) -> Result<(), ProviderError> {
        if self.schema_version != PROVIDER_CONNECTION_EVIDENCE_SCHEMA_VERSION
            || self.provider_id != profile.provider_id
            || self.endpoint_id != profile.endpoint.endpoint_id
            || self.tested_at_ms != tested_at_ms
            || !valid_sha256(&self.response_sha256)
            || self.endpoint_sha256
                != sha256_json(&profile.endpoint).map_err(|_| ProviderError::InvalidDiscovery)?
            || self.credential_reference_sha256
                != sha256_bytes(credential_binding(profile).as_bytes())
            || self.profile_binding_sha256
                != provider_profile_binding(profile, tested_at_ms)
                    .map_err(|_| ProviderError::InvalidDiscovery)?
        {
            return Err(ProviderError::ConnectionProofMismatch);
        }
        Ok(())
    }
}

pub trait ProviderProbe {
    /// Implementations obtain the credential through a short-lived operation
    /// channel. They must not place it in the request, response, diagnostics,
    /// command arguments, or inherited environment.
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure>;
}

/// A non-secret, endpoint-bound request to the production connectivity
/// boundary. Credentials travel separately through `OperationCredentialLease`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConnectionRequest {
    pub provider_id: String,
    pub provider_kind: ProviderKind,
    pub endpoint_id: String,
    pub base_url: String,
    pub api_kind: String,
    pub native_provider_id: String,
    pub authentication: ProviderAuthentication,
    pub headers: BTreeMap<String, String>,
    pub attempted_at_ms: u64,
    pub endpoint_sha256: String,
    pub profile_binding_sha256: String,
    pub operation: String,
}

/// A successful connectivity implementation must report which exact endpoint
/// it observed and a digest of the bounded provider response. It must consume
/// the supplied one-shot credential lease before returning success.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConnectionObservation {
    pub endpoint_sha256: String,
    pub profile_binding_sha256: String,
    pub response_sha256: String,
    pub catalog: Option<ProviderCatalog>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCatalog {
    pub models: Vec<ProviderCatalogModel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCatalogModel {
    pub model_id: String,
    pub display_name: String,
    pub input_modalities: BTreeSet<String>,
    pub output_modalities: BTreeSet<String>,
    pub supported_parameters: BTreeSet<String>,
    pub context_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// Narrow provider network boundary. Implementations may deliver the lease to
/// an anonymous request-local credential channel; they must never serialize a
/// credential into the request, response, URL, diagnostics, argv, or inherited
/// environment.
pub trait ProviderConnectivity {
    fn test_connection(
        &mut self,
        request: &ProviderConnectionRequest,
        credential_lease: Option<&OperationCredentialLease>,
    ) -> Result<ProviderConnectionObservation, ProviderProbeFailure>;
}

/// Bounded production HTTPS transport for macOS. The system curl process is
/// started without user configuration or inherited environment. Its complete
/// request configuration, including the provider credential, is written only
/// through the child's anonymous stdin pipe. No credential reaches argv,
/// environment, a filesystem path, returned evidence, or diagnostics.
pub struct CurlProviderConnectivity {
    executable: PathBuf,
    timeout_seconds: u16,
    max_response_bytes: usize,
}

impl CurlProviderConnectivity {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, ProviderError> {
        let executable = executable.into();
        if !executable.is_absolute()
            || executable
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(ProviderError::InvalidConnectivityTransport);
        }
        Ok(Self {
            executable,
            timeout_seconds: 20,
            max_response_bytes: 1024 * 1024,
        })
    }

    #[cfg(target_os = "macos")]
    pub fn macos_system() -> Result<Self, ProviderError> {
        Self::new("/usr/bin/curl")
    }
}

impl fmt::Debug for CurlProviderConnectivity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CurlProviderConnectivity")
            .field("executable", &self.executable)
            .field("timeout_seconds", &self.timeout_seconds)
            .field("max_response_bytes", &self.max_response_bytes)
            .field("credential", &"<operation-scoped>")
            .finish()
    }
}

impl ProviderConnectivity for CurlProviderConnectivity {
    fn test_connection(
        &mut self,
        request: &ProviderConnectionRequest,
        credential_lease: Option<&OperationCredentialLease>,
    ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
        validate_connection_request(request)?;
        if self.timeout_seconds == 0 || self.max_response_bytes < 4 {
            return Err(ProviderProbeFailure::Internal);
        }
        let maximum = self.max_response_bytes.to_string();
        let timeout = self.timeout_seconds.to_string();
        let allowed_protocols = if request.base_url.starts_with("http://") {
            "=http"
        } else {
            "=https"
        };
        let mut child = Command::new(&self.executable)
            .args([
                "--disable",
                "--silent",
                "--show-error",
                "--proto",
                allowed_protocols,
                "--tlsv1.2",
                "--max-time",
                &timeout,
                "--max-filesize",
                &maximum,
                "--write-out",
                "\n%{http_code}",
                "--config",
                "-",
            ])
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // curl errors can include server-controlled text; provider tests
            // expose only the stable failure taxonomy below.
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| ProviderProbeFailure::Network)?;

        let Some(mut stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProviderProbeFailure::Internal);
        };
        let configuration_result = (|| {
            write_curl_prefix(&mut stdin, request).map_err(|_| ProviderProbeFailure::Internal)?;
            match (&request.authentication, credential_lease) {
                (ProviderAuthentication::None, None) => {
                    stdin.flush().map_err(|_| ProviderProbeFailure::Internal)
                }
                (ProviderAuthentication::None, Some(_)) | (_, None) => {
                    Err(ProviderProbeFailure::Internal)
                }
                (_, Some(credential_lease)) => {
                    write_curl_credential_prefix(&mut stdin, &request.authentication)
                        .map_err(|_| ProviderProbeFailure::Internal)?;
                    let mut credential_writer = CurlCredentialWriter::new(&mut stdin);
                    credential_lease
                        .deliver_to(&mut credential_writer)
                        .map_err(|error| match error {
                            CredentialVaultError::CredentialNotFound
                            | CredentialVaultError::LeaseExpired
                            | CredentialVaultError::LeaseRevoked
                            | CredentialVaultError::OperationChannel => {
                                ProviderProbeFailure::Authentication
                            }
                            _ => ProviderProbeFailure::Internal,
                        })?;
                    credential_writer
                        .finish(request)
                        .map_err(|_| ProviderProbeFailure::Authentication)
                }
            }
        })();
        drop(stdin);
        if let Err(failure) = configuration_result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(failure);
        }

        let output = child
            .wait_with_output()
            .map_err(|_| ProviderProbeFailure::Network)?;
        if !output.status.success() {
            return Err(match output.status.code() {
                Some(6 | 7 | 28) => ProviderProbeFailure::Network,
                _ => ProviderProbeFailure::Incompatible,
            });
        }
        if output.stdout.len() > self.max_response_bytes.saturating_add(4) {
            return Err(ProviderProbeFailure::Incompatible);
        }
        let (response_body, status) = split_curl_response(&output.stdout)?;
        match status {
            200..=299 => Ok(ProviderConnectionObservation {
                endpoint_sha256: request.endpoint_sha256.clone(),
                profile_binding_sha256: request.profile_binding_sha256.clone(),
                response_sha256: sha256_bytes(response_body),
                catalog: parse_provider_catalog(response_body, &request.api_kind).ok(),
            }),
            401 | 403 => Err(ProviderProbeFailure::Authentication),
            429 => Err(ProviderProbeFailure::RateLimited),
            500..=599 => Err(ProviderProbeFailure::Network),
            _ => Err(ProviderProbeFailure::Incompatible),
        }
    }
}

struct CurlCredentialWriter<'a> {
    inner: &'a mut dyn Write,
    bytes_written: usize,
    invalid: bool,
}

impl<'a> CurlCredentialWriter<'a> {
    fn new(inner: &'a mut dyn Write) -> Self {
        Self {
            inner,
            bytes_written: 0,
            invalid: false,
        }
    }

    fn finish(&mut self, request: &ProviderConnectionRequest) -> io::Result<()> {
        if self.bytes_written == 0 || self.invalid {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid provider credential",
            ));
        }
        self.inner.write_all(b"\"\n")?;
        if request.native_provider_id == "anthropic" {
            self.inner
                .write_all(b"header = \"anthropic-version: 2023-06-01\"\n")?;
        }
        self.inner.flush()
    }
}

impl Write for CurlCredentialWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // Provider API keys are printable ASCII bearer/header values. Reject
        // delimiters instead of attempting an escaping scheme that could turn
        // credential bytes into curl configuration directives.
        if bytes.is_empty()
            || bytes
                .iter()
                .any(|byte| !byte.is_ascii_graphic() || matches!(byte, b'"' | b'\\'))
        {
            self.invalid = true;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid provider credential",
            ));
        }
        self.inner.write_all(bytes)?;
        self.bytes_written = self
            .bytes_written
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("provider credential is too large"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Operation-scoped credential capability issuer. The production
/// implementation delegates to the Rust-owned credential vault.
pub trait ProviderCredentialLeaser {
    fn lease_for_provider_test(
        &self,
        reference: &CredentialReference,
        operation: &str,
    ) -> Result<OperationCredentialLease, ProviderProbeFailure>;
}

impl ProviderCredentialLeaser for CredentialVault {
    fn lease_for_provider_test(
        &self,
        reference: &CredentialReference,
        operation: &str,
    ) -> Result<OperationCredentialLease, ProviderProbeFailure> {
        self.lease_for_operation(reference, operation, PROVIDER_CREDENTIAL_LEASE_TTL)
            .map_err(map_credential_probe_failure)
    }
}

impl ProviderCredentialLeaser for Option<CredentialVault> {
    fn lease_for_provider_test(
        &self,
        reference: &CredentialReference,
        operation: &str,
    ) -> Result<OperationCredentialLease, ProviderProbeFailure> {
        self.as_ref()
            .ok_or(ProviderProbeFailure::Internal)?
            .lease_for_provider_test(reference, operation)
    }
}

/// Catalog normalization through the exact-version OpenCode adapter. This
/// probe intentionally emits no connection evidence and therefore cannot make
/// `ProviderService` report success or onboarding readiness by itself. Keep it
/// for adapter inventory conformance; production uses
/// `VerifiedOpenCodeProviderProbe` below.
pub struct OpenCodeProviderProbe<'a, T, D> {
    adapter: &'a mut OpenCodeAdapter<T, D>,
    checked_at_ms: u64,
    session_configuration_sha256: String,
}

impl<'a, T: OpenCodeTransport, D: CommandDriver> OpenCodeProviderProbe<'a, T, D> {
    pub fn new(
        adapter: &'a mut OpenCodeAdapter<T, D>,
        checked_at_ms: u64,
        session_configuration_sha256: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let session_configuration_sha256 = session_configuration_sha256.into();
        if checked_at_ms == 0 || !valid_sha256(&session_configuration_sha256) {
            return Err(ProviderError::InvalidTimestamp);
        }
        Ok(Self {
            adapter,
            checked_at_ms,
            session_configuration_sha256,
        })
    }
}

impl<T: OpenCodeTransport, D: CommandDriver> ProviderProbe for OpenCodeProviderProbe<'_, T, D> {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let native_provider_id = profile.native_provider_id();
        let models = self
            .adapter
            .list_models()
            .map_err(map_opencode_probe_failure)?
            .into_iter()
            .filter(|model| model.provider_id == native_provider_id)
            .enumerate()
            .map(|(index, model)| {
                normalized_opencode_route(
                    profile,
                    model,
                    u32::try_from(index).unwrap_or(u32::MAX),
                    self.checked_at_ms,
                    &self.session_configuration_sha256,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ProviderDiscovery {
            checked_at_ms: self.checked_at_ms,
            models,
            recommended_model_id: None,
            connection_evidence: None,
        })
    }
}

/// Production OpenCode provider test. It first proves the exact saved endpoint
/// using the saved credential reference through a one-shot lease, then joins
/// that proof to OpenCode's normalized model inventory. Either half failing is
/// a failed provider test; catalog availability alone is never accepted.
pub struct VerifiedOpenCodeProviderProbe<'a, T, D, L, C> {
    adapter: &'a mut OpenCodeAdapter<T, D>,
    credential_leaser: &'a L,
    connectivity: &'a mut C,
    checked_at_ms: u64,
    session_configuration_sha256: String,
}

impl<'a, T, D, L, C> VerifiedOpenCodeProviderProbe<'a, T, D, L, C>
where
    T: OpenCodeTransport,
    D: CommandDriver,
    L: ProviderCredentialLeaser,
    C: ProviderConnectivity,
{
    pub fn new(
        adapter: &'a mut OpenCodeAdapter<T, D>,
        credential_leaser: &'a L,
        connectivity: &'a mut C,
        checked_at_ms: u64,
        session_configuration_sha256: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let session_configuration_sha256 = session_configuration_sha256.into();
        if checked_at_ms == 0 || !valid_sha256(&session_configuration_sha256) {
            return Err(ProviderError::InvalidTimestamp);
        }
        Ok(Self {
            adapter,
            credential_leaser,
            connectivity,
            checked_at_ms,
            session_configuration_sha256,
        })
    }
}

impl<T, D, L, C> ProviderProbe for VerifiedOpenCodeProviderProbe<'_, T, D, L, C>
where
    T: OpenCodeTransport,
    D: CommandDriver,
    L: ProviderCredentialLeaser,
    C: ProviderConnectivity,
{
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        profile
            .validate()
            .map_err(|_| ProviderProbeFailure::Incompatible)?;
        let request = provider_connection_request(profile, self.checked_at_ms)?;
        let lease = profile
            .credential_reference
            .as_ref()
            .map(|reference| {
                self.credential_leaser
                    .lease_for_provider_test(reference, &request.operation)
            })
            .transpose()?;
        if lease
            .as_ref()
            .is_some_and(|lease| lease.operation() != request.operation)
        {
            return Err(ProviderProbeFailure::Internal);
        }
        let observation = self
            .connectivity
            .test_connection(&request, lease.as_ref())?;
        // A success that did not consume the exact operation lease did not
        // authenticate the saved profile and must fail closed.
        if lease
            .as_ref()
            .is_some_and(OperationCredentialLease::is_valid)
            || observation.endpoint_sha256 != request.endpoint_sha256
            || observation.profile_binding_sha256 != request.profile_binding_sha256
            || !valid_sha256(&observation.response_sha256)
        {
            return Err(ProviderProbeFailure::Incompatible);
        }

        let mut catalog = OpenCodeProviderProbe::new(
            self.adapter,
            self.checked_at_ms,
            self.session_configuration_sha256.clone(),
        )
        .map_err(|_| ProviderProbeFailure::Incompatible)?
        .test_and_discover(profile)?;
        catalog.connection_evidence = Some(ProviderConnectionEvidence::from_tested_profile(
            profile,
            self.checked_at_ms,
            observation.response_sha256,
        )?);
        Ok(catalog)
    }
}

/// First-launch provider probe for the accepted provider families. It
/// exercises the exact saved model-catalog route through the same
/// credential-safe connectivity boundary and derives only conservative
/// OpenCode-compatible catalog evidence. No Workspace or running sidecar is
/// required, so onboarding cannot become circular on an empty C4OS Home.
pub struct DirectProviderProbe<'a, L, C> {
    credential_leaser: &'a L,
    connectivity: &'a mut C,
    checked_at_ms: u64,
}

impl<'a, L, C> DirectProviderProbe<'a, L, C>
where
    L: ProviderCredentialLeaser,
    C: ProviderConnectivity,
{
    pub fn new(
        credential_leaser: &'a L,
        connectivity: &'a mut C,
        checked_at_ms: u64,
    ) -> Result<Self, ProviderError> {
        if checked_at_ms == 0 {
            return Err(ProviderError::InvalidTimestamp);
        }
        Ok(Self {
            credential_leaser,
            connectivity,
            checked_at_ms,
        })
    }
}

impl<L, C> ProviderProbe for DirectProviderProbe<'_, L, C>
where
    L: ProviderCredentialLeaser,
    C: ProviderConnectivity,
{
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        profile
            .validate()
            .map_err(|_| ProviderProbeFailure::Incompatible)?;
        if !matches!(
            profile.endpoint.api_kind.as_str(),
            "openai" | "openai-compatible" | "anthropic" | "google"
        ) {
            return Err(ProviderProbeFailure::Incompatible);
        }
        let request = provider_connection_request(profile, self.checked_at_ms)?;
        let lease = profile
            .credential_reference
            .as_ref()
            .map(|reference| {
                self.credential_leaser
                    .lease_for_provider_test(reference, &request.operation)
            })
            .transpose()?;
        if lease
            .as_ref()
            .is_some_and(|lease| lease.operation() != request.operation)
        {
            return Err(ProviderProbeFailure::Internal);
        }
        let observation = self
            .connectivity
            .test_connection(&request, lease.as_ref())?;
        if lease
            .as_ref()
            .is_some_and(OperationCredentialLease::is_valid)
            || observation.endpoint_sha256 != request.endpoint_sha256
            || observation.profile_binding_sha256 != request.profile_binding_sha256
            || !valid_sha256(&observation.response_sha256)
        {
            return Err(ProviderProbeFailure::Incompatible);
        }
        let catalog = observation
            .catalog
            .ok_or(ProviderProbeFailure::Incompatible)?;
        let mut models = Vec::with_capacity(catalog.models.len());
        for (rank, model) in catalog.models.into_iter().enumerate() {
            models.push(direct_provider_route(
                profile,
                model,
                rank.try_into()
                    .map_err(|_| ProviderProbeFailure::Internal)?,
                self.checked_at_ms,
                &observation.response_sha256,
                &request.profile_binding_sha256,
            )?);
        }
        let recommended_model_id = models
            .iter()
            .filter(|route| route.is_production_ready())
            .min_by_key(|route| (route.recommendation_rank, route.model_id.as_str()))
            .map(|route| route.model_id.clone());
        Ok(ProviderDiscovery {
            checked_at_ms: self.checked_at_ms,
            models,
            recommended_model_id,
            connection_evidence: Some(ProviderConnectionEvidence::from_tested_profile(
                profile,
                self.checked_at_ms,
                observation.response_sha256,
            )?),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProbeFailure {
    Authentication,
    Network,
    RateLimited,
    Incompatible,
    Cancelled,
    Internal,
}

impl ProviderProbeFailure {
    fn code(self) -> ProviderFailureCode {
        match self {
            Self::Authentication => ProviderFailureCode::Authentication,
            Self::Network => ProviderFailureCode::Network,
            Self::RateLimited => ProviderFailureCode::RateLimited,
            Self::Incompatible => ProviderFailureCode::Incompatible,
            Self::Cancelled => ProviderFailureCode::Cancelled,
            Self::Internal => ProviderFailureCode::Internal,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderFailureCode {
    Authentication,
    Network,
    RateLimited,
    Incompatible,
    Cancelled,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "state"
)]
pub enum ProviderTestStatus {
    Untested,
    Succeeded {
        checked_at_ms: u64,
    },
    SucceededNoUsableModels {
        checked_at_ms: u64,
    },
    Failed {
        checked_at_ms: u64,
        code: ProviderFailureCode,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderRecord {
    pub profile: ProviderProfile,
    pub test_status: ProviderTestStatus,
    pub connection_evidence: Option<ProviderConnectionEvidence>,
    pub models: BTreeMap<String, ModelRoute>,
    #[serde(default)]
    pub disabled_model_ids: BTreeSet<String>,
    pub selected_model_id: Option<String>,
    pub generation: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderSnapshot {
    pub generation: u64,
    #[serde(default)]
    pub onboarding_completed_at_ms: Option<u64>,
    pub providers: Vec<ProviderRecord>,
}

impl ProviderSnapshot {
    pub fn launch_ready(&self) -> bool {
        self.onboarding_completed_at_ms.is_some() && !self.providers.is_empty()
    }

    /// Fresh exact-profile proof used to gate the explicit onboarding commit.
    /// Launch routing uses `launch_ready` after that commit so a configured
    /// provider does not return to onboarding merely because discovery ages.
    pub fn onboarding_ready_at(&self, now_ms: u64) -> bool {
        self.providers.iter().any(|provider| {
            provider
                .selected_model_id
                .as_deref()
                .is_some_and(|model_id| provider_model_ready_at(provider, model_id, now_ms))
        })
    }

    pub fn provider_model_ready_at(&self, provider_id: &str, model_id: &str, now_ms: u64) -> bool {
        self.providers.iter().any(|provider| {
            provider.profile.provider_id == provider_id
                && provider.selected_model_id.as_deref() == Some(model_id)
                && provider_model_ready_at(provider, model_id, now_ms)
        })
    }
}

fn provider_model_ready_at(provider: &ProviderRecord, model_id: &str, now_ms: u64) -> bool {
    provider.profile.enabled
        && matches!(
            provider.test_status,
            ProviderTestStatus::Succeeded { checked_at_ms }
                if checked_at_ms <= now_ms
                    && now_ms.saturating_sub(checked_at_ms) <= PROVIDER_TEST_FRESHNESS_MS
        )
        && provider
            .connection_evidence
            .as_ref()
            .is_some_and(|evidence| {
                evidence.tested_at_ms <= now_ms
                    && now_ms.saturating_sub(evidence.tested_at_ms) <= PROVIDER_TEST_FRESHNESS_MS
                    && evidence
                        .validate_for(&provider.profile, evidence.tested_at_ms)
                        .is_ok()
            })
        && !provider.disabled_model_ids.contains(model_id)
        && provider.models.get(model_id).is_some_and(|route| {
            route.is_production_ready_at(now_ms)
                && route.checked_at_ms <= now_ms
                && now_ms.saturating_sub(route.checked_at_ms) <= PROVIDER_TEST_FRESHNESS_MS
                && route
                    .capabilities
                    .features
                    .values()
                    .chain(
                        route
                            .capabilities
                            .numeric_limits
                            .values()
                            .map(|limit| &limit.evidence),
                    )
                    .all(|evidence| {
                        evidence
                            .expires_at_ms
                            .is_none_or(|expires| now_ms < expires)
                    })
        })
}

#[derive(Default)]
pub struct ProviderService {
    generation: u64,
    onboarding_completed_at_ms: Option<u64>,
    providers: BTreeMap<String, ProviderRecord>,
}

impl ProviderService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn restore(snapshot: ProviderSnapshot) -> Result<Self, ProviderError> {
        if snapshot.onboarding_completed_at_ms == Some(0)
            || (snapshot.onboarding_completed_at_ms.is_some() && snapshot.providers.is_empty())
        {
            return Err(ProviderError::InvalidSnapshot);
        }
        let onboarding_completed_at_ms = snapshot.onboarding_completed_at_ms;
        let mut providers = BTreeMap::new();
        let mut labels = BTreeSet::new();
        for record in snapshot.providers {
            record
                .profile
                .validate()
                .map_err(|_| ProviderError::InvalidSnapshot)?;
            let normalized_label = record.profile.display_name.trim().to_lowercase();
            if record.generation > snapshot.generation
                || record
                    .models
                    .values()
                    .any(|route| route.validate_for(&record.profile).is_err())
                || record
                    .disabled_model_ids
                    .iter()
                    .any(|model_id| !record.models.contains_key(model_id))
                || match &record.test_status {
                    ProviderTestStatus::Succeeded { checked_at_ms }
                    | ProviderTestStatus::SucceededNoUsableModels { checked_at_ms } => {
                        record.connection_evidence.as_ref().is_none_or(|evidence| {
                            evidence
                                .validate_for(&record.profile, *checked_at_ms)
                                .is_err()
                        })
                    }
                    ProviderTestStatus::Untested | ProviderTestStatus::Failed { .. } => {
                        record.connection_evidence.is_some()
                    }
                }
                || record.selected_model_id.as_deref().is_some_and(|selected| {
                    record.disabled_model_ids.contains(selected)
                        || !record
                            .models
                            .get(selected)
                            .is_some_and(ModelRoute::is_usable)
                })
                || providers
                    .insert(record.profile.provider_id.clone(), record)
                    .is_some()
            {
                return Err(ProviderError::InvalidSnapshot);
            }
            if !labels.insert(normalized_label) {
                return Err(ProviderError::InvalidSnapshot);
            }
        }
        Ok(Self {
            generation: snapshot.generation,
            onboarding_completed_at_ms,
            providers,
        })
    }

    pub fn snapshot(&self) -> ProviderSnapshot {
        ProviderSnapshot {
            generation: self.generation,
            onboarding_completed_at_ms: self.onboarding_completed_at_ms,
            providers: self.providers.values().cloned().collect(),
        }
    }

    pub fn save_profile(
        &mut self,
        profile: ProviderProfile,
        expected_generation: u64,
    ) -> Result<u64, ProviderError> {
        self.expect_generation(expected_generation)?;
        profile.validate()?;
        if self.providers.values().any(|current| {
            current.profile.provider_id != profile.provider_id
                && current
                    .profile
                    .display_name
                    .trim()
                    .eq_ignore_ascii_case(profile.display_name.trim())
        }) {
            return Err(ProviderError::DuplicateDisplayName);
        }
        if !self.providers.contains_key(&profile.provider_id)
            && self.providers.len() >= MAX_PROVIDER_PROFILES
        {
            return Err(ProviderError::CapacityExceeded);
        }
        let reset_discovery = self
            .providers
            .get(&profile.provider_id)
            .is_none_or(|current| {
                current.profile.endpoint != profile.endpoint
                    || current.profile.authentication != profile.authentication
                    || current.profile.credential_reference != profile.credential_reference
                    || current.profile.headers != profile.headers
                    || current.profile.kind != profile.kind
                    || current.profile.enabled != profile.enabled
            });
        let generation = self.next_generation()?;
        let (test_status, connection_evidence, models, disabled_model_ids, selected_model_id) =
            if reset_discovery {
                (
                    ProviderTestStatus::Untested,
                    None,
                    BTreeMap::new(),
                    BTreeSet::new(),
                    None,
                )
            } else {
                let current = self
                    .providers
                    .get(&profile.provider_id)
                    .expect("an unchanged profile already exists");
                (
                    current.test_status.clone(),
                    current.connection_evidence.clone(),
                    current.models.clone(),
                    current.disabled_model_ids.clone(),
                    current.selected_model_id.clone(),
                )
            };
        self.providers.insert(
            profile.provider_id.clone(),
            ProviderRecord {
                profile,
                test_status,
                connection_evidence,
                models,
                disabled_model_ids,
                selected_model_id,
                generation,
            },
        );
        self.generation = generation;
        Ok(generation)
    }

    pub fn test_provider(
        &mut self,
        provider_id: &str,
        expected_generation: u64,
        attempted_at_ms: u64,
        probe: &mut impl ProviderProbe,
    ) -> Result<ProviderTestReport, ProviderError> {
        self.expect_generation(expected_generation)?;
        if attempted_at_ms == 0 {
            return Err(ProviderError::InvalidTimestamp);
        }
        let profile = self
            .providers
            .get(provider_id)
            .ok_or(ProviderError::NotFound)?
            .profile
            .clone();
        if !profile.enabled {
            return Err(ProviderError::Disabled);
        }

        let result = probe.test_and_discover(&profile);
        let generation = self.next_generation()?;
        let record = self
            .providers
            .get_mut(provider_id)
            .ok_or(ProviderError::NotFound)?;
        match result {
            Err(failure) => {
                record.test_status = ProviderTestStatus::Failed {
                    checked_at_ms: attempted_at_ms,
                    code: failure.code(),
                };
                record.connection_evidence = None;
                for route in record.models.values_mut() {
                    route.availability = RouteAvailability::Unknown;
                }
                record.selected_model_id = None;
                record.generation = generation;
                self.generation = generation;
                Ok(ProviderTestReport {
                    generation,
                    status: record.test_status.clone(),
                    discovered_models: record.models.len(),
                    usable_models: 0,
                    selected_model_id: None,
                })
            }
            Ok(discovery) => {
                discovery.validate(&profile)?;
                if discovery.checked_at_ms != attempted_at_ms {
                    return Err(ProviderError::InvalidTimestamp);
                }
                let connection_evidence = discovery
                    .connection_evidence
                    .ok_or(ProviderError::MissingConnectionProof)?;
                let models = discovery
                    .models
                    .into_iter()
                    .map(|route| (route.model_id.clone(), route))
                    .collect::<BTreeMap<_, _>>();
                let usable_models = models
                    .values()
                    .filter(|route| route.is_production_ready())
                    .count();
                record
                    .disabled_model_ids
                    .retain(|model_id| models.contains_key(model_id));
                let selected_model_id = choose_recommended(
                    &models,
                    &record.disabled_model_ids,
                    discovery.recommended_model_id.as_deref(),
                    record.selected_model_id.as_deref(),
                );
                record.test_status = if usable_models == 0 {
                    ProviderTestStatus::SucceededNoUsableModels {
                        checked_at_ms: attempted_at_ms,
                    }
                } else {
                    ProviderTestStatus::Succeeded {
                        checked_at_ms: attempted_at_ms,
                    }
                };
                record.connection_evidence = Some(connection_evidence);
                record.models = models;
                record.selected_model_id = selected_model_id.clone();
                record.generation = generation;
                self.generation = generation;
                Ok(ProviderTestReport {
                    generation,
                    status: record.test_status.clone(),
                    discovered_models: record.models.len(),
                    usable_models,
                    selected_model_id,
                })
            }
        }
    }

    pub fn select_model(
        &mut self,
        provider_id: &str,
        model_id: &str,
        expected_generation: u64,
    ) -> Result<u64, ProviderError> {
        self.expect_generation(expected_generation)?;
        let generation = self.next_generation()?;
        let record = self
            .providers
            .get_mut(provider_id)
            .ok_or(ProviderError::NotFound)?;
        if !record
            .models
            .get(model_id)
            .is_some_and(ModelRoute::is_production_ready)
            || record.disabled_model_ids.contains(model_id)
        {
            return Err(ProviderError::ModelUnavailable);
        }
        record.selected_model_id = Some(model_id.into());
        record.generation = generation;
        self.generation = generation;
        Ok(generation)
    }

    pub fn set_models_enabled(
        &mut self,
        provider_id: &str,
        model_ids: &[String],
        enabled: bool,
        expected_generation: u64,
    ) -> Result<u64, ProviderError> {
        self.expect_generation(expected_generation)?;
        if model_ids.is_empty() || model_ids.len() > MAX_MODELS_PER_PROVIDER {
            return Err(ProviderError::InvalidProfile);
        }
        let generation = self.next_generation()?;
        let record = self
            .providers
            .get_mut(provider_id)
            .ok_or(ProviderError::NotFound)?;
        let unique = model_ids.iter().collect::<BTreeSet<_>>();
        if unique.len() != model_ids.len()
            || model_ids.iter().any(|model_id| {
                validate_model_id(model_id).is_err() || !record.models.contains_key(model_id)
            })
        {
            return Err(ProviderError::ModelUnavailable);
        }
        for model_id in model_ids {
            if enabled {
                record.disabled_model_ids.remove(model_id);
            } else {
                record.disabled_model_ids.insert(model_id.clone());
            }
        }
        if record
            .selected_model_id
            .as_ref()
            .is_some_and(|selected| record.disabled_model_ids.contains(selected))
        {
            record.selected_model_id =
                choose_recommended(&record.models, &record.disabled_model_ids, None, None);
        } else if record.selected_model_id.is_none() {
            record.selected_model_id =
                choose_recommended(&record.models, &record.disabled_model_ids, None, None);
        }
        record.generation = generation;
        self.generation = generation;
        Ok(generation)
    }

    pub fn complete_onboarding(
        &mut self,
        expected_generation: u64,
        completed_at_ms: u64,
    ) -> Result<u64, ProviderError> {
        self.expect_generation(expected_generation)?;
        if completed_at_ms == 0 || !self.snapshot().onboarding_ready_at(completed_at_ms) {
            return Err(ProviderError::OnboardingNotReady);
        }
        let generation = self.next_generation()?;
        self.onboarding_completed_at_ms = Some(completed_at_ms);
        self.generation = generation;
        Ok(generation)
    }

    pub fn delete_provider(
        &mut self,
        provider_id: &str,
        expected_generation: u64,
    ) -> Result<u64, ProviderError> {
        self.expect_generation(expected_generation)?;
        if !self.providers.contains_key(provider_id) {
            return Err(ProviderError::NotFound);
        }
        let generation = self.next_generation()?;
        self.providers.remove(provider_id);
        if self.providers.is_empty() {
            self.onboarding_completed_at_ms = None;
        }
        self.generation = generation;
        Ok(generation)
    }

    pub fn invalidate_credential_reference(
        &mut self,
        credential_reference: &CredentialReference,
    ) -> Result<Option<u64>, ProviderError> {
        let affected = self.providers.values().any(|record| {
            record.profile.credential_reference.as_ref() == Some(credential_reference)
        });
        if !affected {
            return Ok(None);
        }
        let generation = self.next_generation()?;
        for record in self.providers.values_mut().filter(|record| {
            record.profile.credential_reference.as_ref() == Some(credential_reference)
        }) {
            record.test_status = ProviderTestStatus::Untested;
            record.connection_evidence = None;
            record.selected_model_id = None;
            for route in record.models.values_mut() {
                route.availability = RouteAvailability::Unknown;
            }
            record.generation = generation;
        }
        self.generation = generation;
        Ok(Some(generation))
    }

    fn expect_generation(&self, expected: u64) -> Result<(), ProviderError> {
        if expected != self.generation {
            return Err(ProviderError::StaleGeneration {
                expected,
                current: self.generation,
            });
        }
        Ok(())
    }

    fn next_generation(&self) -> Result<u64, ProviderError> {
        self.generation
            .checked_add(1)
            .ok_or(ProviderError::GenerationExhausted)
    }
}

fn choose_recommended(
    models: &BTreeMap<String, ModelRoute>,
    disabled_model_ids: &BTreeSet<String>,
    discovered_recommendation: Option<&str>,
    prior_selection: Option<&str>,
) -> Option<String> {
    for candidate in [discovered_recommendation, prior_selection]
        .into_iter()
        .flatten()
    {
        if models
            .get(candidate)
            .is_some_and(ModelRoute::is_production_ready)
            && !disabled_model_ids.contains(candidate)
        {
            return Some(candidate.into());
        }
    }
    models
        .values()
        .filter(|route| {
            route.is_production_ready() && !disabled_model_ids.contains(&route.model_id)
        })
        .min_by_key(|route| (route.recommendation_rank, route.model_id.as_str()))
        .map(|route| route.model_id.clone())
}

fn provider_connection_request(
    profile: &ProviderProfile,
    attempted_at_ms: u64,
) -> Result<ProviderConnectionRequest, ProviderProbeFailure> {
    if attempted_at_ms == 0 {
        return Err(ProviderProbeFailure::Incompatible);
    }
    let endpoint_sha256 = sha256_json(&profile.endpoint)?;
    let profile_binding_sha256 = provider_profile_binding(profile, attempted_at_ms)?;
    Ok(ProviderConnectionRequest {
        provider_id: profile.provider_id.clone(),
        provider_kind: profile.kind,
        endpoint_id: profile.endpoint.endpoint_id.clone(),
        base_url: profile.endpoint.base_url.clone(),
        api_kind: profile.endpoint.api_kind.clone(),
        native_provider_id: profile.native_provider_id().into(),
        authentication: profile.authentication.clone(),
        headers: profile.headers.clone(),
        attempted_at_ms,
        endpoint_sha256,
        operation: format!("provider-connection-test:{profile_binding_sha256}"),
        profile_binding_sha256,
    })
}

fn validate_connection_request(
    request: &ProviderConnectionRequest,
) -> Result<(), ProviderProbeFailure> {
    if validate_id(&request.provider_id).is_err()
        || request.authentication.validate().is_err()
        || validate_literal_headers(&request.headers).is_err()
        || validate_id(&request.endpoint_id).is_err()
        || validate_id(&request.api_kind).is_err()
        || validate_id(&request.native_provider_id).is_err()
        || validate_provider_endpoint(request.provider_kind, &request.base_url).is_err()
        || request.attempted_at_ms == 0
        || !valid_sha256(&request.endpoint_sha256)
        || !valid_sha256(&request.profile_binding_sha256)
        || request.operation
            != format!(
                "provider-connection-test:{}",
                request.profile_binding_sha256
            )
    {
        return Err(ProviderProbeFailure::Incompatible);
    }
    Ok(())
}

fn write_curl_prefix(
    writer: &mut dyn Write,
    request: &ProviderConnectionRequest,
) -> io::Result<()> {
    let base_url = request.base_url.trim_end_matches('/');
    let suffix = if request.api_kind == "anthropic" {
        "v1/models"
    } else {
        "models"
    };
    let url = if base_url.ends_with(suffix) {
        base_url.to_string()
    } else {
        format!("{base_url}/{suffix}")
    };
    if url.len() > MAX_ENDPOINT_BYTES + 16
        || url
            .bytes()
            .any(|byte| matches!(byte, b'"' | b'\\' | b'\r' | b'\n'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid provider endpoint",
        ));
    }
    writer.write_all(b"request = \"GET\"\nurl = \"")?;
    writer.write_all(url.as_bytes())?;
    writer.write_all(b"\"\nheader = \"Accept: application/json\"\n")?;
    for (name, value) in &request.headers {
        writer.write_all(b"header = \"")?;
        writer.write_all(name.as_bytes())?;
        writer.write_all(b": ")?;
        writer.write_all(value.as_bytes())?;
        writer.write_all(b"\"\n")?;
    }
    Ok(())
}

fn write_curl_credential_prefix(
    writer: &mut dyn Write,
    authentication: &ProviderAuthentication,
) -> io::Result<()> {
    writer.write_all(b"header = \"")?;
    match authentication {
        ProviderAuthentication::Bearer => writer.write_all(b"Authorization: Bearer "),
        ProviderAuthentication::ApiKeyHeader { header_name } => {
            writer.write_all(header_name.as_bytes())?;
            writer.write_all(b": ")
        }
        ProviderAuthentication::None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "provider authentication does not use a credential",
        )),
    }
}

fn split_curl_response(bytes: &[u8]) -> Result<(&[u8], u16), ProviderProbeFailure> {
    if bytes.len() < 4 || bytes[bytes.len() - 4] != b'\n' {
        return Err(ProviderProbeFailure::Incompatible);
    }
    let status_bytes = &bytes[bytes.len() - 3..];
    if !status_bytes.iter().all(u8::is_ascii_digit) {
        return Err(ProviderProbeFailure::Incompatible);
    }
    let status = std::str::from_utf8(status_bytes)
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(ProviderProbeFailure::Incompatible)?;
    Ok((&bytes[..bytes.len() - 4], status))
}

fn parse_provider_catalog(
    bytes: &[u8],
    api_kind: &str,
) -> Result<ProviderCatalog, ProviderProbeFailure> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| ProviderProbeFailure::Incompatible)?;
    let entries = value
        .get(if api_kind == "google" {
            "models"
        } else {
            "data"
        })
        .and_then(serde_json::Value::as_array)
        .ok_or(ProviderProbeFailure::Incompatible)?;
    if entries.len() > MAX_MODELS_PER_PROVIDER {
        return Err(ProviderProbeFailure::Incompatible);
    }
    let mut identifiers = BTreeSet::new();
    let mut models = Vec::with_capacity(entries.len());
    for entry in entries {
        let raw_model_id = entry
            .get(if api_kind == "google" { "name" } else { "id" })
            .and_then(serde_json::Value::as_str)
            .ok_or(ProviderProbeFailure::Incompatible)?;
        let model_id = if api_kind == "google" {
            raw_model_id.strip_prefix("models/").unwrap_or(raw_model_id)
        } else {
            raw_model_id
        };
        if validate_model_id(model_id).is_err() {
            continue;
        }
        if !identifiers.insert(model_id.to_owned()) {
            return Err(ProviderProbeFailure::Incompatible);
        }
        let display_name = entry
            .get(if api_kind == "google" {
                "displayName"
            } else {
                "name"
            })
            .and_then(serde_json::Value::as_str)
            .filter(|value| bounded_label(value))
            .unwrap_or(model_id)
            .to_owned();
        let architecture = entry.get("architecture");
        let mut input_modalities = catalog_string_set(
            architecture
                .and_then(|value| value.get("input_modalities"))
                .or_else(|| entry.get("input_modalities")),
        )?;
        let mut output_modalities = catalog_string_set(
            architecture
                .and_then(|value| value.get("output_modalities"))
                .or_else(|| entry.get("output_modalities")),
        )?;
        let supported_parameters = catalog_string_set(entry.get("supported_parameters"))?;
        let is_official_text_model = (api_kind == "anthropic" && model_id.starts_with("claude-"))
            || (api_kind == "google"
                && catalog_string_set(entry.get("supportedGenerationMethods"))?
                    .contains("generatecontent"));
        if is_official_text_model {
            input_modalities.insert("text".into());
            output_modalities.insert("text".into());
        }
        let context_tokens = bounded_catalog_limit(entry.get("context_length"))?;
        let output_tokens = bounded_catalog_limit(
            entry
                .get("top_provider")
                .and_then(|value| value.get("max_completion_tokens"))
                .or_else(|| entry.get("max_output_tokens")),
        )?;
        models.push(ProviderCatalogModel {
            model_id: model_id.into(),
            display_name,
            input_modalities,
            output_modalities,
            supported_parameters,
            context_tokens,
            output_tokens,
        });
    }
    Ok(ProviderCatalog { models })
}

fn catalog_string_set(
    value: Option<&serde_json::Value>,
) -> Result<BTreeSet<String>, ProviderProbeFailure> {
    let Some(value) = value else {
        return Ok(BTreeSet::new());
    };
    let entries = value.as_array().ok_or(ProviderProbeFailure::Incompatible)?;
    if entries.len() > 64 {
        return Err(ProviderProbeFailure::Incompatible);
    }
    entries
        .iter()
        .map(|value| {
            let value = value
                .as_str()
                .filter(|value| !value.is_empty() && value.len() <= MAX_ID_BYTES)
                .ok_or(ProviderProbeFailure::Incompatible)?;
            Ok(value.to_ascii_lowercase())
        })
        .collect()
}

fn bounded_catalog_limit(
    value: Option<&serde_json::Value>,
) -> Result<Option<u64>, ProviderProbeFailure> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let limit = value.as_u64().ok_or(ProviderProbeFailure::Incompatible)?;
    if limit == 0 || limit > 10_000_000_000 {
        return Err(ProviderProbeFailure::Incompatible);
    }
    Ok(Some(limit))
}

fn sparse_openai_chat_model(model_id: &str) -> bool {
    let model_id = model_id.to_ascii_lowercase();
    model_id.starts_with("gpt-")
        || model_id.starts_with("chatgpt-")
        || model_id.starts_with("codex-")
        || ["o1", "o3", "o4"]
            .iter()
            .any(|prefix| model_id == *prefix || model_id.starts_with(&format!("{prefix}-")))
}

fn direct_provider_route(
    profile: &ProviderProfile,
    model: ProviderCatalogModel,
    recommendation_rank: u32,
    checked_at_ms: u64,
    response_sha256: &str,
    session_configuration_sha256: &str,
) -> Result<ModelRoute, ProviderProbeFailure> {
    let expires_at_ms = checked_at_ms
        .checked_add(PROVIDER_TEST_FRESHNESS_MS)
        .ok_or(ProviderProbeFailure::Internal)?;
    let sparse_openai_chat = profile.kind == ProviderKind::OpenAi
        && model.input_modalities.is_empty()
        && model.output_modalities.is_empty()
        && sparse_openai_chat_model(&model.model_id);
    let modality_state = |modalities: &BTreeSet<String>, modality: &str| {
        if modalities.is_empty() {
            if sparse_openai_chat && modality == "text" {
                CapabilityState::Supported
            } else {
                CapabilityState::Unknown
            }
        } else if modalities.contains(modality) {
            CapabilityState::Supported
        } else {
            CapabilityState::Unsupported
        }
    };
    let parameter_state = |names: &[&str]| {
        if model.supported_parameters.is_empty() {
            CapabilityState::Unknown
        } else if names
            .iter()
            .any(|name| model.supported_parameters.contains(*name))
        {
            CapabilityState::Supported
        } else {
            CapabilityState::Unsupported
        }
    };
    // OpenAI's official catalog omits modalities. For its exact fixed endpoint,
    // narrowly recognized Chat model families carry text input/output; other
    // sparse OpenAI and compatible catalogs remain Unknown and fail closed.
    let chat_compatible = modality_state(&model.input_modalities, "text")
        == CapabilityState::Supported
        && modality_state(&model.output_modalities, "text") == CapabilityState::Supported;
    let source = if sparse_openai_chat {
        "c4os.openai-sparse-chat-catalog.v1"
    } else {
        "c4os.openai-compatible-catalog.v1"
    };
    let evidence = |state: CapabilityState, absent_reason: &str| CapabilityEvidence {
        state,
        layer: CapabilityLayer::AdapterNormalized,
        source: source.into(),
        checked_at_ms,
        expires_at_ms: Some(expires_at_ms),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: (state != CapabilityState::Supported).then(|| absent_reason.into()),
    };
    let feature_claims = [
        (
            CapabilityKey::InputText,
            modality_state(&model.input_modalities, "text"),
            "catalog does not declare chat text input",
        ),
        (
            CapabilityKey::InputImage,
            modality_state(&model.input_modalities, "image"),
            "catalog does not declare image input",
        ),
        (
            CapabilityKey::InputAudio,
            modality_state(&model.input_modalities, "audio"),
            "catalog does not declare audio input",
        ),
        (
            CapabilityKey::InputVideo,
            modality_state(&model.input_modalities, "video"),
            "catalog does not declare video input",
        ),
        (
            CapabilityKey::InputPdf,
            modality_state(&model.input_modalities, "pdf"),
            "catalog does not declare PDF input",
        ),
        (
            CapabilityKey::OutputText,
            modality_state(&model.output_modalities, "text"),
            "catalog does not declare chat text output",
        ),
        (
            CapabilityKey::OutputImage,
            modality_state(&model.output_modalities, "image"),
            "catalog does not declare image output",
        ),
        (
            CapabilityKey::Streaming,
            CapabilityState::Unknown,
            "provider catalog does not declare streaming support",
        ),
        (
            CapabilityKey::Reasoning,
            parameter_state(&["reasoning", "reasoning_effort"]),
            "catalog does not declare reasoning controls",
        ),
        (
            CapabilityKey::ToolCalling,
            parameter_state(&["tools", "tool_choice"]),
            "catalog does not declare tool calling",
        ),
        (
            CapabilityKey::StructuredJson,
            parameter_state(&["response_format", "structured_outputs"]),
            "catalog does not declare structured JSON",
        ),
    ];
    let features = feature_claims
        .into_iter()
        .map(|(key, state, reason)| (key, evidence(state, reason)))
        .collect::<BTreeMap<_, _>>();
    let mut numeric_limits = BTreeMap::new();
    for (key, maximum) in [
        (NumericCapabilityKey::ContextTokens, model.context_tokens),
        (NumericCapabilityKey::OutputTokens, model.output_tokens),
    ] {
        if let Some(maximum) = maximum {
            numeric_limits.insert(
                key,
                NumericCapabilityEvidence {
                    evidence: evidence(CapabilityState::Supported, ""),
                    maximum: Some(maximum),
                    confidence: LimitConfidence::Confirmed,
                },
            );
        }
    }
    let declaration_features = feature_claims
        .into_iter()
        .map(|(key, state, reason)| {
            (
                key,
                ProviderFeatureClaim {
                    state,
                    constraints: Vec::new(),
                    allowed_values: Vec::new(),
                    reason: (state != CapabilityState::Supported).then(|| reason.into()),
                },
            )
        })
        .collect();
    let mut declaration_limits = BTreeMap::new();
    for (key, maximum) in [
        (NumericCapabilityKey::ContextTokens, model.context_tokens),
        (NumericCapabilityKey::OutputTokens, model.output_tokens),
    ] {
        if let Some(maximum) = maximum {
            declaration_limits.insert(
                key,
                ProviderNumericClaim {
                    state: CapabilityState::Supported,
                    maximum: Some(maximum),
                    confidence: LimitConfidence::Confirmed,
                    reason: None,
                },
            );
        }
    }
    let lifecycle = if chat_compatible {
        ModelLifecycle::Active
    } else {
        ModelLifecycle::Unavailable
    };
    let route = ModelRoute {
        model_id: model.model_id.clone(),
        display_name: model.display_name,
        recommendation_rank,
        availability: if chat_compatible {
            RouteAvailability::Available
        } else {
            RouteAvailability::Unknown
        },
        checked_at_ms,
        capabilities: CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::AdapterNormalized,
            route: RouteIdentity {
                provider_id: profile.provider_id.clone(),
                endpoint_id: profile.endpoint.endpoint_id.clone(),
                provider_model_id: format!(
                    "{}/{}",
                    profile.opencode_native_provider_id(),
                    model.model_id
                ),
                model_revision: "provider-catalog-v1".into(),
                adapter_kind: "opencode".into(),
                adapter_version: "1.0.0".into(),
                runtime_kind: "opencode".into(),
                native_runtime_version: "1.18.3".into(),
                session_configuration_sha256: session_configuration_sha256.into(),
            },
            lifecycle,
            features,
            numeric_limits,
            raw_evidence_sha256: response_sha256.into(),
        },
        provider_declaration: Some(ProviderModelDeclaration {
            schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
            provider_model_id: model.model_id,
            model_revision: "provider-catalog-v1".into(),
            lifecycle,
            features: declaration_features,
            numeric_limits: declaration_limits,
            raw_catalog_sha256: response_sha256.into(),
            declared_at_ms: checked_at_ms,
            expires_at_ms,
        }),
    };
    route
        .validate_for(profile)
        .map_err(|_| ProviderProbeFailure::Incompatible)?;
    Ok(route)
}

fn provider_profile_binding(
    profile: &ProviderProfile,
    attempted_at_ms: u64,
) -> Result<String, ProviderProbeFailure> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Binding<'a> {
        schema_version: u16,
        provider_id: &'a str,
        provider_kind: ProviderKind,
        endpoint: &'a ProviderEndpoint,
        authentication: &'a ProviderAuthentication,
        credential_reference: Option<&'a str>,
        headers: &'a BTreeMap<String, String>,
        attempted_at_ms: u64,
    }

    sha256_json(&Binding {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: &profile.provider_id,
        provider_kind: profile.kind,
        endpoint: &profile.endpoint,
        authentication: &profile.authentication,
        credential_reference: profile
            .credential_reference
            .as_ref()
            .map(CredentialReference::as_str),
        headers: &profile.headers,
        attempted_at_ms,
    })
}

fn credential_binding(profile: &ProviderProfile) -> &str {
    profile
        .credential_reference
        .as_ref()
        .map_or("none", CredentialReference::as_str)
}

fn normalized_opencode_route(
    profile: &ProviderProfile,
    model: NormalizedModel,
    recommendation_rank: u32,
    checked_at_ms: u64,
    session_configuration_sha256: &str,
) -> Result<ModelRoute, ProviderProbeFailure> {
    let expires_at_ms = checked_at_ms
        .checked_add(PROVIDER_TEST_FRESHNESS_MS)
        .ok_or(ProviderProbeFailure::Internal)?;
    let source = "opencode.1.18.3";
    let mut features = BTreeMap::new();
    let supported = |state, reason: Option<&str>| CapabilityEvidence {
        state,
        layer: CapabilityLayer::AdapterNormalized,
        source: source.into(),
        checked_at_ms,
        expires_at_ms: Some(expires_at_ms),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: reason.map(str::to_owned),
    };
    for (key, present) in [
        (
            CapabilityKey::InputText,
            model.input_modalities.contains("text"),
        ),
        (
            CapabilityKey::InputImage,
            model.input_modalities.contains("image"),
        ),
        (
            CapabilityKey::InputAudio,
            model.input_modalities.contains("audio"),
        ),
        (
            CapabilityKey::InputVideo,
            model.input_modalities.contains("video"),
        ),
        (
            CapabilityKey::InputPdf,
            model.input_modalities.contains("pdf"),
        ),
        (
            CapabilityKey::OutputText,
            model.output_modalities.contains("text"),
        ),
        (
            CapabilityKey::OutputImage,
            model.output_modalities.contains("image"),
        ),
        (CapabilityKey::Reasoning, model.reasoning),
        (CapabilityKey::ToolCalling, model.tool_calling),
    ] {
        features.insert(
            key,
            if present {
                supported(CapabilityState::Supported, None)
            } else {
                supported(
                    CapabilityState::Unsupported,
                    Some("OpenCode model inventory reports this capability unavailable"),
                )
            },
        );
    }
    let mut numeric_limits = BTreeMap::new();
    for (key, maximum) in [
        (NumericCapabilityKey::ContextTokens, model.context_limit),
        (NumericCapabilityKey::OutputTokens, model.output_limit),
    ] {
        numeric_limits.insert(
            key,
            NumericCapabilityEvidence {
                evidence: supported(CapabilityState::Supported, None),
                maximum: Some(maximum),
                confidence: LimitConfidence::Confirmed,
            },
        );
    }
    let provider_declaration = provider_model_declaration(&model, checked_at_ms)?;
    let lifecycle = match model.lifecycle {
        crate::runtime::opencode::ModelLifecycle::Active => ModelLifecycle::Active,
        crate::runtime::opencode::ModelLifecycle::Alpha
        | crate::runtime::opencode::ModelLifecycle::Beta => ModelLifecycle::Preview,
        crate::runtime::opencode::ModelLifecycle::Deprecated => ModelLifecycle::Deprecated,
        crate::runtime::opencode::ModelLifecycle::Unknown => ModelLifecycle::Unavailable,
    };
    let availability = match lifecycle {
        ModelLifecycle::Active => RouteAvailability::Available,
        ModelLifecycle::Preview | ModelLifecycle::Deprecated => RouteAvailability::Degraded,
        ModelLifecycle::Unavailable => RouteAvailability::Unknown,
    };
    let raw_evidence_sha256 = sha256_json(&(
        "c4os.opencode-adapter-normalized.v1",
        &model,
        session_configuration_sha256,
        profile.opencode_native_provider_id(),
    ))?;
    let route = ModelRoute {
        model_id: model.model_id.clone(),
        display_name: model.display_name,
        recommendation_rank,
        availability,
        checked_at_ms,
        capabilities: CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::AdapterNormalized,
            route: RouteIdentity {
                provider_id: profile.provider_id.clone(),
                endpoint_id: profile.endpoint.endpoint_id.clone(),
                provider_model_id: format!(
                    "{}/{}",
                    profile.opencode_native_provider_id(),
                    model.model_id
                ),
                model_revision: "opencode-catalog-1.18.3".into(),
                adapter_kind: "opencode".into(),
                adapter_version: "1.0.0".into(),
                runtime_kind: "opencode".into(),
                native_runtime_version: "1.18.3".into(),
                session_configuration_sha256: session_configuration_sha256.into(),
            },
            lifecycle,
            features,
            numeric_limits,
            raw_evidence_sha256,
        },
        provider_declaration: Some(provider_declaration),
    };
    route
        .validate_for(profile)
        .map_err(|_| ProviderProbeFailure::Incompatible)?;
    Ok(route)
}

fn provider_model_declaration(
    model: &NormalizedModel,
    declared_at_ms: u64,
) -> Result<ProviderModelDeclaration, ProviderProbeFailure> {
    let expires_at_ms = declared_at_ms
        .checked_add(PROVIDER_TEST_FRESHNESS_MS)
        .ok_or(ProviderProbeFailure::Internal)?;
    let raw_catalog_sha256 = sha256_json(model)?;
    let lifecycle = match model.lifecycle {
        crate::runtime::opencode::ModelLifecycle::Active => ModelLifecycle::Active,
        crate::runtime::opencode::ModelLifecycle::Alpha
        | crate::runtime::opencode::ModelLifecycle::Beta => ModelLifecycle::Preview,
        crate::runtime::opencode::ModelLifecycle::Deprecated => ModelLifecycle::Deprecated,
        crate::runtime::opencode::ModelLifecycle::Unknown => ModelLifecycle::Unavailable,
    };
    let feature = |present: bool, absent_reason: &str| ProviderFeatureClaim {
        state: if present {
            CapabilityState::Supported
        } else {
            CapabilityState::Unsupported
        },
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: (!present).then(|| absent_reason.into()),
    };
    let features = BTreeMap::from([
        (
            CapabilityKey::InputText,
            feature(
                model.input_modalities.contains("text"),
                "provider catalog does not declare text input",
            ),
        ),
        (
            CapabilityKey::InputImage,
            feature(
                model.input_modalities.contains("image"),
                "provider catalog does not declare image input",
            ),
        ),
        (
            CapabilityKey::InputAudio,
            feature(
                model.input_modalities.contains("audio"),
                "provider catalog does not declare audio input",
            ),
        ),
        (
            CapabilityKey::InputVideo,
            feature(
                model.input_modalities.contains("video"),
                "provider catalog does not declare video input",
            ),
        ),
        (
            CapabilityKey::InputPdf,
            feature(
                model.input_modalities.contains("pdf"),
                "provider catalog does not declare PDF input",
            ),
        ),
        (
            CapabilityKey::OutputText,
            feature(
                model.output_modalities.contains("text"),
                "provider catalog does not declare text output",
            ),
        ),
        (
            CapabilityKey::OutputImage,
            feature(
                model.output_modalities.contains("image"),
                "provider catalog does not declare image output",
            ),
        ),
        (
            CapabilityKey::Reasoning,
            feature(
                model.reasoning,
                "provider catalog does not declare reasoning support",
            ),
        ),
        (
            CapabilityKey::ToolCalling,
            feature(
                model.tool_calling,
                "provider catalog does not declare tool-calling support",
            ),
        ),
    ]);
    let numeric_limits = BTreeMap::from([
        (
            NumericCapabilityKey::ContextTokens,
            ProviderNumericClaim {
                state: CapabilityState::Supported,
                maximum: Some(model.context_limit),
                confidence: LimitConfidence::Confirmed,
                reason: None,
            },
        ),
        (
            NumericCapabilityKey::OutputTokens,
            ProviderNumericClaim {
                state: CapabilityState::Supported,
                maximum: Some(model.output_limit),
                confidence: LimitConfidence::Confirmed,
                reason: None,
            },
        ),
    ]);
    Ok(ProviderModelDeclaration {
        schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
        provider_model_id: model.model_id.clone(),
        model_revision: "opencode-catalog-1.18.3".into(),
        lifecycle,
        features,
        numeric_limits,
        raw_catalog_sha256,
        declared_at_ms,
        expires_at_ms,
    })
}

fn map_opencode_probe_failure(
    error: crate::runtime::opencode::AdapterError,
) -> ProviderProbeFailure {
    use crate::runtime::opencode::AdapterError;
    match error {
        AdapterError::TransportFailed(
            crate::runtime::opencode::TransportFailureCode::AuthenticationRejected,
        ) => ProviderProbeFailure::Authentication,
        AdapterError::TransportFailed(
            crate::runtime::opencode::TransportFailureCode::Unavailable
            | crate::runtime::opencode::TransportFailureCode::Timeout,
        ) => ProviderProbeFailure::Network,
        AdapterError::IncompatibleVersion => ProviderProbeFailure::Incompatible,
        _ => ProviderProbeFailure::Internal,
    }
}

fn map_credential_probe_failure(error: CredentialVaultError) -> ProviderProbeFailure {
    match error {
        CredentialVaultError::CredentialNotFound
        | CredentialVaultError::LeaseExpired
        | CredentialVaultError::LeaseRevoked => ProviderProbeFailure::Authentication,
        CredentialVaultError::OperationChannel => ProviderProbeFailure::Network,
        _ => ProviderProbeFailure::Internal,
    }
}

fn sha256_json(value: &impl Serialize) -> Result<String, ProviderProbeFailure> {
    let bytes = serde_json::to_vec(value).map_err(|_| ProviderProbeFailure::Internal)?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderTestReport {
    pub generation: u64,
    pub status: ProviderTestStatus,
    pub discovered_models: usize,
    pub usable_models: usize,
    pub selected_model_id: Option<String>,
}

fn validate_id(value: &str) -> Result<(), ProviderError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@' | b':')
        })
    {
        return Err(ProviderError::InvalidProfile);
    }
    Ok(())
}

fn validate_model_id(value: &str) -> Result<(), ProviderError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || value.starts_with('/')
        || value.ends_with('/')
        || value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@' | b'/' | b':')
        })
    {
        return Err(ProviderError::InvalidProfile);
    }
    Ok(())
}

fn bounded_label(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= MAX_LABEL_BYTES
        && !value.chars().any(char::is_control)
}

fn valid_claim_text(
    constraints: &[String],
    allowed_values: &[String],
    reason: Option<&str>,
) -> bool {
    constraints.len() <= 64
        && allowed_values.len() <= 64
        && constraints.iter().chain(allowed_values).all(|value| {
            !value.trim().is_empty()
                && value.len() <= MAX_CLAIM_TEXT_BYTES
                && !value.chars().any(char::is_control)
        })
        && reason.is_none_or(|value| {
            !value.trim().is_empty()
                && value.len() <= MAX_CLAIM_TEXT_BYTES
                && !value.chars().any(char::is_control)
        })
}

fn valid_credential_reference(reference: &CredentialReference) -> bool {
    let value = reference.as_str();
    value
        .strip_prefix("credential:")
        .is_some_and(|id| id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn valid_header_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_HEADER_NAME_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

fn validate_literal_headers(headers: &BTreeMap<String, String>) -> Result<(), ProviderError> {
    if headers.len() > MAX_PROVIDER_HEADERS {
        return Err(ProviderError::InvalidProfile);
    }
    const SECRET_HEADERS: [&str; 7] = [
        "authorization",
        "proxy-authorization",
        "cookie",
        "set-cookie",
        "x-api-key",
        "x-goog-api-key",
        "api-key",
    ];
    let mut normalized = BTreeSet::new();
    for (name, value) in headers {
        let lower = name.to_ascii_lowercase();
        if !valid_header_name(name)
            || !normalized.insert(lower.clone())
            || SECRET_HEADERS.contains(&lower.as_str())
            || sensitive_literal_header_name(&lower)
            || value.is_empty()
            || value.len() > MAX_HEADER_VALUE_BYTES
            || sensitive_literal_header_value(value)
            || value
                .bytes()
                .any(|byte| byte.is_ascii_control() || matches!(byte, b'"' | b'\\'))
        {
            return Err(ProviderError::InvalidProfile);
        }
    }
    Ok(())
}

fn sensitive_literal_header_value(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("bearer ")
        || lower.starts_with("basic ")
        || lower.starts_with("sk-")
        || lower.starts_with("sk_")
        || lower.starts_with("ghp_")
        || lower.starts_with("xox")
        || lower.contains("secret")
        || lower.contains("api_key")
        || lower.contains("api-key")
    {
        return true;
    }
    trimmed.len() >= 16
        && !trimmed.starts_with("http://")
        && !trimmed.starts_with("https://")
        && trimmed.bytes().all(|byte| byte.is_ascii_graphic())
}

fn sensitive_literal_header_name(name: &str) -> bool {
    name.split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| {
            matches!(
                part,
                "auth"
                    | "authentication"
                    | "authorization"
                    | "credential"
                    | "credentials"
                    | "cookie"
                    | "key"
                    | "secret"
                    | "token"
            ) || part.ends_with("token")
                || part.ends_with("apikey")
                || part.ends_with("key")
                || part.ends_with("secret")
        })
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

pub(crate) fn validate_https_endpoint(value: &str) -> Result<(), ProviderError> {
    validate_endpoint_shape(value, "https://").map(|_| ())
}

fn validate_provider_endpoint(kind: ProviderKind, value: &str) -> Result<(), ProviderError> {
    let preset = match kind {
        ProviderKind::OpenAi => Some("https://api.openai.com/v1"),
        ProviderKind::Anthropic => Some("https://api.anthropic.com"),
        ProviderKind::Gemini => Some("https://generativelanguage.googleapis.com/v1beta"),
        ProviderKind::OpenRouter => Some("https://openrouter.ai/api/v1"),
        ProviderKind::HuggingFace => Some("https://router.huggingface.co/v1"),
        ProviderKind::Custom => None,
    };
    if let Some(expected) = preset {
        return (value == expected)
            .then_some(())
            .ok_or(ProviderError::InvalidEndpoint);
    }
    if validate_https_endpoint(value).is_ok() {
        return Ok(());
    }
    let authority = validate_endpoint_shape(value, "http://")?;
    let host = authority
        .strip_prefix('[')
        .and_then(|value| value.split_once(']'))
        .map(|(host, suffix)| (host, suffix))
        .unwrap_or_else(|| {
            authority
                .split_once(':')
                .map_or((authority, ""), |(host, port)| (host, port))
        });
    let loopback = match host {
        ("::1", suffix) => suffix.is_empty() || valid_port(suffix.strip_prefix(':').unwrap_or("")),
        ("127.0.0.1", "") => true,
        ("127.0.0.1", port) => valid_port(port),
        _ => false,
    };
    if !loopback {
        return Err(ProviderError::InvalidEndpoint);
    }
    Ok(())
}

fn validate_endpoint_shape<'a>(value: &'a str, scheme: &str) -> Result<&'a str, ProviderError> {
    if value.is_empty()
        || value.len() > MAX_ENDPOINT_BYTES
        || !value.starts_with(scheme)
        || value.contains('@')
        || value.contains('?')
        || value.contains('#')
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(ProviderError::InvalidEndpoint);
    }
    let authority = value
        .strip_prefix(scheme)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_default();
    if authority.is_empty() || authority.starts_with('.') || authority.ends_with('.') {
        return Err(ProviderError::InvalidEndpoint);
    }
    Ok(authority)
}

fn valid_port(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u16>().is_ok_and(|port| port != 0)
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ProviderError {
    #[error("provider profile is invalid")]
    InvalidProfile,
    #[error(
        "provider endpoint must be bounded HTTPS or an explicit Custom loopback HTTP URL without embedded credentials"
    )]
    InvalidEndpoint,
    #[error("provider discovery result is invalid")]
    InvalidDiscovery,
    #[error("provider discovery did not include runtime-neutral model declarations")]
    MissingModelDeclaration,
    #[error("provider discovery did not include a verified connection proof")]
    MissingConnectionProof,
    #[error("provider connection proof does not match the saved profile")]
    ConnectionProofMismatch,
    #[error("provider connectivity transport is invalid")]
    InvalidConnectivityTransport,
    #[error("provider snapshot is invalid")]
    InvalidSnapshot,
    #[error("provider or model was not found")]
    NotFound,
    #[error("provider is disabled")]
    Disabled,
    #[error("selected model route is unavailable")]
    ModelUnavailable,
    #[error("provider onboarding is not ready to complete")]
    OnboardingNotReady,
    #[error("provider profile capacity was reached")]
    CapacityExceeded,
    #[error("provider display names must be unique")]
    DuplicateDisplayName,
    #[error("provider timestamp is invalid")]
    InvalidTimestamp,
    #[error("provider generation is stale: expected {expected}, current {current}")]
    StaleGeneration { expected: u64, current: u64 },
    #[error("provider generation was exhausted")]
    GenerationExhausted,
}

#[cfg(test)]
mod endpoint_contract_tests {
    use super::*;

    fn profile(kind: ProviderKind, base_url: &str, api_kind: &str) -> ProviderProfile {
        ProviderProfile {
            schema_version: PROVIDER_SCHEMA_VERSION,
            provider_id: "provider-test".into(),
            kind,
            display_name: "Provider Test".into(),
            endpoint: ProviderEndpoint {
                endpoint_id: "endpoint-test".into(),
                base_url: base_url.into(),
                api_kind: api_kind.into(),
            },
            authentication: ProviderAuthentication::None,
            credential_reference: None,
            headers: BTreeMap::new(),
            enabled: true,
        }
    }

    #[test]
    fn preset_provider_endpoints_are_exact() {
        assert!(
            profile(
                ProviderKind::OpenRouter,
                "https://openrouter.ai/api/v1",
                "openai-compatible"
            )
            .validate()
            .is_ok()
        );
        assert_eq!(
            profile(
                ProviderKind::OpenRouter,
                "https://proxy.example/v1",
                "openai-compatible"
            )
            .validate(),
            Err(ProviderError::InvalidEndpoint)
        );
        assert_eq!(
            profile(
                ProviderKind::OpenRouter,
                "https://openrouter.ai/api/v1",
                "openai"
            )
            .validate(),
            Err(ProviderError::InvalidEndpoint)
        );
    }

    #[test]
    fn custom_provider_allows_https_or_explicit_loopback_http_only() {
        assert!(
            profile(
                ProviderKind::Custom,
                "https://proxy.example/v1",
                "openai-compatible"
            )
            .validate()
            .is_ok()
        );
        assert!(
            profile(
                ProviderKind::Custom,
                "http://127.0.0.1:8080/v1",
                "openai-compatible"
            )
            .validate()
            .is_ok()
        );
        assert_eq!(
            profile(
                ProviderKind::Custom,
                "http://proxy.example/v1",
                "openai-compatible"
            )
            .validate(),
            Err(ProviderError::InvalidEndpoint)
        );
    }

    #[test]
    fn legacy_profiles_restore_provider_specific_authentication_semantics() {
        for (kind, base_url, api_kind, expected_header) in [
            (
                ProviderKind::Anthropic,
                "https://api.anthropic.com",
                "anthropic",
                "x-api-key",
            ),
            (
                ProviderKind::Gemini,
                "https://generativelanguage.googleapis.com/v1beta",
                "google",
                "x-goog-api-key",
            ),
        ] {
            let candidate = profile(kind, base_url, api_kind);
            let mut value = serde_json::to_value(candidate).unwrap();
            value.as_object_mut().unwrap().remove("authentication");
            let restored: ProviderProfile = serde_json::from_value(value).unwrap();
            assert_eq!(
                restored.authentication,
                ProviderAuthentication::ApiKeyHeader {
                    header_name: expected_header.into(),
                }
            );
        }
    }

    #[test]
    fn google_catalog_normalizes_generate_content_models_as_text_routes() {
        let catalog = parse_provider_catalog(
            br#"{"models":[{"name":"models/gemini-2.5-flash","displayName":"Gemini 2.5 Flash","supportedGenerationMethods":["generateContent"]}]}"#,
            "google",
        )
        .unwrap();
        let model = catalog.models.first().unwrap();
        assert_eq!(model.model_id, "gemini-2.5-flash");
        assert!(model.input_modalities.contains("text"));
        assert!(model.output_modalities.contains("text"));
    }

    #[test]
    fn openrouter_catalog_keeps_valid_models_around_optional_limits_and_aliases() {
        let catalog = parse_provider_catalog(
            br#"{"data":[
                {
                    "id":"~google/gemini-flash-latest",
                    "name":"Google: Gemini Flash Latest",
                    "architecture":{"input_modalities":["text"],"output_modalities":["text"]},
                    "supported_parameters":["tools"],
                    "context_length":1048576,
                    "top_provider":{"max_completion_tokens":null}
                },
                {
                    "id":"google/gemini-2.5-flash-lite",
                    "name":"Google: Gemini 2.5 Flash Lite",
                    "architecture":{"input_modalities":["text","image","file","audio","video"],"output_modalities":["text"]},
                    "supported_parameters":["tools","tool_choice","response_format"],
                    "context_length":1048576,
                    "top_provider":{"max_completion_tokens":65535}
                },
                {
                    "id":"google/gemini-unknown-output",
                    "name":"Google: Gemini Unknown Output",
                    "architecture":{"input_modalities":["text"],"output_modalities":["text"]},
                    "supported_parameters":[],
                    "context_length":32768,
                    "top_provider":{"max_completion_tokens":null}
                }
            ]}"#,
            "openai-compatible",
        )
        .expect("bounded OpenRouter catalog");

        assert_eq!(catalog.models.len(), 2);
        let requested = catalog
            .models
            .iter()
            .find(|model| model.model_id == "google/gemini-2.5-flash-lite")
            .expect("requested OpenRouter model");
        assert_eq!(requested.context_tokens, Some(1_048_576));
        assert_eq!(requested.output_tokens, Some(65_535));
        assert!(requested.input_modalities.contains("text"));
        assert!(requested.output_modalities.contains("text"));
        assert!(
            catalog
                .models
                .iter()
                .find(|model| model.model_id == "google/gemini-unknown-output")
                .is_some_and(|model| model.output_tokens.is_none())
        );
    }

    #[test]
    fn namespaced_provider_and_endpoint_identifiers_are_valid() {
        let mut profile = profile(
            ProviderKind::Custom,
            "http://127.0.0.1:8080/v1",
            "openai-compatible",
        );
        profile.provider_id = "provider:task-00013-fixture".into();
        profile.endpoint.endpoint_id = "provider:task-00013-fixture:primary".into();

        assert!(profile.validate().is_ok());
        let route = direct_provider_route(
            &profile,
            ProviderCatalogModel {
                model_id: "openai/gpt-5-mini".into(),
                display_name: "GPT-5 Mini".into(),
                input_modalities: BTreeSet::from(["text".into()]),
                output_modalities: BTreeSet::from(["text".into()]),
                supported_parameters: BTreeSet::from(["tools".into()]),
                context_tokens: Some(400_000),
                output_tokens: Some(128_000),
            },
            0,
            1_784_770_000_000,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("namespaced provider route is valid");
        assert_eq!(route.capabilities.route.provider_id, profile.provider_id);
    }

    #[test]
    fn literal_headers_reject_credential_shaped_names() {
        let mut accepted = profile(
            ProviderKind::Custom,
            "https://proxy.example/v1",
            "openai-compatible",
        );
        accepted
            .headers
            .insert("HTTP-Referer".into(), "https://c4os.com".into());
        accepted.headers.insert("X-Title".into(), "C4OS".into());
        assert!(accepted.validate().is_ok());

        for name in ["X-Auth-Token", "X-Client-API-Key", "X-Shared-Secret"] {
            let mut rejected = accepted.clone();
            rejected.headers.insert(name.into(), "inline-value".into());
            assert_eq!(rejected.validate(), Err(ProviderError::InvalidProfile));
        }

        let mut credential_value = accepted;
        credential_value
            .headers
            .insert("X-Client".into(), "sk-example".into());
        assert_eq!(
            credential_value.validate(),
            Err(ProviderError::InvalidProfile)
        );
    }

    #[test]
    fn sparse_official_openai_chat_models_remain_onboarding_eligible() {
        let openai = profile(ProviderKind::OpenAi, "https://api.openai.com/v1", "openai");
        let model = ProviderCatalogModel {
            model_id: "gpt-5".into(),
            display_name: "GPT-5".into(),
            input_modalities: BTreeSet::new(),
            output_modalities: BTreeSet::new(),
            supported_parameters: BTreeSet::new(),
            context_tokens: None,
            output_tokens: None,
        };
        let route = direct_provider_route(
            &openai,
            model.clone(),
            0,
            1_784_770_000_000,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("official sparse OpenAI Chat route");
        assert!(route.is_production_ready());

        let compatible = profile(
            ProviderKind::Custom,
            "https://proxy.example/v1",
            "openai-compatible",
        );
        let route = direct_provider_route(
            &compatible,
            model,
            0,
            1_784_770_000_000,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("compatible sparse route remains inspectable");
        assert!(!route.is_production_ready());
    }

    #[test]
    fn provider_test_status_uses_camel_case_fields_at_the_renderer_boundary() {
        let value = serde_json::to_value(ProviderTestStatus::Failed {
            checked_at_ms: 1_784_770_000_000,
            code: ProviderFailureCode::Incompatible,
        })
        .expect("provider test status serializes");

        assert_eq!(value["checkedAtMs"], 1_784_770_000_000_u64);
        assert!(value.get("checked_at_ms").is_none());
    }
}
