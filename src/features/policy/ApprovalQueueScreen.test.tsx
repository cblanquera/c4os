import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ApprovalQueueScreen } from "./ApprovalQueueScreen";

describe("ApprovalQueueScreen", () => {
  it("shows every lifecycle state and denies before releasing an effect", () => {
    render(<ApprovalQueueScreen />);

    for (const state of [
      "Pending",
      "Queued",
      "Expired",
      "Denied",
      "Completed",
    ]) {
      expect(screen.getByText(state)).toBeInTheDocument();
    }
    expect(screen.getByText(/1 pending · 1 queued/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Deny" }));

    expect(screen.getByRole("status")).toHaveTextContent(
      /no side effect was released/i,
    );
    expect(screen.getByText(/0 pending · 1 queued/i)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Approve once" }),
    ).not.toBeInTheDocument();
  });
});
