import type { McpLifecycle as GeneratedMcpLifecycle } from "../generated/McpLifecycle";
import type { McpServerDefinitionInput as GeneratedMcpServerDefinitionInput } from "../generated/McpServerDefinitionInput";
import type { McpServerSnapshot as GeneratedMcpServerSnapshot } from "../generated/McpServerSnapshot";
import type { McpServiceSnapshot as GeneratedMcpServiceSnapshot } from "../generated/McpServiceSnapshot";
import type { McpTransportDefinition as GeneratedMcpTransportDefinition } from "../generated/McpTransportDefinition";
import type { McpTrustApprovalAnswer as GeneratedMcpTrustApprovalAnswer } from "../generated/McpTrustApprovalAnswer";
import type { McpTrustResponse as GeneratedMcpTrustResponse } from "../generated/McpTrustResponse";
import type { McpTrustState as GeneratedMcpTrustState } from "../generated/McpTrustState";

import { invokeNative } from "./native-transport";
import {
  MAX_DIAGNOSTIC_BYTES,
  MAX_IDENTIFIER_BYTES,
  MAX_SAFE_DETAILS,
  PROTOCOL_VERSION,
  type CorrelationId,
  type ProtocolErrorCode,
  type RequestId,
  type SafeDetailValue,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export const MCP_PROTOCOL_VERSION = "2025-11-25" as const;
export const MIN_MCP_TIMEOUT_MS = 250;
export const MAX_MCP_TIMEOUT_MS = 300_000;
export const MIN_MCP_OUTPUT_BYTES = 1_024;
export const MAX_MCP_OUTPUT_BYTES = 16 * 1_024 * 1_024;

export type McpLifecycle = GeneratedMcpLifecycle;
export type McpTrustState = GeneratedMcpTrustState;
export type McpTransportDefinition = GeneratedMcpTransportDefinition;
export type McpServerDefinitionInput = GeneratedMcpServerDefinitionInput;
export type McpServerSnapshot = GeneratedMcpServerSnapshot;
export type McpServiceSnapshot = GeneratedMcpServiceSnapshot;
export type McpTrustApprovalAnswer = GeneratedMcpTrustApprovalAnswer;
export type McpTrustResponse = GeneratedMcpTrustResponse;

export type McpCommand =
  | "mcp_snapshot"
  | "mcp_save_server"
  | "mcp_request_trust"
  | "mcp_answer_trust"
  | "mcp_test_server"
  | "mcp_enable_server"
  | "mcp_recover_server"
  | "mcp_disable_server"
  | "mcp_revoke_server"
  | "mcp_delete_server";

export interface McpTransport {
  invoke(
    command: McpCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface McpAdapter {
  readSnapshot(): Promise<McpServiceSnapshot>;
  saveServer(input: McpServerDefinitionInput): Promise<McpServiceSnapshot>;
  requestTrust(serverId: string): Promise<McpTrustResponse>;
  answerTrust(
    serverId: string,
    promptId: string,
    answer: McpTrustApprovalAnswer,
  ): Promise<McpTrustResponse>;
  testServer(serverId: string): Promise<McpServiceSnapshot>;
  enableServer(serverId: string): Promise<McpServiceSnapshot>;
  recoverServer(serverId: string): Promise<McpServiceSnapshot>;
  disableServer(serverId: string): Promise<McpServiceSnapshot>;
  revokeServer(serverId: string, reason: string): Promise<McpServiceSnapshot>;
  deleteServer(serverId: string): Promise<McpServiceSnapshot>;
}

const nativeAdapter = createMcpAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readMcpSnapshot = () => nativeAdapter.readSnapshot();
export const saveMcpServer = (input: McpServerDefinitionInput) =>
  nativeAdapter.saveServer(input);
export const requestMcpTrust = (serverId: string) =>
  nativeAdapter.requestTrust(serverId);
export const answerMcpTrust = (
  serverId: string,
  promptId: string,
  answer: McpTrustApprovalAnswer,
) => nativeAdapter.answerTrust(serverId, promptId, answer);
export const testMcpServer = (serverId: string) =>
  nativeAdapter.testServer(serverId);
export const enableMcpServer = (serverId: string) =>
  nativeAdapter.enableServer(serverId);
export const recoverMcpServer = (serverId: string) =>
  nativeAdapter.recoverServer(serverId);
export const disableMcpServer = (serverId: string) =>
  nativeAdapter.disableServer(serverId);
export const revokeMcpServer = (serverId: string, reason: string) =>
  nativeAdapter.revokeServer(serverId, reason);
export const deleteMcpServer = (serverId: string) =>
  nativeAdapter.deleteServer(serverId);

export function createMcpAdapter(
  transport: McpTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): McpAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let generation = 0;

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: generation as StateGeneration,
  });

  const invokeSnapshot = async (
    command: McpCommand,
    input?: Readonly<Record<string, unknown>>,
  ): Promise<McpServiceSnapshot> => {
    const snapshotRequest = request();
    let raw: unknown;
    try {
      raw = await transport.invoke(
        command,
        input === undefined
          ? { request: snapshotRequest }
          : { request: snapshotRequest, input },
      );
    } catch (error) {
      throw normalizeMcpFailure(
        error,
        "The native MCP service is unavailable.",
      );
    }
    const envelope = parseEnvelope(raw, parseMcpSnapshot);
    if (
      envelope.requestId !== snapshotRequest.requestId ||
      envelope.correlationId !== snapshotRequest.correlationId
    ) {
      throw new ProtocolBoundaryError(
        "correlationMismatch",
        "The MCP response identity did not match its request.",
      );
    }
    if (envelope.generation < generation) {
      throw invalidPayload("The MCP response generation regressed.");
    }
    if (envelope.payload.generation !== envelope.generation) {
      throw invalidPayload("The MCP payload generation is inconsistent.");
    }
    generation = envelope.generation;
    return envelope.payload;
  };

  const invokeTrust = async (
    command: "mcp_request_trust" | "mcp_answer_trust",
    input: Readonly<Record<string, unknown>>,
  ): Promise<McpTrustResponse> => {
    const snapshotRequest = request();
    let raw: unknown;
    try {
      raw = await transport.invoke(command, {
        request: snapshotRequest,
        input,
      });
    } catch (error) {
      throw normalizeMcpFailure(
        error,
        "The native MCP trust service is unavailable.",
      );
    }
    const envelope = parseEnvelope(raw, parseMcpTrustResponse);
    if (
      envelope.requestId !== snapshotRequest.requestId ||
      envelope.correlationId !== snapshotRequest.correlationId
    ) {
      throw new ProtocolBoundaryError(
        "correlationMismatch",
        "The MCP trust response identity did not match its request.",
      );
    }
    if (
      envelope.generation < generation ||
      envelope.payload.snapshot.generation !== envelope.generation
    ) {
      throw invalidPayload(
        "The MCP trust response generation is inconsistent.",
      );
    }
    generation = envelope.generation;
    return envelope.payload;
  };

  const serverMutation = (command: McpCommand, serverId: string) =>
    invokeSnapshot(command, {
      expectedGeneration: generation,
      serverId: asIdentifier(serverId, "server ID"),
    });

  return {
    readSnapshot: () => invokeSnapshot("mcp_snapshot"),
    async saveServer(input) {
      const validated = validateDefinitionInput(input);
      return invokeSnapshot("mcp_save_server", {
        ...validated,
        expectedGeneration: generation,
      });
    },
    requestTrust: async (serverId) =>
      invokeTrust("mcp_request_trust", {
        expectedGeneration: generation,
        serverId: asIdentifier(serverId, "server ID"),
      }),
    answerTrust: async (serverId, promptId, answer) =>
      invokeTrust("mcp_answer_trust", {
        expectedGeneration: generation,
        serverId: asIdentifier(serverId, "server ID"),
        promptId: asIdentifier(promptId, "approval prompt ID"),
        answer: asEnum(answer, ["allow", "deny"] as const, "trust answer"),
      }),
    testServer: async (serverId) => serverMutation("mcp_test_server", serverId),
    enableServer: async (serverId) =>
      serverMutation("mcp_enable_server", serverId),
    recoverServer: async (serverId) =>
      serverMutation("mcp_recover_server", serverId),
    disableServer: async (serverId) =>
      serverMutation("mcp_disable_server", serverId),
    async revokeServer(serverId, reason) {
      return invokeSnapshot("mcp_revoke_server", {
        expectedGeneration: generation,
        serverId: asIdentifier(serverId, "server ID"),
        reason: asText(reason, "revocation reason", 16_384),
      });
    },
    deleteServer: async (serverId) =>
      serverMutation("mcp_delete_server", serverId),
  };
}

function parseEnvelope<Payload>(
  raw: unknown,
  parsePayload: (raw: unknown) => Payload,
) {
  const value = exactRecord(
    raw,
    ["protocolVersion", "requestId", "correlationId", "generation", "payload"],
    "MCP envelope",
  );
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw new ProtocolBoundaryError(
      "unknownProtocolVersion",
      "Unsupported protocol version.",
    );
  }
  return {
    requestId: asIdentifier(value.requestId, "request ID") as RequestId,
    correlationId: asIdentifier(
      value.correlationId,
      "correlation ID",
    ) as CorrelationId,
    generation: asGeneration(value.generation, false),
    payload: parsePayload(value.payload),
  };
}

function parseMcpTrustResponse(raw: unknown): McpTrustResponse {
  const value = exactRecord(
    raw,
    [
      "snapshot",
      "status",
      "serverId",
      "definitionSha256",
      "promptId",
      "promptExpiresAtMs",
    ],
    "MCP trust response",
  );
  const status = asEnum(
    value.status,
    ["trusted", "pendingApproval", "denied"] as const,
    "MCP trust response status",
  );
  const promptId = asOptionalText(value.promptId, "approval prompt ID", 255);
  const promptExpiresAtMs = asOptionalGeneration(value.promptExpiresAtMs);
  if (
    (status === "pendingApproval" &&
      (promptId === null || promptExpiresAtMs === null)) ||
    (status !== "pendingApproval" &&
      (promptId !== null || promptExpiresAtMs !== null))
  ) {
    throw invalidPayload("The MCP trust approval state is inconsistent.");
  }
  return {
    snapshot: parseMcpSnapshot(value.snapshot),
    status,
    serverId: asIdentifier(value.serverId, "server ID"),
    definitionSha256: asDigest(value.definitionSha256),
    promptId,
    promptExpiresAtMs,
  };
}

function parseMcpSnapshot(raw: unknown): McpServiceSnapshot {
  const value = exactRecord(
    raw,
    ["schemaVersion", "generation", "servers", "activeWorkers", "lastEventId"],
    "MCP snapshot",
  );
  if (value.schemaVersion !== 1) {
    throw invalidPayload("Unsupported MCP schema.");
  }
  const servers = asArray(value.servers, "servers", 256).map(parseServer);
  const identities = new Set(servers.map((server) => server.serverId));
  if (identities.size !== servers.length) {
    throw invalidPayload("MCP server identities are not unique.");
  }
  return {
    schemaVersion: 1,
    generation: asGeneration(value.generation, false),
    servers,
    activeWorkers: asInteger(value.activeWorkers, "active workers", 65_535),
    lastEventId: asGeneration(value.lastEventId, true),
  };
}

function parseServer(raw: unknown): McpServerSnapshot {
  const value = exactRecord(
    raw,
    [
      "serverId",
      "displayName",
      "source",
      "scope",
      "transport",
      "trust",
      "trustedDefinitionSha256",
      "pendingTrustApproval",
      "lifecycle",
      "timeoutMs",
      "maxOutputBytes",
      "lifecycleGeneration",
      "restartAttempts",
      "nextRestartAtMs",
      "protocolVersion",
      "serverName",
      "serverVersion",
      "instructionsPresent",
      "capabilities",
      "tools",
      "resources",
      "activeRequests",
      "lastConnectedAtMs",
      "lastFailureCode",
      "lastFailureDetail",
      "lastEventId",
    ],
    "MCP server",
  );
  const lifecycle = asEnum(
    value.lifecycle,
    [
      "disabled",
      "testing",
      "connecting",
      "ready",
      "executing",
      "restarting",
      "failed",
      "revoked",
    ] as const,
    "MCP lifecycle",
  );
  const trust = asEnum(
    value.trust,
    ["pending", "trusted", "revoked"] as const,
    "MCP trust",
  );
  const pendingTrustApproval = parsePendingTrustApproval(
    value.pendingTrustApproval,
  );
  const trustedDefinitionSha256 = asOptionalDigest(
    value.trustedDefinitionSha256,
  );
  const protocolVersion = asOptionalText(
    value.protocolVersion,
    "MCP protocol version",
    160,
  );
  if (
    (lifecycle === "ready" && protocolVersion !== MCP_PROTOCOL_VERSION) ||
    (trust === "revoked" && lifecycle !== "revoked") ||
    (trust !== "pending" && pendingTrustApproval !== null) ||
    (trust === "trusted") !== (trustedDefinitionSha256 !== null)
  ) {
    throw invalidPayload("MCP lifecycle state is inconsistent.");
  }
  return {
    serverId: asIdentifier(value.serverId, "server ID"),
    displayName: asText(value.displayName, "server name", 16_384),
    source: parseSource(value.source),
    scope: parseScope(value.scope),
    transport: parseTransport(value.transport),
    trust,
    trustedDefinitionSha256,
    pendingTrustApproval,
    lifecycle,
    timeoutMs: asIntegerRange(
      value.timeoutMs,
      "timeout",
      MIN_MCP_TIMEOUT_MS,
      MAX_MCP_TIMEOUT_MS,
    ),
    maxOutputBytes: asIntegerRange(
      value.maxOutputBytes,
      "output limit",
      MIN_MCP_OUTPUT_BYTES,
      MAX_MCP_OUTPUT_BYTES,
    ),
    lifecycleGeneration: asGeneration(value.lifecycleGeneration, false),
    restartAttempts: asInteger(
      value.restartAttempts,
      "restart attempts",
      65_535,
    ),
    nextRestartAtMs: asOptionalGeneration(value.nextRestartAtMs),
    protocolVersion,
    serverName: asOptionalText(value.serverName, "peer name", 16_384),
    serverVersion: asOptionalText(value.serverVersion, "peer version", 1_024),
    instructionsPresent: asBoolean(
      value.instructionsPresent,
      "instructions state",
    ),
    capabilities: parseCapabilities(value.capabilities),
    tools: asArray(value.tools, "tools", 4_096).map(parseTool),
    resources: asArray(value.resources, "resources", 4_096).map(parseResource),
    activeRequests: asInteger(value.activeRequests, "active requests", 65_535),
    lastConnectedAtMs: asOptionalGeneration(value.lastConnectedAtMs),
    lastFailureCode: asOptionalText(value.lastFailureCode, "failure code", 160),
    lastFailureDetail: asOptionalText(
      value.lastFailureDetail,
      "failure detail",
      16_384,
    ),
    lastEventId: asGeneration(value.lastEventId, true),
  };
}

function parsePendingTrustApproval(
  raw: unknown,
): McpServerSnapshot["pendingTrustApproval"] {
  if (raw === null) return null;
  const value = exactRecord(
    raw,
    [
      "promptId",
      "definitionSha256",
      "actionBindingSha256",
      "actionConfigurationVersion",
      "requestedAtMs",
      "expiresAtMs",
      "state",
    ],
    "MCP pending trust approval",
  );
  const requestedAtMs = asGeneration(value.requestedAtMs, false);
  const expiresAtMs = asGeneration(value.expiresAtMs, false);
  if (requestedAtMs >= expiresAtMs) {
    throw invalidPayload("The MCP trust approval deadline is invalid.");
  }
  return {
    promptId: asIdentifier(value.promptId, "approval prompt ID"),
    definitionSha256: asDigest(value.definitionSha256),
    actionBindingSha256: asDigest(value.actionBindingSha256),
    actionConfigurationVersion: asGeneration(
      value.actionConfigurationVersion,
      false,
    ),
    requestedAtMs,
    expiresAtMs,
    state: asEnum(
      value.state,
      ["pending", "interrupted"] as const,
      "trust approval state",
    ),
  };
}

function parseSource(raw: unknown): McpServerSnapshot["source"] {
  const tagged = taggedRecord(raw, "MCP source");
  if (tagged.kind === "user") {
    exactKeys(tagged, ["kind"], "MCP user source");
    return { kind: "user" };
  }
  if (tagged.kind === "plugin") {
    exactKeys(
      tagged,
      ["kind", "packageId", "declarationId"],
      "MCP Plugin source",
    );
    return {
      kind: "plugin",
      packageId: asIdentifier(tagged.packageId, "Plugin package ID"),
      declarationId: asIdentifier(tagged.declarationId, "declaration ID"),
    };
  }
  throw invalidPayload("MCP source is invalid.");
}

function parseScope(raw: unknown): McpServerSnapshot["scope"] {
  const tagged = taggedRecord(raw, "MCP scope");
  if (tagged.kind === "application") {
    exactKeys(tagged, ["kind"], "application scope");
    return { kind: "application" };
  }
  if (tagged.kind === "workspace") {
    exactKeys(tagged, ["kind", "workspaceId"], "Workspace scope");
    return {
      kind: "workspace",
      workspaceId: asIdentifier(tagged.workspaceId, "workspace ID"),
    };
  }
  if (tagged.kind === "project") {
    exactKeys(tagged, ["kind", "workspaceId", "projectId"], "Project scope");
    return {
      kind: "project",
      workspaceId: asIdentifier(tagged.workspaceId, "workspace ID"),
      projectId: asIdentifier(tagged.projectId, "project ID"),
    };
  }
  if (tagged.kind === "chat") {
    exactKeys(
      tagged,
      ["kind", "workspaceId", "projectId", "sessionId"],
      "Chat scope",
    );
    return {
      kind: "chat",
      workspaceId: asIdentifier(tagged.workspaceId, "workspace ID"),
      projectId: asIdentifier(tagged.projectId, "project ID"),
      sessionId: asIdentifier(tagged.sessionId, "session ID"),
    };
  }
  throw invalidPayload("MCP scope is invalid.");
}

function parseTransport(raw: unknown): McpTransportDefinition {
  const tagged = taggedRecord(raw, "MCP transport");
  if (tagged.kind === "stdio") {
    exactKeys(
      tagged,
      [
        "kind",
        "command",
        "arguments",
        "environment",
        "workingDirectory",
        "executableSha256",
      ],
      "STDIO transport",
    );
    const environment = asArray(tagged.environment, "environment", 128).map(
      parseEnvironmentBinding,
    );
    assertUniqueNames(
      environment.map((binding) => binding.name),
      "environment binding",
    );
    return {
      kind: "stdio",
      command: asAbsolutePath(tagged.command, "command"),
      arguments: asArray(tagged.arguments, "arguments", 128).map((argument) =>
        asText(argument, "argument", 16_384),
      ),
      environment,
      workingDirectory: parseWorkingDirectory(tagged.workingDirectory),
      executableSha256: asDigest(tagged.executableSha256),
    };
  }
  if (tagged.kind === "streamableHttp") {
    exactKeys(
      tagged,
      ["kind", "url", "bearer", "headers"],
      "Streamable HTTP transport",
    );
    const headers = asArray(tagged.headers, "headers", 128).map(
      parseHeaderBinding,
    );
    const names = headers.map((header) => header.name.toLocaleLowerCase());
    assertUniqueNames(names, "HTTP header");
    if (names.some((name) => RESERVED_MCP_HTTP_HEADERS.has(name))) {
      throw invalidPayload("A reserved HTTP header was supplied.");
    }
    return {
      kind: "streamableHttp",
      url: asText(tagged.url, "Streamable HTTP URL", 16_384),
      bearer: parseSecretReference(tagged.bearer),
      headers,
    };
  }
  throw invalidPayload("MCP transport is invalid.");
}

function parseEnvironmentBinding(
  raw: unknown,
): Extract<McpTransportDefinition, { kind: "stdio" }>["environment"][number] {
  const value = exactRecord(raw, ["name", "source"], "environment binding");
  const source = taggedRecord(value.source, "environment source");
  if (source.kind === "literal") {
    exactKeys(source, ["kind", "value"], "literal environment source");
    return {
      name: asEnvironmentName(value.name),
      source: {
        kind: "literal",
        value: asText(source.value, "environment value", 16_384),
      },
    };
  }
  if (source.kind === "passthrough") {
    exactKeys(source, ["kind"], "passthrough environment source");
    return {
      name: asEnvironmentName(value.name),
      source: { kind: "passthrough" },
    };
  }
  if (source.kind === "secret") {
    exactKeys(source, ["kind", "reference"], "secret environment source");
    return {
      name: asEnvironmentName(value.name),
      source: {
        kind: "secret",
        reference: parseSecretReference(source.reference),
      },
    };
  }
  throw invalidPayload("Environment source is invalid.");
}

function parseHeaderBinding(
  raw: unknown,
): Extract<
  McpTransportDefinition,
  { kind: "streamableHttp" }
>["headers"][number] {
  const value = exactRecord(raw, ["name", "source"], "header binding");
  const name = asHeaderName(value.name);
  const source = taggedRecord(value.source, "header source");
  if (source.kind === "literal") {
    exactKeys(source, ["kind", "value"], "literal header source");
    return {
      name,
      source: {
        kind: "literal",
        value: asText(source.value, "header value", 16_384),
      },
    };
  }
  if (source.kind === "environment") {
    exactKeys(source, ["kind", "variable"], "environment header source");
    return {
      name,
      source: {
        kind: "environment",
        variable: asEnvironmentName(source.variable),
      },
    };
  }
  if (source.kind === "secret") {
    exactKeys(source, ["kind", "reference"], "secret header source");
    return {
      name,
      source: {
        kind: "secret",
        reference: parseSecretReference(source.reference),
      },
    };
  }
  throw invalidPayload("Header source is invalid.");
}

function parseSecretReference(raw: unknown) {
  const tagged = taggedRecord(raw, "secret reference");
  if (tagged.kind === "vault") {
    exactKeys(tagged, ["kind", "credentialReference"], "vault reference");
    return {
      kind: "vault" as const,
      credentialReference: asIdentifier(
        tagged.credentialReference,
        "credential reference",
      ),
    };
  }
  if (tagged.kind === "environment") {
    exactKeys(tagged, ["kind", "variable"], "environment reference");
    return {
      kind: "environment" as const,
      variable: asEnvironmentName(tagged.variable),
    };
  }
  throw invalidPayload("Secret reference is invalid.");
}

function parseWorkingDirectory(raw: unknown) {
  const tagged = taggedRecord(raw, "working directory");
  if (tagged.kind === "c4osHome") {
    exactKeys(tagged, ["kind"], "C4OS Home working directory");
    return { kind: "c4osHome" as const };
  }
  if (tagged.kind === "activeProject") {
    exactKeys(tagged, ["kind"], "active Project working directory");
    return { kind: "activeProject" as const };
  }
  if (tagged.kind === "trustedRoot") {
    exactKeys(tagged, ["kind", "path"], "trusted-root working directory");
    return {
      kind: "trustedRoot" as const,
      path: asText(tagged.path, "trusted-root path", 16_384),
    };
  }
  throw invalidPayload("Working directory is invalid.");
}

function parseCapabilities(raw: unknown): McpServerSnapshot["capabilities"] {
  const value = exactRecord(
    raw,
    [
      "tools",
      "toolListChanged",
      "resources",
      "resourceListChanged",
      "resourceSubscribe",
      "prompts",
      "logging",
      "completions",
      "tasks",
      "experimentalKeys",
    ],
    "MCP capabilities",
  );
  return {
    tools: asBoolean(value.tools, "tools capability"),
    toolListChanged: asBoolean(
      value.toolListChanged,
      "tool-list change capability",
    ),
    resources: asBoolean(value.resources, "resources capability"),
    resourceListChanged: asBoolean(
      value.resourceListChanged,
      "resource-list change capability",
    ),
    resourceSubscribe: asBoolean(
      value.resourceSubscribe,
      "resource subscription capability",
    ),
    prompts: asBoolean(value.prompts, "prompts capability"),
    logging: asBoolean(value.logging, "logging capability"),
    completions: asBoolean(value.completions, "completions capability"),
    tasks: asBoolean(value.tasks, "tasks capability"),
    experimentalKeys: asArray(
      value.experimentalKeys,
      "experimental capability keys",
      128,
    ).map((key) => asText(key, "experimental capability key", 1_024)),
  };
}

function parseTool(raw: unknown): McpServerSnapshot["tools"][number] {
  const value = exactRecord(
    raw,
    [
      "name",
      "title",
      "description",
      "inputSchema",
      "inputSchemaSha256",
      "outputSchemaSha256",
    ],
    "MCP tool",
  );
  return {
    name: asText(value.name, "tool name", 255),
    title: asOptionalText(value.title, "tool title", 16_384),
    description: asOptionalText(value.description, "tool description", 16_384),
    inputSchema: asBoundedJsonObject(value.inputSchema, "tool input schema"),
    inputSchemaSha256: asDigest(value.inputSchemaSha256),
    outputSchemaSha256: asOptionalDigest(value.outputSchemaSha256),
  };
}

function parseResource(raw: unknown): McpServerSnapshot["resources"][number] {
  const value = exactRecord(
    raw,
    ["uri", "name", "title", "description", "mimeType", "size"],
    "MCP resource",
  );
  return {
    uri: asText(value.uri, "resource URI", 16_384),
    name: asText(value.name, "resource name", 16_384),
    title: asOptionalText(value.title, "resource title", 16_384),
    description: asOptionalText(
      value.description,
      "resource description",
      16_384,
    ),
    mimeType: asOptionalText(value.mimeType, "resource media type", 16_384),
    size: asOptionalGeneration(value.size),
  };
}

function validateDefinitionInput(
  input: McpServerDefinitionInput,
): McpServerDefinitionInput {
  const value = exactRecord(
    input,
    [
      "expectedGeneration",
      "serverId",
      "displayName",
      "scope",
      "transport",
      "timeoutMs",
      "maxOutputBytes",
    ],
    "MCP definition",
  );
  return {
    expectedGeneration: asGeneration(value.expectedGeneration, true),
    serverId: asIdentifier(value.serverId, "server ID"),
    displayName: asText(value.displayName, "server name", 16_384),
    scope: parseScope(value.scope),
    transport: parseTransport(value.transport),
    timeoutMs: asIntegerRange(
      value.timeoutMs,
      "timeout",
      MIN_MCP_TIMEOUT_MS,
      MAX_MCP_TIMEOUT_MS,
    ),
    maxOutputBytes: asIntegerRange(
      value.maxOutputBytes,
      "output limit",
      MIN_MCP_OUTPUT_BYTES,
      MAX_MCP_OUTPUT_BYTES,
    ),
  };
}

function taggedRecord(raw: unknown, label: string): Record<string, unknown> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  const value = raw as Record<string, unknown>;
  if (typeof value.kind !== "string") {
    throw invalidPayload(`${label} is invalid.`);
  }
  return value;
}

function exactRecord(
  raw: unknown,
  keys: readonly string[],
  label: string,
): Record<string, unknown> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  const value = raw as Record<string, unknown>;
  exactKeys(value, keys, label);
  return value;
}

function exactKeys(
  value: Record<string, unknown>,
  keys: readonly string[],
  label: string,
) {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (
    actual.length !== expected.length ||
    actual.some((key, index) => key !== expected[index])
  ) {
    throw invalidPayload(`${label} fields are invalid.`);
  }
}

function asArray(
  raw: unknown,
  label: string,
  maximum: number,
): readonly unknown[] {
  if (!Array.isArray(raw) || raw.length > maximum) {
    throw invalidPayload(`${label} are invalid.`);
  }
  return raw;
}

function asText(raw: unknown, label: string, maximum: number): string {
  if (
    typeof raw !== "string" ||
    raw.length === 0 ||
    raw.length > maximum ||
    raw.includes("\0")
  ) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw;
}

function asOptionalText(
  raw: unknown,
  label: string,
  maximum: number,
): string | null {
  return raw === null ? null : asText(raw, label, maximum);
}

function asIdentifier(raw: unknown, label: string): string {
  const value = asText(raw, label, MAX_IDENTIFIER_BYTES);
  if (!/^[A-Za-z0-9._:-]+$/.test(value)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return value;
}

function asEnvironmentName(raw: unknown): string {
  const value = asText(raw, "environment variable", 255);
  const normalized = value.toLocaleUpperCase("en-US");
  if (
    !/^[A-Za-z_][A-Za-z0-9_]*$/.test(value) ||
    ["HOME", "PATH", "SHELL", "TMPDIR"].includes(normalized) ||
    normalized.startsWith("DYLD_") ||
    normalized.startsWith("LD_")
  ) {
    throw invalidPayload("Environment variable is invalid.");
  }
  return value;
}

function asAbsolutePath(raw: unknown, label: string): string {
  const value = asText(raw, label, 16_384);
  if (!value.startsWith("/")) {
    throw invalidPayload(`${label} must be an absolute path.`);
  }
  return value;
}

function asHeaderName(raw: unknown): string {
  const value = asText(raw, "HTTP header name", 255);
  if (!/^[!#$%&'*+.^_`|~A-Za-z0-9-]+$/.test(value)) {
    throw invalidPayload("HTTP header name is invalid.");
  }
  return value;
}

const RESERVED_MCP_HTTP_HEADERS = new Set([
  "authorization",
  "cookie",
  "host",
  "accept",
  "content-type",
  "content-length",
  "mcp-session-id",
  "mcp-protocol-version",
  "last-event-id",
]);

function asBoolean(raw: unknown, label: string): boolean {
  if (typeof raw !== "boolean") {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw;
}

function asBoundedJsonObject(
  raw: unknown,
  label: string,
): Record<string, unknown> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  let nodes = 0;
  const visit = (value: unknown, depth: number): void => {
    nodes += 1;
    if (nodes > 4_096 || depth > 12) {
      throw invalidPayload(`${label} is invalid.`);
    }
    if (typeof value === "string") {
      if (value.length > 16_384 || value.includes("\0")) {
        throw invalidPayload(`${label} is invalid.`);
      }
      return;
    }
    if (
      value === null ||
      typeof value === "boolean" ||
      (typeof value === "number" && Number.isFinite(value))
    ) {
      return;
    }
    if (Array.isArray(value)) {
      value.forEach((item) => visit(item, depth + 1));
      return;
    }
    if (typeof value === "object") {
      Object.entries(value as Record<string, unknown>).forEach(
        ([key, item]) => {
          if (key.length === 0 || key.length > 1_024 || key.includes("\0")) {
            throw invalidPayload(`${label} is invalid.`);
          }
          visit(item, depth + 1);
        },
      );
      return;
    }
    throw invalidPayload(`${label} is invalid.`);
  };
  visit(raw, 0);
  const encoded = JSON.stringify(raw);
  if (encoded.length > 64 * 1_024) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as Record<string, unknown>;
}

function asGeneration(raw: unknown, allowZero: boolean): number {
  if (!Number.isSafeInteger(raw) || (raw as number) < (allowZero ? 0 : 1)) {
    throw invalidPayload("Generation is invalid.");
  }
  return raw as number;
}

function asOptionalGeneration(raw: unknown): number | null {
  return raw === null ? null : asGeneration(raw, true);
}

function asInteger(raw: unknown, label: string, maximum: number): number {
  const value = asGeneration(raw, true);
  if (value > maximum) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return value;
}

function asIntegerRange(
  raw: unknown,
  label: string,
  minimum: number,
  maximum: number,
): number {
  const value = asGeneration(raw, true);
  if (value < minimum || value > maximum) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return value;
}

function asDigest(raw: unknown): string {
  if (typeof raw !== "string" || !/^sha256:[0-9a-fA-F]{64}$/.test(raw)) {
    throw invalidPayload("Digest is invalid.");
  }
  return raw;
}

function asOptionalDigest(raw: unknown): string | null {
  return raw === null ? null : asDigest(raw);
}

function asEnum<const Values extends readonly string[]>(
  raw: unknown,
  values: Values,
  label: string,
): Values[number] {
  if (typeof raw !== "string" || !values.includes(raw)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as Values[number];
}

function assertUniqueNames(values: readonly string[], label: string) {
  if (new Set(values).size !== values.length) {
    throw invalidPayload(`${label} names are not unique.`);
  }
}

function normalizeMcpFailure(error: unknown, fallbackMessage: string): Error {
  try {
    const value = exactRecord(
      error,
      ["code", "message", "retryable", "correlationId", "details"],
      "structured MCP error",
    );
    const codes: readonly ProtocolErrorCode[] = [
      "unknownProtocolVersion",
      "invalidIdentifier",
      "invalidGeneration",
      "staleGeneration",
      "futureGeneration",
      "correlationMismatch",
      "invalidPayload",
      "payloadTooLarge",
      "notFound",
      "conflict",
      "unauthorized",
      "forbidden",
      "unavailable",
      "internal",
    ];
    const code = asEnum(value.code, codes, "MCP error code");
    if (value.correlationId !== null) {
      asIdentifier(value.correlationId, "MCP error correlation ID");
    }
    if (
      typeof value.message !== "string" ||
      new TextEncoder().encode(value.message).length > MAX_DIAGNOSTIC_BYTES
    ) {
      throw invalidPayload("The MCP error message is invalid.");
    }
    const detailRecord = exactRecordObject(value.details, "MCP error details");
    if (Object.keys(detailRecord).length > MAX_SAFE_DETAILS) {
      throw invalidPayload("The MCP error details are unbounded.");
    }
    const details: Record<string, SafeDetailValue> = {};
    for (const [key, detail] of Object.entries(detailRecord)) {
      asIdentifier(key, "MCP error detail key");
      details[key] = parseSafeMcpDetail(detail);
    }
    return new ProtocolBoundaryError(
      code,
      value.message,
      asBoolean(value.retryable, "MCP retryable flag"),
      details,
    );
  } catch {
    return new ProtocolBoundaryError("unavailable", fallbackMessage, true);
  }
}

function exactRecordObject(
  raw: unknown,
  label: string,
): Record<string, unknown> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    throw invalidPayload(`${label} are invalid.`);
  }
  return raw as Record<string, unknown>;
}

function parseSafeMcpDetail(raw: unknown): SafeDetailValue {
  const detail = exactRecord(raw, ["kind", "value"], "MCP error detail");
  switch (detail.kind) {
    case "text":
      if (
        typeof detail.value !== "string" ||
        new TextEncoder().encode(detail.value).length > MAX_DIAGNOSTIC_BYTES
      ) {
        throw invalidPayload("The MCP detail text is invalid.");
      }
      return { kind: "text", value: detail.value };
    case "integer":
      if (!Number.isSafeInteger(detail.value)) {
        throw invalidPayload("The MCP detail integer is invalid.");
      }
      return { kind: "integer", value: detail.value as number };
    case "boolean":
      return {
        kind: "boolean",
        value: asBoolean(detail.value, "MCP detail boolean"),
      };
    case "redacted": {
      const marker = exactRecord(
        detail.value,
        ["fieldPath", "reason"],
        "MCP redaction marker",
      );
      const reasons = [
        "credential",
        "authorization",
        "environmentValue",
        "websiteStorage",
        "sensitiveField",
        "policy",
      ] as const;
      return {
        kind: "redacted",
        value: {
          fieldPath: asIdentifier(marker.fieldPath, "redacted field path"),
          reason: asEnum(marker.reason, reasons, "redaction reason"),
        },
      };
    }
    default:
      throw invalidPayload("The MCP error detail kind is invalid.");
  }
}

function secureUuid(label: string): string {
  const value = globalThis.crypto?.randomUUID?.();
  if (!value) {
    throw new ProtocolBoundaryError(
      "unavailable",
      `A secure ${label} is unavailable.`,
    );
  }
  return value;
}

function invalidPayload(message: string): ProtocolBoundaryError {
  return new ProtocolBoundaryError("invalidPayload", message);
}
