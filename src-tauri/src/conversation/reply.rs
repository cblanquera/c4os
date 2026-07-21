const MESSAGE_EXCERPT_WORDS: usize = 8;
const ARTIFACT_EXCERPT_CHARS: usize = 96;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplyTargetKind {
    UserMessage,
    AssistantMessage,
    BrowserArtifact,
    FileArtifact,
    FolderArtifact,
    TerminalArtifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplyBehavior {
    AppendConversation,
    UpdateBrowserInPlace,
    ProposeFileChange,
    AppendTerminalArtifact,
}

/// Simple owned input lets integration map existing message/artifact records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplyTargetInput {
    UserMessage {
        message_id: String,
        source_markdown: String,
    },
    AssistantMessage {
        message_id: String,
        source_markdown: String,
    },
    BrowserArtifact {
        artifact_id: String,
        title: String,
        current_url: String,
    },
    FileArtifact {
        artifact_id: String,
        display_name: String,
    },
    FolderArtifact {
        artifact_id: String,
        display_name: String,
    },
    TerminalArtifact {
        artifact_id: String,
        command: String,
        terminal_session_id: String,
    },
}

/// Safe quoted-reference metadata; no speaker label or mutable artifact body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplyContextMetadata {
    pub target_id: String,
    pub target_kind: ReplyTargetKind,
    pub functional_label: String,
    pub excerpt: String,
    pub behavior: ReplyBehavior,
    pub artifact_id: Option<String>,
    pub terminal_session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplyError {
    EmptyIdentifier,
    EmptyReferenceContent,
}

/// Projects a Reply target into immutable composer-context metadata.
pub fn project_reply_context(target: ReplyTargetInput) -> Result<ReplyContextMetadata, ReplyError> {
    match target {
        ReplyTargetInput::UserMessage {
            message_id,
            source_markdown,
        } => message_context(message_id, source_markdown, ReplyTargetKind::UserMessage),
        ReplyTargetInput::AssistantMessage {
            message_id,
            source_markdown,
        } => message_context(
            message_id,
            source_markdown,
            ReplyTargetKind::AssistantMessage,
        ),
        ReplyTargetInput::BrowserArtifact {
            artifact_id,
            title,
            current_url,
        } => {
            let label = compact(&title).unwrap_or_else(|| current_url.clone());
            artifact_context(
                artifact_id,
                ReplyTargetKind::BrowserArtifact,
                label,
                current_url,
                ReplyBehavior::UpdateBrowserInPlace,
                None,
            )
        }
        ReplyTargetInput::FileArtifact {
            artifact_id,
            display_name,
        } => artifact_context(
            artifact_id,
            ReplyTargetKind::FileArtifact,
            display_name.clone(),
            display_name,
            ReplyBehavior::ProposeFileChange,
            None,
        ),
        ReplyTargetInput::FolderArtifact {
            artifact_id,
            display_name,
        } => artifact_context(
            artifact_id,
            ReplyTargetKind::FolderArtifact,
            display_name.clone(),
            display_name,
            ReplyBehavior::ProposeFileChange,
            None,
        ),
        ReplyTargetInput::TerminalArtifact {
            artifact_id,
            command,
            terminal_session_id,
        } => {
            require_identifier(&terminal_session_id)?;
            artifact_context(
                artifact_id,
                ReplyTargetKind::TerminalArtifact,
                command.clone(),
                command,
                ReplyBehavior::AppendTerminalArtifact,
                Some(terminal_session_id),
            )
        }
    }
}

fn message_context(
    message_id: String,
    source_markdown: String,
    target_kind: ReplyTargetKind,
) -> Result<ReplyContextMetadata, ReplyError> {
    require_identifier(&message_id)?;
    let excerpt = first_words(&source_markdown, MESSAGE_EXCERPT_WORDS)
        .ok_or(ReplyError::EmptyReferenceContent)?;
    Ok(ReplyContextMetadata {
        target_id: message_id,
        target_kind,
        functional_label: excerpt.clone(),
        excerpt,
        behavior: ReplyBehavior::AppendConversation,
        artifact_id: None,
        terminal_session_id: None,
    })
}

fn artifact_context(
    artifact_id: String,
    target_kind: ReplyTargetKind,
    functional_label: String,
    excerpt: String,
    behavior: ReplyBehavior,
    terminal_session_id: Option<String>,
) -> Result<ReplyContextMetadata, ReplyError> {
    require_identifier(&artifact_id)?;
    let functional_label = compact(&functional_label)
        .filter(|value| !value.is_empty())
        .ok_or(ReplyError::EmptyReferenceContent)?;
    let excerpt = compact(&excerpt)
        .filter(|value| !value.is_empty())
        .ok_or(ReplyError::EmptyReferenceContent)?;
    Ok(ReplyContextMetadata {
        target_id: artifact_id.clone(),
        target_kind,
        functional_label: truncate(&functional_label, ARTIFACT_EXCERPT_CHARS),
        excerpt: truncate(&excerpt, ARTIFACT_EXCERPT_CHARS),
        behavior,
        artifact_id: Some(artifact_id),
        terminal_session_id,
    })
}

fn first_words(value: &str, maximum_words: usize) -> Option<String> {
    let words = value
        .split_whitespace()
        .take(maximum_words)
        .collect::<Vec<_>>();
    (!words.is_empty()).then(|| words.join(" "))
}

fn compact(value: &str) -> Option<String> {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!compact.is_empty()).then_some(compact)
}

fn truncate(value: &str, maximum_chars: usize) -> String {
    if value.chars().count() <= maximum_chars {
        return value.to_owned();
    }
    let mut truncated = value.chars().take(maximum_chars - 1).collect::<String>();
    truncated.push('…');
    truncated
}

fn require_identifier(value: &str) -> Result<(), ReplyError> {
    if value.trim().is_empty() {
        Err(ReplyError::EmptyIdentifier)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_reference_uses_first_words_instead_of_speaker_name() {
        let context = project_reply_context(ReplyTargetInput::UserMessage {
            message_id: "message-1".into(),
            source_markdown: "Please update the release plan with current dates and owners".into(),
        })
        .unwrap();
        assert_eq!(
            context.functional_label,
            "Please update the release plan with current dates"
        );
        assert!(!context.functional_label.contains("User"));
        assert_eq!(context.behavior, ReplyBehavior::AppendConversation);
    }

    #[test]
    fn browser_reply_retains_target_for_in_place_update() {
        let context = project_reply_context(ReplyTargetInput::BrowserArtifact {
            artifact_id: "browser-1".into(),
            title: "C4OS documentation".into(),
            current_url: "https://example.test/docs".into(),
        })
        .unwrap();
        assert_eq!(context.artifact_id.as_deref(), Some("browser-1"));
        assert_eq!(context.behavior, ReplyBehavior::UpdateBrowserInPlace);
    }

    #[test]
    fn terminal_reply_carries_session_without_mutating_quoted_artifact() {
        let context = project_reply_context(ReplyTargetInput::TerminalArtifact {
            artifact_id: "terminal-card-1".into(),
            command: "npm test".into(),
            terminal_session_id: "shell-1".into(),
        })
        .unwrap();
        assert_eq!(context.behavior, ReplyBehavior::AppendTerminalArtifact);
        assert_eq!(context.terminal_session_id.as_deref(), Some("shell-1"));
        assert_eq!(context.target_id, "terminal-card-1");
    }

    #[test]
    fn empty_reference_content_fails_closed() {
        assert_eq!(
            project_reply_context(ReplyTargetInput::AssistantMessage {
                message_id: "assistant-1".into(),
                source_markdown: "  ".into(),
            }),
            Err(ReplyError::EmptyReferenceContent)
        );
    }
}
