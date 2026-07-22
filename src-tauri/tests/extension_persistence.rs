use std::{fs, path::Path, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use c4os_lib::core::database::{
    DatabaseActor, DatabaseDescriptor, DatabaseError, ExtensionEventRecord,
    ExtensionStateDocumentRecord, LifecycleState, ProjectPathState, ProjectRecord, SnapshotQuery,
    WorkspaceRecord,
};
use c4os_lib::{
    core::{
        services::save_workspace_with_database,
        workspace::{
            ArchiveLimits, C4osHomeLayout, OpenWorkspaceOutcome, WorkspaceLockOwner, WriterAccess,
            acquire_workspace_writer_lock, create_untitled_working_copy, open_workspace_archive,
            validate_archive,
        },
    },
    extension::{
        ExtensionPackageKind, ExtensionSourceKind,
        package::{
            CatalogDocument, CatalogMarketplace, CatalogRelease, PackageCompatibility,
            PackageHookDeclaration, PackageManifest, PackageMetadata, PackageTrust,
            canonical_catalog_signing_bytes, canonical_content_signing_bytes,
            canonical_origin_signing_bytes, digest_inventory, inventory_for_root,
            public_key_fingerprint,
        },
        service::{
            ExtensionHookReviewInput, ExtensionPackageInput, ExtensionService,
            MarketplaceSourceInput,
        },
        skill_sources::SkillSourceRoot,
    },
};
use ed25519_dalek::{Signer, SigningKey};
use tempfile::TempDir;
use uuid::Uuid;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE_ID: &str = "persistence-plugin";
const HOOK_ARGUMENT: &str = "--verified-package-contract";
const NOW_SECONDS: i64 = 1_721_400_000;

fn state(generation: u64, event_id: u64) -> ExtensionStateDocumentRecord {
    ExtensionStateDocumentRecord {
        generation,
        canonical_document: format!(
            r#"{{"schemaVersion":1,"generation":{generation},"lastEventId":{event_id}}}"#
        ),
        updated_at_ms: 1_721_400_000_000 + generation,
    }
}

fn event(event_id: u64, generation: u64) -> ExtensionEventRecord {
    let canonical_document = format!(
        r#"{{"schemaVersion":1,"eventId":{event_id},"generation":{generation},"operationId":"operation-{event_id}","packageId":"plugin.example","eventKind":"enabled","selectorDigest":"sha256:{}","result":"succeeded","occurredAtMs":{}}}"#,
        "a".repeat(64),
        1_721_400_000_000_u64 + generation,
    );
    ExtensionEventRecord {
        event_id,
        generation,
        operation_id: format!("operation-{event_id}"),
        package_id: Some("plugin.example".into()),
        event_kind: "enabled".into(),
        selector_digest: Some(format!("sha256:{}", "a".repeat(64))),
        result: "succeeded".into(),
        canonical_document,
        occurred_at_ms: 1_721_400_000_000 + generation,
    }
}

#[test]
fn extension_transition_atomically_persists_current_state_and_append_only_event() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, report) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    assert_eq!(report.current_version, 7);
    assert_eq!(database.extension_state_document().unwrap(), None);

    database
        .save_extension_transition(state(1, 1), event(1, 1), None)
        .expect("bootstrap transition");
    assert_eq!(
        database.extension_state_document().unwrap(),
        Some(state(1, 1))
    );

    let conflict = database
        .save_extension_transition(state(2, 2), event(2, 2), Some(99))
        .expect_err("stale transition must fail");
    assert!(matches!(conflict, DatabaseError::Conflict(_)));
    assert_eq!(
        database.extension_state_document().unwrap(),
        Some(state(1, 1))
    );
    assert_eq!(
        database
            .extension_event_page(None, SnapshotQuery::new(10).unwrap())
            .unwrap()
            .events,
        vec![event(1, 1)]
    );

    database
        .save_extension_transition(state(2, 2), event(2, 2), Some(1))
        .expect("second transition");
    let first_page = database
        .extension_event_page(None, SnapshotQuery::new(1).unwrap())
        .unwrap();
    assert_eq!(first_page.events, vec![event(2, 2)]);
    assert_eq!(first_page.next_before_event_id, Some(2));
    let second_page = database
        .extension_event_page(
            first_page.next_before_event_id,
            SnapshotQuery::new(1).unwrap(),
        )
        .unwrap();
    assert_eq!(second_page.events, vec![event(1, 1)]);
    assert_eq!(second_page.next_before_event_id, None);
}

#[test]
fn extension_event_sequence_failure_does_not_replace_current_state() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    database
        .save_extension_transition(state(1, 1), event(1, 1), None)
        .unwrap();

    let sequence_error = database
        .save_extension_transition(state(2, 3), event(3, 2), Some(1))
        .expect_err("event gap must fail");
    assert!(matches!(sequence_error, DatabaseError::Conflict(_)));
    assert_eq!(
        database.extension_state_document().unwrap(),
        Some(state(1, 1))
    );
}

#[test]
fn workspace_local_skill_survives_portable_archive_reopen_and_extension_restart() {
    let temporary = TempDir::new().expect("temporary root");
    let project = temporary.path().join("project");
    let working_root = temporary.path().join("working");
    fs::create_dir(&project).expect("project root");

    let writer_lock = match acquire_workspace_writer_lock(
        &temporary.path().join("workspace.lock"),
        lock_owner("portable-skill-save"),
    )
    .expect("workspace writer lock")
    {
        WriterAccess::Writable(lock) => lock,
        WriterAccess::ReadOnly { .. } => panic!("fixture lock must be writable"),
    };
    let manifest = create_untitled_working_copy(
        &writer_lock,
        &working_root,
        &project,
        "Portable Skill Project",
        APP_VERSION,
    )
    .expect("working copy");
    write_workspace_skill(&working_root, "portable-workspace-skill");

    let workspace_id = manifest.workspace_id.to_string();
    let project_id = manifest.projects[0].project_id.to_string();
    let (workspace_database, _) =
        DatabaseActor::start(DatabaseDescriptor::workspace(&working_root, &workspace_id))
            .expect("workspace database");
    workspace_database
        .create_workspace(WorkspaceRecord {
            workspace_id: workspace_id.clone(),
            display_name: "Portable Skill Workspace".into(),
            created_at: NOW_SECONDS,
            updated_at: NOW_SECONDS,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("workspace record");
    workspace_database
        .add_project(ProjectRecord {
            workspace_id: workspace_id.clone(),
            project_id,
            display_name: "Portable Skill Project".into(),
            current_path: project.to_string_lossy().into_owned(),
            last_known_path: project.to_string_lossy().into_owned(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("project record");

    let archive = temporary.path().join("portable-skill.c4os.zip");
    let saved = save_workspace_with_database(
        &workspace_database,
        &writer_lock,
        &working_root,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("save portable Workspace");
    let validated = validate_archive(&archive, APP_VERSION, ArchiveLimits::default())
        .expect("validate portable Workspace");
    assert_eq!(validated.manifest.workspace_id, saved.manifest.workspace_id);
    assert!(
        validated
            .manifest
            .files
            .iter()
            .any(|file| { file.path == "skills/portable-workspace-skill/SKILL.md" })
    );

    let opened = open_workspace_archive(
        &C4osHomeLayout::new(temporary.path().join("reopened-home")),
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("portable-skill-open"),
        |_, manifest, _| Ok(manifest.clone()),
    )
    .expect("reopen portable Workspace");
    let OpenWorkspaceOutcome::Writable(opened) = opened else {
        panic!("fixture archive must reopen writable");
    };
    assert!(
        opened
            .working_root
            .join("skills/portable-workspace-skill/SKILL.md")
            .is_file()
    );

    let extension_home = temporary.path().join("extension-home");
    let (extension_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(&extension_home)).expect("extension database");
    let extension_database = Arc::new(extension_database);
    let mut service =
        ExtensionService::restore(Arc::clone(&extension_database), &extension_home, 10)
            .expect("extension service");
    let source_root = SkillSourceRoot {
        source_kind: ExtensionSourceKind::WorkspaceLocal,
        source_id: opened.manifest.workspace_id.to_string(),
        root: opened.working_root.join("skills"),
        trusted: true,
    };
    let discovered = service
        .synchronize_skill_source_roots(vec![source_root.clone()], 20)
        .expect("discover reopened Workspace Skill");
    assert_workspace_skill(&mut service, &discovered, &source_root.source_id);

    drop(service);
    let mut restarted =
        ExtensionService::restore(Arc::clone(&extension_database), &extension_home, 30)
            .expect("restart extension service");
    let rediscovered = restarted
        .synchronize_skill_source_roots(vec![source_root.clone()], 40)
        .expect("rebind reopened Workspace after restart");
    assert_workspace_skill(&mut restarted, &rediscovered, &source_root.source_id);
}

#[test]
fn restore_reconciles_legacy_hook_arguments_and_persists_review_invalidation() {
    let temporary = TempDir::new().expect("temporary root");
    let home = temporary.path().join("home");
    let marketplace = temporary.path().join("marketplace");
    fs::create_dir_all(&home).expect("extension home");
    write_signed_hook_marketplace(&marketplace);

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).expect("app database");
    let database = Arc::new(database);
    let mut service =
        ExtensionService::restore(Arc::clone(&database), &home, 10).expect("extension service");
    service
        .add_marketplace(marketplace_input(&marketplace), 20)
        .expect("marketplace");
    service
        .install_disabled(package_input(2), 30)
        .expect("install disabled");
    service.enable(package_input(3), 40).expect("enable");
    let reviewed = service
        .review_hook(
            ExtensionHookReviewInput {
                expected_generation: 4,
                package_id: PACKAGE_ID.into(),
                hook_id: "before-turn".into(),
            },
            50,
        )
        .expect("review hook");
    assert!(reviewed.plugins[0].hooks[0].reviewed);
    assert_eq!(
        reviewed.plugins[0].hooks[0].arguments,
        vec![HOOK_ARGUMENT.to_owned()]
    );
    drop(service);

    let legacy_generation = persist_legacy_hook_without_arguments(&database, 60);
    let restored = ExtensionService::restore(Arc::clone(&database), &home, 70)
        .expect("restore and reconcile legacy hook");
    let migrated = restored.snapshot();
    assert!(migrated.generation > legacy_generation);
    assert_eq!(
        migrated.plugins[0].hooks[0].arguments,
        vec![HOOK_ARGUMENT.to_owned()]
    );
    assert!(!migrated.plugins[0].hooks[0].reviewed);
    assert!(!migrated.plugins[0].has_reviewed_hooks);
    assert!(restored.prepared_hooks("before-turn").unwrap().is_empty());
    let migrated_generation = migrated.generation;
    drop(restored);

    let restarted =
        ExtensionService::restore(database, &home, 80).expect("restart after hook migration");
    let durable = restarted.snapshot();
    assert_eq!(durable.generation, migrated_generation);
    assert_eq!(
        durable.plugins[0].hooks[0].arguments,
        vec![HOOK_ARGUMENT.to_owned()]
    );
    assert!(!durable.plugins[0].hooks[0].reviewed);
    assert!(!durable.plugins[0].has_reviewed_hooks);
    assert!(restarted.prepared_hooks("before-turn").unwrap().is_empty());
}

fn lock_owner(label: &str) -> WorkspaceLockOwner {
    WorkspaceLockOwner {
        process_id: std::process::id(),
        app_instance_id: Uuid::new_v4(),
        acquired_unix_ms: u64::try_from(NOW_SECONDS).unwrap() * 1_000,
        label: label.into(),
    }
}

fn write_workspace_skill(working_root: &Path, skill_id: &str) {
    let root = working_root.join("skills").join(skill_id);
    fs::create_dir_all(&root).expect("Workspace Skill directory");
    fs::write(
        root.join("SKILL.md"),
        format!(
            "---\nname: {skill_id}\ndescription: Portable Workspace-local Skill.\n---\nLoad after archive reopen.\n"
        ),
    )
    .expect("Workspace Skill");
}

fn assert_workspace_skill(
    service: &mut ExtensionService,
    snapshot: &c4os_lib::extension::ExtensionServiceSnapshot,
    source_id: &str,
) {
    let skill = snapshot
        .skills
        .iter()
        .find(|skill| skill.identity.skill_id == "portable-workspace-skill")
        .expect("portable Workspace Skill snapshot");
    assert_eq!(
        skill.identity.source_kind,
        ExtensionSourceKind::WorkspaceLocal
    );
    assert_eq!(skill.identity.source_id, source_id);
    assert!(skill.valid);
    assert!(skill.enabled);
    assert!(skill.eligible);
    assert!(skill.active);
    let loaded = service
        .load_skill_instructions(&skill.identity.stable_id())
        .expect("progressive Workspace Skill load");
    assert!(loaded.instructions.contains("archive reopen"));
}

fn package_input(expected_generation: u64) -> ExtensionPackageInput {
    ExtensionPackageInput {
        expected_generation,
        package_id: PACKAGE_ID.into(),
    }
}

fn marketplace_input(root: &Path) -> MarketplaceSourceInput {
    let origin_key = SigningKey::from_bytes(&[17_u8; 32]);
    MarketplaceSourceInput {
        source: root.display().to_string(),
        git_ref: None,
        sparse_paths: Vec::new(),
        trusted_origin: "local:c4os-persistence-tests".into(),
        signing_key_id: "persistence-origin".into(),
        public_key_sha256: public_key_fingerprint(
            &BASE64.encode(origin_key.verifying_key().as_bytes()),
        )
        .expect("origin fingerprint"),
    }
}

fn write_signed_hook_marketplace(root: &Path) {
    let package_root = root.join("persistence-v1");
    fs::create_dir_all(package_root.join("hooks")).expect("hook package");
    fs::write(
        package_root.join("hooks/before-turn.mjs"),
        "process.stdout.write(JSON.stringify({protocolVersion:1,proposals:[]}));\n",
    )
    .expect("hook executable");

    let origin_key = SigningKey::from_bytes(&[17_u8; 32]);
    let content_key = SigningKey::from_bytes(&[19_u8; 32]);
    let inventory = inventory_for_root(&package_root).expect("package inventory");
    let package_digest = digest_inventory(&inventory);
    let hook_digest = inventory
        .iter()
        .find(|entry| entry.path == "hooks/before-turn.mjs")
        .expect("hook inventory")
        .sha256
        .clone();
    let mut manifest = PackageManifest {
        schema_version: 1,
        package: PackageMetadata {
            id: PACKAGE_ID.into(),
            name: "Persistence Plugin".into(),
            summary: "Legacy hook persistence fixture".into(),
            publisher: "C4OS Tests".into(),
            version: "0.1.0".into(),
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
            origin: "local:c4os-persistence-tests".into(),
            origin_key_id: "persistence-origin".into(),
            delegated_content_key_id: "persistence-content".into(),
            delegated_content_public_key: BASE64.encode(content_key.verifying_key().as_bytes()),
            origin_signature: String::new(),
            content_signature: String::new(),
        },
        capabilities: vec!["context.annotation".into()],
        skills: Vec::new(),
        hooks: vec![PackageHookDeclaration {
            id: "before-turn".into(),
            event: "before-turn".into(),
            executable: "hooks/before-turn.mjs".into(),
            arguments: vec![HOOK_ARGUMENT.into()],
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
        toml::to_string(&manifest).expect("package manifest"),
    )
    .expect("package manifest file");

    let mut catalog = CatalogDocument {
        schema_version: 1,
        marketplace: CatalogMarketplace {
            id: "persistence-tests".into(),
            label: "Persistence tests".into(),
            origin: manifest.trust.origin.clone(),
            signing_key_id: manifest.trust.origin_key_id.clone(),
        },
        releases: vec![CatalogRelease {
            package_id: PACKAGE_ID.into(),
            version: "0.1.0".into(),
            manifest_path: "persistence-v1/c4os-package.toml".into(),
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
        toml::to_string(&catalog).expect("catalog"),
    )
    .expect("catalog file");
    fs::write(
        root.join("c4os-marketplace-origin.toml"),
        format!(
            "schemaVersion = 1\norigin = {:?}\nkeyId = {:?}\npublicKey = {:?}\n",
            catalog.marketplace.origin,
            catalog.marketplace.signing_key_id,
            BASE64.encode(origin_key.verifying_key().as_bytes()),
        ),
    )
    .expect("marketplace authority");
}

fn persist_legacy_hook_without_arguments(database: &DatabaseActor, now_ms: u64) -> u64 {
    let persisted = database
        .extension_state_document()
        .expect("read extension state")
        .expect("extension state exists");
    let mut document: serde_json::Value =
        serde_json::from_str(&persisted.canonical_document).expect("extension document");
    let hook = document
        .pointer_mut("/snapshot/plugins/0/hooks/0")
        .and_then(serde_json::Value::as_object_mut)
        .expect("persisted hook");
    let arguments = hook.remove("arguments").expect("current hook arguments");
    assert_eq!(arguments, serde_json::json!([HOOK_ARGUMENT]));

    let generation = persisted.generation + 1;
    let event_id = document
        .pointer("/snapshot/lastEventId")
        .and_then(serde_json::Value::as_u64)
        .expect("last event id")
        + 1;
    document["snapshot"]["generation"] = serde_json::json!(generation);
    document["snapshot"]["lastEventId"] = serde_json::json!(event_id);
    database
        .save_extension_transition(
            ExtensionStateDocumentRecord {
                generation,
                canonical_document: serde_json::to_string(&document)
                    .expect("legacy extension document"),
                updated_at_ms: now_ms,
            },
            event(event_id, generation),
            Some(persisted.generation),
        )
        .expect("persist legacy extension state");
    generation
}
