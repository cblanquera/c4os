//! Rust-authoritative Plugin and Skill package, trust, lifecycle, and
//! progressive-loading contracts.
//!
//! Extension packages never become renderer code or ambient native authority.
//! Package verification, immutable storage, activation, resolution, hook
//! supervision, revocation, and audit remain C4OS Core responsibilities.

pub mod hook;
pub mod package;
pub mod service;
pub mod skill;
pub mod skill_sources;
pub mod source;
pub mod store;

use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ts_rs::TS;

pub const EXTENSION_STATE_SCHEMA_VERSION: u16 = 1;
pub const EXTENSION_PACKAGE_SCHEMA_VERSION: u16 = 1;
pub const EXTENSION_CATALOG_SCHEMA_VERSION: u16 = 1;
pub const EXTENSION_HOOK_PROTOCOL_VERSION: u16 = 1;
pub const MAX_EXTENSION_RECORDS: usize = 1_024;
pub const MAX_EXTENSION_SKILLS: usize = 4_096;
pub const MAX_EXTENSION_CAPABILITIES: usize = 64;
pub const MAX_EXTENSION_TEXT_BYTES: usize = 16_384;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionPackageKind {
    Plugin,
    Skill,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionLifecycle {
    Available,
    Quarantined,
    InstalledDisabled,
    Enabling,
    Enabled,
    UpdateStaged,
    Executing,
    Revoked,
    Failed,
    RolledBack,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionTrustState {
    Untrusted,
    Trusted,
    Revoked,
    Invalid,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, TS, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionSourceKind {
    ProjectLocal,
    WorkspaceLocal,
    UserGlobal,
    PluginProvided,
    Bundled,
}

impl ExtensionSourceKind {
    /// Lower ranks win unless one exact source-qualified identity is selected.
    pub const fn precedence_rank(self) -> u8 {
        match self {
            Self::ProjectLocal => 0,
            Self::WorkspaceLocal => 1,
            Self::UserGlobal => 2,
            Self::PluginProvided => 3,
            Self::Bundled => 4,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillQualifiedIdentity {
    pub source_kind: ExtensionSourceKind,
    pub source_id: String,
    pub skill_id: String,
}

impl SkillQualifiedIdentity {
    pub fn stable_id(&self) -> String {
        format!(
            "{}:{}:{}",
            match self.source_kind {
                ExtensionSourceKind::ProjectLocal => "project",
                ExtensionSourceKind::WorkspaceLocal => "workspace",
                ExtensionSourceKind::UserGlobal => "user",
                ExtensionSourceKind::PluginProvided => "plugin",
                ExtensionSourceKind::Bundled => "bundled",
            },
            self.source_id,
            self.skill_id
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketplaceSnapshot {
    pub marketplace_id: String,
    pub label: String,
    /// Sanitized display source. Credential-bearing source material is not
    /// represented by this contract.
    pub source: String,
    pub git_ref: Option<String>,
    /// Exact immutable Git commit consumed by this refresh. Raw local
    /// directories deliberately leave this unset.
    pub resolved_commit: Option<String>,
    pub sparse_paths: Vec<String>,
    pub catalog_digest: String,
    pub status: String,
    pub trusted_origin: String,
    pub origin_key_id: String,
    pub package_count: u32,
    pub last_checked_at_ms: Option<u64>,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginHookSnapshot {
    pub hook_id: String,
    pub name: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub event: String,
    pub review_digest: String,
    pub reviewed: bool,
    pub status: String,
    pub grants: Vec<String>,
    pub last_result: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginSettingSnapshot {
    pub setting_id: String,
    pub label: String,
    pub description: String,
    pub kind: String,
    pub required: bool,
    pub choices: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginAppSnapshot {
    pub app_id: String,
    pub title: String,
    pub summary: String,
    pub setting_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginMcpServerSnapshot {
    pub server_id: String,
    pub name: String,
    pub transport: String,
    pub setting_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginSnapshot {
    pub package_id: String,
    pub package_kind: ExtensionPackageKind,
    pub name: String,
    pub summary: String,
    pub publisher: String,
    pub version: String,
    pub digest: String,
    pub origin_key_id: String,
    pub content_key_id: String,
    pub trust: ExtensionTrustState,
    pub lifecycle: ExtensionLifecycle,
    pub capabilities: Vec<String>,
    pub website: Option<String>,
    pub terms: Option<String>,
    pub privacy_policy: Option<String>,
    pub active_digest: Option<String>,
    pub last_known_good_digest: Option<String>,
    pub failure_code: Option<String>,
    pub has_reviewed_hooks: bool,
    pub marketplace_id: String,
    pub source: String,
    pub compatibility: String,
    pub verified_at_ms: Option<u64>,
    pub hooks: Vec<PluginHookSnapshot>,
    pub settings: Vec<PluginSettingSnapshot>,
    pub apps: Vec<PluginAppSnapshot>,
    pub mcp_servers: Vec<PluginMcpServerSnapshot>,
    pub available_version: Option<String>,
    pub available_digest: Option<String>,
    pub staged_version: Option<String>,
    pub staged_digest: Option<String>,
    pub active_version: Option<String>,
    pub last_known_good_version: Option<String>,
    pub revocation_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillSnapshot {
    pub identity: SkillQualifiedIdentity,
    pub name: String,
    pub summary: String,
    pub package_id: Option<String>,
    pub source_label: String,
    pub enabled: bool,
    pub eligible: bool,
    pub valid: bool,
    pub active: bool,
    pub shadowed: bool,
    pub instructions_loaded: bool,
    pub collision_sources: Vec<SkillQualifiedIdentity>,
    pub diagnostic: Option<String>,
    pub version: String,
    pub is_installed: bool,
    pub eligibility_detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionServiceSnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub marketplaces: Vec<MarketplaceSnapshot>,
    pub plugins: Vec<PluginSnapshot>,
    pub skills: Vec<SkillSnapshot>,
    pub selected_skill: Option<SkillQualifiedIdentity>,
    pub active_workers: u16,
    pub last_event_id: u64,
}

impl ExtensionServiceSnapshot {
    pub fn validate(&self) -> Result<(), ExtensionError> {
        if self.schema_version != EXTENSION_STATE_SCHEMA_VERSION || self.generation == 0 {
            return Err(ExtensionError::InvalidState);
        }
        if self.plugins.len() > MAX_EXTENSION_RECORDS
            || self.skills.len() > MAX_EXTENSION_SKILLS
            || self.marketplaces.len() > MAX_EXTENSION_RECORDS
        {
            return Err(ExtensionError::BoundExceeded);
        }
        let mut marketplace_ids = BTreeSet::new();
        for marketplace in &self.marketplaces {
            validate_identifier(&marketplace.marketplace_id)?;
            validate_text(&marketplace.label)?;
            validate_text(&marketplace.source)?;
            validate_digest(&marketplace.catalog_digest)?;
            validate_text(&marketplace.status)?;
            validate_text(&marketplace.trusted_origin)?;
            validate_identifier(&marketplace.origin_key_id)?;
            validate_text(&marketplace.detail)?;
            if marketplace.package_count as usize > MAX_EXTENSION_RECORDS
                || !marketplace_ids.insert(marketplace.marketplace_id.as_str())
                || marketplace.git_ref.as_deref().is_some_and(|git_ref| {
                    git_ref.is_empty()
                        || git_ref.len() > 255
                        || git_ref.chars().any(char::is_control)
                })
                || marketplace
                    .resolved_commit
                    .as_deref()
                    .is_some_and(|commit| {
                        !matches!(commit.len(), 40 | 64)
                            || !commit.bytes().all(|byte| byte.is_ascii_hexdigit())
                    })
                || marketplace.sparse_paths.len() > 128
                || marketplace.sparse_paths.iter().any(|path| {
                    path.is_empty() || path.len() > 1_024 || path.chars().any(char::is_control)
                })
            {
                return Err(ExtensionError::InvalidState);
            }
        }
        let mut package_ids = BTreeSet::new();
        for plugin in &self.plugins {
            validate_identifier(&plugin.package_id)?;
            validate_text(&plugin.name)?;
            validate_text(&plugin.summary)?;
            validate_text(&plugin.publisher)?;
            validate_digest(&plugin.digest)?;
            validate_identifier(&plugin.origin_key_id)?;
            validate_identifier(&plugin.content_key_id)?;
            validate_identifier(&plugin.marketplace_id)?;
            validate_text(&plugin.source)?;
            validate_text(&plugin.compatibility)?;
            if plugin.capabilities.len() > MAX_EXTENSION_CAPABILITIES
                || plugin.settings.len() > MAX_EXTENSION_RECORDS
                || plugin.apps.len() > MAX_EXTENSION_RECORDS
                || plugin.mcp_servers.len() > MAX_EXTENSION_RECORDS
                || !package_ids.insert(plugin.package_id.as_str())
            {
                return Err(ExtensionError::InvalidState);
            }
            let setting_ids = plugin
                .settings
                .iter()
                .map(|setting| setting.setting_id.as_str())
                .collect::<BTreeSet<_>>();
            if setting_ids.len() != plugin.settings.len() {
                return Err(ExtensionError::InvalidState);
            }
            for setting in &plugin.settings {
                validate_identifier(&setting.setting_id)?;
                validate_text(&setting.label)?;
                validate_text(&setting.description)?;
                if !matches!(
                    setting.kind.as_str(),
                    "boolean" | "integer" | "string" | "select" | "credentialReference"
                ) || setting.choices.len() > 64
                {
                    return Err(ExtensionError::InvalidState);
                }
                for choice in &setting.choices {
                    validate_text(choice)?;
                }
            }
            for app in &plugin.apps {
                validate_identifier(&app.app_id)?;
                validate_text(&app.title)?;
                validate_text(&app.summary)?;
                if app
                    .setting_ids
                    .iter()
                    .any(|setting_id| !setting_ids.contains(setting_id.as_str()))
                {
                    return Err(ExtensionError::InvalidState);
                }
            }
            for server in &plugin.mcp_servers {
                validate_identifier(&server.server_id)?;
                validate_text(&server.name)?;
                if !matches!(server.transport.as_str(), "stdio" | "streamableHttp")
                    || server
                        .setting_ids
                        .iter()
                        .any(|setting_id| !setting_ids.contains(setting_id.as_str()))
                {
                    return Err(ExtensionError::InvalidState);
                }
            }
            for hook in &plugin.hooks {
                validate_identifier(&hook.hook_id)?;
                validate_identifier(&hook.event)?;
                validate_digest(&hook.review_digest)?;
                if hook.grants.len() > MAX_EXTENSION_CAPABILITIES || hook.arguments.len() > 32 {
                    return Err(ExtensionError::InvalidState);
                }
                for argument in &hook.arguments {
                    if argument.is_empty()
                        || argument.len() > 1_024
                        || argument.chars().any(char::is_control)
                    {
                        return Err(ExtensionError::InvalidState);
                    }
                }
                for grant in &hook.grants {
                    validate_identifier(grant)?;
                }
            }
        }
        let mut skill_ids = BTreeSet::new();
        for skill in &self.skills {
            validate_identifier(&skill.identity.skill_id)?;
            validate_identifier(&skill.identity.source_id)?;
            validate_text(&skill.name)?;
            validate_text(&skill.summary)?;
            if !skill_ids.insert(skill.identity.stable_id()) {
                return Err(ExtensionError::InvalidState);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionAuditEvent {
    pub schema_version: u16,
    pub event_id: u64,
    pub generation: u64,
    pub operation_id: String,
    pub package_id: Option<String>,
    pub event_kind: String,
    pub selector_digest: Option<String>,
    pub result: String,
    pub occurred_at_ms: u64,
}

#[derive(Debug, Error)]
pub enum ExtensionError {
    #[error("extension input is invalid")]
    InvalidInput,
    #[error("extension package or state is invalid")]
    InvalidState,
    #[error("extension package exceeds a bounded limit")]
    BoundExceeded,
    #[error("extension content is not immutable")]
    MutableContent,
    #[error("extension signature or digest verification failed")]
    VerificationFailed,
    #[error("extension origin is not trusted")]
    UntrustedOrigin,
    #[error("extension origin or content is revoked")]
    Revoked,
    #[error("extension compatibility is unsupported")]
    Incompatible,
    #[error("extension generation or selector is stale")]
    Conflict,
    #[error("extension hook is unavailable on this target")]
    UnsupportedHookTarget,
    #[error("extension hook was denied")]
    HookDenied,
    #[error("extension hook timed out")]
    HookTimedOut,
    #[error("extension hook output exceeded its bound")]
    HookOutputExceeded,
    #[error("extension service is unavailable")]
    Unavailable,
    #[error("extension persistence failed: {0}")]
    Database(#[from] crate::core::database::DatabaseError),
    #[error("extension package I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("extension package TOML failed: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("extension JSON failed: {0}")]
    Json(#[from] serde_json::Error),
}

pub(crate) fn validate_identifier(value: &str) -> Result<(), ExtensionError> {
    if value.is_empty()
        || value.len() > 160
        || value.starts_with('-')
        || value.starts_with('.')
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_text(value: &str) -> Result<(), ExtensionError> {
    if value.is_empty()
        || value.len() > MAX_EXTENSION_TEXT_BYTES
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

pub(crate) fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(7 + digest.len() * 2);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded
}

pub(crate) fn validate_digest(value: &str) -> Result<(), ExtensionError> {
    if value.len() != 71
        || !value.starts_with("sha256:")
        || !value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}
