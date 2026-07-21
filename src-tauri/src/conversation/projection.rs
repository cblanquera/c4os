use std::collections::BTreeSet;

const MAX_PROJECTED_EVENTS: usize = 2_048;
const MAX_ASSISTANT_TEXT_CHARS: usize = 262_144;
const MAX_REASONING_SUMMARY_CHARS: usize = 32_768;

/// Mirrors existing runtime status categories without importing service types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeRunStatusInput {
    Dispatching,
    Streaming,
    CancellationRequested,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}

/// Mirrors existing SessionRecord event categories for a narrow mapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeEventKindInput {
    Status,
    TextDelta,
    ReasoningDelta,
    /// A display-safe summary explicitly normalized by the owning adapter.
    ///
    /// Integration must never map raw reasoning deltas into this category.
    SafeReasoningSummary,
    WorkActivity,
    ActionIntent,
    ActionProgress,
    ActionResult,
    Media,
    Usage,
    Error,
    Completion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeEventInput {
    pub sequence: u64,
    pub kind: RuntimeEventKindInput,
    pub payload: String,
}

/// Simple owned input maps directly from one existing RunAttemptRecord.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeRunInput {
    pub attempt_id: String,
    pub turn_id: String,
    pub status: RuntimeRunStatusInput,
    pub events: Vec<RuntimeEventInput>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedRunStatus {
    Starting,
    Working,
    Cancelling,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedActivityKind {
    Status,
    Work,
    ReasoningSummary,
    Action,
    Media,
    Usage,
    Error,
    Completion,
}

/// Generic activity labels are product-owned and never copy runtime payloads.
/// `detail` is populated only for an explicitly safe reasoning-summary input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedActivity {
    pub sequence: u64,
    pub kind: ProjectedActivityKind,
    pub label: &'static str,
    pub detail: Option<String>,
    pub detail_was_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedRun {
    pub attempt_id: String,
    pub turn_id: String,
    pub status: ProjectedRunStatus,
    pub assistant_markdown: String,
    pub activities: Vec<ProjectedActivity>,
    pub omitted_event_count: usize,
    pub text_was_truncated: bool,
}

/// Projects validated runtime history while dropping all private reasoning text.
pub fn project_runtime_run(mut input: RuntimeRunInput) -> ProjectedRun {
    input.events.sort_by_key(|event| event.sequence);
    let mut seen_sequences = BTreeSet::new();
    let mut assistant_markdown = String::new();
    let mut activities = Vec::new();
    let mut omitted_event_count = 0;
    let mut text_was_truncated = false;
    let mut projected_text_chars = 0;
    let mut projected_reasoning_summary_chars = 0;

    for event in input.events {
        if !seen_sequences.insert(event.sequence) || seen_sequences.len() > MAX_PROJECTED_EVENTS {
            omitted_event_count += 1;
            continue;
        }
        match event.kind {
            RuntimeEventKindInput::TextDelta => {
                text_was_truncated |= append_bounded_text(
                    &mut assistant_markdown,
                    &mut projected_text_chars,
                    &event.payload,
                    MAX_ASSISTANT_TEXT_CHARS,
                );
            }
            RuntimeEventKindInput::ReasoningDelta => {
                // Raw reasoning is never a renderer projection, even when the
                // effective route may separately expose a reviewed summary.
                omitted_event_count += 1;
            }
            RuntimeEventKindInput::SafeReasoningSummary => {
                let (detail, detail_was_truncated) = take_bounded_text(
                    &mut projected_reasoning_summary_chars,
                    &event.payload,
                    MAX_REASONING_SUMMARY_CHARS,
                );
                if detail.is_empty() {
                    omitted_event_count += 1;
                    continue;
                }
                activities.push(ProjectedActivity {
                    sequence: event.sequence,
                    kind: ProjectedActivityKind::ReasoningSummary,
                    label: "Reasoning summary",
                    detail: Some(detail),
                    detail_was_truncated,
                });
            }
            kind => activities.push(project_activity(event.sequence, kind)),
        }
    }

    ProjectedRun {
        attempt_id: input.attempt_id,
        turn_id: input.turn_id,
        status: project_status(input.status),
        assistant_markdown,
        activities,
        omitted_event_count,
        text_was_truncated,
    }
}

fn project_status(status: RuntimeRunStatusInput) -> ProjectedRunStatus {
    match status {
        RuntimeRunStatusInput::Dispatching => ProjectedRunStatus::Starting,
        RuntimeRunStatusInput::Streaming => ProjectedRunStatus::Working,
        RuntimeRunStatusInput::CancellationRequested => ProjectedRunStatus::Cancelling,
        RuntimeRunStatusInput::Completed => ProjectedRunStatus::Completed,
        RuntimeRunStatusInput::Failed => ProjectedRunStatus::Failed,
        RuntimeRunStatusInput::Interrupted => ProjectedRunStatus::Interrupted,
        RuntimeRunStatusInput::Cancelled => ProjectedRunStatus::Cancelled,
    }
}

fn project_activity(sequence: u64, kind: RuntimeEventKindInput) -> ProjectedActivity {
    let (kind, label) = match kind {
        RuntimeEventKindInput::Status => (ProjectedActivityKind::Status, "Runtime status updated"),
        RuntimeEventKindInput::WorkActivity => (ProjectedActivityKind::Work, "Runtime activity"),
        RuntimeEventKindInput::ActionIntent => (ProjectedActivityKind::Action, "Action requested"),
        RuntimeEventKindInput::ActionProgress => {
            (ProjectedActivityKind::Action, "Action in progress")
        }
        RuntimeEventKindInput::ActionResult => (ProjectedActivityKind::Action, "Action completed"),
        RuntimeEventKindInput::Media => (ProjectedActivityKind::Media, "Media received"),
        RuntimeEventKindInput::Usage => (ProjectedActivityKind::Usage, "Usage updated"),
        RuntimeEventKindInput::Error => (ProjectedActivityKind::Error, "Run reported an error"),
        RuntimeEventKindInput::Completion => (ProjectedActivityKind::Completion, "Run completed"),
        RuntimeEventKindInput::TextDelta
        | RuntimeEventKindInput::ReasoningDelta
        | RuntimeEventKindInput::SafeReasoningSummary => {
            unreachable!("text events are projected before activity mapping")
        }
    };
    ProjectedActivity {
        sequence,
        kind,
        label,
        detail: None,
        detail_was_truncated: false,
    }
}

fn take_bounded_text(used_chars: &mut usize, text: &str, maximum_chars: usize) -> (String, bool) {
    if *used_chars >= maximum_chars {
        return (String::new(), !text.is_empty());
    }
    let remaining = maximum_chars - *used_chars;
    let mut text_chars = text.chars();
    let accepted = text_chars.by_ref().take(remaining).collect::<String>();
    *used_chars += accepted.chars().count();
    let was_truncated = text_chars.next().is_some();
    (accepted, was_truncated)
}

fn append_bounded_text(
    target: &mut String,
    used_chars: &mut usize,
    delta: &str,
    maximum_chars: usize,
) -> bool {
    if *used_chars >= maximum_chars {
        return !delta.is_empty();
    }
    let remaining = maximum_chars - *used_chars;
    let mut delta_chars = delta.chars();
    let accepted = delta_chars.by_ref().take(remaining).collect::<String>();
    *used_chars += accepted.chars().count();
    target.push_str(&accepted);
    delta_chars.next().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_text_and_generic_activity_are_projected_in_sequence_order() {
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Completed,
            events: vec![
                RuntimeEventInput {
                    sequence: 3,
                    kind: RuntimeEventKindInput::Completion,
                    payload: "private native completion payload".into(),
                },
                RuntimeEventInput {
                    sequence: 2,
                    kind: RuntimeEventKindInput::TextDelta,
                    payload: "world".into(),
                },
                RuntimeEventInput {
                    sequence: 1,
                    kind: RuntimeEventKindInput::TextDelta,
                    payload: "Hello ".into(),
                },
            ],
        });
        assert_eq!(projected.assistant_markdown, "Hello world");
        assert_eq!(projected.status, ProjectedRunStatus::Completed);
        assert_eq!(projected.activities[0].label, "Run completed");
        assert!(!format!("{projected:?}").contains("private native completion payload"));
    }

    #[test]
    fn private_reasoning_payload_never_enters_projection() {
        let secret = "hidden chain of thought";
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Streaming,
            events: vec![RuntimeEventInput {
                sequence: 1,
                kind: RuntimeEventKindInput::ReasoningDelta,
                payload: secret.into(),
            }],
        });
        assert_eq!(projected.assistant_markdown, "");
        assert!(projected.activities.is_empty());
        assert_eq!(projected.omitted_event_count, 1);
        assert!(!format!("{projected:?}").contains(secret));
    }

    #[test]
    fn explicitly_safe_reasoning_summary_is_distinct_from_private_reasoning() {
        let private_reasoning = "hidden chain of thought";
        let safe_summary = "Compared the accepted runtime boundaries.";
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Streaming,
            events: vec![
                RuntimeEventInput {
                    sequence: 1,
                    kind: RuntimeEventKindInput::ReasoningDelta,
                    payload: private_reasoning.into(),
                },
                RuntimeEventInput {
                    sequence: 2,
                    kind: RuntimeEventKindInput::SafeReasoningSummary,
                    payload: safe_summary.into(),
                },
            ],
        });

        assert_eq!(projected.omitted_event_count, 1);
        assert_eq!(projected.activities.len(), 1);
        assert_eq!(
            projected.activities[0].kind,
            ProjectedActivityKind::ReasoningSummary
        );
        assert_eq!(projected.activities[0].label, "Reasoning summary");
        assert_eq!(
            projected.activities[0].detail.as_deref(),
            Some(safe_summary)
        );
        assert!(!projected.activities[0].detail_was_truncated);
        assert!(!format!("{projected:?}").contains(private_reasoning));
    }

    #[test]
    fn reasoning_summaries_preserve_sequence_with_generic_activity() {
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Streaming,
            events: vec![
                RuntimeEventInput {
                    sequence: 3,
                    kind: RuntimeEventKindInput::SafeReasoningSummary,
                    payload: "Verified the result.".into(),
                },
                RuntimeEventInput {
                    sequence: 1,
                    kind: RuntimeEventKindInput::WorkActivity,
                    payload: "private runtime activity payload".into(),
                },
                RuntimeEventInput {
                    sequence: 2,
                    kind: RuntimeEventKindInput::SafeReasoningSummary,
                    payload: "Checked the constraints.".into(),
                },
            ],
        });

        assert_eq!(
            projected
                .activities
                .iter()
                .map(|activity| activity.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(projected.activities[0].kind, ProjectedActivityKind::Work);
        assert_eq!(projected.activities[0].detail, None);
        assert_eq!(
            projected.activities[1].kind,
            ProjectedActivityKind::ReasoningSummary
        );
        assert_eq!(
            projected.activities[2].kind,
            ProjectedActivityKind::ReasoningSummary
        );
        assert!(!format!("{projected:?}").contains("private runtime activity payload"));
    }

    #[test]
    fn reasoning_summary_content_is_unicode_safe_and_cumulatively_bounded() {
        let first_summary = "é".repeat(MAX_REASONING_SUMMARY_CHARS - 1);
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Completed,
            events: vec![
                RuntimeEventInput {
                    sequence: 1,
                    kind: RuntimeEventKindInput::SafeReasoningSummary,
                    payload: first_summary,
                },
                RuntimeEventInput {
                    sequence: 2,
                    kind: RuntimeEventKindInput::SafeReasoningSummary,
                    payload: "xy".into(),
                },
                RuntimeEventInput {
                    sequence: 3,
                    kind: RuntimeEventKindInput::SafeReasoningSummary,
                    payload: "not projected".into(),
                },
            ],
        });

        assert_eq!(projected.activities.len(), 2);
        assert_eq!(
            projected.activities[0]
                .detail
                .as_deref()
                .expect("first summary")
                .chars()
                .count(),
            MAX_REASONING_SUMMARY_CHARS - 1
        );
        assert_eq!(projected.activities[1].detail.as_deref(), Some("x"));
        assert!(projected.activities[1].detail_was_truncated);
        assert_eq!(projected.omitted_event_count, 1);
    }

    #[test]
    fn runtime_payloads_become_product_owned_generic_activity_labels() {
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Failed,
            events: vec![RuntimeEventInput {
                sequence: 1,
                kind: RuntimeEventKindInput::ActionProgress,
                payload: "raw tool arguments must not render".into(),
            }],
        });
        assert_eq!(projected.activities[0].label, "Action in progress");
        assert!(!format!("{projected:?}").contains("raw tool arguments"));
        assert_eq!(projected.status, ProjectedRunStatus::Failed);
    }

    #[test]
    fn duplicate_event_sequences_fail_closed_to_one_projection() {
        let projected = project_runtime_run(RuntimeRunInput {
            attempt_id: "attempt-1".into(),
            turn_id: "turn-1".into(),
            status: RuntimeRunStatusInput::Streaming,
            events: vec![
                RuntimeEventInput {
                    sequence: 1,
                    kind: RuntimeEventKindInput::TextDelta,
                    payload: "kept".into(),
                },
                RuntimeEventInput {
                    sequence: 1,
                    kind: RuntimeEventKindInput::TextDelta,
                    payload: "duplicate".into(),
                },
            ],
        });
        assert_eq!(projected.assistant_markdown, "kept");
        assert_eq!(projected.omitted_event_count, 1);
    }
}
