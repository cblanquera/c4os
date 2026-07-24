import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
} from "./protocol";
import { createUpdateAdapter, type UpdateTransport } from "./update-service";

const requestId = "request:update-1" as RequestId;
const correlationId = "correlation:update-1" as CorrelationId;
const sha = `sha256:${"a".repeat(64)}`;

function snapshot(overrides: Record<string, unknown> = {}) {
  return {
    schemaVersion: 1,
    generation: 4,
    channels: [
      {
        channel: "runtime",
        componentId: "opencode",
        state: "current",
        currentVersion: "1.0.0",
        candidateVersion: null,
        lastKnownGoodVersion: null,
        stagedArtifactSha256: null,
        revoked: false,
        recoveryAction: null,
        failureCode: null,
        updatedAtMs: 1_784_476_800_000,
      },
    ],
    candidates: [
      {
        candidateId: "candidate:opencode-1-1",
        channel: "runtime",
        componentId: "opencode",
        version: "1.1.0",
        artifactSha256: sha,
        compatibilitySha256: sha,
        discoveredAtMs: 1_784_476_800_000,
      },
    ],
    pendingOperations: [],
    recoveryNotices: [],
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

function fixture(...responses: unknown[]) {
  const transport: UpdateTransport = {
    invoke: vi.fn().mockImplementation(() => {
      const next = responses.shift();
      return Promise.resolve(next);
    }),
  };
  return {
    adapter: createUpdateAdapter(transport, {
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

describe("createUpdateAdapter", () => {
  it("stages only an opaque Rust-discovered candidate", async () => {
    const staged = snapshot({
      generation: 5,
      candidates: [],
      channels: [
        {
          ...snapshot().channels[0],
          state: "staged",
          candidateVersion: "1.1.0",
          stagedArtifactSha256: sha,
        },
      ],
    });
    const subject = fixture(response(), response(staged));

    await subject.adapter.readSnapshot();
    await expect(
      subject.adapter.stageLocal({
        candidateId: "candidate:opencode-1-1",
        channel: "runtime",
        componentId: "opencode",
      }),
    ).resolves.toMatchObject({ generation: 5 });

    expect(subject.transport.invoke).toHaveBeenNthCalledWith(
      2,
      "update_stage_local",
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 4,
        },
        input: {
          expectedGeneration: 4,
          candidateId: "candidate:opencode-1-1",
          channel: "runtime",
          componentId: "opencode",
        },
      },
    );
    expect(
      JSON.stringify(vi.mocked(subject.transport.invoke).mock.calls[1]),
    ).not.toContain("compatibilitySha256");
  });

  it("rejects invented candidates, recovery IDs, and invalid actions before invoke", async () => {
    const subject = fixture(response());
    await subject.adapter.readSnapshot();

    expect(() =>
      subject.adapter.stageLocal({
        candidateId: "candidate:invented",
        channel: "runtime",
        componentId: "opencode",
      }),
    ).toThrowError(/not present in the latest native snapshot/);
    expect(() =>
      subject.adapter.recover({
        channel: "runtime",
        componentId: "opencode",
        recoveryId: "recovery:invented",
      }),
    ).toThrowError(/not present in the latest native snapshot/);
    expect(() =>
      subject.adapter.activate({
        channel: "runtime",
        componentId: "opencode",
      }),
    ).toThrowError(/not valid for the latest native component state/);
    expect(subject.transport.invoke).toHaveBeenCalledTimes(1);
  });

  it.each([
    {
      channel: "application" as const,
      componentId: "c4os",
      recoveryAction: "rebuild_application" as const,
    },
    {
      channel: "runtime" as const,
      componentId: "opencode",
      recoveryAction: "rebuild_runtime" as const,
    },
  ])(
    "rejects direct generic activation while staged $channel requires rebuilding",
    async ({ channel, componentId, recoveryAction }) => {
      const blockedSnapshot = snapshot({
        channels: [
          {
            ...snapshot().channels[0],
            channel,
            componentId,
            state: "staged",
            candidateVersion: "1.1.0",
            stagedArtifactSha256: sha,
            recoveryAction,
          },
        ],
        candidates: [],
      });
      const subject = fixture(response(blockedSnapshot));
      await subject.adapter.readSnapshot();

      expect(() =>
        subject.adapter.activate({ channel, componentId }),
      ).toThrowError(
        /activation is not valid for the latest native component state/,
      );
      expect(subject.transport.invoke).toHaveBeenCalledTimes(1);
    },
  );

  it("allows direct activation when the staged snapshot explicitly requests it", async () => {
    const stagedSnapshot = snapshot({
      channels: [
        {
          ...snapshot().channels[0],
          state: "staged",
          candidateVersion: "1.1.0",
          stagedArtifactSha256: sha,
          recoveryAction: "activate_staged",
        },
      ],
      candidates: [],
    });
    const activatedSnapshot = snapshot({
      generation: 5,
      channels: [
        {
          ...snapshot().channels[0],
          state: "activated",
          currentVersion: "1.1.0",
          lastKnownGoodVersion: "1.0.0",
        },
      ],
      candidates: [],
    });
    const subject = fixture(
      response(stagedSnapshot),
      response(activatedSnapshot),
    );
    await subject.adapter.readSnapshot();

    await expect(
      subject.adapter.activate({
        channel: "runtime",
        componentId: "opencode",
      }),
    ).resolves.toMatchObject({ generation: 5 });
    expect(subject.transport.invoke).toHaveBeenNthCalledWith(
      2,
      "update_activate",
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 4,
        },
        input: {
          expectedGeneration: 4,
          channel: "runtime",
          componentId: "opencode",
        },
      },
    );
  });

  it("stages a failed candidate only under explicit retry_stage authority", async () => {
    const failed = snapshot({
      channels: [
        {
          ...snapshot().channels[0],
          state: "failed",
          recoveryAction: "rebuild_runtime",
          failureCode: "runtime_health_failed",
        },
      ],
    });
    const blocked = fixture(response(failed));
    await blocked.adapter.readSnapshot();
    expect(() =>
      blocked.adapter.stageLocal({
        candidateId: "candidate:opencode-1-1",
        channel: "runtime",
        componentId: "opencode",
      }),
    ).toThrowError(/cannot be staged/);
    expect(blocked.transport.invoke).toHaveBeenCalledTimes(1);

    const retryable = snapshot({
      channels: [
        {
          ...snapshot().channels[0],
          state: "failed",
          recoveryAction: "retry_stage",
          failureCode: "artifact_stage_failed",
        },
      ],
    });
    const staged = snapshot({
      generation: 5,
      candidates: [],
      channels: [
        {
          ...snapshot().channels[0],
          state: "staged",
          candidateVersion: "1.1.0",
          stagedArtifactSha256: sha,
        },
      ],
    });
    const allowed = fixture(response(retryable), response(staged));
    await allowed.adapter.readSnapshot();
    await expect(
      allowed.adapter.stageLocal({
        candidateId: "candidate:opencode-1-1",
        channel: "runtime",
        componentId: "opencode",
      }),
    ).resolves.toMatchObject({ generation: 5 });
  });

  it.each([
    [
      "unknown lifecycle",
      { channels: [{ ...snapshot().channels[0], state: "paused" }] },
    ],
    [
      "unknown recovery action",
      {
        channels: [
          { ...snapshot().channels[0], recoveryAction: "run_arbitrary_script" },
        ],
      },
    ],
    [
      "invalid candidate digest",
      {
        candidates: [{ ...snapshot().candidates[0], artifactSha256: "nope" }],
      },
    ],
    [
      "noncanonical uppercase digest",
      {
        candidates: [
          {
            ...snapshot().candidates[0],
            artifactSha256: `sha256:${"A".repeat(64)}`,
          },
        ],
      },
    ],
  ])("fails closed for %s", async (_label, mutation) => {
    const subject = fixture(response(snapshot(mutation)));
    await expect(subject.adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("rejects mismatched identities and regressing generations", async () => {
    await expect(
      fixture(
        response(snapshot(), { correlationId: "correlation:other" }),
      ).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "correlationMismatch" });

    const subject = fixture(
      response(snapshot({ generation: 4 })),
      response(snapshot({ generation: 3 })),
    );
    await subject.adapter.readSnapshot();
    await expect(subject.adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("rejects a deferred response that regresses behind the live cursor", async () => {
    const older = deferred<unknown>();
    const newer = deferred<unknown>();
    const transport: UpdateTransport = {
      invoke: vi
        .fn()
        .mockResolvedValueOnce(response(snapshot({ generation: 4 })))
        .mockReturnValueOnce(older.promise)
        .mockReturnValueOnce(newer.promise),
    };
    const subject = createUpdateAdapter(transport, {
      requestIdFactory: () => requestId,
      correlationIdFactory: () => correlationId,
    });
    await subject.readSnapshot();

    const olderRead = subject.readSnapshot();
    const olderFailure = expect(olderRead).rejects.toMatchObject({
      code: "invalidPayload",
    });
    const newerRead = subject.readSnapshot();
    newer.resolve(response(snapshot({ generation: 6 })));
    await expect(newerRead).resolves.toMatchObject({ generation: 6 });
    older.resolve(response(snapshot({ generation: 5 })));

    await olderFailure;
  });
});
