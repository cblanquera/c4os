#![cfg(unix)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use c4os_lib::runtime::opencode::{
    CommandDriver, CommandFailureCode, EventCorrelation, HttpMethod, LoopbackEndpoint, ModelRoute,
    NativeAuthorityPolicy, OpenCodeAdapter, OpenCodeCompatibilityManifest, OpenCodeLaunchPlan,
    OpenCodeTransport, PromptDispatch, RandomSecretReference, StateNamespace, TransportAuth,
    TransportFailureCode, TransportRequest,
};
use c4os_lib::runtime::opencode_assets::OpenCodeProductionAssetFactory;
use c4os_lib::runtime::opencode_credential::ProviderCredentialRequest;
use c4os_lib::runtime::opencode_native::{
    LoopbackHttpTransport, NativeBrokerError, OPENCODE_AUTHORITY_CONFIG, OPENCODE_C4OS_TOOL_IDS,
    VaultCredentialResolver, opencode_authority_configuration_sha256, sha256_bytes,
};
use c4os_lib::runtime::opencode_sdk::{OpenCodeSdkIntegrity, dependency_tree_sha256};
use c4os_lib::runtime::provider::{
    PROVIDER_SCHEMA_VERSION, ProviderEndpoint, ProviderKind, ProviderProfile,
};
use c4os_lib::runtime::supervisor::sha256_file;
use c4os_lib::security::credentials::CredentialVault;
use serde_json::{Value, json};
use tempfile::TempDir;

const SECRET: &str = "native-opencode-password-0123456789abcdef";
const PROVIDER_SECRET: &str = "sk-c4os-live-provider-0123456789abcdef";

fn random_reference(id: &str) -> RandomSecretReference {
    RandomSecretReference::new(id, 256).expect("valid random-secret reference")
}

fn resolver_with_secret(
    reference: &RandomSecretReference,
    secret: &[u8],
) -> VaultCredentialResolver {
    let vault = CredentialVault::session_only().expect("session-only vault");
    let credential = vault
        .store("opencode-server-password", secret)
        .expect("store server password");
    let resolver = VaultCredentialResolver::new(vault);
    resolver
        .register(reference, credential)
        .expect("register opaque reference");
    resolver
}

fn request(endpoint: &LoopbackEndpoint, reference: &RandomSecretReference) -> TransportRequest {
    request_path(endpoint, reference, "/global/health")
}

fn request_path(
    endpoint: &LoopbackEndpoint,
    reference: &RandomSecretReference,
    path: &str,
) -> TransportRequest {
    TransportRequest {
        method: HttpMethod::Get,
        base_url: endpoint.base_url(),
        path: path.into(),
        body: None,
        auth: TransportAuth::Basic {
            username: "opencode".into(),
            password_reference: reference.clone(),
        },
        maximum_response_bytes: 16 * 1024,
    }
}

fn serve_once(response: Vec<u8>) -> (LoopbackEndpoint, thread::JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind loopback fixture");
    let address = listener.local_addr().expect("fixture address");
    let endpoint = LoopbackEndpoint::new(address.ip(), address.port()).expect("loopback endpoint");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept fixture request");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("fixture timeout");
        let mut request = Vec::new();
        let mut chunk = [0_u8; 4 * 1024];
        loop {
            let read = stream.read(&mut chunk).expect("read fixture request");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        stream.write_all(&response).expect("write fixture response");
        request
    });
    (endpoint, handle)
}

#[test]
#[ignore = "local-loopback tier: requires permission to bind an authenticated fixture"]
fn authenticated_transport_is_loopback_fixed_bounded_and_redacted() {
    let body = br#"{"healthy":true,"version":"1.18.3"}"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes()
    .into_iter()
    .chain(body.iter().copied())
    .collect();
    let (endpoint, server) = serve_once(response);
    let reference = random_reference("opencode-http-secret");
    let resolver = resolver_with_secret(&reference, SECRET.as_bytes());
    let mut transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        resolver,
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
    .expect("transport");

    let transport_debug = format!("{transport:?}");
    let request = request(&endpoint, &reference);
    let request_debug = format!("{request:?}");
    let response = transport.execute(request).expect("authenticated request");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, body);
    assert!(!transport_debug.contains(SECRET));
    assert!(!request_debug.contains(SECRET));

    let received = String::from_utf8(server.join().expect("fixture server")).expect("HTTP text");
    let expected = format!(
        "Authorization: Basic {}",
        base64_for_test(format!("opencode:{SECRET}").as_bytes())
    );
    assert!(received.contains(&expected));
    assert!(!received.contains(SECRET));
    assert!(received.starts_with("GET /global/health HTTP/1.1\r\n"));
}

#[test]
#[ignore = "local-loopback tier: requires permission to bind an authenticated fixture"]
fn transport_rejects_unmapped_credentials_and_endpoint_substitution_before_io() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("reserve port");
    listener
        .set_nonblocking(true)
        .expect("nonblocking listener");
    let address = listener.local_addr().expect("fixture address");
    let endpoint = LoopbackEndpoint::new(address.ip(), address.port()).expect("endpoint");
    let unmapped = random_reference("unmapped-http-secret");
    let resolver =
        VaultCredentialResolver::new(CredentialVault::session_only().expect("session-only vault"));
    let mut transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        resolver,
        Duration::from_millis(100),
        Duration::from_millis(100),
    )
    .expect("transport");
    assert_eq!(
        transport.execute(request(&endpoint, &unmapped)),
        Err(TransportFailureCode::AuthenticationRejected)
    );
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );

    let mapped = random_reference("mapped-http-secret");
    let resolver = resolver_with_secret(&mapped, SECRET.as_bytes());
    let mut transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        resolver,
        Duration::from_millis(100),
        Duration::from_millis(100),
    )
    .expect("transport");
    let mut substituted = request(&endpoint, &mapped);
    substituted.base_url = "http://127.0.0.1:9".into();
    assert_eq!(
        transport.execute(substituted),
        Err(TransportFailureCode::Protocol)
    );
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
}

#[test]
#[ignore = "local-loopback tier: requires permission to bind an authenticated fixture"]
fn transport_decodes_chunked_responses_and_rejects_ambiguous_framing() {
    let (endpoint, server) = serve_once(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nC4OS\r\n0\r\n\r\n".to_vec(),
    );
    let reference = random_reference("chunked-http-secret");
    let resolver = resolver_with_secret(&reference, SECRET.as_bytes());
    let mut transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        resolver,
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
    .expect("transport");
    assert_eq!(
        transport
            .execute(request(&endpoint, &reference))
            .expect("chunked response")
            .body,
        b"C4OS"
    );
    server.join().expect("fixture server");

    let (endpoint, server) =
        serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{}".to_vec());
    let reference = random_reference("ambiguous-http-secret");
    let resolver = resolver_with_secret(&reference, SECRET.as_bytes());
    let mut transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        resolver,
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
    .expect("transport");
    assert_eq!(
        transport.execute(request(&endpoint, &reference)),
        Err(TransportFailureCode::Protocol)
    );
    server.join().expect("fixture server");
}

#[test]
fn command_driver_fails_closed_before_spawn_on_digest_or_descriptor_drift() {
    let temporary = TempDir::new().expect("temporary root");
    let reference = random_reference("driver-secret");
    let resolver = resolver_with_secret(&reference, SECRET.as_bytes());
    let node = find_executable_on_path("node").expect("Node executable on PATH");
    let prepared = OpenCodeProductionAssetFactory::new(project_root())
        .expect("production asset factory")
        .construct_driver(
            &node,
            &sha256_file(&node).expect("Node digest"),
            resolver,
            11,
            Duration::from_secs(1),
            Duration::from_secs(1),
        )
        .expect("factory-pinned driver");
    let native_executable = prepared.assets().native_executable().to_path_buf();
    let native_sha256 = prepared.assets().native_executable_sha256().to_owned();
    let (_, mut driver) = prepared.into_parts();
    assert!(
        driver
            .install_test_tls_trust_descriptor(Vec::new())
            .is_err()
    );
    assert!(
        driver
            .install_test_tls_trust_descriptor(vec![0; 256 * 1024 + 1])
            .is_err()
    );
    driver
        .install_test_tls_trust_descriptor(br#"{"schemaVersion":1}"#.to_vec())
        .expect("bounded test TLS descriptor");
    assert!(
        driver
            .install_test_tls_trust_descriptor(br#"{"schemaVersion":1}"#.to_vec())
            .is_err(),
        "test TLS descriptor is one-use per driver"
    );
    assert!(!format!("{driver:?}").contains("schemaVersion"));
    assert!(matches!(
        driver.receive_broker_event(Duration::from_millis(1)),
        Err(NativeBrokerError::Unavailable)
    ));
    let substituted = temporary.path().join("substituted-opencode");
    std::os::unix::fs::symlink(&native_executable, &substituted)
        .expect("same-binary path substitution");
    let mut launch = launch_command_fixture(
        temporary.path(),
        substituted,
        native_sha256.clone(),
        reference,
        198,
    );
    assert_eq!(
        driver.spawn(&launch),
        Err(CommandFailureCode::SpawnRejected)
    );

    assert!(
        NativeAuthorityPolicy::new(opencode_authority_configuration_sha256(), ["write", "bash"],)
            .is_err(),
        "native effect tool identities must be rejected before driver launch"
    );

    launch.executable = native_executable;
    launch.expected_binary_sha256 =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".into();
    assert_eq!(
        driver.spawn(&launch),
        Err(CommandFailureCode::SpawnRejected),
        "the launch request cannot replace the factory-owned digest"
    );

    launch.authority_policy = NativeAuthorityPolicy::new(
        opencode_authority_configuration_sha256(),
        OPENCODE_C4OS_TOOL_IDS,
    )
    .expect("C4OS-only tools");
    launch.expected_binary_sha256 = native_sha256;
    launch.secret_channel.inherited_fd = 7;
    assert_eq!(
        driver.spawn(&launch),
        Err(CommandFailureCode::SpawnRejected)
    );
    assert!(!format!("{driver:?}").contains(SECRET));
    assert!(!format!("{launch:?}").contains(SECRET));
}

#[test]
fn factory_pinned_driver_rejects_self_verified_fake_native_before_materialization() {
    let project_root = project_root();
    let node = find_executable_on_path("node").expect("Node executable on PATH");
    let temporary = TempDir::new().expect("native fixture root");
    let executable = temporary.path().join("fake-opencode");
    fs::write(
        &executable,
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 1.18.3; exit 0; fi\nexit 1\n",
    )
    .expect("write self-verified fake OpenCode");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))
        .expect("fake OpenCode permissions");
    let workspace = temporary.path().join("workspace");
    fs::create_dir(&workspace).expect("workspace root");
    let endpoint =
        LoopbackEndpoint::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 49_194).expect("fixture endpoint");
    let reference = random_reference("integrated-broker-password");
    let resolver = resolver_with_secret(&reference, SECRET.as_bytes());
    let namespace = StateNamespace::new(
        temporary.path(),
        "workspace-integrated",
        42,
        "launch-integrated",
    )
    .expect("fixture namespace");
    let plan = OpenCodeLaunchPlan {
        manifest: OpenCodeCompatibilityManifest::pinned(
            sha256_file(&executable).expect("fixture digest"),
        )
        .expect("fixture manifest"),
        endpoint,
        namespace,
        executable,
        workspace_root: workspace,
        basic_auth_username: "opencode".into(),
        password_reference: reference,
        secret_channel_fd: 198,
        authority_policy: NativeAuthorityPolicy::new(
            opencode_authority_configuration_sha256(),
            OPENCODE_C4OS_TOOL_IDS,
        )
        .expect("authority policy"),
    };
    let command = plan.command().expect("fixture launch command");
    let isolated_config = PathBuf::from(command.environment["XDG_CONFIG_HOME"].clone());
    let prepared = OpenCodeProductionAssetFactory::new(&project_root)
        .expect("production asset factory")
        .construct_driver(
            &node,
            &sha256_file(&node).expect("Node digest"),
            resolver,
            42,
            Duration::from_secs(5),
            Duration::from_secs(2),
        )
        .expect("factory-pinned driver");
    let (_, mut driver) = prepared.into_parts();

    assert_eq!(
        driver.spawn(&command),
        Err(CommandFailureCode::SpawnRejected)
    );
    assert!(
        !isolated_config.exists(),
        "a substituted native must fail before isolated state or plugins are materialized"
    );
    assert!(!driver.broker_is_attached());
}

#[test]
fn opaque_reference_mapping_cannot_be_rebound_to_another_vault_record() {
    let vault = CredentialVault::session_only().expect("session-only vault");
    let first = vault.store("password", SECRET.as_bytes()).expect("first");
    let second = vault
        .store("password", b"another-password-value-0123456789")
        .expect("second");
    let resolver = VaultCredentialResolver::new(vault);
    let reference = random_reference("stable-random-reference");
    resolver
        .register(&reference, first.clone())
        .expect("first registration");
    resolver
        .register(&reference, first)
        .expect("idempotent registration");
    assert!(resolver.register(&reference, second).is_err());
    assert!(!format!("{resolver:?}").contains(SECRET));
}

#[test]
#[ignore = "bundle tier: set C4OS_BUNDLED_OPENCODE_SDK after creating the debug app bundle"]
fn bundled_opencode_sdk_tree_matches_runtime_pins() {
    let configured = std::env::var_os("C4OS_BUNDLED_OPENCODE_SDK")
        .map(PathBuf::from)
        .expect("bundled sidecar path");
    let bundled = if configured.is_absolute() {
        configured
    } else {
        project_root().join(configured)
    };
    let source_receipt =
        OpenCodeSdkIntegrity::verify(&project_root().join("sidecars/opencode-sdk"))
            .expect("source SDK integrity");
    assert_eq!(
        dependency_tree_sha256(&bundled.join("node_modules")).expect("bundled dependency digest"),
        source_receipt.dependency_tree_sha256(),
    );
    let receipt = OpenCodeSdkIntegrity::verify(&bundled).expect("bundled SDK integrity");
    assert_eq!(receipt.sidecar_root(), bundled.canonicalize().unwrap());
}

#[test]
#[ignore = "native tier: requires local exact OpenCode 1.18.3 and Node artifacts"]
fn live_exact_opencode_port_collision_fails_before_auth_and_cleans_process_group() {
    let project_root = project_root();
    let opencode = exact_opencode_binary();
    let node = find_executable_on_path("node").expect("Node executable on PATH");
    let temporary = TempDir::new().expect("collision temporary root");
    let workspace = temporary.path().join("workspace");
    fs::create_dir(&workspace).expect("workspace root");
    let (listener, endpoint) = reserve_loopback();
    let squatter = listener.try_clone().expect("duplicate port reservation");
    squatter
        .set_nonblocking(true)
        .expect("nonblocking collision observer");
    let reference = random_reference("collision-native-password");
    let resolver = resolver_with_secret(&reference, SECRET.as_bytes());
    let namespace = StateNamespace::new(
        temporary.path(),
        "workspace-native-collision",
        61,
        "launch-native-collision",
    )
    .expect("collision namespace");
    let plan = OpenCodeLaunchPlan {
        manifest: OpenCodeCompatibilityManifest::pinned(
            sha256_file(&opencode).expect("OpenCode digest"),
        )
        .expect("exact manifest"),
        endpoint: endpoint.clone(),
        namespace,
        executable: opencode,
        workspace_root: workspace,
        basic_auth_username: "opencode".into(),
        password_reference: reference,
        secret_channel_fd: 198,
        authority_policy: NativeAuthorityPolicy::new(
            opencode_authority_configuration_sha256(),
            OPENCODE_C4OS_TOOL_IDS,
        )
        .expect("authority policy"),
    };
    let command = plan.command().expect("collision launch command");
    let prepared = OpenCodeProductionAssetFactory::new(&project_root)
        .expect("production asset factory")
        .construct_driver(
            &node,
            &sha256_file(&node).expect("Node digest"),
            resolver,
            61,
            Duration::from_secs(30),
            Duration::from_secs(3),
        )
        .expect("factory-pinned collision driver");
    let (_, mut driver) = prepared.into_parts();
    driver
        .install_loopback_reservation(listener)
        .expect("install collision reservation");

    assert_eq!(
        driver.spawn(&command),
        Err(CommandFailureCode::SpawnRejected)
    );
    assert!(!driver.broker_is_attached());
    assert!(matches!(
        squatter.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
    drop(squatter);
    let rebound = TcpListener::bind(SocketAddr::new(endpoint.address(), endpoint.port()))
        .expect("failed collision launch must release its process-group listener");
    drop(rebound);
}

#[test]
#[ignore = "native tier: requires local exact OpenCode 1.18.3 and Node artifacts"]
fn live_exact_opencode_launch_authentication_and_process_group_cleanup() {
    let project_root = project_root();
    let opencode = exact_opencode_binary();
    let node = find_executable_on_path("node").expect("Node executable on PATH");
    let temporary = TempDir::new().expect("native temporary root");
    let workspace = temporary.path().join("workspace");
    fs::create_dir(&workspace).expect("workspace root");
    let (listener, endpoint) = reserve_loopback();
    assert!(
        TcpListener::bind(SocketAddr::new(endpoint.address(), endpoint.port())).is_err(),
        "reserved endpoint must remain continuously bound before spawn"
    );
    let reference = random_reference("live-native-password");
    let vault = CredentialVault::session_only().expect("session-only native vault");
    let server_credential = vault
        .store("opencode-server-password", SECRET.as_bytes())
        .expect("server password");
    let provider_credential = vault
        .store("provider-api-key", PROVIDER_SECRET.as_bytes())
        .expect("provider credential");
    let resolver = VaultCredentialResolver::new(vault);
    resolver
        .register(&reference, server_credential)
        .expect("server credential reference");
    let executable_sha256 = sha256_file(&opencode).expect("OpenCode digest");
    let namespace = StateNamespace::new(temporary.path(), "workspace-native", 41, "launch-native")
        .expect("native namespace");
    let plan = OpenCodeLaunchPlan {
        manifest: OpenCodeCompatibilityManifest::pinned(executable_sha256).expect("exact manifest"),
        endpoint: endpoint.clone(),
        namespace,
        executable: opencode,
        workspace_root: workspace,
        basic_auth_username: "opencode".into(),
        password_reference: reference.clone(),
        secret_channel_fd: 198,
        authority_policy: NativeAuthorityPolicy::new(
            opencode_authority_configuration_sha256(),
            OPENCODE_C4OS_TOOL_IDS,
        )
        .expect("authority policy"),
    };
    let command = plan.command().expect("launch command");
    assert!(!format!("{command:?}").contains(SECRET));
    assert!(!format!("{command:?}").contains(PROVIDER_SECRET));

    let prepared = OpenCodeProductionAssetFactory::new(&project_root)
        .expect("production asset factory")
        .construct_driver(
            &node,
            &sha256_file(&node).expect("Node digest"),
            resolver.clone(),
            41,
            Duration::from_secs(30),
            Duration::from_secs(3),
        )
        .expect("factory-pinned native driver");
    assert_eq!(prepared.assets().native_executable(), plan.executable);
    let (_, mut driver) = prepared.into_parts();
    driver
        .install_loopback_reservation(listener)
        .expect("install continuously held listener");
    driver
        .register_provider(&ProviderProfile {
            schema_version: PROVIDER_SCHEMA_VERSION,
            provider_id: "provider-openai".into(),
            kind: ProviderKind::OpenAi,
            display_name: "OpenAI".into(),
            endpoint: ProviderEndpoint {
                endpoint_id: "openai-default".into(),
                base_url: "https://api.openai.com/v1".into(),
                api_kind: "openai".into(),
            },
            credential_reference: provider_credential,
            enabled: true,
        })
        .expect("core-owned provider credential route");

    let isolated_config = PathBuf::from(
        command
            .environment
            .get("XDG_CONFIG_HOME")
            .expect("isolated config path"),
    );
    let namespace_root = isolated_config.parent().expect("namespace root");
    fs::create_dir_all(namespace_root).expect("namespace fixture");
    let redirect = temporary.path().join("redirect-outside-generation");
    fs::create_dir(&redirect).expect("redirect target");
    std::os::unix::fs::symlink(&redirect, &isolated_config).expect("hostile XDG symlink");
    assert_eq!(
        driver.spawn(&command),
        Err(CommandFailureCode::SpawnRejected),
        "an XDG child symlink must not redirect runtime state"
    );
    fs::remove_file(&isolated_config).expect("remove hostile symlink");

    let process = driver.spawn(&command).expect("live OpenCode launch");
    assert_eq!(process.process_generation, 41);
    assert!(driver.broker_is_attached());

    let mut transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        resolver,
        Duration::from_secs(2),
        Duration::from_secs(5),
    )
    .expect("native transport");
    let health = transport
        .execute(request(&endpoint, &reference))
        .expect("authenticated native health");
    assert_eq!(health.status, 200);
    let health: Value = serde_json::from_slice(&health.body).expect("health JSON");
    assert_eq!(health.get("healthy").and_then(Value::as_bool), Some(true));
    assert_eq!(
        health.get("version").and_then(Value::as_str),
        Some("1.18.3")
    );

    let native_config = transport
        .execute(request_path(&endpoint, &reference, "/config"))
        .expect("authenticated native config");
    assert_eq!(native_config.status, 200);
    let native_config: Value =
        serde_json::from_slice(&native_config.body).expect("native config JSON");
    let permissions = native_config
        .get("permission")
        .and_then(Value::as_object)
        .expect("native permission map");
    assert_eq!(permissions.get("*").and_then(Value::as_str), Some("deny"));
    assert_eq!(
        permissions
            .get("c4os_propose_action")
            .and_then(Value::as_str),
        Some("allow")
    );
    assert_eq!(
        permissions
            .get("c4os_read_resource")
            .and_then(Value::as_str),
        Some("allow")
    );
    assert!(!permissions.contains_key("write"));
    assert!(!permissions.contains_key("bash"));

    let registered_tools = transport
        .execute(request_path(
            &endpoint,
            &reference,
            "/experimental/tool/ids",
        ))
        .expect("authenticated generated SDK tool registry");
    assert_eq!(registered_tools.status, 200);
    let registered_tools: Vec<String> =
        serde_json::from_slice(&registered_tools.body).expect("tool registry JSON");
    assert!(
        registered_tools
            .iter()
            .any(|tool| tool == "c4os_propose_action")
    );
    assert!(
        registered_tools
            .iter()
            .any(|tool| tool == "c4os_read_resource")
    );
    let mut c4os_tools = registered_tools
        .iter()
        .map(String::as_str)
        .filter(|tool| tool.starts_with("c4os_"))
        .collect::<Vec<_>>();
    c4os_tools.sort_unstable();
    assert_eq!(c4os_tools, OPENCODE_C4OS_TOOL_IDS);

    driver
        .authorize_provider_credential_attempt(ProviderCredentialRequest {
            process_generation: 41,
            native_session_id: "native-session-credential-scan".into(),
            provider_id: "provider-openai".into(),
            model_id: "gpt-5".into(),
            operation_id: "attempt-credential-scan".into(),
        })
        .expect("provider attempt authorization");
    thread::sleep(Duration::from_millis(50));

    let process_surfaces = process_group_surfaces(process.process_id);
    assert!(
        !process_surfaces.contains(SECRET),
        "server credential reached runtime-group argv or inherited environment"
    );
    assert!(
        !process_surfaces.contains(PROVIDER_SECRET),
        "provider credential reached runtime-group argv or inherited environment"
    );
    assert!(
        tree_is_absent_of(namespace_root, SECRET.as_bytes()),
        "server credential reached native state, auth.json, config, cache, or logs"
    );
    assert!(
        tree_is_absent_of(namespace_root, PROVIDER_SECRET.as_bytes()),
        "provider credential reached native state, auth.json, config, cache, or logs"
    );

    let wrong_reference = random_reference("wrong-native-password");
    let wrong_resolver =
        resolver_with_secret(&wrong_reference, b"wrong-native-password-0123456789abcdef");
    let mut wrong_transport = LoopbackHttpTransport::new(
        endpoint.clone(),
        wrong_resolver,
        Duration::from_secs(2),
        Duration::from_secs(2),
    )
    .expect("wrong-auth transport");
    assert_eq!(
        wrong_transport.execute(request(&endpoint, &wrong_reference)),
        Err(TransportFailureCode::AuthenticationRejected)
    );

    driver
        .terminate_process_group(&process)
        .expect("process-group cleanup");
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline
        && TcpStream::connect_timeout(
            &SocketAddr::new(endpoint.address(), endpoint.port()),
            Duration::from_millis(50),
        )
        .is_ok()
    {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        TcpStream::connect_timeout(
            &SocketAddr::new(endpoint.address(), endpoint.port()),
            Duration::from_millis(50),
        )
        .is_err(),
        "the exact native descendant listener survived process-group cleanup"
    );
}

#[test]
#[ignore = "native tier: requires local exact OpenCode 1.18.3 and Node artifacts"]
fn live_exact_opencode_chat_headers_authenticates_a_loopback_provider_without_persistence() {
    let project_root = project_root();
    let opencode = exact_opencode_binary();
    let node = find_executable_on_path("node").expect("Node executable on PATH");
    let temporary = TempDir::new().expect("credentialed native temporary root");
    let workspace = temporary.path().join("workspace");
    fs::create_dir(&workspace).expect("workspace root");
    let provider_listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind provider fixture");
    let provider_address = provider_listener.local_addr().expect("provider address");
    let provider_server = serve_openai_compatible_once(provider_listener);
    let (listener, endpoint) = reserve_loopback();
    let password_reference = random_reference("live-credentialed-native-password");
    let vault = CredentialVault::session_only().expect("credentialed native vault");
    let server_credential = vault
        .store("opencode-server-password", SECRET.as_bytes())
        .expect("server password");
    let provider_credential = vault
        .store("provider-api-key", PROVIDER_SECRET.as_bytes())
        .expect("provider API key");
    let resolver = VaultCredentialResolver::new(vault);
    resolver
        .register(&password_reference, server_credential)
        .expect("server credential mapping");
    let namespace = StateNamespace::new(
        temporary.path(),
        "workspace-native-credentialed",
        51,
        "launch-native-credentialed",
    )
    .expect("credentialed namespace");
    let plan = OpenCodeLaunchPlan {
        manifest: OpenCodeCompatibilityManifest::pinned(
            sha256_file(&opencode).expect("OpenCode digest"),
        )
        .expect("exact manifest"),
        endpoint: endpoint.clone(),
        namespace,
        executable: opencode,
        workspace_root: workspace,
        basic_auth_username: "opencode".into(),
        password_reference: password_reference.clone(),
        secret_channel_fd: 198,
        authority_policy: NativeAuthorityPolicy::new(
            opencode_authority_configuration_sha256(),
            OPENCODE_C4OS_TOOL_IDS,
        )
        .expect("authority policy"),
    };
    let command = plan.command().expect("credentialed launch command");
    let config_home = PathBuf::from(&command.environment["XDG_CONFIG_HOME"]);
    let opencode_config_home = config_home.join("opencode");
    fs::create_dir_all(&opencode_config_home).expect("native config directory");
    fs::set_permissions(&opencode_config_home, fs::Permissions::from_mode(0o700))
        .expect("native config permissions");
    let provider_config = json!({
        "provider": {
            "c4os-fixture": {
                "npm": "@ai-sdk/openai-compatible",
                "name": "C4OS loopback fixture",
                "options": {
                    "baseURL": format!("http://{provider_address}/v1")
                },
                "models": {
                    "fixture-model": {
                        "name": "C4OS fixture model",
                        "limit": {
                            "context": 128000,
                            "output": 4096
                        }
                    }
                }
            }
        }
    });
    let provider_config_path = opencode_config_home.join("opencode.json");
    fs::write(
        &provider_config_path,
        serde_json::to_vec(&provider_config).unwrap(),
    )
    .expect("provider-only native config");
    fs::set_permissions(&provider_config_path, fs::Permissions::from_mode(0o600))
        .expect("provider config permissions");

    let prepared = OpenCodeProductionAssetFactory::new(&project_root)
        .expect("production asset factory")
        .construct_driver(
            &node,
            &sha256_file(&node).expect("Node digest"),
            resolver.clone(),
            51,
            Duration::from_secs(30),
            Duration::from_secs(3),
        )
        .expect("factory-pinned credentialed driver");
    let (_, mut driver) = prepared.into_parts();
    driver
        .install_loopback_reservation(listener)
        .expect("install continuously held listener");
    let transport = LoopbackHttpTransport::new(
        endpoint,
        resolver,
        Duration::from_secs(2),
        Duration::from_secs(5),
    )
    .expect("credentialed native transport");
    let mut adapter = OpenCodeAdapter::new(plan, transport, driver).expect("credentialed adapter");
    adapter
        .register_provider_credential_route(&ProviderProfile {
            schema_version: PROVIDER_SCHEMA_VERSION,
            provider_id: "c4os-fixture".into(),
            kind: ProviderKind::Custom,
            display_name: "C4OS Fixture".into(),
            endpoint: ProviderEndpoint {
                endpoint_id: "fixture-default".into(),
                // Product state remains HTTPS-only. The local HTTP URL exists
                // only in the isolated exact-native test configuration above.
                base_url: "https://fixture.invalid/v1".into(),
                api_kind: "openai-compatible".into(),
            },
            credential_reference: provider_credential,
            enabled: true,
        })
        .expect("core-owned credential route");
    adapter
        .start(1_000)
        .expect("credentialed exact native start");
    assert!(
        adapter
            .list_models()
            .expect("credentialed provider inventory")
            .iter()
            .any(|model| {
                model.provider_id == "c4os-fixture" && model.model_id == "fixture-model"
            }),
        "the exact native provider configuration did not expose the fixture model"
    );
    let binding = adapter
        .create_session("chat-credentialed", "Credentialed native test")
        .expect("credentialed native session");
    adapter
        .send_with_provider_credential(PromptDispatch {
            correlation: EventCorrelation {
                workspace_id: "workspace-native-credentialed".into(),
                c4os_session_id: "chat-credentialed".into(),
                c4os_turn_id: "turn-credentialed".into(),
                c4os_run_id: "run-credentialed".into(),
                correlation_id: "correlation-credentialed".into(),
                native_session_id: binding.native_session_id,
                process_generation: 51,
            },
            model: ModelRoute {
                provider_id: "c4os-fixture".into(),
                model_id: "c4os-fixture/fixture-model".into(),
            },
            text: "Return the fixture response.".into(),
            eligible_tool_ids: BTreeSet::from(["c4os_propose_action".into()]),
            attachments: Vec::new(),
            native_overrides: json!({}),
        })
        .expect("credentialed exact-native dispatch");

    let mut provider_request = provider_server
        .join()
        .expect("provider fixture thread")
        .expect("provider request");
    {
        let provider_request_text = String::from_utf8_lossy(&provider_request);
        assert!(
            provider_request_text.starts_with("POST /v1/chat/completions HTTP/1.1\r\n")
                || provider_request_text.starts_with("POST /chat/completions HTTP/1.1\r\n"),
            "the exact native provider used an unexpected request path"
        );
        assert!(
            provider_request_text.contains(&format!("Authorization: Bearer {PROVIDER_SECRET}"))
                || provider_request_text
                    .contains(&format!("authorization: Bearer {PROVIDER_SECRET}")),
            "the exact chat.headers hook did not authenticate the provider request"
        );
        assert!(provider_request_text.contains("fixture-model"));
    }
    provider_request.fill(0);

    let process_id = adapter.process_id().expect("exact native process ID");
    let process_surfaces = process_group_surfaces(process_id);
    assert!(!process_surfaces.contains(SECRET));
    assert!(!process_surfaces.contains(PROVIDER_SECRET));
    let namespace_root = config_home.parent().expect("credentialed namespace root");
    assert!(tree_is_absent_of(namespace_root, SECRET.as_bytes()));
    assert!(tree_is_absent_of(
        namespace_root,
        PROVIDER_SECRET.as_bytes()
    ));
    adapter.stop().expect("credentialed native cleanup");
}

fn launch_command_fixture(
    root: &Path,
    executable: PathBuf,
    executable_sha256: String,
    reference: RandomSecretReference,
    descriptor: u32,
) -> c4os_lib::runtime::opencode::LaunchCommand {
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).expect("fixture workspace");
    let mut environment = BTreeMap::new();
    environment.insert(
        "XDG_CONFIG_HOME".into(),
        root.join("config").display().to_string(),
    );
    environment.insert(
        "XDG_DATA_HOME".into(),
        root.join("data").display().to_string(),
    );
    environment.insert(
        "XDG_CACHE_HOME".into(),
        root.join("cache").display().to_string(),
    );
    environment.insert("TMPDIR".into(), root.join("tmp").display().to_string());
    environment.insert("NO_PROXY".into(), "127.0.0.1,::1,localhost".into());
    c4os_lib::runtime::opencode::LaunchCommand {
        executable,
        expected_binary_sha256: executable_sha256,
        arguments: vec![
            "serve".into(),
            "--hostname".into(),
            "127.0.0.1".into(),
            "--port".into(),
            "49193".into(),
        ],
        environment,
        working_directory: workspace,
        secret_channel: c4os_lib::runtime::opencode::SecretChannel {
            inherited_fd: descriptor,
            reference,
        },
        authority_policy: NativeAuthorityPolicy::new(
            opencode_authority_configuration_sha256(),
            OPENCODE_C4OS_TOOL_IDS,
        )
        .expect("authority policy"),
    }
}

fn reserve_loopback() -> (TcpListener, LoopbackEndpoint) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("reserve native listener");
    let address = listener.local_addr().expect("reserved address");
    let endpoint = LoopbackEndpoint::new(address.ip(), address.port()).expect("reserved endpoint");
    (listener, endpoint)
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
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

fn exact_opencode_binary() -> PathBuf {
    project_root()
        .join("sidecars/opencode-native/node_modules/opencode-darwin-arm64/bin/opencode")
        .canonicalize()
        .expect("project-owned production OpenCode 1.18.3 executable")
}

fn serve_openai_compatible_once(
    listener: TcpListener,
) -> thread::JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        listener.set_nonblocking(true)?;
        let deadline = Instant::now() + Duration::from_secs(30);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "OpenCode did not reach the provider fixture",
                        ));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(error),
            }
        };
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut request = Vec::new();
        let mut buffer = [0_u8; 8 * 1024];
        let mut expected = None;
        loop {
            let read = stream.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            if expected.is_none()
                && let Some(header_end) =
                    request.windows(4).position(|window| window == b"\r\n\r\n")
            {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.split_once(':').and_then(|(name, value)| {
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                    })
                    .unwrap_or(0);
                expected = Some(header_end + 4 + content_length);
            }
            if expected.is_some_and(|length| request.len() >= length) {
                break;
            }
        }
        let chunk = json!({
            "id": "chatcmpl-c4os",
            "object": "chat.completion.chunk",
            "created": 1,
            "model": "fixture-model",
            "choices": [{
                "index": 0,
                "delta": { "role": "assistant", "content": "fixture-ok" },
                "finish_reason": "stop"
            }]
        });
        let body = format!("data: {chunk}\n\ndata: [DONE]\n\n");
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )?;
        Ok(request)
    })
}

fn process_group_surfaces(process_group_id: u32) -> String {
    let listing = Command::new("/bin/ps")
        .args(["-ax", "-o", "pid=,pgid="])
        .output()
        .expect("list runtime process group");
    assert!(listing.status.success());
    let mut surfaces = String::new();
    let listing = String::from_utf8_lossy(&listing.stdout);
    let mut matched = 0_usize;
    for line in listing.lines() {
        let mut fields = line.split_whitespace();
        let Some(pid) = fields.next() else {
            continue;
        };
        let Some(pgid) = fields.next() else {
            continue;
        };
        if pgid.parse::<u32>().ok() != Some(process_group_id) {
            continue;
        }
        let output = Command::new("/bin/ps")
            .args(["eww", "-p", pid])
            .output()
            .expect("inspect runtime-group process");
        assert!(output.status.success());
        matched = matched.saturating_add(1);
        surfaces.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    assert!(matched > 0, "exact native process group was not observable");
    surfaces
}

fn tree_is_absent_of(root: &Path, needle: &[u8]) -> bool {
    fn visit(path: &Path, needle: &[u8]) -> bool {
        let Ok(metadata) = fs::symlink_metadata(path) else {
            return false;
        };
        if metadata.file_type().is_symlink() {
            return true;
        }
        if metadata.is_dir() {
            let Ok(entries) = fs::read_dir(path) else {
                return false;
            };
            return entries
                .filter_map(Result::ok)
                .all(|entry| visit(&entry.path(), needle));
        }
        if !metadata.is_file() {
            return true;
        }
        let Ok(bytes) = fs::read(path) else {
            return false;
        };
        !bytes.windows(needle.len()).any(|window| window == needle)
    }

    visit(root, needle)
}

fn base64_for_test(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        encoded.push(char::from(TABLE[usize::from(chunk[0] >> 2)]));
        encoded.push(char::from(
            TABLE[usize::from(((chunk[0] & 3) << 4) | chunk.get(1).copied().unwrap_or(0) >> 4)],
        ));
        encoded.push(if chunk.len() > 1 {
            char::from(
                TABLE
                    [usize::from(((chunk[1] & 15) << 2) | chunk.get(2).copied().unwrap_or(0) >> 6)],
            )
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            char::from(TABLE[usize::from(chunk[2] & 63)])
        } else {
            '='
        });
    }
    encoded
}

#[test]
fn authority_configuration_digest_is_stable_and_sha_prefixed() {
    let digest = opencode_authority_configuration_sha256();
    assert_eq!(digest, sha256_bytes(OPENCODE_AUTHORITY_CONFIG.as_bytes()));
    assert!(digest.starts_with("sha256:"));
    assert_eq!(digest.len(), 71);

    let configuration: Value =
        serde_json::from_str(OPENCODE_AUTHORITY_CONFIG).expect("authority JSON");
    let permissions = configuration
        .get("permission")
        .and_then(Value::as_object)
        .expect("permission map");
    assert_eq!(permissions.get("*").and_then(Value::as_str), Some("deny"));
    assert_eq!(
        permissions
            .get("c4os_propose_action")
            .and_then(Value::as_str),
        Some("allow")
    );
    assert_eq!(
        permissions
            .get("c4os_read_resource")
            .and_then(Value::as_str),
        Some("allow")
    );
    assert!(
        ["bash", "write", "edit", "patch", "external_directory"]
            .iter()
            .all(|tool| !permissions.contains_key(*tool))
    );
}
