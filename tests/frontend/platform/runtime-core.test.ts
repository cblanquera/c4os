import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type ApprovalId,
  type CorrelationId,
  type ProcessGeneration,
  type RequestId,
  type RuntimeId,
  type SnapshotRequest,
  type StateGeneration,
} from "../../../src/frontend/platform/protocol";
import {
  createRuntimeCoreAdapter,
  RUNTIME_CORE_SNAPSHOT_COMMAND,
  RUNTIME_REVIEW_CRASH_LOOP_COMMAND,
  RUNTIME_PRODUCTION_ACTIVATE_COMMAND,
  RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND,
  RUNTIME_PRODUCTION_PUMP_COMMAND,
  RUNTIME_PRODUCTION_SHUTDOWN_COMMAND,
  type RuntimeCoreTransport,
} from "../../../src/frontend/platform/runtime-core";

const requestId = "request-runtime" as RequestId;
const requestCorrelationId = "correlation-runtime" as CorrelationId;
const runtimeId = "runtime-opencode" as RuntimeId;
const runCorrelationId = "correlation-run" as CorrelationId;
const promptId = "approval:prompt-1" as ApprovalId;

function snapshotPayload() {
  return {
    authority: "rust-core",
    providerGeneration: 3,
    capabilityGeneration: 6,
    runtimeGeneration: 2,
    onboardingReady: true,
    providers: [
      {
        providerId: "provider-openai",
        displayName: "OpenAI",
        enabled: true,
        testStatus: { state: "succeeded", checkedAtMs: 10 },
        modelCount: 2,
        selectedModelId: "openai/gpt-5",
      },
    ],
    modelRoutes: [
      {
        providerId: "provider-openai",
        modelId: "openai/gpt-5",
        adapterKind: "opencode",
        runtimeKind: "opencode",
        nativeRuntimeVersion: "1.18.3",
        lifecycle: "active",
        contextTokens: 400_000,
        capabilities: Object.fromEntries(
          ["vision", "tools", "reasoning", "audio"].map((key) => [
            key,
            {
              state: key === "audio" ? "unsupported" : "supported",
              source: "c4os.effective",
              checkedAtMs: 10,
              expiresAtMs: 20,
              detail: key === "audio" ? "Audio input is unavailable." : null,
            },
          ]),
        ),
      },
    ],
    runtimes: [
      {
        runtimeId,
        runtimeKind: "open-code",
        nativeVersion: "1.18.3",
        lifecycle: "ready",
        health: "healthy",
        processGeneration: 7,
      },
    ],
    pendingApprovals: [
      {
        runtimeId,
        correlationId: runCorrelationId,
        promptId,
        approvalKind: "runtime-effect",
        summary: "Approval required by opencode-primary.",
        serverId: null,
        providerId: null,
        modelId: null,
        maxTokens: null,
        expiresAtMs: null,
        messageCount: null,
        inputBytes: null,
        hasSystemPrompt: null,
        parentOperation: null,
        disclosureScope: null,
      },
    ],
  };
}

function envelope(
  request: SnapshotRequest,
  generation: number,
  payload: unknown,
  overrides: Readonly<Record<string, unknown>> = {},
) {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: request.requestId,
    correlationId: request.correlationId,
    generation,
    payload,
    ...overrides,
  };
}

function adapterWithTransport(
  transport: RuntimeCoreTransport,
  initialGeneration = 3,
) {
  return createRuntimeCoreAdapter(transport, {
    requestIdFactory: () => requestId,
    correlationIdFactory: () => requestCorrelationId,
    initialGeneration: initialGeneration as StateGeneration,
  });
}

describe("runtime core adapter", () => {
  it("uses exact command shapes and advances one shared CAS cursor", async () => {
    const calls: unknown[] = [];
    const responses = [
      { generation: 4, payload: snapshotPayload() },
      {
        generation: 5,
        payload: {
          coordinatorGeneration: 5,
          capabilityGeneration: 7,
          authorityGeneration: 4,
          runtimeId,
          processGeneration: 8,
          processId: 42,
        },
      },
      {
        generation: 5,
        payload: {
          runtimeId,
          pumpedEvents: 2,
          coordinatorGeneration: 5,
        },
      },
      {
        generation: 6,
        payload: { runtimeId, correlationId: runCorrelationId, promptId },
      },
      {
        generation: 7,
        payload: { runtimeId, coordinatorGeneration: 7 },
      },
    ];
    let responseIndex = 0;
    const transport: RuntimeCoreTransport = {
      async invoke(command, args) {
        calls.push({ command, args });
        const response = responses[responseIndex];
        responseIndex += 1;
        if (response === undefined) {
          throw new Error("The deterministic response queue was exhausted.");
        }
        return envelope(args.request, response.generation, response.payload);
      },
    };
    const adapter = adapterWithTransport(transport);

    const snapshot = await adapter.readSnapshot();
    expect(snapshot).toMatchObject({
      authority: "rust-core",
      generation: 4,
      providerGeneration: 3,
      capabilityGeneration: 6,
      runtimeGeneration: 2,
      onboardingReady: true,
    });
    expect(snapshot.providers[0]?.selectedModelId).toBe("openai/gpt-5");
    expect(snapshot.modelRoutes[0]).toMatchObject({
      providerId: "provider-openai",
      modelId: "openai/gpt-5",
      contextTokens: 400_000,
      capabilities: { tools: { state: "supported" } },
    });
    const publishedApproval = snapshot.pendingApprovals[0];
    if (publishedApproval === undefined) {
      throw new Error("The runtime snapshot omitted its pending approval.");
    }
    await expect(adapter.activateRuntime(runtimeId)).resolves.toEqual({
      runtimeId,
      coordinatorGeneration: 5,
      capabilityGeneration: 7,
      authorityGeneration: 4,
      processGeneration: 8,
      processId: 42,
    });
    await expect(adapter.pumpRuntime(runtimeId)).resolves.toEqual({
      runtimeId,
      pumpedEvents: 2,
      coordinatorGeneration: 5,
    });
    await expect(
      adapter.answerApproval({
        ...publishedApproval,
        answer: "allow",
        remember: "once",
      }),
    ).resolves.toEqual({
      runtimeId,
      correlationId: runCorrelationId,
      promptId,
    });
    await expect(adapter.shutdownRuntime(runtimeId)).resolves.toEqual({
      runtimeId,
      coordinatorGeneration: 7,
    });

    expect(adapter.currentGeneration).toBe(7);
    expect(calls).toEqual([
      {
        command: RUNTIME_CORE_SNAPSHOT_COMMAND,
        args: { request: requestAt(3) },
      },
      {
        command: RUNTIME_PRODUCTION_ACTIVATE_COMMAND,
        args: { request: requestAt(4), runtimeId },
      },
      {
        command: RUNTIME_PRODUCTION_PUMP_COMMAND,
        args: { request: requestAt(5), runtimeId },
      },
      {
        command: RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND,
        args: {
          request: requestAt(5),
          runtimeId,
          correlationId: runCorrelationId,
          promptId,
          answer: "allow",
          remember: "once",
        },
      },
      {
        command: RUNTIME_PRODUCTION_SHUTDOWN_COMMAND,
        args: { request: requestAt(6), runtimeId },
      },
    ]);
  });

  it("rejects stale generations without advancing the CAS cursor", async () => {
    const transport: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 2, {
          runtimeId,
          coordinatorGeneration: 2,
        });
      },
    };
    const adapter = adapterWithTransport(transport);

    await expect(adapter.shutdownRuntime(runtimeId)).rejects.toMatchObject({
      code: "staleGeneration",
    });
    expect(adapter.currentGeneration).toBe(3);
  });

  it("reviews only the exact runtime crash-loop process under the shared cursor", async () => {
    const invoke = vi.fn(
      async (command, args: { readonly request: SnapshotRequest }) => {
        expect(command).toBe(RUNTIME_REVIEW_CRASH_LOOP_COMMAND);
        return envelope(args.request, 4, {
          authority: "rust-core",
          runtimeId,
          processGeneration: 7,
          coordinatorGeneration: 4,
        });
      },
    );
    const transport: RuntimeCoreTransport = {
      invoke,
    };
    const adapter = adapterWithTransport(transport);

    await expect(
      adapter.reviewCrashLoop({
        runtimeId,
        processGeneration: 7 as ProcessGeneration,
      }),
    ).resolves.toEqual({
      authority: "rust-core",
      runtimeId,
      processGeneration: 7,
      coordinatorGeneration: 4,
    });
    expect(invoke).toHaveBeenCalledWith(RUNTIME_REVIEW_CRASH_LOOP_COMMAND, {
      request: requestAt(3),
      input: {
        expectedCoordinatorGeneration: 3,
        runtimeId,
        processGeneration: 7,
      },
    });
    expect(adapter.currentGeneration).toBe(4);
  });

  it("preserves structured crash-loop review failures", async () => {
    const transport: RuntimeCoreTransport = {
      invoke: vi.fn().mockRejectedValue({
        code: "conflict",
        message: "The crash-loop identity changed.",
        retryable: true,
      }),
    };

    await expect(
      adapterWithTransport(transport).reviewCrashLoop({
        runtimeId,
        processGeneration: 7 as ProcessGeneration,
      }),
    ).rejects.toMatchObject({
      code: "conflict",
      message: "The crash-loop identity changed.",
      retryable: true,
    });
  });

  it("publishes an informed MCP sampling approval without private prompt content", async () => {
    const payload = snapshotPayload();
    const samplingApproval = {
      runtimeId: "pi-primary",
      correlationId: "correlation-sampling",
      promptId: "approval:sampling-1",
      approvalKind: "mcp-sampling",
      summary:
        "MCP server docs requests up to 256 tokens from provider-openai/gpt-5-mini using 2 bounded text message(s).",
      serverId: "docs",
      providerId: "provider-openai",
      modelId: "gpt-5-mini",
      maxTokens: 256,
      expiresAtMs: 1_721_300_010_000,
      messageCount: 2,
      inputBytes: 412,
      hasSystemPrompt: true,
      parentOperation: "c4os_propose_action",
      disclosureScope:
        "Private active-operation text will be disclosed to the selected model provider; credentials remain operation-scoped and hidden.",
    };
    const transport: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, {
          ...payload,
          pendingApprovals: [samplingApproval],
        });
      },
    };

    const snapshot = await adapterWithTransport(transport).readSnapshot();

    expect(snapshot.pendingApprovals).toEqual([samplingApproval]);
    expect(JSON.stringify(snapshot.pendingApprovals)).not.toContain(
      "private prompt canary",
    );
  });

  it("rejects partially populated MCP sampling approval metadata", async () => {
    const payload = snapshotPayload();
    const transport: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, {
          ...payload,
          pendingApprovals: [
            {
              ...payload.pendingApprovals[0],
              approvalKind: "mcp-sampling",
              serverId: "docs",
            },
          ],
        });
      },
    };

    await expect(
      adapterWithTransport(transport).readSnapshot(),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("rejects protocol and request-correlation mismatches", async () => {
    const unknownProtocol: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, snapshotPayload(), {
          protocolVersion: 2,
        });
      },
    };
    const wrongCorrelation: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, snapshotPayload(), {
          correlationId: "correlation-other",
        });
      },
    };

    await expect(
      adapterWithTransport(unknownProtocol).readSnapshot(),
    ).rejects.toMatchObject({ code: "unknownProtocolVersion" });
    await expect(
      adapterWithTransport(wrongCorrelation).readSnapshot(),
    ).rejects.toMatchObject({ code: "correlationMismatch" });
  });

  it("rejects unknown command payload shapes and generation disagreement", async () => {
    const unknownPayload: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, {
          runtimeId,
          pumpedEvents: 1,
          coordinatorGeneration: 4,
          capabilityGeneration: 7,
          authorityGeneration: 4,
          unknownField: true,
        });
      },
    };
    const generationDisagreement: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, {
          runtimeId,
          pumpedEvents: 1,
          coordinatorGeneration: 5,
        });
      },
    };

    await expect(
      adapterWithTransport(unknownPayload).pumpRuntime(runtimeId),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    await expect(
      adapterWithTransport(generationDisagreement).pumpRuntime(runtimeId),
    ).rejects.toMatchObject({ code: "staleGeneration" });
  });

  it("rejects unsafe request and response identifiers before publication", async () => {
    const invoke = vi.fn();
    const unavailableTransport: RuntimeCoreTransport = {
      invoke,
    };
    const badResponse: RuntimeCoreTransport = {
      async invoke(_command, { request }) {
        return envelope(request, 4, {
          coordinatorGeneration: 4,
          runtimeId: "runtime with spaces",
          processGeneration: 8,
          processId: 42,
        });
      },
    };

    await expect(
      adapterWithTransport(unavailableTransport).activateRuntime(
        "runtime with spaces" as RuntimeId,
      ),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    expect(invoke).not.toHaveBeenCalled();
    await expect(
      adapterWithTransport(badResponse).activateRuntime(runtimeId),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("rejects unknown approval answers before invoking native code", async () => {
    const invoke = vi.fn();
    const transport: RuntimeCoreTransport = { invoke };

    await expect(
      adapterWithTransport(transport).answerApproval({
        runtimeId,
        correlationId: runCorrelationId,
        promptId,
        answer: "approve" as never,
        remember: "once",
      }),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    expect(invoke).not.toHaveBeenCalled();
  });
});

function requestAt(expectedGeneration: number): SnapshotRequest {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId,
    correlationId: requestCorrelationId,
    expectedGeneration: expectedGeneration as StateGeneration,
  };
}
