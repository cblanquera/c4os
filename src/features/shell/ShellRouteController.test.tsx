import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { Provider } from "react-redux";
import { createMemoryRouter, RouterProvider } from "react-router";
import { describe, expect, it } from "vitest";

import { APP_ROUTE_DEFINITIONS } from "../../app/route-contract";
import { createAppStore } from "../../app/store";
import {
  initialQaState,
  initialShellAuthorityState,
  initialShellDraftState,
  shellDraftActions,
} from "./state";
import type { ArtifactId } from "../../platform/protocol";
import { ShellRouteController } from "./ShellRouteController";

function renderShellAt(path: string, qaEnabled = false) {
  const store = createAppStore({
    shellAuthority: initialShellAuthorityState,
    shellDrafts: initialShellDraftState,
    shellQa: qaEnabled
      ? { enabled: true, fixtureId: "test", activeWorkflow: "chat" }
      : initialQaState,
  });
  const router = createMemoryRouter(
    APP_ROUTE_DEFINITIONS.map((route) => ({
      path: route.path,
      element: <ShellRouteController route={route.path} />,
    })),
    { initialEntries: [path] },
  );
  return {
    store,
    router,
    ...render(
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>,
    ),
  };
}

describe("ShellRouteController", () => {
  it("keeps every accepted route directly addressable in one composed shell", () => {
    for (const definition of APP_ROUTE_DEFINITIONS) {
      const rendered = renderShellAt(definition.path);
      expect(
        screen.getByRole("heading", { name: definition.title, level: 1 }),
      ).toBeVisible();
      expect(
        document.querySelector(`[data-route="${definition.path}"]`),
      ).toBeInTheDocument();
      rendered.unmount();
    }
  });

  it("round-trips through Settings without losing panel, composer, or focus state", async () => {
    const { store } = renderShellAt("/chat", true);
    const composer = screen.getByRole("textbox", { name: "Message" });
    fireEvent.change(composer, { target: { value: "Keep this exact draft" } });
    fireEvent.click(
      screen.getByRole("button", { name: "Collapse project panel" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(
      await screen.findByRole("heading", { name: "Providers", level: 1 }),
    ).toBeVisible();
    expect(store.getState().shellDrafts.settings.visit?.route).toBe("/chat");

    fireEvent.click(screen.getByRole("button", { name: "Back to C4OS" }));
    const restoredComposer = await screen.findByRole("textbox", {
      name: "Message",
    });
    expect(restoredComposer).toHaveValue("Keep this exact draft");
    expect(
      screen.getByRole("button", { name: "Show project panel" }),
    ).toBeVisible();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Settings" })).toHaveFocus(),
    );
    expect(store.getState().shellDrafts.settings.visit).toBeNull();
  });

  it("does not expose the review Settings control in production state", () => {
    renderShellAt("/chat");
    expect(screen.queryByRole("button", { name: "Settings" })).toBeNull();
  });

  it("shows contextual Chat only from explicit artifact-focus draft state", () => {
    const store = createAppStore({
      shellAuthority: initialShellAuthorityState,
      shellDrafts: {
        ...initialShellDraftState,
        workspace: {
          ...initialShellDraftState.workspace,
          focusedArtifactId: "artifact:focused" as ArtifactId,
        },
      },
      shellQa: initialQaState,
    });
    const router = createMemoryRouter(
      [{ path: "/chat", element: <ShellRouteController route="/chat" /> }],
      { initialEntries: ["/chat"] },
    );
    render(
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>,
    );

    expect(screen.getByRole("complementary")).toHaveAccessibleName(
      "Projects and contextual chat",
    );
    expect(screen.getByRole("button", { name: "Detach Chat" })).toBeDisabled();
    expect(
      screen.getByRole("combobox", { name: "Composer mode" }),
    ).toBeDisabled();
  });

  it("reconciles retained panel width and ARIA bounds on every viewport resize", async () => {
    const originalWidth = window.innerWidth;
    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: 1280,
    });
    const rendered = renderShellAt("/chat");

    act(() => {
      rendered.store.dispatch(
        shellDraftActions.leftPanelResized({
          width: 900,
          viewportWidth: 1280,
        }),
      );
    });
    let resizer = screen.getByRole("separator", {
      name: "Resize project panel",
    });
    expect(resizer).toHaveAttribute("aria-valuenow", "704");
    expect(resizer).toHaveAttribute("aria-valuemax", "704");

    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: 760,
    });
    fireEvent(window, new Event("resize"));
    await waitFor(() =>
      expect(
        document.querySelector("[data-shell-layout='workspace']"),
      ).toHaveAttribute("data-panel-mode", "overlay"),
    );
    fireEvent.click(screen.getByRole("button", { name: "Show project panel" }));
    resizer = await screen.findByRole("separator", {
      name: "Resize project panel",
    });
    await waitFor(() => {
      expect(resizer).toHaveAttribute("aria-valuemax", "340");
      expect(resizer).toHaveAttribute("aria-valuenow", "340");
      expect(rendered.store.getState().shellDrafts.workspace.panel.width).toBe(
        340,
      );
    });

    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: 700,
    });
    fireEvent(window, new Event("resize"));
    await waitFor(() => {
      expect(resizer).toHaveAttribute("aria-valuemax", "280");
      expect(resizer).toHaveAttribute("aria-valuenow", "280");
    });

    rendered.unmount();
    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: originalWidth,
    });
  });
});
