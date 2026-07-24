import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  createQaFixtureAdapter,
  QA_FIXTURE_CLOCK,
} from "../../../src/frontend/qa/fixture";
import { QaFoundationRoute } from "../../../src/frontend/qa/route";

describe("QA foundation route", () => {
  it("identifies deterministic fixture state without claiming production authority", () => {
    render(
      <QaFoundationRoute adapter={createQaFixtureAdapter({ enabled: true })} />,
    );

    expect(
      screen.getByRole("heading", { name: "C4OS QA Foundation" }),
    ).toBeTruthy();
    expect(screen.getByLabelText("Fixture mode")).toBeTruthy();
    expect(
      screen.getByText("Deterministic QA data only — not production state."),
    ).toBeTruthy();
    expect(screen.getByText("qa-fixture-only")).toBeTruthy();
    expect(screen.getByText(QA_FIXTURE_CLOCK)).toBeTruthy();
  });

  it("resets explicitly without writing browser storage", () => {
    const storageSpies = [
      vi.spyOn(Storage.prototype, "getItem"),
      vi.spyOn(Storage.prototype, "setItem"),
      vi.spyOn(Storage.prototype, "removeItem"),
      vi.spyOn(Storage.prototype, "clear"),
    ];
    render(
      <QaFoundationRoute adapter={createQaFixtureAdapter({ enabled: true })} />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Reset fixture" }));

    expect(
      screen.getByText(`Fixture reset to ${QA_FIXTURE_CLOCK}`),
    ).toBeTruthy();
    storageSpies.forEach((spy) => {
      expect(spy).not.toHaveBeenCalled();
      spy.mockRestore();
    });
  });

  it("renders a fail-closed explanation when the build gate is disabled", () => {
    render(
      <QaFoundationRoute
        adapter={createQaFixtureAdapter({ enabled: false })}
      />,
    );

    expect(screen.getByRole("status").textContent).toBe(
      "Fixture mode unavailable",
    );
    expect(screen.queryByText("workspace-qa-0001")).toBeNull();
    expect(screen.getByText(/contains no production state/i)).toBeTruthy();
  });
});
