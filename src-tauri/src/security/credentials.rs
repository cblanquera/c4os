//! Rust-authoritative credential isolation for C4OS.
//!
//! The vault intentionally exposes no API that returns a stored secret, builds
//! command-line arguments, or populates an inherited environment. A caller can
//! only deliver one credential to an operation-scoped writer through a
//! revocable, expiring lease.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use thiserror::Error;
use uuid::Uuid;
use zeroize::Zeroizing;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const VAULT_MAGIC: &[u8; 12] = b"C4OS-VAULT\0\0";
const VAULT_SCHEMA_VERSION: u16 = 1;
const INSTALLATION_KEY_BYTES: usize = 32;
const PASSWORD_SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 24;
const MAX_VAULT_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_CREDENTIALS: usize = 4_096;
const MAX_SECRET_BYTES: usize = 64 * 1024;
const MAX_KIND_BYTES: usize = 128;
const MAX_OPERATION_BYTES: usize = 256;
const MAX_LEASE_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_REAUTHENTICATION_TTL: Duration = Duration::from_secs(5 * 60);
const PASSWORD_MEMORY_KIB: u32 = 65_536;
const PASSWORD_ITERATIONS: u32 = 3;
const PASSWORD_LANES: u32 = 1;

pub type CredentialVaultResult<T> = Result<T, CredentialVaultError>;

/// Errors never include credential values, derived keys, or plaintext payloads.
#[derive(Debug, Error)]
pub enum CredentialVaultError {
    #[error("the operating-system credential service is unavailable")]
    KeychainUnavailable,
    #[error("the operating-system credential service returned an invalid installation key")]
    InvalidInstallationKey,
    #[error("the credential vault has an unsupported or malformed format")]
    CorruptVault,
    #[error("credential vault authentication failed")]
    AuthenticationFailed,
    #[error("the selected unlock method does not match this credential vault")]
    ProtectionModeMismatch,
    #[error("session-only credentials cannot be persisted")]
    SessionOnlyPersistenceForbidden,
    #[error("the credential reference is unknown or inactive")]
    CredentialNotFound,
    #[error("the credential value or kind is outside the accepted bounds")]
    InvalidCredential,
    #[error("the credential vault reached its bounded record capacity")]
    CapacityExceeded,
    #[error("credential import requires recent reauthentication")]
    ReauthenticationRequired,
    #[error("credential reauthentication failed")]
    ReauthenticationFailed,
    #[error("the credential reauthentication grant expired")]
    ReauthenticationExpired,
    #[error("the credential reauthentication grant was already consumed")]
    ReauthenticationConsumed,
    #[error("the credential operation lease expired")]
    LeaseExpired,
    #[error("the credential operation lease was revoked")]
    LeaseRevoked,
    #[error("the credential operation lease belongs to another vault")]
    LeaseVaultMismatch,
    #[error("the requested lease lifetime is outside the accepted bounds")]
    InvalidLeaseLifetime,
    #[error("credential delivery to the operation channel failed")]
    OperationChannel,
    #[error("a raw credential was detected on a forbidden surface")]
    SecretLeakDetected,
    #[error("credential vault entropy is unavailable")]
    EntropyUnavailable,
    #[error("credential vault state is unavailable")]
    StateUnavailable,
    #[error("credential vault storage failed during {operation}")]
    Storage {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("credential vault serialization failed")]
    Serialization,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultProtection {
    InstallationKey,
    PasswordProtected,
    SessionOnly,
}

/// An opaque identifier safe to persist in product records and ordinary exports.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CredentialReference(String);

impl CredentialReference {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CredentialReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CredentialReference")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for CredentialReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Key bytes are zeroized on drop and never rendered by `Debug`.
pub struct InstallationKey(Zeroizing<[u8; INSTALLATION_KEY_BYTES]>);

impl InstallationKey {
    pub fn from_bytes(bytes: [u8; INSTALLATION_KEY_BYTES]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    fn from_slice(bytes: &[u8]) -> CredentialVaultResult<Self> {
        if bytes.len() != INSTALLATION_KEY_BYTES {
            return Err(CredentialVaultError::InvalidInstallationKey);
        }
        let mut key = Zeroizing::new([0_u8; INSTALLATION_KEY_BYTES]);
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }

    fn as_bytes(&self) -> &[u8; INSTALLATION_KEY_BYTES] {
        &self.0
    }
}

impl fmt::Debug for InstallationKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("InstallationKey([REDACTED])")
    }
}

/// The production implementation is backed by the macOS credential service;
/// tests provide an in-memory implementation and never touch a user's keychain.
pub trait InstallationKeyStore: Send + Sync {
    fn load_or_create(&self) -> CredentialVaultResult<InstallationKey>;
}

/// A bridge to platform-local user-presence or password reauthentication.
pub trait ReauthenticationProvider: Send + Sync {
    fn reauthenticate(&self, purpose: ReauthenticationPurpose)
    -> Result<(), ReauthenticationError>;
}

#[derive(Clone, Copy, Debug, Error)]
#[error("credential reauthentication was not satisfied")]
pub struct ReauthenticationError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReauthenticationPurpose {
    ImportCredential,
}

pub trait MonotonicClock: Send + Sync {
    fn now(&self) -> Duration;
}

pub trait EntropySource: Send + Sync {
    fn fill(&self, destination: &mut [u8]) -> CredentialVaultResult<()>;
}

pub struct SystemMonotonicClock {
    origin: Instant,
}

impl Default for SystemMonotonicClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}

#[derive(Default)]
pub struct OsEntropy;

impl EntropySource for OsEntropy {
    fn fill(&self, destination: &mut [u8]) -> CredentialVaultResult<()> {
        getrandom::fill(destination).map_err(|_| CredentialVaultError::EntropyUnavailable)
    }
}

/// macOS Keychain adapter for the one installation master key.
#[cfg(target_os = "macos")]
pub struct MacOsInstallationKeyStore {
    service: String,
    account: String,
}

#[cfg(target_os = "macos")]
impl MacOsInstallationKeyStore {
    pub fn new(bundle_identifier: impl Into<String>) -> Self {
        Self {
            service: format!("{}.credential-vault", bundle_identifier.into()),
            account: "installation-master-key-v1".into(),
        }
    }

    fn read(&self) -> Result<Vec<u8>, security_framework::base::Error> {
        use security_framework::passwords::{PasswordOptions, generic_password};

        generic_password(PasswordOptions::new_generic_password(
            &self.service,
            &self.account,
        ))
    }
}

#[cfg(target_os = "macos")]
impl InstallationKeyStore for MacOsInstallationKeyStore {
    fn load_or_create(&self) -> CredentialVaultResult<InstallationKey> {
        // Apple's errSecItemNotFound OSStatus. Other errors must not silently
        // become a new key or a plaintext fallback.
        const ERR_SEC_ITEM_NOT_FOUND: i32 = -25_300;

        match self.read() {
            Ok(bytes) => {
                let bytes = Zeroizing::new(bytes);
                InstallationKey::from_slice(&bytes)
            }
            Err(error) if error.code() == ERR_SEC_ITEM_NOT_FOUND => {
                use security_framework::passwords::set_generic_password;

                let mut generated = Zeroizing::new([0_u8; INSTALLATION_KEY_BYTES]);
                OsEntropy.fill(&mut generated[..])?;
                set_generic_password(&self.service, &self.account, &generated[..])
                    .map_err(|_| CredentialVaultError::KeychainUnavailable)?;

                // Read back the authoritative value. This also avoids returning
                // a local value that the credential service did not retain.
                let stored = Zeroizing::new(
                    self.read()
                        .map_err(|_| CredentialVaultError::KeychainUnavailable)?,
                );
                InstallationKey::from_slice(&stored)
            }
            Err(_) => Err(CredentialVaultError::KeychainUnavailable),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CredentialMetadata {
    pub credential_reference: CredentialReference,
    pub kind: String,
    pub created_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CredentialMetadataExport {
    pub schema_version: u16,
    pub vault_id: Uuid,
    pub generation: u64,
    pub credentials: Vec<CredentialMetadata>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretSurface {
    CommandLineArguments,
    BroadEnvironment,
    Diagnostic,
    Export,
    Log,
    RendererState,
    WorkspaceFile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretLeakFinding {
    pub surface: SecretSurface,
    pub direct_secret_matches: usize,
    pub sensitive_field_paths: Vec<String>,
}

impl SecretLeakFinding {
    pub fn is_clean(&self) -> bool {
        self.direct_secret_matches == 0 && self.sensitive_field_paths.is_empty()
    }
}

pub struct CredentialVault {
    inner: Arc<VaultInner>,
}

impl Clone for CredentialVault {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl CredentialVault {
    pub fn open_or_create_with_installation_key(
        path: impl Into<PathBuf>,
        key_store: &dyn InstallationKeyStore,
    ) -> CredentialVaultResult<Self> {
        Self::open_or_create_installation_with_components(
            path.into(),
            key_store,
            Arc::new(SystemMonotonicClock::default()),
            Arc::new(OsEntropy),
        )
    }

    pub fn open_or_create_password(
        path: impl Into<PathBuf>,
        password: &[u8],
    ) -> CredentialVaultResult<Self> {
        Self::open_or_create_password_with_components(
            path.into(),
            password,
            Arc::new(SystemMonotonicClock::default()),
            Arc::new(OsEntropy),
        )
    }

    pub fn session_only() -> CredentialVaultResult<Self> {
        Self::session_only_with_components(
            Arc::new(SystemMonotonicClock::default()),
            Arc::new(OsEntropy),
        )
    }

    pub fn protection(&self) -> VaultProtection {
        self.inner.persistence.protection()
    }

    pub fn vault_id(&self) -> CredentialVaultResult<Uuid> {
        Ok(self.lock_state()?.vault_id)
    }

    pub fn generation(&self) -> CredentialVaultResult<u64> {
        Ok(self.lock_state()?.generation)
    }

    pub fn metadata(&self) -> CredentialVaultResult<Vec<CredentialMetadata>> {
        Ok(self
            .lock_state()?
            .entries
            .values()
            .map(StoredCredential::metadata)
            .collect())
    }

    /// Stores or replaces a secret under a new opaque reference.
    pub fn store(
        &self,
        kind: impl Into<String>,
        secret: &[u8],
    ) -> CredentialVaultResult<CredentialReference> {
        let kind = kind.into();
        validate_credential(&kind, secret)?;

        let mut state = self.lock_state()?;
        if state.entries.len() >= MAX_CREDENTIALS {
            return Err(CredentialVaultError::CapacityExceeded);
        }
        let credential_reference = (0..8)
            .find_map(|_| {
                let candidate = self.new_credential_reference().ok()?;
                (!state.entries.contains_key(&candidate)).then_some(candidate)
            })
            .ok_or(CredentialVaultError::EntropyUnavailable)?;

        let next_generation = state
            .generation
            .checked_add(1)
            .ok_or(CredentialVaultError::CorruptVault)?;
        let mut candidate = state.clone();
        candidate.generation = next_generation;
        candidate.entries.insert(
            credential_reference.clone(),
            StoredCredential {
                credential_reference: credential_reference.clone(),
                kind,
                secret: SecretBytes::new(secret.to_vec()),
                created_generation: next_generation,
                revision: 1,
            },
        );
        self.inner.persist(&candidate)?;
        *state = candidate;
        Ok(credential_reference)
    }

    /// Generates and stores an opaque operation credential without exposing
    /// the random bytes to application or renderer state. This is used for
    /// authenticated loopback runtime channels owned by the Rust core.
    pub fn store_random(
        &self,
        kind: impl Into<String>,
        byte_length: usize,
    ) -> CredentialVaultResult<CredentialReference> {
        if !(32..=MAX_SECRET_BYTES).contains(&byte_length) {
            return Err(CredentialVaultError::InvalidCredential);
        }
        let mut secret = Zeroizing::new(vec![0_u8; byte_length]);
        self.inner.entropy.fill(secret.as_mut_slice())?;
        self.store(kind, secret.as_slice())
    }

    /// Generates random bytes and stores their lowercase hexadecimal encoding.
    /// This preserves the requested entropy while satisfying native boundaries
    /// that require printable text, without constructing an immutable string.
    pub fn store_random_hex(
        &self,
        kind: impl Into<String>,
        byte_length: usize,
    ) -> CredentialVaultResult<CredentialReference> {
        if !(32..=(MAX_SECRET_BYTES / 2)).contains(&byte_length) {
            return Err(CredentialVaultError::InvalidCredential);
        }
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut random = Zeroizing::new(vec![0_u8; byte_length]);
        self.inner.entropy.fill(random.as_mut_slice())?;
        let mut encoded = Zeroizing::new(vec![0_u8; byte_length * 2]);
        for (index, byte) in random.iter().copied().enumerate() {
            encoded[index * 2] = HEX[usize::from(byte >> 4)];
            encoded[index * 2 + 1] = HEX[usize::from(byte & 0x0f)];
        }
        self.store(kind, encoded.as_slice())
    }

    /// Replaces a credential without changing the opaque product reference.
    /// Existing operation leases are revoked immediately after the durable swap.
    pub fn replace(
        &self,
        credential_reference: &CredentialReference,
        secret: &[u8],
    ) -> CredentialVaultResult<u64> {
        if secret.is_empty() || secret.len() > MAX_SECRET_BYTES {
            return Err(CredentialVaultError::InvalidCredential);
        }

        let mut state = self.lock_state()?;
        let current = state
            .entries
            .get(credential_reference)
            .ok_or(CredentialVaultError::CredentialNotFound)?;
        let next_generation = state
            .generation
            .checked_add(1)
            .ok_or(CredentialVaultError::CorruptVault)?;
        let next_revision = current
            .revision
            .checked_add(1)
            .ok_or(CredentialVaultError::CorruptVault)?;

        let mut candidate = state.clone();
        candidate.generation = next_generation;
        let replacement = candidate
            .entries
            .get_mut(credential_reference)
            .ok_or(CredentialVaultError::CredentialNotFound)?;
        replacement.secret = SecretBytes::new(secret.to_vec());
        replacement.revision = next_revision;
        self.inner.persist(&candidate)?;
        *state = candidate;
        self.inner.revoke_reference(credential_reference)?;
        Ok(next_generation)
    }

    /// Removes the only usable copy and revokes every outstanding lease.
    pub fn remove(&self, credential_reference: &CredentialReference) -> CredentialVaultResult<u64> {
        let mut state = self.lock_state()?;
        if !state.entries.contains_key(credential_reference) {
            return Err(CredentialVaultError::CredentialNotFound);
        }
        let next_generation = state
            .generation
            .checked_add(1)
            .ok_or(CredentialVaultError::CorruptVault)?;
        let mut candidate = state.clone();
        candidate.generation = next_generation;
        candidate.entries.remove(credential_reference);
        self.inner.persist(&candidate)?;
        *state = candidate;
        self.inner.revoke_reference(credential_reference)?;
        Ok(next_generation)
    }

    /// Creates a short-lived, single-operation capability. The lease itself
    /// contains no secret bytes.
    pub fn lease_for_operation(
        &self,
        credential_reference: &CredentialReference,
        operation: impl Into<String>,
        ttl: Duration,
    ) -> CredentialVaultResult<OperationCredentialLease> {
        if ttl.is_zero() || ttl > MAX_LEASE_TTL {
            return Err(CredentialVaultError::InvalidLeaseLifetime);
        }
        let operation = operation.into();
        if operation.is_empty() || operation.len() > MAX_OPERATION_BYTES {
            return Err(CredentialVaultError::InvalidCredential);
        }

        let state = self.lock_state()?;
        let credential = state
            .entries
            .get(credential_reference)
            .ok_or(CredentialVaultError::CredentialNotFound)?;
        let lease_id = self.new_uuid()?;
        let revoked = Arc::new(AtomicBool::new(false));
        let expires_at = self
            .inner
            .clock
            .now()
            .checked_add(ttl)
            .ok_or(CredentialVaultError::InvalidLeaseLifetime)?;

        let lease = OperationCredentialLease {
            vault_id: state.vault_id,
            lease_id,
            credential_reference: credential_reference.clone(),
            credential_revision: credential.revision,
            operation,
            expires_at,
            revoked: Arc::clone(&revoked),
            consumed: AtomicBool::new(false),
            inner: Arc::downgrade(&self.inner),
        };
        drop(state);
        self.inner.lock_leases()?.insert(
            lease_id,
            ActiveLease {
                credential_reference: credential_reference.clone(),
                revoked: Arc::downgrade(&revoked),
            },
        );
        Ok(lease)
    }

    /// Reauthentication grants are vault-bound, generation-bound, expiring,
    /// and one-use. Import has no alternate unchecked entry point.
    pub fn authorize_import(
        &self,
        provider: &dyn ReauthenticationProvider,
        ttl: Duration,
    ) -> CredentialVaultResult<ReauthenticationGrant> {
        if ttl.is_zero() || ttl > MAX_REAUTHENTICATION_TTL {
            return Err(CredentialVaultError::ReauthenticationRequired);
        }
        provider
            .reauthenticate(ReauthenticationPurpose::ImportCredential)
            .map_err(|_| CredentialVaultError::ReauthenticationFailed)?;
        let state = self.lock_state()?;
        Ok(ReauthenticationGrant {
            vault_id: state.vault_id,
            generation: state.generation,
            expires_at: self
                .inner
                .clock
                .now()
                .checked_add(ttl)
                .ok_or(CredentialVaultError::ReauthenticationRequired)?,
            consumed: AtomicBool::new(false),
        })
    }

    pub fn import(
        &self,
        grant: &ReauthenticationGrant,
        kind: impl Into<String>,
        secret: &[u8],
    ) -> CredentialVaultResult<CredentialReference> {
        {
            let state = self.lock_state()?;
            if grant.vault_id != state.vault_id || grant.generation != state.generation {
                return Err(CredentialVaultError::ReauthenticationRequired);
            }
            if self.inner.clock.now() >= grant.expires_at {
                return Err(CredentialVaultError::ReauthenticationExpired);
            }
            if grant.consumed.swap(true, Ordering::AcqRel) {
                return Err(CredentialVaultError::ReauthenticationConsumed);
            }
        }
        self.store(kind, secret)
    }

    pub fn export_metadata(&self) -> CredentialVaultResult<CredentialMetadataExport> {
        let state = self.lock_state()?;
        let export = CredentialMetadataExport {
            schema_version: VAULT_SCHEMA_VERSION,
            vault_id: state.vault_id,
            generation: state.generation,
            credentials: state
                .entries
                .values()
                .map(StoredCredential::metadata)
                .collect(),
        };
        let serialized =
            serde_json::to_vec(&export).map_err(|_| CredentialVaultError::Serialization)?;
        let finding = scan_secrets_in_state(&state, SecretSurface::Export, &serialized);
        if !finding.is_clean() {
            return Err(CredentialVaultError::SecretLeakDetected);
        }
        Ok(export)
    }

    pub fn scan_bytes(
        &self,
        surface: SecretSurface,
        bytes: &[u8],
    ) -> CredentialVaultResult<SecretLeakFinding> {
        let state = self.lock_state()?;
        Ok(scan_secrets_in_state(&state, surface, bytes))
    }

    pub fn ensure_clean(&self, surface: SecretSurface, bytes: &[u8]) -> CredentialVaultResult<()> {
        if self.scan_bytes(surface, bytes)?.is_clean() {
            Ok(())
        } else {
            Err(CredentialVaultError::SecretLeakDetected)
        }
    }

    pub fn redact_text(&self, text: &str) -> CredentialVaultResult<String> {
        let state = self.lock_state()?;
        let mut redacted = text.to_owned();
        for entry in state.entries.values() {
            let secret = match std::str::from_utf8(entry.secret.as_slice()) {
                Ok(secret) if !secret.is_empty() => secret,
                _ => continue,
            };
            redacted = redacted.replace(secret, "[REDACTED]");
        }
        Ok(redacted)
    }

    pub fn redact_diagnostic_value(&self, value: &mut Value) -> CredentialVaultResult<()> {
        redact_sensitive_fields(value, "$");
        redact_value_strings(self, value)
    }

    #[doc(hidden)]
    pub fn open_or_create_installation_with_components(
        path: PathBuf,
        key_store: &dyn InstallationKeyStore,
        clock: Arc<dyn MonotonicClock>,
        entropy: Arc<dyn EntropySource>,
    ) -> CredentialVaultResult<Self> {
        let envelope = read_envelope_if_present(&path)?;
        match &envelope {
            Some(envelope) if !matches!(envelope.protection, ProtectionHeader::InstallationKey) => {
                return Err(CredentialVaultError::ProtectionModeMismatch);
            }
            _ => {}
        }
        let installation_key = key_store.load_or_create()?;
        let persistence = Persistence::File {
            path,
            key: SecretKey::from_slice(installation_key.as_bytes())?,
            protection: ProtectionHeader::InstallationKey,
        };
        Self::finish_open(envelope, persistence, clock, entropy)
    }

    #[doc(hidden)]
    pub fn open_or_create_password_with_components(
        path: PathBuf,
        password: &[u8],
        clock: Arc<dyn MonotonicClock>,
        entropy: Arc<dyn EntropySource>,
    ) -> CredentialVaultResult<Self> {
        if password.is_empty() {
            return Err(CredentialVaultError::AuthenticationFailed);
        }
        let envelope = read_envelope_if_present(&path)?;
        let protection = match &envelope {
            Some(envelope) => match &envelope.protection {
                ProtectionHeader::Password(parameters) => {
                    validate_password_parameters(parameters)?;
                    envelope.protection.clone()
                }
                ProtectionHeader::InstallationKey => {
                    return Err(CredentialVaultError::ProtectionModeMismatch);
                }
            },
            None => {
                let mut salt = vec![0_u8; PASSWORD_SALT_BYTES];
                entropy.fill(&mut salt)?;
                ProtectionHeader::Password(PasswordParameters {
                    salt,
                    memory_kib: PASSWORD_MEMORY_KIB,
                    iterations: PASSWORD_ITERATIONS,
                    lanes: PASSWORD_LANES,
                })
            }
        };
        let key = derive_password_key(password, &protection)?;
        let persistence = Persistence::File {
            path,
            key,
            protection,
        };
        Self::finish_open(envelope, persistence, clock, entropy)
    }

    #[doc(hidden)]
    pub fn session_only_with_components(
        clock: Arc<dyn MonotonicClock>,
        entropy: Arc<dyn EntropySource>,
    ) -> CredentialVaultResult<Self> {
        let vault_id = random_uuid(&*entropy)?;
        Ok(Self {
            inner: Arc::new(VaultInner {
                persistence: Persistence::SessionOnly,
                state: Mutex::new(VaultState::empty(vault_id)),
                leases: Mutex::new(HashMap::new()),
                clock,
                entropy,
            }),
        })
    }

    fn finish_open(
        envelope: Option<VaultEnvelope>,
        persistence: Persistence,
        clock: Arc<dyn MonotonicClock>,
        entropy: Arc<dyn EntropySource>,
    ) -> CredentialVaultResult<Self> {
        let state = match envelope {
            Some(envelope) => persistence.decrypt(&envelope)?,
            None => VaultState::empty(random_uuid(&*entropy)?),
        };
        let vault = Self {
            inner: Arc::new(VaultInner {
                persistence,
                state: Mutex::new(state),
                leases: Mutex::new(HashMap::new()),
                clock,
                entropy,
            }),
        };
        if vault.inner.persistence.requires_initial_write() {
            let state = vault.lock_state()?;
            vault.inner.persist(&state)?;
        }
        Ok(vault)
    }

    fn lock_state(&self) -> CredentialVaultResult<std::sync::MutexGuard<'_, VaultState>> {
        self.inner
            .state
            .lock()
            .map_err(|_| CredentialVaultError::StateUnavailable)
    }

    fn new_uuid(&self) -> CredentialVaultResult<Uuid> {
        random_uuid(&*self.inner.entropy)
    }

    fn new_credential_reference(&self) -> CredentialVaultResult<CredentialReference> {
        Ok(CredentialReference(format!(
            "credential:{}",
            self.new_uuid()?.as_simple()
        )))
    }
}

pub struct ReauthenticationGrant {
    vault_id: Uuid,
    generation: u64,
    expires_at: Duration,
    consumed: AtomicBool,
}

impl fmt::Debug for ReauthenticationGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReauthenticationGrant")
            .field("vault_id", &self.vault_id)
            .field("generation", &self.generation)
            .field("expires_at", &self.expires_at)
            .field("consumed", &self.consumed.load(Ordering::Acquire))
            .finish()
    }
}

pub struct OperationCredentialLease {
    vault_id: Uuid,
    lease_id: Uuid,
    credential_reference: CredentialReference,
    credential_revision: u64,
    operation: String,
    expires_at: Duration,
    revoked: Arc<AtomicBool>,
    consumed: AtomicBool,
    inner: Weak<VaultInner>,
}

impl OperationCredentialLease {
    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Delivers the secret only to the supplied operation-local writer. The
    /// writer should be an anonymous pipe or equivalent narrow IPC channel.
    /// Delivery is one-use and holds the vault mutation locks through the
    /// write, so replacement/revocation cannot race the bytes in flight.
    pub fn deliver_to(&self, writer: &mut dyn Write) -> CredentialVaultResult<()> {
        if self.revoked.load(Ordering::Acquire) || self.consumed.load(Ordering::Acquire) {
            return Err(CredentialVaultError::LeaseRevoked);
        }
        let inner = self
            .inner
            .upgrade()
            .ok_or(CredentialVaultError::LeaseRevoked)?;
        let state = inner
            .state
            .lock()
            .map_err(|_| CredentialVaultError::StateUnavailable)?;
        let mut leases = inner.lock_leases()?;
        if inner.clock.now() >= self.expires_at {
            self.revoked.store(true, Ordering::Release);
            leases.remove(&self.lease_id);
            return Err(CredentialVaultError::LeaseExpired);
        }
        if self.revoked.load(Ordering::Acquire)
            || state.vault_id != self.vault_id
            || !leases.contains_key(&self.lease_id)
        {
            return Err(CredentialVaultError::LeaseRevoked);
        }
        let entry = state
            .entries
            .get(&self.credential_reference)
            .ok_or(CredentialVaultError::LeaseRevoked)?;
        if entry.revision != self.credential_revision || self.consumed.swap(true, Ordering::AcqRel)
        {
            return Err(CredentialVaultError::LeaseRevoked);
        }
        leases.remove(&self.lease_id);
        writer
            .write_all(entry.secret.as_slice())
            .map_err(|_| CredentialVaultError::OperationChannel)?;
        writer
            .flush()
            .map_err(|_| CredentialVaultError::OperationChannel)?;
        self.revoked.store(true, Ordering::Release);
        Ok(())
    }

    fn validate(&self) -> CredentialVaultResult<Arc<VaultInner>> {
        if self.revoked.load(Ordering::Acquire) || self.consumed.load(Ordering::Acquire) {
            return Err(CredentialVaultError::LeaseRevoked);
        }
        let inner = self
            .inner
            .upgrade()
            .ok_or(CredentialVaultError::LeaseRevoked)?;
        if inner.clock.now() >= self.expires_at {
            self.revoked.store(true, Ordering::Release);
            return Err(CredentialVaultError::LeaseExpired);
        }
        {
            let state = inner
                .state
                .lock()
                .map_err(|_| CredentialVaultError::StateUnavailable)?;
            let entry = state
                .entries
                .get(&self.credential_reference)
                .ok_or(CredentialVaultError::LeaseRevoked)?;
            if state.vault_id != self.vault_id || entry.revision != self.credential_revision {
                self.revoked.store(true, Ordering::Release);
                return Err(CredentialVaultError::LeaseRevoked);
            }
        }
        Ok(inner)
    }
}

impl fmt::Debug for OperationCredentialLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OperationCredentialLease")
            .field("vault_id", &self.vault_id)
            .field("lease_id", &self.lease_id)
            .field("credential_reference", &self.credential_reference)
            .field("operation", &self.operation)
            .field("expires_at", &self.expires_at)
            .field("revoked", &self.revoked.load(Ordering::Acquire))
            .field("consumed", &self.consumed.load(Ordering::Acquire))
            .finish()
    }
}

impl Drop for OperationCredentialLease {
    fn drop(&mut self) {
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        if let Ok(mut leases) = inner.leases.lock() {
            leases.remove(&self.lease_id);
        }
    }
}

struct VaultInner {
    persistence: Persistence,
    state: Mutex<VaultState>,
    leases: Mutex<HashMap<Uuid, ActiveLease>>,
    clock: Arc<dyn MonotonicClock>,
    entropy: Arc<dyn EntropySource>,
}

impl VaultInner {
    fn persist(&self, state: &VaultState) -> CredentialVaultResult<()> {
        self.persistence.persist(state, &*self.entropy)
    }

    fn lock_leases(
        &self,
    ) -> CredentialVaultResult<std::sync::MutexGuard<'_, HashMap<Uuid, ActiveLease>>> {
        self.leases
            .lock()
            .map_err(|_| CredentialVaultError::StateUnavailable)
    }

    fn revoke_reference(
        &self,
        credential_reference: &CredentialReference,
    ) -> CredentialVaultResult<()> {
        let mut leases = self.lock_leases()?;
        leases.retain(|_, lease| {
            if &lease.credential_reference == credential_reference {
                if let Some(revoked) = lease.revoked.upgrade() {
                    revoked.store(true, Ordering::Release);
                }
                false
            } else {
                lease.revoked.strong_count() > 0
            }
        });
        Ok(())
    }
}

struct ActiveLease {
    credential_reference: CredentialReference,
    revoked: Weak<AtomicBool>,
}

#[derive(Clone)]
struct VaultState {
    vault_id: Uuid,
    generation: u64,
    entries: BTreeMap<CredentialReference, StoredCredential>,
}

impl VaultState {
    fn empty(vault_id: Uuid) -> Self {
        Self {
            vault_id,
            generation: 0,
            entries: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VaultPayload {
    vault_id: Uuid,
    generation: u64,
    entries: Vec<StoredCredential>,
}

impl From<&VaultState> for VaultPayload {
    fn from(state: &VaultState) -> Self {
        Self {
            vault_id: state.vault_id,
            generation: state.generation,
            entries: state.entries.values().cloned().collect(),
        }
    }
}

impl TryFrom<VaultPayload> for VaultState {
    type Error = CredentialVaultError;

    fn try_from(payload: VaultPayload) -> Result<Self, Self::Error> {
        if payload.entries.len() > MAX_CREDENTIALS {
            return Err(CredentialVaultError::CorruptVault);
        }
        let mut entries = BTreeMap::new();
        for entry in payload.entries {
            validate_stored_credential(&entry)?;
            if entries
                .insert(entry.credential_reference.clone(), entry)
                .is_some()
            {
                return Err(CredentialVaultError::CorruptVault);
            }
        }
        Ok(Self {
            vault_id: payload.vault_id,
            generation: payload.generation,
            entries,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredCredential {
    credential_reference: CredentialReference,
    kind: String,
    secret: SecretBytes,
    created_generation: u64,
    revision: u64,
}

impl StoredCredential {
    fn metadata(&self) -> CredentialMetadata {
        CredentialMetadata {
            credential_reference: self.credential_reference.clone(),
            kind: self.kind.clone(),
            created_generation: self.created_generation,
        }
    }
}

#[derive(Clone)]
struct SecretBytes(Zeroizing<Vec<u8>>);

impl SecretBytes {
    fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl Serialize for SecretBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_slice())
    }
}

impl<'de> Deserialize<'de> for SecretBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<u8>::deserialize(deserializer).map(Self::new)
    }
}

enum Persistence {
    File {
        path: PathBuf,
        key: SecretKey,
        protection: ProtectionHeader,
    },
    SessionOnly,
}

impl Persistence {
    fn protection(&self) -> VaultProtection {
        match self {
            Self::File {
                protection: ProtectionHeader::InstallationKey,
                ..
            } => VaultProtection::InstallationKey,
            Self::File {
                protection: ProtectionHeader::Password(_),
                ..
            } => VaultProtection::PasswordProtected,
            Self::SessionOnly => VaultProtection::SessionOnly,
        }
    }

    fn requires_initial_write(&self) -> bool {
        match self {
            Self::File { path, .. } => !path.exists(),
            Self::SessionOnly => false,
        }
    }

    fn persist(
        &self,
        state: &VaultState,
        entropy: &dyn EntropySource,
    ) -> CredentialVaultResult<()> {
        match self {
            Self::SessionOnly => Ok(()),
            Self::File {
                path,
                key,
                protection,
            } => {
                let envelope = encrypt_state(state, key, protection.clone(), entropy)?;
                write_envelope_atomically(path, &envelope)
            }
        }
    }

    fn decrypt(&self, envelope: &VaultEnvelope) -> CredentialVaultResult<VaultState> {
        let (key, expected_protection) = match self {
            Self::File {
                key, protection, ..
            } => (key, protection),
            Self::SessionOnly => {
                return Err(CredentialVaultError::SessionOnlyPersistenceForbidden);
            }
        };
        if &envelope.protection != expected_protection {
            return Err(CredentialVaultError::ProtectionModeMismatch);
        }
        decrypt_state(envelope, key)
    }
}

struct SecretKey(Zeroizing<[u8; INSTALLATION_KEY_BYTES]>);

impl SecretKey {
    fn from_slice(bytes: &[u8]) -> CredentialVaultResult<Self> {
        if bytes.len() != INSTALLATION_KEY_BYTES {
            return Err(CredentialVaultError::InvalidInstallationKey);
        }
        let mut key = Zeroizing::new([0_u8; INSTALLATION_KEY_BYTES]);
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }

    fn as_slice(&self) -> &[u8] {
        &self.0[..]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ProtectionHeader {
    InstallationKey,
    Password(PasswordParameters),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordParameters {
    salt: Vec<u8>,
    memory_kib: u32,
    iterations: u32,
    lanes: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VaultEnvelope {
    magic: Vec<u8>,
    schema_version: u16,
    vault_id: Uuid,
    protection: ProtectionHeader,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

fn derive_password_key(
    password: &[u8],
    protection: &ProtectionHeader,
) -> CredentialVaultResult<SecretKey> {
    let parameters = match protection {
        ProtectionHeader::Password(parameters) => parameters,
        ProtectionHeader::InstallationKey => {
            return Err(CredentialVaultError::ProtectionModeMismatch);
        }
    };
    validate_password_parameters(parameters)?;
    let params = Params::new(
        parameters.memory_kib,
        parameters.iterations,
        parameters.lanes,
        Some(INSTALLATION_KEY_BYTES),
    )
    .map_err(|_| CredentialVaultError::CorruptVault)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut derived = Zeroizing::new([0_u8; INSTALLATION_KEY_BYTES]);
    argon
        .hash_password_into(password, &parameters.salt, &mut derived[..])
        .map_err(|_| CredentialVaultError::AuthenticationFailed)?;
    Ok(SecretKey(derived))
}

fn validate_password_parameters(parameters: &PasswordParameters) -> CredentialVaultResult<()> {
    if parameters.salt.len() != PASSWORD_SALT_BYTES
        || parameters.memory_kib != PASSWORD_MEMORY_KIB
        || parameters.iterations != PASSWORD_ITERATIONS
        || parameters.lanes != PASSWORD_LANES
    {
        return Err(CredentialVaultError::CorruptVault);
    }
    Ok(())
}

fn encrypt_state(
    state: &VaultState,
    key: &SecretKey,
    protection: ProtectionHeader,
    entropy: &dyn EntropySource,
) -> CredentialVaultResult<VaultEnvelope> {
    let plaintext = Zeroizing::new(
        serde_json::to_vec(&VaultPayload::from(state))
            .map_err(|_| CredentialVaultError::Serialization)?,
    );
    let mut nonce = vec![0_u8; NONCE_BYTES];
    entropy.fill(&mut nonce)?;
    let aad = envelope_aad(VAULT_SCHEMA_VERSION, state.vault_id, &protection)?;
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|_| CredentialVaultError::CorruptVault)?;
    let nonce_array =
        XNonce::try_from(nonce.as_slice()).map_err(|_| CredentialVaultError::CorruptVault)?;
    let ciphertext = cipher
        .encrypt(
            &nonce_array,
            Payload {
                msg: &plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| CredentialVaultError::AuthenticationFailed)?;
    Ok(VaultEnvelope {
        magic: VAULT_MAGIC.to_vec(),
        schema_version: VAULT_SCHEMA_VERSION,
        vault_id: state.vault_id,
        protection,
        nonce,
        ciphertext,
    })
}

fn decrypt_state(envelope: &VaultEnvelope, key: &SecretKey) -> CredentialVaultResult<VaultState> {
    validate_envelope(envelope)?;
    let aad = envelope_aad(
        envelope.schema_version,
        envelope.vault_id,
        &envelope.protection,
    )?;
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_slice())
        .map_err(|_| CredentialVaultError::CorruptVault)?;
    let nonce = XNonce::try_from(envelope.nonce.as_slice())
        .map_err(|_| CredentialVaultError::CorruptVault)?;
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &envelope.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| CredentialVaultError::AuthenticationFailed)?,
    );
    let payload: VaultPayload =
        serde_json::from_slice(&plaintext).map_err(|_| CredentialVaultError::CorruptVault)?;
    if payload.vault_id != envelope.vault_id {
        return Err(CredentialVaultError::CorruptVault);
    }
    payload.try_into()
}

fn validate_envelope(envelope: &VaultEnvelope) -> CredentialVaultResult<()> {
    if envelope.magic != VAULT_MAGIC
        || envelope.schema_version != VAULT_SCHEMA_VERSION
        || envelope.nonce.len() != NONCE_BYTES
        || envelope.ciphertext.is_empty()
        || envelope.ciphertext.len() as u64 > MAX_VAULT_FILE_BYTES
    {
        return Err(CredentialVaultError::CorruptVault);
    }
    if let ProtectionHeader::Password(parameters) = &envelope.protection {
        validate_password_parameters(parameters)?;
    }
    Ok(())
}

fn envelope_aad(
    schema_version: u16,
    vault_id: Uuid,
    protection: &ProtectionHeader,
) -> CredentialVaultResult<Vec<u8>> {
    serde_json::to_vec(&(VAULT_MAGIC, schema_version, vault_id, protection))
        .map_err(|_| CredentialVaultError::Serialization)
}

fn read_envelope_if_present(path: &Path) -> CredentialVaultResult<Option<VaultEnvelope>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CredentialVaultError::Storage {
                operation: "open",
                source,
            });
        }
    };
    let metadata = file
        .metadata()
        .map_err(|source| CredentialVaultError::Storage {
            operation: "inspect",
            source,
        })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_VAULT_FILE_BYTES {
        return Err(CredentialVaultError::CorruptVault);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_VAULT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| CredentialVaultError::Storage {
            operation: "read",
            source,
        })?;
    if bytes.len() as u64 > MAX_VAULT_FILE_BYTES {
        return Err(CredentialVaultError::CorruptVault);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| CredentialVaultError::CorruptVault)
}

fn write_envelope_atomically(path: &Path, envelope: &VaultEnvelope) -> CredentialVaultResult<()> {
    let bytes = serde_json::to_vec(envelope).map_err(|_| CredentialVaultError::Serialization)?;
    if bytes.len() as u64 > MAX_VAULT_FILE_BYTES {
        return Err(CredentialVaultError::CapacityExceeded);
    }
    let parent = path.parent().ok_or(CredentialVaultError::CorruptVault)?;
    fs::create_dir_all(parent).map_err(|source| CredentialVaultError::Storage {
        operation: "create vault directory",
        source,
    })?;
    #[cfg(unix)]
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|source| {
        CredentialVaultError::Storage {
            operation: "secure vault directory",
            source,
        }
    })?;

    let temporary = parent.join(format!(".credentials.{}.tmp", Uuid::new_v4().as_simple()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let write_result = (|| {
        let mut file =
            options
                .open(&temporary)
                .map_err(|source| CredentialVaultError::Storage {
                    operation: "create encrypted temporary vault",
                    source,
                })?;
        file.write_all(&bytes)
            .map_err(|source| CredentialVaultError::Storage {
                operation: "write encrypted temporary vault",
                source,
            })?;
        file.sync_all()
            .map_err(|source| CredentialVaultError::Storage {
                operation: "sync encrypted temporary vault",
                source,
            })?;
        fs::rename(&temporary, path).map_err(|source| CredentialVaultError::Storage {
            operation: "activate encrypted vault",
            source,
        })?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|source| CredentialVaultError::Storage {
                operation: "sync vault directory",
                source,
            })?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn validate_credential(kind: &str, secret: &[u8]) -> CredentialVaultResult<()> {
    if kind.is_empty()
        || kind.len() > MAX_KIND_BYTES
        || !kind
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        || secret.is_empty()
        || secret.len() > MAX_SECRET_BYTES
    {
        return Err(CredentialVaultError::InvalidCredential);
    }
    Ok(())
}

fn validate_stored_credential(entry: &StoredCredential) -> CredentialVaultResult<()> {
    validate_credential(&entry.kind, entry.secret.as_slice())?;
    let valid_reference = entry
        .credential_reference
        .as_str()
        .strip_prefix("credential:")
        .is_some_and(|identifier| {
            identifier.len() == 32 && identifier.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    if !valid_reference || entry.created_generation == 0 || entry.revision == 0 {
        return Err(CredentialVaultError::CorruptVault);
    }
    Ok(())
}

fn random_uuid(entropy: &dyn EntropySource) -> CredentialVaultResult<Uuid> {
    let mut bytes = [0_u8; 16];
    entropy.fill(&mut bytes)?;
    // RFC 4122 variant and version 4 keep the opaque IDs conventional while
    // their entropy still comes from the injected cryptographic source.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(Uuid::from_bytes(bytes))
}

fn scan_secrets_in_state(
    state: &VaultState,
    surface: SecretSurface,
    bytes: &[u8],
) -> SecretLeakFinding {
    let direct_secret_matches = state
        .entries
        .values()
        .map(|entry| count_subslices(bytes, entry.secret.as_slice()))
        .sum();
    SecretLeakFinding {
        surface,
        direct_secret_matches,
        sensitive_field_paths: Vec::new(),
    }
}

fn count_subslices(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|candidate| *candidate == needle)
        .count()
}

fn redact_sensitive_fields(value: &mut Value, path: &str) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_path = format!("{path}.{key}");
                if is_sensitive_field(key) {
                    *child = Value::String("[REDACTED]".into());
                } else {
                    redact_sensitive_fields(child, &child_path);
                }
            }
        }
        Value::Array(array) => {
            for (index, child) in array.iter_mut().enumerate() {
                redact_sensitive_fields(child, &format!("{path}[{index}]"));
            }
        }
        _ => {}
    }
}

fn is_sensitive_field(field: &str) -> bool {
    let normalized: String = field
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    if matches!(
        normalized.as_str(),
        "credentialreference" | "credentialref" | "secretreference" | "secretref"
    ) {
        return false;
    }
    [
        "apikey",
        "authorization",
        "bearertoken",
        "credential",
        "password",
        "privatekey",
        "secret",
        "token",
    ]
    .iter()
    .any(|sensitive| normalized.contains(sensitive))
}

fn redact_value_strings(vault: &CredentialVault, value: &mut Value) -> CredentialVaultResult<()> {
    match value {
        Value::String(string) => {
            *string = vault.redact_text(string)?;
        }
        Value::Array(array) => {
            for child in array {
                redact_value_strings(vault, child)?;
            }
        }
        Value::Object(object) => {
            for child in object.values_mut() {
                redact_value_strings(vault, child)?;
            }
        }
        _ => {}
    }
    Ok(())
}
