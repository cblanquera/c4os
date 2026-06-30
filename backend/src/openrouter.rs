use crate::provider_models::{configured_provider_api_key, DEFAULT_MODEL};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

pub const DEFAULT_OPENROUTER_MODEL: &str = DEFAULT_MODEL;
const OPENROUTER_CHAT_COMPLETIONS_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const OPENROUTER_CONNECT_TIMEOUT_SECONDS: u64 = 15;
const OPENROUTER_TOTAL_TIMEOUT_SECONDS: u64 = 90;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvent {
    pub kind: String,
    pub text: String,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterChatResult {
    pub prompt: String,
    pub run: String,
    pub agent: String,
    pub model: String,
    pub events: Vec<RuntimeEvent>,
}

pub fn api_key_from_env() -> Result<String, String> {
    std::env::var("OPENROUTER_API_KEY")
        .map(|key| key.trim().to_string())
        .map_err(|_| "OPENROUTER_API_KEY is not set".to_string())
        .and_then(|key| {
            if key.is_empty() {
                Err("OPENROUTER_API_KEY is not set".to_string())
            } else {
                Ok(key)
            }
        })
}

pub fn run_chat_stream<F>(prompt: &str, mut emit: F) -> Result<OpenRouterChatResult, String>
where
    F: FnMut(RuntimeEvent),
{
    run_chat_stream_with_model(prompt, DEFAULT_OPENROUTER_MODEL, &mut emit)
}

pub fn run_chat_stream_with_model<F>(
    prompt: &str,
    model: &str,
    mut emit: F,
) -> Result<OpenRouterChatResult, String>
where
    F: FnMut(RuntimeEvent),
{
    let api_key = configured_provider_api_key()
        .map(Ok)
        .unwrap_or_else(api_key_from_env)?;
    let selected_model = if model.trim().is_empty() {
        DEFAULT_OPENROUTER_MODEL
    } else {
        model.trim()
    };
    let request = chat_request_with_model(prompt, selected_model);
    let start = RuntimeEvent {
        kind: "activity".into(),
        text: "Starting OpenRouter request.".into(),
        sequence: 1,
        tool: None,
        args: None,
    };
    emit(start.clone());
    let mut events = vec![start];
    let mut sequence = 1;
    let mut child = Command::new("curl")
        .args(["--no-progress-meter", "--no-buffer", "--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Cannot start OpenRouter request: {error}"))?;

    {
        let mut stdin = child.stdin.take().ok_or("Cannot open curl stdin")?;
        stdin
            .write_all(curl_config(&api_key, &request).as_bytes())
            .map_err(|error| format!("Cannot write OpenRouter request: {error}"))?;
    }

    let stdout = child.stdout.take().ok_or("Cannot read OpenRouter stream")?;
    let mut final_response = String::new();
    let mut tool_state = ProviderToolCallState::default();

    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|error| format!("Cannot read OpenRouter stream: {error}"))?;
        for event in
            runtime_events_from_sse_line_with_tool_state(&line, &mut sequence, &mut tool_state)
        {
            if event.kind == "assistant" {
                final_response.push_str(&event.text);
            }
            emit(event.clone());
            events.push(event);
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Cannot finish OpenRouter request: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OpenRouter request failed: {}", stderr.trim()));
    }

    let complete = RuntimeEvent {
        kind: "complete".into(),
        text: "OpenRouter stream complete".into(),
        sequence: sequence + 1,
        tool: None,
        args: None,
    };
    emit(complete.clone());
    events.push(complete);

    Ok(OpenRouterChatResult {
        prompt: prompt.into(),
        run: "OpenRouter stream complete".into(),
        agent: if final_response.trim().is_empty() {
            "OpenRouter returned no assistant content.".into()
        } else {
            final_response
        },
        model: selected_model.into(),
        events,
    })
}

pub fn chat_request(prompt: &str) -> Value {
    chat_request_with_model(prompt, DEFAULT_OPENROUTER_MODEL)
}

pub fn chat_request_with_model(prompt: &str, model: &str) -> Value {
    json!({
        "model": model,
        "stream": true,
        "reasoning": {
            "enabled": true
        },
        "tools": c4os_provider_tools(),
        "messages": [
            {
                "role": "system",
                "content": "You are C4OS, a local project workspace assistant. Request C4OS tools with structured tool calls when workspace command output is needed; C4OS owns execution, approval, and terminal reflection. Explain useful visible work briefly before the final answer when reasoning is available."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    })
}

fn c4os_provider_tools() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": crate::tool_gateway::TOOL_TERMINAL_RUN,
                "description": "Request C4OS to run a trusted workspace terminal command through its tool gateway.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Shell command to request inside the trusted workspace."
                        },
                        "terminalKind": {
                            "type": "string",
                            "enum": ["agent"],
                            "description": "Structured runtime command output is reflected only in the read-only Agent terminal."
                        },
                        "approved": {
                            "type": "boolean",
                            "description": "True only when C4OS prompt approval policy allows the requested command."
                        },
                        "approvalReason": {
                            "type": "string",
                            "description": "Approval evidence for the C4OS tool gateway audit record."
                        }
                    },
                    "required": ["command", "terminalKind", "approved", "approvalReason"],
                    "additionalProperties": false
                }
            }
        }
    ])
}

pub fn runtime_events_from_sse_line(line: &str, sequence: &mut u64) -> Vec<RuntimeEvent> {
    runtime_events_from_sse_line_with_tool_state(
        line,
        sequence,
        &mut ProviderToolCallState::default(),
    )
}

fn runtime_events_from_sse_line_with_tool_state(
    line: &str,
    sequence: &mut u64,
    tool_state: &mut ProviderToolCallState,
) -> Vec<RuntimeEvent> {
    let Some(data) = line.strip_prefix("data:") else {
        return Vec::new();
    };
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(data) else {
        return Vec::new();
    };
    let Some(choice) = value.get("choices").and_then(|choices| choices.get(0)) else {
        return Vec::new();
    };
    let Some(delta) = choice.get("delta") else {
        return Vec::new();
    };

    let mut events = Vec::new();
    let reasoning = reasoning_text(delta);
    if !reasoning.is_empty() {
        *sequence += 1;
        events.push(RuntimeEvent {
            kind: "reasoning".into(),
            text: reasoning,
            sequence: *sequence,
            tool: None,
            args: None,
        });
    }

    if let Some(content) = delta.get("content").and_then(Value::as_str) {
        if !content.is_empty() {
            *sequence += 1;
            events.push(RuntimeEvent {
                kind: "assistant".into(),
                text: content.into(),
                sequence: *sequence,
                tool: None,
                args: None,
            });
        }
    }

    for tool_call in provider_tool_call_events(delta, choice, sequence, tool_state) {
        events.push(tool_call);
    }

    events
}

#[derive(Default)]
struct ProviderToolCallState {
    pending: BTreeMap<usize, ProviderToolCallBuffer>,
}

#[derive(Default)]
struct ProviderToolCallBuffer {
    name: Option<String>,
    arguments: String,
}

fn provider_tool_call_events(
    delta: &Value,
    choice: &Value,
    sequence: &mut u64,
    state: &mut ProviderToolCallState,
) -> Vec<RuntimeEvent> {
    let mut events = Vec::new();
    if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
        for (fallback_index, tool_call) in tool_calls.iter().enumerate() {
            let index = tool_call
                .get("index")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
                .unwrap_or(fallback_index);
            let buffer = state.pending.entry(index).or_default();
            if let Some(function) = tool_call.get("function") {
                if let Some(name) = function.get("name").and_then(Value::as_str) {
                    if !name.trim().is_empty() {
                        buffer.name = Some(name.into());
                    }
                }
                if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                    buffer.arguments.push_str(arguments);
                }
            }
        }
    }

    let finish_reason = choice.get("finish_reason").and_then(Value::as_str);
    let ready_indexes = state
        .pending
        .iter()
        .filter_map(|(index, buffer)| {
            let complete_json = serde_json::from_str::<Value>(&buffer.arguments).ok();
            if complete_json.is_some() || finish_reason == Some("tool_calls") {
                Some(*index)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for index in ready_indexes {
        let Some(buffer) = state.pending.remove(&index) else {
            continue;
        };
        let Some(name) = buffer.name else {
            continue;
        };
        let Ok(args) = serde_json::from_str::<Value>(&buffer.arguments) else {
            continue;
        };
        *sequence += 1;
        events.push(RuntimeEvent {
            kind: crate::tool_gateway::EVENT_TOOL_CALL_REQUESTED.into(),
            text: "Provider requested C4OS tool execution.".into(),
            sequence: *sequence,
            tool: Some(name),
            args: Some(args),
        });
    }

    events
}

fn reasoning_text(delta: &Value) -> String {
    if let Some(text) = delta.get("reasoning").and_then(Value::as_str) {
        return text.to_string();
    }

    delta
        .get("reasoning_details")
        .and_then(Value::as_array)
        .map(|details| {
            details
                .iter()
                .filter_map(|detail| {
                    detail
                        .get("text")
                        .or_else(|| detail.get("content"))
                        .and_then(Value::as_str)
                })
                .collect::<String>()
        })
        .unwrap_or_default()
}

fn curl_config(api_key: &str, request: &Value) -> String {
    format!(
        "url = {}\nrequest = POST\nno-buffer\nconnect-timeout = {}\nmax-time = {}\nheader = {}\nheader = {}\ndata = {}\n",
        curl_quote(OPENROUTER_CHAT_COMPLETIONS_URL),
        OPENROUTER_CONNECT_TIMEOUT_SECONDS,
        OPENROUTER_TOTAL_TIMEOUT_SECONDS,
        curl_quote(&format!("Authorization: Bearer {api_key}")),
        curl_quote("Content-Type: application/json"),
        curl_quote(&request.to_string())
    )
}

fn curl_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_005_parses_reasoning_details_before_final_content() {
        let mut sequence = 0;
        let events = runtime_events_from_sse_line(
            r#"data: {"choices":[{"delta":{"reasoning_details":[{"type":"reasoning.text","text":"Checking project state. "}],"content":"Final answer."}}]}"#,
            &mut sequence,
        );

        assert_eq!(events[0].kind, "reasoning");
        assert_eq!(events[0].text, "Checking project state. ");
        assert_eq!(events[1].kind, "assistant");
        assert_eq!(events[1].text, "Final answer.");
        assert_eq!(events[0].sequence, 1);
        assert_eq!(events[1].sequence, 2);
    }

    #[test]
    fn task_005_parses_plain_reasoning_and_ignores_done_lines() {
        let mut sequence = 0;
        let events = runtime_events_from_sse_line(
            r#"data: {"choices":[{"delta":{"reasoning":"Thinking out loud."}}]}"#,
            &mut sequence,
        );
        let done = runtime_events_from_sse_line("data: [DONE]", &mut sequence);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "reasoning");
        assert_eq!(events[0].text, "Thinking out loud.");
        assert!(done.is_empty());
    }

    #[test]
    fn task_017_normalizes_provider_tool_calls_to_c4os_lifecycle_events() {
        let mut sequence = 0;
        let events = runtime_events_from_sse_line(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"type":"function","function":{"name":"terminal.run","arguments":"{\"command\":\"ls\",\"terminalKind\":\"agent\",\"approved\":true,\"approvalReason\":\"runtime-structured-terminal-run\"}"}}]}}]}"#,
            &mut sequence,
        );

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].kind,
            crate::tool_gateway::EVENT_TOOL_CALL_REQUESTED
        );
        assert_eq!(
            events[0].tool.as_deref(),
            Some(crate::tool_gateway::TOOL_TERMINAL_RUN)
        );
        assert_eq!(events[0].args.as_ref().expect("args")["command"], "ls");
        assert_eq!(
            events[0].args.as_ref().expect("args")["terminalKind"],
            "agent"
        );
        assert_eq!(events[0].sequence, 1);
    }

    #[test]
    fn task_017_assembles_streamed_provider_tool_call_arguments_before_emitting() {
        let mut sequence = 0;
        let mut state = ProviderToolCallState::default();
        let first = runtime_events_from_sse_line_with_tool_state(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"type":"function","function":{"name":"terminal.run","arguments":"{\"command\":\""}}]}}]}"#,
            &mut sequence,
            &mut state,
        );
        let second = runtime_events_from_sse_line_with_tool_state(
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"ls\",\"terminalKind\":\"agent\",\"approved\":true,\"approvalReason\":\"runtime-structured-terminal-run\"}"}}]},"finish_reason":"tool_calls"}]}"#,
            &mut sequence,
            &mut state,
        );

        assert!(first.is_empty());
        assert_eq!(second.len(), 1);
        assert_eq!(
            second[0].kind,
            crate::tool_gateway::EVENT_TOOL_CALL_REQUESTED
        );
        assert_eq!(
            second[0].tool.as_deref(),
            Some(crate::tool_gateway::TOOL_TERMINAL_RUN)
        );
        assert_eq!(second[0].args.as_ref().expect("args")["command"], "ls");
        assert_eq!(second[0].args.as_ref().expect("args")["approved"], true);
    }

    #[test]
    fn task_017_openrouter_request_advertises_c4os_terminal_tool() {
        let request = chat_request("List files");

        assert_eq!(request["tools"][0]["type"], "function");
        assert_eq!(
            request["tools"][0]["function"]["name"],
            crate::tool_gateway::TOOL_TERMINAL_RUN
        );
        assert_eq!(
            request["tools"][0]["function"]["parameters"]["properties"]["terminalKind"]["enum"][0],
            "agent"
        );
        assert_eq!(
            request["tools"][0]["function"]["parameters"]["properties"]["approved"]["type"],
            "boolean"
        );
        assert_eq!(
            request["tools"][0]["function"]["parameters"]["required"][2],
            "approved"
        );
    }

    #[test]
    fn task_005_request_uses_review_model_and_reasoning_config() {
        let request = chat_request("Explain the slice");

        assert_eq!(request["model"], DEFAULT_OPENROUTER_MODEL);
        assert_eq!(request["stream"], true);
        assert_eq!(request["reasoning"]["enabled"], true);
        assert_eq!(request["messages"][1]["content"], "Explain the slice");
    }

    #[test]
    fn task_005_curl_config_bounds_openrouter_wait_time() {
        let config = curl_config("review-key", &chat_request("Explain the slice"));

        assert!(config.contains("connect-timeout = 15"));
        assert!(config.contains("max-time = 90"));
        assert!(config.contains("Authorization: Bearer review-key"));
    }
}
