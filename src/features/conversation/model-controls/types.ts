export type ModelCapability = "vision" | "tools" | "reasoning" | "audio";

export type ModelCapabilityFilter = "all" | ModelCapability;

export interface ModelControlProvider {
  readonly id: string;
  readonly name: string;
}

export interface ModelControlModel {
  readonly capabilities: readonly ModelCapability[];
  readonly contextLabel: string;
  readonly id: string;
  readonly isAvailable: boolean;
  readonly isSelected: boolean;
  readonly name: string;
  readonly providerId: string;
}

export type ReasoningEffort = "off" | "low" | "medium" | "high";

export interface ChatContextUsage {
  readonly totalTokens: number;
  readonly usedTokens: number;
}

export interface ChatInformation {
  readonly contextUsage: ChatContextUsage;
  readonly environment: string;
  readonly health: string;
  readonly model: string;
  readonly runtime: string;
  readonly workspace: string;
}
