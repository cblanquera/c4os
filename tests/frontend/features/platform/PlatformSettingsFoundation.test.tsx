import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { PickerOutcome } from "../../../../src/frontend/platform/platform-service";
import type {
  PickerGrantId,
  RequestId,
} from "../../../../src/frontend/platform/protocol";
import { NativePlatformSettingsContent } from "../../../../src/frontend/features/platform/PlatformSettingsFoundation";

function renderSettings(chooseProjectFolder: () => Promise<PickerOutcome>) {
  return render(
    <NativePlatformSettingsContent chooseProjectFolder={chooseProjectFolder} />,
  );
}

describe("NativePlatformSettingsContent", () => {
  it("displays only picker-safe metadata", async () => {
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

    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder…" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "The native picker returned c4os-project. No Project was changed.",
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

  it("fails closed when native selection is unavailable", async () => {
    renderSettings(() => Promise.reject(new Error("native unavailable")));
    fireEvent.click(
      screen.getByRole("button", { name: "Choose project folder…" }),
    );

    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent(
        "Native folder access is unavailable",
      ),
    );
  });
});
