import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GitBranchControl } from "./GitBranchControl";
import type { GitBranchControlProps, GitBranchOption } from "./types";

const BRANCHES: readonly GitBranchOption[] = [
  { name: "main", targetOid: "1234567890abcdef" },
  { name: "feature/shell", targetOid: "abcdef1234567890" },
];

/** Supplies inert authority callbacks so each scenario can override one boundary. */
function createProps(
  overrides: Partial<GitBranchControlProps> = {},
): GitBranchControlProps {
  return {
    activeBranch: "main",
    branches: BRANCHES,
    onApprove: vi.fn(),
    onCreate: vi.fn(),
    onDeny: vi.fn(),
    onSwitch: vi.fn(),
    ...overrides,
  };
}

describe("GitBranchControl", () => {
  it("shows the active branch in an upward menu and reports branch selection", async () => {
    const onSwitch = vi.fn();
    render(<GitBranchControl {...createProps({ onSwitch })} />);

    const trigger = screen.getByRole("button", { name: "Git branch" });
    expect(trigger).toHaveTextContent("main");
    fireEvent.click(trigger);

    const menu = await screen.findByRole("menu");
    expect(menu).toHaveAttribute("aria-label", "Local Git branches");
    expect(menu.closest("[data-placement]")).toHaveAttribute(
      "data-placement",
      "top",
    );
    expect(
      within(menu).getByRole("menuitemradio", {
        name: "main, 12345678",
      }),
    ).toHaveAttribute("aria-checked", "true");
    expect(
      within(menu).getByRole("menuitemradio", {
        name: "feature/shell, abcdef12",
      }),
    ).toHaveAttribute("aria-checked", "false");
    expect(
      within(menu).getByRole("menuitem", { name: "+ Create New" }),
    ).toBeVisible();

    fireEvent.click(
      within(menu).getByRole("menuitemradio", {
        name: "feature/shell, abcdef12",
      }),
    );
    expect(onSwitch).toHaveBeenCalledWith("feature/shell");
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(trigger).toHaveFocus();
  });

  it("validates a controlled branch name before reporting create", async () => {
    const onCreate = vi.fn();
    render(<GitBranchControl {...createProps({ onCreate })} />);

    const trigger = screen.getByRole("button", { name: "Git branch" });
    fireEvent.click(trigger);
    fireEvent.click(
      await screen.findByRole("menuitem", { name: "+ Create New" }),
    );

    const dialog = await screen.findByRole("dialog", {
      name: "Create branch",
    });
    const input = within(dialog).getByRole("textbox", { name: "Branch name" });
    expect(input).toHaveFocus();
    fireEvent.click(within(dialog).getByRole("button", { name: "Create" }));
    expect(within(dialog).getByRole("alert")).toHaveTextContent(
      "Enter a branch name.",
    );
    expect(onCreate).not.toHaveBeenCalled();

    fireEvent.change(input, { target: { value: "bad branch" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create" }));
    expect(within(dialog).getByRole("alert")).toHaveTextContent(
      "Use a valid Git branch name",
    );

    fireEvent.change(input, { target: { value: " feature/composer " } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Create" }));
    expect(onCreate).toHaveBeenCalledWith("feature/composer");
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(trigger).toHaveFocus();
  });

  it("restores focus after keyboard dismissal of the menu and create dialog", async () => {
    render(<GitBranchControl {...createProps()} />);

    const trigger = screen.getByRole("button", { name: "Git branch" });
    fireEvent.focus(trigger);
    fireEvent.click(trigger);
    const menu = await screen.findByRole("menu");
    fireEvent.keyDown(menu, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(trigger).toHaveFocus();

    fireEvent.click(trigger);
    fireEvent.click(
      await screen.findByRole("menuitem", { name: "+ Create New" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Create branch",
    });
    fireEvent.keyDown(dialog, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(trigger).toHaveFocus();
  });

  it("exposes an inline pending approval with authoritative approve and deny IDs", () => {
    const onApprove = vi.fn();
    const onDeny = vi.fn();
    const props = createProps({
      onApprove,
      onDeny,
      pendingApprovalId: "approval-17",
    });
    const { rerender } = render(<GitBranchControl {...props} />);

    const approval = screen.getByRole("region", {
      name: "Branch change approval",
    });
    expect(approval).toHaveTextContent("Branch change requires approval.");
    expect(screen.getByRole("button", { name: "Git branch" })).toBeDisabled();
    fireEvent.click(within(approval).getByRole("button", { name: "Approve" }));
    fireEvent.click(within(approval).getByRole("button", { name: "Cancel" }));
    expect(onApprove).toHaveBeenCalledWith("approval-17");
    expect(onDeny).toHaveBeenCalledWith("approval-17");

    rerender(<GitBranchControl {...props} isBusy />);
    expect(screen.getByRole("button", { name: "Git branch" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Approve" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  });
});
