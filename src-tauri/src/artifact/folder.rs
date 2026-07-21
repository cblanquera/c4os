use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use thiserror::Error;

use super::file::{FileLiveVersion, MAX_PROJECT_RELATIVE_PATH_BYTES};
use super::record::{self, ArtifactResourceVersion, MAX_ARTIFACT_DISPLAY_BYTES};

pub const MAX_FOLDER_ENTRIES: usize = 512;
pub const MAX_FOLDER_BREADCRUMBS: usize = 128;
pub const MAX_FOLDER_ENTRY_METADATA_BYTES: usize = 1_024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderResourceReference {
    /// Empty means the authoritative Project root. Non-empty values are
    /// slash-delimited paths below it and never ambient filesystem authority.
    pub project_relative_path: String,
    pub display_path: String,
}

impl FolderResourceReference {
    pub fn new(
        project_relative_path: impl Into<String>,
        display_path: impl Into<String>,
    ) -> Result<Self, FolderStateError> {
        let reference = Self {
            project_relative_path: project_relative_path.into(),
            display_path: display_path.into(),
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn validate(&self) -> Result<(), FolderStateError> {
        validate_folder_relative_path(&self.project_relative_path)?;
        validate_bounded_text(
            "folder display path",
            &self.display_path,
            MAX_PROJECT_RELATIVE_PATH_BYTES,
            false,
        )?;
        if self.display_path.chars().any(char::is_control) {
            return Err(FolderStateError::InvalidDisplayPath);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderBreadcrumb {
    pub label: String,
    pub project_relative_path: String,
}

impl FolderBreadcrumb {
    pub fn validate(&self) -> Result<(), FolderStateError> {
        validate_bounded_text(
            "folder breadcrumb label",
            &self.label,
            MAX_ARTIFACT_DISPLAY_BYTES,
            false,
        )?;
        if self.label.chars().any(char::is_control)
            || self.label.contains('/')
            || self.label.contains('\\')
        {
            return Err(FolderStateError::InvalidBreadcrumb);
        }
        validate_folder_relative_path(&self.project_relative_path)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FolderEntryKind {
    Folder,
    File,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderEntry {
    pub entry_id: String,
    pub name: String,
    pub kind: FolderEntryKind,
    pub project_relative_path: String,
    pub display_metadata: Option<String>,
    pub byte_length: Option<u64>,
    pub modified_at_ms: Option<u64>,
    pub content_sha256: Option<String>,
}

impl FolderEntry {
    pub fn validate(&self, parent: &str) -> Result<(), FolderStateError> {
        validate_identifier("folder entry id", &self.entry_id)?;
        validate_bounded_text(
            "folder entry name",
            &self.name,
            MAX_ARTIFACT_DISPLAY_BYTES,
            false,
        )?;
        if self.name.chars().any(char::is_control)
            || self.name.contains('/')
            || self.name.contains('\\')
            || matches!(self.name.as_str(), "." | "..")
        {
            return Err(FolderStateError::InvalidEntry);
        }
        validate_folder_relative_path(&self.project_relative_path)?;
        let expected = join_relative(parent, &self.name);
        if self.project_relative_path != expected {
            return Err(FolderStateError::RecursiveOrMismatchedEntry);
        }
        if let Some(metadata) = &self.display_metadata {
            validate_bounded_text(
                "folder entry metadata",
                metadata,
                MAX_FOLDER_ENTRY_METADATA_BYTES,
                true,
            )?;
        }
        if self.modified_at_ms == Some(0) {
            return Err(FolderStateError::InvalidEntry);
        }
        match self.kind {
            FolderEntryKind::Folder
                if self.byte_length.is_some() || self.content_sha256.is_some() =>
            {
                Err(FolderStateError::InvalidEntry)
            }
            FolderEntryKind::File => {
                if let Some(digest) = &self.content_sha256 {
                    validate_sha256(digest)?;
                }
                Ok(())
            }
            FolderEntryKind::Folder => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderListingVersion {
    pub sequence: u64,
    /// Exact opaque descriptor identity returned by
    /// `ProjectFolderListing::target_version()`. This remains distinct from
    /// the canonical entry digest so a restart can retain the filesystem's
    /// conflict identity without trusting rendered listing data.
    pub target_version: String,
    pub listing_sha256: String,
    pub observed_at_ms: u64,
}

impl FolderListingVersion {
    pub fn from_entries(
        sequence: u64,
        entries: &[FolderEntry],
        observed_at_ms: u64,
    ) -> Result<Self, FolderStateError> {
        let listing_sha256 = listing_sha256(entries);
        Self::from_target_version(sequence, listing_sha256.clone(), entries, observed_at_ms)
    }

    /// Production constructor that preserves the exact filesystem listing
    /// descriptor independently of the canonical artifact entry digest.
    pub fn from_target_version(
        sequence: u64,
        target_version: impl Into<String>,
        entries: &[FolderEntry],
        observed_at_ms: u64,
    ) -> Result<Self, FolderStateError> {
        let version = Self {
            sequence,
            target_version: target_version.into(),
            listing_sha256: listing_sha256(entries),
            observed_at_ms,
        };
        version.validate()?;
        Ok(version)
    }

    pub fn validate(&self) -> Result<(), FolderStateError> {
        if self.sequence == 0 || self.observed_at_ms == 0 {
            return Err(FolderStateError::InvalidListingVersion);
        }
        validate_sha256(&self.target_version)?;
        validate_sha256(&self.listing_sha256)?;
        Ok(())
    }

    pub fn as_resource_version(&self) -> ArtifactResourceVersion {
        ArtifactResourceVersion {
            sequence: self.sequence,
            sha256: self.target_version.clone(),
            observed_at_ms: self.observed_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderSelection {
    pub entry_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderToFileConversion {
    pub source_folder_relative_path: String,
    pub selected_entry_id: String,
    pub target_file_relative_path: String,
    pub captured_file_version: Option<FileLiveVersion>,
    pub converted_at_ms: u64,
}

impl FolderToFileConversion {
    fn validate(&self) -> Result<(), FolderStateError> {
        validate_folder_relative_path(&self.source_folder_relative_path)?;
        validate_identifier("selected folder entry id", &self.selected_entry_id)?;
        if self.target_file_relative_path.is_empty() {
            return Err(FolderStateError::InvalidConversion);
        }
        validate_folder_relative_path(&self.target_file_relative_path)?;
        if let Some(version) = &self.captured_file_version {
            version
                .validate()
                .map_err(|_| FolderStateError::InvalidCapturedFileVersion)?;
        }
        if self.converted_at_ms == 0 {
            return Err(FolderStateError::InvalidConversion);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderArtifactState {
    pub resource: FolderResourceReference,
    pub breadcrumbs: Vec<FolderBreadcrumb>,
    pub entries: Vec<FolderEntry>,
    pub listing_version: FolderListingVersion,
    pub selection: Option<FolderSelection>,
    pub conversion: Option<FolderToFileConversion>,
}

impl FolderArtifactState {
    pub fn new(
        resource: FolderResourceReference,
        breadcrumbs: Vec<FolderBreadcrumb>,
        entries: Vec<FolderEntry>,
        listing_version: FolderListingVersion,
    ) -> Result<Self, FolderStateError> {
        let state = Self {
            resource,
            breadcrumbs,
            entries,
            listing_version,
            selection: None,
            conversion: None,
        };
        state.validate()?;
        Ok(state)
    }

    pub fn validate(&self) -> Result<(), FolderStateError> {
        self.resource.validate()?;
        validate_breadcrumbs(&self.breadcrumbs, &self.resource.project_relative_path)?;
        validate_entries(&self.entries, &self.resource.project_relative_path)?;
        self.listing_version.validate()?;
        if self.listing_version.listing_sha256 != listing_sha256(&self.entries) {
            return Err(FolderStateError::ListingVersionMismatch);
        }
        if let Some(selection) = &self.selection {
            validate_identifier("folder selection id", &selection.entry_id)?;
            if !self
                .entries
                .iter()
                .any(|entry| entry.entry_id == selection.entry_id)
            {
                return Err(FolderStateError::UnknownSelection);
            }
        }
        if let Some(conversion) = &self.conversion {
            conversion.validate()?;
            let selected = self
                .entries
                .iter()
                .find(|entry| entry.entry_id == conversion.selected_entry_id)
                .ok_or(FolderStateError::InvalidConversion)?;
            if selected.kind != FolderEntryKind::File
                || conversion.source_folder_relative_path != self.resource.project_relative_path
                || conversion.target_file_relative_path != selected.project_relative_path
            {
                return Err(FolderStateError::InvalidConversion);
            }
        }
        Ok(())
    }

    pub fn navigate(&mut self, navigation: FolderNavigation) -> Result<(), FolderStateError> {
        navigation.validate()?;
        self.resource = navigation.resource;
        self.breadcrumbs = navigation.breadcrumbs;
        self.entries = navigation.entries;
        self.listing_version = navigation.listing_version;
        self.selection = None;
        self.conversion = None;
        Ok(())
    }

    pub fn select(&mut self, entry_id: &str) -> Result<(), FolderStateError> {
        validate_identifier("folder selection id", entry_id)?;
        if !self.entries.iter().any(|entry| entry.entry_id == entry_id) {
            return Err(FolderStateError::UnknownSelection);
        }
        self.selection = Some(FolderSelection {
            entry_id: entry_id.into(),
        });
        self.conversion = None;
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.conversion = None;
    }

    pub fn convert_selected_file(
        &mut self,
        captured_file_version: Option<FileLiveVersion>,
        converted_at_ms: u64,
    ) -> Result<FolderToFileConversion, FolderStateError> {
        if let Some(version) = &captured_file_version {
            version
                .validate()
                .map_err(|_| FolderStateError::InvalidCapturedFileVersion)?;
        }
        if converted_at_ms == 0 {
            return Err(FolderStateError::InvalidConversion);
        }
        let selection = self
            .selection
            .as_ref()
            .ok_or(FolderStateError::UnknownSelection)?;
        let selected = self
            .entries
            .iter()
            .find(|entry| entry.entry_id == selection.entry_id)
            .ok_or(FolderStateError::UnknownSelection)?;
        if selected.kind != FolderEntryKind::File {
            return Err(FolderStateError::SelectionIsNotFile);
        }
        let conversion = FolderToFileConversion {
            source_folder_relative_path: self.resource.project_relative_path.clone(),
            selected_entry_id: selected.entry_id.clone(),
            target_file_relative_path: selected.project_relative_path.clone(),
            captured_file_version,
            converted_at_ms,
        };
        conversion.validate()?;
        self.conversion = Some(conversion.clone());
        Ok(conversion)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FolderNavigation {
    pub resource: FolderResourceReference,
    pub breadcrumbs: Vec<FolderBreadcrumb>,
    pub entries: Vec<FolderEntry>,
    pub listing_version: FolderListingVersion,
}

impl FolderNavigation {
    pub fn validate(&self) -> Result<(), FolderStateError> {
        self.resource.validate()?;
        validate_breadcrumbs(&self.breadcrumbs, &self.resource.project_relative_path)?;
        validate_entries(&self.entries, &self.resource.project_relative_path)?;
        self.listing_version.validate()?;
        if self.listing_version.listing_sha256 != listing_sha256(&self.entries) {
            return Err(FolderStateError::ListingVersionMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FolderStateError {
    #[error("invalid Folder artifact identifier: {0}")]
    InvalidIdentifier(&'static str),
    #[error("invalid bounded Folder artifact text: {0}")]
    InvalidText(&'static str),
    #[error("invalid Folder artifact SHA-256 value")]
    InvalidSha256,
    #[error("invalid Project-relative folder path")]
    InvalidProjectRelativePath,
    #[error("invalid folder display path")]
    InvalidDisplayPath,
    #[error("invalid folder breadcrumb")]
    InvalidBreadcrumb,
    #[error("folder breadcrumbs do not describe the current path")]
    BreadcrumbMismatch,
    #[error("folder entry is invalid")]
    InvalidEntry,
    #[error("folder entry is recursive or has a mismatched parent")]
    RecursiveOrMismatchedEntry,
    #[error("folder entries exceed their bound")]
    EntryBoundExceeded,
    #[error("folder entries must be unique and deterministically sorted")]
    NonDeterministicEntries,
    #[error("folder listing version is invalid")]
    InvalidListingVersion,
    #[error("folder entries do not match the listing version")]
    ListingVersionMismatch,
    #[error("folder selection is unavailable")]
    UnknownSelection,
    #[error("folder selection is not a file")]
    SelectionIsNotFile,
    #[error("folder to File conversion is invalid")]
    InvalidConversion,
    #[error("folder to File conversion captured an invalid File version")]
    InvalidCapturedFileVersion,
}

fn validate_identifier(label: &'static str, value: &str) -> Result<(), FolderStateError> {
    record::validate_identifier(label, value)
        .map_err(|_| FolderStateError::InvalidIdentifier(label))
}

fn validate_bounded_text(
    label: &'static str,
    value: &str,
    maximum_bytes: usize,
    allow_empty: bool,
) -> Result<(), FolderStateError> {
    record::validate_bounded_text(label, value, maximum_bytes, allow_empty)
        .map_err(|_| FolderStateError::InvalidText(label))
}

fn validate_sha256(value: &str) -> Result<(), FolderStateError> {
    record::validate_sha256(value).map_err(|_| FolderStateError::InvalidSha256)
}

fn validate_folder_relative_path(path: &str) -> Result<(), FolderStateError> {
    if path.len() > MAX_PROJECT_RELATIVE_PATH_BYTES
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains('\0')
        || (!path.is_empty()
            && path.split('/').any(|segment| {
                segment.is_empty()
                    || matches!(segment, "." | "..")
                    || segment.chars().any(char::is_control)
            }))
    {
        Err(FolderStateError::InvalidProjectRelativePath)
    } else {
        Ok(())
    }
}

fn validate_breadcrumbs(
    breadcrumbs: &[FolderBreadcrumb],
    current_path: &str,
) -> Result<(), FolderStateError> {
    if breadcrumbs.is_empty() || breadcrumbs.len() > MAX_FOLDER_BREADCRUMBS {
        return Err(FolderStateError::BreadcrumbMismatch);
    }
    let mut previous: Option<&str> = None;
    let mut paths = BTreeSet::new();
    for breadcrumb in breadcrumbs {
        breadcrumb.validate()?;
        if !paths.insert(breadcrumb.project_relative_path.as_str()) {
            return Err(FolderStateError::BreadcrumbMismatch);
        }
        if let Some(parent) = previous
            && parent_path(&breadcrumb.project_relative_path) != Some(parent)
        {
            return Err(FolderStateError::BreadcrumbMismatch);
        }
        previous = Some(&breadcrumb.project_relative_path);
    }
    if previous != Some(current_path) {
        return Err(FolderStateError::BreadcrumbMismatch);
    }
    Ok(())
}

fn validate_entries(entries: &[FolderEntry], parent: &str) -> Result<(), FolderStateError> {
    if entries.len() > MAX_FOLDER_ENTRIES {
        return Err(FolderStateError::EntryBoundExceeded);
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut previous_key: Option<(String, String, FolderEntryKind)> = None;
    for entry in entries {
        entry.validate(parent)?;
        if !ids.insert(entry.entry_id.as_str()) || !names.insert(entry.name.to_ascii_lowercase()) {
            return Err(FolderStateError::NonDeterministicEntries);
        }
        let key = (
            entry.name.to_ascii_lowercase(),
            entry.name.clone(),
            entry.kind,
        );
        if previous_key
            .as_ref()
            .is_some_and(|previous| previous >= &key)
        {
            return Err(FolderStateError::NonDeterministicEntries);
        }
        previous_key = Some(key);
    }
    Ok(())
}

fn listing_sha256(entries: &[FolderEntry]) -> String {
    let mut hasher = Sha256::new();
    for entry in entries {
        for value in [
            entry.entry_id.as_str(),
            entry.name.as_str(),
            entry.project_relative_path.as_str(),
        ] {
            hasher.update((value.len() as u64).to_be_bytes());
            hasher.update(value.as_bytes());
        }
        hasher.update([match entry.kind {
            FolderEntryKind::Folder => 0,
            FolderEntryKind::File => 1,
        }]);
        hash_optional_text(&mut hasher, entry.display_metadata.as_deref());
        hash_optional_u64(&mut hasher, entry.byte_length);
        hash_optional_u64(&mut hasher, entry.modified_at_ms);
        hash_optional_text(&mut hasher, entry.content_sha256.as_deref());
    }
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in hasher.finalize() {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded
}

fn hash_optional_text(hasher: &mut Sha256, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hasher.update((value.len() as u64).to_be_bytes());
            hasher.update(value.as_bytes());
        }
        None => hasher.update([0]),
    }
}

fn hash_optional_u64(hasher: &mut Sha256, value: Option<u64>) {
    match value {
        Some(value) => {
            hasher.update([1]);
            hasher.update(value.to_be_bytes());
        }
        None => hasher.update([0]),
    }
}

fn join_relative(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.into()
    } else {
        format!("{parent}/{name}")
    }
}

fn parent_path(path: &str) -> Option<&str> {
    path.rsplit_once('/').map_or_else(
        || (!path.is_empty()).then_some(""),
        |(parent, _)| Some(parent),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::file::sha256_text;

    fn file_entry(id: &str, name: &str, parent: &str) -> FolderEntry {
        FolderEntry {
            entry_id: id.into(),
            name: name.into(),
            kind: FolderEntryKind::File,
            project_relative_path: join_relative(parent, name),
            display_metadata: Some("Markdown".into()),
            byte_length: Some(4),
            modified_at_ms: Some(10),
            content_sha256: Some(sha256_text("test")),
        }
    }

    fn folder_entry(id: &str, name: &str, parent: &str) -> FolderEntry {
        FolderEntry {
            entry_id: id.into(),
            name: name.into(),
            kind: FolderEntryKind::Folder,
            project_relative_path: join_relative(parent, name),
            display_metadata: None,
            byte_length: None,
            modified_at_ms: Some(10),
            content_sha256: None,
        }
    }

    fn state() -> FolderArtifactState {
        let entries = vec![
            folder_entry("entry-api", "api", "docs"),
            file_entry("entry-plan", "plan.md", "docs"),
        ];
        FolderArtifactState::new(
            FolderResourceReference::new("docs", "docs").unwrap(),
            vec![
                FolderBreadcrumb {
                    label: "Project".into(),
                    project_relative_path: String::new(),
                },
                FolderBreadcrumb {
                    label: "docs".into(),
                    project_relative_path: "docs".into(),
                },
            ],
            entries.clone(),
            FolderListingVersion::from_entries(1, &entries, 10).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn production_listing_version_keeps_target_identity_separate_from_integrity() {
        let entries = vec![file_entry("entry-plan", "plan.md", "docs")];
        let target_version = format!("sha256:{}", "f".repeat(64));
        let version =
            FolderListingVersion::from_target_version(3, target_version.clone(), &entries, 20)
                .unwrap();

        assert_eq!(version.target_version, target_version);
        assert_ne!(version.listing_sha256, version.target_version);
        assert_eq!(version.as_resource_version().sha256, target_version);
        assert_eq!(
            FolderListingVersion::from_target_version(3, "not-a-digest", &entries, 20),
            Err(FolderStateError::InvalidSha256)
        );
    }

    #[test]
    fn listing_is_bounded_non_recursive_and_deterministic() {
        let mut folder = state();
        folder.entries.swap(0, 1);
        assert_eq!(
            folder.validate(),
            Err(FolderStateError::NonDeterministicEntries)
        );

        let mut recursive = state();
        recursive.entries[0].project_relative_path = "docs/api/nested".into();
        assert_eq!(
            recursive.validate(),
            Err(FolderStateError::RecursiveOrMismatchedEntry)
        );

        let entries = vec![
            file_entry("entry-a", "Plan.md", "docs"),
            file_entry("entry-b", "plan.md", "docs"),
        ];
        assert_eq!(
            validate_entries(&entries, "docs"),
            Err(FolderStateError::NonDeterministicEntries)
        );
    }

    #[test]
    fn navigation_replaces_listing_and_clears_selection() {
        let mut folder = state();
        folder.select("entry-api").unwrap();
        let entries = vec![file_entry("entry-schema", "schema.json", "docs/api")];
        folder
            .navigate(FolderNavigation {
                resource: FolderResourceReference::new("docs/api", "docs/api").unwrap(),
                breadcrumbs: vec![
                    FolderBreadcrumb {
                        label: "Project".into(),
                        project_relative_path: String::new(),
                    },
                    FolderBreadcrumb {
                        label: "docs".into(),
                        project_relative_path: "docs".into(),
                    },
                    FolderBreadcrumb {
                        label: "api".into(),
                        project_relative_path: "docs/api".into(),
                    },
                ],
                listing_version: FolderListingVersion::from_entries(1, &entries, 20).unwrap(),
                entries,
            })
            .unwrap();
        assert_eq!(folder.resource.project_relative_path, "docs/api");
        assert_eq!(folder.selection, None);
    }

    #[test]
    fn file_selection_produces_explicit_conversion_metadata() {
        let mut folder = state();
        folder.select("entry-plan").unwrap();
        let conversion = folder.convert_selected_file(None, 20).unwrap();
        assert_eq!(conversion.source_folder_relative_path, "docs");
        assert_eq!(conversion.target_file_relative_path, "docs/plan.md");
        folder.validate().unwrap();
    }

    #[test]
    fn folder_selection_cannot_convert_to_file() {
        let mut folder = state();
        folder.select("entry-api").unwrap();
        assert_eq!(
            folder.convert_selected_file(None, 20),
            Err(FolderStateError::SelectionIsNotFile)
        );
    }
}
