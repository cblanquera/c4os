import { spawn } from "node:child_process";
import { closeSync } from "node:fs";

const CONFIG_ENV = "C4OS_OPENCODE_CONFIG_CONTENT";
const SECRET_FD_ENV = "C4OS_OPENCODE_SECRET_FD";
const SERVER_SECRET_FD_ENV = "C4OS_OPENCODE_SERVER_SECRET_FD";
const BROKER_FD_ENV = "C4OS_OPENCODE_BROKER_FD";
const CREDENTIAL_FD_ENV = "C4OS_OPENCODE_CREDENTIAL_FD";
const READY_FD_ENV = "C4OS_OPENCODE_READY_FD";
const TEST_TLS_TRUST_FD_ENV = "C4OS_OPENCODE_TEST_TLS_TRUST_FD";
const PROCESS_GENERATION_ENV = "C4OS_OPENCODE_PROCESS_GENERATION";

function fail(reason = "unavailable") {
  process.stderr.write(`c4os-opencode-launcher:${reason}\n`);
  process.exitCode = 78;
}

function validDescriptor(value) {
  if (!/^[0-9]+$/.test(value ?? "")) return false;
  const descriptor = Number(value);
  return (
    Number.isSafeInteger(descriptor) &&
    descriptor >= 64 &&
    descriptor <= 1023 &&
    String(descriptor) === value
  );
}

function validGeneration(value) {
  if (!/^[1-9][0-9]{0,9}$/.test(value ?? "")) return false;
  const generation = Number(value);
  return Number.isSafeInteger(generation) && generation <= 0xffffffff;
}

function childStdio(...descriptors) {
  const stdio = Array(Math.max(...descriptors) + 1).fill("ignore");
  for (const descriptor of descriptors) {
    stdio[descriptor] = descriptor;
  }
  return stdio;
}

function validArguments(executable, args) {
  if (!executable?.startsWith("/") || args[0] !== "serve") return false;
  if (args.some((argument) => /[\u0000\r\n]/.test(argument))) return false;
  if (
    args.some(
      (argument) => argument === "--mdns" || argument.startsWith("--cors"),
    )
  )
    return false;
  return args.includes("--hostname") && args.includes("--port");
}

async function main() {
  const [separator, executable, ...nativeArguments] = process.argv.slice(2);
  const descriptorText = process.env[SECRET_FD_ENV];
  const brokerDescriptorText = process.env[BROKER_FD_ENV];
  const credentialDescriptorText = process.env[CREDENTIAL_FD_ENV];
  const readyDescriptorText = process.env[READY_FD_ENV];
  const testTlsTrustDescriptorText = process.env[TEST_TLS_TRUST_FD_ENV];
  const descriptorTexts = [
    descriptorText,
    brokerDescriptorText,
    credentialDescriptorText,
    readyDescriptorText,
    ...(testTlsTrustDescriptorText === undefined
      ? []
      : [testTlsTrustDescriptorText]),
  ];
  const processGeneration = process.env[PROCESS_GENERATION_ENV];
  const configuration = process.env[CONFIG_ENV];
  if (
    separator !== "--" ||
    !validDescriptor(descriptorText) ||
    !validDescriptor(brokerDescriptorText) ||
    !validDescriptor(credentialDescriptorText) ||
    !validDescriptor(readyDescriptorText) ||
    (testTlsTrustDescriptorText !== undefined &&
      !validDescriptor(testTlsTrustDescriptorText)) ||
    new Set(descriptorTexts).size !== descriptorTexts.length ||
    !validGeneration(processGeneration) ||
    !configuration ||
    !validArguments(executable, nativeArguments)
  ) {
    fail("invalid-contract");
    return;
  }

  const childEnvironment = { ...process.env };
  delete childEnvironment[SECRET_FD_ENV];
  delete childEnvironment[CONFIG_ENV];
  delete childEnvironment.OPENCODE_SERVER_USERNAME;
  delete childEnvironment.OPENCODE_SERVER_PASSWORD;
  // The pinned C4OS build based on OpenCode 1.18.3 consumes this descriptor
  // only while constructing its authenticated listener, clears the descriptor
  // metadata, zeroes the temporary byte buffer, and retains authentication
  // only in worker memory.
  childEnvironment[SERVER_SECRET_FD_ENV] = descriptorText;
  childEnvironment[BROKER_FD_ENV] = brokerDescriptorText;
  childEnvironment[CREDENTIAL_FD_ENV] = credentialDescriptorText;
  childEnvironment[READY_FD_ENV] = readyDescriptorText;
  if (testTlsTrustDescriptorText !== undefined) {
    childEnvironment[TEST_TLS_TRUST_FD_ENV] = testTlsTrustDescriptorText;
  }
  childEnvironment.OPENCODE_CONFIG_CONTENT = configuration;
  childEnvironment.OPENCODE_DISABLE_AUTOUPDATE = "true";
  childEnvironment.OPENCODE_DISABLE_PROJECT_CONFIG = "true";
  childEnvironment.OPENCODE_DISABLE_PRUNE = "true";
  childEnvironment.OPENCODE_CLIENT = "c4os";

  const args = nativeArguments;
  let child;
  try {
    child = spawn(executable, args, {
      cwd: process.cwd(),
      env: childEnvironment,
      // Node/libuv closes descriptors absent from this map. Keep exactly the
      // authenticated Action Gateway broker and the separate, one-operation
      // provider-credential capability plus the one-shot native readiness
      // capability and optional test-only TLS trust capability at their
      // bounded descriptor numbers.
      stdio: childStdio(
        ...descriptorTexts.map(Number),
      ),
    });
  } catch {
    fail("native-spawn-rejected");
  }
  if (!child) return;

  try {
    closeSync(Number(descriptorText));
    closeSync(Number(brokerDescriptorText));
    closeSync(Number(credentialDescriptorText));
    closeSync(Number(readyDescriptorText));
    if (testTlsTrustDescriptorText !== undefined) {
      closeSync(Number(testTlsTrustDescriptorText));
    }
  } catch {
    child.kill();
    fail("descriptor-close-failed");
    return;
  }

  child.once("error", (error) => {
    const code = [
      "EACCES",
      "ENOENT",
      "EMFILE",
      "EINVAL",
      "EBADF",
      "EAGAIN",
    ].includes(error?.code)
      ? error.code.toLowerCase()
      : "unknown";
    fail(`native-process-error-${code}`);
  });
  child.once("exit", (code, signal) => {
    if (signal) process.kill(process.pid, signal);
    else process.exitCode = code ?? 1;
  });
}

await main();
