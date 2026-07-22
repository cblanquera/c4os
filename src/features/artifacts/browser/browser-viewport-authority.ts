import type {
  ArtifactBrowserIdentityInput,
  ArtifactSnapshot,
} from "../../../platform/artifact-service";
import type { BrowserViewportLifecycleEvent } from "./browser-types";

type BrowserViewportIdentity = Pick<
  BrowserViewportLifecycleEvent,
  "artifactId" | "controllerGeneration" | "mountGeneration"
>;

/**
 * Rebinds a renderer lifecycle cleanup to the latest Rust-owned Browser
 * snapshot. A replaced native controller must not detach its successor, and a
 * stale renderer record revision must never be reused for the detach CAS.
 */
export function currentBrowserViewportDetachInput(
  current: ArtifactSnapshot | undefined,
  event: BrowserViewportIdentity,
): ArtifactBrowserIdentityInput | null {
  if (
    current?.providerState.type !== "browser" ||
    current.artifactId !== event.artifactId ||
    current.providerState.value.controllerGeneration !==
      event.controllerGeneration ||
    current.providerState.value.mountGeneration !== event.mountGeneration
  ) {
    return null;
  }
  return {
    artifactId: current.artifactId,
    baseRecordRevision: current.recordRevision,
    controllerGeneration: event.controllerGeneration,
    mountGeneration: event.mountGeneration,
  };
}
