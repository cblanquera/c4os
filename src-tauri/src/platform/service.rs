use crate::protocol::{PickerGrantId, RequestId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const PLATFORM_CONTRACT_VERSION: u16 = 1;
pub const PICKER_CONTRACT_VERSION: u16 = 1;
pub const MAX_PICKER_SELECTIONS: usize = 64;
pub const INITIAL_REVEAL_FALLBACK_MS: u64 = 5_000;
pub const SETTINGS_MENU_ITEM_ID: &str = "c4os.menu.settings";
pub const OPEN_SETTINGS_COMMAND_ID: &str = "c4os.command.openSettings";
pub const SETTINGS_ROUTE: &str = "/settings/providers";
pub const SETTINGS_ACCELERATOR: &str = "CmdOrCtrl+,";

const SUPPORTED_OS: &str = "macos";
const SUPPORTED_ARCHITECTURE: &str = "aarch64";
const SETTINGS_KEYBOARD_LABEL: &str = "⌘,";
const MAX_DISPLAY_NAME_BYTES: usize = 512;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlatformTarget {
    pub os: String,
    pub architecture: String,
}

impl PlatformTarget {
    pub fn new(os: impl Into<String>, architecture: impl Into<String>) -> Self {
        Self {
            os: os.into(),
            architecture: architecture.into(),
        }
    }

    pub fn current_build() -> Self {
        Self::new(std::env::consts::OS, std::env::consts::ARCH)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformFamily {
    Macos,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InitialThemeSource {
    #[serde(rename = "macosAppearance")]
    MacosAppearance,
    WebviewPreferredColorScheme,
    SemanticFallback,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveThemeSource {
    WebviewPrefersColorScheme,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InitialThemeSnapshot {
    pub scheme: ColorScheme,
    pub source: InitialThemeSource,
}

impl InitialThemeSnapshot {
    pub fn new(scheme: ColorScheme, source: InitialThemeSource) -> Self {
        Self { scheme, source }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowDecorations {
    Standard,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WindowChromeSnapshot {
    pub decorations: WindowDecorations,
    pub titlebar_transparent: bool,
    pub titlebar_overlay: bool,
    pub initially_visible: bool,
    pub reveal_fallback_timeout_ms: u64,
}

impl Default for WindowChromeSnapshot {
    fn default() -> Self {
        Self {
            decorations: WindowDecorations::Standard,
            titlebar_transparent: false,
            titlebar_overlay: false,
            initially_visible: false,
            reveal_fallback_timeout_ms: INITIAL_REVEAL_FALLBACK_MS,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlatformVocabulary {
    pub reveal_action: String,
    pub primary_modifier_symbol: String,
    pub alternate_modifier_symbol: String,
    pub shift_modifier_symbol: String,
}

impl Default for PlatformVocabulary {
    fn default() -> Self {
        Self {
            reveal_action: "Reveal in Finder".into(),
            primary_modifier_symbol: "⌘".into(),
            alternate_modifier_symbol: "⌥".into(),
            shift_modifier_symbol: "⇧".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub native_application_menu: bool,
    pub native_settings_shortcut: bool,
    pub native_file_picker: bool,
    pub native_folder_picker: bool,
    pub native_workspace_picker: bool,
    pub standard_window_decorations: bool,
}

impl PlatformCapabilities {
    fn validate(self) -> Result<(), PlatformError> {
        if self.native_settings_shortcut && !self.native_application_menu {
            return Err(PlatformError::InvalidCapabilities(
                "native Settings shortcut requires the native application menu",
            ));
        }
        if self.native_workspace_picker && !self.native_file_picker {
            return Err(PlatformError::InvalidCapabilities(
                "native Workspace picker requires the native file picker",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NativeMenuItemId {
    #[serde(rename = "c4os.menu.settings")]
    Settings,
}

impl NativeMenuItemId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Settings => SETTINGS_MENU_ITEM_ID,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NativeCommandId {
    #[serde(rename = "c4os.command.openSettings")]
    OpenSettings,
}

impl NativeCommandId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenSettings => OPEN_SETTINGS_COMMAND_ID,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SettingsMenuContract {
    pub menu_item_id: NativeMenuItemId,
    pub command_id: NativeCommandId,
    pub route: String,
    pub accelerator: String,
    pub keyboard_label: String,
}

impl Default for SettingsMenuContract {
    fn default() -> Self {
        Self {
            menu_item_id: NativeMenuItemId::Settings,
            command_id: NativeCommandId::OpenSettings,
            route: SETTINGS_ROUTE.into(),
            accelerator: SETTINGS_ACCELERATOR.into(),
            keyboard_label: SETTINGS_KEYBOARD_LABEL.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlatformSnapshot {
    pub contract_version: u16,
    pub platform: PlatformFamily,
    pub architecture: String,
    pub initial_theme: InitialThemeSnapshot,
    pub live_theme_source: LiveThemeSource,
    pub window: WindowChromeSnapshot,
    pub vocabulary: PlatformVocabulary,
    pub capabilities: PlatformCapabilities,
    pub settings_menu: SettingsMenuContract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformService {
    target: PlatformTarget,
    capabilities: PlatformCapabilities,
}

impl PlatformService {
    pub fn qualify(target: PlatformTarget) -> Result<Self, PlatformError> {
        Self::qualify_installed(target, PlatformCapabilities::default())
    }

    /// Qualifies a target plus only those native facilities already installed
    /// by the Tauri composition root. The ordinary qualifier reports no
    /// facilities, so a target match alone never turns capability flags on.
    pub fn qualify_installed(
        target: PlatformTarget,
        capabilities: PlatformCapabilities,
    ) -> Result<Self, PlatformError> {
        if target.os != SUPPORTED_OS || target.architecture != SUPPORTED_ARCHITECTURE {
            return Err(PlatformError::UnsupportedTarget {
                os: target.os,
                architecture: target.architecture,
            });
        }
        capabilities.validate()?;
        Ok(Self {
            target,
            capabilities,
        })
    }

    pub fn for_current_build() -> Result<Self, PlatformError> {
        Self::qualify(PlatformTarget::current_build())
    }

    pub fn target(&self) -> &PlatformTarget {
        &self.target
    }

    /// Resolves the complete Rust-owned snapshot that must be available before
    /// the renderer reveals its shell. Live changes remain independently owned
    /// by the webview `prefers-color-scheme` observer.
    pub fn initial_snapshot(&self, theme: InitialThemeSnapshot) -> PlatformSnapshot {
        PlatformSnapshot {
            contract_version: PLATFORM_CONTRACT_VERSION,
            platform: PlatformFamily::Macos,
            architecture: self.target.architecture.clone(),
            initial_theme: theme,
            live_theme_source: LiveThemeSource::WebviewPrefersColorScheme,
            window: WindowChromeSnapshot::default(),
            vocabulary: PlatformVocabulary::default(),
            capabilities: self.capabilities,
            settings_menu: SettingsMenuContract::default(),
        }
    }

    pub fn picker_request(
        &self,
        request_id: RequestId,
        purpose: PickerPurpose,
    ) -> NativePickerRequest {
        NativePickerRequest::new(request_id, purpose)
    }

    pub fn cancelled_picker(
        &self,
        request: &NativePickerRequest,
    ) -> Result<PickerOutcome, PlatformError> {
        request.validate()?;
        Ok(PickerOutcome::Cancelled {
            contract_version: PICKER_CONTRACT_VERSION,
            request_id: request.request_id.clone(),
        })
    }

    /// Correlates native-only paths with opaque grants minted by the core.
    /// Paths are validated for the request but are never copied into the
    /// serializable result.
    pub fn selected_picker(
        &self,
        request: &NativePickerRequest,
        selections: &[NativePickerSelection],
        grants: Vec<PickerGrantSnapshot>,
    ) -> Result<PickerOutcome, PlatformError> {
        request.validate()?;
        validate_native_selections(request, selections)?;
        if grants.len() != selections.len() {
            return Err(PlatformError::InvalidPickerGrant(
                "picker grants must correspond one-to-one with native selections",
            ));
        }

        let mut grant_ids = BTreeSet::new();
        for (selection, grant) in selections.iter().zip(&grants) {
            grant.validate()?;
            if grant.object_kind != selection.object_kind {
                return Err(PlatformError::InvalidPickerGrant(
                    "picker grant kind does not match its native selection",
                ));
            }
            if !grant_ids.insert(grant.grant_id.as_str()) {
                return Err(PlatformError::InvalidPickerGrant(
                    "picker grant identifiers must be unique",
                ));
            }
        }

        Ok(PickerOutcome::Selected {
            contract_version: PICKER_CONTRACT_VERSION,
            request_id: request.request_id.clone(),
            grants,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PickerPurpose {
    OpenProjectFolder,
    RelocateProjectFolder,
    OpenWorkspaceArchive,
    SaveWorkspaceArchive,
    AttachChatFiles,
    OpenFile,
    OpenFolder,
}

impl PickerPurpose {
    pub fn policy(self) -> PickerSelectionPolicy {
        match self {
            Self::OpenProjectFolder | Self::RelocateProjectFolder | Self::OpenFolder => {
                PickerSelectionPolicy {
                    object_kind: PickerObjectKind::Folder,
                    allows_multiple: false,
                    allowed_extensions: Vec::new(),
                }
            }
            Self::OpenWorkspaceArchive | Self::SaveWorkspaceArchive => PickerSelectionPolicy {
                object_kind: PickerObjectKind::File,
                allows_multiple: false,
                allowed_extensions: vec!["zip".into()],
            },
            Self::AttachChatFiles => PickerSelectionPolicy {
                object_kind: PickerObjectKind::File,
                allows_multiple: true,
                allowed_extensions: Vec::new(),
            },
            Self::OpenFile => PickerSelectionPolicy {
                object_kind: PickerObjectKind::File,
                allows_multiple: false,
                allowed_extensions: Vec::new(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PickerObjectKind {
    File,
    Folder,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PickerSelectionPolicy {
    pub object_kind: PickerObjectKind,
    pub allows_multiple: bool,
    pub allowed_extensions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NativePickerRequest {
    pub contract_version: u16,
    pub request_id: RequestId,
    pub purpose: PickerPurpose,
    pub selection: PickerSelectionPolicy,
}

impl NativePickerRequest {
    pub fn new(request_id: RequestId, purpose: PickerPurpose) -> Self {
        Self {
            contract_version: PICKER_CONTRACT_VERSION,
            request_id,
            purpose,
            selection: purpose.policy(),
        }
    }

    pub fn validate(&self) -> Result<(), PlatformError> {
        if self.contract_version != PICKER_CONTRACT_VERSION {
            return Err(PlatformError::UnsupportedContractVersion {
                expected: PICKER_CONTRACT_VERSION,
                actual: self.contract_version,
            });
        }
        if !is_valid_protocol_identifier(self.request_id.as_str()) {
            return Err(PlatformError::InvalidPickerRequest(
                "picker request identifier is invalid",
            ));
        }
        if self.selection != self.purpose.policy() {
            return Err(PlatformError::InvalidPickerRequest(
                "picker selection policy does not match its purpose",
            ));
        }
        Ok(())
    }
}

/// A native-only path selected by the operating-system picker. This type is
/// intentionally not serializable; the core must replace it with an opaque
/// picker grant before replying across the application boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePickerSelection {
    path: PathBuf,
    object_kind: PickerObjectKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePickerGrant {
    request_id: RequestId,
    purpose: PickerPurpose,
    selection: NativePickerSelection,
    issued_at_ms: u64,
}

impl NativePickerGrant {
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    pub fn purpose(&self) -> PickerPurpose {
        self.purpose
    }

    pub fn path(&self) -> &Path {
        self.selection.path()
    }

    pub fn object_kind(&self) -> PickerObjectKind {
        self.selection.object_kind()
    }

    pub fn issued_at_ms(&self) -> u64 {
        self.issued_at_ms
    }
}

/// Rust-only, process-local picker authority. Raw paths never enter its
/// serializable snapshots, and taking a grant consumes it exactly once.
#[derive(Debug, Default)]
pub struct PickerGrantRegistry {
    grants: BTreeMap<PickerGrantId, NativePickerGrant>,
}

impl PickerGrantRegistry {
    pub fn register_batch(
        &mut self,
        request: &NativePickerRequest,
        selections: &[NativePickerSelection],
        grant_ids: Vec<PickerGrantId>,
        issued_at_ms: u64,
    ) -> Result<Vec<PickerGrantSnapshot>, PlatformError> {
        request.validate()?;
        validate_native_selections(request, selections)?;
        if selections.len() != grant_ids.len() {
            return Err(PlatformError::InvalidPickerGrant(
                "picker grants must correspond one-to-one with native selections",
            ));
        }

        let mut pending_ids = BTreeSet::new();
        for grant_id in &grant_ids {
            if !is_valid_protocol_identifier(grant_id.as_str()) {
                return Err(PlatformError::InvalidPickerGrant(
                    "picker grant identifier is invalid",
                ));
            }
            if self.grants.contains_key(grant_id) || !pending_ids.insert(grant_id) {
                return Err(PlatformError::PickerGrantConflict);
            }
        }

        let snapshots = selections
            .iter()
            .zip(&grant_ids)
            .map(|(selection, grant_id)| {
                PickerGrantSnapshot::new(
                    grant_id.clone(),
                    selection.object_kind(),
                    display_name_for(selection),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        for ((selection, grant_id), _) in selections.iter().zip(grant_ids).zip(&snapshots) {
            self.grants.insert(
                grant_id,
                NativePickerGrant {
                    request_id: request.request_id.clone(),
                    purpose: request.purpose,
                    selection: selection.clone(),
                    issued_at_ms,
                },
            );
        }
        Ok(snapshots)
    }

    pub fn take(&mut self, grant_id: &PickerGrantId) -> Option<NativePickerGrant> {
        self.grants.remove(grant_id)
    }

    /// Consumes a complete ordered grant set atomically. Missing or repeated
    /// identifiers leave the registry unchanged, so a partial attachment
    /// submission cannot strand otherwise valid picker authority.
    pub fn take_batch(
        &mut self,
        grant_ids: &[PickerGrantId],
    ) -> Result<Vec<NativePickerGrant>, PlatformError> {
        let unique = grant_ids.iter().collect::<BTreeSet<_>>();
        if grant_ids.is_empty()
            || unique.len() != grant_ids.len()
            || grant_ids
                .iter()
                .any(|grant_id| !self.grants.contains_key(grant_id))
        {
            return Err(PlatformError::InvalidPickerGrant(
                "picker grant batch is missing or repeated",
            ));
        }
        Ok(grant_ids
            .iter()
            .map(|grant_id| {
                self.grants
                    .remove(grant_id)
                    .expect("validated grant remains present until removal")
            })
            .collect())
    }

    pub fn len(&self) -> usize {
        self.grants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }
}

fn display_name_for(selection: &NativePickerSelection) -> String {
    selection
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(match selection.object_kind() {
            PickerObjectKind::File => "Selected file",
            PickerObjectKind::Folder => "Selected folder",
        })
        .to_owned()
}

impl NativePickerSelection {
    pub fn new(
        path: impl Into<PathBuf>,
        object_kind: PickerObjectKind,
    ) -> Result<Self, PlatformError> {
        let path = path.into();
        if !path.is_absolute() {
            return Err(PlatformError::InvalidPickerSelection(
                "native picker selections must be absolute",
            ));
        }
        Ok(Self { path, object_kind })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn object_kind(&self) -> PickerObjectKind {
        self.object_kind
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PickerGrantSnapshot {
    pub grant_id: PickerGrantId,
    pub object_kind: PickerObjectKind,
    pub display_name: String,
}

impl PickerGrantSnapshot {
    pub fn new(
        grant_id: PickerGrantId,
        object_kind: PickerObjectKind,
        display_name: impl Into<String>,
    ) -> Result<Self, PlatformError> {
        let grant = Self {
            grant_id,
            object_kind,
            display_name: display_name.into(),
        };
        grant.validate()?;
        Ok(grant)
    }

    pub fn validate(&self) -> Result<(), PlatformError> {
        if !is_valid_protocol_identifier(self.grant_id.as_str()) {
            return Err(PlatformError::InvalidPickerGrant(
                "picker grant identifier is invalid",
            ));
        }
        let display_name = self.display_name.trim();
        if display_name.is_empty()
            || display_name.len() > MAX_DISPLAY_NAME_BYTES
            || display_name.chars().any(char::is_control)
            || display_name.contains('/')
            || display_name.contains('\\')
        {
            return Err(PlatformError::InvalidPickerGrant(
                "picker grant display name is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    deny_unknown_fields,
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PickerOutcome {
    Cancelled {
        contract_version: u16,
        request_id: RequestId,
    },
    Selected {
        contract_version: u16,
        request_id: RequestId,
        grants: Vec<PickerGrantSnapshot>,
    },
}

impl PickerOutcome {
    pub fn validate_for(&self, request: &NativePickerRequest) -> Result<(), PlatformError> {
        request.validate()?;
        let (contract_version, request_id) = match self {
            Self::Cancelled {
                contract_version,
                request_id,
            }
            | Self::Selected {
                contract_version,
                request_id,
                ..
            } => (*contract_version, request_id),
        };
        if contract_version != PICKER_CONTRACT_VERSION {
            return Err(PlatformError::UnsupportedContractVersion {
                expected: PICKER_CONTRACT_VERSION,
                actual: contract_version,
            });
        }
        if request_id != &request.request_id {
            return Err(PlatformError::PickerCorrelationMismatch);
        }
        if let Self::Selected { grants, .. } = self {
            validate_grant_cardinality(request, grants)?;
            let mut grant_ids = BTreeSet::new();
            for grant in grants {
                grant.validate()?;
                if grant.object_kind != request.selection.object_kind {
                    return Err(PlatformError::InvalidPickerGrant(
                        "picker grant kind does not match the request",
                    ));
                }
                if !grant_ids.insert(grant.grant_id.as_str()) {
                    return Err(PlatformError::InvalidPickerGrant(
                        "picker grant identifiers must be unique",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn validate_native_selections(
    request: &NativePickerRequest,
    selections: &[NativePickerSelection],
) -> Result<(), PlatformError> {
    if selections.is_empty() || selections.len() > MAX_PICKER_SELECTIONS {
        return Err(PlatformError::InvalidPickerSelection(
            "native picker selection count is invalid",
        ));
    }
    if !request.selection.allows_multiple && selections.len() != 1 {
        return Err(PlatformError::InvalidPickerSelection(
            "picker purpose permits exactly one selection",
        ));
    }
    let mut paths = BTreeSet::new();
    for selection in selections {
        if selection.object_kind != request.selection.object_kind {
            return Err(PlatformError::InvalidPickerSelection(
                "native picker selection kind does not match the request",
            ));
        }
        if !selection.path.is_absolute() {
            return Err(PlatformError::InvalidPickerSelection(
                "native picker selections must be absolute",
            ));
        }
        if !request.selection.allowed_extensions.is_empty()
            && !selection
                .path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    request
                        .selection
                        .allowed_extensions
                        .iter()
                        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
                })
        {
            return Err(PlatformError::InvalidPickerSelection(
                "native picker selection extension does not match the request",
            ));
        }
        if !paths.insert(selection.path.as_path()) {
            return Err(PlatformError::InvalidPickerSelection(
                "native picker selections must be unique",
            ));
        }
    }
    Ok(())
}

fn validate_grant_cardinality(
    request: &NativePickerRequest,
    grants: &[PickerGrantSnapshot],
) -> Result<(), PlatformError> {
    if grants.is_empty() || grants.len() > MAX_PICKER_SELECTIONS {
        return Err(PlatformError::InvalidPickerGrant(
            "picker grant count is invalid",
        ));
    }
    if !request.selection.allows_multiple && grants.len() != 1 {
        return Err(PlatformError::InvalidPickerGrant(
            "picker purpose permits exactly one grant",
        ));
    }
    Ok(())
}

fn is_valid_protocol_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

#[derive(Debug, Eq, Error, PartialEq)]
pub enum PlatformError {
    #[error("unsupported native target {os}/{architecture}")]
    UnsupportedTarget { os: String, architecture: String },
    #[error("unsupported platform contract version {actual}; expected {expected}")]
    UnsupportedContractVersion { expected: u16, actual: u16 },
    #[error("invalid picker request: {0}")]
    InvalidPickerRequest(&'static str),
    #[error("invalid native picker selection: {0}")]
    InvalidPickerSelection(&'static str),
    #[error("invalid picker grant: {0}")]
    InvalidPickerGrant(&'static str),
    #[error("invalid installed platform capabilities: {0}")]
    InvalidCapabilities(&'static str),
    #[error("picker outcome does not match its request")]
    PickerCorrelationMismatch,
    #[error("picker grant identifier is already active")]
    PickerGrantConflict,
}
