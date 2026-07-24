import { render, screen } from "@testing-library/react";

import type {
  FoundationSnapshot,
  StateGeneration,
} from "../../../../src/frontend/platform/protocol";
import { FoundationScreen } from "../../../../src/frontend/features/foundation/FoundationScreen";

const snapshot: FoundationSnapshot = {
  protocolVersion: 1,
  generation: 0 as StateGeneration,
  authority: "rust-core",
  redactions: [],
};

describe("FoundationScreen", () => {
  it("shows a verified Rust authority and disables unfinished behavior", async () => {
    render(<FoundationScreen loadSnapshot={() => Promise.resolve(snapshot)} />);

    expect(
      screen.getByRole("heading", { name: "C4OS foundation is running" }),
    ).toBeVisible();
    expect(await screen.findByText("Connected · Rust core")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Start a Workspace" }),
    ).toBeDisabled();
  });

  it("fails closed when the native authority is unavailable", async () => {
    render(
      <FoundationScreen
        loadSnapshot={() => Promise.reject(new Error("native unavailable"))}
      />,
    );

    expect(await screen.findByText("Unavailable · fail closed")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Start a Workspace" }),
    ).toBeDisabled();
  });
});
