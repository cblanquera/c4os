import { describe, expect, it, vi } from "vitest";

import {
  createMcpAdapter,
  type McpServerDefinitionInput,
  type McpTransport,
} from "../../../src/frontend/platform/mcp-service";

const digest = `sha256:${"a".repeat(64)}`;

function server() {
  return {
    serverId: "docs.server",
    displayName: "Documentation server",
    source: { kind: "user" },
    scope: { kind: "application" },
    transport: {
      kind: "stdio",
      command: "/usr/bin/docs-mcp",
      arguments: ["--stdio"],
      environment: [
        { name: "LANG", source: { kind: "passthrough" } },
        {
          name: "DOCS_TOKEN",
          source: {
            kind: "secret",
            reference: {
              kind: "vault",
              credentialReference: "credential.docs",
            },
          },
        },
      ],
      workingDirectory: { kind: "c4osHome" },
      executableSha256: digest,
    },
    trust: "trusted",
    trustedDefinitionSha256: digest,
    pendingTrustApproval: null,
    lifecycle: "disabled",
    timeoutMs: 30_000,
    maxOutputBytes: 1_048_576,
    lifecycleGeneration: 1,
    restartAttempts: 0,
    nextRestartAtMs: null,
    protocolVersion: null,
    serverName: null,
    serverVersion: null,
    instructionsPresent: false,
    capabilities: {
      tools: false,
      toolListChanged: false,
      resources: false,
      resourceListChanged: false,
      resourceSubscribe: false,
      prompts: false,
      logging: false,
      completions: false,
      tasks: false,
      experimentalKeys: [],
    },
    tools: [],
    resources: [],
    activeRequests: 0,
    lastConnectedAtMs: null,
    lastFailureCode: null,
    lastFailureDetail: null,
    lastEventId: 1,
  };
}

function snapshot(generation: number) {
  return {
    schemaVersion: 1,
    generation,
    servers: [server()],
    activeWorkers: 0,
    lastEventId: generation,
  };
}

function responder() {
  return vi.fn<McpTransport["invoke"]>(async (command, args) => {
    const request = args.request as {
      requestId: string;
      correlationId: string;
    };
    const generation = command === "mcp_snapshot" ? 4 : 5;
    return {
      protocolVersion: 1,
      requestId: request.requestId,
      correlationId: request.correlationId,
      generation,
      payload: snapshot(generation),
    };
  });
}

describe("MCP adapter", () => {
  it("carries the observed generation into exact lifecycle mutations", async () => {
    const invoke = responder();
    const adapter = createMcpAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-mcp" as never,
        correlationIdFactory: () => "correlation-mcp" as never,
      },
    );

    await adapter.readSnapshot();
    await adapter.enableServer("docs.server");

    expect(invoke).toHaveBeenLastCalledWith("mcp_enable_server", {
      request: expect.objectContaining({ expectedGeneration: 4 }),
      input: { expectedGeneration: 4, serverId: "docs.server" },
    });

    await adapter.recoverServer("docs.server");
    expect(invoke).toHaveBeenLastCalledWith("mcp_recover_server", {
      request: expect.objectContaining({ expectedGeneration: 5 }),
      input: { expectedGeneration: 5, serverId: "docs.server" },
    });
  });

  it("carries one exact pending trust prompt into an explicit answer", async () => {
    const invoke = vi.fn<McpTransport["invoke"]>(async (command, args) => {
      const request = args.request as {
        requestId: string;
        correlationId: string;
      };
      const envelope = (generation: number, payload: unknown) => ({
        protocolVersion: 1,
        requestId: request.requestId,
        correlationId: request.correlationId,
        generation,
        payload,
      });
      if (command === "mcp_snapshot") return envelope(4, snapshot(4));
      if (command === "mcp_request_trust") {
        const pendingApproval = {
          promptId: "prompt-mcp",
          definitionSha256: digest,
          actionBindingSha256: `sha256:${"b".repeat(64)}`,
          actionConfigurationVersion: 4,
          requestedAtMs: 10,
          expiresAtMs: 100,
          state: "pending",
        };
        return envelope(5, {
          snapshot: {
            ...snapshot(5),
            servers: [
              {
                ...server(),
                trust: "pending",
                trustedDefinitionSha256: null,
                pendingTrustApproval: pendingApproval,
              },
            ],
          },
          status: "pendingApproval",
          serverId: "docs.server",
          definitionSha256: digest,
          promptId: "prompt-mcp",
          promptExpiresAtMs: 100,
        });
      }
      return envelope(6, {
        snapshot: snapshot(6),
        status: "trusted",
        serverId: "docs.server",
        definitionSha256: digest,
        promptId: null,
        promptExpiresAtMs: null,
      });
    });
    const adapter = createMcpAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-trust" as never,
        correlationIdFactory: () => "correlation-trust" as never,
      },
    );

    await adapter.readSnapshot();
    const pending = await adapter.requestTrust("docs.server");
    expect(pending.status).toBe("pendingApproval");
    expect(pending.promptId).toBe("prompt-mcp");
    if (pending.promptId === null) throw new Error("missing pending prompt");
    const trusted = await adapter.answerTrust(
      "docs.server",
      pending.promptId,
      "allow",
    );
    expect(trusted.status).toBe("trusted");
    expect(invoke).toHaveBeenNthCalledWith(2, "mcp_request_trust", {
      request: expect.objectContaining({ expectedGeneration: 4 }),
      input: { expectedGeneration: 4, serverId: "docs.server" },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "mcp_answer_trust", {
      request: expect.objectContaining({ expectedGeneration: 5 }),
      input: {
        expectedGeneration: 5,
        serverId: "docs.server",
        promptId: "prompt-mcp",
        answer: "allow",
      },
    });
  });

  it("overrides a stale form generation and forwards only opaque references", async () => {
    const invoke = responder();
    const adapter = createMcpAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-save" as never,
        correlationIdFactory: () => "correlation-save" as never,
      },
    );
    await adapter.readSnapshot();

    const input = {
      expectedGeneration: 1,
      serverId: "remote.docs",
      displayName: "Remote documentation",
      scope: { kind: "workspace", workspaceId: "workspace.docs" },
      transport: {
        kind: "streamableHttp",
        url: "https://mcp.example.test/rpc",
        bearer: { kind: "environment", variable: "DOCS_MCP_TOKEN" },
        headers: [
          {
            name: "X-Workspace",
            source: { kind: "environment", variable: "WORKSPACE_SLUG" },
          },
          {
            name: "X-API-Key",
            source: {
              kind: "secret",
              reference: {
                kind: "vault",
                credentialReference: "credential.remote-docs",
              },
            },
          },
        ],
      },
      timeoutMs: 45_000,
      maxOutputBytes: 2_097_152,
    } satisfies McpServerDefinitionInput;

    await adapter.saveServer(input);

    expect(invoke).toHaveBeenLastCalledWith("mcp_save_server", {
      request: expect.objectContaining({ expectedGeneration: 4 }),
      input: { ...input, expectedGeneration: 4 },
    });
    expect(JSON.stringify(invoke.mock.calls.at(-1))).not.toContain(
      "raw-secret-value",
    );
  });

  it("strictly rejects unknown native fields before publishing state", async () => {
    const adapter = createMcpAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 1,
            payload: { ...snapshot(1), rawCredential: "raw-secret-value" },
          };
        },
      },
      {
        requestIdFactory: () => "request-strict" as never,
        correlationIdFactory: () => "correlation-strict" as never,
      },
    );

    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("rejects renderer-authored trust before native invocation", async () => {
    const invoke = responder();
    const adapter = createMcpAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-forged-trust" as never,
        correlationIdFactory: () => "correlation-forged-trust" as never,
      },
    );
    await adapter.readSnapshot();

    await expect(
      adapter.saveServer({
        expectedGeneration: 4,
        serverId: "forged.trust",
        displayName: "Forged trust",
        scope: { kind: "application" },
        transport: {
          kind: "stdio",
          command: "/usr/bin/false",
          arguments: [],
          environment: [],
          workingDirectory: { kind: "c4osHome" },
          executableSha256: digest,
        },
        timeoutMs: 30_000,
        maxOutputBytes: 1_048_576,
        trusted: true,
      } as unknown as McpServerDefinitionInput),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("rejects ready peers that did not negotiate the frozen MCP version", async () => {
    const adapter = createMcpAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          const incompatible = {
            ...server(),
            lifecycle: "ready",
            protocolVersion: "2025-03-26",
          };
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 1,
            payload: { ...snapshot(1), servers: [incompatible] },
          };
        },
      },
      {
        requestIdFactory: () => "request-version" as never,
        correlationIdFactory: () => "correlation-version" as never,
      },
    );

    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("rejects response identity mismatches", async () => {
    const adapter = createMcpAdapter(
      {
        async invoke() {
          return {
            protocolVersion: 1,
            requestId: "different-request",
            correlationId: "different-correlation",
            generation: 1,
            payload: snapshot(1),
          };
        },
      },
      {
        requestIdFactory: () => "request-correlation" as never,
        correlationIdFactory: () => "correlation-correlation" as never,
      },
    );

    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "correlationMismatch",
    });
  });

  it("rejects reserved HTTP headers before native invocation", async () => {
    const invoke = responder();
    const adapter = createMcpAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-header" as never,
        correlationIdFactory: () => "correlation-header" as never,
      },
    );
    await adapter.readSnapshot();

    await expect(
      adapter.saveServer({
        expectedGeneration: 4,
        serverId: "bad.header",
        displayName: "Bad header",
        scope: { kind: "application" },
        transport: {
          kind: "streamableHttp",
          url: "https://mcp.example.test",
          bearer: { kind: "environment", variable: "MCP_TOKEN" },
          headers: [
            { name: "Authorization", source: { kind: "literal", value: "no" } },
          ],
        },
        timeoutMs: 30_000,
        maxOutputBytes: 1_048_576,
      }),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    await expect(
      adapter.saveServer({
        expectedGeneration: 4,
        serverId: "bad.protocol-header",
        displayName: "Bad protocol header",
        scope: { kind: "application" },
        transport: {
          kind: "streamableHttp",
          url: "https://mcp.example.test",
          bearer: { kind: "environment", variable: "MCP_TOKEN" },
          headers: [
            {
              name: "MCP-Protocol-Version",
              source: { kind: "literal", value: "forged" },
            },
          ],
        },
        timeoutMs: 30_000,
        maxOutputBytes: 1_048_576,
      }),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("rejects relative STDIO commands and reserved process variables before native invocation", async () => {
    const invoke = responder();
    const adapter = createMcpAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-stdio-boundary" as never,
        correlationIdFactory: () => "correlation-stdio-boundary" as never,
      },
    );
    await adapter.readSnapshot();
    const definition = {
      expectedGeneration: 4,
      serverId: "bad.stdio",
      displayName: "Bad STDIO",
      scope: { kind: "application" as const },
      transport: {
        kind: "stdio" as const,
        command: "relative-mcp",
        arguments: [],
        environment: [],
        workingDirectory: { kind: "c4osHome" as const },
        executableSha256: digest,
      },
      timeoutMs: 30_000,
      maxOutputBytes: 1_048_576,
    };
    await expect(adapter.saveServer(definition)).rejects.toMatchObject({
      code: "invalidPayload",
    });
    await expect(
      adapter.saveServer({
        ...definition,
        transport: {
          ...definition.transport,
          command: "/usr/bin/false",
          environment: [{ name: "PATH", source: { kind: "passthrough" } }],
        },
      }),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("preserves bounded structured native MCP failures", async () => {
    const adapter = createMcpAdapter(
      {
        async invoke() {
          throw {
            code: "conflict",
            message: "The MCP lifecycle generation changed.",
            retryable: true,
            correlationId: "correlation-native-conflict",
            details: {
              lifecycle: { kind: "text", value: "executing" },
              generation: { kind: "integer", value: 9 },
            },
          };
        },
      },
      {
        requestIdFactory: () => "request-native-conflict" as never,
        correlationIdFactory: () => "correlation-native-conflict" as never,
      },
    );

    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "conflict",
      message: "The MCP lifecycle generation changed.",
      retryable: true,
      details: {
        lifecycle: { kind: "text", value: "executing" },
        generation: { kind: "integer", value: 9 },
      },
    });
  });

  it("fails closed when a native MCP rejection is malformed", async () => {
    const adapter = createMcpAdapter(
      {
        async invoke() {
          throw {
            code: "conflict",
            message: "forged",
            retryable: "yes",
            correlationId: null,
            details: { secret: "raw-secret-value" },
          };
        },
      },
      {
        requestIdFactory: () => "request-malformed-error" as never,
        correlationIdFactory: () => "correlation-malformed-error" as never,
      },
    );

    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "unavailable",
      retryable: true,
      message: "The native MCP service is unavailable.",
    });
  });
});
