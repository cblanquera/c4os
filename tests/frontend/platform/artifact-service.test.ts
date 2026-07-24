import { describe, expect, it, vi } from "vitest";

import {
  createArtifactAdapter,
  type ArtifactTransport,
} from "../../../src/frontend/platform/artifact-service";
import type { SnapshotRequest } from "../../../src/frontend/platform/protocol";

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

function terminalArtifact(overrides: Record<string, unknown> = {}) {
  return {
    artifactId: "artifact-terminal-1",
    projectId: "project-1",
    sessionId: "session-1",
    providerType: "terminal",
    providerVersion: 1,
    stateSchemaVersion: 1,
    recordRevision: 4,
    title: "$ read value",
    focusSupported: true,
    pendingApprovalId: null,
    status: { kind: "ready", message: null },
    sourceLabel: "Direct operation",
    resourceVersion: { sequence: 4, sha256: DIGEST, observedAtMs: 13 },
    history: [
      {
        recordRevision: 1,
        kind: "commandQueued",
        recordedAtMs: 10,
        resourceVersion: { sequence: 1, sha256: DIGEST, observedAtMs: 10 },
      },
    ],
    providerState: {
      type: "terminal",
      value: {
        terminalSessionId: "terminal-session-1",
        commandId: "terminal-command-1",
        commandSequence: 1,
        command: "read value",
        workingDirectoryDisplay: "/project",
        shellPath: "/bin/zsh",
        environmentId: "desktop",
        environmentGeneration: 1,
        processGeneration: 1,
        shellProcessId: 42,
        foregroundProcessGroupId: 43,
        columns: 80,
        rows: 24,
        outputBase64: "aGVsbG8K",
        outputText: "hello\n",
        outputSequence: 2,
        retainedBytes: 6,
        droppedBytes: 0,
        phase: "stdinReady",
        exitCode: null,
        statusMessage: null,
        stdinReady: true,
        stopAvailable: true,
        promptReady: false,
        shellReplaced: false,
      },
    },
    ...overrides,
  };
}

function browserArtifact(overrides: Record<string, unknown> = {}) {
  return {
    artifactId: "artifact-browser-1",
    projectId: "project-1",
    sessionId: "session-1",
    providerType: "browser",
    providerVersion: 1,
    stateSchemaVersion: 1,
    recordRevision: 6,
    title: "Example",
    focusSupported: true,
    pendingApprovalId: null,
    status: { kind: "ready", message: null },
    sourceLabel: "Direct operation",
    resourceVersion: { sequence: 6, sha256: DIGEST, observedAtMs: 15 },
    history: [
      {
        recordRevision: 1,
        kind: "browserOpened",
        recordedAtMs: 10,
        resourceVersion: { sequence: 1, sha256: DIGEST, observedAtMs: 10 },
      },
    ],
    providerState: {
      type: "browser",
      value: {
        currentUrl: "https://example.com/",
        pageTitle: "Example",
        phase: "ready",
        refreshing: false,
        canGoBack: true,
        canGoForward: false,
        controllerGeneration: 2,
        mountGeneration: 3,
        environmentScope: "per-chat-session",
        pendingOperation: null,
        pendingTargetUrl: null,
        notices: [
          {
            id: "browser-notice-1",
            kind: "information",
            title: "Browser ready",
            message: "The website finished loading.",
          },
        ],
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

  it("validates Terminal lifecycle state and serializes every exact live operation", async () => {
    const calls: { command: string; input: unknown }[] = [];
    const adapter = createArtifactAdapter({
      async invoke(command, args) {
        calls.push({ command, input: args.input });
        const request = args.request as SnapshotRequest;
        return response(
          request,
          calls.length,
          workspace(calls.length, [terminalArtifact()]),
        );
      },
    });

    const read = await adapter.read();
    expect(read.artifacts[0]?.providerState).toMatchObject({
      type: "terminal",
      value: { phase: "stdinReady", outputText: "hello\n" },
    });
    await adapter.runTerminal({ command: "pwd", columns: 90, rows: 28 });
    await adapter.submitTerminalStdin({
      artifactId: "artifact-terminal-1" as never,
      baseRecordRevision: 4,
      processGeneration: 1,
      text: "answer",
    });
    await adapter.resizeTerminal({
      artifactId: "artifact-terminal-1" as never,
      baseRecordRevision: 4,
      processGeneration: 1,
      columns: 100,
      rows: 30,
    });
    await adapter.stopTerminal({
      artifactId: "artifact-terminal-1" as never,
      baseRecordRevision: 4,
      processGeneration: 1,
    });
    await adapter.acknowledgeTerminalOutput({
      artifactId: "artifact-terminal-1" as never,
      processGeneration: 1,
      outputSequence: 2,
    });

    expect(calls).toEqual([
      { command: "artifact_snapshot", input: undefined },
      {
        command: "artifact_run_terminal",
        input: { command: "pwd", columns: 90, rows: 28 },
      },
      {
        command: "artifact_terminal_stdin",
        input: {
          artifactId: "artifact-terminal-1",
          baseRecordRevision: 4,
          processGeneration: 1,
          text: "answer",
        },
      },
      {
        command: "artifact_terminal_resize",
        input: {
          artifactId: "artifact-terminal-1",
          baseRecordRevision: 4,
          processGeneration: 1,
          columns: 100,
          rows: 30,
        },
      },
      {
        command: "artifact_terminal_stop",
        input: {
          artifactId: "artifact-terminal-1",
          baseRecordRevision: 4,
          processGeneration: 1,
        },
      },
      {
        command: "artifact_terminal_ack_output",
        input: {
          artifactId: "artifact-terminal-1",
          processGeneration: 1,
          outputSequence: 2,
        },
      },
    ]);
  });

  it("fails closed for unsafe or lifecycle-inconsistent Terminal projections", async () => {
    const invalid = [
      terminalArtifact({
        providerState: {
          ...terminalArtifact().providerState,
          value: {
            ...(terminalArtifact().providerState as { value: object }).value,
            outputText: "visible\u001bhidden",
          },
        },
      }),
      terminalArtifact({
        providerState: {
          ...terminalArtifact().providerState,
          value: {
            ...(terminalArtifact().providerState as { value: object }).value,
            retainedBytes: 7,
          },
        },
      }),
      terminalArtifact({
        providerState: {
          ...terminalArtifact().providerState,
          value: {
            ...(terminalArtifact().providerState as { value: object }).value,
            phase: "completed",
            exitCode: 0,
            foregroundProcessGroupId: 43,
            stdinReady: false,
            stopAvailable: false,
            promptReady: true,
          },
        },
      }),
    ];
    for (const artifact of invalid) {
      const adapter = createArtifactAdapter({
        async invoke(_command, args) {
          const request = args.request as SnapshotRequest;
          return response(request, 1, workspace(1, [artifact]));
        },
      });
      await expect(adapter.read()).rejects.toMatchObject({
        code: "invalidPayload",
      });
    }
  });

  it("accepts a pending live Terminal control only when Stop authority is withheld", async () => {
    const pendingLive = terminalArtifact({
      pendingApprovalId: "approval-terminal-input",
      status: {
        kind: "ready",
        message:
          "Approval is required before this Terminal operation can continue.",
      },
      providerState: {
        ...terminalArtifact().providerState,
        value: {
          ...(terminalArtifact().providerState as { value: object }).value,
          stopAvailable: false,
        },
      },
    });
    const adapter = createArtifactAdapter({
      async invoke(_command, args) {
        const request = args.request as SnapshotRequest;
        return response(request, 1, workspace(1, [pendingLive]));
      },
    });
    await expect(adapter.read()).resolves.toMatchObject({
      artifacts: [
        {
          pendingApprovalId: "approval-terminal-input",
          providerState: {
            type: "terminal",
            value: { phase: "stdinReady", stopAvailable: false },
          },
        },
      ],
    });

    const unsafe = {
      ...pendingLive,
      providerState: {
        ...pendingLive.providerState,
        value: {
          ...(pendingLive.providerState as { value: object }).value,
          stopAvailable: true,
        },
      },
    };
    const unsafeAdapter = createArtifactAdapter({
      async invoke(_command, args) {
        const request = args.request as SnapshotRequest;
        return response(request, 1, workspace(1, [unsafe]));
      },
    });
    await expect(unsafeAdapter.read()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("validates Browser state and serializes every exact native operation", async () => {
    const calls: { command: string; input: unknown }[] = [];
    const adapter = createArtifactAdapter({
      async invoke(command, args) {
        calls.push({ command, input: args.input });
        const request = args.request as SnapshotRequest;
        return response(
          request,
          calls.length,
          workspace(calls.length, [browserArtifact()]),
        );
      },
    });
    const identity = {
      artifactId: "artifact-browser-1" as never,
      baseRecordRevision: 6,
      controllerGeneration: 2,
      mountGeneration: 3,
    };

    const read = await adapter.read();
    expect(read.artifacts[0]?.providerState).toMatchObject({
      type: "browser",
      value: {
        currentUrl: "https://example.com/",
        environmentScope: "per-chat-session",
        pendingOperation: null,
        phase: "ready",
      },
    });
    await adapter.openBrowser({ address: "example.com" });
    await adapter.navigateBrowser({ ...identity, intent: "back" });
    await adapter.clearBrowserData(identity);
    await adapter.mountBrowser({
      ...identity,
      x: 10,
      y: 20,
      width: 900,
      height: 600,
      focus: false,
    });
    await adapter.resizeBrowser({
      ...identity,
      x: 11,
      y: 21,
      width: 901,
      height: 601,
      focus: true,
    });
    await adapter.focusNativeBrowser(identity);
    await adapter.detachBrowser(identity);

    expect(calls).toEqual([
      { command: "artifact_snapshot", input: undefined },
      { command: "artifact_open_browser", input: { address: "example.com" } },
      {
        command: "artifact_navigate_browser",
        input: { ...identity, intent: "back" },
      },
      { command: "artifact_clear_browser_data", input: identity },
      {
        command: "artifact_mount_browser",
        input: {
          ...identity,
          x: 10,
          y: 20,
          width: 900,
          height: 600,
          focus: false,
        },
      },
      {
        command: "artifact_resize_browser",
        input: {
          ...identity,
          x: 11,
          y: 21,
          width: 901,
          height: 601,
          focus: true,
        },
      },
      { command: "artifact_focus_native_browser", input: identity },
      { command: "artifact_detach_browser", input: identity },
    ]);
  });

  it("fails closed for inconsistent Browser approval and lifecycle projections", async () => {
    const invalid = [
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            phase: "executing",
          },
        },
      }),
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            environmentScope: "global",
          },
        },
      }),
      browserArtifact({
        pendingApprovalId: "approval-browser-1",
      }),
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pendingOperation: "clear-everything",
          },
        },
      }),
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pendingOperation: "clear-data",
          },
        },
      }),
      browserArtifact({
        pendingApprovalId: "approval-browser-1",
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pendingOperation: "website-navigation",
          },
        },
      }),
      browserArtifact({
        pendingApprovalId: "approval-browser-1",
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pendingOperation: "website-navigation",
            pendingTargetUrl: "https://example.com/path?private=value",
          },
        },
      }),
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pendingTargetUrl: "https://example.com/path",
          },
        },
      }),
      browserArtifact({
        pendingApprovalId: "approval-browser-1",
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pendingOperation: "clear-data",
            pendingTargetUrl: "https://example.com/path",
          },
        },
      }),
      ...[
        "javascript:alert(1)",
        "file:///etc/passwd",
        "https://user:password@example.com/",
        "https://@example.com/",
        "https://example.com/?private=value",
        "https://example.com/#private",
      ].map((currentUrl) =>
        browserArtifact({
          providerState: {
            ...browserArtifact().providerState,
            value: {
              ...(browserArtifact().providerState as { value: object }).value,
              currentUrl,
            },
          },
        }),
      ),
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            pageTitle: "Unsafe\npage title",
          },
        },
      }),
      browserArtifact({
        providerState: {
          ...browserArtifact().providerState,
          value: {
            ...(browserArtifact().providerState as { value: object }).value,
            notices: [
              {
                id: "browser-notice-unsafe",
                kind: "warning",
                title: "Unsafe notice",
                message: "Unsafe\u001bmessage",
              },
            ],
          },
        },
      }),
    ];

    for (const artifact of invalid) {
      const adapter = createArtifactAdapter({
        async invoke(_command, args) {
          const request = args.request as SnapshotRequest;
          return response(request, 1, workspace(1, [artifact]));
        },
      });
      await expect(adapter.read()).rejects.toMatchObject({
        code: "invalidPayload",
      });
    }

    const pending = browserArtifact({
      pendingApprovalId: "approval-browser-1",
      providerState: {
        ...browserArtifact().providerState,
        value: {
          ...(browserArtifact().providerState as { value: object }).value,
          pendingOperation: "clear-data",
        },
      },
    });
    const adapter = createArtifactAdapter({
      async invoke(_command, args) {
        const request = args.request as SnapshotRequest;
        return response(request, 1, workspace(1, [pending]));
      },
    });
    await expect(adapter.read()).resolves.toMatchObject({
      artifacts: [
        {
          pendingApprovalId: "approval-browser-1",
          providerState: {
            type: "browser",
            value: { pendingOperation: "clear-data" },
          },
        },
      ],
    });

    const websiteNavigation = browserArtifact({
      pendingApprovalId: "approval-browser-2",
      providerState: {
        ...browserArtifact().providerState,
        value: {
          ...(browserArtifact().providerState as { value: object }).value,
          pendingOperation: "website-navigation",
          pendingTargetUrl: "https://example.com/path",
        },
      },
    });
    const websiteAdapter = createArtifactAdapter({
      async invoke(_command, args) {
        const request = args.request as SnapshotRequest;
        return response(request, 2, workspace(2, [websiteNavigation]));
      },
    });
    await expect(websiteAdapter.read()).resolves.toMatchObject({
      artifacts: [
        {
          pendingApprovalId: "approval-browser-2",
          providerState: {
            type: "browser",
            value: {
              pendingOperation: "website-navigation",
              pendingTargetUrl: "https://example.com/path",
            },
          },
        },
      ],
    });
  });
});
