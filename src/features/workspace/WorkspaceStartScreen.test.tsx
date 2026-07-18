import { act, fireEvent, render, screen } from "@testing-library/react";

import {
  WorkspaceStartScreen,
  type RecentWorkspace,
  type WorkspaceOpenResult,
} from "./WorkspaceStartScreen";

const recents: readonly RecentWorkspace[] = [
  { id: "one", name: "One", detail: "Today" },
  { id: "two", name: "Two", detail: "Yesterday" },
  { id: "three", name: "Three", detail: "Last week", isMissing: true },
];

function deferredResult() {
  let resolve!: (value: WorkspaceOpenResult) => void;
  const promise = new Promise<WorkspaceOpenResult>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("WorkspaceStartScreen", () => {
  it("shows exactly three actions and deterministic recent rows", () => {
    render(
      <WorkspaceStartScreen
        recents={recents}
        openWorkspace={() => Promise.reject(new Error("unused"))}
      />,
    );

    expect(screen.getByLabelText("Open options").children).toHaveLength(3);
    expect(screen.getByRole("list").children).toHaveLength(3);
    expect(screen.getByRole("button", { name: /Three/ })).toHaveTextContent(
      "Locate",
    );
  });

  it("announces progress, disables competing actions, and reports recovery", async () => {
    const opening = deferredResult();
    render(
      <WorkspaceStartScreen
        recents={recents}
        openWorkspace={() => opening.promise}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Open a folder/ }));
    expect(screen.getByRole("status")).toHaveTextContent(
      "Opening Open a folder…",
    );
    expect(
      screen.getByRole("button", { name: /Clone Repository/ }),
    ).toBeDisabled();

    opening.resolve({ workspaceName: "Recovered Fixture", recovered: true });
    expect(
      await screen.findByText(/Recovered Recovered Fixture/),
    ).toBeVisible();
  });

  it("keeps existing state and surfaces a bounded failure", async () => {
    render(
      <WorkspaceStartScreen
        recents={recents}
        openWorkspace={() => Promise.reject(new Error("sensitive path"))}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Open a workspace/ }));
    expect(
      await screen.findByText(/Your existing state is unchanged/),
    ).toBeVisible();
    expect(screen.queryByText("sensitive path")).not.toBeInTheDocument();
  });

  it("ignores a stale completion when two presses race a render", async () => {
    const first = deferredResult();
    const second = deferredResult();
    const openWorkspace = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    render(
      <WorkspaceStartScreen recents={recents} openWorkspace={openWorkspace} />,
    );

    const folder = screen.getByRole("button", { name: /Open a folder/ });
    act(() => {
      folder.click();
      folder.click();
    });
    expect(openWorkspace).toHaveBeenCalledTimes(2);
    second.resolve({ workspaceName: "Current", recovered: false });
    expect(await screen.findByText(/Opened Current/)).toBeVisible();

    first.resolve({ workspaceName: "Stale", recovered: false });
    await Promise.resolve();
    expect(screen.queryByText(/Opened Stale/)).not.toBeInTheDocument();
  });
});
