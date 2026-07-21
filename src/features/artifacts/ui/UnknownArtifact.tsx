import { ArtifactShell } from "./ArtifactShell";
import type { ArtifactContext, ArtifactShellActionCallbacks } from "../types";

export interface UnknownArtifactProps extends ArtifactShellActionCallbacks {
  readonly artifactId: string;
  readonly context: ArtifactContext;
  readonly message: string;
  readonly requestedType: string;
  readonly requestedVersion: number;
  readonly summary?: string;
  readonly title: string;
}

/** Renders unsupported provider data without exposing focus or provider controls. */
export function UnknownArtifact({
  artifactId,
  context,
  message,
  onCopy,
  onReply,
  requestedType,
  requestedVersion,
  summary = "This artifact can be reviewed only as a bounded inline record.",
  title,
}: UnknownArtifactProps) {
  return (
    <ArtifactShell
      context={context}
      focusSupported={false}
      identity={{
        accessibleLabel: `Unknown response artifact: ${title}`,
        id: artifactId,
        title,
        typeLabel: "Unknown artifact",
      }}
      onCopy={onCopy}
      onReply={onReply}
      status={{ kind: "degraded", message }}
      footerContent={
        <span>
          Requested {requestedType} provider version {requestedVersion}
        </span>
      }
    >
      <p>{summary}</p>
    </ArtifactShell>
  );
}
