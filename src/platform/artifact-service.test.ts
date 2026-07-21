import { describe, expect, it, vi } from "vitest";

import {
  createArtifactAdapter,
  type ArtifactTransport,
} from "./artifact-service";
import type { SnapshotRequest } from "./protocol";

const DIGEST = `sha256:${"a".repeat(64)}`;

function fileArtifact(overrides: Record<string, unknown> = {}) {
  return {
    artifactId: "artifact-1",
    projectId: "project-1",
    sessionId: "session-1",
    providerType: "file",
    providerVersion: 1,
    stateSchemaVersion: 1,
    recordRevision: 3,
    title: "notes.txt",
    focusSupported: true,
    pendingApprovalId: null,
    status: { kind: "ready", message: null },
    sourceLabel: "Direct operation",
    resourceVersion: { sequence: 1, sha256: DIGEST, observedAtMs: 10 },
    history: [
      {
        recordRevision: 1,
        kind: "created",
        recordedAtMs: 10,
        resourceVersion: { sequence: 1, sha256: DIGEST, observedAtMs: 10 },
      },
    ],
    providerState: {
      type: "file",
      value: {
        breadcrumbs: [{ id: "notes.txt", label: "notes.txt", isCurrent: true }],
        languageLabel: "Plain text",
        versionLabel: "Version 1",
        state: { phase: "read", content: "hello" },
      },
    },
    ...overrides,
  };
}

function workspace(generation: number, artifacts: readonly unknown[]) {
  return {
    protocolVersion: 1,
    generation,
    authority: "rust-core",
    workspaceId: "workspace-1",
    activeProjectId: "project-1",
    activeSessionId: "session-1",
    focusedArtifactId: null,
    artifacts,
  };
}

function response(
  request: SnapshotRequest,
  generation: number,
  payload: unknown,
) {
  return {
    protocolVersion: 1,
    requestId: request.requestId,
    correlationId: request.correlationId,
    generation,
    payload,
  };
}

function expandedContext(maximumBytes: number) {
  return {
    snapshotId: "artifact-context-1",
    stableReference: `artifact:artifact-1:record:3:resource:1:${DIGEST}`,
    artifactId: "artifact-1",
    projectId: "project-1",
    sessionId: "session-1",
    providerType: "file",
    providerVersion: 1,
    artifactRecordRevision: 3,
    capturedResourceVersion: {
      sequence: 1,
      sha256: DIGEST,
      observedAtMs: 10,
    },
    payloadKind: "file",
    segments: [
      {
        priority: "selection",
        source: "selected-text",
        text: "hello",
        originalBytes: 5,
        omittedBytes: 0,
      },
    ],
    maximumBytes,
    usedBytes: 5,
    omittedBytes: 0,
    omittedSegments: 0,
    truncated: false,
    unsaved: false,
    redactions: ["secrets"],
    capabilities: [
      { capabilityId: "artifact.read", access: "readable", reasonCode: null },
    ],
    capturedAtMs: 11,
  };
}

describe("Artifact native adapter", () => {
  it("carries the exact generation and conflict intent through a serialized mutation", async () => {
    const invoke = vi.fn<ArtifactTransport["invoke"]>(async (command, args) => {
      const request = args.request as SnapshotRequest;
      if (command === "artifact_snapshot") {
        expect(request.expectedGeneration).toBe(0);
        return response(request, 4, workspace(4, [fileArtifact()]));
      }
      expect(command).toBe("artifact_resolve_file_conflict");
      expect(request.expectedGeneration).toBe(4);
      expect(args.input).toEqual({
        artifactId: "artifact-1",
        baseRecordRevision: 3,
        resolution: "keepDraft",
      });
      return response(request, 5, workspace(5, [fileArtifact()]));
    });
    const adapter = createArtifactAdapter({ invoke });

    await adapter.read();
    await adapter.resolveFileConflict({
      artifactId: "artifact-1" as never,
      baseRecordRevision: 3,
      resolution: "keepDraft",
    });

    expect(adapter.currentGeneration).toBe(5);
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("routes proposal rejection and exact approval answers through dedicated commands", async () => {
    const commands: string[] = [];
    const adapter = createArtifactAdapter({
      async invoke(command, args) {
        commands.push(command);
        const request = args.request as SnapshotRequest;
        return response(
          request,
          commands.length,
          workspace(commands.length, [fileArtifact()]),
        );
      },
    });

    await adapter.rejectFileProposal({
      artifactId: "artifact-1" as never,
      baseRecordRevision: 3,
    });
    await adapter.answerApproval({ promptId: "prompt-1", answer: "deny" });

    expect(commands).toEqual([
      "artifact_reject_file_proposal",
      "artifact_answer_approval",
    ]);
  });

  it("serializes selection-aware Reply and validates brokered context expansion", async () => {
    const commands: string[] = [];
    const adapter = createArtifactAdapter({
      async invoke(command, args) {
        commands.push(command);
        const request = args.request as SnapshotRequest;
        if (command === "artifact_reply") {
          expect(args.input).toEqual({
            artifactId: "artifact-1",
            baseRecordRevision: 3,
            selectedText: "hello",
          });
          return response(request, 2, workspace(2, [fileArtifact()]));
        }
        if (command === "artifact_expand_context") {
          expect(request.expectedGeneration).toBe(2);
          expect(args.input).toEqual({
            artifactId: "artifact-1",
            baseRecordRevision: 3,
            expectedResourceVersion: {
              sequence: 1,
              sha256: DIGEST,
              observedAtMs: 10,
            },
            maximumBytes: 4_096,
            selectedText: "hello",
          });
          return response(request, 2, expandedContext(4_096));
        }
        return response(request, 1, workspace(1, [fileArtifact()]));
      },
    });

    await adapter.read();
    await adapter.reply({
      artifactId: "artifact-1" as never,
      baseRecordRevision: 3,
      selectedText: "hello",
    });
    const context = await adapter.expandContext({
      artifactId: "artifact-1" as never,
      baseRecordRevision: 3,
      expectedResourceVersion: {
        sequence: 1,
        sha256: DIGEST,
        observedAtMs: 10,
      },
      maximumBytes: 4_096,
      selectedText: "hello",
    });

    expect(context.segments[0]).toMatchObject({
      priority: "selection",
      text: "hello",
    });
    expect(commands).toEqual([
      "artifact_snapshot",
      "artifact_reply",
      "artifact_expand_context",
    ]);
  });

  it("fails closed when a known provider version is projected as unknown", async () => {
    const adapter = createArtifactAdapter({
      async invoke(_command, args) {
        const request = args.request as SnapshotRequest;
        return response(
          request,
          1,
          workspace(1, [
            fileArtifact({
              focusSupported: false,
              status: { kind: "degraded", message: "Unsupported" },
              providerState: { type: "unknown" },
            }),
          ]),
        );
      },
    });

    await expect(adapter.read()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("accepts a future unknown provider only as degraded inline-only state", async () => {
    const adapter = createArtifactAdapter({
      async invoke(_command, args) {
        const request = args.request as SnapshotRequest;
        return response(
          request,
          1,
          workspace(1, [
            fileArtifact({
              providerType: "future-provider",
              providerVersion: 7,
              stateSchemaVersion: 4,
              focusSupported: false,
              status: { kind: "degraded", message: "Unsupported" },
              providerState: { type: "unknown" },
            }),
          ]),
        );
      },
    });

    const snapshot = await adapter.read();
    expect(snapshot.artifacts[0]?.providerState).toEqual({ type: "unknown" });
    expect(snapshot.artifacts[0]?.focusSupported).toBe(false);
  });
});
