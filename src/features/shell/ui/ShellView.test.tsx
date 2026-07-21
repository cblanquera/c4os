import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import type {
  ShellFocusRestoreRequest,
  ShellViewProps,
} from "./shell-view.types";
import {
  SETTINGS_DESTINATIONS,
  SHELL_ROUTE_PATHS,
  type ShellRoutePath,
} from "./shell-routes";
import { ShellView } from "./ShellView";

function createProps(overrides: Partial<ShellViewProps> = {}): ShellViewProps {
  return {
    route: "/chat",
    composer: { draft: "", mode: "chat" },
    projectPanel: {
      mode: "docked",
      isOpen: true,
      width: 228,
      minimumWidth: 180,
      maximumWidth: 520,
    },
    onNavigate: vi.fn(),
    onVisitSettings: vi.fn(),
    onBackFromSettings: vi.fn(),
    onPanelWidthChange: vi.fn(),
    onPanelOpenChange: vi.fn(),
    onPanelOverlayDismiss: vi.fn(),
    onComposerDraftChange: vi.fn(),
    onComposerModeChange: vi.fn(),
    onFocusRestored: vi.fn(),
    showReviewSettingsControl: true,
    ...overrides,
  };
}

function renderShell(
  overrides: Partial<ShellViewProps> = {},
  routeContent?: ReactNode,
) {
  return render(
    <ShellView
      {...createProps(overrides)}
      {...(routeContent === undefined ? {} : { routeContent })}
    />,
  );
}

describe("ShellView", () => {
  it("renders the structural workspace with one main, a left panel, and a fixed composer", () => {
    const { container } = renderShell();

    expect(screen.getByRole("banner")).toHaveAccessibleName(
      "C4OS window title",
    );
    expect(screen.getByRole("complementary")).toHaveAccessibleName("Projects");
    expect(screen.getByRole("main")).toContainElement(
      screen.getByRole("heading", { name: "Chat", level: 1 }),
    );
    expect(
      screen.getByRole("form", { name: "Message composer" }),
    ).toBeVisible();
    expect(container.querySelectorAll("main")).toHaveLength(1);
    expect(
      container.querySelector('[data-shell-region="right-panel"]'),
    ).toBeNull();
  });

  it("exposes controlled Settings, panel, composer, and resize callbacks", () => {
    const onVisitSettings = vi.fn();
    const onPanelOpenChange = vi.fn();
    const onPanelWidthChange = vi.fn();
    const onComposerDraftChange = vi.fn();
    const onComposerModeChange = vi.fn();
    renderShell({
      onVisitSettings,
      onPanelOpenChange,
      onPanelWidthChange,
      onComposerDraftChange,
      onComposerModeChange,
    });

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(onVisitSettings).toHaveBeenCalledWith("workspace-settings");

    fireEvent.click(
      screen.getByRole("button", { name: "Collapse project panel" }),
    );
    expect(onPanelOpenChange).toHaveBeenCalledWith(false);

    fireEvent.keyDown(
      screen.getByRole("separator", { name: "Resize project panel" }),
      { key: "ArrowRight" },
    );
    expect(onPanelWidthChange).toHaveBeenCalledWith(240);

    fireEvent.change(screen.getByRole("textbox", { name: "Message" }), {
      target: { value: "Keep this draft" },
    });
    expect(onComposerDraftChange).toHaveBeenCalledWith("Keep this draft");

    fireEvent.change(screen.getByRole("combobox", { name: "Composer mode" }), {
      target: { value: "files" },
    });
    expect(onComposerModeChange).toHaveBeenCalledWith("files");
  });

  it("keeps the review-only Settings control out of the production shell", () => {
    renderShell({ showReviewSettingsControl: false });

    expect(screen.queryByRole("button", { name: "Settings" })).toBeNull();
  });

  it("dismisses an overlay without consuming the intended center action", () => {
    const onPanelOverlayDismiss = vi.fn();
    const centerAction = vi.fn();
    renderShell(
      {
        projectPanel: {
          mode: "overlay",
          isOpen: true,
          width: 228,
        },
        onPanelOverlayDismiss,
      },
      <button type="button" onClick={centerAction}>
        Center action
      </button>,
    );

    fireEvent.pointerDown(
      screen.getByRole("button", { name: "Center action" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Center action" }));
    expect(onPanelOverlayDismiss).toHaveBeenCalledTimes(1);
    expect(centerAction).toHaveBeenCalledTimes(1);
  });

  it("renders every accepted direct route with its own accessible title", () => {
    const { rerender } = renderShell();

    for (const route of SHELL_ROUTE_PATHS) {
      const props = createProps({ route });
      rerender(<ShellView {...props} />);
      const title = routeTitle(route);
      expect(
        screen.getByRole("heading", { name: title, level: 1 }),
      ).toBeVisible();
      expect(
        document.querySelector(`[data-route="${route}"]`),
      ).toBeInTheDocument();
    }
  });

  it("keeps Settings navigation in accepted order with compressed labels", () => {
    const onNavigate = vi.fn();
    renderShell({ route: "/settings/advanced-policies", onNavigate });

    const navigation = screen.getByRole("navigation", { name: "Settings" });
    const destinationButtons = SETTINGS_DESTINATIONS.map((destination) =>
      screen.getByRole("button", { name: destination.label }),
    );
    expect(destinationButtons.map((button) => button.textContent)).toEqual(
      SETTINGS_DESTINATIONS.map(
        (destination) => `${destination.symbol}${destination.label}`,
      ),
    );
    for (const [index, destination] of SETTINGS_DESTINATIONS.entries()) {
      const button = destinationButtons[index];
      expect(navigation).toContainElement(button ?? null);
      expect(button).toHaveAttribute(
        "data-compressed-label",
        destination.symbol,
      );
    }
    expect(
      screen.getByRole("button", { name: "Configuration" }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      document.querySelector('[data-shell-layout="policy"]'),
    ).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Models" }));
    expect(onNavigate).toHaveBeenCalledWith("/settings/models");
  });

  it("requests Settings Back and restores focus after the workspace remounts", () => {
    const onBackFromSettings = vi.fn();
    const onFocusRestored = vi.fn();
    const focusRequest: ShellFocusRestoreRequest = {
      requestId: 7,
      target: "workspace-settings",
    };
    const { rerender } = renderShell({
      route: "/settings/providers",
      onBackFromSettings,
      onFocusRestored,
    });

    fireEvent.click(screen.getByRole("button", { name: "Back to C4OS" }));
    expect(onBackFromSettings).toHaveBeenCalledTimes(1);

    rerender(
      <ShellView
        {...createProps({
          route: "/chat",
          focusRestoreRequest: focusRequest,
          onFocusRestored,
        })}
      />,
    );
    expect(screen.getByRole("button", { name: "Settings" })).toHaveFocus();
    expect(onFocusRestored).toHaveBeenCalledWith(focusRequest);
  });

  it("renders the accepted prohibited behaviors as disabled visible gates", () => {
    const { rerender } = renderShell({ showContextualChat: true });

    expect(screen.getByRole("button", { name: "Detach Chat" })).toBeDisabled();
    expect(
      screen.getByText("Not available: Detached native windows."),
    ).toBeVisible();

    rerender(<ShellView {...createProps({ route: "/browser" })} />);
    expect(screen.getByRole("button", { name: "New tab" })).toBeDisabled();

    rerender(<ShellView {...createProps({ route: "/terminal" })} />);
    expect(
      screen.getByRole("button", { name: "Enter full screen" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Enter password" }),
    ).toBeDisabled();
  });

  it("reserves contextual Chat for explicit artifact focus", () => {
    const { rerender } = renderShell({ showContextualChat: false });

    expect(screen.getByRole("complementary")).toHaveAccessibleName("Projects");
    expect(
      screen.queryByRole("heading", { name: "Chat", level: 2 }),
    ).toBeNull();
    expect(screen.queryByRole("button", { name: "Detach Chat" })).toBeNull();

    rerender(<ShellView {...createProps({ showContextualChat: true })} />);
    expect(screen.getByRole("complementary")).toHaveAccessibleName(
      "Projects and contextual chat",
    );
    expect(screen.getByRole("button", { name: "Detach Chat" })).toBeDisabled();
  });
});

/** Mirrors the accepted direct-route copy for a route-matrix assertion. */
function routeTitle(route: ShellRoutePath): string {
  const titles: Record<ShellRoutePath, string> = {
    "/onboarding": "Connect your AI provider",
    "/start": "Workspace Start",
    "/chat": "Chat",
    "/chat-search": "Search Chat Sessions",
    "/chat-capabilities": "Capability-aware Chat",
    "/files": "Files",
    "/browser": "Browser",
    "/terminal": "Terminal",
    "/settings/providers": "Providers",
    "/settings/models": "Models",
    "/settings/runtimes": "Runtimes",
    "/settings/configuration": "Configuration",
    "/settings/plugins": "Plugins",
    "/settings/skills": "Skills",
    "/settings/mcp": "MCP Servers",
    "/settings/advanced-policies": "Advanced Policies",
  };
  return titles[route];
}
