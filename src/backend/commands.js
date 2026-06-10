//client
import { createLocalDiagnostics } from './diagnostics.js';

// Backend command names exposed through the app boundary.
export const BACKEND_COMMANDS = Object.freeze({
  getAppStatus: 'get_app_status',
});

/**
 * Creates the backend command registry used by UI and Tauri adapters.
 */
export function createBackendCommandRegistry(options = {}) {
  const diagnostics = options.diagnostics ?? createLocalDiagnostics();

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
          rootPath: null,
          branch: null,
          dirty: false,
          changedFileCount: 0,
          browserMode: 'read-only',
          rootAgentsDisplay: 'passive',
        },
        timeline: {
          toolActivityVisible: true,
          rawOutputExportAvailable: false,
          liveDrawerLabel: 'live/ephemeral',
        },
        session: {
          active: false,
          runtimeState: 'stopped',
          runtimeStates: [
            'running',
            'waiting_for_approval',
            'stopped',
            'failed',
            'complete',
          ],
          transcriptAppendOnly: true,
          canStartNewRun: true,
          failureDisplay: null,
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
