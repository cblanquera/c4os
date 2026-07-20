#!/usr/bin/env node

import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { readFileSync, renameSync, writeFileSync } from "node:fs";
import { createServer } from "node:https";
import process from "node:process";
import { setTimeout as delay } from "node:timers/promises";
import { URL } from "node:url";

const MAX_BODY_BYTES = 2 * 1024 * 1024;
const OPENCODE_GOLDEN_TOOL_NAMES = [
  "c4os_propose_action",
  "c4os_read_resource",
];
const options = parseOptions(process.argv.slice(2));
const evidence = { schemaVersion: 1, requests: [] };
const stages = new Map();

const server = createServer(
  {
    cert: readFileSync(options.cert),
    key: readFileSync(options.key),
    minVersion: "TLSv1.2",
  },
  async (request, response) => {
    try {
      await handleRequest(request, response);
    } catch {
      response.writeHead(500, { "content-type": "application/json" });
      response.end('{"error":{"message":"fixture rejected request"}}');
    }
  },
);

server.listen(0, "127.0.0.1", () => {
  const address = server.address();
  if (!address || typeof address === "string") process.exit(2);
  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: 1,
      baseUrl: `https://127.0.0.1:${address.port}/v1`,
      processId: process.pid,
    })}\n`,
  );
});

for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  process.on(signal, () => server.close(() => process.exit(0)));
}

async function handleRequest(request, response) {
  const path = new URL(request.url ?? "/", "https://127.0.0.1").pathname;
  const credential = bearerCredential(request.headers.authorization);
  const credentialSha256 = sha256(credential);
  credential.fill(0);

  if (request.method === "GET" && path === "/v1/models") {
    recordEvidence(request, {
      path,
      modelId: null,
      credentialSha256,
      imageParts: [],
      providerToolNames: [],
      stage: "models",
      observedToolResult: null,
    });
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        object: "list",
        data: [
          { id: "gpt-4o-mini", object: "model", owned_by: "c4os-fixture" },
        ],
      }),
    );
    return;
  }

  if (request.method !== "POST") {
    response.writeHead(404, { "content-type": "application/json" });
    response.end('{"error":{"message":"not found"}}');
    return;
  }

  const body = await readBoundedBody(request);
  let payload;
  try {
    payload = JSON.parse(body.toString("utf8"));
  } finally {
    body.fill(0);
  }
  const modelId = typeof payload.model === "string" ? payload.model : null;
  const serialized = JSON.stringify(payload);
  const flow = goldenFlow(serialized);
  const imageParts = collectImageParts(payload);
  const providerToolNames = collectProviderToolNames(payload);
  const brokerResults = collectBrokerResults(payload);
  const observedToolResult = serialized.includes("user-denied")
    ? "user-denied"
    : serialized.includes("installed-facility-completed")
      ? "installed-facility-completed"
      : null;

  if (path.endsWith("/chat/completions")) {
    if (
      !sameStrings(providerToolNames, OPENCODE_GOLDEN_TOOL_NAMES) ||
      !["opencode-allow", "opencode-deny"].includes(flow)
    ) {
      recordEvidence(request, {
        path,
        modelId,
        credentialSha256,
        imageParts,
        providerToolNames,
        brokerResults,
        stage: "auxiliary",
        flow,
        observedToolResult,
      });
      writeAuxiliaryChatCompletion(response, modelId ?? "gpt-4o-mini");
      return;
    }
    const stage = nextRecordedStage(
      request,
      `chat:${credentialSha256}:${flow}`,
      observedToolResult,
      {
        path,
        modelId,
        credentialSha256,
        imageParts,
        providerToolNames,
        brokerResults,
        flow,
      },
      flow,
    );
    recordEvidence(request, {
      path,
      modelId,
      credentialSha256,
      imageParts,
      providerToolNames,
      brokerResults,
      stage,
      flow,
      observedToolResult,
    });
    await writeChatCompletion(response, modelId ?? "gpt-4o-mini", stage);
    return;
  }

  if (path === "/v1/responses") {
    const stage = nextRecordedStage(
      request,
      `responses:${credentialSha256}`,
      observedToolResult,
      {
        path,
        modelId,
        credentialSha256,
        imageParts,
        providerToolNames,
        brokerResults,
      },
      "pi-sequential",
    );
    recordEvidence(request, {
      path,
      modelId,
      credentialSha256,
      imageParts,
      providerToolNames,
      brokerResults,
      stage,
      observedToolResult,
    });
    writeResponses(response, stage);
    return;
  }

  response.writeHead(404, { "content-type": "application/json" });
  response.end('{"error":{"message":"not found"}}');
}

function nextRecordedStage(request, key, observedToolResult, fields, flow) {
  try {
    return nextStage(key, observedToolResult, flow);
  } catch (error) {
    recordEvidence(request, {
      ...fields,
      stage: "rejected",
      observedToolResult,
      rejectionCode: fixtureRejectionCode(error),
    });
    throw error;
  }
}

function goldenFlow(serialized) {
  for (const flow of ["opencode-allow", "opencode-deny"]) {
    if (serialized.includes(`C4OS-NATIVE-FLOW: ${flow}`)) return flow;
  }
  return null;
}

function fixtureRejectionCode(error) {
  switch (error instanceof Error ? error.message : "") {
    case "unexpected tool result":
      return "unexpected_tool_result";
    case "missing allowed tool result":
      return "missing_allowed_tool_result";
    case "missing denied tool result":
      return "missing_denied_tool_result";
    case "unexpected extra provider request":
      return "unexpected_extra_provider_request";
    default:
      return "fixture_rejected";
  }
}

function sameStrings(actual, expected) {
  return (
    actual.length === expected.length &&
    expected.every((value) => actual.includes(value))
  );
}

function nextStage(key, observedToolResult, flow) {
  const prior = stages.get(key) ?? 0;
  if (flow === "opencode-allow") {
    if (prior === 0 && observedToolResult !== null)
      throw new Error("unexpected tool result");
    if (prior === 1 && observedToolResult !== "installed-facility-completed")
      throw new Error("missing allowed tool result");
    if (prior >= 2) throw new Error("unexpected extra provider request");
    const next = prior + 1;
    stages.set(key, next);
    return next === 1 ? "allow-tool" : "complete";
  }
  if (flow === "opencode-deny") {
    if (prior === 0 && observedToolResult !== null)
      throw new Error("unexpected tool result");
    if (prior === 1 && observedToolResult !== "user-denied")
      throw new Error("missing denied tool result");
    if (prior >= 2) throw new Error("unexpected extra provider request");
    const next = prior + 1;
    stages.set(key, next);
    return next === 1 ? "deny-tool" : "complete";
  }
  if (prior === 0 && observedToolResult !== null)
    throw new Error("unexpected tool result");
  if (prior === 1 && observedToolResult !== "installed-facility-completed") {
    throw new Error("missing allowed tool result");
  }
  if (prior === 2 && observedToolResult !== "user-denied") {
    throw new Error("missing denied tool result");
  }
  if (prior >= 3) throw new Error("unexpected extra provider request");
  const next = prior + 1;
  stages.set(key, next);
  return next === 1 ? "allow-tool" : next === 2 ? "deny-tool" : "complete";
}

async function writeChatCompletion(response, model, stage) {
  response.writeHead(200, {
    "cache-control": "no-store",
    connection: "close",
    "content-type": "text/event-stream; charset=utf-8",
  });
  const id = `chatcmpl-c4os-${stage}`;
  if (stage === "complete") {
    writeData(response, {
      id,
      object: "chat.completion.chunk",
      created: 1,
      model,
      choices: [
        {
          index: 0,
          delta: { role: "assistant", content: "golden-complete-opencode" },
          finish_reason: null,
        },
      ],
    });
    writeData(response, {
      id,
      object: "chat.completion.chunk",
      created: 1,
      model,
      choices: [{ index: 0, delta: {}, finish_reason: "stop" }],
    });
  } else {
    const allow = stage === "allow-tool";
    const callId = allow ? "call_allow_1" : "call_deny_1";
    const target = allow ? "allow-window" : "deny-window";
    writeData(response, {
      id,
      object: "chat.completion.chunk",
      created: 1,
      model,
      choices: [
        {
          index: 0,
          delta: {
            role: "assistant",
            content: "Preparing the C4OS approval request.",
          },
          finish_reason: null,
        },
      ],
    });
    await delay(150);
    writeData(response, {
      id,
      object: "chat.completion.chunk",
      created: 1,
      model,
      choices: [
        {
          index: 0,
          delta: {
            tool_calls: [
              {
                index: 0,
                id: callId,
                type: "function",
                function: {
                  name: "c4os_propose_action",
                  arguments: JSON.stringify({
                    operation: "window.focus",
                    target,
                    arguments: {},
                  }),
                },
              },
            ],
          },
          finish_reason: null,
        },
      ],
    });
    writeData(response, {
      id,
      object: "chat.completion.chunk",
      created: 1,
      model,
      choices: [{ index: 0, delta: {}, finish_reason: "tool_calls" }],
    });
  }
  response.end("data: [DONE]\n\n");
}

function writeAuxiliaryChatCompletion(response, model) {
  response.writeHead(200, {
    "cache-control": "no-store",
    connection: "close",
    "content-type": "text/event-stream; charset=utf-8",
  });
  const id = "chatcmpl-c4os-auxiliary";
  writeData(response, {
    id,
    object: "chat.completion.chunk",
    created: 1,
    model,
    choices: [
      {
        index: 0,
        delta: { role: "assistant", content: "C4OS native golden path" },
        finish_reason: null,
      },
    ],
  });
  writeData(response, {
    id,
    object: "chat.completion.chunk",
    created: 1,
    model,
    choices: [{ index: 0, delta: {}, finish_reason: "stop" }],
  });
  response.end("data: [DONE]\n\n");
}

function writeResponses(response, stage) {
  response.writeHead(200, {
    "cache-control": "no-store",
    connection: "close",
    "content-type": "text/event-stream; charset=utf-8",
  });
  const responseId = `resp_c4os_${stage}`;
  writeEvent(response, {
    type: "response.created",
    response: { id: responseId, status: "in_progress", output: [] },
  });
  if (stage === "complete") {
    const item = {
      id: "msg_pi_final",
      type: "message",
      role: "assistant",
      status: "completed",
      content: [
        {
          type: "output_text",
          text: "golden-complete-pi",
          annotations: [],
        },
      ],
    };
    writeEvent(response, {
      type: "response.output_item.added",
      output_index: 0,
      item: { ...item, status: "in_progress", content: [] },
    });
    writeEvent(response, {
      type: "response.output_text.delta",
      output_index: 0,
      content_index: 0,
      delta: "golden-complete-pi",
    });
    writeEvent(response, {
      type: "response.output_item.done",
      output_index: 0,
      item,
    });
    writeEvent(response, completedResponse(responseId, [item]));
  } else {
    const allow = stage === "allow-tool";
    const item = {
      id: allow ? "fc_allow_1" : "fc_deny_1",
      type: "function_call",
      call_id: allow ? "call_allow_1" : "call_deny_1",
      name: "c4os_propose_action",
      arguments: JSON.stringify({
        operation: "window.focus",
        target: allow ? "allow-window" : "deny-window",
        arguments: {},
      }),
    };
    writeEvent(response, {
      type: "response.output_item.added",
      output_index: 0,
      item: { ...item, arguments: "" },
    });
    writeEvent(response, {
      type: "response.function_call_arguments.done",
      output_index: 0,
      item_id: item.id,
      arguments: item.arguments,
    });
    writeEvent(response, {
      type: "response.output_item.done",
      output_index: 0,
      item,
    });
    writeEvent(response, completedResponse(responseId, [item]));
  }
  response.end("data: [DONE]\n\n");
}

function completedResponse(id, output) {
  return {
    type: "response.completed",
    response: {
      id,
      status: "completed",
      output,
      usage: { input_tokens: 8, output_tokens: 4, total_tokens: 12 },
    },
  };
}

function writeData(response, value) {
  response.write(`data: ${JSON.stringify(value)}\n\n`);
}

function writeEvent(response, value) {
  response.write(`event: ${value.type}\ndata: ${JSON.stringify(value)}\n\n`);
}

function recordEvidence(request, fields) {
  evidence.requests.push({
    requestNumber: evidence.requests.length + 1,
    method: request.method,
    tlsEncrypted: request.socket.encrypted === true,
    tlsProtocol: request.socket.getProtocol?.() ?? null,
    ...fields,
  });
  const temporary = `${options.evidence}.tmp`;
  writeFileSync(temporary, `${JSON.stringify(evidence, null, 2)}\n`, {
    mode: 0o600,
  });
  renameSync(temporary, options.evidence);
}

function collectImageParts(value) {
  const parts = [];
  const visit = (candidate) => {
    if (typeof candidate === "string" && candidate.startsWith("data:image/")) {
      const match = /^data:(image\/[a-z0-9.+-]+);base64,/i.exec(candidate);
      if (match) {
        const decoded = Buffer.from(candidate.slice(match[0].length), "base64");
        parts.push({ mimeType: match[1], sha256: sha256(decoded) });
        decoded.fill(0);
      }
      return;
    }
    if (Array.isArray(candidate)) {
      for (const item of candidate) visit(item);
      return;
    }
    if (candidate && typeof candidate === "object") {
      for (const item of Object.values(candidate)) visit(item);
    }
  };
  visit(value);
  return parts;
}

function collectProviderToolNames(payload) {
  if (!Array.isArray(payload?.tools)) return [];
  const names = [];
  for (const tool of payload.tools) {
    if (!tool || typeof tool !== "object" || tool.type !== "function") continue;
    if (typeof tool.name === "string") {
      names.push(tool.name);
    } else if (typeof tool.function?.name === "string") {
      names.push(tool.function.name);
    }
  }
  return names;
}

function collectBrokerResults(payload) {
  const results = [];
  const seen = new Set();
  const visit = (candidate, depth = 0) => {
    if (depth > 16) return;
    if (typeof candidate === "string") {
      if (candidate.length > 16 * 1024) return;
      try {
        const decoded = JSON.parse(candidate);
        if (
          decoded &&
          typeof decoded === "object" &&
          !Array.isArray(decoded) &&
          ["result", "denied", "cancelled"].includes(decoded.status)
        ) {
          const reasonCode =
            typeof decoded.reasonCode === "string" ? decoded.reasonCode : null;
          const key = `${decoded.status}:${reasonCode ?? ""}`;
          if (!seen.has(key)) {
            seen.add(key);
            results.push({ status: decoded.status, reasonCode });
          }
        }
      } catch {
        // Only exact JSON broker outputs are evidence; arbitrary model text is
        // never copied into the native evidence file.
      }
      return;
    }
    if (Array.isArray(candidate)) {
      for (const item of candidate) visit(item, depth + 1);
      return;
    }
    if (candidate && typeof candidate === "object") {
      for (const item of Object.values(candidate)) visit(item, depth + 1);
    }
  };
  visit(payload);
  return results;
}

async function readBoundedBody(request) {
  const chunks = [];
  let total = 0;
  for await (const chunk of request) {
    const owned = Buffer.from(chunk);
    total += owned.length;
    if (total > MAX_BODY_BYTES) {
      for (const buffered of chunks) buffered.fill(0);
      owned.fill(0);
      throw new Error("request body exceeded fixture bound");
    }
    chunks.push(owned);
  }
  const body = Buffer.concat(chunks, total);
  for (const chunk of chunks) chunk.fill(0);
  return body;
}

function bearerCredential(value) {
  if (typeof value !== "string" || !value.startsWith("Bearer ")) {
    throw new Error("missing bearer credential");
  }
  const credential = Buffer.from(value.slice("Bearer ".length), "utf8");
  if (credential.length === 0 || credential.length > 64 * 1024) {
    credential.fill(0);
    throw new Error("invalid bearer credential");
  }
  return credential;
}

function sha256(bytes) {
  return `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
}

function parseOptions(arguments_) {
  const parsed = {};
  for (const argument of arguments_) {
    const separator = argument.indexOf("=");
    if (!argument.startsWith("--") || separator < 3) process.exit(2);
    const key = argument.slice(2, separator);
    const value = argument.slice(separator + 1);
    if (!value || parsed[key] !== undefined) process.exit(2);
    parsed[key] = value;
  }
  if (!parsed.cert || !parsed.key || !parsed.evidence) process.exit(2);
  return parsed;
}
