import { createOpencodeClient } from "@opencode-ai/sdk";

import { C4OS_TOOL_IDS } from "./broker-channel.mjs";

function assertLoopbackBaseUrl(baseUrl) {
  const url = new URL(baseUrl);
  if (url.protocol !== "http:")
    throw new TypeError("OpenCode SDK transport must use loopback HTTP");
  if (!["127.0.0.1", "localhost", "[::1]"].includes(url.hostname)) {
    throw new TypeError("OpenCode SDK transport must use loopback HTTP");
  }
  if (url.username || url.password || url.search || url.hash) {
    throw new TypeError(
      "OpenCode SDK URL must not carry credentials or metadata",
    );
  }
  return url.toString();
}

export function bindPluginOpenCodeClient(pluginInput) {
  if (!pluginInput || !pluginInput.client || !pluginInput.serverUrl) {
    throw new TypeError(
      "OpenCode must supply its authenticated generated SDK client",
    );
  }
  assertLoopbackBaseUrl(pluginInput.serverUrl);
  if (
    !pluginInput.client.tool ||
    typeof pluginInput.client.tool.ids !== "function"
  ) {
    throw new TypeError(
      "OpenCode must supply its authenticated generated SDK client",
    );
  }
  return pluginInput.client;
}

export function createPinnedOpenCodeClient({
  baseUrl,
  directory,
  fetch,
  headers,
} = {}) {
  if (typeof fetch !== "function") {
    throw new TypeError(
      "A C4OS-owned authenticated fetch boundary is required",
    );
  }
  if (
    headers &&
    Object.keys(headers).some((key) =>
      /authorization|cookie|token|secret/iu.test(key),
    )
  ) {
    throw new TypeError("OpenCode SDK headers must not carry credentials");
  }
  return createOpencodeClient({
    baseUrl: assertLoopbackBaseUrl(baseUrl),
    ...(directory === undefined ? {} : { directory }),
    ...(fetch === undefined ? {} : { fetch }),
    ...(headers === undefined ? {} : { headers }),
  });
}

export async function readRegisteredC4osToolIds(client) {
  if (!client?.tool || typeof client.tool.ids !== "function") {
    throw new TypeError("A generated OpenCode SDK client is required");
  }
  const response = await client.tool.ids({ throwOnError: true });
  const ids = response?.data;
  if (!Array.isArray(ids) || ids.some((id) => typeof id !== "string")) {
    throw new TypeError("OpenCode returned an invalid tool registry");
  }
  const c4osIds = ids.filter((id) => id.startsWith("c4os_"));
  if (
    c4osIds.length !== C4OS_TOOL_IDS.length ||
    C4OS_TOOL_IDS.some((id) => !c4osIds.includes(id))
  ) {
    throw new Error(
      "OpenCode C4OS tool registry does not match the pinned bridge",
    );
  }
  return Object.freeze([...c4osIds].sort());
}
