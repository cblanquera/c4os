import type { AppDispatch } from "../../app/store";
import {
  readPlatformSnapshot,
  type PlatformSnapshot,
} from "../../platform/platform-service";
import {
  readRuntimeCoreSnapshot,
  type RuntimeCoreSnapshot,
} from "../../platform/runtime-core";
import { readWorkspaceStartSnapshot } from "../../platform/workspace-start";
import type {
  RuntimeId,
  StateGeneration,
  WorkspaceStartSnapshot,
} from "../../platform/protocol";
import { shellAuthorityActions, type AuthoritativePublication } from "./state";

export interface NativeShellReaders {
  readonly readPlatform: () => Promise<PlatformSnapshot>;
  readonly readRuntime: () => Promise<RuntimeCoreSnapshot>;
  readonly readWorkspaceStart: () => Promise<WorkspaceStartSnapshot>;
  readonly readReducedMotion: () => boolean;
}

export interface NativeShellIngestionResult {
  readonly publishedDomains: readonly AuthoritativePublication["domain"][];
  readonly unavailableSources: readonly (
    "platform" | "runtime" | "workspaceStart"
  )[];
}

const defaultReaders: NativeShellReaders = {
  readPlatform: readPlatformSnapshot,
  readRuntime: readRuntimeCoreSnapshot,
  readWorkspaceStart: readWorkspaceStartSnapshot,
  readReducedMotion: () =>
    globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ??
    false,
};

/**
 * Publishes only projections derived from validated, allowlisted native
 * snapshots. Unsupported domains stay uninitialized until their owning task
 * supplies a real service projection.
 */
export async function ingestNativeShellProjections(
  dispatch: AppDispatch,
  readers: NativeShellReaders = defaultReaders,
): Promise<NativeShellIngestionResult> {
  const [platform, runtime, workspaceStart] = await Promise.allSettled([
    readers.readPlatform(),
    readers.readRuntime(),
    readers.readWorkspaceStart(),
  ]);
  const publications: AuthoritativePublication[] = [];
  const unavailableSources: NativeShellIngestionResult["unavailableSources"][number][] =
    [];

  if (platform.status === "fulfilled") {
    publications.push(platformPublication(platform.value, readers));
  } else {
    unavailableSources.push("platform");
  }

  if (runtime.status === "fulfilled") {
    publications.push(...runtimePublications(runtime.value));
  } else {
    unavailableSources.push("runtime");
  }

  if (workspaceStart.status === "fulfilled") {
    publications.push(workspacePublication(workspaceStart.value));
  } else {
    unavailableSources.push("workspaceStart");
  }

  if (runtime.status === "fulfilled" && workspaceStart.status === "fulfilled") {
    publications.push(launchPublication(runtime.value, workspaceStart.value));
  }

  for (const publication of publications) {
    dispatch(shellAuthorityActions.publicationReceived(publication));
  }

  return {
    publishedDomains: publications.map(({ domain }) => domain),
    unavailableSources,
  };
}

function platformPublication(
  snapshot: PlatformSnapshot,
  readers: NativeShellReaders,
): AuthoritativePublication {
  return {
    source: "snapshot",
    domain: "platform",
    generation: 0 as StateGeneration,
    value: {
      appearance: snapshot.initialTheme.scheme,
      appearanceSource: snapshot.initialTheme.source,
      reducedMotion: readers.readReducedMotion(),
    },
  };
}

function runtimePublications(
  snapshot: RuntimeCoreSnapshot,
): AuthoritativePublication[] {
  return [
    {
      source: "snapshot",
      domain: "runtime",
      generation: snapshot.generation,
      value: {
        runtimes: snapshot.runtimes.map((runtime) => ({
          id: runtime.runtimeId as RuntimeId,
          kind: runtime.runtimeKind,
          lifecycle: runtime.lifecycle,
          health: runtime.health,
        })),
      },
    },
    {
      source: "snapshot",
      domain: "approvals",
      generation: snapshot.generation,
      value: {
        approvals: snapshot.pendingApprovals.map((approval) => ({
          id: approval.promptId,
          summary: `Approval required by ${approval.runtimeId}.`,
          state: "pending",
        })),
      },
    },
  ];
}

function workspacePublication(
  snapshot: WorkspaceStartSnapshot,
): AuthoritativePublication {
  return {
    source: "snapshot",
    domain: "workspace",
    generation: snapshot.generation,
    value: {
      activeWorkspaceId: null,
      displayName: null,
      projects: [],
      activeProjectId: null,
    },
  };
}

function launchPublication(
  runtime: RuntimeCoreSnapshot,
  workspaceStart: WorkspaceStartSnapshot,
): AuthoritativePublication {
  const generation = Math.max(
    runtime.generation,
    workspaceStart.generation,
  ) as StateGeneration;
  return {
    source: "snapshot",
    domain: "launch",
    generation,
    value: {
      destination: runtime.onboardingReady ? "workspace-start" : "onboarding",
      providerConfigured: runtime.providers.some(({ enabled }) => enabled),
      onboardingReady: runtime.onboardingReady,
      recentWorkspaceIds: workspaceStart.recents.map(
        ({ workspaceId }) => workspaceId,
      ),
    },
  };
}
