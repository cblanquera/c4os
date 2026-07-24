import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  CorrelationId,
  PickerGrantId,
  RequestId,
} from "../../../src/frontend/platform/protocol";
import { createPlatformAdapter } from "../../../src/frontend/platform/platform-service";
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
