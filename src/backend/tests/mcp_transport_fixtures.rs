#![cfg(unix)]

use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    os::unix::process::CommandExt as _,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use c4os_lib::mcp::{
    MCP_PROTOCOL_VERSION, McpError, McpSecretReference, McpTransportDefinition,
    McpWorkingDirectory,
    transport::{McpCancellation, McpTransportClient, ResolvedMcpLaunch},
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const OPERATION_TIMEOUT_MS: u64 = 5_000;
const FIXTURE_SECRET: &str = "fixture-secret-never-persist";

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("build MCP fixture runtime")
}

fn node_executable() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.build/app/c4os-runtime-assets/node")
        .canonicalize()
        .expect("the production-pinned standalone Node runtime must be generated")
}

fn fixture_script(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/mcp")
        .join(name)
}

fn private_scratch(home: &Path) -> PathBuf {
    let scratch = home.join("mcp-worker-scratch");
    fs::create_dir_all(&scratch).unwrap();
    scratch.canonicalize().unwrap()
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap();
    let digest = Sha256::digest(bytes);
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{hex}")
}

fn launch(home: &Path) -> ResolvedMcpLaunch {
    let fixture_root = fixture_script("stdio_server.cjs")
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    ResolvedMcpLaunch {
        sampling_context: None,
        c4os_home: home.canonicalize().unwrap(),
        scratch_root: private_scratch(home),
        active_project: None,
        trusted_roots: vec![fixture_root],
        environment: BTreeMap::new(),
        bearer: None,
        headers: BTreeMap::new(),
    }
}

fn stdio_definition(_home: &Path, audit_path: &Path, scenario: &str) -> McpTransportDefinition {
    let executable = node_executable();
    McpTransportDefinition::Stdio {
        command: executable.to_string_lossy().into_owned(),
        arguments: vec![
            fixture_script("stdio_server.cjs")
                .to_string_lossy()
                .into_owned(),
            "--scenario".into(),
            scenario.into(),
            "--audit-path".into(),
            audit_path.to_string_lossy().into_owned(),
        ],
        environment: Vec::new(),
        working_directory: McpWorkingDirectory::C4osHome,
        executable_sha256: Some(sha256_file(&executable)),
    }
}

async fn wait_for_notification_epoch(client: &McpTransportClient, expected: u64) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while client.notification_epoch() < expected && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(
        client.notification_epoch() >= expected,
        "fixture list-change notifications were not observed"
    );
}

async fn wait_for_audit(path: &Path, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let audit = fs::read_to_string(path).unwrap_or_default();
        if audit.contains(needle) {
            return audit;
        }
        assert!(
            Instant::now() < deadline,
            "audit did not contain {needle:?}: {audit}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[test]
fn stdio_exact_golden_path_is_bounded_cancelable_and_restartable() {
    let temporary = TempDir::new().unwrap();
    let audit_path = private_scratch(temporary.path()).join("stdio-audit.jsonl");
    let definition = stdio_definition(temporary.path(), &audit_path, "normal");

    runtime().block_on(async {
        let client = McpTransportClient::connect(
            &definition,
            launch(temporary.path()),
            OPERATION_TIMEOUT_MS,
        )
        .await
        .unwrap();
        assert_eq!(client.handshake().protocol_version, MCP_PROTOCOL_VERSION);
        assert_eq!(client.handshake().server_name, "c4os-stdio-fixture");
        assert!(client.handshake().instructions_present);
        assert!(client.handshake().capabilities.tools);
        assert!(client.handshake().capabilities.tool_list_changed);
        assert!(client.handshake().capabilities.resources);
        assert!(client.handshake().capabilities.resource_list_changed);
        assert!(client.handshake().capabilities.resource_subscribe);

        let tools = client.list_tools(OPERATION_TIMEOUT_MS).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "fixture_tool");
        assert!(tools[0].input_schema_sha256.starts_with("sha256:"));
        let resources = client.list_resources(OPERATION_TIMEOUT_MS).await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "fixture://resource/one");
        wait_for_notification_epoch(&client, 2).await;

        let echo = client
            .call_tool(
                "fixture_tool",
                json!({"mode": "echo", "value": "golden"}),
                OPERATION_TIMEOUT_MS,
                4 * 1_024,
                McpCancellation::default(),
            )
            .await
            .unwrap();
        assert!(!echo.is_error);
        assert!(echo.value.to_string().contains("golden"));
        assert!(echo.output_bytes < 4 * 1_024);

        let hostile = client
            .call_tool(
                "fixture_tool",
                json!({"mode": "hostile"}),
                OPERATION_TIMEOUT_MS,
                4 * 1_024,
                McpCancellation::default(),
            )
            .await
            .unwrap();
        assert!(hostile.value.to_string().contains("<script>"));
        let peer_error = client
            .call_tool(
                "fixture_tool",
                json!({"mode": "peer-error"}),
                OPERATION_TIMEOUT_MS,
                4 * 1_024,
                McpCancellation::default(),
            )
            .await;
        match peer_error {
            Err(McpError::Transport(detail)) => {
                assert_eq!(detail, "transport operation failed");
                assert!(!detail.contains(FIXTURE_SECRET));
            }
            other => panic!("expected sanitized peer error, got {other:?}"),
        }
        let resource = client
            .read_resource(
                "fixture://resource/one",
                OPERATION_TIMEOUT_MS,
                4 * 1_024,
                McpCancellation::default(),
            )
            .await
            .unwrap();
        assert!(resource.value.to_string().contains("file:///etc/passwd"));

        assert!(matches!(
            client
                .call_tool(
                    "fixture_tool",
                    json!({"mode": "oversized"}),
                    OPERATION_TIMEOUT_MS,
                    1_024,
                    McpCancellation::default(),
                )
                .await,
            Err(McpError::BoundExceeded)
        ));

        let cancellation = McpCancellation::default();
        let cancellation_signal = cancellation.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            cancellation_signal.cancel();
        });
        assert!(matches!(
            client
                .call_tool(
                    "fixture_tool",
                    json!({"mode": "slow"}),
                    OPERATION_TIMEOUT_MS,
                    4 * 1_024,
                    cancellation,
                )
                .await,
            Err(McpError::Cancelled)
        ));
        wait_for_audit(&audit_path, "\"event\":\"cancelled\"").await;

        assert!(matches!(
            client
                .call_tool(
                    "fixture_tool",
                    json!({"mode": "crash"}),
                    OPERATION_TIMEOUT_MS,
                    4 * 1_024,
                    McpCancellation::default(),
                )
                .await,
            Err(McpError::Transport(_))
        ));
        drop(client);

        let restarted = McpTransportClient::connect(
            &definition,
            launch(temporary.path()),
            OPERATION_TIMEOUT_MS,
        )
        .await
        .unwrap();
        assert_eq!(
            restarted
                .list_tools(OPERATION_TIMEOUT_MS)
                .await
                .unwrap()
                .len(),
            1
        );
        restarted.shutdown(OPERATION_TIMEOUT_MS).await.unwrap();

        let audit = wait_for_audit(&audit_path, "\"event\":\"crash\"").await;
        assert_eq!(audit.matches("\"event\":\"started\"").count(), 2);
    });
}

#[test]
fn stdio_rejects_invalid_malformed_and_unreviewed_sampling_peers() {
    let temporary = TempDir::new().unwrap();
    runtime().block_on(async {
        let invalid_audit = private_scratch(temporary.path()).join("stdio-invalid.jsonl");
        let invalid = stdio_definition(temporary.path(), &invalid_audit, "invalid-version");
        assert!(matches!(
            McpTransportClient::connect(&invalid, launch(temporary.path()), OPERATION_TIMEOUT_MS,)
                .await,
            Err(McpError::UnsupportedProtocol)
        ));

        let malformed_audit = private_scratch(temporary.path()).join("stdio-malformed.jsonl");
        let malformed = stdio_definition(temporary.path(), &malformed_audit, "malformed");
        assert!(matches!(
            McpTransportClient::connect(&malformed, launch(temporary.path()), 1_000).await,
            Err(McpError::Transport(_)) | Err(McpError::TimedOut)
        ));

        let sampling_audit = private_scratch(temporary.path()).join("stdio-sampling.jsonl");
        let sampling = stdio_definition(temporary.path(), &sampling_audit, "sampling");
        let client =
            McpTransportClient::connect(&sampling, launch(temporary.path()), OPERATION_TIMEOUT_MS)
                .await
                .unwrap();
        let audit = wait_for_audit(&sampling_audit, "\"event\":\"sampling-response\"").await;
        assert!(audit.contains("\"samplingAdvertised\":false"));
        assert!(audit.contains("\"succeeded\":false"));
        client.shutdown(OPERATION_TIMEOUT_MS).await.unwrap();
    });
}

#[derive(Deserialize)]
struct HttpReady {
    ready: bool,
    port: u16,
}

struct HttpFixture {
    child: Child,
    port: u16,
    audit_path: PathBuf,
}

impl HttpFixture {
    fn start(root: &Path, scenario: &str, bearer: &str, requested_port: u16) -> Self {
        let audit_path = root.join(format!("http-{scenario}-{requested_port}.jsonl"));
        let mut child = Command::new(node_executable())
            .arg(fixture_script("streamable_http_server.cjs"))
            .args(["--scenario", scenario])
            .args(["--audit-path", audit_path.to_str().unwrap()])
            .arg("--port")
            .arg(requested_port.to_string())
            .env_clear()
            .env("MCP_FIXTURE_BEARER", bearer)
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("start Streamable HTTP MCP fixture");
        let stdout = child.stdout.take().expect("fixture readiness pipe");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).expect("read fixture readiness");
        let ready: HttpReady = serde_json::from_str(line.trim()).expect("fixture readiness JSON");
        assert!(ready.ready);
        assert_ne!(ready.port, 0);
        Self {
            child,
            port: ready.port,
            audit_path,
        }
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    fn wait_for_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if self.child.try_wait().unwrap().is_some() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "HTTP fixture did not exit after crash"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for HttpFixture {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn http_definition(url: String) -> McpTransportDefinition {
    McpTransportDefinition::StreamableHttp {
        url,
        bearer: Some(McpSecretReference::Environment {
            variable: "MCP_FIXTURE_TOKEN".into(),
        }),
        headers: Vec::new(),
    }
}

fn http_launch(home: &Path, bearer: &str) -> ResolvedMcpLaunch {
    ResolvedMcpLaunch {
        sampling_context: None,
        c4os_home: home.canonicalize().unwrap(),
        scratch_root: private_scratch(home),
        active_project: None,
        trusted_roots: Vec::new(),
        environment: BTreeMap::new(),
        bearer: Some(bearer.into()),
        headers: BTreeMap::new(),
    }
}

#[test]
#[ignore = "local-loopback tier: requires permission to bind an authenticated MCP fixture"]
fn streamable_http_is_authenticated_stateful_bounded_cancelable_and_restartable() {
    let temporary = TempDir::new().unwrap();
    let mut fixture = HttpFixture::start(temporary.path(), "normal", FIXTURE_SECRET, 0);
    let definition = http_definition(fixture.url());

    runtime().block_on(async {
        let client = McpTransportClient::connect(
            &definition,
            http_launch(temporary.path(), FIXTURE_SECRET),
            OPERATION_TIMEOUT_MS,
        )
        .await
        .unwrap();
        assert_eq!(client.handshake().protocol_version, MCP_PROTOCOL_VERSION);
        assert_eq!(client.handshake().server_name, "c4os-http-fixture");
        assert_eq!(
            client.list_tools(OPERATION_TIMEOUT_MS).await.unwrap().len(),
            1
        );
        assert_eq!(
            client
                .list_resources(OPERATION_TIMEOUT_MS)
                .await
                .unwrap()
                .len(),
            1
        );
        wait_for_notification_epoch(&client, 2).await;

        let hostile = client
            .call_tool(
                "fixture_tool",
                json!({"mode": "hostile"}),
                OPERATION_TIMEOUT_MS,
                4 * 1_024,
                McpCancellation::default(),
            )
            .await
            .unwrap();
        assert!(hostile.value.to_string().contains("IGNORE POLICY"));
        assert!(matches!(
            client
                .call_tool(
                    "fixture_tool",
                    json!({"mode": "oversized"}),
                    OPERATION_TIMEOUT_MS,
                    1_024,
                    McpCancellation::default(),
                )
                .await,
            Err(McpError::BoundExceeded)
        ));

        let cancellation = McpCancellation::default();
        let cancellation_signal = cancellation.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            cancellation_signal.cancel();
        });
        assert!(matches!(
            client
                .call_tool(
                    "fixture_tool",
                    json!({"mode": "slow"}),
                    OPERATION_TIMEOUT_MS,
                    4 * 1_024,
                    cancellation,
                )
                .await,
            Err(McpError::Cancelled)
        ));
        wait_for_audit(&fixture.audit_path, "\"event\":\"cancelled\"").await;

        assert!(matches!(
            client
                .call_tool(
                    "fixture_tool",
                    json!({"mode": "crash"}),
                    OPERATION_TIMEOUT_MS,
                    4 * 1_024,
                    McpCancellation::default(),
                )
                .await,
            Err(McpError::Transport(_))
        ));
        drop(client);
    });
    fixture.wait_for_exit();
    let port = fixture.port;
    let audit_path = fixture.audit_path.clone();
    drop(fixture);

    let restarted = HttpFixture::start(temporary.path(), "normal", FIXTURE_SECRET, port);
    let restarted_definition = http_definition(restarted.url());
    runtime().block_on(async {
        let client = McpTransportClient::connect(
            &restarted_definition,
            http_launch(temporary.path(), FIXTURE_SECRET),
            OPERATION_TIMEOUT_MS,
        )
        .await
        .unwrap();
        client.shutdown(OPERATION_TIMEOUT_MS).await.unwrap();
        let audit = wait_for_audit(&restarted.audit_path, "\"event\":\"session-deleted\"").await;
        assert!(audit.contains("\"accepted\":true"));
        assert!(audit.contains("\"sessionPresent\":true"));
        assert!(audit.contains("\"protocolPresent\":true"));
        assert!(!audit.contains(FIXTURE_SECRET));
    });
    assert!(
        fs::read_to_string(audit_path)
            .unwrap()
            .contains("\"event\":\"crash\"")
    );
}

#[test]
#[ignore = "local-loopback tier: requires permission to bind an authenticated MCP fixture"]
fn streamable_http_rejects_bad_credentials_and_invalid_version() {
    let temporary = TempDir::new().unwrap();
    let denied = HttpFixture::start(temporary.path(), "normal", FIXTURE_SECRET, 0);
    let denied_definition = http_definition(denied.url());
    runtime().block_on(async {
        let error = match McpTransportClient::connect(
            &denied_definition,
            http_launch(temporary.path(), "wrong-secret"),
            OPERATION_TIMEOUT_MS,
        )
        .await
        {
            Err(error) => error,
            Ok(client) => {
                drop(client);
                panic!("wrong HTTP bearer was accepted")
            }
        };
        let safe_error = error.to_string();
        assert!(!safe_error.contains(FIXTURE_SECRET));
        assert!(!safe_error.contains("wrong-secret"));
    });
    let denied_audit = fs::read_to_string(&denied.audit_path).unwrap();
    assert!(denied_audit.contains("\"accepted\":false"));
    assert!(!denied_audit.contains(FIXTURE_SECRET));
    drop(denied);

    let invalid = HttpFixture::start(temporary.path(), "invalid-version", FIXTURE_SECRET, 0);
    let invalid_definition = http_definition(invalid.url());
    runtime().block_on(async {
        assert!(matches!(
            McpTransportClient::connect(
                &invalid_definition,
                http_launch(temporary.path(), FIXTURE_SECRET),
                OPERATION_TIMEOUT_MS,
            )
            .await,
            Err(McpError::UnsupportedProtocol)
        ));
    });
    drop(invalid);

    let invalid_session =
        HttpFixture::start(temporary.path(), "invalid-session", FIXTURE_SECRET, 0);
    let invalid_session_definition = http_definition(invalid_session.url());
    runtime().block_on(async {
        assert!(matches!(
            McpTransportClient::connect(
                &invalid_session_definition,
                http_launch(temporary.path(), FIXTURE_SECRET),
                OPERATION_TIMEOUT_MS,
            )
            .await,
            Err(McpError::Transport(_))
        ));
    });
}
