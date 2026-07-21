import type { ArtifactProviderDefinition } from "../types";

export const FOLDER_ARTIFACT_PROVIDER: ArtifactProviderDefinition = {
  accessibleIdentity: "Folder response artifact",
  focusSupported: true,
  label: "Folder",
  supportedContexts: ["inline", "focused", "contextual"],
  type: "folder",
  version: 1,
};
