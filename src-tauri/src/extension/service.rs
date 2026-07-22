//! App-owned ExtensionService composition and durable lifecycle transitions.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::database::{DatabaseActor, ExtensionEventRecord, ExtensionStateDocumentRecord};

use super::{
    EXTENSION_STATE_SCHEMA_VERSION, ExtensionAuditEvent, ExtensionError, ExtensionLifecycle,
    ExtensionServiceSnapshot, ExtensionSourceKind, ExtensionTrustState, MAX_EXTENSION_RECORDS,
    MarketplaceSnapshot, PluginAppSnapshot, PluginHookSnapshot, PluginMcpServerSnapshot,
    PluginSettingSnapshot, PluginSnapshot, SkillQualifiedIdentity, SkillSnapshot,
    hook::{HookActivation, ReviewedHookContract},
    package::{
        PACKAGE_MANIFEST_NAME, PackageManifest, PackageMcpTransport, PackageSettingKind,
        PackageVerificationPolicy, RevocationSet, TrustedOrigin, parse_manifest,
        public_key_fingerprint, validate_origin, verify_catalog, verify_package_directory,
    },
    skill::{
        DiscoveredSkill, LoadedSkill, SKILL_ENTRYPOINT, SkillCandidate, discover_skill, load_skill,
        load_skill_resource, resolve_candidates,
    },
    skill_sources::{SkillSourceContext, SkillSourceRoot, discover_skill_sources},
    source::{MarketplaceSourceResolver, ResolvedMarketplaceSource},
    store::ExtensionStore,
};

const MARKETPLACE_CATALOG: &str = "c4os-marketplace.toml";
const MARKETPLACE_ORIGIN: &str = "c4os-marketplace-origin.toml";
const MARKETPLACE_SOURCE_SCHEMA_VERSION: u16 = 1;
const EXTENSION_SERVICE_DOCUMENT_SCHEMA_VERSION: u16 = 1;
pub const MAX_ACTIVE_TURN_SKILLS: usize = 16;
pub const MAX_ACTIVE_TURN_SKILL_BYTES: usize = 512 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarketplaceSourceInput {
    pub source: String,
    pub git_ref: Option<String>,
    pub sparse_paths: Vec<String>,
    pub trusted_origin: String,
    pub signing_key_id: String,
    pub public_key_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionPackageInput {
    pub expected_generation: u64,
    pub package_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionPackageMutation {
    Disable,
    ActivateStagedUpdate,
    Rollback,
    Uninstall,
    Revoke,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionSkillInput {
    pub expected_generation: u64,
    pub skill_identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionSkillAvailabilityInput {
    pub expected_generation: u64,
    pub skill_identity: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionHookReviewInput {
    pub expected_generation: u64,
    pub package_id: String,
    pub hook_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionRevocationInput {
    pub expected_generation: u64,
    pub package_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionKeyRevocationInput {
    pub expected_generation: u64,
    pub key_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionPublisherLinkInput {
    pub package_id: String,
    pub link: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillInstructionsSnapshot {
    pub identity: SkillQualifiedIdentity,
    pub package_id: Option<String>,
    pub instructions: String,
    pub entrypoint_digest: String,
    pub referenced_resources: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PreparedExtensionHook {
    pub activation: HookActivation,
    pub contract: ReviewedHookContract,
    pub package_root: PathBuf,
    pub grants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionHookExecutionRecord {
    pub package_id: String,
    pub hook_id: String,
    pub succeeded: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MarketplaceOriginDocument {
    schema_version: u16,
    origin: String,
    key_id: String,
    public_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedMarketplace {
    snapshot: MarketplaceSnapshot,
    root: PathBuf,
    authority: PersistedAuthority,
    #[serde(default)]
    requested_source: String,
    #[serde(default)]
    requested_ref: Option<String>,
    #[serde(default)]
    sparse_paths: Vec<String>,
    #[serde(default)]
    resolved_commit: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PersistedAuthority {
    origin: String,
    key_id: String,
    public_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MarketplaceTrustPin {
    origin: String,
    key_id: String,
    public_key_sha256: String,
}

impl MarketplaceTrustPin {
    fn validate(&self) -> Result<(), ExtensionError> {
        validate_origin(&self.origin)?;
        super::validate_identifier(&self.key_id)?;
        super::validate_digest(&self.public_key_sha256)
    }

    fn from_authority(authority: &PersistedAuthority) -> Result<Self, ExtensionError> {
        Ok(Self {
            origin: authority.origin.clone(),
            key_id: authority.key_id.clone(),
            public_key_sha256: public_key_fingerprint(&authority.public_key)?,
        })
    }
}

impl PersistedAuthority {
    fn trusted_origin(&self) -> TrustedOrigin {
        TrustedOrigin {
            origin: self.origin.clone(),
            key_id: self.key_id.clone(),
            public_key: self.public_key.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PackageCandidate {
    marketplace_id: String,
    root: PathBuf,
    manifest: PackageManifest,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExtensionServiceDocument {
    schema_version: u16,
    snapshot: ExtensionServiceSnapshot,
    marketplaces: Vec<PersistedMarketplace>,
    candidates: Vec<PackageCandidate>,
    skill_enabled: BTreeMap<String, bool>,
    revoked_keys: BTreeSet<String>,
    revoked_digests: BTreeSet<String>,
    #[serde(default)]
    revoked_packages: BTreeSet<String>,
}

impl ExtensionServiceDocument {
    fn empty() -> Self {
        Self {
            schema_version: EXTENSION_SERVICE_DOCUMENT_SCHEMA_VERSION,
            snapshot: ExtensionServiceSnapshot {
                schema_version: EXTENSION_STATE_SCHEMA_VERSION,
                generation: 1,
                marketplaces: Vec::new(),
                plugins: Vec::new(),
                skills: Vec::new(),
                selected_skill: None,
                active_workers: 0,
                last_event_id: 1,
            },
            marketplaces: Vec::new(),
            candidates: Vec::new(),
            skill_enabled: BTreeMap::new(),
            revoked_keys: BTreeSet::new(),
            revoked_digests: BTreeSet::new(),
            revoked_packages: BTreeSet::new(),
        }
    }

    fn revocations(&self) -> RevocationSet {
        RevocationSet {
            key_ids: self.revoked_keys.clone(),
            package_digests: self.revoked_digests.clone(),
        }
    }

    fn validate(&self) -> Result<(), ExtensionError> {
        if self.schema_version != EXTENSION_SERVICE_DOCUMENT_SCHEMA_VERSION
            || self.marketplaces.len() > MAX_EXTENSION_RECORDS
            || self.candidates.len() > MAX_EXTENSION_RECORDS
            || self.revoked_keys.len() > MAX_EXTENSION_RECORDS
            || self.revoked_digests.len() > MAX_EXTENSION_RECORDS
            || self.revoked_packages.len() > MAX_EXTENSION_RECORDS
        {
            return Err(ExtensionError::InvalidState);
        }
        for package_id in &self.revoked_packages {
            super::validate_identifier(package_id)?;
        }
        for digest in &self.revoked_digests {
            super::validate_digest(digest)?;
        }
        self.snapshot.validate()?;
        if self.snapshot.marketplaces.len() != self.marketplaces.len()
            || self
                .marketplaces
                .iter()
                .zip(&self.snapshot.marketplaces)
                .any(|(persisted, projected)| &persisted.snapshot != projected)
        {
            return Err(ExtensionError::InvalidState);
        }
        for marketplace in &self.marketplaces {
            if !marketplace.root.is_absolute()
                || marketplace.requested_source != marketplace.snapshot.source
                || marketplace.requested_ref != marketplace.snapshot.git_ref
                || marketplace.sparse_paths != marketplace.snapshot.sparse_paths
                || marketplace.resolved_commit != marketplace.snapshot.resolved_commit
            {
                return Err(ExtensionError::InvalidState);
            }
        }
        Ok(())
    }
}

pub struct ExtensionService {
    database: Arc<DatabaseActor>,
    home: PathBuf,
    store: ExtensionStore,
    source_resolver: MarketplaceSourceResolver,
    document: ExtensionServiceDocument,
    skill_source_roots: Vec<SkillSourceRoot>,
    discovered: BTreeMap<String, DiscoveredSkill>,
    loaded: BTreeMap<String, LoadedSkill>,
}

impl ExtensionService {
    pub fn restore(
        database: Arc<DatabaseActor>,
        home: impl AsRef<Path>,
        now_ms: u64,
    ) -> Result<Self, ExtensionError> {
        if now_ms == 0 {
            return Err(ExtensionError::InvalidInput);
        }
        let home = home.as_ref().to_path_buf();
        let skill_source_roots = default_skill_source_roots(&home);
        let store = ExtensionStore::new(home.join("extensions"))?;
        let source_resolver = MarketplaceSourceResolver::new(home.join("marketplace-sources"))?;
        let persisted = database.extension_state_document()?;
        let mut service = if let Some(persisted) = persisted {
            let document: ExtensionServiceDocument =
                serde_json::from_str(&persisted.canonical_document)?;
            if document.snapshot.generation != persisted.generation {
                return Err(ExtensionError::InvalidState);
            }
            document.validate()?;
            Self {
                database,
                home,
                store,
                source_resolver,
                document,
                skill_source_roots: skill_source_roots.clone(),
                discovered: BTreeMap::new(),
                loaded: BTreeMap::new(),
            }
        } else {
            let document = ExtensionServiceDocument::empty();
            let canonical_document = serde_json::to_string(&document)?;
            let audit = ExtensionAuditEvent {
                schema_version: EXTENSION_STATE_SCHEMA_VERSION,
                event_id: 1,
                generation: 1,
                operation_id: "extension-bootstrap".into(),
                package_id: None,
                event_kind: "serviceRestored".into(),
                selector_digest: None,
                result: "succeeded".into(),
                occurred_at_ms: now_ms,
            };
            database.save_extension_transition(
                ExtensionStateDocumentRecord {
                    generation: 1,
                    canonical_document,
                    updated_at_ms: now_ms,
                },
                ExtensionEventRecord {
                    event_id: 1,
                    generation: 1,
                    operation_id: audit.operation_id.clone(),
                    package_id: None,
                    event_kind: audit.event_kind.clone(),
                    selector_digest: None,
                    result: audit.result.clone(),
                    canonical_document: serde_json::to_string(&audit)?,
                    occurred_at_ms: now_ms,
                },
                None,
            )?;
            Self {
                database,
                home,
                store,
                source_resolver,
                document,
                skill_source_roots,
                discovered: BTreeMap::new(),
                loaded: BTreeMap::new(),
            }
        };
        service.reconcile_persisted_hook_contracts(now_ms)?;
        service.reconcile_selector_state(now_ms.saturating_add(1))?;
        service.synchronize_skills(now_ms.saturating_add(2))?;
        Ok(service)
    }

    pub fn snapshot(&self) -> ExtensionServiceSnapshot {
        let mut snapshot = self.document.snapshot.clone();
        for skill in &mut snapshot.skills {
            skill.instructions_loaded = self.loaded.contains_key(&skill.identity.stable_id());
        }
        snapshot
    }

    /// Rebinds discovery to exact coordinator-authorized roots. Paths are
    /// transient composition inputs; only stable source-qualified identities,
    /// availability, selection, and validation outcomes are persisted.
    pub fn synchronize_skill_source_roots(
        &mut self,
        roots: Vec<SkillSourceRoot>,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        if roots == self.skill_source_roots {
            return Ok(self.snapshot());
        }
        let previous = std::mem::replace(&mut self.skill_source_roots, roots);
        if let Err(error) = self.synchronize_skills(now_ms) {
            self.skill_source_roots = previous;
            return Err(error);
        }
        Ok(self.snapshot())
    }

    /// Performs every deterministic lifecycle and immutable-store check before
    /// the command layer revokes any in-memory worker or Action Gateway
    /// authority. The caller retains the service lock through the subsequent
    /// revocation barrier and mutation so another command cannot invalidate
    /// this preflight.
    pub fn preflight_package_mutation(
        &self,
        input: &ExtensionPackageInput,
        mutation: ExtensionPackageMutation,
    ) -> Result<(), ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?;
        let selector = self.store.read_selector(&input.package_id)?;
        match mutation {
            ExtensionPackageMutation::Disable => {
                if !matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::Enabled
                        | ExtensionLifecycle::UpdateStaged
                        | ExtensionLifecycle::Executing
                ) || selector
                    .as_ref()
                    .and_then(|value| value.active_digest.as_deref())
                    != Some(plugin.digest.as_str())
                {
                    return Err(ExtensionError::InvalidState);
                }
            }
            ExtensionPackageMutation::ActivateStagedUpdate => {
                if plugin.trust != ExtensionTrustState::Trusted
                    || plugin.lifecycle != ExtensionLifecycle::UpdateStaged
                {
                    return Err(ExtensionError::InvalidState);
                }
                let staged_digest = plugin
                    .staged_digest
                    .as_deref()
                    .ok_or(ExtensionError::InvalidState)?;
                plugin
                    .staged_version
                    .as_deref()
                    .ok_or(ExtensionError::InvalidState)?;
                if selector
                    .as_ref()
                    .and_then(|value| value.active_digest.as_deref())
                    != Some(plugin.digest.as_str())
                {
                    return Err(ExtensionError::InvalidState);
                }
                let candidate = self
                    .document
                    .candidates
                    .iter()
                    .find(|candidate| candidate.manifest.package_digest == staged_digest)
                    .ok_or(ExtensionError::InvalidState)?;
                let trusted_origins = self.trusted_origins();
                let revocations = self.document.revocations();
                let policy = package_verification_policy(&trusted_origins, &revocations);
                self.store.verify_installed(staged_digest, &policy)?;
                let mut staged_plugin = plugin.clone();
                apply_candidate_to_plugin(&mut staged_plugin, candidate, 1);
                staged_plugin.digest = staged_digest.to_owned();
                validate_activation_content(&self.store, &staged_plugin, &policy)?;
            }
            ExtensionPackageMutation::Rollback => {
                if !matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::Enabled | ExtensionLifecycle::UpdateStaged
                ) {
                    return Err(ExtensionError::InvalidState);
                }
                let selector = selector.as_ref().ok_or(ExtensionError::InvalidState)?;
                if selector.active_digest.as_deref() != Some(plugin.digest.as_str()) {
                    return Err(ExtensionError::InvalidState);
                }
                let digest = selector
                    .last_known_good_digest
                    .as_deref()
                    .ok_or(ExtensionError::InvalidState)?;
                let candidate = self
                    .document
                    .candidates
                    .iter()
                    .find(|candidate| candidate.manifest.package_digest == digest)
                    .ok_or(ExtensionError::InvalidState)?;
                let trusted_origins = self.trusted_origins();
                let revocations = self.document.revocations();
                let policy = package_verification_policy(&trusted_origins, &revocations);
                self.store.verify_installed(digest, &policy)?;
                let mut rollback_plugin = plugin.clone();
                apply_candidate_to_plugin(&mut rollback_plugin, candidate, 1);
                validate_activation_content(&self.store, &rollback_plugin, &policy)?;
            }
            ExtensionPackageMutation::Uninstall => {
                if matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::Available
                        | ExtensionLifecycle::Quarantined
                        | ExtensionLifecycle::Enabling
                        | ExtensionLifecycle::Executing
                ) {
                    return Err(ExtensionError::InvalidState);
                }
                self.latest_candidate(&input.package_id)?;
            }
            ExtensionPackageMutation::Revoke => {
                if plugin.lifecycle == ExtensionLifecycle::Revoked
                    || plugin.trust == ExtensionTrustState::Revoked
                {
                    return Err(ExtensionError::Revoked);
                }
            }
        }
        Ok(())
    }

    pub fn add_marketplace(
        &mut self,
        input: MarketplaceSourceInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        let trust_pin = MarketplaceTrustPin {
            origin: input.trusted_origin,
            key_id: input.signing_key_id,
            public_key_sha256: input.public_key_sha256,
        };
        trust_pin.validate()?;
        let resolved = self.source_resolver.resolve(
            &input.source,
            input.git_ref.as_deref(),
            &input.sparse_paths,
        )?;
        let (marketplace, candidates) = self.load_marketplace(resolved, &trust_pin, now_ms)?;
        if self
            .document
            .marketplaces
            .iter()
            .any(|existing| existing.snapshot.marketplace_id == marketplace.snapshot.marketplace_id)
        {
            return Err(ExtensionError::Conflict);
        }
        let mut next = self.document.clone();
        next.marketplaces.push(marketplace);
        next.marketplaces.sort_by(|left, right| {
            left.snapshot
                .marketplace_id
                .cmp(&right.snapshot.marketplace_id)
        });
        next.candidates.extend(candidates);
        sort_and_validate_candidates(&mut next.candidates)?;
        rebuild_plugin_projection(&mut next, now_ms)?;
        self.commit(next, "marketplaceAdded", None, None, now_ms)?;
        Ok(self.snapshot())
    }

    pub fn refresh_catalogs(
        &mut self,
        expected_generation: u64,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(expected_generation)?;
        let mut marketplaces = Vec::new();
        let mut candidates = Vec::new();
        for current in &self.document.marketplaces {
            let trust_pin = MarketplaceTrustPin::from_authority(&current.authority)?;
            let requested_source = if current.requested_source.is_empty() {
                current.snapshot.source.as_str()
            } else {
                current.requested_source.as_str()
            };
            let requested_ref = current
                .requested_ref
                .as_deref()
                .or(current.snapshot.git_ref.as_deref());
            let sparse_paths = if current.sparse_paths.is_empty() {
                current.snapshot.sparse_paths.as_slice()
            } else {
                current.sparse_paths.as_slice()
            };
            let resolved =
                self.source_resolver
                    .resolve(requested_source, requested_ref, sparse_paths)?;
            let (marketplace, mut releases) =
                self.load_marketplace(resolved, &trust_pin, now_ms)?;
            if marketplace.authority != current.authority {
                return Err(ExtensionError::UntrustedOrigin);
            }
            marketplaces.push(marketplace);
            candidates.append(&mut releases);
        }
        sort_and_validate_candidates(&mut candidates)?;
        let mut next = self.document.clone();
        next.marketplaces = marketplaces;
        next.candidates = candidates;
        rebuild_plugin_projection(&mut next, now_ms)?;
        self.commit(next, "catalogRefreshed", None, None, now_ms)?;
        Ok(self.snapshot())
    }

    pub fn install_disabled(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        if self.document.revoked_packages.contains(&input.package_id) {
            return Err(ExtensionError::Revoked);
        }
        let candidate = self.latest_candidate(&input.package_id)?.clone();
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        let verified = verify_package_directory(&candidate.root, &policy)?;
        let quarantined = self.store.quarantine_verified(&verified, &policy)?;
        let installed = self.store.install_quarantined(quarantined, &policy)?;
        let mut validated_plugin = plugin_from_candidate(&candidate, now_ms);
        validated_plugin.digest = installed.digest.clone();
        if validate_activation_content(&self.store, &validated_plugin, &policy).is_err() {
            let mut next = self.document.clone();
            let plugin = plugin_mut(&mut next, &input.package_id)?;
            apply_candidate_to_plugin(plugin, &candidate, now_ms);
            plugin.digest = installed.digest.clone();
            plugin.lifecycle = ExtensionLifecycle::Failed;
            plugin.active_digest = None;
            plugin.last_known_good_digest = None;
            plugin.failure_code = Some("package-content-invalid".into());
            self.commit_with_skill_projection(
                next,
                "packageInstallFailedValidation",
                Some(&input.package_id),
                Some(&installed.digest),
                now_ms,
            )?;
            return Ok(self.snapshot());
        }
        let current_selector = self.store.read_selector(&input.package_id)?;
        if current_selector.as_ref().is_some_and(|selector| {
            selector.active_digest.is_some() || selector.last_known_good_digest.is_some()
        }) {
            return Err(ExtensionError::Conflict);
        }
        self.store.compare_and_swap_selector(
            &input.package_id,
            current_selector
                .as_ref()
                .map(|selector| selector.generation),
            None,
            None,
        )?;
        let mut next = self.document.clone();
        let plugin = plugin_mut(&mut next, &input.package_id)?;
        apply_candidate_to_plugin(plugin, &candidate, now_ms);
        plugin.digest = installed.digest.clone();
        plugin.lifecycle = ExtensionLifecycle::InstalledDisabled;
        plugin.active_digest = None;
        plugin.last_known_good_digest = None;
        plugin.failure_code = None;
        self.commit_with_skill_projection(
            next,
            "packageInstalledDisabled",
            Some(&input.package_id),
            Some(&installed.digest),
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub fn enable(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        if plugin.trust != ExtensionTrustState::Trusted
            || !matches!(
                plugin.lifecycle,
                ExtensionLifecycle::InstalledDisabled | ExtensionLifecycle::RolledBack
            )
        {
            return Err(ExtensionError::InvalidState);
        }
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        self.store.verify_installed(&plugin.digest, &policy)?;
        validate_activation_content(&self.store, &plugin, &policy)?;
        let selector = self
            .store
            .read_selector(&input.package_id)?
            .ok_or(ExtensionError::InvalidState)?;
        let prior = self.document.clone();
        let mut next = prior.clone();
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        next_plugin.lifecycle = ExtensionLifecycle::Enabled;
        next_plugin.active_digest = Some(plugin.digest.clone());
        next_plugin.last_known_good_digest = Some(plugin.digest.clone());
        next_plugin.active_version = Some(next_plugin.version.clone());
        next_plugin.last_known_good_version = Some(next_plugin.version.clone());
        next_plugin.failure_code = None;
        self.commit_with_skill_projection(
            next,
            "packageEnabledForNextTurn",
            Some(&input.package_id),
            Some(&plugin.digest),
            now_ms,
        )?;
        if let Err(error) = self.store.compare_and_swap_selector(
            &input.package_id,
            Some(selector.generation),
            Some(&plugin.digest),
            Some(&plugin.digest),
        ) {
            self.commit_with_skill_projection(
                prior,
                "packageEnablePublicationCompensated",
                Some(&input.package_id),
                Some(&plugin.digest),
                now_ms.saturating_add(1),
            )?;
            return Err(error);
        }
        Ok(self.snapshot())
    }

    pub fn disable(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        if !matches!(
            plugin.lifecycle,
            ExtensionLifecycle::Enabled
                | ExtensionLifecycle::UpdateStaged
                | ExtensionLifecycle::Executing
        ) {
            return Err(ExtensionError::InvalidState);
        }
        let selector = self
            .store
            .read_selector(&input.package_id)?
            .ok_or(ExtensionError::InvalidState)?;
        let mut next = self.document.clone();
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        next_plugin.lifecycle = ExtensionLifecycle::InstalledDisabled;
        next_plugin.active_digest = None;
        next_plugin.active_version = None;
        next_plugin.last_known_good_digest = selector.last_known_good_digest.clone();
        next_plugin.failure_code = None;
        self.commit_with_skill_projection(
            next,
            "packageDisabled",
            Some(&input.package_id),
            Some(&plugin.digest),
            now_ms,
        )?;
        self.store.compare_and_swap_selector(
            &input.package_id,
            Some(selector.generation),
            None,
            selector.last_known_good_digest.as_deref(),
        )?;
        Ok(self.snapshot())
    }

    pub fn stage_update(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        if plugin.trust != ExtensionTrustState::Trusted {
            return Err(ExtensionError::Revoked);
        }
        let candidate = self.latest_candidate(&input.package_id)?.clone();
        if Version::parse(&candidate.manifest.package.version)
            .map_err(|_| ExtensionError::InvalidInput)?
            <= Version::parse(&plugin.version).map_err(|_| ExtensionError::InvalidInput)?
        {
            return Err(ExtensionError::Conflict);
        }
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        let verified = verify_package_directory(&candidate.root, &policy)?;
        let staged = self
            .store
            .install_quarantined(self.store.quarantine_verified(&verified, &policy)?, &policy)?;
        let mut next = self.document.clone();
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        next_plugin.lifecycle = ExtensionLifecycle::UpdateStaged;
        next_plugin.staged_version = Some(candidate.manifest.package.version.clone());
        next_plugin.staged_digest = Some(staged.digest.clone());
        next_plugin.failure_code = None;
        self.commit_with_skill_projection(
            next,
            "packageUpdateStaged",
            Some(&input.package_id),
            Some(&staged.digest),
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub fn activate_staged_update(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        if plugin.trust != ExtensionTrustState::Trusted {
            return Err(ExtensionError::Revoked);
        }
        if plugin.lifecycle != ExtensionLifecycle::UpdateStaged {
            return Err(ExtensionError::InvalidState);
        }
        let staged_digest = plugin
            .staged_digest
            .clone()
            .ok_or(ExtensionError::InvalidState)?;
        let staged_version = plugin
            .staged_version
            .clone()
            .ok_or(ExtensionError::InvalidState)?;
        let candidate = self
            .document
            .candidates
            .iter()
            .find(|candidate| candidate.manifest.package_digest == staged_digest)
            .cloned()
            .ok_or(ExtensionError::InvalidState)?;
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        self.store.verify_installed(&staged_digest, &policy)?;
        let mut staged_plugin = plugin.clone();
        apply_candidate_to_plugin(&mut staged_plugin, &candidate, now_ms);
        staged_plugin.digest = staged_digest.clone();
        validate_activation_content(&self.store, &staged_plugin, &policy)?;
        let selector = self
            .store
            .read_selector(&input.package_id)?
            .ok_or(ExtensionError::InvalidState)?;
        let prior_digest = plugin
            .active_digest
            .clone()
            .unwrap_or(plugin.digest.clone());
        let prior = self.document.clone();
        let mut next = prior.clone();
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        apply_candidate_to_plugin(next_plugin, &candidate, now_ms);
        next_plugin.lifecycle = ExtensionLifecycle::Enabled;
        next_plugin.last_known_good_digest = Some(prior_digest.clone());
        next_plugin.last_known_good_version = Some(plugin.version.clone());
        next_plugin.active_digest = Some(staged_digest.clone());
        next_plugin.active_version = Some(staged_version.clone());
        next_plugin.version = staged_version;
        next_plugin.digest = staged_digest.clone();
        next_plugin.staged_version = None;
        next_plugin.staged_digest = None;
        next_plugin.available_version = None;
        next_plugin.available_digest = None;
        next_plugin.failure_code = None;
        self.loaded
            .retain(|_, skill| skill.discovered.package_id.as_deref() != Some(&input.package_id));
        self.commit_with_skill_projection(
            next,
            "packageUpdateActivated",
            Some(&input.package_id),
            Some(&staged_digest),
            now_ms,
        )?;
        if let Err(error) = self.store.compare_and_swap_selector(
            &input.package_id,
            Some(selector.generation),
            Some(&staged_digest),
            Some(&prior_digest),
        ) {
            self.commit_with_skill_projection(
                prior,
                "packageUpdatePublicationCompensated",
                Some(&input.package_id),
                Some(&prior_digest),
                now_ms.saturating_add(1),
            )?;
            return Err(error);
        }
        Ok(self.snapshot())
    }

    pub fn rollback(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        let selector = self
            .store
            .read_selector(&input.package_id)?
            .ok_or(ExtensionError::InvalidState)?;
        let digest = selector
            .last_known_good_digest
            .clone()
            .ok_or(ExtensionError::InvalidState)?;
        let candidate = self
            .document
            .candidates
            .iter()
            .find(|candidate| candidate.manifest.package_digest == digest)
            .cloned()
            .ok_or(ExtensionError::InvalidState)?;
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        self.store.verify_installed(&digest, &policy)?;
        let mut rollback_plugin = plugin.clone();
        apply_candidate_to_plugin(&mut rollback_plugin, &candidate, now_ms);
        validate_activation_content(&self.store, &rollback_plugin, &policy)?;
        let prior = self.document.clone();
        let mut next = prior.clone();
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        apply_candidate_to_plugin(next_plugin, &candidate, now_ms);
        next_plugin.lifecycle = ExtensionLifecycle::RolledBack;
        next_plugin.active_digest = None;
        next_plugin.last_known_good_digest = Some(digest.clone());
        next_plugin.active_version = None;
        next_plugin.last_known_good_version = Some(candidate.manifest.package.version.clone());
        next_plugin.failure_code = None;
        self.loaded
            .retain(|_, skill| skill.discovered.package_id.as_deref() != Some(&plugin.package_id));
        self.commit_with_skill_projection(
            next,
            "packageRolledBackDisabled",
            Some(&input.package_id),
            Some(&digest),
            now_ms,
        )?;
        if let Err(error) = self.store.compare_and_swap_selector(
            &input.package_id,
            Some(selector.generation),
            None,
            Some(&digest),
        ) {
            self.commit_with_skill_projection(
                prior,
                "packageRollbackPublicationCompensated",
                Some(&input.package_id),
                Some(&plugin.digest),
                now_ms.saturating_add(1),
            )?;
            return Err(error);
        }
        Ok(self.snapshot())
    }

    pub fn revoke(
        &mut self,
        input: ExtensionPackageInput,
        reason: &str,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        if reason.trim().is_empty() || reason.len() > 1_024 {
            return Err(ExtensionError::InvalidInput);
        }
        let plugin = self.plugin(&input.package_id)?.clone();
        let selector = self.store.read_selector(&input.package_id)?;
        let mut next = self.document.clone();
        next.revoked_packages.insert(plugin.package_id.clone());
        let package_digests = next
            .candidates
            .iter()
            .filter(|candidate| candidate.manifest.package.id == plugin.package_id)
            .map(|candidate| candidate.manifest.package_digest.clone())
            .chain(std::iter::once(plugin.digest.clone()))
            .chain(plugin.active_digest.clone())
            .chain(plugin.staged_digest.clone())
            .chain(plugin.last_known_good_digest.clone())
            .collect::<Vec<_>>();
        next.revoked_digests.extend(package_digests);
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        next_plugin.lifecycle = ExtensionLifecycle::Revoked;
        next_plugin.trust = ExtensionTrustState::Revoked;
        next_plugin.active_digest = None;
        next_plugin.revocation_reason = Some(reason.trim().into());
        self.loaded
            .retain(|_, skill| skill.discovered.package_id.as_deref() != Some(&input.package_id));
        self.commit_with_skill_projection(
            next,
            "packageRevoked",
            Some(&input.package_id),
            Some(&plugin.digest),
            now_ms,
        )?;
        if let Some(selector) = selector {
            self.store.compare_and_swap_selector(
                &input.package_id,
                Some(selector.generation),
                None,
                selector.last_known_good_digest.as_deref(),
            )?;
        }
        Ok(self.snapshot())
    }

    pub fn package_ids_signed_by_key(&self, key_id: &str) -> Result<Vec<String>, ExtensionError> {
        super::validate_identifier(key_id)?;
        let package_ids = self
            .document
            .snapshot
            .plugins
            .iter()
            .filter(|plugin| plugin.origin_key_id == key_id || plugin.content_key_id == key_id)
            .map(|plugin| plugin.package_id.clone())
            .collect::<Vec<_>>();
        if package_ids.is_empty() {
            return Err(ExtensionError::InvalidInput);
        }
        Ok(package_ids)
    }

    /// Revokes one exact signing key before clearing package selectors. The
    /// durable trust transition is the authority boundary; a crash during
    /// selector cleanup can therefore leave only inactive storage metadata.
    pub fn revoke_key(
        &mut self,
        input: ExtensionKeyRevocationInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        super::validate_identifier(&input.key_id)?;
        if input.reason.trim().is_empty() || input.reason.len() > 1_024 {
            return Err(ExtensionError::InvalidInput);
        }
        let package_ids = self.package_ids_signed_by_key(&input.key_id)?;
        let package_ids_set = package_ids.iter().cloned().collect::<BTreeSet<_>>();
        let mut next = self.document.clone();
        next.revoked_keys.insert(input.key_id.clone());
        for plugin in &mut next.snapshot.plugins {
            if !package_ids_set.contains(&plugin.package_id) {
                continue;
            }
            plugin.lifecycle = ExtensionLifecycle::Revoked;
            plugin.trust = ExtensionTrustState::Revoked;
            plugin.active_digest = None;
            plugin.revocation_reason = Some(input.reason.trim().into());
        }
        self.loaded.retain(|_, skill| {
            skill
                .discovered
                .package_id
                .as_ref()
                .is_none_or(|package_id| !package_ids_set.contains(package_id))
        });
        self.commit_with_skill_projection(next, "publisherKeyRevoked", None, None, now_ms)?;
        for package_id in package_ids {
            if let Some(selector) = self.store.read_selector(&package_id)? {
                self.store.compare_and_swap_selector(
                    &package_id,
                    Some(selector.generation),
                    None,
                    selector.last_known_good_digest.as_deref(),
                )?;
            }
        }
        Ok(self.snapshot())
    }

    pub fn uninstall(
        &mut self,
        input: ExtensionPackageInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        let selector = self.store.read_selector(&input.package_id)?;
        let mut digests = BTreeSet::from([plugin.digest.clone()]);
        digests.extend(plugin.staged_digest.clone());
        digests.extend(plugin.last_known_good_digest.clone());
        digests.extend(
            self.document
                .candidates
                .iter()
                .filter(|candidate| candidate.manifest.package.id == input.package_id)
                .map(|candidate| candidate.manifest.package_digest.clone()),
        );
        let candidate = self.latest_candidate(&input.package_id)?.clone();
        let mut next = self.document.clone();
        let package_remains_revoked = next.revoked_packages.contains(&input.package_id);
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        *next_plugin = plugin_from_candidate(&candidate, now_ms);
        if package_remains_revoked {
            next_plugin.trust = ExtensionTrustState::Revoked;
            next_plugin.revocation_reason = plugin.revocation_reason.clone();
        }
        self.loaded
            .retain(|_, skill| skill.discovered.package_id.as_deref() != Some(&input.package_id));
        self.commit_with_skill_projection(
            next,
            "packageUninstalledWithoutExecution",
            Some(&input.package_id),
            Some(&plugin.digest),
            now_ms,
        )?;
        if let Some(selector) = selector {
            let cleared = self.store.compare_and_swap_selector(
                &input.package_id,
                Some(selector.generation),
                None,
                None,
            )?;
            if cleared.active_digest.is_some() {
                return Err(ExtensionError::Conflict);
            }
        }
        // Publish the non-active lifecycle before reclaiming immutable bytes.
        // A crash or cleanup error can therefore leave only an inert orphan,
        // never durable state that points at a missing installed package.
        for digest in digests {
            self.store
                .uninstall_without_execution(&input.package_id, &digest)?;
        }
        Ok(self.snapshot())
    }

    pub fn set_skill_enabled(
        &mut self,
        input: ExtensionSkillAvailabilityInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let identity = self.skill_identity(&input.skill_identity)?;
        let mut next = self.document.clone();
        next.skill_enabled
            .insert(identity.stable_id(), input.enabled);
        if !input.enabled {
            self.loaded.remove(&identity.stable_id());
        }
        self.commit_with_skill_projection(next, "skillAvailabilityChanged", None, None, now_ms)?;
        Ok(self.snapshot())
    }

    pub fn select_skill(
        &mut self,
        input: ExtensionSkillInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let identity = self.skill_identity(&input.skill_identity)?;
        let mut next = self.document.clone();
        next.snapshot.selected_skill = Some(identity);
        self.commit_with_skill_projection(next, "skillSelectedExplicitly", None, None, now_ms)?;
        Ok(self.snapshot())
    }

    /// Creates a user-owned copy of an immutable or scoped Skill without
    /// mutating or deleting its source. The copy becomes the explicit
    /// source-qualified selection only after complete progressive validation.
    pub fn customize_skill(
        &mut self,
        input: ExtensionSkillInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let identity = self.skill_identity(&input.skill_identity)?;
        if identity.source_kind == ExtensionSourceKind::UserGlobal {
            return Err(ExtensionError::InvalidState);
        }
        let discovered = self
            .discovered
            .get(&input.skill_identity)
            .cloned()
            .ok_or(ExtensionError::InvalidState)?;
        let loaded = load_skill(&discovered)?;
        let new_identity = SkillQualifiedIdentity {
            source_kind: ExtensionSourceKind::UserGlobal,
            source_id: "user-global".into(),
            skill_id: identity.skill_id,
        };
        let (_, preflight_discovered) = self.resolve_skills(&self.document)?;
        if preflight_discovered.contains_key(&new_identity.stable_id()) {
            return Err(ExtensionError::Conflict);
        }
        let source_hash = super::sha256_prefixed(new_identity.skill_id.as_bytes());
        let directory_id = format!("custom-{}", &source_hash["sha256:".len()..][..24]);
        super::validate_identifier(&directory_id)?;
        let user_root = self.home.join("skills/user");
        ensure_private_skill_directory(&user_root)?;
        let target = user_root.join(&directory_id);
        match fs::symlink_metadata(&target) {
            Ok(_) => return Err(ExtensionError::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let staging_root = self.home.join("skills/.staging");
        ensure_private_skill_directory(&staging_root)?;
        let temporary = tempfile::Builder::new()
            .prefix("custom-skill-")
            .tempdir_in(&staging_root)?;
        ensure_private_skill_directory(temporary.path())?;
        let entrypoint_bytes = fs::read(&loaded.discovered.entrypoint)?;
        if super::sha256_prefixed(&entrypoint_bytes) != loaded.entrypoint_digest {
            return Err(ExtensionError::MutableContent);
        }
        write_private_skill_file(&temporary.path().join(SKILL_ENTRYPOINT), &entrypoint_bytes)?;
        for relative in &loaded.referenced_resources {
            let bytes = load_skill_resource(&loaded, relative)?;
            let destination = temporary.path().join(relative);
            if let Some(parent) = destination.parent() {
                ensure_private_skill_directory(parent)?;
            }
            write_private_skill_file(&destination, &bytes)?;
        }
        let temporary = temporary.keep();
        fs::rename(&temporary, &target)?;
        sync_skill_directory(&staging_root)?;
        sync_skill_directory(&user_root)?;

        let mut next = self.document.clone();
        next.skill_enabled.insert(new_identity.stable_id(), true);
        next.snapshot.selected_skill = Some(new_identity.clone());
        let publication = (|| {
            let (skills, discovered) = self.resolve_skills(&next)?;
            if !skills.iter().any(|skill| {
                skill.identity == new_identity && skill.valid && skill.enabled && skill.active
            }) {
                return Err(ExtensionError::InvalidState);
            }
            next.snapshot.skills = skills;
            self.commit(next, "skillCustomizedToUserOverride", None, None, now_ms)?;
            Ok(discovered)
        })();
        let discovered = match publication {
            Ok(discovered) => discovered,
            Err(error) => {
                fs::remove_dir_all(&target)?;
                sync_skill_directory(&user_root)?;
                return Err(error);
            }
        };
        self.discovered = discovered;
        self.loaded.clear();
        Ok(self.snapshot())
    }

    pub fn load_skill_instructions(
        &mut self,
        skill_identity: &str,
    ) -> Result<SkillInstructionsSnapshot, ExtensionError> {
        let identity = self.skill_identity(skill_identity)?;
        let current = self
            .document
            .snapshot
            .skills
            .iter()
            .find(|skill| skill.identity == identity)
            .ok_or(ExtensionError::InvalidState)?;
        if !current.active || !current.valid || !current.enabled || !current.eligible {
            return Err(ExtensionError::InvalidState);
        }
        if let Some(package_id) = current.package_id.as_deref() {
            let plugin = self.plugin(package_id)?;
            if !self.plugin_selector_active(plugin)? {
                return Err(ExtensionError::InvalidState);
            }
        }
        let discovered = self
            .discovered
            .get(skill_identity)
            .ok_or(ExtensionError::InvalidState)?;
        let loaded = load_skill(discovered)?;
        let result = SkillInstructionsSnapshot {
            identity: loaded.discovered.identity.clone(),
            package_id: loaded.discovered.package_id.clone(),
            instructions: loaded.instructions.clone(),
            entrypoint_digest: loaded.entrypoint_digest.clone(),
            referenced_resources: loaded.referenced_resources.clone(),
        };
        self.loaded.insert(skill_identity.into(), loaded);
        Ok(result)
    }

    pub fn active_turn_skills(&mut self) -> Result<Vec<SkillInstructionsSnapshot>, ExtensionError> {
        let active = self
            .document
            .snapshot
            .skills
            .iter()
            .filter(|skill| skill.active)
            .map(|skill| skill.identity.stable_id())
            .collect::<Vec<_>>();
        if active.len() > MAX_ACTIVE_TURN_SKILLS {
            return Err(ExtensionError::BoundExceeded);
        }
        let loaded = active
            .iter()
            .map(|identity| self.load_skill_instructions(identity))
            .collect::<Result<Vec<_>, _>>()?;
        let total_bytes = loaded.iter().try_fold(0usize, |total, skill| {
            total
                .checked_add(skill.instructions.len())
                .ok_or(ExtensionError::BoundExceeded)
        })?;
        if total_bytes > MAX_ACTIVE_TURN_SKILL_BYTES {
            return Err(ExtensionError::BoundExceeded);
        }
        Ok(loaded)
    }

    /// Resolves only currently enabled, trusted, explicitly reviewed hooks and
    /// re-verifies their immutable package bytes before any worker can start.
    pub fn prepared_hooks(
        &self,
        event: &str,
    ) -> Result<Vec<PreparedExtensionHook>, ExtensionError> {
        super::validate_identifier(event)?;
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        let mut prepared = Vec::new();
        for plugin in &self.document.snapshot.plugins {
            if plugin.trust != ExtensionTrustState::Trusted
                || !matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::Enabled
                        | ExtensionLifecycle::UpdateStaged
                        | ExtensionLifecycle::Executing
                )
            {
                continue;
            }
            let selected = plugin
                .hooks
                .iter()
                .filter(|hook| hook.reviewed && hook.event == event)
                .collect::<Vec<_>>();
            if selected.is_empty() {
                continue;
            }
            if !self.plugin_selector_active(plugin)? {
                return Err(ExtensionError::Conflict);
            }
            let installed = self.store.verify_installed(&plugin.digest, &policy)?;
            let manifest = parse_manifest(&read_bounded_text(
                &installed.path.join(PACKAGE_MANIFEST_NAME),
                512 * 1024,
            )?)?;
            if manifest.package.id != plugin.package_id || manifest.package_digest != plugin.digest
            {
                return Err(ExtensionError::MutableContent);
            }
            for hook in selected {
                let declaration = manifest
                    .hooks
                    .iter()
                    .find(|candidate| candidate.id == hook.hook_id)
                    .ok_or(ExtensionError::MutableContent)?;
                if declaration.event != hook.event
                    || declaration.executable != hook.name
                    || declaration.arguments != hook.arguments
                    || declaration.review_digest != hook.review_digest
                {
                    return Err(ExtensionError::MutableContent);
                }
                let contract = ReviewedHookContract {
                    hook_id: hook.hook_id.clone(),
                    event: hook.event.clone(),
                    executable: hook.name.clone(),
                    arguments: declaration.arguments.clone(),
                    review_digest: hook.review_digest.clone(),
                    timeout_ms: super::hook::DEFAULT_HOOK_TIMEOUT_MS,
                    maximum_output_bytes: super::hook::DEFAULT_HOOK_OUTPUT_BYTES,
                };
                contract.validate()?;
                prepared.push(PreparedExtensionHook {
                    activation: HookActivation {
                        package_id: plugin.package_id.clone(),
                        package_digest: plugin.digest.clone(),
                        generation: self.document.snapshot.generation,
                        enabled: true,
                        explicitly_reviewed: true,
                        signature_verified: true,
                        revoked: false,
                    },
                    contract,
                    package_root: installed.path.clone(),
                    grants: hook.grants.clone(),
                });
            }
        }
        if prepared.len() > super::hook::MAX_ACTIVE_HOOK_WORKERS {
            return Err(ExtensionError::BoundExceeded);
        }
        Ok(prepared)
    }

    /// Publishes the reviewable executing lifecycle before supervised hook
    /// workers receive input. The caller must prepare once, transition every
    /// participating package, then prepare again so worker activations bind to
    /// the new durable generation.
    pub fn begin_hook_execution(
        &mut self,
        expected_generation: u64,
        package_ids: &[String],
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(expected_generation)?;
        if package_ids.is_empty() || package_ids.len() > super::hook::MAX_ACTIVE_HOOK_WORKERS {
            return Err(ExtensionError::InvalidInput);
        }
        let unique = package_ids.iter().collect::<BTreeSet<_>>();
        if unique.len() != package_ids.len() {
            return Err(ExtensionError::InvalidInput);
        }
        let mut next = self.document.clone();
        for package_id in package_ids {
            super::validate_identifier(package_id)?;
            let plugin = plugin_mut(&mut next, package_id)?;
            if plugin.trust != ExtensionTrustState::Trusted
                || !matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::Enabled | ExtensionLifecycle::UpdateStaged
                )
                || !plugin.hooks.iter().any(|hook| hook.reviewed)
                || !self.plugin_selector_active(plugin)?
            {
                return Err(ExtensionError::Conflict);
            }
            plugin.lifecycle = ExtensionLifecycle::Executing;
        }
        self.commit(next, "reviewedHooksExecuting", None, None, now_ms)?;
        Ok(self.snapshot())
    }

    /// Publishes one batch of worker outcomes only if the extension generation
    /// that authorized the batch is still current. A concurrent disable,
    /// update, or revocation therefore invalidates all stale hook output.
    pub fn record_hook_execution_batch(
        &mut self,
        expected_generation: u64,
        records: &[ExtensionHookExecutionRecord],
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(expected_generation)?;
        if records.is_empty() || records.len() > super::hook::MAX_ACTIVE_HOOK_WORKERS {
            return Err(ExtensionError::InvalidInput);
        }
        let mut next = self.document.clone();
        for record in records {
            super::validate_identifier(&record.package_id)?;
            super::validate_identifier(&record.hook_id)?;
            super::validate_text(&record.detail)?;
            let plugin = plugin_mut(&mut next, &record.package_id)?;
            if plugin.trust != ExtensionTrustState::Trusted
                || plugin.lifecycle != ExtensionLifecycle::Executing
            {
                return Err(ExtensionError::Conflict);
            }
            let hook = plugin
                .hooks
                .iter_mut()
                .find(|hook| hook.hook_id == record.hook_id && hook.reviewed)
                .ok_or(ExtensionError::Conflict)?;
            hook.status = if record.succeeded { "ready" } else { "failed" }.into();
            hook.last_result = Some(record.detail.clone());
        }
        let completed_packages = records
            .iter()
            .map(|record| record.package_id.as_str())
            .collect::<BTreeSet<_>>();
        for package_id in completed_packages {
            let plugin = plugin_mut(&mut next, package_id)?;
            plugin.lifecycle = if plugin.staged_digest.is_some() {
                ExtensionLifecycle::UpdateStaged
            } else {
                ExtensionLifecycle::Enabled
            };
        }
        self.commit(next, "reviewedHooksExecuted", None, None, now_ms)?;
        Ok(self.snapshot())
    }

    pub fn review_hook(
        &mut self,
        input: ExtensionHookReviewInput,
        now_ms: u64,
    ) -> Result<ExtensionServiceSnapshot, ExtensionError> {
        self.require_generation(input.expected_generation)?;
        let plugin = self.plugin(&input.package_id)?.clone();
        if plugin.trust != ExtensionTrustState::Trusted
            || !matches!(
                plugin.lifecycle,
                ExtensionLifecycle::Enabled | ExtensionLifecycle::UpdateStaged
            )
        {
            return Err(ExtensionError::HookDenied);
        }
        let hook = plugin
            .hooks
            .iter()
            .find(|hook| hook.hook_id == input.hook_id)
            .ok_or(ExtensionError::InvalidInput)?;
        let contract = ReviewedHookContract {
            hook_id: hook.hook_id.clone(),
            event: hook.event.clone(),
            executable: hook.name.clone(),
            arguments: hook.arguments.clone(),
            review_digest: hook.review_digest.clone(),
            timeout_ms: super::hook::DEFAULT_HOOK_TIMEOUT_MS,
            maximum_output_bytes: super::hook::DEFAULT_HOOK_OUTPUT_BYTES,
        };
        contract.validate()?;
        let mut next = self.document.clone();
        let next_hook = plugin_mut(&mut next, &input.package_id)?
            .hooks
            .iter_mut()
            .find(|hook| hook.hook_id == input.hook_id)
            .ok_or(ExtensionError::InvalidInput)?;
        next_hook.reviewed = true;
        next_hook.status = "ready".into();
        let next_plugin = plugin_mut(&mut next, &input.package_id)?;
        next_plugin.has_reviewed_hooks = next_plugin.hooks.iter().any(|hook| hook.reviewed);
        self.commit(
            next,
            "hookReviewed",
            Some(&input.package_id),
            Some(&plugin.digest),
            now_ms,
        )?;
        Ok(self.snapshot())
    }

    pub fn publisher_link(
        &self,
        input: &ExtensionPublisherLinkInput,
    ) -> Result<String, ExtensionError> {
        let plugin = self.plugin(&input.package_id)?;
        let value = match input.link.as_str() {
            "website" => plugin.website.as_deref(),
            "terms" => plugin.terms.as_deref(),
            "privacy" => plugin.privacy_policy.as_deref(),
            _ => return Err(ExtensionError::InvalidInput),
        }
        .ok_or(ExtensionError::InvalidState)?;
        let url = url::Url::parse(value).map_err(|_| ExtensionError::InvalidInput)?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(ExtensionError::InvalidInput);
        }
        Ok(url.to_string())
    }

    fn load_marketplace(
        &self,
        resolved: ResolvedMarketplaceSource,
        trust_pin: &MarketplaceTrustPin,
        now_ms: u64,
    ) -> Result<(PersistedMarketplace, Vec<PackageCandidate>), ExtensionError> {
        let ResolvedMarketplaceSource {
            checkout_root: root,
            display_source,
            requested_ref,
            resolved_commit,
            sparse_paths,
        } = resolved;
        let origin: MarketplaceOriginDocument = read_bounded_toml(&root.join(MARKETPLACE_ORIGIN))?;
        if origin.schema_version != MARKETPLACE_SOURCE_SCHEMA_VERSION {
            return Err(ExtensionError::InvalidState);
        }
        let authority = PersistedAuthority {
            origin: origin.origin,
            key_id: origin.key_id,
            public_key: origin.public_key,
        };
        if authority.origin != trust_pin.origin
            || authority.key_id != trust_pin.key_id
            || public_key_fingerprint(&authority.public_key)? != trust_pin.public_key_sha256
        {
            return Err(ExtensionError::UntrustedOrigin);
        }
        let catalog_text = read_bounded_text(&root.join(MARKETPLACE_CATALOG), 512 * 1024)?;
        let revocations = self.document.revocations();
        let trusted_origins = [authority.trusted_origin()];
        let verified = verify_catalog(&catalog_text, &trusted_origins, &revocations)?;
        // A revoked package must stay catalog-visible so Settings can explain
        // and uninstall it. Source refresh still rejects revoked signing keys,
        // while package activation/install uses the full digest revocation set.
        let source_verification_revocations = RevocationSet {
            key_ids: revocations.key_ids.clone(),
            package_digests: BTreeSet::new(),
        };
        let mut candidates = Vec::new();
        for release in &verified.document.releases {
            let manifest_path = root.join(&release.manifest_path);
            if manifest_path.file_name().and_then(|value| value.to_str())
                != Some(PACKAGE_MANIFEST_NAME)
            {
                return Err(ExtensionError::InvalidInput);
            }
            let package_root = manifest_path.parent().ok_or(ExtensionError::InvalidInput)?;
            let policy =
                package_verification_policy(&trusted_origins, &source_verification_revocations);
            let package = verify_package_directory(package_root, &policy)?;
            if package.manifest.package.id != release.package_id
                || package.manifest.package.version != release.version
                || package.digest != release.package_digest
            {
                return Err(ExtensionError::VerificationFailed);
            }
            candidates.push(PackageCandidate {
                marketplace_id: verified.document.marketplace.id.clone(),
                root: package.root,
                manifest: package.manifest,
            });
        }
        let snapshot = MarketplaceSnapshot {
            marketplace_id: verified.document.marketplace.id.clone(),
            label: verified.document.marketplace.label,
            source: display_source.clone(),
            git_ref: requested_ref.clone(),
            resolved_commit: resolved_commit.clone(),
            sparse_paths: sparse_paths.clone(),
            catalog_digest: verified.digest,
            status: "ready".into(),
            trusted_origin: verified.document.marketplace.origin,
            origin_key_id: verified.document.marketplace.signing_key_id,
            package_count: u32::try_from(candidates.len())
                .map_err(|_| ExtensionError::BoundExceeded)?,
            last_checked_at_ms: Some(now_ms),
            detail: "Origin signature and every immutable release verified.".into(),
        };
        Ok((
            PersistedMarketplace {
                snapshot,
                root,
                authority,
                requested_source: display_source,
                requested_ref,
                sparse_paths,
                resolved_commit,
            },
            candidates,
        ))
    }

    fn commit(
        &mut self,
        mut next: ExtensionServiceDocument,
        event_kind: &str,
        package_id: Option<&str>,
        selector_digest: Option<&str>,
        now_ms: u64,
    ) -> Result<(), ExtensionError> {
        if now_ms == 0 {
            return Err(ExtensionError::InvalidInput);
        }
        let expected_generation = self.document.snapshot.generation;
        next.snapshot.generation = expected_generation
            .checked_add(1)
            .ok_or(ExtensionError::BoundExceeded)?;
        next.snapshot.last_event_id = self
            .document
            .snapshot
            .last_event_id
            .checked_add(1)
            .ok_or(ExtensionError::BoundExceeded)?;
        next.snapshot.marketplaces = next
            .marketplaces
            .iter()
            .map(|marketplace| marketplace.snapshot.clone())
            .collect();
        next.validate()?;
        let operation_id = format!("extension-operation-{}", next.snapshot.last_event_id);
        let audit = ExtensionAuditEvent {
            schema_version: EXTENSION_STATE_SCHEMA_VERSION,
            event_id: next.snapshot.last_event_id,
            generation: next.snapshot.generation,
            operation_id: operation_id.clone(),
            package_id: package_id.map(str::to_owned),
            event_kind: event_kind.into(),
            selector_digest: selector_digest.map(str::to_owned),
            result: "succeeded".into(),
            occurred_at_ms: now_ms,
        };
        self.database.save_extension_transition(
            ExtensionStateDocumentRecord {
                generation: next.snapshot.generation,
                canonical_document: serde_json::to_string(&next)?,
                updated_at_ms: now_ms,
            },
            ExtensionEventRecord {
                event_id: audit.event_id,
                generation: audit.generation,
                operation_id,
                package_id: audit.package_id.clone(),
                event_kind: audit.event_kind.clone(),
                selector_digest: audit.selector_digest.clone(),
                result: audit.result.clone(),
                canonical_document: serde_json::to_string(&audit)?,
                occurred_at_ms: now_ms,
            },
            Some(expected_generation),
        )?;
        self.document = next;
        Ok(())
    }

    fn commit_with_skill_projection(
        &mut self,
        mut next: ExtensionServiceDocument,
        event_kind: &str,
        package_id: Option<&str>,
        selector_digest: Option<&str>,
        now_ms: u64,
    ) -> Result<(), ExtensionError> {
        let (skills, discovered) = self.resolve_skills(&next)?;
        next.snapshot.skills = skills;
        self.commit(next, event_kind, package_id, selector_digest, now_ms)?;
        self.discovered = discovered;
        self.loaded
            .retain(|identity, _| self.discovered.contains_key(identity));
        Ok(())
    }

    fn synchronize_skills(&mut self, now_ms: u64) -> Result<(), ExtensionError> {
        let (skills, discovered) = self.resolve_skills(&self.document)?;
        if skills != self.document.snapshot.skills {
            let mut next = self.document.clone();
            next.snapshot.skills = skills;
            self.commit(next, "skillDiscoveryRefreshed", None, None, now_ms)?;
        }
        self.discovered = discovered;
        self.loaded.clear();
        Ok(())
    }

    fn reconcile_persisted_hook_contracts(&mut self, now_ms: u64) -> Result<(), ExtensionError> {
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        let mut next = self.document.clone();
        let mut changed = false;
        for plugin in &mut next.snapshot.plugins {
            if plugin.trust != ExtensionTrustState::Trusted
                || !matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::InstalledDisabled
                        | ExtensionLifecycle::Enabled
                        | ExtensionLifecycle::UpdateStaged
                        | ExtensionLifecycle::Executing
                        | ExtensionLifecycle::Failed
                        | ExtensionLifecycle::RolledBack
                )
            {
                continue;
            }
            let installed = self.store.verify_installed(&plugin.digest, &policy)?;
            let manifest = parse_manifest(&read_bounded_text(
                &installed.path.join(PACKAGE_MANIFEST_NAME),
                512 * 1024,
            )?)?;
            if manifest.package.id != plugin.package_id
                || manifest.package_digest != plugin.digest
                || manifest.hooks.len() != plugin.hooks.len()
            {
                return Err(ExtensionError::MutableContent);
            }
            for hook in &mut plugin.hooks {
                let declaration = manifest
                    .hooks
                    .iter()
                    .find(|candidate| candidate.id == hook.hook_id)
                    .ok_or(ExtensionError::MutableContent)?;
                if declaration.event != hook.event
                    || declaration.executable != hook.name
                    || declaration.review_digest != hook.review_digest
                {
                    return Err(ExtensionError::MutableContent);
                }
                if hook.arguments != declaration.arguments {
                    hook.arguments = declaration.arguments.clone();
                    hook.reviewed = false;
                    hook.status = "notReviewed".into();
                    hook.last_result = None;
                    changed = true;
                }
            }
            plugin.has_reviewed_hooks = plugin.hooks.iter().any(|hook| hook.reviewed);
        }
        if changed {
            self.commit(next, "legacyHookContractsReconciled", None, None, now_ms)?;
        }
        Ok(())
    }

    fn reconcile_selector_state(&mut self, now_ms: u64) -> Result<(), ExtensionError> {
        let mut next = self.document.clone();
        let mut changed = false;
        let trusted_origins = self.trusted_origins();
        let revocations = self.document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        for plugin in &mut next.snapshot.plugins {
            if plugin.lifecycle == ExtensionLifecycle::Executing {
                plugin.lifecycle = if plugin.staged_digest.is_some() {
                    ExtensionLifecycle::UpdateStaged
                } else {
                    ExtensionLifecycle::Enabled
                };
                plugin.failure_code = Some("worker-restart-recovered".into());
                changed = true;
            } else if plugin.lifecycle == ExtensionLifecycle::Enabling {
                plugin.lifecycle = ExtensionLifecycle::InstalledDisabled;
                plugin.active_digest = None;
                plugin.active_version = None;
                plugin.failure_code = Some("enable-restart-recovered-disabled".into());
                changed = true;
            }

            let selector = self.store.read_selector(&plugin.package_id)?;
            let expected_active = match plugin.lifecycle {
                ExtensionLifecycle::Enabled | ExtensionLifecycle::UpdateStaged => {
                    Some(plugin.digest.clone())
                }
                _ => None,
            };
            if let Some(expected_active) = expected_active {
                self.store.verify_installed(&expected_active, &policy)?;
                validate_activation_content(&self.store, plugin, &policy)?;
                let expected_last_known_good = plugin
                    .last_known_good_digest
                    .as_deref()
                    .unwrap_or(&expected_active);
                let selector_matches = selector.as_ref().is_some_and(|value| {
                    value.active_digest.as_deref() == Some(expected_active.as_str())
                        && value.last_known_good_digest.as_deref() == Some(expected_last_known_good)
                });
                if !selector_matches {
                    self.store.compare_and_swap_selector(
                        &plugin.package_id,
                        selector.as_ref().map(|value| value.generation),
                        Some(&expected_active),
                        Some(expected_last_known_good),
                    )?;
                    plugin.failure_code = Some("selector-repaired-from-durable-state".into());
                    changed = true;
                }
                plugin.active_digest = Some(expected_active);
                plugin.active_version = Some(plugin.version.clone());
            } else {
                let retained_last_known_good = if matches!(
                    plugin.lifecycle,
                    ExtensionLifecycle::InstalledDisabled | ExtensionLifecycle::RolledBack
                ) {
                    plugin.last_known_good_digest.as_deref()
                } else {
                    None
                };
                let selector_needs_repair = selector.as_ref().is_none_or(|value| {
                    value.active_digest.is_some()
                        || value.last_known_good_digest.as_deref() != retained_last_known_good
                });
                if selector_needs_repair {
                    self.store.compare_and_swap_selector(
                        &plugin.package_id,
                        selector.as_ref().map(|value| value.generation),
                        None,
                        retained_last_known_good,
                    )?;
                    plugin.active_digest = None;
                    plugin.active_version = None;
                    if plugin.failure_code.is_none() {
                        plugin.failure_code = Some("selector-repaired-from-durable-state".into());
                    }
                    changed = true;
                }
            }
        }
        if changed {
            self.commit(next, "selectorRestartRecovery", None, None, now_ms)?;
        }
        Ok(())
    }

    fn resolve_skills(
        &self,
        document: &ExtensionServiceDocument,
    ) -> Result<(Vec<SkillSnapshot>, BTreeMap<String, DiscoveredSkill>), ExtensionError> {
        let existing_enabled = document.skill_enabled.clone();
        let mut plugin_provided = Vec::new();
        let trusted_origins = document
            .marketplaces
            .iter()
            .map(|marketplace| marketplace.authority.trusted_origin())
            .collect::<Vec<_>>();
        let revocations = document.revocations();
        let policy = package_verification_policy(&trusted_origins, &revocations);
        for plugin in &document.snapshot.plugins {
            if !matches!(
                plugin.lifecycle,
                ExtensionLifecycle::InstalledDisabled
                    | ExtensionLifecycle::Enabled
                    | ExtensionLifecycle::UpdateStaged
                    | ExtensionLifecycle::Executing
                    | ExtensionLifecycle::RolledBack
            ) {
                continue;
            }
            let installed = self.store.verify_installed(&plugin.digest, &policy)?;
            let manifest = parse_manifest(&read_bounded_text(
                &installed.path.join(PACKAGE_MANIFEST_NAME),
                512 * 1024,
            )?)?;
            if manifest.package.id != plugin.package_id || manifest.package_digest != plugin.digest
            {
                return Err(ExtensionError::MutableContent);
            }
            for declaration in &manifest.skills {
                let entrypoint = installed.path.join(&declaration.path);
                let root = entrypoint.parent().ok_or(ExtensionError::InvalidInput)?;
                let skill = discover_skill(
                    root,
                    ExtensionSourceKind::PluginProvided,
                    &plugin.package_id,
                    Some(&plugin.package_id),
                )?;
                if skill.identity.skill_id != declaration.id {
                    return Err(ExtensionError::VerificationFailed);
                }
                let enabled = existing_enabled
                    .get(&skill.identity.stable_id())
                    .copied()
                    .unwrap_or(true);
                plugin_provided.push(SkillCandidate {
                    skill,
                    enabled,
                    eligible: true,
                    trusted_root: true,
                    provider_trusted_and_enabled: plugin.trust == ExtensionTrustState::Trusted
                        && matches!(
                            plugin.lifecycle,
                            ExtensionLifecycle::Enabled
                                | ExtensionLifecycle::UpdateStaged
                                | ExtensionLifecycle::Executing
                        ),
                    revoked: plugin.lifecycle == ExtensionLifecycle::Revoked,
                });
            }
        }
        let discovery = discover_skill_sources(&SkillSourceContext {
            filesystem_roots: self.skill_source_roots.clone(),
            plugin_provided,
            enabled_by_identity: existing_enabled,
        })?;
        let candidates = discovery.candidates;
        let discovered = discovery.discovered_by_identity;
        let mut invalid_skills = discovery.invalid_skills;
        let resolution = resolve_candidates(candidates, document.snapshot.selected_skill.as_ref())?;
        let mut skills = Vec::new();
        for record in resolution.records {
            let package_id = record.candidate.skill.package_id.clone();
            let version = package_id
                .as_deref()
                .and_then(|package_id| {
                    document
                        .snapshot
                        .plugins
                        .iter()
                        .find(|plugin| plugin.package_id == package_id)
                })
                .map(|plugin| plugin.version.clone())
                .unwrap_or_else(|| "local".into());
            skills.push(SkillSnapshot {
                identity: record.candidate.skill.identity.clone(),
                name: record.candidate.skill.portable.name.clone(),
                summary: record.candidate.skill.portable.description.clone(),
                package_id: package_id.clone(),
                source_label: source_label(&record.candidate.skill.identity),
                enabled: record.candidate.enabled,
                eligible: record.candidate.eligible,
                valid: true,
                active: record.active,
                shadowed: record.shadowed,
                instructions_loaded: false,
                collision_sources: record.collisions,
                diagnostic: None,
                version,
                is_installed: package_id.is_some(),
                eligibility_detail: if record.candidate.can_activate() {
                    "Eligible for the next turn.".into()
                } else {
                    "Unavailable until its source is trusted and enabled.".into()
                },
            });
        }
        skills.append(&mut invalid_skills);
        skills.sort_by(|left, right| left.identity.stable_id().cmp(&right.identity.stable_id()));
        Ok((skills, discovered))
    }

    fn trusted_origins(&self) -> Vec<TrustedOrigin> {
        self.document
            .marketplaces
            .iter()
            .map(|marketplace| marketplace.authority.trusted_origin())
            .collect()
    }

    fn require_generation(&self, expected: u64) -> Result<(), ExtensionError> {
        if expected != self.document.snapshot.generation {
            Err(ExtensionError::Conflict)
        } else {
            Ok(())
        }
    }

    fn plugin(&self, package_id: &str) -> Result<&PluginSnapshot, ExtensionError> {
        self.document
            .snapshot
            .plugins
            .iter()
            .find(|plugin| plugin.package_id == package_id)
            .ok_or(ExtensionError::InvalidInput)
    }

    fn plugin_selector_active(&self, plugin: &PluginSnapshot) -> Result<bool, ExtensionError> {
        Ok(self
            .store
            .read_selector(&plugin.package_id)?
            .is_some_and(|selector| {
                selector.active_digest.as_deref() == Some(plugin.digest.as_str())
            }))
    }

    fn latest_candidate(&self, package_id: &str) -> Result<&PackageCandidate, ExtensionError> {
        self.document
            .candidates
            .iter()
            .filter(|candidate| candidate.manifest.package.id == package_id)
            .max_by(|left, right| {
                Version::parse(&left.manifest.package.version)
                    .ok()
                    .cmp(&Version::parse(&right.manifest.package.version).ok())
            })
            .ok_or(ExtensionError::InvalidInput)
    }

    fn skill_identity(&self, stable_id: &str) -> Result<SkillQualifiedIdentity, ExtensionError> {
        self.document
            .snapshot
            .skills
            .iter()
            .find(|skill| skill.identity.stable_id() == stable_id)
            .map(|skill| skill.identity.clone())
            .ok_or(ExtensionError::InvalidInput)
    }
}

fn rebuild_plugin_projection(
    document: &mut ExtensionServiceDocument,
    now_ms: u64,
) -> Result<(), ExtensionError> {
    let previous = document
        .snapshot
        .plugins
        .iter()
        .map(|plugin| (plugin.package_id.clone(), plugin.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut package_ids = document
        .candidates
        .iter()
        .map(|candidate| candidate.manifest.package.id.clone())
        .collect::<BTreeSet<_>>();
    package_ids.extend(previous.keys().cloned());
    let mut plugins = Vec::new();
    for package_id in package_ids {
        let latest = document
            .candidates
            .iter()
            .filter(|candidate| candidate.manifest.package.id == package_id)
            .max_by(|left, right| {
                Version::parse(&left.manifest.package.version)
                    .ok()
                    .cmp(&Version::parse(&right.manifest.package.version).ok())
            });
        let mut plugin = previous
            .get(&package_id)
            .cloned()
            .or_else(|| latest.map(|candidate| plugin_from_candidate(candidate, now_ms)))
            .ok_or(ExtensionError::InvalidState)?;
        let Some(latest) = latest else {
            plugin.available_version = None;
            plugin.available_digest = None;
            plugin.failure_code = Some("catalog-release-unavailable".into());
            if plugin.lifecycle == ExtensionLifecycle::Available {
                plugin.trust = ExtensionTrustState::Invalid;
            }
            plugins.push(plugin);
            continue;
        };
        if Version::parse(&latest.manifest.package.version).ok()
            > Version::parse(&plugin.version).ok()
        {
            plugin.available_version = Some(latest.manifest.package.version.clone());
            plugin.available_digest = Some(latest.manifest.package_digest.clone());
        }
        if plugin.lifecycle == ExtensionLifecycle::Available {
            plugin = plugin_from_candidate(latest, now_ms);
        }
        if document.revoked_packages.contains(&package_id)
            || document.revoked_digests.contains(&plugin.digest)
            || document.revoked_keys.contains(&plugin.origin_key_id)
            || document.revoked_keys.contains(&plugin.content_key_id)
        {
            plugin.trust = ExtensionTrustState::Revoked;
            plugin.revocation_reason = previous
                .get(&package_id)
                .and_then(|prior| prior.revocation_reason.clone());
        }
        plugins.push(plugin);
    }
    plugins.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    document.snapshot.plugins = plugins;
    Ok(())
}

fn plugin_from_candidate(candidate: &PackageCandidate, now_ms: u64) -> PluginSnapshot {
    let package = &candidate.manifest.package;
    PluginSnapshot {
        package_id: package.id.clone(),
        package_kind: package.kind,
        name: package.name.clone(),
        summary: package.summary.clone(),
        publisher: package.publisher.clone(),
        version: package.version.clone(),
        digest: candidate.manifest.package_digest.clone(),
        origin_key_id: candidate.manifest.trust.origin_key_id.clone(),
        content_key_id: candidate.manifest.trust.delegated_content_key_id.clone(),
        trust: ExtensionTrustState::Trusted,
        lifecycle: ExtensionLifecycle::Available,
        capabilities: candidate.manifest.capabilities.clone(),
        website: package.website.clone(),
        terms: package.terms.clone(),
        privacy_policy: package.privacy_policy.clone(),
        active_digest: None,
        last_known_good_digest: None,
        failure_code: None,
        has_reviewed_hooks: false,
        marketplace_id: candidate.marketplace_id.clone(),
        source: candidate.root.display().to_string(),
        compatibility: candidate.manifest.compatibility.c4os.clone(),
        verified_at_ms: Some(now_ms),
        hooks: candidate
            .manifest
            .hooks
            .iter()
            .map(|hook| PluginHookSnapshot {
                hook_id: hook.id.clone(),
                name: hook.executable.clone(),
                arguments: hook.arguments.clone(),
                event: hook.event.clone(),
                review_digest: hook.review_digest.clone(),
                reviewed: false,
                status: "notReviewed".into(),
                grants: candidate.manifest.capabilities.clone(),
                last_result: None,
            })
            .collect(),
        settings: candidate
            .manifest
            .settings
            .iter()
            .map(|setting| PluginSettingSnapshot {
                setting_id: setting.id.clone(),
                label: setting.label.clone(),
                description: setting.description.clone(),
                kind: match setting.kind {
                    PackageSettingKind::Boolean => "boolean",
                    PackageSettingKind::Integer => "integer",
                    PackageSettingKind::String => "string",
                    PackageSettingKind::Select => "select",
                    PackageSettingKind::CredentialReference => "credentialReference",
                }
                .into(),
                required: setting.required,
                choices: setting.choices.clone(),
            })
            .collect(),
        apps: candidate
            .manifest
            .apps
            .iter()
            .map(|app| PluginAppSnapshot {
                app_id: app.id.clone(),
                title: app.title.clone(),
                summary: app.summary.clone(),
                setting_ids: app.settings.clone(),
            })
            .collect(),
        mcp_servers: candidate
            .manifest
            .mcp_servers
            .iter()
            .map(|server| PluginMcpServerSnapshot {
                server_id: server.id.clone(),
                name: server.name.clone(),
                transport: match server.transport {
                    PackageMcpTransport::Stdio => "stdio",
                    PackageMcpTransport::StreamableHttp => "streamableHttp",
                }
                .into(),
                setting_ids: server.settings.clone(),
            })
            .collect(),
        available_version: None,
        available_digest: None,
        staged_version: None,
        staged_digest: None,
        active_version: None,
        last_known_good_version: None,
        revocation_reason: None,
    }
}

fn apply_candidate_to_plugin(
    plugin: &mut PluginSnapshot,
    candidate: &PackageCandidate,
    now_ms: u64,
) {
    let prior_lifecycle = plugin.lifecycle;
    *plugin = plugin_from_candidate(candidate, now_ms);
    plugin.lifecycle = prior_lifecycle;
    // Every package version is a new reviewed execution contract. Even an
    // unchanged executable digest may carry changed arguments, event routing,
    // or grants, so update and rollback always require an explicit re-review.
}

fn package_verification_policy<'a>(
    trusted_origins: &'a [TrustedOrigin],
    revocations: &'a RevocationSet,
) -> PackageVerificationPolicy<'a> {
    PackageVerificationPolicy {
        c4os_version: env!("CARGO_PKG_VERSION"),
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        trusted_origins,
        revocations,
    }
}

fn plugin_mut<'a>(
    document: &'a mut ExtensionServiceDocument,
    package_id: &str,
) -> Result<&'a mut PluginSnapshot, ExtensionError> {
    document
        .snapshot
        .plugins
        .iter_mut()
        .find(|plugin| plugin.package_id == package_id)
        .ok_or(ExtensionError::InvalidInput)
}

fn validate_activation_content(
    store: &ExtensionStore,
    plugin: &PluginSnapshot,
    policy: &PackageVerificationPolicy<'_>,
) -> Result<(), ExtensionError> {
    let installed = store.verify_installed(&plugin.digest, policy)?;
    let manifest_text = read_bounded_text(&installed.path.join(PACKAGE_MANIFEST_NAME), 512 * 1024)?;
    let manifest = parse_manifest(&manifest_text)?;
    for skill in &manifest.skills {
        let entrypoint = installed.path.join(&skill.path);
        let root = entrypoint.parent().ok_or(ExtensionError::InvalidInput)?;
        let discovered = discover_skill(
            root,
            ExtensionSourceKind::PluginProvided,
            &plugin.package_id,
            Some(&plugin.package_id),
        )?;
        if discovered.identity.skill_id != skill.id {
            return Err(ExtensionError::VerificationFailed);
        }
    }
    Ok(())
}

fn sort_and_validate_candidates(
    candidates: &mut Vec<PackageCandidate>,
) -> Result<(), ExtensionError> {
    candidates.sort_by(|left, right| {
        left.manifest
            .package
            .id
            .cmp(&right.manifest.package.id)
            .then_with(|| {
                left.manifest
                    .package
                    .version
                    .cmp(&right.manifest.package.version)
            })
            .then_with(|| left.marketplace_id.cmp(&right.marketplace_id))
    });
    if candidates.windows(2).any(|pair| {
        pair[0].manifest.package.id == pair[1].manifest.package.id
            && pair[0].manifest.package.version == pair[1].manifest.package.version
    }) {
        return Err(ExtensionError::Conflict);
    }
    Ok(())
}

fn read_bounded_text(path: &Path, maximum: u64) -> Result<String, ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return Err(ExtensionError::BoundExceeded);
    }
    fs::read_to_string(path).map_err(Into::into)
}

fn read_bounded_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ExtensionError> {
    let text = read_bounded_text(path, 64 * 1024)?;
    toml::from_str(&text).map_err(Into::into)
}

fn ensure_private_skill_directory(path: &Path) -> Result<(), ExtensionError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(ExtensionError::InvalidState);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(path)?;
        }
        Err(error) => return Err(error.into()),
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn write_private_skill_file(path: &Path, bytes: &[u8]) -> Result<(), ExtensionError> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_skill_directory(path: &Path) -> Result<(), ExtensionError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

fn source_label(identity: &SkillQualifiedIdentity) -> String {
    match identity.source_kind {
        ExtensionSourceKind::ProjectLocal => format!("Project · {}", identity.source_id),
        ExtensionSourceKind::WorkspaceLocal => format!("Workspace · {}", identity.source_id),
        ExtensionSourceKind::UserGlobal => format!("User · {}", identity.source_id),
        ExtensionSourceKind::PluginProvided => format!("Plugin · {}", identity.source_id),
        ExtensionSourceKind::Bundled => format!("Bundled · {}", identity.source_id),
    }
}

fn default_skill_source_roots(home: &Path) -> Vec<SkillSourceRoot> {
    vec![SkillSourceRoot {
        source_kind: ExtensionSourceKind::UserGlobal,
        source_id: "user-global".into(),
        root: home.join("skills/user"),
        trusted: true,
    }]
}
