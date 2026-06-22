use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

pub const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-lite";
const OPENROUTER_CHAT_COMPLETIONS_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const OPENROUTER_CONNECT_TIMEOUT_SECONDS: u64 = 15;
const OPENROUTER_TOTAL_TIMEOUT_SECONDS: u64 = 90;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvent {
    pub kind: String,
    pub text: String,
    pub sequence: u64,
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
    let api_key = api_key_from_env()?;
    let request = chat_request(prompt);
    let start = RuntimeEvent {
        kind: "activity".into(),
        text: "Starting OpenRouter request.".into(),
        sequence: 1,
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

    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|error| format!("Cannot read OpenRouter stream: {error}"))?;
        for event in runtime_events_from_sse_line(&line, &mut sequence) {
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
        model: DEFAULT_OPENROUTER_MODEL.into(),
        events,
    })
}

pub fn chat_request(prompt: &str) -> Value {
    json!({
        "model": DEFAULT_OPENROUTER_MODEL,
        "stream": true,
        "reasoning": {
            "enabled": true
        },
        "messages": [
            {
                "role": "system",
                "content": "You are C4OS, a local project workspace assistant. Explain your useful visible work briefly before the final answer when reasoning is available."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
    })
}

pub fn runtime_events_from_sse_line(line: &str, sequence: &mut u64) -> Vec<RuntimeEvent> {
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
    let Some(delta) = value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("delta"))
    else {
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
        });
    }

    if let Some(content) = delta.get("content").and_then(Value::as_str) {
        if !content.is_empty() {
            *sequence += 1;
            events.push(RuntimeEvent {
                kind: "assistant".into(),
                text: content.into(),
                sequence: *sequence,
            });
        }
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
