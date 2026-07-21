import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

import type {
  ApprovalId,
  ArtifactId,
  ProjectId,
  RuntimeId,
  SessionId,
  StateGeneration,
  WorkspaceId,
} from "../../../platform/protocol";
import { initialShellAuthorityState } from "./authority";
import { initialShellDraftState } from "./drafts";
import type { QaState, ShellPreloadedState } from "./types";

export const initialQaState: QaState = {
  enabled: false,
  fixtureId: null,
  activeWorkflow: null,
};

const qaSlice = createSlice({
  name: "shellQa",
  initialState: initialQaState,
  reducers: {
    workflowSelected(state, { payload }: PayloadAction<string>) {
      if (state.enabled) {
        state.activeWorkflow = payload;
      }
    },
  },
});

const generation = 50 as StateGeneration;
const workspaceId = "workspace:qa" as WorkspaceId;
const projectId = "project:qa" as ProjectId;
const sessionId = "session:qa" as SessionId;
const artifactId = "artifact:qa-file" as ArtifactId;
const runtimeId = "runtime:qa-opencode" as RuntimeId;

function deterministicQaPreloadedState(): ShellPreloadedState {
  return {
    shellAuthority: {
      ...initialShellAuthorityState,
      platform: {
        generation,
        value: {
          appearance: "dark",
          appearanceSource: "native-snapshot",
          reducedMotion: false,
        },
      },
      launch: {
        generation,
        value: {
          destination: "workspace",
          providerConfigured: true,
          onboardingReady: true,
          recentWorkspaceIds: [workspaceId],
        },
      },
      workspace: {
        generation,
        value: {
          activeWorkspaceId: workspaceId,
          displayName: "C4OS QA Workspace",
          activeProjectId: projectId,
          projects: [
            {
              id: projectId,
              name: "AI Desktop UI",
              pathState: "found",
              gitVersioned: true,
            },
          ],
        },
      },
      sessions: {
        generation,
        value: {
          activeSessionId: sessionId,
          sessions: [
            {
              id: sessionId,
              projectId,
              title: "Build onboarding start screens",
              lifecycle: "saved",
            },
          ],
        },
      },
      conversation: {
        generation,
        value: {
          sessionId,
          title: "Build onboarding start screens",
          activeAttemptId: null,
          turns: [
            {
              id: "turn:qa-user",
              author: "user",
              markdown: "Build the **workspace shell**.",
              status: "completed",
            },
            {
              id: "turn:qa-assistant",
              author: "assistant",
              markdown: "The deterministic shell projection is ready.",
              status: "completed",
            },
          ],
        },
      },
      composer: {
        generation,
        value: {
          activeModelId: "openai/gpt-5",
          activeReasoningEffort: "medium",
          models: [
            {
              providerId: "openai",
              providerName: "OpenAI",
              modelId: "openai/gpt-5",
              selected: true,
              available: true,
              supportsVision: true,
              supportsTools: true,
              supportsReasoning: true,
              supportsAudio: false,
              contextTokens: 400_000,
            },
          ],
          allowedModes: ["chat", "files", "browser", "terminal"],
          reasoningEfforts: ["off", "low", "medium", "high"],
          activeBranch: "main",
          branches: [
            { name: "main", targetOid: "1".repeat(40) },
            { name: "feature/demo", targetOid: "2".repeat(40) },
          ],
          branchPendingApprovalId: null,
          branchOperationStatus: null,
          branchOperationMessage: null,
        },
      },
      artifacts: {
        generation,
        value: {
          artifacts: [
            {
              id: artifactId,
              kind: "file",
              title: "src/App.tsx",
              focusSupported: true,
              sourceTurnId: "turn:qa-assistant",
            },
          ],
        },
      },
      settings: {
        generation,
        value: {
          savedRuntimeId: runtimeId,
          approvalPreset: "ask",
          restoreLastWorkspace: true,
          shellEnvironmentEnabled: true,
          browserEnvironment: "per-project",
        },
      },
      approvals: {
        generation,
        value: {
          approvals: [
            {
              id: "approval:qa" as ApprovalId,
              summary: "Write src/App.tsx",
              state: "pending",
            },
          ],
        },
      },
      runtime: {
        generation,
        value: {
          runtimes: [
            {
              id: runtimeId,
              kind: "open-code",
              lifecycle: "ready",
              health: "healthy",
            },
          ],
        },
      },
      extensions: {
        generation,
        value: {
          extensions: [
            {
              id: "plugin:github-workflow",
              kind: "plugin",
              name: "GitHub Workflow",
              state: "enabled",
            },
            {
              id: "skill:review",
              kind: "skill",
              name: "Review",
              state: "installed-disabled",
            },
          ],
        },
      },
      notices: {
        generation,
        value: {
          notices: [
            {
              id: "notice:qa-ready",
              tone: "success",
              message: "Deterministic QA projection ready.",
            },
          ],
        },
      },
    },
    shellDrafts: {
      ...initialShellDraftState,
      composer: {
        ...initialShellDraftState.composer,
        text: "Preserved QA composer draft",
      },
    },
    shellQa: {
      enabled: true,
      fixtureId: "r013-shell-v1",
      activeWorkflow: "chat",
    },
  };
}

/**
 * Returns deterministic review data only when Vite compiled the dedicated QA
 * build flag. There is intentionally no runtime toggle or persistence lookup.
 */
export function createBuildGatedQaPreloadedState():
  ShellPreloadedState | undefined {
  if (import.meta.env.VITE_C4OS_QA_FIXTURES !== "1") {
    return undefined;
  }
  return deterministicQaPreloadedState();
}

export const qaActions = qaSlice.actions;
export const qaReducer = qaSlice.reducer;
