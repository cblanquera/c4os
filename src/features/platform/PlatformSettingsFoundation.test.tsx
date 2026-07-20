import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router";
import { describe, expect, it } from "vitest";

import type { PickerOutcome } from "../../platform/platform-service";
import type { PickerGrantId, RequestId } from "../../platform/protocol";
import { PlatformSettingsFoundation } from "./PlatformSettingsFoundation";

function renderSettings(
  chooseProjectFolder: () => Promise<PickerOutcome>,
  initialEntries = ["/settings/providers"],
  resolveBackRoute: () => string | null = () => null,
) {
  return render(
    <MemoryRouter
      initialEntries={initialEntries}
      initialIndex={initialEntries.length - 1}
    >
      <Routes>
        <Route
          path="/settings/providers"
          element={
            <PlatformSettingsFoundation
              chooseProjectFolder={chooseProjectFolder}
              resolveBackRoute={resolveBackRoute}
            />
          }
        />
        <Route path="/start" element={<h1>Workspace Start</h1>} />
        <Route path="/" element={<h1>C4OS Home</h1>} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("PlatformSettingsFoundation", () => {
  it("uses the production Settings route and displays only picker-safe metadata", async () => {
    renderSettings(() =>
      Promise.resolve({
        type: "selected",
        contractVersion: 1,
        requestId: "picker-request" as RequestId,
        grants: [
          {
            grantId: "picker-grant" as PickerGrantId,
            objectKind: "folder",
            displayName: "c4os-project",
          },
        ],
      }),
    );

    expect(screen.getByRole("heading", { name: "Providers" })).toBeVisible();
    expect(
      screen.getByRole("navigation", { name: "Settings" }),
    ).toHaveTextContent("MCP Servers");
    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder…" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "Access granted to c4os-project.",
      ),
    );
    expect(document.body).not.toHaveTextContent("picker-grant");
    expect(document.body).not.toHaveTextContent("/Users/");
  });

  it("reports cancellation without minting visible access", async () => {
    renderSettings(() =>
      Promise.resolve({
        type: "cancelled",
        contractVersion: 1,
        requestId: "picker-request" as RequestId,
      }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder…" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "Folder selection cancelled. No access was granted.",
      ),
    );
  });

  it("returns through the same router history", () => {
    renderSettings(
      () => Promise.reject(new Error("unused")),
      ["/start", "/settings/providers"],
      () => "/start",
    );
    fireEvent.click(screen.getByRole("button", { name: "Back to C4OS" }));
    expect(
      screen.getByRole("heading", { name: "Workspace Start" }),
    ).toBeVisible();
  });
});
