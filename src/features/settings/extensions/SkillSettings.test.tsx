import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SkillSettings } from "./SkillSettings";
import type {
  SkillSettingsActions,
  SkillSettingsSnapshot,
  SkillView,
} from "./types";

function createActions(): SkillSettingsActions {
  return {
    onCustomize: vi.fn(),
    onLoadInstructions: vi.fn(),
    onRetry: vi.fn(),
    onSelectExplicitly: vi.fn(),
    onSetEnabled: vi.fn(),
    onTryInChat: vi.fn(),
    onUninstall: vi.fn(),
  };
}

const skill: SkillView = {
  collisions: [
    {
      sourceLabel: "Bundled C4OS",
      sourcePrecedence: 10,
      sourceQualifiedId: "bundled:c4os/review",
    },
  ],
  effectiveState: "active",
  eligibilityDetail: "Eligible for Chat and the active Project.",
  enabled: true,
  frontmatter: { state: "valid" },
  id: "plugin:github-workflow/review",
  instructions: { state: "notLoaded" },
  isExplicitSelection: false,
  isInstalled: true,
  name: "Review",
  sourceKind: "plugin",
  sourceLabel: "GitHub Workflow",
  sourcePrecedence: 20,
  sourceQualifiedId: "plugin:github-workflow/review",
  summary: "Review a proposed change against the active Project.",
  version: "1.3.2",
};

function snapshot(
  skills: readonly SkillView[] = [skill],
): SkillSettingsSnapshot {
  return {
    discoveryDetail: "Metadata validated across 2 source-qualified Skills.",
    discoveryStatus: "current",
    generation: 7,
    skills,
    status: "ready",
  };
}

describe("SkillSettings", () => {
  it("keeps row availability controlled and exposes effective resolution", () => {
    const actions = createActions();
    render(<SkillSettings actions={actions} snapshot={snapshot()} />);

    expect(
      screen.getByText("plugin:github-workflow/review"),
    ).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();
    expect(screen.getByText("Name collision")).toBeInTheDocument();
    const toggle = screen.getByRole("switch", {
      name: /Review availability/u,
    });
    expect(toggle).toBeChecked();
    fireEvent.click(toggle);
    expect(actions.onSetEnabled).toHaveBeenCalledWith(
      "plugin:github-workflow/review",
      false,
    );
    expect(toggle).toBeChecked();
  });

  it("loads instructions progressively only after a details request", async () => {
    const actions = createActions();
    render(<SkillSettings actions={actions} snapshot={snapshot()} />);

    expect(document.body).not.toHaveTextContent("validated instructions");
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", { name: "Review" });
    expect(within(dialog).getByText(/metadata only/u)).toBeInTheDocument();
    expect(
      within(dialog).getByRole("button", { name: "Try in Chat" }),
    ).toBeDisabled();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Load instructions" }),
    );
    expect(actions.onLoadInstructions).toHaveBeenCalledWith(
      "plugin:github-workflow/review",
    );
    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Select this Skill explicitly",
      }),
    );
    expect(actions.onSelectExplicitly).toHaveBeenCalledWith(
      "plugin:github-workflow/review",
    );
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Customize as user copy" }),
    );
    expect(actions.onCustomize).toHaveBeenCalledWith(
      "plugin:github-workflow/review",
    );
  });

  it("renders loaded instructions as inert text", async () => {
    render(
      <SkillSettings
        actions={createActions()}
        snapshot={snapshot([
          {
            ...skill,
            instructions: {
              instructions: "Review carefully. <img src=x onerror=alert(1)>",
              state: "loaded",
            },
          },
        ])}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", { name: "Review" });
    expect(within(dialog).getByText(/<img src=x/u)).toBeInTheDocument();
    expect(dialog.querySelector("img")).toBeNull();
    expect(
      within(dialog).getByRole("button", { name: "Try in Chat" }),
    ).toBeEnabled();
  });

  it("surfaces invalid frontmatter and blocks activation controls", async () => {
    render(
      <SkillSettings
        actions={createActions()}
        snapshot={snapshot([
          {
            ...skill,
            effectiveState: "invalid",
            enabled: false,
            frontmatter: {
              diagnostic: "Unknown key `network_access` at line 4.",
              state: "invalid",
            },
          },
        ])}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Unknown key `network_access` at line 4.",
    );
    expect(
      screen.getByRole("switch", { name: /Review availability/u }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", { name: "Review" });
    expect(
      within(dialog).getByRole("switch", { name: /Available in Chat/u }),
    ).toBeDisabled();
  });

  it("renders discovery failure and empty state without stale rows", () => {
    const actions = createActions();
    const { rerender } = render(
      <SkillSettings
        actions={actions}
        snapshot={{
          message: "Skill metadata could not be validated.",
          retryable: false,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Skills are unavailable",
    );
    expect(screen.queryByRole("button", { name: "Try again" })).toBeNull();

    rerender(<SkillSettings actions={actions} snapshot={snapshot([])} />);
    expect(
      screen.getByRole("heading", { name: "No skills discovered" }),
    ).toBeInTheDocument();
  });
});
