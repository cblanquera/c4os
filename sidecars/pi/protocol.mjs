import { Buffer } from "node:buffer";

export const PROTOCOL_SCHEMA_VERSION = 1;
export const PROTOCOL_NAME = "c4os.pi.ndjson.v1";
export const MAX_LINE_BYTES = 256 * 1024;
export const MAX_PAYLOAD_DEPTH = 12;
export const MAX_COLLECTION_ITEMS = 512;
export const MAX_STRING_BYTES = 64 * 1024;
export const MAX_DIRECT_IMAGE_BYTES = 128 * 1024;
export const MAX_IMAGE_BASE64_BYTES = Math.ceil(MAX_DIRECT_IMAGE_BYTES / 3) * 4;

const ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._:@/-]{0,127}$/;
const REQUEST_KEYS = new Set([
  "schemaVersion",
  "kind",
  "requestId",
  "correlationId",
  "processGeneration",
  "operation",
  "workspaceId",
  "sessionId",
  "turnId",
  "runId",
  "payload",
]);
const FORBIDDEN_FIELD =
  /(?:^|_)(?:api[_-]?key|access[_-]?token|refresh[_-]?token|password|passwd|secret|authorization|cookie|private[_-]?key)(?:$|_)/i;
const SECRET_VALUE =
  /(?:bearer\s+[A-Za-z0-9._~+/-]{8,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|[a-z][a-z0-9+.-]*:\/\/[^\s/:]+:[^\s/@]+@)/i;

export class ProtocolFault extends Error {
  constructor(code, message) {
    super(message);
    this.name = "ProtocolFault";
    this.code = code;
  }
}

export function parseRequestLine(line) {
  if (typeof line !== "string")
    throw new ProtocolFault("invalid_line", "Protocol input must be text");
  const bytes = Buffer.byteLength(line, "utf8");
  if (bytes === 0 || bytes > MAX_LINE_BYTES) {
    throw new ProtocolFault(
      "line_bounds",
      `Protocol line must be between 1 and ${MAX_LINE_BYTES} bytes`,
    );
  }
  if (line.includes("\n") || line.includes("\r")) {
    throw new ProtocolFault(
      "invalid_line",
      "Protocol input must contain exactly one JSON line",
    );
  }

  let value;
  try {
    value = JSON.parse(line);
  } catch {
    throw new ProtocolFault("invalid_json", "Protocol line is not valid JSON");
  }
  return validateRequest(value);
}

export function validateRequest(value) {
  requireRecord(value, "request");
  rejectUnknownKeys(value, REQUEST_KEYS, "request");
  if (
    value.schemaVersion !== PROTOCOL_SCHEMA_VERSION ||
    value.kind !== "request"
  ) {
    throw new ProtocolFault(
      "unsupported_protocol",
      "Unsupported request schema or kind",
    );
  }
  for (const key of ["requestId", "correlationId", "operation"])
    requireId(value[key], key);
  for (const key of ["workspaceId", "sessionId", "turnId", "runId"]) {
    if (value[key] !== undefined) requireId(value[key], key);
  }
  if (
    !Number.isSafeInteger(value.processGeneration) ||
    value.processGeneration < 1
  ) {
    throw new ProtocolFault(
      "invalid_generation",
      "processGeneration must be a positive safe integer",
    );
  }
  requireRecord(value.payload, "payload");
  inspectValue(value.payload, "payload", 0, new Set());
  return globalThis.structuredClone(value);
}

export function encodeLine(value) {
  inspectValue(value, "envelope", 0, new Set());
  const line = JSON.stringify(value);
  if (Buffer.byteLength(line, "utf8") > MAX_LINE_BYTES) {
    throw new ProtocolFault(
      "line_bounds",
      "Encoded protocol line exceeds the byte limit",
    );
  }
  return `${line}\n`;
}

export function safeDiagnosticMessage(error) {
  const message = error instanceof Error ? error.message : String(error);
  return redactText(message).slice(0, 512);
}

export function redactValue(value, depth = 0) {
  if (depth > MAX_PAYLOAD_DEPTH) return "[TRUNCATED]";
  if (typeof value === "string") return redactText(value).slice(0, 16_384);
  if (Array.isArray(value))
    return value
      .slice(0, MAX_COLLECTION_ITEMS)
      .map((item) => redactValue(item, depth + 1));
  if (value && typeof value === "object") {
    const output = {};
    for (const [key, item] of Object.entries(value).slice(
      0,
      MAX_COLLECTION_ITEMS,
    )) {
      output[key] = isForbiddenField(key)
        ? "[REDACTED]"
        : redactValue(item, depth + 1);
    }
    return output;
  }
  return value;
}

export function assertSafeLaunch(argv, env) {
  for (const value of argv) {
    if (
      SECRET_VALUE.test(String(value)) ||
      /(?:api[_-]?key|password|secret|token)=/i.test(String(value))
    ) {
      throw new ProtocolFault(
        "unsafe_launch",
        "Sidecar arguments may not carry credentials",
      );
    }
  }
  for (const key of Object.keys(env)) {
    if (isForbiddenField(key)) {
      throw new ProtocolFault(
        "unsafe_launch",
        "Sidecar environment may not carry credentials",
      );
    }
  }
}

export function requireId(value, field) {
  if (typeof value !== "string" || !ID_PATTERN.test(value)) {
    throw new ProtocolFault(
      "invalid_identifier",
      `${field} is not a bounded identifier`,
    );
  }
}

function inspectValue(value, path, depth, seen) {
  if (depth > MAX_PAYLOAD_DEPTH)
    throw new ProtocolFault("payload_bounds", `${path} is nested too deeply`);
  if (value === null || typeof value === "boolean") return;
  if (typeof value === "number") {
    if (!Number.isFinite(value))
      throw new ProtocolFault("invalid_value", `${path} is not finite`);
    return;
  }
  if (typeof value === "string") {
    if (isDirectImageContentPath(path)) {
      const encodedBytes = Buffer.byteLength(value, "utf8");
      if (encodedBytes > MAX_IMAGE_BASE64_BYTES) {
        throw new ProtocolFault(
          "payload_bounds",
          `${path} exceeds the encoded image bound`,
        );
      }
      const content = Buffer.from(value, "base64");
      const canonical = content.toString("base64") === value;
      const decodedBytes = content.length;
      content.fill(0);
      if (decodedBytes > MAX_DIRECT_IMAGE_BYTES || !canonical) {
        throw new ProtocolFault(
          "payload_bounds",
          `${path} is not bounded canonical image content`,
        );
      }
      return;
    }
    if (Buffer.byteLength(value, "utf8") > MAX_STRING_BYTES)
      throw new ProtocolFault("payload_bounds", `${path} is too large`);
    return;
  }
  if (typeof value !== "object")
    throw new ProtocolFault(
      "invalid_value",
      `${path} contains an unsupported value`,
    );
  if (seen.has(value))
    throw new ProtocolFault("invalid_value", `${path} contains a cycle`);
  seen.add(value);
  const entries = Array.isArray(value)
    ? value.entries()
    : Object.entries(value);
  if (
    (Array.isArray(value) ? value.length : Object.keys(value).length) >
    MAX_COLLECTION_ITEMS
  ) {
    throw new ProtocolFault(
      "payload_bounds",
      `${path} contains too many items`,
    );
  }
  for (const [key, item] of entries) {
    const field = String(key);
    if (!Array.isArray(value) && isForbiddenField(field)) {
      throw new ProtocolFault(
        "secret_field",
        `${path} contains a forbidden credential field`,
      );
    }
    inspectValue(item, `${path}.${field}`, depth + 1, seen);
  }
  seen.delete(value);
}

function isDirectImageContentPath(path) {
  return /^payload\.attachments\.\d+\.contentBase64$/u.test(path);
}

function redactText(value) {
  return String(value)
    .replace(/(bearer\s+)[A-Za-z0-9._~+/-]+/gi, "$1[REDACTED]")
    .replace(
      /-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----/gi,
      "[REDACTED PRIVATE KEY]",
    )
    .replace(/([a-z][a-z0-9+.-]*:\/\/[^\s/:]+:)[^\s/@]+@/gi, "$1[REDACTED]@")
    .replace(
      /((?:api[_-]?key|access[_-]?token|refresh[_-]?token|password|secret)\s*[=:]\s*)[^\s,;]+/gi,
      "$1[REDACTED]",
    );
}

function isForbiddenField(key) {
  return (
    FORBIDDEN_FIELD.test(key) &&
    !/(?:reference|referenceId|ref|refId|leaseId)$/i.test(key)
  );
}

function requireRecord(value, name) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new ProtocolFault("invalid_value", `${name} must be an object`);
  }
}

function rejectUnknownKeys(value, allowed, name) {
  for (const key of Object.keys(value)) {
    if (!allowed.has(key))
      throw new ProtocolFault(
        "unknown_field",
        `${name} contains an unknown field`,
      );
  }
}
