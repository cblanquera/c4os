use crate::path_resolver::ProjectPathResolver;
use crate::storage::{AppStore, NewToolCall};
use serde_json::{json, Value};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RuntimeLaunchRequest {
    pub session_id: String,
    pub project_id: String,
    pub project_root: PathBuf,
    pub title: String,
}

#[derive(Debug)]
pub struct SupervisedRuntimeProcess {
    pub session_id: String,
    pub pid: u32,
    child: Child,
}

impl SupervisedRuntimeProcess {
    pub fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
pub enum RuntimeLaunchError {
    InvalidProjectRoot,
    MissingProviderState(String),
    SpawnFailed(String),
    StoreFailed(rusqlite::Error),
}

#[derive(Debug)]
pub struct RuntimePromptRequest {
    pub session_id: String,
    pub project_root: PathBuf,
    pub prompt: String,
    pub model_id: String,
    pub openrouter_api_key: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RuntimePromptResult {
    pub assistant_text: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RuntimePromptError {
    InvalidProjectRoot,
    MissingAssistantOutput,
    SpawnFailed(String),
    ExecutionFailed(String),
    StoreFailed(String),
}

pub trait CodingRuntimeRunner {
    fn run_prompt(
        &self,
        app_store: &AppStore,
        request: RuntimePromptRequest,
    ) -> Result<RuntimePromptResult, RuntimePromptError>;
}

pub struct OpenCodeJsonRunner {
    command_path: PathBuf,
    timeout: Duration,
}

impl OpenCodeJsonRunner {
    pub fn default_from_workspace() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir
            .parent()
            .map(PathBuf::from)
            .unwrap_or(manifest_dir);

        Self {
            command_path: workspace_dir.join("node_modules/opencode-ai/bin/opencode.exe"),
            timeout: Duration::from_secs(120),
        }
    }

    pub fn new(command_path: impl Into<PathBuf>) -> Self {
        Self {
            command_path: command_path.into(),
            timeout: Duration::from_secs(120),
        }
    }

    pub fn new_with_timeout(command_path: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            command_path: command_path.into(),
            timeout,
        }
    }
}

impl CodingRuntimeRunner for OpenCodeJsonRunner {
    fn run_prompt(
        &self,
        app_store: &AppStore,
        request: RuntimePromptRequest,
    ) -> Result<RuntimePromptResult, RuntimePromptError> {
        ProjectPathResolver::new(&request.project_root)
            .map_err(|_| RuntimePromptError::InvalidProjectRoot)?;

        let selected_model = openrouter_model_id(&request.model_id);
        let runtime_config = openrouter_runtime_config(&selected_model, &request.model_id);
        let mut command = Command::new(&self.command_path);
        command
            .args([
                "run",
                "--pure",
                "--format",
                "json",
                "--model",
                &selected_model,
                "--dir",
                &request.project_root.to_string_lossy(),
                &request.prompt,
            ])
            .current_dir(&request.project_root)
            .env("OPENROUTER_API_KEY", &request.openrouter_api_key)
            .env("OPENCODE_CONFIG_CONTENT", runtime_config.to_string());

        let output = run_command_with_timeout(&mut command, self.timeout)?;

        ingest_opencode_json_events(app_store, &request.session_id, &output.stdout)?;

        if !output.status_success {
            return Err(RuntimePromptError::ExecutionFailed(bounded_process_error(
                &output.stderr,
                &output.stdout,
            )));
        }

        let assistant_text = assistant_text_from_json_events(&output.stdout);

        if assistant_text.trim().is_empty() {
            return Err(RuntimePromptError::MissingAssistantOutput);
        }

        Ok(RuntimePromptResult { assistant_text })
    }
}

struct TimedCommandOutput {
    status_success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<TimedCommandOutput, RuntimePromptError> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| RuntimePromptError::SpawnFailed(error.to_string()))?;
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let stdout_reader = thread::spawn(move || read_pipe(stdout.take()));
    let stderr_reader = thread::spawn(move || read_pipe(stderr.take()));
    let started_at = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| RuntimePromptError::ExecutionFailed(error.to_string()))?
        {
            break status;
        }

        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();

            return Err(RuntimePromptError::ExecutionFailed(format!(
                "OpenCode runtime timed out after {} seconds.",
                timeout.as_secs().max(1)
            )));
        }

        thread::sleep(Duration::from_millis(50));
    };

    Ok(TimedCommandOutput {
        status_success: status.success(),
        stdout: stdout_reader.join().unwrap_or_default(),
        stderr: stderr_reader.join().unwrap_or_default(),
    })
}

fn read_pipe(pipe: Option<impl Read>) -> Vec<u8> {
    let mut buffer = Vec::new();

    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut buffer);
    }

    buffer
}

pub struct OpenCodeRuntimeLauncher {
    command_path: PathBuf,
    command_args: Vec<String>,
}

impl OpenCodeRuntimeLauncher {
    pub fn new(command_path: impl Into<PathBuf>, command_args: Vec<String>) -> Self {
        Self {
            command_path: command_path.into(),
            command_args,
        }
    }

    pub fn launch(
        &self,
        app_store: &AppStore,
        request: RuntimeLaunchRequest,
    ) -> Result<SupervisedRuntimeProcess, RuntimeLaunchError> {
        ProjectPathResolver::new(&request.project_root)
            .map_err(|_| RuntimeLaunchError::InvalidProjectRoot)?;

        app_store
            .start_openrouter_session(&request.session_id, &request.project_id, &request.title)
            .map_err(|error| RuntimeLaunchError::MissingProviderState(error.to_string()))?;

        let session = app_store
            .get_session(&request.session_id)
            .map_err(RuntimeLaunchError::StoreFailed)?
            .ok_or_else(|| RuntimeLaunchError::MissingProviderState("missing session".into()))?;
        let credential_reference = app_store
            .session_credential_reference(&request.session_id)
            .map_err(RuntimeLaunchError::StoreFailed)?
            .ok_or_else(|| {
                RuntimeLaunchError::MissingProviderState(
                    "missing session credential reference".into(),
                )
            })?;

        let mut command = Command::new(&self.command_path);
        command
            .args(&self.command_args)
            .current_dir(&request.project_root)
            .env("C4OS_SESSION_ID", &request.session_id)
            .env("C4OS_PROJECT_ROOT", &request.project_root)
            .env("C4OS_MODEL_ID", &session.model_id)
            .env("C4OS_CREDENTIAL_REF", credential_reference)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = command
            .spawn()
            .map_err(|error| RuntimeLaunchError::SpawnFailed(error.to_string()))?;
        let pid = child.id();

        app_store
            .record_adapter_ref(
                &format!("session-{}-runtime-process", request.session_id),
                Some(&request.session_id),
                "opencode",
                "process_id",
                &pid.to_string(),
                r#"{"supervisedByApp":true}"#,
            )
            .map_err(RuntimeLaunchError::StoreFailed)?;

        Ok(SupervisedRuntimeProcess {
            session_id: request.session_id,
            pid,
            child,
        })
    }
}

fn openrouter_model_id(model_id: &str) -> String {
    if model_id.starts_with("openrouter/") {
        model_id.into()
    } else {
        format!("openrouter/{model_id}")
    }
}

fn openrouter_runtime_config(full_model_id: &str, model_id: &str) -> Value {
    json!({
        "model": full_model_id,
        "small_model": full_model_id,
        "enabled_providers": ["openrouter"],
        "disabled_providers": ["openai", "anthropic", "google", "gemini", "opencode"],
        "provider": {
            "openrouter": {
                "options": {
                    "apiKey": "{env:OPENROUTER_API_KEY}",
                    "baseURL": "https://openrouter.ai/api/v1",
                    "timeout": 30000,
                },
                "models": {
                    model_id: {
                        "name": model_id,
                        "tool_call": true,
                        "limit": { "context": 8192, "output": 4096 },
                        "cost": { "input": 0, "output": 0 },
                    },
                },
            },
        },
        "instructions": [],
        "permission": {
            "read": "allow",
            "list": "allow",
            "grep": "allow",
            "glob": "allow",
            "bash": "deny",
            "edit": "deny",
            "webfetch": "deny",
            "external_directory": "deny",
        },
    })
}

fn ingest_opencode_json_events(
    app_store: &AppStore,
    session_id: &str,
    stdout: &[u8],
) -> Result<(), RuntimePromptError> {
    for event in json_lines(stdout) {
        let Some(tool_event) = tool_event_record(&event) else {
            continue;
        };

        app_store
            .record_tool_call(NewToolCall {
                id: &format!(
                    "opencode-tool-{}-{}",
                    tool_event.tool_call_id, tool_event.tool_name
                ),
                session_id,
                message_id: None,
                tool_source: "opencode",
                tool_name: &tool_event.tool_name,
                arguments_json: &tool_event.arguments_json,
                status: &tool_event.status,
                runtime_call_ref: Some(&tool_event.tool_call_id),
            })
            .map_err(|error| RuntimePromptError::StoreFailed(error.to_string()))?;
    }

    Ok(())
}

struct ToolEventRecord {
    tool_name: String,
    tool_call_id: String,
    arguments_json: String,
    status: String,
}

fn tool_event_record(event: &Value) -> Option<ToolEventRecord> {
    match event.get("type").and_then(Value::as_str)? {
        "command.executed" => {
            let tool_name = event.pointer("/properties/name")?.as_str()?.to_string();
            let tool_call_id = event
                .pointer("/properties/messageID")
                .and_then(Value::as_str)
                .unwrap_or(&tool_name)
                .to_string();
            let arguments_json = event
                .pointer("/properties/arguments")
                .and_then(Value::as_str)
                .unwrap_or("{}")
                .to_string();

            Some(ToolEventRecord {
                tool_name,
                tool_call_id,
                arguments_json,
                status: "complete".into(),
            })
        }
        "tool_use" => {
            let tool_name = event.pointer("/part/tool")?.as_str()?.to_string();
            let tool_call_id = event
                .pointer("/part/callID")
                .and_then(Value::as_str)
                .unwrap_or(&tool_name)
                .to_string();
            let status = normalize_tool_status(
                event
                    .pointer("/part/state/status")
                    .and_then(Value::as_str)
                    .unwrap_or("running"),
            );
            let arguments_json = event
                .pointer("/part/state/input")
                .map(Value::to_string)
                .unwrap_or_else(|| "{}".into());

            Some(ToolEventRecord {
                tool_name,
                tool_call_id,
                arguments_json,
                status,
            })
        }
        _ => None,
    }
}

fn normalize_tool_status(status: &str) -> String {
    match status {
        "completed" => "complete".into(),
        "error" => "failed".into(),
        "pending" => "pending".into(),
        _ => "running".into(),
    }
}

fn assistant_text_from_json_events(stdout: &[u8]) -> String {
    let mut text = String::new();
    let mut latest_message_text = None;

    for event in json_lines(stdout) {
        if let Some(delta) = event
            .get("type")
            .and_then(Value::as_str)
            .filter(|event_type| *event_type == "message.part.delta")
            .and_then(|_| event.pointer("/properties/field"))
            .and_then(Value::as_str)
            .filter(|field| *field == "text")
            .and_then(|_| event.pointer("/properties/delta"))
            .and_then(Value::as_str)
        {
            text.push_str(delta);
        }

        if event
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|event_type| event_type == "message.updated")
            && event
                .pointer("/properties/info/role")
                .and_then(Value::as_str)
                .is_some_and(|role| role == "assistant")
        {
            latest_message_text = assistant_parts_text(event.pointer("/properties/parts"));
        }
    }

    if text.trim().is_empty() {
        latest_message_text.unwrap_or_default()
    } else {
        text
    }
}

fn assistant_parts_text(parts: Option<&Value>) -> Option<String> {
    let text = parts?
        .as_array()?
        .iter()
        .filter(|part| {
            part.get("type")
                .and_then(Value::as_str)
                .is_some_and(|part_type| part_type == "text")
        })
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");

    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

fn json_lines(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn bounded_process_error(stderr: &[u8], stdout: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let stderr = stderr.trim();

    if !stderr.is_empty() {
        return stderr.chars().take(800).collect();
    }

    if let Some(error) = opencode_error_message(stdout) {
        return error;
    }

    if let Some(summary) = last_json_event_summary(stdout) {
        return format!(
            "OpenCode exited before returning assistant output. Last event: {summary}."
        );
    }

    let stdout = String::from_utf8_lossy(stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        "OpenCode exited without assistant output.".into()
    } else {
        trimmed.chars().take(800).collect()
    }
}

fn opencode_error_message(stdout: &[u8]) -> Option<String> {
    json_lines(stdout).into_iter().find_map(|event| {
        match event.get("type").and_then(Value::as_str)? {
            "session.error" | "error" => event_error_message(&event).or_else(|| {
                Some(format!(
                    "OpenCode emitted an error event without a message: {}",
                    bounded_json_event(&event)
                ))
            }),
            _ => None,
        }
    })
}

fn event_error_message(event: &Value) -> Option<String> {
    let candidates = [
        event.get("error"),
        event.get("message"),
        event.get("reason"),
        event.pointer("/properties/error"),
        event.pointer("/properties/message"),
        event.pointer("/properties/reason"),
        event.pointer("/properties/data"),
        event.get("data"),
        event.get("properties"),
    ];
    let error = candidates.into_iter().flatten().next()?;

    if let Some(message) = error.as_str().filter(|message| !message.trim().is_empty()) {
        return Some(message.to_string());
    }

    let message = error_message_value(error)?;
    let name = error.get("name").and_then(Value::as_str);

    Some(match name {
        Some(name) if !name.trim().is_empty() => format!("{name}: {message}"),
        _ => message.to_string(),
    })
}

fn error_message_value(value: &Value) -> Option<&str> {
    if let Some(message) = value
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| value.get("text").and_then(Value::as_str))
        .or_else(|| value.get("reason").and_then(Value::as_str))
        .filter(|message| !message.trim().is_empty())
    {
        return Some(message);
    }

    match value {
        Value::Array(items) => items.iter().find_map(error_message_value),
        Value::Object(map) => map.values().find_map(error_message_value),
        _ => None,
    }
}

fn bounded_json_event(event: &Value) -> String {
    event.to_string().chars().take(800).collect()
}

fn last_json_event_summary(stdout: &[u8]) -> Option<String> {
    json_lines(stdout).into_iter().rev().find_map(|event| {
        match event.get("type").and_then(Value::as_str)? {
            "tool_use" => {
                let tool = event.pointer("/part/tool").and_then(Value::as_str)?;
                let status = event
                    .pointer("/part/state/status")
                    .and_then(Value::as_str)
                    .unwrap_or("updated");

                Some(format!("{tool} {status}"))
            }
            "step_start" => Some("step started".into()),
            event_type => Some(event_type.replace('_', " ")),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use crate::storage::NewProject;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn extracts_assistant_text_from_opencode_json_delta_events() {
        let stdout = br#"
{"type":"message.part.delta","properties":{"field":"text","delta":"hello "}}
{"type":"message.part.delta","properties":{"field":"text","delta":"world"}}
"#;

        let text = assistant_text_from_json_events(stdout);

        assert_eq!(text, "hello world");
    }

    #[test]
    fn extracts_assistant_text_from_message_updated_parts_when_no_delta_exists() {
        let stdout = br#"
{"type":"message.updated","properties":{"info":{"role":"assistant"},"parts":[{"type":"text","text":"complete answer"}]}}
"#;

        let text = assistant_text_from_json_events(stdout);

        assert_eq!(text, "complete answer");
    }

    #[test]
    fn ingests_command_executed_events_as_tool_records() {
        let store = prepared_store_with_session();
        let stdout = br#"
{"type":"command.executed","properties":{"name":"read","messageID":"message-1","arguments":"{\"path\":\"README.md\"}"}}
"#;

        ingest_opencode_json_events(&store, "session-1", stdout).expect("events ingested");

        assert_eq!(store.tool_call_count("session-1").expect("tool count"), 1);
    }

    #[test]
    fn ingests_tool_use_events_as_tool_records() {
        let store = prepared_store_with_session();
        let stdout = br#"
{"type":"tool_use","timestamp":1781168186606,"sessionID":"ses_1","part":{"type":"tool","tool":"glob","callID":"tool_glob_1","state":{"status":"completed","input":{"pattern":"*"},"output":"/tmp/src/App.jsx\n/tmp/src/main.jsx"}}}
"#;

        ingest_opencode_json_events(&store, "session-1", stdout).expect("events ingested");
        let timeline = store
            .list_tool_timeline("session-1")
            .expect("timeline records");

        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].tool_name, "glob");
        assert_eq!(timeline[0].status, "complete");
    }

    #[test]
    fn openrouter_config_uses_selected_model_as_model_key() {
        let config = openrouter_runtime_config("openrouter/openai/gpt-4.1", "openai/gpt-4.1");

        assert_eq!(
            config
                .pointer("/provider/openrouter/models/openai~1gpt-4.1/name")
                .and_then(Value::as_str),
            Some("openai/gpt-4.1")
        );
    }

    #[test]
    fn openrouter_config_allows_passive_tools_and_denies_mutation_tools() {
        let config = openrouter_runtime_config("openrouter/openai/gpt-4.1", "openai/gpt-4.1");

        assert_eq!(
            config.pointer("/permission/read").and_then(Value::as_str),
            Some("allow")
        );
        assert_eq!(
            config.pointer("/permission/list").and_then(Value::as_str),
            Some("allow")
        );
        assert_eq!(
            config.pointer("/permission/grep").and_then(Value::as_str),
            Some("allow")
        );
        assert_eq!(
            config.pointer("/permission/glob").and_then(Value::as_str),
            Some("allow")
        );
        assert_eq!(
            config.pointer("/permission/bash").and_then(Value::as_str),
            Some("deny")
        );
        assert_eq!(
            config.pointer("/permission/edit").and_then(Value::as_str),
            Some("deny")
        );
    }

    #[test]
    fn opencode_prompt_runner_times_out_when_process_does_not_exit() {
        let directory = tempdir().expect("tempdir");
        let script_path = directory.path().join("slow-opencode");
        fs::write(&script_path, "#!/bin/sh\nsleep 2\n").expect("script written");
        let mut permissions = fs::metadata(&script_path)
            .expect("script metadata")
            .permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(0o755);
            fs::set_permissions(&script_path, permissions).expect("script executable");
        }
        let store = prepared_store_with_session();
        let runner = OpenCodeJsonRunner::new_with_timeout(&script_path, Duration::from_millis(100));

        let result = runner.run_prompt(
            &store,
            RuntimePromptRequest {
                session_id: "session-1".into(),
                project_root: directory.path().to_path_buf(),
                prompt: "-c 'sleep 2'".into(),
                model_id: "openai/gpt-4.1".into(),
                openrouter_api_key: "sk-or-test-secret".into(),
            },
        );

        assert!(
            matches!(result, Err(RuntimePromptError::ExecutionFailed(message)) if message.contains("timed out"))
        );
    }

    #[test]
    fn process_error_prefers_stderr_over_json_stdout() {
        let stderr = b"OpenRouter returned 401 Unauthorized";
        let stdout = br#"
{"type":"tool_use","part":{"type":"tool","tool":"glob","callID":"tool_1","state":{"status":"completed","input":{"pattern":"*"},"output":"/tmp/src/App.jsx"}}}
"#;

        let message = bounded_process_error(stderr, stdout);

        assert_eq!(message, "OpenRouter returned 401 Unauthorized");
    }

    #[test]
    fn process_error_extracts_session_error_from_json_events() {
        let stdout = br#"
{"type":"step_start","part":{"type":"step-start"}}
{"type":"session.error","properties":{"error":{"name":"ProviderError","message":"No endpoints found that support tool use."}}}
"#;

        let message = bounded_process_error(b"", stdout);

        assert_eq!(
            message,
            "ProviderError: No endpoints found that support tool use."
        );
    }

    #[test]
    fn process_error_extracts_top_level_error_event_message() {
        let stdout = br#"
{"type":"step_start","part":{"type":"step-start"}}
{"type":"error","error":{"name":"ProviderError","message":"OpenRouter rejected the selected model route."}}
"#;

        let message = bounded_process_error(b"", stdout);

        assert_eq!(
            message,
            "ProviderError: OpenRouter rejected the selected model route."
        );
    }

    #[test]
    fn process_error_extracts_error_event_properties_message() {
        let stdout = br#"
{"type":"error","properties":{"message":"Model does not support tool use."}}
"#;

        let message = bounded_process_error(b"", stdout);

        assert_eq!(message, "Model does not support tool use.");
    }

    #[test]
    fn process_error_extracts_nested_error_event_payload() {
        let stdout = br#"
{"type":"error","properties":{"data":{"cause":{"message":"OpenRouter route timed out before first token."}}}}
"#;

        let message = bounded_process_error(b"", stdout);

        assert_eq!(message, "OpenRouter route timed out before first token.");
    }

    #[test]
    fn process_error_includes_bounded_error_event_when_message_is_missing() {
        let stdout = br#"
{"type":"error","properties":{"code":"MODEL_ROUTE_FAILED","details":{"provider":"openrouter"}}}
"#;

        let message = bounded_process_error(b"", stdout);

        assert!(message.starts_with("OpenCode emitted an error event without a message: "));
        assert!(message.contains("MODEL_ROUTE_FAILED"));
    }

    #[test]
    fn process_error_summarizes_json_events_without_dumping_raw_payload() {
        let stdout = br#"
{"type":"step_start","part":{"type":"step-start"}}
{"type":"tool_use","part":{"type":"tool","tool":"glob","callID":"tool_1","state":{"status":"completed","input":{"pattern":"*"},"output":"/tmp/src/App.jsx"}}}
"#;

        let message = bounded_process_error(b"", stdout);

        assert_eq!(
            message,
            "OpenCode exited before returning assistant output. Last event: glob completed."
        );
        assert!(!message.contains(r#""type":"tool_use""#));
    }

    #[test]
    fn launches_supervised_runtime_for_registered_project_and_provider() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");

        let launcher = OpenCodeRuntimeLauncher::new("/bin/sh", vec!["-c".into(), "sleep 2".into()]);
        let mut process = launcher
            .launch(
                &store,
                RuntimeLaunchRequest {
                    session_id: "session-1".into(),
                    project_id: "project-1".into(),
                    project_root: directory.path().to_path_buf(),
                    title: "Run".into(),
                },
            )
            .expect("runtime launched");

        assert!(process.pid > 0);
        assert_eq!(
            store
                .get_session("session-1")
                .expect("session query")
                .expect("session exists")
                .model_id,
            "openai/gpt-4.1"
        );

        process.terminate();
    }

    #[test]
    fn launch_fails_before_spawn_when_project_root_is_invalid() {
        let store = AppStore::open_in_memory().expect("store opens");
        let launcher = OpenCodeRuntimeLauncher::new("/bin/sh", vec![]);

        let result = launcher.launch(
            &store,
            RuntimeLaunchRequest {
                session_id: "session-1".into(),
                project_id: "project-1".into(),
                project_root: PathBuf::from("/definitely/not/a/c4os/project"),
                title: "Run".into(),
            },
        );

        assert!(matches!(
            result,
            Err(RuntimeLaunchError::InvalidProjectRoot)
        ));
    }

    #[test]
    fn launch_failure_is_reported_when_command_is_missing() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");

        let launcher = OpenCodeRuntimeLauncher::new("/missing/c4os-opencode", vec![]);
        let result = launcher.launch(
            &store,
            RuntimeLaunchRequest {
                session_id: "session-1".into(),
                project_id: "project-1".into(),
                project_root: directory.path().to_path_buf(),
                title: "Run".into(),
            },
        );

        assert!(matches!(result, Err(RuntimeLaunchError::SpawnFailed(_))));
    }

    fn prepared_store_with_session() -> AppStore {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");
        store
            .start_openrouter_session("session-1", "project-1", "Run")
            .expect("session started");

        store
    }
}
