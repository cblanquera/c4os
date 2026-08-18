import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

import type { AppRoutePath } from "../../../app/route-contract";
import { restoreAttachmentDraft } from "../../conversation/composer/attachment-draft";
import type {
  ArtifactId,
  AttachmentId,
  RuntimeId,
  SessionId,
  WorkspaceId,
} from "../../../platform/protocol";
import type { ComposerMode, SettingsSection, ShellDraftState } from "./types";
import {
  SHELL_CENTER_MINIMUM_WIDTH,
  SHELL_PROJECT_PANEL_INITIAL_WIDTH,
  SHELL_PROJECT_PANEL_MINIMUM_WIDTH,
} from "../ui/shell-geometry";

/** Keeps view-level ARIA bounds and reducer clamping on one exact contract. */
export function shellPanelBounds(viewportWidth: number) {
  return {
    minimum: SHELL_PROJECT_PANEL_MINIMUM_WIDTH,
    maximum: Math.max(
      SHELL_PROJECT_PANEL_MINIMUM_WIDTH,
      Math.min(
        viewportWidth * 0.55,
        viewportWidth - SHELL_CENTER_MINIMUM_WIDTH,
      ),
    ),
  };
}

export const initialShellDraftState: ShellDraftState = {
  workspace: {
    panel: {
      collapsed: false,
      width: SHELL_PROJECT_PANEL_INITIAL_WIDTH,
      overlayOpen: false,
    },
    focusedArtifactId: null,
    focusRestoreTarget: null,
  },
  composer: {
    mode: "chat",
    modeBeforeFocus: null,
    text: "",
    attachments: [],
    nextAttachmentReference: 1,
    replyTargetId: null,
  },
  settings: {
    activeSection: "providers",
    visit: null,
    runtimeId: null,
    dirty: false,
  },
  dismissedNoticeIds: [],
};

const shellDraftsSlice = createSlice({
  name: "shellDrafts",
  initialState: initialShellDraftState,
  reducers: {
    leftPanelResized(
      state,
      {
        payload,
      }: PayloadAction<{
        readonly width: number;
        readonly viewportWidth: number;
      }>,
    ) {
      const { maximum } = shellPanelBounds(payload.viewportWidth);
      state.workspace.panel.width = Math.round(
        Math.min(
          maximum,
          Math.max(SHELL_PROJECT_PANEL_MINIMUM_WIDTH, payload.width),
        ),
      );
    },
    leftPanelCollapsed(state, { payload }: PayloadAction<boolean>) {
      state.workspace.panel.collapsed = payload;
      if (payload) {
        state.workspace.panel.overlayOpen = false;
      }
    },
    leftPanelOverlayChanged(state, { payload }: PayloadAction<boolean>) {
      state.workspace.panel.overlayOpen = payload;
      if (payload) {
        state.workspace.panel.collapsed = false;
      }
    },
    artifactFocused(
      state,
      {
        payload,
      }: PayloadAction<{
        readonly artifactId: ArtifactId;
        readonly restoreTarget: string | null;
      }>,
    ) {
      state.workspace.focusedArtifactId = payload.artifactId;
      state.workspace.focusRestoreTarget = payload.restoreTarget;
      if (state.composer.modeBeforeFocus === null) {
        state.composer.modeBeforeFocus = state.composer.mode;
      }
      state.composer.mode = "chat";
    },
    chatRestored(state) {
      state.workspace.focusedArtifactId = null;
      if (state.composer.modeBeforeFocus !== null) {
        state.composer.mode = state.composer.modeBeforeFocus;
        state.composer.modeBeforeFocus = null;
      }
    },
    focusRestoreTargetCleared(state) {
      state.workspace.focusRestoreTarget = null;
    },
    composerModeChanged(state, { payload }: PayloadAction<ComposerMode>) {
      if (state.workspace.focusedArtifactId === null) {
        state.composer.mode = payload;
      }
    },
    composerTextChanged(state, { payload }: PayloadAction<string>) {
      state.composer.text = payload;
    },
    composerAttachmentsReconciled(
      state,
      {
        payload,
      }: PayloadAction<{
        readonly attachments: readonly {
          readonly id: AttachmentId;
          readonly name: string;
          readonly byteLength?: number;
          readonly mediaType?: string;
          readonly stableReference?: string;
          readonly referenceNumber: number;
          readonly compatibility:
            | "ready"
            | "needs-vision"
            | "needs-audio"
            | "converted"
            | "incompatible";
        }[];
        readonly nextAttachmentReference: number;
      }>,
    ) {
      const restored = restoreAttachmentDraft(
        payload.attachments,
        payload.nextAttachmentReference,
      );
      state.composer.attachments = restored.attachments.map((attachment) => ({
        ...attachment,
      }));
      state.composer.nextAttachmentReference = restored.nextReferenceNumber;
    },
    composerAttachmentRemoved(state, { payload }: PayloadAction<AttachmentId>) {
      state.composer.attachments = state.composer.attachments.filter(
        ({ id }) => id !== payload,
      );
    },
    composerReplyChanged(state, { payload }: PayloadAction<string | null>) {
      state.composer.replyTargetId = payload;
    },
    composerReplyReconciled(
      state,
      {
        payload,
      }: PayloadAction<{
        readonly expectedReplyTargetId: string | null;
        readonly authoritativeReplyTargetId: string | null;
      }>,
    ) {
      if (state.composer.replyTargetId === payload.expectedReplyTargetId) {
        state.composer.replyTargetId = payload.authoritativeReplyTargetId;
      }
    },
    settingsVisited(
      state,
      {
        payload,
      }: PayloadAction<{
        readonly section: SettingsSection;
        readonly route: AppRoutePath;
        readonly workspaceId: WorkspaceId | null;
        readonly sessionId: SessionId | null;
        readonly focusTarget: string | null;
      }>,
    ) {
      state.settings.activeSection = payload.section;
      if (state.settings.visit === null) {
        state.settings.visit = {
          route: payload.route,
          workspaceId: payload.workspaceId,
          sessionId: payload.sessionId,
          focusTarget: payload.focusTarget,
        };
      }
    },
    settingsSectionChanged(state, { payload }: PayloadAction<SettingsSection>) {
      state.settings.activeSection = payload;
    },
    settingsVisitEnded(state) {
      state.settings.visit = null;
    },
    settingsRuntimeDraftChanged(
      state,
      { payload }: PayloadAction<RuntimeId | null>,
    ) {
      state.settings.runtimeId = payload;
      state.settings.dirty = true;
    },
    settingsRuntimeDraftReconciled(
      state,
      { payload }: PayloadAction<RuntimeId | null>,
    ) {
      state.settings.runtimeId = payload;
      state.settings.dirty = false;
    },
    noticeDismissed(state, { payload }: PayloadAction<string>) {
      if (!state.dismissedNoticeIds.includes(payload)) {
        state.dismissedNoticeIds.push(payload);
      }
    },
  },
});

export const shellDraftActions = shellDraftsSlice.actions;
export const shellDraftReducer = shellDraftsSlice.reducer;
