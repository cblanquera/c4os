import {
  act,
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
import { ProviderOnboarding } from "../../../../../src/frontend/features/settings/providers/ProviderOnboarding";

const providerService = vi.hoisted(() => ({
  acceptProviderSessionCredentials: vi.fn(),
  answerProviderApproval: vi.fn(),
  completeProviderOnboarding: vi.fn(),
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

describe("ProviderOnboarding", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    providerService.readProviderSnapshot.mockResolvedValue(emptySnapshot());
    providerService.acceptProviderSessionCredentials.mockResolvedValue({
      ...emptySnapshot(),
      credentialProtection: "session-only",
    });
    providerService.answerProviderApproval.mockResolvedValue(emptySnapshot());
    providerService.saveProviderProfile.mockImplementation(
      async (draft: ProviderProfileDraft) =>
        snapshotWith(providerFromDraft(draft, { state: "untested" })),
    );
    providerService.testProviderConnection.mockResolvedValue(
      transientSnapshotWith(testedProvider()),
    );
    providerService.selectProviderModel.mockResolvedValue(
      snapshotWith(testedProvider({ selectedModelId: "gpt-5" })),
    );
    providerService.completeProviderOnboarding.mockResolvedValue({
      ...snapshotWith(testedProvider({ selectedModelId: "gpt-5" })),
      onboardingCompleted: true,
      modelRoute: "gpt-5",
      defaultRuntime: "opencode",
      defaultEnvironment: "local",
    });
  });

  it("tests transiently and completes once with an automatic model", async () => {
    const onComplete = vi.fn();
    let resolveTest!: (snapshot: ProviderSettingsSnapshot) => void;
    providerService.testProviderConnection.mockReturnValueOnce(
      new Promise<ProviderSettingsSnapshot>((resolve) => {
        resolveTest = resolve;
      }),
    );
    render(<ProviderOnboarding onComplete={onComplete} />);

    const onboardingForm = await screen.findByRole("form", {
      name: "Provider onboarding",
    });
    expect(
      within(onboardingForm).getByRole("button", { name: "Test Connection" }),
    ).toBeVisible();
    expect(
      within(onboardingForm).getByRole("button", { name: "Continue" }),
    ).toBeDisabled();
    expect(within(onboardingForm).getByRole("status")).toHaveTextContent(
      "Connection not tested",
    );

    fireEvent.change(screen.getByRole("textbox", { name: "Profile label" }), {
      target: { value: "OpenAI Work" },
    });
    fireEvent.change(screen.getByLabelText("API key"), {
      target: { value: "raw-secret-value" },
    });
    const testButton = screen.getByRole("button", { name: "Test Connection" });
    act(() => {
      testButton.click();
      testButton.click();
    });

    expect(await screen.findByText("Testing connection")).toBeVisible();
    resolveTest(transientSnapshotWith(testedProvider()));
    expect(await screen.findByText("Connection passed")).toBeVisible();
    expect(
      screen.queryByRole("group", { name: "Model for new Chats" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Initial defaults")).not.toBeInTheDocument();
    expect(screen.queryByText("GPT-5")).not.toBeInTheDocument();
    expect(screen.queryByText("GPT-5 mini")).not.toBeInTheDocument();
    expect(screen.getByLabelText("API key")).toHaveValue("");
    expect(screen.getByLabelText("API key")).toHaveAttribute(
      "placeholder",
      "Leave blank to keep the existing key",
    );
    expect(providerService.saveProviderProfile).not.toHaveBeenCalled();
    expect(providerService.testProviderConnection).toHaveBeenCalledWith(
      expect.objectContaining({
        providerId: "provider:openai-work",
        displayName: "OpenAI Work",
        secret: "raw-secret-value",
      }),
    );
    expect(providerService.testProviderConnection).toHaveBeenCalledTimes(1);

    const continueButton = screen.getByRole("button", { name: "Continue" });
    expect(continueButton).toBeEnabled();
    let resolveCompletion!: (snapshot: ProviderSettingsSnapshot) => void;
    providerService.completeProviderOnboarding.mockReturnValueOnce(
      new Promise<ProviderSettingsSnapshot>((resolve) => {
        resolveCompletion = resolve;
      }),
    );
    act(() => {
      continueButton.click();
      continueButton.click();
    });
    expect(await screen.findByText("Saving provider setup")).toBeVisible();
    await waitFor(() =>
      expect(providerService.completeProviderOnboarding).toHaveBeenCalledWith(
        "provider-test-token",
        "gpt-5",
      ),
    );
    resolveCompletion({
      ...snapshotWith(testedProvider({ selectedModelId: "gpt-5" })),
      onboardingCompleted: true,
      modelRoute: "gpt-5",
      defaultRuntime: "opencode",
      defaultEnvironment: "local",
    });
    await waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    expect(providerService.completeProviderOnboarding).toHaveBeenCalledTimes(1);
    expect(providerService.selectProviderModel).not.toHaveBeenCalled();

    fireEvent.change(screen.getByRole("combobox", { name: "Provider type" }), {
      target: { value: "custom" },
    });
    expect(screen.getByRole("textbox", { name: "API base URL" })).toBeVisible();
    expect(screen.getByText("Connection test no longer current")).toBeVisible();
    expect(
      screen.queryByRole("group", { name: "Model for new Chats" }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
  });

  it("keeps a successful transient test retryable when Continue fails", async () => {
    const onComplete = vi.fn();
    providerService.completeProviderOnboarding.mockRejectedValueOnce(
      new Error("Provider setup could not be saved. Press Continue to retry"),
    );
    render(<ProviderOnboarding onComplete={onComplete} />);

    await enterRequiredProviderFields();
    fireEvent.click(screen.getByRole("button", { name: "Test Connection" }));
    expect(await screen.findByText("Connection passed")).toBeVisible();
    expect(screen.getByLabelText("API key")).toHaveValue("");

    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(await screen.findByText("Provider action failed")).toBeVisible();
    expect(
      screen.getByText(
        "Provider setup could not be saved. Press Continue to retry",
      ),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Continue" })).toBeEnabled();

    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    await waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    expect(providerService.completeProviderOnboarding).toHaveBeenCalledTimes(2);
    expect(providerService.testProviderConnection).toHaveBeenCalledTimes(1);
  });

  it("requires credential re-entry after a session-only restart", async () => {
    providerService.readProviderSnapshot.mockResolvedValueOnce(
      snapshotWith(testedProvider({ hasCredential: false })),
    );
    render(<ProviderOnboarding onComplete={vi.fn()} />);

    expect(await screen.findByText("Connection passed")).toBeVisible();
    expect(screen.getByLabelText("API key")).toHaveValue("");
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
  });

  it("resumes a transient Test Connection after one explicit approval", async () => {
    const pendingTest = {
      ...emptySnapshot(),
      credentialProtection: "session-only" as const,
      pendingApproval: {
        promptId: "provider-test-prompt",
        operation: "test-connection" as const,
        providerId: "provider:openai-work",
        providerName: "OpenAI Work",
        expiresAtMs: 1_784_476_800_000,
      },
    };
    const connected = {
      ...transientSnapshotWith(testedProvider({ generation: 3 })),
      credentialProtection: "session-only" as const,
    };
    providerService.readProviderSnapshot.mockResolvedValueOnce({
      ...emptySnapshot(),
      credentialProtection: "session-only",
    });
    providerService.answerProviderApproval.mockResolvedValueOnce(connected);
    providerService.testProviderConnection.mockResolvedValueOnce(pendingTest);

    render(<ProviderOnboarding onComplete={vi.fn()} />);
    await enterRequiredProviderFields();
    fireEvent.click(screen.getByRole("button", { name: "Test Connection" }));

    expect(
      await screen.findByText(
        /use the OpenAI Work credential and contact the provider/i,
      ),
    ).toBeVisible();
    expect(screen.getByLabelText("API key")).toHaveValue("");
    expect(screen.queryByText("Enter an API key.")).not.toBeInTheDocument();
    expect(providerService.testProviderConnection).toHaveBeenCalledWith(
      expect.objectContaining({
        providerId: "provider:openai-work",
        secret: "test-key",
      }),
    );

    fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
    expect(await screen.findByText("Connection passed")).toBeVisible();
    expect(
      screen.queryByText("Provider approval required"),
    ).not.toBeInTheDocument();
    expect(providerService.answerProviderApproval).toHaveBeenCalledWith(
      "provider-test-prompt",
      "allow",
    );
    expect(providerService.saveProviderProfile).not.toHaveBeenCalled();
    expect(providerService.testProviderConnection).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "Continue" })).toBeEnabled();
  });

  it("validates only compatible-provider fields that remain visible", async () => {
    render(<ProviderOnboarding onComplete={vi.fn()} />);
    fireEvent.change(
      await screen.findByRole("textbox", { name: "Profile label" }),
      { target: { value: "Local Gateway" } },
    );
    fireEvent.change(screen.getByRole("combobox", { name: "Provider type" }), {
      target: { value: "custom" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: "API base URL" }), {
      target: { value: "http://127.0.0.1:4000/v1" },
    });
    fireEvent.change(screen.getByRole("combobox", { name: "Authentication" }), {
      target: { value: "none" },
    });
    expect(screen.queryByLabelText("API key")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Test Connection" }));
    await waitFor(() =>
      expect(providerService.testProviderConnection).toHaveBeenCalled(),
    );
    const testedDraft =
      providerService.testProviderConnection.mock.calls[0]?.[0];
    expect(testedDraft).toEqual(
      expect.objectContaining({ authentication: { type: "none" } }),
    );
    expect(testedDraft).not.toHaveProperty("secret");
    expect(providerService.saveProviderProfile).not.toHaveBeenCalled();
  });

  it("keeps zero-model and failed tests explicit and non-continuable", async () => {
    providerService.testProviderConnection.mockResolvedValueOnce(
      transientSnapshotWith(
        providerRecord({
          models: [],
          testStatus: { state: "succeededNoUsableModels", checkedAtMs: 5 },
        }),
      ),
    );
    const { unmount } = render(<ProviderOnboarding onComplete={vi.fn()} />);
    await enterRequiredProviderFields();
    fireEvent.click(screen.getByRole("button", { name: "Test Connection" }));
    expect(await screen.findByText("No usable models")).toBeVisible();
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
    unmount();

    providerService.readProviderSnapshot.mockResolvedValue(emptySnapshot());
    providerService.testProviderConnection.mockResolvedValueOnce(
      transientSnapshotWith(
        providerRecord({
          models: [],
          testStatus: {
            state: "failed",
            checkedAtMs: 6,
            code: "authentication",
          },
        }),
      ),
    );
    render(<ProviderOnboarding onComplete={vi.fn()} />);
    await enterRequiredProviderFields();
    fireEvent.click(screen.getByRole("button", { name: "Test Connection" }));
    expect(await screen.findByText("Connection failed")).toBeVisible();
    expect(screen.getByText(/Authentication was rejected/)).toBeVisible();
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
  });

  it("requires an explicit session-only choice when macOS storage is unavailable", async () => {
    providerService.readProviderSnapshot.mockResolvedValueOnce({
      ...emptySnapshot(),
      credentialProtection: "session-only",
      credentialFallbackRequired: true,
    });
    providerService.acceptProviderSessionCredentials.mockResolvedValueOnce({
      ...emptySnapshot(),
      credentialProtection: "session-only",
      credentialFallbackRequired: false,
    });
    render(<ProviderOnboarding onComplete={vi.fn()} />);

    expect(
      await screen.findByText("macOS secure credential storage is unavailable"),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Test Connection" }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Use session-only credentials" }),
    );
    expect(
      await screen.findByText("Session-only credentials active"),
    ).toBeVisible();
    expect(providerService.acceptProviderSessionCredentials).toHaveBeenCalled();
  });
});

/** Completes the two required standard-provider inputs in the active render. */
async function enterRequiredProviderFields() {
  fireEvent.change(
    await screen.findByRole("textbox", { name: "Profile label" }),
    { target: { value: "OpenAI Work" } },
  );
  fireEvent.change(screen.getByLabelText("API key"), {
    target: { value: "test-key" },
  });
}

/** Creates the empty first-launch Provider snapshot. */
function emptySnapshot(): ProviderSettingsSnapshot {
  return {
    authority: "rust-provider-service",
    coordinatorGeneration: 1,
    configurationGeneration: 1,
    credentialProtection: "installation-key",
    credentialFallbackRequired: false,
    generation: 1,
    onboardingCompleted: false,
    providers: [],
    modelRoute: null,
    defaultRuntime: null,
    defaultEnvironment: null,
    pendingApproval: null,
    transientTest: null,
  };
}

/** Creates one snapshot around a provider fixture. */
function snapshotWith(provider: ProviderRecord): ProviderSettingsSnapshot {
  return {
    ...emptySnapshot(),
    generation: provider.generation,
    providers: [provider],
  };
}

/** Creates a non-durable connection-test projection around one provider. */
function transientSnapshotWith(
  provider: ProviderRecord,
): ProviderSettingsSnapshot {
  return {
    ...emptySnapshot(),
    generation: provider.generation,
    transientTest: {
      testToken: "provider-test-token",
      provider,
    },
  };
}

/** Converts the exact submitted profile into a safe mock native record. */
function providerFromDraft(
  draft: ProviderProfileDraft,
  testStatus: ProviderRecord["testStatus"],
): ProviderRecord {
  return providerRecord({
    authentication: draft.authentication,
    displayName: draft.displayName,
    enabled: draft.enabled,
    endpoint: draft.endpoint,
    hasCredential: draft.secret !== undefined,
    headers: draft.headers,
    kind: draft.kind,
    providerId: draft.providerId,
    testStatus,
  });
}

/** Creates a successful provider whose most capable model is not listed first. */
function testedProvider(
  overrides: Partial<ProviderRecord> = {},
): ProviderRecord {
  return providerRecord({
    displayName: "OpenAI Work",
    hasCredential: true,
    models: [
      {
        modelId: "gpt-5-mini",
        displayName: "GPT-5 mini",
        recommendationRank: 0,
        availability: "available",
        checkedAtMs: 5,
        lifecycle: "active",
        features: { tools: "supported" },
        contextTokens: 128_000,
        outputTokens: 16_000,
        productionReady: true,
      },
      {
        modelId: "gpt-5",
        displayName: "GPT-5",
        recommendationRank: 1,
        availability: "available",
        checkedAtMs: 5,
        lifecycle: "active",
        features: {
          reasoning: "supported",
          tools: "supported",
          vision: "supported",
        },
        contextTokens: 400_000,
        outputTokens: 32_000,
        productionReady: true,
      },
    ],
    testStatus: { state: "succeeded", checkedAtMs: 5 },
    ...overrides,
  });
}

/** Creates a complete provider record with overridable visible state. */
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
    testStatus: { state: "untested" },
    models: [],
    selectedModelId: null,
    generation: 2,
    ...overrides,
  };
}
