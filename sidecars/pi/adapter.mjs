import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { URL } from "node:url";
import {
  PROTOCOL_NAME,
  PROTOCOL_SCHEMA_VERSION,
  ProtocolFault,
  redactValue,
  requireId,
  safeDiagnosticMessage,
  validateRequest,
} from "./protocol.mjs";

export const SIDECAR_VERSION = "0.1.0";
export const PI_NATIVE_VERSION = "0.80.10";
export const C4OS_TOOL_NAMES = Object.freeze([
  "c4os_read_resource",
  "c4os_propose_action",
]);

const OPERATIONS = new Set([
  "health",
  "version",
  "model.preflight",
  "session.create",
  "session.resume",
  "session.close",
  "dispatch",
  "tool.resolve",
  "cancel",
  "shutdown",
]);
const NORMAL_CATEGORIES = new Map([
  ["agent_start", "lifecycle.started"],
  ["agent_end", "lifecycle.completed"],
  ["message_start", "message.started"],
  ["message_update", "content.delta"],
  ["message_end", "message.completed"],
  ["turn_start", "turn.started"],
  ["turn_end", "turn.completed"],
  ["tool_execution_start", "tool.started"],
  ["tool_execution_update", "tool.progress"],
  ["tool_execution_end", "tool.completed"],
]);
const ATTACHMENT_KEYS = new Set([
  "attachmentId",
  "stableReference",
  "displayName",
  "mediaType",
  "byteLength",
  "contentSha256",
  "snapshotVersion",
  "contentBase64",
]);
const MAX_DIRECT_ATTACHMENTS = 32;
const MAX_DIRECT_ATTACHMENT_BYTES = 128 * 1024;
const SHA256 = /^sha256:[a-f0-9]{64}$/;
const PI_IMAGE_TYPES = new Set([
  "image/jpeg",
  "image/png",
  "image/gif",
  "image/webp",
]);

export class C4osPiSidecar {
  #driver;
  #emitLine;
  #generation;
  #status = "ready";
  #sequence = 0;
  #sessions = new Map();
  #pendingTools = new Map();
  #brokeredResults = new Map();
  #staleEventsRejected = 0;

  constructor({ driver, emitLine, processGeneration }) {
    if (
      !driver ||
      typeof driver.preflightModel !== "function" ||
      typeof driver.createSession !== "function"
    ) {
      throw new TypeError("A Pi SDK driver is required");
    }
    if (typeof emitLine !== "function")
      throw new TypeError("An event sink is required");
    if (!Number.isSafeInteger(processGeneration) || processGeneration < 1) {
      throw new TypeError("processGeneration must be a positive safe integer");
    }
    this.#driver = driver;
    this.#emitLine = emitLine;
    this.#generation = processGeneration;
  }

  async handle(rawRequest) {
    let request;
    try {
      request = validateRequest(rawRequest);
      this.#assertCurrentGeneration(request);
      if (!OPERATIONS.has(request.operation)) {
        throw new ProtocolFault(
          "unsupported_operation",
          "The requested Pi operation is unsupported",
        );
      }
      const payload = await this.#route(request);
      return this.#response(request, "ok", payload);
    } catch (error) {
      const fault =
        error instanceof ProtocolFault
          ? error
          : new ProtocolFault("adapter_failure", safeDiagnosticMessage(error));
      return this.#response(request ?? rawRequest, "error", {
        code: fault.code,
        message: safeDiagnosticMessage(fault),
      });
    }
  }

  ingestNativeEvent(sessionId, runIdentity, nativeEvent) {
    const session = this.#sessions.get(sessionId);
    if (
      !session ||
      runIdentity.processGeneration !== this.#generation ||
      session.state !== "running" ||
      !session.activeRun ||
      session.activeRun.runId !== runIdentity.runId ||
      session.activeRun.correlationId !== runIdentity.correlationId
    ) {
      this.#staleEventsRejected += 1;
      return false;
    }

    const { nativeType, category, payload } = normalizeNativeEvent(nativeEvent);
    this.#event(session.activeRun, {
      category,
      nativeType,
      payload,
    });
    return true;
  }

  async interceptTool(sessionId, runIdentity, context) {
    const session = this.#sessions.get(sessionId);
    if (
      !session ||
      session.state !== "running" ||
      !sameRun(session.activeRun, runIdentity)
    ) {
      return { block: true, reason: "C4OS rejected a stale Pi tool proposal" };
    }
    const toolName = context?.toolCall?.name;
    const toolCallId = context?.toolCall?.id;
    if (!C4OS_TOOL_NAMES.includes(toolName) || typeof toolCallId !== "string") {
      return { block: true, reason: "Pi may invoke only C4OS-brokered tools" };
    }
    if (!session.eligibleTools.has(toolName)) {
      return {
        block: true,
        reason: "C4OS did not authorize this brokered tool for the Pi session",
      };
    }
    requireId(toolCallId, "toolCallId");
    const key = toolKey(sessionId, runIdentity.runId, toolCallId);
    if (this.#pendingTools.has(key) || this.#brokeredResults.has(key)) {
      return { block: true, reason: "Duplicate Pi tool proposal rejected" };
    }

    let settle;
    const outcome = new Promise((resolve) => {
      settle = resolve;
    });
    this.#pendingTools.set(key, {
      ...runIdentity,
      sessionId,
      toolName,
      toolCallId,
      settle,
    });
    this.#event(runIdentity, {
      category: "tool.action_intent",
      nativeType: "beforeToolCall",
      toolCallId,
      payload: {
        tool: toolName,
        arguments: redactValue(context.args ?? {}),
        authority: "c4os-action-gateway-required",
      },
    });
    return outcome;
  }

  consumeBrokeredResult(sessionId, runId, toolCallId) {
    const key = toolKey(sessionId, runId, toolCallId);
    const result = this.#brokeredResults.get(key);
    if (!result)
      throw new ProtocolFault(
        "missing_broker_result",
        "C4OS did not provide a brokered tool result",
      );
    this.#brokeredResults.delete(key);
    return globalThis.structuredClone(result);
  }

  #assertCurrentGeneration(request) {
    if (request.processGeneration !== this.#generation) {
      throw new ProtocolFault(
        "stale_generation",
        "Request belongs to a replaced Pi process generation",
      );
    }
    if (this.#status === "stopped" && request.operation !== "health") {
      throw new ProtocolFault("sidecar_stopped", "Pi sidecar has shut down");
    }
  }

  async #route(request) {
    switch (request.operation) {
      case "health":
        return this.#health();
      case "version":
        return this.#version();
      case "model.preflight":
        return this.#preflightModel(request);
      case "session.create":
        return this.#createSession(request);
      case "session.resume":
        return this.#resumeSession(request);
      case "session.close":
        return this.#closeSession(request);
      case "dispatch":
        return this.#dispatch(request);
      case "tool.resolve":
        return this.#resolveTool(request);
      case "cancel":
        return this.#cancel(request);
      case "shutdown":
        return this.#shutdown();
      default:
        throw new ProtocolFault(
          "unsupported_operation",
          "Unsupported Pi operation",
        );
    }
  }

  #health() {
    return {
      status: this.#status,
      runtime: "pi",
      transport: "c4os-node-sdk-sidecar",
      protocol: PROTOCOL_NAME,
      processGeneration: this.#generation,
      sessions: this.#sessions.size,
      staleEventsRejected: this.#staleEventsRejected,
      capabilities: {
        streaming: capability("supported"),
        cancellation: capability("supported"),
        tools: capability(
          "supported",
          "Only C4OS-brokered custom tools are exposed",
        ),
        nativePersistence: capability(
          "unsupported",
          "C4OS owns session persistence",
        ),
        nativeExtensions: capability(
          "unsupported",
          "Pi extension discovery is disabled",
        ),
        nativeTools: capability("unsupported", "Pi built-in tools are omitted"),
        crashResume: capability(
          "degraded",
          "C4OS must replay an authoritative transcript after process replacement",
        ),
        providerAuthentication: this.#driver.credentialChannelAvailable
          ? capability("supported", "One-shot credential lease channel")
          : capability("degraded", "Credential lease channel is unavailable"),
        rpcTransport: capability(
          "unsupported",
          "SDK sidecar is the selected initial transport",
        ),
      },
    };
  }

  #version() {
    return {
      adapter: "PIAdapter",
      adapterVersion: SIDECAR_VERSION,
      nativePackage: "@earendil-works/pi-coding-agent",
      nativeVersion: PI_NATIVE_VERSION,
      protocol: PROTOCOL_NAME,
    };
  }

  async #preflightModel(request) {
    if (
      ["workspaceId", "sessionId", "turnId", "runId"].some(
        (field) => request[field] !== undefined,
      )
    ) {
      throw new ProtocolFault(
        "invalid_model_preflight",
        "Pi model preflight must not claim session or run scope",
      );
    }
    const pair = validateNativeModelPair(request.payload);
    const available = await this.#driver.preflightModel(pair);
    if (typeof available !== "boolean") {
      throw new ProtocolFault(
        "invalid_model_preflight",
        "Pi model preflight returned an invalid availability state",
      );
    }
    return {
      available,
      provider: pair.provider,
      modelId: pair.modelId,
      nativeVersion: PI_NATIVE_VERSION,
    };
  }

  async #createSession(request) {
    this.#requireScope(request, ["workspaceId", "sessionId"]);
    if (this.#sessions.has(request.sessionId)) {
      throw new ProtocolFault(
        "session_exists",
        "C4OS session is already bound in this process",
      );
    }
    const route = validateModelRoute(request.payload.modelRoute);
    const eligibleTools = validateEligibleTools(request.payload.eligibleTools);
    const sessionId = request.sessionId;
    const driverSession = await this.#driver.createSession({
      c4osSessionId: sessionId,
      workspaceId: request.workspaceId,
      modelRoute: route,
      tools: eligibleTools,
      onNativeEvent: (identity, event) =>
        this.ingestNativeEvent(sessionId, identity, event),
      beforeToolCall: (identity, context) =>
        this.interceptTool(sessionId, identity, context),
      consumeBrokeredResult: (runId, toolCallId) =>
        this.consumeBrokeredResult(sessionId, runId, toolCallId),
    });
    this.#sessions.set(sessionId, {
      workspaceId: request.workspaceId,
      state: "idle",
      driverSession,
      nativeSessionId: driverSession.nativeSessionId ?? `pi:${sessionId}`,
      eligibleTools: new Set(eligibleTools),
      activeRun: undefined,
    });
    return {
      sessionId,
      nativeSessionId: this.#sessions.get(sessionId).nativeSessionId,
      persistence: "c4os-authoritative",
    };
  }

  #resumeSession(request) {
    this.#requireScope(request, ["workspaceId", "sessionId"]);
    const session = this.#sessions.get(request.sessionId);
    if (!session) {
      return {
        status: "degraded",
        reason:
          "Native Pi sessions are not persisted; replay C4OS authoritative history into a new process session",
      };
    }
    if (session.workspaceId !== request.workspaceId) {
      throw new ProtocolFault(
        "scope_mismatch",
        "Session is bound to another Workspace",
      );
    }
    return {
      status: "ready",
      sessionId: request.sessionId,
      nativeSessionId: session.nativeSessionId,
    };
  }

  async #closeSession(request) {
    this.#requireScope(request, ["sessionId"]);
    const session = this.#sessions.get(request.sessionId);
    if (!session) return { closed: false };
    if (session.state === "running")
      throw new ProtocolFault(
        "session_busy",
        "Cancel the active run before closing its Pi session",
      );
    await session.driverSession.dispose?.();
    this.#sessions.delete(request.sessionId);
    return { closed: true };
  }

  #dispatch(request) {
    this.#requireScope(request, [
      "workspaceId",
      "sessionId",
      "turnId",
      "runId",
    ]);
    const session = this.#sessionFor(request);
    if (session.state === "running")
      throw new ProtocolFault(
        "session_busy",
        "Pi session already has an active run",
      );
    const input = request.payload.input;
    if (
      Object.hasOwn(request.payload, "credentialLeaseId") ||
      Object.hasOwn(request.payload, "credentialReference")
    ) {
      throw new ProtocolFault(
        "credential_handle_forbidden",
        "Dispatch may not serialize a credential handle",
      );
    }
    const runtimeId = request.payload.runtimeId;
    const providerId = request.payload.providerId;
    if ((runtimeId === undefined) !== (providerId === undefined)) {
      throw new ProtocolFault(
        "invalid_credential_operation",
        "Credential operation identity must be complete",
      );
    }
    if (runtimeId !== undefined) {
      requireId(runtimeId, "runtimeId");
      requireId(providerId, "providerId");
    }
    const attachments = validateDirectAttachments(request.payload.attachments);
    if (
      typeof input !== "string" ||
      Buffer.byteLength(input, "utf8") > 64 * 1024 ||
      (input.length === 0 && attachments.length === 0)
    ) {
      throw new ProtocolFault(
        "invalid_input",
        "Dispatch requires bounded text or direct attachment metadata",
      );
    }
    const identity = Object.freeze({
      workspaceId: request.workspaceId,
      sessionId: request.sessionId,
      turnId: request.turnId,
      runId: request.runId,
      correlationId: request.correlationId,
      processGeneration: this.#generation,
      ...(runtimeId === undefined ? {} : { runtimeId, providerId }),
    });
    session.state = "running";
    session.activeRun = identity;
    void this.#runDispatch(session, identity, input, attachments);
    return { accepted: true, runId: request.runId };
  }

  async #runDispatch(session, identity, input, attachments) {
    try {
      await session.driverSession.dispatch({
        input,
        attachments,
        runIdentity: identity,
      });
      if (session.state === "running" && sameRun(session.activeRun, identity)) {
        this.#event(identity, {
          category: "lifecycle.settled",
          nativeType: "c4os.dispatch.settled",
          payload: {},
        });
        session.state = "idle";
        session.activeRun = undefined;
      }
    } catch (error) {
      if (session.state === "running" && sameRun(session.activeRun, identity)) {
        this.#event(identity, {
          category: "lifecycle.error",
          nativeType: "c4os.dispatch.error",
          payload: {
            code: "pi_dispatch_failed",
            message: safeDiagnosticMessage(error),
          },
        });
        session.state = "idle";
        session.activeRun = undefined;
      }
    }
  }

  #resolveTool(request) {
    this.#requireScope(request, ["sessionId", "runId"]);
    const toolCallId = request.payload.toolCallId;
    requireId(toolCallId, "toolCallId");
    const key = toolKey(request.sessionId, request.runId, toolCallId);
    const pending = this.#pendingTools.get(key);
    if (
      !pending ||
      pending.correlationId !== request.correlationId ||
      pending.processGeneration !== this.#generation
    ) {
      throw new ProtocolFault(
        "stale_tool_resolution",
        "Tool decision does not match a pending action intent",
      );
    }
    this.#pendingTools.delete(key);
    if (request.payload.decision === "denied") {
      pending.settle({
        block: true,
        reason: boundedReason(request.payload.reason, "C4OS denied the action"),
      });
      return { resolved: true, executedBySidecar: false };
    }
    if (request.payload.decision !== "completed") {
      pending.settle({ block: true, reason: "Invalid C4OS tool decision" });
      throw new ProtocolFault(
        "invalid_tool_decision",
        "Tool decision must be denied or completed",
      );
    }
    if (
      !request.payload.result ||
      typeof request.payload.result !== "object" ||
      Array.isArray(request.payload.result)
    ) {
      pending.settle({
        block: true,
        reason: "C4OS did not provide a normalized result",
      });
      throw new ProtocolFault(
        "invalid_tool_result",
        "Completed tool decisions require a normalized result",
      );
    }
    this.#brokeredResults.set(key, redactValue(request.payload.result));
    pending.settle(undefined);
    return { resolved: true, executedBySidecar: false };
  }

  async #cancel(request) {
    this.#requireScope(request, ["sessionId", "runId"]);
    const session = this.#sessions.get(request.sessionId);
    if (
      !session ||
      session.state !== "running" ||
      session.activeRun?.runId !== request.runId
    ) {
      return { cancelled: false, alreadyTerminal: true };
    }
    const identity = session.activeRun;
    session.state = "cancelled";
    session.activeRun = undefined;
    for (const [key, pending] of this.#pendingTools) {
      if (
        pending.sessionId === request.sessionId &&
        pending.runId === request.runId
      ) {
        pending.settle({ block: true, reason: "C4OS cancelled the run" });
        this.#pendingTools.delete(key);
        this.#brokeredResults.delete(key);
      }
    }
    await session.driverSession.abort?.();
    this.#event(identity, {
      category: "lifecycle.cancelled",
      nativeType: "c4os.cancel",
      payload: {},
    });
    session.state = "idle";
    return { cancelled: true, alreadyTerminal: false };
  }

  async #shutdown() {
    for (const [sessionId, session] of this.#sessions) {
      if (session.state === "running") await session.driverSession.abort?.();
      await session.driverSession.dispose?.();
      this.#sessions.delete(sessionId);
    }
    for (const pending of this.#pendingTools.values()) {
      pending.settle({ block: true, reason: "C4OS shut down the Pi runtime" });
    }
    this.#pendingTools.clear();
    this.#brokeredResults.clear();
    await this.#driver.shutdown?.();
    this.#status = "stopped";
    return { stopped: true };
  }

  #sessionFor(request) {
    const session = this.#sessions.get(request.sessionId);
    if (!session)
      throw new ProtocolFault("unknown_session", "Unknown C4OS Pi session");
    if (session.workspaceId !== request.workspaceId)
      throw new ProtocolFault("scope_mismatch", "Session Workspace mismatch");
    return session;
  }

  #requireScope(request, fields) {
    for (const field of fields) requireId(request[field], field);
  }

  #response(request, status, payload) {
    return {
      schemaVersion: PROTOCOL_SCHEMA_VERSION,
      kind: "response",
      requestId:
        typeof request?.requestId === "string"
          ? request.requestId
          : "invalid-request",
      correlationId:
        typeof request?.correlationId === "string"
          ? request.correlationId
          : "invalid-correlation",
      processGeneration: this.#generation,
      status,
      payload: redactValue(payload),
    };
  }

  #event(identity, event) {
    const envelope = {
      schemaVersion: PROTOCOL_SCHEMA_VERSION,
      kind: "event",
      eventId: `pi-event-${this.#generation}-${this.#sequence + 1}`,
      correlationId: identity.correlationId,
      processGeneration: this.#generation,
      sequence: ++this.#sequence,
      runtime: "pi",
      workspaceId: identity.workspaceId,
      sessionId: identity.sessionId,
      turnId: identity.turnId,
      runId: identity.runId,
      category: event.category,
      nativeType: event.nativeType,
      ...(event.toolCallId ? { toolCallId: event.toolCallId } : {}),
      payload: redactValue(event.payload),
    };
    this.#emitLine(envelope);
  }
}

function normalizeNativeEvent(nativeEvent) {
  const nativeType =
    typeof nativeEvent?.type === "string" ? nativeEvent.type : "unknown";
  if (nativeType === "message_update") {
    const update = nativeEvent?.assistantMessageEvent;
    if (update?.type === "thinking_delta" && typeof update.delta === "string") {
      return {
        nativeType,
        category: "reasoning.delta",
        payload: { delta: update.delta },
      };
    }
    const delta =
      update?.type === "text_delta" && typeof update.delta === "string"
        ? update.delta
        : typeof nativeEvent?.delta === "string"
          ? nativeEvent.delta
          : undefined;
    if (delta !== undefined) {
      return { nativeType, category: "content.delta", payload: { delta } };
    }
    return {
      nativeType,
      category: "message.updated",
      payload: redactValue(nativeEvent ?? {}),
    };
  }
  return {
    nativeType,
    category: NORMAL_CATEGORIES.get(nativeType) ?? "diagnostic.unknown",
    payload: redactValue(nativeEvent ?? {}),
  };
}

/** Validates exact Rust-owned direct attachment metadata without resolving it. */
function validateDirectAttachments(value) {
  if (value === undefined) return [];
  if (
    !Array.isArray(value) ||
    value.length === 0 ||
    value.length > MAX_DIRECT_ATTACHMENTS
  ) {
    throw new ProtocolFault(
      "invalid_attachments",
      "Direct attachments must be a non-empty bounded array",
    );
  }
  const identities = new Set();
  let totalBytes = 0;
  return value.map((attachment) => {
    if (
      !attachment ||
      typeof attachment !== "object" ||
      Array.isArray(attachment) ||
      Object.keys(attachment).some((key) => !ATTACHMENT_KEYS.has(key)) ||
      Object.keys(attachment).length !== ATTACHMENT_KEYS.size
    ) {
      throw new ProtocolFault(
        "invalid_attachments",
        "Direct attachment metadata has an invalid shape",
      );
    }
    requireId(attachment.attachmentId, "attachmentId");
    requireId(attachment.stableReference, "stableReference");
    if (identities.has(attachment.attachmentId)) {
      throw new ProtocolFault(
        "invalid_attachments",
        "Direct attachment identities must be unique",
      );
    }
    identities.add(attachment.attachmentId);
    for (const [field, maximum] of [
      ["displayName", 1_024],
      ["mediaType", 256],
    ]) {
      const item = attachment[field];
      if (
        typeof item !== "string" ||
        item.length === 0 ||
        item.length > maximum ||
        [...item].some((character) => character.charCodeAt(0) < 32)
      ) {
        throw new ProtocolFault("invalid_attachments", `${field} is invalid`);
      }
    }
    if (
      !Number.isSafeInteger(attachment.byteLength) ||
      attachment.byteLength < 1 ||
      attachment.byteLength > MAX_DIRECT_ATTACHMENT_BYTES ||
      !SHA256.test(attachment.contentSha256) ||
      !Number.isSafeInteger(attachment.snapshotVersion) ||
      attachment.snapshotVersion < 1 ||
      !PI_IMAGE_TYPES.has(attachment.mediaType) ||
      typeof attachment.contentBase64 !== "string"
    ) {
      throw new ProtocolFault(
        "invalid_attachments",
        "Direct attachment provenance is invalid",
      );
    }
    const content = Buffer.from(attachment.contentBase64, "base64");
    totalBytes += content.length;
    if (
      content.length !== attachment.byteLength ||
      totalBytes > MAX_DIRECT_ATTACHMENT_BYTES ||
      content.toString("base64") !== attachment.contentBase64 ||
      `sha256:${createHash("sha256").update(content).digest("hex")}` !==
        attachment.contentSha256
    ) {
      content.fill(0);
      throw new ProtocolFault(
        "invalid_attachments",
        "Direct attachment content failed verification",
      );
    }
    content.fill(0);
    return {
      attachmentId: attachment.attachmentId,
      stableReference: attachment.stableReference,
      displayName: attachment.displayName,
      mediaType: attachment.mediaType,
      byteLength: attachment.byteLength,
      contentSha256: attachment.contentSha256,
      snapshotVersion: attachment.snapshotVersion,
      contentBase64: attachment.contentBase64,
    };
  });
}

function validateModelRoute(route) {
  if (!route || typeof route !== "object" || Array.isArray(route)) {
    throw new ProtocolFault(
      "invalid_model_route",
      "A C4OS model route is required",
    );
  }
  const keys = Object.keys(route);
  if (keys.some((key) => !["provider", "modelId", "baseUrl"].includes(key))) {
    throw new ProtocolFault(
      "invalid_model_route",
      "Model route contains an unknown field",
    );
  }
  requireId(route.provider, "provider");
  requireId(route.modelId, "modelId");
  requireHttpsBaseUrl(route.baseUrl);
  return globalThis.structuredClone(route);
}

function validateNativeModelPair(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new ProtocolFault(
      "invalid_model_preflight",
      "Pi model preflight requires a native provider and model pair",
    );
  }
  const keys = Object.keys(value);
  if (
    keys.length !== 2 ||
    !keys.includes("provider") ||
    !keys.includes("modelId")
  ) {
    throw new ProtocolFault(
      "invalid_model_preflight",
      "Pi model preflight payload must contain only provider and modelId",
    );
  }
  requireId(value.provider, "provider");
  requireId(value.modelId, "modelId");
  return Object.freeze({ provider: value.provider, modelId: value.modelId });
}

function validateEligibleTools(value) {
  if (!Array.isArray(value) || value.length > C4OS_TOOL_NAMES.length) {
    throw new ProtocolFault(
      "invalid_tool_scope",
      "Eligible tools must be a bounded array of C4OS broker-tool IDs",
    );
  }
  const unique = new Set(value);
  if (
    unique.size !== value.length ||
    value.some(
      (toolName) =>
        typeof toolName !== "string" || !C4OS_TOOL_NAMES.includes(toolName),
    )
  ) {
    throw new ProtocolFault(
      "invalid_tool_scope",
      "Eligible tools must contain unique canonical C4OS broker-tool IDs",
    );
  }
  return Object.freeze([...value]);
}

function requireHttpsBaseUrl(value) {
  if (typeof value !== "string" || value.length === 0 || value.length > 2048) {
    throw new ProtocolFault(
      "invalid_model_route",
      "Model route baseUrl must be a bounded HTTPS URL",
    );
  }
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    throw new ProtocolFault(
      "invalid_model_route",
      "Model route baseUrl must be a bounded HTTPS URL",
    );
  }
  if (
    parsed.protocol !== "https:" ||
    parsed.username ||
    parsed.password ||
    parsed.search ||
    parsed.hash
  ) {
    throw new ProtocolFault(
      "invalid_model_route",
      "Model route baseUrl must be a bounded HTTPS URL",
    );
  }
}

function capability(state, reason = undefined) {
  return { state, ...(reason ? { reason } : {}) };
}

function sameRun(left, right) {
  return Boolean(
    left &&
    right &&
    left.processGeneration === right.processGeneration &&
    left.sessionId === right.sessionId &&
    left.runId === right.runId &&
    left.correlationId === right.correlationId,
  );
}

function toolKey(sessionId, runId, toolCallId) {
  return `${sessionId}\u0000${runId}\u0000${toolCallId}`;
}

function boundedReason(reason, fallback) {
  return typeof reason === "string" && reason.length > 0
    ? reason.slice(0, 512)
    : fallback;
}
