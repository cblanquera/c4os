#!/usr/bin/env node

import { Buffer } from "node:buffer";
import { timingSafeEqual } from "node:crypto";
import { createServer } from "node:http";
import process from "node:process";
import { setTimeout } from "node:timers";
import { URL } from "node:url";

const HOST = "127.0.0.1";
const SECRET_ENVIRONMENT_VARIABLE = "C4OS_TASK_00013_PROVIDER_SECRET";
const HEADER_ENVIRONMENT_VARIABLE = "C4OS_TASK_00013_PROVIDER_API_KEY_HEADER";
const MAX_SECRET_BYTES = 4 * 1024;
const MODE_PATHS = Object.freeze({
  auth: "/auth/v1/models",
  server: "/server/v1/models",
  zero: "/zero/v1/models",
  many: "/many/v1/models",
});
const QUERY_PATH = "/v1/models";
const MODES = new Set(Object.keys(MODE_PATHS));

const secret = requiredSecret();
const apiKeyHeader = configuredApiKeyHeader();

// Remove secret-bearing configuration from the inherited environment after the
// fixture owns its bounded byte buffer. The raw value is never logged.
delete process.env[SECRET_ENVIRONMENT_VARIABLE];

const server = createServer(
  { maxHeaderSize: 16 * 1024 },
  (request, response) => {
    handleRequest(request, response);
  },
);

server.on("clientError", (_error, socket) => {
  if (socket.writable) {
    socket.end("HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n");
  }
});

server.on("error", () => {
  zeroSecretAndExit(2);
});

server.listen(0, HOST, () => {
  const address = server.address();
  if (!address || typeof address === "string") {
    zeroSecretAndExit(2);
    return;
  }

  const origin = `http://${HOST}:${address.port}`;
  process.stdout.write(
    `${JSON.stringify({
      origin,
      port: address.port,
      modePaths: MODE_PATHS,
    })}\n`,
  );
});

let shutdownStarted = false;
for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => shutdown());
}

function handleRequest(request, response) {
  if (request.method !== "GET") {
    writeJson(response, 405, {
      error: {
        message: "Only GET is supported.",
        type: "invalid_request_error",
        param: null,
        code: "method_not_allowed",
      },
    });
    return;
  }

  const mode = requestMode(request.url);
  if (!mode) {
    writeJson(response, 404, {
      error: {
        message: "Unknown fixture route.",
        type: "invalid_request_error",
        param: null,
        code: "not_found",
      },
    });
    return;
  }

  if (!hasExpectedCredential(request.headers)) {
    writeAuthenticationFailure(response);
    return;
  }

  if (mode === "auth") {
    writeAuthenticationFailure(response);
    return;
  }

  if (mode === "server") {
    writeJson(response, 503, {
      error: {
        message: "The deterministic provider fixture is unavailable.",
        type: "server_error",
        param: null,
        code: "service_unavailable",
      },
    });
    return;
  }

  writeJson(response, 200, {
    object: "list",
    data: mode === "zero" ? [] : richModels(),
  });
}

function requestMode(rawUrl) {
  let url;
  try {
    url = new URL(rawUrl ?? "/", `http://${HOST}`);
  } catch {
    return null;
  }

  for (const [mode, path] of Object.entries(MODE_PATHS)) {
    if (url.pathname === path && url.search === "") return mode;
  }

  if (url.pathname !== QUERY_PATH) return null;
  const mode = url.searchParams.get("mode");
  return mode && MODES.has(mode) && [...url.searchParams.keys()].length === 1
    ? mode
    : null;
}

function hasExpectedCredential(headers) {
  const raw = apiKeyHeader
    ? singleHeader(headers[apiKeyHeader])
    : bearerCredential(singleHeader(headers.authorization));
  if (raw === null) return false;

  const candidate = Buffer.from(raw, "utf8");
  const matches =
    candidate.length === secret.length && timingSafeEqual(candidate, secret);
  candidate.fill(0);
  return matches;
}

function singleHeader(value) {
  return typeof value === "string" ? value : null;
}

function bearerCredential(value) {
  return value?.startsWith("Bearer ") ? value.slice("Bearer ".length) : null;
}

function writeAuthenticationFailure(response) {
  writeJson(response, 401, {
    error: {
      message: "Invalid authentication credentials.",
      type: "invalid_request_error",
      param: null,
      code: "invalid_api_key",
    },
  });
}

function writeJson(response, status, value) {
  const body = JSON.stringify(value);
  response.writeHead(status, {
    "cache-control": "no-store",
    connection: "close",
    "content-length": Buffer.byteLength(body),
    "content-type": "application/json; charset=utf-8",
  });
  response.end(body);
}

function richModels() {
  return [
    {
      id: "openai/gpt-5-mini",
      object: "model",
      owned_by: "openai",
      canonical_slug: "openai/gpt-5-mini-2026-06-30",
      hugging_face_id: null,
      name: "OpenAI: GPT-5 Mini",
      created: 1_783_296_000,
      description:
        "Deterministic fixture route with vision, tools, reasoning, and structured output support.",
      context_length: 400_000,
      architecture: {
        modality: "text+image->text",
        input_modalities: ["text", "image"],
        output_modalities: ["text"],
        tokenizer: "GPT",
        instruct_type: null,
      },
      pricing: {
        prompt: "0.00000025",
        completion: "0.000002",
        image: "0.000001",
        request: "0",
      },
      top_provider: {
        context_length: 400_000,
        max_completion_tokens: 128_000,
        is_moderated: true,
      },
      per_request_limits: null,
      supported_parameters: [
        "tools",
        "tool_choice",
        "reasoning",
        "response_format",
        "structured_outputs",
      ],
    },
    {
      id: "anthropic/claude-sonnet-4.5",
      object: "model",
      owned_by: "anthropic",
      canonical_slug: "anthropic/claude-sonnet-4.5-2026-07-01",
      hugging_face_id: null,
      name: "Anthropic: Claude Sonnet 4.5",
      created: 1_783_382_400,
      description:
        "Deterministic fixture route with image input, tool use, reasoning, and a 200K context window.",
      context_length: 200_000,
      architecture: {
        modality: "text+image->text",
        input_modalities: ["text", "image"],
        output_modalities: ["text"],
        tokenizer: "Claude",
        instruct_type: null,
      },
      pricing: {
        prompt: "0.000003",
        completion: "0.000015",
        image: "0.0000048",
        request: "0",
      },
      top_provider: {
        context_length: 200_000,
        max_completion_tokens: 64_000,
        is_moderated: false,
      },
      per_request_limits: null,
      supported_parameters: [
        "tools",
        "tool_choice",
        "reasoning",
        "response_format",
      ],
    },
  ];
}

function requiredSecret() {
  const value = process.env[SECRET_ENVIRONMENT_VARIABLE];
  if (!value || /[\0\r\n]/u.test(value)) process.exit(2);
  const bytes = Buffer.from(value, "utf8");
  if (bytes.length < 16 || bytes.length > MAX_SECRET_BYTES) {
    bytes.fill(0);
    process.exit(2);
  }
  return bytes;
}

function configuredApiKeyHeader() {
  const value = process.env[HEADER_ENVIRONMENT_VARIABLE];
  if (value === undefined || value === "") return null;
  if (!/^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/u.test(value)) process.exit(2);
  return value.toLowerCase();
}

function shutdown() {
  if (shutdownStarted) return;
  shutdownStarted = true;

  server.close(() => zeroSecretAndExit(0));
  server.closeIdleConnections?.();
  const deadline = setTimeout(() => {
    server.closeAllConnections?.();
    zeroSecretAndExit(0);
  }, 1_000);
  deadline.unref();
}

function zeroSecretAndExit(status) {
  secret.fill(0);
  process.exit(status);
}
