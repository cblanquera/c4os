import { act, fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ProviderSettingsSnapshot } from "../../platform/provider-service";
import type { StartupRecoverySnapshot } from "../../platform/startup-recovery-service";
import { ProtocolBoundaryError } from "../../platform/tauri-adapter";
import { LaunchRoute, ProviderRouteGate } from "./LaunchRoute";

const providerService = vi.hoisted(() => ({
  readProviderSnapshot: vi.fn(),
}));
const startupRecoveryService = vi.hoisted(() => ({
  performStartupRecoveryAction: vi.fn(),
  readStartupRecoverySnapshot: vi.fn(),
}));
const workspaceStartService = vi.hoisted(() => ({
  readWorkspaceStartSnapshot: vi.fn(),
}));

vi.mock("../../platform/provider-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/provider-service")
  >("../../platform/provider-service");
  return { ...actual, ...providerService };
});

vi.mock("../../platform/startup-recovery-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/startup-recovery-service")
  >("../../platform/startup-recovery-service");
  return { ...actual, ...startupRecoveryService };
});

vi.mock("../../platform/workspace-start", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/workspace-start")
  >("../../platform/workspace-start");
  return { ...actual, ...workspaceStartService };
});

describe("production provider route gate", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  beforeEach(() => {
    providerService.readProviderSnapshot.mockReset();
    startupRecoveryService.performStartupRecoveryAction.mockReset();
    startupRecoveryService.readStartupRecoverySnapshot.mockReset();
    workspaceStartService.readWorkspaceStartSnapshot.mockReset();
    startupRecoveryService.readStartupRecoverySnapshot.mockResolvedValue(
      startupRecoverySnapshot(),
    );
    workspaceStartService.readWorkspaceStartSnapshot.mockResolvedValue(
      workspaceStartSnapshot(),
    );
  });

  it("lets LaunchRoute enter Workspace Start after durable provider setup", async () => {
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    const router = createMemoryRouter(
      [
        { path: "/", element: <LaunchRoute /> },
        { path: "/start", element: <p>Workspace Start route</p> },
      ],
      { initialEntries: ["/"] },
    );

    render(<RouterProvider router={router} />);

    expect(await screen.findByText("Workspace Start route")).toBeVisible();
    expect(providerService.readProviderSnapshot).toHaveBeenCalledTimes(1);
  });

  it("holds a protected route behind the gate and redirects missing setup", async () => {
    let resolveSnapshot!: (snapshot: ProviderSettingsSnapshot) => void;
    providerService.readProviderSnapshot.mockReturnValueOnce(
      new Promise<ProviderSettingsSnapshot>((resolve) => {
        resolveSnapshot = resolve;
      }),
    );
    const router = createMemoryRouter(
      [
        {
          path: "/settings/models",
          element: (
            <ProviderRouteGate routePath="/settings/models">
              <p>Protected model settings</p>
            </ProviderRouteGate>
          ),
        },
        { path: "/onboarding", element: <p>Provider onboarding route</p> },
      ],
      { initialEntries: ["/settings/models"] },
    );

    render(<RouterProvider router={router} />);

    expect(await screen.findByText("Checking provider setup…")).toBeVisible();
    expect(
      screen.queryByText("Protected model settings"),
    ).not.toBeInTheDocument();

    await act(async () => {
      resolveSnapshot(providerSnapshot({ onboardingCompleted: false }));
    });

    expect(await screen.findByText("Provider onboarding route")).toBeVisible();
    expect(
      screen.queryByText("Protected model settings"),
    ).not.toBeInTheDocument();
  });

  it("re-reads durable provider setup when onboarding navigates to Start", async () => {
    providerService.readProviderSnapshot
      .mockResolvedValueOnce(providerSnapshot({ onboardingCompleted: false }))
      .mockResolvedValueOnce(providerSnapshot({ onboardingCompleted: true }));
    const router = createMemoryRouter(
      [
        {
          path: "/onboarding",
          element: (
            <ProviderRouteGate routePath="/onboarding">
              <button onClick={() => void router.navigate("/start")}>
                Complete onboarding
              </button>
            </ProviderRouteGate>
          ),
        },
        {
          path: "/start",
          element: (
            <ProviderRouteGate routePath="/start">
              <p>Workspace Start after onboarding</p>
            </ProviderRouteGate>
          ),
        },
      ],
      { initialEntries: ["/onboarding"] },
    );

    render(<RouterProvider router={router} />);

    fireEvent.click(
      await screen.findByRole("button", { name: "Complete onboarding" }),
    );

    expect(
      await screen.findByText("Workspace Start after onboarding"),
    ).toBeVisible();
    expect(providerService.readProviderSnapshot).toHaveBeenCalledTimes(2);
  });

  it("keeps the route blocked after a read failure and recovers on retry", async () => {
    providerService.readProviderSnapshot
      .mockRejectedValueOnce(new Error("provider authority unavailable"))
      .mockResolvedValueOnce(providerSnapshot());
    const router = createMemoryRouter(
      [
        {
          path: "/settings/models",
          element: (
            <ProviderRouteGate routePath="/settings/models">
              <p>Protected model settings</p>
            </ProviderRouteGate>
          ),
        },
      ],
      { initialEntries: ["/settings/models"] },
    );

    render(<RouterProvider router={router} />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "C4OS could not read provider setup",
    );
    expect(
      screen.queryByText("Protected model settings"),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(await screen.findByText("Protected model settings")).toBeVisible();
    expect(providerService.readProviderSnapshot).toHaveBeenCalledTimes(2);
  });

  it("blocks every provider route before provider reads while startup is degraded", async () => {
    startupRecoveryService.readStartupRecoverySnapshot.mockResolvedValueOnce(
      startupRecoverySnapshot({
        generation: 8,
        lifecycle: "degraded",
        normalWorkAuthorized: false,
        failure: {
          boundary: "database",
          correlationId: "correlation:database-startup",
          diagnosticCode: "database_startup_failed",
          message: "The database did not pass startup validation.",
          failedAtMs: 1_784_476_800_000,
          validatedBackupAvailable: true,
          recoveryLocationAvailable: true,
        },
        availableActions: [
          "retry",
          "restoreValidatedBackup",
          "openRecoveryLocation",
        ],
        history: [],
      }),
    );
    startupRecoveryService.performStartupRecoveryAction.mockResolvedValueOnce(
      startupRecoverySnapshot({
        generation: 9,
        lifecycle: "degraded",
        normalWorkAuthorized: false,
        failure: {
          boundary: "database",
          correlationId: "correlation:database-startup",
          diagnosticCode: "database_startup_failed",
          message: "The recovery location opened; repair is still required.",
          failedAtMs: 1_784_476_800_000,
          validatedBackupAvailable: true,
          recoveryLocationAvailable: true,
        },
        availableActions: [
          "retry",
          "restoreValidatedBackup",
          "openRecoveryLocation",
        ],
        history: [],
      }),
    );
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );

    expect(await screen.findByText("Startup recovery required")).toBeVisible();
    expect(providerService.readProviderSnapshot).not.toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: "Restore validated backup" }),
    ).toBeEnabled();

    fireEvent.click(
      screen.getByRole("button", { name: "Open recovery location" }),
    );
    expect(
      await screen.findByText(/native recovery location was opened/i),
    ).toBeVisible();
    expect(
      startupRecoveryService.performStartupRecoveryAction,
    ).toHaveBeenCalledWith("openRecoveryLocation");
    expect(providerService.readProviderSnapshot).not.toHaveBeenCalled();
    expect(
      screen.queryByText("Protected model settings"),
    ).not.toBeInTheDocument();
  });

  it("uses Finder wording for recovery-location actions on macOS", async () => {
    const userAgent = vi
      .spyOn(window.navigator, "userAgent", "get")
      .mockReturnValue(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15",
      );
    startupRecoveryService.readStartupRecoverySnapshot.mockResolvedValueOnce(
      startupRecoverySnapshot({
        generation: 8,
        lifecycle: "degraded",
        normalWorkAuthorized: false,
        failure: {
          boundary: "database",
          correlationId: "correlation:database-startup",
          diagnosticCode: "database_startup_failed",
          message: "The database did not pass startup validation.",
          failedAtMs: 1_784_476_800_000,
          validatedBackupAvailable: false,
          recoveryLocationAvailable: true,
        },
        availableActions: ["openRecoveryLocation"],
      }),
    );
    startupRecoveryService.performStartupRecoveryAction.mockResolvedValueOnce(
      startupRecoverySnapshot({
        generation: 9,
        lifecycle: "degraded",
        normalWorkAuthorized: false,
        failure: {
          boundary: "database",
          correlationId: "correlation:database-startup",
          diagnosticCode: "database_startup_failed",
          message: "The recovery location was revealed for review.",
          failedAtMs: 1_784_476_800_000,
          validatedBackupAvailable: false,
          recoveryLocationAvailable: true,
        },
        availableActions: ["openRecoveryLocation"],
      }),
    );

    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Reveal in Finder" }),
    );
    expect(
      await screen.findByText(
        /native recovery location was revealed in Finder/i,
      ),
    ).toBeVisible();
    expect(
      startupRecoveryService.performStartupRecoveryAction,
    ).toHaveBeenCalledWith("openRecoveryLocation");
    userAgent.mockRestore();
  });

  it("reads provider authority only after native startup recovery succeeds", async () => {
    startupRecoveryService.readStartupRecoverySnapshot.mockResolvedValueOnce(
      startupRecoverySnapshot({
        generation: 8,
        lifecycle: "degraded",
        normalWorkAuthorized: false,
        failure: {
          boundary: "runtime",
          correlationId: "correlation:runtime-startup",
          diagnosticCode: "runtime_startup_failed",
          message: "Runtime health checks failed.",
          failedAtMs: 1_784_476_800_000,
          validatedBackupAvailable: false,
          recoveryLocationAvailable: false,
        },
        availableActions: ["retry"],
        history: [],
      }),
    );
    startupRecoveryService.performStartupRecoveryAction.mockResolvedValueOnce(
      startupRecoverySnapshot({ generation: 9, lifecycle: "recovered" }),
    );
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());

    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Retry startup checks" }),
    );

    expect(await screen.findByText("Protected model settings")).toBeVisible();
    expect(
      startupRecoveryService.performStartupRecoveryAction,
    ).toHaveBeenCalledWith("retry");
    expect(providerService.readProviderSnapshot).toHaveBeenCalledTimes(1);
  });

  it("refreshes authoritative startup actions after a stale recovery command", async () => {
    startupRecoveryService.readStartupRecoverySnapshot
      .mockResolvedValueOnce(
        startupRecoverySnapshot({
          generation: 8,
          lifecycle: "degraded",
          normalWorkAuthorized: false,
          failure: {
            boundary: "runtime",
            correlationId: "correlation:runtime-startup",
            diagnosticCode: "runtime_startup_failed",
            message: "Runtime health checks failed.",
            failedAtMs: 1_784_476_800_000,
            validatedBackupAvailable: false,
            recoveryLocationAvailable: false,
          },
          availableActions: ["retry"],
        }),
      )
      .mockResolvedValueOnce(
        startupRecoverySnapshot({
          generation: 9,
          lifecycle: "degraded",
          normalWorkAuthorized: false,
          failure: {
            boundary: "runtime",
            correlationId: "correlation:runtime-startup-new",
            diagnosticCode: "runtime_startup_failed",
            message: "A newer failure requires native location review.",
            failedAtMs: 1_784_476_800_100,
            validatedBackupAvailable: false,
            recoveryLocationAvailable: true,
          },
          availableActions: ["openRecoveryLocation"],
        }),
      );
    startupRecoveryService.performStartupRecoveryAction.mockRejectedValueOnce(
      new ProtocolBoundaryError(
        "staleGeneration",
        "Startup recovery generation changed.",
        true,
      ),
    );
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Retry startup checks" }),
    );

    expect(
      await screen.findByText(/Startup recovery state changed/u),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Retry startup checks" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Open recovery location" }),
    ).toBeEnabled();
    expect(
      startupRecoveryService.readStartupRecoverySnapshot,
    ).toHaveBeenCalledTimes(2);
    expect(providerService.readProviderSnapshot).not.toHaveBeenCalled();
  });

  it("keeps first load blocked until retry obtains startup authority", async () => {
    startupRecoveryService.readStartupRecoverySnapshot
      .mockRejectedValueOnce(
        new ProtocolBoundaryError(
          "unavailable",
          "Startup recovery is temporarily unavailable.",
          true,
        ),
      )
      .mockResolvedValueOnce(startupRecoverySnapshot());
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Startup recovery is temporarily unavailable.",
    );
    expect(providerService.readProviderSnapshot).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(await screen.findByText("Protected model settings")).toBeVisible();
    expect(
      startupRecoveryService.readStartupRecoverySnapshot,
    ).toHaveBeenCalledTimes(2);
    expect(providerService.readProviderSnapshot).toHaveBeenCalledTimes(1);
  });

  it("polls retrying startup authority until normal work is released", async () => {
    startupRecoveryService.readStartupRecoverySnapshot
      .mockResolvedValueOnce(
        startupRecoverySnapshot({
          generation: 8,
          lifecycle: "retrying",
          normalWorkAuthorized: false,
          failure: {
            boundary: "database",
            correlationId: "correlation:database-retry",
            diagnosticCode: "database_retrying",
            message: "Native database recovery is still running.",
            failedAtMs: 1_784_476_800_000,
            validatedBackupAvailable: true,
            recoveryLocationAvailable: true,
          },
          availableActions: [],
          activeAction: {
            boundary: "database",
            action: "retry",
            startedAtMs: 1_784_476_800_100,
          },
        }),
      )
      .mockResolvedValueOnce(
        startupRecoverySnapshot({
          generation: 9,
          lifecycle: "recovered",
          normalWorkAuthorized: true,
        }),
      );
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );

    expect(await screen.findByText("Recovery in progress")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Refresh recovery status" }),
    ).toBeDisabled();
    expect(providerService.readProviderSnapshot).not.toHaveBeenCalled();

    expect(
      await screen.findByText(
        "Protected model settings",
        {},
        { timeout: 2_500 },
      ),
    ).toBeVisible();
    expect(
      startupRecoveryService.readStartupRecoverySnapshot,
    ).toHaveBeenCalledTimes(2);
    expect(providerService.readProviderSnapshot).toHaveBeenCalledTimes(1);
  });

  it("does not let a deferred recovery refresh overwrite a newer action", async () => {
    const lateRefresh = deferred<StartupRecoverySnapshot>();
    const degraded = startupRecoverySnapshot({
      generation: 8,
      lifecycle: "degraded",
      normalWorkAuthorized: false,
      failure: {
        boundary: "runtime",
        correlationId: "correlation:runtime-startup",
        diagnosticCode: "runtime_startup_failed",
        message: "Runtime health checks failed.",
        failedAtMs: 1_784_476_800_000,
        validatedBackupAvailable: false,
        recoveryLocationAvailable: false,
      },
      availableActions: ["retry"],
    });
    startupRecoveryService.readStartupRecoverySnapshot
      .mockResolvedValueOnce(degraded)
      .mockReturnValueOnce(lateRefresh.promise);
    startupRecoveryService.performStartupRecoveryAction.mockResolvedValueOnce(
      startupRecoverySnapshot({
        generation: 10,
        lifecycle: "recovered",
        normalWorkAuthorized: true,
      }),
    );
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    render(
      <ProviderRouteGate routePath="/settings/models">
        <p>Protected model settings</p>
      </ProviderRouteGate>,
    );

    await screen.findByRole("button", { name: "Retry startup checks" });
    fireEvent.click(
      screen.getByRole("button", { name: "Refresh recovery status" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Retry startup checks" }),
    );
    expect(await screen.findByText("Protected model settings")).toBeVisible();

    await act(async () => {
      lateRefresh.resolve(
        startupRecoverySnapshot({
          ...degraded,
          generation: 9,
        }),
      );
      await lateRefresh.promise;
    });

    expect(screen.getByText("Protected model settings")).toBeVisible();
    expect(
      startupRecoveryService.performStartupRecoveryAction,
    ).toHaveBeenCalledTimes(1);
  });

  it("redirects a direct Chat route to Start while Workspace recovery review is pending", async () => {
    workspaceStartService.readWorkspaceStartSnapshot.mockResolvedValueOnce({
      ...workspaceStartSnapshot(),
      activeRecoveryNotice: {
        recoveryId: "recovery:workspace-active:9:8",
        correlationId: "correlation:workspace-active-recovery",
        workspaceId: "workspace:active",
        workspaceName: "Active Workspace",
        summary: "Review recovered generation 9 before the next save.",
        action: "review_recovered_workspace_before_save",
        workingGeneration: 9,
        archiveGeneration: 8,
        mustNotifyBeforeNextSave: true,
      },
    });
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    const router = createMemoryRouter(
      [
        {
          path: "/chat",
          element: (
            <ProviderRouteGate routePath="/chat">
              <p>Protected Chat</p>
            </ProviderRouteGate>
          ),
        },
        { path: "/start", element: <p>Workspace recovery review</p> },
      ],
      { initialEntries: ["/chat"] },
    );

    render(<RouterProvider router={router} />);

    expect(await screen.findByText("Workspace recovery review")).toBeVisible();
    expect(screen.queryByText("Protected Chat")).not.toBeInTheDocument();
    expect(
      workspaceStartService.readWorkspaceStartSnapshot,
    ).toHaveBeenCalledTimes(1);
  });

  it("allows a reviewed recovery in this process but shows a new notice on the next gate instance", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-active:10:9",
      correlationId: "correlation:workspace-active-recovery-next",
      workspaceId: "workspace:active",
      workspaceName: "Active Workspace",
      summary: "Review the new recovered generation before the next save.",
      action: "review_recovered_workspace_before_save",
      workingGeneration: 10,
      archiveGeneration: 9,
      mustNotifyBeforeNextSave: true,
    };
    workspaceStartService.readWorkspaceStartSnapshot
      .mockResolvedValueOnce(workspaceStartSnapshot())
      .mockResolvedValueOnce({
        ...workspaceStartSnapshot(),
        generation: 3,
        activeRecoveryNotice: recoveryNotice,
      });
    providerService.readProviderSnapshot.mockResolvedValue(providerSnapshot());
    const firstRouter = createMemoryRouter(
      [
        {
          path: "/chat",
          element: (
            <ProviderRouteGate routePath="/chat">
              <p>Protected Chat</p>
            </ProviderRouteGate>
          ),
        },
      ],
      { initialEntries: ["/chat"] },
    );

    const first = render(<RouterProvider router={firstRouter} />);
    expect(await screen.findByText("Protected Chat")).toBeVisible();
    first.unmount();

    const secondRouter = createMemoryRouter(
      [
        {
          path: "/chat",
          element: (
            <ProviderRouteGate routePath="/chat">
              <p>Protected Chat after restart</p>
            </ProviderRouteGate>
          ),
        },
        { path: "/start", element: <p>New Workspace recovery review</p> },
      ],
      { initialEntries: ["/chat"] },
    );
    render(<RouterProvider router={secondRouter} />);

    expect(
      await screen.findByText("New Workspace recovery review"),
    ).toBeVisible();
    expect(
      screen.queryByText("Protected Chat after restart"),
    ).not.toBeInTheDocument();
    expect(
      workspaceStartService.readWorkspaceStartSnapshot,
    ).toHaveBeenCalledTimes(2);
  });
});

function startupRecoverySnapshot(
  overrides: Partial<StartupRecoverySnapshot> = {},
): StartupRecoverySnapshot {
  return {
    schemaVersion: 1,
    authority: "rust-startup-recovery",
    generation: 1,
    lifecycle: "healthy",
    normalWorkAuthorized: true,
    failure: null,
    availableActions: [],
    activeAction: null,
    history: [],
    historyTruncated: 0,
    ...overrides,
  };
}

function workspaceStartSnapshot() {
  return {
    protocolVersion: 1,
    authority: "rust-core",
    generation: 2,
    recents: [],
    activeRecoveryNotice: null,
  };
}

function providerSnapshot(
  overrides: Partial<ProviderSettingsSnapshot> = {},
): ProviderSettingsSnapshot {
  return {
    authority: "rust-provider-service",
    coordinatorGeneration: 4,
    configurationGeneration: 3,
    credentialProtection: "installation-key",
    credentialFallbackRequired: false,
    generation: 2,
    onboardingCompleted: true,
    providers: [],
    modelRoute: "provider:openai-work::gpt-5",
    defaultRuntime: "opencode",
    defaultEnvironment: "local",
    pendingApproval: null,
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
