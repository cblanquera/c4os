import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

import type { AppRoutePath } from "../../../app/route-contract";
import type {
  ArtifactId,
  AttachmentId,
  RuntimeId,
  SessionId,
  WorkspaceId,
} from "../../../platform/protocol";
import type { ComposerMode, SettingsSection, ShellDraftState } from "./types";

const DEFAULT_PANEL_WIDTH = 228;
const MIN_PANEL_WIDTH = 180;
const MIN_CENTER_WIDTH = 420;

/** Keeps view-level ARIA bounds and reducer clamping on one exact contract. */
export function shellPanelBounds(viewportWidth: number) {
  return {
    minimum: MIN_PANEL_WIDTH,
    maximum: Math.max(
      MIN_PANEL_WIDTH,
      Math.min(viewportWidth * 0.55, viewportWidth - MIN_CENTER_WIDTH),
    ),
  };
}

export const initialShellDraftState: ShellDraftState = {
  workspace: {
    panel: {
      collapsed: false,
      width: DEFAULT_PANEL_WIDTH,
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
    replyArtifactId: null,
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
        Math.min(maximum, Math.max(MIN_PANEL_WIDTH, payload.width)),
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
    composerAttachmentAdded(
      state,
      {
        payload,
      }: PayloadAction<{
        readonly id: AttachmentId;
        readonly name: string;
        readonly compatibility:
          "ready" | "needs-vision" | "needs-audio" | "converted";
      }>,
    ) {
      if (!state.composer.attachments.some(({ id }) => id === payload.id)) {
        state.composer.attachments.push(payload);
      }
    },
    composerAttachmentRemoved(state, { payload }: PayloadAction<AttachmentId>) {
      state.composer.attachments = state.composer.attachments.filter(
        ({ id }) => id !== payload,
      );
    },
    composerReplyChanged(state, { payload }: PayloadAction<ArtifactId | null>) {
      state.composer.replyArtifactId = payload;
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
