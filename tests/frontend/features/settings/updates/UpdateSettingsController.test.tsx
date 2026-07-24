import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { UpdateCoordinatorSnapshot } from "../../../../../src/frontend/platform/update-service";
import { UpdateSettingsController } from "../../../../../src/frontend/features/settings/updates/UpdateSettingsController";

const updateService = vi.hoisted(() => ({
  activateUpdate: vi.fn(),
  readUpdateSnapshot: vi.fn(),
  recoverUpdate: vi.fn(),
  revokeUpdate: vi.fn(),
  rollbackUpdate: vi.fn(),
  stageLocalUpdate: vi.fn(),
}));
const runtimeService = vi.hoisted(() => ({
  readRuntimeCoreSnapshot: vi.fn(),
  reviewRuntimeCrashLoop: vi.fn(),
}));

vi.mock("../../../../../src/frontend/platform/update-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../../../../src/frontend/platform/update-service")
  >("../../../../../src/frontend/platform/update-service");
  return { ...actual, ...updateService };
});

vi.mock("../../../../../src/frontend/platform/runtime-core", async () => {
  const actual = await vi.importActual<
    typeof import("../../../../../src/frontend/platform/runtime-core")
  >("../../../../../src/frontend/platform/runtime-core");
  return { ...actual, ...runtimeService };
});

function snapshot(
  overrides: Partial<UpdateCoordinatorSnapshot> = {},
): UpdateCoordinatorSnapshot {
  return {
    schemaVersion: 1,
    generation: 7,
    channels: [
      {
        channel: "runtime",
        componentId: "opencode",
        state: "staged",
        currentVersion: "1.0.0",
        candidateVersion: "1.1.0",
        lastKnownGoodVersion: "1.0.0",
        stagedArtifactSha256: `sha256:${"a".repeat(64)}`,
        revoked: false,
        recoveryAction: null,
        failureCode: null,
        updatedAtMs: 1_784_476_800_000,
      },
    ],
    candidates: [],
    pendingOperations: [],
    recoveryNotices: [],
    ...overrides,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("UpdateSettingsController", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("reports a native activation deferral without claiming completion", async () => {
    updateService.readUpdateSnapshot.mockResolvedValue(snapshot());
    updateService.activateUpdate.mockResolvedValue(
      snapshot({
        generation: 8,
        channels: [
          {
            ...snapshot().channels[0]!,
            recoveryAction: "wait_for_active_runs",
          },
        ],
      }),
    );
    render(<UpdateSettingsController />);

    fireEvent.click(
      await screen.findByRole("button", { name: "Activate staged update" }),
    );
    expect(
      await screen.findByText(
        /activation is deferred in staged state: wait_for_active_runs/i,
      ),
    ).toBeVisible();
    expect(screen.queryByText(/activation completed/i)).not.toBeInTheDocument();
  });

  it("reviews the exact crash-loop process and refreshes update authority", async () => {
    const degraded = snapshot({
      channels: [
        {
          ...snapshot().channels[0]!,
          componentId: "opencode-primary",
          state: "failed",
          recoveryAction: "review_runtime_crash_loop",
          failureCode: "runtime_crash_loop",
        },
      ],
      recoveryNotices: [
        {
          recoveryId: "recovery:runtime-crash-loop",
          correlationId: "correlation:runtime-crash-loop",
          channel: "runtime",
          componentId: "opencode-primary",
          summary: "Explicit crash-loop review is required.",
          action: "review_runtime_crash_loop",
          createdAtMs: 1_784_476_800_000,
        },
      ],
    });
    const refreshed = snapshot({
      generation: 8,
      channels: [
        {
          ...degraded.channels[0]!,
          recoveryAction: null,
          failureCode: null,
        },
      ],
      recoveryNotices: [],
    });
    updateService.readUpdateSnapshot
      .mockResolvedValueOnce(degraded)
      .mockResolvedValueOnce(refreshed);
    runtimeService.readRuntimeCoreSnapshot.mockResolvedValue({
      authority: "rust-core",
      generation: 11,
      providerGeneration: 4,
      capabilityGeneration: 5,
      runtimeGeneration: 6,
      onboardingReady: true,
      providers: [],
      runtimes: [
        {
          runtimeId: "opencode-primary",
          runtimeKind: "open-code",
          nativeVersion: "1.18.3",
          lifecycle: "failed",
          health: "unhealthy",
          processGeneration: 17,
        },
      ],
      modelRoutes: [],
      pendingApprovals: [],
    });
    runtimeService.reviewRuntimeCrashLoop.mockResolvedValue({
      authority: "rust-core",
      runtimeId: "opencode-primary",
      processGeneration: 17,
      coordinatorGeneration: 12,
    });
    render(<UpdateSettingsController />);

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Review runtime crash loop",
      }),
    );

    await waitFor(() =>
      expect(runtimeService.reviewRuntimeCrashLoop).toHaveBeenCalledWith({
        runtimeId: "opencode-primary",
        processGeneration: 17,
      }),
    );
    await waitFor(() =>
      expect(updateService.readUpdateSnapshot).toHaveBeenCalledTimes(2),
    );
    expect(
      await screen.findByText(
        /opencode-primary crash-loop review is recorded\./,
      ),
    ).toBeVisible();
  });

  it("does not let a deferred refresh overwrite a newer update action", async () => {
    const lateRefresh = deferred<UpdateCoordinatorSnapshot>();
    const activated = snapshot({
      generation: 9,
      channels: [
        {
          ...snapshot().channels[0]!,
          state: "activated",
          currentVersion: "1.1.0",
          candidateVersion: null,
          stagedArtifactSha256: null,
        },
      ],
    });
    updateService.readUpdateSnapshot
      .mockResolvedValueOnce(snapshot())
      .mockReturnValueOnce(lateRefresh.promise);
    updateService.activateUpdate.mockResolvedValueOnce(activated);
    render(<UpdateSettingsController />);

    await screen.findByRole("button", { name: "Activate staged update" });
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Activate staged update" }),
    );
    expect(
      await screen.findByText(/opencode is active at 1\.1\.0\./),
    ).toBeVisible();

    await act(async () => {
      lateRefresh.resolve(snapshot({ generation: 8 }));
      await lateRefresh.promise;
    });

    expect(screen.getByText(/Generation 9/)).toBeVisible();
    expect(screen.getByText(/opencode is active at 1\.1\.0\./)).toBeVisible();
  });
});
