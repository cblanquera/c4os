#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use c4os_lib::runtime::opencode::OPENCODE_NATIVE_VERSION;
use c4os_lib::runtime::opencode_assets::{
    OpenCodeAssetError, OpenCodeProductionAssetFactory, native_dependency_tree_sha256,
};
use c4os_lib::runtime::opencode_native::VaultCredentialResolver;
use c4os_lib::runtime::supervisor::sha256_file;
use c4os_lib::security::credentials::CredentialVault;

const EXPECTED_NATIVE_SHA256: &str =
    "sha256:99d5d922f715ea0df9605f72849c68c0404d48b43ff319cef4cf943a0ee650e8";
const EXPECTED_NATIVE_TREE_SHA256: &str =
    "sha256:b743779f98c84d624462c6c789b4a8188b8b68355c7ee7dba32085027374c4ea";
const EXPECTED_SDK_TREE_SHA256: &str =
    "sha256:747f554f7f533c29c61d4da3034478a46869c70f10053209d5b82464b36a96e8";

#[test]
fn project_owned_exact_assets_resolve_without_proof_dependencies() {
    let root = project_root();
    assert_eq!(
        native_dependency_tree_sha256(&root.join("sidecars/opencode-native/node_modules"))
            .expect("native dependency tree digest"),
        EXPECTED_NATIVE_TREE_SHA256
    );
    let assets = OpenCodeProductionAssetFactory::new(&root)
        .expect("production asset factory")
        .resolve()
        .expect("verified production assets");

    assert_eq!(assets.resource_root(), root.canonicalize().unwrap());
    assert_eq!(assets.native_executable_sha256(), EXPECTED_NATIVE_SHA256);
    assert_eq!(assets.native_build_flavor(), "c4os-auth-fd-terminal-stop.2");
    assert_eq!(assets.native_tree_sha256(), EXPECTED_NATIVE_TREE_SHA256);
    assert_eq!(
        assets.sdk_dependency_tree_sha256(),
        EXPECTED_SDK_TREE_SHA256
    );
    assert!(
        assets
            .launcher()
            .ends_with("sidecars/opencode-launcher/main.mjs")
    );
    assert!(assets.sdk_root().ends_with("sidecars/opencode-sdk"));
    assert!(assets.native_root().ends_with("sidecars/opencode-native"));
    assert!(
        assets
            .native_executable()
            .ends_with("sidecars/opencode-native/node_modules/opencode-darwin-arm64/bin/opencode")
    );
    for path in [
        assets.launcher(),
        assets.sdk_root(),
        assets.native_root(),
        assets.native_executable(),
    ] {
        assert!(
            !path
                .components()
                .any(|component| component.as_os_str() == "proofs")
        );
    }

    let output = std::process::Command::new(assets.native_executable())
        .arg("--version")
        .output()
        .expect("exact native version");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        OPENCODE_NATIVE_VERSION
    );
}

#[test]
fn factory_verifies_assets_and_node_before_driver_construction() {
    let factory = OpenCodeProductionAssetFactory::new(project_root()).unwrap();
    let node = find_executable_on_path("node").expect("Node executable on PATH");
    let node_sha256 = sha256_file(&node).expect("Node digest");
    let resolver = VaultCredentialResolver::new(
        CredentialVault::session_only().expect("session-only credential vault"),
    );

    let prepared = factory
        .construct_driver(
            &node,
            &node_sha256,
            resolver.clone(),
            91,
            Duration::from_secs(3),
            Duration::from_secs(1),
        )
        .expect("verified production driver");
    assert_eq!(
        prepared.assets().native_executable_sha256(),
        EXPECTED_NATIVE_SHA256
    );

    assert!(matches!(
        factory.construct_driver(
            &node,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            resolver,
            91,
            Duration::from_secs(3),
            Duration::from_secs(1),
        ),
        Err(OpenCodeAssetError::InvalidIntegrity)
    ));
}

#[test]
#[ignore = "bundle tier: requires a built C4OS.app resource tree"]
fn bundled_opencode_assets_match_production_pins() {
    let configured = PathBuf::from(
        std::env::var_os("C4OS_BUNDLED_RESOURCE_ROOT")
            .expect("set C4OS_BUNDLED_RESOURCE_ROOT to the app resource directory"),
    );
    let bundled = if configured.is_absolute() {
        configured
    } else {
        project_root().join(configured)
    };
    let assets = OpenCodeProductionAssetFactory::new(bundled)
        .expect("bundled asset factory")
        .resolve()
        .expect("verified bundled assets");
    assert_eq!(assets.native_executable_sha256(), EXPECTED_NATIVE_SHA256);
    assert_eq!(assets.native_tree_sha256(), EXPECTED_NATIVE_TREE_SHA256);
    assert_eq!(
        assets.sdk_dependency_tree_sha256(),
        EXPECTED_SDK_TREE_SHA256
    );
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("project root")
        .to_path_buf()
}

fn find_executable_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|directory| {
        let candidate = directory.join(name);
        candidate.is_file().then(|| {
            candidate
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(name))
        })
    })
}
