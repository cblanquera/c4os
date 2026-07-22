import { Buffer } from "node:buffer";
import { createHash, X509Certificate } from "node:crypto";
import { closeSync, createReadStream, readSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { clearTimeout, setTimeout } from "node:timers";
import * as tls from "node:tls";
import { URL } from "node:url";
import { TextDecoder } from "node:util";

const MAX_LEASES = 32;
const MAX_CREDENTIAL_SECRET_BYTES = 64 * 1024;
const MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES = 256 * 1024;
const MAX_TEST_TLS_TRUST_CAPABILITIES = 16;
const MAX_TEST_TLS_CA_BYTES = 128 * 1024;
const MAX_NATIVE_TOOL_CALL_ID_BYTES = 1024;
const MAX_TIMER_DELAY_MS = 2_147_483_647;
const CREDENTIAL_READY_TIMEOUT_MS = 2_000;
const CREDENTIAL_HEADER_BYTES = 21;
const CREDENTIAL_MAGIC = Buffer.from("C4OSCRED", "ascii");
const CREDENTIAL_SCHEMA_VERSION = 1;
const ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:@-]{0,127}$/;
const SHA256_PATTERN = /^sha256:[0-9a-f]{64}$/;
const TEST_TLS_TRUST_DESCRIPTOR_KEYS = Object.freeze([
  "schemaVersion",
  "capabilities",
]);
const TEST_TLS_TRUST_CAPABILITY_KEYS = Object.freeze([
  "providerId",
  "nativeProviderId",
  "baseUrl",
  "baseUrlSha256",
  "tlsCaPem",
  "tlsCaSha256",
]);
const PACKAGE_VERSION = "0.80.10";
const SDK_PACKAGES = Object.freeze([
  ["@earendil-works/pi-agent-core", "@earendil-works/pi-agent-core"],
  ["@earendil-works/pi-coding-agent", "@earendil-works/pi-coding-agent"],
  ["@earendil-works/pi-ai/compat", "@earendil-works/pi-ai"],
]);
const fatalUtf8 = new TextDecoder("utf-8", { fatal: true });

/** Owns an actual Pi SDK Agent while preserving the C4OS authority boundary. */
export class PiSdkDriver {
  #credentials;
  #sdk;
  #testTlsTrustCapabilities;

  /** Creates a driver with a narrow credential broker and optional test SDK. */
  constructor({
    credentialBroker = undefined,
    sdk = undefined,
    testTlsTrustCapabilities = undefined,
  } = {}) {
    this.#credentials = credentialBroker;
    this.#sdk = sdk;
    this.#testTlsTrustCapabilities =
      testTlsTrustCapabilities === undefined
        ? undefined
        : validateTestTlsTrustRoutes(testTlsTrustCapabilities);
  }

  /** Reports whether the dedicated operation credential channel is present. */
  get credentialChannelAvailable() {
    return Boolean(this.#credentials);
  }

  /** Resolves one exact pinned-catalog model without provider I/O or Agent creation. */
  async preflightModel({ provider, modelId }) {
    const sdk = this.#sdk ?? (await loadSdk());
    this.#sdk = sdk;
    const model = sdk.getModel(provider, modelId);
    return Boolean(
      model && model.provider === provider && model.id === modelId,
    );
  }

  /** Constructs one Pi Agent for a C4OS-owned session without provider I/O. */
  async createSession(options) {
    const sdk = this.#sdk ?? (await loadSdk());
    this.#sdk = sdk;
    const modelRoute = Object.freeze({ ...options.modelRoute });
    const catalogModel = sdk.getModel(modelRoute.provider, modelRoute.modelId);
    if (!catalogModel)
      throw new Error("The selected Pi model route is unavailable");
    const model = { ...catalogModel, baseUrl: modelRoute.baseUrl };

    let activeRun;
    let activeCredential;
    let dispatchAbort;
    let dispatchSettled;
    let settleDispatch;
    let samplingAbort;
    let samplingSettled;
    let settleSampling;
    const tools = options.tools.map((toolName) => ({
      name: toolName,
      label:
        toolName === "c4os_read_resource"
          ? "Read through C4OS"
          : "Propose an action to C4OS",
      description:
        toolName === "c4os_read_resource"
          ? "Request bounded resource content through the C4OS broker."
          : "Request an exact effect through the C4OS Action Gateway.",
      parameters: toolParameters(sdk, toolName),
      executionMode: "sequential",
      execute: async (toolCallId) => {
        if (!activeRun)
          throw new Error("No active C4OS run owns this Pi tool call");
        const result = options.consumeBrokeredResult(
          activeRun.runId,
          c4osToolCallId(toolCallId),
        );
        return normalizeToolResult(result);
      },
    }));

    const agent = new sdk.Agent({
      sessionId: options.c4osSessionId,
      initialState: {
        systemPrompt:
          "You are the Pi runtime inside C4OS. Use only the listed C4OS-brokered tools. C4OS owns policy, effects, credentials, and persistence.",
        model,
        thinkingLevel: "medium",
        tools,
        messages: [],
      },
      convertToLlm: sdk.convertToLlm,
      streamFn: sdk.streamSimple,
      getApiKey: async (provider) => {
        if (!activeCredential) return undefined;
        return activeCredential.get(provider);
      },
      toolExecution: "sequential",
      beforeToolCall: async (context) => {
        if (!activeRun)
          return {
            block: true,
            reason: "No active C4OS run owns this tool proposal",
          };
        const nativeToolCallId = context?.toolCall?.id;
        const brokerContext =
          typeof nativeToolCallId === "string"
            ? {
                ...context,
                toolCall: {
                  ...context.toolCall,
                  id: c4osToolCallId(nativeToolCallId),
                },
              }
            : context;
        return options.beforeToolCall(activeRun, brokerContext);
      },
    });
    agent.subscribe((event) => {
      if (activeRun) options.onNativeEvent(activeRun, event);
    });

    return {
      nativeSessionId: agent.sessionId ?? `pi:${options.c4osSessionId}`,
      dispatch: async ({ input, attachments = [], runIdentity }) => {
        if (activeRun) throw new Error("Pi session already has an active run");
        if (this.#testTlsTrustCapabilities) {
          assertTestTlsTrustRoute(
            this.#testTlsTrustCapabilities,
            runIdentity,
            modelRoute,
          );
        }
        const hasCredentialOperation =
          runIdentity.runtimeId !== undefined ||
          runIdentity.providerId !== undefined;
        if (
          hasCredentialOperation &&
          (!runIdentity.runtimeId || !runIdentity.providerId)
        ) {
          throw new Error("Credential operation identity is incomplete");
        }
        activeRun = runIdentity;
        dispatchAbort = new AbortController();
        dispatchSettled = new Promise((resolve) => {
          settleDispatch = resolve;
        });
        let operationCredential;
        try {
          operationCredential = hasCredentialOperation
            ? await this.#credentials?.claimForOperation(
                runIdentity,
                modelRoute.provider,
                CREDENTIAL_READY_TIMEOUT_MS,
              )
            : undefined;
          if (hasCredentialOperation && !operationCredential) {
            throw new Error("Credential lease channel is unavailable");
          }
          dispatchAbort.signal.throwIfAborted();
          activeCredential = operationCredential;
          await agent.prompt(
            input,
            attachments.map((attachment) => ({
              type: "image",
              data: attachment.contentBase64,
              mimeType: attachment.mediaType,
            })),
          );
          await agent.waitForIdle();
        } finally {
          operationCredential?.release();
          activeRun = undefined;
          activeCredential = undefined;
          dispatchAbort = undefined;
          settleDispatch?.();
          dispatchSettled = undefined;
          settleDispatch = undefined;
        }
      },
      abort: async () => {
        const settled = dispatchSettled;
        dispatchAbort?.abort();
        agent.abort();
        await settled;
        await agent.waitForIdle();
      },
      sample: async ({
        messages,
        systemPrompt,
        maxTokens,
        temperature,
        runIdentity,
      }) => {
        if (activeRun) throw new Error("Pi session already has an active run");
        if (this.#testTlsTrustCapabilities) {
          assertTestTlsTrustRoute(
            this.#testTlsTrustCapabilities,
            runIdentity,
            modelRoute,
          );
        }
        if (!runIdentity.runtimeId || !runIdentity.providerId) {
          throw new Error("Sampling credential operation identity is incomplete");
        }
        activeRun = runIdentity;
        samplingAbort = new AbortController();
        samplingSettled = new Promise((resolve) => {
          settleSampling = resolve;
        });
        let operationCredential;
        try {
          operationCredential = await this.#credentials?.claimForOperation(
            runIdentity,
            modelRoute.provider,
            CREDENTIAL_READY_TIMEOUT_MS,
          );
          if (!operationCredential) {
            throw new Error("Credential lease channel is unavailable");
          }
          activeCredential = operationCredential;
          samplingAbort.signal.throwIfAborted();
          const stream = sdk.streamSimple(
            model,
            {
              ...(systemPrompt === undefined ? {} : { systemPrompt }),
              messages: messages.map((message, index) =>
                samplingMessage(model, message, index + 1),
              ),
              tools: [],
            },
            {
              apiKey: operationCredential.get(modelRoute.provider),
              maxTokens,
              ...(temperature === undefined ? {} : { temperature }),
              signal: samplingAbort.signal,
              maxRetries: 0,
            },
          );
          const result = await stream.result();
          return normalizeSamplingResult(result);
        } finally {
          operationCredential?.release();
          activeRun = undefined;
          activeCredential = undefined;
          samplingAbort = undefined;
          settleSampling?.();
          samplingSettled = undefined;
          settleSampling = undefined;
        }
      },
      abortSampling: async () => {
        const settled = samplingSettled;
        samplingAbort?.abort();
        await settled;
      },
      dispose: async () => {
        const dispatchDone = dispatchSettled;
        dispatchAbort?.abort();
        agent.abort();
        await dispatchDone;
        const settled = samplingSettled;
        samplingAbort?.abort();
        await settled;
        await agent.waitForIdle();
        agent.reset();
      },
    };
  }
}

function samplingMessage(model, message, timestamp) {
  if (message.role === "user") {
    return { role: "user", content: message.text, timestamp };
  }
  return {
    role: "assistant",
    content: [{ type: "text", text: message.text }],
    api: model.api,
    provider: model.provider,
    model: model.id,
    usage: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
    },
    stopReason: "stop",
    timestamp,
  };
}

function normalizeSamplingResult(result) {
  if (!result || result.role !== "assistant") {
    throw new Error("Pi sampling returned no assistant message");
  }
  if (result.stopReason === "error") {
    throw new Error(result.errorMessage || "Pi sampling failed");
  }
  if (result.stopReason === "aborted") {
    throw new Error("Pi sampling was cancelled");
  }
  if (!new Set(["stop", "length"]).has(result.stopReason)) {
    throw new Error("Pi sampling returned an unsupported stop reason");
  }
  if (!Array.isArray(result.content)) {
    throw new Error("Pi sampling returned malformed content");
  }
  const text = [];
  for (const content of result.content) {
    if (content?.type === "thinking") continue;
    if (content?.type !== "text" || typeof content.text !== "string") {
      throw new Error("Pi sampling returned unsupported non-text content");
    }
    text.push(content.text);
  }
  if (text.length === 0) throw new Error("Pi sampling returned no text");
  if (typeof result.model !== "string" || result.model.length === 0) {
    throw new Error("Pi sampling returned a malformed model identity");
  }
  return {
    text: text.join(""),
    model: result.model,
    stopReason: result.stopReason === "length" ? "maxTokens" : "endTurn",
  };
}

/**
 * Consumes one inherited, test-only TLS trust descriptor before any lazy SDK
 * import and returns only its non-secret dispatch route bindings.
 */
export function consumeTestTlsTrustDescriptor(fd) {
  if (!Number.isSafeInteger(fd) || fd < 3) {
    throw new Error("Test TLS trust descriptor fd is invalid");
  }
  const encoded = Buffer.alloc(MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES + 1);
  let encodedBytes = 0;
  try {
    while (encodedBytes < encoded.length) {
      const read = readSync(
        fd,
        encoded,
        encodedBytes,
        encoded.length - encodedBytes,
        null,
      );
      if (read === 0) break;
      encodedBytes += read;
    }
  } finally {
    closeSync(fd);
  }
  try {
    if (
      encodedBytes === 0 ||
      encodedBytes > MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES
    ) {
      throw new Error("Test TLS trust descriptor exceeds its bound");
    }
    let descriptor;
    try {
      descriptor = JSON.parse(
        fatalUtf8.decode(encoded.subarray(0, encodedBytes)),
      );
    } catch {
      throw new Error("Test TLS trust descriptor is invalid");
    }
    const validated = validateTestTlsTrustDescriptor(descriptor);
    const defaults = tls.getCACertificates("default");
    tls.setDefaultCACertificates([...defaults, ...validated.roots]);
    return validated.routes;
  } finally {
    encoded.fill(0);
  }
}

/** Validates the complete test-only descriptor before changing process trust. */
function validateTestTlsTrustDescriptor(descriptor) {
  if (
    !isExactRecord(descriptor, TEST_TLS_TRUST_DESCRIPTOR_KEYS) ||
    descriptor.schemaVersion !== 1 ||
    !Array.isArray(descriptor.capabilities) ||
    descriptor.capabilities.length === 0 ||
    descriptor.capabilities.length > MAX_TEST_TLS_TRUST_CAPABILITIES
  ) {
    throw new Error("Test TLS trust descriptor schema is invalid");
  }
  const providerIds = new Set();
  const roots = [];
  const routes = descriptor.capabilities.map((capability) => {
    if (!isExactRecord(capability, TEST_TLS_TRUST_CAPABILITY_KEYS)) {
      throw new Error("Test TLS trust capability schema is invalid");
    }
    if (
      !ID_PATTERN.test(capability.providerId) ||
      !ID_PATTERN.test(capability.nativeProviderId) ||
      providerIds.has(capability.providerId)
    ) {
      throw new Error("Test TLS trust capability identity is invalid");
    }
    validateHttpsBaseUrl(capability.baseUrl);
    if (
      !SHA256_PATTERN.test(capability.baseUrlSha256) ||
      capability.baseUrlSha256 !== sha256(capability.baseUrl)
    ) {
      throw new Error("Test TLS trust base URL evidence is invalid");
    }
    validateCertificateAuthority(capability.tlsCaPem);
    if (
      !SHA256_PATTERN.test(capability.tlsCaSha256) ||
      capability.tlsCaSha256 !== sha256(capability.tlsCaPem)
    ) {
      throw new Error("Test TLS trust CA evidence is invalid");
    }
    providerIds.add(capability.providerId);
    roots.push(capability.tlsCaPem);
    return Object.freeze({
      providerId: capability.providerId,
      nativeProviderId: capability.nativeProviderId,
      baseUrl: capability.baseUrl,
    });
  });
  return {
    roots: Object.freeze(roots),
    routes: Object.freeze(routes),
  };
}

/** Revalidates injected route metadata before a driver can enforce it. */
function validateTestTlsTrustRoutes(routes) {
  if (
    !Array.isArray(routes) ||
    routes.length === 0 ||
    routes.length > MAX_TEST_TLS_TRUST_CAPABILITIES
  ) {
    throw new Error("Test TLS trust routes are invalid");
  }
  const providerIds = new Set();
  return Object.freeze(
    routes.map((route) => {
      if (
        !isExactRecord(route, ["providerId", "nativeProviderId", "baseUrl"]) ||
        !ID_PATTERN.test(route.providerId) ||
        !ID_PATTERN.test(route.nativeProviderId) ||
        providerIds.has(route.providerId)
      ) {
        throw new Error("Test TLS trust route identity is invalid");
      }
      validateHttpsBaseUrl(route.baseUrl);
      providerIds.add(route.providerId);
      return Object.freeze({ ...route });
    }),
  );
}

/** Requires a descriptor route to match all Rust-owned dispatch coordinates. */
function assertTestTlsTrustRoute(routes, identity, modelRoute) {
  if (
    !identity ||
    !ID_PATTERN.test(identity.providerId) ||
    !modelRoute ||
    !routes.some(
      (route) =>
        route.providerId === identity.providerId &&
        route.nativeProviderId === modelRoute.provider &&
        route.baseUrl === modelRoute.baseUrl,
    )
  ) {
    throw new Error("Test TLS trust route does not match this C4OS operation");
  }
}

/** Accepts only an exact, bounded, credential-free HTTPS endpoint string. */
function validateHttpsBaseUrl(value) {
  if (typeof value !== "string" || value.length === 0 || value.length > 2048) {
    throw new Error("Test TLS trust base URL is invalid");
  }
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    throw new Error("Test TLS trust base URL is invalid");
  }
  if (
    parsed.protocol !== "https:" ||
    parsed.hostname !== "127.0.0.1" ||
    parsed.port === "" ||
    parsed.pathname !== "/v1" ||
    parsed.username ||
    parsed.password ||
    parsed.search ||
    parsed.hash
  ) {
    throw new Error("Test TLS trust base URL is invalid");
  }
}

/** Accepts one bounded PEM-encoded CA certificate and no trailing payload. */
function validateCertificateAuthority(value) {
  if (
    typeof value !== "string" ||
    Buffer.byteLength(value, "utf8") === 0 ||
    Buffer.byteLength(value, "utf8") > MAX_TEST_TLS_CA_BYTES ||
    (value.match(/-----BEGIN CERTIFICATE-----/g) ?? []).length !== 1 ||
    (value.match(/-----END CERTIFICATE-----/g) ?? []).length !== 1 ||
    !value.startsWith("-----BEGIN CERTIFICATE-----") ||
    !value.trimEnd().endsWith("-----END CERTIFICATE-----")
  ) {
    throw new Error("Test TLS trust CA is invalid");
  }
  let certificate;
  try {
    certificate = new X509Certificate(value);
  } catch {
    throw new Error("Test TLS trust CA is invalid");
  }
  if (!certificate.ca) throw new Error("Test TLS trust CA is invalid");
}

/** Returns the canonical digest encoding used by Rust-authored evidence. */
function sha256(value) {
  return `sha256:${createHash("sha256").update(value, "utf8").digest("hex")}`;
}

/** Requires a plain object with every expected field and no unknown fields. */
function isExactRecord(value, expectedKeys) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const keys = Object.keys(value);
  return (
    keys.length === expectedKeys.length &&
    keys.every((key) => expectedKeys.includes(key))
  );
}

/** Maps an upstream composite Responses identifier into the bounded C4OS grammar. */
function c4osToolCallId(nativeToolCallId) {
  if (
    typeof nativeToolCallId !== "string" ||
    nativeToolCallId.length === 0 ||
    Buffer.byteLength(nativeToolCallId, "utf8") > MAX_NATIVE_TOOL_CALL_ID_BYTES
  ) {
    throw new Error("Pi native tool-call identity is invalid");
  }
  if (/^[A-Za-z0-9][A-Za-z0-9._:@/-]{0,127}$/.test(nativeToolCallId)) {
    return nativeToolCallId;
  }
  return `pi-tool-${createHash("sha256")
    .update(nativeToolCallId, "utf8")
    .digest("hex")}`;
}

/** Builds the exact parameter contract for each C4OS-brokered Pi tool. */
function toolParameters(sdk, toolName) {
  if (toolName === "c4os_read_resource") {
    return sdk.Type.Object(
      {
        resource: sdk.Type.String({ maxLength: 512 }),
        selector: sdk.Type.Optional(sdk.Type.String({ maxLength: 512 })),
      },
      { additionalProperties: false },
    );
  }
  return sdk.Type.Object(
    {
      operation: sdk.Type.String({ maxLength: 128 }),
      target: sdk.Type.String({ maxLength: 512 }),
      arguments: sdk.Type.Optional(
        sdk.Type.Record(
          sdk.Type.String({ maxLength: 128 }),
          sdk.Type.Unknown(),
        ),
      ),
    },
    { additionalProperties: false },
  );
}

/** Owns bounded descriptor-delivered credentials until one operation claims them. */
export class CredentialLeaseBroker {
  #active = new Set();
  #leases = new Map();
  #seen = new Map();
  #waiters = new Map();

  /** Takes ownership of one secret Buffer from the binary channel. */
  provide({ leaseId, provider, value, expiresAtMs }) {
    if (
      typeof leaseId !== "string" ||
      typeof provider !== "string" ||
      !ID_PATTERN.test(leaseId) ||
      !ID_PATTERN.test(provider) ||
      !Buffer.isBuffer(value) ||
      value.length === 0 ||
      value.length > MAX_CREDENTIAL_SECRET_BYTES
    ) {
      if (Buffer.isBuffer(value)) value.fill(0);
      throw new Error("Invalid credential lease record");
    }
    const now = Date.now();
    if (!Number.isSafeInteger(expiresAtMs) || expiresAtMs <= now) {
      value.fill(0);
      throw new Error("Credential lease is already expired");
    }
    this.#pruneSeen(now);
    if (this.#seen.has(leaseId)) {
      value.fill(0);
      throw new Error("Credential lease identity was already delivered");
    }
    if (this.#leases.size >= MAX_LEASES) {
      value.fill(0);
      throw new Error("Credential lease capacity exceeded");
    }
    const expiryTimer = this.#scheduleExpiry(leaseId, expiresAtMs, now);
    this.#seen.set(leaseId, expiresAtMs);
    this.#leases.set(leaseId, {
      provider,
      // The binary channel transfers ownership of its sole secret Buffer here.
      value,
      expiresAtMs,
      expiryTimer,
    });
    this.#waiters.get(leaseId)?.ready();
  }

  /** Waits boundedly for descriptor decode, then claims one exact operation. */
  async claimForOperation(
    identity,
    provider,
    timeoutMs = CREDENTIAL_READY_TIMEOUT_MS,
  ) {
    const operationId = credentialOperationId(identity, provider);
    if (!this.#leases.has(operationId)) {
      this.#pruneSeen(Date.now());
      if (this.#seen.has(operationId)) {
        throw new Error("Credential lease is unavailable or already claimed");
      }
      if (
        this.#waiters.has(operationId) ||
        this.#waiters.size >= MAX_LEASES ||
        !Number.isSafeInteger(timeoutMs) ||
        timeoutMs < 1
      ) {
        throw new Error("Credential operation barrier is unavailable");
      }
      await new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          this.#waiters.delete(operationId);
          reject(new Error("Credential operation barrier timed out"));
        }, timeoutMs);
        this.#waiters.set(operationId, {
          ready: () => {
            clearTimeout(timer);
            this.#waiters.delete(operationId);
            resolve();
          },
          reject,
          timer,
        });
      });
    }
    return this.#claim(operationId, provider);
  }

  /** Claims one delivered operation for repeated SDK reads in its active run. */
  #claim(operationId, provider) {
    const lease = this.#leases.get(operationId);
    if (!lease)
      throw new Error("Credential lease is unavailable or already claimed");
    this.#leases.delete(operationId);
    clearTimeout(lease.expiryTimer);
    if (lease.expiresAtMs <= Date.now()) {
      lease.value.fill(0);
      throw new Error("Credential lease is already expired");
    }
    if (lease.provider !== provider) {
      lease.value.fill(0);
      throw new Error("Credential lease provider binding changed");
    }
    // Pi's upstream getApiKey contract requires an immutable JS string. This
    // is the only unavoidable non-zeroizable copy and stays run-local.
    const value = lease.value.toString("utf8");
    lease.value.fill(0);
    const operation = new CredentialOperationLease(
      lease.provider,
      value,
      () => {
        this.#active.delete(operation);
      },
    );
    this.#active.add(operation);
    return operation;
  }

  /** Releases active handles and overwrites every unclaimed credential Buffer. */
  clear() {
    for (const operation of this.#active) operation.release();
    for (const lease of this.#leases.values()) {
      clearTimeout(lease.expiryTimer);
      lease.value.fill(0);
    }
    this.#active.clear();
    this.#leases.clear();
    this.#seen.clear();
    for (const waiter of this.#waiters.values()) {
      clearTimeout(waiter.timer);
      waiter.reject(new Error("Credential channel closed"));
    }
    this.#waiters.clear();
  }

  /** Zeroes one unclaimed credential as soon as its descriptor TTL expires. */
  #expire(leaseId) {
    const lease = this.#leases.get(leaseId);
    if (!lease) return;
    const now = Date.now();
    if (lease.expiresAtMs > now) {
      lease.expiryTimer = this.#scheduleExpiry(leaseId, lease.expiresAtMs, now);
      return;
    }
    lease.value.fill(0);
    this.#leases.delete(leaseId);
  }

  /** Drops only expired replay markers; an original replay is then TTL-invalid. */
  #pruneSeen(now) {
    for (const [leaseId, expiresAtMs] of this.#seen) {
      if (expiresAtMs <= now) this.#seen.delete(leaseId);
    }
  }

  /** Schedules bounded timer hops for lease expiries beyond Node's timer range. */
  #scheduleExpiry(leaseId, expiresAtMs, now) {
    const delay = Math.max(1, Math.min(expiresAtMs - now, MAX_TIMER_DELAY_MS));
    const expiryTimer = setTimeout(() => this.#expire(leaseId), delay);
    expiryTimer.unref?.();
    return expiryTimer;
  }
}

/** Derives the descriptor key from the full non-secret dispatch operation. */
export function credentialOperationId(identity, nativeProvider) {
  const fields = [
    identity.runtimeId,
    identity.workspaceId,
    identity.sessionId,
    identity.turnId,
    identity.runId,
    identity.correlationId,
    String(identity.processGeneration),
    identity.providerId,
    nativeProvider,
  ];
  if (
    fields.some((field) => typeof field !== "string" || !ID_PATTERN.test(field))
  ) {
    throw new Error("Credential operation identity is invalid");
  }
  const digest = createHash("sha256");
  for (const field of fields) {
    const encoded = Buffer.from(field, "utf8");
    const length = Buffer.alloc(8);
    length.writeBigUInt64BE(BigInt(encoded.length));
    digest.update(length);
    digest.update(encoded);
    length.fill(0);
  }
  return `pi-provider:${digest.digest("hex")}`;
}

/** Keeps the sole immutable SDK credential copy scoped to one active run. */
class CredentialOperationLease {
  #onRelease;
  #provider;
  #value;

  /** Captures one claimed secret and its exact native provider binding. */
  constructor(provider, value, onRelease) {
    this.#provider = provider;
    this.#value = value;
    this.#onRelease = onRelease;
  }

  /** Returns the same run-local credential for matching provider calls only. */
  get(provider) {
    if (this.#value === undefined) return undefined;
    if (provider !== this.#provider) {
      this.release();
      return undefined;
    }
    return this.#value;
  }

  /** Makes the unavoidable immutable string unreachable after the run settles. */
  release() {
    if (this.#value === undefined) return;
    this.#value = undefined;
    this.#onRelease();
  }
}

/** Incrementally decodes the bounded binary credential descriptor protocol. */
export class CredentialFrameDecoder {
  #queue = new ByteQueue();
  #state = "header";
  #metadata;
  #pendingChunkBytes = 0;
  #secretBytes = 0;
  #secretParts = [];

  /** Takes ownership of a stream Buffer and returns any completed records. */
  push(chunk) {
    if (!Buffer.isBuffer(chunk) || chunk.length === 0) {
      throw new Error("Credential channel requires a non-empty Buffer");
    }
    this.#queue.push(chunk);
    const records = [];
    try {
      while (this.#advance(records)) {
        // Keep decoding complete frames without retaining stream chunks.
      }
      return records;
    } catch (error) {
      for (const record of records) record.value.fill(0);
      this.clear();
      throw error;
    }
  }

  /** Rejects an EOF that could otherwise turn a partial frame into authority. */
  finish() {
    if (this.#state !== "header" || this.#queue.length !== 0) {
      this.clear();
      throw new Error("Credential channel ended with a partial frame");
    }
  }

  /** Overwrites all buffered secret material and resets protocol state. */
  clear() {
    this.#queue.clear();
    for (const part of this.#secretParts) part.fill(0);
    this.#secretParts = [];
    this.#metadata = undefined;
    this.#pendingChunkBytes = 0;
    this.#secretBytes = 0;
    this.#state = "header";
  }

  /** Advances exactly one decoder state when enough bytes are available. */
  #advance(records) {
    if (this.#state === "header") return this.#readHeader();
    if (this.#state === "metadata") return this.#readMetadata();
    if (this.#state === "chunk-length") return this.#readChunkLength(records);
    return this.#readSecretChunk();
  }

  /** Validates the fixed credential frame header and declared metadata sizes. */
  #readHeader() {
    const header = this.#queue.take(CREDENTIAL_HEADER_BYTES);
    if (!header) return false;
    try {
      if (
        !header.subarray(0, CREDENTIAL_MAGIC.length).equals(CREDENTIAL_MAGIC) ||
        header[8] !== CREDENTIAL_SCHEMA_VERSION
      ) {
        throw new Error("Credential channel header is invalid");
      }
      const leaseIdBytes = header.readUInt16BE(9);
      const providerBytes = header.readUInt16BE(11);
      const expiresAt = header.readBigUInt64BE(13);
      if (
        leaseIdBytes === 0 ||
        leaseIdBytes > 128 ||
        providerBytes === 0 ||
        providerBytes > 128 ||
        expiresAt > BigInt(Number.MAX_SAFE_INTEGER)
      ) {
        throw new Error("Credential channel metadata is invalid");
      }
      this.#metadata = {
        leaseIdBytes,
        providerBytes,
        expiresAtMs: Number(expiresAt),
      };
      this.#state = "metadata";
      return true;
    } finally {
      header.fill(0);
    }
  }

  /** Decodes only bounded, strict UTF-8 lease and provider identifiers. */
  #readMetadata() {
    const length = this.#metadata.leaseIdBytes + this.#metadata.providerBytes;
    const metadata = this.#queue.take(length);
    if (!metadata) return false;
    try {
      const leaseId = fatalUtf8.decode(
        metadata.subarray(0, this.#metadata.leaseIdBytes),
      );
      const provider = fatalUtf8.decode(
        metadata.subarray(this.#metadata.leaseIdBytes),
      );
      if (!ID_PATTERN.test(leaseId) || !ID_PATTERN.test(provider)) {
        throw new Error("Credential channel identity is invalid");
      }
      this.#metadata = {
        leaseId,
        provider,
        expiresAtMs: this.#metadata.expiresAtMs,
      };
      this.#state = "chunk-length";
      return true;
    } finally {
      metadata.fill(0);
    }
  }

  /** Selects the next bounded secret chunk or completes the current frame. */
  #readChunkLength(records) {
    const encodedLength = this.#queue.take(4);
    if (!encodedLength) return false;
    const chunkBytes = encodedLength.readUInt32BE(0);
    encodedLength.fill(0);
    if (chunkBytes === 0) {
      if (this.#secretBytes === 0)
        throw new Error("Credential channel secret is empty");
      const value = this.#finishSecret();
      records.push({ ...this.#metadata, value });
      this.#metadata = undefined;
      this.#state = "header";
      return true;
    }
    if (chunkBytes > MAX_CREDENTIAL_SECRET_BYTES - this.#secretBytes) {
      throw new Error("Credential channel secret exceeds its bound");
    }
    this.#pendingChunkBytes = chunkBytes;
    this.#state = "chunk";
    return true;
  }

  /** Moves one complete secret chunk into the frame-owned secret set. */
  #readSecretChunk() {
    const chunk = this.#queue.take(this.#pendingChunkBytes);
    if (!chunk) return false;
    this.#secretParts.push(chunk);
    this.#secretBytes += chunk.length;
    this.#pendingChunkBytes = 0;
    this.#state = "chunk-length";
    return true;
  }

  /** Returns the sole zeroizable secret Buffer with no copy when possible. */
  #finishSecret() {
    let value;
    if (this.#secretParts.length === 1) {
      [value] = this.#secretParts;
    } else {
      value = Buffer.concat(this.#secretParts, this.#secretBytes);
      for (const part of this.#secretParts) part.fill(0);
    }
    this.#secretParts = [];
    this.#secretBytes = 0;
    return value;
  }
}

/** Owns incoming stream Buffers and overwrites bytes as they are consumed. */
class ByteQueue {
  #chunks = [];
  #length = 0;

  /** Reports the number of unread channel bytes. */
  get length() {
    return this.#length;
  }

  /** Takes ownership of one Buffer emitted by the credential stream. */
  push(chunk) {
    this.#chunks.push(chunk);
    this.#length += chunk.length;
  }

  /** Copies exactly one logical field and clears the consumed source bytes. */
  take(length) {
    if (this.#length < length) return undefined;
    const output = Buffer.allocUnsafe(length);
    let outputOffset = 0;
    while (outputOffset < length) {
      const source = this.#chunks[0];
      const consumed = Math.min(source.length, length - outputOffset);
      source.copy(output, outputOffset, 0, consumed);
      source.fill(0, 0, consumed);
      outputOffset += consumed;
      this.#length -= consumed;
      if (consumed === source.length) {
        this.#chunks.shift();
      } else {
        this.#chunks[0] = source.subarray(consumed);
      }
    }
    return output;
  }

  /** Overwrites and releases every unread stream Buffer. */
  clear() {
    for (const chunk of this.#chunks) chunk.fill(0);
    this.#chunks = [];
    this.#length = 0;
  }
}

/** Starts a fail-closed credential reader without treating secrets as text. */
export function startCredentialChannel(fd, broker) {
  const input = createReadStream("", { fd, autoClose: true });
  const decoder = new CredentialFrameDecoder();
  void (async () => {
    try {
      for await (const chunk of input) {
        for (const record of decoder.push(chunk)) broker.provide(record);
      }
      decoder.finish();
    } catch {
      // Credential-channel input is intentionally never echoed or logged.
      input.destroy();
    } finally {
      decoder.clear();
      broker.clear();
    }
  })();
  return input;
}

/** Imports and verifies the exact installed Pi packages used by production. */
async function loadSdk() {
  const [agentCore, codingAgent, piAi, typebox] = await Promise.all([
    import("@earendil-works/pi-agent-core"),
    import("@earendil-works/pi-coding-agent"),
    import("@earendil-works/pi-ai/compat"),
    import("typebox"),
  ]);
  await Promise.all(
    SDK_PACKAGES.map(([specifier, name]) =>
      assertPackageVersion(specifier, name),
    ),
  );
  if (
    typeof agentCore.Agent !== "function" ||
    typeof codingAgent.convertToLlm !== "function" ||
    typeof piAi.getModel !== "function" ||
    typeof piAi.streamSimple !== "function" ||
    typeof typebox.Type?.Object !== "function"
  ) {
    throw new Error("The installed Pi SDK surface is incompatible");
  }
  return {
    Agent: agentCore.Agent,
    convertToLlm: codingAgent.convertToLlm,
    getModel: piAi.getModel,
    streamSimple: piAi.streamSimple,
    Type: typebox.Type,
  };
}

/** Fails closed when an imported Pi entrypoint is not from the exact lock row. */
async function assertPackageVersion(specifier, expectedName) {
  const entrypoint = import.meta.resolve(specifier);
  const packageUrl = new URL("../package.json", entrypoint);
  const packageJson = JSON.parse(await readFile(packageUrl, "utf8"));
  if (
    packageJson.name !== expectedName ||
    packageJson.version !== PACKAGE_VERSION
  ) {
    throw new Error("The installed Pi SDK package identity is incompatible");
  }
}

/** Narrows a completed C4OS tool result to Pi's custom-tool result contract. */
function normalizeToolResult(result) {
  const boundedResultText =
    typeof result.resultCode === "string"
      ? result.resultCode.slice(0, 64 * 1024)
      : typeof result.summary === "string"
        ? result.summary.slice(0, 64 * 1024)
        : "C4OS completed the action.";
  const content = Array.isArray(result.content)
    ? result.content
        .filter(
          (item) =>
            item && item.type === "text" && typeof item.text === "string",
        )
        .slice(0, 64)
        .map((item) => ({ type: "text", text: item.text.slice(0, 64 * 1024) }))
    : [
        {
          type: "text",
          text: boundedResultText,
        },
      ];
  return {
    content,
    details: result.details ?? {},
    terminate: result.terminate === true,
  };
}
