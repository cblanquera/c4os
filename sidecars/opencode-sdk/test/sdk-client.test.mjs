import assert from "node:assert/strict";
import test from "node:test";

import {
  bindPluginOpenCodeClient,
  createPinnedOpenCodeClient,
  readRegisteredC4osToolIds,
} from "../sdk-client.mjs";

test("uses the generated SDK tool registry endpoint and accepts only pinned C4OS tools", async () => {
  const requests = [];
  const client = createPinnedOpenCodeClient({
    baseUrl: "http://127.0.0.1:4096",
    directory: "/tmp/c4os-sdk-test",
    fetch: async (request) => {
      requests.push(request);
      return new Response(
        JSON.stringify([
          "bash",
          "read",
          "c4os_propose_action",
          "c4os_read_resource",
        ]),
        { status: 200, headers: { "content-type": "application/json" } },
      );
    },
  });

  assert.deepEqual(await readRegisteredC4osToolIds(client), [
    "c4os_propose_action",
    "c4os_read_resource",
  ]);
  assert.equal(requests.length, 1);
  const requestUrl = new URL(requests[0].url);
  assert.equal(requestUrl.pathname, "/experimental/tool/ids");
  assert.equal(requestUrl.searchParams.get("directory"), "/tmp/c4os-sdk-test");
});

test("rejects an unexpected C4OS tool registration", async () => {
  const client = {
    tool: {
      async ids() {
        return {
          data: ["c4os_propose_action", "c4os_read_resource", "c4os_shell"],
        };
      },
    },
  };
  await assert.rejects(readRegisteredC4osToolIds(client), /does not match/u);
});

test("rejects non-loopback or credential-bearing SDK transports", () => {
  assert.throws(
    () =>
      createPinnedOpenCodeClient({
        baseUrl: "https://example.com",
        fetch: async () => {},
      }),
    /loopback HTTP/u,
  );
  assert.throws(
    () =>
      createPinnedOpenCodeClient({
        baseUrl: "http://127.0.0.1:4096",
        fetch: async () => {},
        headers: { authorization: "Bearer value" },
      }),
    /must not carry credentials/u,
  );
});

test("uses OpenCode's authenticated generated plugin client without copying a secret", () => {
  const generatedClient = {
    tool: {
      async ids() {
        return { data: [] };
      },
    },
  };
  assert.equal(
    bindPluginOpenCodeClient({
      client: generatedClient,
      serverUrl: new URL("http://127.0.0.1:4096"),
    }),
    generatedClient,
  );
  assert.throws(
    () =>
      bindPluginOpenCodeClient({
        client: generatedClient,
        serverUrl: new URL("https://example.com"),
      }),
    /loopback HTTP/u,
  );
});
