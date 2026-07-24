import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { QaFixtureBoundary } from "./QaFixtureBoundary";

describe("QA fixture product boundary", () => {
  it("keeps fixture authority visible around a production-composed route", () => {
    const { container } = render(
      <QaFixtureBoundary route="/settings/models">
        <main>Production-composed Models surface</main>
      </QaFixtureBoundary>,
    );

    expect(screen.getByLabelText("QA fixture identity")).toHaveTextContent(
      "not production state",
    );
    expect(
      screen.getByRole("note", { name: "QA fixture identity" }),
    ).not.toHaveAttribute("tabindex");
    expect(
      container.querySelector('[data-qa-product-adapter="deterministic"]'),
    ).toHaveAttribute("data-qa-route", "/settings/models");
  });
});
