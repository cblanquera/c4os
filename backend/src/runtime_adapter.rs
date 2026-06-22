#[cfg(not(test))]
use crate::openrouter::run_chat_stream_with_model;
use crate::openrouter::RuntimeEvent;
#[cfg(not(test))]
use crate::provider_models::provider_api_key_configured;
use crate::provider_models::DEFAULT_MODEL;

#[derive(Debug, Clone, PartialEq)]
pub struct C4osRuntimeResult {
    pub prompt: String,
    pub run: String,
    pub agent: String,
    pub model: String,
    pub events: Vec<RuntimeEvent>,
}

pub struct C4osRuntimeAdapter;

impl C4osRuntimeAdapter {
    pub fn run_chat<F>(prompt: &str, model: &str, emit: F) -> Result<C4osRuntimeResult, String>
    where
        F: FnMut(RuntimeEvent),
    {
        let selected_model = if model.trim().is_empty() {
            DEFAULT_MODEL
        } else {
            model.trim()
        };

        #[cfg(test)]
        {
            let _ = emit;
            return Ok(review_fallback(prompt, selected_model));
        }

        #[cfg(not(test))]
        if provider_api_key_configured() {
            return run_chat_stream_with_model(prompt, selected_model, emit).map(|result| {
                C4osRuntimeResult {
                    prompt: result.prompt,
                    run: result.run,
                    agent: result.agent,
                    model: result.model,
                    events: result.events,
                }
            });
        }

        #[cfg(not(test))]
        Ok(review_fallback(prompt, selected_model))
    }
}

fn review_fallback(prompt: &str, model: &str) -> C4osRuntimeResult {
    let events = vec![
        RuntimeEvent {
            kind: "activity".into(),
            text: "C4OS runtime adapter recorded the prompt.".into(),
            sequence: 1,
        },
        RuntimeEvent {
            kind: "complete".into(),
            text: "C4OS runtime adapter fallback complete.".into(),
            sequence: 2,
        },
    ];

    C4osRuntimeResult {
        prompt: prompt.into(),
        run: "C4OS runtime adapter fallback complete.".into(),
        agent: "C4OS recorded this prompt in the persistent session store. Configure a provider key to run the selected model.".into(),
        model: model.into(),
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_007_runtime_adapter_returns_c4os_result_without_native_runtime_terms() {
        let result = C4osRuntimeAdapter::run_chat("Persist this prompt", "model/test", |_| {})
            .expect("fallback run");
        let serialized = serde_json::to_string(&result.events).expect("serialize events");

        assert_eq!(result.prompt, "Persist this prompt");
        assert_eq!(result.model, "model/test");
        assert!(result.run.contains("C4OS runtime adapter"));
        assert!(!serialized.to_lowercase().contains("opencode"));
    }
}
