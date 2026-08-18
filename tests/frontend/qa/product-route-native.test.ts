import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  CorrelationId,
  PickerGrantId,
  RequestId,
} from "../../../src/frontend/platform/protocol";
import { createPlatformAdapter } from "../../../src/frontend/platform/platform-service";
import { createProviderAdapter } from "../../../src/frontend/platform/provider-service";
import { createRuntimeCoreAdapter } from "../../../src/frontend/platform/runtime-core";
import {
  createWorkspaceStartAdapter,
  type WorkspaceStartTransport,
} from "../../../src/frontend/platform/workspace-start";
import {
  continueQaWorkspaceRecovery,
  createQaWorkspaceFixtureAdapter,
  openQaWorkspace,
  QA_WORKSPACE_RECOVERY_NOTICE,
  resetQaWorkspaceFixture,
} from "../../../src/frontend/qa/workspace-fixture";
import {
  createQaProductRouteAdapter,
  invokeQaProductRoute,
  resetQaProductRoute,
} from "../../../src/frontend/qa/product-route-native";

describe("QA Workspace recovery fixtures", () => {
  beforeEach(() => {
    vi.stubEnv("VITE_C4OS_QA_FIXTURES", "1");
    resetQaProductRoute();
    resetQaWorkspaceFixture();
  });

  it("clears direct recovery only through explicit Continue", async () => {
    const opened = await openQaWorkspace({
      type: "openRecent",
      workspaceId: QA_WORKSPACE_RECOVERY_NOTICE.workspaceId,
    });
    expect(opened.recoveryNotice).toEqual(QA_WORKSPACE_RECOVERY_NOTICE);

    await expect(continueQaWorkspaceRecovery()).resolves.toMatchObject({
      workspaceName: "Legacy UI",
      recovered: true,
      recoveryNotice: null,
    });
    await expect(continueQaWorkspaceRecovery()).rejects.toThrow(
      /No QA Workspace recovery is pending/,
    );
  });

  it("implements the complete production acknowledgement protocol", async () => {
    const transport: WorkspaceStartTransport = {
      async invoke(command, args) {
        return invokeQaProductRoute(command, args);
      },
    };
    const adapter = createWorkspaceStartAdapter(transport, {
      requestIdFactory: () => "request:qa-workspace" as RequestId,
      correlationIdFactory: () => "correlation:qa-workspace" as CorrelationId,
    });

    await expect(adapter.readSnapshot()).resolves.toMatchObject({
      generation: 51,
      activeRecoveryNotice: null,
    });
    const opened = await adapter.openArchive(
      "picker-grant:legacy-workspace" as PickerGrantId,
    );
    expect(opened).toMatchObject({
      workspaceId: "workspace-qa-0003",
      workspaceName: "Legacy UI",
      recovered: true,
      recoveryNotice: QA_WORKSPACE_RECOVERY_NOTICE,
    });

    await expect(
      adapter.acknowledgeRecovery(opened.recoveryNotice!),
    ).resolves.toMatchObject({
      generation: 53,
      activeRecoveryNotice: null,
    });
  });

  it("isolates mutable production-route fixture state and resets exactly", async () => {
    const first = createQaProductRouteAdapter();
    const second = createQaProductRouteAdapter();
    const args = {
      request: {
        protocolVersion: 1,
        requestId: "request:qa-isolation",
        correlationId: "correlation:qa-isolation",
        expectedGeneration: 51,
      },
      input: {
        workspaceId: "workspace-qa-0001",
        expectedGeneration: 51,
      },
    };

    await first.invoke("workspace_start_open_recent", args);

    expect(first.snapshot()).toMatchObject({
      workspaceGeneration: 52,
      conversationGeneration: 52,
      activatedWorkspaceId: "workspace-qa-0001",
    });
    expect(second.snapshot()).toMatchObject({
      workspaceGeneration: 51,
      conversationGeneration: 51,
      activatedWorkspaceId: "workspace-qa-0001",
    });
    expect(first.reset()).toEqual(second.snapshot());
  });

  it("boots and reveals the native QA window through deterministic platform authority", async () => {
    const fixture = createQaProductRouteAdapter();
    const platform = createPlatformAdapter(
      {
        invoke(command, args) {
          return fixture.invoke(command, args);
        },
      },
      {
        requestIdFactory: () => "request:qa-platform" as RequestId,
        correlationIdFactory: () => "correlation:qa-platform" as CorrelationId,
      },
    );

    await expect(platform.readSnapshot()).resolves.toMatchObject({
      platform: "macos",
      architecture: "aarch64",
      initialTheme: {
        scheme: "light",
        source: "webviewPreferredColorScheme",
      },
      window: {
        decorations: "standard",
        initiallyVisible: false,
      },
      capabilities: {
        nativeApplicationMenu: true,
        nativeSettingsShortcut: true,
        standardWindowDecorations: true,
      },
      settingsMenu: {
        route: "/settings/providers",
        accelerator: "CmdOrCtrl+,",
      },
    });
    await expect(platform.revealMainWindow()).resolves.toBeUndefined();
  });

  it("keeps Provider Test transient until Continue persists the confirmed model", async () => {
    const fixture = createQaProductRouteAdapter();
    const provider = createProviderAdapter(
      {
        invoke(command, args) {
          return fixture.invoke(command, args);
        },
      },
      {
        requestIdFactory: () => "request:qa-provider" as RequestId,
        correlationIdFactory: () => "correlation:qa-provider" as CorrelationId,
      },
    );

    await expect(provider.readSnapshot()).resolves.toMatchObject({
      onboardingCompleted: false,
      providers: [],
      transientTest: null,
    });
    const tested = await provider.testConnection({
      providerId: "provider:qa-openai",
      kind: "open-ai",
      displayName: "QA OpenAI",
      endpoint: {
        endpointId: "provider:qa-openai:primary",
        baseUrl: "https://api.openai.com/v1",
        apiKind: "openai",
      },
      authentication: { type: "bearer" },
      headers: {},
      secret: "qa-renderer-only-key",
      enabled: true,
    });
    expect(tested).toMatchObject({
      onboardingCompleted: false,
      providers: [],
      transientTest: {
        testToken: "provider-test:qa-onboarding",
        provider: {
          providerId: "provider:qa-openai",
          selectedModelId: "openai/gpt-5",
        },
      },
    });
    expect(fixture.snapshot().providerConfigured).toBe(false);

    await expect(
      provider.completeOnboarding(
        "provider-test:qa-onboarding",
        "openai/gpt-5",
      ),
    ).resolves.toMatchObject({
      onboardingCompleted: true,
      transientTest: null,
      providers: [
        {
          providerId: "provider:qa-openai",
          selectedModelId: "openai/gpt-5",
        },
      ],
    });
    expect(fixture.snapshot()).toMatchObject({
      onboardingCompleted: true,
      providerConfigured: true,
    });
  });

  it("exposes failed and zero-model onboarding outcomes only through QA secrets", async () => {
    for (const [displayName, state] of [
      ["QA Failed Provider", "failed"],
      ["QA No Models Provider", "succeededNoUsableModels"],
    ] as const) {
      const fixture = createQaProductRouteAdapter();
      const provider = createProviderAdapter(
        {
          invoke(command, args) {
            return fixture.invoke(command, args);
          },
        },
        {
          requestIdFactory: () => `request:${state}` as RequestId,
          correlationIdFactory: () => `correlation:${state}` as CorrelationId,
        },
      );

      await expect(
        provider.testConnection({
          providerId: "provider:qa-openai",
          kind: "open-ai",
          displayName,
          endpoint: {
            endpointId: "provider:qa-openai:primary",
            baseUrl: "https://api.openai.com/v1",
            apiKind: "openai",
          },
          authentication: { type: "bearer" },
          headers: {},
          secret: "qa-renderer-only-key",
          enabled: true,
        }),
      ).resolves.toMatchObject({
        onboardingCompleted: false,
        providers: [],
        transientTest: {
          provider: {
            testStatus: { state },
            models: [],
            selectedModelId: null,
          },
        },
      });
      expect(fixture.snapshot().providerConfigured).toBe(false);
    }
  });

  it("projects an explicit Ask provider test without persisting the draft", async () => {
    const fixture = createQaProductRouteAdapter();
    const provider = createProviderAdapter(
      {
        invoke(command, args) {
          return fixture.invoke(command, args);
        },
      },
      {
        requestIdFactory: () => "request:qa-provider-ask" as RequestId,
        correlationIdFactory: () =>
          "correlation:qa-provider-ask" as CorrelationId,
      },
    );

    const pending = await provider.testConnection({
      providerId: "provider:qa-ask",
      kind: "open-ai",
      displayName: "QA Ask Provider",
      endpoint: {
        endpointId: "provider:qa-ask:primary",
        baseUrl: "https://api.openai.com/v1",
        apiKind: "openai",
      },
      authentication: { type: "bearer" },
      headers: {},
      secret: "qa-renderer-only-key",
      enabled: true,
    });

    expect(pending).toMatchObject({
      providers: [],
      transientTest: null,
      pendingApproval: {
        promptId: "provider-approval:qa-explicit-ask",
        operation: "test-connection",
        providerId: "provider:qa-ask",
        providerName: "QA Ask Provider",
      },
    });
    expect(fixture.snapshot().providerConfigured).toBe(false);

    await expect(
      provider.answerApproval("provider-approval:qa-explicit-ask", "allow"),
    ).resolves.toMatchObject({
      pendingApproval: null,
      providers: [],
      transientTest: {
        testToken: "provider-test:qa-onboarding",
      },
    });
    expect(fixture.snapshot().providerConfigured).toBe(false);
  });

  it("projects and settles one runtime-initiated credential approval", async () => {
    const fixture = createQaProductRouteAdapter();
    const runtime = createRuntimeCoreAdapter(
      {
        invoke(command, args) {
          return fixture.invoke(command, args);
        },
      },
      {
        requestIdFactory: () => "request:qa-runtime" as RequestId,
        correlationIdFactory: () => "correlation:qa-runtime" as CorrelationId,
      },
    );

    const snapshot = await runtime.readSnapshot();
    expect(snapshot.pendingApprovals).toHaveLength(1);
    expect(snapshot.pendingApprovals[0]).toMatchObject({
      runtimeId: "opencode-primary",
      promptId: "approval:runtime-credential-review",
      approvalKind: "runtime-effect",
      providerId: null,
      modelId: null,
      parentOperation: null,
    });
    expect(snapshot.pendingApprovals[0]?.summary).toContain(
      "temporary use of the OpenAI credential",
    );

    const approval = snapshot.pendingApprovals[0]!;
    await expect(
      runtime.answerApproval({
        runtimeId: approval.runtimeId,
        correlationId: approval.correlationId,
        promptId: approval.promptId,
        answer: "deny",
        remember: "once",
      }),
    ).resolves.toMatchObject({
      promptId: "approval:runtime-credential-review",
    });
    await expect(runtime.readSnapshot()).resolves.toMatchObject({
      pendingApprovals: [],
    });
  });

  it("isolates direct Workspace recovery adapters", async () => {
    const first = createQaWorkspaceFixtureAdapter();
    const second = createQaWorkspaceFixtureAdapter();

    await first.open({
      type: "openRecent",
      workspaceId: QA_WORKSPACE_RECOVERY_NOTICE.workspaceId,
    });

    expect(first.pendingRecovery()).toEqual(QA_WORKSPACE_RECOVERY_NOTICE);
    expect(second.pendingRecovery()).toBeNull();
    first.reset();
    expect(first.pendingRecovery()).toBeNull();
  });

  it("fails closed for unhandled commands without invoking native state", async () => {
    const adapter = createQaProductRouteAdapter();

    await expect(
      adapter.invoke("configuration_open_external", {
        request: {
          protocolVersion: 1,
          requestId: "request:qa-denied",
          correlationId: "correlation:qa-denied",
          expectedGeneration: 1,
        },
        input: { target: "diagnostics-folder" },
      }),
    ).rejects.toThrow(
      "QA fixture command is not allowlisted: configuration_open_external",
    );
    expect(adapter.snapshot()).toEqual(
      createQaProductRouteAdapter().snapshot(),
    );
  });
});
