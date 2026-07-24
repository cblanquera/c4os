import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type CorrelationId,
  type ProcessGeneration,
  type RequestId,
  type StateGeneration,
} from "../../../src/frontend/platform/protocol";
import {
  createTauriAdapter,
  FOUNDATION_SNAPSHOT_COMMAND,
  type FoundationTauriTransport,
} from "../../../src/frontend/platform/tauri-adapter";

const requestId = "request-foundation-1" as RequestId;
const correlationId = "corr-foundation-1" as CorrelationId;

function response(
  generation: number,
  overrides: Record<string, unknown> = {},
): {
  protocolVersion: number;
  requestId: RequestId;
  correlationId: CorrelationId;
  generation: number;
  payload: {
    protocolVersion: number;
    generation: number;
    authority: string;
    redactions: unknown[];
  };
} & Record<string, unknown> {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId,
    correlationId,
    generation,
    payload: {
      protocolVersion: PROTOCOL_VERSION,
      generation,
      authority: "rust-core",
      redactions: [],
    },
    ...overrides,
  };
}

function transportReturning(value: unknown): FoundationTauriTransport {
  return { invoke: vi.fn().mockResolvedValue(value) };
}

function options(initialGeneration = 0) {
  return {
    requestIdFactory: () => requestId,
    correlationIdFactory: () => correlationId,
    initialGeneration: initialGeneration as StateGeneration,
  };
}

describe("createTauriAdapter", () => {
  it("invokes only the allowlisted foundation command with a v1 request", async () => {
    const transport = transportReturning(response(4));
    const adapter = createTauriAdapter(transport, options(3));

    const snapshot = await adapter.readFoundationSnapshot();

    expect(snapshot.generation).toBe(4);
    expect(adapter.currentGeneration).toBe(4);
    expect(transport.invoke).toHaveBeenCalledWith(FOUNDATION_SNAPSHOT_COMMAND, {
      request: {
        protocolVersion: PROTOCOL_VERSION,
        requestId,
        correlationId,
        expectedGeneration: 3,
      },
    });
  });

  it("fails closed on unknown envelope and payload protocol versions", async () => {
    const envelopeAdapter = createTauriAdapter(
      transportReturning(response(1, { protocolVersion: 2 })),
      options(),
    );
    await expect(
      envelopeAdapter.readFoundationSnapshot(),
    ).rejects.toMatchObject({
      code: "unknownProtocolVersion",
    });

    const raw = response(1);
    raw.payload.protocolVersion = 2;
    const payloadAdapter = createTauriAdapter(
      transportReturning(raw),
      options(),
    );
    await expect(payloadAdapter.readFoundationSnapshot()).rejects.toMatchObject(
      {
        code: "unknownProtocolVersion",
      },
    );
  });

  it("rejects mismatched request or correlation identities", async () => {
    const adapter = createTauriAdapter(
      transportReturning(response(1, { correlationId: "corr-other" })),
      options(),
    );

    await expect(adapter.readFoundationSnapshot()).rejects.toMatchObject({
      code: "correlationMismatch",
    });
  });

  it("rejects stale response generations without moving its cursor", async () => {
    const adapter = createTauriAdapter(
      transportReturning(response(6)),
      options(7),
    );

    await expect(adapter.readFoundationSnapshot()).rejects.toMatchObject({
      code: "staleGeneration",
    });
    expect(adapter.currentGeneration).toBe(7);
  });

  it("rejects a payload generation that disagrees with its envelope", async () => {
    const raw = response(3);
    raw.payload.generation = 2;
    const adapter = createTauriAdapter(transportReturning(raw), options());

    await expect(adapter.readFoundationSnapshot()).rejects.toMatchObject({
      code: "staleGeneration",
    });
  });

  it("turns a redacted structured Tauri failure into a boundary error", async () => {
    const transport: FoundationTauriTransport = {
      invoke: vi.fn().mockRejectedValue({
        code: "unavailable",
        message: "Core is starting.",
        retryable: true,
        correlationId,
        details: {
          credential: {
            kind: "redacted",
            value: { fieldPath: "providers.key", reason: "credential" },
          },
        },
      }),
    };
    const adapter = createTauriAdapter(transport, options());

    await expect(adapter.readFoundationSnapshot()).rejects.toMatchObject({
      code: "unavailable",
      retryable: true,
      details: {
        credential: {
          kind: "redacted",
          value: { fieldPath: "providers.key", reason: "credential" },
        },
      },
    });
  });

  it("rejects stale events and accepts a fresh state event", () => {
    const adapter = createTauriAdapter(
      transportReturning(response(8)),
      options(8),
    );
    const event = (generation: number) => ({
      protocolVersion: PROTOCOL_VERSION,
      eventId: "event-1",
      correlationId,
      generation,
      runScope: null,
      event: { type: "stateChanged", payload: { areas: ["workspace"] } },
    });

    expect(() => adapter.acceptCoreEvent(event(7))).toThrow(
      expect.objectContaining({ code: "staleGeneration" }),
    );
    expect(adapter.acceptCoreEvent(event(9)).generation).toBe(9);
    expect(adapter.currentGeneration).toBe(9);
  });

  it("rejects run-scope correlation and process-generation mismatches", () => {
    const adapter = createTauriAdapter(transportReturning(response(1)), {
      ...options(),
      expectedProcessGeneration: 4 as ProcessGeneration,
    });
    const runEvent = (scopeCorrelation: string, processGeneration: number) => ({
      protocolVersion: PROTOCOL_VERSION,
      eventId: "event-run-1",
      correlationId,
      generation: 1,
      runScope: {
        workspaceId: "workspace-1",
        sessionId: "session-1",
        turnId: "turn-1",
        attemptId: "attempt-1",
        runtimeId: "runtime-1",
        environmentId: "environment-1",
        correlationId: scopeCorrelation,
        processGeneration,
      },
      event: {
        type: "runChanged",
        payload: { sequence: 1, phase: "running" },
      },
    });

    expect(() => adapter.acceptCoreEvent(runEvent("corr-other", 4))).toThrow(
      expect.objectContaining({ code: "correlationMismatch" }),
    );
    expect(() => adapter.acceptCoreEvent(runEvent(correlationId, 3))).toThrow(
      expect.objectContaining({ code: "staleGeneration" }),
    );
  });
});
