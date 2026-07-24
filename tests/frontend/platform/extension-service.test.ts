import { describe, expect, it, vi } from "vitest";

import {
  createExtensionAdapter,
  type ExtensionTransport,
} from "../../../src/frontend/platform/extension-service";

const digest = `sha256:${"a".repeat(64)}`;

function snapshot(generation: number) {
  return {
    schemaVersion: 1,
    generation,
    marketplaces: [],
    plugins: [],
    skills: [],
    selectedSkill: null,
    activeWorkers: 0,
    lastEventId: generation,
  };
}

function declarativePlugin() {
  return {
    packageId: "plugin.example",
    packageKind: "plugin",
    name: "Declarative Plugin",
    summary: "Host-owned contributions",
    publisher: "C4OS Tests",
    version: "1.0.0",
    digest,
    originKeyId: "origin-key",
    contentKeyId: "content-key",
    trust: "trusted",
    lifecycle: "available",
    capabilities: ["context.annotation"],
    website: null,
    terms: null,
    privacyPolicy: null,
    activeDigest: null,
    lastKnownGoodDigest: null,
    failureCode: null,
    hasReviewedHooks: false,
    marketplaceId: "marketplace.example",
    source: "/private/tmp/plugin.example",
    compatibility: "^0.1",
    verifiedAtMs: 1,
    hooks: [],
    settings: [
      {
        settingId: "credential",
        label: "Credential",
        description: "Opaque vault reference",
        kind: "credentialReference",
        required: true,
        choices: [],
      },
    ],
    apps: [
      {
        appId: "summary",
        title: "Summary",
        summary: "Host-rendered app",
        settingIds: [],
      },
    ],
    mcpServers: [
      {
        serverId: "server",
        name: "Server metadata",
        transport: "stdio",
        settingIds: ["credential"],
      },
    ],
    availableVersion: null,
    availableDigest: null,
    stagedVersion: null,
    stagedDigest: null,
    activeVersion: null,
    lastKnownGoodVersion: null,
    revocationReason: null,
  };
}

describe("Extension adapter", () => {
  it("carries the exact observed generation into a package mutation", async () => {
    const invoke = vi.fn<ExtensionTransport["invoke"]>(
      async (command, args) => {
        const request = args.request as {
          requestId: string;
          correlationId: string;
        };
        const generation = command === "extension_snapshot" ? 1 : 2;
        return {
          protocolVersion: 1,
          requestId: request.requestId,
          correlationId: request.correlationId,
          generation,
          payload: snapshot(generation),
        };
      },
    );
    const adapter = createExtensionAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-extension" as never,
        correlationIdFactory: () => "correlation-extension" as never,
      },
    );

    await adapter.readSnapshot();
    await adapter.installDisabled("plugin.example");

    expect(invoke).toHaveBeenLastCalledWith("extension_install_disabled", {
      request: expect.objectContaining({ expectedGeneration: 1 }),
      input: { expectedGeneration: 1, packageId: "plugin.example" },
    });
  });

  it("customizes one exact source-qualified Skill at the observed generation", async () => {
    const invoke = vi.fn<ExtensionTransport["invoke"]>(
      async (command, args) => {
        const request = args.request as {
          requestId: string;
          correlationId: string;
        };
        const generation = command === "extension_snapshot" ? 7 : 8;
        return {
          protocolVersion: 1,
          requestId: request.requestId,
          correlationId: request.correlationId,
          generation,
          payload: snapshot(generation),
        };
      },
    );
    const adapter = createExtensionAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-customize" as never,
        correlationIdFactory: () => "correlation-customize" as never,
      },
    );

    await adapter.readSnapshot();
    await adapter.customizeSkill("plugin:github-workflow:review");

    expect(invoke).toHaveBeenLastCalledWith("extension_customize_skill", {
      request: expect.objectContaining({ expectedGeneration: 7 }),
      input: {
        expectedGeneration: 7,
        skillIdentity: "plugin:github-workflow:review",
      },
    });
  });

  it("rejects unknown native fields before publishing Extension state", async () => {
    const adapter = createExtensionAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 1,
            payload: { ...snapshot(1), digest },
          };
        },
      },
      {
        requestIdFactory: () => "request-extension" as never,
        correlationIdFactory: () => "correlation-extension" as never,
      },
    );

    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("forwards an independently obtained marketplace trust pin exactly", async () => {
    const invoke = vi.fn<ExtensionTransport["invoke"]>(
      async (command, args) => {
        const request = args.request as {
          requestId: string;
          correlationId: string;
        };
        const generation = command === "extension_snapshot" ? 1 : 2;
        return {
          protocolVersion: 1,
          requestId: request.requestId,
          correlationId: request.correlationId,
          generation,
          payload: snapshot(generation),
        };
      },
    );
    const adapter = createExtensionAdapter(
      { invoke },
      {
        requestIdFactory: () => "request-marketplace" as never,
        correlationIdFactory: () => "correlation-marketplace" as never,
      },
    );
    await adapter.readSnapshot();
    await adapter.addMarketplace({
      source: "https://example.test/extensions.git",
      gitRef: "refs/heads/main",
      sparsePaths: ["catalog"],
      trustedOrigin: "example-publisher",
      signingKeyId: "origin-key-2026",
      publicKeySha256: `sha256:${"b".repeat(64)}`,
    });
    expect(invoke).toHaveBeenLastCalledWith("extension_add_marketplace", {
      request: expect.objectContaining({ expectedGeneration: 1 }),
      input: {
        source: "https://example.test/extensions.git",
        gitRef: "refs/heads/main",
        sparsePaths: ["catalog"],
        trustedOrigin: "example-publisher",
        signingKeyId: "origin-key-2026",
        publicKeySha256: `sha256:${"b".repeat(64)}`,
      },
    });
  });

  it("strictly parses signed declarative contribution metadata", async () => {
    const adapter = createExtensionAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 1,
            payload: { ...snapshot(1), plugins: [declarativePlugin()] },
          };
        },
      },
      {
        requestIdFactory: () => "request-declarations" as never,
        correlationIdFactory: () => "correlation-declarations" as never,
      },
    );
    const parsed = await adapter.readSnapshot();
    expect(parsed.plugins[0]).toMatchObject({
      packageKind: "plugin",
      settings: [{ kind: "credentialReference", settingId: "credential" }],
      apps: [{ appId: "summary" }],
      mcpServers: [{ serverId: "server", transport: "stdio" }],
    });
  });

  it("strictly parses signed hook invocation arguments", async () => {
    const plugin = {
      ...declarativePlugin(),
      hooks: [
        {
          hookId: "before-turn",
          name: "Before turn",
          arguments: ["--reviewed-contract"],
          event: "before-turn",
          reviewDigest: digest,
          reviewed: true,
          status: "ready",
          grants: ["context.annotation"],
          lastResult: null,
        },
      ],
    };
    const adapter = createExtensionAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 1,
            payload: { ...snapshot(1), plugins: [plugin] },
          };
        },
      },
      {
        requestIdFactory: () => "request-hook-arguments" as never,
        correlationIdFactory: () => "correlation-hook-arguments" as never,
      },
    );

    const parsed = await adapter.readSnapshot();
    expect(parsed.plugins[0]?.hooks[0]?.arguments).toEqual([
      "--reviewed-contract",
    ]);
  });

  it("rejects raw secret-shaped fields in native setting declarations", async () => {
    const plugin = declarativePlugin();
    plugin.settings[0] = { ...plugin.settings[0], value: "forbidden" } as never;
    const adapter = createExtensionAdapter(
      {
        async invoke(_command, args) {
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 1,
            payload: { ...snapshot(1), plugins: [plugin] },
          };
        },
      },
      {
        requestIdFactory: () => "request-secret-field" as never,
        correlationIdFactory: () => "correlation-secret-field" as never,
      },
    );
    await expect(adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });
});
