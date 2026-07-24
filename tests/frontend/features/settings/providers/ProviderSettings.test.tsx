import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  ProviderProfileDraft,
  ProviderRecord,
  ProviderSettingsSnapshot,
} from "../../../../../src/frontend/platform/provider-service";
import { ProviderSettings } from "../../../../../src/frontend/features/settings/providers/ProviderSettings";

const providerService = vi.hoisted(() => ({
  acceptProviderSessionCredentials: vi.fn(),
  answerProviderApproval: vi.fn(),
  deleteProviderProfile: vi.fn(),
  readProviderSnapshot: vi.fn(),
  saveProviderProfile: vi.fn(),
  selectProviderModel: vi.fn(),
  testProviderConnection: vi.fn(),
}));

vi.mock("../../../../../src/frontend/platform/provider-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../../../../src/frontend/platform/provider-service")
  >("../../../../../src/frontend/platform/provider-service");
  return { ...actual, ...providerService };
});

describe("ProviderSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    const snapshot = providerSnapshot();
    providerService.acceptProviderSessionCredentials.mockResolvedValue({
      ...snapshot,
      credentialProtection: "session-only",
      credentialFallbackRequired: false,
    });
    providerService.answerProviderApproval.mockResolvedValue(snapshot);
    providerService.readProviderSnapshot.mockResolvedValue(snapshot);
    providerService.saveProviderProfile.mockImplementation(
      async (draft: ProviderProfileDraft) =>
        providerSnapshot({
          ...snapshot.providers[0]!,
          authentication: draft.authentication,
          displayName: draft.displayName,
          enabled: draft.enabled,
          endpoint: draft.endpoint,
          headers: draft.headers,
          kind: draft.kind,
          providerId: draft.providerId,
        }),
    );
    providerService.testProviderConnection.mockResolvedValue(snapshot);
    providerService.selectProviderModel.mockImplementation(
      async (_providerId: string, modelId: string) =>
        providerSnapshot({
          ...snapshot.providers[0]!,
          selectedModelId: modelId,
        }),
    );
    providerService.deleteProviderProfile.mockResolvedValue({
      ...snapshot,
      generation: 4,
      providers: [],
    });
  });

  it("requires an explicit answer for a native Provider approval", async () => {
    const snapshot = providerSnapshot();
    providerService.readProviderSnapshot.mockResolvedValue({
      ...snapshot,
      pendingApproval: {
        promptId: "provider-prompt-1",
        operation: "test-connection",
        providerId: "provider:openai-work",
        providerName: "OpenAI Work",
        expiresAtMs: 1_784_476_800_000,
      },
    });
    providerService.answerProviderApproval.mockResolvedValue(snapshot);

    render(<ProviderSettings />);

    expect(await screen.findByText("Provider approval required")).toBeVisible();
    expect(screen.getByRole("status")).toHaveTextContent(
      /use the stored credential and contact OpenAI Work/i,
    );
    expect(
      screen.getByRole("button", { name: "Test OpenAI Work" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
    await waitFor(() =>
      expect(providerService.answerProviderApproval).toHaveBeenCalledWith(
        "provider-prompt-1",
        "allow",
      ),
    );
    await waitFor(() =>
      expect(
        screen.queryByText("Provider approval required"),
      ).not.toBeInTheDocument(),
    );
  });

  it("continues Save & Test through separate save and connection approvals", async () => {
    const saved = providerSnapshot(
      providerRecord({ testStatus: { state: "untested" }, models: [] }),
    );
    const saveApproval = {
      ...saved,
      pendingApproval: {
        promptId: "provider-save-prompt",
        operation: "save-profile" as const,
        providerId: "provider:openai-work",
        providerName: "OpenAI Work",
        expiresAtMs: 1_784_476_800_000,
      },
    };
    const testApproval = {
      ...saved,
      pendingApproval: {
        promptId: "provider-test-prompt",
        operation: "test-connection" as const,
        providerId: "provider:openai-work",
        providerName: "OpenAI Work",
        expiresAtMs: 1_784_476_800_000,
      },
    };
    const connected = providerSnapshot();
    providerService.saveProviderProfile.mockResolvedValueOnce(saveApproval);
    providerService.answerProviderApproval
      .mockResolvedValueOnce(saved)
      .mockResolvedValueOnce(connected);
    providerService.testProviderConnection.mockResolvedValueOnce(testApproval);

    render(<ProviderSettings />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Edit OpenAI Work" }),
    );
    const dialog = await screen.findByRole("dialog", {
      name: "Edit OpenAI Work",
    });
    fireEvent.change(within(dialog).getByLabelText("API key"), {
      target: { value: "replacement-secret" },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Save & Test Connection" }),
    );

    expect(
      await screen.findByText(/change the securely stored credential/i),
    ).toBeVisible();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Deny" })).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
    await waitFor(() =>
      expect(providerService.testProviderConnection).toHaveBeenCalledWith(
        "provider:openai-work",
      ),
    );
    expect(
      await screen.findByText(/contact OpenAI Work for this connection test/i),
    ).toBeVisible();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Deny" })).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
    await waitFor(() =>
      expect(providerService.answerProviderApproval).toHaveBeenLastCalledWith(
        "provider-test-prompt",
        "allow",
      ),
    );
    expect(await screen.findByText("Connection passed")).toBeVisible();
    expect(screen.getByText("2 production-ready models")).toBeVisible();
    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Provider profiles" }),
      ).toHaveFocus(),
    );
  });

  it("supports enablement, testing, model selection, editing, and deletion", async () => {
    render(<ProviderSettings />);
    expect(
      await screen.findByRole("heading", { name: "OpenAI Work" }),
    ).toBeVisible();

    fireEvent.click(
      screen.getByRole("switch", { name: /OpenAI Work availability/ }),
    );
    await waitFor(() =>
      expect(providerService.saveProviderProfile).toHaveBeenCalled(),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("switch", { name: /OpenAI Work availability/ }),
      ).toBeEnabled(),
    );
    const toggleDraft = providerService.saveProviderProfile.mock.calls[0]?.[0];
    expect(toggleDraft).toEqual(expect.objectContaining({ enabled: false }));
    expect(toggleDraft).not.toHaveProperty("secret");

    fireEvent.click(screen.getByRole("button", { name: "Test OpenAI Work" }));
    await waitFor(() =>
      expect(providerService.testProviderConnection).toHaveBeenCalledWith(
        "provider:openai-work",
      ),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Test OpenAI Work" }),
      ).toBeEnabled(),
    );

    fireEvent.change(
      screen.getByRole("combobox", {
        name: "OpenAI Work selected model",
      }),
      {
        target: { value: "gpt-4.1" },
      },
    );
    await waitFor(() =>
      expect(providerService.selectProviderModel).toHaveBeenCalledWith(
        "provider:openai-work",
        "gpt-4.1",
      ),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Edit OpenAI Work" }),
      ).toBeEnabled(),
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit OpenAI Work" }));
    const editDialog = await screen.findByRole("dialog", {
      name: "Edit OpenAI Work",
    });
    expect(within(editDialog).getByLabelText("API key")).toHaveValue("");
    expect(within(editDialog).getByLabelText("API key")).toHaveAttribute(
      "placeholder",
      "Leave blank to keep the existing key",
    );
    fireEvent.click(
      within(editDialog).getByRole("button", { name: "Save Profile" }),
    );
    await waitFor(() =>
      expect(providerService.saveProviderProfile).toHaveBeenCalledTimes(2),
    );
    const editDraft = providerService.saveProviderProfile.mock.calls[1]?.[0];
    expect(editDraft).toEqual(
      expect.objectContaining({ providerId: "provider:openai-work" }),
    );
    expect(editDraft).not.toHaveProperty("secret");
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Edit OpenAI Work" }),
      ).toBeNull(),
    );

    fireEvent.click(screen.getByRole("button", { name: "Delete OpenAI Work" }));
    const deleteDialog = await screen.findByRole("dialog", {
      name: "Delete OpenAI Work?",
    });
    fireEvent.click(
      within(deleteDialog).getByRole("button", { name: "Delete Provider" }),
    );
    await waitFor(() =>
      expect(providerService.deleteProviderProfile).toHaveBeenCalledWith(
        "provider:openai-work",
      ),
    );
  });

  it("serializes provider mutations and qualifies every repeated row action", async () => {
    let resolveTest!: (snapshot: ProviderSettingsSnapshot) => void;
    providerService.testProviderConnection.mockReturnValueOnce(
      new Promise<ProviderSettingsSnapshot>((resolve) => {
        resolveTest = resolve;
      }),
    );
    render(<ProviderSettings />);

    const testButton = await screen.findByRole("button", {
      name: "Test OpenAI Work",
    });
    const switchControl = screen.getByRole("switch", {
      name: "OpenAI Work availability",
    });
    const modelSelect = screen.getByRole("combobox", {
      name: "OpenAI Work selected model",
    });
    fireEvent.click(testButton);

    await waitFor(() => {
      expect(testButton).toBeDisabled();
      expect(switchControl).toBeDisabled();
      expect(modelSelect).toBeDisabled();
      expect(
        screen.getByRole("button", { name: "Add Provider" }),
      ).toBeDisabled();
      expect(
        screen.getByRole("button", { name: "Edit OpenAI Work" }),
      ).toBeDisabled();
      expect(
        screen.getByRole("button", { name: "Delete OpenAI Work" }),
      ).toBeDisabled();
    });
    expect(providerService.testProviderConnection).toHaveBeenCalledTimes(1);

    resolveTest(providerSnapshot());
    await waitFor(() => {
      expect(testButton).toBeEnabled();
      expect(switchControl).toBeEnabled();
      expect(modelSelect).toBeEnabled();
    });
  });

  it("shares conditional compatible fields and visible validation in Add Provider", async () => {
    const snapshot = { ...providerSnapshot(), providers: [] };
    providerService.readProviderSnapshot.mockResolvedValueOnce(snapshot);
    providerService.saveProviderProfile.mockImplementationOnce(
      async (draft: ProviderProfileDraft) =>
        providerSnapshot(providerFromDraft(draft)),
    );
    render(<ProviderSettings />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Add Provider" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Add Provider" });
    fireEvent.change(
      within(dialog).getByRole("combobox", { name: "Provider type" }),
      {
        target: { value: "custom" },
      },
    );
    fireEvent.change(
      within(dialog).getByRole("textbox", { name: "Profile label" }),
      {
        target: { value: "Local Gateway" },
      },
    );
    fireEvent.change(
      within(dialog).getByRole("textbox", { name: "API base URL" }),
      {
        target: { value: "http://127.0.0.1:4000/v1" },
      },
    );
    fireEvent.change(
      within(dialog).getByRole("combobox", { name: "Authentication" }),
      {
        target: { value: "apiKeyHeader" },
      },
    );
    fireEvent.change(
      within(dialog).getByRole("textbox", { name: "API key header name" }),
      { target: { value: "bad header" } },
    );
    fireEvent.change(within(dialog).getByLabelText("API key"), {
      target: { value: "new-secret" },
    });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Save Profile" }),
    );
    expect(
      await within(dialog).findByText("Enter a valid API key header name."),
    ).toBeVisible();
    expect(providerService.saveProviderProfile).not.toHaveBeenCalled();

    fireEvent.change(
      within(dialog).getByRole("textbox", { name: "API key header name" }),
      { target: { value: "X-Local-Key" } },
    );
    fireEvent.change(
      within(dialog).getByRole("textbox", {
        name: "Additional literal headers (JSON)",
      }),
      { target: { value: '{"X-Client":"c4os"}' } },
    );
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Save Profile" }),
    );
    await waitFor(() =>
      expect(providerService.saveProviderProfile).toHaveBeenCalledWith(
        expect.objectContaining({
          authentication: {
            type: "apiKeyHeader",
            headerName: "X-Local-Key",
          },
          headers: { "X-Client": "c4os" },
          secret: "new-secret",
        }),
      ),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Add Provider" })).toBeNull(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add Provider" }));
    const freshDialog = await screen.findByRole("dialog", {
      name: "Add Provider",
    });
    expect(
      within(freshDialog).getByRole("textbox", { name: "Profile label" }),
    ).toHaveValue("");
  });

  it("makes Add Provider test persistence explicit before the dialog closes", async () => {
    providerService.readProviderSnapshot.mockResolvedValueOnce({
      ...providerSnapshot(),
      providers: [],
    });
    render(<ProviderSettings />);
    fireEvent.click(
      await screen.findByRole("button", { name: "Add Provider" }),
    );
    const dialog = await screen.findByRole("dialog", { name: "Add Provider" });
    fireEvent.change(
      within(dialog).getByRole("textbox", { name: "Profile label" }),
      { target: { value: "OpenAI Work" } },
    );
    fireEvent.change(within(dialog).getByLabelText("API key"), {
      target: { value: "new-secret" },
    });

    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Save & Test Connection",
      }),
    );
    await waitFor(() =>
      expect(providerService.testProviderConnection).toHaveBeenCalledWith(
        "provider:openai-work",
      ),
    );
    expect(within(dialog).getByRole("button", { name: "Close" })).toBeVisible();
    expect(within(dialog).queryByRole("button", { name: "Cancel" })).toBeNull();
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Add Provider" })).toBeNull(),
    );
    expect(providerService.deleteProviderProfile).not.toHaveBeenCalled();
  });

  it("blocks credential-backed tests until session-only storage is accepted", async () => {
    providerService.readProviderSnapshot.mockResolvedValueOnce({
      ...providerSnapshot(),
      credentialProtection: "session-only",
      credentialFallbackRequired: true,
    });
    render(<ProviderSettings />);

    const testButton = await screen.findByRole("button", {
      name: "Test OpenAI Work",
    });
    expect(testButton).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Use session-only credentials" }),
    );
    await waitFor(() => expect(testButton).toBeEnabled());
    expect(providerService.acceptProviderSessionCredentials).toHaveBeenCalled();
  });

  it("requires credential re-entry when a session-only reference is unavailable", async () => {
    providerService.readProviderSnapshot.mockResolvedValueOnce(
      providerSnapshot(providerRecord({ hasCredential: false })),
    );
    render(<ProviderSettings />);

    expect(
      await screen.findByRole("button", { name: "Test OpenAI Work" }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Edit OpenAI Work" }));
    const editDialog = await screen.findByRole("dialog", {
      name: "Edit OpenAI Work",
    });
    expect(within(editDialog).getByLabelText("API key")).toHaveAttribute(
      "placeholder",
      "Enter API key",
    );
  });
});

/** Creates a complete Provider Settings snapshot with one default record. */
function providerSnapshot(
  provider: ProviderRecord = providerRecord(),
): ProviderSettingsSnapshot {
  return {
    authority: "rust-provider-service",
    coordinatorGeneration: 3,
    configurationGeneration: 3,
    credentialProtection: "installation-key",
    credentialFallbackRequired: false,
    generation: 3,
    onboardingCompleted: true,
    providers: [provider],
    modelRoute: provider.selectedModelId,
    defaultRuntime: "opencode",
    defaultEnvironment: "local",
    pendingApproval: null,
  };
}

/** Converts an Add Provider submission into a mock native record. */
function providerFromDraft(draft: ProviderProfileDraft): ProviderRecord {
  return providerRecord({
    authentication: draft.authentication,
    displayName: draft.displayName,
    enabled: draft.enabled,
    endpoint: draft.endpoint,
    hasCredential: draft.secret !== undefined,
    headers: draft.headers,
    kind: draft.kind,
    providerId: draft.providerId,
    testStatus: { state: "untested" },
  });
}

/** Creates a tested provider with two production-ready model choices. */
function providerRecord(
  overrides: Partial<ProviderRecord> = {},
): ProviderRecord {
  return {
    providerId: "provider:openai-work",
    kind: "open-ai",
    displayName: "OpenAI Work",
    endpoint: {
      endpointId: "provider:openai-work:primary",
      baseUrl: "https://api.openai.com/v1",
      apiKind: "openai",
    },
    authentication: { type: "bearer" },
    headers: {},
    hasCredential: true,
    enabled: true,
    disabledModelIds: [],
    testStatus: { state: "succeeded", checkedAtMs: 3 },
    models: [
      {
        modelId: "gpt-5",
        displayName: "GPT-5",
        recommendationRank: 0,
        availability: "available",
        checkedAtMs: 3,
        lifecycle: "active",
        features: { tools: "supported" },
        contextTokens: 400_000,
        outputTokens: 32_000,
        productionReady: true,
      },
      {
        modelId: "gpt-4.1",
        displayName: "GPT-4.1",
        recommendationRank: 1,
        availability: "available",
        checkedAtMs: 3,
        lifecycle: "active",
        features: { vision: "supported" },
        contextTokens: 1_000_000,
        outputTokens: 32_000,
        productionReady: true,
      },
    ],
    selectedModelId: "gpt-5",
    generation: 3,
    ...overrides,
  };
}
