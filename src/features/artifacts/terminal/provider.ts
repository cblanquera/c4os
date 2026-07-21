import type { ArtifactProviderDefinition } from "../types";

export const TERMINAL_ARTIFACT_PROVIDER: ArtifactProviderDefinition = {
  accessibleIdentity: "Terminal response artifact",
  focusSupported: true,
  label: "Terminal",
  supportedContexts: ["inline", "focused", "contextual"],
  type: "terminal",
  version: 1,
};
