use std::{fs, path::Path, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use c4os_lib::{
    core::database::{
        DatabaseActor, DatabaseDescriptor, ExtensionStateDocumentRecord, SnapshotQuery,
    },
    extension::{
        ExtensionError, ExtensionLifecycle, ExtensionPackageKind, ExtensionServiceSnapshot,
        ExtensionTrustState,
        package::{
            CatalogDocument, CatalogMarketplace, CatalogRelease, PackageCompatibility,
            PackageHookDeclaration, PackageManifest, PackageMetadata, PackageSkillDeclaration,
            PackageTrust, canonical_catalog_signing_bytes, canonical_content_signing_bytes,
            canonical_origin_signing_bytes, digest_inventory, inventory_for_root,
            public_key_fingerprint,
        },
        service::{
            ExtensionHookExecutionRecord, ExtensionHookReviewInput, ExtensionPackageInput,
            ExtensionService, ExtensionSkillInput, MarketplaceSourceInput,
        },
        store::{ExtensionStore, PackageSelector},
    },
};
use ed25519_dalek::{Signer, SigningKey};
use tempfile::TempDir;

const PACKAGE_ID: &str = "transaction-plugin";
const SKILL_ID: &str = "transaction-skill";

#[test]
fn stale_and_invalid_lifecycle_requests_have_no_selector_or_authority_side_effects() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    install_and_enable(&mut service, &marketplace);

    let before_snapshot = service.snapshot();
    let before_selector = selector(&home);
    let before_state = database.extension_state_document().unwrap().unwrap();
    let before_events = database
        .extension_event_page(None, SnapshotQuery::new(16).unwrap())
        .unwrap();

    assert!(matches!(
        service.disable(package_input(before_snapshot.generation - 1), 50),
        Err(ExtensionError::Conflict)
    ));
    assert!(matches!(
        service.enable(package_input(before_snapshot.generation), 60),
        Err(ExtensionError::InvalidState)
    ));
    assert!(matches!(
        service.activate_staged_update(package_input(before_snapshot.generation), 70),
        Err(ExtensionError::InvalidState)
    ));

    assert_eq!(service.snapshot(), before_snapshot);
    assert_eq!(selector(&home), before_selector);
    assert_eq!(
        database.extension_state_document().unwrap().unwrap(),
        before_state
    );
    assert_eq!(
        database
            .extension_event_page(None, SnapshotQuery::new(16).unwrap())
            .unwrap(),
        before_events
    );
    assert_eq!(service.active_turn_skills().unwrap().len(), 1);
}

#[test]
fn failed_customization_removes_its_copy_when_an_alias_directory_owns_the_user_identity() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut service, &marketplace);
    let plugin_identity = enabled.skills[0].identity.stable_id();

    let existing = home.join("skills/user/existing-alias");
    fs::create_dir_all(&existing).unwrap();
    fs::write(
        existing.join("SKILL.md"),
        format!(
            "---\nname: {SKILL_ID}\ndescription: Existing user-owned transaction Skill\n---\nUse the existing user-owned transaction Skill.\n"
        ),
    )
    .unwrap();
    let before = service.snapshot();

    assert!(
        service
            .customize_skill(
                ExtensionSkillInput {
                    expected_generation: before.generation,
                    skill_identity: plugin_identity,
                },
                50,
            )
            .is_err(),
        "duplicate source-qualified user identity must fail customization"
    );
    assert_eq!(service.snapshot(), before);

    let mut user_entries = fs::read_dir(home.join("skills/user"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    user_entries.sort();
    assert_eq!(
        user_entries,
        vec![std::ffi::OsString::from("existing-alias")]
    );

    drop(service);
    let mut restored = ExtensionService::restore(database, &home, 60).unwrap();
    let restored_snapshot = restored.snapshot();
    let user_skill = restored_snapshot
        .skills
        .iter()
        .find(|skill| {
            skill.identity.source_kind == c4os_lib::extension::ExtensionSourceKind::UserGlobal
                && skill.identity.skill_id == SKILL_ID
        })
        .expect("existing user Skill remains discoverable");
    assert!(user_skill.valid && user_skill.eligible && user_skill.active);
    assert!(restored_snapshot.skills.iter().any(|skill| {
        skill.identity.source_kind == c4os_lib::extension::ExtensionSourceKind::PluginProvided
            && skill.identity.skill_id == SKILL_ID
            && skill.shadowed
    }));
    assert!(
        restored
            .load_skill_instructions(&user_skill.identity.stable_id())
            .unwrap()
            .instructions
            .contains("existing user-owned transaction Skill")
    );
}

#[cfg(unix)]
#[test]
fn failed_enable_selector_publication_compensates_exact_installed_disabled_state() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    let installed = service.install_disabled(package_input(2), 30).unwrap();
    let prior_snapshot = service.snapshot();
    let prior_document = database.extension_state_document().unwrap().unwrap();
    let prior_selector = selector(&home);

    set_selector_directory_mode(&home, 0o500);
    let result = service.enable(package_input(installed.generation), 40);
    set_selector_directory_mode(&home, 0o700);
    assert!(
        result.is_err(),
        "the fixture must fail selector publication"
    );
    assert_compensated_to_prior_state(
        &database,
        &service,
        &home,
        &prior_snapshot,
        &prior_document,
        &prior_selector,
        "packageEnabledForNextTurn",
    );
    let compensated_generation = database
        .extension_state_document()
        .unwrap()
        .unwrap()
        .generation;

    drop(service);
    let mut restored = ExtensionService::restore(Arc::clone(&database), &home, 50).unwrap();
    assert_same_working_snapshot(&restored.snapshot(), &prior_snapshot);
    assert_eq!(selector(&home), prior_selector);
    assert!(restored.active_turn_skills().unwrap().is_empty());
    assert_eq!(
        database
            .extension_state_document()
            .unwrap()
            .unwrap()
            .generation,
        compensated_generation,
        "restart must not need a second recovery transition"
    );
}

#[cfg(unix)]
#[test]
fn failed_update_selector_publication_compensates_exact_staged_state() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut service, &marketplace);
    write_signed_update(&marketplace);
    let available = service.refresh_catalogs(enabled.generation, 50).unwrap();
    let staged = service
        .stage_update(package_input(available.generation), 60)
        .unwrap();
    let prior_snapshot = service.snapshot();
    let prior_document = database.extension_state_document().unwrap().unwrap();
    let prior_selector = selector(&home);

    set_selector_directory_mode(&home, 0o500);
    let result = service.activate_staged_update(package_input(staged.generation), 70);
    set_selector_directory_mode(&home, 0o700);
    assert!(
        result.is_err(),
        "the fixture must fail selector publication"
    );
    assert_compensated_to_prior_state(
        &database,
        &service,
        &home,
        &prior_snapshot,
        &prior_document,
        &prior_selector,
        "packageUpdateActivated",
    );
    let compensated_generation = database
        .extension_state_document()
        .unwrap()
        .unwrap()
        .generation;

    drop(service);
    let mut restored = ExtensionService::restore(Arc::clone(&database), &home, 80).unwrap();
    assert_same_working_snapshot(&restored.snapshot(), &prior_snapshot);
    assert_eq!(selector(&home), prior_selector);
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::UpdateStaged
    );
    assert!(
        restored.active_turn_skills().unwrap()[0]
            .instructions
            .contains("original transaction Skill")
    );
    assert_eq!(
        database
            .extension_state_document()
            .unwrap()
            .unwrap()
            .generation,
        compensated_generation,
        "restart must not need a second recovery transition"
    );
}

#[cfg(unix)]
#[test]
fn failed_rollback_selector_publication_compensates_exact_updated_state() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut service, &marketplace);
    write_signed_update(&marketplace);
    let available = service.refresh_catalogs(enabled.generation, 50).unwrap();
    let staged = service
        .stage_update(package_input(available.generation), 60)
        .unwrap();
    let updated = service
        .activate_staged_update(package_input(staged.generation), 70)
        .unwrap();
    let prior_snapshot = service.snapshot();
    let prior_document = database.extension_state_document().unwrap().unwrap();
    let prior_selector = selector(&home);

    set_selector_directory_mode(&home, 0o500);
    let result = service.rollback(package_input(updated.generation), 80);
    set_selector_directory_mode(&home, 0o700);
    assert!(
        result.is_err(),
        "the fixture must fail selector publication"
    );
    assert_compensated_to_prior_state(
        &database,
        &service,
        &home,
        &prior_snapshot,
        &prior_document,
        &prior_selector,
        "packageRolledBackDisabled",
    );
    let compensated_generation = database
        .extension_state_document()
        .unwrap()
        .unwrap()
        .generation;

    drop(service);
    let mut restored = ExtensionService::restore(Arc::clone(&database), &home, 90).unwrap();
    assert_same_working_snapshot(&restored.snapshot(), &prior_snapshot);
    assert_eq!(selector(&home), prior_selector);
    assert_eq!(restored.snapshot().plugins[0].version, "0.2.0");
    assert!(
        restored.active_turn_skills().unwrap()[0]
            .instructions
            .contains("updated transaction Skill")
    );
    assert_eq!(
        database
            .extension_state_document()
            .unwrap()
            .unwrap()
            .generation,
        compensated_generation,
        "restart must not need a second recovery transition"
    );
}

#[test]
fn disabling_an_executing_reviewed_hook_invalidates_stale_completion_and_restart_authority() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut service, &marketplace);
    let reviewed = service
        .review_hook(
            ExtensionHookReviewInput {
                expected_generation: enabled.generation,
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
            },
            50,
        )
        .unwrap();
    let executing = service
        .begin_hook_execution(reviewed.generation, &[PACKAGE_ID.into()], 60)
        .unwrap();
    assert_eq!(
        executing.plugins[0].lifecycle,
        ExtensionLifecycle::Executing
    );

    let disabled = service
        .disable(package_input(executing.generation), 70)
        .unwrap();
    assert_eq!(
        disabled.plugins[0].lifecycle,
        ExtensionLifecycle::InstalledDisabled
    );
    assert!(service.active_turn_skills().unwrap().is_empty());
    assert!(service.prepared_hooks("before-turn").unwrap().is_empty());
    assert!(matches!(
        service.record_hook_execution_batch(
            executing.generation,
            &[ExtensionHookExecutionRecord {
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
                succeeded: true,
                detail: "stale completion must not publish".into(),
            }],
            80,
        ),
        Err(ExtensionError::Conflict)
    ));

    drop(service);
    let mut restored = ExtensionService::restore(database, &home, 90).unwrap();
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::InstalledDisabled
    );
    assert!(restored.active_turn_skills().unwrap().is_empty());
    assert!(restored.prepared_hooks("before-turn").unwrap().is_empty());
}

#[test]
fn failed_update_publication_restores_the_prior_selector_and_restart_authority() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut coordinator = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut coordinator, &marketplace);
    write_signed_update(&marketplace);
    let available = coordinator
        .refresh_catalogs(enabled.generation, 50)
        .unwrap();
    let staged = coordinator
        .stage_update(package_input(available.generation), 60)
        .unwrap();
    let prior_digest = staged.plugins[0].digest.clone();
    let staged_digest = staged.plugins[0].staged_digest.clone().unwrap();
    assert_eq!(
        staged.plugins[0].lifecycle,
        ExtensionLifecycle::UpdateStaged
    );

    let mut stale = ExtensionService::restore(Arc::clone(&database), &home, 61).unwrap();
    advance_without_selector_change(&mut coordinator, staged.generation, 70);
    assert!(
        stale
            .activate_staged_update(package_input(staged.generation), 80)
            .is_err()
    );

    let selector_after_failure = selector(&home);
    assert_eq!(
        selector_after_failure.active_digest.as_deref(),
        Some(prior_digest.as_str()),
        "a failed durable publication must compensate the selector switch"
    );
    assert_ne!(
        selector_after_failure.active_digest.as_deref(),
        Some(staged_digest.as_str())
    );

    drop(stale);
    drop(coordinator);
    let mut restored = ExtensionService::restore(database, &home, 90).unwrap();
    assert_eq!(restored.snapshot().generation, staged.generation + 1);
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::UpdateStaged
    );
    assert_eq!(restored.snapshot().plugins[0].digest, prior_digest);
    assert_eq!(restored.active_turn_skills().unwrap().len(), 1);
    assert!(
        restored.active_turn_skills().unwrap()[0]
            .instructions
            .contains("original transaction Skill")
    );
}

#[test]
fn failed_rollback_publication_restores_the_updated_selector_and_restart_authority() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut coordinator = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut coordinator, &marketplace);
    write_signed_update(&marketplace);
    let available = coordinator
        .refresh_catalogs(enabled.generation, 50)
        .unwrap();
    let staged = coordinator
        .stage_update(package_input(available.generation), 60)
        .unwrap();
    let updated = coordinator
        .activate_staged_update(package_input(staged.generation), 70)
        .unwrap();
    let updated_digest = updated.plugins[0].digest.clone();
    let prior_digest = updated.plugins[0].last_known_good_digest.clone().unwrap();

    let mut stale = ExtensionService::restore(Arc::clone(&database), &home, 71).unwrap();
    advance_without_selector_change(&mut coordinator, updated.generation, 80);
    assert!(
        stale
            .rollback(package_input(updated.generation), 90)
            .is_err()
    );

    let selector_after_failure = selector(&home);
    assert_eq!(
        selector_after_failure.active_digest.as_deref(),
        Some(updated_digest.as_str()),
        "a failed durable rollback must compensate both selector mutations"
    );
    assert_eq!(
        selector_after_failure.last_known_good_digest.as_deref(),
        Some(prior_digest.as_str())
    );

    drop(stale);
    drop(coordinator);
    let mut restored = ExtensionService::restore(database, &home, 100).unwrap();
    assert_eq!(restored.snapshot().generation, updated.generation + 1);
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::Enabled
    );
    assert_eq!(restored.snapshot().plugins[0].version, "0.2.0");
    assert!(
        restored.active_turn_skills().unwrap()[0]
            .instructions
            .contains("updated transaction Skill")
    );
}

#[test]
fn failed_package_revocation_publication_preserves_existing_authority_on_restart() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut coordinator = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut coordinator, &marketplace);
    let active_digest = enabled.plugins[0].digest.clone();

    let mut stale = ExtensionService::restore(Arc::clone(&database), &home, 41).unwrap();
    advance_without_selector_change(&mut coordinator, enabled.generation, 50);
    assert!(
        stale
            .revoke(
                package_input(enabled.generation),
                "transaction conflict",
                60,
            )
            .is_err()
    );
    assert_eq!(
        selector(&home).active_digest.as_deref(),
        Some(active_digest.as_str()),
        "a revocation that was not durably published must not clear authority"
    );

    drop(stale);
    drop(coordinator);
    let mut restored = ExtensionService::restore(database, &home, 70).unwrap();
    assert_eq!(restored.snapshot().generation, enabled.generation + 1);
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::Enabled
    );
    assert_eq!(
        restored.snapshot().plugins[0].trust,
        ExtensionTrustState::Trusted
    );
    assert_eq!(restored.active_turn_skills().unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn package_revocation_is_durable_before_selector_cleanup_failure_and_restart() {
    use std::os::unix::fs::PermissionsExt;

    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();
    let enabled = install_and_enable(&mut service, &marketplace);

    let selectors = home.join("extensions/selectors");
    fs::set_permissions(&selectors, fs::Permissions::from_mode(0o500)).unwrap();
    let cleanup_result =
        service.revoke(package_input(enabled.generation), "durable revocation", 50);
    fs::set_permissions(&selectors, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        cleanup_result.is_err(),
        "the fixture must inject a selector-cleanup failure"
    );

    let durable = database.extension_state_document().unwrap().unwrap();
    assert_eq!(
        durable.generation,
        enabled.generation + 1,
        "revocation must commit before best-effort selector cleanup"
    );
    assert!(durable.canonical_document.contains("\"trust\":\"revoked\""));

    drop(service);
    let restored = ExtensionService::restore(database, &home, 60).unwrap();
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::Revoked
    );
    assert_eq!(
        restored.snapshot().plugins[0].trust,
        ExtensionTrustState::Revoked
    );
    assert!(restored.snapshot().skills.is_empty());
    assert!(selector(&home).active_digest.is_none());
}

fn install_and_enable(
    service: &mut ExtensionService,
    marketplace: &Path,
) -> c4os_lib::extension::ExtensionServiceSnapshot {
    service
        .add_marketplace(marketplace_input(marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap()
}

fn advance_without_selector_change(
    service: &mut ExtensionService,
    expected_generation: u64,
    now_ms: u64,
) {
    let skill_identity = service.snapshot().skills[0].identity.stable_id();
    service
        .select_skill(
            ExtensionSkillInput {
                expected_generation,
                skill_identity,
            },
            now_ms,
        )
        .unwrap();
}

fn package_input(expected_generation: u64) -> ExtensionPackageInput {
    ExtensionPackageInput {
        expected_generation,
        package_id: PACKAGE_ID.into(),
    }
}

fn selector(home: &Path) -> PackageSelector {
    ExtensionStore::new(home.join("extensions"))
        .unwrap()
        .read_selector(PACKAGE_ID)
        .unwrap()
        .unwrap()
}

#[cfg(unix)]
fn set_selector_directory_mode(home: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(
        home.join("extensions/selectors"),
        fs::Permissions::from_mode(mode),
    )
    .unwrap();
}

fn assert_compensated_to_prior_state(
    database: &DatabaseActor,
    service: &ExtensionService,
    home: &Path,
    prior_snapshot: &ExtensionServiceSnapshot,
    prior_document: &ExtensionStateDocumentRecord,
    prior_selector: &PackageSelector,
    attempted_event_kind: &str,
) {
    assert_same_working_snapshot(&service.snapshot(), prior_snapshot);
    assert_eq!(&selector(home), prior_selector);
    let compensated_document = database.extension_state_document().unwrap().unwrap();
    assert!(compensated_document.generation > prior_document.generation);
    assert_eq!(
        normalized_service_document(&compensated_document),
        normalized_service_document(prior_document),
        "compensation must restore the exact prior service document"
    );
    let events = database
        .extension_event_page(None, SnapshotQuery::new(64).unwrap())
        .unwrap();
    assert!(
        events.events.iter().any(|event| {
            event.event_kind == attempted_event_kind
                && event.generation == prior_document.generation + 1
        }),
        "the operation document must commit before selector publication is forced to fail"
    );
}

fn assert_same_working_snapshot(
    actual: &ExtensionServiceSnapshot,
    expected: &ExtensionServiceSnapshot,
) {
    let mut actual = serde_json::to_value(actual).unwrap();
    let mut expected = serde_json::to_value(expected).unwrap();
    for value in [&mut actual, &mut expected] {
        let object = value.as_object_mut().unwrap();
        object.remove("generation");
        object.remove("lastEventId");
    }
    assert_eq!(actual, expected);
}

fn normalized_service_document(record: &ExtensionStateDocumentRecord) -> serde_json::Value {
    let mut document: serde_json::Value = serde_json::from_str(&record.canonical_document).unwrap();
    let snapshot = document
        .get_mut("snapshot")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    snapshot.remove("generation");
    snapshot.remove("lastEventId");
    document
}

fn marketplace_input(root: &Path) -> MarketplaceSourceInput {
    let origin_key = SigningKey::from_bytes(&[17u8; 32]);
    MarketplaceSourceInput {
        source: root.display().to_string(),
        git_ref: None,
        sparse_paths: Vec::new(),
        trusted_origin: "local:c4os-transaction-tests".into(),
        signing_key_id: "transaction-origin".into(),
        public_key_sha256: public_key_fingerprint(
            &BASE64.encode(origin_key.verifying_key().as_bytes()),
        )
        .unwrap(),
    }
}

fn write_signed_marketplace(root: &Path) {
    let origin_key = SigningKey::from_bytes(&[17u8; 32]);
    let content_key = SigningKey::from_bytes(&[19u8; 32]);
    let (manifest, package_digest) = write_signed_release(
        root,
        "transaction-v1",
        "0.1.0",
        "Use the original transaction Skill.",
        &origin_key,
        &content_key,
    );
    let mut catalog = CatalogDocument {
        schema_version: 1,
        marketplace: CatalogMarketplace {
            id: "transaction-tests".into(),
            label: "Transaction tests".into(),
            origin: manifest.trust.origin.clone(),
            signing_key_id: manifest.trust.origin_key_id.clone(),
        },
        releases: vec![CatalogRelease {
            package_id: PACKAGE_ID.into(),
            version: "0.1.0".into(),
            manifest_path: "transaction-v1/c4os-package.toml".into(),
            package_digest,
        }],
        signature: String::new(),
    };
    sign_catalog(&mut catalog, &origin_key);
    fs::write(
        root.join("c4os-marketplace.toml"),
        toml::to_string(&catalog).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("c4os-marketplace-origin.toml"),
        format!(
            "schemaVersion = 1\norigin = {:?}\nkeyId = {:?}\npublicKey = {:?}\n",
            catalog.marketplace.origin,
            catalog.marketplace.signing_key_id,
            BASE64.encode(origin_key.verifying_key().as_bytes()),
        ),
    )
    .unwrap();
}

fn write_signed_update(root: &Path) {
    let origin_key = SigningKey::from_bytes(&[17u8; 32]);
    let content_key = SigningKey::from_bytes(&[19u8; 32]);
    let (_, package_digest) = write_signed_release(
        root,
        "transaction-v2",
        "0.2.0",
        "Use the updated transaction Skill.",
        &origin_key,
        &content_key,
    );
    let catalog_path = root.join("c4os-marketplace.toml");
    let mut catalog: CatalogDocument =
        toml::from_str(&fs::read_to_string(&catalog_path).unwrap()).unwrap();
    catalog.releases.push(CatalogRelease {
        package_id: PACKAGE_ID.into(),
        version: "0.2.0".into(),
        manifest_path: "transaction-v2/c4os-package.toml".into(),
        package_digest,
    });
    sign_catalog(&mut catalog, &origin_key);
    fs::write(catalog_path, toml::to_string(&catalog).unwrap()).unwrap();
}

fn write_signed_release(
    root: &Path,
    directory: &str,
    version: &str,
    instructions: &str,
    origin_key: &SigningKey,
    content_key: &SigningKey,
) -> (PackageManifest, String) {
    let package_root = root.join(directory);
    fs::create_dir_all(package_root.join("skill")).unwrap();
    fs::create_dir_all(package_root.join("hooks")).unwrap();
    fs::write(
        package_root.join("skill/SKILL.md"),
        format!(
            "---\nname: {SKILL_ID}\ndescription: Transaction fixture Skill\n---\n{instructions}\n"
        ),
    )
    .unwrap();
    fs::write(
        package_root.join("hooks/before-turn.mjs"),
        "process.stdout.write(JSON.stringify({protocolVersion:1,proposals:[]}));\n",
    )
    .unwrap();
    let inventory = inventory_for_root(&package_root).unwrap();
    let package_digest = digest_inventory(&inventory);
    let hook_digest = inventory
        .iter()
        .find(|entry| entry.path == "hooks/before-turn.mjs")
        .unwrap()
        .sha256
        .clone();
    let mut manifest = PackageManifest {
        schema_version: 1,
        package: PackageMetadata {
            id: PACKAGE_ID.into(),
            name: "Transaction Plugin".into(),
            summary: "Signed lifecycle transaction fixture".into(),
            publisher: "C4OS Tests".into(),
            version: version.into(),
            kind: ExtensionPackageKind::Plugin,
            website: None,
            terms: None,
            privacy_policy: None,
        },
        compatibility: PackageCompatibility {
            c4os: "^0.1".into(),
            operating_systems: vec![std::env::consts::OS.into()],
            architectures: vec![std::env::consts::ARCH.into()],
        },
        trust: PackageTrust {
            origin: "local:c4os-transaction-tests".into(),
            origin_key_id: "transaction-origin".into(),
            delegated_content_key_id: "transaction-content".into(),
            delegated_content_public_key: BASE64.encode(content_key.verifying_key().as_bytes()),
            origin_signature: String::new(),
            content_signature: String::new(),
        },
        capabilities: Vec::new(),
        skills: vec![PackageSkillDeclaration {
            id: SKILL_ID.into(),
            path: "skill/SKILL.md".into(),
        }],
        hooks: vec![PackageHookDeclaration {
            id: "before-turn".into(),
            event: "before-turn".into(),
            executable: "hooks/before-turn.mjs".into(),
            arguments: Vec::new(),
            review_digest: hook_digest,
        }],
        settings: Vec::new(),
        apps: Vec::new(),
        mcp_servers: Vec::new(),
        inventory,
        package_digest: package_digest.clone(),
    };
    manifest.trust.origin_signature = BASE64.encode(
        origin_key
            .sign(&canonical_origin_signing_bytes(&manifest))
            .to_bytes(),
    );
    manifest.trust.content_signature = BASE64.encode(
        content_key
            .sign(&canonical_content_signing_bytes(&manifest))
            .to_bytes(),
    );
    fs::write(
        package_root.join("c4os-package.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
    (manifest, package_digest)
}

fn sign_catalog(catalog: &mut CatalogDocument, origin_key: &SigningKey) {
    catalog.signature.clear();
    catalog.signature = BASE64.encode(
        origin_key
            .sign(&canonical_catalog_signing_bytes(catalog))
            .to_bytes(),
    );
}
