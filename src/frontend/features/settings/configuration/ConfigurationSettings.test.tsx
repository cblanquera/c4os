import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConfigurationSettings } from "./ConfigurationSettings";
import type {
  ConfigurationSettingsActions,
  ConfigurationSettingsSnapshot,
  ConfigurationValues,
} from "./types";

function actions(): ConfigurationSettingsActions {
  return {
    onNavigateAdvanced: vi.fn().mockResolvedValue(undefined),
    onOpenConfigurationFile: vi.fn().mockResolvedValue(undefined),
    onRetry: vi.fn().mockResolvedValue(undefined),
    onSaveConfiguration: vi.fn().mockResolvedValue(undefined),
  };
}

const saved: ConfigurationValues = {
  approvalPreset: "approveSafeActions",
  browserEnvironment: "none",
  inheritShellEnvironment: false,
  restoreLastWorkspace: false,
};

function ready(
  overrides: Partial<
    Extract<ConfigurationSettingsSnapshot, { status: "ready" }>
  > = {},
): Extract<ConfigurationSettingsSnapshot, { status: "ready" }> {
  return {
    generation: 17,
    live: {
      detail: "Last-known-good configuration is active.",
      displayPath: "~/.c4os/config.toml",
      source: "lastKnownGood",
    },
    openConfiguration: {
      message: "Open the file to edit advanced values.",
      status: "idle",
    },
    save: { message: "Configuration is live.", status: "idle" },
    saved,
    status: "ready",
    ...overrides,
  };
}

describe("ConfigurationSettings", () => {
  it("keeps loading, error, and retry states distinct", () => {
    const handlers = actions();
    const { rerender } = render(
      <ConfigurationSettings
        actions={handlers}
        snapshot={{ message: "Reading the active file.", status: "loading" }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Loading configuration",
    );

    rerender(
      <ConfigurationSettings
        actions={handlers}
        snapshot={{
          message: "The last-known-good snapshot could not be read.",
          retryable: true,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Configuration is unavailable",
    );
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(handlers.onRetry).toHaveBeenCalledTimes(1);
  });

  it("shows all approval presets, guardrails, and the Advanced route action", () => {
    const handlers = actions();
    render(<ConfigurationSettings actions={handlers} snapshot={ready()} />);

    const policy = screen.getByRole("combobox", {
      name: "Default Approval Policy",
    });
    expect(policy).toHaveValue("approveSafeActions");
    expect(
      Array.from(policy.querySelectorAll("option")).map(
        (item) => item.textContent,
      ),
    ).toEqual([
      "Ask for approval",
      "Approve safe actions",
      "Approve for me",
      "Custom",
    ]);
    expect(
      screen.getByText(/active sandbox, trusted roots/u),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Advanced" }));
    expect(handlers.onNavigateAdvanced).toHaveBeenCalledTimes(1);
    expect(
      screen.queryByRole("combobox", { name: "Default runtime" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("combobox", { name: "Default environment" }),
    ).not.toBeInTheDocument();
  });

  it("keeps every editable default in one local draft and submits its generation", () => {
    const handlers = actions();
    render(<ConfigurationSettings actions={handlers} snapshot={ready()} />);

    const saveButton = screen.getByRole("button", {
      name: "Save Configuration",
    });
    expect(saveButton).toBeDisabled();

    fireEvent.change(
      screen.getByRole("combobox", { name: "Default Approval Policy" }),
      { target: { value: "approveForMe" } },
    );
    fireEvent.click(
      screen.getByRole("switch", { name: /^Restore last workspace/u }),
    );
    fireEvent.click(
      screen.getByRole("switch", { name: /^Shell environment/u }),
    );
    fireEvent.click(screen.getByRole("radio", { name: /Per project/u }));

    expect(saveButton).toBeEnabled();
    expect(
      screen.getByText("Configuration has unsaved changes."),
    ).toBeInTheDocument();
    fireEvent.click(saveButton);
    expect(handlers.onSaveConfiguration).toHaveBeenCalledWith(
      {
        approvalPreset: "approveForMe",
        browserEnvironment: "workspaceProject",
        inheritShellEnvironment: true,
        restoreLastWorkspace: true,
      },
      17,
    );
  });

  it("reverts the draft and exposes every Browser Environment scope", () => {
    const handlers = actions();
    render(<ConfigurationSettings actions={handlers} snapshot={ready()} />);

    expect(screen.getAllByRole("radio")).toHaveLength(4);
    expect(
      screen.getByRole("radio", { name: /All browsers/u }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("radio", { name: /Per project/u }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("radio", { name: /Per chat session/u }),
    ).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /^None/u })).toBeChecked();

    fireEvent.click(screen.getByRole("radio", { name: /All browsers/u }));
    expect(
      screen.getByText("Configuration has unsaved changes."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Revert" }));
    expect(screen.getByRole("radio", { name: /^None/u })).toBeChecked();
    expect(
      screen.getByRole("button", { name: "Save Configuration" }),
    ).toBeDisabled();
  });

  it("retains an unsaved draft when a conflict refresh publishes newer saved values", () => {
    const handlers = actions();
    const { rerender } = render(
      <ConfigurationSettings actions={handlers} snapshot={ready()} />,
    );

    fireEvent.click(screen.getByRole("radio", { name: /Per project/u }));
    rerender(
      <ConfigurationSettings
        actions={handlers}
        snapshot={ready({
          generation: 18,
          save: {
            message:
              "Configuration changed in browser_environment. Review the retained draft before retrying Save.",
            status: "error",
          },
          saved: {
            ...saved,
            restoreLastWorkspace: true,
          },
        })}
      />,
    );

    expect(screen.getByRole("radio", { name: /Per project/u })).toBeChecked();
    expect(
      screen.getByRole("switch", { name: /^Restore last workspace/u }),
    ).not.toBeChecked();
    expect(
      screen.getByRole("button", { name: "Save Configuration" }),
    ).toBeEnabled();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Configuration changed in browser_environment",
    );
  });

  it("renders live, pending, failure, and external-open state from the service", () => {
    const handlers = actions();
    const { rerender } = render(
      <ConfigurationSettings actions={handlers} snapshot={ready()} />,
    );
    expect(screen.getByText("Live")).toBeInTheDocument();
    expect(screen.getByText("~/.c4os/config.toml")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Open C4OS config" }));
    expect(handlers.onOpenConfigurationFile).toHaveBeenCalledTimes(1);

    rerender(
      <ConfigurationSettings
        actions={handlers}
        snapshot={ready({
          openConfiguration: {
            message: "Opening the C4OS configuration file.",
            status: "pending",
          },
          save: {
            message: "The active generation changed before Save.",
            status: "error",
          },
        })}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Configuration action failed",
    );
    expect(screen.getByRole("button", { name: "Opening…" })).toBeDisabled();
  });
});
