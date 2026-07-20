//! Native process and authenticated HTTP boundary for OpenCode 1.18.3.
//!
//! OpenCode is a disposable worker. This module verifies its pinned executable,
//! starts it in an isolated process group through a digest-pinned launcher, and
//! resolves vault credentials only into one-use operation leases. The server
//! password is absent from argv, ordinary environment inherited by the
//! launcher, persisted state, diagnostics, and `Debug` output.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::runtime::opencode::{
    C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL, CommandDriver, CommandFailureCode,
    HttpMethod, LaunchCommand, LoopbackEndpoint, NativePermissionDefault, OPENCODE_NATIVE_VERSION,
    OpenCodeTransport, ProcessHandle, RandomSecretReference, TransportAuth, TransportFailureCode,
    TransportRequest, TransportResponse,
};
use crate::runtime::opencode_assets::PinnedOpenCodeLaunchIdentity;
use crate::runtime::opencode_credential::{
    OpenCodeProviderCredentialIssuer, OpenCodeProviderCredentialLaunchBinding,
    ProviderCredentialAuthorizationReceipt, ProviderCredentialRequest,
};
use crate::runtime::opencode_sdk::{
    BrokerDecision, BrokerEvent, OpenCodeSdkBroker, OpenCodeSdkError, OpenCodeSdkIntegrity,
    prepare_opencode_sdk_launch,
};
use crate::runtime::opencode_stream::{
    OpenCodeEventSubscription, OpenCodeStreamConnector, OpenCodeStreamError,
};
use crate::runtime::provider::ProviderProfile;
use crate::runtime::supervisor::sha256_file;
use crate::security::credentials::{
    CredentialReference, CredentialVault, CredentialVaultError, OperationCredentialLease,
};

/// The only tool identities the native runtime may see as available. They are
/// proposal/read boundaries owned by C4OS, never native effect implementations.
pub const OPENCODE_C4OS_TOOL_IDS: [&str; 2] = [C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL];
/// OpenCode's own permission system is not a sandbox. Deny every native tool
/// and allow only the exact C4OS-owned proposal/read identities above.
pub const OPENCODE_AUTHORITY_CONFIG: &str =
    r#"{"permission":{"*":"deny","c4os_propose_action":"allow","c4os_read_resource":"allow"}}"#;

const MAX_HTTP_HEADER_BYTES: usize = 32 * 1024;
const MAX_HTTP_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_HTTP_PATH_BYTES: usize = 4 * 1024;
const MAX_HTTP_HEADERS: usize = 128;
const MAX_TIMEOUT: Duration = Duration::from_secs(30);
const CREDENTIAL_LEASE_TTL: Duration = Duration::from_secs(30);
const MIN_SECRET_DESCRIPTOR: u32 = 64;
const MAX_SECRET_DESCRIPTOR: u32 = 1_023;
const OPENCODE_READY_DESCRIPTOR: i32 = 199;
const OPENCODE_READY_FD_ENV: &str = "C4OS_OPENCODE_READY_FD";
const OPENCODE_READY_SIGNAL: u8 = 0x01;
const MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeBoundaryError {
    InvalidConfiguration,
    CredentialMappingConflict,
}

#[derive(Debug, thiserror::Error)]
pub enum NativeBrokerError {
    #[error("OpenCode SDK broker is unavailable")]
    Unavailable,
    #[error("OpenCode SDK broker channel failed")]
    Channel(#[source] OpenCodeSdkError),
}

/// A cloneable mapping from an adapter-owned random-secret identity to the
/// vault's opaque credential identity. Neither identity contains secret bytes.
#[derive(Clone)]
pub struct VaultCredentialResolver {
    vault: CredentialVault,
    references: Arc<RwLock<BTreeMap<String, CredentialReference>>>,
}

impl VaultCredentialResolver {
    pub fn new(vault: CredentialVault) -> Self {
        Self {
            vault,
            references: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn register(
        &self,
        random_reference: &RandomSecretReference,
        credential_reference: CredentialReference,
    ) -> Result<(), NativeBoundaryError> {
        let mut references = self
            .references
            .write()
            .map_err(|_| NativeBoundaryError::InvalidConfiguration)?;
        match references.get(random_reference.reference_id()) {
            Some(current) if current != &credential_reference => {
                Err(NativeBoundaryError::CredentialMappingConflict)
            }
            Some(_) => Ok(()),
            None => {
                references.insert(
                    random_reference.reference_id().to_owned(),
                    credential_reference,
                );
                Ok(())
            }
        }
    }

    fn lease(
        &self,
        random_reference: &RandomSecretReference,
        operation: &str,
    ) -> Result<OperationCredentialLease, CredentialVaultError> {
        let credential_reference = self
            .references
            .read()
            .map_err(|_| CredentialVaultError::StateUnavailable)?
            .get(random_reference.reference_id())
            .cloned()
            .ok_or(CredentialVaultError::CredentialNotFound)?;
        self.vault
            .lease_for_operation(&credential_reference, operation, CREDENTIAL_LEASE_TTL)
    }

    pub(crate) fn credential_vault(&self) -> CredentialVault {
        self.vault.clone()
    }

    pub(crate) fn unregister(
        &self,
        random_reference: &RandomSecretReference,
        credential_reference: &CredentialReference,
    ) -> Result<(), NativeBoundaryError> {
        let mut references = self
            .references
            .write()
            .map_err(|_| NativeBoundaryError::InvalidConfiguration)?;
        match references.get(random_reference.reference_id()) {
            Some(current) if current == credential_reference => {
                references.remove(random_reference.reference_id());
                Ok(())
            }
            Some(_) => Err(NativeBoundaryError::CredentialMappingConflict),
            None => Ok(()),
        }
    }
}

impl fmt::Debug for VaultCredentialResolver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VaultCredentialResolver")
            .field(
                "registered_references",
                &self.references.read().map(|value| value.len()).unwrap_or(0),
            )
            .field("credentials", &"<opaque>")
            .finish()
    }
}

/// Bounded HTTP/1.1 transport fixed to one loopback endpoint.
pub struct LoopbackHttpTransport {
    endpoint: LoopbackEndpoint,
    credentials: VaultCredentialResolver,
    connect_timeout: Duration,
    io_timeout: Duration,
}

impl LoopbackHttpTransport {
    pub fn new(
        endpoint: LoopbackEndpoint,
        credentials: VaultCredentialResolver,
        connect_timeout: Duration,
        io_timeout: Duration,
    ) -> Result<Self, NativeBoundaryError> {
        if connect_timeout.is_zero()
            || io_timeout.is_zero()
            || connect_timeout > MAX_TIMEOUT
            || io_timeout > MAX_TIMEOUT
        {
            return Err(NativeBoundaryError::InvalidConfiguration);
        }
        Ok(Self {
            endpoint,
            credentials,
            connect_timeout,
            io_timeout,
        })
    }

    fn execute_inner(
        &self,
        request: TransportRequest,
    ) -> Result<TransportResponse, TransportFailureCode> {
        if request.base_url != self.endpoint.base_url()
            || !valid_http_path(&request.path)
            || request.maximum_response_bytes == 0
            || request.maximum_response_bytes > MAX_HTTP_RESPONSE_BYTES
            || request
                .body
                .as_ref()
                .is_some_and(|body| body.len() > MAX_HTTP_RESPONSE_BYTES)
        {
            return Err(TransportFailureCode::Protocol);
        }

        let (username, password_reference) = match &request.auth {
            TransportAuth::Basic {
                username,
                password_reference,
            } if valid_basic_username(username) => (username.as_str(), password_reference),
            _ => return Err(TransportFailureCode::AuthenticationRejected),
        };
        let operation = format!(
            "opencode-http-{}-{}",
            method_name(request.method).to_ascii_lowercase(),
            sha256_bytes(request.path.as_bytes()).trim_start_matches("sha256:")
        );
        let lease = self
            .credentials
            .lease(password_reference, &operation)
            .map_err(|_| TransportFailureCode::AuthenticationRejected)?;

        let mut user_password = Zeroizing::new(Vec::with_capacity(username.len() + 65));
        user_password.extend_from_slice(username.as_bytes());
        user_password.push(b':');
        lease
            .deliver_to(&mut *user_password)
            .map_err(|_| TransportFailureCode::AuthenticationRejected)?;
        let authorization = Zeroizing::new(base64_encode(&user_password));

        let body = request.body.as_deref().unwrap_or_default();
        let mut wire = Zeroizing::new(Vec::with_capacity(
            384usize
                .saturating_add(request.path.len())
                .saturating_add(authorization.len())
                .saturating_add(body.len()),
        ));
        write!(
            &mut *wire,
            "{} {} HTTP/1.1\r\nHost: {}\r\nAuthorization: Basic ",
            method_name(request.method),
            request.path,
            host_header(&self.endpoint)
        )
        .map_err(|_| TransportFailureCode::Protocol)?;
        wire.extend_from_slice(&authorization);
        write!(
            &mut *wire,
            "\r\nAccept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .map_err(|_| TransportFailureCode::Protocol)?;
        wire.extend_from_slice(body);

        let address = SocketAddr::new(self.endpoint.address(), self.endpoint.port());
        let mut stream = TcpStream::connect_timeout(&address, self.connect_timeout)
            .map_err(map_connect_error)?;
        stream
            .set_read_timeout(Some(self.io_timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.io_timeout)))
            .map_err(map_io_error)?;
        stream.write_all(&wire).map_err(map_io_error)?;
        let response =
            read_http_response(&mut stream, request.maximum_response_bytes, self.io_timeout)?;
        if matches!(response.status, 401 | 403) {
            return Err(TransportFailureCode::AuthenticationRejected);
        }
        Ok(response)
    }

    pub(crate) fn event_stream_connector(
        &self,
        username: String,
        password_reference: RandomSecretReference,
    ) -> Result<LoopbackEventStreamConnector, NativeBoundaryError> {
        LoopbackEventStreamConnector::new(
            self.endpoint.clone(),
            self.credentials.clone(),
            username,
            password_reference,
            self.connect_timeout,
        )
    }
}

impl OpenCodeTransport for LoopbackHttpTransport {
    fn execute(
        &mut self,
        request: TransportRequest,
    ) -> Result<TransportResponse, TransportFailureCode> {
        self.execute_inner(request)
    }
}

impl fmt::Debug for LoopbackHttpTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoopbackHttpTransport")
            .field("endpoint", &self.endpoint.base_url())
            .field("credentials", &"<opaque>")
            .field("connect_timeout", &self.connect_timeout)
            .field("io_timeout", &self.io_timeout)
            .finish()
    }
}

/// Narrow authenticated connector for the pinned SDK's workspace `GET /event`
/// SSE contract. The path is produced by [`OpenCodeEventSubscription`]; callers
/// cannot substitute an endpoint or arbitrary request path.
pub(crate) struct LoopbackEventStreamConnector {
    endpoint: LoopbackEndpoint,
    credentials: VaultCredentialResolver,
    username: String,
    password_reference: RandomSecretReference,
    connect_timeout: Duration,
}

impl LoopbackEventStreamConnector {
    pub(crate) fn new(
        endpoint: LoopbackEndpoint,
        credentials: VaultCredentialResolver,
        username: String,
        password_reference: RandomSecretReference,
        connect_timeout: Duration,
    ) -> Result<Self, NativeBoundaryError> {
        if username != "opencode" || connect_timeout.is_zero() || connect_timeout > MAX_TIMEOUT {
            return Err(NativeBoundaryError::InvalidConfiguration);
        }
        Ok(Self {
            endpoint,
            credentials,
            username,
            password_reference,
            connect_timeout,
        })
    }
}

impl OpenCodeStreamConnector for LoopbackEventStreamConnector {
    type Connection = TcpStream;

    fn connect(
        &mut self,
        subscription: &OpenCodeEventSubscription,
        read_poll_timeout: Duration,
    ) -> Result<Self::Connection, OpenCodeStreamError> {
        let path = subscription.request_path()?;
        let operation = format!(
            "opencode-http-get-{}",
            sha256_bytes(path.as_bytes()).trim_start_matches("sha256:")
        );
        let lease = self
            .credentials
            .lease(&self.password_reference, &operation)
            .map_err(|_| OpenCodeStreamError::AuthenticationRejected)?;

        let mut user_password = Zeroizing::new(Vec::with_capacity(self.username.len() + 65));
        user_password.extend_from_slice(self.username.as_bytes());
        user_password.push(b':');
        lease
            .deliver_to(&mut *user_password)
            .map_err(|_| OpenCodeStreamError::AuthenticationRejected)?;
        let authorization = Zeroizing::new(base64_encode(&user_password));
        let mut wire = Zeroizing::new(Vec::with_capacity(
            384usize
                .saturating_add(path.len())
                .saturating_add(authorization.len()),
        ));
        write!(
            &mut *wire,
            "GET {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Basic ",
            host_header(&self.endpoint)
        )
        .map_err(|_| OpenCodeStreamError::Protocol)?;
        wire.extend_from_slice(&authorization);
        wire.extend_from_slice(
            b"\r\nAccept: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        );

        let address = SocketAddr::new(self.endpoint.address(), self.endpoint.port());
        let mut stream =
            TcpStream::connect_timeout(&address, self.connect_timeout).map_err(|error| {
                match map_connect_error(error) {
                    TransportFailureCode::Timeout => OpenCodeStreamError::Timeout,
                    _ => OpenCodeStreamError::Unavailable,
                }
            })?;
        stream
            .set_read_timeout(Some(read_poll_timeout))
            .and_then(|_| stream.set_write_timeout(Some(self.connect_timeout)))
            .map_err(|_| OpenCodeStreamError::Unavailable)?;
        stream
            .write_all(&wire)
            .map_err(|_| OpenCodeStreamError::Unavailable)?;
        Ok(stream)
    }
}

impl fmt::Debug for LoopbackEventStreamConnector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoopbackEventStreamConnector")
            .field("endpoint", &self.endpoint.base_url())
            .field("credentials", &"<opaque>")
            .field("connect_timeout", &self.connect_timeout)
            .finish()
    }
}

struct ManagedOpenCodeProcess {
    child: Child,
    process_group_id: u32,
}

/// Exact-version process driver that launches OpenCode through the private-FD
/// password bridge in `sidecars/opencode-launcher`.
pub struct OpenCodeNativeCommandDriver {
    node_executable: PathBuf,
    expected_node_sha256: String,
    launcher: PathBuf,
    expected_launcher_sha256: String,
    sdk_sidecar_root: PathBuf,
    pinned_native: PinnedOpenCodeLaunchIdentity,
    credentials: VaultCredentialResolver,
    process_generation: u64,
    startup_timeout: Duration,
    shutdown_timeout: Duration,
    process: Option<ManagedOpenCodeProcess>,
    broker: Option<OpenCodeSdkBroker>,
    provider_credentials: OpenCodeProviderCredentialIssuer,
    provider_credential_binding: Option<OpenCodeProviderCredentialLaunchBinding>,
    reserved_listener: Option<TcpListener>,
    test_tls_trust_descriptor: Option<Vec<u8>>,
    test_tls_trust_descriptor_installed: bool,
}

impl OpenCodeNativeCommandDriver {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        node_executable: impl Into<PathBuf>,
        expected_node_sha256: impl Into<String>,
        launcher: impl Into<PathBuf>,
        expected_launcher_sha256: impl Into<String>,
        sdk_sidecar_root: impl Into<PathBuf>,
        pinned_native: PinnedOpenCodeLaunchIdentity,
        credentials: VaultCredentialResolver,
        provider_credential_vault: CredentialVault,
        process_generation: u64,
        startup_timeout: Duration,
        shutdown_timeout: Duration,
    ) -> Result<Self, NativeBoundaryError> {
        let sdk_sidecar_root = sdk_sidecar_root.into();
        let verified_sdk = OpenCodeSdkIntegrity::verify(&sdk_sidecar_root)
            .map_err(|_| NativeBoundaryError::InvalidConfiguration)?;
        let (provider_credentials, provider_credential_binding) =
            OpenCodeProviderCredentialIssuer::authenticated_pair(
                provider_credential_vault,
                process_generation,
            )
            .map_err(|_| NativeBoundaryError::InvalidConfiguration)?;
        let driver = Self {
            node_executable: node_executable.into(),
            expected_node_sha256: expected_node_sha256.into(),
            launcher: launcher.into(),
            expected_launcher_sha256: expected_launcher_sha256.into(),
            sdk_sidecar_root: verified_sdk.sidecar_root().to_path_buf(),
            pinned_native,
            credentials,
            process_generation,
            startup_timeout,
            shutdown_timeout,
            process: None,
            broker: None,
            provider_credentials,
            provider_credential_binding: Some(provider_credential_binding),
            reserved_listener: None,
            test_tls_trust_descriptor: None,
            test_tls_trust_descriptor_installed: false,
        };
        if !valid_absolute_path(&driver.node_executable)
            || !valid_absolute_path(&driver.launcher)
            || !valid_absolute_path(&driver.sdk_sidecar_root)
            || !valid_sha256(&driver.expected_node_sha256)
            || !valid_sha256(&driver.expected_launcher_sha256)
            || driver.process_generation == 0
            || driver.startup_timeout.is_zero()
            || driver.startup_timeout > MAX_TIMEOUT
            || driver.shutdown_timeout.is_zero()
            || driver.shutdown_timeout > MAX_TIMEOUT
        {
            return Err(NativeBoundaryError::InvalidConfiguration);
        }
        Ok(driver)
    }

    #[doc(hidden)]
    pub fn install_loopback_reservation(
        &mut self,
        listener: TcpListener,
    ) -> Result<(), NativeBoundaryError> {
        let address = listener
            .local_addr()
            .map_err(|_| NativeBoundaryError::InvalidConfiguration)?;
        if self.process.is_some()
            || self.broker.is_some()
            || self.reserved_listener.is_some()
            || !address.ip().is_loopback()
            || address.port() == 0
        {
            return Err(NativeBoundaryError::InvalidConfiguration);
        }
        self.reserved_listener = Some(listener);
        Ok(())
    }

    #[doc(hidden)]
    pub fn install_test_tls_trust_descriptor(
        &mut self,
        descriptor: Vec<u8>,
    ) -> Result<(), NativeBoundaryError> {
        if self.process.is_some()
            || self.broker.is_some()
            || self.test_tls_trust_descriptor_installed
            || descriptor.is_empty()
            || descriptor.len() > MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES
        {
            return Err(NativeBoundaryError::InvalidConfiguration);
        }
        self.test_tls_trust_descriptor = Some(descriptor);
        self.test_tls_trust_descriptor_installed = true;
        Ok(())
    }

    fn take_or_bind_listener(
        &mut self,
        endpoint: SocketAddr,
    ) -> Result<TcpListener, CommandFailureCode> {
        let listener = match self.reserved_listener.take() {
            Some(listener) => listener,
            None => TcpListener::bind(endpoint).map_err(|_| CommandFailureCode::SpawnRejected)?,
        };
        if listener.local_addr().ok() != Some(endpoint) {
            return Err(CommandFailureCode::SpawnRejected);
        }
        listener
            .set_nonblocking(true)
            .map_err(|_| CommandFailureCode::SpawnRejected)?;
        Ok(listener)
    }

    /// Binds an enabled, validated core-owned provider profile. The issuer
    /// derives and retains its opaque vault reference; no caller supplies raw
    /// secret bytes or a credential lease identity.
    pub fn register_provider(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<(), NativeBoundaryError> {
        self.provider_credentials
            .register_provider(profile)
            .map_err(|_| NativeBoundaryError::InvalidConfiguration)
    }

    pub fn receive_broker_event(
        &mut self,
        timeout: Duration,
    ) -> Result<BrokerEvent, NativeBrokerError> {
        self.broker
            .as_mut()
            .ok_or(NativeBrokerError::Unavailable)?
            .receive(timeout)
            .map_err(NativeBrokerError::Channel)
    }

    pub fn respond_to_broker_event(
        &mut self,
        correlation_id: &str,
        decision: BrokerDecision,
        timeout: Duration,
    ) -> Result<(), NativeBrokerError> {
        self.broker
            .as_mut()
            .ok_or(NativeBrokerError::Unavailable)?
            .respond(correlation_id, decision, timeout)
            .map_err(NativeBrokerError::Channel)
    }

    pub(crate) fn take_broker_for_pump(&mut self) -> Result<OpenCodeSdkBroker, NativeBrokerError> {
        if self.process.is_none() {
            return Err(NativeBrokerError::Unavailable);
        }
        self.broker.take().ok_or(NativeBrokerError::Unavailable)
    }

    pub fn broker_is_attached(&self) -> bool {
        self.process.is_some() && self.broker.is_some()
    }

    fn cleanup_failed_spawn(
        &mut self,
        mut process: ManagedOpenCodeProcess,
        broker: OpenCodeSdkBroker,
        failure: CommandFailureCode,
    ) -> CommandFailureCode {
        if terminate_process_group(&mut process, self.shutdown_timeout).is_err() {
            self.process = Some(process);
            self.broker = Some(broker);
        }
        failure
    }

    fn spawn_inner(&mut self, launch: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode> {
        if self.process.is_some() || self.broker.is_some() || !valid_launch_command(launch) {
            return Err(CommandFailureCode::SpawnRejected);
        }
        if sha256_file(&self.node_executable).ok().as_deref()
            != Some(self.expected_node_sha256.as_str())
            || sha256_file(&self.launcher).ok().as_deref()
                != Some(self.expected_launcher_sha256.as_str())
            || !self
                .pinned_native
                .accepts(&launch.executable, &launch.expected_binary_sha256)
            || launch.authority_policy.configuration_sha256()
                != opencode_authority_configuration_sha256()
        {
            return Err(CommandFailureCode::SpawnRejected);
        }

        prepare_isolated_directories(launch)?;
        let isolated_config = Path::new(
            launch
                .environment
                .get("XDG_CONFIG_HOME")
                .ok_or(CommandFailureCode::SpawnRejected)?,
        );
        let (broker, mut sdk_binding) = prepare_opencode_sdk_launch(
            &self.sdk_sidecar_root,
            isolated_config,
            self.process_generation,
            self.test_tls_trust_descriptor.take(),
        )
        .map_err(|_| CommandFailureCode::SpawnRejected)?;
        let endpoint = endpoint_from_launch(launch)?;
        let listener = self.take_or_bind_listener(endpoint)?;
        let provider_credential_binding = self
            .provider_credential_binding
            .as_ref()
            .ok_or(CommandFailureCode::ProviderCredentialUnavailable)?;
        let (mut credential_channel, child_credential_channel) =
            UnixStream::pair().map_err(|_| CommandFailureCode::SecretChannelUnavailable)?;
        let (mut readiness_channel, child_readiness_channel) =
            UnixStream::pair().map_err(|_| CommandFailureCode::SecretChannelUnavailable)?;
        let source_fd = child_credential_channel.as_raw_fd();
        let readiness_source_fd = child_readiness_channel.as_raw_fd();
        let target_fd = i32::try_from(launch.secret_channel.inherited_fd)
            .map_err(|_| CommandFailureCode::SecretChannelUnavailable)?;
        let readiness_target_fd = OPENCODE_READY_DESCRIPTOR;
        if !descriptor_channels_are_distinct(
            source_fd,
            target_fd,
            readiness_source_fd,
            readiness_target_fd,
            sdk_binding.child_fd(),
            provider_credential_binding.child_fd(),
            sdk_binding.test_tls_trust_child_fd(),
        ) {
            return Err(CommandFailureCode::SecretChannelUnavailable);
        }

        let mut command = Command::new(&self.node_executable);
        command
            .arg(&self.launcher)
            .arg("--")
            .arg(&launch.executable)
            .args(&launch.arguments)
            .current_dir(&launch.working_directory)
            .env_clear()
            .envs(&launch.environment)
            .env("C4OS_OPENCODE_SECRET_FD", target_fd.to_string())
            .env(OPENCODE_READY_FD_ENV, readiness_target_fd.to_string())
            .env("C4OS_OPENCODE_CONFIG_CONTENT", OPENCODE_AUTHORITY_CONFIG)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0);
        // This must remain after `env_clear`: the binding installs only
        // non-secret descriptor metadata and clears CLOEXEC in this child.
        sdk_binding.configure_command(&mut command);
        provider_credential_binding.configure_command(&mut command);
        // SAFETY: the closure calls only async-signal-safe descriptor syscalls.
        // The source descriptors remain owned by live Rust values through
        // `spawn`; both targets are bounded away from stdio and checked for
        // collisions with the broker and provider-credential capabilities.
        unsafe {
            command.pre_exec(move || {
                if source_fd != target_fd {
                    if libc::dup2(source_fd, target_fd) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::close(source_fd);
                } else {
                    let flags = libc::fcntl(target_fd, libc::F_GETFD);
                    if flags < 0
                        || libc::fcntl(target_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if readiness_source_fd != readiness_target_fd {
                    if libc::dup2(readiness_source_fd, readiness_target_fd) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::close(readiness_source_fd);
                } else {
                    let flags = libc::fcntl(readiness_target_fd, libc::F_GETFD);
                    if flags < 0
                        || libc::fcntl(
                            readiness_target_fd,
                            libc::F_SETFD,
                            flags & !libc::FD_CLOEXEC,
                        ) < 0
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }

        // The reservation prevents endpoint substitution while launch inputs
        // and private capabilities are prepared. Release it only at the final
        // spawn boundary. The pinned native process must bind this exact port
        // and prove that bind through the private readiness capability before
        // C4OS sends any authenticated network request.
        drop(listener);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                drop(sdk_binding);
                drop(self.provider_credential_binding.take());
                return Err(CommandFailureCode::SpawnRejected);
            }
        };
        // Close the parent's duplicate of the worker capability immediately
        // after the exact launcher inherits it. The issuer endpoint is the
        // only retained C4OS credential-delivery capability.
        drop(self.provider_credential_binding.take());
        drop(child_credential_channel);
        drop(child_readiness_channel);
        let process_group_id = child.id();
        let mut managed = ManagedOpenCodeProcess {
            child,
            process_group_id,
        };
        let lease = match self.credentials.lease(
            &launch.secret_channel.reference,
            "opencode-native-server-password",
        ) {
            Ok(lease) => lease,
            Err(_) => {
                return Err(self.cleanup_failed_spawn(
                    managed,
                    broker,
                    CommandFailureCode::SecretChannelUnavailable,
                ));
            }
        };
        let delivery = lease.deliver_to(&mut credential_channel).and_then(|_| {
            credential_channel
                .shutdown(Shutdown::Write)
                .map_err(|_| CredentialVaultError::OperationChannel)
        });
        drop(credential_channel);
        if delivery.is_err() {
            return Err(self.cleanup_failed_spawn(
                managed,
                broker,
                CommandFailureCode::SecretChannelUnavailable,
            ));
        }
        // The launcher cannot start OpenCode until the server password above
        // arrives. Deliver the optional test-only trust descriptor only after
        // that boundary, so its bounded socket cannot deadlock startup before
        // the plugin synchronously consumes it. No descriptor content enters
        // argv, environment values, or the isolated filesystem.
        if sdk_binding
            .deliver_test_tls_trust(self.startup_timeout)
            .is_err()
        {
            return Err(self.cleanup_failed_spawn(
                managed,
                broker,
                CommandFailureCode::SpawnRejected,
            ));
        }
        // Close C4OS's worker endpoints after the launcher inherited them and
        // the optional one-use trust descriptor was delivered. The retained
        // broker below is the only parent-side SDK endpoint.
        drop(sdk_binding);
        let startup_deadline = Instant::now() + self.startup_timeout;
        if wait_for_native_ready(&mut managed, &mut readiness_channel, startup_deadline).is_err() {
            return Err(self.cleanup_failed_spawn(
                managed,
                broker,
                CommandFailureCode::SpawnRejected,
            ));
        }
        if wait_for_authenticated_health(
            &mut managed,
            endpoint,
            &self.credentials,
            &launch.secret_channel.reference,
            startup_deadline,
        )
        .is_err()
        {
            return Err(self.cleanup_failed_spawn(
                managed,
                broker,
                CommandFailureCode::SpawnRejected,
            ));
        }

        let handle = ProcessHandle {
            process_id: process_group_id,
            process_generation: self.process_generation,
        };
        self.process = Some(managed);
        self.broker = Some(broker);
        Ok(handle)
    }
}

impl CommandDriver for OpenCodeNativeCommandDriver {
    fn spawn(&mut self, command: &LaunchCommand) -> Result<ProcessHandle, CommandFailureCode> {
        self.spawn_inner(command)
    }

    fn terminate_process_group(
        &mut self,
        process: &ProcessHandle,
    ) -> Result<(), CommandFailureCode> {
        let managed = self
            .process
            .as_ref()
            .filter(|managed| {
                managed.process_group_id == process.process_id
                    && process.process_generation == self.process_generation
            })
            .ok_or(CommandFailureCode::TerminationFailed)?;
        if managed.process_group_id == 0 {
            return Err(CommandFailureCode::TerminationFailed);
        }
        let shutdown_timeout = self.shutdown_timeout;
        terminate_owned_process_with(
            &mut self.process,
            &mut self.broker,
            CommandFailureCode::TerminationFailed,
            |managed| {
                terminate_process_group(managed, shutdown_timeout)
                    .map_err(|_| CommandFailureCode::TerminationFailed)
            },
        )
    }

    fn authorize_provider_credential_attempt(
        &mut self,
        request: ProviderCredentialRequest,
    ) -> Result<Option<ProviderCredentialAuthorizationReceipt>, CommandFailureCode> {
        if self.process.is_none() {
            return Err(CommandFailureCode::ProviderCredentialUnavailable);
        }
        self.provider_credentials
            .authorize_attempt(request)
            .map(Some)
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)
    }

    fn revoke_provider_credential_attempt(
        &mut self,
        request: &ProviderCredentialRequest,
    ) -> Result<(), CommandFailureCode> {
        self.provider_credentials
            .revoke_attempt(request)
            .map(|_| ())
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)
    }

    fn revoke_all_provider_credential_attempts(&mut self) -> Result<(), CommandFailureCode> {
        self.provider_credentials
            .revoke_all_attempts()
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)
    }

    fn register_provider_credential_route(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<(), CommandFailureCode> {
        self.register_provider(profile)
            .map_err(|_| CommandFailureCode::ProviderCredentialUnavailable)
    }
}

impl fmt::Debug for OpenCodeNativeCommandDriver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenCodeNativeCommandDriver")
            .field("node_executable", &self.node_executable)
            .field("launcher", &self.launcher)
            .field("pinned_native", &self.pinned_native)
            .field("sdk_sidecar_root", &self.sdk_sidecar_root)
            .field("credentials", &"<opaque>")
            .field("process_generation", &self.process_generation)
            .field(
                "process_id",
                &self
                    .process
                    .as_ref()
                    .map(|process| process.process_group_id),
            )
            .field("broker_attached", &self.broker_is_attached())
            .field("listener_reserved", &self.reserved_listener.is_some())
            .finish()
    }
}

impl Drop for OpenCodeNativeCommandDriver {
    fn drop(&mut self) {
        if let Some(mut descriptor) = self.test_tls_trust_descriptor.take() {
            descriptor.fill(0);
        }
        self.broker.take();
        if let Some(mut process) = self.process.take() {
            let _ = terminate_process_group(&mut process, self.shutdown_timeout);
        }
    }
}

pub fn opencode_authority_configuration_sha256() -> String {
    sha256_bytes(OPENCODE_AUTHORITY_CONFIG.as_bytes())
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        use fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded
}

fn valid_launch_command(launch: &LaunchCommand) -> bool {
    let environment_keys = launch
        .environment
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_environment = BTreeSet::from([
        "NO_PROXY",
        "TMPDIR",
        "XDG_CACHE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
    ]);
    valid_absolute_path(&launch.executable)
        && valid_absolute_path(&launch.working_directory)
        && launch.working_directory.is_dir()
        && valid_sha256(&launch.expected_binary_sha256)
        && launch.arguments.len() == 5
        && launch.arguments[0] == "serve"
        && launch.arguments[1] == "--hostname"
        && launch.arguments[3] == "--port"
        && environment_keys == expected_environment
        && launch.environment.get("NO_PROXY").map(String::as_str) == Some("127.0.0.1,::1,localhost")
        && isolated_namespace_root(launch).is_some()
        && launch.secret_channel.inherited_fd >= MIN_SECRET_DESCRIPTOR
        && launch.secret_channel.inherited_fd <= MAX_SECRET_DESCRIPTOR
        && launch.authority_policy.permission_default() == NativePermissionDefault::Deny
        && launch
            .authority_policy
            .allowed_tool_proposals()
            .iter()
            .map(String::as_str)
            .eq(OPENCODE_C4OS_TOOL_IDS)
        && !launch.authority_policy.remember_native_decisions()
}

fn prepare_isolated_directories(launch: &LaunchCommand) -> Result<(), CommandFailureCode> {
    let namespace_root =
        isolated_namespace_root(launch).ok_or(CommandFailureCode::SpawnRejected)?;
    let runtimes_root = namespace_root
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "runtimes"))
        .ok_or(CommandFailureCode::SpawnRejected)?;
    let c4os_home = runtimes_root
        .parent()
        .filter(|parent| parent.is_dir())
        .ok_or(CommandFailureCode::SpawnRejected)?;

    // Create each managed component one level at a time so an existing symlink
    // cannot redirect the version/workspace/generation namespace.
    let mut current = c4os_home.to_path_buf();
    for component in runtimes_root
        .strip_prefix(c4os_home)
        .ok()
        .into_iter()
        .flat_map(Path::components)
        .chain(
            namespace_root
                .strip_prefix(runtimes_root)
                .ok()
                .into_iter()
                .flat_map(Path::components),
        )
    {
        let Component::Normal(component) = component else {
            return Err(CommandFailureCode::SpawnRejected);
        };
        current.push(component);
        secure_directory(&current)?;
    }

    let canonical_root = namespace_root
        .canonicalize()
        .map_err(|_| CommandFailureCode::SpawnRejected)?;
    for (key, expected_name) in [
        ("TMPDIR", "tmp"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
    ] {
        let path = Path::new(
            launch
                .environment
                .get(key)
                .ok_or(CommandFailureCode::SpawnRejected)?,
        );
        if path.file_name().is_none_or(|name| name != expected_name)
            || path.parent() != Some(namespace_root.as_path())
        {
            return Err(CommandFailureCode::SpawnRejected);
        }
        secure_directory(path)?;
        let canonical = path
            .canonicalize()
            .map_err(|_| CommandFailureCode::SpawnRejected)?;
        if canonical.parent() != Some(canonical_root.as_path()) {
            return Err(CommandFailureCode::SpawnRejected);
        }
    }
    Ok(())
}

fn isolated_namespace_root(launch: &LaunchCommand) -> Option<PathBuf> {
    let paths = [
        ("TMPDIR", "tmp"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
    ]
    .map(|(key, expected_name)| {
        let path = Path::new(launch.environment.get(key)?);
        (valid_absolute_path(path) && path.file_name()? == expected_name)
            .then(|| path.parent().map(Path::to_path_buf))
            .flatten()
    });
    let root = paths.first()?.clone()?;
    if paths
        .into_iter()
        .any(|candidate| candidate != Some(root.clone()))
    {
        return None;
    }
    let runtimes_root = root
        .ancestors()
        .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "runtimes"))?;
    let components = root
        .strip_prefix(runtimes_root)
        .ok()?
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();
    (components.len() == 7
        && components[0] == "opencode"
        && components[1] == OPENCODE_NATIVE_VERSION
        && components[2] == "workspaces"
        && !components[3].is_empty()
        && components[4] == "generations"
        && components[5]
            .parse::<u64>()
            .is_ok_and(|generation| generation != 0)
        && !components[6].is_empty())
    .then_some(root)
}

fn secure_directory(path: &Path) -> Result<(), CommandFailureCode> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(CommandFailureCode::SpawnRejected);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path).map_err(|_| CommandFailureCode::SpawnRejected)?;
        }
        Err(_) => return Err(CommandFailureCode::SpawnRejected),
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| CommandFailureCode::SpawnRejected)
}

fn endpoint_from_launch(launch: &LaunchCommand) -> Result<SocketAddr, CommandFailureCode> {
    let address = launch.arguments[2]
        .parse::<IpAddr>()
        .ok()
        .filter(IpAddr::is_loopback)
        .ok_or(CommandFailureCode::SpawnRejected)?;
    let port = launch.arguments[4]
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or(CommandFailureCode::SpawnRejected)?;
    Ok(SocketAddr::new(address, port))
}

fn descriptor_channels_are_distinct(
    secret_source_fd: i32,
    secret_target_fd: i32,
    readiness_source_fd: i32,
    readiness_target_fd: i32,
    broker_fd: u32,
    provider_fd: u32,
    test_tls_trust_fd: Option<u32>,
) -> bool {
    let Ok(secret_target) = u32::try_from(secret_target_fd) else {
        return false;
    };
    let Ok(readiness_target) = u32::try_from(readiness_target_fd) else {
        return false;
    };
    let mut descriptors = BTreeSet::from([secret_target, readiness_target, broker_fd, provider_fd]);
    if let Some(test_tls_trust_fd) = test_tls_trust_fd {
        descriptors.insert(test_tls_trust_fd);
    }
    descriptors.len() == 4 + usize::from(test_tls_trust_fd.is_some())
        && secret_source_fd != readiness_target_fd
        && readiness_source_fd != secret_target_fd
}

fn wait_for_native_ready(
    process: &mut ManagedOpenCodeProcess,
    readiness: &mut UnixStream,
    deadline: Instant,
) -> Result<(), ()> {
    if process.child.try_wait().map_err(|_| ())?.is_some() {
        return Err(());
    }
    read_native_ready(readiness, deadline)?;
    if process.child.try_wait().map_err(|_| ())?.is_some() {
        return Err(());
    }
    Ok(())
}

fn read_native_ready(readiness: &mut UnixStream, deadline: Instant) -> Result<(), ()> {
    wait_until_readable(readiness.as_raw_fd(), deadline)?;
    let mut signal = [0_u8; 2];
    let count = readiness.read(&mut signal).map_err(|_| ())?;
    if count != 1 || signal[0] != OPENCODE_READY_SIGNAL {
        return Err(());
    }
    readiness.set_nonblocking(true).map_err(|_| ())?;
    match readiness.read(&mut signal[1..]) {
        Ok(0) => {}
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
        Ok(_) | Err(_) => return Err(()),
    }
    Ok(())
}

fn wait_until_readable(descriptor: i32, deadline: Instant) -> Result<(), ()> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(());
        }
        let timeout_ms = remaining.as_millis().clamp(1, i32::MAX as u128) as i32;
        let mut poll = libc::pollfd {
            fd: descriptor,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: `poll` receives one initialized descriptor record and a
        // bounded millisecond timeout; the readiness stream owns the fd.
        let result = unsafe { libc::poll(&mut poll, 1, timeout_ms) };
        if result > 0 && poll.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
            return Ok(());
        }
        if result == 0 {
            return Err(());
        }
        if result < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
            continue;
        }
        return Err(());
    }
}

fn wait_for_authenticated_health(
    process: &mut ManagedOpenCodeProcess,
    endpoint: SocketAddr,
    credentials: &VaultCredentialResolver,
    password_reference: &RandomSecretReference,
    deadline: Instant,
) -> Result<(), ()> {
    loop {
        if process.child.try_wait().map_err(|_| ())?.is_some() {
            return Err(());
        }
        let loopback = LoopbackEndpoint::new(endpoint.ip(), endpoint.port()).map_err(|_| ())?;
        let mut transport = LoopbackHttpTransport::new(
            loopback.clone(),
            credentials.clone(),
            Duration::from_millis(100),
            Duration::from_millis(500),
        )
        .map_err(|_| ())?;
        let response = transport.execute(TransportRequest {
            method: HttpMethod::Get,
            base_url: loopback.base_url(),
            path: "/global/health".into(),
            body: None,
            auth: TransportAuth::Basic {
                username: "opencode".into(),
                password_reference: password_reference.clone(),
            },
            maximum_response_bytes: 4 * 1024,
        });
        let healthy = response.ok().is_some_and(|response| {
            response.status == 200
                && serde_json::from_slice::<Value>(&response.body)
                    .ok()
                    .is_some_and(|health| {
                        health.get("healthy").and_then(Value::as_bool) == Some(true)
                            && health.get("version").and_then(Value::as_str)
                                == Some(OPENCODE_NATIVE_VERSION)
                    })
        });
        if healthy {
            let tools = transport.execute(TransportRequest {
                method: HttpMethod::Get,
                base_url: loopback.base_url(),
                path: "/experimental/tool/ids".into(),
                body: None,
                auth: TransportAuth::Basic {
                    username: "opencode".into(),
                    password_reference: password_reference.clone(),
                },
                maximum_response_bytes: 32 * 1024,
            });
            if tools.ok().is_some_and(|response| {
                if response.status != 200 {
                    return false;
                }
                let Ok(tools) = serde_json::from_slice::<Vec<String>>(&response.body) else {
                    return false;
                };
                let mut c4os_tools = tools
                    .iter()
                    .map(String::as_str)
                    .filter(|tool| tool.starts_with("c4os_"))
                    .collect::<Vec<_>>();
                c4os_tools.sort_unstable();
                c4os_tools == OPENCODE_C4OS_TOOL_IDS
            }) {
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            return Err(());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_process_group(
    process: &mut ManagedOpenCodeProcess,
    timeout: Duration,
) -> std::io::Result<()> {
    signal_process_group(process.process_group_id, libc::SIGTERM)?;
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let _ = process.child.try_wait()?;
        if !process_group_exists(process.process_group_id)? {
            let _ = process.child.wait();
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    signal_process_group(process.process_group_id, libc::SIGKILL)?;
    let kill_deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < kill_deadline && process_group_exists(process.process_group_id)? {
        thread::sleep(Duration::from_millis(10));
    }
    let _ = process.child.wait();
    (!process_group_exists(process.process_group_id)?)
        .then_some(())
        .ok_or_else(|| std::io::Error::other("OpenCode process group survived cleanup"))
}

/// Temporarily transfers process and broker ownership into one cleanup
/// transaction. Failed cleanup restores both slots so the caller can retry and
/// the driver's `Drop` implementation still has an owned process to reap.
fn terminate_owned_process_with<P, B, E, F>(
    process: &mut Option<P>,
    broker: &mut Option<B>,
    unavailable: E,
    terminate: F,
) -> Result<(), E>
where
    F: FnOnce(&mut P) -> Result<(), E>,
{
    let Some(mut owned_process) = process.take() else {
        return Err(unavailable);
    };
    let owned_broker = broker.take();
    match terminate(&mut owned_process) {
        Ok(()) => Ok(()),
        Err(error) => {
            *process = Some(owned_process);
            *broker = owned_broker;
            Err(error)
        }
    }
}

fn signal_process_group(process_group_id: u32, signal: i32) -> std::io::Result<()> {
    let process_group_id = i32::try_from(process_group_id)
        .map_err(|_| std::io::Error::other("invalid process group"))?;
    // SAFETY: `kill` receives a validated process-group identifier and a
    // constant signal; it neither dereferences pointers nor retains memory.
    if unsafe { libc::kill(-process_group_id, signal) } == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

fn process_group_exists(process_group_id: u32) -> std::io::Result<bool> {
    let process_group_id = i32::try_from(process_group_id)
        .map_err(|_| std::io::Error::other("invalid process group"))?;
    // SAFETY: signal zero performs only an existence/permission check for the
    // validated process-group identifier.
    if unsafe { libc::kill(-process_group_id, 0) } == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error),
    }
}

fn read_http_response(
    stream: &mut TcpStream,
    maximum_response_bytes: usize,
    io_timeout: Duration,
) -> Result<TransportResponse, TransportFailureCode> {
    let maximum_wire_bytes = maximum_response_bytes
        .checked_mul(8)
        .and_then(|value| value.checked_add(MAX_HTTP_HEADER_BYTES))
        .ok_or(TransportFailureCode::Protocol)?;
    let deadline = Instant::now() + io_timeout;
    let mut wire = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        if let Some(response) = try_parse_http_response(&wire, maximum_response_bytes, false)? {
            return Ok(response);
        }
        if wire.len() >= maximum_wire_bytes {
            return Err(TransportFailureCode::Protocol);
        }
        if Instant::now() >= deadline {
            return Err(TransportFailureCode::Timeout);
        }
        match stream.read(&mut buffer) {
            Ok(0) => {
                return try_parse_http_response(&wire, maximum_response_bytes, true)?
                    .ok_or(TransportFailureCode::Protocol);
            }
            Ok(read) => {
                if wire.len().saturating_add(read) > maximum_wire_bytes {
                    return Err(TransportFailureCode::Protocol);
                }
                wire.extend_from_slice(&buffer[..read]);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                return Err(TransportFailureCode::Timeout);
            }
            Err(_) => return Err(TransportFailureCode::Unavailable),
        }
    }
}

fn try_parse_http_response(
    wire: &[u8],
    maximum_response_bytes: usize,
    eof: bool,
) -> Result<Option<TransportResponse>, TransportFailureCode> {
    let Some(header_end) = find_bytes(wire, b"\r\n\r\n") else {
        if wire.len() > MAX_HTTP_HEADER_BYTES || eof {
            return Err(TransportFailureCode::Protocol);
        }
        return Ok(None);
    };
    if header_end > MAX_HTTP_HEADER_BYTES {
        return Err(TransportFailureCode::Protocol);
    }
    let header =
        std::str::from_utf8(&wire[..header_end]).map_err(|_| TransportFailureCode::Protocol)?;
    let mut lines = header.split("\r\n");
    let mut status_parts = lines
        .next()
        .ok_or(TransportFailureCode::Protocol)?
        .split_ascii_whitespace();
    if !matches!(status_parts.next(), Some("HTTP/1.1" | "HTTP/1.0")) {
        return Err(TransportFailureCode::Protocol);
    }
    let status = status_parts
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|status| (100..=599).contains(status))
        .ok_or(TransportFailureCode::Protocol)?;
    let mut content_length = None;
    let mut chunked = false;
    let mut transfer_encoding_seen = false;
    let mut header_count = 0usize;
    for line in lines {
        header_count = header_count.saturating_add(1);
        if header_count > MAX_HTTP_HEADERS || line.starts_with(' ') || line.starts_with('\t') {
            return Err(TransportFailureCode::Protocol);
        }
        let (name, value) = line.split_once(':').ok_or(TransportFailureCode::Protocol)?;
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || value
                .bytes()
                .any(|byte| !byte.is_ascii() || byte.is_ascii_control())
        {
            return Err(TransportFailureCode::Protocol);
        }
        if name == "content-length" {
            if content_length.is_some() {
                return Err(TransportFailureCode::Protocol);
            }
            content_length = Some(
                value
                    .parse::<usize>()
                    .ok()
                    .filter(|length| *length <= maximum_response_bytes)
                    .ok_or(TransportFailureCode::Protocol)?,
            );
        } else if name == "transfer-encoding" {
            if transfer_encoding_seen || !value.eq_ignore_ascii_case("chunked") {
                return Err(TransportFailureCode::Protocol);
            }
            transfer_encoding_seen = true;
            chunked = true;
        } else if name == "content-encoding" && !value.eq_ignore_ascii_case("identity") {
            return Err(TransportFailureCode::Protocol);
        }
    }
    if content_length.is_some() && chunked {
        return Err(TransportFailureCode::Protocol);
    }

    let body_start = header_end + 4;
    let available = &wire[body_start..];
    let body = if let Some(content_length) = content_length {
        if available.len() < content_length {
            if eof {
                return Err(TransportFailureCode::Protocol);
            }
            return Ok(None);
        }
        available[..content_length].to_vec()
    } else if chunked {
        match decode_chunked(available, maximum_response_bytes)? {
            Some(body) => body,
            None if eof => return Err(TransportFailureCode::Protocol),
            None => return Ok(None),
        }
    } else {
        if !eof {
            return Ok(None);
        }
        if available.len() > maximum_response_bytes {
            return Err(TransportFailureCode::Protocol);
        }
        available.to_vec()
    };
    Ok(Some(TransportResponse { status, body }))
}

fn decode_chunked(
    bytes: &[u8],
    maximum_response_bytes: usize,
) -> Result<Option<Vec<u8>>, TransportFailureCode> {
    let mut body = Vec::new();
    let mut position = 0usize;
    loop {
        let Some(relative_line_end) = find_bytes(&bytes[position..], b"\r\n") else {
            return Ok(None);
        };
        let line_end = position + relative_line_end;
        let size_text = std::str::from_utf8(&bytes[position..line_end])
            .map_err(|_| TransportFailureCode::Protocol)?;
        let size = usize::from_str_radix(size_text.split(';').next().unwrap_or_default(), 16)
            .map_err(|_| TransportFailureCode::Protocol)?;
        let data_start = line_end + 2;
        if size == 0 {
            if bytes.get(data_start..data_start + 2) == Some(b"\r\n") {
                return Ok(Some(body));
            }
            return Ok(find_bytes(&bytes[data_start..], b"\r\n\r\n").map(|_| body));
        }
        let data_end = data_start
            .checked_add(size)
            .ok_or(TransportFailureCode::Protocol)?;
        let chunk_end = data_end
            .checked_add(2)
            .ok_or(TransportFailureCode::Protocol)?;
        if chunk_end > bytes.len() {
            return Ok(None);
        }
        if bytes.get(data_end..chunk_end) != Some(b"\r\n")
            || body.len().saturating_add(size) > maximum_response_bytes
        {
            return Err(TransportFailureCode::Protocol);
        }
        body.extend_from_slice(&bytes[data_start..data_end]);
        position = chunk_end;
    }
}

fn base64_encode(bytes: &[u8]) -> Vec<u8> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = Vec::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        encoded.push(TABLE[usize::from(chunk[0] >> 2)]);
        encoded.push(
            TABLE[usize::from(
                ((chunk[0] & 0b0000_0011) << 4) | chunk.get(1).copied().unwrap_or(0) >> 4,
            )],
        );
        encoded.push(if chunk.len() > 1 {
            TABLE[usize::from(
                ((chunk[1] & 0b0000_1111) << 2) | chunk.get(2).copied().unwrap_or(0) >> 6,
            )]
        } else {
            b'='
        });
        encoded.push(if chunk.len() > 2 {
            TABLE[usize::from(chunk[2] & 0b0011_1111)]
        } else {
            b'='
        });
    }
    encoded
}

fn valid_http_path(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= MAX_HTTP_PATH_BYTES
        && !path.contains("..")
        && !path
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
}

fn valid_basic_username(username: &str) -> bool {
    !username.is_empty()
        && username.len() <= 128
        && !username
            .bytes()
            .any(|byte| byte == b':' || byte.is_ascii_control() || !byte.is_ascii())
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn host_header(endpoint: &LoopbackEndpoint) -> String {
    match endpoint.address() {
        IpAddr::V4(address) => format!("{address}:{}", endpoint.port()),
        IpAddr::V6(address) => format!("[{address}]:{}", endpoint.port()),
    }
}

fn method_name(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
    }
}

fn map_connect_error(error: std::io::Error) -> TransportFailureCode {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            TransportFailureCode::Timeout
        }
        _ => TransportFailureCode::Unavailable,
    }
}

fn map_io_error(error: std::io::Error) -> TransportFailureCode {
    map_connect_error(error)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::time::{Duration, Instant};

    use super::{
        descriptor_channels_are_distinct, read_native_ready, terminate_owned_process_with,
    };

    #[test]
    fn inherited_readiness_descriptor_is_collision_checked_before_spawn() {
        assert!(descriptor_channels_are_distinct(
            12, 198, 13, 199, 64, 256, None
        ));
        assert!(!descriptor_channels_are_distinct(
            199, 198, 13, 199, 64, 256, None
        ));
        assert!(!descriptor_channels_are_distinct(
            12, 198, 198, 199, 64, 256, None
        ));
        assert!(!descriptor_channels_are_distinct(
            12,
            198,
            13,
            199,
            64,
            256,
            Some(64)
        ));
        assert!(!descriptor_channels_are_distinct(
            12, 199, 13, 199, 64, 256, None
        ));
        assert!(!descriptor_channels_are_distinct(
            12, 198, 13, 199, 199, 256, None
        ));
        assert!(!descriptor_channels_are_distinct(
            12, 198, 13, 199, 64, 199, None
        ));
    }

    #[test]
    fn readiness_requires_one_exact_signal_without_waiting_for_eof() {
        for bytes in [Vec::new(), vec![0x00], vec![0x01, 0x01]] {
            let (mut reader, mut writer) = UnixStream::pair().expect("readiness pair");
            writer.write_all(&bytes).expect("readiness fixture");
            drop(writer);
            assert_eq!(
                read_native_ready(&mut reader, Instant::now() + Duration::from_secs(1)),
                Err(())
            );
        }

        let (mut reader, mut writer) = UnixStream::pair().expect("readiness pair");
        writer.write_all(&[0x01]).expect("readiness fixture");
        drop(writer);
        read_native_ready(&mut reader, Instant::now() + Duration::from_secs(1))
            .expect("one signal is accepted");

        let (mut reader, mut writer) = UnixStream::pair().expect("readiness pair");
        writer.write_all(&[0x01]).expect("readiness fixture");
        read_native_ready(&mut reader, Instant::now() + Duration::from_secs(1))
            .expect("one signal does not depend on descendant EOF timing");
    }

    #[test]
    fn failed_termination_restores_process_and_broker_for_retry() {
        let mut process = Some(41_u64);
        let mut broker = Some("broker-capability");

        let first = terminate_owned_process_with(
            &mut process,
            &mut broker,
            "missing ownership",
            |owned_process| {
                *owned_process += 1;
                Err("injected cleanup failure")
            },
        );

        assert_eq!(first, Err("injected cleanup failure"));
        assert_eq!(process, Some(42));
        assert_eq!(broker, Some("broker-capability"));

        terminate_owned_process_with(&mut process, &mut broker, "missing ownership", |_| Ok(()))
            .expect("retry consumes retained ownership after cleanup succeeds");
        assert_eq!(process, None);
        assert_eq!(broker, None);
    }
}
