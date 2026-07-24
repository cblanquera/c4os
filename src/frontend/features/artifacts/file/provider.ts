import type { ArtifactProviderDefinition } from "../types";

export const FILE_ARTIFACT_PROVIDER: ArtifactProviderDefinition = {
  accessibleIdentity: "File response artifact",
  focusSupported: true,
  label: "File",
  supportedContexts: ["inline", "focused", "contextual"],
  type: "file",
  version: 1,
};
