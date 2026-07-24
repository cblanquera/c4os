//! Pure, versioned Response Artifact domain contracts.
//!
//! This module owns no database, filesystem, renderer, native picker, runtime,
//! or Action Gateway authority. Callers must resolve privileged resources and
//! commit validated replacements through the owning application services.

pub mod browser;
pub mod context;
pub mod file;
pub mod folder;
pub mod record;
pub mod terminal;

pub use browser::{
    BrowserArtifactState, BrowserControllerEventMeta, BrowserEnvironmentReference,
    BrowserEnvironmentScope, BrowserErrorCode, BrowserHistoryEntry, BrowserNavigationIntent,
    BrowserNavigationKind, BrowserNavigationTarget, BrowserPhase, BrowserRecoveryCode,
    BrowserStateError, MAX_BROWSER_DISPLAY_URL_BYTES, MAX_BROWSER_HISTORY_ENTRIES,
    MAX_BROWSER_NAVIGATION_URL_BYTES, MAX_BROWSER_REFERENCE_ID_BYTES, MAX_BROWSER_TITLE_BYTES,
    normalize_browser_address,
};
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
pub use terminal::{
    MAX_TERMINAL_COMMAND_BYTES, MAX_TERMINAL_OUTPUT_BYTES, TerminalArtifactState,
    TerminalCommandIdentity, TerminalCommandStatus, TerminalDimensions, TerminalOutputBuffer,
    TerminalProcessProvenance, TerminalStateError,
};
