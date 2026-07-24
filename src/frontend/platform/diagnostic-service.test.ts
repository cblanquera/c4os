import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
} from "./protocol";
import {
  createDiagnosticAdapter,
  type DiagnosticTransport,
} from "./diagnostic-service";

const requestId = "request:diagnostic-1" as RequestId;
const correlationId = "correlation:diagnostic-1" as CorrelationId;

function snapshot(overrides: Record<string, unknown> = {}) {
  return {
    schemaVersion: 1,
    generation: 7,
    records: [
      {
        diagnosticId: "diagnostic:1",
        correlationId: "correlation:native-1",
        category: "recovery",
        severity: "warning",
        componentBoundary: "workspace",
        message: "Interrupted archive save recovered from validated state.",
        recoveryAction: "rebuild_last_known_good",
        createdAtMs: 1_784_476_800_000,
      },
    ],
    truncated: false,
    ...overrides,
  };
}

function response(
  payload: Record<string, unknown> = snapshot(),
  overrides: Record<string, unknown> = {},
) {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId,
    correlationId,
    generation: payload.generation,
    payload,
    ...overrides,
  };
}

function adapter(raw: unknown) {
  const transport: DiagnosticTransport = {
    invoke: vi.fn().mockResolvedValue(raw),
  };
  return {
    adapter: createDiagnosticAdapter(transport, {
      requestIdFactory: () => requestId,
      correlationIdFactory: () => correlationId,
    }),
    transport,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("createDiagnosticAdapter", () => {
  it("returns only the bounded redacted diagnostic projection", async () => {
    const subject = adapter(response());
    await expect(subject.adapter.readSnapshot()).resolves.toMatchObject({
      generation: 7,
      records: [
        {
          category: "recovery",
          severity: "warning",
          recoveryAction: "rebuild_last_known_good",
        },
      ],
    });
    expect(subject.transport.invoke).toHaveBeenCalledWith(
      "diagnostics_snapshot",
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 0,
        },
      },
    );
  });

  it.each([
    ["unknown category", { category: "credential_dump" }],
    ["unknown severity", { severity: "critical" }],
    ["unknown recovery action", { recoveryAction: "print_environment" }],
    ["control character", { message: "safe\nunsafe" }],
  ])("fails closed for %s", async (_label, recordMutation) => {
    const subject = adapter(
      response(
        snapshot({
          records: [{ ...snapshot().records[0], ...recordMutation }],
        }),
      ),
    );
    await expect(subject.adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("fails closed for an unknown schema version", async () => {
    await expect(
      adapter(response(snapshot({ schemaVersion: 2 }))).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("validates export digest and never accepts a path field as authority", async () => {
    const exported = {
      ...snapshot(),
      exportId: "export:1",
      createdAtMs: 1_784_476_800_000,
      sha256: `sha256:${"b".repeat(64)}`,
      path: "/private/secret/export.json",
    };
    const subject = adapter(response(exported));

    await expect(subject.adapter.exportSnapshot()).resolves.toEqual({
      schemaVersion: 1,
      generation: 7,
      records: snapshot().records,
      truncated: false,
      exportId: "export:1",
      createdAtMs: 1_784_476_800_000,
      sha256: `sha256:${"b".repeat(64)}`,
    });
  });

  it("rejects correlation mismatch and malformed export digest", async () => {
    await expect(
      adapter(
        response(snapshot(), { requestId: "request:other" }),
      ).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "correlationMismatch" });

    await expect(
      adapter(
        response({
          ...snapshot(),
          exportId: "export:1",
          createdAtMs: 1_784_476_800_000,
          sha256: "invalid",
        }),
      ).adapter.exportSnapshot(),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("rejects a deferred response that regresses behind the live cursor", async () => {
    const older = deferred<unknown>();
    const newer = deferred<unknown>();
    const transport: DiagnosticTransport = {
      invoke: vi
        .fn()
        .mockReturnValueOnce(older.promise)
        .mockReturnValueOnce(newer.promise),
    };
    const subject = createDiagnosticAdapter(transport, {
      requestIdFactory: () => requestId,
      correlationIdFactory: () => correlationId,
    });

    const olderRead = subject.readSnapshot();
    const olderFailure = expect(olderRead).rejects.toMatchObject({
      code: "invalidPayload",
    });
    const newerRead = subject.readSnapshot();
    newer.resolve(response(snapshot({ generation: 9 })));
    await expect(newerRead).resolves.toMatchObject({ generation: 9 });
    older.resolve(response(snapshot({ generation: 8 })));

    await olderFailure;
  });
});
