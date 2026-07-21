//! Pure, versioned Response Artifact domain contracts.
//!
//! This module owns no database, filesystem, renderer, native picker, runtime,
//! or Action Gateway authority. Callers must resolve privileged resources and
//! commit validated replacements through the owning application services.

pub mod context;
pub mod file;
pub mod folder;
pub mod record;

pub use context::{
    ARTIFACT_CONTEXT_SCHEMA_VERSION, ArtifactContextSnapshot, BrowserContextInput,
    CapabilityAccess, CapabilitySummaryEntry, ContextBudget, ContextCaptureIdentity,
    ContextCaptureInput, ContextPayload, ContextPriority, ContextRedaction, ContextSegment,
    FileContextInput, FolderContextInput, SafeContextText, TerminalContextInput,
    capture_artifact_context,
};
pub use file::{
    FileArtifactState, FileConflict, FileDraft, FileLiveVersion, FileProposal, FileProposalStatus,
    FileRecoveryState, FileResourceReference, FileSaveResult, FileStateError,
    FileVersionHistoryEntry, FileVersionOrigin,
};
pub use folder::{
    FolderArtifactState, FolderBreadcrumb, FolderEntry, FolderEntryKind, FolderListingVersion,
    FolderNavigation, FolderResourceReference, FolderSelection, FolderStateError,
    FolderToFileConversion,
};
pub use record::{
    ARTIFACT_PROVIDER_SCHEMA_VERSION, ARTIFACT_SCHEMA_VERSION, ArtifactFocusCapability,
    ArtifactHistoryEntry, ArtifactHistoryKind, ArtifactLifecycle, ArtifactProviderDescriptor,
    ArtifactRecord, ArtifactRecordError, ArtifactResourceVersion, ArtifactSource, ArtifactState,
    ArtifactWorkspaceUiState, UnknownArtifactState,
};
