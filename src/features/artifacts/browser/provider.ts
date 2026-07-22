import type { ArtifactProviderDefinition } from "../types";

export const BROWSER_ARTIFACT_PROVIDER: ArtifactProviderDefinition = {
  accessibleIdentity: "Browser response artifact",
  focusSupported: true,
  label: "Browser",
  supportedContexts: ["inline", "focused", "contextual"],
  type: "browser",
  version: 1,
};
