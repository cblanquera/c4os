//! Adds a signed Task 00011 Plugin/Skill fixture to an isolated acceptance home.
//!
//! Run this only after `task_00007_native_seed`; it preserves that workspace
//! and uses the production ExtensionService for every published transition.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use c4os_lib::{
    core::database::{DatabaseActor, DatabaseDescriptor},
    extension::{
        ExtensionPackageKind,
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
            ExtensionHookReviewInput, ExtensionPackageInput, ExtensionService,
            MarketplaceSourceInput,
        },
    },
};
use ed25519_dalek::{Signer, SigningKey};

const PACKAGE_ID: &str = "sample-plugin";
const SKILL_ID: &str = "sample-skill";

fn main() -> Result<(), Box<dyn Error>> {
    let home = acceptance_home_argument()?;
    let now_ms: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?;
    let marketplace = home.join("task-00011-marketplace");
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let user_skill = home.join("skills/user/sample-skill");
    fs::create_dir_all(&user_skill)?;
    fs::write(
        user_skill.join("SKILL.md"),
        "---\nname: sample-skill\ndescription: User-global collision fixture\n---\nUse the user-global source unless a Plugin source is explicitly selected.\n",
    )?;
    let broken_skill = home.join("skills/user/broken-skill");
    fs::create_dir_all(&broken_skill)?;
    fs::write(
        broken_skill.join("SKILL.md"),
        "---\nname: ../invalid\ndescription: Broken acceptance fixture\n---\nThis source must remain visible but ineligible.\n",
    )?;
    write_signed_marketplace(&marketplace)?;

    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home))?;
    let mut service = ExtensionService::restore(Arc::new(database), &home, now_ms)?;
    let snapshot = service.add_marketplace(
        MarketplaceSourceInput {
            source: marketplace.display().to_string(),
            git_ref: None,
            sparse_paths: Vec::new(),
            trusted_origin: "local:c4os-task-00011".into(),
            signing_key_id: "origin-test".into(),
            public_key_sha256: public_key_fingerprint(
                &BASE64.encode(origin_key.verifying_key().as_bytes()),
            )?,
        },
        now_ms + 1,
    )?;
    let snapshot = service.install_disabled(
        ExtensionPackageInput {
            expected_generation: snapshot.generation,
            package_id: PACKAGE_ID.into(),
        },
        now_ms + 2,
    )?;
    let snapshot = service.enable(
        ExtensionPackageInput {
            expected_generation: snapshot.generation,
            package_id: PACKAGE_ID.into(),
        },
        now_ms + 3,
    )?;
    let snapshot = service.review_hook(
        ExtensionHookReviewInput {
            expected_generation: snapshot.generation,
            package_id: PACKAGE_ID.into(),
            hook_id: "before-turn".into(),
        },
        now_ms + 4,
    )?;
    write_signed_update(&marketplace)?;
    let snapshot = service.refresh_catalogs(snapshot.generation, now_ms + 5)?;
    println!(
        "{}\n{}\nextension-generation={}",
        home.display(),
        marketplace.display(),
        snapshot.generation
    );
    Ok(())
}

fn acceptance_home_argument() -> Result<PathBuf, Box<dyn Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: task_00011_extension_seed <task-00007-acceptance-home>")?;
    if !path.is_absolute() || !path.join("state/app.sqlite3").is_file() {
        return Err("acceptance home must be an absolute Task 00007 seed home".into());
    }
    let canonical = path.canonicalize()?;
    if canonical != path {
        return Err("acceptance home must be canonical".into());
    }
    let permitted = [std::env::temp_dir(), PathBuf::from("/private/tmp")]
        .into_iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| canonical.starts_with(root));
    if !permitted {
        return Err("acceptance home must remain under the system temporary root".into());
    }
    Ok(canonical)
}

fn write_signed_marketplace(root: &Path) -> Result<(), Box<dyn Error>> {
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let content_key = SigningKey::from_bytes(&[9u8; 32]);
    let digest = write_signed_package(
        root,
        "sample-v1",
        "0.1.0",
        "Use the signed Skill golden path.",
        &origin_key,
        &content_key,
    )?;
    write_catalog(
        root,
        vec![CatalogRelease {
            package_id: PACKAGE_ID.into(),
            version: "0.1.0".into(),
            manifest_path: "sample-v1/c4os-package.toml".into(),
            package_digest: digest,
        }],
        &origin_key,
    )
}

fn write_signed_update(root: &Path) -> Result<(), Box<dyn Error>> {
    let origin_key = SigningKey::from_bytes(&[7u8; 32]);
    let content_key = SigningKey::from_bytes(&[9u8; 32]);
    let first: PackageManifest = toml::from_str(&fs::read_to_string(
        root.join("sample-v1/c4os-package.toml"),
    )?)?;
    let digest = write_signed_package(
        root,
        "sample-v2",
        "0.2.0",
        "Use the updated signed Skill golden path.",
        &origin_key,
        &content_key,
    )?;
    write_catalog(
        root,
        vec![
            CatalogRelease {
                package_id: PACKAGE_ID.into(),
                version: "0.1.0".into(),
                manifest_path: "sample-v1/c4os-package.toml".into(),
                package_digest: first.package_digest,
            },
            CatalogRelease {
                package_id: PACKAGE_ID.into(),
                version: "0.2.0".into(),
                manifest_path: "sample-v2/c4os-package.toml".into(),
                package_digest: digest,
            },
        ],
        &origin_key,
    )
}

fn write_signed_package(
    root: &Path,
    directory: &str,
    version: &str,
    instructions: &str,
    origin_key: &SigningKey,
    content_key: &SigningKey,
) -> Result<String, Box<dyn Error>> {
    let package_root = root.join(directory);
    fs::create_dir_all(package_root.join("skill"))?;
    fs::create_dir_all(package_root.join("hooks"))?;
    fs::write(
        package_root.join("skill/SKILL.md"),
        format!("---\nname: {SKILL_ID}\ndescription: Signed sample Skill\n---\n{instructions}\n"),
    )?;
    fs::write(
        package_root.join("hooks/before-turn.mjs"),
        "process.stdout.write(JSON.stringify({protocolVersion:1,proposals:[]}));\n",
    )?;
    let inventory = inventory_for_root(&package_root)?;
    let package_digest = digest_inventory(&inventory);
    let hook_digest = inventory
        .iter()
        .find(|entry| entry.path == "hooks/before-turn.mjs")
        .ok_or("hook inventory missing")?
        .sha256
        .clone();
    let mut manifest = PackageManifest {
        schema_version: 1,
        package: PackageMetadata {
            id: PACKAGE_ID.into(),
            name: "Sample Plugin".into(),
            summary: "Signed production lifecycle acceptance fixture".into(),
            publisher: "C4OS Tests".into(),
            version: version.into(),
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
            origin: "local:c4os-task-00011".into(),
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
        toml::to_string(&manifest)?,
    )?;
    Ok(package_digest)
}

fn write_catalog(
    root: &Path,
    releases: Vec<CatalogRelease>,
    origin_key: &SigningKey,
) -> Result<(), Box<dyn Error>> {
    let mut catalog = CatalogDocument {
        schema_version: 1,
        marketplace: CatalogMarketplace {
            id: "task-00011-local".into(),
            label: "Task 00011 signed local catalog".into(),
            origin: "local:c4os-task-00011".into(),
            signing_key_id: "origin-test".into(),
        },
        releases,
        signature: String::new(),
    };
    catalog.signature = BASE64.encode(
        origin_key
            .sign(&canonical_catalog_signing_bytes(&catalog))
            .to_bytes(),
    );
    fs::create_dir_all(root)?;
    fs::write(
        root.join("c4os-marketplace.toml"),
        toml::to_string(&catalog)?,
    )?;
    fs::write(
        root.join("c4os-marketplace-origin.toml"),
        format!(
            "schemaVersion = 1\norigin = {:?}\nkeyId = {:?}\npublicKey = {:?}\n",
            catalog.marketplace.origin,
            catalog.marketplace.signing_key_id,
            BASE64.encode(origin_key.verifying_key().as_bytes()),
        ),
    )?;
    Ok(())
}
