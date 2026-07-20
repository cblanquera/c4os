import { randomUUID } from "node:crypto";
import { EventEmitter } from "node:events";
import { close, read, write } from "node:fs";

export const BROKER_SCHEMA_VERSION = 1;
export const BROKER_FD_ENV = "C4OS_OPENCODE_BROKER_FD";
export const PROCESS_GENERATION_ENV = "C4OS_OPENCODE_PROCESS_GENERATION";
export const C4OS_TOOL_IDS = Object.freeze([
  "c4os_propose_action",
  "c4os_read_resource",
]);

const MAX_FRAME_BYTES = 512 * 1024;
const MAX_PENDING_REQUESTS = 256;
const DESCRIPTOR_READ_BYTES = 64 * 1024;
const MAX_QUEUED_WRITE_BYTES = 8 * 1024 * 1024;
const SAFE_ID = /^[A-Za-z0-9][A-Za-z0-9._:@-]{0,127}$/u;
const SECRET_KEY =
  /^(?:api[-_]?key|authorization|cookie|credential|password|secret|token)$/iu;
const SECRET_VALUE =
  /(?:\bBearer\s+[A-Za-z0-9._~+/=-]{8,}|\b(?:sk|rk|pk)-[A-Za-z0-9_-]{12,})/u;

export class BrokerProtocolError extends Error {
  constructor(code) {
    super(`C4OS broker protocol rejected the frame (${code})`);
    this.name = "BrokerProtocolError";
    this.code = code;
  }
}

function assertSafeId(value, field) {
  if (typeof value !== "string" || !SAFE_ID.test(value)) {
    throw new BrokerProtocolError(`invalid_${field}`);
  }
  return value;
}

function assertNoSecretSurface(value, depth = 0) {
  if (depth > 16) throw new BrokerProtocolError("payload_too_deep");
  if (typeof value === "string") {
    if (value.length > 64 * 1024)
      throw new BrokerProtocolError("value_too_large");
    if (SECRET_VALUE.test(value)) throw new BrokerProtocolError("secret_value");
    return;
  }
  if (value === null || typeof value === "number" || typeof value === "boolean")
    return;
  if (Array.isArray(value)) {
    if (value.length > 1024) throw new BrokerProtocolError("array_too_large");
    for (const item of value) assertNoSecretSurface(item, depth + 1);
    return;
  }
  if (typeof value !== "object")
    throw new BrokerProtocolError("unsupported_value");
  const entries = Object.entries(value);
  if (entries.length > 1024) throw new BrokerProtocolError("object_too_large");
  for (const [key, item] of entries) {
    if (SECRET_KEY.test(key)) throw new BrokerProtocolError("secret_field");
    assertNoSecretSurface(item, depth + 1);
  }
}

function encodeFrame(frame) {
  assertNoSecretSurface(frame);
  const encoded = `${JSON.stringify(frame)}\n`;
  if (Buffer.byteLength(encoded) > MAX_FRAME_BYTES) {
    throw new BrokerProtocolError("frame_too_large");
  }
  return encoded;
}

function validateResultFrame(frame) {
  if (frame === null || typeof frame !== "object" || Array.isArray(frame)) {
    throw new BrokerProtocolError("invalid_result");
  }
  if (
    frame.schemaVersion !== BROKER_SCHEMA_VERSION ||
    frame.kind !== "result"
  ) {
    throw new BrokerProtocolError("invalid_result_envelope");
  }
  assertSafeId(frame.correlationId, "correlation_id");
  if (!C4OS_TOOL_IDS.includes(frame.tool))
    throw new BrokerProtocolError("invalid_tool");
  if (!["result", "denied", "cancelled"].includes(frame.status)) {
    throw new BrokerProtocolError("invalid_status");
  }
  const allowed = new Set([
    "schemaVersion",
    "kind",
    "correlationId",
    "tool",
    "status",
    "payload",
    "reasonCode",
  ]);
  for (const key of Object.keys(frame)) {
    if (!allowed.has(key))
      throw new BrokerProtocolError("unknown_result_field");
  }
  assertNoSecretSurface(frame);
  return frame;
}

function parseGeneration(value) {
  if (!/^[1-9][0-9]{0,9}$/u.test(value ?? "")) {
    throw new BrokerProtocolError("invalid_process_generation");
  }
  return Number.parseInt(value, 10);
}

export class InheritedBrokerTransport extends EventEmitter {
  constructor(fd, operations = {}) {
    super();
    if (!Number.isSafeInteger(fd) || fd < 3 || fd > 1023) {
      throw new BrokerProtocolError("invalid_broker_fd");
    }
    this.fd = fd;
    this.read = operations.read ?? read;
    this.writeOperation = operations.write ?? write;
    this.closeOperation = operations.close ?? close;
    this.closed = false;
    this.reading = false;
    this.writing = false;
    this.writeQueue = [];
    this.queuedWriteBytes = 0;
    queueMicrotask(() => this.#readNext());
  }

  write(value, callback) {
    if (this.closed) throw new Error("broker descriptor is closed");
    const buffer = Buffer.isBuffer(value)
      ? Buffer.from(value)
      : Buffer.from(value, "utf8");
    if (
      buffer.length > MAX_FRAME_BYTES ||
      this.queuedWriteBytes > MAX_QUEUED_WRITE_BYTES - buffer.length
    ) {
      buffer.fill(0);
      throw new Error("broker descriptor write queue exceeded its bound");
    }
    this.writeQueue.push({ buffer, offset: 0, callback });
    this.queuedWriteBytes += buffer.length;
    this.#writeNext();
    return true;
  }

  destroy() {
    if (this.closed) return;
    this.closed = true;
    const error = new Error("broker descriptor is closed");
    for (const pending of this.writeQueue) {
      pending.buffer.fill(0);
      pending.callback?.(error);
    }
    this.writeQueue = [];
    this.queuedWriteBytes = 0;
    this.closeOperation(this.fd, () => this.emit("close"));
  }

  #readNext() {
    if (this.closed || this.reading) return;
    this.reading = true;
    const buffer = Buffer.alloc(DESCRIPTOR_READ_BYTES);
    this.read(this.fd, buffer, 0, buffer.length, null, (error, bytesRead) => {
      this.reading = false;
      if (this.closed) {
        buffer.fill(0);
        return;
      }
      if (error?.code === "EINTR") {
        buffer.fill(0);
        this.#readNext();
        return;
      }
      if (error || !Number.isSafeInteger(bytesRead) || bytesRead < 1) {
        buffer.fill(0);
        this.#fail(error ?? new Error("broker descriptor reached EOF"));
        return;
      }
      const chunk = Buffer.from(buffer.subarray(0, bytesRead));
      buffer.fill(0);
      this.emit("data", chunk);
      chunk.fill(0);
      this.#readNext();
    });
  }

  #writeNext() {
    if (this.closed || this.writing || this.writeQueue.length === 0) return;
    this.writing = true;
    const pending = this.writeQueue[0];
    this.writeOperation(
      this.fd,
      pending.buffer,
      pending.offset,
      pending.buffer.length - pending.offset,
      null,
      (error, bytesWritten) => {
        this.writing = false;
        if (this.closed) return;
        if (error?.code === "EINTR") {
          this.#writeNext();
          return;
        }
        if (error || !Number.isSafeInteger(bytesWritten) || bytesWritten < 1) {
          this.#fail(
            error ?? new Error("broker descriptor write made no progress"),
          );
          return;
        }
        pending.offset += bytesWritten;
        if (pending.offset < pending.buffer.length) {
          this.#writeNext();
          return;
        }
        this.writeQueue.shift();
        this.queuedWriteBytes -= pending.buffer.length;
        pending.buffer.fill(0);
        pending.callback?.();
        this.#writeNext();
      },
    );
  }

  #fail(error) {
    if (this.closed) return;
    this.emit("error", error);
    this.destroy();
  }
}

export class FdBrokerChannel {
  static fromEnvironment(environment = process.env, operations = {}) {
    const rawFd = environment[BROKER_FD_ENV];
    if (!/^[0-9]{1,4}$/u.test(rawFd ?? ""))
      throw new BrokerProtocolError("invalid_broker_fd");
    const fd = Number.parseInt(rawFd, 10);
    if (fd < 3 || fd > 1023) throw new BrokerProtocolError("invalid_broker_fd");
    delete environment[BROKER_FD_ENV];
    return new FdBrokerChannel(
      new InheritedBrokerTransport(fd, operations),
      parseGeneration(environment[PROCESS_GENERATION_ENV]),
    );
  }

  constructor(transport, processGeneration) {
    if (
      !transport ||
      typeof transport.write !== "function" ||
      typeof transport.on !== "function"
    ) {
      throw new TypeError("A preconnected duplex broker transport is required");
    }
    if (!Number.isSafeInteger(processGeneration) || processGeneration < 1) {
      throw new BrokerProtocolError("invalid_process_generation");
    }
    this.transport = transport;
    this.processGeneration = processGeneration;
    this.pending = new Map();
    this.buffer = Buffer.alloc(0);
    this.closed = false;
    transport.on("data", (chunk) => this.#acceptData(chunk));
    transport.on("error", () => this.#failAll("transport_error"));
    transport.on("close", () => this.#failAll("transport_closed"));
  }

  request(toolId, payload, context, { signal } = {}) {
    if (!C4OS_TOOL_IDS.includes(toolId))
      throw new BrokerProtocolError("invalid_tool");
    if (this.closed) throw new BrokerProtocolError("transport_closed");
    if (this.pending.size >= MAX_PENDING_REQUESTS)
      throw new BrokerProtocolError("pending_limit");
    const correlationId = randomUUID();
    const binding = {
      tool: toolId,
      sessionId: assertSafeId(context.sessionId, "session_id"),
      messageId: assertSafeId(context.messageId, "message_id"),
    };
    const frame = {
      schemaVersion: BROKER_SCHEMA_VERSION,
      kind: "proposal",
      correlationId,
      tool: toolId,
      sessionId: binding.sessionId,
      messageId: binding.messageId,
      processGeneration: this.processGeneration,
      payload,
    };
    const encoded = encodeFrame(frame);

    return new Promise((resolve, reject) => {
      const onAbort = () => {
        if (!this.pending.delete(correlationId)) return;
        try {
          this.transport.write(
            encodeFrame({
              schemaVersion: BROKER_SCHEMA_VERSION,
              kind: "cancel",
              correlationId,
              tool: toolId,
              sessionId: binding.sessionId,
              messageId: binding.messageId,
              processGeneration: this.processGeneration,
            }),
          );
        } finally {
          reject(new BrokerProtocolError("cancelled"));
        }
      };
      if (signal?.aborted) {
        reject(new BrokerProtocolError("cancelled"));
        return;
      }
      this.pending.set(correlationId, {
        resolve,
        reject,
        binding,
        signal,
        onAbort,
      });
      signal?.addEventListener("abort", onAbort, { once: true });
      try {
        this.transport.write(encoded);
      } catch {
        this.pending.delete(correlationId);
        signal?.removeEventListener("abort", onAbort);
        reject(new BrokerProtocolError("transport_write"));
      }
    });
  }

  close() {
    this.#failAll("transport_closed");
    this.transport.destroy?.();
  }

  #acceptData(chunk) {
    if (this.closed) return;
    const next = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    this.buffer = Buffer.concat([this.buffer, next]);
    if (this.buffer.length > MAX_FRAME_BYTES) {
      this.#failAll("frame_too_large");
      return;
    }
    let newline = this.buffer.indexOf(0x0a);
    while (newline !== -1) {
      const line = this.buffer.subarray(0, newline);
      this.buffer = this.buffer.subarray(newline + 1);
      if (line.length > 0) this.#acceptLine(line);
      if (this.closed) return;
      newline = this.buffer.indexOf(0x0a);
    }
  }

  #acceptLine(line) {
    let frame;
    try {
      frame = validateResultFrame(JSON.parse(line.toString("utf8")));
      const pending = this.pending.get(frame.correlationId);
      if (!pending) throw new BrokerProtocolError("unknown_correlation");
      if (frame.tool !== pending.binding.tool)
        throw new BrokerProtocolError("binding_mismatch");
      this.pending.delete(frame.correlationId);
      pending.signal?.removeEventListener("abort", pending.onAbort);
      pending.resolve(frame);
    } catch (error) {
      this.#failAll(
        error instanceof BrokerProtocolError ? error.code : "invalid_json",
      );
    }
  }

  #failAll(code) {
    if (this.closed) return;
    this.closed = true;
    const error = new BrokerProtocolError(code);
    for (const pending of this.pending.values()) {
      pending.signal?.removeEventListener("abort", pending.onAbort);
      pending.reject(error);
    }
    this.pending.clear();
    if (code !== "transport_closed") this.transport.destroy?.();
  }
}

export const brokerProtocol = Object.freeze({
  encodeFrame,
  validateResultFrame,
});
