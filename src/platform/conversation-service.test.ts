import { describe, expect, it } from "vitest";

import type { StateGeneration } from "./protocol";
import {
  createConversationAdapter,
  type ConversationSnapshot,
  type ConversationTransport,
} from "./conversation-service";

describe("Conversation native boundary", () => {
  it("correlates responses and advances one shared CAS cursor", async () => {
    const expected: number[] = [];
    let generation = 8;
    const adapter = createConversationAdapter({
      async invoke(_command, args) {
        const request = args.request as Record<string, unknown>;
        expected.push(request.expectedGeneration as number);
        const response = envelope(request, generation);
        generation += 1;
        return response;
      },
    });

    expect((await adapter.read()).generation).toBe(8);
    expect((await adapter.begin("project-1" as never)).generation).toBe(9);
    expect(expected).toEqual([0, 8]);
    expect(adapter.currentGeneration).toBe(9);
  });

  it("rejects a payload that substitutes renderer authority", async () => {
    const transport: ConversationTransport = {
      async invoke(_command, args) {
        const request = args.request as Record<string, unknown>;
        const response = envelope(request, 1);
        return {
          ...response,
          payload: { ...response.payload, authority: "renderer" },
        };
      },
    };
    await expect(createConversationAdapter(transport).read()).rejects.toThrow(
      "authority",
    );
  });

  it("routes cancellation and retry through the shared generation cursor", async () => {
    const calls: {
      command: string;
      args: Readonly<Record<string, unknown>>;
    }[] = [];
    let generation = 3;
    const adapter = createConversationAdapter({
      async invoke(command, args) {
        calls.push({ command, args });
        const request = args.request as Record<string, unknown>;
        generation += 1;
        return envelope(request, generation);
      },
    });

    await adapter.cancelAttempt("attempt-1" as never);
    await adapter.retryAttempt({
      parentAttemptId: "attempt-1" as never,
      providerId: "provider-1",
      modelId: "model-1",
      reasoningMode: "low",
    });

    expect(calls.map(({ command }) => command)).toEqual([
      "conversation_cancel_attempt",
      "conversation_retry_attempt",
    ]);
    expect(calls[0]?.args.attemptId).toBe("attempt-1");
    expect(calls[1]?.args.input).toEqual({
      parentAttemptId: "attempt-1",
      providerId: "provider-1",
      modelId: "model-1",
      reasoningMode: "low",
    });
    expect(
      (calls[1]?.args.request as Record<string, unknown>).expectedGeneration,
    ).toBe(4);
  });

  it("routes branch requests and exact approval answers through the shared cursor", async () => {
    const calls: {
      command: string;
      args: Readonly<Record<string, unknown>>;
    }[] = [];
    let generation = 10;
    const adapter = createConversationAdapter({
      async invoke(command, args) {
        calls.push({ command, args });
        const request = args.request as Record<string, unknown>;
        generation += 1;
        return envelope(request, generation);
      },
    });

    await adapter.requestBranch({
      operation: "switch",
      branch: "feature/safe",
    });
    await adapter.answerBranchApproval({
      promptId: "approval:branch-1",
      answer: "allow",
    });

    expect(calls.map(({ command }) => command)).toEqual([
      "conversation_request_branch",
      "conversation_answer_branch_approval",
    ]);
    expect(calls[0]?.args.input).toEqual({
      operation: "switch",
      branch: "feature/safe",
    });
    expect(calls[1]?.args.input).toEqual({
      promptId: "approval:branch-1",
      answer: "allow",
    });
    expect(
      (calls[1]?.args.request as Record<string, unknown>).expectedGeneration,
    ).toBe(11);
  });

  it("accepts only a correlated bounded preview for the exact draft attachment", async () => {
    const adapter = createConversationAdapter({
      async invoke(command, args) {
        const request = args.request as Record<string, unknown>;
        if (command === "conversation_attachment_preview") {
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 7,
            payload: {
              attachmentId: "attachment-1",
              mediaType: "image/png",
              dataUrl: "data:image/png;base64,AA==",
            },
          };
        }
        return envelope(request, 7);
      },
    });

    await adapter.read();
    await expect(
      adapter.previewAttachment({
        attachmentId: "attachment-1" as never,
        stableReference: `workspace-blob:sha256:${"a".repeat(64)}:v1`,
      }),
    ).resolves.toEqual({
      attachmentId: "attachment-1",
      mediaType: "image/png",
      dataUrl: "data:image/png;base64,AA==",
    });
  });

  it("rejects substituted attachment preview content", async () => {
    const adapter = createConversationAdapter({
      async invoke(command, args) {
        const request = args.request as Record<string, unknown>;
        if (command === "conversation_attachment_preview") {
          return {
            protocolVersion: 1,
            requestId: request.requestId,
            correlationId: request.correlationId,
            generation: 0,
            payload: {
              attachmentId: "attachment-other",
              mediaType: "text/html",
              dataUrl: "data:text/html;base64,PHNjcmlwdD4=",
            },
          };
        }
        return envelope(request, 0);
      },
    });

    await expect(
      adapter.previewAttachment({
        attachmentId: "attachment-1" as never,
        stableReference: `workspace-blob:sha256:${"a".repeat(64)}:v1`,
      }),
    ).rejects.toThrow("invalid");
  });
});

function envelope(request: Record<string, unknown>, generation: number) {
  return {
    protocolVersion: 1,
    requestId: request.requestId,
    correlationId: request.correlationId,
    generation,
    payload: snapshot(generation as StateGeneration),
  };
}

function snapshot(generation: StateGeneration): ConversationSnapshot {
  return {
    protocolVersion: 1,
    generation,
    authority: "rust-core",
    workspaceId: "workspace-1" as never,
    workspaceName: "Workspace",
    activeProjectId: "project-1" as never,
    activeSessionId: null,
    pending: null,
    draft: {
      prompt: "",
      attachments: [],
      nextAttachmentReference: 1,
      providerId: null,
      modelId: null,
      reasoningMode: null,
      mode: "chat",
      replyTargetId: null,
    },
    projects: [
      {
        projectId: "project-1" as never,
        displayName: "Project",
        pathState: "found",
        position: 0,
        gitVersioned: false,
      },
    ],
    sessions: [],
    activeConversation: null,
    models: [],
    branchControl: null,
  };
}
