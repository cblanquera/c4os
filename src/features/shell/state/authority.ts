import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

import {
  UNINITIALIZED_GENERATION,
  type AuthoritativePublication,
  type ShellAuthorityState,
} from "./types";

export const initialShellAuthorityState: ShellAuthorityState = {
  platform: {
    generation: UNINITIALIZED_GENERATION,
    value: {
      appearance: "light",
      appearanceSource: "native-snapshot",
      reducedMotion: false,
    },
  },
  launch: {
    generation: UNINITIALIZED_GENERATION,
    value: {
      destination: "onboarding",
      providerConfigured: false,
      onboardingReady: false,
      recentWorkspaceIds: [],
    },
  },
  workspace: {
    generation: UNINITIALIZED_GENERATION,
    value: {
      activeWorkspaceId: null,
      displayName: null,
      projects: [],
      activeProjectId: null,
    },
  },
  sessions: {
    generation: UNINITIALIZED_GENERATION,
    value: { activeSessionId: null, sessions: [] },
  },
  conversation: {
    generation: UNINITIALIZED_GENERATION,
    value: { sessionId: null, title: null, turns: [], activeAttemptId: null },
  },
  composer: {
    generation: UNINITIALIZED_GENERATION,
    value: {
      activeModelId: null,
      activeReasoningEffort: null,
      models: [],
      allowedModes: ["chat"],
      reasoningEfforts: [],
      activeBranch: null,
      branches: [],
      branchPendingApprovalId: null,
      branchOperationStatus: null,
      branchOperationMessage: null,
    },
  },
  artifacts: {
    generation: UNINITIALIZED_GENERATION,
    value: { artifacts: [] },
  },
  settings: {
    generation: UNINITIALIZED_GENERATION,
    value: {
      savedRuntimeId: null,
      approvalPreset: "ask",
      restoreLastWorkspace: false,
      shellEnvironmentEnabled: false,
      browserEnvironment: "none",
    },
  },
  approvals: {
    generation: UNINITIALIZED_GENERATION,
    value: { approvals: [] },
  },
  runtime: {
    generation: UNINITIALIZED_GENERATION,
    value: { runtimes: [] },
  },
  extensions: {
    generation: UNINITIALIZED_GENERATION,
    value: { extensions: [] },
  },
  notices: {
    generation: UNINITIALIZED_GENERATION,
    value: { notices: [] },
  },
};

const shellAuthoritySlice = createSlice({
  name: "shellAuthority",
  initialState: initialShellAuthorityState,
  reducers: {
    publicationReceived(
      state,
      { payload }: PayloadAction<AuthoritativePublication>,
    ) {
      const current = state[payload.domain];
      if (payload.generation <= current.generation) {
        return;
      }

      // Each domain has its own monotonic cursor. This permits one Rust snapshot
      // generation to publish several domains while still rejecting replay in
      // every individual domain.
      return {
        ...state,
        [payload.domain]: {
          generation: payload.generation,
          value: payload.value,
        },
      } as ShellAuthorityState;
    },
    platformReducedMotionChanged(state, { payload }: PayloadAction<boolean>) {
      state.platform.value.reducedMotion = payload;
    },
  },
});

export const shellAuthorityActions = shellAuthoritySlice.actions;
export const shellAuthorityReducer = shellAuthoritySlice.reducer;
