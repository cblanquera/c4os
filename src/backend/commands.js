//client
import { createLocalDiagnostics } from './diagnostics.js';

// Backend command names exposed through the app boundary.
export const BACKEND_COMMANDS = Object.freeze({
  getAppStatus: 'get_app_status',
  configureOpenRouter: 'configure_openrouter',
  registerProject: 'register_project',
  submitPrompt: 'submit_prompt',
});

/**
 * Creates the backend command registry used by UI and Tauri adapters.
 */
export function createBackendCommandRegistry(options = {}) {
  const diagnostics = options.diagnostics ?? createLocalDiagnostics();
  const state = {
    provider: {
      id: 'openrouter',
      ready: false,
      selectedModel: null,
      metadataStatus: 'unknown',
      disclosure:
        'Prompts and bounded context may be sent through OpenRouter.',
    },
    project: {
      active: false,
      id: null,
      rootPath: null,
      branch: null,
      dirty: false,
      changedFileCount: 0,
    },
    session: {
      active: false,
      id: null,
      title: null,
      runtimeState: 'stopped',
      messages: [],
    },
  };

  // Keep the command map explicit so future protected commands have one
  // obvious registration point.
  const handlers = new Map([
    [
      BACKEND_COMMANDS.getAppStatus,
      async () => ({
        appName: 'C4OS',
        backendReady: true,
        telemetryEnabled: false,
        diagnostics,
        provider: { ...state.provider },
        project: {
          active: state.project.active,
          rootPath: state.project.rootPath,
          branch: state.project.branch,
          dirty: state.project.dirty,
          changedFileCount: state.project.changedFileCount,
          browserMode: 'read-only',
          rootAgentsDisplay: 'passive',
          instructionResolution: {
            nestedAgentsSupported: true,
            supportTier: 'display_guidance_order_only',
            permissionEffect: 'none',
            modelContextEffect: 'none_without_explicit_runtime_read',
            editReloadBehavior: 'next_preflight_only',
            conflictDiagnostics: 'ordered_sources_no_semantic_merge',
          },
          selector: {
            available: true,
            listRegisteredProjects: true,
            selectExactlyOneActive: true,
            registeredProjectCount: state.project.active ? 1 : 0,
            multipleActiveProjectsAllowed: false,
            searchAvailable: false,
            groupingAvailable: false,
            archiveAvailable: false,
            deleteAvailable: false,
            favoritesAvailable: false,
            metadataEditingAvailable: false,
            crossProjectViewsAvailable: false,
            nonGitProjectsAllowed: false,
            worktreeManagementAvailable: false,
          },
        },
        timeline: {
          toolActivityVisible: true,
          rawOutputExportAvailable: false,
          liveDrawerLabel: 'live/ephemeral',
        },
        session: {
          active: state.session.active,
          id: state.session.id,
          title: state.session.title,
          runtimeState: state.session.runtimeState,
          runtimeStates: [
            'running',
            'waiting_for_approval',
            'stopped',
            'failed',
            'complete',
          ],
          transcriptAppendOnly: true,
          canStartNewRun: !state.session.active,
          failureDisplay: state.session.failureDisplay ?? null,
          messages: [...state.session.messages],
        },
        approvals: {
          pendingCount: 0,
          choices: ['allow_once', 'allow_session', 'deny'],
          highRiskSessionAllow: false,
          policyBlocksExecute: false,
        },
        recovery: {
          canStop: false,
          canRetry: false,
          retryAppendsNewAction: true,
          autoContinueAfterRestart: false,
          reattachAfterRestart: false,
          resendPendingPromptsAfterRestart: false,
        },
        changes: {
          changedFileCount: 0,
          diffViewerAvailable: true,
          gitInspectionLogged: true,
          approvalRequiredForInspection: false,
          refreshAfterToolExecution: true,
        },
        artifacts: {
          visibleRecords: true,
          searchAvailable: false,
          activeHtmlPreview: false,
          richPreview: false,
          exportAvailable: false,
          includeInModelContextByDefault: false,
        },
      }),
    ],
    [
      BACKEND_COMMANDS.configureOpenRouter,
      async (payload) => {
        const apiKey = `${payload.apiKey ?? ''}`.trim();
        const selectedModel = `${payload.selectedModel ?? ''}`.trim();

        if (!apiKey) {
          throw new Error('OpenRouter key is required.');
        }

        if (!selectedModel) {
          throw new Error('Select a model before saving provider setup.');
        }

        state.provider.ready = true;
        state.provider.selectedModel = selectedModel;

        return { ...state.provider };
      },
    ],
    [
      BACKEND_COMMANDS.registerProject,
      async (payload) => {
        const rootPath = `${payload.rootPath ?? ''}`.trim();

        if (!rootPath) {
          throw new Error('Project path is required.');
        }

        state.project = {
          active: true,
          id: `project-${stableHash(rootPath)}`,
          rootPath,
          branch: 'unknown',
          dirty: false,
          changedFileCount: 0,
        };

        return { ...state.project };
      },
    ],
    [
      BACKEND_COMMANDS.submitPrompt,
      async (payload) => {
        const prompt = `${payload.prompt ?? ''}`.trim();

        if (!state.provider.ready) {
          throw new Error('Configure OpenRouter before starting a session.');
        }

        if (!state.project.active) {
          throw new Error('Register and select a Git project before starting a session.');
        }

        if (!prompt) {
          throw new Error('Enter a prompt before starting a session.');
        }

        state.session = {
          active: false,
          id: `session-${Date.now()}`,
          title: firstLine(prompt),
          runtimeState: 'complete',
          failureDisplay: null,
          messages: [
            {
              id: `message-${Date.now()}`,
              role: 'user',
              content: prompt,
              status: 'submitted',
            },
            {
              id: `message-${Date.now()}-assistant`,
              role: 'assistant',
              content:
                'Browser smoke mode recorded this prompt. The desktop build runs OpenCode through the backend.',
              status: 'complete',
            },
          ],
        };

        return { ...state.session, messages: [...state.session.messages] };
      },
    ],
  ]);

  return {
    /**
     * Invokes a registered backend command by its stable command name.
     */
    async invoke(commandName, payload = {}) {
      const handler = handlers.get(commandName);

      if (!handler) {
        throw new Error(`Unknown backend command: ${commandName}`);
      }

      return handler(payload);
    },
  };
}

/**
 * Creates a deterministic browser-smoke identifier without storing raw paths.
 */
function stableHash(value) {
  let hash = 0;

  for (const character of value) {
    hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  }

  return hash.toString(16);
}

/**
 * Uses the first prompt line as the local session title.
 */
function firstLine(value) {
  return value.split('\n')[0].slice(0, 80);
}
