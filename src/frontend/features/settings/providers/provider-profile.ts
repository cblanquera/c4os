import type {
  ConfigurableProviderKind,
  ProviderAuthentication,
  ProviderModel,
  ProviderProfileDraft,
  ProviderRecord,
} from "../../../platform/provider-service";

export type ProviderFormDraft = {
  readonly authenticationType: ProviderAuthentication["type"];
  readonly baseUrl: string;
  readonly credentialRevision: number;
  readonly displayName: string;
  readonly enabled: boolean;
  readonly endpointId: string;
  readonly hasCredential: boolean;
  readonly headerName: string;
  readonly headersJson: string;
  readonly kind: ConfigurableProviderKind;
  readonly providerId: string;
  readonly secret: string;
};

export type ProviderFormField =
  "baseUrl" | "displayName" | "headerName" | "headersJson" | "secret";

export type ProviderFormErrors = Readonly<
  Partial<Record<ProviderFormField, string>>
>;

export const PROVIDER_TYPE_OPTIONS: readonly {
  readonly kind: ConfigurableProviderKind;
  readonly label: string;
}[] = [
  { kind: "open-router", label: "OpenRouter" },
  { kind: "hugging-face", label: "Hugging Face" },
  { kind: "open-ai", label: "OpenAI" },
  { kind: "custom", label: "OpenAI Compatible" },
];

const PROVIDER_PRESETS: Readonly<
  Record<Exclude<ConfigurableProviderKind, "custom">, string>
> = {
  "open-ai": "https://api.openai.com/v1",
  "open-router": "https://openrouter.ai/api/v1",
  "hugging-face": "https://router.huggingface.co/v1",
};

const HEADER_NAME_PATTERN = /^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/u;

/** Creates the blank, enabled profile used by onboarding and Add Provider. */
export function createProviderDraft(): ProviderFormDraft {
  return {
    authenticationType: "bearer",
    baseUrl: PROVIDER_PRESETS["open-ai"],
    credentialRevision: 0,
    displayName: "",
    enabled: true,
    endpointId: "",
    hasCredential: false,
    headerName: "X-API-Key",
    headersJson: "{}",
    kind: "open-ai",
    providerId: "",
    secret: "",
  };
}

/** Converts one opaque provider record into an editable renderer draft. */
export function draftFromProvider(provider: ProviderRecord): ProviderFormDraft {
  if (!isConfigurableProvider(provider)) {
    throw new Error("This provider kind is not configurable in this form.");
  }
  return {
    authenticationType: provider.authentication.type,
    baseUrl: provider.endpoint.baseUrl,
    credentialRevision: 0,
    displayName: provider.displayName,
    enabled: provider.enabled,
    endpointId: provider.endpoint.endpointId,
    hasCredential: provider.hasCredential,
    headerName:
      provider.authentication.type === "apiKeyHeader"
        ? provider.authentication.headerName
        : "X-API-Key",
    headersJson: JSON.stringify(provider.headers, null, 2),
    kind: provider.kind,
    providerId: provider.providerId,
    secret: "",
  };
}

/** Applies a provider-type change without retaining a secret across families. */
export function draftForProviderKind(
  current: ProviderFormDraft,
  kind: ConfigurableProviderKind,
): ProviderFormDraft {
  return {
    ...current,
    authenticationType: "bearer",
    baseUrl: kind === "custom" ? "" : PROVIDER_PRESETS[kind],
    credentialRevision: current.credentialRevision + 1,
    hasCredential: false,
    headerName: "X-API-Key",
    headersJson: kind === "custom" ? current.headersJson : "{}",
    kind,
    secret: "",
  };
}

/** Returns whether the native provider kind belongs to the four UI profiles. */
export function isConfigurableProvider(
  provider: ProviderRecord,
): provider is ProviderRecord & { readonly kind: ConfigurableProviderKind } {
  return PROVIDER_TYPE_OPTIONS.some(({ kind }) => kind === provider.kind);
}

/** Finds the strongest production-ready model that C4OS can use by default. */
export function recommendedProviderModel(
  models: readonly ProviderModel[],
): ProviderModel | null {
  return (
    [...models]
      .filter(({ productionReady }) => productionReady)
      .sort(
        (left, right) =>
          supportedFeatureCount(right) - supportedFeatureCount(left) ||
          left.recommendationRank - right.recommendationRank ||
          left.modelId.localeCompare(right.modelId),
      )[0] ?? null
  );
}

/** Counts model features whose normalized C4OS state is explicitly supported. */
function supportedFeatureCount(model: ProviderModel): number {
  return Object.values(model.features).filter((state) => state === "supported")
    .length;
}

/** Builds a secret-free identity for deciding whether test evidence is fresh. */
export function providerTestFingerprint(draft: ProviderFormDraft): string {
  const headers =
    draft.kind === "custom" ? canonicalHeaders(draft.headersJson) : "{}";
  const credentialIntent =
    draft.authenticationType === "none"
      ? "none"
      : draft.secret.length > 0
        ? `replace:${draft.credentialRevision}`
        : draft.hasCredential
          ? "preserve"
          : "missing";
  return JSON.stringify({
    authenticationType:
      draft.kind === "custom" ? draft.authenticationType : "bearer",
    baseUrl:
      draft.kind === "custom"
        ? draft.baseUrl.trim()
        : PROVIDER_PRESETS[draft.kind],
    credentialIntent,
    headerName:
      draft.kind === "custom" && draft.authenticationType === "apiKeyHeader"
        ? draft.headerName.trim().toLocaleLowerCase()
        : null,
    headers,
    kind: draft.kind,
    providerId: draft.providerId,
  });
}

/** Produces the frozen Provider service draft after visible validation passes. */
export function providerProfileForSave(
  draft: ProviderFormDraft,
  providers: readonly ProviderRecord[],
): ProviderProfileDraft {
  const errors = validateProviderDraft(draft, providers);
  if (Object.keys(errors).length > 0) {
    throw new Error("The visible provider fields are invalid.");
  }
  const providerId =
    draft.providerId || uniqueProviderId(draft.displayName, providers);
  const authentication: ProviderAuthentication =
    draft.kind !== "custom"
      ? { type: "bearer" }
      : draft.authenticationType === "apiKeyHeader"
        ? { type: "apiKeyHeader", headerName: draft.headerName.trim() }
        : { type: draft.authenticationType };
  return {
    providerId,
    kind: draft.kind,
    displayName: draft.displayName.trim(),
    endpoint: {
      endpointId: draft.endpointId || `${providerId}:primary`,
      baseUrl:
        draft.kind === "custom"
          ? draft.baseUrl.trim()
          : PROVIDER_PRESETS[draft.kind],
      apiKind: draft.kind === "open-ai" ? "openai" : "openai-compatible",
    },
    authentication,
    headers:
      draft.kind === "custom" ? parseLiteralHeaders(draft.headersJson) : {},
    ...(draft.secret.length === 0 ? {} : { secret: draft.secret }),
    enabled: draft.enabled,
  };
}

/** Returns errors only for fields currently visible in the selected profile. */
export function validateProviderDraft(
  draft: ProviderFormDraft,
  providers: readonly ProviderRecord[],
): ProviderFormErrors {
  const errors: Partial<Record<ProviderFormField, string>> = {};
  const displayName = draft.displayName.trim();
  if (displayName.length === 0) {
    errors.displayName = "Enter a profile label.";
  } else if (
    providers.some(
      (provider) =>
        provider.providerId !== draft.providerId &&
        provider.displayName.trim().toLocaleLowerCase() ===
          displayName.toLocaleLowerCase(),
    )
  ) {
    errors.displayName = "Use a unique profile label.";
  }

  if (draft.kind === "custom") {
    if (!isHttpUrl(draft.baseUrl)) {
      errors.baseUrl = "Enter a valid HTTP or HTTPS API base URL.";
    }
    if (
      draft.authenticationType === "apiKeyHeader" &&
      !HEADER_NAME_PATTERN.test(draft.headerName.trim())
    ) {
      errors.headerName = "Enter a valid API key header name.";
    }
    try {
      parseLiteralHeaders(draft.headersJson);
    } catch (error) {
      errors.headersJson =
        error instanceof Error
          ? error.message
          : "Enter a JSON object with literal string headers.";
    }
  }

  const requiresSecret =
    draft.kind !== "custom" || draft.authenticationType !== "none";
  if (requiresSecret && !draft.hasCredential && draft.secret.length === 0) {
    errors.secret = "Enter an API key.";
  }
  return errors;
}

/** Returns the user-facing label for one frozen provider kind. */
export function providerKindLabel(kind: ProviderRecord["kind"]): string {
  return (
    PROVIDER_TYPE_OPTIONS.find((option) => option.kind === kind)?.label ?? kind
  );
}

/** Parses literal, non-secret headers from the compatible-provider field. */
function parseLiteralHeaders(value: string): Record<string, string> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value || "{}");
  } catch {
    throw new Error("Enter valid JSON for additional headers.");
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    throw new Error("Additional headers must be a JSON object.");
  }
  const entries = Object.entries(parsed);
  if (entries.length > 32) {
    throw new Error("Additional headers cannot exceed 32 entries.");
  }
  for (const [name, headerValue] of entries) {
    if (!HEADER_NAME_PATTERN.test(name) || typeof headerValue !== "string") {
      throw new Error("Header names must be valid and values must be strings.");
    }
  }
  return Object.fromEntries(entries) as Record<string, string>;
}

/** Canonicalizes valid headers without exposing their values outside the form. */
function canonicalHeaders(value: string): string {
  try {
    return JSON.stringify(
      Object.fromEntries(
        Object.entries(parseLiteralHeaders(value)).sort(([left], [right]) =>
          left.localeCompare(right),
        ),
      ),
    );
  } catch {
    return "invalid";
  }
}

/** Checks the visible compatible-provider URL without resolving local paths. */
function isHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

/** Creates a bounded stable profile identity and avoids existing collisions. */
function uniqueProviderId(
  displayName: string,
  providers: readonly ProviderRecord[],
): string {
  const slug =
    displayName
      .normalize("NFKD")
      .replace(/[^0-9A-Za-z]+/gu, "-")
      .replace(/^-+|-+$/gu, "")
      .toLocaleLowerCase()
      .slice(0, 96) || "provider";
  const base = `provider:${slug}`;
  const identities = new Set(providers.map(({ providerId }) => providerId));
  if (!identities.has(base)) return base;
  let suffix = 2;
  while (identities.has(`${base}-${suffix}`)) suffix += 1;
  return `${base}-${suffix}`;
}
