import type {
  ArtifactProviderDefinition,
  ResolvedArtifactProvider,
} from "./types";

export interface ArtifactProviderRegistry {
  readonly providers: readonly ArtifactProviderDefinition[];
  resolve(
    providerType: string,
    providerVersion: number,
  ): ResolvedArtifactProvider;
}

export const UNKNOWN_ARTIFACT_PROVIDER: ArtifactProviderDefinition = {
  accessibleIdentity: "Unknown response artifact",
  focusSupported: false,
  label: "Unknown artifact",
  supportedContexts: ["inline"],
  type: "unknown",
  version: 1,
};

/** Creates an immutable exact-version provider lookup with a safe fallback. */
export function createArtifactProviderRegistry(
  providers: readonly ArtifactProviderDefinition[],
): ArtifactProviderRegistry {
  const byKey = new Map<string, ArtifactProviderDefinition>();
  const versionsByType = new Map<string, Set<number>>();

  // Validate definitions before any caller can resolve a partially valid set.
  for (const provider of providers) {
    validateProvider(provider);
    const key = providerKey(provider.type, provider.version);
    if (byKey.has(key)) {
      throw new Error(`Duplicate artifact provider registration: ${key}`);
    }
    byKey.set(
      key,
      Object.freeze({
        ...provider,
        supportedContexts: Object.freeze([...provider.supportedContexts]),
      }),
    );
    const versions = versionsByType.get(provider.type) ?? new Set<number>();
    versions.add(provider.version);
    versionsByType.set(provider.type, versions);
  }

  const frozenProviders = Object.freeze(Array.from(byKey.values()));

  const registry: ArtifactProviderRegistry = {
    providers: frozenProviders,
    resolve(
      providerType: string,
      providerVersion: number,
    ): ResolvedArtifactProvider {
      const exact = byKey.get(providerKey(providerType, providerVersion));
      if (exact) {
        return { kind: "registered", provider: exact };
      }

      const reason = versionsByType.has(providerType)
        ? "unknown-version"
        : "unknown-provider";
      const description =
        reason === "unknown-version"
          ? `Artifact version ${providerVersion} is not supported.`
          : `Artifact provider ${providerType} is not installed.`;
      return {
        kind: "fallback",
        message: description,
        provider: UNKNOWN_ARTIFACT_PROVIDER,
        reason,
        requestedType: providerType,
        requestedVersion: providerVersion,
      };
    },
  };
  return Object.freeze(registry);
}

/** Builds the collision-resistant registry key for one exact provider version. */
function providerKey(type: string, version: number): string {
  return `${type}\u0000${version}`;
}

/** Rejects provider declarations that could weaken focus or version handling. */
function validateProvider(provider: ArtifactProviderDefinition): void {
  if (provider.type.trim().length === 0 || provider.label.trim().length === 0) {
    throw new Error("Artifact providers require a type and label.");
  }
  if (!Number.isSafeInteger(provider.version) || provider.version <= 0) {
    throw new Error("Artifact provider versions must be positive integers.");
  }
  if (!provider.supportedContexts.includes("inline")) {
    throw new Error("Artifact providers must support the inline context.");
  }
  if (
    provider.focusSupported &&
    !provider.supportedContexts.includes("focused")
  ) {
    throw new Error(
      "Focus-capable providers must support the focused context.",
    );
  }
}
