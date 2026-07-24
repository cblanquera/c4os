//! Strict package/catalog parsing and cryptographic verification.
//!
//! A package is a directory whose `c4os-package.toml` inventory describes
//! every other regular file. The manifest is deliberately excluded from the
//! content digest so signatures can be embedded without a circular digest.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ed25519_dalek::{Signature, VerifyingKey};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::{
    EXTENSION_CATALOG_SCHEMA_VERSION, EXTENSION_PACKAGE_SCHEMA_VERSION, ExtensionError,
    ExtensionPackageKind, MAX_EXTENSION_CAPABILITIES, MAX_EXTENSION_RECORDS, validate_digest,
    validate_identifier, validate_text,
};

pub const PACKAGE_MANIFEST_NAME: &str = "c4os-package.toml";
pub const MAX_PACKAGE_FILES: usize = 2_048;
pub const MAX_PACKAGE_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_PACKAGE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u64 = 512 * 1024;

const ORIGIN_SIGNATURE_DOMAIN: &[u8] = b"C4OS-PACKAGE-ORIGIN-V1\0";
const CONTENT_SIGNATURE_DOMAIN: &[u8] = b"C4OS-PACKAGE-CONTENT-V1\0";
const CATALOG_SIGNATURE_DOMAIN: &[u8] = b"C4OS-CATALOG-V1\0";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageManifest {
    pub schema_version: u16,
    pub package: PackageMetadata,
    pub compatibility: PackageCompatibility,
    pub trust: PackageTrust,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub skills: Vec<PackageSkillDeclaration>,
    #[serde(default)]
    pub hooks: Vec<PackageHookDeclaration>,
    #[serde(default)]
    pub settings: Vec<PackageSettingDeclaration>,
    #[serde(default)]
    pub apps: Vec<PackageAppDeclaration>,
    #[serde(default)]
    pub mcp_servers: Vec<PackageMcpServerDeclaration>,
    pub inventory: Vec<PackageInventoryEntry>,
    pub package_digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageMetadata {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub publisher: String,
    pub version: String,
    pub kind: ExtensionPackageKind,
    pub website: Option<String>,
    pub terms: Option<String>,
    pub privacy_policy: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageCompatibility {
    pub c4os: String,
    #[serde(default)]
    pub operating_systems: Vec<String>,
    #[serde(default)]
    pub architectures: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageTrust {
    pub origin: String,
    pub origin_key_id: String,
    pub delegated_content_key_id: String,
    pub delegated_content_public_key: String,
    pub origin_signature: String,
    pub content_signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageInventoryEntry {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSkillDeclaration {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageHookDeclaration {
    pub id: String,
    pub event: String,
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub review_digest: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PackageSettingKind {
    Boolean,
    Integer,
    String,
    Select,
    CredentialReference,
}

/// Host-rendered setting metadata. Package manifests never contain a raw
/// sensitive value; `CredentialReference` accepts only an opaque vault record
/// reference when Task 00013 composes the editor.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSettingDeclaration {
    pub id: String,
    pub label: String,
    pub description: String,
    pub kind: PackageSettingKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Vec<String>,
}

/// A host-owned declarative app contribution. It can reference setting IDs,
/// but cannot name renderer code, a native library, or an executable.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageAppDeclaration {
    pub id: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub settings: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PackageMcpTransport {
    Stdio,
    StreamableHttp,
}

/// Package-owned MCP declaration metadata. Task 00012 owns executable
/// configuration and supervision; this declaration only reserves a strict,
/// signed host-readable identity and its opaque setting references.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageMcpServerDeclaration {
    pub id: String,
    pub name: String,
    pub transport: PackageMcpTransport,
    #[serde(default)]
    pub settings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogDocument {
    pub schema_version: u16,
    pub marketplace: CatalogMarketplace,
    pub releases: Vec<CatalogRelease>,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogMarketplace {
    pub id: String,
    pub label: String,
    pub origin: String,
    pub signing_key_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogRelease {
    pub package_id: String,
    pub version: String,
    pub manifest_path: String,
    pub package_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedOrigin {
    pub origin: String,
    pub key_id: String,
    /// Raw 32-byte Ed25519 public key encoded as standard base64.
    pub public_key: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RevocationSet {
    pub key_ids: BTreeSet<String>,
    pub package_digests: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct PackageVerificationPolicy<'a> {
    pub c4os_version: &'a str,
    pub operating_system: &'a str,
    pub architecture: &'a str,
    pub trusted_origins: &'a [TrustedOrigin],
    pub revocations: &'a RevocationSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedPackage {
    pub root: PathBuf,
    pub manifest: PackageManifest,
    pub inventory: Vec<PackageInventoryEntry>,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedCatalog {
    pub document: CatalogDocument,
    pub digest: String,
}

pub fn parse_manifest(input: &str) -> Result<PackageManifest, ExtensionError> {
    let manifest: PackageManifest = toml::from_str(input)?;
    validate_manifest_shape(&manifest)?;
    Ok(manifest)
}

pub fn parse_catalog(input: &str) -> Result<CatalogDocument, ExtensionError> {
    let catalog: CatalogDocument = toml::from_str(input)?;
    validate_catalog_shape(&catalog)?;
    Ok(catalog)
}

/// Verifies a local package without executing or dynamically loading any of it.
pub fn verify_package_directory(
    root: &Path,
    policy: &PackageVerificationPolicy<'_>,
) -> Result<VerifiedPackage, ExtensionError> {
    reject_symlink_or_special(root, true)?;
    let root = root.canonicalize()?;
    if !fs::symlink_metadata(&root)?.file_type().is_dir() {
        return Err(ExtensionError::InvalidInput);
    }

    let manifest_path = root.join(PACKAGE_MANIFEST_NAME);
    reject_symlink_or_special(&manifest_path, false)?;
    let manifest_metadata = fs::metadata(&manifest_path)?;
    if manifest_metadata.len() > MAX_MANIFEST_BYTES {
        return Err(ExtensionError::BoundExceeded);
    }
    let manifest = parse_manifest(&fs::read_to_string(&manifest_path)?)?;
    let inventory = inventory_for_root(&root)?;
    if inventory != manifest.inventory {
        return Err(ExtensionError::VerificationFailed);
    }
    let digest = digest_inventory(&inventory);
    if digest != manifest.package_digest {
        return Err(ExtensionError::VerificationFailed);
    }

    verify_compatibility(&manifest.compatibility, policy)?;
    verify_manifest_signatures(&manifest, policy)?;

    Ok(VerifiedPackage {
        root,
        manifest,
        inventory,
        digest,
    })
}

pub fn verify_catalog(
    input: &str,
    trusted_origins: &[TrustedOrigin],
    revocations: &RevocationSet,
) -> Result<VerifiedCatalog, ExtensionError> {
    let catalog = parse_catalog(input)?;
    let authority = trusted_origins
        .iter()
        .find(|authority| {
            authority.origin == catalog.marketplace.origin
                && authority.key_id == catalog.marketplace.signing_key_id
        })
        .ok_or(ExtensionError::UntrustedOrigin)?;
    if revocations.key_ids.contains(&authority.key_id) {
        return Err(ExtensionError::Revoked);
    }
    let bytes = canonical_catalog_signing_bytes(&catalog);
    verify_ed25519(&authority.public_key, &catalog.signature, &bytes)?;
    let digest = sha256_prefixed(&bytes);
    Ok(VerifiedCatalog {
        document: catalog,
        digest,
    })
}

pub fn inventory_for_root(root: &Path) -> Result<Vec<PackageInventoryEntry>, ExtensionError> {
    let root = root.canonicalize()?;
    let mut inventory = Vec::new();
    let mut total_bytes = 0u64;

    for entry in WalkDir::new(&root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
    {
        let entry = entry.map_err(|error| {
            ExtensionError::Io(error.into_io_error().unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid package tree")
            }))
        })?;
        let path = entry.path();
        if path == root {
            continue;
        }
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err(ExtensionError::InvalidInput);
        }
        if metadata.is_dir() {
            continue;
        }
        if !metadata.is_file() {
            return Err(ExtensionError::InvalidInput);
        }
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| ExtensionError::InvalidInput)?;
        let normalized = normalize_relative_path(relative)?;
        if normalized == PACKAGE_MANIFEST_NAME {
            continue;
        }
        reject_ambient_code_surface(&normalized)?;
        if metadata.len() > MAX_PACKAGE_FILE_BYTES {
            return Err(ExtensionError::BoundExceeded);
        }
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or(ExtensionError::BoundExceeded)?;
        if total_bytes > MAX_PACKAGE_BYTES || inventory.len() >= MAX_PACKAGE_FILES {
            return Err(ExtensionError::BoundExceeded);
        }
        let bytes = fs::read(path)?;
        inventory.push(PackageInventoryEntry {
            path: normalized,
            bytes: metadata.len(),
            sha256: sha256_prefixed(&bytes),
        });
    }
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    if inventory.is_empty() {
        return Err(ExtensionError::InvalidState);
    }
    Ok(inventory)
}

pub fn digest_inventory(inventory: &[PackageInventoryEntry]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"C4OS-PACKAGE-INVENTORY-V1\0");
    append_u64(&mut bytes, inventory.len() as u64);
    for entry in inventory {
        append_field(&mut bytes, &entry.path);
        append_u64(&mut bytes, entry.bytes);
        append_field(&mut bytes, &entry.sha256);
    }
    sha256_prefixed(&bytes)
}

pub fn canonical_origin_signing_bytes(manifest: &PackageManifest) -> Vec<u8> {
    let mut bytes = ORIGIN_SIGNATURE_DOMAIN.to_vec();
    append_manifest_common(&mut bytes, manifest);
    append_field(&mut bytes, &manifest.trust.origin);
    append_field(&mut bytes, &manifest.trust.origin_key_id);
    append_field(&mut bytes, &manifest.trust.delegated_content_key_id);
    append_field(&mut bytes, &manifest.trust.delegated_content_public_key);
    bytes
}

pub fn canonical_content_signing_bytes(manifest: &PackageManifest) -> Vec<u8> {
    let mut bytes = CONTENT_SIGNATURE_DOMAIN.to_vec();
    append_field(&mut bytes, &manifest.package.id);
    append_field(&mut bytes, &manifest.package.version);
    append_field(&mut bytes, &manifest.package_digest);
    bytes
}

pub fn canonical_catalog_signing_bytes(catalog: &CatalogDocument) -> Vec<u8> {
    let mut bytes = CATALOG_SIGNATURE_DOMAIN.to_vec();
    append_u64(&mut bytes, catalog.schema_version as u64);
    append_field(&mut bytes, &catalog.marketplace.id);
    append_field(&mut bytes, &catalog.marketplace.label);
    append_field(&mut bytes, &catalog.marketplace.origin);
    append_field(&mut bytes, &catalog.marketplace.signing_key_id);
    append_u64(&mut bytes, catalog.releases.len() as u64);
    for release in &catalog.releases {
        append_field(&mut bytes, &release.package_id);
        append_field(&mut bytes, &release.version);
        append_field(&mut bytes, &release.manifest_path);
        append_field(&mut bytes, &release.package_digest);
    }
    bytes
}

fn validate_manifest_shape(manifest: &PackageManifest) -> Result<(), ExtensionError> {
    if manifest.schema_version != EXTENSION_PACKAGE_SCHEMA_VERSION {
        return Err(ExtensionError::InvalidState);
    }
    validate_identifier(&manifest.package.id)?;
    validate_text(&manifest.package.name)?;
    validate_text(&manifest.package.summary)?;
    validate_text(&manifest.package.publisher)?;
    Version::parse(&manifest.package.version).map_err(|_| ExtensionError::InvalidInput)?;
    validate_identifier(&manifest.trust.origin_key_id)?;
    validate_identifier(&manifest.trust.delegated_content_key_id)?;
    validate_origin(&manifest.trust.origin)?;
    validate_digest(&manifest.package_digest)?;
    if manifest.capabilities.len() > MAX_EXTENSION_CAPABILITIES
        || manifest.skills.len() > MAX_EXTENSION_RECORDS
        || manifest.hooks.len() > MAX_EXTENSION_RECORDS
        || manifest.settings.len() > MAX_EXTENSION_RECORDS
        || manifest.apps.len() > MAX_EXTENSION_RECORDS
        || manifest.mcp_servers.len() > MAX_EXTENSION_RECORDS
        || manifest.inventory.is_empty()
        || manifest.inventory.len() > MAX_PACKAGE_FILES
    {
        return Err(ExtensionError::BoundExceeded);
    }

    let mut capabilities = BTreeSet::new();
    for capability in &manifest.capabilities {
        validate_identifier(capability)?;
        if !capabilities.insert(capability) {
            return Err(ExtensionError::InvalidState);
        }
    }
    let mut skill_ids = BTreeSet::new();
    for skill in &manifest.skills {
        validate_identifier(&skill.id)?;
        validate_package_path(&skill.path)?;
        if !skill_ids.insert(&skill.id) {
            return Err(ExtensionError::InvalidState);
        }
    }
    let mut hook_ids = BTreeSet::new();
    for hook in &manifest.hooks {
        validate_identifier(&hook.id)?;
        validate_identifier(&hook.event)?;
        validate_package_path(&hook.executable)?;
        validate_digest(&hook.review_digest)?;
        if hook.arguments.len() > 32
            || hook.arguments.iter().any(|argument| {
                argument.is_empty()
                    || argument.len() > 1_024
                    || argument.chars().any(char::is_control)
            })
            || !hook_ids.insert(&hook.id)
        {
            return Err(ExtensionError::InvalidState);
        }
    }
    let mut setting_ids = BTreeSet::new();
    for setting in &manifest.settings {
        validate_identifier(&setting.id)?;
        validate_text(&setting.label)?;
        validate_text(&setting.description)?;
        if !setting_ids.insert(setting.id.as_str())
            || setting.choices.len() > 64
            || (setting.kind == PackageSettingKind::Select && setting.choices.len() < 2)
            || (setting.kind != PackageSettingKind::Select && !setting.choices.is_empty())
        {
            return Err(ExtensionError::InvalidState);
        }
        let mut choices = BTreeSet::new();
        for choice in &setting.choices {
            validate_text(choice)?;
            if !choices.insert(choice.as_str()) {
                return Err(ExtensionError::InvalidState);
            }
        }
    }
    let mut app_ids = BTreeSet::new();
    for app in &manifest.apps {
        validate_identifier(&app.id)?;
        validate_text(&app.title)?;
        validate_text(&app.summary)?;
        validate_setting_references(&app.settings, &setting_ids)?;
        if !app_ids.insert(app.id.as_str()) {
            return Err(ExtensionError::InvalidState);
        }
    }
    let mut mcp_ids = BTreeSet::new();
    for server in &manifest.mcp_servers {
        validate_identifier(&server.id)?;
        validate_text(&server.name)?;
        validate_setting_references(&server.settings, &setting_ids)?;
        if !mcp_ids.insert(server.id.as_str()) {
            return Err(ExtensionError::InvalidState);
        }
    }
    match manifest.package.kind {
        ExtensionPackageKind::Skill
            if manifest.skills.is_empty()
                || !manifest.hooks.is_empty()
                || !manifest.apps.is_empty()
                || !manifest.mcp_servers.is_empty() =>
        {
            return Err(ExtensionError::InvalidState);
        }
        ExtensionPackageKind::Plugin
            if manifest.skills.is_empty()
                && manifest.hooks.is_empty()
                && manifest.settings.is_empty()
                && manifest.apps.is_empty()
                && manifest.mcp_servers.is_empty() =>
        {
            return Err(ExtensionError::InvalidState);
        }
        _ => {}
    }

    let mut previous: Option<&str> = None;
    let mut total = 0u64;
    for entry in &manifest.inventory {
        validate_package_path(&entry.path)?;
        validate_digest(&entry.sha256)?;
        if entry.bytes > MAX_PACKAGE_FILE_BYTES
            || previous.is_some_and(|path| path >= entry.path.as_str())
        {
            return Err(ExtensionError::InvalidState);
        }
        total = total
            .checked_add(entry.bytes)
            .ok_or(ExtensionError::BoundExceeded)?;
        previous = Some(&entry.path);
    }
    if total > MAX_PACKAGE_BYTES {
        return Err(ExtensionError::BoundExceeded);
    }
    let indexed: BTreeMap<&str, &PackageInventoryEntry> = manifest
        .inventory
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    if manifest
        .skills
        .iter()
        .any(|skill| !indexed.contains_key(skill.path.as_str()))
        || manifest
            .hooks
            .iter()
            .any(|hook| !indexed.contains_key(hook.executable.as_str()))
    {
        return Err(ExtensionError::InvalidState);
    }
    Ok(())
}

fn validate_catalog_shape(catalog: &CatalogDocument) -> Result<(), ExtensionError> {
    if catalog.schema_version != EXTENSION_CATALOG_SCHEMA_VERSION
        || catalog.releases.is_empty()
        || catalog.releases.len() > MAX_EXTENSION_RECORDS
    {
        return Err(ExtensionError::InvalidState);
    }
    validate_identifier(&catalog.marketplace.id)?;
    validate_identifier(&catalog.marketplace.signing_key_id)?;
    validate_text(&catalog.marketplace.label)?;
    validate_origin(&catalog.marketplace.origin)?;
    let mut previous: Option<(&str, &str)> = None;
    for release in &catalog.releases {
        validate_identifier(&release.package_id)?;
        Version::parse(&release.version).map_err(|_| ExtensionError::InvalidInput)?;
        validate_package_path(&release.manifest_path)?;
        validate_digest(&release.package_digest)?;
        let current = (release.package_id.as_str(), release.version.as_str());
        if previous.is_some_and(|value| value >= current) {
            return Err(ExtensionError::InvalidState);
        }
        previous = Some(current);
    }
    Ok(())
}

fn verify_manifest_signatures(
    manifest: &PackageManifest,
    policy: &PackageVerificationPolicy<'_>,
) -> Result<(), ExtensionError> {
    if policy
        .revocations
        .package_digests
        .contains(&manifest.package_digest)
        || policy
            .revocations
            .key_ids
            .contains(&manifest.trust.origin_key_id)
        || policy
            .revocations
            .key_ids
            .contains(&manifest.trust.delegated_content_key_id)
    {
        return Err(ExtensionError::Revoked);
    }
    let authority = policy
        .trusted_origins
        .iter()
        .find(|authority| {
            authority.origin == manifest.trust.origin
                && authority.key_id == manifest.trust.origin_key_id
        })
        .ok_or(ExtensionError::UntrustedOrigin)?;
    verify_ed25519(
        &authority.public_key,
        &manifest.trust.origin_signature,
        &canonical_origin_signing_bytes(manifest),
    )?;
    verify_ed25519(
        &manifest.trust.delegated_content_public_key,
        &manifest.trust.content_signature,
        &canonical_content_signing_bytes(manifest),
    )
}

fn verify_compatibility(
    compatibility: &PackageCompatibility,
    policy: &PackageVerificationPolicy<'_>,
) -> Result<(), ExtensionError> {
    let requirement =
        VersionReq::parse(&compatibility.c4os).map_err(|_| ExtensionError::InvalidInput)?;
    let version = Version::parse(policy.c4os_version).map_err(|_| ExtensionError::InvalidInput)?;
    if !requirement.matches(&version)
        || (!compatibility.operating_systems.is_empty()
            && !compatibility
                .operating_systems
                .iter()
                .any(|value| value == policy.operating_system))
        || (!compatibility.architectures.is_empty()
            && !compatibility
                .architectures
                .iter()
                .any(|value| value == policy.architecture))
    {
        return Err(ExtensionError::Incompatible);
    }
    Ok(())
}

fn verify_ed25519(
    public_key: &str,
    encoded_signature: &str,
    message: &[u8],
) -> Result<(), ExtensionError> {
    let public_key = BASE64
        .decode(public_key)
        .map_err(|_| ExtensionError::VerificationFailed)?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| ExtensionError::VerificationFailed)?;
    let signature = BASE64
        .decode(encoded_signature)
        .map_err(|_| ExtensionError::VerificationFailed)?;
    let signature =
        Signature::from_slice(&signature).map_err(|_| ExtensionError::VerificationFailed)?;
    let verifying_key =
        VerifyingKey::from_bytes(&public_key).map_err(|_| ExtensionError::VerificationFailed)?;
    verifying_key
        .verify_strict(message, &signature)
        .map_err(|_| ExtensionError::VerificationFailed)
}

fn append_manifest_common(bytes: &mut Vec<u8>, manifest: &PackageManifest) {
    append_u64(bytes, manifest.schema_version as u64);
    append_field(bytes, &manifest.package.id);
    append_field(bytes, &manifest.package.name);
    append_field(bytes, &manifest.package.summary);
    append_field(bytes, &manifest.package.publisher);
    append_field(bytes, &manifest.package.version);
    append_field(
        bytes,
        match manifest.package.kind {
            ExtensionPackageKind::Plugin => "plugin",
            ExtensionPackageKind::Skill => "skill",
        },
    );
    append_optional(bytes, manifest.package.website.as_deref());
    append_optional(bytes, manifest.package.terms.as_deref());
    append_optional(bytes, manifest.package.privacy_policy.as_deref());
    append_field(bytes, &manifest.compatibility.c4os);
    append_strings(bytes, &manifest.compatibility.operating_systems);
    append_strings(bytes, &manifest.compatibility.architectures);
    append_strings(bytes, &manifest.capabilities);
    append_u64(bytes, manifest.skills.len() as u64);
    for skill in &manifest.skills {
        append_field(bytes, &skill.id);
        append_field(bytes, &skill.path);
    }
    append_u64(bytes, manifest.hooks.len() as u64);
    for hook in &manifest.hooks {
        append_field(bytes, &hook.id);
        append_field(bytes, &hook.event);
        append_field(bytes, &hook.executable);
        append_strings(bytes, &hook.arguments);
        append_field(bytes, &hook.review_digest);
    }
    append_u64(bytes, manifest.settings.len() as u64);
    for setting in &manifest.settings {
        append_field(bytes, &setting.id);
        append_field(bytes, &setting.label);
        append_field(bytes, &setting.description);
        append_field(
            bytes,
            match setting.kind {
                PackageSettingKind::Boolean => "boolean",
                PackageSettingKind::Integer => "integer",
                PackageSettingKind::String => "string",
                PackageSettingKind::Select => "select",
                PackageSettingKind::CredentialReference => "credentialReference",
            },
        );
        bytes.push(u8::from(setting.required));
        append_strings(bytes, &setting.choices);
    }
    append_u64(bytes, manifest.apps.len() as u64);
    for app in &manifest.apps {
        append_field(bytes, &app.id);
        append_field(bytes, &app.title);
        append_field(bytes, &app.summary);
        append_strings(bytes, &app.settings);
    }
    append_u64(bytes, manifest.mcp_servers.len() as u64);
    for server in &manifest.mcp_servers {
        append_field(bytes, &server.id);
        append_field(bytes, &server.name);
        append_field(
            bytes,
            match server.transport {
                PackageMcpTransport::Stdio => "stdio",
                PackageMcpTransport::StreamableHttp => "streamableHttp",
            },
        );
        append_strings(bytes, &server.settings);
    }
    append_u64(bytes, manifest.inventory.len() as u64);
    for entry in &manifest.inventory {
        append_field(bytes, &entry.path);
        append_u64(bytes, entry.bytes);
        append_field(bytes, &entry.sha256);
    }
    append_field(bytes, &manifest.package_digest);
}

fn append_strings(bytes: &mut Vec<u8>, values: &[String]) {
    append_u64(bytes, values.len() as u64);
    for value in values {
        append_field(bytes, value);
    }
}

fn validate_setting_references<'a>(
    references: &'a [String],
    declared: &BTreeSet<&'a str>,
) -> Result<(), ExtensionError> {
    if references.len() > MAX_EXTENSION_RECORDS {
        return Err(ExtensionError::BoundExceeded);
    }
    let mut unique = BTreeSet::new();
    for reference in references {
        validate_identifier(reference)?;
        if !declared.contains(reference.as_str()) || !unique.insert(reference.as_str()) {
            return Err(ExtensionError::InvalidState);
        }
    }
    Ok(())
}

fn append_optional(bytes: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            bytes.push(1);
            append_field(bytes, value);
        }
        None => bytes.push(0),
    }
}

fn append_field(bytes: &mut Vec<u8>, value: &str) {
    append_u64(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn append_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    super::sha256_prefixed(bytes)
}

pub(crate) fn validate_origin(value: &str) -> Result<(), ExtensionError> {
    if value.is_empty()
        || value.len() > 2_048
        || value.chars().any(char::is_control)
        || value.contains('@')
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

pub fn public_key_fingerprint(encoded_public_key: &str) -> Result<String, ExtensionError> {
    let public_key = BASE64
        .decode(encoded_public_key)
        .map_err(|_| ExtensionError::VerificationFailed)?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| ExtensionError::VerificationFailed)?;
    Ok(super::sha256_prefixed(&public_key))
}

fn validate_package_path(value: &str) -> Result<(), ExtensionError> {
    if value.is_empty() || value.len() > 1_024 || value.contains('\\') {
        return Err(ExtensionError::InvalidInput);
    }
    normalize_relative_path(Path::new(value)).map(|_| ())
}

fn normalize_relative_path(path: &Path) -> Result<String, ExtensionError> {
    if path.is_absolute() {
        return Err(ExtensionError::InvalidInput);
    }
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                let segment = segment.to_str().ok_or(ExtensionError::InvalidInput)?;
                if segment.is_empty() || segment == "." || segment == ".." {
                    return Err(ExtensionError::InvalidInput);
                }
                segments.push(segment);
            }
            _ => return Err(ExtensionError::InvalidInput),
        }
    }
    if segments.is_empty() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(segments.join("/"))
}

fn reject_ambient_code_surface(path: &str) -> Result<(), ExtensionError> {
    let lower = path.to_ascii_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or(&lower);
    let denied_name = matches!(
        file_name,
        "install" | "install.sh" | "preinstall" | "postinstall" | "uninstall" | "uninstall.sh"
    );
    let denied_extension = [
        ".dylib", ".so", ".dll", ".node", ".wasm", ".html", ".htm", ".tsx", ".jsx",
    ]
    .iter()
    .any(|extension| lower.ends_with(extension));
    if denied_name || denied_extension {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn reject_symlink_or_special(path: &Path, allow_directory: bool) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || (!metadata.is_file() && !(allow_directory && metadata.is_dir()))
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn strict_manifest_rejects_unknown_fields_and_traversal() {
        let input = r#"
schemaVersion = 1
packageDigest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
surprise = true

[package]
id = "sample"
name = "Sample"
summary = "Sample package"
publisher = "C4OS"
version = "1.0.0"
kind = "plugin"

[compatibility]
c4os = "^0.1"

[trust]
origin = "local:test"
originKeyId = "origin"
delegatedContentKeyId = "content"
delegatedContentPublicKey = "invalid"
originSignature = "invalid"
contentSignature = "invalid"

[[inventory]]
path = "../escape"
bytes = 1
sha256 = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#;
        assert!(parse_manifest(input).is_err());
    }

    #[test]
    fn inventory_is_sorted_and_rejects_symlinks() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("z.md"), b"z").expect("write z");
        fs::write(root.path().join("a.md"), b"a").expect("write a");
        let inventory = inventory_for_root(root.path()).expect("inventory");
        assert_eq!(inventory[0].path, "a.md");
        assert_eq!(inventory[1].path, "z.md");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.path().join("a.md"), root.path().join("link.md"))
                .expect("symlink");
            assert!(matches!(
                inventory_for_root(root.path()),
                Err(ExtensionError::InvalidInput)
            ));
        }
    }

    #[test]
    fn compatibility_is_fail_closed() {
        let compatibility = PackageCompatibility {
            c4os: "^1.2".to_owned(),
            operating_systems: vec!["macos".to_owned()],
            architectures: vec!["aarch64".to_owned()],
        };
        let origins = [];
        let revocations = RevocationSet::default();
        let policy = PackageVerificationPolicy {
            c4os_version: "1.3.0",
            operating_system: "linux",
            architecture: "aarch64",
            trusted_origins: &origins,
            revocations: &revocations,
        };
        assert!(matches!(
            verify_compatibility(&compatibility, &policy),
            Err(ExtensionError::Incompatible)
        ));
    }

    #[test]
    fn content_signature_binds_identity_version_and_digest() {
        let manifest = PackageManifest {
            schema_version: 1,
            package: PackageMetadata {
                id: "sample".to_owned(),
                name: "Sample".to_owned(),
                summary: "Summary".to_owned(),
                publisher: "C4OS".to_owned(),
                version: "1.0.0".to_owned(),
                kind: ExtensionPackageKind::Plugin,
                website: None,
                terms: None,
                privacy_policy: None,
            },
            compatibility: PackageCompatibility {
                c4os: "^0.1".to_owned(),
                operating_systems: vec![],
                architectures: vec![],
            },
            trust: PackageTrust {
                origin: "local:test".to_owned(),
                origin_key_id: "origin".to_owned(),
                delegated_content_key_id: "content".to_owned(),
                delegated_content_public_key: "key".to_owned(),
                origin_signature: "signature".to_owned(),
                content_signature: "signature".to_owned(),
            },
            capabilities: vec![],
            skills: vec![],
            hooks: vec![],
            settings: vec![],
            apps: vec![],
            mcp_servers: vec![],
            inventory: vec![PackageInventoryEntry {
                path: "README.md".to_owned(),
                bytes: 1,
                sha256: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_owned(),
            }],
            package_digest:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        };
        let original = canonical_content_signing_bytes(&manifest);
        let mut changed = manifest.clone();
        changed.package.version = "1.0.1".to_owned();
        assert_ne!(original, canonical_content_signing_bytes(&changed));
    }

    #[test]
    fn declarative_contributions_are_signed_strict_and_never_embed_secret_values() {
        let manifest = PackageManifest {
            schema_version: 1,
            package: PackageMetadata {
                id: "declarative-plugin".into(),
                name: "Declarative Plugin".into(),
                summary: "Host-owned contributions".into(),
                publisher: "C4OS".into(),
                version: "1.0.0".into(),
                kind: ExtensionPackageKind::Plugin,
                website: None,
                terms: None,
                privacy_policy: None,
            },
            compatibility: PackageCompatibility {
                c4os: "^0.1".into(),
                operating_systems: Vec::new(),
                architectures: Vec::new(),
            },
            trust: PackageTrust {
                origin: "local:test".into(),
                origin_key_id: "origin".into(),
                delegated_content_key_id: "content".into(),
                delegated_content_public_key: "key".into(),
                origin_signature: "signature".into(),
                content_signature: "signature".into(),
            },
            capabilities: Vec::new(),
            skills: Vec::new(),
            hooks: Vec::new(),
            settings: vec![
                PackageSettingDeclaration {
                    id: "credential".into(),
                    label: "Credential".into(),
                    description: "Opaque vault reference".into(),
                    kind: PackageSettingKind::CredentialReference,
                    required: true,
                    choices: Vec::new(),
                },
                PackageSettingDeclaration {
                    id: "mode".into(),
                    label: "Mode".into(),
                    description: "Host-rendered selection".into(),
                    kind: PackageSettingKind::Select,
                    required: true,
                    choices: vec!["focused".into(), "expanded".into()],
                },
            ],
            apps: vec![PackageAppDeclaration {
                id: "summary".into(),
                title: "Summary".into(),
                summary: "Declarative app".into(),
                settings: vec!["mode".into()],
            }],
            mcp_servers: vec![PackageMcpServerDeclaration {
                id: "server".into(),
                name: "Server metadata".into(),
                transport: PackageMcpTransport::Stdio,
                settings: vec!["credential".into()],
            }],
            inventory: vec![PackageInventoryEntry {
                path: "README.md".into(),
                bytes: 1,
                sha256: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
            }],
            package_digest:
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        };
        validate_manifest_shape(&manifest).expect("valid declarative manifest");
        let signed = canonical_origin_signing_bytes(&manifest);
        let mut changed = manifest.clone();
        changed.apps[0].summary = "Changed signed declaration".into();
        assert_ne!(signed, canonical_origin_signing_bytes(&changed));

        let mut unknown_secret_field = toml::to_string(&manifest).unwrap();
        unknown_secret_field = unknown_secret_field.replacen(
            "required = true",
            "required = true\nvalue = \"raw-secret-forbidden\"",
            1,
        );
        assert!(parse_manifest(&unknown_secret_field).is_err());

        let mut unknown_reference = manifest;
        unknown_reference.apps[0].settings = vec!["missing".into()];
        assert!(validate_manifest_shape(&unknown_reference).is_err());
    }

    #[test]
    fn strict_ed25519_verification_rejects_tampering() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let message = b"c4os verified package";
        let signature = signing_key.sign(message);
        let public_key = BASE64.encode(signing_key.verifying_key().as_bytes());
        let signature = BASE64.encode(signature.to_bytes());
        assert!(verify_ed25519(&public_key, &signature, message).is_ok());
        assert!(matches!(
            verify_ed25519(&public_key, &signature, b"tampered"),
            Err(ExtensionError::VerificationFailed)
        ));
    }
}
