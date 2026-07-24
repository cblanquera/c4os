import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { DiagnosticSettings } from "./DiagnosticSettings";
import type {
  DiagnosticSettingsActions,
  DiagnosticSettingsSnapshot,
} from "./types";

function actions(): DiagnosticSettingsActions {
  return {
    onRetry: vi.fn().mockResolvedValue(undefined),
    onExport: vi.fn().mockResolvedValue(undefined),
  };
}

function ready(): Extract<DiagnosticSettingsSnapshot, { status: "ready" }> {
  return {
    status: "ready",
    schemaVersion: 1,
    generation: 12,
    truncated: false,
    operation: {
      status: "idle",
      message: "Diagnostics are ready.",
      export: null,
    },
    records: [
      {
        diagnosticId: "diagnostic:workspace-1",
        correlationId: "correlation:workspace-1",
        category: "recovery",
        severity: "warning",
        componentBoundary: "workspace",
        message: "Interrupted archive save recovered from validated state.",
        recoveryAction: "rebuild_last_known_good",
        createdAtMs: 1_784_476_800_000,
      },
    ],
  };
}

describe("DiagnosticSettings", () => {
  it("renders redacted records with correlation and recovery metadata", () => {
    const handlers = actions();
    render(<DiagnosticSettings actions={handlers} snapshot={ready()} />);

    expect(screen.getByText(/Generation 12/)).toBeVisible();
    expect(screen.getByText("correlation:workspace-1")).toBeVisible();
    expect(screen.getByText("Rebuild Last Known Good")).toBeVisible();
    expect(screen.getByText(/does not return a filesystem path/)).toBeVisible();
    expect(screen.queryByText(/private\\/)).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Prepare redacted export" }),
    );
    expect(handlers.onExport).toHaveBeenCalledTimes(1);
  });

  it("shows a path-free export digest and focuses export completion", async () => {
    const handlers = actions();
    const initial = ready();
    const { rerender } = render(
      <DiagnosticSettings actions={handlers} snapshot={initial} />,
    );
    rerender(
      <DiagnosticSettings
        actions={handlers}
        snapshot={{
          ...initial,
          operation: {
            status: "success",
            message: "Prepared redacted export export:12.",
            export: {
              schemaVersion: 1,
              generation: 12,
              records: initial.records,
              truncated: false,
              exportId: "export:12",
              createdAtMs: 1_784_476_800_000,
              sha256: `sha256:${"b".repeat(64)}`,
            },
          },
        }}
      />,
    );

    expect(screen.getByText("export:12")).toBeVisible();
    expect(screen.getByText(`sha256:${"b".repeat(64)}`)).toBeVisible();
    const message = screen.getByText(/Prepared redacted export/);
    await waitFor(() =>
      expect(message.parentElement?.parentElement).toHaveFocus(),
    );
  });

  it("renders honest unavailable and truncated states", () => {
    const handlers = actions();
    const { rerender } = render(
      <DiagnosticSettings
        actions={handlers}
        snapshot={{
          status: "error",
          stateLabel: "Unavailable",
          message: "The native Diagnostics service is unavailable.",
          retryable: true,
        }}
      />,
    );
    expect(screen.getByText("Diagnostics are unavailable")).toBeVisible();
    expect(screen.getByRole("button", { name: "Try again" })).toBeEnabled();

    rerender(
      <DiagnosticSettings
        actions={handlers}
        snapshot={{ ...ready(), truncated: true }}
      />,
    );
    expect(screen.getByText("Older records are not shown")).toBeVisible();
  });

  it("omits dateTime for an out-of-range diagnostic timestamp", () => {
    const snapshot = ready();
    render(
      <DiagnosticSettings
        actions={actions()}
        snapshot={{
          ...snapshot,
          records: [
            {
              ...snapshot.records[0]!,
              createdAtMs: Number.MAX_SAFE_INTEGER,
            },
          ],
        }}
      />,
    );

    const observed = screen.getByText("Unknown time");
    expect(observed).toHaveRole("time");
    expect(observed).not.toHaveAttribute("datetime");
  });
});
