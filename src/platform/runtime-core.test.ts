import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type ApprovalId,
  type CorrelationId,
  type RequestId,
  type RuntimeId,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import {
  createRuntimeCoreAdapter,
  RUNTIME_CORE_SNAPSHOT_COMMAND,
  RUNTIME_PRODUCTION_ACTIVATE_COMMAND,
  RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND,
  RUNTIME_PRODUCTION_PUMP_COMMAND,
  RUNTIME_PRODUCTION_SHUTDOWN_COMMAND,
  type RuntimeCoreTransport,
} from "./runtime-core";

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
        selectedModelId: "gpt-5",
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
