import type { ArtifactWorkspaceSnapshot } from "../../platform/artifact-service";
import type { SessionId } from "../../platform/protocol";
import type { SessionsProjection } from "./state";

/** Artifacts are durable Chat records and never attach to a blank provisional Chat. */
export function activeSessionCanOwnArtifacts(
  sessions: SessionsProjection,
): boolean {
  return (
    sessions.activeSessionId !== null &&
    sessions.sessions.some(
      (session) =>
        session.id === sessions.activeSessionId &&
        session.lifecycle === "saved",
    )
  );
}

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
