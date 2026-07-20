import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { setImmediate } from "node:timers";
import { fileURLToPath, URL } from "node:url";
import {
  C4OS_TOOL_NAMES,
  C4osPiSidecar,
  PI_NATIVE_VERSION,
} from "../adapter.mjs";
import {
  MAX_DIRECT_IMAGE_BYTES,
  MAX_LINE_BYTES,
  MAX_STRING_BYTES,
  assertSafeLaunch,
  encodeLine,
  parseRequestLine,
} from "../protocol.mjs";

const root = new URL("../", import.meta.url);
const generation = 7;

test("sidecar package and product manifest pin Pi 0.80.10 and the C4OS boundary", async () => {
  const packageJson = JSON.parse(
    await readFile(new URL("package.json", root), "utf8"),
  );
  const packageLock = JSON.parse(
    await readFile(new URL("package-lock.json", root), "utf8"),
  );
  const manifest = JSON.parse(
    await readFile(new URL("sidecar-manifest.json", root), "utf8"),
  );
  assert.equal(
    packageJson.dependencies["@earendil-works/pi-coding-agent"],
    "0.80.10",
  );
  assert.equal(
    packageJson.dependencies["@earendil-works/pi-agent-core"],
    "0.80.10",
  );
  assert.equal(packageJson.dependencies["@earendil-works/pi-ai"], "0.80.10");
  assert.equal(manifest.nativeVersion, PI_NATIVE_VERSION);
  assert.equal(manifest.tools, "c4os-brokered-only");
  assert.equal(manifest.extensions, "disabled");
  assert.equal(manifest.persistence, "c4os-authoritative-in-memory-worker");
  assert.deepEqual(C4OS_TOOL_NAMES, [
    "c4os_read_resource",
    "c4os_propose_action",
  ]);
  for (const [name, version] of Object.entries(packageJson.dependencies)) {
    assert.equal(packageLock.packages[`node_modules/${name}`].version, version);
    assert.match(
      packageLock.packages[`node_modules/${name}`].integrity,
      /^sha512-/,
    );
  }
});

test("bounded line parser rejects framing, unknown fields, and credential-shaped payload fields", () => {
  assert.throws(
    () => parseRequestLine(""),
    (error) => error.code === "line_bounds",
  );
  assert.throws(
    () => parseRequestLine("{}\n{}"),
    (error) => error.code === "invalid_line",
  );
  assert.throws(
    () => parseRequestLine("x".repeat(MAX_LINE_BYTES + 1)),
    (error) => error.code === "line_bounds",
  );
  assert.throws(
    () =>
      parseRequestLine(
        JSON.stringify({ ...request("health"), surprise: true }),
      ),
    (error) => error.code === "unknown_field",
  );
  assert.throws(
    () =>
      parseRequestLine(
        JSON.stringify(request("dispatch", { apiKey: "not-allowed" })),
      ),
    (error) => error.code === "secret_field",
  );
});

test("encoder bounds output and launch validation forbids argv or environment credentials", () => {
  assert.throws(
    () => encodeLine({ value: "x".repeat(MAX_LINE_BYTES) }),
    (error) => error.code === "line_bounds" || error.code === "payload_bounds",
  );
  assert.throws(
    () => assertSafeLaunch(["--token=abc123456789"], {}),
    (error) => error.code === "unsafe_launch",
  );
  assert.throws(
    () => assertSafeLaunch([], { API_KEY: "abc" }),
    (error) => error.code === "unsafe_launch",
  );
  assert.doesNotThrow(() =>
    assertSafeLaunch(["--generation=1", "--credential-fd=3"], { PATH: "/bin" }),
  );
});

test("protocol admits the 128 KiB decoded image boundary while ordinary strings remain capped at 64 KiB", () => {
  const content = Buffer.alloc(MAX_DIRECT_IMAGE_BYTES, 0xa5);
  const attachment = attachmentForContent(content);
  const line = JSON.stringify(
    dispatchRequest("", { attachments: [attachment] }),
  );
  assert.ok(Buffer.byteLength(line, "utf8") < MAX_LINE_BYTES);
  assert.deepEqual(parseRequestLine(line).payload.attachments, [attachment]);

  const oversized = Buffer.alloc(MAX_DIRECT_IMAGE_BYTES + 1, 0xa5);
  assert.throws(
    () =>
      parseRequestLine(
        JSON.stringify(
          dispatchRequest("", {
            attachments: [attachmentForContent(oversized)],
          }),
        ),
      ),
    (error) => error.code === "payload_bounds",
  );
  assert.doesNotThrow(() =>
    parseRequestLine(
      JSON.stringify(
        request("health", { message: "x".repeat(MAX_STRING_BYTES) }),
      ),
    ),
  );
  assert.throws(
    () =>
      parseRequestLine(
        JSON.stringify(
          request("health", { message: "x".repeat(MAX_STRING_BYTES + 1) }),
        ),
      ),
    (error) => error.code === "payload_bounds",
  );
});

test("health and version declare supported, degraded, and unsupported capability states", async () => {
  const { sidecar } = fixture({ credentialChannelAvailable: false });
  const health = await sidecar.handle(request("health"));
  assert.equal(health.status, "ok");
  assert.equal(health.payload.capabilities.streaming.state, "supported");
  assert.equal(
    health.payload.capabilities.providerAuthentication.state,
    "degraded",
  );
  assert.equal(
    health.payload.capabilities.nativePersistence.state,
    "unsupported",
  );
  assert.equal(
    health.payload.capabilities.nativeExtensions.state,
    "unsupported",
  );
  assert.equal(health.payload.capabilities.nativeTools.state, "unsupported");
  const version = await sidecar.handle(request("version"));
  assert.equal(version.payload.nativeVersion, "0.80.10");
  assert.equal(version.correlationId, "correlation-1");
});

test("model preflight is strict, credential-free, and does not create a session", async () => {
  const available = fixture();
  const accepted = await available.sidecar.handle(
    request("model.preflight", {
      provider: "openai",
      modelId: "gpt-4o-mini",
    }),
  );
  assert.deepEqual(accepted.payload, {
    available: true,
    provider: "openai",
    modelId: "gpt-4o-mini",
    nativeVersion: "0.80.10",
  });
  assert.deepEqual(available.driver.modelPreflights, [
    { provider: "openai", modelId: "gpt-4o-mini" },
  ]);
  assert.equal(available.driver.createSessions, 0);

  const missing = fixture();
  missing.driver.availableModels.clear();
  const unavailable = await missing.sidecar.handle(
    request("model.preflight", {
      provider: "openai",
      modelId: "gpt-4o-mini",
    }),
  );
  assert.equal(unavailable.payload.available, false);
  assert.equal(missing.driver.createSessions, 0);

  for (const payload of [
    { provider: "openai" },
    {
      provider: "openai",
      modelId: "gpt-4o-mini",
      baseUrl: "https://api.openai.com/v1",
    },
  ]) {
    const rejected = await available.sidecar.handle(
      request("model.preflight", payload),
    );
    assert.equal(rejected.status, "error");
    assert.equal(rejected.payload.code, "invalid_model_preflight");
  }
  const scoped = await available.sidecar.handle(
    request(
      "model.preflight",
      { provider: "openai", modelId: "gpt-4o-mini" },
      { sessionId: "session-must-not-exist" },
    ),
  );
  assert.equal(scoped.status, "error");
  assert.equal(scoped.payload.code, "invalid_model_preflight");
  const credential = await available.sidecar.handle(
    request("model.preflight", {
      provider: "openai",
      modelId: "gpt-4o-mini",
      credentialReference: "must-not-cross",
    }),
  );
  assert.equal(credential.status, "error");
  assert.equal(credential.payload.code, "invalid_model_preflight");
});

test("session create is C4OS-scoped and missing restart state is explicitly degraded", async () => {
  const { sidecar } = fixture();
  const missing = await sidecar.handle(
    request(
      "session.resume",
      {},
      { workspaceId: "workspace-1", sessionId: "session-1" },
    ),
  );
  assert.equal(missing.status, "ok");
  assert.equal(missing.payload.status, "degraded");
  const created = await createSession(sidecar);
  assert.equal(created.payload.persistence, "c4os-authoritative");
  const resumed = await sidecar.handle(
    request(
      "session.resume",
      {},
      { workspaceId: "workspace-1", sessionId: "session-1" },
    ),
  );
  assert.equal(resumed.payload.status, "ready");
  const wrongWorkspace = await sidecar.handle(
    request(
      "session.resume",
      {},
      { workspaceId: "workspace-2", sessionId: "session-1" },
    ),
  );
  assert.equal(wrongWorkspace.status, "error");
  assert.equal(wrongWorkspace.payload.code, "scope_mismatch");
});

test("session create requires a bounded unique canonical eligible tool scope", async () => {
  for (const eligibleTools of [
    undefined,
    null,
    "c4os_read_resource",
    ["c4os_read_resource", "c4os_read_resource"],
    ["c4os_read_resource", "c4os_propose_action", "c4os_unknown"],
    ["c4os_unknown"],
  ]) {
    const { sidecar, driver } = fixture();
    const payload = {
      modelRoute: modelRoute(),
      ...(eligibleTools === undefined ? {} : { eligibleTools }),
    };
    const rejected = await sidecar.handle(
      request("session.create", payload, {
        workspaceId: "workspace-1",
        sessionId: "session-1",
      }),
    );
    assert.equal(rejected.status, "error");
    assert.equal(rejected.payload.code, "invalid_tool_scope");
    assert.equal(driver.createSessions, 0);
  }
});

test("session tool interception enforces the exact eligible set including empty scope", async () => {
  const allowed = fixture();
  const created = await createSession(allowed.sidecar, ["c4os_read_resource"]);
  assert.equal(created.status, "ok");
  assert.deepEqual(allowed.driver.lastEligibleTools, ["c4os_read_resource"]);
  await allowed.sidecar.handle(dispatchRequest("tool"));
  await settle();
  assert.ok(
    allowed.events.some((event) => event.category === "tool.action_intent"),
  );
  await allowed.sidecar.handle(toolResolution("denied"));
  await settle();

  for (const eligibleTools of [[], ["c4os_propose_action"]]) {
    const omitted = fixture();
    const scoped = await createSession(omitted.sidecar, eligibleTools);
    assert.equal(scoped.status, "ok");
    assert.deepEqual(omitted.driver.lastEligibleTools, eligibleTools);
    await omitted.sidecar.handle(dispatchRequest("tool"));
    await settle();
    assert.equal(omitted.driver.lastBlocked, true);
    assert.equal(omitted.driver.toolBodies, 0);
    assert.equal(
      omitted.events.some((event) => event.category === "tool.action_intent"),
      false,
    );
  }
});

test("session model routes reject credential references at the worker boundary", async () => {
  const { sidecar } = fixture();
  const rejected = await sidecar.handle(
    request(
      "session.create",
      {
        modelRoute: {
          provider: "openai",
          modelId: "gpt-4o-mini",
          baseUrl: "https://api.openai.com/v1",
          credentialReference: "vault-reference",
        },
      },
      { workspaceId: "workspace-1", sessionId: "session-1" },
    ),
  );
  assert.equal(rejected.status, "error");
  assert.equal(rejected.payload.code, "invalid_model_route");
});

test("session model routes accept only bounded credential-free HTTPS base URLs", async () => {
  const { sidecar } = fixture();
  for (const [index, baseUrl] of [
    "http://127.0.0.1:9443/v1",
    "https://user:password@provider.example/v1",
    "https://provider.example/v1?credential=value",
    "https://provider.example/v1#fragment",
  ].entries()) {
    const rejected = await sidecar.handle(
      request(
        "session.create",
        {
          modelRoute: {
            provider: "openai",
            modelId: "gpt-4o-mini",
            baseUrl,
          },
        },
        { workspaceId: "workspace-1", sessionId: `session-${index}` },
      ),
    );
    assert.equal(rejected.status, "error");
    assert.equal(rejected.payload.code, "invalid_model_route");
  }
});

test("dispatch normalizes native events with full correlation and rejects stale generation events", async () => {
  const { sidecar, events } = fixture();
  await createSession(sidecar);
  const accepted = await sidecar.handle(dispatchRequest("ordinary"));
  assert.equal(accepted.payload.accepted, true);
  await settle();
  assert.ok(
    events.some(
      (event) =>
        event.category === "content.delta" && event.payload.delta === "hello",
    ),
  );
  for (const event of events) {
    assert.equal(event.processGeneration, generation);
    assert.equal(event.workspaceId, "workspace-1");
    assert.equal(event.sessionId, "session-1");
    assert.equal(event.turnId, "turn-1");
    assert.equal(event.runId, "run-1");
    assert.equal(event.correlationId, "correlation-1");
  }
  assert.equal(
    sidecar.ingestNativeEvent(
      "session-1",
      identity({ processGeneration: generation - 1 }),
      { type: "message_update" },
    ),
    false,
  );
  const health = await sidecar.handle(request("health"));
  assert.equal(health.payload.staleEventsRejected, 1);
});

test("dispatch rejects credential handles and forwards only full non-secret operation identity", async () => {
  const { sidecar, driver } = fixture();
  await createSession(sidecar);
  const forbidden = await sidecar.handle(
    dispatchRequest("ordinary", {
      credentialLeaseId: "lease-must-not-cross-ndjson",
    }),
  );
  assert.equal(forbidden.status, "error");
  assert.equal(forbidden.payload.code, "credential_handle_forbidden");

  const accepted = await sidecar.handle(
    dispatchRequest("ordinary", {
      runtimeId: "pi-production",
      providerId: "provider-openai",
    }),
  );
  assert.equal(accepted.status, "ok");
  await settle();
  assert.equal(driver.lastDispatch.runIdentity.runtimeId, "pi-production");
  assert.equal(driver.lastDispatch.runIdentity.providerId, "provider-openai");
  assert.equal(
    JSON.stringify(driver.lastDispatch).includes("credentialLeaseId"),
    false,
  );
});

test("pre-tool action intent denial prevents the broker-only tool body", async () => {
  const { sidecar, events, driver } = fixture();
  await createSession(sidecar);
  await sidecar.handle(dispatchRequest("tool"));
  await settle();
  const intent = events.find(
    (event) => event.category === "tool.action_intent",
  );
  assert.equal(intent.toolCallId, "tool-call-1");
  assert.equal(intent.payload.authority, "c4os-action-gateway-required");
  const denied = await sidecar.handle(
    toolResolution("denied", { reason: "Policy denied" }),
  );
  assert.equal(denied.status, "ok");
  assert.equal(denied.payload.executedBySidecar, false);
  await settle();
  assert.equal(driver.toolBodies, 0);
  assert.equal(driver.lastBlocked, true);
});

test("completed tool resolution returns a C4OS-produced result exactly once", async () => {
  const { sidecar, driver } = fixture();
  await createSession(sidecar);
  await sidecar.handle(dispatchRequest("tool"));
  await settle();
  const completed = await sidecar.handle(
    toolResolution("completed", {
      result: {
        content: [{ type: "text", text: "C4OS read result" }],
        details: { source: "gateway" },
      },
    }),
  );
  assert.equal(completed.payload.executedBySidecar, false);
  await settle();
  assert.equal(driver.toolBodies, 1);
  assert.equal(driver.lastToolResult.content[0].text, "C4OS read result");
  assert.throws(
    () => sidecar.consumeBrokeredResult("session-1", "run-1", "tool-call-1"),
    (error) => error.code === "missing_broker_result",
  );
});

test("stale or mismatched tool resolutions cannot release a pending tool", async () => {
  const { sidecar, driver } = fixture();
  await createSession(sidecar);
  await sidecar.handle(dispatchRequest("tool"));
  await settle();
  const stale = await sidecar.handle({
    ...toolResolution("completed", { result: { summary: "x" } }),
    correlationId: "other-correlation",
  });
  assert.equal(stale.status, "error");
  assert.equal(stale.payload.code, "stale_tool_resolution");
  assert.equal(driver.toolBodies, 0);
  await sidecar.handle(toolResolution("denied"));
  await settle();
});

test("cancel is idempotent, rejects late events, and aborts the SDK run", async () => {
  const { sidecar, events, driver } = fixture();
  await createSession(sidecar);
  await sidecar.handle(dispatchRequest("hold"));
  const first = await sidecar.handle(
    request("cancel", {}, { sessionId: "session-1", runId: "run-1" }),
  );
  assert.equal(first.payload.cancelled, true);
  const second = await sidecar.handle(
    request("cancel", {}, { sessionId: "session-1", runId: "run-1" }),
  );
  assert.equal(second.payload.cancelled, false);
  assert.equal(driver.aborts, 1);
  assert.ok(events.some((event) => event.category === "lifecycle.cancelled"));
  assert.equal(
    sidecar.ingestNativeEvent("session-1", identity(), {
      type: "message_update",
      delta: "late",
    }),
    false,
  );
});

test("direct attachment metadata reaches the driver exactly and attachment-only dispatch is accepted", async () => {
  const { sidecar, driver } = fixture();
  await createSession(sidecar);
  const attachment = directAttachment();
  const accepted = await sidecar.handle(
    dispatchRequest("", { attachments: [attachment] }),
  );
  assert.equal(accepted.status, "ok");
  assert.equal(accepted.payload.accepted, true);
  await settle();
  assert.equal(driver.lastDispatch.input, "");
  assert.deepEqual(driver.lastDispatch.attachments, [attachment]);
});

test("128 KiB decoded image reaches the native driver and one byte over is rejected first", async () => {
  const content = Buffer.alloc(MAX_DIRECT_IMAGE_BYTES, 0xa5);
  const attachment = attachmentForContent(content);
  const accepted = fixture();
  await createSession(accepted.sidecar);
  const response = await accepted.sidecar.handle(
    parseRequestLine(
      JSON.stringify(dispatchRequest("", { attachments: [attachment] })),
    ),
  );
  assert.equal(response.status, "ok");
  await settle();
  assert.deepEqual(accepted.driver.lastDispatch.attachments, [attachment]);

  const oversized = fixture();
  await createSession(oversized.sidecar);
  const over = Buffer.alloc(MAX_DIRECT_IMAGE_BYTES + 1, 0xa5);
  const rejected = await oversized.sidecar.handle(
    dispatchRequest("", { attachments: [attachmentForContent(over)] }),
  );
  assert.equal(rejected.status, "error");
  assert.equal(rejected.payload.code, "payload_bounds");
  assert.equal(oversized.driver.lastDispatch, undefined);
});

test("unknown operations, stale requests, invalid attachments, and secret-like result fields fail closed", async () => {
  const { sidecar } = fixture();
  assert.equal(
    (await sidecar.handle(request("invented.operation"))).payload.code,
    "unsupported_operation",
  );
  assert.equal(
    (
      await sidecar.handle({
        ...request("health"),
        processGeneration: generation + 1,
      })
    ).payload.code,
    "stale_generation",
  );
  await createSession(sidecar);
  const attachments = await sidecar.handle(
    dispatchRequest("", { attachments: [] }),
  );
  assert.equal(attachments.payload.code, "invalid_attachments");
  const pdf = await sidecar.handle(
    dispatchRequest("", {
      attachments: [{ ...directAttachment(), mediaType: "application/pdf" }],
    }),
  );
  assert.equal(pdf.payload.code, "invalid_attachments");
  const tampered = await sidecar.handle(
    dispatchRequest("", {
      attachments: [{ ...directAttachment(), contentBase64: "ZXZpbA==" }],
    }),
  );
  assert.equal(tampered.payload.code, "invalid_attachments");
  await sidecar.handle(dispatchRequest("tool"));
  await settle();
  const secret = await sidecar.handle(
    toolResolution("completed", { result: { access_token: "raw-secret" } }),
  );
  assert.equal(secret.status, "error");
  assert.equal(secret.payload.code, "secret_field");
  await sidecar.handle(toolResolution("denied"));
});

test("session close and shutdown release driver state and reject future work", async () => {
  const { sidecar, driver } = fixture();
  await createSession(sidecar);
  const closed = await sidecar.handle(
    request("session.close", {}, { sessionId: "session-1" }),
  );
  assert.equal(closed.payload.closed, true);
  assert.equal(driver.disposals, 1);
  const shutdown = await sidecar.handle(request("shutdown"));
  assert.equal(shutdown.payload.stopped, true);
  assert.equal(driver.shutdowns, 1);
  const version = await sidecar.handle(request("version"));
  assert.equal(version.payload.code, "sidecar_stopped");
});

function fixture({ credentialChannelAvailable = true } = {}) {
  const events = [];
  const driver = new FakePiDriver(credentialChannelAvailable);
  const sidecar = new C4osPiSidecar({
    driver,
    emitLine: (event) => events.push(event),
    processGeneration: generation,
  });
  return { sidecar, events, driver };
}

class FakePiDriver {
  constructor(credentialChannelAvailable) {
    this.credentialChannelAvailable = credentialChannelAvailable;
    this.createSessions = 0;
    this.toolBodies = 0;
    this.aborts = 0;
    this.disposals = 0;
    this.shutdowns = 0;
    this.modelPreflights = [];
    this.availableModels = new Set(["openai\u0000gpt-4o-mini"]);
  }

  async preflightModel(pair) {
    this.modelPreflights.push(globalThis.structuredClone(pair));
    return this.availableModels.has(`${pair.provider}\u0000${pair.modelId}`);
  }

  async createSession(options) {
    this.createSessions += 1;
    this.lastEligibleTools = [...options.tools];
    let held;
    return {
      nativeSessionId: `native-${options.c4osSessionId}`,
      dispatch: async ({ input, attachments, runIdentity }) => {
        this.lastDispatch = { input, attachments, runIdentity };
        if (input === "hold")
          await new Promise((resolve) => {
            held = resolve;
          });
        if (input === "tool") {
          const resolution = await options.beforeToolCall(runIdentity, {
            toolCall: { name: "c4os_read_resource", id: "tool-call-1" },
            args: { resource: "workspace:/README.md" },
          });
          this.lastBlocked = resolution?.block === true;
          if (!resolution?.block) {
            this.toolBodies += 1;
            this.lastToolResult = options.consumeBrokeredResult(
              runIdentity.runId,
              "tool-call-1",
            );
          }
        } else if (input !== "hold") {
          options.onNativeEvent(runIdentity, {
            type: "message_update",
            assistantMessageEvent: { type: "text_delta", delta: "hello" },
          });
        }
      },
      abort: async () => {
        this.aborts += 1;
        held?.();
      },
      dispose: async () => {
        this.disposals += 1;
        held?.();
      },
    };
  }

  async shutdown() {
    this.shutdowns += 1;
  }
}

async function createSession(sidecar, eligibleTools = C4OS_TOOL_NAMES) {
  return sidecar.handle(
    request(
      "session.create",
      {
        modelRoute: modelRoute(),
        eligibleTools,
      },
      { workspaceId: "workspace-1", sessionId: "session-1" },
    ),
  );
}

function modelRoute() {
  return {
    provider: "openai",
    modelId: "gpt-4o-mini",
    baseUrl: "https://api.openai.com/v1",
  };
}

function dispatchRequest(input, extraPayload = {}) {
  return request(
    "dispatch",
    { input, ...extraPayload },
    {
      workspaceId: "workspace-1",
      sessionId: "session-1",
      turnId: "turn-1",
      runId: "run-1",
    },
  );
}

function toolResolution(decision, extraPayload = {}) {
  return request(
    "tool.resolve",
    { toolCallId: "tool-call-1", decision, ...extraPayload },
    {
      sessionId: "session-1",
      runId: "run-1",
    },
  );
}

function directAttachment() {
  return {
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
}

function attachmentForContent(content) {
  return {
    attachmentId: "attachment-boundary",
    stableReference: "blob:attachment-boundary",
    displayName: "boundary.png",
    mediaType: "image/png",
    byteLength: content.length,
    contentSha256: `sha256:${createHash("sha256").update(content).digest("hex")}`,
    snapshotVersion: 1,
    contentBase64: content.toString("base64"),
  };
}

function request(operation, payload = {}, scope = {}) {
  return {
    schemaVersion: 1,
    kind: "request",
    requestId: `request-${operation.replaceAll(".", "-")}`,
    correlationId: "correlation-1",
    processGeneration: generation,
    operation,
    ...scope,
    payload,
  };
}

function identity(overrides = {}) {
  return {
    workspaceId: "workspace-1",
    sessionId: "session-1",
    turnId: "turn-1",
    runId: "run-1",
    correlationId: "correlation-1",
    processGeneration: generation,
    ...overrides,
  };
}

async function settle() {
  await new Promise((resolve) => setImmediate(resolve));
}

test("test suite resolves from the sidecar-local package root", () => {
  assert.equal(fileURLToPath(root).endsWith("/sidecars/pi/"), true);
});
