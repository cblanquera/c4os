import type { ArtifactWorkspaceSnapshot } from "../../platform/artifact-service";
import type { SessionId } from "../../platform/protocol";

/** Rejects cached Artifact state that belongs to a previously active Chat. */
export function artifactWorkspaceForActiveSession(
  snapshot: ArtifactWorkspaceSnapshot | null,
  activeSessionId: SessionId | null,
): ArtifactWorkspaceSnapshot | null {
  return snapshot?.activeSessionId === activeSessionId &&
    activeSessionId !== null
    ? snapshot
    : null;
}
