import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";

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

  it("collects an HTTPS repository URL before cloning", async () => {
    const openWorkspace = vi.fn().mockResolvedValue(null);
    render(
      <WorkspaceStartScreen recents={recents} openWorkspace={openWorkspace} />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Clone Repository/ }));
    fireEvent.change(screen.getByRole("textbox", { name: "Repository URL" }), {
      target: { value: "https://github.com/example/c4os-fixture.git" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Clone" }));

    expect(openWorkspace).toHaveBeenCalledWith({
      type: "cloneRepository",
      repositoryUrl: "https://github.com/example/c4os-fixture.git",
    });
    expect(await screen.findByRole("status")).toHaveTextContent("");
  });

  it("freezes start actions inside a focus-managed clone approval", async () => {
    const answerCloneApproval = vi.fn().mockResolvedValue(null);
    render(
      <WorkspaceStartScreen
        answerCloneApproval={answerCloneApproval}
        recents={recents}
        openWorkspace={() =>
          Promise.resolve({
            promptId: "approval:clone-1",
            summary: "Clone this repository into the selected folder?",
          })
        }
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Clone Repository/ }));
    fireEvent.change(screen.getByRole("textbox", { name: "Repository URL" }), {
      target: { value: "https://github.com/example/c4os-fixture.git" },
    });
    const clone = screen.getByRole("button", { name: "Clone" });
    fireEvent.click(clone);

    expect(
      await screen.findByRole("dialog", { name: "Clone approval" }),
    ).toBeVisible();
    expect(screen.getByText("Open a folder").closest("button")).toBeDisabled();
    expect(
      screen.getByText("Open a workspace").closest("button"),
    ).toBeDisabled();
    expect(
      screen.getByText("Clone Repository").closest("button"),
    ).toBeDisabled();
    expect(clone).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(answerCloneApproval).toHaveBeenCalledWith(
        "approval:clone-1",
        "deny",
      ),
    );
    await waitFor(() => expect(clone).toHaveFocus());
  });

  it("acquires a synchronous lock before two presses can race a render", async () => {
    const first = deferredResult();
    const openWorkspace = vi.fn().mockReturnValueOnce(first.promise);
    render(
      <WorkspaceStartScreen recents={recents} openWorkspace={openWorkspace} />,
    );

    const folder = screen.getByRole("button", { name: /Open a folder/ });
    act(() => {
      folder.click();
      folder.click();
    });
    expect(openWorkspace).toHaveBeenCalledTimes(1);
    first.resolve({ workspaceName: "Current", recovered: false });
    expect(await screen.findByText(/Opened Current/)).toBeVisible();
  });
});
