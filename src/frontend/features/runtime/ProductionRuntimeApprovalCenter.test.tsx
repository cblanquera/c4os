import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it, vi } from "vitest";

import { PROTOCOL_VERSION } from "../../platform/protocol";
import {
  RUNTIME_CORE_SNAPSHOT_COMMAND,
  RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND,
  type RuntimeCoreSnapshot,
} from "../../platform/runtime-core";
import { ProductionRuntimeApprovalCenter } from "./ProductionRuntimeApprovalCenter";

const approval = {
  runtimeId: "pi-primary" as never,
  correlationId: "correlation-sampling" as never,
  promptId: "approval:sampling-1" as never,
  approvalKind: "mcp-sampling" as const,
  summary:
    "MCP server docs requests up to 256 tokens from provider-openai/gpt-5-mini using 2 bounded text message(s).",
  serverId: "docs",
  providerId: "provider-openai",
  modelId: "gpt-5-mini",
  maxTokens: 256,
  expiresAtMs: 1_721_300_010_000,
  messageCount: 2,
  inputBytes: 412,
  hasSystemPrompt: true,
  parentOperation: "c4os_propose_action",
  disclosureScope:
    "Private active-operation text will be disclosed to the selected model provider; credentials remain operation-scoped and hidden.",
};

function snapshot(
  pendingApprovals: RuntimeCoreSnapshot["pendingApprovals"],
): RuntimeCoreSnapshot {
  return {
    authority: "rust-core",
    generation: 7 as never,
    providerGeneration: 3,
    capabilityGeneration: 4,
    runtimeGeneration: 5,
    onboardingReady: true,
    providers: [],
    modelRoutes: [],
    runtimes: [],
    pendingApprovals,
  };
}

describe("ProductionRuntimeApprovalCenter", () => {
  afterEach(() => clearMocks());

  it("polls after startup and settles an informed MCP sampling prompt by exact identity", async () => {
    let answered = false;
    let reads = 0;
    const readSnapshot = vi.fn(async () => {
      reads += 1;
      return snapshot(reads === 1 || answered ? [] : [approval]);
    });
    const answerApproval = vi.fn(async () => {
      answered = true;
      return {
        runtimeId: approval.runtimeId,
        correlationId: approval.correlationId,
        promptId: approval.promptId,
      };
    });
    render(
      <ProductionRuntimeApprovalCenter
        answerApproval={answerApproval}
        enabled
        pollIntervalMs={5}
        readSnapshot={readSnapshot}
      />,
    );

    expect(
      await screen.findByRole("dialog", { name: "Allow MCP model sampling?" }),
    ).toBeInTheDocument();
    expect(screen.getByText("docs")).toBeInTheDocument();
    expect(screen.getByText("provider-openai/gpt-5-mini")).toBeInTheDocument();
    expect(screen.getByText(/2 message\(s\), 412 bytes/)).toBeInTheDocument();
    expect(document.body.textContent).not.toContain("private prompt canary");

    fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
    await waitFor(() =>
      expect(answerApproval).toHaveBeenCalledWith({
        runtimeId: approval.runtimeId,
        correlationId: approval.correlationId,
        promptId: approval.promptId,
        answer: "allow",
        remember: "once",
      }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
  });

  it("settles Deny from the normal renderer through the allowlisted Tauri command", async () => {
    let answered = false;
    const calls: Array<{ command: string; payload: unknown }> = [];
    mockIPC((command, payload) => {
      calls.push({ command, payload });
      if (
        payload === undefined ||
        Array.isArray(payload) ||
        typeof payload !== "object"
      ) {
        throw new Error("The native command payload was not an object.");
      }
      const request = (payload as Record<string, unknown>).request as {
        requestId: string;
        correlationId: string;
        expectedGeneration: number;
      };
      if (command === RUNTIME_CORE_SNAPSHOT_COMMAND) {
        const runtimeSnapshot = snapshot(answered ? [] : [approval]);
        return {
          protocolVersion: PROTOCOL_VERSION,
          requestId: request.requestId,
          correlationId: request.correlationId,
          generation: answered ? 102 : 101,
          payload: {
            authority: runtimeSnapshot.authority,
            providerGeneration: runtimeSnapshot.providerGeneration,
            capabilityGeneration: runtimeSnapshot.capabilityGeneration,
            runtimeGeneration: runtimeSnapshot.runtimeGeneration,
            onboardingReady: runtimeSnapshot.onboardingReady,
            providers: runtimeSnapshot.providers,
            modelRoutes: runtimeSnapshot.modelRoutes,
            runtimes: runtimeSnapshot.runtimes,
            pendingApprovals: runtimeSnapshot.pendingApprovals,
          },
        };
      }
      if (command === RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND) {
        expect(request.expectedGeneration).toBe(101);
        expect(payload).toMatchObject({
          runtimeId: approval.runtimeId,
          correlationId: approval.correlationId,
          promptId: approval.promptId,
          answer: "deny",
          remember: "once",
        });
        answered = true;
        return {
          protocolVersion: PROTOCOL_VERSION,
          requestId: request.requestId,
          correlationId: request.correlationId,
          generation: 102,
          payload: {
            runtimeId: approval.runtimeId,
            correlationId: approval.correlationId,
            promptId: approval.promptId,
          },
        };
      }
      throw new Error(`Unexpected native command: ${command}`);
    });

    render(<ProductionRuntimeApprovalCenter enabled pollIntervalMs={60_000} />);
    expect(
      await screen.findByRole("dialog", { name: "Allow MCP model sampling?" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Deny" }));

    await waitFor(() =>
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
    );
    expect(calls.map(({ command }) => command)).toEqual([
      RUNTIME_CORE_SNAPSHOT_COMMAND,
      RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND,
      RUNTIME_CORE_SNAPSHOT_COMMAND,
    ]);
  });
});
