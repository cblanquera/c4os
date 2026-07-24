import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { PlatformThemeQaSurface } from "../../../../src/frontend/features/platform/PlatformThemeQaSurface";
import { resetPlatformReviewStateForTests } from "../../../../src/frontend/features/platform/review-state";
import { APPEARANCE_CHANGE_EVENT } from "../../../../src/frontend/features/platform/theme";

describe("PlatformThemeQaSurface", () => {
  beforeEach(() => resetPlatformReviewStateForTests());

  it("exposes deterministic semantic tokens, component states, and platform facts", () => {
    render(
      <PlatformThemeQaSurface
        isEnabled
        appearance={{
          platform: "macos",
          colorScheme: "dark",
          source: "macosAppearance",
        }}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "System appearance, one source at a time",
      }),
    ).toBeVisible();
    const appearance = screen.getByLabelText("Resolved platform appearance");
    expect(appearance).toHaveTextContent("macOS");
    expect(appearance).toHaveTextContent("Dark");
    expect(appearance).toHaveTextContent("macOS appearance");
    expect(appearance).toHaveTextContent("Not stored");

    for (const token of [
      "--surface-window",
      "--surface-sidebar",
      "--surface-raised",
      "--surface-sunken",
      "--surface-overlay",
    ]) {
      expect(screen.getByText(token)).toBeVisible();
    }
    for (const state of [
      "Default",
      "Hover",
      "Pressed",
      "Selected",
      "Disabled",
      "Dirty",
      "Loading…",
    ]) {
      expect(screen.getByRole("button", { name: state })).toBeVisible();
    }
    expect(screen.getByRole("button", { name: "Disabled" })).toBeDisabled();
  });

  it("retains interaction state across an independently published theme change", () => {
    document.documentElement.dataset.platform = "macos";
    document.documentElement.dataset.colorScheme = "light";
    document.documentElement.dataset.appearanceSource = "macosAppearance";
    render(<PlatformThemeQaSurface isEnabled />);

    fireEvent.click(screen.getByRole("button", { name: "Dirty" }));
    expect(screen.getByText(/^Last reviewed state:/)).toHaveTextContent(
      "Last reviewed state: Dirty",
    );

    act(() => {
      document.documentElement.dataset.colorScheme = "dark";
      document.documentElement.dataset.appearanceSource =
        "webviewPrefersColorScheme";
      document.dispatchEvent(new Event(APPEARANCE_CHANGE_EVENT));
    });

    expect(
      screen.getByLabelText("Resolved platform appearance"),
    ).toHaveTextContent("Dark");
    expect(screen.getByText(/^Last reviewed state:/)).toHaveTextContent(
      "Last reviewed state: Dirty",
    );
  });

  it("retains interaction state across a same-process route remount", () => {
    const rendered = render(
      <PlatformThemeQaSurface
        isEnabled
        appearance={{
          platform: "macos",
          colorScheme: "dark",
          source: "macosAppearance",
        }}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Dirty" }));
    rendered.unmount();
    render(
      <PlatformThemeQaSurface
        isEnabled
        appearance={{
          platform: "macos",
          colorScheme: "dark",
          source: "macosAppearance",
        }}
      />,
    );

    expect(screen.getByText(/^Last reviewed state:/)).toHaveTextContent(
      "Last reviewed state: Dirty",
    );
  });

  it("uses an accessible modal primitive with dismissal and trigger focus restoration", async () => {
    render(
      <PlatformThemeQaSurface
        isEnabled
        appearance={{
          platform: "macos",
          colorScheme: "light",
          source: "webviewPreferredColorScheme",
        }}
      />,
    );

    const trigger = screen.getByRole("button", { name: "Review dialog" });
    trigger.focus();
    fireEvent.click(trigger);
    expect(
      await screen.findByRole("dialog", {
        name: "Keep unsaved appearance work?",
      }),
    ).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Keep changes" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("fails closed when the deterministic fixture gate is disabled", () => {
    render(<PlatformThemeQaSurface isEnabled={false} />);

    expect(
      screen.getByRole("heading", {
        name: "QA platform fixtures are disabled",
      }),
    ).toBeVisible();
    expect(
      screen.queryByRole("heading", {
        name: "System appearance, one source at a time",
      }),
    ).not.toBeInTheDocument();
  });
});
