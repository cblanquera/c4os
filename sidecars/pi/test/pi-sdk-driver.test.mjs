import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import test from "node:test";
import { setImmediate, setTimeout } from "node:timers";
import {
  CredentialFrameDecoder,
  CredentialLeaseBroker,
  PiSdkDriver,
  credentialOperationId,
} from "../pi-sdk-driver.mjs";

test("model preflight resolves the exact local Pi catalog pair without Agent or provider I/O", async () => {
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ sdk });
  assert.equal(
    await driver.preflightModel({
      provider: "openai",
      modelId: "gpt-4o-mini",
    }),
    true,
  );
  assert.equal(
    await driver.preflightModel({
      provider: "openai",
      modelId: "missing-model",
    }),
    false,
  );
  assert.deepEqual(sdk.modelReads, [
    ["openai", "gpt-4o-mini"],
    ["openai", "missing-model"],
  ]);
  assert.equal(sdk.agents.length, 0);
  assert.equal(sdk.providerCalls, 0);
});

test("SDK sampling forwards exact controls through streamSimple and releases one-shot credentials", async () => {
  const released = [];
  const claims = [];
  const credentialBroker = {
    claimForOperation: async (identity, provider) => {
      claims.push([globalThis.structuredClone(identity), provider]);
      return {
        get: (requested) =>
          requested === "openai" ? "operation-credential" : undefined,
        release: () => released.push(true),
      };
    },
  };
  const sdk = fakeSdk();
  sdk.streamSimple = (model, context, options) => {
    sdk.lastSampling = { model, context, options };
    return {
      result: async () => ({
        role: "assistant",
        content: [
          { type: "thinking", thinking: "private" },
          { type: "text", text: "sampled" },
        ],
        provider: "openai",
        model: "gpt-4o-mini",
        responseModel: "gpt-4o-mini-2026-07-01",
        stopReason: "length",
      }),
    };
  };
  const driver = new PiSdkDriver({ credentialBroker, sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    tools: [],
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => undefined,
  });
  const identity = credentialRunIdentity();
  const result = await session.sample({
    messages: [
      { role: "user", text: "Question" },
      { role: "assistant", text: "Prior answer" },
    ],
    systemPrompt: "System",
    maxTokens: 99,
    temperature: 0.2,
    runIdentity: identity,
  });
  assert.deepEqual(result, {
    text: "sampled",
    model: "gpt-4o-mini",
    stopReason: "maxTokens",
  });
  assert.equal(sdk.lastSampling.options.apiKey, "operation-credential");
  assert.equal(sdk.lastSampling.options.maxTokens, 99);
  assert.equal(sdk.lastSampling.options.temperature, 0.2);
  assert.equal(sdk.lastSampling.options.maxRetries, 0);
  assert.equal(sdk.lastSampling.context.systemPrompt, "System");
  assert.equal(sdk.lastSampling.context.tools.length, 0);
  assert.equal(sdk.lastSampling.context.messages[0].role, "user");
  assert.equal(sdk.lastSampling.context.messages[1].role, "assistant");
  assert.deepEqual(claims, [[identity, "openai"]]);
  assert.equal(released.length, 1);
});

test("sampling cancellation waits for native settlement and credential release without mutating Agent messages", async () => {
  const released = [];
  const credentialBroker = {
    claimForOperation: async () => ({
      get: () => "operation-credential",
      release: () => released.push(true),
    }),
  };
  const sdk = fakeSdk();
  let rejectResult;
  sdk.streamSimple = (_model, _context, options) => {
    sdk.lastSamplingSignal = options.signal;
    return {
      result: () =>
        new Promise((_resolve, reject) => {
          rejectResult = reject;
        }),
    };
  };
  const driver = new PiSdkDriver({ credentialBroker, sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    tools: [],
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => undefined,
  });
  const sampling = session.sample({
    messages: [{ role: "user", text: "Keep this out of Chat state" }],
    maxTokens: 8,
    runIdentity: credentialRunIdentity(),
  });
  await new Promise((resolve) => setImmediate(resolve));
  let abortSettled = false;
  const abort = session.abortSampling().then(() => {
    abortSettled = true;
  });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(sdk.lastSamplingSignal.aborted, true);
  assert.equal(abortSettled, false);
  assert.equal(released.length, 0);
  rejectResult(new Error("native stream acknowledged abort"));
  await assert.rejects(sampling, /acknowledged abort/);
  await abort;
  assert.equal(abortSettled, true);
  assert.equal(released.length, 1);
  assert.deepEqual(sdk.lastAgent.options.initialState.messages, []);
});

test("sampling rejects unsupported native terminal reasons and releases its credential", async () => {
  const released = [];
  const credentialBroker = {
    claimForOperation: async () => ({
      get: () => "operation-credential",
      release: () => released.push(true),
    }),
  };
  const sdk = fakeSdk();
  sdk.streamSimple = () => ({
    result: async () => ({
      role: "assistant",
      content: [{ type: "text", text: "must not be accepted" }],
      provider: "openai",
      model: "gpt-4o-mini",
      responseModel: "gpt-4o-mini",
      stopReason: "toolUse",
    }),
  });
  const driver = new PiSdkDriver({ credentialBroker, sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    tools: [],
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => undefined,
  });
  await assert.rejects(
    session.sample({
      messages: [{ role: "user", text: "Question" }],
      maxTokens: 8,
      runIdentity: credentialRunIdentity(),
    }),
    /unsupported stop reason/,
  );
  assert.equal(released.length, 1);
});

test("test TLS trust routes require the exact C4OS profile, native provider, and base URL", async () => {
  const routes = [
    {
      providerId: "provider-openai",
      nativeProviderId: "openai",
      baseUrl: "https://127.0.0.1:4443/v1",
    },
  ];
  let credentialClaims = 0;
  const credentialBroker = {
    claimForOperation: async () => {
      credentialClaims += 1;
      return {
        get: () => "test-only-value",
        release: () => {},
      };
    },
  };
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({
    credentialBroker,
    sdk,
    testTlsTrustCapabilities: routes,
  });
  const matching = await driver.createSession({
    ...sessionOptions(),
    modelRoute: {
      ...sessionOptions().modelRoute,
      baseUrl: "https://127.0.0.1:4443/v1",
    },
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  await matching.dispatch({
    input: "matching-test-route",
    runIdentity: credentialRunIdentity(),
  });
  assert.equal(credentialClaims, 1);

  await assert.rejects(
    matching.dispatch({
      input: "changed-profile",
      runIdentity: {
        ...credentialRunIdentity(),
        providerId: "provider-substitution",
      },
    }),
    /does not match this C4OS operation/,
  );
  assert.equal(credentialClaims, 1);

  sdk.getModel = (provider, modelId) => ({ provider, id: modelId });
  const changedNativeProvider = await driver.createSession({
    ...sessionOptions(),
    c4osSessionId: "session-native-substitution",
    modelRoute: {
      ...sessionOptions().modelRoute,
      provider: "anthropic",
    },
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  await assert.rejects(
    changedNativeProvider.dispatch({
      input: "changed-native-provider",
      runIdentity: credentialRunIdentity(),
    }),
    /does not match this C4OS operation/,
  );

  const changedBaseUrl = await driver.createSession({
    ...sessionOptions(),
    c4osSessionId: "session-base-url-substitution",
    modelRoute: {
      ...sessionOptions().modelRoute,
      baseUrl: "https://provider.example/v1",
    },
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  await assert.rejects(
    changedBaseUrl.dispatch({
      input: "changed-base-url",
      runIdentity: credentialRunIdentity(),
    }),
    /does not match this C4OS operation/,
  );
  assert.equal(credentialClaims, 1);
});

test("test TLS trust route metadata denies unknown fields and duplicate profiles", () => {
  assert.throws(
    () =>
      new PiSdkDriver({
        testTlsTrustCapabilities: [
          {
            providerId: "provider-openai",
            nativeProviderId: "openai",
            baseUrl: "https://api.openai.com/v1",
          },
        ],
      }),
    /base URL is invalid/,
  );
  assert.throws(
    () =>
      new PiSdkDriver({
        testTlsTrustCapabilities: [
          {
            providerId: "provider-openai",
            nativeProviderId: "openai",
            baseUrl: "https://127.0.0.1:4443/v1",
            extra: true,
          },
        ],
      }),
    /route identity is invalid/,
  );
  assert.throws(
    () =>
      new PiSdkDriver({
        testTlsTrustCapabilities: [
          {
            providerId: "provider-openai",
            nativeProviderId: "openai",
            baseUrl: "https://127.0.0.1:4443/v1",
          },
          {
            providerId: "provider-openai",
            nativeProviderId: "anthropic",
            baseUrl: "https://127.0.0.1:4444/v1",
          },
        ],
      }),
    /route identity is invalid/,
  );
});

test("SDK driver exposes only supplied C4OS tools and blocks before every tool body", async () => {
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ sdk });
  let intercepted = 0;
  let consumed = 0;
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async () => {
      intercepted += 1;
      return { block: true, reason: "C4OS denied" };
    },
    consumeBrokeredResult: () => {
      consumed += 1;
      return { summary: "must not execute" };
    },
  });
  await session.dispatch({ input: "tool", runIdentity: runIdentity() });
  assert.equal(intercepted, 1);
  assert.equal(consumed, 0);
  assert.deepEqual(
    sdk.lastAgent.options.initialState.tools.map((tool) => tool.name),
    ["c4os_read_resource", "c4os_propose_action"],
  );
  assert.equal(
    sdk.lastAgent.options.initialState.model.baseUrl,
    "https://api.openai.com/v1",
  );
  const [readTool, actionTool] = sdk.lastAgent.options.initialState.tools;
  assert.deepEqual(Object.keys(readTool.parameters.properties), [
    "resource",
    "selector",
  ]);
  assert.deepEqual(Object.keys(actionTool.parameters.properties), [
    "operation",
    "target",
    "arguments",
  ]);
  assert.equal(
    sdk.lastAgent.options.initialState.tools.every(
      (tool) => tool.executionMode === "sequential",
    ),
    true,
  );
  assert.match(
    sdk.lastAgent.options.initialState.systemPrompt,
    /C4OS owns policy, effects, credentials, and persistence/,
  );
});

test("dispatch cancellation waits through credential claim and prevents a late native prompt", async () => {
  const released = [];
  let deliverCredential;
  const credentialBroker = {
    claimForOperation: () =>
      new Promise((resolve) => {
        deliverCredential = () =>
          resolve({
            get: () => "late-operation-credential",
            release: () => released.push(true),
          });
      }),
  };
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ credentialBroker, sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    tools: [],
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => undefined,
  });

  const dispatch = session.dispatch({
    input: "must-never-reach-prompt",
    runIdentity: credentialRunIdentity(),
  });
  await new Promise((resolve) => setImmediate(resolve));
  let abortSettled = false;
  const abort = session.abort().then(() => {
    abortSettled = true;
  });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(abortSettled, false);
  assert.equal(sdk.lastAgent.lastPrompt, undefined);

  deliverCredential();
  await assert.rejects(dispatch, /aborted/i);
  await abort;
  assert.equal(abortSettled, true);
  assert.equal(sdk.lastAgent.lastPrompt, undefined);
  assert.equal(released.length, 1);
});

test("SDK driver sends verified image content through the exact native image API", async () => {
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  const attachment = {
    attachmentId: "attachment-1",
    stableReference: "blob:attachment-1",
    displayName: "diagram.png",
    mediaType: "image/png",
    byteLength: 17,
    contentSha256:
      "sha256:9a581a44576e790657922cad780dec2e57816e8c20291bce0a4be0bd26520fcb",
    snapshotVersion: 1,
    contentBase64: "a25vd24tcG5nLWNvbnRlbnQ=",
  };

  await session.dispatch({
    input: "",
    attachments: [attachment],
    runIdentity: runIdentity(),
  });

  assert.equal(sdk.lastAgent.lastPrompt, "");
  assert.deepEqual(sdk.lastAgent.lastImages, [
    {
      type: "image",
      data: attachment.contentBase64,
      mimeType: "image/png",
    },
  ]);
});

test("SDK driver tool body can return only the already-completed C4OS broker result", async () => {
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ sdk });
  const nativeEvents = [];
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async () => undefined,
    consumeBrokeredResult: (runId, toolCallId) => {
      assert.equal(runId, "run-1");
      assert.equal(toolCallId, "tool-call-1");
      return {
        status: "succeeded",
        resultCode: "installed-facility-completed",
        details: { authority: "action-gateway" },
      };
    },
    onNativeEvent: (identity, event) => nativeEvents.push({ identity, event }),
  });
  await session.dispatch({ input: "tool", runIdentity: runIdentity() });
  assert.equal(sdk.lastAgent.toolExecutions, 1);
  assert.equal(
    sdk.lastAgent.lastToolResult.content[0].text,
    "installed-facility-completed",
  );
  assert.equal(
    nativeEvents.every(({ identity }) => identity.runId === "run-1"),
    true,
  );
});

test("SDK driver maps Responses composite tool IDs into one stable C4OS ID", async () => {
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ sdk });
  let interceptedToolCallId;
  let consumedToolCallId;
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async (_identity, context) => {
      interceptedToolCallId = context.toolCall.id;
      return undefined;
    },
    consumeBrokeredResult: (_runId, toolCallId) => {
      consumedToolCallId = toolCallId;
      return { summary: "C4OS completed the composite tool call" };
    },
  });

  await session.dispatch({
    input: "composite-tool",
    runIdentity: runIdentity(),
  });

  assert.match(interceptedToolCallId, /^pi-tool-[a-f0-9]{64}$/);
  assert.equal(consumedToolCallId, interceptedToolCallId);
  assert.notEqual(interceptedToolCallId, "call_allow_1|fc_allow_1");
  assert.equal(sdk.lastAgent.toolExecutions, 1);
});

test("one descriptor lease authorizes every provider call in one operation only", async () => {
  const broker = new CredentialLeaseBroker();
  const secret = Buffer.from("one-shot-value");
  const identity = credentialRunIdentity();
  const operationId = credentialOperationId(identity, "openai");
  broker.provide({
    leaseId: operationId,
    provider: "openai",
    value: secret,
    expiresAtMs: Date.now() + 10_000,
  });
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ credentialBroker: broker, sdk });
  assert.equal(driver.credentialChannelAvailable, true);
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  await session.dispatch({ input: "credential", runIdentity: identity });
  assert.deepEqual(sdk.lastAgent.apiKeys, ["one-shot-value", "one-shot-value"]);
  assert.deepEqual([...secret], new Array(secret.length).fill(0));
  await assert.rejects(
    session.dispatch({ input: "credential", runIdentity: identity }),
    /unavailable or already claimed/,
  );
  broker.clear();
});

test("two C4OS profiles sharing native openai claim only their exact operation secret", async () => {
  const broker = new CredentialLeaseBroker();
  const secretA = Buffer.from("team-a-value");
  const secretB = Buffer.from("team-b-value");
  const identityA = {
    ...credentialRunIdentity(),
    sessionId: "session-team-a",
    turnId: "turn-team-a",
    runId: "run-team-a",
    correlationId: "correlation-team-a",
    providerId: "openai-team-a",
  };
  const identityB = {
    ...credentialRunIdentity(),
    sessionId: "session-team-b",
    turnId: "turn-team-b",
    runId: "run-team-b",
    correlationId: "correlation-team-b",
    providerId: "openai-team-b",
  };
  const operationA = credentialOperationId(identityA, "openai");
  const operationB = credentialOperationId(identityB, "openai");
  assert.notEqual(operationA, operationB);
  broker.provide({
    leaseId: operationA,
    provider: "openai",
    value: secretA,
    expiresAtMs: Date.now() + 10_000,
  });
  broker.provide({
    leaseId: operationB,
    provider: "openai",
    value: secretB,
    expiresAtMs: Date.now() + 10_000,
  });
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ credentialBroker: broker, sdk });
  const sessionA = await driver.createSession({
    ...sessionOptions(),
    sessionId: identityA.sessionId,
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  const agentA = sdk.lastAgent;
  const sessionB = await driver.createSession({
    ...sessionOptions(),
    sessionId: identityB.sessionId,
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });
  const agentB = sdk.lastAgent;

  await sessionA.dispatch({ input: "credential", runIdentity: identityA });
  await sessionB.dispatch({ input: "credential", runIdentity: identityB });
  assert.deepEqual(agentA.apiKeys, ["team-a-value", "team-a-value"]);
  assert.deepEqual(agentB.apiKeys, ["team-b-value", "team-b-value"]);
  assert.deepEqual([...secretA], new Array(secretA.length).fill(0));
  assert.deepEqual([...secretB], new Array(secretB.length).fill(0));
  broker.clear();
});

test("credential operation digest matches Rust's full dispatch binding", () => {
  assert.equal(
    credentialOperationId(
      {
        ...credentialRunIdentity(),
        correlationId: "run-correlation-1",
        processGeneration: 4,
      },
      "openai",
    ),
    "pi-provider:01d169892782dbb9ee5101c7d006daff42630eabe2602db71e95ca16be186b6b",
  );
});

test("credential remains operation-local through a brokered tool continuation", async () => {
  const broker = new CredentialLeaseBroker();
  const identity = credentialRunIdentity();
  broker.provide({
    leaseId: credentialOperationId(identity, "openai"),
    provider: "openai",
    value: Buffer.from("tool-value"),
    expiresAtMs: Date.now() + 10_000,
  });
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ credentialBroker: broker, sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async () => undefined,
    consumeBrokeredResult: () => ({ summary: "C4OS completed the action" }),
  });

  await session.dispatch({
    input: "credential-tool",
    runIdentity: identity,
  });

  assert.deepEqual(sdk.lastAgent.apiKeys, ["tool-value", "tool-value"]);
  assert.equal(sdk.lastAgent.toolExecutions, 1);
  assert.equal(
    sdk.lastAgent.lastToolResult.content[0].text,
    "C4OS completed the action",
  );
  broker.clear();
});

test("dispatch waits for descriptor decode and never needs a serialized credential handle", async () => {
  const broker = new CredentialLeaseBroker();
  const identity = credentialRunIdentity();
  const sdk = fakeSdk();
  const driver = new PiSdkDriver({ credentialBroker: broker, sdk });
  const session = await driver.createSession({
    ...sessionOptions(),
    beforeToolCall: async () => ({ block: true }),
    consumeBrokeredResult: () => ({ summary: "unused" }),
  });

  const dispatch = session.dispatch({
    input: "credential",
    runIdentity: identity,
  });
  await new Promise((resolve) => setImmediate(resolve));
  const secret = Buffer.from("delayed-descriptor-value");
  broker.provide({
    leaseId: credentialOperationId(identity, "openai"),
    provider: "openai",
    value: secret,
    expiresAtMs: Date.now() + 10_000,
  });
  await dispatch;

  assert.deepEqual(sdk.lastAgent.apiKeys, [
    "delayed-descriptor-value",
    "delayed-descriptor-value",
  ]);
  assert.deepEqual([...secret], new Array(secret.length).fill(0));
  broker.clear();
});

test("credential broker rejects duplicate delivery, replay, and provider substitution", async () => {
  const broker = new CredentialLeaseBroker();
  const original = Buffer.from("original-value");
  const identity = credentialRunIdentity();
  const operationId = credentialOperationId(identity, "openai");
  broker.provide({
    leaseId: operationId,
    provider: "openai",
    value: original,
    expiresAtMs: Date.now() + 10_000,
  });
  const duplicate = Buffer.from("substitute-value");
  assert.throws(
    () =>
      broker.provide({
        leaseId: operationId,
        provider: "openai",
        value: duplicate,
        expiresAtMs: Date.now() + 10_000,
      }),
    /already delivered/,
  );
  assert.deepEqual([...duplicate], new Array(duplicate.length).fill(0));

  const operation = await broker.claimForOperation(identity, "openai");
  assert.equal(operation.get("anthropic"), undefined);
  assert.equal(operation.get("openai"), undefined);
  const replay = Buffer.from("replay-value");
  assert.throws(
    () =>
      broker.provide({
        leaseId: operationId,
        provider: "openai",
        value: replay,
        expiresAtMs: Date.now() + 10_000,
      }),
    /already delivered/,
  );
  assert.deepEqual([...replay], new Array(replay.length).fill(0));
  assert.deepEqual([...original], new Array(original.length).fill(0));
  broker.clear();
});

test("credential broker zeroes unclaimed secrets when their TTL expires", async () => {
  const broker = new CredentialLeaseBroker();
  const expiring = Buffer.from("expiring-value");
  const identity = credentialRunIdentity();
  broker.provide({
    leaseId: credentialOperationId(identity, "openai"),
    provider: "openai",
    value: expiring,
    expiresAtMs: Date.now() + 20,
  });
  await new Promise((resolve) => setTimeout(resolve, 40));
  assert.deepEqual([...expiring], new Array(expiring.length).fill(0));
  await assert.rejects(
    broker.claimForOperation(identity, "openai", 10),
    /barrier timed out/,
  );

  const expired = Buffer.from("expired-value");
  assert.throws(
    () =>
      broker.provide({
        leaseId: "expired",
        provider: "openai",
        value: expired,
        expiresAtMs: Date.now() - 1,
      }),
    /already expired/,
  );
  assert.deepEqual([...expired], new Array(expired.length).fill(0));
  broker.clear();
});

test("binary credential decoder handles fragmented frames and zeroes every consumed input Buffer", () => {
  const decoder = new CredentialFrameDecoder();
  const encoded = credentialFrame({
    leaseId: "lease-binary-1",
    provider: "openai",
    value: Buffer.from("native-binary-secret"),
    expiresAtMs: Date.now() + 10_000,
  });
  assert.equal(encoded.includes(Buffer.from("valueHex")), false);
  const chunks = [
    Buffer.from(encoded.subarray(0, 3)),
    Buffer.from(encoded.subarray(3, 22)),
    Buffer.from(encoded.subarray(22, 31)),
    Buffer.from(encoded.subarray(31)),
  ];
  const records = chunks.flatMap((chunk) => decoder.push(chunk));
  decoder.finish();
  assert.equal(records.length, 1);
  assert.equal(records[0].leaseId, "lease-binary-1");
  assert.equal(records[0].provider, "openai");
  assert.equal(records[0].value.toString("utf8"), "native-binary-secret");
  for (const chunk of chunks)
    assert.deepEqual([...chunk], new Array(chunk.length).fill(0));
  records[0].value.fill(0);
  encoded.fill(0);
});

test("binary credential decoder fails closed on oversize, malformed, and truncated frames", () => {
  const oversize = credentialFrame({
    leaseId: "lease-oversize",
    provider: "openai",
    value: Buffer.from("x"),
    expiresAtMs: Date.now() + 10_000,
  });
  oversize.writeUInt32BE(
    64 * 1024 + 1,
    21 + "lease-oversize".length + "openai".length,
  );
  assert.throws(
    () => new CredentialFrameDecoder().push(oversize),
    /exceeds its bound/,
  );

  const malformed = credentialFrame({
    leaseId: "lease-malformed",
    provider: "openai",
    value: Buffer.from("x"),
    expiresAtMs: Date.now() + 10_000,
  });
  malformed[0] ^= 0xff;
  assert.throws(
    () => new CredentialFrameDecoder().push(malformed),
    /header is invalid/,
  );

  const truncated = credentialFrame({
    leaseId: "lease-truncated",
    provider: "openai",
    value: Buffer.from("secret"),
    expiresAtMs: Date.now() + 10_000,
  });
  const decoder = new CredentialFrameDecoder();
  decoder.push(Buffer.from(truncated.subarray(0, truncated.length - 2)));
  assert.throws(() => decoder.finish(), /partial frame/);
  truncated.fill(0);
});

function sessionOptions() {
  return {
    c4osSessionId: "session-1",
    workspaceId: "workspace-1",
    modelRoute: {
      provider: "openai",
      modelId: "gpt-4o-mini",
      baseUrl: "https://api.openai.com/v1",
    },
    tools: ["c4os_read_resource", "c4os_propose_action"],
    onNativeEvent: () => {},
  };
}

function runIdentity() {
  return {
    workspaceId: "workspace-1",
    sessionId: "session-1",
    turnId: "turn-1",
    runId: "run-1",
    correlationId: "correlation-1",
    processGeneration: 7,
  };
}

function credentialRunIdentity() {
  return {
    ...runIdentity(),
    runtimeId: "pi-production",
    providerId: "provider-openai",
  };
}

function fakeSdk() {
  const sdk = {
    agents: [],
    modelReads: [],
    providerCalls: 0,
    Type: {
      String: (options) => ({ type: "string", ...options }),
      Unknown: () => ({}),
      Optional: (schema) => schema,
      Record: (key, value) => ({ type: "object", key, value }),
      Object: (properties, options) => ({
        type: "object",
        properties,
        ...options,
      }),
    },
    convertToLlm: (messages) => messages,
    streamSimple: () => {},
    getModel: (provider, modelId) => {
      sdk.modelReads.push([provider, modelId]);
      return provider === "openai" && modelId === "gpt-4o-mini"
        ? { provider, id: modelId }
        : undefined;
    },
  };
  sdk.Agent = class FakeAgent {
    constructor(options) {
      this.options = options;
      this.sessionId = options.sessionId;
      this.listeners = [];
      this.toolExecutions = 0;
      this.apiKeys = [];
      sdk.lastAgent = this;
      sdk.agents.push(this);
    }

    subscribe(listener) {
      this.listeners.push(listener);
    }

    async prompt(input, images = []) {
      this.lastPrompt = input;
      this.lastImages = images;
      this.emit({ type: "agent_start" });
      if (input === "credential") {
        this.apiKeys.push(await this.options.getApiKey("openai"));
        this.apiKeys.push(await this.options.getApiKey("openai"));
      }
      if (input === "credential-tool") {
        this.apiKeys.push(await this.options.getApiKey("openai"));
      }
      if (
        input === "tool" ||
        input === "credential-tool" ||
        input === "composite-tool"
      ) {
        const tool = this.options.initialState.tools[0];
        const toolCallId =
          input === "composite-tool"
            ? "call_allow_1|fc_allow_1"
            : "tool-call-1";
        const decision = await this.options.beforeToolCall({
          toolCall: { name: tool.name, id: toolCallId },
          args: { resource: "workspace:/README.md" },
        });
        if (!decision?.block) {
          this.toolExecutions += 1;
          this.lastToolResult = await tool.execute(toolCallId);
        }
      }
      if (input === "credential-tool") {
        this.apiKeys.push(await this.options.getApiKey("openai"));
      }
      this.emit({ type: "agent_end" });
    }

    emit(event) {
      for (const listener of this.listeners) listener(event);
    }
    async waitForIdle() {}
    abort() {}
    reset() {}
  };
  return sdk;
}

function credentialFrame({ leaseId, provider, value, expiresAtMs }) {
  const leaseIdBytes = Buffer.from(leaseId, "utf8");
  const providerBytes = Buffer.from(provider, "utf8");
  const frame = Buffer.alloc(
    21 + leaseIdBytes.length + providerBytes.length + 4 + value.length + 4,
  );
  frame.write("C4OSCRED", 0, "ascii");
  frame[8] = 1;
  frame.writeUInt16BE(leaseIdBytes.length, 9);
  frame.writeUInt16BE(providerBytes.length, 11);
  frame.writeBigUInt64BE(BigInt(expiresAtMs), 13);
  let offset = 21;
  leaseIdBytes.copy(frame, offset);
  offset += leaseIdBytes.length;
  providerBytes.copy(frame, offset);
  offset += providerBytes.length;
  frame.writeUInt32BE(value.length, offset);
  offset += 4;
  value.copy(frame, offset);
  offset += value.length;
  frame.writeUInt32BE(0, offset);
  return frame;
}
