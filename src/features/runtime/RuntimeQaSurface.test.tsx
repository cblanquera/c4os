import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { RuntimeQaSurface } from "./RuntimeQaSurface";
import type { RuntimeCoreSnapshot } from "../../platform/runtime-core";

/** Opens one named Task 4 panel through the deterministic surface navigation. */
function openSurface(name: string) {
  fireEvent.click(screen.getByRole("button", { name }));
}

describe("RuntimeQaSurface", () => {
  it("renders the validated Rust-owned runtime authority projection", async () => {
    const snapshot: RuntimeCoreSnapshot = {
      authority: "rust-core",
      generation: 9 as never,
      providerGeneration: 4,
      capabilityGeneration: 6,
      runtimeGeneration: 7,
      onboardingReady: true,
      providers: [],
      runtimes: [],
      pendingApprovals: [],
    };
    render(
      <RuntimeQaSurface
        isEnabled
        readCoreSnapshot={() => Promise.resolve(snapshot)}
      />,
    );

    const authority = screen.getByLabelText("Rust runtime authority");
    expect(await within(authority).findByText("rust-core")).toBeInTheDocument();
    expect(within(authority).getByText("9")).toBeInTheDocument();
  });

  it("fails closed when the deterministic fixture gate is disabled", () => {
    render(<RuntimeQaSurface isEnabled={false} />);

    expect(
      screen.getByRole("heading", { name: "QA runtime fixtures are disabled" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", {
        name: "Providers, models, and recoverable Chats",
      }),
    ).not.toBeInTheDocument();
  });

  it("settles a published Rust-owned approval with its exact identifiers", async () => {
    const approval = {
      runtimeId: "opencode-primary" as never,
      correlationId: "correlation-approval" as never,
      promptId: "approval:prompt-1" as never,
    };
    const snapshot: RuntimeCoreSnapshot = {
      authority: "rust-core",
      generation: 9 as never,
      providerGeneration: 4,
      capabilityGeneration: 6,
      runtimeGeneration: 7,
      onboardingReady: true,
      providers: [],
      runtimes: [],
      pendingApprovals: [approval],
    };
    const answerApproval = vi.fn().mockResolvedValue({});
    render(
      <RuntimeQaSurface
        isEnabled
        readCoreSnapshot={() => Promise.resolve(snapshot)}
        answerApproval={answerApproval}
      />,
    );

    expect(
      await screen.findByRole("heading", { name: "Runtime effect is paused" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Deny" }));

    expect(
      await screen.findByText("Approval denied without executing an effect."),
    ).toBeInTheDocument();
    expect(answerApproval).toHaveBeenCalledWith({
      ...approval,
      answer: "deny",
    });
    expect(
      screen.queryByRole("heading", { name: "Runtime effect is paused" }),
    ).not.toBeInTheDocument();
  });

  it("keeps provider credentials opaque across zero, one, and many discovery", () => {
    render(<RuntimeQaSurface isEnabled />);

    expect(
      screen.getByRole("heading", { name: "Profiles and discovery" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Stored securely · ref only")).toHaveLength(4);
    expect(screen.getByText(/3 usable routes discovered/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "No usable models" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Continue is blocked");

    fireEvent.click(screen.getByRole("button", { name: "One usable model" }));
    expect(
      screen.getByText(/1 usable model route discovered/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Selected explicitly: OpenAI - Work/i),
    ).toBeInTheDocument();
  });

  it("requires an explicit route-compatible attachment resolution", () => {
    render(<RuntimeQaSurface isEnabled />);
    openSurface("Models & preflight");

    const details = screen
      .getByRole("heading", {
        name: "moonshotai/kimi-k2",
      })
      .closest("article");
    expect(details).not.toBeNull();

    const scopedDetails = within(details as HTMLElement);
    for (const state of ["Supported", "Unsupported", "Unknown", "Degraded"]) {
      expect(scopedDetails.getByText(state)).toBeInTheDocument();
    }
    expect(screen.getByText("Needs Vision")).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Use compatible model" }),
    );

    expect(
      document.querySelector('button[aria-pressed="true"]'),
    ).toHaveTextContent("openai/gpt-5");
    expect(screen.getByRole("status")).toHaveTextContent(
      /attachment ready for submission/i,
    );
  });

  it("saves only a dirty runtime draft and restarts the selected exact pin", () => {
    render(<RuntimeQaSurface isEnabled />);
    openSurface("Runtimes");

    const saveButton = screen.getByRole("button", { name: "Save Runtime" });
    expect(saveButton).toBeDisabled();
    expect(screen.getByText("OpenCode 1.18.3")).toBeInTheDocument();
    expect(screen.getByText("Pi 0.80.10")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: /Pi 0.80.10/i }));
    expect(saveButton).toBeEnabled();
    fireEvent.click(saveButton);
    expect(screen.getByRole("status")).toHaveTextContent(
      /Pi 0.80.10 saved for new Chats · existing bindings unchanged/i,
    );

    fireEvent.click(screen.getByRole("button", { name: "Restart Pi" }));
    expect(screen.getByRole("status")).toHaveTextContent(
      /health checked on generation 10/i,
    );
    expect(screen.getByText("Process generation 10")).toBeInTheDocument();
  });

  it("binds provisionally, gates retry on conflicts, and cancels only the new attempt", () => {
    render(<RuntimeQaSurface isEnabled />);
    openSurface("Session recovery");

    fireEvent.click(
      screen.getByRole("button", { name: "Submit first valid prompt" }),
    );
    expect(screen.getByText("Bound")).toBeInTheDocument();
    expect(screen.getByText("Immutable for this Chat")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: "Pi 0.80.10" }));
    expect(screen.getAllByText("OpenCode 1.18.3")).toHaveLength(2);

    const retryButton = screen.getByRole("button", {
      name: "Retry immutable turn",
    });
    expect(retryButton).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Reload current state" }),
    );
    expect(retryButton).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Review effect" }));
    expect(retryButton).toBeEnabled();

    fireEvent.click(retryButton);
    expect(screen.getByText("Run Attempt 2")).toBeInTheDocument();
    expect(screen.getByText("Running")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Cancel active attempt" }),
    );

    expect(screen.getByText("Cancelled")).toBeInTheDocument();
    expect(screen.getByText("Interrupted")).toBeInTheDocument();
    expect(screen.getByText("Immutable prompt snapshot")).toBeInTheDocument();
  });
});
