import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import test from "node:test";
import { once } from "node:events";
import process from "node:process";
import { rootCertificates } from "node:tls";
import { fileURLToPath, URL } from "node:url";

const main = fileURLToPath(new URL("../main.mjs", import.meta.url));
const sidecarRoot = fileURLToPath(new URL("../", import.meta.url));

test("real process consumes one private test TLS descriptor before serving requests", async () => {
  const child = spawn(
    process.execPath,
    [main, "--generation=10", "--tls-trust-fd=3"],
    {
      env: { PATH: process.env.PATH ?? "" },
      stdio: ["pipe", "pipe", "pipe", "pipe"],
    },
  );
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8").on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.setEncoding("utf8").on("data", (chunk) => {
    stderr += chunk;
  });
  child.stdio[3].end(JSON.stringify(testTlsTrustDescriptor()));
  for (const [index, operation] of ["health", "shutdown"].entries()) {
    child.stdin.write(
      `${JSON.stringify({
        schemaVersion: 1,
        kind: "request",
        requestId: `request-private-trust-${operation}`,
        correlationId: `correlation-private-trust-${index + 1}`,
        processGeneration: 10,
        operation,
        payload: {},
      })}\n`,
    );
  }
  child.stdin.end();
  const [code] = await once(child, "close");
  assert.equal(code, 0, stderr);
  assert.equal(stderr, "");
  const responses = stdout
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    responses.map((response) => response.status),
    ["ok", "ok"],
  );
});

test("real process rejects a changed test TLS descriptor without echoing its CA", async () => {
  const descriptor = testTlsTrustDescriptor();
  descriptor.capabilities[0].tlsCaSha256 = `sha256:${"0".repeat(64)}`;
  const child = spawn(
    process.execPath,
    [main, "--generation=10", "--tls-trust-fd=3"],
    {
      env: { PATH: process.env.PATH ?? "" },
      stdio: ["ignore", "pipe", "pipe", "pipe"],
    },
  );
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8").on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.setEncoding("utf8").on("data", (chunk) => {
    stderr += chunk;
  });
  child.stdio[3].end(JSON.stringify(descriptor));
  const [code] = await once(child, "close");
  assert.notEqual(code, 0);
  assert.equal(stdout, "");
  assert.match(stderr, /Test TLS trust CA evidence is invalid/);
  assert.equal(stderr.includes(descriptor.capabilities[0].tlsCaPem), false);
});

test("real process rejects inherited trust and credential descriptor collision", async () => {
  const child = spawn(
    process.execPath,
    [main, "--generation=10", "--credential-fd=3", "--tls-trust-fd=3"],
    {
      env: { PATH: process.env.PATH ?? "" },
      stdio: ["ignore", "pipe", "pipe", "pipe"],
    },
  );
  let stderr = "";
  child.stderr.setEncoding("utf8").on("data", (chunk) => {
    stderr += chunk;
  });
  child.stdio[3].end();
  const [code] = await once(child, "close");
  assert.notEqual(code, 0);
  assert.match(stderr, /Dedicated inherited descriptors must not collide/);
});

test("real NDJSON process handles health, version, and shutdown with a sanitized launch environment", async () => {
  const child = spawn(process.execPath, [main, "--generation=11"], {
    env: { PATH: process.env.PATH ?? "" },
    stdio: ["pipe", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8").on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.setEncoding("utf8").on("data", (chunk) => {
    stderr += chunk;
  });
  for (const [index, operation] of [
    "health",
    "version",
    "shutdown",
  ].entries()) {
    child.stdin.write(
      `${JSON.stringify({
        schemaVersion: 1,
        kind: "request",
        requestId: `request-${operation}`,
        correlationId: `correlation-${index + 1}`,
        processGeneration: 11,
        operation,
        payload: {},
      })}\n`,
    );
  }
  child.stdin.end();
  const [code] = await once(child, "close");
  assert.equal(code, 0, stderr);
  assert.equal(stderr, "");
  const responses = stdout
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.equal(responses.length, 3);
  assert.deepEqual(
    responses.map((response) => response.status),
    ["ok", "ok", "ok"],
  );
  assert.equal(responses[0].payload.runtime, "pi");
  assert.equal(responses[0].payload.processGeneration, 11);
  assert.equal(responses[1].payload.nativeVersion, "0.80.10");
  assert.equal(responses[2].payload.stopped, true);
});

test("real process refuses credential-bearing inherited environment without echoing its value", async () => {
  const marker = "must-not-appear-in-diagnostics";
  const child = spawn(process.execPath, [main, "--generation=12"], {
    env: { PATH: process.env.PATH ?? "", API_KEY: marker },
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8").on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.setEncoding("utf8").on("data", (chunk) => {
    stderr += chunk;
  });
  const [code] = await once(child, "close");
  assert.notEqual(code, 0);
  assert.equal(stdout.includes(marker), false);
  assert.equal(stderr.includes(marker), false);
  assert.match(stderr, /environment may not carry credentials/);
});

test("real no-network process preflights the exact Pi model before constructing an SDK Agent", async () => {
  const child = spawn(
    process.execPath,
    ["--permission", `--allow-fs-read=${sidecarRoot}`, main, "--generation=13"],
    {
      env: { PATH: process.env.PATH ?? "" },
      stdio: ["pipe", "pipe", "pipe"],
    },
  );
  let stdout = "";
  let stderr = "";
  child.stdout.setEncoding("utf8").on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.setEncoding("utf8").on("data", (chunk) => {
    stderr += chunk;
  });
  const requests = [
    {
      schemaVersion: 1,
      kind: "request",
      requestId: "request-preflight",
      correlationId: "correlation-preflight",
      processGeneration: 13,
      operation: "model.preflight",
      payload: {
        provider: "openai",
        modelId: "gpt-4o-mini",
      },
    },
    {
      schemaVersion: 1,
      kind: "request",
      requestId: "request-create",
      correlationId: "correlation-create",
      processGeneration: 13,
      operation: "session.create",
      workspaceId: "workspace-native",
      sessionId: "session-native",
      payload: {
        modelRoute: {
          provider: "openai",
          modelId: "gpt-4o-mini",
          baseUrl: "https://api.openai.com/v1",
        },
        eligibleTools: ["c4os_read_resource", "c4os_propose_action"],
      },
    },
    {
      schemaVersion: 1,
      kind: "request",
      requestId: "request-shutdown",
      correlationId: "correlation-shutdown",
      processGeneration: 13,
      operation: "shutdown",
      payload: {},
    },
  ];
  for (const request of requests)
    child.stdin.write(`${JSON.stringify(request)}\n`);
  child.stdin.end();
  const [code] = await once(child, "close");
  assert.equal(code, 0, stderr);
  assert.equal(stderr, "");
  const responses = stdout
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.equal(responses.length, 3);
  assert.equal(responses[0].status, "ok");
  assert.deepEqual(responses[0].payload, {
    available: true,
    provider: "openai",
    modelId: "gpt-4o-mini",
    nativeVersion: "0.80.10",
  });
  assert.equal(responses[1].status, "ok");
  assert.equal(responses[1].payload.sessionId, "session-native");
  assert.equal(responses[1].payload.nativeSessionId, "session-native");
  assert.equal(responses[1].payload.persistence, "c4os-authoritative");
  assert.equal(responses[2].payload.stopped, true);
});

function testTlsTrustDescriptor() {
  const baseUrl = "https://127.0.0.1:4443/v1";
  const tlsCaPem = rootCertificates[0];
  return {
    schemaVersion: 1,
    capabilities: [
      {
        providerId: "provider-openai",
        nativeProviderId: "openai",
        baseUrl,
        baseUrlSha256: sha256(baseUrl),
        tlsCaPem,
        tlsCaSha256: sha256(tlsCaPem),
      },
    ],
  };
}

function sha256(value) {
  return `sha256:${createHash("sha256").update(value, "utf8").digest("hex")}`;
}
