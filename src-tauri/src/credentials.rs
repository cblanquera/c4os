use std::process::Command;

pub const OPENROUTER_PROVIDER_ID: &str = "openrouter";
const KEYCHAIN_SERVICE: &str = "dev.c4os.desktop.openrouter";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialReference {
    pub provider: String,
    pub reference: String,
}

#[derive(Debug)]
pub enum CredentialError {
    StoreUnavailable(String),
    StoreFailed(String),
}

pub trait CredentialStore {
    fn save_openrouter_key(&self, key: &str) -> Result<CredentialReference, CredentialError>;
    fn read_openrouter_key(&self, reference: &str) -> Result<String, CredentialError>;
    fn delete_openrouter_key(&self, reference: &str) -> Result<(), CredentialError>;
}

pub struct MacOsKeychainCredentialStore {
    account: String,
}

impl MacOsKeychainCredentialStore {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
        }
    }

    fn reference(&self) -> CredentialReference {
        CredentialReference {
            provider: OPENROUTER_PROVIDER_ID.to_string(),
            reference: format!("keychain://{KEYCHAIN_SERVICE}/{}", self.account),
        }
    }

    fn parse_account(reference: &str) -> Result<&str, CredentialError> {
        let prefix = format!("keychain://{KEYCHAIN_SERVICE}/");

        reference.strip_prefix(&prefix).ok_or_else(|| {
            CredentialError::StoreFailed("credential reference is not a C4OS keychain ref".into())
        })
    }

    fn run_security(args: &[&str]) -> Result<Vec<u8>, CredentialError> {
        let output = Command::new("/usr/bin/security")
            .args(args)
            .output()
            .map_err(|error| {
                CredentialError::StoreUnavailable(format!(
                    "macOS security tool unavailable: {error}"
                ))
            })?;

        if !output.status.success() {
            return Err(CredentialError::StoreFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        Ok(output.stdout)
    }
}

impl CredentialStore for MacOsKeychainCredentialStore {
    fn save_openrouter_key(&self, key: &str) -> Result<CredentialReference, CredentialError> {
        if key.trim().is_empty() {
            return Err(CredentialError::StoreFailed(
                "OpenRouter key cannot be empty".into(),
            ));
        }

        Self::run_security(&[
            "add-generic-password",
            "-U",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            &self.account,
            "-w",
            key,
        ])?;

        Ok(self.reference())
    }

    fn read_openrouter_key(&self, reference: &str) -> Result<String, CredentialError> {
        let account = Self::parse_account(reference)?;
        let output = Self::run_security(&[
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            account,
            "-w",
        ])?;

        Ok(String::from_utf8_lossy(&output).trim_end().to_string())
    }

    fn delete_openrouter_key(&self, reference: &str) -> Result<(), CredentialError> {
        let account = Self::parse_account(reference)?;

        Self::run_security(&[
            "delete-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            account,
        ])?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::RefCell;

    pub struct FakeCredentialStore {
        stored: RefCell<Option<String>>,
        should_fail: bool,
    }

    impl FakeCredentialStore {
        pub fn new() -> Self {
            Self {
                stored: RefCell::new(None),
                should_fail: false,
            }
        }

        pub fn failing() -> Self {
            Self {
                stored: RefCell::new(None),
                should_fail: true,
            }
        }
    }

    impl CredentialStore for FakeCredentialStore {
        fn save_openrouter_key(&self, key: &str) -> Result<CredentialReference, CredentialError> {
            if self.should_fail {
                return Err(CredentialError::StoreUnavailable(
                    "fake credential store unavailable".into(),
                ));
            }

            *self.stored.borrow_mut() = Some(key.to_string());

            Ok(CredentialReference {
                provider: OPENROUTER_PROVIDER_ID.to_string(),
                reference: "fake-keychain://openrouter/default".into(),
            })
        }

        fn read_openrouter_key(&self, _reference: &str) -> Result<String, CredentialError> {
            self.stored
                .borrow()
                .clone()
                .ok_or_else(|| CredentialError::StoreFailed("missing fake credential".into()))
        }

        fn delete_openrouter_key(&self, _reference: &str) -> Result<(), CredentialError> {
            *self.stored.borrow_mut() = None;
            Ok(())
        }
    }

    #[test]
    fn macos_keychain_references_do_not_embed_secret_values() {
        let store = MacOsKeychainCredentialStore::new("default");
        let reference = store.reference();

        assert_eq!(reference.provider, OPENROUTER_PROVIDER_ID);
        assert!(!reference.reference.contains("sk-or-"));
        assert!(reference.reference.starts_with("keychain://"));
    }

    #[test]
    #[ignore = "writes and deletes a dummy credential in the macOS keychain"]
    fn macos_keychain_round_trip_smoke() {
        let store = MacOsKeychainCredentialStore::new("codex-smoke-test");
        let reference = store
            .save_openrouter_key("sk-or-codex-smoke-test")
            .expect("key saved");
        let stored_key = store
            .read_openrouter_key(&reference.reference)
            .expect("key read");

        assert_eq!(stored_key, "sk-or-codex-smoke-test");

        store
            .delete_openrouter_key(&reference.reference)
            .expect("key deleted");
    }
}
