import type { ReactNode } from "react";

export const ARTIFACT_CONTEXTS = ["inline", "focused", "contextual"] as const;

export type ArtifactContext = (typeof ARTIFACT_CONTEXTS)[number];

export type ArtifactShellStatus =
  | { readonly kind: "ready"; readonly message?: string }
  | { readonly kind: "loading"; readonly message: string }
  | { readonly kind: "error"; readonly message: string }
  | { readonly kind: "degraded"; readonly message: string }
  | { readonly kind: "recovery"; readonly message: string };

export interface ArtifactIdentity {
  readonly accessibleLabel: string;
  readonly icon?: ReactNode;
  readonly id: string;
  readonly title: string;
  readonly typeLabel: string;
}

export interface ArtifactBreadcrumb {
  readonly id: string;
  readonly isCurrent?: boolean;
  readonly label: string;
}

export interface ArtifactProviderDefinition {
  readonly accessibleIdentity: string;
  readonly focusSupported: boolean;
  readonly label: string;
  readonly supportedContexts: readonly ArtifactContext[];
  readonly type: string;
  readonly version: number;
}

export interface ArtifactModel<State = unknown> {
  readonly artifactId: string;
  readonly providerState: State;
  readonly providerType: string;
  readonly providerVersion: number;
  readonly sourceTurnId: string;
  readonly status: ArtifactShellStatus;
  readonly title: string;
}

export type ArtifactEnvelope<State = unknown> = ArtifactModel<State>;

export type ArtifactProviderFallbackReason =
  "unknown-provider" | "unknown-version";

export type ResolvedArtifactProvider =
  | {
      readonly kind: "registered";
      readonly provider: ArtifactProviderDefinition;
    }
  | {
      readonly kind: "fallback";
      readonly message: string;
      readonly provider: ArtifactProviderDefinition;
      readonly reason: ArtifactProviderFallbackReason;
      readonly requestedType: string;
      readonly requestedVersion: number;
    };

export interface ArtifactShellActionCallbacks {
  readonly onClose?: (() => void) | undefined;
  readonly onCopy?: (() => void) | undefined;
  readonly onExpand?: (() => void) | undefined;
  readonly onReply?: (() => void) | undefined;
}
