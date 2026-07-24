import type {
  McpServerDefinitionInput,
  McpServiceSnapshot,
} from "../../../platform/mcp-service";

export type McpActionResult = void | Promise<void>;

export type McpVisibleState =
  | "disabled"
  | "testing"
  | "connecting"
  | "ready"
  | "executing"
  | "denied"
  | "timedOut"
  | "revoked"
  | "restarting"
  | "failed";

export interface McpServerView {
  readonly activeRequests: number;
  readonly capabilities: readonly string[];
  readonly canConfigure: boolean;
  readonly canDelete: boolean;
  readonly canDisable: boolean;
  readonly canEnable: boolean;
  readonly canRevoke: boolean;
  readonly canRecover: boolean;
  readonly canRequestTrust: boolean;
  readonly canTest: boolean;
  readonly definition: McpServerDefinitionInput;
  readonly failureCode: string | null;
  readonly failureDetail: string | null;
  readonly id: string;
  readonly lastConnectedAt: string | null;
  readonly lifecycleGeneration: number;
  readonly name: string;
  readonly nextRestartAt: string | null;
  readonly protocolVersion: string | null;
  readonly resourceCount: number;
  readonly restartAttempts: number;
  readonly scopeLabel: string;
  readonly serverIdentity: string | null;
  readonly sourceKind: "user" | "plugin";
  readonly sourceLabel: string;
  readonly state: McpVisibleState;
  readonly toolCount: number;
  readonly transportLabel: string;
  readonly transportSummary: string;
  readonly trust: "pending" | "trusted" | "revoked";
  readonly trustApproval: {
    readonly promptId: string;
    readonly state: "interrupted" | "pending";
  } | null;
}

export type McpSettingsSnapshot =
  | {
      readonly message: string;
      readonly status: "loading";
    }
  | {
      readonly message: string;
      readonly retryable: boolean;
      readonly status: "error";
    }
  | {
      readonly activeWorkers: number;
      readonly generation: number;
      readonly operationError?: string;
      readonly servers: readonly McpServerView[];
      readonly status: "ready";
    };

export interface McpSettingsActions {
  readonly onDelete: (serverId: string) => McpActionResult;
  readonly onDisable: (serverId: string) => McpActionResult;
  readonly onEnableForNextTurn: (serverId: string) => McpActionResult;
  readonly onAnswerTrust: (
    serverId: string,
    promptId: string,
    answer: "allow" | "deny",
  ) => McpActionResult;
  readonly onRequestTrust: (serverId: string) => McpActionResult;
  readonly onRecover: (serverId: string) => McpActionResult;
  readonly onRetry: () => McpActionResult;
  readonly onRevoke: (serverId: string, reason: string) => McpActionResult;
  readonly onSave: (input: McpServerDefinitionInput) => McpActionResult;
  readonly onTest: (serverId: string) => McpActionResult;
}

/** Maps only native-validated MCP data into the controlled Settings surface. */
export function projectMcpSettingsSnapshot(
  snapshot: McpServiceSnapshot,
  nowMs = Date.now(),
): Extract<McpSettingsSnapshot, { status: "ready" }> {
  return {
    activeWorkers: snapshot.activeWorkers,
    generation: snapshot.generation,
    servers: snapshot.servers.map((server) => {
      const state = visibleState(server.lifecycle, server.lastFailureCode);
      const isBusy = [
        "testing",
        "connecting",
        "executing",
        "restarting",
      ].includes(state);
      const sourceKind = server.source.kind;
      return {
        activeRequests: server.activeRequests,
        capabilities: capabilityLabels(server.capabilities),
        canConfigure:
          sourceKind === "user" &&
          !isBusy &&
          ["disabled", "failed", "denied", "timedOut"].includes(state),
        canDelete:
          sourceKind === "user" &&
          !isBusy &&
          ["disabled", "failed", "denied", "timedOut"].includes(state),
        canDisable:
          server.trust !== "revoked" &&
          [
            "testing",
            "connecting",
            "ready",
            "executing",
            "restarting",
          ].includes(state),
        canEnable: server.trust === "trusted" && state === "disabled",
        canRecover:
          server.trust === "trusted" &&
          ["failed", "denied", "timedOut"].includes(state) &&
          (server.nextRestartAtMs === null || server.nextRestartAtMs <= nowMs),
        canRevoke: server.trust !== "revoked",
        canRequestTrust:
          server.trust === "pending" &&
          !isBusy &&
          (server.pendingTrustApproval === null ||
            server.pendingTrustApproval.state === "interrupted"),
        canTest:
          server.trust === "trusted" &&
          ["disabled", "failed", "denied", "timedOut"].includes(state),
        definition: {
          expectedGeneration: snapshot.generation,
          serverId: server.serverId,
          displayName: server.displayName,
          scope: server.scope,
          transport: server.transport,
          timeoutMs: server.timeoutMs,
          maxOutputBytes: server.maxOutputBytes,
        },
        failureCode: server.lastFailureCode,
        failureDetail: server.lastFailureDetail,
        id: server.serverId,
        lastConnectedAt: formatTimestamp(server.lastConnectedAtMs),
        lifecycleGeneration: server.lifecycleGeneration,
        name: server.displayName,
        nextRestartAt: formatTimestamp(server.nextRestartAtMs),
        protocolVersion: server.protocolVersion,
        resourceCount: server.resources.length,
        restartAttempts: server.restartAttempts,
        scopeLabel: scopeLabel(server.scope),
        serverIdentity:
          server.serverName === null
            ? null
            : `${server.serverName}${
                server.serverVersion === null ? "" : ` ${server.serverVersion}`
              }`,
        sourceKind,
        sourceLabel:
          server.source.kind === "user"
            ? "User configuration"
            : `Plugin ${server.source.packageId}`,
        state,
        toolCount: server.tools.length,
        transportLabel:
          server.transport.kind === "stdio" ? "STDIO" : "Streamable HTTP",
        transportSummary:
          server.transport.kind === "stdio"
            ? server.transport.command
            : server.transport.url,
        trust: server.trust,
        trustApproval:
          server.pendingTrustApproval === null
            ? null
            : {
                promptId: server.pendingTrustApproval.promptId,
                state: server.pendingTrustApproval.state,
              },
      } satisfies McpServerView;
    }),
    status: "ready",
  };
}

function visibleState(
  lifecycle: McpServiceSnapshot["servers"][number]["lifecycle"],
  failureCode: string | null,
): McpVisibleState {
  if (lifecycle !== "failed" || failureCode === null) return lifecycle;
  const normalized = failureCode.replaceAll(/[-_. ]/gu, "").toLocaleLowerCase();
  if (normalized.includes("denied")) return "denied";
  if (normalized.includes("timeout")) return "timedOut";
  return "failed";
}

function capabilityLabels(
  capabilities: McpServiceSnapshot["servers"][number]["capabilities"],
): string[] {
  return [
    capabilities.tools ? "Tools" : null,
    capabilities.resources ? "Resources" : null,
    capabilities.prompts ? "Prompts" : null,
    capabilities.logging ? "Logging" : null,
    capabilities.completions ? "Completions" : null,
    capabilities.tasks ? "Tasks" : null,
    ...capabilities.experimentalKeys.map((key) => `Experimental: ${key}`),
  ].filter((label): label is string => label !== null);
}

function scopeLabel(
  scope: McpServiceSnapshot["servers"][number]["scope"],
): string {
  if (scope.kind === "application") return "Application";
  if (scope.kind === "workspace") return `Workspace · ${scope.workspaceId}`;
  if (scope.kind === "project") return `Project · ${scope.projectId}`;
  return `Chat · ${scope.sessionId}`;
}

function formatTimestamp(value: number | null): string | null {
  if (value === null) return null;
  return new Date(value).toLocaleString();
}
