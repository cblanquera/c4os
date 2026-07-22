use std::{fs, path::Path, process::Command, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use c4os_lib::{
    core::database::{DatabaseActor, DatabaseDescriptor, SnapshotQuery},
    extension::{
        EXTENSION_HOOK_PROTOCOL_VERSION, ExtensionLifecycle, ExtensionPackageKind,
        ExtensionSourceKind, ExtensionTrustState,
        hook::{HookEventEnvelope, HookSupervisor, HookSupervisorPolicy},
        package::{
            CatalogDocument, CatalogMarketplace, CatalogRelease, PackageAppDeclaration,
            PackageCompatibility, PackageHookDeclaration, PackageManifest,
            PackageMcpServerDeclaration, PackageMcpTransport, PackageMetadata,
            PackageSettingDeclaration, PackageSettingKind, PackageSkillDeclaration, PackageTrust,
            canonical_catalog_signing_bytes, canonical_content_signing_bytes,
            canonical_origin_signing_bytes, digest_inventory, inventory_for_root,
            public_key_fingerprint,
        },
        service::{
            ExtensionHookExecutionRecord, ExtensionHookReviewInput, ExtensionKeyRevocationInput,
            ExtensionPackageInput, ExtensionPackageMutation, ExtensionService, ExtensionSkillInput,
            MarketplaceSourceInput,
        },
        store::ExtensionStore,
    },
};
use ed25519_dalek::{Signer, SigningKey};
use tempfile::TempDir;

const PACKAGE_ID: &str = "sample-plugin";
const SKILL_ID: &str = "sample-skill";

#[test]
fn immutable_skill_customization_creates_a_selected_user_owned_copy() {
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
    service.install_disabled(package_input(2), 30).unwrap();
    let enabled = service.enable(package_input(3), 40).unwrap();
    let plugin_identity = enabled.skills[0].identity.stable_id();

    let customized = service
        .customize_skill(
            ExtensionSkillInput {
                expected_generation: enabled.generation,
                skill_identity: plugin_identity,
            },
            50,
        )
        .unwrap();
    assert_eq!(customized.generation, enabled.generation + 1);
    let selected = customized.selected_skill.clone().unwrap();
    assert_eq!(selected.source_kind, ExtensionSourceKind::UserGlobal);
    assert_eq!(selected.source_id, "user-global");
    assert_eq!(selected.skill_id, SKILL_ID);
    assert!(customized.skills.iter().any(|skill| {
        skill.identity == selected && skill.active && skill.enabled && !skill.is_installed
    }));
    assert!(customized.skills.iter().any(|skill| {
        skill.identity.source_kind == ExtensionSourceKind::PluginProvided
            && skill.identity.skill_id == SKILL_ID
            && skill.shadowed
    }));
    let loaded = service
        .load_skill_instructions(&selected.stable_id())
        .unwrap();
    assert!(loaded.instructions.contains("signed Skill golden path"));

    let user_entries = fs::read_dir(home.join("skills/user"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(user_entries.len(), 1);
    assert!(user_entries[0].path().join("SKILL.md").is_file());

    let before_invalid_request = service.snapshot();
    assert!(matches!(
        service.customize_skill(
            ExtensionSkillInput {
                expected_generation: before_invalid_request.generation,
                skill_identity: selected.stable_id(),
            },
            60,
        ),
        Err(c4os_lib::extension::ExtensionError::InvalidState)
    ));
    assert_eq!(service.snapshot(), before_invalid_request);

    drop(service);
    let mut restored = ExtensionService::restore(database, &home, 70).unwrap();
    assert_eq!(restored.snapshot().selected_skill.as_ref(), Some(&selected));
    assert!(
        restored
            .load_skill_instructions(&selected.stable_id())
            .unwrap()
            .instructions
            .contains("signed Skill golden path")
    );
}

#[test]
fn reviewed_hook_execution_is_a_visible_durable_lifecycle() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    let enabled = service.enable(package_input(3), 40).unwrap();
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
    let prepared = service.prepared_hooks("before-turn").unwrap();
    assert_eq!(prepared.len(), 1);
    assert_eq!(prepared[0].activation.generation, executing.generation);
    assert_eq!(
        prepared[0].contract.arguments,
        executing.plugins[0].hooks[0].arguments
    );

    let completed = service
        .record_hook_execution_batch(
            executing.generation,
            &[ExtensionHookExecutionRecord {
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
                succeeded: true,
                detail: "completed in the reviewed sandbox".into(),
            }],
            70,
        )
        .unwrap();
    assert_eq!(completed.plugins[0].lifecycle, ExtensionLifecycle::Enabled);
    assert_eq!(
        completed.plugins[0].hooks[0].last_result.as_deref(),
        Some("completed in the reviewed sandbox")
    );
}

#[test]
fn restart_recovers_an_interrupted_hook_execution_without_losing_authority() {
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
    service.install_disabled(package_input(2), 30).unwrap();
    let enabled = service.enable(package_input(3), 40).unwrap();
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
    drop(service);

    let mut restored = ExtensionService::restore(database, &home, 70).unwrap();
    assert_eq!(restored.snapshot().generation, executing.generation + 1);
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::Enabled
    );
    assert_eq!(
        restored.snapshot().plugins[0].failure_code.as_deref(),
        Some("worker-restart-recovered")
    );
    assert_eq!(restored.active_turn_skills().unwrap().len(), 1);
    assert_eq!(restored.prepared_hooks("before-turn").unwrap().len(), 1);
}

#[test]
fn signed_plugin_skill_golden_path_is_durable_disabled_first_and_revocable() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let database = Arc::new(database);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 10).unwrap();

    let available = service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    assert_eq!(available.generation, 2);
    assert_eq!(
        available.plugins[0].lifecycle,
        ExtensionLifecycle::Available
    );

    let installed = service.install_disabled(package_input(2), 30).unwrap();
    assert_eq!(installed.generation, 3);
    assert_eq!(
        installed.plugins[0].lifecycle,
        ExtensionLifecycle::InstalledDisabled
    );
    assert_eq!(installed.skills.len(), 1);
    assert!(!installed.skills[0].active);

    let enabled = service.enable(package_input(3), 40).unwrap();
    assert_eq!(enabled.generation, 4);
    assert_eq!(enabled.plugins[0].lifecycle, ExtensionLifecycle::Enabled);
    assert!(enabled.skills[0].active);
    assert_eq!(
        enabled.skills[0].identity.source_kind,
        ExtensionSourceKind::PluginProvided
    );

    drop(service);
    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 50).unwrap();
    assert_eq!(service.snapshot().generation, 4);
    assert!(service.snapshot().skills[0].active);
    let turn_skills = service.active_turn_skills().unwrap();
    assert_eq!(turn_skills.len(), 1);
    assert_eq!(turn_skills[0].identity.skill_id, SKILL_ID);
    assert!(
        turn_skills[0]
            .instructions
            .contains("signed Skill golden path")
    );

    let reviewed = service
        .review_hook(
            ExtensionHookReviewInput {
                expected_generation: 4,
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
            },
            60,
        )
        .unwrap();
    assert_eq!(reviewed.generation, 5);
    assert!(reviewed.plugins[0].hooks[0].reviewed);
    let prepared = service.prepared_hooks("before-turn").unwrap();
    assert_eq!(prepared.len(), 1);
    assert_eq!(prepared[0].activation.generation, 5);
    assert_eq!(prepared[0].activation.package_id, PACKAGE_ID);
    assert_eq!(prepared[0].contract.hook_id, "before-turn");

    let disabled = service.disable(package_input(5), 70).unwrap();
    assert_eq!(disabled.generation, 6);
    assert!(!disabled.skills[0].active);
    assert!(service.active_turn_skills().unwrap().is_empty());

    let enabled_again = service.enable(package_input(6), 80).unwrap();
    assert_eq!(enabled_again.generation, 7);
    assert!(enabled_again.skills[0].active);

    let revoked = service
        .revoke(package_input(7), "publisher key revoked", 90)
        .unwrap();
    assert_eq!(revoked.generation, 8);
    assert_eq!(revoked.plugins[0].lifecycle, ExtensionLifecycle::Revoked);
    assert_eq!(revoked.plugins[0].trust, ExtensionTrustState::Revoked);
    assert!(revoked.skills.is_empty());

    let uninstalled = service.uninstall(package_input(8), 100).unwrap();
    assert_eq!(uninstalled.generation, 9);
    assert_eq!(
        uninstalled.plugins[0].lifecycle,
        ExtensionLifecycle::Available
    );
    assert_eq!(uninstalled.plugins[0].trust, ExtensionTrustState::Revoked);
    assert!(uninstalled.skills.is_empty());

    let events = database
        .extension_event_page(None, SnapshotQuery::new(32).unwrap())
        .unwrap();
    assert_eq!(events.events.len(), 9);
    assert_eq!(events.events.first().unwrap().generation, 9);
    assert_eq!(
        events.events.first().unwrap().event_kind,
        "packageUninstalledWithoutExecution"
    );
}

#[test]
fn selector_drift_fails_closed_immediately_and_restart_repairs_from_durable_state() {
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
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();

    let selector_path = home.join("extensions/selectors/sample-plugin.json");
    let mut selector: serde_json::Value =
        serde_json::from_slice(&fs::read(&selector_path).unwrap()).unwrap();
    selector["generation"] = serde_json::json!(3);
    selector["activeDigest"] = serde_json::Value::Null;
    fs::write(&selector_path, serde_json::to_vec(&selector).unwrap()).unwrap();

    assert!(service.active_turn_skills().is_err());
    assert!(service.prepared_hooks("before-turn").unwrap().is_empty());
    drop(service);

    let mut service = ExtensionService::restore(Arc::clone(&database), &home, 50).unwrap();
    let snapshot = service.snapshot();
    assert_eq!(snapshot.generation, 5);
    assert_eq!(snapshot.plugins[0].lifecycle, ExtensionLifecycle::Enabled);
    assert_eq!(
        snapshot.plugins[0].failure_code.as_deref(),
        Some("selector-repaired-from-durable-state")
    );
    assert!(snapshot.skills[0].active);
    assert_eq!(service.active_turn_skills().unwrap().len(), 1);
    let events = database
        .extension_event_page(None, SnapshotQuery::new(8).unwrap())
        .unwrap();
    assert_eq!(events.events[0].event_kind, "selectorRestartRecovery");
}

#[test]
fn transactional_update_switches_once_and_rollback_returns_disabled_last_known_good() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();

    write_signed_update(&marketplace);
    let available = service.refresh_catalogs(4, 50).unwrap();
    assert_eq!(available.generation, 5);
    assert_eq!(
        available.plugins[0].available_version.as_deref(),
        Some("0.2.0")
    );
    let staged = service.stage_update(package_input(5), 60).unwrap();
    assert_eq!(
        staged.plugins[0].lifecycle,
        ExtensionLifecycle::UpdateStaged
    );
    assert_eq!(staged.plugins[0].version, "0.1.0");
    assert_eq!(staged.plugins[0].staged_version.as_deref(), Some("0.2.0"));
    assert!(staged.skills[0].active);

    let activated = service
        .activate_staged_update(package_input(6), 70)
        .unwrap();
    assert_eq!(activated.generation, 7);
    assert_eq!(activated.plugins[0].version, "0.2.0");
    assert_eq!(
        activated.plugins[0].last_known_good_version.as_deref(),
        Some("0.1.0")
    );
    assert!(
        service.active_turn_skills().unwrap()[0]
            .instructions
            .contains("updated signed Skill")
    );

    let rolled_back = service.rollback(package_input(7), 80).unwrap();
    assert_eq!(rolled_back.generation, 8);
    assert_eq!(
        rolled_back.plugins[0].lifecycle,
        ExtensionLifecycle::RolledBack
    );
    assert_eq!(rolled_back.plugins[0].version, "0.1.0");
    assert!(!rolled_back.skills[0].active);
    let enabled = service.enable(package_input(8), 90).unwrap();
    assert_eq!(enabled.generation, 9);
    assert_eq!(enabled.plugins[0].version, "0.1.0");
    assert!(
        service.active_turn_skills().unwrap()[0]
            .instructions
            .contains("signed Skill golden path")
    );
}

#[cfg(unix)]
#[test]
fn rollback_rejects_corrupted_last_known_good_bytes_before_lifecycle_change() {
    use std::os::unix::fs::PermissionsExt;

    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();

    write_signed_update(&marketplace);
    service.refresh_catalogs(4, 50).unwrap();
    service.stage_update(package_input(5), 60).unwrap();
    let updated = service
        .activate_staged_update(package_input(6), 70)
        .unwrap();
    let prior_digest = updated.plugins[0].last_known_good_digest.clone().unwrap();
    let manifest = ExtensionStore::new(home.join("extensions"))
        .unwrap()
        .root()
        .join("packages/sha256")
        .join(&prior_digest["sha256:".len()..])
        .join("c4os-package.toml");
    fs::set_permissions(&manifest, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&manifest, "corrupted retained package").unwrap();

    let input = package_input(updated.generation);
    assert!(
        service
            .preflight_package_mutation(&input, ExtensionPackageMutation::Rollback)
            .is_err()
    );
    assert!(service.rollback(input, 80).is_err());
    assert_eq!(service.snapshot(), updated);
}

#[test]
fn update_and_rollback_require_new_hook_review_for_the_exact_signed_contract() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();
    service
        .review_hook(
            ExtensionHookReviewInput {
                expected_generation: 4,
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
            },
            50,
        )
        .unwrap();
    assert_eq!(service.prepared_hooks("before-turn").unwrap().len(), 1);

    write_signed_update(&marketplace);
    service.refresh_catalogs(5, 60).unwrap();
    service.stage_update(package_input(6), 70).unwrap();
    let updated = service
        .activate_staged_update(package_input(7), 80)
        .unwrap();
    assert!(!updated.plugins[0].hooks[0].reviewed);
    assert!(service.prepared_hooks("before-turn").unwrap().is_empty());

    service
        .review_hook(
            ExtensionHookReviewInput {
                expected_generation: 8,
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
            },
            90,
        )
        .unwrap();
    let rolled_back = service.rollback(package_input(9), 100).unwrap();
    assert!(!rolled_back.plugins[0].hooks[0].reviewed);
    assert!(service.prepared_hooks("before-turn").unwrap().is_empty());
}

#[test]
fn package_revocation_survives_staged_update_uninstall_and_catalog_refresh() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();

    write_signed_update(&marketplace);
    service.refresh_catalogs(4, 50).unwrap();
    service.stage_update(package_input(5), 60).unwrap();
    let revoked = service
        .revoke(package_input(6), "package authority revoked", 70)
        .unwrap();
    assert_eq!(revoked.plugins[0].trust, ExtensionTrustState::Revoked);

    let available = service.uninstall(package_input(7), 80).unwrap();
    assert_eq!(available.plugins[0].version, "0.2.0");
    assert_eq!(
        available.plugins[0].lifecycle,
        ExtensionLifecycle::Available
    );
    assert_eq!(available.plugins[0].trust, ExtensionTrustState::Revoked);
    assert!(matches!(
        service.install_disabled(package_input(8), 90),
        Err(c4os_lib::extension::ExtensionError::Revoked)
    ));

    let refreshed = service.refresh_catalogs(8, 100).unwrap();
    assert_eq!(refreshed.plugins[0].trust, ExtensionTrustState::Revoked);
    assert_eq!(
        refreshed.plugins[0].revocation_reason.as_deref(),
        Some("package authority revoked")
    );
}

#[test]
fn signing_key_revocation_disables_every_bound_package_and_survives_restart() {
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
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();

    let revoked = service
        .revoke_key(
            ExtensionKeyRevocationInput {
                expected_generation: 4,
                key_id: "content-test".into(),
                reason: "delegated publisher key revoked".into(),
            },
            50,
        )
        .unwrap();
    assert_eq!(revoked.generation, 5);
    assert_eq!(revoked.plugins[0].trust, ExtensionTrustState::Revoked);
    assert_eq!(revoked.plugins[0].lifecycle, ExtensionLifecycle::Revoked);
    assert!(revoked.skills.is_empty());
    assert!(service.prepared_hooks("before-turn").unwrap().is_empty());

    drop(service);
    let service = ExtensionService::restore(database, &home, 60).unwrap();
    assert_eq!(
        service.snapshot().plugins[0].trust,
        ExtensionTrustState::Revoked
    );
    assert!(service.snapshot().skills.is_empty());
}

#[test]
fn standalone_signed_skill_package_installs_without_plugin_authority() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    resign_as_standalone_skill(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();

    let available = service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    assert_eq!(
        available.plugins[0].package_kind,
        ExtensionPackageKind::Skill
    );
    assert!(available.plugins[0].hooks.is_empty());
    assert!(available.plugins[0].settings.is_empty());
    assert!(available.plugins[0].apps.is_empty());
    assert!(available.plugins[0].mcp_servers.is_empty());
    service.install_disabled(package_input(2), 30).unwrap();
    let enabled = service.enable(package_input(3), 40).unwrap();
    assert_eq!(enabled.plugins[0].lifecycle, ExtensionLifecycle::Enabled);
    assert_eq!(enabled.skills.len(), 1);
    assert!(enabled.skills[0].active);
    assert_eq!(service.active_turn_skills().unwrap().len(), 1);
    assert!(service.prepared_hooks("before-turn").unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn uninstall_cleanup_failure_keeps_durable_non_active_state_recoverable() {
    use std::os::unix::fs::PermissionsExt;

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
    let digest = installed.plugins[0].digest.clone();
    let package_root = home.join("extensions/packages/sha256").join(&digest[7..]);
    fs::set_permissions(&package_root, fs::Permissions::from_mode(0o700)).unwrap();
    let manifest_path = package_root.join("c4os-package.toml");
    fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&manifest_path, "tampered = true\n").unwrap();

    assert!(service.uninstall(package_input(3), 40).is_err());
    assert_eq!(service.snapshot().generation, 4);
    assert_eq!(
        service.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::Available
    );
    drop(service);

    let restored = ExtensionService::restore(database, &home, 50).unwrap();
    assert_eq!(restored.snapshot().generation, 4);
    assert_eq!(
        restored.snapshot().plugins[0].lifecycle,
        ExtensionLifecycle::Available
    );
    assert!(restored.snapshot().skills.is_empty());
}

#[test]
fn hostile_package_mutations_fail_without_publishing_catalog_state() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    let input = || marketplace_input(&marketplace);

    fs::write(
        marketplace.join("sample-v1/skill/SKILL.md"),
        "---\nname: sample-skill\ndescription: Tampered\n---\nChanged after signing.\n",
    )
    .unwrap();
    assert!(service.add_marketplace(input(), 20).is_err());
    assert_eq!(service.snapshot().generation, 1);

    write_signed_marketplace(&marketplace);
    let ambient = marketplace.join("sample-v1/native.dylib");
    fs::write(
        &ambient,
        b"not executable, but still forbidden package surface",
    )
    .unwrap();
    assert!(service.add_marketplace(input(), 30).is_err());
    fs::remove_file(ambient).unwrap();
    assert_eq!(service.snapshot().generation, 1);

    write_signed_marketplace(&marketplace);
    let catalog_path = marketplace.join("c4os-marketplace.toml");
    let mut catalog = fs::read_to_string(&catalog_path).unwrap();
    catalog.push_str("\nsurprise = true\n");
    fs::write(&catalog_path, catalog).unwrap();
    assert!(service.add_marketplace(input(), 40).is_err());
    assert_eq!(service.snapshot().generation, 1);

    write_signed_marketplace(&marketplace);
    let snapshot = service.add_marketplace(input(), 50).unwrap();
    assert_eq!(snapshot.generation, 2);
    assert_eq!(snapshot.marketplaces.len(), 1);
}

#[test]
fn git_marketplace_refresh_publishes_a_new_immutable_resolved_commit() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    git(&marketplace, &["init", "--quiet"]);
    git(&marketplace, &["add", "."]);
    git(
        &marketplace,
        &[
            "-c",
            "user.name=C4OS Tests",
            "-c",
            "user.email=c4os@example.test",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--quiet",
            "-m",
            "v1",
        ],
    );
    let first_commit = git_output(&marketplace, &["rev-parse", "HEAD"]);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    let first = service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    assert_eq!(
        first.marketplaces[0].resolved_commit.as_deref(),
        Some(first_commit.as_str())
    );

    write_signed_update(&marketplace);
    git(&marketplace, &["add", "."]);
    git(
        &marketplace,
        &[
            "-c",
            "user.name=C4OS Tests",
            "-c",
            "user.email=c4os@example.test",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--quiet",
            "-m",
            "v2",
        ],
    );
    let second_commit = git_output(&marketplace, &["rev-parse", "HEAD"]);
    assert_ne!(first_commit, second_commit);
    let refreshed = service.refresh_catalogs(2, 30).unwrap();
    assert_eq!(
        refreshed.marketplaces[0].resolved_commit.as_deref(),
        Some(second_commit.as_str())
    );
    assert_eq!(refreshed.plugins[0].version, "0.2.0");
    assert_eq!(refreshed.plugins[0].available_version, None);
}

#[test]
fn invalid_user_skill_remains_visible_but_ineligible() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let broken = home.join("skills/user/broken-skill");
    fs::create_dir_all(&broken).unwrap();
    fs::write(
        broken.join("SKILL.md"),
        "---\nname: ../invalid\ndescription: Broken\n---\nDo not load.\n",
    )
    .unwrap();
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    let snapshot = service.snapshot();
    assert_eq!(snapshot.generation, 2);
    assert_eq!(snapshot.skills.len(), 1);
    assert_eq!(snapshot.skills[0].name, "broken-skill");
    assert!(!snapshot.skills[0].valid);
    assert!(!snapshot.skills[0].eligible);
    assert!(service.active_turn_skills().unwrap().is_empty());
}

#[test]
fn signed_package_with_invalid_skill_transitions_to_visible_failed_state() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    resign_with_invalid_skill(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    let failed = service.install_disabled(package_input(2), 30).unwrap();
    assert_eq!(failed.generation, 3);
    assert_eq!(failed.plugins[0].lifecycle, ExtensionLifecycle::Failed);
    assert_eq!(
        failed.plugins[0].failure_code.as_deref(),
        Some("package-content-invalid")
    );
    assert!(failed.skills.is_empty());
    let available = service.uninstall(package_input(3), 40).unwrap();
    assert_eq!(
        available.plugins[0].lifecycle,
        ExtensionLifecycle::Available
    );
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires the project-owned pinned Node runtime asset"]
fn reviewed_hook_runs_in_the_native_sandbox_and_persists_bounded_outcome() {
    let temporary = TempDir::new().unwrap();
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).unwrap();
    write_signed_marketplace(&marketplace);
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).unwrap();
    let mut service = ExtensionService::restore(Arc::new(database), &home, 10).unwrap();
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .unwrap();
    service.install_disabled(package_input(2), 30).unwrap();
    service.enable(package_input(3), 40).unwrap();
    service
        .review_hook(
            ExtensionHookReviewInput {
                expected_generation: 4,
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
            },
            50,
        )
        .unwrap();
    let executing = service
        .begin_hook_execution(5, &[PACKAGE_ID.into()], 55)
        .unwrap();
    let prepared = service.prepared_hooks("before-turn").unwrap();
    assert_eq!(prepared.len(), 1);
    assert_eq!(prepared[0].activation.generation, executing.generation);

    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let runtime_root = repository_root.join("target/c4os-runtime-assets");
    let scratch_root = home.join("extensions/hook-native-test");
    fs::create_dir(&scratch_root).unwrap();
    let supervisor = HookSupervisor::new(HookSupervisorPolicy {
        trusted_runtime: runtime_root.join("node"),
        runtime_read_roots: vec![runtime_root],
        scratch_root,
    })
    .unwrap();
    let hook = &prepared[0];
    let result = supervisor
        .run(
            &hook.activation,
            &hook.contract,
            &hook.package_root,
            &HookEventEnvelope {
                protocol_version: EXTENSION_HOOK_PROTOCOL_VERSION,
                operation_id: "native-hook-test".into(),
                package_id: PACKAGE_ID.into(),
                package_digest: hook.activation.package_digest.clone(),
                event: "before-turn".into(),
                payload: serde_json::json!({"promptSha256": "sha256:bounded"}),
            },
        )
        .unwrap();
    assert!(result.proposals.is_empty());
    assert_eq!(supervisor.active_worker_count().unwrap(), 0);
    let snapshot = service
        .record_hook_execution_batch(
            executing.generation,
            &[ExtensionHookExecutionRecord {
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
                succeeded: true,
                detail: "completed with no effect proposals".into(),
            }],
            60,
        )
        .unwrap();
    assert_eq!(snapshot.generation, executing.generation + 1);
    assert_eq!(
        snapshot.plugins[0].hooks[0].last_result.as_deref(),
        Some("completed with no effect proposals")
    );
}

fn package_input(expected_generation: u64) -> ExtensionPackageInput {
    ExtensionPackageInput {
        expected_generation,
        package_id: PACKAGE_ID.into(),
    }
}

fn git(root: &Path, arguments: &[&str]) {
    let status = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin")
        .status()
        .unwrap();
    assert!(status.success(), "git command failed: {arguments:?}");
}

fn git_output(root: &Path, arguments: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(output.status.success(), "git command failed: {arguments:?}");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn marketplace_input(root: &Path) -> MarketplaceSourceInput {
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    MarketplaceSourceInput {
        source: root.display().to_string(),
        git_ref: None,
        sparse_paths: Vec::new(),
        trusted_origin: "local:c4os-test-marketplace".into(),
        signing_key_id: "origin-test".into(),
        public_key_sha256: public_key_fingerprint(
            &BASE64.encode(origin_key.verifying_key().as_bytes()),
        )
        .unwrap(),
    }
}

fn write_signed_marketplace(root: &Path) {
    let package_root = root.join("sample-v1");
    fs::create_dir_all(package_root.join("skill")).unwrap();
    fs::create_dir_all(package_root.join("hooks")).unwrap();
    fs::write(
        package_root.join("skill/SKILL.md"),
        "---\nname: sample-skill\ndescription: Signed sample Skill\n---\nUse the signed Skill golden path.\n",
    )
    .unwrap();
    fs::write(
        package_root.join("hooks/before-turn.mjs"),
        "process.stdout.write(JSON.stringify({protocolVersion:1,proposals:[]}));\n",
    )
    .unwrap();

    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let content_key = SigningKey::from_bytes(&[9u8; 32]);
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
            name: "Sample Plugin".into(),
            summary: "Signed package lifecycle fixture".into(),
            publisher: "C4OS Tests".into(),
            version: "0.1.0".into(),
            kind: ExtensionPackageKind::Plugin,
            website: Some("https://example.com/plugin".into()),
            terms: Some("https://example.com/terms".into()),
            privacy_policy: Some("https://example.com/privacy".into()),
        },
        compatibility: PackageCompatibility {
            c4os: "^0.1".into(),
            operating_systems: vec![std::env::consts::OS.into()],
            architectures: vec![std::env::consts::ARCH.into()],
        },
        trust: PackageTrust {
            origin: "local:c4os-test-marketplace".into(),
            origin_key_id: "origin-test".into(),
            delegated_content_key_id: "content-test".into(),
            delegated_content_public_key: BASE64.encode(content_key.verifying_key().as_bytes()),
            origin_signature: String::new(),
            content_signature: String::new(),
        },
        capabilities: vec!["context.annotation".into()],
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
        settings: vec![
            PackageSettingDeclaration {
                id: "workspace-mode".into(),
                label: "Workspace mode".into(),
                description: "Controls the host-owned declarative app projection.".into(),
                kind: PackageSettingKind::Select,
                required: true,
                choices: vec!["focused".into(), "expanded".into()],
            },
            PackageSettingDeclaration {
                id: "api-credential".into(),
                label: "API credential".into(),
                description: "Opaque credential reference; raw secret values are never packaged."
                    .into(),
                kind: PackageSettingKind::CredentialReference,
                required: false,
                choices: Vec::new(),
            },
        ],
        apps: vec![PackageAppDeclaration {
            id: "workspace-summary".into(),
            title: "Workspace summary".into(),
            summary: "Host-rendered declarative Plugin contribution.".into(),
            settings: vec!["workspace-mode".into()],
        }],
        mcp_servers: vec![PackageMcpServerDeclaration {
            id: "sample-mcp".into(),
            name: "Sample MCP metadata".into(),
            transport: PackageMcpTransport::Stdio,
            settings: vec!["api-credential".into()],
        }],
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

    let mut catalog = CatalogDocument {
        schema_version: 1,
        marketplace: CatalogMarketplace {
            id: "local-tests".into(),
            label: "Local signed tests".into(),
            origin: manifest.trust.origin.clone(),
            signing_key_id: manifest.trust.origin_key_id.clone(),
        },
        releases: vec![CatalogRelease {
            package_id: PACKAGE_ID.into(),
            version: "0.1.0".into(),
            manifest_path: "sample-v1/c4os-package.toml".into(),
            package_digest,
        }],
        signature: String::new(),
    };
    catalog.signature = BASE64.encode(
        origin_key
            .sign(&canonical_catalog_signing_bytes(&catalog))
            .to_bytes(),
    );
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
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let content_key = SigningKey::from_bytes(&[9u8; 32]);
    let package_root = root.join("sample-v2");
    fs::create_dir_all(package_root.join("skill")).unwrap();
    fs::create_dir_all(package_root.join("hooks")).unwrap();
    fs::write(
        package_root.join("skill/SKILL.md"),
        "---\nname: sample-skill\ndescription: Signed sample Skill\n---\nUse the updated signed Skill golden path.\n",
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
    let original: PackageManifest =
        toml::from_str(&fs::read_to_string(root.join("sample-v1/c4os-package.toml")).unwrap())
            .unwrap();
    let mut manifest = original;
    manifest.package.version = "0.2.0".into();
    manifest.inventory = inventory;
    manifest.package_digest = package_digest.clone();
    manifest.hooks[0].review_digest = hook_digest;
    manifest.hooks[0].arguments = vec!["--updated-contract".into()];
    manifest.trust.origin_signature.clear();
    manifest.trust.content_signature.clear();
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

    let mut catalog: CatalogDocument =
        toml::from_str(&fs::read_to_string(root.join("c4os-marketplace.toml")).unwrap()).unwrap();
    catalog.releases.push(CatalogRelease {
        package_id: PACKAGE_ID.into(),
        version: "0.2.0".into(),
        manifest_path: "sample-v2/c4os-package.toml".into(),
        package_digest,
    });
    catalog.signature.clear();
    catalog.signature = BASE64.encode(
        origin_key
            .sign(&canonical_catalog_signing_bytes(&catalog))
            .to_bytes(),
    );
    fs::write(
        root.join("c4os-marketplace.toml"),
        toml::to_string(&catalog).unwrap(),
    )
    .unwrap();
}

fn resign_with_invalid_skill(root: &Path) {
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let content_key = SigningKey::from_bytes(&[9u8; 32]);
    let package_root = root.join("sample-v1");
    fs::write(
        package_root.join("skill/SKILL.md"),
        "---\nname: ../invalid\ndescription: Signed but invalid\n---\nDo not load.\n",
    )
    .unwrap();
    let mut manifest: PackageManifest =
        toml::from_str(&fs::read_to_string(package_root.join("c4os-package.toml")).unwrap())
            .unwrap();
    manifest.inventory = inventory_for_root(&package_root).unwrap();
    manifest.package_digest = digest_inventory(&manifest.inventory);
    manifest.hooks[0].review_digest = manifest
        .inventory
        .iter()
        .find(|entry| entry.path == "hooks/before-turn.mjs")
        .unwrap()
        .sha256
        .clone();
    manifest.trust.origin_signature.clear();
    manifest.trust.content_signature.clear();
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
    let mut catalog: CatalogDocument =
        toml::from_str(&fs::read_to_string(root.join("c4os-marketplace.toml")).unwrap()).unwrap();
    catalog.releases[0].package_digest = manifest.package_digest;
    catalog.signature.clear();
    catalog.signature = BASE64.encode(
        origin_key
            .sign(&canonical_catalog_signing_bytes(&catalog))
            .to_bytes(),
    );
    fs::write(
        root.join("c4os-marketplace.toml"),
        toml::to_string(&catalog).unwrap(),
    )
    .unwrap();
}

fn resign_as_standalone_skill(root: &Path) {
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let content_key = SigningKey::from_bytes(&[9u8; 32]);
    let package_root = root.join("sample-v1");
    fs::remove_file(package_root.join("hooks/before-turn.mjs")).unwrap();
    let mut manifest: PackageManifest =
        toml::from_str(&fs::read_to_string(package_root.join("c4os-package.toml")).unwrap())
            .unwrap();
    manifest.package.kind = ExtensionPackageKind::Skill;
    manifest.package.name = "Sample standalone Skill".into();
    manifest.capabilities.clear();
    manifest.hooks.clear();
    manifest.settings.clear();
    manifest.apps.clear();
    manifest.mcp_servers.clear();
    manifest.inventory = inventory_for_root(&package_root).unwrap();
    manifest.package_digest = digest_inventory(&manifest.inventory);
    manifest.trust.origin_signature.clear();
    manifest.trust.content_signature.clear();
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
    let mut catalog: CatalogDocument =
        toml::from_str(&fs::read_to_string(root.join("c4os-marketplace.toml")).unwrap()).unwrap();
    catalog.releases[0].package_digest = manifest.package_digest;
    catalog.signature.clear();
    catalog.signature = BASE64.encode(
        origin_key
            .sign(&canonical_catalog_signing_bytes(&catalog))
            .to_bytes(),
    );
    fs::write(
        root.join("c4os-marketplace.toml"),
        toml::to_string(&catalog).unwrap(),
    )
    .unwrap();
}
