import { randomUUID } from "node:crypto";
import { EventEmitter } from "node:events";
import { closeSync, readSync, writeSync } from "node:fs";

export const CREDENTIAL_SCHEMA_VERSION = 3;
export const CREDENTIAL_FD_ENV = "C4OS_OPENCODE_CREDENTIAL_FD";

const MAX_HEADER_BYTES = 4 * 1024;
const MAX_SECRET_BYTES = 64 * 1024;
const MAX_PENDING_REQUESTS = 64;
const MAX_CONSUMED_LEASES = 4096;
const MAX_BUFFER_BYTES =
  MAX_PENDING_REQUESTS * (MAX_HEADER_BYTES + MAX_SECRET_BYTES);
const MAX_TTL_MS = 30_000;
const DEFAULT_WAIT_TIMEOUT_MS = 10_000;
const DESCRIPTOR_READ_CHUNK_BYTES = 4 * 1024;
const SAFE_ID = /^[A-Za-z0-9][A-Za-z0-9._:@/-]{0,191}$/u;
const ALLOWED_HEADERS = new Set([
  "Authorization",
  "x-api-key",
  "x-goog-api-key",
]);
const REJECTION_CODES = new Set([
  "authorization_expired",
  "backpressure_exceeded",
  "binding_mismatch",
  "credential_unavailable",
  "lease_replay",
  "provider_unavailable",
]);

export class CredentialProtocolError extends Error {
  constructor(code) {
    super(`C4OS provider credential channel rejected the frame (${code})`);
    this.name = "CredentialProtocolError";
    this.code = code;
  }
}

function fail(code) {
  throw new CredentialProtocolError(code);
}

function safeId(value, field) {
  if (typeof value !== "string" || !SAFE_ID.test(value)) {
    fail(`invalid_${field}`);
  }
  return value;
}

function parseGeneration(value) {
  if (!/^[1-9][0-9]{0,9}$/u.test(value ?? "")) {
    fail("invalid_process_generation");
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed > 0xffffffff) {
    fail("invalid_process_generation");
  }
  return parsed;
}

function operationIdFromNativeMessage(nativeMessageId) {
  const accepted = safeId(nativeMessageId, "native_message_id");
  if (!accepted.startsWith("msg_")) fail("invalid_native_message_id");
  return safeId(accepted.slice(4), "operation_id");
}

function exactBinding(binding) {
  const nativeSessionId = safeId(binding?.nativeSessionId, "native_session_id");
  const providerId = safeId(binding?.providerId, "provider_id");
  const modelId = safeId(binding?.modelId, "model_id");
  const operationId = safeId(binding?.operationId, "operation_id");
  const nativeMessageId = safeId(binding?.nativeMessageId, "native_message_id");
  if (operationIdFromNativeMessage(nativeMessageId) !== operationId) {
    fail("invalid_operation_binding");
  }
  return Object.freeze({
    nativeSessionId,
    providerId,
    modelId,
    operationId,
    nativeMessageId,
  });
}

function validateCredentialMetadata(metadata, processGeneration) {
  if (
    metadata === null ||
    typeof metadata !== "object" ||
    Array.isArray(metadata)
  ) {
    fail("invalid_metadata");
  }
  const allowed = new Set([
    "schemaVersion",
    "kind",
    "requestId",
    "leaseId",
    "authorizationId",
    "processGeneration",
    "nativeSessionId",
    "providerId",
    "modelId",
    "operationId",
    "nativeMessageId",
    "headerName",
    "headerPrefix",
    "secretLength",
    "ttlMs",
  ]);
  if (Object.keys(metadata).some((key) => !allowed.has(key))) {
    fail("unknown_metadata_field");
  }
  if (
    metadata.schemaVersion !== CREDENTIAL_SCHEMA_VERSION ||
    metadata.kind !== "providerCredential"
  ) {
    fail("invalid_envelope");
  }
  safeId(metadata.requestId, "request_id");
  safeId(metadata.leaseId, "lease_id");
  safeId(metadata.authorizationId, "authorization_id");
  const binding = exactBinding(metadata);
  if (metadata.processGeneration !== processGeneration) {
    fail("invalid_process_generation");
  }
  if (!ALLOWED_HEADERS.has(metadata.headerName)) {
    fail("invalid_header");
  }
  if (
    !["", "Bearer "].includes(metadata.headerPrefix) ||
    (metadata.headerName === "Authorization" &&
      metadata.headerPrefix !== "Bearer ") ||
    (metadata.headerName !== "Authorization" && metadata.headerPrefix !== "")
  ) {
    fail("invalid_header_prefix");
  }
  if (
    !Number.isSafeInteger(metadata.secretLength) ||
    metadata.secretLength < 1 ||
    metadata.secretLength > MAX_SECRET_BYTES
  ) {
    fail("invalid_secret_length");
  }
  if (
    !Number.isSafeInteger(metadata.ttlMs) ||
    metadata.ttlMs < 1 ||
    metadata.ttlMs > MAX_TTL_MS
  ) {
    fail("invalid_ttl");
  }
  return Object.freeze({ ...metadata, ...binding });
}

function validateRejection(metadata) {
  const allowed = new Set(["schemaVersion", "kind", "requestId", "code"]);
  if (
    metadata === null ||
    typeof metadata !== "object" ||
    Array.isArray(metadata) ||
    Object.keys(metadata).some((key) => !allowed.has(key)) ||
    metadata.schemaVersion !== CREDENTIAL_SCHEMA_VERSION ||
    metadata.kind !== "providerCredentialRejected" ||
    !REJECTION_CODES.has(metadata.code)
  ) {
    fail("invalid_rejection");
  }
  safeId(metadata.requestId, "request_id");
  return Object.freeze({ ...metadata });
}

function sameBinding(left, right) {
  return (
    left.nativeSessionId === right.nativeSessionId &&
    left.providerId === right.providerId &&
    left.modelId === right.modelId &&
    left.operationId === right.operationId &&
    left.nativeMessageId === right.nativeMessageId
  );
}

function decodeSecret(secret) {
  let value;
  try {
    value = new TextDecoder("utf-8", { fatal: true }).decode(secret);
  } catch {
    fail("invalid_secret_encoding");
  }
  if (
    !value ||
    value.includes("\0") ||
    value.includes("\r") ||
    value.includes("\n")
  ) {
    fail("invalid_secret_encoding");
  }
  return value;
}

export class InheritedCredentialTransport extends EventEmitter {
  constructor(fd, operations = {}) {
    super();
    if (!Number.isSafeInteger(fd) || fd < 64 || fd > 1023) {
      fail("invalid_credential_fd");
    }
    this.fd = fd;
    this.readSync = operations.readSync ?? readSync;
    this.writeSync = operations.writeSync ?? writeSync;
    this.closeSync = operations.closeSync ?? closeSync;
    this.closed = false;
  }

  write(frame, callback) {
    if (this.closed) {
      callback?.(new Error("credential descriptor is closed"));
      return false;
    }
    try {
      let offset = 0;
      while (offset < frame.length) {
        const written = this.writeSync(
          this.fd,
          frame,
          offset,
          frame.length - offset,
        );
        if (!Number.isSafeInteger(written) || written < 1) {
          throw new Error("credential descriptor write made no progress");
        }
        offset += written;
      }
      callback?.();
      const response = this.#readResponse();
      this.emit("data", response);
      response.fill(0);
      return true;
    } catch (error) {
      callback?.(error);
      this.emit("error", error);
      return false;
    }
  }

  destroy() {
    if (this.closed) return;
    this.closed = true;
    try {
      this.closeSync(this.fd);
    } finally {
      this.emit("close");
    }
  }

  #readResponse() {
    const maximum = MAX_HEADER_BYTES + 1 + MAX_SECRET_BYTES;
    const response = Buffer.alloc(maximum);
    let length = 0;
    let expected;
    for (;;) {
      if (expected !== undefined && length === expected) {
        return Buffer.from(response.subarray(0, length));
      }
      if (length >= maximum) fail("response_buffer_exceeded");
      const remaining =
        expected === undefined
          ? Math.min(DESCRIPTOR_READ_CHUNK_BYTES, MAX_HEADER_BYTES + 1 - length)
          : expected - length;
      if (remaining < 1) fail("invalid_metadata");
      const read = this.readSync(this.fd, response, length, remaining, null);
      if (!Number.isSafeInteger(read) || read < 1) {
        fail("transport_closed");
      }
      length += read;
      if (expected !== undefined) continue;
      const newline = response.subarray(0, length).indexOf(0x0a);
      if (newline === -1) {
        if (length > MAX_HEADER_BYTES) fail("metadata_too_large");
        continue;
      }
      if (newline === 0 || newline > MAX_HEADER_BYTES) fail("invalid_metadata");
      let metadata;
      try {
        metadata = JSON.parse(response.subarray(0, newline).toString("utf8"));
      } catch {
        fail("invalid_metadata_json");
      }
      if (metadata?.kind === "providerCredentialRejected") {
        expected = newline + 1;
      } else if (
        metadata?.kind === "providerCredential" &&
        Number.isSafeInteger(metadata.secretLength) &&
        metadata.secretLength >= 1 &&
        metadata.secretLength <= MAX_SECRET_BYTES
      ) {
        expected = newline + 1 + metadata.secretLength;
      } else {
        fail("invalid_metadata");
      }
      if (length > expected) fail("unexpected_credential_response");
    }
  }
}

export class ProviderCredentialChannel {
  static fromEnvironment(environment = process.env, operations = {}) {
    const rawFd = environment[CREDENTIAL_FD_ENV];
    if (!/^[0-9]{1,4}$/u.test(rawFd ?? "")) fail("invalid_credential_fd");
    const fd = Number.parseInt(rawFd, 10);
    if (fd < 64 || fd > 1023) fail("invalid_credential_fd");
    delete environment[CREDENTIAL_FD_ENV];
    return new ProviderCredentialChannel(
      new InheritedCredentialTransport(fd, operations),
      parseGeneration(environment.C4OS_OPENCODE_PROCESS_GENERATION),
    );
  }

  constructor(transport, processGeneration, options = {}) {
    if (
      !transport ||
      typeof transport.on !== "function" ||
      typeof transport.write !== "function"
    ) {
      throw new TypeError(
        "A preconnected duplex provider credential transport is required",
      );
    }
    if (!Number.isSafeInteger(processGeneration) || processGeneration < 1) {
      fail("invalid_process_generation");
    }
    this.transport = transport;
    this.processGeneration = processGeneration;
    this.now = options.now ?? (() => Date.now());
    this.waitTimeoutMs = options.waitTimeoutMs ?? DEFAULT_WAIT_TIMEOUT_MS;
    this.requestId = options.requestId ?? (() => `request:${randomUUID()}`);
    if (
      typeof this.now !== "function" ||
      typeof this.requestId !== "function" ||
      !Number.isSafeInteger(this.waitTimeoutMs) ||
      this.waitTimeoutMs < 1 ||
      this.waitTimeoutMs > MAX_TTL_MS
    ) {
      fail("invalid_configuration");
    }
    this.buffer = Buffer.alloc(0);
    this.metadata = undefined;
    this.pending = new Map();
    this.consumed = new Set();
    this.closed = false;
    transport.on("data", (chunk) => this.#acceptData(chunk));
    transport.on("error", () => this.#close("transport_error"));
    transport.on("close", () => this.#close("transport_closed"));
  }

  get pendingCount() {
    return this.pending.size;
  }

  get consumedCount() {
    return this.consumed.size;
  }

  request(binding) {
    if (this.closed)
      return Promise.reject(new CredentialProtocolError("transport_closed"));
    if (this.pending.size >= MAX_PENDING_REQUESTS) {
      return Promise.reject(
        new CredentialProtocolError("request_backpressure_exceeded"),
      );
    }
    const accepted = exactBinding(binding);
    const requestId = safeId(this.requestId(), "request_id");
    if (this.pending.has(requestId)) {
      return Promise.reject(new CredentialProtocolError("request_id_replay"));
    }
    const frame = Buffer.from(
      `${JSON.stringify({
        schemaVersion: CREDENTIAL_SCHEMA_VERSION,
        kind: "providerCredentialRequest",
        requestId,
        processGeneration: this.processGeneration,
        ...accepted,
      })}\n`,
      "utf8",
    );
    if (frame.length > MAX_HEADER_BYTES) {
      frame.fill(0);
      return Promise.reject(new CredentialProtocolError("request_too_large"));
    }
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(requestId);
        reject(new CredentialProtocolError("credential_binding_unavailable"));
      }, this.waitTimeoutMs);
      timeout.unref?.();
      this.pending.set(requestId, {
        binding: accepted,
        requestedAt: this.now(),
        resolve,
        reject,
        timeout,
      });
      this.transport.write(frame, (error) => {
        frame.fill(0);
        if (!error) return;
        const pending = this.pending.get(requestId);
        if (!pending) return;
        clearTimeout(pending.timeout);
        this.pending.delete(requestId);
        pending.reject(new CredentialProtocolError("transport_error"));
      });
    });
  }

  acceptForTest(metadata, secret) {
    this.#acceptResponse(metadata, Buffer.from(secret));
  }

  dispose() {
    this.#close("transport_closed");
    this.transport.destroy?.();
  }

  #acceptData(chunk) {
    if (this.closed) return;
    try {
      const next = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
      if (
        next.length > MAX_BUFFER_BYTES ||
        this.buffer.length > MAX_BUFFER_BYTES - next.length
      ) {
        fail("response_buffer_exceeded");
      }
      this.buffer = Buffer.concat([this.buffer, next]);
      for (;;) {
        if (this.metadata === undefined) {
          const newline = this.buffer.indexOf(0x0a);
          if (newline === -1) {
            if (this.buffer.length > MAX_HEADER_BYTES)
              fail("metadata_too_large");
            return;
          }
          if (newline === 0 || newline > MAX_HEADER_BYTES)
            fail("invalid_metadata");
          let parsed;
          try {
            parsed = JSON.parse(
              this.buffer.subarray(0, newline).toString("utf8"),
            );
          } catch {
            fail("invalid_metadata_json");
          }
          this.buffer = this.buffer.subarray(newline + 1);
          if (parsed?.kind === "providerCredentialRejected") {
            this.#acceptRejection(validateRejection(parsed));
            continue;
          }
          this.metadata = validateCredentialMetadata(
            parsed,
            this.processGeneration,
          );
        }
        if (this.buffer.length < this.metadata.secretLength) return;
        const secret = Buffer.from(
          this.buffer.subarray(0, this.metadata.secretLength),
        );
        this.buffer.fill(0, 0, this.metadata.secretLength);
        this.buffer = this.buffer.subarray(this.metadata.secretLength);
        const metadata = this.metadata;
        this.metadata = undefined;
        this.#acceptResponse(metadata, secret);
      }
    } catch (error) {
      this.#close(
        error instanceof CredentialProtocolError ? error.code : "invalid_frame",
      );
    }
  }

  #acceptRejection(metadata) {
    const pending = this.pending.get(metadata.requestId);
    if (!pending) fail("unexpected_rejection");
    clearTimeout(pending.timeout);
    this.pending.delete(metadata.requestId);
    pending.reject(new CredentialProtocolError(metadata.code));
  }

  #acceptResponse(metadata, secret) {
    if (this.closed) {
      secret.fill(0);
      fail("transport_closed");
    }
    const accepted = validateCredentialMetadata(
      metadata,
      this.processGeneration,
    );
    if (!Buffer.isBuffer(secret) || secret.length !== accepted.secretLength) {
      secret.fill(0);
      fail("invalid_secret_length");
    }
    if (this.consumed.has(accepted.leaseId)) {
      secret.fill(0);
      fail("lease_replay");
    }
    const pending = this.pending.get(accepted.requestId);
    if (!pending) {
      secret.fill(0);
      fail("unexpected_credential_response");
    }
    if (!sameBinding(pending.binding, accepted)) {
      secret.fill(0);
      fail("credential_binding_substitution");
    }
    if (this.now() - pending.requestedAt > accepted.ttlMs) {
      clearTimeout(pending.timeout);
      this.pending.delete(accepted.requestId);
      secret.fill(0);
      pending.reject(new CredentialProtocolError("credential_lease_expired"));
      return;
    }
    const value = decodeSecret(secret);
    secret.fill(0);
    clearTimeout(pending.timeout);
    this.pending.delete(accepted.requestId);
    this.consumed.add(accepted.leaseId);
    if (this.consumed.size > MAX_CONSUMED_LEASES) {
      pending.reject(new CredentialProtocolError("lease_history_exhausted"));
      this.#close("lease_history_exhausted");
      return;
    }
    pending.resolve({
      headerName: accepted.headerName,
      headerValue: `${accepted.headerPrefix}${value}`,
    });
  }

  #close(code) {
    if (this.closed) return;
    this.closed = true;
    this.buffer.fill(0);
    this.buffer = Buffer.alloc(0);
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timeout);
      pending.reject(new CredentialProtocolError(code));
    }
    this.pending.clear();
  }
}
