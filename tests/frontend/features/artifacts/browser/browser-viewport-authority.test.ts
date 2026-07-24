import { describe, expect, it } from "vitest";

import type { ArtifactSnapshot } from "../../../../../src/frontend/platform/artifact-service";
import { currentBrowserViewportDetachInput } from "../../../../../src/frontend/features/artifacts/browser/browser-viewport-authority";

const DIGEST = `sha256:${"a".repeat(64)}`;

function browserSnapshot(
  overrides: Partial<ArtifactSnapshot> = {},
): ArtifactSnapshot {
  return {
    artifactId: "artifact-browser" as never,
    projectId: "project-browser" as never,
    sessionId: "session-browser" as never,
    providerType: "browser",
    providerVersion: 1,
    stateSchemaVersion: 1,
    recordRevision: 17,
    title: "Example",
    focusSupported: true,
    pendingApprovalId: null,
    status: { kind: "ready" },
    sourceLabel: "Direct operation",
    resourceVersion: { sequence: 18, sha256: DIGEST, observedAtMs: 18 },
    history: [],
    providerState: {
      type: "browser",
      value: {
        currentUrl: "https://example.com/",
        pageTitle: "Example",
        phase: "ready",
        refreshing: false,
        canGoBack: false,
        canGoForward: false,
        controllerGeneration: 3,
        mountGeneration: 2,
        environmentScope: "per-chat-session",
        pendingOperation: null,
        pendingTargetUrl: null,
        notices: [],
      },
    },
    ...overrides,
  };
}

describe("currentBrowserViewportDetachInput", () => {
  it("uses the latest record revision for a still-current native controller", () => {
    expect(
      currentBrowserViewportDetachInput(browserSnapshot(), {
        artifactId: "artifact-browser",
        controllerGeneration: 3,
        mountGeneration: 2,
      }),
    ).toEqual({
      artifactId: "artifact-browser",
      baseRecordRevision: 17,
      controllerGeneration: 3,
      mountGeneration: 2,
    });
  });

  it("ignores cleanup from a native controller replaced during profile clear", () => {
    expect(
      currentBrowserViewportDetachInput(browserSnapshot(), {
        artifactId: "artifact-browser",
        controllerGeneration: 2,
        mountGeneration: 2,
      }),
    ).toBeNull();
  });
});
