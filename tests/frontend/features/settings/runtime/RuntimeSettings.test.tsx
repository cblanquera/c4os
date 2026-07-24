import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { RuntimeSettings } from "../../../../../src/frontend/features/settings/runtime/RuntimeSettings";
import type {
  RuntimeOptionView,
  RuntimeSettingsActions,
  RuntimeSettingsSnapshot,
} from "../../../../../src/frontend/features/settings/runtime/types";

function actions(): RuntimeSettingsActions {
  return {
    onRetry: vi.fn(),
    onSaveRuntime: vi.fn(),
  };
}

const openCode: RuntimeOptionView = {
  detail: "Verified native runtime and adapter are healthy.",
  health: "healthy",
  id: "runtime-opencode",
  kind: "open-code",
  label: "OpenCode",
  version: "1.18.3",
};

const pi: RuntimeOptionView = {
  detail: "Verified native runtime is available with degraded resume support.",
  health: "degraded",
  id: "runtime-pi",
  kind: "pi",
  label: "Pi",
  version: "0.80.10",
};

const unavailablePi: RuntimeOptionView = {
  ...pi,
  detail: "The pinned Pi installation is unavailable.",
  health: "unavailable",
};

function ready(
  overrides: Partial<
    Extract<RuntimeSettingsSnapshot, { status: "ready" }>
  > = {},
): Extract<RuntimeSettingsSnapshot, { status: "ready" }> {
  return {
    generation: 11,
    runtimes: [openCode, pi],
    save: { message: "OpenCode is the saved default.", status: "idle" },
    savedRuntimeId: openCode.id,
    status: "ready",
    ...overrides,
  };
}

describe("RuntimeSettings", () => {
  it("keeps loading, error, retry, and empty states distinct", () => {
    const handlers = actions();
    const { rerender } = render(
      <RuntimeSettings
        actions={handlers}
        snapshot={{ message: "Reading runtime defaults.", status: "loading" }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading runtimes");

    rerender(
      <RuntimeSettings
        actions={handlers}
        snapshot={{
          message: "The runtime snapshot could not be read.",
          retryable: true,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Runtimes are unavailable",
    );
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(handlers.onRetry).toHaveBeenCalledTimes(1);

    rerender(
      <RuntimeSettings
        actions={handlers}
        snapshot={ready({ runtimes: [], savedRuntimeId: null })}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "No runtimes available" }),
    ).toBeInTheDocument();
  });

  it("keeps an explicit local draft and saves it through the service callback", () => {
    const handlers = actions();
    render(<RuntimeSettings actions={handlers} snapshot={ready()} />);

    const save = screen.getByRole("button", { name: "Save Runtime" });
    expect(save).toBeDisabled();
    expect(
      screen.getByText(/Existing Chats retain their current runtime bindings/u),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: /Pi 0.80.10/u }));
    expect(save).toBeEnabled();
    expect(
      screen.getByText("Pi is selected as an unsaved draft."),
    ).toBeInTheDocument();
    fireEvent.click(save);
    expect(handlers.onSaveRuntime).toHaveBeenCalledWith("runtime-pi", 11);
  });

  it("requires an explicit selection when no runtime default is saved", () => {
    const handlers = actions();
    render(
      <RuntimeSettings
        actions={handlers}
        snapshot={ready({ savedRuntimeId: null })}
      />,
    );

    const save = screen.getByRole("button", { name: "Save Runtime" });
    expect(save).toBeDisabled();
    expect(
      screen.getByRole("radio", { name: /OpenCode 1.18.3/u }),
    ).not.toBeChecked();
    expect(
      screen.getByRole("radio", { name: /Pi 0.80.10/u }),
    ).not.toBeChecked();
    expect(
      screen.getByText(
        "No runtime default is saved. Choose an available runtime to activate Save Runtime.",
      ),
    ).toBeInTheDocument();
    expect(handlers.onSaveRuntime).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("radio", { name: /OpenCode 1.18.3/u }));
    expect(save).toBeEnabled();
    fireEvent.click(save);
    expect(handlers.onSaveRuntime).toHaveBeenCalledWith("runtime-opencode", 11);
  });

  it("tracks pending and accepted service state without losing the selected draft", () => {
    const handlers = actions();
    const { rerender } = render(
      <RuntimeSettings actions={handlers} snapshot={ready()} />,
    );
    fireEvent.click(screen.getByRole("radio", { name: /Pi 0.80.10/u }));

    rerender(
      <RuntimeSettings
        actions={handlers}
        snapshot={ready({
          save: { message: "Persisting Pi as the default.", status: "pending" },
        })}
      />,
    );
    expect(
      screen.getByRole("button", { name: "Saving Runtime…" }),
    ).toBeDisabled();
    expect(screen.getByRole("radio", { name: /Pi 0.80.10/u })).toBeChecked();

    rerender(
      <RuntimeSettings
        actions={handlers}
        snapshot={ready({
          generation: 12,
          save: { message: "Pi is saved for new Chats.", status: "success" },
          savedRuntimeId: pi.id,
        })}
      />,
    );
    expect(screen.getByText("Runtime default saved")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save Runtime" })).toBeDisabled();
    expect(screen.getByRole("radio", { name: /Pi 0.80.10/u })).toBeChecked();
  });

  it("surfaces save failure and prevents unavailable runtime selection", () => {
    const handlers = actions();
    render(
      <RuntimeSettings
        actions={handlers}
        snapshot={ready({
          runtimes: [openCode, unavailablePi],
          save: {
            message: "The configuration generation changed.",
            status: "error",
          },
        })}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Runtime default was not saved",
    );
    expect(screen.getByRole("radio", { name: /Pi 0.80.10/u })).toBeDisabled();
  });
});
