import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
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
    providerService.saveProviderProfile.mockImplementation(
      async (draft: ProviderProfileDraft) =>
        snapshotWith(providerFromDraft(draft, { state: "untested" })),
    );
    providerService.testProviderConnection.mockResolvedValue(
      snapshotWith(testedProvider()),
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

  it("clears the raw key after save and gates Continue on fresh model evidence", async () => {
    const onComplete = vi.fn();
    let resolveTest!: (snapshot: ProviderSettingsSnapshot) => void;
    providerService.testProviderConnection.mockReturnValueOnce(
      new Promise<ProviderSettingsSnapshot>((resolve) => {
        resolveTest = resolve;
      }),
    );
    render(<ProviderOnboarding onComplete={onComplete} />);

    fireEvent.change(
      await screen.findByRole("textbox", { name: "Profile label" }),
      { target: { value: "OpenAI Work" } },
    );
    fireEvent.change(screen.getByLabelText("API key"), {
      target: { value: "raw-secret-value" },
    });
    const testButton = screen.getByRole("button", { name: "Test Connection" });
    act(() => {
      testButton.click();
      testButton.click();
    });

    expect(await screen.findByText("Testing connection")).toBeVisible();
    resolveTest(snapshotWith(testedProvider()));
    expect(await screen.findByText("Connection passed")).toBeVisible();
    expect(screen.getByText("Recommended")).toBeVisible();
    expect(screen.getByText("OpenCode")).toBeVisible();
    expect(screen.getByText("Local")).toBeVisible();
    expect(screen.getByLabelText("API key")).toHaveValue("");
    expect(screen.getByLabelText("API key")).toHaveAttribute(
      "placeholder",
      "Leave blank to keep the existing key",
    );
    expect(providerService.saveProviderProfile).toHaveBeenCalledWith(
      expect.objectContaining({ secret: "raw-secret-value" }),
    );
    expect(providerService.saveProviderProfile).toHaveBeenCalledTimes(1);
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
        "provider:openai-work",
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

    fireEvent.change(screen.getByRole("combobox", { name: "Provider type" }), {
      target: { value: "custom" },
    });
    expect(screen.getByRole("textbox", { name: "API base URL" })).toBeVisible();
    expect(screen.getByText("Connection test no longer current")).toBeVisible();
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
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
      expect(providerService.saveProviderProfile).toHaveBeenCalled(),
    );
    const savedDraft = providerService.saveProviderProfile.mock.calls[0]?.[0];
    expect(savedDraft).toEqual(
      expect.objectContaining({ authentication: { type: "none" } }),
    );
    expect(savedDraft).not.toHaveProperty("secret");
  });

  it("keeps zero-model and failed tests explicit and non-continuable", async () => {
    providerService.testProviderConnection.mockResolvedValueOnce(
      snapshotWith(
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
      snapshotWith(
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

/** Creates the successful provider used by the default-confirmation flow. */
function testedProvider(
  overrides: Partial<ProviderRecord> = {},
): ProviderRecord {
  return providerRecord({
    displayName: "OpenAI Work",
    hasCredential: true,
    models: [
      {
        modelId: "gpt-5",
        displayName: "GPT-5",
        recommendationRank: 0,
        availability: "available",
        checkedAtMs: 5,
        lifecycle: "active",
        features: { tools: "supported" },
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
