#![cfg(unix)]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use c4os_lib::runtime::opencode_credential::{
    OPENCODE_CREDENTIAL_FD_ENV, OPENCODE_CREDENTIAL_SCHEMA_VERSION,
    OpenCodeProviderCredentialIssuer, ProviderCredentialRequest, opencode_message_id_for_operation,
};
use c4os_lib::runtime::provider::{
    PROVIDER_SCHEMA_VERSION, ProviderAuthentication, ProviderEndpoint, ProviderKind,
    ProviderProfile,
};
use c4os_lib::security::credentials::{CredentialReference, CredentialVault};
use serde_json::{Value, json};

const PROVIDER_SECRET: &str = "sk-c4os-native-provider-0123456789abcdef";

fn profile(credential_reference: CredentialReference) -> ProviderProfile {
    profile_with_id("provider-openai", credential_reference)
}

fn profile_with_id(
    provider_id: &str,
    credential_reference: CredentialReference,
) -> ProviderProfile {
    ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: provider_id.into(),
        kind: ProviderKind::OpenAi,
        display_name: "OpenAI".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "openai-default".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_kind: "openai".into(),
        },
        authentication: ProviderAuthentication::Bearer,
        credential_reference: Some(credential_reference),
        headers: BTreeMap::new(),
        enabled: true,
    }
}

fn attempt(
    generation: u64,
    profile_id: &str,
    native_session_id: &str,
    model_id: &str,
    operation_id: &str,
) -> ProviderCredentialRequest {
    ProviderCredentialRequest {
        process_generation: generation,
        native_session_id: native_session_id.into(),
        provider_id: profile_id.into(),
        model_id: model_id.into(),
        operation_id: operation_id.into(),
    }
}

fn request_lease(
    worker: &mut UnixStream,
    generation: u64,
    request_id: &str,
    native_session_id: &str,
    native_provider_id: &str,
    native_model_id: &str,
    operation_id: &str,
) -> Value {
    let request = json!({
        "schemaVersion": OPENCODE_CREDENTIAL_SCHEMA_VERSION,
        "kind": "providerCredentialRequest",
        "requestId": request_id,
        "processGeneration": generation,
        "nativeSessionId": native_session_id,
        "providerId": native_provider_id,
        "modelId": native_model_id,
        "operationId": operation_id,
        "nativeMessageId": format!("msg_{operation_id}"),
    });
    worker
        .write_all(format!("{request}\n").as_bytes())
        .expect("worker credential request");
    worker.flush().expect("worker credential request flush");
    let mut line = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        worker
            .read_exact(&mut byte)
            .expect("Rust credential response");
        line.push(byte[0]);
        if byte[0] == b'\n' {
            break;
        }
        assert!(line.len() <= 4 * 1024, "credential response is bounded");
    }
    serde_json::from_slice(&line).expect("credential response metadata")
}

fn read_secret(worker: &mut UnixStream, metadata: &Value) -> Vec<u8> {
    let secret_length = metadata["secretLength"].as_u64().unwrap() as usize;
    let mut secret = vec![0_u8; secret_length];
    worker
        .read_exact(&mut secret)
        .expect("credential response secret");
    secret
}

#[test]
fn authorizes_without_delivery_then_answers_one_exact_worker_request() {
    let vault = CredentialVault::session_only().unwrap();
    let reference = vault
        .store("provider-api-key", PROVIDER_SECRET.as_bytes())
        .unwrap();
    let provider = profile(reference.clone());
    let (mut issuer, binding) =
        OpenCodeProviderCredentialIssuer::authenticated_pair(vault, 11).unwrap();
    issuer.register_provider(&provider).unwrap();
    let mut worker = binding.duplicate_worker_for_test().unwrap();
    worker.set_nonblocking(true).unwrap();

    let receipt = issuer
        .authorize_attempt(attempt(
            11,
            "provider-openai",
            "native-session-1",
            "provider-openai/gpt-5",
            "attempt-correlation-1",
        ))
        .unwrap();
    let mut absent = [0_u8; 1];
    assert!(
        matches!(worker.read(&mut absent), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "authorization must not pre-deliver a credential"
    );
    worker.set_nonblocking(false).unwrap();
    worker
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    let metadata = request_lease(
        &mut worker,
        11,
        "request-1",
        "native-session-1",
        "provider-openai",
        "gpt-5",
        "attempt-correlation-1",
    );
    assert_eq!(metadata["kind"], "providerCredential");
    assert_eq!(metadata["requestId"], "request-1");
    assert_eq!(metadata["processGeneration"], 11);
    assert_eq!(metadata["nativeSessionId"], "native-session-1");
    assert_eq!(metadata["providerId"], "provider-openai");
    assert_eq!(metadata["modelId"], "gpt-5");
    assert_eq!(metadata["operationId"], "attempt-correlation-1");
    assert_eq!(metadata["nativeMessageId"], "msg_attempt-correlation-1");
    assert_eq!(metadata["headerName"], "Authorization");
    assert_eq!(metadata["headerPrefix"], "Bearer ");
    assert_eq!(
        metadata["authorizationId"],
        receipt.operation_authorization_id
    );
    let mut secret = read_secret(&mut worker, &metadata);
    assert_eq!(secret, PROVIDER_SECRET.as_bytes());
    secret.fill(0);

    let debug = format!("{issuer:?} {binding:?} {receipt:?}");
    assert!(!debug.contains(PROVIDER_SECRET));
    assert!(!debug.contains(reference.as_str()));
    let environment = binding.environment();
    assert_eq!(environment.len(), 1);
    assert_eq!(
        environment[OPENCODE_CREDENTIAL_FD_ENV],
        binding.child_fd().to_string()
    );
    assert!(
        environment
            .values()
            .all(|value| !value.contains(PROVIDER_SECRET))
    );
}

#[test]
fn derives_a_bounded_opencode_message_identity_from_the_core_correlation() {
    assert_eq!(
        opencode_message_id_for_operation("attempt-correlation-1").unwrap(),
        "msg_attempt-correlation-1"
    );
    assert!(opencode_message_id_for_operation(&"a".repeat(189)).is_err());
    assert!(opencode_message_id_for_operation("invalid correlation").is_err());
}

#[test]
fn rejects_substitution_replay_and_revoked_attempts_without_secret_delivery() {
    let vault = CredentialVault::session_only().unwrap();
    let reference = vault
        .store("provider-api-key", PROVIDER_SECRET.as_bytes())
        .unwrap();
    let provider = profile(reference);
    let (mut issuer, binding) =
        OpenCodeProviderCredentialIssuer::authenticated_pair(vault, 12).unwrap();
    issuer.register_provider(&provider).unwrap();
    let accepted = attempt(
        12,
        "provider-openai",
        "native-session-1",
        "provider-openai/gpt-5",
        "operation-active",
    );
    issuer.authorize_attempt(accepted.clone()).unwrap();
    let mut worker = binding.duplicate_worker_for_test().unwrap();
    worker
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    let substituted = request_lease(
        &mut worker,
        12,
        "request-substitution",
        "native-session-substituted",
        "provider-openai",
        "gpt-5",
        "operation-active",
    );
    assert_eq!(substituted["kind"], "providerCredentialRejected");
    assert_eq!(substituted["code"], "binding_mismatch");

    let first = request_lease(
        &mut worker,
        12,
        "request-active",
        "native-session-1",
        "provider-openai",
        "gpt-5",
        "operation-active",
    );
    let mut first_secret = read_secret(&mut worker, &first);
    assert_eq!(first_secret, PROVIDER_SECRET.as_bytes());
    first_secret.fill(0);
    let replay = request_lease(
        &mut worker,
        12,
        "request-active",
        "native-session-1",
        "provider-openai",
        "gpt-5",
        "operation-active",
    );
    assert_eq!(replay["kind"], "providerCredentialRejected");
    assert_eq!(replay["code"], "lease_replay");

    assert!(issuer.revoke_attempt(&accepted).unwrap());
    let revoked = request_lease(
        &mut worker,
        12,
        "request-after-revoke",
        "native-session-1",
        "provider-openai",
        "gpt-5",
        "operation-active",
    );
    assert_eq!(revoked["kind"], "providerCredentialRejected");
    assert_eq!(revoked["code"], "binding_mismatch");
}

#[test]
fn one_active_attempt_can_receive_fresh_leases_for_provider_continuations() {
    let vault = CredentialVault::session_only().unwrap();
    let reference = vault
        .store("provider-api-key", PROVIDER_SECRET.as_bytes())
        .unwrap();
    let (mut issuer, binding) =
        OpenCodeProviderCredentialIssuer::authenticated_pair(vault, 13).unwrap();
    issuer.register_provider(&profile(reference)).unwrap();
    issuer
        .authorize_attempt(attempt(
            13,
            "provider-openai",
            "native-session-continue",
            "provider-openai/gpt-5",
            "operation-continue",
        ))
        .unwrap();
    let mut worker = binding.duplicate_worker_for_test().unwrap();
    worker
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    let mut lease_ids = Vec::new();
    for request_id in ["request-initial", "request-after-tool"] {
        let response = request_lease(
            &mut worker,
            13,
            request_id,
            "native-session-continue",
            "provider-openai",
            "gpt-5",
            "operation-continue",
        );
        lease_ids.push(response["leaseId"].as_str().unwrap().to_owned());
        let mut secret = read_secret(&mut worker, &response);
        assert_eq!(secret, PROVIDER_SECRET.as_bytes());
        secret.fill(0);
    }
    assert_ne!(lease_ids[0], lease_ids[1]);
}

#[test]
fn distinct_core_profiles_can_share_one_native_provider_across_distinct_sessions() {
    const TEAM_A_SECRET: &str = "sk-c4os-team-a-0123456789abcdef";
    const TEAM_B_SECRET: &str = "sk-c4os-team-b-0123456789abcdef";

    let vault = CredentialVault::session_only().unwrap();
    let team_a_reference = vault
        .store("team-a-api-key", TEAM_A_SECRET.as_bytes())
        .unwrap();
    let team_b_reference = vault
        .store("team-b-api-key", TEAM_B_SECRET.as_bytes())
        .unwrap();
    let (mut issuer, binding) =
        OpenCodeProviderCredentialIssuer::authenticated_pair(vault, 14).unwrap();
    issuer
        .register_provider(&profile_with_id("openai-team-a", team_a_reference))
        .unwrap();
    issuer
        .register_provider(&profile_with_id("openai-team-b", team_b_reference))
        .unwrap();
    issuer
        .authorize_attempt(attempt(
            14,
            "openai-team-a",
            "native-session-team-a",
            "openai-team-a/gpt-5",
            "operation-team-a",
        ))
        .unwrap();
    issuer
        .authorize_attempt(attempt(
            14,
            "openai-team-b",
            "native-session-team-b",
            "openai-team-b/gpt-5",
            "operation-team-b",
        ))
        .unwrap();

    let mut worker = binding.duplicate_worker_for_test().unwrap();
    worker
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    for (request_id, native_session_id, operation_id, expected_secret) in [
        (
            "request-team-a",
            "native-session-team-a",
            "operation-team-a",
            TEAM_A_SECRET,
        ),
        (
            "request-team-b",
            "native-session-team-b",
            "operation-team-b",
            TEAM_B_SECRET,
        ),
    ] {
        let metadata = request_lease(
            &mut worker,
            14,
            request_id,
            native_session_id,
            if request_id == "request-team-a" {
                "openai-team-a"
            } else {
                "openai-team-b"
            },
            "gpt-5",
            operation_id,
        );
        let mut secret = read_secret(&mut worker, &metadata);
        assert_eq!(secret, expected_secret.as_bytes());
        secret.fill(0);
    }
}
