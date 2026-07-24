import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { UpdateCoordinatorSnapshot } from "../../../../../src/frontend/platform/update-service";
import { UpdateSettings } from "../../../../../src/frontend/features/settings/updates/UpdateSettings";
import type {
  UpdateSettingsActions,
  UpdateSettingsSnapshot,
} from "../../../../../src/frontend/features/settings/updates/types";

const sha = `sha256:${"a".repeat(64)}`;

function native(
  overrides: Partial<UpdateCoordinatorSnapshot> = {},
): UpdateCoordinatorSnapshot {
  return {
    schemaVersion: 1,
    generation: 8,
    channels: [
      {
        channel: "runtime",
        componentId: "opencode",
        state: "current",
        currentVersion: "1.0.0",
        candidateVersion: null,
        lastKnownGoodVersion: "1.0.0",
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

function ready(
  overrides: Partial<UpdateCoordinatorSnapshot> = {},
): Extract<UpdateSettingsSnapshot, { status: "ready" }> {
  return {
    status: "ready",
    ...native(overrides),
    operation: {
      status: "idle",
      message: "No update operation is running.",
    },
  };
}

function actions(): UpdateSettingsActions {
  return {
    onRetry: vi.fn().mockResolvedValue(undefined),
    onStage: vi.fn().mockResolvedValue(undefined),
    onActivate: vi.fn().mockResolvedValue(undefined),
    onRollback: vi.fn().mockResolvedValue(undefined),
    onRevoke: vi.fn().mockResolvedValue(undefined),
    onRecover: vi.fn().mockResolvedValue(undefined),
    onReviewRuntimeCrashLoop: vi.fn().mockResolvedValue(undefined),
  };
}

describe("UpdateSettings", () => {
  it("composes native current, candidate, generation, and last-known-good state", () => {
    const handlers = actions();
    render(<UpdateSettings actions={handlers} snapshot={ready()} />);

    expect(screen.getByText(/Generation 8/)).toBeVisible();
    expect(screen.getByText("Current").nextElementSibling).toHaveTextContent(
      "1.0.0",
    );
    expect(screen.getByText("Candidate").nextElementSibling).toHaveTextContent(
      "1.1.0",
    );
    expect(
      screen.getByText("Last known-good").nextElementSibling,
    ).toHaveTextContent("1.0.0");
    expect(screen.getByText("Waiting to stage")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Stage 1.1.0" }));
    expect(handlers.onStage).toHaveBeenCalledWith({
      candidateId: "candidate:opencode-1-1",
      channel: "runtime",
      componentId: "opencode",
    });
  });

  it("keeps revoked state blocked without offering unavailable removal", () => {
    const handlers = actions();
    const channel = {
      ...native().channels[0]!,
      state: "revoked" as const,
      currentVersion: "1.1.0",
      lastKnownGoodVersion: "1.0.0",
      revoked: true,
      recoveryAction: null,
    };
    render(
      <UpdateSettings
        actions={handlers}
        snapshot={ready({
          channels: [channel],
          candidates: [],
          recoveryNotices: [],
        })}
      />,
    );

    expect(screen.getByText("Version revoked")).toBeVisible();
    expect(screen.getByText(/Activation stays blocked/)).toBeVisible();
    expect(
      screen.queryByText("Recovery: Remove Revocation"),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Run recovery" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Roll back to 1.0.0" }),
    ).toBeEnabled();
    expect(
      screen.queryByRole("button", { name: /Activate staged update/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Revoke staged update/ }),
    ).not.toBeInTheDocument();
    expect(handlers.onRecover).not.toHaveBeenCalled();
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
    "does not offer generic activation while staged $channel requires rebuilding",
    ({ channel, componentId, recoveryAction }) => {
      const handlers = actions();
      const component = {
        ...native().channels[0]!,
        channel,
        componentId,
        state: "staged" as const,
        candidateVersion: "1.1.0",
        stagedArtifactSha256: sha,
        recoveryAction,
      };
      render(
        <UpdateSettings
          actions={handlers}
          snapshot={ready({ channels: [component], candidates: [] })}
        />,
      );

      expect(
        screen.queryByRole("button", { name: "Activate staged update" }),
      ).not.toBeInTheDocument();
      expect(screen.getByText(/Recovery: Rebuild/)).toBeVisible();
      expect(handlers.onActivate).not.toHaveBeenCalled();
    },
  );

  it("offers generic activation for an explicitly activatable staged snapshot", () => {
    const handlers = actions();
    const component = {
      ...native().channels[0]!,
      state: "staged" as const,
      candidateVersion: "1.1.0",
      stagedArtifactSha256: sha,
      recoveryAction: "activate_staged" as const,
    };
    render(
      <UpdateSettings
        actions={handlers}
        snapshot={ready({ channels: [component], candidates: [] })}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Activate staged update" }),
    );
    expect(handlers.onActivate).toHaveBeenCalledWith({
      channel: "runtime",
      componentId: "opencode",
    });
  });

  it("selects the highest semantic candidate independent of array order", () => {
    const handlers = actions();
    const baseCandidate = native().candidates[0]!;
    render(
      <UpdateSettings
        actions={handlers}
        snapshot={ready({
          candidates: [
            baseCandidate,
            {
              ...baseCandidate,
              candidateId: "candidate:opencode-1-10",
              version: "1.10.0",
              discoveredAtMs: baseCandidate.discoveredAtMs - 1_000,
            },
            {
              ...baseCandidate,
              candidateId: "candidate:opencode-1-2",
              version: "1.2.0",
              discoveredAtMs: baseCandidate.discoveredAtMs + 1_000,
            },
          ],
        })}
      />,
    );

    expect(screen.getByText("Candidate").nextElementSibling).toHaveTextContent(
      "1.10.0",
    );
    fireEvent.click(screen.getByRole("button", { name: "Stage 1.10.0" }));
    expect(handlers.onStage).toHaveBeenCalledWith({
      candidateId: "candidate:opencode-1-10",
      channel: "runtime",
      componentId: "opencode",
    });
  });

  it("claims a degraded last-known-good fallback only where established", () => {
    const channel = {
      ...native().channels[0]!,
      state: "failed" as const,
      lastKnownGoodVersion: null,
      recoveryAction: "rebuild_runtime" as const,
      failureCode: "runtime_health_failed",
    };
    render(
      <UpdateSettings
        actions={actions()}
        snapshot={ready({ channels: [channel], candidates: [] })}
      />,
    );

    expect(screen.getByText("Update service is degraded")).toBeVisible();
    expect(
      screen.getByText(/fallback is available only where an established/i),
    ).toBeVisible();
    expect(
      screen.queryByText(/versions remain available/i),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText("Last known-good").nextElementSibling,
    ).toHaveTextContent("Not established");
  });

  it("keeps a failed component authoritative over a retained candidate", () => {
    const channel = {
      ...native().channels[0]!,
      state: "failed" as const,
      candidateVersion: "1.1.0",
      recoveryAction: "rebuild_runtime" as const,
      failureCode: "runtime_health_failed",
    };
    render(
      <UpdateSettings
        actions={actions()}
        snapshot={ready({ channels: [channel] })}
      />,
    );

    expect(screen.getByText("Failed")).toBeVisible();
    expect(screen.getByText("Activation failed")).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Stage 1.1.0" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Waiting to stage")).not.toBeInTheDocument();
  });

  it("offers candidate staging in degraded state only for retry_stage", () => {
    const handlers = actions();
    const channel = {
      ...native().channels[0]!,
      state: "failed" as const,
      candidateVersion: "1.1.0",
      recoveryAction: "retry_stage" as const,
      failureCode: "artifact_stage_failed",
    };
    render(
      <UpdateSettings
        actions={handlers}
        snapshot={ready({ channels: [channel] })}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Retry staging 1.1.0" }),
    );
    expect(handlers.onStage).toHaveBeenCalledWith({
      candidateId: "candidate:opencode-1-1",
      channel: "runtime",
      componentId: "opencode",
    });
    expect(screen.getByText("Failed")).toBeVisible();
  });

  it("renders an out-of-range native timestamp without throwing", () => {
    const channel = {
      ...native().channels[0]!,
      updatedAtMs: Number.MAX_SAFE_INTEGER,
    };

    render(
      <UpdateSettings
        actions={actions()}
        snapshot={ready({ channels: [channel] })}
      />,
    );

    const observed = screen.getByText("Unknown time");
    expect(observed).toHaveRole("time");
    expect(observed).not.toHaveAttribute("datetime");
  });

  it("routes crash-loop review through its exact runtime action", () => {
    const handlers = actions();
    const channel = {
      ...native().channels[0]!,
      componentId: "opencode-primary",
      state: "failed" as const,
      recoveryAction: "review_runtime_crash_loop" as const,
      failureCode: "runtime_crash_loop",
    };
    render(
      <UpdateSettings
        actions={handlers}
        snapshot={ready({
          channels: [channel],
          candidates: [],
          recoveryNotices: [
            {
              recoveryId: "recovery:runtime-crash-loop",
              correlationId: "correlation:runtime-crash-loop",
              channel: "runtime",
              componentId: "opencode-primary",
              summary: "Explicit review is required.",
              action: "review_runtime_crash_loop",
              createdAtMs: 1_784_476_800_000,
            },
          ],
        })}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Review runtime crash loop" }),
    );
    expect(handlers.onReviewRuntimeCrashLoop).toHaveBeenCalledWith({
      channel: "runtime",
      componentId: "opencode-primary",
    });
    expect(
      screen.queryByRole("button", { name: "Run recovery" }),
    ).not.toBeInTheDocument();
    expect(handlers.onRecover).not.toHaveBeenCalled();
  });

  it("focuses the complete operation status after failure", async () => {
    const handlers = actions();
    const initial = ready();
    const { rerender } = render(
      <UpdateSettings actions={handlers} snapshot={initial} />,
    );
    rerender(
      <UpdateSettings
        actions={handlers}
        snapshot={{
          ...initial,
          operation: {
            status: "error",
            componentKey: "runtime:opencode",
            message: "Activation conflicted with a newer generation.",
          },
        }}
      />,
    );

    const message = screen.getByText(/Activation conflicted/);
    await waitFor(() =>
      expect(message.parentElement?.parentElement).toHaveFocus(),
    );
  });
});
