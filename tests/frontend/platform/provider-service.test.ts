import { describe, expect, it } from "vitest";

import { PROTOCOL_VERSION } from "../../../src/frontend/platform/protocol";
import {
  createProviderAdapter,
  type ProviderCommand,
  type ProviderTransport,
} from "../../../src/frontend/platform/provider-service";

const REQUEST_ID = "request:00000000-0000-4000-8000-000000000001";
const CORRELATION_ID = "correlation:00000000-0000-4000-8000-000000000002";

function model() {
  const evidence = {
    state: "supported",
    layer: "adapter-normalized",
    source: "c4os.openai-compatible-catalog.v1",
    checkedAtMs: 1_721_300_000_000,
    expiresAtMs: 1_721_300_300_000,
    constraints: [],
    allowedValues: [],
    reason: null,
  };
  return {
    modelId: "gpt-4o-mini",
    displayName: "GPT-4o mini",
    recommendationRank: 0,
    availability: "available",
    checkedAtMs: 1_721_300_000_000,
    capabilities: {
      schemaVersion: 1,
      layer: "adapter-normalized",
      route: {},
      lifecycle: "active",
      features: { "input-text": evidence, "output-text": evidence },
      numericLimits: {
        "context-tokens": {
          evidence,
          maximum: 128_000,
          confidence: "confirmed",
        },
      },
      rawEvidenceSha256: `sha256:${"a".repeat(64)}`,
    },
    providerDeclaration: {
      schemaVersion: 1,
      providerModelId: "gpt-4o-mini",
    },
  };
}

function payload(
  generation = 3,
  coordinatorGeneration = 8,
  transientTest: Readonly<Record<string, unknown>> | null = null,
) {
  return {
    authority: "rust-provider-service",
    coordinatorGeneration,
    configurationGeneration: 4,
    credentialProtection: "installation-key",
    credentialFallbackRequired: false,
    onboardingCompleted: true,
    providers: {
      generation,
      onboardingCompletedAtMs: 1_721_300_000_001,
      providers: [
        {
          profile: {
            schemaVersion: 1,
            providerId: "provider-openai",
            kind: "open-ai",
            displayName: "OpenAI",
            endpoint: {
              endpointId: "openai-api",
              baseUrl: "https://api.openai.com/v1",
              apiKind: "openai",
            },
            authentication: { type: "bearer" },
            credentialReference: `credential:${"1".repeat(32)}`,
            headers: {},
            enabled: true,
          },
          testStatus: {
            state: "succeeded",
            checkedAtMs: 1_721_300_000_000,
          },
          connectionEvidence: {},
          models: { "gpt-4o-mini": model() },
          selectedModelId: "gpt-4o-mini",
          generation,
        },
      ],
    },
    modelRoute: "provider-openai::gpt-4o-mini",
    defaultRuntime: "opencode",
    defaultEnvironment: "local",
    pendingApproval: null,
    transientTest,
  };
}

class FixtureTransport implements ProviderTransport {
  readonly calls: Array<{
    command: ProviderCommand;
    args: Readonly<Record<string, unknown>>;
  }> = [];

  async invoke(
    command: ProviderCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown> {
    this.calls.push({ command, args });
    const request = args.request as {
      requestId: string;
      correlationId: string;
    };
    const nextGeneration = command === "provider_snapshot" ? 3 : 4;
    return {
      protocolVersion: PROTOCOL_VERSION,
      requestId: request.requestId,
      correlationId: request.correlationId,
      generation: nextGeneration,
      payload: payload(nextGeneration, command === "provider_snapshot" ? 8 : 9),
    };
  }
}

describe("provider service adapter", () => {
  it("parses the non-secret authoritative snapshot and advances both CAS cursors", async () => {
    const transport = new FixtureTransport();
    const adapter = createProviderAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });

    const snapshot = await adapter.readSnapshot();
    expect(snapshot.onboardingCompleted).toBe(true);
    expect(snapshot.providers[0]).toMatchObject({
      providerId: "provider-openai",
      hasCredential: true,
      selectedModelId: "gpt-4o-mini",
    });
    expect(snapshot.providers[0]?.models[0]?.contextTokens).toBe(128_000);

    await adapter.testConnection({
      providerId: "provider-openai",
      kind: "open-ai",
      displayName: "OpenAI",
      endpoint: {
        endpointId: "openai-api",
        baseUrl: "https://api.openai.com/v1",
        apiKind: "openai",
      },
      authentication: { type: "bearer" },
      headers: {},
      secret: "transient-provider-secret",
      enabled: true,
    });
    expect(transport.calls[1]?.args.input).toEqual({
      expectedCoordinatorGeneration: 8,
      expectedProviderGeneration: 3,
      providerId: "provider-openai",
      kind: "open-ai",
      displayName: "OpenAI",
      endpoint: {
        endpointId: "openai-api",
        baseUrl: "https://api.openai.com/v1",
        apiKind: "openai",
      },
      authentication: { type: "bearer" },
      headers: {},
      secret: "transient-provider-secret",
      enabled: true,
    });
    expect(JSON.stringify(snapshot)).not.toContain("credential:1111");
  });

  it("parses a transient test projection and completes it with one token", async () => {
    const calls: Array<{
      command: ProviderCommand;
      args: Readonly<Record<string, unknown>>;
    }> = [];
    const adapter = createProviderAdapter(
      {
        async invoke(command, args) {
          calls.push({ command, args });
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          const generation = command === "provider_snapshot" ? 3 : 4;
          return {
            protocolVersion: PROTOCOL_VERSION,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation,
            payload: payload(
              generation,
              command === "provider_snapshot" ? 8 : 9,
              command === "provider_complete_onboarding"
                ? null
                : {
                    testToken:
                      "provider-test:00000000-0000-4000-8000-000000000003",
                    provider: payload().providers.providers[0],
                  },
            ),
          };
        },
      },
      {
        requestIdFactory: () => REQUEST_ID as never,
        correlationIdFactory: () => CORRELATION_ID as never,
      },
    );

    const tested = await adapter.readSnapshot();
    expect(tested.transientTest).toMatchObject({
      testToken: "provider-test:00000000-0000-4000-8000-000000000003",
      provider: { providerId: "provider-openai" },
    });
    expect(JSON.stringify(tested.transientTest)).not.toContain("secret");
    await adapter.completeOnboarding(
      "provider-test:00000000-0000-4000-8000-000000000003",
      "gpt-4o-mini",
    );
    expect(calls[1]).toMatchObject({
      command: "provider_complete_onboarding",
      args: {
        input: {
          expectedCoordinatorGeneration: 8,
          expectedProviderGeneration: 3,
          expectedConfigurationGeneration: 4,
          testToken: "provider-test:00000000-0000-4000-8000-000000000003",
          modelId: "gpt-4o-mini",
          runtimeId: "opencode",
          environmentId: "local",
        },
      },
    });
  });

  it("passes a raw key only in the save command and never places it in parsed state", async () => {
    const transport = new FixtureTransport();
    const adapter = createProviderAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });
    await adapter.readSnapshot();
    await adapter.saveProfile({
      providerId: "provider-compatible",
      kind: "custom",
      displayName: "Local compatible",
      endpoint: {
        endpointId: "compatible-api",
        baseUrl: "http://127.0.0.1:4010/v1",
        apiKind: "openai-compatible",
      },
      authentication: { type: "apiKeyHeader", headerName: "X-Local-Key" },
      headers: { "X-Workspace": "test" },
      secret: "fixture-provider-secret",
      enabled: true,
    });
    expect(transport.calls[1]?.args.input).toMatchObject({
      expectedCoordinatorGeneration: 8,
      expectedProviderGeneration: 3,
      secret: "fixture-provider-secret",
    });
    expect(JSON.stringify(payload(4, 9))).not.toContain(
      "fixture-provider-secret",
    );
  });

  it("retries a stale pre-effect save once after refreshing private CAS cursors", async () => {
    const calls: Array<{
      command: ProviderCommand;
      args: Readonly<Record<string, unknown>>;
    }> = [];
    let coordinatorGeneration = 8;
    let saveAttempts = 0;
    const adapter = createProviderAdapter(
      {
        async invoke(command, args) {
          calls.push({ command, args });
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          if (command === "provider_save_profile" && saveAttempts++ === 0) {
            coordinatorGeneration = 9;
            throw {
              code: "staleGeneration",
              message: "Provider state changed before the operation",
              retryable: true,
            };
          }
          const generation = command === "provider_save_profile" ? 4 : 3;
          return {
            protocolVersion: PROTOCOL_VERSION,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation,
            payload: payload(generation, coordinatorGeneration),
          };
        },
      },
      {
        requestIdFactory: () => REQUEST_ID as never,
        correlationIdFactory: () => CORRELATION_ID as never,
      },
    );
    const draft = {
      providerId: "provider-compatible",
      kind: "custom" as const,
      displayName: "Local compatible",
      endpoint: {
        endpointId: "compatible-api",
        baseUrl: "http://127.0.0.1:4010/v1",
        apiKind: "openai-compatible" as const,
      },
      authentication: { type: "bearer" as const },
      secret: "fixture-provider-secret",
      headers: {},
      enabled: true,
    };

    await adapter.readSnapshot();
    await adapter.saveProfile(draft);
    expect(calls.map(({ command }) => command)).toEqual([
      "provider_snapshot",
      "provider_save_profile",
      "provider_snapshot",
      "provider_save_profile",
    ]);
    expect(calls[3]?.args.input).toMatchObject({
      expectedCoordinatorGeneration: 9,
      expectedProviderGeneration: 3,
    });
  });

  it("retries a stale pre-effect connection test with the same draft once", async () => {
    const calls: Array<{
      command: ProviderCommand;
      args: Readonly<Record<string, unknown>>;
    }> = [];
    let coordinatorGeneration = 8;
    let testAttempts = 0;
    const adapter = createProviderAdapter(
      {
        async invoke(command, args) {
          calls.push({ command, args });
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          if (command === "provider_test_connection" && testAttempts++ === 0) {
            coordinatorGeneration = 9;
            throw {
              code: "staleGeneration",
              message: "Provider state changed before the connection test",
              retryable: true,
            };
          }
          const generation = command === "provider_test_connection" ? 4 : 3;
          return {
            protocolVersion: PROTOCOL_VERSION,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation,
            payload: payload(generation, coordinatorGeneration),
          };
        },
      },
      {
        requestIdFactory: () => REQUEST_ID as never,
        correlationIdFactory: () => CORRELATION_ID as never,
      },
    );
    const draft = {
      providerId: "provider-compatible",
      kind: "custom" as const,
      displayName: "Local compatible",
      endpoint: {
        endpointId: "compatible-api",
        baseUrl: "http://127.0.0.1:4010/v1",
        apiKind: "openai-compatible" as const,
      },
      authentication: { type: "bearer" as const },
      secret: "fixture-provider-secret",
      headers: {},
      enabled: true,
    };

    await adapter.readSnapshot();
    await adapter.testConnection(draft);
    expect(calls.map(({ command }) => command)).toEqual([
      "provider_snapshot",
      "provider_test_connection",
      "provider_snapshot",
      "provider_test_connection",
    ]);
    expect(calls[3]?.args.input).toMatchObject({
      expectedCoordinatorGeneration: 9,
      expectedProviderGeneration: 3,
      secret: "fixture-provider-secret",
    });
  });

  it("publishes and answers an explicit one-time Provider approval", async () => {
    const calls: Array<{
      command: ProviderCommand;
      args: Readonly<Record<string, unknown>>;
    }> = [];
    const adapter = createProviderAdapter(
      {
        async invoke(command, args) {
          calls.push({ command, args });
          const request = args.request as {
            requestId: string;
            correlationId: string;
          };
          return {
            protocolVersion: PROTOCOL_VERSION,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: command === "provider_snapshot" ? 3 : 4,
            payload: {
              ...payload(command === "provider_snapshot" ? 3 : 4, 9),
              pendingApproval:
                command === "provider_answer_approval"
                  ? null
                  : {
                      promptId: "provider-prompt-1",
                      operation: "test-connection",
                      providerId: "provider-openai",
                      providerName: "OpenAI",
                      expiresAtMs: 1_721_300_300_000,
                    },
            },
          };
        },
      },
      {
        requestIdFactory: () => REQUEST_ID as never,
        correlationIdFactory: () => CORRELATION_ID as never,
      },
    );

    const pending = await adapter.readSnapshot();
    expect(pending.pendingApproval).toEqual({
      promptId: "provider-prompt-1",
      operation: "test-connection",
      providerId: "provider-openai",
      providerName: "OpenAI",
      expiresAtMs: 1_721_300_300_000,
    });
    expect(
      await adapter.answerApproval("provider-prompt-1", "allow"),
    ).toMatchObject({ pendingApproval: null });
    expect(calls[1]).toMatchObject({
      command: "provider_answer_approval",
      args: {
        input: { promptId: "provider-prompt-1", answer: "allow" },
      },
    });
  });

  it("rejects a response whose identity does not match its request", async () => {
    const adapter = createProviderAdapter(
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
});
