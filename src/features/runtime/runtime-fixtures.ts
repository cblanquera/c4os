export type CapabilityState =
  "supported" | "unsupported" | "unknown" | "degraded";

export type DiscoveryScenario = "zero" | "one" | "many";

export type RuntimeKind = "opencode" | "pi";

export interface CapabilityFixture {
  readonly evidence: string;
  readonly label: string;
  readonly reason: string;
  readonly state: CapabilityState;
}

export interface ModelRouteFixture {
  readonly adapter: string;
  readonly capabilities: readonly CapabilityFixture[];
  readonly contextWindow: string;
  readonly id: string;
  readonly model: string;
  readonly provider: string;
  readonly revision: string;
  readonly runtime: RuntimeKind;
}

export interface ProviderProfileFixture {
  readonly endpoint: string;
  readonly family: string;
  readonly id: string;
  readonly isEnabled: boolean;
  readonly label: string;
  readonly modelCount: number;
}

export interface RuntimeFixture {
  readonly boundary: string;
  readonly digest: string;
  readonly health: "Healthy" | "Degraded";
  readonly healthDetail: string;
  readonly id: RuntimeKind;
  readonly label: string;
  readonly version: string;
}

// Provider data spans every accepted first-test discovery result.
export const PROVIDER_PROFILES: readonly ProviderProfileFixture[] = [
  {
    id: "openrouter-personal",
    label: "OpenRouter - Personal",
    family: "OpenRouter",
    endpoint: "https://openrouter.ai/api/v1",
    isEnabled: true,
    modelCount: 5,
  },
  {
    id: "huggingface-personal",
    label: "Hugging Face - Personal",
    family: "Hugging Face",
    endpoint: "https://router.huggingface.co/v1",
    isEnabled: true,
    modelCount: 3,
  },
  {
    id: "openai-work",
    label: "OpenAI - Work",
    family: "OpenAI",
    endpoint: "https://api.openai.com/v1",
    isEnabled: true,
    modelCount: 4,
  },
  {
    id: "litellm-local",
    label: "LiteLLM Local",
    family: "OpenAI Compatible",
    endpoint: "http://127.0.0.1:4000/v1",
    isEnabled: false,
    modelCount: 4,
  },
];

// Route fixtures preserve the full provider/model/revision/adapter/runtime key.
export const MODEL_ROUTES: readonly ModelRouteFixture[] = [
  {
    id: "openai-gpt5-opencode",
    provider: "OpenAI - Work",
    model: "openai/gpt-5",
    revision: "2026-06-30",
    adapter: "OCAdapter 1",
    runtime: "opencode",
    contextWindow: "400K",
    capabilities: [
      {
        label: "Vision",
        state: "supported",
        reason: "PNG and JPEG inputs are accepted on this route.",
        evidence: "Provider discovery · checked 05:14",
      },
      {
        label: "Tools",
        state: "supported",
        reason: "Structured tool calls passed the latest route probe.",
        evidence: "Observed probe · checked 05:14",
      },
      {
        label: "Reasoning",
        state: "supported",
        reason: "Low, medium, and high effort controls are available.",
        evidence: "Adapter-normalized · checked 05:14",
      },
      {
        label: "Streaming",
        state: "supported",
        reason: "Typed event streaming is healthy.",
        evidence: "Runtime health · generation 17",
      },
    ],
  },
  {
    id: "openrouter-kimi-opencode",
    provider: "OpenRouter - Personal",
    model: "moonshotai/kimi-k2",
    revision: "2026-07-01",
    adapter: "OCAdapter 1",
    runtime: "opencode",
    contextWindow: "128K",
    capabilities: [
      {
        label: "Vision",
        state: "unsupported",
        reason: "This route accepts text input only.",
        evidence: "Provider declaration · checked 05:12",
      },
      {
        label: "Tools",
        state: "supported",
        reason: "Structured tool calls are available through C4OS.",
        evidence: "Observed probe · checked 05:12",
      },
      {
        label: "Reasoning",
        state: "unknown",
        reason: "The provider supplied no route-specific effort evidence.",
        evidence: "Missing evidence · checked 05:12",
      },
      {
        label: "Streaming",
        state: "degraded",
        reason: "Text streams normally; usage totals arrive at completion.",
        evidence: "Adapter observation · generation 17",
      },
    ],
  },
  {
    id: "huggingface-deepseek-pi",
    provider: "Hugging Face - Personal",
    model: "deepseek-ai/DeepSeek-R1",
    revision: "main@7f91c4a",
    adapter: "PIAdapter 1",
    runtime: "pi",
    contextWindow: "200K",
    capabilities: [
      {
        label: "Vision",
        state: "unsupported",
        reason: "The selected endpoint exposes text input only.",
        evidence: "Provider declaration · checked 05:11",
      },
      {
        label: "Tools",
        state: "unknown",
        reason: "A tool-call probe has not completed for this revision.",
        evidence: "Probe pending · checked 05:11",
      },
      {
        label: "Reasoning",
        state: "degraded",
        reason: "Reasoning summaries are available without effort control.",
        evidence: "Adapter-normalized · checked 05:11",
      },
      {
        label: "Streaming",
        state: "supported",
        reason: "C4OS-owned sidecar events are healthy.",
        evidence: "Runtime health · generation 9",
      },
    ],
  },
];

// Runtime fixtures use the exact Task 4 upstream pins.
export const RUNTIMES: readonly RuntimeFixture[] = [
  {
    id: "opencode",
    label: "OpenCode",
    version: "1.18.3",
    health: "Healthy",
    healthDetail: "Authenticated loopback · isolated state",
    boundary: "OCAdapter · HTTP and typed events",
    digest: "sha256:2a681f91…e13c",
  },
  {
    id: "pi",
    label: "Pi",
    version: "0.80.10",
    health: "Degraded",
    healthDetail: "Healthy sidecar · native crash resume unavailable",
    boundary: "PIAdapter · C4OS-owned SDK sidecar",
    digest: "sha256:95ac8a42…c774",
  },
];

/** Finds a route by its stable fixture identity. */
export function getModelRoute(routeId: string): ModelRouteFixture {
  const route = MODEL_ROUTES.find((candidate) => candidate.id === routeId);

  if (!route) {
    throw new Error(`Unknown model route: ${routeId}`);
  }

  return route;
}

/** Finds a runtime by its stable fixture identity. */
export function getRuntime(runtimeKind: RuntimeKind): RuntimeFixture {
  const runtime = RUNTIMES.find((candidate) => candidate.id === runtimeKind);

  if (!runtime) {
    throw new Error(`Unknown runtime: ${runtimeKind}`);
  }

  return runtime;
}
