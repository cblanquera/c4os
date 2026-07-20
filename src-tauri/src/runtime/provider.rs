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

use serde::{Deserialize, Serialize};
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
const MAX_DECLARATION_CLAIMS: usize = 256;
const MAX_CLAIM_TEXT_BYTES: usize = 2_048;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    OpenAi,
    Anthropic,
    Gemini,
    OpenRouter,
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
    fn validate(&self) -> Result<(), ProviderError> {
        validate_id(&self.endpoint_id)?;
        validate_id(&self.api_kind)?;
        validate_https_endpoint(&self.base_url)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderProfile {
    pub schema_version: u16,
    pub provider_id: String,
    pub kind: ProviderKind,
    pub display_name: String,
    pub endpoint: ProviderEndpoint,
    pub credential_reference: CredentialReference,
    pub enabled: bool,
}

impl ProviderProfile {
    pub fn validate(&self) -> Result<(), ProviderError> {
        if self.schema_version != PROVIDER_SCHEMA_VERSION
            || !bounded_label(&self.display_name)
            || !valid_credential_reference(&self.credential_reference)
        {
            return Err(ProviderError::InvalidProfile);
        }
        validate_id(&self.provider_id)?;
        self.endpoint.validate()
    }

    pub fn native_provider_id(&self) -> &str {
        match self.kind {
            ProviderKind::OpenAi => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Gemini => "google",
            ProviderKind::OpenRouter => "openrouter",
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
        validate_id(&self.model_id)?;
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
            credential_reference_sha256: sha256_bytes(
                profile.credential_reference.as_str().as_bytes(),
            ),
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
                != sha256_bytes(profile.credential_reference.as_str().as_bytes())
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
    pub endpoint_id: String,
    pub base_url: String,
    pub api_kind: String,
    pub native_provider_id: String,
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
}

/// Narrow provider network boundary. Implementations may deliver the lease to
/// an anonymous request-local credential channel; they must never serialize a
/// credential into the request, response, URL, diagnostics, argv, or inherited
/// environment.
pub trait ProviderConnectivity {
    fn test_connection(
        &mut self,
        request: &ProviderConnectionRequest,
        credential_lease: &OperationCredentialLease,
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
        credential_lease: &OperationCredentialLease,
    ) -> Result<ProviderConnectionObservation, ProviderProbeFailure> {
        validate_connection_request(request)?;
        if self.timeout_seconds == 0 || self.max_response_bytes < 4 {
            return Err(ProviderProbeFailure::Internal);
        }
        let maximum = self.max_response_bytes.to_string();
        let timeout = self.timeout_seconds.to_string();
        let mut child = Command::new(&self.executable)
            .args([
                "--disable",
                "--silent",
                "--show-error",
                "--proto",
                "=https",
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
        let lease = self
            .credential_leaser
            .lease_for_provider_test(&profile.credential_reference, &request.operation)?;
        if lease.operation() != request.operation {
            return Err(ProviderProbeFailure::Internal);
        }
        let observation = self.connectivity.test_connection(&request, &lease)?;
        // A success that did not consume the exact operation lease did not
        // authenticate the saved profile and must fail closed.
        if lease.is_valid()
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
#[serde(rename_all = "camelCase", tag = "state")]
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
    pub selected_model_id: Option<String>,
    pub generation: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderSnapshot {
    pub generation: u64,
    pub providers: Vec<ProviderRecord>,
}

impl ProviderSnapshot {
    pub fn onboarding_ready_at(&self, now_ms: u64) -> bool {
        self.providers.iter().any(|provider| {
            provider.profile.enabled
                && matches!(
                    provider.test_status,
                    ProviderTestStatus::Succeeded { checked_at_ms }
                        if checked_at_ms <= now_ms
                            && now_ms.saturating_sub(checked_at_ms)
                                <= PROVIDER_TEST_FRESHNESS_MS
                )
                && provider
                    .connection_evidence
                    .as_ref()
                    .is_some_and(|evidence| {
                        evidence.tested_at_ms <= now_ms
                            && now_ms.saturating_sub(evidence.tested_at_ms)
                                <= PROVIDER_TEST_FRESHNESS_MS
                            && evidence
                                .validate_for(&provider.profile, evidence.tested_at_ms)
                                .is_ok()
                    })
                && provider
                    .selected_model_id
                    .as_deref()
                    .is_some_and(|model_id| {
                        provider.models.get(model_id).is_some_and(|route| {
                            route.is_production_ready_at(now_ms)
                                && route.checked_at_ms <= now_ms
                                && now_ms.saturating_sub(route.checked_at_ms)
                                    <= PROVIDER_TEST_FRESHNESS_MS
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
                    })
        })
    }
}

#[derive(Default)]
pub struct ProviderService {
    generation: u64,
    providers: BTreeMap<String, ProviderRecord>,
}

impl ProviderService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn restore(snapshot: ProviderSnapshot) -> Result<Self, ProviderError> {
        let mut providers = BTreeMap::new();
        let mut labels = BTreeSet::new();
        for record in snapshot.providers {
            record.profile.validate()?;
            let normalized_label = record.profile.display_name.trim().to_lowercase();
            if record.generation > snapshot.generation
                || record
                    .models
                    .values()
                    .any(|route| route.validate_for(&record.profile).is_err())
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
                    !record
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
            providers,
        })
    }

    pub fn snapshot(&self) -> ProviderSnapshot {
        ProviderSnapshot {
            generation: self.generation,
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
                    || current.profile.credential_reference != profile.credential_reference
                    || current.profile.kind != profile.kind
                    || current.profile.enabled != profile.enabled
            });
        let generation = self.next_generation()?;
        let (test_status, connection_evidence, models, selected_model_id) = if reset_discovery {
            (ProviderTestStatus::Untested, None, BTreeMap::new(), None)
        } else {
            let current = self
                .providers
                .get(&profile.provider_id)
                .expect("an unchanged profile already exists");
            (
                current.test_status.clone(),
                current.connection_evidence.clone(),
                current.models.clone(),
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
                let selected_model_id = choose_recommended(
                    &models,
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
        {
            return Err(ProviderError::ModelUnavailable);
        }
        record.selected_model_id = Some(model_id.into());
        record.generation = generation;
        self.generation = generation;
        Ok(generation)
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
        {
            return Some(candidate.into());
        }
    }
    models
        .values()
        .filter(|route| route.is_production_ready())
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
        endpoint_id: profile.endpoint.endpoint_id.clone(),
        base_url: profile.endpoint.base_url.clone(),
        api_kind: profile.endpoint.api_kind.clone(),
        native_provider_id: profile.native_provider_id().into(),
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
        || validate_id(&request.endpoint_id).is_err()
        || validate_id(&request.api_kind).is_err()
        || validate_id(&request.native_provider_id).is_err()
        || validate_https_endpoint(&request.base_url).is_err()
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
    let suffix = if request.native_provider_id == "openrouter" {
        "auth/key"
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
    writer.write_all(b"\"\nheader = \"Accept: application/json\"\nheader = \"")?;
    let header = match request.native_provider_id.as_str() {
        "anthropic" => "x-api-key: ",
        "google" => "x-goog-api-key: ",
        _ => "Authorization: Bearer ",
    };
    writer.write_all(header.as_bytes())
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
        credential_reference: &'a str,
        attempted_at_ms: u64,
    }

    sha256_json(&Binding {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: &profile.provider_id,
        provider_kind: profile.kind,
        endpoint: &profile.endpoint,
        credential_reference: profile.credential_reference.as_str(),
        attempted_at_ms,
    })
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
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
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

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

pub(crate) fn validate_https_endpoint(value: &str) -> Result<(), ProviderError> {
    if value.is_empty()
        || value.len() > MAX_ENDPOINT_BYTES
        || !value.starts_with("https://")
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
        .strip_prefix("https://")
        .and_then(|rest| rest.split('/').next())
        .unwrap_or_default();
    if authority.is_empty() || authority.starts_with('.') || authority.ends_with('.') {
        return Err(ProviderError::InvalidEndpoint);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ProviderError {
    #[error("provider profile is invalid")]
    InvalidProfile,
    #[error("provider endpoint must be a bounded HTTPS URL without embedded credentials")]
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
