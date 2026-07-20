import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { EventEmitter } from "node:events";
import test from "node:test";
import { rootCertificates } from "node:tls";

import {
  BrokerProtocolError,
  C4OS_TOOL_IDS,
  FdBrokerChannel,
  InheritedBrokerTransport,
} from "../broker-channel.mjs";
import {
  TestTlsTrustCapabilities,
  createC4osBrokerTools,
  createC4osCredentialHeadersHook,
  createC4osPluginHooks,
} from "../c4os-tools-plugin.mjs";

class BrokerTransport extends EventEmitter {
  constructor(onFrame) {
    super();
    this.onFrame = onFrame;
    this.writes = [];
  }

  write(encoded) {
    const frame = JSON.parse(encoded.trim());
    this.writes.push(frame);
    this.onFrame?.(frame, this);
    return true;
  }

  reply(frame) {
    queueMicrotask(() =>
      this.emit("data", Buffer.from(`${JSON.stringify(frame)}\n`)),
    );
  }
}

const context = (abort = new AbortController().signal) => ({
  sessionID: "session-1",
  messageID: "message-1",
  agent: "test",
  directory: "/tmp/project",
  worktree: "/tmp/project",
  abort,
  metadata() {},
  async ask() {},
});

function generatedClient(onRead = () => {}) {
  return {
    tool: {
      async ids() {
        onRead();
        return { data: ["bash", ...C4OS_TOOL_IDS] };
      },
    },
  };
}

const testTlsCa = rootCertificates[0];
const testTlsBaseUrl = "https://127.0.0.1:4443/v1";

function sha256(value) {
  return `sha256:${createHash("sha256").update(value, "utf8").digest("hex")}`;
}

function testTlsDescriptor(capabilityOverrides = {}, envelopeOverrides = {}) {
  return Buffer.from(
    JSON.stringify({
      schemaVersion: 1,
      capabilities: [
        {
          providerId: "provider-c4os-test",
          nativeProviderId: "openai",
          baseUrl: testTlsBaseUrl,
          baseUrlSha256: sha256(testTlsBaseUrl),
          tlsCaPem: testTlsCa,
          tlsCaSha256: sha256(testTlsCa),
          ...capabilityOverrides,
        },
      ],
      ...envelopeOverrides,
    }),
  );
}

function consumeTestTlsDescriptor(encoded) {
  let offset = 0;
  let closed = false;
  const environment = { C4OS_OPENCODE_TEST_TLS_TRUST_FD: "68" };
  const capability = TestTlsTrustCapabilities.fromEnvironment(environment, {
    readSync(_fd, buffer, bufferOffset, length) {
      const count = Math.min(length, 7, encoded.length - offset);
      if (count === 0) return 0;
      encoded.copy(buffer, bufferOffset, offset, offset + count);
      offset += count;
      return count;
    },
    closeSync(fd) {
      assert.equal(fd, 68);
      closed = true;
    },
  });
  assert.equal(environment.C4OS_OPENCODE_TEST_TLS_TRUST_FD, undefined);
  assert.equal(closed, true);
  return capability;
}

test("materializes exactly the two C4OS broker tools", () => {
  const tools = createC4osBrokerTools({ request() {} }, generatedClient());
  assert.deepEqual(Object.keys(tools), C4OS_TOOL_IDS);
});

test("correlates a proposal and result without executing an external effect", async () => {
  let externalEffects = 0;
  const transport = new BrokerTransport((frame, socket) => {
    assert.equal(frame.kind, "proposal");
    assert.equal(frame.tool, "c4os_propose_action");
    assert.equal(frame.processGeneration, 7);
    socket.reply({
      schemaVersion: 1,
      kind: "result",
      correlationId: frame.correlationId,
      tool: frame.tool,
      status: "result",
      payload: { disposition: "accepted_for_evaluation" },
    });
  });
  let registryReads = 0;
  const tools = createC4osBrokerTools(
    new FdBrokerChannel(transport, 7),
    generatedClient(() => {
      registryReads += 1;
    }),
  );
  const result = await tools.c4os_propose_action.execute(
    { operation: "window.focus", target: "workspace-main" },
    context(),
  );

  assert.equal(externalEffects, 0);
  assert.equal(
    registryReads,
    1,
    "the generated SDK registry gates every tool execution",
  );
  assert.match(result.output, /accepted_for_evaluation/u);
  assert.equal(
    result.metadata.c4osCorrelationId,
    transport.writes[0].correlationId,
  );
});

test("returns a broker denial before any effect", async () => {
  let externalEffects = 0;
  const transport = new BrokerTransport((frame, socket) =>
    socket.reply({
      schemaVersion: 1,
      kind: "result",
      correlationId: frame.correlationId,
      tool: frame.tool,
      status: "denied",
      reasonCode: "policy_denied",
    }),
  );
  const tools = createC4osBrokerTools(
    new FdBrokerChannel(transport, 8),
    generatedClient(),
  );
  const result = await tools.c4os_read_resource.execute(
    { resource: "workspace.summary" },
    context(),
  );

  assert.equal(externalEffects, 0);
  assert.match(result.output, /policy_denied/u);
});

test("rejects secret-bearing frames and fails the channel closed", async () => {
  const transport = new BrokerTransport((frame, socket) =>
    socket.reply({
      schemaVersion: 1,
      kind: "result",
      correlationId: frame.correlationId,
      tool: frame.tool,
      status: "result",
      payload: { apiKey: "not-returnable" },
    }),
  );
  const channel = new FdBrokerChannel(transport, 9);

  await assert.rejects(
    channel.request(
      "c4os_read_resource",
      { resource: "workspace.summary" },
      { sessionId: "session-1", messageId: "message-1" },
    ),
    (error) =>
      error instanceof BrokerProtocolError && error.code === "secret_field",
  );
  assert.throws(
    () =>
      channel.request(
        "c4os_read_resource",
        { resource: "workspace.summary" },
        { sessionId: "session-1", messageId: "message-1" },
      ),
    /transport_closed/u,
  );
});

test("cancels an in-flight request with the same correlation binding", async () => {
  const controller = new AbortController();
  const transport = new BrokerTransport();
  const channel = new FdBrokerChannel(transport, 10);
  const pending = channel.request(
    "c4os_propose_action",
    { operation: "window.focus", target: "workspace-main" },
    { sessionId: "session-1", messageId: "message-1" },
    { signal: controller.signal },
  );
  controller.abort();

  await assert.rejects(
    pending,
    (error) =>
      error instanceof BrokerProtocolError && error.code === "cancelled",
  );
  assert.equal(transport.writes.length, 2);
  assert.equal(transport.writes[1].kind, "cancel");
  assert.equal(
    transport.writes[1].correlationId,
    transport.writes[0].correlationId,
  );
});

test("inherited broker descriptor serializes partial writes and fragmented reads", async () => {
  const response = Buffer.from('{"kind":"result"}\n', "utf8");
  const writes = [];
  let responseOffset = 0;
  let closed = false;
  const transport = new InheritedBrokerTransport(66, {
    write(_fd, buffer, offset, length, _position, callback) {
      const accepted = Math.min(length, 3);
      writes.push(Buffer.from(buffer.subarray(offset, offset + accepted)));
      queueMicrotask(() => callback(undefined, accepted));
    },
    read(_fd, buffer, offset, length, _position, callback) {
      if (responseOffset >= response.length) return;
      const accepted = Math.min(length, 5, response.length - responseOffset);
      response.copy(buffer, offset, responseOffset, responseOffset + accepted);
      responseOffset += accepted;
      queueMicrotask(() => callback(undefined, accepted));
    },
    close(_fd, callback) {
      closed = true;
      queueMicrotask(callback);
    },
  });
  transport.on("error", () => {});
  const received = new Promise((resolve) =>
    transport.on("data", (chunk) => resolve(Buffer.from(chunk))),
  );
  transport.write("proposal-frame\n");

  assert.equal((await received).toString("utf8"), '{"kin');
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(Buffer.concat(writes).toString("utf8"), "proposal-frame\n");
  transport.destroy();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(closed, true);
});

test("broker environment capture removes inherited fd metadata", () => {
  const environment = {
    C4OS_OPENCODE_BROKER_FD: "67",
    C4OS_OPENCODE_PROCESS_GENERATION: "19",
  };
  const channel = FdBrokerChannel.fromEnvironment(environment, {
    read() {},
    write() {},
    close(_fd, callback) {
      callback();
    },
  });
  assert.equal(environment.C4OS_OPENCODE_BROKER_FD, undefined);
  channel.close();
});

test("plugin disposal closes both private descriptor capabilities", async () => {
  let brokerClosed = false;
  let credentialsClosed = false;
  const hooks = createC4osPluginHooks(
    {
      client: generatedClient(),
      serverUrl: "http://127.0.0.1:49123/",
    },
    {
      request() {},
      close() {
        brokerClosed = true;
      },
    },
    {
      request() {},
      dispose() {
        credentialsClosed = true;
      },
    },
  );

  await hooks.dispose();
  assert.equal(brokerClosed, true);
  assert.equal(credentialsClosed, true);
});

test("test TLS trust is absent from the normal production plugin route", () => {
  const environment = {};
  assert.equal(
    TestTlsTrustCapabilities.fromEnvironment(environment),
    undefined,
  );
  const hooks = createC4osPluginHooks(
    {
      client: generatedClient(),
      serverUrl: "http://127.0.0.1:49123/",
    },
    { request() {}, close() {} },
    { request() {}, dispose() {} },
  );
  assert.equal(hooks.config, undefined);
});

test("test TLS descriptor is synchronously consumed and installs one exact provider fetch", async () => {
  const capability = consumeTestTlsDescriptor(testTlsDescriptor());
  const calls = [];
  const config = { provider: { openai: { options: { timeout: 1000 } } } };
  capability.configure(config, async (...args) => {
    calls.push(args);
    return new Response("ok");
  });
  assert.equal(config.provider.openai.options.baseURL, testTlsBaseUrl);
  const response = await config.provider.openai.options.fetch(
    `${testTlsBaseUrl}/responses?stream=true`,
    { method: "POST" },
  );
  assert.equal(await response.text(), "ok");
  assert.equal(calls.length, 1);
  assert.equal(calls[0][1].redirect, "error");
  assert.deepEqual(calls[0][1].tls, {
    ca: testTlsCa,
    rejectUnauthorized: true,
  });
  assert.equal(capability.credentialProviderId("openai"), "provider-c4os-test");
});

test("test TLS descriptor rejects malformed, unknown, and digest-mismatched content after closing", () => {
  for (const encoded of [
    Buffer.from("not-json"),
    testTlsDescriptor({}, { unknown: true }),
    testTlsDescriptor({ tlsCaSha256: `sha256:${"0".repeat(64)}` }),
    testTlsDescriptor({ baseUrlSha256: `sha256:${"0".repeat(64)}` }),
    testTlsDescriptor({
      baseUrl: "https://api.openai.com/v1",
      baseUrlSha256: sha256("https://api.openai.com/v1"),
    }),
  ]) {
    assert.throws(
      () => consumeTestTlsDescriptor(encoded),
      /test TLS trust rejected/u,
    );
  }
});

test("test TLS configuration conflict and replay leave credential delivery failed closed", async () => {
  const conflicting = consumeTestTlsDescriptor(testTlsDescriptor());
  let credentialRequests = 0;
  const credentialHook = createC4osCredentialHeadersHook(
    {
      request() {
        credentialRequests += 1;
      },
    },
    conflicting,
  );
  assert.throws(
    () =>
      conflicting.configure({
        provider: { openai: { options: { fetch: async () => {} } } },
      }),
    /provider_configuration_conflict/u,
  );
  assert.throws(
    () => conflicting.credentialProviderId("openai"),
    /configuration_inactive/u,
  );
  await assert.rejects(
    credentialHook(
      {
        sessionID: "native-session-1",
        model: { providerID: "openai", id: "gpt-5" },
        message: { id: "msg_attempt-1" },
      },
      { headers: {} },
    ),
    /configuration_inactive/u,
  );
  assert.equal(credentialRequests, 0);

  const replayed = consumeTestTlsDescriptor(testTlsDescriptor());
  replayed.configure({}, async () => new Response("ok"));
  assert.throws(
    () => replayed.configure({}, async () => new Response("ok")),
    /configuration_replay/u,
  );
  assert.throws(
    () => replayed.credentialProviderId("openai"),
    /configuration_inactive/u,
  );
});

test("test TLS fetch rejects another origin, path escape, and caller TLS overrides", () => {
  const capability = consumeTestTlsDescriptor(testTlsDescriptor());
  const config = {};
  capability.configure(config, async () => new Response("ok"));
  const providerFetch = config.provider.openai.options.fetch;
  assert.throws(
    () => providerFetch("https://example.com/v1/responses"),
    /request_origin_rejected/u,
  );
  assert.throws(
    () => providerFetch("https://127.0.0.1:4443/v10/responses"),
    /request_origin_rejected/u,
  );
  assert.throws(
    () =>
      providerFetch(`${testTlsBaseUrl}/responses`, {
        tls: { rejectUnauthorized: false },
      }),
    /caller_tls_rejected/u,
  );
});
