import { act, fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ProviderSettingsSnapshot } from "../../platform/provider-service";
import { LaunchRoute, ProviderRouteGate } from "./LaunchRoute";

const providerService = vi.hoisted(() => ({
  readProviderSnapshot: vi.fn(),
}));

vi.mock("../../platform/provider-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/provider-service")
  >("../../platform/provider-service");
  return { ...actual, ...providerService };
});

describe("production provider route gate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
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

    expect(screen.getByRole("status")).toHaveTextContent(
      "Checking provider setup…",
    );
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
});

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
