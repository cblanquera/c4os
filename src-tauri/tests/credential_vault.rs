// This integration suite imports the isolated module directly until the task
// coordinator wires the shared `security` module into the crate root.
#[allow(dead_code)]
#[path = "../src/security/credentials.rs"]
mod credentials;

use credentials::{
    CredentialVault, CredentialVaultError, CredentialVaultResult, EntropySource, InstallationKey,
    InstallationKeyStore, MonotonicClock, ReauthenticationError, ReauthenticationProvider,
    ReauthenticationPurpose, SecretSurface, VaultProtection,
};
use serde_json::json;
use std::{
    fs,
    io::Cursor,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tempfile::TempDir;

const INSTALLATION_KEY: [u8; 32] = [0x42; 32];
const PRIMARY_SECRET: &[u8] = b"sk-live-C4OS-do-not-leak-0123456789";

struct FakeInstallationKeyStore {
    key: Option<[u8; 32]>,
}

impl FakeInstallationKeyStore {
    fn available() -> Self {
        Self {
            key: Some(INSTALLATION_KEY),
        }
    }

    fn unavailable() -> Self {
        Self { key: None }
    }
}

impl InstallationKeyStore for FakeInstallationKeyStore {
    fn load_or_create(&self) -> CredentialVaultResult<InstallationKey> {
        self.key
            .map(InstallationKey::from_bytes)
            .ok_or(CredentialVaultError::KeychainUnavailable)
    }
}

#[derive(Default)]
struct FakeClock {
    milliseconds: AtomicU64,
}

impl FakeClock {
    fn advance(&self, duration: Duration) {
        self.milliseconds.fetch_add(
            u64::try_from(duration.as_millis()).expect("test duration fits"),
            Ordering::SeqCst,
        );
    }
}

impl MonotonicClock for FakeClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.milliseconds.load(Ordering::SeqCst))
    }
}

#[derive(Default)]
struct CountingEntropy {
    counter: AtomicU64,
}

impl EntropySource for CountingEntropy {
    fn fill(&self, destination: &mut [u8]) -> CredentialVaultResult<()> {
        let sequence = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        for (index, byte) in destination.iter_mut().enumerate() {
            *byte = sequence.wrapping_add(index as u64) as u8;
        }
        Ok(())
    }
}

struct FakeReauthentication(bool);

impl ReauthenticationProvider for FakeReauthentication {
    fn reauthenticate(
        &self,
        purpose: ReauthenticationPurpose,
    ) -> Result<(), ReauthenticationError> {
        assert_eq!(purpose, ReauthenticationPurpose::ImportCredential);
        self.0.then_some(()).ok_or(ReauthenticationError)
    }
}

fn vault_path(directory: &TempDir) -> std::path::PathBuf {
    directory.path().join("vault").join("credentials.vault")
}

fn test_vault(path: &Path, clock: Arc<FakeClock>) -> CredentialVaultResult<CredentialVault> {
    CredentialVault::open_or_create_installation_with_components(
        path.to_owned(),
        &FakeInstallationKeyStore::available(),
        clock,
        Arc::new(CountingEntropy::default()),
    )
}

#[test]
fn installation_key_vault_persists_only_ciphertext_and_opaque_references() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let vault = test_vault(&path, Arc::new(FakeClock::default())).expect("open vault");

    let credential_reference = vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store secret");
    assert!(credential_reference.as_str().starts_with("credential:"));
    assert!(!credential_reference.as_str().contains("sk-live"));
    assert_eq!(vault.protection(), VaultProtection::InstallationKey);

    let stored_bytes = fs::read(&path).expect("read encrypted vault");
    assert!(!contains(&stored_bytes, PRIMARY_SECRET));
    assert!(!String::from_utf8_lossy(&stored_bytes).contains("sk-live"));

    let reopened = test_vault(&path, Arc::new(FakeClock::default())).expect("reopen vault");
    let lease = reopened
        .lease_for_operation(
            &credential_reference,
            "provider.connection-test",
            Duration::from_secs(30),
        )
        .expect("lease secret");
    let mut operation_pipe = Cursor::new(Vec::new());
    lease
        .deliver_to(&mut operation_pipe)
        .expect("deliver through narrow operation channel");
    assert_eq!(operation_pipe.into_inner(), PRIMARY_SECRET);
}

#[test]
fn password_vault_rejects_wrong_password_without_mutating_ciphertext() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let password = b"correct horse battery staple";
    let vault = CredentialVault::open_or_create_password(&path, password).expect("create vault");
    vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store secret");
    let before = fs::read(&path).expect("read vault before failed open");

    let error = match CredentialVault::open_or_create_password(&path, b"wrong password") {
        Ok(_) => panic!("wrong password unexpectedly opened vault"),
        Err(error) => error,
    };
    assert!(matches!(error, CredentialVaultError::AuthenticationFailed));
    assert_eq!(
        fs::read(&path).expect("read vault after failed open"),
        before
    );

    let reopened = CredentialVault::open_or_create_password(&path, password).expect("reopen");
    assert_eq!(reopened.metadata().expect("metadata").len(), 1);
}

#[test]
fn corrupted_or_hostile_vault_content_fails_closed() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let vault = test_vault(&path, Arc::new(FakeClock::default())).expect("open vault");
    vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store secret");

    let mut envelope: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("read envelope")).expect("parse envelope");
    let ciphertext = envelope["ciphertext"]
        .as_array_mut()
        .expect("ciphertext array");
    ciphertext[0] = json!(ciphertext[0].as_u64().expect("cipher byte") ^ 0xff);
    fs::write(
        &path,
        serde_json::to_vec(&envelope).expect("serialize tampered envelope"),
    )
    .expect("write tampered envelope");

    let error = match test_vault(&path, Arc::new(FakeClock::default())) {
        Ok(_) => panic!("tampered vault unexpectedly opened"),
        Err(error) => error,
    };
    assert!(matches!(error, CredentialVaultError::AuthenticationFailed));

    fs::write(&path, br#"{"schema_version":1,"ciphertext":[]}"#).expect("write malformed vault");
    let error = match test_vault(&path, Arc::new(FakeClock::default())) {
        Ok(_) => panic!("malformed vault unexpectedly opened"),
        Err(error) => error,
    };
    assert!(matches!(error, CredentialVaultError::CorruptVault));
}

#[test]
fn unavailable_keychain_never_creates_a_plaintext_fallback() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let error = match CredentialVault::open_or_create_installation_with_components(
        path.clone(),
        &FakeInstallationKeyStore::unavailable(),
        Arc::new(FakeClock::default()),
        Arc::new(CountingEntropy::default()),
    ) {
        Ok(_) => panic!("unavailable keychain unexpectedly opened vault"),
        Err(error) => error,
    };

    assert!(matches!(error, CredentialVaultError::KeychainUnavailable));
    assert!(!path.exists());
}

#[test]
fn explicit_session_only_fallback_never_writes_a_vault_file() {
    let vault = CredentialVault::session_only_with_components(
        Arc::new(FakeClock::default()),
        Arc::new(CountingEntropy::default()),
    )
    .expect("create session-only vault");
    assert_eq!(vault.protection(), VaultProtection::SessionOnly);
    let credential_reference = vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store session credential");
    let lease = vault
        .lease_for_operation(
            &credential_reference,
            "provider.request",
            Duration::from_secs(10),
        )
        .expect("lease session credential");
    let mut pipe = Cursor::new(Vec::new());
    lease.deliver_to(&mut pipe).expect("deliver credential");
    assert_eq!(pipe.into_inner(), PRIMARY_SECRET);
}

#[test]
fn removal_and_replacement_immediately_revoke_outstanding_leases() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let vault = test_vault(&path, Arc::new(FakeClock::default())).expect("open vault");
    let credential_reference = vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store secret");

    let replaced_lease = vault
        .lease_for_operation(
            &credential_reference,
            "provider.request-1",
            Duration::from_secs(30),
        )
        .expect("lease before replacement");
    vault
        .replace(&credential_reference, b"replacement-secret")
        .expect("replace secret");
    assert!(!replaced_lease.is_valid());
    assert!(matches!(
        replaced_lease.deliver_to(&mut Cursor::new(Vec::new())),
        Err(CredentialVaultError::LeaseRevoked)
    ));

    let removed_lease = vault
        .lease_for_operation(
            &credential_reference,
            "provider.request-2",
            Duration::from_secs(30),
        )
        .expect("lease before removal");
    vault.remove(&credential_reference).expect("remove secret");
    assert!(!removed_lease.is_valid());
    assert!(matches!(
        removed_lease.deliver_to(&mut Cursor::new(Vec::new())),
        Err(CredentialVaultError::LeaseRevoked)
    ));
    assert!(matches!(
        vault.lease_for_operation(
            &credential_reference,
            "provider.request-3",
            Duration::from_secs(30)
        ),
        Err(CredentialVaultError::CredentialNotFound)
    ));
}

#[test]
fn operation_delivery_expires_and_never_builds_argv_or_environment_values() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let clock = Arc::new(FakeClock::default());
    let vault = test_vault(&path, Arc::clone(&clock)).expect("open vault");
    let credential_reference = vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store secret");
    let lease = vault
        .lease_for_operation(
            &credential_reference,
            "provider.request",
            Duration::from_secs(2),
        )
        .expect("lease secret");

    assert_eq!(lease.operation(), "provider.request");
    assert!(lease.is_valid());
    clock.advance(Duration::from_secs(2));
    assert!(!lease.is_valid());
    assert!(matches!(
        lease.deliver_to(&mut Cursor::new(Vec::new())),
        Err(CredentialVaultError::LeaseRevoked | CredentialVaultError::LeaseExpired)
    ));
    assert!(!format!("{lease:?}").contains("sk-live"));
}

#[test]
fn credential_import_requires_recent_single_use_reauthentication() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let clock = Arc::new(FakeClock::default());
    let vault = test_vault(&path, Arc::clone(&clock)).expect("open vault");

    assert!(matches!(
        vault.authorize_import(&FakeReauthentication(false), Duration::from_secs(30)),
        Err(CredentialVaultError::ReauthenticationFailed)
    ));

    let expired = vault
        .authorize_import(&FakeReauthentication(true), Duration::from_secs(2))
        .expect("authorize import");
    clock.advance(Duration::from_secs(2));
    assert!(matches!(
        vault.import(&expired, "provider.api_key", PRIMARY_SECRET),
        Err(CredentialVaultError::ReauthenticationExpired)
    ));

    let grant = vault
        .authorize_import(&FakeReauthentication(true), Duration::from_secs(30))
        .expect("authorize import");
    vault
        .import(&grant, "provider.api_key", PRIMARY_SECRET)
        .expect("import credential");
    assert!(matches!(
        vault.import(&grant, "provider.api_key", b"second-secret"),
        Err(CredentialVaultError::ReauthenticationRequired)
            | Err(CredentialVaultError::ReauthenticationConsumed)
    ));
}

#[test]
fn redaction_and_forbidden_surface_scans_find_no_raw_secret_in_safe_exports() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    let vault = test_vault(&path, Arc::new(FakeClock::default())).expect("open vault");
    let credential_reference = vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store secret");

    let export = vault.export_metadata().expect("safe metadata export");
    let export_bytes = serde_json::to_vec(&export).expect("serialize export");
    assert!(
        vault
            .scan_bytes(SecretSurface::Export, &export_bytes)
            .expect("scan export")
            .is_clean()
    );
    assert!(String::from_utf8_lossy(&export_bytes).contains(credential_reference.as_str()));

    let argv = [b"provider-worker --api-key=".as_slice(), PRIMARY_SECRET].concat();
    assert!(matches!(
        vault.ensure_clean(SecretSurface::CommandLineArguments, &argv),
        Err(CredentialVaultError::SecretLeakDetected)
    ));
    let environment = [b"OPENAI_API_KEY=".as_slice(), PRIMARY_SECRET].concat();
    assert!(matches!(
        vault.ensure_clean(SecretSurface::BroadEnvironment, &environment),
        Err(CredentialVaultError::SecretLeakDetected)
    ));
    assert!(matches!(
        vault.ensure_clean(SecretSurface::Log, PRIMARY_SECRET),
        Err(CredentialVaultError::SecretLeakDetected)
    ));

    let diagnostic = format!(
        "connection failed for {}",
        String::from_utf8_lossy(PRIMARY_SECRET)
    );
    let redacted = vault.redact_text(&diagnostic).expect("redact diagnostic");
    assert_eq!(redacted, "connection failed for [REDACTED]");

    let mut structured = json!({
        "authorization": "Bearer unrelated-secret",
        "nested": { "message": diagnostic, "safe": "retained" }
    });
    vault
        .redact_diagnostic_value(&mut structured)
        .expect("redact structured diagnostic");
    assert_eq!(structured["authorization"], "[REDACTED]");
    assert_eq!(
        structured["nested"]["message"],
        "connection failed for [REDACTED]"
    );
    assert_eq!(structured["nested"]["safe"], "retained");
}

#[test]
fn password_and_keychain_modes_are_explicit_and_never_cross_open() {
    let directory = TempDir::new().expect("temporary directory");
    let path = vault_path(&directory);
    CredentialVault::open_or_create_password(&path, b"explicit-password")
        .expect("create password vault");

    let error = match test_vault(&path, Arc::new(FakeClock::default())) {
        Ok(_) => panic!("keychain mode unexpectedly opened password vault"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CredentialVaultError::ProtectionModeMismatch
    ));
}

/// Opt-in boundary check only. It writes one random installation key to the
/// current macOS login Keychain and must be followed by the cleanup command
/// reported by the coordinator. The deterministic suite never runs this test.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires explicit disposable-keychain authorization"]
fn live_macos_keychain_round_trip_is_opt_in() {
    use credentials::MacOsInstallationKeyStore;

    let bundle_identifier = std::env::var("C4OS_LIVE_KEYCHAIN_TEST_BUNDLE")
        .expect("set a unique dev.c4os.live-test.* bundle identifier");
    assert!(
        bundle_identifier.starts_with("dev.c4os.live-test."),
        "refusing to use a production-looking Keychain service"
    );
    let directory = TempDir::new().expect("temporary directory");
    let store = MacOsInstallationKeyStore::new(bundle_identifier);
    let vault =
        CredentialVault::open_or_create_with_installation_key(vault_path(&directory), &store)
            .expect("open Keychain-backed vault");
    let credential_reference = vault
        .store("provider.api_key", PRIMARY_SECRET)
        .expect("store encrypted credential");
    assert!(credential_reference.as_str().starts_with("credential:"));
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}
