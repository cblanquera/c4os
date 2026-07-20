#!/usr/bin/env node
import { createInterface } from "node:readline";
import { C4osPiSidecar } from "./adapter.mjs";
import {
  consumeTestTlsTrustDescriptor,
  CredentialLeaseBroker,
  PiSdkDriver,
  startCredentialChannel,
} from "./pi-sdk-driver.mjs";
import {
  PROTOCOL_SCHEMA_VERSION,
  ProtocolFault,
  assertSafeLaunch,
  encodeLine,
  parseRequestLine,
  safeDiagnosticMessage,
} from "./protocol.mjs";

assertSafeLaunch(process.argv.slice(2), process.env);
const processGeneration = integerArgument("--generation");
const credentialFd = optionalIntegerArgument("--credential-fd");
const testTlsTrustFd = optionalIntegerArgument("--tls-trust-fd");
if (credentialFd !== undefined && credentialFd === testTlsTrustFd) {
  throw new Error("Dedicated inherited descriptors must not collide");
}
const testTlsTrustCapabilities =
  testTlsTrustFd === undefined
    ? undefined
    : consumeTestTlsTrustDescriptor(testTlsTrustFd);
const credentials =
  credentialFd === undefined ? undefined : new CredentialLeaseBroker();
if (credentials) startCredentialChannel(credentialFd, credentials);

const writeEnvelope = (envelope) => process.stdout.write(encodeLine(envelope));
const sidecar = new C4osPiSidecar({
  driver: new PiSdkDriver({
    credentialBroker: credentials,
    testTlsTrustCapabilities,
  }),
  emitLine: writeEnvelope,
  processGeneration,
});

const lines = createInterface({
  input: process.stdin,
  crlfDelay: Infinity,
  terminal: false,
});
for await (const line of lines) {
  try {
    const request = parseRequestLine(line);
    writeEnvelope(await sidecar.handle(request));
  } catch (error) {
    const fault =
      error instanceof ProtocolFault
        ? error
        : new ProtocolFault("adapter_failure", safeDiagnosticMessage(error));
    writeEnvelope({
      schemaVersion: PROTOCOL_SCHEMA_VERSION,
      kind: "response",
      requestId: "invalid-request",
      correlationId: "invalid-correlation",
      processGeneration,
      status: "error",
      payload: { code: fault.code, message: safeDiagnosticMessage(fault) },
    });
  }
}

credentials?.clear();

function integerArgument(name) {
  const value = optionalIntegerArgument(name);
  if (value === undefined) throw new Error(`${name} is required`);
  return value;
}

function optionalIntegerArgument(name) {
  const prefix = `${name}=`;
  const matches = process.argv
    .slice(2)
    .filter((candidate) => candidate.startsWith(prefix));
  if (matches.length > 1)
    throw new Error(`${name} must be supplied at most once`);
  const [argument] = matches;
  if (!argument) return undefined;
  const value = Number(argument.slice(prefix.length));
  if (!Number.isSafeInteger(value) || value < 1)
    throw new Error(`${name} must be a positive integer`);
  return value;
}
