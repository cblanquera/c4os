//! Worker-requested provider credential delivery for OpenCode 1.18.3.
//!
//! This channel is deliberately separate from the secret-rejecting Action
//! Gateway broker. Before native prompt dispatch, the Rust core authorizes an
//! exact process/session/provider/model/attempt/message binding derived from a
//! validated provider profile. Each `chat.headers` call must then request a
//! fresh one-use lease over the private inherited descriptor. Rust validates
//! the active binding and resolves the opaque vault reference; neither the
//! reference nor the raw secret is serialized into prompt state.

#![cfg(unix)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::runtime::provider::{ProviderAuthentication, ProviderProfile};
use crate::security::credentials::{CredentialReference, CredentialVault, CredentialVaultError};

pub const OPENCODE_CREDENTIAL_FD_ENV: &str = "C4OS_OPENCODE_CREDENTIAL_FD";
pub const OPENCODE_CREDENTIAL_SCHEMA_VERSION: u16 = 3;

const MIN_CREDENTIAL_FD: RawFd = 256;
const MAX_CREDENTIAL_FD: RawFd = 1_023;
const MAX_ID_BYTES: usize = 192;
const MAX_PROVIDERS: usize = 128;
const MAX_ACTIVE_ATTEMPTS: usize = 128;
const MAX_REQUESTS_PER_ATTEMPT: u16 = 128;
const MAX_USED_REQUESTS: usize = 4_096;
const MAX_REQUEST_FRAME_BYTES: usize = 4 * 1024;
const MAX_SECRET_BYTES: usize = 64 * 1024;
const ATTEMPT_AUTHORIZATION_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const OPERATION_LEASE_TTL: Duration = Duration::from_secs(30);
const DESCRIPTOR_TTL_MS: u64 = 15_000;
const OPENCODE_MESSAGE_ID_PREFIX: &str = "msg_";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredentialRequest {
    pub process_generation: u64,
    pub native_session_id: String,
    pub provider_id: String,
    pub model_id: String,
    /// Core-owned attempt/correlation identity. This is not a credential lease
    /// ID; every matching worker request receives a separately minted lease.
    pub operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCredentialAuthorizationReceipt {
    pub process_generation: u64,
    pub native_session_id: String,
    pub c4os_provider_id: String,
    pub native_provider_id: String,
    pub native_model_id: String,
    pub operation_authorization_id: String,
    pub credential_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProviderHeader {
    AuthorizationBearer,
    Named(String),
}

impl ProviderHeader {
    fn name(&self) -> &str {
        match self {
            Self::AuthorizationBearer => "Authorization",
            Self::Named(name) => name,
        }
    }

    fn prefix(&self) -> &'static str {
        match self {
            Self::AuthorizationBearer => "Bearer ",
            Self::Named(_) => "",
        }
    }
}

#[derive(Clone)]
struct ProviderCredentialRoute {
    profile_id: String,
    native_provider_id: String,
    credential_reference: Option<CredentialReference>,
    header: Option<ProviderHeader>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AttemptBinding {
    process_generation: u64,
    native_session_id: String,
    provider_id: String,
    model_id: String,
    operation_id: String,
    native_message_id: String,
}

struct AttemptAuthorization {
    authorization_id: String,
    profile_id: String,
    credential_reference: CredentialReference,
    header: ProviderHeader,
    expires_at: Instant,
    issued_requests: u16,
}

struct CredentialIssuerState {
    vault: CredentialVault,
    process_generation: u64,
    routes: BTreeMap<String, ProviderCredentialRoute>,
    authorizations: BTreeMap<AttemptBinding, AttemptAuthorization>,
    used_request_ids: BTreeSet<String>,
    failed: bool,
}

/// The parent endpoint and authorization registry for the private credential
/// request/response channel.
pub struct OpenCodeProviderCredentialIssuer {
    state: Arc<Mutex<CredentialIssuerState>>,
    control_channel: UnixStream,
    responder: Option<JoinHandle<()>>,
}

/// The worker endpoint capability. It contains descriptor metadata only and
/// becomes inheritable exclusively inside the exact launch command.
pub struct OpenCodeProviderCredentialLaunchBinding {
    worker_channel: UnixStream,
}

impl OpenCodeProviderCredentialIssuer {
    pub fn authenticated_pair(
        vault: CredentialVault,
        process_generation: u64,
    ) -> Result<(Self, OpenCodeProviderCredentialLaunchBinding), ProviderCredentialError> {
        if process_generation == 0 || process_generation > u32::MAX.into() {
            return Err(ProviderCredentialError::InvalidConfiguration);
        }
        let (c4os, worker) = UnixStream::pair().map_err(ProviderCredentialError::Io)?;
        c4os.set_write_timeout(Some(Duration::from_secs(10)))
            .map_err(ProviderCredentialError::Io)?;
        let worker = duplicate_bounded_descriptor(&worker)?;
        worker
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(ProviderCredentialError::Io)?;
        worker
            .set_write_timeout(Some(Duration::from_secs(10)))
            .map_err(ProviderCredentialError::Io)?;
        let control_channel = c4os.try_clone().map_err(ProviderCredentialError::Io)?;
        let state = Arc::new(Mutex::new(CredentialIssuerState {
            vault,
            process_generation,
            routes: BTreeMap::new(),
            authorizations: BTreeMap::new(),
            used_request_ids: BTreeSet::new(),
            failed: false,
        }));
        let responder_state = Arc::clone(&state);
        let responder = thread::Builder::new()
            .name(format!("c4os-opencode-credential-{process_generation}"))
            .spawn(move || serve_worker_requests(c4os, responder_state))
            .map_err(ProviderCredentialError::Io)?;
        Ok((
            Self {
                state,
                control_channel,
                responder: Some(responder),
            },
            OpenCodeProviderCredentialLaunchBinding {
                worker_channel: worker,
            },
        ))
    }

    /// Registers only a validated core-owned provider profile. The raw secret
    /// is never accepted by this API and the opaque reference stays in Rust.
    pub fn register_provider(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<(), ProviderCredentialError> {
        profile
            .validate()
            .map_err(|_| ProviderCredentialError::InvalidProviderRoute)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProviderCredentialError::ChannelUnavailable)?;
        if state.failed || !profile.enabled || state.routes.len() >= MAX_PROVIDERS {
            return Err(ProviderCredentialError::InvalidProviderRoute);
        }
        let native_provider_id = profile.opencode_native_provider_id().to_owned();
        let header = match &profile.authentication {
            ProviderAuthentication::Bearer => Some(ProviderHeader::AuthorizationBearer),
            ProviderAuthentication::ApiKeyHeader { header_name } => {
                Some(ProviderHeader::Named(header_name.clone()))
            }
            ProviderAuthentication::None => None,
        };
        let credential_reference = profile.credential_reference.clone();
        let route = ProviderCredentialRoute {
            profile_id: profile.provider_id.clone(),
            native_provider_id,
            credential_reference,
            header,
        };
        if state
            .routes
            .get(&profile.provider_id)
            .is_some_and(|current| current.credential_reference != route.credential_reference)
        {
            return Err(ProviderCredentialError::ProviderRouteConflict);
        }
        state.routes.insert(profile.provider_id.clone(), route);
        Ok(())
    }

    /// Authorizes an exact active C4OS attempt. This method delivers no secret;
    /// the worker must request each one-use lease from `chat.headers`.
    pub fn authorize_attempt(
        &mut self,
        request: ProviderCredentialRequest,
    ) -> Result<ProviderCredentialAuthorizationReceipt, ProviderCredentialError> {
        validate_request(&request)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProviderCredentialError::ChannelUnavailable)?;
        if state.failed {
            return Err(ProviderCredentialError::ChannelUnavailable);
        }
        if request.process_generation != state.process_generation {
            return Err(ProviderCredentialError::BindingMismatch);
        }
        let now = Instant::now();
        state
            .authorizations
            .retain(|_, authorization| authorization.expires_at > now);
        let route = state
            .routes
            .get(&request.provider_id)
            .ok_or(ProviderCredentialError::ProviderUnavailable)?
            .clone();
        let native_message_id = opencode_message_id_for_operation(&request.operation_id)?;
        let native_model_id = request
            .model_id
            .strip_prefix(&route.native_provider_id)
            .and_then(|value| value.strip_prefix('/'))
            .unwrap_or(&request.model_id)
            .to_owned();
        if !valid_id(&native_model_id) {
            return Err(ProviderCredentialError::BindingMismatch);
        }
        let binding = AttemptBinding {
            process_generation: state.process_generation,
            native_session_id: request.native_session_id.clone(),
            provider_id: route.native_provider_id.clone(),
            model_id: native_model_id.clone(),
            operation_id: request.operation_id,
            native_message_id,
        };
        if state.authorizations.contains_key(&binding) {
            return Err(ProviderCredentialError::AuthorizationReplay);
        }
        if state.authorizations.len() >= MAX_ACTIVE_ATTEMPTS {
            return Err(ProviderCredentialError::BackpressureExceeded);
        }
        let operation_authorization_id =
            format!("credential-authorization:{}", Uuid::new_v4().as_simple());
        let credential_required = route.credential_reference.is_some();
        if credential_required != route.header.is_some() {
            return Err(ProviderCredentialError::InvalidProviderRoute);
        }
        if !credential_required {
            return Ok(ProviderCredentialAuthorizationReceipt {
                process_generation: state.process_generation,
                native_session_id: request.native_session_id,
                c4os_provider_id: route.profile_id,
                native_provider_id: route.native_provider_id,
                native_model_id,
                operation_authorization_id,
                credential_required: false,
            });
        }
        state.authorizations.insert(
            binding,
            AttemptAuthorization {
                authorization_id: operation_authorization_id.clone(),
                profile_id: route.profile_id.clone(),
                credential_reference: route
                    .credential_reference
                    .expect("validated credential route"),
                header: route.header.expect("validated credential header"),
                expires_at: now + ATTEMPT_AUTHORIZATION_TTL,
                issued_requests: 0,
            },
        );
        Ok(ProviderCredentialAuthorizationReceipt {
            process_generation: state.process_generation,
            native_session_id: request.native_session_id,
            c4os_provider_id: route.profile_id,
            native_provider_id: route.native_provider_id,
            native_model_id,
            operation_authorization_id,
            credential_required: true,
        })
    }

    pub fn revoke_attempt(
        &mut self,
        request: &ProviderCredentialRequest,
    ) -> Result<bool, ProviderCredentialError> {
        validate_request(request)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProviderCredentialError::ChannelUnavailable)?;
        if request.process_generation != state.process_generation {
            return Err(ProviderCredentialError::BindingMismatch);
        }
        let route = state
            .routes
            .get(&request.provider_id)
            .ok_or(ProviderCredentialError::ProviderUnavailable)?;
        let native_model_id = request
            .model_id
            .strip_prefix(&route.native_provider_id)
            .and_then(|value| value.strip_prefix('/'))
            .unwrap_or(&request.model_id);
        let binding = AttemptBinding {
            process_generation: request.process_generation,
            native_session_id: request.native_session_id.clone(),
            provider_id: route.native_provider_id.clone(),
            model_id: native_model_id.to_owned(),
            operation_id: request.operation_id.clone(),
            native_message_id: opencode_message_id_for_operation(&request.operation_id)?,
        };
        Ok(state.authorizations.remove(&binding).is_some())
    }

    pub fn revoke_all_attempts(&mut self) -> Result<(), ProviderCredentialError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ProviderCredentialError::ChannelUnavailable)?;
        state.authorizations.clear();
        Ok(())
    }
}

impl Drop for OpenCodeProviderCredentialIssuer {
    fn drop(&mut self) {
        let _ = self.control_channel.shutdown(Shutdown::Both);
        if let Some(responder) = self.responder.take() {
            let _ = responder.join();
        }
    }
}

impl OpenCodeProviderCredentialLaunchBinding {
    pub fn child_fd(&self) -> u32 {
        u32::try_from(self.worker_channel.as_raw_fd()).expect("credential descriptor is positive")
    }

    pub fn environment(&self) -> BTreeMap<String, String> {
        BTreeMap::from([(
            OPENCODE_CREDENTIAL_FD_ENV.into(),
            self.child_fd().to_string(),
        )])
    }

    pub fn configure_command(&self, command: &mut Command) {
        command.envs(self.environment());
        let descriptor = self.worker_channel.as_raw_fd();
        // SAFETY: the callback performs only descriptor flag syscalls against
        // a descriptor kept alive by this binding through `Command::spawn`.
        unsafe {
            command.pre_exec(move || {
                let flags = libc::fcntl(descriptor, libc::F_GETFD);
                if flags < 0
                    || libc::fcntl(descriptor, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    #[doc(hidden)]
    pub fn duplicate_worker_for_test(&self) -> std::io::Result<UnixStream> {
        self.worker_channel.try_clone()
    }
}

impl fmt::Debug for OpenCodeProviderCredentialIssuer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.lock().ok();
        formatter
            .debug_struct("OpenCodeProviderCredentialIssuer")
            .field(
                "process_generation",
                &state.as_ref().map(|state| state.process_generation),
            )
            .field(
                "registered_routes",
                &state.as_ref().map(|state| state.routes.len()).unwrap_or(0),
            )
            .field(
                "active_attempts",
                &state
                    .as_ref()
                    .map(|state| state.authorizations.len())
                    .unwrap_or(0),
            )
            .field(
                "used_requests",
                &state
                    .as_ref()
                    .map(|state| state.used_request_ids.len())
                    .unwrap_or(0),
            )
            .field(
                "channel_failed",
                &state.as_ref().is_none_or(|state| state.failed),
            )
            .field("credential_references", &"<opaque>")
            .finish()
    }
}

impl fmt::Debug for OpenCodeProviderCredentialLaunchBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenCodeProviderCredentialLaunchBinding")
            .field("child_fd", &self.child_fd())
            .field("secret", &"<descriptor-only>")
            .finish()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CredentialLeaseRequest {
    schema_version: u16,
    kind: String,
    request_id: String,
    process_generation: u64,
    native_session_id: String,
    provider_id: String,
    model_id: String,
    operation_id: String,
    native_message_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialFrameMetadata {
    schema_version: u16,
    kind: &'static str,
    request_id: String,
    lease_id: String,
    authorization_id: String,
    process_generation: u64,
    native_session_id: String,
    provider_id: String,
    model_id: String,
    operation_id: String,
    native_message_id: String,
    header_name: String,
    header_prefix: &'static str,
    secret_length: usize,
    ttl_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialRejection<'a> {
    schema_version: u16,
    kind: &'static str,
    request_id: &'a str,
    code: &'static str,
}

#[derive(Debug, Error)]
pub enum ProviderCredentialError {
    #[error("OpenCode provider credential channel configuration is invalid")]
    InvalidConfiguration,
    #[error("OpenCode provider credential route is invalid")]
    InvalidProviderRoute,
    #[error("OpenCode provider credential route conflicts with an existing route")]
    ProviderRouteConflict,
    #[error("OpenCode provider credential request binding did not match")]
    BindingMismatch,
    #[error("OpenCode provider credential route is unavailable")]
    ProviderUnavailable,
    #[error("OpenCode provider credential attempt authorization was replayed")]
    AuthorizationReplay,
    #[error("OpenCode provider credential lease request was replayed")]
    LeaseReplay,
    #[error("OpenCode provider credential channel reached its bound")]
    BackpressureExceeded,
    #[error("OpenCode provider credential attempt authorization expired")]
    AuthorizationExpired,
    #[error("OpenCode provider credential value is not a bounded textual API key")]
    InvalidProviderCredential,
    #[error("OpenCode provider credential channel is unavailable")]
    ChannelUnavailable,
    #[error("OpenCode provider credential vault operation failed")]
    Vault(#[source] CredentialVaultError),
    #[error("OpenCode provider credential descriptor I/O failed")]
    Io(#[source] std::io::Error),
}

fn serve_worker_requests(channel: UnixStream, state: Arc<Mutex<CredentialIssuerState>>) {
    let mut reader = BufReader::new(channel);
    loop {
        let mut line = Vec::new();
        let read = (&mut reader)
            .take((MAX_REQUEST_FRAME_BYTES + 1) as u64)
            .read_until(b'\n', &mut line);
        let Ok(bytes_read) = read else {
            mark_failed(&state);
            return;
        };
        if bytes_read == 0 {
            mark_failed(&state);
            return;
        }
        if line.len() > MAX_REQUEST_FRAME_BYTES || line.last() != Some(&b'\n') {
            mark_failed(&state);
            return;
        }
        line.pop();
        let Ok(request) = serde_json::from_slice::<CredentialLeaseRequest>(&line) else {
            line.fill(0);
            mark_failed(&state);
            return;
        };
        line.fill(0);
        let delivery = issue_worker_lease(&state, &request).and_then(|(metadata, mut secret)| {
            let mut header = serde_json::to_vec(&metadata)
                .map_err(|_| ProviderCredentialError::InvalidConfiguration)?;
            header.push(b'\n');
            let result = reader
                .get_mut()
                .write_all(&header)
                .and_then(|_| reader.get_mut().write_all(&secret))
                .and_then(|_| reader.get_mut().flush())
                .map_err(ProviderCredentialError::Io);
            header.fill(0);
            secret.fill(0);
            result
        });
        if let Err(error) = delivery {
            let rejection = CredentialRejection {
                schema_version: OPENCODE_CREDENTIAL_SCHEMA_VERSION,
                kind: "providerCredentialRejected",
                request_id: &request.request_id,
                code: rejection_code(&error),
            };
            let response = serde_json::to_vec(&rejection)
                .map_err(|_| ProviderCredentialError::InvalidConfiguration)
                .and_then(|mut response| {
                    response.push(b'\n');
                    let result = reader
                        .get_mut()
                        .write_all(&response)
                        .and_then(|_| reader.get_mut().flush())
                        .map_err(ProviderCredentialError::Io);
                    response.fill(0);
                    result
                });
            if response.is_err() || matches!(error, ProviderCredentialError::ChannelUnavailable) {
                mark_failed(&state);
                return;
            }
        }
    }
}

fn issue_worker_lease(
    shared: &Arc<Mutex<CredentialIssuerState>>,
    request: &CredentialLeaseRequest,
) -> Result<(CredentialFrameMetadata, Zeroizing<Vec<u8>>), ProviderCredentialError> {
    validate_lease_request(request)?;
    let mut state = shared
        .lock()
        .map_err(|_| ProviderCredentialError::ChannelUnavailable)?;
    if state.failed {
        return Err(ProviderCredentialError::ChannelUnavailable);
    }
    if request.process_generation != state.process_generation {
        return Err(ProviderCredentialError::BindingMismatch);
    }
    if state.used_request_ids.len() >= MAX_USED_REQUESTS
        || !state.used_request_ids.insert(request.request_id.clone())
    {
        return Err(ProviderCredentialError::LeaseReplay);
    }
    let binding = AttemptBinding {
        process_generation: request.process_generation,
        native_session_id: request.native_session_id.clone(),
        provider_id: request.provider_id.clone(),
        model_id: request.model_id.clone(),
        operation_id: request.operation_id.clone(),
        native_message_id: request.native_message_id.clone(),
    };
    let now = Instant::now();
    let authorization = state
        .authorizations
        .get_mut(&binding)
        .ok_or(ProviderCredentialError::BindingMismatch)?;
    if authorization.expires_at <= now {
        return Err(ProviderCredentialError::AuthorizationExpired);
    }
    if authorization.issued_requests >= MAX_REQUESTS_PER_ATTEMPT {
        return Err(ProviderCredentialError::BackpressureExceeded);
    }
    authorization.issued_requests += 1;
    let authorization_id = authorization.authorization_id.clone();
    let profile_id = authorization.profile_id.clone();
    let credential_reference = authorization.credential_reference.clone();
    let header = authorization.header.clone();
    let lease_id = format!("credential-lease:{}", Uuid::new_v4().as_simple());
    let operation = format!(
        "opencode-provider-request:{}:{}:{}:{}:{}",
        request.process_generation,
        request.native_session_id,
        profile_id,
        request.operation_id,
        request.request_id
    );
    let lease = state
        .vault
        .lease_for_operation(&credential_reference, &operation, OPERATION_LEASE_TTL)
        .map_err(ProviderCredentialError::Vault)?;
    let mut secret = Zeroizing::new(Vec::new());
    lease
        .deliver_to(&mut *secret)
        .map_err(ProviderCredentialError::Vault)?;
    if secret.is_empty()
        || secret.len() > MAX_SECRET_BYTES
        || std::str::from_utf8(&secret).is_err()
        || secret
            .iter()
            .any(|byte| matches!(byte, b'\0' | b'\r' | b'\n'))
    {
        return Err(ProviderCredentialError::InvalidProviderCredential);
    }
    let metadata = CredentialFrameMetadata {
        schema_version: OPENCODE_CREDENTIAL_SCHEMA_VERSION,
        kind: "providerCredential",
        request_id: request.request_id.clone(),
        lease_id,
        authorization_id,
        process_generation: request.process_generation,
        native_session_id: request.native_session_id.clone(),
        provider_id: request.provider_id.clone(),
        model_id: request.model_id.clone(),
        operation_id: request.operation_id.clone(),
        native_message_id: request.native_message_id.clone(),
        header_name: header.name().to_owned(),
        header_prefix: header.prefix(),
        secret_length: secret.len(),
        ttl_ms: DESCRIPTOR_TTL_MS,
    };
    Ok((metadata, secret))
}

fn rejection_code(error: &ProviderCredentialError) -> &'static str {
    match error {
        ProviderCredentialError::BindingMismatch => "binding_mismatch",
        ProviderCredentialError::AuthorizationExpired => "authorization_expired",
        ProviderCredentialError::LeaseReplay => "lease_replay",
        ProviderCredentialError::BackpressureExceeded => "backpressure_exceeded",
        ProviderCredentialError::ProviderUnavailable => "provider_unavailable",
        _ => "credential_unavailable",
    }
}

fn mark_failed(state: &Arc<Mutex<CredentialIssuerState>>) {
    if let Ok(mut state) = state.lock() {
        state.failed = true;
        state.authorizations.clear();
    }
}

fn validate_request(request: &ProviderCredentialRequest) -> Result<(), ProviderCredentialError> {
    if request.process_generation == 0
        || !valid_id(&request.native_session_id)
        || !valid_id(&request.provider_id)
        || !valid_id(&request.model_id)
        || opencode_message_id_for_operation(&request.operation_id).is_err()
    {
        return Err(ProviderCredentialError::BindingMismatch);
    }
    Ok(())
}

fn validate_lease_request(request: &CredentialLeaseRequest) -> Result<(), ProviderCredentialError> {
    if request.schema_version != OPENCODE_CREDENTIAL_SCHEMA_VERSION
        || request.kind != "providerCredentialRequest"
        || request.process_generation == 0
        || !valid_id(&request.request_id)
        || !valid_id(&request.native_session_id)
        || !valid_id(&request.provider_id)
        || !valid_id(&request.model_id)
        || !valid_id(&request.operation_id)
        || opencode_message_id_for_operation(&request.operation_id)? != request.native_message_id
    {
        return Err(ProviderCredentialError::BindingMismatch);
    }
    Ok(())
}

/// Maps one core-owned attempt/correlation identity to the exact OpenCode
/// user-message identity carried by `prompt_async`. OpenCode 1.18.3 requires
/// caller-supplied message identities to begin with `msg`.
pub fn opencode_message_id_for_operation(
    operation_id: &str,
) -> Result<String, ProviderCredentialError> {
    if !valid_id(operation_id)
        || operation_id.len() + OPENCODE_MESSAGE_ID_PREFIX.len() > MAX_ID_BYTES
    {
        return Err(ProviderCredentialError::BindingMismatch);
    }
    Ok(format!("{OPENCODE_MESSAGE_ID_PREFIX}{operation_id}"))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_BYTES
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric()
                || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'@' | b'-' | b'/'))
        })
}

fn duplicate_bounded_descriptor(
    stream: &UnixStream,
) -> Result<UnixStream, ProviderCredentialError> {
    // SAFETY: `fcntl` duplicates one live descriptor; ownership transfers to
    // the returned `UnixStream` only after a successful bounded result.
    let descriptor =
        unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_DUPFD_CLOEXEC, MIN_CREDENTIAL_FD) };
    if !(MIN_CREDENTIAL_FD..=MAX_CREDENTIAL_FD).contains(&descriptor) {
        if descriptor >= 0 {
            // SAFETY: this branch owns the just-duplicated descriptor.
            unsafe { libc::close(descriptor) };
        }
        return Err(ProviderCredentialError::InvalidConfiguration);
    }
    // SAFETY: `descriptor` is a unique successful duplicate owned here.
    Ok(unsafe { UnixStream::from_raw_fd(descriptor) })
}
