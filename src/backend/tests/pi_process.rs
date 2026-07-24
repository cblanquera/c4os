#![cfg(unix)]

use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use c4os_lib::runtime::pi::{PiAdapter, PiAdapterState, PiModelRoute, PiSidecarManifest};
use c4os_lib::runtime::pi_process::{
    PiCredentialLeaseMetadata, PiProcessError, PiSidecarIntegrity, SpawnedPiRunner,
};
use c4os_lib::runtime::supervisor::sha256_file;
use c4os_lib::security::credentials::CredentialVault;

// Every test traverses or clones the same large exact dependency graph. Keep
// this binary deterministic: parallel graph I/O can starve the live sidecar's
// bounded health exchange and create a false protocol timeout.
static PI_GRAPH_TEST: Mutex<()> = Mutex::new(());

#[test]
fn test_tls_trust_spawn_requires_one_bounded_private_descriptor() {
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi");
    let manifest = PiSidecarManifest::load(&sidecar_root).unwrap();
    for descriptor in [Vec::new(), vec![0_u8; 256 * 1024 + 1]] {
        assert!(matches!(
            SpawnedPiRunner::spawn_with_test_tls_trust(
                Path::new("/usr/bin/false"),
                "sha256:not-read-for-invalid-descriptor",
                &sidecar_root,
                &manifest,
                1,
                Duration::from_secs(1),
                descriptor,
            ),
            Err(PiProcessError::TestTlsTrustChannel)
        ));
    }
}

fn node_executable() -> PathBuf {
    let output = Command::new("node")
        .args(["-p", "process.execPath"])
        .output()
        .expect("Node must be available for the native Pi sidecar test");
    assert!(output.status.success());
    PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
}

#[test]
fn spawned_pi_sidecar_uses_exact_health_version_and_dedicated_one_shot_credential_fd() {
    let _graph_test = PI_GRAPH_TEST.lock().expect("Pi graph test lock");
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi");
    let manifest = PiSidecarManifest::load(&sidecar_root).unwrap();
    let node = node_executable();
    let node_sha256 = sha256_file(&node).unwrap();
    let mut runner = SpawnedPiRunner::spawn(
        &node,
        &node_sha256,
        &sidecar_root,
        &manifest,
        7,
        Duration::from_secs(10),
    )
    .unwrap();

    let vault = CredentialVault::session_only().unwrap();
    let reference = vault
        .store("openai-api-key", b"native-process-secret")
        .unwrap();
    let lease = vault
        .lease_for_operation(&reference, "pi-provider-test", Duration::from_secs(30))
        .unwrap();
    let metadata = PiCredentialLeaseMetadata {
        lease_id: "lease-native-1".into(),
        provider: "openai".into(),
        expires_at_ms: 9_999_999_999_999,
    };
    runner.deliver_credential_lease(&metadata, &lease).unwrap();
    assert!(
        runner.deliver_credential_lease(&metadata, &lease).is_err(),
        "the credential lease must be one-shot"
    );

    let mut adapter = PiAdapter::new(manifest, runner, 7).unwrap();
    let (health, version) = adapter.start().unwrap();
    assert_eq!(health.status, "ready");
    assert_eq!(health.process_generation, 7);
    assert_eq!(
        health.capabilities["providerAuthentication"].state,
        "supported"
    );
    assert_eq!(version.native_version, "0.80.10");
    assert_eq!(adapter.state(), &PiAdapterState::Degraded);
    adapter
        .create_session(
            "workspace-native",
            "session-native",
            PiModelRoute {
                provider: "openai".into(),
                model_id: "gpt-4o-mini".into(),
                base_url: "https://api.openai.com/v1".into(),
            },
            &BTreeSet::from(["c4os_propose_action".into()]),
        )
        .unwrap();
    adapter.shutdown().unwrap();
    assert_eq!(adapter.state(), &PiAdapterState::Stopped);
}

#[test]
fn rust_verifies_every_local_sidecar_source_package_and_lock_pin() {
    let _graph_test = PI_GRAPH_TEST.lock().expect("Pi graph test lock");
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi");
    assert_eq!(
        PiSidecarIntegrity::dependency_tree_sha256(&sidecar_root).unwrap(),
        "sha256:f42d94fbecf198d004e91a456c8f0f9e5339eef2cfdf49c880110b310d219a06"
    );
    PiSidecarIntegrity::verify(&sidecar_root).unwrap();

    for tampered in ["main.mjs", "package.json", "package-lock.json"] {
        let temporary = tempfile::tempdir().unwrap();
        copy_source_graph(&sidecar_root, temporary.path());
        let target = temporary.path().join(tampered);
        fs::write(&target, b"tampered").unwrap();
        assert!(matches!(
            PiSidecarIntegrity::verify(temporary.path()),
            Err(PiProcessError::InvalidSidecarIntegrity)
        ));
    }
}

#[test]
fn dependency_tree_tamper_substitution_and_escape_fail_closed() {
    let _graph_test = PI_GRAPH_TEST.lock().expect("Pi graph test lock");
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi");
    let temporary = tempfile::tempdir().unwrap();
    copy_integrity_graph(&sidecar_root, temporary.path());
    PiSidecarIntegrity::verify(temporary.path()).unwrap();

    let package = temporary
        .path()
        .join("node_modules/@earendil-works/pi-agent-core/package.json");
    let original_package = fs::read(&package).unwrap();
    fs::remove_file(&package).unwrap();
    fs::write(
        &package,
        b"{\"name\":\"substituted\",\"version\":\"0.80.10\"}",
    )
    .unwrap();
    assert!(matches!(
        PiSidecarIntegrity::verify(temporary.path()),
        Err(PiProcessError::InvalidSidecarIntegrity)
    ));
    fs::write(&package, original_package).unwrap();

    let injected = temporary.path().join("node_modules/injected.mjs");
    fs::write(&injected, b"export const substituted = true;\n").unwrap();
    assert!(matches!(
        PiSidecarIntegrity::verify(temporary.path()),
        Err(PiProcessError::InvalidSidecarIntegrity)
    ));
    fs::remove_file(injected).unwrap();

    let bin = temporary.path().join("node_modules/.bin/pi");
    let original_link = fs::read_link(&bin).unwrap();
    fs::remove_file(&bin).unwrap();
    let outside = temporary.path().join("outside");
    fs::write(&outside, b"outside").unwrap();
    symlink(&outside, &bin).unwrap();
    assert!(matches!(
        PiSidecarIntegrity::verify(temporary.path()),
        Err(PiProcessError::InvalidSidecarIntegrity)
    ));
    assert!(!original_link.as_os_str().is_empty());
}

#[test]
#[ignore = "set C4OS_BUNDLED_PI_SIDECAR to the packaged sidecars/pi resource"]
fn bundled_pi_dependency_tree_matches_production_pins() {
    let _graph_test = PI_GRAPH_TEST.lock().expect("Pi graph test lock");
    let configured = std::env::var_os("C4OS_BUNDLED_PI_SIDECAR")
        .map(PathBuf::from)
        .expect("C4OS_BUNDLED_PI_SIDECAR must name the packaged Pi sidecar");
    let bundled = if configured.is_absolute() {
        configured
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("project root")
            .join(configured)
    };
    PiSidecarIntegrity::verify(&bundled).unwrap();
    assert_eq!(
        PiSidecarIntegrity::dependency_tree_sha256(&bundled).unwrap(),
        "sha256:f42d94fbecf198d004e91a456c8f0f9e5339eef2cfdf49c880110b310d219a06"
    );
}

#[test]
fn spawn_rejects_source_substitution_before_creating_the_node_process() {
    let _graph_test = PI_GRAPH_TEST.lock().expect("Pi graph test lock");
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi");
    let manifest = PiSidecarManifest::load(&sidecar_root).unwrap();
    let node = node_executable();
    let node_sha256 = sha256_file(&node).unwrap();
    let mut substituted_manifest = manifest.clone();
    substituted_manifest.adapter_version = "9.9.9".into();
    assert!(matches!(
        SpawnedPiRunner::spawn(
            &node,
            &node_sha256,
            &sidecar_root,
            &substituted_manifest,
            9,
            Duration::from_secs(1),
        ),
        Err(PiProcessError::InvalidSidecarIntegrity)
    ));

    let temporary = tempfile::tempdir().unwrap();
    copy_source_graph(&sidecar_root, temporary.path());
    fs::write(temporary.path().join("main.mjs"), b"tampered after load").unwrap();
    assert!(matches!(
        SpawnedPiRunner::spawn(
            &node,
            &node_sha256,
            temporary.path(),
            &manifest,
            9,
            Duration::from_secs(1),
        ),
        Err(PiProcessError::InvalidSidecarIntegrity)
    ));
}

fn copy_integrity_graph(source: &Path, destination: &Path) {
    copy_source_graph(source, destination);
    copy_dependency_tree(
        &source.join("node_modules"),
        &destination.join("node_modules"),
    );
}

fn copy_source_graph(source: &Path, destination: &Path) {
    for relative in [
        "main.mjs",
        "adapter.mjs",
        "pi-sdk-driver.mjs",
        "protocol.mjs",
        "sidecar-manifest.json",
        "package.json",
        "package-lock.json",
    ] {
        fs::copy(source.join(relative), destination.join(relative)).unwrap();
    }
}

fn copy_dependency_tree(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path).unwrap();
        if metadata.file_type().is_symlink() {
            symlink(fs::read_link(source_path).unwrap(), destination_path).unwrap();
        } else if metadata.is_dir() {
            copy_dependency_tree(&source_path, &destination_path);
        } else {
            fs::hard_link(source_path, destination_path).unwrap();
        }
    }
}
