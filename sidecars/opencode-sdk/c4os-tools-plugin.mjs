import { tool } from "@opencode-ai/plugin";
import { createHash, X509Certificate } from "node:crypto";
import { closeSync, readSync } from "node:fs";

import { C4OS_TOOL_IDS, FdBrokerChannel } from "./broker-channel.mjs";
import { ProviderCredentialChannel } from "./credential-channel.mjs";
import {
  bindPluginOpenCodeClient,
  readRegisteredC4osToolIds,
} from "./sdk-client.mjs";

const shortText = tool.schema.string().min(1).max(512);
const jsonRecord = tool.schema.record(tool.schema.string(), tool.schema.json());
const TEST_TLS_TRUST_FD_ENV = "C4OS_OPENCODE_TEST_TLS_TRUST_FD";
const TEST_TLS_TRUST_SCHEMA_VERSION = 1;
const MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES = 256 * 1024;
const MAX_TEST_TLS_TRUST_CAPABILITIES = 16;
const MAX_TEST_TLS_CA_BYTES = 128 * 1024;
const MAX_TEST_TLS_BASE_URL_BYTES = 4 * 1024;
const SAFE_PROVIDER_ID = /^[A-Za-z0-9][A-Za-z0-9._:@-]{0,191}$/u;
const SHA256 = /^sha256:[0-9a-f]{64}$/u;
const CERTIFICATE_PEM =
  /^-----BEGIN CERTIFICATE-----\n(?:[A-Za-z0-9+/=]{1,76}\n)+-----END CERTIFICATE-----\n?$/u;

function testTlsFail(code) {
  throw new Error(`C4OS test TLS trust rejected (${code})`);
}

function exactKeys(value, expected, code) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    Object.keys(value).length !== expected.length ||
    Object.keys(value).some((key) => !expected.includes(key))
  ) {
    testTlsFail(code);
  }
}

function sha256(value) {
  return `sha256:${createHash("sha256").update(value, "utf8").digest("hex")}`;
}

function validateTestTlsBaseUrl(value, expectedSha256) {
  if (
    typeof value !== "string" ||
    Buffer.byteLength(value, "utf8") > MAX_TEST_TLS_BASE_URL_BYTES ||
    !SHA256.test(expectedSha256) ||
    sha256(value) !== expectedSha256
  ) {
    testTlsFail("invalid_base_url");
  }
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    testTlsFail("invalid_base_url");
  }
  if (
    parsed.protocol !== "https:" ||
    parsed.hostname !== "127.0.0.1" ||
    parsed.port === "" ||
    parsed.pathname !== "/v1" ||
    parsed.username !== "" ||
    parsed.password !== "" ||
    parsed.search !== "" ||
    parsed.hash !== "" ||
    parsed.toString() !== value ||
    (parsed.pathname !== "/" && parsed.pathname.endsWith("/"))
  ) {
    testTlsFail("invalid_base_url");
  }
  return parsed;
}

function validateTestTlsCapability(value) {
  exactKeys(
    value,
    [
      "providerId",
      "nativeProviderId",
      "baseUrl",
      "baseUrlSha256",
      "tlsCaPem",
      "tlsCaSha256",
    ],
    "invalid_capability",
  );
  if (
    typeof value.providerId !== "string" ||
    !SAFE_PROVIDER_ID.test(value.providerId) ||
    typeof value.nativeProviderId !== "string" ||
    !SAFE_PROVIDER_ID.test(value.nativeProviderId)
  ) {
    testTlsFail("invalid_provider_id");
  }
  if (
    typeof value.tlsCaPem !== "string" ||
    Buffer.byteLength(value.tlsCaPem, "utf8") > MAX_TEST_TLS_CA_BYTES ||
    !CERTIFICATE_PEM.test(value.tlsCaPem) ||
    !SHA256.test(value.tlsCaSha256) ||
    sha256(value.tlsCaPem) !== value.tlsCaSha256
  ) {
    testTlsFail("invalid_tls_ca");
  }
  try {
    if (!new X509Certificate(value.tlsCaPem).ca) {
      testTlsFail("invalid_tls_ca");
    }
  } catch {
    testTlsFail("invalid_tls_ca");
  }
  const baseUrl = validateTestTlsBaseUrl(value.baseUrl, value.baseUrlSha256);
  return Object.freeze({
    providerId: value.providerId,
    nativeProviderId: value.nativeProviderId,
    baseUrl: value.baseUrl,
    baseOrigin: baseUrl.origin,
    basePath: baseUrl.pathname,
    tlsCaPem: value.tlsCaPem,
  });
}

function parseTestTlsTrustDescriptor(encoded) {
  let decoded;
  try {
    decoded = JSON.parse(encoded.toString("utf8"));
  } catch {
    testTlsFail("invalid_json");
  }
  exactKeys(decoded, ["schemaVersion", "capabilities"], "invalid_envelope");
  if (
    decoded.schemaVersion !== TEST_TLS_TRUST_SCHEMA_VERSION ||
    !Array.isArray(decoded.capabilities) ||
    decoded.capabilities.length < 1 ||
    decoded.capabilities.length > MAX_TEST_TLS_TRUST_CAPABILITIES
  ) {
    testTlsFail("invalid_envelope");
  }
  const capabilities = decoded.capabilities.map(validateTestTlsCapability);
  if (
    new Set(capabilities.map((value) => value.providerId)).size !==
      capabilities.length ||
    new Set(capabilities.map((value) => value.nativeProviderId)).size !==
      capabilities.length
  ) {
    testTlsFail("duplicate_provider_id");
  }
  return capabilities;
}

function readTestTlsTrustDescriptor(fd, operations) {
  const chunks = [];
  let length = 0;
  try {
    for (;;) {
      const chunk = Buffer.alloc(
        Math.min(16 * 1024, MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES + 1 - length),
      );
      if (chunk.length === 0) testTlsFail("descriptor_too_large");
      const count = operations.readSync(fd, chunk, 0, chunk.length, null);
      if (!Number.isSafeInteger(count) || count < 0 || count > chunk.length) {
        testTlsFail("invalid_descriptor_read");
      }
      if (count === 0) break;
      chunks.push(Buffer.from(chunk.subarray(0, count)));
      length += count;
      if (length > MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES) {
        testTlsFail("descriptor_too_large");
      }
    }
  } finally {
    operations.closeSync(fd);
  }
  if (length === 0) testTlsFail("empty_descriptor");
  return Buffer.concat(chunks, length);
}

function requestUrl(input) {
  try {
    if (typeof input === "string" || input instanceof URL)
      return new URL(input);
    if (input && typeof input.url === "string") return new URL(input.url);
  } catch {
    testTlsFail("invalid_request_url");
  }
  testTlsFail("invalid_request_url");
}

export function createTestTlsFetch(capability, fetchImplementation) {
  if (typeof fetchImplementation !== "function") {
    testTlsFail("fetch_unavailable");
  }
  return function c4osTestTlsFetch(input, init = undefined) {
    if (
      (input !== null &&
        typeof input === "object" &&
        Object.prototype.hasOwnProperty.call(input, "tls")) ||
      (init !== null &&
        typeof init === "object" &&
        Object.prototype.hasOwnProperty.call(init, "tls"))
    ) {
      testTlsFail("caller_tls_rejected");
    }
    const target = requestUrl(input);
    if (
      target.protocol !== "https:" ||
      target.username !== "" ||
      target.password !== "" ||
      target.origin !== capability.baseOrigin ||
      (target.pathname !== capability.basePath &&
        !target.pathname.startsWith(
          capability.basePath === "/" ? "/" : `${capability.basePath}/`,
        ))
    ) {
      testTlsFail("request_origin_rejected");
    }
    return fetchImplementation(input, {
      ...(init ?? {}),
      redirect: "error",
      tls: {
        ca: capability.tlsCaPem,
        rejectUnauthorized: true,
      },
    });
  };
}

export class TestTlsTrustCapabilities {
  static fromEnvironment(
    environment = process.env,
    operations = { readSync, closeSync },
  ) {
    const rawFd = environment[TEST_TLS_TRUST_FD_ENV];
    if (rawFd === undefined) return undefined;
    delete environment[TEST_TLS_TRUST_FD_ENV];
    if (!/^[0-9]{1,4}$/u.test(rawFd)) testTlsFail("invalid_descriptor_fd");
    const fd = Number.parseInt(rawFd, 10);
    if (fd < 64 || fd > 1023) testTlsFail("invalid_descriptor_fd");
    return new TestTlsTrustCapabilities(
      parseTestTlsTrustDescriptor(readTestTlsTrustDescriptor(fd, operations)),
    );
  }

  #capabilities;
  #configurationState = "pending";

  constructor(capabilities) {
    this.#capabilities = capabilities;
  }

  configure(config, fetchImplementation = globalThis.fetch) {
    if (this.#configurationState !== "pending") {
      this.#configurationState = "failed";
      testTlsFail("configuration_replay");
    }
    try {
      if (
        config === null ||
        typeof config !== "object" ||
        Array.isArray(config)
      ) {
        testTlsFail("invalid_configuration");
      }
      const currentProviders = config.provider ?? {};
      if (
        currentProviders === null ||
        typeof currentProviders !== "object" ||
        Array.isArray(currentProviders)
      ) {
        testTlsFail("invalid_configuration");
      }
      const installed = { ...currentProviders };
      for (const capability of this.#capabilities) {
        const current = currentProviders[capability.nativeProviderId] ?? {};
        if (
          current === null ||
          typeof current !== "object" ||
          Array.isArray(current) ||
          current.options === null ||
          (current.options !== undefined &&
            (typeof current.options !== "object" ||
              Array.isArray(current.options))) ||
          (current.options?.baseURL !== undefined &&
            current.options.baseURL !== capability.baseUrl) ||
          current.options?.fetch !== undefined
        ) {
          testTlsFail("provider_configuration_conflict");
        }
        installed[capability.nativeProviderId] = {
          ...current,
          options: {
            ...(current.options ?? {}),
            baseURL: capability.baseUrl,
            fetch: createTestTlsFetch(capability, fetchImplementation),
          },
        };
      }
      config.provider = installed;
      this.#configurationState = "active";
    } catch (error) {
      this.#configurationState = "failed";
      throw error;
    }
  }

  credentialProviderId(nativeProviderId) {
    if (this.#configurationState !== "active") {
      testTlsFail("configuration_inactive");
    }
    const capability = this.#capabilities.find(
      (value) => value.nativeProviderId === nativeProviderId,
    );
    if (!capability) testTlsFail("provider_not_authorized");
    return capability.providerId;
  }
}

function formatBrokerResult(frame) {
  return {
    title:
      frame.status === "denied" ? "C4OS request denied" : "C4OS broker result",
    output: JSON.stringify({
      correlationId: frame.correlationId,
      status: frame.status,
      ...(frame.reasonCode === undefined
        ? {}
        : { reasonCode: frame.reasonCode }),
      ...(frame.payload === undefined ? {} : { payload: frame.payload }),
    }),
    metadata: {
      c4osCorrelationId: frame.correlationId,
      c4osBrokerStatus: frame.status,
    },
  };
}

async function executeThroughBroker(channel, client, toolId, args, context) {
  await readRegisteredC4osToolIds(client);
  return channel
    .request(
      toolId,
      args,
      { sessionId: context.sessionID, messageId: context.messageID },
      { signal: context.abort },
    )
    .then(formatBrokerResult);
}

export function createC4osBrokerTools(channel, client) {
  const definitions = {
    c4os_propose_action: tool({
      description:
        "Propose an action to C4OS for policy evaluation; this tool cannot perform the action.",
      args: {
        operation: shortText,
        target: shortText,
        arguments: jsonRecord.optional(),
      },
      execute(args, context) {
        return executeThroughBroker(
          channel,
          client,
          C4OS_TOOL_IDS[0],
          args,
          context,
        );
      },
    }),
    c4os_read_resource: tool({
      description:
        "Request a C4OS-owned resource result through the authenticated broker; this tool cannot read directly.",
      args: {
        resource: shortText,
        selector: shortText.optional(),
      },
      execute(args, context) {
        return executeThroughBroker(
          channel,
          client,
          C4OS_TOOL_IDS[1],
          args,
          context,
        );
      },
    }),
  };
  if (Object.keys(definitions).join("\0") !== C4OS_TOOL_IDS.join("\0")) {
    throw new Error("C4OS OpenCode tool materialization invariant failed");
  }
  return Object.freeze(definitions);
}

export function createC4osCredentialHeadersHook(channel, testTlsTrust) {
  return async function credentialHeaders(input, output) {
    const providerId = input?.provider?.info?.id ?? input?.model?.providerID;
    if (typeof providerId !== "string") {
      throw new TypeError("OpenCode did not supply the provider identity");
    }
    const modelId = input?.model?.id;
    if (typeof modelId !== "string") {
      throw new TypeError("OpenCode did not supply the model identity");
    }
    const nativeMessageId = input?.message?.id;
    if (
      typeof nativeMessageId !== "string" ||
      !nativeMessageId.startsWith("msg_")
    ) {
      throw new TypeError(
        "OpenCode did not supply the native user-message identity",
      );
    }
    const credential = await channel.request({
      nativeSessionId: input.sessionID,
      providerId: testTlsTrust
        ? testTlsTrust.credentialProviderId(providerId)
        : providerId,
      modelId,
      operationId: nativeMessageId.slice(4),
      nativeMessageId,
    });
    if (
      Object.keys(output.headers).some(
        (name) => name.toLowerCase() === credential.headerName.toLowerCase(),
      )
    ) {
      throw new Error("OpenCode provider credential header already exists");
    }
    output.headers[credential.headerName] = credential.headerValue;
  };
}

export function createC4osPluginHooks(
  pluginInput,
  broker,
  credentials,
  testTlsTrust,
) {
  // OpenCode supplies this generated SDK client to the plugin. It is already
  // bound to the authenticated loopback server, so no password is copied into
  // this plugin's argv, environment, headers, or broker frames.
  const client = bindPluginOpenCodeClient(pluginInput);
  return {
    tool: createC4osBrokerTools(broker, client),
    "chat.headers": createC4osCredentialHeadersHook(credentials, testTlsTrust),
    ...(testTlsTrust
      ? {
          async config(config) {
            testTlsTrust.configure(config);
          },
        }
      : {}),
    async dispose() {
      credentials.dispose();
      broker.close();
    },
  };
}

export async function C4osBrokerToolsPlugin(pluginInput) {
  const broker = FdBrokerChannel.fromEnvironment();
  const credentials = ProviderCredentialChannel.fromEnvironment();
  try {
    const testTlsTrust = TestTlsTrustCapabilities.fromEnvironment();
    return createC4osPluginHooks(
      pluginInput,
      broker,
      credentials,
      testTlsTrust,
    );
  } catch (error) {
    credentials.dispose();
    broker.close();
    throw error;
  }
}
