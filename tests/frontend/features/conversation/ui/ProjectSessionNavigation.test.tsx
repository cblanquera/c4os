import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ProjectSessionNavigation } from "../../../../../src/frontend/features/conversation/ui/ProjectSessionNavigation";
import type { ProjectNavigationItem } from "../../../../../src/frontend/features/conversation/ui/types";

const PROJECTS: readonly ProjectNavigationItem[] = [
  {
    id: "project-bravo",
    isExpanded: true,
    name: "Bravo",
    pathState: "found",
    sessions: [
      { id: "session-release", title: "Release checklist" },
      { id: "session-design", title: "Design notes" },
    ],
  },
  {
    id: "project-alpha",
    isExpanded: false,
    name: "Alpha",
    pathState: "missing",
    sessions: [{ id: "session-research", title: "Release research" }],
  },
];

function createProps() {
  return {
    activeProjectId: "project-bravo",
    activeSessionId: "session-release",
    onAddProject: vi.fn(),
    onNewChat: vi.fn(),
    onProjectActivate: vi.fn(),
    onProjectCopyPath: vi.fn(),
    onProjectExpansionChange: vi.fn(),
    onProjectOrderChange: vi.fn(),
    onProjectRelocate: vi.fn(),
    onProjectRemove: vi.fn(),
    onProjectRename: vi.fn(),
    onProjectReveal: vi.fn(),
    onSearchClear: vi.fn(),
    onSearchQueryChange: vi.fn(),
    onSessionActivate: vi.fn(),
    onSessionRemove: vi.fn(),
    projects: PROJECTS,
    searchQuery: "",
  };
}

describe("ProjectSessionNavigation", () => {
  it("preserves project order, expansion, active state, and missing-path semantics", () => {
    const props = createProps();
    const { container } = render(<ProjectSessionNavigation {...props} />);

    const projectRows = Array.from(
      container.querySelectorAll(".conversation-navigation__project-list > li"),
    );
    expect(projectRows).toHaveLength(2);
    expect(projectRows[0]).toHaveTextContent("Bravo");
    expect(projectRows[1]).toHaveTextContent("Alpha");
    expect(projectRows[0]).toHaveAttribute("data-active", "true");
    expect(projectRows[1]).toHaveAttribute("data-path-state", "missing");
    expect(projectRows[1]).toHaveTextContent("Project folder is missing");

    expect(
      screen.getByRole("button", { name: "Collapse Bravo" }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(
      screen.getByRole("button", { name: "Release checklist" }),
    ).toHaveAttribute("aria-current", "page");
    expect(
      screen.queryByRole("button", { name: "Release research" }),
    ).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Expand Alpha" }));
    expect(props.onProjectExpansionChange).toHaveBeenCalledWith(
      "project-alpha",
      true,
    );
  });

  it("exposes found and missing Project actions through accessible menus", () => {
    const props = createProps();
    render(<ProjectSessionNavigation {...props} />);

    fireEvent.click(screen.getByRole("button", { name: "New chat in Alpha" }));
    expect(props.onNewChat).toHaveBeenCalledWith("project-alpha");

    const bravoMore = screen.getByRole("button", {
      name: "More actions for Bravo",
    });
    fireEvent.click(bravoMore);
    const bravoMenu = screen.getByRole("menu", {
      name: "Actions for Bravo",
    });
    expect(bravoMore).toHaveAttribute("aria-expanded", "true");
    expect(
      within(bravoMenu).getByRole("menuitem", { name: "Reveal" }),
    ).toHaveFocus();
    expect(
      within(bravoMenu).queryByRole("menuitem", { name: "Relocate" }),
    ).toBeNull();
    fireEvent.click(
      within(bravoMenu).getByRole("menuitem", { name: "Copy path" }),
    );
    expect(props.onProjectCopyPath).toHaveBeenCalledWith("project-bravo");
    expect(bravoMore).toHaveFocus();

    const alphaMore = screen.getByRole("button", {
      name: "More actions for Alpha",
    });
    fireEvent.click(alphaMore);
    const alphaMenu = screen.getByRole("menu", {
      name: "Actions for Alpha",
    });
    expect(
      within(alphaMenu).getByRole("menuitem", { name: "Relocate" }),
    ).toHaveFocus();
    expect(
      within(alphaMenu).queryByRole("menuitem", { name: "Reveal" }),
    ).toBeNull();
    fireEvent.click(
      within(alphaMenu).getByRole("menuitem", { name: "Relocate" }),
    );
    expect(props.onProjectRelocate).toHaveBeenCalledWith("project-alpha");

    fireEvent.click(screen.getByRole("button", { name: "Add project" }));
    expect(props.onAddProject).toHaveBeenCalledTimes(1);
  });

  it("supports menu navigation, explicit actions, and focus restoration", () => {
    const props = createProps();
    render(<ProjectSessionNavigation {...props} />);

    const more = screen.getByRole("button", {
      name: "More actions for Bravo",
    });
    fireEvent.click(more);
    const menu = screen.getByRole("menu", { name: "Actions for Bravo" });
    const reveal = within(menu).getByRole("menuitem", {
      name: "Reveal",
    });
    const copyPath = within(menu).getByRole("menuitem", { name: "Copy path" });

    expect(reveal).toHaveFocus();
    fireEvent.keyDown(menu, { key: "ArrowDown" });
    expect(copyPath).toHaveFocus();
    fireEvent.keyDown(menu, { key: "End" });
    expect(
      within(menu).getByRole("menuitem", { name: "Move down" }),
    ).toHaveFocus();
    fireEvent.keyDown(menu, { key: "Escape" });
    expect(
      screen.queryByRole("menu", { name: "Actions for Bravo" }),
    ).toBeNull();
    expect(more).toHaveFocus();

    fireEvent.click(more);
    const reopenedMenu = screen.getByRole("menu", {
      name: "Actions for Bravo",
    });
    fireEvent.click(
      within(reopenedMenu).getByRole("menuitem", { name: "Rename" }),
    );
    const renameDialog = screen.getByRole("dialog", { name: "Rename Project" });
    const renameInput = within(renameDialog).getByRole("textbox", {
      name: "Project name",
    });
    expect(renameInput).toHaveFocus();
    expect(renameInput).toHaveValue("Bravo");
    fireEvent.change(renameInput, { target: { value: "Bravo Next" } });
    fireEvent.click(
      within(renameDialog).getByRole("button", { name: "Rename" }),
    );
    expect(props.onProjectRename).toHaveBeenCalledWith(
      "project-bravo",
      "Bravo Next",
    );
    expect(more).toHaveFocus();

    fireEvent.click(more);
    fireEvent.click(
      within(screen.getByRole("menu", { name: "Actions for Bravo" })).getByRole(
        "menuitem",
        { name: "Reveal" },
      ),
    );
    expect(props.onProjectReveal).toHaveBeenCalledWith("project-bravo");

    fireEvent.click(more);
    fireEvent.click(
      within(screen.getByRole("menu", { name: "Actions for Bravo" })).getByRole(
        "menuitem",
        { name: "Remove" },
      ),
    );
    expect(props.onProjectRemove).toHaveBeenCalledWith("project-bravo");
  });

  it("contains rename focus and closes from Escape or the modal backdrop", () => {
    const props = createProps();
    render(<ProjectSessionNavigation {...props} />);

    const more = screen.getByRole("button", {
      name: "More actions for Bravo",
    });
    fireEvent.click(more);
    fireEvent.click(
      within(screen.getByRole("menu", { name: "Actions for Bravo" })).getByRole(
        "menuitem",
        { name: "Rename" },
      ),
    );

    let dialog = screen.getByRole("dialog", { name: "Rename Project" });
    const input = within(dialog).getByRole("textbox", {
      name: "Project name",
    });
    const rename = within(dialog).getByRole("button", { name: "Rename" });
    rename.focus();
    fireEvent.keyDown(rename, { key: "Tab" });
    expect(input).toHaveFocus();
    fireEvent.keyDown(input, { key: "Tab", shiftKey: true });
    expect(rename).toHaveFocus();
    fireEvent.keyDown(rename, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Rename Project" })).toBeNull();
    expect(more).toHaveFocus();

    fireEvent.click(more);
    fireEvent.click(
      within(screen.getByRole("menu", { name: "Actions for Bravo" })).getByRole(
        "menuitem",
        { name: "Rename" },
      ),
    );
    dialog = screen.getByRole("dialog", { name: "Rename Project" });
    fireEvent.click(dialog);
    expect(screen.queryByRole("dialog", { name: "Rename Project" })).toBeNull();
    expect(more).toHaveFocus();
  });

  it("reorders whole Projects from the keyboard without flattening sessions", () => {
    const props = createProps();
    render(<ProjectSessionNavigation {...props} />);

    const bravo = screen.getByRole("button", { name: "Bravo" });
    expect(bravo).toHaveAttribute(
      "aria-keyshortcuts",
      "Alt+ArrowUp Alt+ArrowDown",
    );
    fireEvent.keyDown(bravo, { altKey: true, key: "ArrowDown" });
    expect(props.onProjectOrderChange).toHaveBeenCalledWith([
      "project-alpha",
      "project-bravo",
    ]);
    expect(PROJECTS[0]?.sessions.map((session) => session.id)).toEqual([
      "session-release",
      "session-design",
    ]);

    fireEvent.click(
      screen.getByRole("button", { name: "More actions for Bravo" }),
    );
    fireEvent.click(
      within(screen.getByRole("menu", { name: "Actions for Bravo" })).getByRole(
        "menuitem",
        { name: "Move down" },
      ),
    );
    expect(props.onProjectOrderChange).toHaveBeenLastCalledWith([
      "project-alpha",
      "project-bravo",
    ]);
  });

  it("reorders whole Projects through pointer drag", () => {
    const props = createProps();
    const { container } = render(<ProjectSessionNavigation {...props} />);
    const rows = Array.from(
      container.querySelectorAll<HTMLElement>(
        ".conversation-navigation__project-list > li",
      ),
    );
    const transfer = {
      dropEffect: "none",
      effectAllowed: "none",
      setData: vi.fn(),
    };

    fireEvent.dragStart(rows[0] as HTMLElement, { dataTransfer: transfer });
    fireEvent.dragOver(rows[1] as HTMLElement, { dataTransfer: transfer });
    fireEvent.drop(rows[1] as HTMLElement, { dataTransfer: transfer });

    expect(transfer.setData).toHaveBeenCalledWith(
      "text/plain",
      "project-bravo",
    );
    expect(props.onProjectOrderChange).toHaveBeenCalledWith([
      "project-alpha",
      "project-bravo",
    ]);
  });

  it("keeps Session Remove actions focusable in hierarchy and search", () => {
    const props = createProps();
    const { rerender } = render(<ProjectSessionNavigation {...props} />);

    const removeRelease = screen.getByRole("button", {
      name: "Remove Release checklist from Bravo",
    });
    removeRelease.focus();
    expect(removeRelease).toHaveFocus();
    fireEvent.click(removeRelease);
    expect(props.onSessionRemove).toHaveBeenCalledWith(
      "session-release",
      "project-bravo",
    );

    const searchProps = { ...createProps(), searchQuery: "research" };
    rerender(<ProjectSessionNavigation {...searchProps} />);
    fireEvent.click(
      screen.getByRole("button", {
        name: "Remove Release research from Alpha",
      }),
    );
    expect(searchProps.onSessionRemove).toHaveBeenCalledWith(
      "session-research",
      "project-alpha",
    );
  });

  it("replaces the project hierarchy with flat matching sessions without clearing on activation", () => {
    const props = { ...createProps(), searchQuery: "release" };
    render(<ProjectSessionNavigation {...props} />);

    expect(screen.queryByRole("heading", { name: "Projects" })).toBeNull();
    const results = screen.getByRole("heading", {
      name: "Search results",
    }).parentElement;
    expect(results).not.toBeNull();
    const resultItems = within(results as HTMLElement).getAllByRole("listitem");
    expect(resultItems).toHaveLength(2);
    expect(resultItems[0]).toHaveTextContent("Release checklistBravo");
    expect(resultItems[1]).toHaveTextContent("Release researchAlpha");

    fireEvent.click(
      within(resultItems[1] as HTMLElement).getByRole("button", {
        name: "Release researchAlpha",
      }),
    );
    expect(props.onSessionActivate).toHaveBeenCalledWith(
      "session-research",
      "project-alpha",
    );
    expect(screen.getByRole("searchbox", { name: "Search chats" })).toHaveValue(
      "release",
    );
  });

  it("shows no-results and clears with button or Escape while restoring search focus", () => {
    const props = { ...createProps(), searchQuery: "no match" };
    const { rerender } = render(<ProjectSessionNavigation {...props} />);

    expect(screen.getByRole("status")).toHaveTextContent(
      "No chat sessions found.",
    );
    fireEvent.click(screen.getByRole("button", { name: "Clear chat search" }));
    expect(props.onSearchQueryChange).toHaveBeenCalledWith("");
    expect(props.onSearchClear).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole("searchbox", { name: "Search chats" }),
    ).toHaveFocus();

    const escapeProps = { ...createProps(), searchQuery: "design" };
    rerender(<ProjectSessionNavigation {...escapeProps} />);
    const search = screen.getByRole("searchbox", { name: "Search chats" });
    fireEvent.keyDown(search, { key: "Escape" });
    expect(escapeProps.onSearchQueryChange).toHaveBeenCalledWith("");
    expect(escapeProps.onSearchClear).toHaveBeenCalledTimes(1);
    expect(search).toHaveFocus();
  });
});
