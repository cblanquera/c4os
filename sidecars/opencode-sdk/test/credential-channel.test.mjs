import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import test from "node:test";

import {
  CredentialProtocolError,
  InheritedCredentialTransport,
  ProviderCredentialChannel,
} from "../credential-channel.mjs";
import { createC4osCredentialHeadersHook } from "../c4os-tools-plugin.mjs";

class CredentialTransport extends EventEmitter {
  constructor() {
    super();
    this.destroyed = false;
    this.requests = [];
  }

  write(frame, callback) {
    this.requests.push(JSON.parse(frame.toString("utf8")));
    callback?.();
    return true;
  }

  deliver(metadata, secret, splitAt = undefined) {
    const header = Buffer.from(`${JSON.stringify(metadata)}\n`, "utf8");
    const frame = Buffer.concat([header, Buffer.from(secret, "utf8")]);
    if (splitAt === undefined) {
      this.emit("data", frame);
      return;
    }
    this.emit("data", frame.subarray(0, splitAt));
    this.emit("data", frame.subarray(splitAt));
  }

  reject(requestId, code) {
    this.emit(
      "data",
      Buffer.from(
        `${JSON.stringify({
          schemaVersion: 3,
          kind: "providerCredentialRejected",
          requestId,
          code,
        })}\n`,
      ),
    );
  }

  destroy() {
    this.destroyed = true;
    this.emit("close");
  }
}

function requestIds(...ids) {
  let index = 0;
  return () => ids[index++];
}

function channel(transport, generation, ids, options = {}) {
  return new ProviderCredentialChannel(transport, generation, {
    requestId: requestIds(...ids),
    ...options,
  });
}

function response(request, overrides = {}) {
  return {
    schemaVersion: 3,
    kind: "providerCredential",
    requestId: request.requestId,
    leaseId: `lease:${request.requestId}`,
    authorizationId: "authorization:active-attempt",
    processGeneration: request.processGeneration,
    nativeSessionId: request.nativeSessionId,
    providerId: request.providerId,
    modelId: request.modelId,
    operationId: request.operationId,
    nativeMessageId: request.nativeMessageId,
    headerName: "Authorization",
    headerPrefix: "Bearer ",
    secretLength: 35,
    ttlMs: 15_000,
    ...overrides,
  };
}

function hookInput(providerId = "provider-openai", overrides = {}) {
  return {
    sessionID: "native-session-1",
    agent: "c4os",
    model: { providerID: providerId, id: "gpt-5" },
    provider: { info: { id: providerId } },
    message: { id: "msg_attempt-correlation-1" },
    ...overrides,
  };
}

test("chat.headers requests then consumes one exact Rust-authorized lease", async () => {
  const secret = "sk-c4os-provider-0123456789abcdef";
  const transport = new CredentialTransport();
  const credentials = channel(transport, 7, ["request:initial"]);
  const hook = createC4osCredentialHeadersHook(credentials);
  const output = { headers: {} };
  const pending = hook(hookInput(), output);

  assert.deepEqual(transport.requests, [
    {
      schemaVersion: 3,
      kind: "providerCredentialRequest",
      requestId: "request:initial",
      processGeneration: 7,
      nativeSessionId: "native-session-1",
      providerId: "provider-openai",
      modelId: "gpt-5",
      operationId: "attempt-correlation-1",
      nativeMessageId: "msg_attempt-correlation-1",
    },
  ]);
  transport.deliver(
    response(transport.requests[0], {
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
    19,
  );
  await pending;

  assert.equal(output.headers.Authorization, `Bearer ${secret}`);
  assert.equal(credentials.pendingCount, 0);
  assert.equal(credentials.consumedCount, 1);
});

test("each same-attempt continuation requests a fresh one-use lease", async () => {
  const initialSecret = "sk-c4os-initial-0123456789abcdef";
  const continuationSecret = "sk-c4os-continue-0123456789abcde";
  const transport = new CredentialTransport();
  const credentials = channel(transport, 8, [
    "request:initial",
    "request:continuation",
  ]);
  const hook = createC4osCredentialHeadersHook(credentials);

  const initialOutput = { headers: {} };
  const initial = hook(hookInput(), initialOutput);
  transport.deliver(
    response(transport.requests[0], {
      leaseId: "lease:initial",
      secretLength: Buffer.byteLength(initialSecret),
    }),
    initialSecret,
  );
  await initial;

  const continuationOutput = { headers: {} };
  const continuation = hook(hookInput(), continuationOutput);
  transport.deliver(
    response(transport.requests[1], {
      leaseId: "lease:continuation",
      secretLength: Buffer.byteLength(continuationSecret),
    }),
    continuationSecret,
  );
  await continuation;

  assert.equal(initialOutput.headers.Authorization, `Bearer ${initialSecret}`);
  assert.equal(
    continuationOutput.headers.Authorization,
    `Bearer ${continuationSecret}`,
  );
  assert.equal(credentials.consumedCount, 2);
  assert.equal(
    transport.requests[0].operationId,
    transport.requests[1].operationId,
  );
  assert.notEqual(
    transport.requests[0].requestId,
    transport.requests[1].requestId,
  );
});

test("matches provider-specific header schemes", async () => {
  const secret = "anthropic-c4os-provider-secret-0123";
  const transport = new CredentialTransport();
  const credentials = channel(transport, 9, ["request:anthropic"]);
  const hook = createC4osCredentialHeadersHook(credentials);
  const output = { headers: {} };
  const pending = hook(hookInput("provider-anthropic"), output);
  transport.deliver(
    response(transport.requests[0], {
      leaseId: "lease:anthropic",
      headerName: "x-api-key",
      headerPrefix: "",
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
  );
  await pending;
  assert.deepEqual(output.headers, { "x-api-key": secret });
});

test("fails closed on Rust rejection and exact binding substitution", async () => {
  const transport = new CredentialTransport();
  const credentials = channel(transport, 10, [
    "request:rejected",
    "request:substituted",
  ]);
  const first = credentials.request({
    nativeSessionId: "native-session-1",
    providerId: "provider-openai",
    modelId: "gpt-5",
    operationId: "attempt-correlation-1",
    nativeMessageId: "msg_attempt-correlation-1",
  });
  transport.reject("request:rejected", "binding_mismatch");
  await assert.rejects(first, /binding_mismatch/u);

  const second = credentials.request({
    nativeSessionId: "native-session-1",
    providerId: "provider-openai",
    modelId: "gpt-5",
    operationId: "attempt-correlation-1",
    nativeMessageId: "msg_attempt-correlation-1",
  });
  const secret = "sk-c4os-provider-0123456789abcdef";
  transport.deliver(
    response(transport.requests[1], {
      modelId: "substituted-model",
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
  );
  await assert.rejects(second, /credential_binding_substitution/u);
});

test("rejects expired responses and replayed lease identities", async () => {
  let now = 100;
  const transport = new CredentialTransport();
  const credentials = channel(
    transport,
    11,
    ["request:expired", "request:first", "request:replay"],
    { now: () => now },
  );
  const binding = {
    nativeSessionId: "native-session-1",
    providerId: "provider-openai",
    modelId: "gpt-5",
    operationId: "attempt-correlation-1",
    nativeMessageId: "msg_attempt-correlation-1",
  };
  const secret = "sk-c4os-provider-0123456789abcdef";

  const expired = credentials.request(binding);
  now = 106;
  transport.deliver(
    response(transport.requests[0], {
      ttlMs: 5,
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
  );
  await assert.rejects(expired, /credential_lease_expired/u);

  const first = credentials.request(binding);
  transport.deliver(
    response(transport.requests[1], {
      leaseId: "lease:one-use",
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
  );
  await first;
  const replay = credentials.request(binding);
  transport.deliver(
    response(transport.requests[2], {
      leaseId: "lease:one-use",
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
  );
  await assert.rejects(replay, /lease_replay/u);
});

test("rejects worker-side operation substitution before writing a request", () => {
  const transport = new CredentialTransport();
  const credentials = channel(transport, 12, ["request:unused"]);
  assert.throws(
    () =>
      credentials.request({
        nativeSessionId: "native-session-1",
        providerId: "provider-openai",
        modelId: "gpt-5",
        operationId: "attempt-substituted",
        nativeMessageId: "msg_attempt-correlation-1",
      }),
    /operation_binding/u,
  );
  assert.equal(transport.requests.length, 0);
});

test("does not overwrite a provider credential header", async () => {
  const secret = "sk-c4os-provider-0123456789abcdef";
  const transport = new CredentialTransport();
  const credentials = channel(transport, 13, ["request:header-conflict"]);
  const hook = createC4osCredentialHeadersHook(credentials);
  const output = { headers: { authorization: "existing" } };
  const pending = hook(hookInput(), output);
  transport.deliver(
    response(transport.requests[0], {
      secretLength: Buffer.byteLength(secret),
    }),
    secret,
  );
  await assert.rejects(pending, /already exists/u);
  assert.equal(output.headers.authorization, "existing");
});

test("typed protocol failures remain explicit", () => {
  const error = new CredentialProtocolError("binding_mismatch");
  assert.equal(error.code, "binding_mismatch");
  assert.match(error.message, /binding_mismatch/u);
});

test("inherited descriptor transport preserves partial I/O and scrubs fd metadata", async () => {
  const secret = "sk-c4os-provider-0123456789abcdef";
  const requestId = "request:descriptor";
  const request = {
    schemaVersion: 3,
    kind: "providerCredentialRequest",
    requestId,
    processGeneration: 17,
    nativeSessionId: "native-session-1",
    providerId: "provider-openai",
    modelId: "gpt-5",
    operationId: "attempt-correlation-1",
    nativeMessageId: "msg_attempt-correlation-1",
  };
  const encodedResponse = Buffer.concat([
    Buffer.from(
      `${JSON.stringify(
        response(request, {
          leaseId: "lease:descriptor",
          secretLength: Buffer.byteLength(secret),
        }),
      )}\n`,
      "utf8",
    ),
    Buffer.from(secret, "utf8"),
  ]);
  const written = [];
  let responseOffset = 0;
  let closed = false;
  const operations = {
    writeSync(_fd, buffer, offset, length) {
      const accepted = Math.min(length, 7);
      written.push(Buffer.from(buffer.subarray(offset, offset + accepted)));
      return accepted;
    },
    readSync(_fd, buffer, offset, length) {
      const accepted = Math.min(
        length,
        11,
        encodedResponse.length - responseOffset,
      );
      encodedResponse.copy(
        buffer,
        offset,
        responseOffset,
        responseOffset + accepted,
      );
      responseOffset += accepted;
      return accepted;
    },
    closeSync() {
      closed = true;
    },
  };
  const environment = {
    C4OS_OPENCODE_CREDENTIAL_FD: "64",
    C4OS_OPENCODE_PROCESS_GENERATION: "17",
  };
  const credentials = ProviderCredentialChannel.fromEnvironment(
    environment,
    operations,
  );
  credentials.requestId = () => requestId;
  const delivered = await credentials.request({
    nativeSessionId: request.nativeSessionId,
    providerId: request.providerId,
    modelId: request.modelId,
    operationId: request.operationId,
    nativeMessageId: request.nativeMessageId,
  });

  assert.equal(environment.C4OS_OPENCODE_CREDENTIAL_FD, undefined);
  assert.equal(
    JSON.parse(Buffer.concat(written).toString("utf8")).requestId,
    requestId,
  );
  assert.deepEqual(delivered, {
    headerName: "Authorization",
    headerValue: `Bearer ${secret}`,
  });
  credentials.dispose();
  assert.equal(closed, true);
});

test("inherited descriptor transport fails closed on EOF", () => {
  const transport = new InheritedCredentialTransport(65, {
    writeSync: (_fd, _buffer, _offset, length) => length,
    readSync: () => 0,
    closeSync: () => {},
  });
  transport.on("error", () => {});
  let failure;
  assert.equal(
    transport.write(Buffer.from("request\n"), (error) => {
      failure = error;
    }),
    false,
  );
  assert.match(failure.message, /transport_closed/u);
});
