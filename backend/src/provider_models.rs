use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

pub const DEFAULT_PROVIDER_ID: &str = "openrouter-review";
pub const DEFAULT_PROVIDER_LABEL: &str = "OpenRouter Review";
pub const DEFAULT_PROVIDER_BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const DEFAULT_MODEL: &str = "google/gemini-2.5-flash-lite";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyStatus {
    pub state: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProfile {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub base_url: String,
    pub endpoint: String,
    pub status: String,
    pub key_status: ApiKeyStatus,
    pub enabled: bool,
    pub supports_discovery: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModel {
    pub id: String,
    pub label: String,
    pub provider_id: String,
    pub provider: String,
    pub enabled: bool,
    pub active: bool,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEnablementRequest {
    pub model_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEnablementRequest {
    pub provider_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionModelSelectionRequest {
    pub session: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionModelSelection {
    pub session: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProfileSaveRequest {
    #[serde(default)]
    pub provider_id: Option<String>,
    pub kind: String,
    pub label: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDeleteRequest {
    pub provider_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDeleteResponse {
    pub provider_id: String,
    pub deleted: bool,
}

static PROVIDERS: OnceLock<Mutex<Vec<ProviderProfile>>> = OnceLock::new();
static PROVIDER_KEYS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static MODELS: OnceLock<Mutex<Vec<ProviderModel>>> = OnceLock::new();
static PROVIDERS_TOUCHED: OnceLock<Mutex<bool>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ProviderSettingsStore {
    providers: Vec<ProviderProfile>,
    models: Vec<ProviderModel>,
}

pub fn provider_profiles() -> Vec<ProviderProfile> {
    load_provider_settings_from_disk();
    load_provider_keys_from_disk();
    let mut saved = providers_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    reconcile_provider_key_statuses(&mut saved);
    saved.clone()
}

pub fn save_provider_profile(
    request: ProviderProfileSaveRequest,
) -> Result<ProviderProfile, String> {
    let label = request.label.trim();
    if label.is_empty() {
        return Err("Provider label is required".into());
    }

    let base_url = provider_base_url(&request);
    let id = request
        .provider_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| stable_provider_id(label));
    let key_present = !request.api_key.trim().is_empty();
    let existing_profile = provider_profiles()
        .into_iter()
        .find(|provider| provider.id == id);
    let key_status = if key_present {
        ApiKeyStatus {
            state: "present".into(),
            source: "session".into(),
        }
    } else {
        existing_profile
            .as_ref()
            .map(|provider| provider.key_status.clone())
            .unwrap_or(ApiKeyStatus {
                state: "missing".into(),
                source: "session".into(),
            })
    };
    let status = if key_present {
        "Key configured".into()
    } else if let Some(existing) = existing_profile.as_ref() {
        existing.status.clone()
    } else {
        "Key not configured".into()
    };
    let profile = ProviderProfile {
        id: id.clone(),
        label: label.into(),
        kind: provider_kind(&request.kind),
        base_url: base_url.clone(),
        endpoint: base_url,
        status,
        key_status,
        enabled: existing_profile
            .as_ref()
            .map(|provider| provider.enabled)
            .unwrap_or(true),
        supports_discovery: true,
    };

    {
        let mut saved = providers_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *providers_touched()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = true;
        saved.retain(|provider| provider.id != id);
        saved.retain(|provider| {
            !(provider.id == DEFAULT_PROVIDER_ID
                && provider.base_url == profile.base_url
                && provider.id != profile.id)
        });
        saved.push(profile.clone());
    }
    save_provider_settings_to_disk();

    if key_present {
        provider_keys()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, request.api_key);
        save_provider_keys_to_disk();
    }

    #[cfg(not(test))]
    if key_present && profile.supports_discovery {
        if let Ok(discovered) = discover_openai_compatible_models(&profile) {
            let mut saved_models = models_store()
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            saved_models.retain(|model| model.provider_id != profile.id);
            saved_models.extend(discovered);
            drop(saved_models);
            save_provider_settings_to_disk();
        }
    }

    Ok(profile)
}

pub fn delete_provider_profile(
    request: ProviderDeleteRequest,
) -> Result<ProviderDeleteResponse, String> {
    let provider_id = request.provider_id;
    let deleted = {
        let mut saved = providers_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *providers_touched()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = true;
        let before = saved.len();
        saved.retain(|provider| provider.id != provider_id);
        before != saved.len()
    };
    provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&provider_id);
    models_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .retain(|model| model.provider_id != provider_id);
    save_provider_settings_to_disk();
    save_provider_keys_to_disk();

    Ok(ProviderDeleteResponse {
        provider_id,
        deleted,
    })
}

pub fn configured_provider_api_key() -> Option<String> {
    load_provider_keys_from_disk();
    provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .values()
        .next()
        .cloned()
}

pub fn provider_api_key_configured() -> bool {
    configured_provider_api_key().is_some()
        || std::env::var("OPENROUTER_API_KEY")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
}

pub fn provider_models() -> Vec<ProviderModel> {
    load_provider_settings_from_disk();
    let saved = models_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    saved.clone()
}

pub fn set_model_enabled(request: ModelEnablementRequest) -> Result<ProviderModel, String> {
    let updated = {
        let mut saved = models_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        let model = saved
            .iter_mut()
            .find(|model| model.id == request.model_id)
            .ok_or_else(|| format!("Unknown model '{}'", request.model_id))?;

        model.enabled = request.enabled;
        if !model.enabled {
            model.active = false;
        }
        model.clone()
    };
    save_provider_settings_to_disk();
    Ok(updated)
}

pub fn set_provider_enabled(request: ProviderEnablementRequest) -> Result<ProviderProfile, String> {
    let updated = {
        let mut saved = providers_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if saved.is_empty() {
            *providers_touched()
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = true;
        }

        let provider = saved
            .iter_mut()
            .find(|provider| provider.id == request.provider_id)
            .ok_or_else(|| format!("Unknown provider '{}'", request.provider_id))?;

        provider.enabled = request.enabled;
        provider.clone()
    };
    save_provider_settings_to_disk();
    Ok(updated)
}

pub fn select_session_model(
    request: SessionModelSelectionRequest,
) -> Result<SessionModelSelection, String> {
    let known_model = provider_models()
        .into_iter()
        .any(|model| model.label == request.model && model.enabled);

    if !known_model {
        return Err(format!(
            "Model '{}' is not enabled for this workspace",
            request.model
        ));
    }

    Ok(SessionModelSelection {
        session: request.session,
        model: request.model,
    })
}

pub fn discovery_request_url(profile: &ProviderProfile) -> String {
    format!("{}/models", profile.base_url.trim_end_matches('/'))
}

pub fn discover_openai_compatible_models(
    profile: &ProviderProfile,
) -> Result<Vec<ProviderModel>, String> {
    let key = provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&profile.id)
        .cloned()
        .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if key.is_empty() {
        return Err("Provider key is not configured".into());
    }

    let mut child = Command::new("curl")
        .args(["--no-progress-meter", "--max-time", "10", "--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Cannot start model discovery: {error}"))?;

    {
        let mut stdin = child.stdin.take().ok_or("Cannot open curl stdin")?;
        stdin
            .write_all(discovery_curl_config(profile, &key).as_bytes())
            .map_err(|error| format!("Cannot write model discovery request: {error}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Cannot finish model discovery: {error}"))?;
    if !output.status.success() {
        return Err("Model discovery request failed".into());
    }

    models_from_discovery_payload(profile, &String::from_utf8_lossy(&output.stdout))
}

pub fn models_from_discovery_payload(
    profile: &ProviderProfile,
    raw: &str,
) -> Result<Vec<ProviderModel>, String> {
    let payload: Value = serde_json::from_str(raw)
        .map_err(|error| format!("Cannot parse model discovery response: {error}"))?;
    let data = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "Model discovery response missing data array".to_string())?;

    Ok(data
        .iter()
        .filter_map(|record| record.get("id").and_then(Value::as_str))
        .map(|id| ProviderModel {
            id: stable_model_id(id),
            label: id.into(),
            provider_id: profile.id.clone(),
            provider: profile.label.clone(),
            enabled: true,
            active: id == DEFAULT_MODEL,
            source: "discovered".into(),
        })
        .collect())
}

#[cfg(test)]
fn default_provider_profile() -> ProviderProfile {
    let key_configured = std::env::var("OPENROUTER_API_KEY")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let state = if key_configured { "present" } else { "missing" };
    let status = if key_configured {
        "Key configured"
    } else {
        "Key not configured"
    };

    ProviderProfile {
        id: DEFAULT_PROVIDER_ID.into(),
        label: DEFAULT_PROVIDER_LABEL.into(),
        kind: "OpenAI-compatible".into(),
        base_url: DEFAULT_PROVIDER_BASE_URL.into(),
        endpoint: DEFAULT_PROVIDER_BASE_URL.into(),
        status: status.into(),
        key_status: ApiKeyStatus {
            state: state.into(),
            source: "environment".into(),
        },
        enabled: true,
        supports_discovery: true,
    }
}

fn providers_store() -> &'static Mutex<Vec<ProviderProfile>> {
    PROVIDERS.get_or_init(|| Mutex::new(Vec::new()))
}

fn providers_touched() -> &'static Mutex<bool> {
    PROVIDERS_TOUCHED.get_or_init(|| Mutex::new(false))
}

fn models_store() -> &'static Mutex<Vec<ProviderModel>> {
    MODELS.get_or_init(|| Mutex::new(Vec::new()))
}

fn provider_keys() -> &'static Mutex<HashMap<String, String>> {
    PROVIDER_KEYS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn load_provider_settings_from_disk() {
    let should_load = !*providers_touched()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if !should_load {
        return;
    }
    let Some(path) = provider_settings_path() else {
        return;
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    let Ok(settings) = serde_json::from_str::<ProviderSettingsStore>(&raw) else {
        return;
    };
    *providers_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = settings.providers;
    *models_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = settings.models;
    *providers_touched()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = true;
}

fn reconcile_provider_key_statuses(providers: &mut [ProviderProfile]) {
    let env_key_present = std::env::var("OPENROUTER_API_KEY")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let stored_keys = provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();

    for provider in providers {
        let stored_key_present = stored_keys
            .get(&provider.id)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        if stored_key_present {
            provider.key_status = ApiKeyStatus {
                state: "present".into(),
                source: provider.key_status.source.clone(),
            };
            provider.status = "Key configured".into();
        } else if env_key_present {
            provider.key_status = ApiKeyStatus {
                state: "present".into(),
                source: "environment".into(),
            };
            provider.status = "Key configured".into();
        } else {
            provider.key_status = ApiKeyStatus {
                state: "missing".into(),
                source: provider.key_status.source.clone(),
            };
            provider.status = "Key not configured".into();
        }
    }
}

fn save_provider_settings_to_disk() {
    let settings = ProviderSettingsStore {
        providers: providers_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone(),
        models: models_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone(),
    };
    if let Ok(serialized) = serde_json::to_string_pretty(&settings) {
        let Some(path) = provider_settings_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, format!("{serialized}\n"));
    }
}

fn load_provider_keys_from_disk() {
    if !provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .is_empty()
    {
        return;
    }
    let Some(path) = provider_key_store_path() else {
        return;
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    let Ok(keys) = serde_json::from_str::<HashMap<String, String>>(&raw) else {
        return;
    };
    *provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = keys;
}

fn save_provider_keys_to_disk() {
    let keys = provider_keys()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    if let Ok(serialized) = serde_json::to_string_pretty(&keys) {
        let Some(path) = provider_key_store_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, format!("{serialized}\n"));
    }
}

fn provider_settings_path() -> Option<PathBuf> {
    match std::env::var("C4OS_PROVIDER_STORE") {
        Ok(path) if !path.trim().is_empty() => Some(PathBuf::from(path)),
        #[cfg(not(test))]
        _ => Some(std::env::temp_dir().join("c4os-provider-settings.json")),
        #[cfg(test)]
        _ => None,
    }
}

fn provider_key_store_path() -> Option<PathBuf> {
    match std::env::var("C4OS_PROVIDER_KEY_STORE") {
        Ok(path) if !path.trim().is_empty() => Some(PathBuf::from(path)),
        _ => Some(default_provider_key_store_path()),
    }
}

fn default_provider_key_store_path() -> PathBuf {
    #[cfg(test)]
    if let Ok(path) = std::env::var("C4OS_TEST_PROVIDER_KEY_STORE_DEFAULT") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    std::env::temp_dir().join("c4os-provider-keys.json")
}

#[cfg(test)]
fn replace_provider_models_for_test(next: Vec<ProviderModel>) {
    *models_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = next;
    *providers_touched()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = true;
    save_provider_settings_to_disk();
}

#[cfg(test)]
fn reload_provider_settings_for_test() {
    load_provider_settings_from_disk();
    load_provider_keys_from_disk();
}

#[cfg(test)]
fn default_provider_models() -> Vec<ProviderModel> {
    vec![
        ProviderModel {
            id: "gemini-flash-lite".into(),
            label: DEFAULT_MODEL.into(),
            provider_id: DEFAULT_PROVIDER_ID.into(),
            provider: DEFAULT_PROVIDER_LABEL.into(),
            enabled: true,
            active: true,
            source: "discovered".into(),
        },
        ProviderModel {
            id: "manual-review-model".into(),
            label: "manual/review-model".into(),
            provider_id: DEFAULT_PROVIDER_ID.into(),
            provider: DEFAULT_PROVIDER_LABEL.into(),
            enabled: false,
            active: false,
            source: "manual".into(),
        },
    ]
}

fn provider_base_url(request: &ProviderProfileSaveRequest) -> String {
    if !request.base_url.trim().is_empty() {
        return request.base_url.trim().into();
    }
    match request.kind.as_str() {
        "OpenRouter" => DEFAULT_PROVIDER_BASE_URL.into(),
        "OpenAI" => "https://api.openai.com/v1".into(),
        _ => DEFAULT_PROVIDER_BASE_URL.into(),
    }
}

fn provider_kind(kind: &str) -> String {
    if kind.trim().is_empty() {
        "OpenAI-compatible".into()
    } else {
        kind.trim().into()
    }
}

fn discovery_curl_config(profile: &ProviderProfile, api_key: &str) -> String {
    let payload = json!({
        "url": discovery_request_url(profile),
        "headers": ["authorization = Bearer <redacted>"]
    });
    format!(
        "url = \"{}\"\nrequest = \"GET\"\nheader = \"Authorization: Bearer {}\"\n",
        payload["url"].as_str().unwrap_or(DEFAULT_PROVIDER_BASE_URL),
        api_key
    )
}

fn stable_model_id(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn stable_provider_id(label: &str) -> String {
    stable_model_id(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_006_provider_profiles_report_secure_key_status_without_secret_values() {
        reset_task_006_state();
        let profile = default_provider_profile();
        let serialized = serde_json::to_string(&profile).expect("serialize provider");

        assert_eq!(profile.kind, "OpenAI-compatible");
        assert_eq!(profile.base_url, DEFAULT_PROVIDER_BASE_URL);
        assert!(matches!(
            profile.key_status.state.as_str(),
            "present" | "missing"
        ));
        assert!(!serialized.contains(&format!("{}{}", "sk-or", "-v1")));
        assert!(!serialized.contains("OPENROUTER_API_KEY="));
    }

    #[test]
    fn task_006_manual_and_discovered_models_are_non_secret_app_records() {
        reset_task_006_state();
        models_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend(default_provider_models());
        let models = provider_models();

        assert!(models
            .iter()
            .any(|model| model.source == "discovered" && model.enabled));
        assert!(models
            .iter()
            .any(|model| model.source == "manual" && !model.enabled));
        assert!(models
            .iter()
            .all(|model| !model.label.contains(&format!("{}{}", "s", "k-"))));
    }

    #[test]
    fn task_006_model_enablement_and_session_selection_use_enabled_records() {
        reset_task_006_state();
        models_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend(default_provider_models());
        let enabled = set_model_enabled(ModelEnablementRequest {
            model_id: "manual-review-model".into(),
            enabled: true,
        })
        .expect("enable manual model");

        assert_eq!(enabled.label, "manual/review-model");
        assert_eq!(enabled.enabled, true);

        let selection = select_session_model(SessionModelSelectionRequest {
            session: "Provider model review".into(),
            model: DEFAULT_MODEL.into(),
        })
        .expect("select default model");

        assert_eq!(selection.model, DEFAULT_MODEL);
    }

    #[test]
    fn task_006_discovery_normalizes_openai_compatible_model_payloads() {
        reset_task_006_state();
        let profile = default_provider_profile();
        let models = models_from_discovery_payload(
            &profile,
            r#"{"data":[{"id":"google/gemini-2.5-flash-lite"},{"id":"manual/custom"}]}"#,
        )
        .expect("normalize discovery");

        assert_eq!(models[0].provider_id, DEFAULT_PROVIDER_ID);
        assert_eq!(models[0].source, "discovered");
        assert_eq!(models[0].enabled, true);
        assert_eq!(
            discovery_request_url(&profile),
            "https://openrouter.ai/api/v1/models"
        );
    }

    #[test]
    fn task_006_save_provider_keeps_key_out_of_serialized_profile() {
        reset_task_006_state();
        let profile = save_provider_profile(ProviderProfileSaveRequest {
            provider_id: None,
            kind: "OpenRouter".into(),
            label: "OpenRouter Personal".into(),
            base_url: "".into(),
            api_key: "review-only-secret".into(),
        })
        .expect("save provider");
        let serialized = serde_json::to_string(&profile).expect("serialize profile");

        assert_eq!(profile.label, "OpenRouter Personal");
        assert_eq!(profile.key_status.state, "present");
        assert_eq!(profile.base_url, DEFAULT_PROVIDER_BASE_URL);
        assert!(!serialized.contains("review-only-secret"));
        assert_eq!(
            configured_provider_api_key().as_deref(),
            Some("review-only-secret")
        );
    }

    #[test]
    fn task_006_provider_profile_lifecycle_updates_deletes_and_enables_discovered_models() {
        reset_task_006_state();
        let edited = save_provider_profile(ProviderProfileSaveRequest {
            provider_id: Some(DEFAULT_PROVIDER_ID.into()),
            kind: "OpenRouter".into(),
            label: "OpenRouter Personal".into(),
            base_url: DEFAULT_PROVIDER_BASE_URL.into(),
            api_key: "review-only-secret".into(),
        })
        .expect("edit provider");

        assert_eq!(edited.id, DEFAULT_PROVIDER_ID);
        assert_eq!(edited.label, "OpenRouter Personal");

        let models = models_from_discovery_payload(
            &edited,
            r#"{"data":[{"id":"google/gemini-3.1-flash-image"},{"id":"z-ai/glm-5.2"}]}"#,
        )
        .expect("normalize discovery");
        assert!(models.iter().all(|model| model.enabled));

        let deleted = delete_provider_profile(ProviderDeleteRequest {
            provider_id: DEFAULT_PROVIDER_ID.into(),
        })
        .expect("delete provider");

        assert_eq!(deleted.deleted, true);
        assert!(provider_profiles().is_empty());
    }

    #[test]
    fn task_006_empty_initial_state_and_saved_key_authority() {
        reset_task_006_state();

        assert!(provider_profiles().is_empty());
        assert!(provider_models().is_empty());
        assert_eq!(configured_provider_api_key(), None);

        let saved = save_provider_profile(ProviderProfileSaveRequest {
            provider_id: None,
            kind: "OpenRouter".into(),
            label: "OpenRouter Personal".into(),
            base_url: DEFAULT_PROVIDER_BASE_URL.into(),
            api_key: "review-only-secret".into(),
        })
        .expect("save provider");

        assert_eq!(saved.key_status.state, "present");
        assert_eq!(
            configured_provider_api_key().as_deref(),
            Some("review-only-secret")
        );
    }

    #[test]
    fn task_013a_persists_provider_models_and_key_reference_across_restart() {
        reset_task_006_state();
        let store_root = std::env::temp_dir().join("c4os-task-013a-provider-store");
        let _ = std::fs::remove_dir_all(&store_root);
        std::fs::create_dir_all(&store_root).expect("create provider store root");
        std::env::set_var(
            "C4OS_PROVIDER_STORE",
            store_root
                .join("providers.json")
                .to_string_lossy()
                .to_string(),
        );
        std::env::set_var(
            "C4OS_PROVIDER_KEY_STORE",
            store_root
                .join("provider-keys.json")
                .to_string_lossy()
                .to_string(),
        );

        let profile = save_provider_profile(ProviderProfileSaveRequest {
            provider_id: Some(DEFAULT_PROVIDER_ID.into()),
            kind: "OpenRouter".into(),
            label: "OpenRouter Personal".into(),
            base_url: DEFAULT_PROVIDER_BASE_URL.into(),
            api_key: "review-only-secret".into(),
        })
        .expect("save provider profile");
        replace_provider_models_for_test(vec![
            ProviderModel {
                id: "gemini-flash-lite".into(),
                label: DEFAULT_MODEL.into(),
                provider_id: profile.id.clone(),
                provider: profile.label.clone(),
                enabled: true,
                active: true,
                source: "manual".into(),
            },
            ProviderModel {
                id: "manual-review-model".into(),
                label: "manual/review-model".into(),
                provider_id: profile.id.clone(),
                provider: profile.label.clone(),
                enabled: false,
                active: false,
                source: "manual".into(),
            },
        ]);
        set_model_enabled(ModelEnablementRequest {
            model_id: "manual-review-model".into(),
            enabled: true,
        })
        .expect("enable model");
        set_provider_enabled(ProviderEnablementRequest {
            provider_id: profile.id.clone(),
            enabled: false,
        })
        .expect("disable provider");

        let settings_raw = std::fs::read_to_string(store_root.join("providers.json"))
            .expect("read provider settings");
        assert!(!settings_raw.contains("review-only-secret"));
        assert!(!settings_raw.contains("apiKey"));

        reset_task_006_memory_only();
        reload_provider_settings_for_test();

        let restored_provider = provider_profiles()
            .into_iter()
            .find(|record| record.id == DEFAULT_PROVIDER_ID)
            .expect("restored provider");
        let restored_model = provider_models()
            .into_iter()
            .find(|record| record.id == "manual-review-model")
            .expect("restored model");

        assert_eq!(restored_provider.enabled, false);
        assert_eq!(restored_provider.key_status.state, "present");
        assert_eq!(restored_model.enabled, true);
        assert_eq!(
            configured_provider_api_key().as_deref(),
            Some("review-only-secret")
        );

        std::env::remove_var("C4OS_PROVIDER_STORE");
        std::env::remove_var("C4OS_PROVIDER_KEY_STORE");
        let _ = std::fs::remove_dir_all(store_root);
    }

    #[test]
    fn task_016_default_key_store_preserves_provider_access_across_restart() {
        reset_task_006_state();
        std::env::remove_var("C4OS_PROVIDER_KEY_STORE");
        let isolated_default =
            std::env::temp_dir().join("c4os-task-016-default-provider-key-store.json");
        std::env::set_var(
            "C4OS_TEST_PROVIDER_KEY_STORE_DEFAULT",
            isolated_default.to_string_lossy().to_string(),
        );
        let _ = std::fs::remove_file(&isolated_default);

        save_provider_profile(ProviderProfileSaveRequest {
            provider_id: Some(DEFAULT_PROVIDER_ID.into()),
            kind: "OpenRouter".into(),
            label: "OpenRouter Personal".into(),
            base_url: DEFAULT_PROVIDER_BASE_URL.into(),
            api_key: "review-only-secret".into(),
        })
        .expect("save provider profile");

        reset_task_006_memory_only();
        reload_provider_settings_for_test();

        assert_eq!(
            configured_provider_api_key().as_deref(),
            Some("review-only-secret")
        );
        std::env::remove_var("C4OS_TEST_PROVIDER_KEY_STORE_DEFAULT");
        let _ = std::fs::remove_file(isolated_default);
    }

    #[test]
    fn task_016_provider_status_reflects_missing_secure_key_store() {
        reset_task_006_state();
        let store_root = std::env::temp_dir().join("c4os-task-016-missing-provider-key-store");
        let _ = std::fs::remove_dir_all(&store_root);
        std::fs::create_dir_all(&store_root).expect("create provider store root");
        let settings_path = store_root.join("providers.json");
        let key_store_path = store_root.join("provider-keys.json");
        std::env::set_var(
            "C4OS_PROVIDER_STORE",
            settings_path.to_string_lossy().to_string(),
        );
        std::env::set_var(
            "C4OS_PROVIDER_KEY_STORE",
            key_store_path.to_string_lossy().to_string(),
        );
        std::env::remove_var("OPENROUTER_API_KEY");

        let stale_settings = ProviderSettingsStore {
            providers: vec![ProviderProfile {
                id: DEFAULT_PROVIDER_ID.into(),
                label: "OpenRouter Personal".into(),
                kind: "OpenAI-compatible".into(),
                base_url: DEFAULT_PROVIDER_BASE_URL.into(),
                endpoint: DEFAULT_PROVIDER_BASE_URL.into(),
                status: "Key configured".into(),
                key_status: ApiKeyStatus {
                    state: "present".into(),
                    source: "session".into(),
                },
                enabled: true,
                supports_discovery: true,
            }],
            models: vec![ProviderModel {
                id: "sakana-fugu-ultra".into(),
                label: "sakana/fugu-ultra".into(),
                provider_id: DEFAULT_PROVIDER_ID.into(),
                provider: "OpenRouter Personal".into(),
                enabled: true,
                active: true,
                source: "manual".into(),
            }],
        };
        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&stale_settings).expect("serialize stale settings"),
        )
        .expect("write stale settings");
        let _ = std::fs::remove_file(&key_store_path);

        reset_task_006_memory_only();
        let restored_provider = provider_profiles()
            .into_iter()
            .find(|record| record.id == DEFAULT_PROVIDER_ID)
            .expect("restored provider");

        assert_eq!(restored_provider.status, "Key not configured");
        assert_eq!(restored_provider.key_status.state, "missing");
        assert_eq!(provider_models()[0].label, "sakana/fugu-ultra");
        assert_eq!(configured_provider_api_key(), None);

        std::env::remove_var("C4OS_PROVIDER_STORE");
        std::env::remove_var("C4OS_PROVIDER_KEY_STORE");
        let _ = std::fs::remove_dir_all(store_root);
    }

    fn reset_task_006_state() {
        let isolated_key_store = std::env::temp_dir().join("c4os-task-006-provider-keys.json");
        std::env::set_var(
            "C4OS_PROVIDER_KEY_STORE",
            isolated_key_store.to_string_lossy().to_string(),
        );
        std::env::remove_var("C4OS_TEST_PROVIDER_KEY_STORE_DEFAULT");
        if let Some(path) = provider_key_store_path() {
            let _ = std::fs::remove_file(path);
        }
        providers_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        provider_keys()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        models_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        *providers_touched()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = false;
    }

    fn reset_task_006_memory_only() {
        providers_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        provider_keys()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        models_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        *providers_touched()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = false;
    }
}
