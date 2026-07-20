import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import {
  chmodSync,
  closeSync,
  mkdtempSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

const SERVER_SECRET = "c4os-loopback-password-0123456789abcdef";
const PROVIDER_SECRET_SENTINEL = "c4os-provider-secret-must-not-appear";

function waitForExit(child) {
  return new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (code, signal) => resolve({ code, signal }));
  });
}

test("keeps server and provider credentials descriptor-only for the C4OS 1.18.3 build", async () => {
  const directory = mkdtempSync(join(tmpdir(), "c4os-opencode-launcher-"));
  const secretPath = join(directory, "server-secret");
  const providerSecretPath = join(directory, "provider-secret");
  const readinessPath = join(directory, "readiness-capability");
  const testTlsTrustPath = join(directory, "test-tls-trust-capability");
  const childPath = join(directory, "fake-opencode.mjs");
  const outputPath = join(directory, "child-state.json");
  writeFileSync(secretPath, SERVER_SECRET, { mode: 0o600 });
  writeFileSync(providerSecretPath, PROVIDER_SECRET_SENTINEL, { mode: 0o600 });
  writeFileSync(readinessPath, "", { mode: 0o600 });
  writeFileSync(testTlsTrustPath, "c4os-test-tls-trust-descriptor", {
    mode: 0o600,
  });
  writeFileSync(
    childPath,
    `#!/usr/bin/env node
import { fstatSync, readFileSync, writeFileSync, writeSync } from "node:fs";
const state = {
  argv: process.argv.slice(2),
  environment: process.env,
  serverDescriptorReadable:
    fstatSync(64).isFile() &&
    readFileSync(64, "utf8") === "c4os-loopback-password-0123456789abcdef",
  brokerDescriptorOpen: fstatSync(65).isCharacterDevice(),
  credentialDescriptorReadable:
    fstatSync(66).isFile() &&
    readFileSync(66, "utf8") === "c4os-provider-secret-must-not-appear",
  readinessDescriptorOpen: fstatSync(67).isFile(),
  testTlsTrustDescriptorReadable:
    fstatSync(68).isFile() &&
    readFileSync(68, "utf8") === "c4os-test-tls-trust-descriptor",
};
writeSync(67, Buffer.from([0x01]));
writeFileSync(process.env.C4OS_LAUNCHER_TEST_OUTPUT, JSON.stringify(state));
`,
    { mode: 0o700 },
  );
  chmodSync(childPath, 0o700);

  const secretDescriptor = openSync(secretPath, "r");
  const brokerDescriptor = openSync("/dev/null", "r");
  const credentialDescriptor = openSync(providerSecretPath, "r");
  const readinessDescriptor = openSync(readinessPath, "r+");
  const testTlsTrustDescriptor = openSync(testTlsTrustPath, "r");
  const stdio = Array(69).fill("ignore");
  stdio[64] = secretDescriptor;
  stdio[65] = brokerDescriptor;
  stdio[66] = credentialDescriptor;
  stdio[67] = readinessDescriptor;
  stdio[68] = testTlsTrustDescriptor;

  try {
    const launcher = spawn(
      process.execPath,
      [
        new URL("./main.mjs", import.meta.url).pathname,
        "--",
        childPath,
        "serve",
        "--hostname",
        "127.0.0.1",
        "--port",
        "4096",
      ],
      {
        env: {
          PATH: process.env.PATH,
          C4OS_OPENCODE_SECRET_FD: "64",
          C4OS_OPENCODE_BROKER_FD: "65",
          C4OS_OPENCODE_CREDENTIAL_FD: "66",
          C4OS_OPENCODE_READY_FD: "67",
          C4OS_OPENCODE_TEST_TLS_TRUST_FD: "68",
          C4OS_OPENCODE_PROCESS_GENERATION: "7",
          C4OS_OPENCODE_CONFIG_CONTENT: '{"permission":{}}',
          C4OS_LAUNCHER_TEST_OUTPUT: outputPath,
        },
        stdio,
      },
    );
    assert.deepEqual(await waitForExit(launcher), { code: 0, signal: null });

    const state = JSON.parse(readFileSync(outputPath, "utf8"));
    assert.deepEqual(state.argv, [
      "serve",
      "--hostname",
      "127.0.0.1",
      "--port",
      "4096",
    ]);
    assert.equal(state.brokerDescriptorOpen, true);
    assert.equal(state.serverDescriptorReadable, true);
    assert.equal(state.credentialDescriptorReadable, true);
    assert.equal(state.readinessDescriptorOpen, true);
    assert.equal(state.testTlsTrustDescriptorReadable, true);
    assert.equal(state.environment.C4OS_OPENCODE_SECRET_FD, undefined);
    assert.equal(state.environment.C4OS_OPENCODE_SERVER_SECRET_FD, "64");
    assert.equal(state.environment.C4OS_OPENCODE_CONFIG_CONTENT, undefined);
    assert.equal(state.environment.C4OS_OPENCODE_BROKER_FD, "65");
    assert.equal(state.environment.C4OS_OPENCODE_CREDENTIAL_FD, "66");
    assert.equal(state.environment.C4OS_OPENCODE_READY_FD, "67");
    assert.equal(state.environment.C4OS_OPENCODE_TEST_TLS_TRUST_FD, "68");
    assert.equal(state.environment.OPENCODE_SERVER_USERNAME, undefined);
    assert.equal(state.environment.OPENCODE_SERVER_PASSWORD, undefined);
    assert.equal(
      state.environment.OPENCODE_CONFIG_CONTENT,
      '{"permission":{}}',
    );
    assert.equal(
      JSON.stringify(state).includes(PROVIDER_SECRET_SENTINEL),
      false,
    );
    assert.equal(JSON.stringify(state).includes(SERVER_SECRET), false);
    assert.equal(state.argv.includes(SERVER_SECRET), false);
    assert.deepEqual(readFileSync(readinessPath), Buffer.from([0x01]));
  } finally {
    closeSync(secretDescriptor);
    closeSync(brokerDescriptor);
    closeSync(credentialDescriptor);
    closeSync(readinessDescriptor);
    closeSync(testTlsTrustDescriptor);
    rmSync(directory, { recursive: true, force: true });
  }
});

test("rejects a readiness descriptor collision before native spawn", async () => {
  const first = openSync("/dev/null", "r");
  const second = openSync("/dev/null", "r");
  const third = openSync("/dev/null", "r");
  const stdio = Array(67).fill("ignore");
  stdio[64] = first;
  stdio[65] = second;
  stdio[66] = third;
  try {
    const launcher = spawn(
      process.execPath,
      [
        new URL("./main.mjs", import.meta.url).pathname,
        "--",
        "/usr/bin/true",
        "serve",
        "--hostname",
        "127.0.0.1",
        "--port",
        "4096",
      ],
      {
        env: {
          C4OS_OPENCODE_SECRET_FD: "64",
          C4OS_OPENCODE_BROKER_FD: "65",
          C4OS_OPENCODE_CREDENTIAL_FD: "66",
          C4OS_OPENCODE_READY_FD: "66",
          C4OS_OPENCODE_PROCESS_GENERATION: "7",
          C4OS_OPENCODE_CONFIG_CONTENT: '{"permission":{}}',
        },
        stdio,
      },
    );
    assert.deepEqual(await waitForExit(launcher), {
      code: 78,
      signal: null,
    });
  } finally {
    closeSync(first);
    closeSync(second);
    closeSync(third);
  }
});
