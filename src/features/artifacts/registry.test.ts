import { describe, expect, it } from "vitest";

import {
  ARTIFACT_PROVIDER_REGISTRY,
  createArtifactProviderRegistry,
  UNKNOWN_ARTIFACT_PROVIDER,
} from "./registry";
import type { ArtifactProviderDefinition } from "./types";

const FILE_PROVIDER: ArtifactProviderDefinition = {
  accessibleIdentity: "File response artifact",
  focusSupported: true,
  label: "File",
  supportedContexts: ["inline", "focused", "contextual"],
  type: "file",
  version: 1,
};

describe("artifact provider registry", () => {
  it("production-composes the exact Terminal provider version", () => {
    expect(ARTIFACT_PROVIDER_REGISTRY.resolve("terminal", 1)).toMatchObject({
      kind: "registered",
      provider: { type: "terminal", version: 1, focusSupported: true },
    });
  });

  it("resolves only an exact registered provider version", () => {
    const versionTwo = { ...FILE_PROVIDER, version: 2 };
    const registry = createArtifactProviderRegistry([
      FILE_PROVIDER,
      versionTwo,
    ]);

    expect(registry.resolve("file", 1)).toEqual({
      kind: "registered",
      provider: FILE_PROVIDER,
    });
    expect(registry.resolve("file", 2)).toEqual({
      kind: "registered",
      provider: versionTwo,
    });
    expect(registry.providers).toHaveLength(2);
  });

  it("fails closed to an inline-only provider for unknown types and versions", () => {
    const registry = createArtifactProviderRegistry([FILE_PROVIDER]);

    expect(registry.resolve("file", 99)).toMatchObject({
      kind: "fallback",
      provider: UNKNOWN_ARTIFACT_PROVIDER,
      reason: "unknown-version",
      requestedType: "file",
      requestedVersion: 99,
    });
    expect(registry.resolve("future-canvas", 1)).toMatchObject({
      kind: "fallback",
      provider: UNKNOWN_ARTIFACT_PROVIDER,
      reason: "unknown-provider",
    });
    expect(UNKNOWN_ARTIFACT_PROVIDER.focusSupported).toBe(false);
    expect(UNKNOWN_ARTIFACT_PROVIDER.supportedContexts).toEqual(["inline"]);
  });

  it("rejects duplicate and focus-incoherent registrations", () => {
    expect(() =>
      createArtifactProviderRegistry([FILE_PROVIDER, FILE_PROVIDER]),
    ).toThrow("Duplicate artifact provider registration");
    expect(() =>
      createArtifactProviderRegistry([
        {
          ...FILE_PROVIDER,
          supportedContexts: ["inline"],
        },
      ]),
    ).toThrow("Focus-capable providers must support the focused context");
  });
});
