import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
} from "../../../src/frontend/platform/protocol";
import {
  createStartupRecoveryAdapter,
  type StartupRecoveryTransport,
} from "../../../src/frontend/platform/startup-recovery-service";

const requestId = "request:startup-recovery-1" as RequestId;
const correlationId = "correlation:startup-recovery-1" as CorrelationId;

function failure() {
  return {
    boundary: "database",
    correlationId: "correlation:database-startup",
    diagnosticCode: "database_startup_failed",
    message: "The database did not pass startup validation.",
    failedAtMs: 1_784_476_800_000,
    validatedBackupAvailable: true,
    recoveryLocationAvailable: true,
  };
}

function degraded(overrides: Record<string, unknown> = {}) {
  return {
    schemaVersion: 1,
    authority: "rust-startup-recovery",
    generation: 4,
    lifecycle: "degraded",
    normalWorkAuthorized: false,
    failure: failure(),
    availableActions: [
      "retry",
      "restoreValidatedBackup",
      "openRecoveryLocation",
    ],
    activeAction: null,
    history: [
      {
        generation: 4,
        boundary: "database",
        lifecycle: "degraded",
        kind: "failureReported",
        action: null,
        correlationId: "correlation:database-startup",
        diagnosticCode: "database_startup_failed",
        message: "The database did not pass startup validation.",
        occurredAtMs: 1_784_476_800_000,
      },
    ],
    historyTruncated: 0,
    ...overrides,
  };
}

function healthy(overrides: Record<string, unknown> = {}) {
  return {
    schemaVersion: 1,
    authority: "rust-startup-recovery",
    generation: 5,
    lifecycle: "recovered",
    normalWorkAuthorized: true,
    failure: null,
    availableActions: [],
    activeAction: null,
    history: [],
    historyTruncated: 0,
    ...overrides,
  };
}

function response(
  payload: Record<string, unknown>,
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

function fixture(...responses: unknown[]) {
  const transport: StartupRecoveryTransport = {
    invoke: vi
      .fn()
      .mockImplementation(() => Promise.resolve(responses.shift())),
  };
  return {
    adapter: createStartupRecoveryAdapter(transport, {
      requestIdFactory: () => requestId,
      correlationIdFactory: () => correlationId,
      now: () => 1_784_476_800_100,
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

describe("createStartupRecoveryAdapter", () => {
  it("performs only an advertised exact action with CAS and no path authority", async () => {
    const subject = fixture(response(degraded()), response(healthy()));
    await subject.adapter.readSnapshot();

    await expect(
      subject.adapter.performAction("restoreValidatedBackup"),
    ).resolves.toMatchObject({
      lifecycle: "recovered",
      normalWorkAuthorized: true,
    });
    expect(subject.transport.invoke).toHaveBeenNthCalledWith(
      2,
      "startup_recovery_action",
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 4,
        },
        input: {
          expectedGeneration: 4,
          action: "restoreValidatedBackup",
          requestedAtMs: 1_784_476_800_100,
        },
      },
    );
    expect(
      JSON.stringify(vi.mocked(subject.transport.invoke).mock.calls[1]),
    ).not.toMatch(/path|locationRoot|backupFile/i);
  });

  it("rejects unavailable and pre-snapshot actions before native invoke", async () => {
    const subject = fixture(
      response(degraded()),
      response(degraded({ generation: 5 })),
    );
    expect(() => subject.adapter.performAction("retry")).toThrowError(
      /not available in the latest native snapshot/,
    );
    await subject.adapter.readSnapshot();
    await expect(
      subject.adapter.performAction("openRecoveryLocation"),
    ).resolves.toMatchObject({ lifecycle: "degraded" });

    const noRestore = fixture(
      response(
        degraded({
          failure: {
            ...failure(),
            validatedBackupAvailable: false,
          },
          availableActions: ["retry", "openRecoveryLocation"],
        }),
      ),
    );
    await noRestore.adapter.readSnapshot();
    expect(() =>
      noRestore.adapter.performAction("restoreValidatedBackup"),
    ).toThrowError(/not available in the latest native snapshot/);
    expect(noRestore.transport.invoke).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["lifecycle", { lifecycle: "paused" }],
    ["boundary", { failure: { ...failure(), boundary: "filesystem" } }],
    ["action", { availableActions: ["retry", "runShell"] }],
    [
      "history kind",
      {
        history: [{ ...degraded().history[0], kind: "secretExported" }],
      },
    ],
  ])("fails closed for unknown %s", async (_label, mutation) => {
    await expect(
      fixture(response(degraded(mutation))).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it.each([
    ["authorized degraded state", { normalWorkAuthorized: true }],
    [
      "competing retry actions",
      {
        lifecycle: "retrying",
        activeAction: {
          boundary: "database",
          action: "retry",
          startedAtMs: 1_784_476_800_001,
        },
      },
    ],
    [
      "mismatched action boundary",
      {
        lifecycle: "retrying",
        availableActions: [],
        activeAction: {
          boundary: "runtime",
          action: "retry",
          startedAtMs: 1_784_476_800_001,
        },
      },
    ],
    [
      "history above snapshot generation",
      {
        history: [{ ...degraded().history[0], generation: 5 }],
      },
    ],
    [
      "descending history generations",
      {
        history: [
          degraded().history[0],
          {
            ...degraded().history[0],
            generation: 3,
            kind: "actionFailed",
            action: "retry",
          },
        ],
      },
    ],
    [
      "zero failure timestamp",
      {
        failure: { ...failure(), failedAtMs: 0 },
      },
    ],
  ])("rejects inconsistent %s", async (_label, mutation) => {
    await expect(
      fixture(response(degraded(mutation))).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("ignores unowned path fields and rejects wire identity mismatch", async () => {
    const subject = fixture(
      response(
        degraded({
          recoveryPath: "/private/secret/recovery",
          backupPath: "/Users/example/backup.db",
        }),
      ),
    );
    await expect(subject.adapter.readSnapshot()).resolves.not.toHaveProperty(
      "recoveryPath",
    );

    await expect(
      fixture(
        response(degraded(), { correlationId: "correlation:other" }),
      ).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "correlationMismatch" });
  });

  it("rejects a deferred action response behind a newer live refresh", async () => {
    const action = deferred<unknown>();
    const refresh = deferred<unknown>();
    const transport: StartupRecoveryTransport = {
      invoke: vi
        .fn()
        .mockResolvedValueOnce(response(degraded()))
        .mockReturnValueOnce(action.promise)
        .mockReturnValueOnce(refresh.promise),
    };
    const subject = createStartupRecoveryAdapter(transport, {
      requestIdFactory: () => requestId,
      correlationIdFactory: () => correlationId,
      now: () => 1_784_476_800_100,
    });
    await subject.readSnapshot();

    const olderAction = subject.performAction("retry");
    const olderFailure = expect(olderAction).rejects.toMatchObject({
      code: "invalidPayload",
    });
    const newerRefresh = subject.readSnapshot();
    refresh.resolve(response(degraded({ generation: 6 })));
    await expect(newerRefresh).resolves.toMatchObject({ generation: 6 });
    action.resolve(response(healthy({ generation: 5 })));

    await olderFailure;
  });
});
