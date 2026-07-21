import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useLocation, useNavigate } from "react-router";

import { useAppDispatch, useAppSelector, useAppStore } from "../../app/hooks";
import {
  createSettingsVisit,
  settingsSectionForRoute,
} from "../../app/settings-visit";
import type { AppRoutePath } from "../../app/route-contract";
import { Notice } from "../../components/accessible";
import {
  Composer,
  type ComposerReplyReference,
} from "../conversation/composer";
import { GitBranchControl } from "../conversation/branch-control";
import { ConversationFileDropOverlay } from "../conversation/drop";
import {
  ConversationFocusComposition,
  type FocusedConversationArtifact,
} from "../conversation/focus";
import {
  FileArtifact,
  FolderArtifact,
  UnknownArtifact,
  type ArtifactContext,
  type FileArtifactState,
} from "../artifacts";
import {
  ChatInformationPopover,
  ModelSelector,
  ReasoningEffortControl,
  type ModelCapability,
  type ModelControlModel,
  type ReasoningEffort,
} from "../conversation/model-controls";
import {
  ConversationTranscript,
  PendingConversationPrompt,
  ProjectSessionNavigation,
  type ConversationTranscriptTurn,
} from "../conversation/ui";
import {
  addConversationProject,
  activateConversationProject,
  activateConversationSession,
  beginPendingConversation,
  cancelConversationAttempt,
  cancelPendingConversation,
  copyConversationProjectPath,
  inactivateConversationProject,
  inactivateConversationSession,
  previewConversationAttachment,
  readConversationSnapshot,
  requestConversationBranch,
  answerConversationBranchApproval,
  relocateConversationProject,
  renameConversationProject,
  reorderConversationProjects,
  retryConversationAttempt,
  revealConversationProject,
  submitConversation,
  updateConversationDraft,
  type ConversationSnapshot,
} from "../../platform/conversation-service";
import {
  answerArtifactApproval,
  beginFileArtifactEdit,
  closeArtifactFocus,
  discardFileArtifactDraft,
  focusArtifact,
  navigateFolderArtifact,
  openFileArtifact,
  openFolderArtifact,
  readArtifactWorkspaceSnapshot,
  rejectFileArtifactProposal,
  refreshFolderArtifact,
  replyToArtifact,
  resolveFileArtifactConflict,
  saveFileArtifact,
  selectFolderArtifactEntry,
  updateFileArtifactDraft,
  type ArtifactMutationInput,
  type ArtifactSnapshot as NativeArtifactSnapshot,
  type ArtifactWorkspaceSnapshot,
  type ArtifactReplyInput,
} from "../../platform/artifact-service";
import { pickNative } from "../../platform/platform-service";
import type {
  ArtifactId,
  AttachmentId,
  AttemptId,
  PickerGrantId,
  ProjectId,
  SessionId,
} from "../../platform/protocol";
import { NativePlatformSettingsContent } from "../platform";
import {
  selectComposerDraft,
  selectComposerProjection,
  selectConversation,
  selectLaunch,
  selectPlatform,
  selectRuntime,
  selectSettingsProjection,
  selectSettingsReturnState,
  selectSessions,
  selectWorkspace,
  selectWorkspaceUiDraft,
  shellPanelBounds,
  shellDraftActions,
  UNINITIALIZED_GENERATION,
} from "./state";
import {
  ShellView,
  isSettingsRoute,
  type ShellFocusRestoreRequest,
  type ShellFocusTarget,
  type ShellRoutePath,
} from "./ui";
import { publishConversationSnapshot } from "./native-bootstrap";
import { artifactWorkspaceForActiveSession } from "./artifact-session";

interface ShellRouteControllerProps {
  readonly route: ShellRoutePath;
}

type ShellNavigationState = {
  readonly focusRestoreRequest?: ShellFocusRestoreRequest;
};

let focusRequestSequence = 0;

/** Binds the accepted shell view to the single application router and store. */
export function ShellRouteController({ route }: ShellRouteControllerProps) {
  const dispatch = useAppDispatch();
  const store = useAppStore();
  const navigate = useNavigate();
  const location = useLocation();
  const workspace = useAppSelector(selectWorkspace);
  const launch = useAppSelector(selectLaunch);
  const platform = useAppSelector(selectPlatform);
  const runtime = useAppSelector(selectRuntime);
  const sessions = useAppSelector(selectSessions);
  const conversation = useAppSelector(selectConversation);
  const settings = useAppSelector(selectSettingsProjection);
  const uiDraft = useAppSelector(selectWorkspaceUiDraft);
  const composerDraft = useAppSelector(selectComposerDraft);
  const composerProjection = useAppSelector(selectComposerProjection);
  const settingsReturn = useAppSelector(selectSettingsReturnState);
  const qaEnabled = useAppSelector((state) => state.shellQa.enabled);
  const viewportWidth = useViewportWidth();
  const overlayPanel = viewportWidth <= 992;
  const navigationState = location.state as ShellNavigationState | null;
  const focusRestoreRequest = navigationState?.focusRestoreRequest ?? null;
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedProjects, setExpandedProjects] =
    useState<ReadonlySet<string> | null>(null);
  const [pendingPickerGrantIds, setPendingPickerGrantIds] = useState<
    readonly PickerGrantId[]
  >([]);
  const [artifactWorkspace, setArtifactWorkspace] =
    useState<ArtifactWorkspaceSnapshot | null>(null);
  const activeArtifactWorkspace = artifactWorkspaceForActiveSession(
    artifactWorkspace,
    sessions.value.activeSessionId,
  );
  const artifactWorkspaceRef = useRef<ArtifactWorkspaceSnapshot | null>(null);
  const artifactOperationQueue = useRef<Promise<unknown>>(Promise.resolve());
  const artifactDraftTimers = useRef(new Map<string, number>());
  const artifactDraftOverridesRef = useRef<Record<string, string>>({});
  const [artifactDraftOverrides, setArtifactDraftOverrides] = useState<
    Readonly<Record<string, string>>
  >({});
  const replyReference = useMemo(
    () =>
      composerDraft.replyTargetId === null
        ? null
        : replyReferenceFromProjection(
            conversation.value.turns,
            composerDraft.replyTargetId,
            activeArtifactWorkspace?.artifacts ?? [],
          ),
    [
      activeArtifactWorkspace?.artifacts,
      composerDraft.replyTargetId,
      conversation.value.turns,
    ],
  );
  const [workExpanded, setWorkExpanded] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const [provenanceExpanded, setProvenanceExpanded] = useState<
    ReadonlySet<string>
  >(() => new Set());
  const [conversationBusy, setConversationBusy] = useState(false);
  const [conversationError, setConversationError] = useState<string | null>(
    null,
  );
  const [completionAnnouncements, setCompletionAnnouncements] = useState<
    ReadonlySet<string>
  >(() => new Set());
  const [attachmentPreviews, setAttachmentPreviews] = useState<
    Readonly<Record<string, string>>
  >({});
  const attachmentPreviewRequests = useRef(new Set<string>());
  const selectableModels = useMemo(
    () => composerProjection.value.models,
    [composerProjection.value.models],
  );
  const [requestedModelKey, setSelectedModelKey] = useState("");
  const defaultSelectableModel =
    selectableModels.find(({ selected }) => selected) ??
    selectableModels.find(({ available }) => available) ??
    selectableModels.at(0);
  const selectedModelKey = selectableModels.some(
    (model) => conversationModelKey(model) === requestedModelKey,
  )
    ? requestedModelKey
    : defaultSelectableModel
      ? conversationModelKey(defaultSelectableModel)
      : "";
  const activeComposerModel = selectableModels.find(
    (model) => conversationModelKey(model) === selectedModelKey,
  );
  const [requestedReasoning, setSelectedReasoning] =
    useState<ReasoningEffort | null>(null);
  const selectedReasoning =
    activeComposerModel?.available && activeComposerModel.supportsReasoning
      ? (requestedReasoning ?? composerProjection.value.activeReasoningEffort)
      : null;
  const [dismissedConflictAttachmentId, setDismissedConflictAttachmentId] =
    useState<string | null>(null);
  const persistedDraftSignature = useRef("");
  const attemptStatuses = useRef(new Map<string, string>());
  const priorActiveAttemptId = useRef(conversation.value.activeAttemptId);
  const panelOpen = overlayPanel
    ? !uiDraft.panel.collapsed && uiDraft.panel.overlayOpen
    : !uiDraft.panel.collapsed;
  const panelBounds = shellPanelBounds(viewportWidth);
  const panelWidth = Math.round(
    Math.min(
      panelBounds.maximum,
      Math.max(panelBounds.minimum, uiDraft.panel.width),
    ),
  );

  useEffect(() => {
    const target = uiDraft.focusRestoreTarget;
    if (uiDraft.focusedArtifactId !== null || target === null) return;
    const trigger = Array.from(
      document.querySelectorAll<HTMLButtonElement>(
        "[data-artifact-focus-trigger]",
      ),
    ).find((candidate) => candidate.dataset.artifactFocusTrigger === target);
    trigger?.focus();
    dispatch(shellDraftActions.focusRestoreTargetCleared());
  }, [dispatch, uiDraft.focusRestoreTarget, uiDraft.focusedArtifactId]);

  const publishConversation = useCallback(
    (snapshot: ConversationSnapshot) => {
      const completed = new Set<string>();
      for (const attempt of snapshot.activeConversation?.attempts ?? []) {
        const prior = attemptStatuses.current.get(attempt.attemptId);
        if (
          attempt.status === "completed" &&
          (prior === "starting" ||
            prior === "working" ||
            prior === "cancelling")
        ) {
          completed.add(attempt.attemptId);
        }
        attemptStatuses.current.set(attempt.attemptId, attempt.status);
      }
      setCompletionAnnouncements(completed);
      publishConversationSnapshot(dispatch, snapshot);
    },
    [dispatch],
  );

  const publishArtifactWorkspace = useCallback(
    (snapshot: ArtifactWorkspaceSnapshot) => {
      artifactWorkspaceRef.current = snapshot;
      setArtifactWorkspace(snapshot);
      const liveIds = new Set(
        snapshot.artifacts.map(({ artifactId }) => artifactId),
      );
      setArtifactDraftOverrides((current) =>
        Object.fromEntries(
          Object.entries(current).filter(([artifactId]) =>
            liveIds.has(artifactId as ArtifactId),
          ),
        ),
      );
      if (snapshot.focusedArtifactId === null) {
        dispatch(shellDraftActions.chatRestored());
      } else {
        dispatch(
          shellDraftActions.artifactFocused({
            artifactId: snapshot.focusedArtifactId,
            restoreTarget: snapshot.focusedArtifactId,
          }),
        );
      }
    },
    [dispatch],
  );

  const queueArtifactWorkspaceOperation = useCallback(
    (
      operation: () => Promise<ArtifactWorkspaceSnapshot>,
      options: { readonly rebaseConversation?: boolean } = {},
    ): Promise<ArtifactWorkspaceSnapshot> => {
      const queued = artifactOperationQueue.current
        .catch(() => undefined)
        .then(operation)
        .then(async (snapshot) => {
          const conversationSnapshot = options.rebaseConversation
            ? await readConversationSnapshot()
            : null;
          publishArtifactWorkspace(snapshot);
          if (conversationSnapshot !== null) {
            publishConversation(conversationSnapshot);
          }
          return snapshot;
        });
      artifactOperationQueue.current = queued;
      queued.catch((error: unknown) => setConversationError(messageFor(error)));
      return queued;
    },
    [publishArtifactWorkspace, publishConversation, setConversationError],
  );

  const queueArtifactMutation = useCallback(
    (
      artifactId: string,
      operation: (
        input: ArtifactMutationInput,
      ) => Promise<ArtifactWorkspaceSnapshot>,
    ): Promise<ArtifactWorkspaceSnapshot> =>
      queueArtifactWorkspaceOperation(
        async () => {
          // Conversation and Artifact commands share the Workspace generation.
          // Catch up after any intervening Conversation mutation before deriving
          // the exact record revision for this Artifact mutation.
          const latest = await readArtifactWorkspaceSnapshot();
          const artifact = latest.artifacts.find(
            (candidate) => candidate.artifactId === artifactId,
          );
          if (artifact === undefined) {
            throw new Error("The Artifact is no longer available.");
          }
          return operation({
            artifactId: artifact.artifactId,
            baseRecordRevision: artifact.recordRevision,
          });
        },
        { rebaseConversation: true },
      ),
    [queueArtifactWorkspaceOperation],
  );

  useEffect(() => {
    if (sessions.value.activeSessionId === null) {
      artifactWorkspaceRef.current = null;
      dispatch(shellDraftActions.chatRestored());
      return;
    }
    void queueArtifactWorkspaceOperation(readArtifactWorkspaceSnapshot);
  }, [
    dispatch,
    queueArtifactWorkspaceOperation,
    sessions.value.activeSessionId,
    workspace.value.activeProjectId,
  ]);

  useEffect(() => {
    const prior = priorActiveAttemptId.current;
    const current = conversation.value.activeAttemptId;
    priorActiveAttemptId.current = current;
    if (
      prior === null ||
      current !== null ||
      sessions.value.activeSessionId === null
    ) {
      return;
    }
    // A terminal File Reply may have produced a Rust-reconciled proposal.
    // Refresh only after the durable run closes so streaming never replaces
    // artifact state with partial assistant output.
    void queueArtifactWorkspaceOperation(readArtifactWorkspaceSnapshot);
  }, [
    conversation.value.activeAttemptId,
    queueArtifactWorkspaceOperation,
    sessions.value.activeSessionId,
  ]);

  useEffect(
    () => () => {
      for (const timer of artifactDraftTimers.current.values()) {
        window.clearTimeout(timer);
      }
      artifactDraftTimers.current.clear();
    },
    [],
  );

  const syncPendingAttachments = useCallback(
    (
      snapshot: ConversationSnapshot,
      replyReconciliation:
        | {
            readonly expectedReplyTargetId: string | null;
            readonly force: boolean;
          }
        | undefined = undefined,
    ) => {
      dispatch(
        shellDraftActions.composerAttachmentsReconciled({
          attachments: snapshot.draft.attachments.map((attachment) => ({
            id: attachment.attachmentId,
            name: attachment.displayName,
            byteLength: attachment.byteLength,
            mediaType: attachment.mediaType,
            stableReference: attachment.stableReference,
            referenceNumber: attachment.originalReference,
            compatibility: "ready",
          })),
          nextAttachmentReference: snapshot.draft.nextAttachmentReference,
        }),
      );
      dispatch(shellDraftActions.composerTextChanged(snapshot.draft.prompt));
      dispatch(shellDraftActions.composerModeChanged(snapshot.draft.mode));
      if (replyReconciliation?.force) {
        dispatch(
          shellDraftActions.composerReplyChanged(snapshot.draft.replyTargetId),
        );
      } else if (replyReconciliation) {
        dispatch(
          shellDraftActions.composerReplyReconciled({
            expectedReplyTargetId: replyReconciliation.expectedReplyTargetId,
            authoritativeReplyTargetId: snapshot.draft.replyTargetId,
          }),
        );
      }
      if (snapshot.draft.providerId && snapshot.draft.modelId) {
        setSelectedModelKey(
          conversationModelKey({
            providerId: snapshot.draft.providerId,
            modelId: snapshot.draft.modelId,
          }),
        );
      }
      setSelectedReasoning(snapshot.draft.reasoningMode);
      persistedDraftSignature.current = conversationDraftSignature({
        prompt: snapshot.draft.prompt,
        attachmentIds: snapshot.draft.attachments.map(
          (attachment) => attachment.attachmentId,
        ),
        providerId: snapshot.draft.providerId,
        modelId: snapshot.draft.modelId,
        reasoningMode: snapshot.draft.reasoningMode,
        mode: snapshot.draft.mode,
        replyTargetId: snapshot.draft.replyTargetId,
      });
      setPendingPickerGrantIds([]);
    },
    [dispatch],
  );

  const beginChat = async (projectId: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const snapshot = await beginPendingConversation(projectId as ProjectId);
      dispatch(shellDraftActions.chatRestored());
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const cancelPendingChat = async () => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const snapshot = await cancelPendingConversation();
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const activateSession = async (sessionId: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const snapshot = await activateConversationSession(
        sessionId as SessionId,
      );
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const activateProject = async (projectId: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const snapshot = await activateConversationProject(
        projectId as ProjectId,
      );
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
      setExpandedProjects(
        (current) =>
          new Set([
            ...(current ??
              workspace.value.projects.map((project) => String(project.id))),
            projectId,
          ]),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const addProject = async () => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const outcome = await pickNative("openProjectFolder");
      if (outcome.type === "cancelled") return;
      const grant = outcome.grants.at(0);
      if (grant === undefined) throw new Error("Project selection is empty.");
      const snapshot = await addConversationProject(grant.grantId);
      publishConversation(snapshot);
      setExpandedProjects(
        new Set(snapshot.projects.map(({ projectId }) => projectId)),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const relocateProject = async (projectId: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const outcome = await pickNative("relocateProjectFolder");
      if (outcome.type === "cancelled") return;
      const grant = outcome.grants.at(0);
      if (grant === undefined) throw new Error("Project selection is empty.");
      publishConversation(
        await relocateConversationProject(
          projectId as ProjectId,
          grant.grantId,
        ),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const reorderProjects = async (projectIds: readonly string[]) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      publishConversation(
        await reorderConversationProjects(projectIds as readonly ProjectId[]),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const removeProject = async (projectId: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const snapshot = await inactivateConversationProject(
        projectId as ProjectId,
      );
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const copyProjectPath = async (projectId: string) => {
    setConversationError(null);
    try {
      publishConversation(
        await copyConversationProjectPath(projectId as ProjectId),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    }
  };

  const revealProject = async (projectId: string) => {
    setConversationError(null);
    try {
      publishConversation(
        await revealConversationProject(projectId as ProjectId),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    }
  };

  const renameProject = async (projectId: string, displayName: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      publishConversation(
        await renameConversationProject(projectId as ProjectId, displayName),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const removeSession = async (sessionId: string) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      const snapshot = await inactivateConversationSession(
        sessionId as SessionId,
      );
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const requestBranch = async (
    operation: "switch" | "create",
    branch: string,
  ) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      publishConversation(
        await requestConversationBranch({ operation, branch }),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const answerBranchApproval = async (
    promptId: string,
    answer: "allow" | "deny",
  ) => {
    setConversationBusy(true);
    setConversationError(null);
    try {
      publishConversation(
        await answerConversationBranchApproval({ promptId, answer }),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const attachPickerGrants = async (
    pickerGrantIds: readonly PickerGrantId[],
  ) => {
    const expectedReplyTargetId = composerDraft.replyTargetId;
    setConversationBusy(true);
    setConversationError(null);
    try {
      const selectedModel = selectableModels.find(
        (model) => conversationModelKey(model) === selectedModelKey,
      );
      const snapshot = await updateConversationDraft({
        prompt: composerDraft.text,
        pickerGrantIds,
        retainedAttachmentIds: composerDraft.attachments.map(
          (attachment) => attachment.id,
        ),
        providerId: selectedModel?.providerId ?? null,
        modelId: selectedModel?.modelId ?? null,
        reasoningMode: selectedReasoning,
        mode: replyReference ? "chat" : composerDraft.mode,
        replyTargetId: replyReference?.id ?? null,
      });
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId,
        force: false,
      });
    } catch (error) {
      setConversationError(messageFor(error));
    } finally {
      setConversationBusy(false);
    }
  };

  const attachFiles = async () => {
    setConversationError(null);
    try {
      const outcome = await pickNative("attachChatFiles");
      if (outcome.type === "cancelled") return;
      await attachPickerGrants(outcome.grants.map(({ grantId }) => grantId));
    } catch (error) {
      setConversationError(messageFor(error));
    }
  };

  const browseArtifact = async (kind: "file" | "folder") => {
    setConversationError(null);
    try {
      const outcome = await pickNative(
        kind === "file" ? "openFile" : "openFolder",
      );
      if (outcome.type === "cancelled") return;
      const grant = outcome.grants.at(0);
      if (grant === undefined) throw new Error("Artifact selection is empty.");
      await queueArtifactWorkspaceOperation(
        async () => {
          // A native picker can remain open while an unrelated Workspace watcher
          // advances the enclosing snapshot generation. Rebase the Artifact
          // adapter after the picker returns and before consuming its one-use
          // grant; Rust still performs the exact mutation CAS.
          await readArtifactWorkspaceSnapshot();
          return kind === "file"
            ? openFileArtifact(grant.grantId)
            : openFolderArtifact(grant.grantId);
        },
        { rebaseConversation: true },
      );
    } catch (error) {
      setConversationError(messageFor(error));
    }
  };

  const clearArtifactDraftOverride = useCallback((artifactId: string) => {
    const timer = artifactDraftTimers.current.get(artifactId);
    if (timer !== undefined) window.clearTimeout(timer);
    artifactDraftTimers.current.delete(artifactId);
    const next = { ...artifactDraftOverridesRef.current };
    delete next[artifactId];
    artifactDraftOverridesRef.current = next;
    setArtifactDraftOverrides(next);
  }, []);

  const retainArtifactDraft = useCallback(
    (artifactId: string, content: string) => {
      const next = {
        ...artifactDraftOverridesRef.current,
        [artifactId]: content,
      };
      artifactDraftOverridesRef.current = next;
      setArtifactDraftOverrides(next);
      const currentTimer = artifactDraftTimers.current.get(artifactId);
      if (currentTimer !== undefined) window.clearTimeout(currentTimer);
      artifactDraftTimers.current.set(
        artifactId,
        window.setTimeout(() => {
          artifactDraftTimers.current.delete(artifactId);
          void queueArtifactMutation(artifactId, (input) =>
            updateFileArtifactDraft({ ...input, content }),
          ).then(() => {
            if (artifactDraftOverridesRef.current[artifactId] === content) {
              clearArtifactDraftOverride(artifactId);
            }
          });
        }, 250),
      );
    },
    [clearArtifactDraftOverride, queueArtifactMutation],
  );

  const saveArtifactDraft = useCallback(
    async (artifactId: string, fallback: string) => {
      const content = artifactDraftOverridesRef.current[artifactId] ?? fallback;
      const timer = artifactDraftTimers.current.get(artifactId);
      if (timer !== undefined) {
        window.clearTimeout(timer);
        artifactDraftTimers.current.delete(artifactId);
      }
      const current = artifactWorkspaceRef.current?.artifacts.find(
        (artifact) => artifact.artifactId === artifactId,
      );
      const persistedDraft =
        current?.providerState.type === "file" &&
        (current.providerState.value.state.phase === "edit" ||
          current.providerState.value.state.phase === "dirty")
          ? current.providerState.value.state.draft
          : null;
      if (persistedDraft !== content) {
        await queueArtifactMutation(artifactId, (input) =>
          updateFileArtifactDraft({ ...input, content }),
        );
      }
      clearArtifactDraftOverride(artifactId);
      await queueArtifactMutation(artifactId, saveFileArtifact);
    },
    [clearArtifactDraftOverride, queueArtifactMutation],
  );

  const selectArtifactForReply = useCallback(
    async (
      artifactId: string,
      selection?: Pick<ArtifactReplyInput, "selectedText" | "selectedEntryId">,
    ) => {
      await queueArtifactMutation(artifactId, (input) =>
        replyToArtifact({ ...input, ...selection }),
      );
      const snapshot = await readConversationSnapshot();
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId: composerDraft.replyTargetId,
        force: true,
      });
    },
    [
      composerDraft.replyTargetId,
      publishConversation,
      queueArtifactMutation,
      syncPendingAttachments,
    ],
  );

  const focusNativeArtifact = useCallback(
    (artifactId: string) => queueArtifactMutation(artifactId, focusArtifact),
    [queueArtifactMutation],
  );

  const closeNativeArtifactFocus = useCallback(
    () =>
      queueArtifactWorkspaceOperation(
        async () => {
          await readArtifactWorkspaceSnapshot();
          return closeArtifactFocus();
        },
        { rebaseConversation: true },
      ),
    [queueArtifactWorkspaceOperation],
  );

  const decideArtifactApproval = useCallback(
    (promptId: string, answer: "allow" | "deny") =>
      queueArtifactWorkspaceOperation(
        async () => {
          await readArtifactWorkspaceSnapshot();
          return answerArtifactApproval({ promptId, answer });
        },
        { rebaseConversation: true },
      ),
    [queueArtifactWorkspaceOperation],
  );

  useEffect(() => {
    if (
      sessions.value.activeSessionId === null ||
      conversationBusy ||
      pendingPickerGrantIds.length > 0
    ) {
      return;
    }
    const selectedModel = selectableModels.find(
      (model) => conversationModelKey(model) === selectedModelKey,
    );
    const signature = conversationDraftSignature({
      prompt: composerDraft.text,
      attachmentIds: composerDraft.attachments.map(
        (attachment) => attachment.id,
      ),
      providerId: selectedModel?.providerId ?? null,
      modelId: selectedModel?.modelId ?? null,
      reasoningMode: selectedReasoning,
      mode: replyReference ? "chat" : composerDraft.mode,
      replyTargetId: replyReference?.id ?? null,
    });
    if (signature === persistedDraftSignature.current) return;
    const timer = window.setTimeout(() => {
      void updateConversationDraft({
        prompt: composerDraft.text,
        pickerGrantIds: [],
        retainedAttachmentIds: composerDraft.attachments.map(
          (attachment) => attachment.id,
        ),
        providerId: selectedModel?.providerId ?? null,
        modelId: selectedModel?.modelId ?? null,
        reasoningMode: selectedReasoning,
        mode: replyReference ? "chat" : composerDraft.mode,
        replyTargetId: replyReference?.id ?? null,
      })
        .then((snapshot) => {
          persistedDraftSignature.current = signature;
          publishConversation(snapshot);
        })
        .catch((error: unknown) => setConversationError(messageFor(error)));
    }, 300);
    return () => window.clearTimeout(timer);
  }, [
    composerDraft.attachments,
    composerDraft.mode,
    composerDraft.text,
    conversationBusy,
    pendingPickerGrantIds,
    publishConversation,
    replyReference,
    selectableModels,
    selectedModelKey,
    selectedReasoning,
    sessions.value.activeSessionId,
  ]);

  const submitChat = async (source: string) => {
    const expectedReplyTargetId = composerDraft.replyTargetId;
    setConversationBusy(true);
    setConversationError(null);
    try {
      const selectedModel = selectableModels.find(
        (model) => conversationModelKey(model) === selectedModelKey,
      );
      const snapshot = await submitConversation({
        prompt: source.trim().length > 0 ? source : null,
        pickerGrantIds: pendingPickerGrantIds,
        retainedAttachmentIds: composerDraft.attachments
          .filter(
            (attachment) =>
              !pendingPickerGrantIds.some(
                (grantId) => grantId === (attachment.id as unknown as string),
              ),
          )
          .map((attachment) => attachment.id),
        providerId: selectedModel?.providerId ?? null,
        modelId: selectedModel?.modelId ?? null,
        reasoningMode: selectedReasoning,
        resumeMode: composerDraft.mode,
      });
      publishConversation(snapshot);
      syncPendingAttachments(snapshot, {
        expectedReplyTargetId,
        force: false,
      });
      dispatch(shellDraftActions.composerTextChanged(""));
    } catch (error) {
      setConversationError(messageFor(error));
      try {
        const snapshot = await readConversationSnapshot();
        publishConversation(snapshot);
        syncPendingAttachments(snapshot, {
          expectedReplyTargetId,
          force: false,
        });
      } catch {
        // The original bounded service error remains the useful user state.
      }
    } finally {
      setConversationBusy(false);
    }
  };

  const cancelAttempt = async (attemptId: string) => {
    setConversationError(null);
    try {
      publishConversation(
        await cancelConversationAttempt(attemptId as AttemptId),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    }
  };

  useEffect(() => {
    let disposed = false;
    for (const attachment of composerDraft.attachments) {
      if (
        attachmentPreviews[attachment.id] !== undefined ||
        attachmentPreviewRequests.current.has(attachment.id) ||
        attachment.stableReference === undefined ||
        attachment.byteLength === undefined ||
        attachment.byteLength > 2 * 1024 * 1024 ||
        !new Set(["image/png", "image/jpeg", "image/gif", "image/webp"]).has(
          attachment.mediaType ?? "",
        )
      ) {
        continue;
      }
      attachmentPreviewRequests.current.add(attachment.id);
      void previewConversationAttachment({
        attachmentId: attachment.id,
        stableReference: attachment.stableReference,
      })
        .then((preview) => {
          if (disposed) return;
          setAttachmentPreviews((current) => ({
            ...current,
            [preview.attachmentId]: preview.dataUrl,
          }));
        })
        .catch(() => {
          // Integrity, size, or stale-generation failures keep the safe file icon.
        })
        .finally(() => {
          attachmentPreviewRequests.current.delete(attachment.id);
        });
    }
    return () => {
      disposed = true;
    };
  }, [attachmentPreviews, composerDraft.attachments]);

  const retryAttempt = async (parentAttemptId: string) => {
    const selectedModel = selectableModels.find(
      (model) => conversationModelKey(model) === selectedModelKey,
    );
    setConversationError(null);
    try {
      publishConversation(
        await retryConversationAttempt({
          parentAttemptId: parentAttemptId as AttemptId,
          providerId: selectedModel?.providerId ?? null,
          modelId: selectedModel?.modelId ?? null,
          reasoningMode: selectedReasoning,
        }),
      );
    } catch (error) {
      setConversationError(messageFor(error));
    }
  };

  useEffect(() => {
    if (conversation.value.activeAttemptId === null) return;
    let disposed = false;
    let inFlight = false;
    const refresh = () => {
      if (disposed || inFlight) return;
      inFlight = true;
      void readConversationSnapshot()
        .then((snapshot) => {
          if (!disposed) publishConversation(snapshot);
        })
        .catch((error: unknown) => {
          if (!disposed) setConversationError(messageFor(error));
        })
        .finally(() => {
          inFlight = false;
        });
    };
    refresh();
    const timer = window.setInterval(refresh, 150);
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, [conversation.value.activeAttemptId, publishConversation]);

  useEffect(() => {
    if (panelWidth === uiDraft.panel.width) return;
    dispatch(
      shellDraftActions.leftPanelResized({
        width: panelWidth,
        viewportWidth,
      }),
    );
  }, [dispatch, panelWidth, uiDraft.panel.width, viewportWidth]);

  const navigateWithinShell = (destination: ShellRoutePath) => {
    if (isSettingsRoute(destination)) {
      dispatch(
        shellDraftActions.settingsSectionChanged(
          settingsSectionForRoute(destination),
        ),
      );
    }
    void navigate(destination);
  };

  const visitSettings = (focusTarget: ShellFocusTarget) => {
    dispatch(
      shellDraftActions.settingsVisited(
        createSettingsVisit(store.getState(), route, focusTarget),
      ),
    );
    void navigate("/settings/providers");
  };

  const returnFromSettings = () => {
    const destination = settingsReturn?.route ?? "/start";
    const focusTarget = asShellFocusTarget(settingsReturn?.focusTarget ?? null);
    dispatch(shellDraftActions.settingsVisitEnded());
    if (focusTarget === null) {
      void navigate(destination);
      return;
    }
    focusRequestSequence += 1;
    void navigate(destination, {
      state: {
        focusRestoreRequest: {
          requestId: focusRequestSequence,
          target: focusTarget,
        },
      } satisfies ShellNavigationState,
    });
  };

  const changePanelOpen = (isOpen: boolean) => {
    if (overlayPanel) {
      dispatch(shellDraftActions.leftPanelOverlayChanged(isOpen));
    } else {
      dispatch(shellDraftActions.leftPanelCollapsed(!isOpen));
    }
  };

  const projectItems = workspace.value.projects.map((project) => ({
    id: project.id as string,
    name: project.name,
    pathState: project.pathState,
    isExpanded:
      expandedProjects?.has(project.id as string) ??
      workspace.value.projects.length > 0,
    sessions: sessions.value.sessions
      .filter((session) => session.projectId === project.id)
      .map((session) => ({ id: session.id as string, title: session.title })),
  }));
  const renderNativeArtifact = useCallback(
    (
      nativeArtifact: NativeArtifactSnapshot,
      context: ArtifactContext,
    ): ReactNode => {
      const common = {
        onClose: () => void closeNativeArtifactFocus(),
        onCopy: (_artifactId: string, value: string) =>
          void navigator.clipboard?.writeText(value),
        onExpand: (artifactId: string) => void focusNativeArtifact(artifactId),
      };
      if (nativeArtifact.providerState.type === "file") {
        const file = nativeArtifact.providerState.value;
        let state: FileArtifactState;
        if (file.state.phase === "approval") {
          state = {
            ...file.state,
            approvalId:
              nativeArtifact.pendingApprovalId ?? "unavailable-approval",
          };
        } else if (
          (file.state.phase === "edit" || file.state.phase === "dirty") &&
          artifactDraftOverrides[nativeArtifact.artifactId] !== undefined
        ) {
          const draft =
            artifactDraftOverrides[nativeArtifact.artifactId] ??
            file.state.draft;
          state = {
            phase: draft === file.state.content ? "edit" : "dirty",
            content: file.state.content,
            draft,
          };
        } else {
          state = file.state;
        }
        return (
          <FileArtifact
            context={context}
            model={{
              artifactId: nativeArtifact.artifactId,
              breadcrumbs: file.breadcrumbs,
              ...(file.languageLabel === null
                ? {}
                : { languageLabel: file.languageLabel }),
              state,
              status: nativeArtifact.status,
              title: nativeArtifact.title,
              ...(file.versionLabel === null
                ? {}
                : { versionLabel: file.versionLabel }),
            }}
            {...common}
            onReply={(artifactId, selection) =>
              void selectArtifactForReply(artifactId, selection)
            }
            onAllowApproval={(_artifactId, approvalId) =>
              void decideArtifactApproval(approvalId, "allow")
            }
            onApproveProposal={() =>
              void queueArtifactMutation(
                nativeArtifact.artifactId,
                saveFileArtifact,
              )
            }
            onBreadcrumbSelect={(_artifactId, breadcrumbId) =>
              void queueArtifactMutation(nativeArtifact.artifactId, (input) =>
                navigateFolderArtifact({
                  ...input,
                  projectRelativePath: breadcrumbId,
                }),
              )
            }
            onDenyApproval={(_artifactId, approvalId) =>
              void decideArtifactApproval(approvalId, "deny")
            }
            onDiscard={(artifactId) => {
              clearArtifactDraftOverride(artifactId);
              void queueArtifactMutation(artifactId, discardFileArtifactDraft);
            }}
            onDraftChange={retainArtifactDraft}
            onEdit={(artifactId) =>
              void queueArtifactMutation(artifactId, beginFileArtifactEdit)
            }
            onRecoverDraft={(artifactId) =>
              void saveArtifactDraft(
                artifactId,
                file.state.phase === "recovery"
                  ? file.state.draft
                  : file.state.content,
              )
            }
            onRejectProposal={(artifactId) =>
              void queueArtifactMutation(artifactId, rejectFileArtifactProposal)
            }
            onResolveConflict={(artifactId, resolution) => {
              clearArtifactDraftOverride(artifactId);
              void queueArtifactMutation(artifactId, (input) =>
                resolveFileArtifactConflict({
                  ...input,
                  resolution:
                    resolution === "reload-current"
                      ? "reloadCurrent"
                      : "keepDraft",
                }),
              );
            }}
            onSave={(artifactId, content) =>
              void saveArtifactDraft(artifactId, content)
            }
          />
        );
      }
      if (nativeArtifact.providerState.type === "folder") {
        const folder = nativeArtifact.providerState.value;
        const listing =
          folder.listing.phase === "ready"
            ? ({ phase: "ready" } as const)
            : ({
                phase: folder.listing.phase,
                message:
                  folder.listing.message ?? "Folder listing is unavailable.",
              } as const);
        return (
          <FolderArtifact
            context={context}
            model={{
              artifactId: nativeArtifact.artifactId,
              breadcrumbs: folder.breadcrumbs,
              entries: folder.entries.map((entry) => ({
                id: entry.id,
                kind: entry.kind,
                name: entry.name,
                ...(entry.metadata === null
                  ? {}
                  : { metadata: entry.metadata }),
              })),
              listing,
              listingLimit: folder.listingLimit,
              ...(folder.selectedEntryId === null
                ? {}
                : { selectedEntryId: folder.selectedEntryId }),
              status: nativeArtifact.status,
              title: nativeArtifact.title,
            }}
            {...common}
            onReply={(artifactId, selection) =>
              void selectArtifactForReply(artifactId, selection)
            }
            onBreadcrumbSelect={(_artifactId, breadcrumbId) =>
              void queueArtifactMutation(nativeArtifact.artifactId, (input) =>
                navigateFolderArtifact({
                  ...input,
                  projectRelativePath: breadcrumbId,
                }),
              )
            }
            onConvertFile={(_artifactId, entryId) =>
              void queueArtifactMutation(nativeArtifact.artifactId, (input) =>
                selectFolderArtifactEntry({ ...input, entryId }),
              )
            }
            onNavigateFolder={(_artifactId, entryId) =>
              void queueArtifactMutation(nativeArtifact.artifactId, (input) =>
                selectFolderArtifactEntry({ ...input, entryId }),
              )
            }
            onRefresh={(artifactId) =>
              void queueArtifactMutation(artifactId, refreshFolderArtifact)
            }
          />
        );
      }
      return (
        <UnknownArtifact
          artifactId={nativeArtifact.artifactId}
          context="inline"
          message={
            nativeArtifact.status.message ??
            "This artifact provider version is not supported."
          }
          onCopy={() =>
            void navigator.clipboard?.writeText(nativeArtifact.sourceLabel)
          }
          requestedType={nativeArtifact.providerType}
          requestedVersion={nativeArtifact.providerVersion}
          title={nativeArtifact.title}
        />
      );
    },
    [
      artifactDraftOverrides,
      clearArtifactDraftOverride,
      closeNativeArtifactFocus,
      decideArtifactApproval,
      focusNativeArtifact,
      queueArtifactMutation,
      retainArtifactDraft,
      saveArtifactDraft,
      selectArtifactForReply,
    ],
  );

  const conversationTranscriptTurns: ConversationTranscriptTurn[] =
    conversation.value.turns.map((turn) => {
      if (turn.author === "user") {
        return {
          id: turn.id,
          author: "user",
          markdownSource: turn.markdown,
          status: turn.status,
          ...(turn.attachments === undefined
            ? {}
            : {
                attachments: turn.attachments.map((attachment) => ({
                  id: attachment.id,
                  name: attachment.name,
                  metadata: `${attachmentExtension(attachment.name)} · ${formatByteLength(attachment.byteLength)}`,
                  referenceNumber: attachment.referenceNumber,
                })),
              }),
          ...(turn.artifactContext === undefined
            ? {}
            : {
                replyContext: {
                  artifactId: turn.artifactContext.artifactId,
                  providerType: turn.artifactContext.providerType,
                  providerVersion: turn.artifactContext.providerVersion,
                  recordRevision: turn.artifactContext.artifactRecordRevision,
                  stableReference: turn.artifactContext.stableReference,
                  suppliedBytes: turn.artifactContext.usedBytes,
                  maximumBytes: turn.artifactContext.maximumBytes,
                  omittedBytes: turn.artifactContext.omittedBytes,
                  truncated: turn.artifactContext.truncated,
                  unsaved: turn.artifactContext.unsaved,
                  segments: turn.artifactContext.segments.map((segment) => ({
                    source: segment.source,
                    text: segment.text,
                    omittedBytes: segment.omittedBytes,
                  })),
                  capabilities: turn.artifactContext.capabilities,
                },
              }),
        };
      }
      const workActivityId = `activity:${turn.id}`;
      const hasWorkActivity = (turn.activities?.length ?? 0) > 0;
      return {
        id: turn.id,
        author: "assistant",
        markdownSource: turn.markdown,
        status: turn.status,
        modelLabel: turn.modelLabel ?? "Selected model",
        responseVisible:
          turn.markdown.length > 0 || turn.status !== "streaming",
        announceCompletion: completionAnnouncements.has(turn.id),
        work: {
          kind: turn.activities?.some(
            (activity) => activity.kind === "reasoning-summary",
          )
            ? "reasoning"
            : "activity",
          summary:
            turn.status === "streaming" ? "Working…" : "Activity complete",
          ...(turn.durationMs === undefined
            ? {}
            : { durationLabel: formatWorkDuration(turn.durationMs) }),
          isExpanded: workExpanded.has(turn.id),
          progress: [],
          details: (turn.activities ?? []).map((activity) => ({
            id: activity.id,
            label: activity.label,
            ...(activity.detail === undefined
              ? {}
              : { detail: activity.detail }),
            state: activity.state,
          })),
        },
        provenance: {
          runtime: turn.runtimeLabel ?? "Runtime",
          adapter: turn.adapterLabel ?? "Adapter",
          environment: turn.environmentLabel ?? "Local",
          capabilitySummary: "Run-bound effective snapshot",
          isExpanded: provenanceExpanded.has(turn.id),
        },
        ...(hasWorkActivity
          ? {
              artifact: {
                focusSupported: false,
                id: workActivityId,
                isFocused: false,
                summary: `${turn.activities?.length ?? 0} safe run ${turn.activities?.length === 1 ? "event" : "events"}`,
                title: "Run activity",
                type: "unknown" as const,
              },
            }
          : {}),
      };
    });
  const artifactTranscriptTurns: ConversationTranscriptTurn[] =
    activeArtifactWorkspace?.artifacts.map((nativeArtifact) => ({
      id: `artifact-turn:${nativeArtifact.artifactId}`,
      author: "assistant" as const,
      markdownSource: "",
      status: "completed" as const,
      modelLabel: "C4OS facility",
      responseVisible: true,
      work: {
        kind: "activity" as const,
        summary: "Direct operation complete",
        isExpanded: false,
        progress: [],
        details: [],
      },
      provenance: {
        runtime: "C4OS core",
        adapter: "Native facility",
        environment: "Local Project",
        capabilitySummary: nativeArtifact.sourceLabel,
        isExpanded: false,
      },
      artifact: {
        id: nativeArtifact.artifactId,
        title: nativeArtifact.title,
        summary: nativeArtifact.sourceLabel,
        type: nativeArtifact.providerState.type,
        focusSupported: nativeArtifact.focusSupported,
        isFocused:
          activeArtifactWorkspace.focusedArtifactId ===
          nativeArtifact.artifactId,
        renderContent: (placement) =>
          renderNativeArtifact(
            nativeArtifact,
            placement === "context-pane" ? "contextual" : "inline",
          ),
      },
    })) ?? [];
  const transcriptTurns: ConversationTranscriptTurn[] = [
    ...conversationTranscriptTurns,
    ...artifactTranscriptTurns,
  ];
  const focusedNativeArtifact = activeArtifactWorkspace?.artifacts.find(
    (artifact) =>
      artifact.artifactId === activeArtifactWorkspace.focusedArtifactId,
  );
  const focusedConversationArtifact: FocusedConversationArtifact | null =
    focusedNativeArtifact === undefined
      ? null
      : {
          id: focusedNativeArtifact.artifactId,
          ownsClose: true,
          title: focusedNativeArtifact.title,
          type: focusedNativeArtifact.providerState.type,
          content: renderNativeArtifact(focusedNativeArtifact, "focused"),
        };
  const hasPendingChat = sessions.value.sessions.some(
    (session) =>
      session.id === sessions.value.activeSessionId &&
      session.lifecycle === "pending",
  );
  const canSubmitChat =
    sessions.value.activeSessionId !== null &&
    conversation.value.activeAttemptId === null &&
    activeComposerModel?.available === true;
  const composerAttachments = composerDraft.attachments.map((attachment) => {
    const compatibility = attachmentCompatibility(
      attachment.mediaType,
      activeComposerModel,
    );
    return {
      id: attachment.id as string,
      fileName: attachment.name,
      compatibility,
      referenceNumber: attachment.referenceNumber,
      ...(attachmentPreviews[attachment.id] === undefined
        ? {}
        : { previewUrl: attachmentPreviews[attachment.id] }),
      sizeLabel:
        attachment.byteLength === undefined
          ? "Selected file"
          : formatByteLength(attachment.byteLength),
    };
  });
  const incompatibleAttachment = composerAttachments.find(
    ({ compatibility }) =>
      compatibility === "needs-vision" ||
      compatibility === "needs-audio" ||
      compatibility === "incompatible",
  );
  const composerConflict =
    incompatibleAttachment === undefined ||
    dismissedConflictAttachmentId === incompatibleAttachment.id
      ? undefined
      : {
          attachmentId: incompatibleAttachment.id,
          title: "Attachment needs a compatible model",
          description: `${incompatibleAttachment.fileName} cannot be sent on the selected route as-is.`,
          actions: ["use-compatible-model", "remove-file", "cancel"] as const,
        };
  const modelControlModels: readonly ModelControlModel[] = selectableModels.map(
    (model) => ({
      capabilities: modelCapabilities(model),
      contextLabel:
        model.contextTokens > 0
          ? `${formatCompactNumber(model.contextTokens)} context`
          : "Context limit unavailable",
      id: model.modelId,
      isAvailable: model.available,
      isSelected: conversationModelKey(model) === selectedModelKey,
      name: model.modelId,
      providerId: model.providerId,
    }),
  );
  const modelControlProviders = Array.from(
    new Map(
      selectableModels.map((model) => [
        model.providerId,
        { id: model.providerId, name: model.providerName },
      ]),
    ).values(),
  );
  const latestAssistantTurn = [...conversation.value.turns]
    .reverse()
    .find((turn) => turn.author === "assistant");
  const activeRuntime = runtime.value.runtimes.find(
    (candidate) => candidate.id === latestAssistantTurn?.runtimeId,
  );
  const contextUsed =
    (latestAssistantTurn?.inputTokens ?? 0) +
    (latestAssistantTurn?.outputTokens ?? 0);
  const directModeUnavailable =
    replyReference === null && composerDraft.mode !== "chat";
  const conversationTranscript = (
    <ConversationTranscript
      onArtifactFocusRequest={(artifactId) =>
        void focusNativeArtifact(artifactId)
      }
      onCancelAttempt={(attemptId) => void cancelAttempt(attemptId)}
      onCopy={(_turnId, markdown) =>
        void navigator.clipboard?.writeText(markdown)
      }
      onProvenanceExpandedChange={(turnId, isExpanded) =>
        setProvenanceExpanded((current) =>
          changedSet(current, turnId, isExpanded),
        )
      }
      onReply={(turnId) => {
        dispatch(shellDraftActions.composerReplyChanged(turnId));
      }}
      onRetryAttempt={(attemptId) => void retryAttempt(attemptId)}
      onWorkExpandedChange={(turnId, isExpanded) =>
        setWorkExpanded((current) => changedSet(current, turnId, isExpanded))
      }
      reducedMotion={platform.value.reducedMotion}
      turns={transcriptTurns}
    />
  );

  const renderShell = (
    routeContent: ReactNode,
    contextualChatContent: ReactNode | null = null,
  ) => (
    <ShellView
      route={route}
      composer={{
        draft: composerDraft.text,
        mode: composerDraft.mode,
        isModeLocked: uiDraft.focusedArtifactId !== null,
      }}
      focusRestoreRequest={focusRestoreRequest}
      composerContent={
        <Composer
          attachments={composerAttachments}
          {...(composerConflict === undefined
            ? {}
            : { conflict: composerConflict })}
          controls={{
            model: (
              <ModelSelector
                isDisabled={conversationBusy}
                models={modelControlModels}
                onModelSelect={(model) => {
                  const nextKey = conversationModelKey({
                    providerId: model.providerId,
                    modelId: model.id,
                  });
                  setSelectedModelKey(nextKey);
                  setDismissedConflictAttachmentId(null);
                  const nextModel = selectableModels.find(
                    (model) => conversationModelKey(model) === nextKey,
                  );
                  if (!nextModel?.supportsReasoning) setSelectedReasoning(null);
                }}
                providers={modelControlProviders}
              />
            ),
            ...(activeComposerModel?.available &&
            activeComposerModel.supportsReasoning
              ? {
                  reasoning: (
                    <ReasoningEffortControl
                      isDisabled={conversationBusy}
                      onChange={(effort) => setSelectedReasoning(effort)}
                      options={["off", "low", "medium", "high"]}
                      value={selectedReasoning as ReasoningEffort | null}
                    />
                  ),
                }
              : {}),
            ...(workspace.value.projects.find(
              (project) => project.id === workspace.value.activeProjectId,
            )?.gitVersioned
              ? {
                  branch: (
                    <>
                      <GitBranchControl
                        activeBranch={
                          composerProjection.value.activeBranch ??
                          "Detached HEAD"
                        }
                        branches={composerProjection.value.branches}
                        isBusy={conversationBusy}
                        onApprove={(promptId) =>
                          void answerBranchApproval(promptId, "allow")
                        }
                        onCreate={(branch) =>
                          void requestBranch("create", branch)
                        }
                        onDeny={(promptId) =>
                          void answerBranchApproval(promptId, "deny")
                        }
                        onSwitch={(branch) =>
                          void requestBranch("switch", branch)
                        }
                        pendingApprovalId={
                          composerProjection.value.branchPendingApprovalId
                        }
                      />
                      {composerProjection.value.branchOperationMessage ? (
                        <span
                          className="conversation-branch-control__status"
                          role="status"
                        >
                          {composerProjection.value.branchOperationMessage}
                        </span>
                      ) : null}
                    </>
                  ),
                }
              : {}),
            ...(hasPendingChat
              ? {
                  approval: (
                    <button
                      type="button"
                      onClick={() => void cancelPendingChat()}
                    >
                      Cancel new chat
                    </button>
                  ),
                }
              : {}),
          }}
          isDisabled={conversationBusy}
          isModeLocked={focusedConversationArtifact !== null}
          isSubmitDisabled={
            !canSubmitChat ||
            directModeUnavailable ||
            incompatibleAttachment !== undefined
          }
          mode={replyReference ? "reply" : composerDraft.mode}
          onAttach={() => void attachFiles()}
          onBrowse={() => void browseArtifact("file")}
          onBrowseFolder={() => void browseArtifact("folder")}
          onModeChange={(mode) =>
            dispatch(shellDraftActions.composerModeChanged(mode))
          }
          onConflictAction={(action) => {
            if (incompatibleAttachment === undefined) return;
            if (action === "remove-file") {
              dispatch(
                shellDraftActions.composerAttachmentRemoved(
                  incompatibleAttachment.id as AttachmentId,
                ),
              );
              setDismissedConflictAttachmentId(null);
              return;
            }
            if (action === "use-compatible-model") {
              const attachment = composerDraft.attachments.find(
                (candidate) => candidate.id === incompatibleAttachment.id,
              );
              const compatible = selectableModels.find((model) =>
                modelAcceptsAttachment(model, attachment?.mediaType),
              );
              if (compatible) {
                setSelectedModelKey(conversationModelKey(compatible));
                setDismissedConflictAttachmentId(null);
              } else {
                setConversationError(
                  "No configured model can accept this attachment.",
                );
              }
              return;
            }
            setDismissedConflictAttachmentId(incompatibleAttachment.id);
          }}
          onRemoveAttachment={(attachmentId) => {
            dispatch(
              shellDraftActions.composerAttachmentRemoved(
                attachmentId as AttachmentId,
              ),
            );
            setPendingPickerGrantIds((current) =>
              current.filter((grantId) => grantId !== attachmentId),
            );
          }}
          onRemoveReply={() =>
            dispatch(shellDraftActions.composerReplyChanged(null))
          }
          onSubmit={(submission) => void submitChat(submission.source)}
          onValueChange={(value) =>
            dispatch(shellDraftActions.composerTextChanged(value))
          }
          {...(replyReference === null ? {} : { replyReference })}
          value={composerDraft.text}
        />
      }
      projectPanel={{
        mode: overlayPanel ? "overlay" : "docked",
        isOpen: panelOpen,
        width: panelWidth,
        minimumWidth: panelBounds.minimum,
        maximumWidth: panelBounds.maximum,
      }}
      projectPanelContent={
        <ProjectSessionNavigation
          activeProjectId={workspace.value.activeProjectId}
          activeSessionId={sessions.value.activeSessionId}
          onAddProject={() => void addProject()}
          onNewChat={(projectId) => void beginChat(projectId)}
          onProjectActivate={(projectId) => void activateProject(projectId)}
          onProjectExpansionChange={(projectId, isExpanded) =>
            setExpandedProjects((current) => {
              const next = new Set(
                current ??
                  workspace.value.projects.map((project) => String(project.id)),
              );
              if (isExpanded) next.add(projectId);
              else next.delete(projectId);
              return next;
            })
          }
          onProjectCopyPath={(projectId) => void copyProjectPath(projectId)}
          onProjectOrderChange={(projectIds) =>
            void reorderProjects(projectIds)
          }
          onProjectRelocate={(projectId) => void relocateProject(projectId)}
          onProjectRemove={(projectId) => void removeProject(projectId)}
          onProjectRename={(projectId, displayName) =>
            void renameProject(projectId, displayName)
          }
          onProjectReveal={(projectId) => void revealProject(projectId)}
          onSearchClear={() => undefined}
          onSearchQueryChange={setSearchQuery}
          onSessionActivate={(sessionId) => void activateSession(sessionId)}
          onSessionRemove={(sessionId) => void removeSession(sessionId)}
          projects={projectItems}
          searchQuery={searchQuery}
        />
      }
      projectPanelContentOwnsHeading
      routeContent={routeContent}
      {...(contextualChatContent === null ? {} : { contextualChatContent })}
      {...(conversation.value.title === null
        ? {}
        : { workspaceTitle: conversation.value.title })}
      {...(route !== "/chat" || sessions.value.activeSessionId === null
        ? {}
        : {
            workspaceTitleAccessory: (
              <ChatInformationPopover
                information={{
                  contextUsage: {
                    totalTokens: activeComposerModel?.contextTokens ?? 0,
                    usedTokens: contextUsed,
                  },
                  environment:
                    latestAssistantTurn?.environmentLabel ??
                    "Local environment",
                  health: activeRuntime
                    ? `${capitalize(activeRuntime.health)} runtime`
                    : "Runtime status unavailable",
                  model:
                    activeComposerModel?.modelId ??
                    latestAssistantTurn?.modelLabel ??
                    "Model unavailable",
                  runtime:
                    latestAssistantTurn?.runtimeLabel ?? "Runtime not bound",
                  workspace: workspace.value.displayName ?? "Workspace",
                }}
              />
            ),
          })}
      showReviewSettingsControl={qaEnabled}
      showContextualChat={focusedConversationArtifact !== null}
      onBackFromSettings={returnFromSettings}
      onComposerDraftChange={(draft) =>
        dispatch(shellDraftActions.composerTextChanged(draft))
      }
      onComposerModeChange={(mode) =>
        dispatch(shellDraftActions.composerModeChanged(mode))
      }
      onFocusRestored={() => undefined}
      onNavigate={navigateWithinShell}
      onPanelOpenChange={changePanelOpen}
      onPanelOverlayDismiss={() =>
        dispatch(shellDraftActions.leftPanelOverlayChanged(false))
      }
      onPanelWidthChange={(width) =>
        dispatch(
          shellDraftActions.leftPanelResized({
            width,
            viewportWidth,
          }),
        )
      }
      onVisitSettings={visitSettings}
    />
  );
  const conversationNotice = conversationError ? (
    <Notice title="Conversation needs attention" tone="danger">
      {conversationError}
    </Notice>
  ) : null;
  const ordinaryRouteContent =
    route === "/chat" ? (
      <>
        {conversationNotice}
        {hasPendingChat ? (
          <PendingConversationPrompt
            projectName={
              workspace.value.projects.find(
                (project) => project.id === workspace.value.activeProjectId,
              )?.name ?? "this project"
            }
          />
        ) : (
          conversationTranscript
        )}
      </>
    ) : (
      <ProjectedRouteContent
        route={route}
        conversation={conversation}
        launch={launch}
        sessions={sessions}
        settings={settings}
        workspace={workspace}
      />
    );
  const shellContent =
    route === "/chat" && !hasPendingChat ? (
      <ConversationFocusComposition
        focusedArtifact={focusedConversationArtifact}
        onCloseFocusedArtifact={() => void closeNativeArtifactFocus()}
        onRestoreChat={() => void closeNativeArtifactFocus()}
        transcript={conversationTranscript}
      >
        {({ center, contextualChat }) =>
          renderShell(
            <>
              {conversationNotice}
              {center}
            </>,
            contextualChat,
          )
        }
      </ConversationFocusComposition>
    ) : (
      renderShell(ordinaryRouteContent)
    );

  return (
    <>
      {shellContent}
      <ConversationFileDropOverlay
        isChatActive={
          route === "/chat" &&
          sessions.value.activeSessionId !== null &&
          (composerDraft.mode === "chat" || replyReference !== null) &&
          !conversationBusy
        }
        onDrop={(grants) =>
          void attachPickerGrants(
            grants.map(({ grantId }) => grantId as PickerGrantId),
          )
        }
        onInvalidEvent={(error) => setConversationError(messageFor(error))}
      />
    </>
  );
}

function useViewportWidth(): number {
  const [viewportWidth, setViewportWidth] = useState(() => window.innerWidth);

  useEffect(() => {
    const update = () => setViewportWidth(window.innerWidth);
    update();
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);

  return viewportWidth;
}

type WorkspaceState = ReturnType<typeof selectWorkspace>;
type SessionsState = ReturnType<typeof selectSessions>;
type ConversationState = ReturnType<typeof selectConversation>;
type SettingsState = ReturnType<typeof selectSettingsProjection>;
type LaunchState = ReturnType<typeof selectLaunch>;

interface ProjectedRouteContentProps {
  readonly route: AppRoutePath;
  readonly launch: LaunchState;
  readonly workspace: WorkspaceState;
  readonly sessions: SessionsState;
  readonly conversation: ConversationState;
  readonly settings: SettingsState;
}

/** Renders only state already present in authoritative projections. */
function ProjectedRouteContent({
  route,
  launch,
  workspace,
  sessions,
  conversation,
  settings,
}: ProjectedRouteContentProps): ReactNode {
  if (route === "/settings/providers") {
    return <NativePlatformSettingsContent />;
  }

  if (route === "/onboarding") {
    return projectionNotice(
      launch.generation,
      "Provider setup is unavailable",
      "C4OS is waiting for the native launch projection.",
    );
  }

  if (route === "/start") {
    return projectionNotice(
      workspace.generation,
      "Workspace Start is unavailable",
      "C4OS is waiting for the native workspace projection.",
      workspace.value.displayName
        ? `Continue ${workspace.value.displayName}.`
        : "Choose a workspace to continue.",
    );
  }

  if (route === "/chat") {
    if (conversation.generation === UNINITIALIZED_GENERATION) {
      return projectionNotice(
        conversation.generation,
        "Conversation unavailable",
        "C4OS is waiting for an authoritative conversation projection.",
      );
    }
    return (
      <div className="shell-projection-list" aria-label="Conversation turns">
        {conversation.value.turns.map((turn) => (
          <article key={turn.id} data-author={turn.author}>
            <strong>{turn.author === "user" ? "You" : "C4OS"}</strong>
            <p>{turn.markdown}</p>
          </article>
        ))}
      </div>
    );
  }

  if (route === "/chat-search") {
    return projectionList(
      sessions.generation,
      "Saved chat sessions",
      sessions.value.sessions.map((session) => session.title),
    );
  }

  if (route.startsWith("/settings/")) {
    return projectionNotice(
      settings.generation,
      `${routeTitle(route)} unavailable`,
      "C4OS is waiting for the authoritative Settings projection.",
      "The Settings projection is ready for service-owned controls.",
    );
  }

  return projectionNotice(
    workspace.generation,
    `${routeTitle(route)} unavailable`,
    "C4OS is waiting for the authoritative workspace projection.",
    `${routeTitle(route)} is ready for its service-owned content.`,
  );
}

function projectionNotice(
  generation: number,
  unavailableTitle: string,
  unavailableDetail: string,
  readyDetail?: string,
) {
  const isReady = generation !== UNINITIALIZED_GENERATION;
  return (
    <Notice
      title={isReady ? "Ready" : unavailableTitle}
      tone={isReady ? "success" : "warning"}
    >
      {isReady ? readyDetail : unavailableDetail}
    </Notice>
  );
}

function projectionList(
  generation: number,
  label: string,
  items: readonly string[],
) {
  if (generation === UNINITIALIZED_GENERATION) {
    return projectionNotice(
      generation,
      `${label} unavailable`,
      "C4OS is waiting for authoritative session state.",
    );
  }
  return (
    <ul className="shell-projection-list" aria-label={label}>
      {items.map((item) => (
        <li key={item}>{item}</li>
      ))}
    </ul>
  );
}

function routeTitle(route: AppRoutePath): string {
  return route
    .split("/")
    .filter(Boolean)
    .at(-1)!
    .split("-")
    .map((word) => `${word[0]?.toLocaleUpperCase()}${word.slice(1)}`)
    .join(" ");
}

function asShellFocusTarget(value: string | null): ShellFocusTarget | null {
  return value === "workspace-settings" ||
    value === "project-panel-toggle" ||
    value === "composer-draft"
    ? value
    : null;
}

function changedSet(
  current: ReadonlySet<string>,
  value: string,
  included: boolean,
): ReadonlySet<string> {
  const next = new Set(current);
  if (included) next.add(value);
  else next.delete(value);
  return next;
}

function conversationModelKey(model: {
  readonly providerId: string;
  readonly modelId: string;
}): string {
  return `${model.providerId}\u0000${model.modelId}`;
}

function conversationDraftSignature(input: {
  readonly prompt: string;
  readonly attachmentIds: readonly string[];
  readonly providerId: string | null;
  readonly modelId: string | null;
  readonly reasoningMode: string | null;
  readonly mode: string;
  readonly replyTargetId: string | null;
}): string {
  return JSON.stringify(input);
}

function formatByteLength(bytes: number): string {
  if (bytes < 1_024) return `${bytes} B`;
  if (bytes < 1_048_576) return `${(bytes / 1_024).toFixed(1)} KB`;
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}

function formatWorkDuration(durationMs: number): string {
  if (durationMs < 1_000) return `Worked for ${durationMs}ms`;
  const seconds = Math.max(1, Math.round(durationMs / 1_000));
  if (seconds < 60) return `Worked for ${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return `Worked for ${minutes}m${remainder === 0 ? "" : ` ${remainder}s`}`;
}

function attachmentExtension(name: string): string {
  const extension = name.split(".").at(-1);
  return extension && extension !== name
    ? extension.toLocaleUpperCase()
    : "FILE";
}

function attachmentCompatibility(
  mediaType: string | undefined,
  model:
    | {
        readonly available: boolean;
        readonly supportsVision: boolean;
        readonly supportsAudio: boolean;
      }
    | undefined,
): "ready" | "needs-vision" | "needs-audio" | "incompatible" {
  if (!mediaType || !model) return "ready";
  if (!model.available) return "incompatible";
  if (mediaType.startsWith("image/") && !model.supportsVision) {
    return "needs-vision";
  }
  if (mediaType.startsWith("audio/") && !model.supportsAudio) {
    return "needs-audio";
  }
  if (mediaType.startsWith("video/")) return "incompatible";
  return "ready";
}

function modelAcceptsAttachment(
  model: {
    readonly available: boolean;
    readonly supportsVision: boolean;
    readonly supportsAudio: boolean;
  },
  mediaType: string | undefined,
): boolean {
  return attachmentCompatibility(mediaType, model) === "ready";
}

function modelCapabilities(model: {
  readonly supportsVision: boolean;
  readonly supportsTools: boolean;
  readonly supportsReasoning: boolean;
  readonly supportsAudio: boolean;
}): readonly ModelCapability[] {
  const capabilities: ModelCapability[] = [];
  if (model.supportsVision) capabilities.push("vision");
  if (model.supportsTools) capabilities.push("tools");
  if (model.supportsReasoning) capabilities.push("reasoning");
  if (model.supportsAudio) capabilities.push("audio");
  return capabilities;
}

function formatCompactNumber(value: number): string {
  return new Intl.NumberFormat("en", {
    maximumFractionDigits: 1,
    notation: "compact",
  }).format(value);
}

function capitalize(value: string): string {
  return `${value[0]?.toLocaleUpperCase() ?? ""}${value.slice(1)}`;
}

function replyReferenceFromProjection(
  turns: readonly {
    readonly id: string;
    readonly author: "user" | "assistant";
    readonly markdown: string;
    readonly attachments?: readonly { readonly name: string }[];
  }[],
  targetId: string,
  artifacts: readonly NativeArtifactSnapshot[],
): ComposerReplyReference {
  const artifact = artifacts.find(
    (candidate) => candidate.artifactId === targetId,
  );
  if (artifact !== undefined && artifact.providerState.type !== "unknown") {
    return {
      id: targetId,
      kind: artifact.providerState.type,
      label: `Reply to ${artifact.providerState.type}`,
      excerpt: replyExcerpt(artifact.title),
    };
  }
  const target = turns.find((turn) => turn.id === targetId);
  if (target?.author === "user") {
    return {
      id: targetId,
      kind: "user-message",
      label: "Reply to message",
      excerpt: replyExcerpt(
        target.markdown || target.attachments?.[0]?.name || "Attachment",
      ),
    };
  }
  return {
    id: targetId,
    kind: "assistant-message",
    label: "Reply to message",
    excerpt: replyExcerpt(target?.markdown ?? "Saved response reference"),
  };
}

function replyExcerpt(source: string): string {
  return source.trim().split(/\s+/).slice(0, 8).join(" ");
}

function messageFor(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Conversation service could not complete the request.";
}
