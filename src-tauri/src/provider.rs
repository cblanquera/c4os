use crate::credentials::CredentialStore;
use crate::storage::{AppStore, ProviderCredentialError};

pub const DEFAULT_OPENROUTER_MODEL: &str = "anthropic/claude-3.5-sonnet";
pub const OPENROUTER_DISCLOSURE: &str =
    "Prompts and bounded context may be sent through OpenRouter.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSetupState {
    pub provider: String,
    pub ready: bool,
    pub selected_model: Option<String>,
    pub credential_reference: Option<String>,
    pub metadata_status: MetadataStatus,
    pub disclosure: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataStatus {
    Current,
    Unknown,
    Stale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderValidation {
    Valid,
    Invalid(String),
    NetworkFailed(String),
}

#[derive(Debug)]
pub enum ProviderSetupError {
    ActiveSession(String),
    ValidationFailed(String),
    CredentialStorage(ProviderCredentialError),
}

pub trait OpenRouterValidator {
    fn validate_key(&self, key: &str) -> ProviderValidation;
}

pub struct ProviderSetupService<'a, Store, Validator>
where
    Store: CredentialStore,
    Validator: OpenRouterValidator,
{
    app_store: &'a AppStore,
    credential_store: &'a Store,
    validator: &'a Validator,
}

impl<'a, Store, Validator> ProviderSetupService<'a, Store, Validator>
where
    Store: CredentialStore,
    Validator: OpenRouterValidator,
{
    pub fn new(
        app_store: &'a AppStore,
        credential_store: &'a Store,
        validator: &'a Validator,
    ) -> Self {
        Self {
            app_store,
            credential_store,
            validator,
        }
    }

    pub fn configure_openrouter(
        &self,
        key: &str,
        selected_model: Option<&str>,
    ) -> Result<ProviderSetupState, ProviderSetupError> {
        if self.app_store.has_active_sessions().map_err(|error| {
            ProviderSetupError::CredentialStorage(ProviderCredentialError::MetadataStore(error))
        })? {
            return Err(ProviderSetupError::ActiveSession(
                "Stop or complete the running session before updating provider credentials.".into(),
            ));
        }

        match self.validator.validate_key(key) {
            ProviderValidation::Valid => {}
            ProviderValidation::Invalid(message) | ProviderValidation::NetworkFailed(message) => {
                return Err(ProviderSetupError::ValidationFailed(message));
            }
        }

        let model = selected_model.unwrap_or(DEFAULT_OPENROUTER_MODEL);
        let credential_reference = self
            .app_store
            .save_openrouter_credential(self.credential_store, key)
            .map_err(ProviderSetupError::CredentialStorage)?;

        self.app_store
            .set_setting("provider.openrouter.selected_model", &json_string(model))
            .map_err(|error| {
                ProviderSetupError::CredentialStorage(ProviderCredentialError::MetadataStore(error))
            })?;
        self.app_store
            .set_setting(
                "provider.openrouter.disclosure_acknowledged",
                r#"{"acknowledged":true}"#,
            )
            .map_err(|error| {
                ProviderSetupError::CredentialStorage(ProviderCredentialError::MetadataStore(error))
            })?;

        Ok(ProviderSetupState {
            provider: "openrouter".into(),
            ready: true,
            selected_model: Some(model.into()),
            credential_reference: Some(credential_reference),
            metadata_status: MetadataStatus::Unknown,
            disclosure: OPENROUTER_DISCLOSURE.into(),
        })
    }

    pub fn revoke_openrouter(&self) -> Result<(), ProviderSetupError> {
        if self.app_store.has_active_sessions().map_err(|error| {
            ProviderSetupError::CredentialStorage(ProviderCredentialError::MetadataStore(error))
        })? {
            return Err(ProviderSetupError::ActiveSession(
                "Stop or complete the running session before revoking provider credentials.".into(),
            ));
        }

        Ok(())
    }

    pub fn current_state(&self) -> rusqlite::Result<ProviderSetupState> {
        let selected_model = self
            .app_store
            .read_setting("provider.openrouter.selected_model")?
            .map(|value| strip_json_string(&value));
        let ready = self
            .app_store
            .read_setting("provider.openrouter.ready")?
            .is_some();

        Ok(ProviderSetupState {
            provider: "openrouter".into(),
            ready,
            selected_model,
            credential_reference: None,
            metadata_status: MetadataStatus::Unknown,
            disclosure: OPENROUTER_DISCLOSURE.into(),
        })
    }
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn strip_json_string(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;

    struct FakeValidator {
        result: ProviderValidation,
    }

    impl OpenRouterValidator for FakeValidator {
        fn validate_key(&self, _key: &str) -> ProviderValidation {
            self.result.clone()
        }
    }

    #[test]
    fn valid_provider_setup_returns_ready_state_without_raw_key_metadata() {
        let app_store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();
        let validator = FakeValidator {
            result: ProviderValidation::Valid,
        };
        let service = ProviderSetupService::new(&app_store, &credential_store, &validator);

        let state = service
            .configure_openrouter("sk-or-valid-test-key", Some("openai/gpt-4.1"))
            .expect("provider configured");
        let current_state = service.current_state().expect("current state");

        assert!(state.ready);
        assert_eq!(state.selected_model, Some("openai/gpt-4.1".into()));
        assert_eq!(state.metadata_status, MetadataStatus::Unknown);
        assert_eq!(current_state.selected_model, Some("openai/gpt-4.1".into()));
        assert_ne!(
            state.credential_reference,
            Some("sk-or-valid-test-key".into())
        );
    }

    #[test]
    fn invalid_provider_key_fails_closed_before_keychain_storage() {
        let app_store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();
        let validator = FakeValidator {
            result: ProviderValidation::Invalid("OpenRouter rejected the key".into()),
        };
        let service = ProviderSetupService::new(&app_store, &credential_store, &validator);

        let result = service.configure_openrouter("sk-or-invalid-test-key", None);

        assert!(matches!(
            result,
            Err(ProviderSetupError::ValidationFailed(_))
        ));
        assert_eq!(
            credential_store.read_openrouter_key("fake").err().is_some(),
            true
        );
        assert!(!service.current_state().expect("state").ready);
    }

    #[test]
    fn metadata_unknown_does_not_downgrade_provider_readiness() {
        let app_store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();
        let validator = FakeValidator {
            result: ProviderValidation::Valid,
        };
        let service = ProviderSetupService::new(&app_store, &credential_store, &validator);

        let state = service
            .configure_openrouter("sk-or-valid-test-key", None)
            .expect("provider configured");

        assert!(state.ready);
        assert_eq!(state.metadata_status, MetadataStatus::Unknown);
    }

    #[test]
    fn provider_update_is_blocked_while_session_is_running() {
        let app_store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();
        let validator = FakeValidator {
            result: ProviderValidation::Valid,
        };
        let service = ProviderSetupService::new(&app_store, &credential_store, &validator);

        app_store
            .create_project(crate::storage::NewProject {
                id: "project-1",
                name: "Example",
                root_path: "/tmp/example",
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        service
            .configure_openrouter("sk-or-valid-test-key", Some("openai/gpt-4.1"))
            .expect("provider configured");
        app_store
            .start_openrouter_session("session-1", "project-1", "Run")
            .expect("session started");

        let result = service.configure_openrouter("sk-or-new-test-key", None);

        assert!(matches!(result, Err(ProviderSetupError::ActiveSession(_))));
    }
}
