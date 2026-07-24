import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  DiagnosticsExportSnapshot,
  DiagnosticsSnapshot,
} from "../../../platform/diagnostic-service";
import { DiagnosticSettingsController } from "./DiagnosticSettingsController";

const diagnosticService = vi.hoisted(() => ({
  exportDiagnosticsSnapshot: vi.fn(),
  readDiagnosticsSnapshot: vi.fn(),
}));

vi.mock("../../../platform/diagnostic-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../../platform/diagnostic-service")
  >("../../../platform/diagnostic-service");
  return { ...actual, ...diagnosticService };
});

function snapshot(generation: number): DiagnosticsSnapshot {
  return {
    schemaVersion: 1,
    generation,
    records: [],
    truncated: false,
  };
}

function exported(generation: number): DiagnosticsExportSnapshot {
  return {
    ...snapshot(generation),
    exportId: `export:${generation}`,
    createdAtMs: 1_784_476_800_000,
    sha256: `sha256:${"b".repeat(64)}`,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("DiagnosticSettingsController", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not let a deferred refresh overwrite a newer export", async () => {
    const lateRefresh = deferred<DiagnosticsSnapshot>();
    diagnosticService.readDiagnosticsSnapshot
      .mockResolvedValueOnce(snapshot(12))
      .mockReturnValueOnce(lateRefresh.promise);
    diagnosticService.exportDiagnosticsSnapshot.mockResolvedValueOnce(
      exported(14),
    );
    render(<DiagnosticSettingsController />);

    await screen.findByRole("button", { name: "Refresh" });
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    fireEvent.click(
      screen.getByRole("button", { name: "Prepare redacted export" }),
    );
    expect(
      await screen.findByText(/Prepared redacted export export:14\./),
    ).toBeVisible();

    await act(async () => {
      lateRefresh.resolve(snapshot(13));
      await lateRefresh.promise;
    });

    expect(screen.getByText(/Generation 14/)).toBeVisible();
    expect(screen.getByText("export:14")).toBeVisible();
  });
});
