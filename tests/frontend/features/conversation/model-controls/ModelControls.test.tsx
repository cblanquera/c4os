import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  ApprovalPresetControl,
  ChatInformationPopover,
  ModelSelector,
  ReasoningEffortControl,
} from "../../../../../src/frontend/features/conversation/model-controls/index";
import type {
  ModelControlModel,
  ModelControlProvider,
} from "../../../../../src/frontend/features/conversation/model-controls/types";

describe("ApprovalPresetControl", () => {
  it("offers all accepted presets and restores focus after selection", async () => {
    const onChange = vi.fn();
    render(<ApprovalPresetControl onChange={onChange} value="ask" />);

    const trigger = screen.getByRole("button", { name: "Approval preset" });
    expect(trigger).toHaveTextContent("Ask");
    fireEvent.click(trigger);

    const menu = await screen.findByRole("menu");
    expect(
      within(menu)
        .getAllByRole("menuitemradio")
        .map((option) => option.textContent?.trim()),
    ).toEqual([
      "Ask for approval",
      "Approve safe actions",
      "Approve for me",
      "Custom",
    ]);
    fireEvent.click(
      within(menu).getByRole("menuitemradio", {
        name: "Approve safe actions",
      }),
    );
    expect(onChange).toHaveBeenCalledWith("approve-safe");
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(trigger).toHaveFocus();
  });
});

const PROVIDERS: readonly ModelControlProvider[] = [
  { id: "openai", name: "OpenAI" },
  { id: "anthropic", name: "Anthropic" },
  { id: "local", name: "Local" },
];

const MODELS: readonly ModelControlModel[] = [
  {
    capabilities: ["vision", "tools", "reasoning"],
    contextLabel: "400K",
    id: "gpt-5",
    isAvailable: true,
    isSelected: true,
    name: "GPT-5",
    providerId: "openai",
  },
  {
    capabilities: ["vision", "tools"],
    contextLabel: "128K",
    id: "gpt-5-fast",
    isAvailable: true,
    isSelected: false,
    name: "GPT-5 fast",
    providerId: "openai",
  },
  {
    capabilities: ["vision", "tools", "reasoning"],
    contextLabel: "200K",
    id: "claude-opus-4-1",
    isAvailable: true,
    isSelected: false,
    name: "Claude Opus 4.1",
    providerId: "anthropic",
  },
  {
    capabilities: [],
    contextLabel: "32K",
    id: "local-text",
    isAvailable: true,
    isSelected: false,
    name: "Local text",
    providerId: "local",
  },
];

describe("ModelSelector", () => {
  it("opens on the active provider with summaries and capability filters", async () => {
    render(
      <ModelSelector
        models={MODELS}
        onModelSelect={vi.fn()}
        providers={PROVIDERS}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Model" }));

    const list = await screen.findByRole("listbox", {
      name: "Models from OpenAI",
    });
    expect(screen.getByRole("button", { name: "OpenAI" })).toBeVisible();
    expect(
      within(list).getByRole("option", {
        name: "GPT-5, Vision · Tools · Reasoning · 400K",
      }),
    ).toHaveAttribute("aria-selected", "true");
    expect(list).toHaveTextContent("Reasoning · 400K");
    expect(
      screen
        .getAllByRole("button", { pressed: false })
        .map((button) => button.textContent?.trim()),
    ).toEqual(["Vision", "Tools", "Reasoning", "Audio"]);
  });

  it("browses providers without changing the selected model until a model is chosen", async () => {
    const onModelSelect = vi.fn();
    render(
      <ModelSelector
        models={MODELS}
        onModelSelect={onModelSelect}
        providers={PROVIDERS}
      />,
    );

    const trigger = screen.getByRole("button", { name: "Model" });
    expect(trigger).toHaveTextContent("GPT-5");
    fireEvent.click(trigger);
    fireEvent.click(await screen.findByRole("button", { name: "OpenAI" }));

    const providers = screen.getByRole("listbox", {
      name: "Configured providers",
    });
    expect(
      within(providers).getByRole("option", {
        name: "OpenAI, active provider",
      }),
    ).toHaveAttribute("aria-selected", "true");
    fireEvent.click(
      within(providers).getByRole("option", { name: "Anthropic" }),
    );

    expect(onModelSelect).not.toHaveBeenCalled();
    expect(trigger).toHaveTextContent("GPT-5");
    const models = await screen.findByRole("listbox", {
      name: "Models from Anthropic",
    });
    fireEvent.click(
      within(models).getByRole("option", { name: /Claude Opus 4.1/ }),
    );
    expect(onModelSelect).toHaveBeenCalledWith(MODELS[2]);
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(trigger).toHaveFocus();
  });

  it("filters models, announces empty results, and restores the active provider after Escape", async () => {
    render(
      <ModelSelector
        models={MODELS}
        onModelSelect={vi.fn()}
        providers={PROVIDERS}
      />,
    );

    const trigger = screen.getByRole("button", { name: "Model" });
    fireEvent.click(trigger);
    fireEvent.click(await screen.findByRole("button", { name: "Audio" }));
    expect(screen.getByRole("status")).toHaveTextContent(
      "No Audio models from OpenAI.",
    );

    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(trigger).toHaveFocus();

    fireEvent.click(trigger);
    expect(
      await screen.findByRole("listbox", { name: "Models from OpenAI" }),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "All" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });
});

describe("ReasoningEffortControl", () => {
  it("renders only supplied accepted options and reports controlled changes", () => {
    const onChange = vi.fn();
    const { rerender } = render(
      <ReasoningEffortControl
        onChange={onChange}
        options={["off", "medium", "high"]}
        value="medium"
      />,
    );

    const select = screen.getByRole("combobox", { name: "Reasoning effort" });
    expect(
      within(select)
        .getAllByRole("option")
        .map((option) => option.textContent),
    ).toEqual(["Off", "Medium", "High"]);
    expect(select).toHaveValue("medium");
    fireEvent.change(select, { target: { value: "high" } });
    expect(onChange).toHaveBeenCalledWith("high");

    rerender(
      <ReasoningEffortControl onChange={onChange} options={[]} value={null} />,
    );
    expect(
      screen.queryByRole("combobox", { name: "Reasoning effort" }),
    ).toBeNull();
  });
});

describe("ChatInformationPopover", () => {
  it("shows route fields and semantic context usage, then restores focus on Escape", async () => {
    render(
      <ChatInformationPopover
        information={{
          contextUsage: { totalTokens: 400_000, usedTokens: 114_000 },
          environment: "Local",
          health: "Healthy",
          model: "GPT-5",
          runtime: "OpenCode 1.18.3",
          workspace: "~/c4os",
        }}
      />,
    );

    const trigger = screen.getByRole("button", { name: "Chat information" });
    fireEvent.click(trigger);
    const dialog = await screen.findByRole("dialog", {
      name: "Chat information",
    });
    expect(dialog).toHaveTextContent("Healthy");
    expect(dialog).toHaveTextContent("OpenCode 1.18.3");
    expect(dialog).toHaveTextContent("Local");
    expect(dialog).toHaveTextContent("~/c4os");
    expect(dialog).toHaveTextContent("GPT-5");
    expect(dialog).toHaveTextContent("29% used · 71% left");
    expect(dialog).toHaveTextContent("114K / 400K tokens");
    expect(
      within(dialog).getByRole("progressbar", { name: "Context window used" }),
    ).toHaveAttribute(
      "aria-valuetext",
      "114K of 400K tokens used; 286K tokens remaining",
    );

    fireEvent.keyDown(dialog, { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    await waitFor(() => expect(trigger).toHaveFocus());
  });
});
