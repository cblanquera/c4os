import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ModelSettings } from "../../../../../src/frontend/features/settings/runtime/ModelSettings";
import type {
  ModelCapabilityView,
  ModelRouteView,
  ModelSettingsActions,
  ModelSettingsSnapshot,
} from "../../../../../src/frontend/features/settings/runtime/types";

function actions(): ModelSettingsActions {
  return {
    onRefresh: vi.fn(),
    onRetry: vi.fn(),
    onSetEnabled: vi.fn(),
    onSetVisibleEnabled: vi.fn(),
  };
}

const capabilities: readonly ModelCapabilityView[] = [
  {
    evidence: [
      {
        checkedAt: "2026-07-23T04:00:00Z",
        detail: "Image input passed the exact route probe.",
        source: "Observed provider probe",
      },
    ],
    key: "vision",
    state: "supported",
    summary: "PNG and JPEG inputs are supported.",
  },
  {
    evidence: [
      {
        checkedAt: "2026-07-23T04:00:01Z",
        detail: "Structured tool invocation was normalized by OpenCode.",
        source: "Adapter evidence",
      },
    ],
    key: "tools",
    state: "supported",
    summary: "Structured tools are available.",
  },
  {
    evidence: [],
    key: "reasoning",
    state: "degraded",
    summary: "Reasoning summaries are available without every effort level.",
  },
  {
    evidence: [],
    key: "audio",
    state: "unknown",
    summary: "The provider supplied no audio evidence.",
  },
];

const openAi: ModelRouteView = {
  available: true,
  availabilityDetail: "Available from the latest successful provider test.",
  capabilities,
  contextTokens: 400_000,
  enabled: true,
  id: "route-openai-gpt5-mini",
  modelId: "openai/gpt-5-mini",
  modelName: "GPT-5 Mini",
  providerId: "provider-openai",
  providerName: "OpenAI",
  revision: "2026-06-30",
  runtimeLabel: "OpenCode 1.18.3",
};

const openRouter: ModelRouteView = {
  available: true,
  availabilityDetail: "Available from the latest successful provider test.",
  capabilities: capabilities.map((capability) =>
    capability.key === "vision"
      ? { ...capability, evidence: [], state: "unsupported" as const }
      : capability.key === "reasoning"
        ? { ...capability, evidence: [], state: "unknown" as const }
        : capability,
  ),
  contextTokens: 128_000,
  enabled: false,
  id: "route-openrouter-kimi",
  modelId: "moonshotai/kimi-k2",
  modelName: "Kimi K2",
  providerId: "provider-openrouter",
  providerName: "OpenRouter",
  revision: "2026-07-01",
  runtimeLabel: "OpenCode 1.18.3",
};

const unavailable: ModelRouteView = {
  ...openAi,
  available: false,
  availabilityDetail: "The provider is disabled.",
  enabled: false,
  id: "route-disabled",
  modelId: "openai/gpt-disabled",
  modelName: "Disabled Route",
};

function ready(
  models: readonly ModelRouteView[] = [openAi, openRouter, unavailable],
  overrides: Partial<Extract<ModelSettingsSnapshot, { status: "ready" }>> = {},
): Extract<ModelSettingsSnapshot, { status: "ready" }> {
  return {
    generation: 7,
    models,
    refresh: { message: "Capability evidence is current.", status: "idle" },
    status: "ready",
    writesDisabled: false,
    ...overrides,
  };
}

describe("ModelSettings", () => {
  it("keeps loading, error, retry, and empty states distinct", () => {
    const handlers = actions();
    const { rerender } = render(
      <ModelSettings
        actions={handlers}
        snapshot={{ message: "Reading model routes.", status: "loading" }}
      />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading models");

    rerender(
      <ModelSettings
        actions={handlers}
        snapshot={{
          message: "The capability snapshot could not be read.",
          retryable: true,
          status: "error",
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Models are unavailable",
    );
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(handlers.onRetry).toHaveBeenCalledTimes(1);

    rerender(<ModelSettings actions={handlers} snapshot={ready([])} />);
    expect(
      screen.getByRole("heading", { name: "No models discovered" }),
    ).toBeInTheDocument();
  });

  it("filters authoritative routes and applies bulk changes only to visible ids", () => {
    const handlers = actions();
    render(<ModelSettings actions={handlers} snapshot={ready()} />);

    fireEvent.change(screen.getByLabelText("Search models"), {
      target: { value: "kimi" },
    });
    expect(screen.getByText("1 visible model route")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: "Enable visible models" }),
    );
    expect(handlers.onSetVisibleEnabled).toHaveBeenLastCalledWith(
      ["route-openrouter-kimi"],
      true,
    );

    fireEvent.change(screen.getByLabelText("Search models"), {
      target: { value: "" },
    });
    fireEvent.change(screen.getByLabelText("Filter models by provider"), {
      target: { value: "provider-openai" },
    });
    fireEvent.change(screen.getByLabelText("Filter models by capability"), {
      target: { value: "reasoning" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Disable visible models" }),
    );
    expect(handlers.onSetVisibleEnabled).toHaveBeenLastCalledWith(
      ["route-openai-gpt5-mini"],
      false,
    );
  });

  it("opens route evidence and delegates availability changes", async () => {
    const handlers = actions();
    render(<ModelSettings actions={handlers} snapshot={ready()} />);

    const toggle = screen.getByRole("switch", {
      name: /OpenAI GPT-5 Mini availability/u,
    });
    expect(toggle).toBeChecked();
    fireEvent.click(toggle);
    expect(handlers.onSetEnabled).toHaveBeenCalledWith(
      "route-openai-gpt5-mini",
      false,
    );
    expect(
      screen.getByRole("switch", {
        name: /OpenAI Disabled Route availability/u,
      }),
    ).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "GPT-5 Mini" }));
    const dialog = await screen.findByRole("dialog", {
      name: "OpenAI / GPT-5 Mini",
    });
    expect(within(dialog).getByText("openai/gpt-5-mini")).toBeInTheDocument();
    expect(within(dialog).getByText("400K")).toBeInTheDocument();
    expect(
      within(dialog).getByText("Observed provider probe"),
    ).toBeInTheDocument();
    expect(within(dialog).getByText("Unknown")).toBeInTheDocument();
  });

  it("renders refresh pending, success, failure, and operation errors", () => {
    const handlers = actions();
    const { rerender } = render(
      <ModelSettings actions={handlers} snapshot={ready()} />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    expect(handlers.onRefresh).toHaveBeenCalledTimes(1);

    rerender(
      <ModelSettings
        actions={handlers}
        snapshot={ready(undefined, {
          refresh: {
            message: "Discovering current routes.",
            status: "pending",
          },
          writesDisabled: true,
        })}
      />,
    );
    expect(screen.getByRole("button", { name: "Refreshing…" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Disable visible models" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("switch", { name: /OpenAI GPT-5 Mini availability/u }),
    ).toBeDisabled();
    expect(screen.getByText("Discovering current routes.")).toBeInTheDocument();

    rerender(
      <ModelSettings
        actions={handlers}
        snapshot={ready(undefined, {
          refresh: { message: "Three routes refreshed.", status: "success" },
        })}
      />,
    );
    expect(screen.getByText("Models refreshed")).toBeInTheDocument();

    rerender(
      <ModelSettings
        actions={handlers}
        snapshot={ready(undefined, {
          operationError: "The model generation changed.",
          refresh: { message: "Provider test expired.", status: "error" },
        })}
      />,
    );
    expect(screen.getByText("Model refresh failed")).toBeInTheDocument();
    expect(
      screen.getByText("Model change was not applied"),
    ).toBeInTheDocument();
  });
});
