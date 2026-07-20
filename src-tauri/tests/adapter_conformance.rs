use std::path::PathBuf;

use c4os_lib::runtime::adapter::{
    AdapterAuthority, AdapterConformanceDescriptor, AdapterEventIdentity,
};
use c4os_lib::runtime::capability::CapabilityState;
use c4os_lib::runtime::opencode::OpenCodeCompatibilityManifest;
use c4os_lib::runtime::pi::PiSidecarManifest;
use c4os_lib::runtime::supervisor::RuntimeKind;

const DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn assert_shared_peer_contract(
    descriptor: AdapterConformanceDescriptor,
    expected_kind: RuntimeKind,
    expected_native_version: &str,
) {
    descriptor.validate().unwrap();
    assert_eq!(descriptor.runtime_kind, expected_kind);
    assert_eq!(descriptor.native_version, expected_native_version);
    assert_eq!(
        descriptor.authority,
        AdapterAuthority::C4osActionGatewayOnly
    );
    for capability in [
        "health",
        "session-create",
        "session-resume",
        "model-discovery",
        "streaming",
        "action-intents",
        "credential-channel",
        "cancellation",
        "restart",
    ] {
        assert!(
            descriptor.capabilities.contains_key(capability),
            "{expected_kind:?} omitted {capability} evidence"
        );
    }
    assert!(matches!(
        descriptor.capabilities["action-intents"],
        CapabilityState::Supported | CapabilityState::Degraded
    ));

    let identity = AdapterEventIdentity {
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        run_id: "run-1".into(),
        correlation_id: "correlation-1".into(),
        runtime_kind: expected_kind,
        process_generation: descriptor.process_generation,
        sequence: 1,
    };
    identity.validate_against(&descriptor).unwrap();

    let mut stale = identity.clone();
    stale.process_generation += 1;
    assert!(stale.validate_against(&descriptor).is_err());
    stale = identity;
    stale.runtime_kind = if expected_kind == RuntimeKind::OpenCode {
        RuntimeKind::Pi
    } else {
        RuntimeKind::OpenCode
    };
    assert!(stale.validate_against(&descriptor).is_err());
}

#[test]
fn opencode_and_pi_pass_one_c4os_owned_peer_conformance_matrix() {
    let open_code = OpenCodeCompatibilityManifest::pinned(DIGEST)
        .unwrap()
        .conformance_descriptor(7)
        .unwrap();
    assert_shared_peer_contract(open_code, RuntimeKind::OpenCode, "1.18.3");

    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
    let pi = PiSidecarManifest::load(&sidecar_root)
        .unwrap()
        .conformance_descriptor(7, true)
        .unwrap();
    assert_shared_peer_contract(pi, RuntimeKind::Pi, "0.80.10");
}

#[test]
fn peer_descriptors_preserve_behavior_differences_instead_of_copying_defaults() {
    let open_code = OpenCodeCompatibilityManifest::pinned(DIGEST)
        .unwrap()
        .conformance_descriptor(7)
        .unwrap();
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
    let pi = PiSidecarManifest::load(&sidecar_root)
        .unwrap()
        .conformance_descriptor(7, true)
        .unwrap();

    assert_eq!(
        open_code.capabilities["model-discovery"],
        CapabilityState::Supported
    );
    assert_eq!(
        pi.capabilities["model-discovery"],
        CapabilityState::Unsupported
    );
    assert_eq!(open_code.capabilities["restart"], CapabilityState::Degraded);
    assert_eq!(pi.capabilities["restart"], CapabilityState::Degraded);
    assert_eq!(
        open_code.capabilities["session-resume"],
        CapabilityState::Degraded
    );
    assert_eq!(
        open_code.capabilities["action-intents"],
        CapabilityState::Degraded
    );
    assert_eq!(
        pi.capabilities["action-intents"],
        CapabilityState::Supported
    );
}

#[test]
fn pi_declares_degraded_native_resume_instead_of_silently_claiming_support() {
    let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
    let descriptor = PiSidecarManifest::load(&sidecar_root)
        .unwrap()
        .conformance_descriptor(1, false)
        .unwrap();
    assert_eq!(
        descriptor.capabilities["session-resume"],
        CapabilityState::Degraded
    );
    assert_eq!(
        descriptor.capabilities["credential-channel"],
        CapabilityState::Degraded
    );
}

#[test]
fn unknown_descriptor_fields_fail_strict_deserialization() {
    let descriptor = OpenCodeCompatibilityManifest::pinned(DIGEST)
        .unwrap()
        .conformance_descriptor(1)
        .unwrap();
    let mut value = serde_json::to_value(descriptor).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("nativeAuthority".into(), serde_json::json!(true));
    assert!(serde_json::from_value::<AdapterConformanceDescriptor>(value).is_err());
}
