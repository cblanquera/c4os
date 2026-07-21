//! Pure conversation-domain projections and transition helpers.
//!
//! These modules intentionally own no database, runtime, renderer, or native
//! authority. Integration code maps authoritative records into the small
//! inputs here and commits any returned transition through the owning service.

pub mod attachments;
pub mod navigation;
pub mod pending;
pub mod projection;
pub mod reply;
pub mod title;

pub use attachments::{
    AttachmentDraft, AttachmentDraftError, DraftAttachment, DraftAttachmentInput,
};
pub use navigation::{
    ActiveNavigation, InactivationError, InactivationPlan, InactivationTarget, NavigationProject,
    NavigationSession, SessionSearchResult, normalize_search_text, plan_inactivation,
    search_sessions,
};
pub use pending::{
    PendingChatCancellation, PendingChatCompletion, PendingChatDraft, PendingChatError,
    PendingChatPromotion, PendingChatState, PendingComposerMode, begin_pending_chat,
    cancel_pending_chat, complete_pending_chat, fail_pending_chat_promotion,
    request_pending_chat_promotion, update_pending_chat_draft,
};
pub use projection::{
    ProjectedActivity, ProjectedActivityKind, ProjectedRun, ProjectedRunStatus, RuntimeEventInput,
    RuntimeEventKindInput, RuntimeRunInput, RuntimeRunStatusInput, project_runtime_run,
};
pub use reply::{
    ReplyBehavior, ReplyContextMetadata, ReplyError, ReplyTargetInput, ReplyTargetKind,
    project_reply_context,
};
pub use title::{CHAT_TITLE_MAX_CHARS, derive_chat_title};
