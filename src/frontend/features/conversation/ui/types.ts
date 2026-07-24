export type ProjectPathState = "found" | "missing";

export interface ConversationSessionNavigationItem {
  readonly id: string;
  readonly title: string;
}

export interface ProjectNavigationItem {
  readonly id: string;
  readonly isExpanded: boolean;
  readonly name: string;
  readonly pathState: ProjectPathState;
  readonly sessions: readonly ConversationSessionNavigationItem[];
}

export interface TranscriptActivityItem {
  readonly detail?: string;
  readonly id: string;
  readonly label: string;
  readonly state?: "completed" | "failed" | "running";
}

export interface TranscriptWorkDisclosure {
  readonly details: readonly TranscriptActivityItem[];
  readonly durationLabel?: string;
  readonly isExpanded: boolean;
  readonly kind: "activity" | "reasoning";
  readonly progress: readonly string[];
  readonly summary: string;
}

export interface TranscriptProvenance {
  readonly adapter: string;
  readonly capabilitySummary: string;
  readonly environment: string;
  readonly isExpanded: boolean;
  readonly runtime: string;
}

export interface TranscriptArtifactPresentation {
  readonly focusSupported: boolean;
  readonly id: string;
  readonly isFocused: boolean;
  readonly summary: string;
  readonly title: string;
  readonly type: "browser" | "file" | "folder" | "terminal" | "unknown";
  readonly renderContent?: (
    placement: ConversationTranscriptPlacement,
  ) => ReactNode;
}

interface TranscriptTurnBase {
  readonly id: string;
  readonly markdownSource: string;
  readonly status: "completed" | "failed" | "streaming";
}

export interface UserTranscriptTurn extends TranscriptTurnBase {
  readonly author: "user";
  readonly attachments?: readonly {
    readonly id: string;
    readonly name: string;
    readonly metadata: string;
    readonly referenceNumber: number;
  }[];
  readonly replyContext?: {
    readonly artifactId: string;
    readonly providerType: "file" | "folder" | "browser" | "terminal";
    readonly providerVersion: number;
    readonly recordRevision: number;
    readonly stableReference: string;
    readonly suppliedBytes: number;
    readonly maximumBytes: number;
    readonly omittedBytes: number;
    readonly truncated: boolean;
    readonly unsaved: boolean;
    readonly segments: readonly {
      readonly source: string;
      readonly text: string;
      readonly omittedBytes: number;
    }[];
    readonly capabilities: readonly {
      readonly capabilityId: string;
      readonly access: string;
      readonly reasonCode: string | null;
    }[];
  };
}

export interface AssistantTranscriptTurn extends TranscriptTurnBase {
  readonly announceCompletion?: boolean;
  readonly artifact?: TranscriptArtifactPresentation;
  readonly author: "assistant";
  readonly modelLabel: string;
  readonly provenance: TranscriptProvenance;
  readonly responseVisible?: boolean;
  readonly work: TranscriptWorkDisclosure;
}

export type ConversationTranscriptTurn =
  UserTranscriptTurn | AssistantTranscriptTurn;

export type ConversationTranscriptPlacement = "center" | "context-pane";
import type { ReactNode } from "react";
