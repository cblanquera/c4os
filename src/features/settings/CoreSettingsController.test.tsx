import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  ProviderPendingApproval,
  ProviderRecord,
  ProviderSettingsSnapshot,
} from "../../platform/provider-service";
import {
  POLICY_SETTING_KEYS,
  type PolicySettingKey,
} from "../../platform/policy-service";
import type { RuntimeCoreSnapshot } from "../../platform/runtime-core";
import { ProtocolBoundaryError } from "../../platform/tauri-adapter";
import { CoreSettingsController } from "./CoreSettingsController";

const policyMocks = vi.hoisted(() => ({
  readPolicySettings: vi.fn(),
  revokePolicyException: vi.fn(),
  savePolicySettings: vi.fn(),
}));

const providerMocks = vi.hoisted(() => ({
  answerProviderApproval: vi.fn(),
  readProviderSnapshot: vi.fn(),
  testProviderConnection: vi.fn(),
}));

const runtimeMocks = vi.hoisted(() => ({
  readRuntimeCoreSnapshot: vi.fn(),
}));

vi.mock("../../platform/policy-service", async (importOriginal) => {
  const original =
    await importOriginal<typeof import("../../platform/policy-service")>();
  return { ...original, ...policyMocks };
});

vi.mock("../../platform/provider-service", async (importOriginal) => {
  const original =
    await importOriginal<typeof import("../../platform/provider-service")>();
  return { ...original, ...providerMocks };
});

vi.mock("../../platform/runtime-core", async (importOriginal) => {
  const original =
    await importOriginal<typeof import("../../platform/runtime-core")>();
  return { ...original, ...runtimeMocks };
});

function categories(): Record<PolicySettingKey, null> {
  return Object.fromEntries(
    POLICY_SETTING_KEYS.map((key) => [key, null]),
  ) as Record<PolicySettingKey, null>;
}

function snapshot(policyVersion: number, coordinatorGeneration: number) {
  return {
    authority: "rust-policy-service" as const,
    basePreset: "ask-for-approval" as const,
    categoryValues: categories(),
    coordinatorGeneration,
    effectiveCategoryValues: categories(),
    exceptions: [],
    managedRequirementCount: 0,
    maximumAuthorityRuleCount: 0,
    policyVersion,
    preset: "ask-for-approval" as const,
    revocationEpoch: policyVersion,
  };
}

function provider(providerId: string): ProviderRecord {
  return {
    authentication: { type: "bearer" },
    disabledModelIds: [],
    displayName: providerId,
    enabled: true,
    endpoint: {
      apiKind: "openai-compatible",
      baseUrl: `https://${providerId}.example/v1`,
      endpointId: `${providerId}-endpoint`,
    },
    generation: 1,
    hasCredential: true,
    headers: {},
    kind: "custom",
    models: [
      {
        availability: "available",
        checkedAtMs: 1_784_476_800_000,
        contextTokens: 128_000,
        displayName: `${providerId} model`,
        features: {},
        lifecycle: "active",
        modelId: "model-one",
        outputTokens: 16_000,
        productionReady: true,
        recommendationRank: 0,
      },
    ],
    providerId,
    selectedModelId: null,
    testStatus: { state: "untested" },
  };
}

function providerSnapshot(
  generation: number,
  pendingApproval: ProviderPendingApproval | null = null,
): ProviderSettingsSnapshot {
  return {
    authority: "rust-provider-service",
    configurationGeneration: 1,
    coordinatorGeneration: generation,
    credentialFallbackRequired: false,
    credentialProtection: "installation-key",
    defaultEnvironment: "local",
    defaultRuntime: "opencode",
    generation,
    modelRoute: null,
    onboardingCompleted: true,
    pendingApproval,
    providers: [provider("provider-one"), provider("provider-two")],
  };
}

function runtimeSnapshot(): RuntimeCoreSnapshot {
  const capability = {
    checkedAtMs: 1_784_476_800_000,
    detail: null,
    expiresAtMs: null,
    source: "fixture",
    state: "supported" as const,
  };
  return {
    authority: "rust-core",
    capabilityGeneration: 1,
    generation: 1 as RuntimeCoreSnapshot["generation"],
    modelRoutes: ["provider-one", "provider-two"].map((providerId) => ({
      adapterKind: "fixture-adapter",
      capabilities: {
        audio: capability,
        reasoning: capability,
        tools: capability,
        vision: capability,
      },
      contextTokens: 128_000,
      lifecycle: "active" as const,
      modelId: "model-one",
      nativeRuntimeVersion: "1.0.0",
      providerId,
      runtimeKind: "open-code",
    })),
    onboardingReady: true,
    pendingApprovals: [],
    providerGeneration: 1,
    providers: [],
    runtimeGeneration: 1,
    runtimes: [],
  };
}

describe("CoreSettingsController policy authority", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("refreshes stale native authority while retaining the unsaved policy draft", async () => {
    policyMocks.readPolicySettings
      .mockResolvedValueOnce(snapshot(7, 12))
      .mockResolvedValueOnce(snapshot(8, 13));
    policyMocks.savePolicySettings.mockRejectedValueOnce(
      new ProtocolBoundaryError(
        "staleGeneration",
        "Policy authority changed before Save.",
        true,
      ),
    );

    render(
      <CoreSettingsController
        onNavigate={vi.fn()}
        route="/settings/advanced-policies"
      />,
    );

    const select = await screen.findByRole("combobox", {
      name: "workspace.modify policy",
    });
    fireEvent.change(select, { target: { value: "ask" } });
    fireEvent.click(screen.getByRole("button", { name: "Save policies" }));

    await waitFor(() => {
      expect(policyMocks.readPolicySettings).toHaveBeenCalledTimes(2);
    });
    expect(select).toHaveValue("ask");
    expect(screen.getByText("Custom (unsaved)")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save policies" })).toBeEnabled();
  });
});

describe("CoreSettingsController model refresh authority", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    providerMocks.readProviderSnapshot.mockResolvedValue(providerSnapshot(1));
    runtimeMocks.readRuntimeCoreSnapshot.mockResolvedValue(runtimeSnapshot());
  });

  it("pauses at an exact Provider approval and resumes the remaining refreshes", async () => {
    const pending = providerSnapshot(2, {
      expiresAtMs: 10_000,
      operation: "test-connection",
      promptId: "approval-provider-one",
      providerId: "provider-one",
      providerName: "provider-one",
    });
    const approved = providerSnapshot(3);
    const complete = providerSnapshot(4);
    providerMocks.testProviderConnection
      .mockResolvedValueOnce(pending)
      .mockResolvedValueOnce(complete);
    providerMocks.answerProviderApproval.mockResolvedValueOnce(approved);

    render(
      <CoreSettingsController onNavigate={vi.fn()} route="/settings/models" />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Refresh" }));
    expect(
      await screen.findByText("Provider approval required"),
    ).toBeInTheDocument();
    expect(screen.queryByText("Models refreshed")).not.toBeInTheDocument();
    expect(providerMocks.testProviderConnection).toHaveBeenCalledTimes(1);
    expect(
      screen.getByRole("button", { name: "Disable visible models" }),
    ).toBeDisabled();
    for (const toggle of screen.getAllByRole("switch")) {
      expect(toggle).toBeDisabled();
    }

    fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
    expect(await screen.findByText("Models refreshed")).toBeInTheDocument();
    expect(providerMocks.answerProviderApproval).toHaveBeenCalledWith(
      "approval-provider-one",
      "allow",
    );
    expect(providerMocks.testProviderConnection).toHaveBeenNthCalledWith(
      1,
      "provider-one",
    );
    expect(providerMocks.testProviderConnection).toHaveBeenNthCalledWith(
      2,
      "provider-two",
    );
  });

  it("stops without a success claim when the refresh approval is denied", async () => {
    const pending = providerSnapshot(2, {
      expiresAtMs: 10_000,
      operation: "test-connection",
      promptId: "approval-provider-one",
      providerId: "provider-one",
      providerName: "provider-one",
    });
    providerMocks.testProviderConnection.mockResolvedValueOnce(pending);
    providerMocks.answerProviderApproval.mockResolvedValueOnce(
      providerSnapshot(3),
    );

    render(
      <CoreSettingsController onNavigate={vi.fn()} route="/settings/models" />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Refresh" }));
    fireEvent.click(await screen.findByRole("button", { name: "Deny" }));

    expect(await screen.findByText("Model refresh failed")).toBeInTheDocument();
    expect(screen.queryByText("Models refreshed")).not.toBeInTheDocument();
    expect(providerMocks.testProviderConnection).toHaveBeenCalledTimes(1);
  });
});
