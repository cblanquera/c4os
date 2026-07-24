import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BrowserArtifact } from "./BrowserArtifact";
import type { BrowserArtifactModel } from "./browser-types";

const MODEL: BrowserArtifactModel = {
  artifactId: "artifact:browser-example",
  canGoBack: true,
  canGoForward: false,
  controllerGeneration: 4,
  currentUrl: "https://example.com/docs",
  environmentScope: "per-project",
  mountGeneration: 2,
  notices: [],
  pageTitle: "Example documentation",
  pendingApproval: null,
  phase: "ready",
  recordRevision: 8,
  refreshing: false,
  title: "Example documentation",
};

function callbacks() {
  return {
    onAllowApproval: vi.fn(),
    onBack: vi.fn(),
    onClose: vi.fn(),
    onClearData: vi.fn(),
    onCopy: vi.fn(),
    onDenyApproval: vi.fn(),
    onExpand: vi.fn(),
    onForward: vi.fn(),
    onRefresh: vi.fn(),
    onReply: vi.fn(),
    onViewportFocusIntent: vi.fn(),
    onViewportLifecycle: vi.fn(),
  };
}

describe("BrowserArtifact", () => {
  it("renders controlled inline navigation and compact metadata without website DOM", () => {
    const actions = callbacks();
    render(<BrowserArtifact context="inline" model={MODEL} {...actions} />);

    const artifact = screen.getByRole("article", {
      name: "Browser response artifact: Example documentation",
    });
    expect(
      within(artifact).getByRole("textbox", { name: "Current web address" }),
    ).toHaveValue("https://example.com/docs");
    expect(
      within(artifact).getByRole("button", { name: "Back" }),
    ).toBeEnabled();
    expect(
      within(artifact).getByRole("button", { name: "Forward" }),
    ).toBeDisabled();
    expect(
      within(artifact).getByRole("region", { name: "Browser page metadata" }),
    ).toHaveAttribute("data-browser-preview", "compact");
    expect(
      within(artifact).queryByRole("group", {
        name: /Native Browser viewport/,
      }),
    ).toBeNull();

    fireEvent.click(within(artifact).getByRole("button", { name: "Back" }));
    fireEvent.click(within(artifact).getByRole("button", { name: "Refresh" }));
    fireEvent.click(
      within(artifact).getByRole("button", { name: "Clear browser data" }),
    );
    fireEvent.click(within(artifact).getByRole("button", { name: "Copy" }));
    expect(
      within(artifact).getByRole("button", { name: "Reply" }),
    ).toBeDisabled();
    fireEvent.click(within(artifact).getByRole("button", { name: "Expand" }));
    expect(actions.onBack).toHaveBeenCalledWith(MODEL.artifactId);
    expect(actions.onRefresh).toHaveBeenCalledWith(MODEL.artifactId);
    expect(actions.onClearData).toHaveBeenCalledWith(MODEL.artifactId);
    expect(actions.onCopy).toHaveBeenCalledWith(
      MODEL.artifactId,
      MODEL.currentUrl,
    );
    expect(actions.onReply).not.toHaveBeenCalled();
    expect(actions.onExpand).toHaveBeenCalledWith(MODEL.artifactId);
  });

  it("gives focused Browser one native viewport and controlled permission actions", () => {
    const actions = callbacks();
    render(
      <BrowserArtifact
        context="focused"
        model={{
          ...MODEL,
          pendingApproval: {
            approvalId: "approval:camera",
            message: "Allow this site to use the camera?",
            origin: "https://example.com",
            permission: "Camera access",
          },
        }}
        {...actions}
      />,
    );

    expect(
      screen.getByRole("group", {
        name: "Native Browser viewport: Example documentation",
      }),
    ).toBeVisible();
    expect(screen.queryByRole("button", { name: "Expand" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Allow" }));
    fireEvent.click(screen.getByRole("button", { name: "Deny" }));
    fireEvent.click(screen.getByRole("button", { name: "Reply" }));
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(actions.onAllowApproval).toHaveBeenCalledWith(
      MODEL.artifactId,
      "approval:camera",
    );
    expect(actions.onDenyApproval).toHaveBeenCalledWith(
      MODEL.artifactId,
      "approval:camera",
    );
    expect(actions.onReply).toHaveBeenCalledWith(MODEL.artifactId);
    expect(actions.onClose).toHaveBeenCalledWith(MODEL.artifactId);
  });

  it("keeps contextual Browser read-only and never mounts or duplicates controls", () => {
    const actions = callbacks();
    render(
      <BrowserArtifact
        context="contextual"
        model={{
          ...MODEL,
          pendingApproval: {
            approvalId: "approval:location",
            message: "Location access is awaiting a decision.",
            origin: "https://example.com",
            permission: "Location access",
          },
        }}
        {...actions}
      />,
    );

    expect(screen.queryByRole("button", { name: "Back" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Forward" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Refresh" })).toBeNull();
    expect(
      screen.queryByRole("button", { name: "Clear browser data" }),
    ).toBeNull();
    expect(screen.queryByRole("button", { name: "Allow" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Deny" })).toBeNull();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Location access is awaiting a decision.",
    );
    expect(
      screen.queryByRole("group", { name: /Native Browser viewport/ }),
    ).toBeNull();
    expect(
      screen.getByRole("region", { name: "Browser page metadata" }),
    ).toHaveAttribute("data-browser-preview", "contextual");
    expect(screen.getByRole("button", { name: "Expand" })).toBeVisible();
    expect(
      screen.getByRole("textbox", { name: "Current web address" }),
    ).toHaveAttribute("readonly");
  });

  it("announces bounded lifecycle states, refresh, and notices", () => {
    const { rerender } = render(
      <BrowserArtifact
        context="focused"
        model={{ ...MODEL, phase: "queued" }}
        {...callbacks()}
      />,
    );
    expect(screen.getByRole("article")).toHaveAttribute("aria-busy", "false");
    expect(screen.getByText("Preparing Browser")).toBeVisible();
    expect(
      screen.getByRole("group", {
        name: "Native Browser viewport: Example documentation",
      }),
    ).toBeVisible();

    rerender(
      <BrowserArtifact
        context="inline"
        model={{ ...MODEL, phase: "ready", refreshing: true }}
      />,
    );
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByText("Refreshing · Latest history entry")).toBeVisible();

    rerender(
      <BrowserArtifact
        context="inline"
        model={{
          ...MODEL,
          notices: [
            {
              id: "popup:blocked",
              kind: "warning",
              message: "A new window request was blocked.",
              title: "Popup blocked",
            },
          ],
        }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "A new window request was blocked.",
    );

    rerender(
      <BrowserArtifact context="inline" model={{ ...MODEL, phase: "error" }} />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "The web page could not be loaded.",
    );

    rerender(
      <BrowserArtifact
        context="inline"
        model={{ ...MODEL, phase: "recovery" }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "The Browser controller is ready to recover.",
    );
  });
});
