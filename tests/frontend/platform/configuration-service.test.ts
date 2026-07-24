import { describe, expect, it } from "vitest";

import { PROTOCOL_VERSION } from "../../../src/frontend/platform/protocol";
import {
  createConfigurationAdapter,
  type ConfigurationTransport,
} from "../../../src/frontend/platform/configuration-service";

const REQUEST_ID = "request:00000000-0000-4000-8000-000000000031";
const CORRELATION_ID = "correlation:00000000-0000-4000-8000-000000000032";

function payload(generation = 3, coordinatorGeneration = 8, policyVersion = 5) {
  return {
    authority: "rust-configuration-service",
    generation,
    coordinatorGeneration,
    policyVersion,
    defaultApprovalPreset: "approve_safe_actions",
    restoreLastWorkspace: true,
    inheritShellEnvironment: false,
    shellEnvironmentAllowlist: ["PATH"],
    browserEnvironment: "workspace_project",
    defaultRuntime: "opencode",
    defaultEnvironment: "local",
    modelRoute: "provider-openai::gpt-4o-mini",
    hasExternalError: false,
  };
}

class FixtureTransport implements ConfigurationTransport {
  readonly calls: Array<{
    command: Parameters<ConfigurationTransport["invoke"]>[0];
    args: Readonly<Record<string, unknown>>;
  }> = [];

  async invoke(
    command: Parameters<ConfigurationTransport["invoke"]>[0],
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown> {
    this.calls.push({ command, args });
    const request = args.request as {
      requestId: string;
      correlationId: string;
    };
    const generation = command === "configuration_snapshot" ? 3 : 4;
    return {
      protocolVersion: PROTOCOL_VERSION,
      requestId: request.requestId,
      correlationId: request.correlationId,
      generation,
      payload:
        command === "configuration_open_external"
          ? { opened: true }
          : payload(
              generation,
              generation === 3 ? 8 : 9,
              generation === 3 ? 5 : 6,
            ),
    };
  }
}

describe("configuration service adapter", () => {
  it("advances configuration, coordinator, and policy CAS identities", async () => {
    const transport = new FixtureTransport();
    const adapter = createConfigurationAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });

    const snapshot = await adapter.readSnapshot();
    expect(snapshot).toMatchObject({
      generation: 3,
      coordinatorGeneration: 8,
      policyVersion: 5,
      browserEnvironment: "workspace_project",
    });
    await adapter.saveSettings({
      defaultApprovalPreset: "approve_for_me",
      restoreLastWorkspace: false,
      inheritShellEnvironment: true,
      browserEnvironment: "chat",
      defaultRuntime: "pi",
      defaultEnvironment: "local",
    });
    expect(transport.calls[1]?.args.input).toEqual({
      expectedConfigurationGeneration: 3,
      expectedCoordinatorGeneration: 8,
      expectedPolicyVersion: 5,
      defaultApprovalPreset: "approve_for_me",
      restoreLastWorkspace: false,
      inheritShellEnvironment: true,
      browserEnvironment: "chat",
      defaultRuntime: "pi",
      defaultEnvironment: "local",
    });
  });

  it("opens only against the latest configuration cursor", async () => {
    const transport = new FixtureTransport();
    const adapter = createConfigurationAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });
    await adapter.readSnapshot();
    await adapter.openExternal();
    expect(
      (transport.calls[1]?.args.request as { expectedGeneration: number })
        .expectedGeneration,
    ).toBe(3);
  });

  it("rejects a response whose identity does not match", async () => {
    const adapter = createConfigurationAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as { requestId: string };
          return {
            protocolVersion: PROTOCOL_VERSION,
            requestId: request.requestId,
            correlationId: "correlation:wrong",
            generation: 3,
            payload: payload(),
          };
        },
      },
      {
        requestIdFactory: () => REQUEST_ID as never,
        correlationIdFactory: () => CORRELATION_ID as never,
      },
    );
    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "correlationMismatch",
    });
  });

  it("preserves stale-generation conflict evidence for draft reconciliation", async () => {
    const transport = new FixtureTransport();
    const adapter = createConfigurationAdapter(
      {
        async invoke(command, args) {
          if (command === "configuration_save_settings") {
            throw {
              code: "staleGeneration",
              message: "Configuration changed before Save.",
              retryable: true,
              details: {
                activeGeneration: { kind: "unsigned", value: 4 },
                baseGeneration: { kind: "unsigned", value: 3 },
                changedKeys: {
                  kind: "text",
                  value: "browser_environment, restore_last_workspace",
                },
              },
            };
          }
          return transport.invoke(command, args);
        },
      },
      {
        requestIdFactory: () => REQUEST_ID as never,
        correlationIdFactory: () => CORRELATION_ID as never,
      },
    );

    await adapter.readSnapshot();
    await expect(
      adapter.saveSettings({
        defaultApprovalPreset: "approve_for_me",
        restoreLastWorkspace: false,
        inheritShellEnvironment: true,
        browserEnvironment: "chat",
        defaultRuntime: "pi",
        defaultEnvironment: "local",
      }),
    ).rejects.toMatchObject({
      code: "staleGeneration",
      details: {
        activeGeneration: { kind: "unsigned", value: 4 },
        baseGeneration: { kind: "unsigned", value: 3 },
        changedKeys: {
          kind: "text",
          value: "browser_environment, restore_last_workspace",
        },
      },
    });
  });
});
