import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AdvancedPoliciesScreen } from "../../../../src/frontend/features/policy/AdvancedPoliciesScreen";

describe("AdvancedPoliciesScreen", () => {
  it("renders seven groups, searches across them, and supports dirty save and revert", () => {
    render(<AdvancedPoliciesScreen />);

    expect(
      screen.getAllByRole("button", { name: /workspace files/i }),
    ).toHaveLength(1);
    const groupNavigation = screen.getByRole("navigation", {
      name: "Policy groups",
    });
    expect(within(groupNavigation).getAllByRole("button")).toHaveLength(7);

    const save = screen.getByRole("button", { name: "Save Policies" });
    const revert = screen.getByRole("button", { name: "Revert" });
    expect(save).toBeDisabled();
    fireEvent.change(
      screen.getByRole("combobox", { name: "workspace.modify policy" }),
      {
        target: { value: "ask" },
      },
    );
    expect(save).toBeEnabled();
    expect(screen.getByText(/custom \(unsaved\)/i)).toBeInTheDocument();
    fireEvent.click(revert);
    expect(save).toBeDisabled();

    fireEvent.change(
      screen.getByRole("searchbox", { name: "Search policies" }),
      {
        target: { value: "credential" },
      },
    );
    expect(screen.getByText("credential.reveal")).toBeInTheDocument();
    expect(screen.getByText("credential.use")).toBeInTheDocument();
    expect(screen.queryByText("workspace.modify")).not.toBeInTheDocument();
  });

  it("keeps exceptions separate and revokes them without changing category defaults", () => {
    render(<AdvancedPoliciesScreen />);
    fireEvent.click(screen.getByRole("tab", { name: /exceptions 2/i }));
    expect(
      screen.getByRole("heading", { name: "Saved exceptions" }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Revoke" })).toHaveLength(2);

    fireEvent.click(screen.getAllByRole("button", { name: "Revoke" })[0]!);

    expect(screen.getByRole("status")).toHaveTextContent(
      /outstanding matching authorizations expire/i,
    );
    expect(
      screen.getByRole("tab", { name: /exceptions 1/i }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("tab", { name: "Category rules" }));
    expect(
      screen.getByRole("combobox", { name: "workspace.read policy" }),
    ).toHaveValue("default");
  });
});
