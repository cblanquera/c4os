use super::title::derive_chat_title;

/// Memory-only pending Chat state. Durable creation remains one service commit.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum PendingChatState {
    #[default]
    Inactive,
    Draft(PendingChatDraft),
    PromotionRequested(PendingChatPromotion),
}

/// Starting a Chat clears stale Reply/focus/mode state and retains restoration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingChatDraft {
    pub session_id: String,
    pub project_id: String,
    pub previous_session_id: Option<String>,
    pub prompt: String,
    pub attachment_ids: Vec<String>,
    pub mode: PendingComposerMode,
    pub reply_target_id: Option<String>,
    pub focused_artifact_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingComposerMode {
    Chat,
}

/// A valid first submission is ready for one atomic authoritative promotion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingChatPromotion {
    pub session_id: String,
    pub project_id: String,
    pub previous_session_id: Option<String>,
    pub prompt: Option<String>,
    pub attachment_ids: Vec<String>,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingChatCompletion {
    pub state: PendingChatState,
    pub active_project_id: String,
    pub active_session_id: String,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingChatCancellation {
    pub state: PendingChatState,
    pub restore_session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PendingChatError {
    EmptyIdentifier,
    AlreadyPending,
    NotPending,
    PromotionAlreadyRequested,
    EmptySubmission,
    AttachmentMetadataMismatch,
    DuplicateAttachment,
}

/// Begins one blank memory-only Chat and records the saved Chat to restore.
pub fn begin_pending_chat(
    state: &PendingChatState,
    session_id: impl Into<String>,
    project_id: impl Into<String>,
    previous_session_id: Option<String>,
) -> Result<PendingChatState, PendingChatError> {
    if !matches!(state, PendingChatState::Inactive) {
        return Err(PendingChatError::AlreadyPending);
    }
    let session_id = session_id.into();
    let project_id = project_id.into();
    if session_id.trim().is_empty() || project_id.trim().is_empty() {
        return Err(PendingChatError::EmptyIdentifier);
    }
    Ok(PendingChatState::Draft(PendingChatDraft {
        session_id,
        project_id,
        previous_session_id,
        prompt: String::new(),
        attachment_ids: Vec::new(),
        mode: PendingComposerMode::Chat,
        reply_target_id: None,
        focused_artifact_id: None,
    }))
}

/// Replaces the controlled pending draft without mutating the prior state.
pub fn update_pending_chat_draft(
    state: &PendingChatState,
    prompt: impl Into<String>,
    attachment_ids: Vec<String>,
) -> Result<PendingChatState, PendingChatError> {
    let PendingChatState::Draft(draft) = state else {
        return Err(PendingChatError::NotPending);
    };
    validate_attachment_ids(&attachment_ids)?;
    let mut replacement = draft.clone();
    replacement.prompt = prompt.into();
    replacement.attachment_ids = attachment_ids;
    Ok(PendingChatState::Draft(replacement))
}

/// Requests promotion only for text, attachment, or combined valid submission.
pub fn request_pending_chat_promotion<'a>(
    state: &PendingChatState,
    attachment_file_names: impl IntoIterator<Item = &'a str>,
) -> Result<PendingChatState, PendingChatError> {
    let PendingChatState::Draft(draft) = state else {
        return if matches!(state, PendingChatState::PromotionRequested(_)) {
            Err(PendingChatError::PromotionAlreadyRequested)
        } else {
            Err(PendingChatError::NotPending)
        };
    };
    let attachment_file_names = attachment_file_names.into_iter().collect::<Vec<_>>();
    if attachment_file_names.len() != draft.attachment_ids.len() {
        return Err(PendingChatError::AttachmentMetadataMismatch);
    }
    let prompt = (!draft.prompt.trim().is_empty()).then(|| draft.prompt.clone());
    let Some(title) = derive_chat_title(prompt.as_deref(), attachment_file_names) else {
        return Err(PendingChatError::EmptySubmission);
    };
    Ok(PendingChatState::PromotionRequested(PendingChatPromotion {
        session_id: draft.session_id.clone(),
        project_id: draft.project_id.clone(),
        previous_session_id: draft.previous_session_id.clone(),
        prompt,
        attachment_ids: draft.attachment_ids.clone(),
        title,
    }))
}

/// Completes the memory transition after the authoritative atomic create wins.
pub fn complete_pending_chat(
    state: &PendingChatState,
) -> Result<PendingChatCompletion, PendingChatError> {
    let PendingChatState::PromotionRequested(promotion) = state else {
        return Err(PendingChatError::NotPending);
    };
    Ok(PendingChatCompletion {
        state: PendingChatState::Inactive,
        active_project_id: promotion.project_id.clone(),
        active_session_id: promotion.session_id.clone(),
        title: promotion.title.clone(),
    })
}

/// Restores an editable draft when authoritative promotion fails before commit.
pub fn fail_pending_chat_promotion(
    state: &PendingChatState,
) -> Result<PendingChatState, PendingChatError> {
    let PendingChatState::PromotionRequested(promotion) = state else {
        return Err(PendingChatError::NotPending);
    };
    Ok(PendingChatState::Draft(PendingChatDraft {
        session_id: promotion.session_id.clone(),
        project_id: promotion.project_id.clone(),
        previous_session_id: promotion.previous_session_id.clone(),
        prompt: promotion.prompt.clone().unwrap_or_default(),
        attachment_ids: promotion.attachment_ids.clone(),
        mode: PendingComposerMode::Chat,
        reply_target_id: None,
        focused_artifact_id: None,
    }))
}

/// Cancels only an uncommitted draft and restores the prior saved selection.
pub fn cancel_pending_chat(
    state: &PendingChatState,
) -> Result<PendingChatCancellation, PendingChatError> {
    let PendingChatState::Draft(draft) = state else {
        return Err(PendingChatError::NotPending);
    };
    Ok(PendingChatCancellation {
        state: PendingChatState::Inactive,
        restore_session_id: draft.previous_session_id.clone(),
    })
}

fn validate_attachment_ids(attachment_ids: &[String]) -> Result<(), PendingChatError> {
    let mut unique = std::collections::BTreeSet::new();
    for attachment_id in attachment_ids {
        if attachment_id.trim().is_empty() {
            return Err(PendingChatError::EmptyIdentifier);
        }
        if !unique.insert(attachment_id.as_str()) {
            return Err(PendingChatError::DuplicateAttachment);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> PendingChatState {
        begin_pending_chat(
            &PendingChatState::Inactive,
            "pending-1",
            "project-1",
            Some("saved-1".into()),
        )
        .unwrap()
    }

    #[test]
    fn begin_clears_transient_state_and_preserves_saved_restoration() {
        let PendingChatState::Draft(draft) = draft() else {
            panic!("expected a pending draft");
        };
        assert_eq!(draft.previous_session_id.as_deref(), Some("saved-1"));
        assert_eq!(draft.prompt, "");
        assert!(draft.attachment_ids.is_empty());
        assert_eq!(draft.mode, PendingComposerMode::Chat);
        assert_eq!(draft.reply_target_id, None);
        assert_eq!(draft.focused_artifact_id, None);
    }

    #[test]
    fn attachment_only_submission_requests_atomic_promotion() {
        let state = update_pending_chat_draft(&draft(), "", vec!["attachment-1".into()]).unwrap();
        let promoted = request_pending_chat_promotion(&state, ["diagram.png"]).unwrap();
        let PendingChatState::PromotionRequested(promotion) = promoted else {
            panic!("expected promotion");
        };
        assert_eq!(promotion.prompt, None);
        assert_eq!(promotion.title, "diagram.png");
    }

    #[test]
    fn failed_promotion_restores_exact_pending_draft() {
        let state =
            update_pending_chat_draft(&draft(), "Build this", vec!["attachment-1".into()]).unwrap();
        let requested = request_pending_chat_promotion(&state, ["plan.md"]).unwrap();
        let restored = fail_pending_chat_promotion(&requested).unwrap();
        let PendingChatState::Draft(restored) = restored else {
            panic!("expected restored draft");
        };
        assert_eq!(restored.prompt, "Build this");
        assert_eq!(restored.attachment_ids, ["attachment-1"]);
    }

    #[test]
    fn completed_promotion_activates_origin_project_and_new_session() {
        let state = update_pending_chat_draft(&draft(), "Hello", vec![]).unwrap();
        let requested = request_pending_chat_promotion(&state, []).unwrap();
        let completion = complete_pending_chat(&requested).unwrap();
        assert_eq!(completion.state, PendingChatState::Inactive);
        assert_eq!(completion.active_project_id, "project-1");
        assert_eq!(completion.active_session_id, "pending-1");
        assert_eq!(completion.title, "Hello");
    }

    #[test]
    fn cancellation_restores_previous_saved_session() {
        let cancellation = cancel_pending_chat(&draft()).unwrap();
        assert_eq!(cancellation.state, PendingChatState::Inactive);
        assert_eq!(cancellation.restore_session_id.as_deref(), Some("saved-1"));
    }

    #[test]
    fn empty_submission_and_duplicate_attachments_fail_closed() {
        assert_eq!(
            request_pending_chat_promotion(&draft(), []),
            Err(PendingChatError::EmptySubmission)
        );
        assert_eq!(
            update_pending_chat_draft(&draft(), "", vec!["attachment".into(), "attachment".into()],),
            Err(PendingChatError::DuplicateAttachment)
        );
    }
}
