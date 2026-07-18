/**
 * Renderer names for the Rust-owned protocol v1 wire contract.
 *
 * Struct fields and tagged-enum values use the generated camelCase JSON
 * representation. The generated ts-rs declarations in `src/generated` remain
 * the source; these branded boundary types add renderer-side identity safety.
 */

import type { ApprovalDecision as GeneratedApprovalDecision } from "../generated/ApprovalDecision";
import type { CoreEvent as GeneratedCoreEvent } from "../generated/CoreEvent";
import type { FoundationSnapshot as GeneratedFoundationSnapshot } from "../generated/FoundationSnapshot";
import type { ProtocolEnvelope as GeneratedProtocolEnvelope } from "../generated/ProtocolEnvelope";
import type { ProtocolErrorCode as GeneratedProtocolErrorCode } from "../generated/ProtocolErrorCode";
import type { RedactionReason as GeneratedRedactionReason } from "../generated/RedactionReason";
import type { RunPhase as GeneratedRunPhase } from "../generated/RunPhase";
import type { RuntimeHealth as GeneratedRuntimeHealth } from "../generated/RuntimeHealth";
import type { SnapshotRequest as GeneratedSnapshotRequest } from "../generated/SnapshotRequest";
import type { WorkspaceRecentSnapshot as GeneratedWorkspaceRecentSnapshot } from "../generated/WorkspaceRecentSnapshot";
import type { WorkspaceStartSnapshot as GeneratedWorkspaceStartSnapshot } from "../generated/WorkspaceStartSnapshot";

export const PROTOCOL_VERSION = 1 as const;
export const MAX_IDENTIFIER_BYTES = 160;
export const MAX_DIAGNOSTIC_BYTES = 8_192;
export const MAX_SAFE_DETAILS = 32;
export const MAX_REDACTION_MARKERS = 256;

declare const protocolBrand: unique symbol;

export type Brand<T, Name extends string> = T & {
  readonly [protocolBrand]: Name;
};

export type RequestId = Brand<string, "RequestId">;
export type CommandId = Brand<string, "CommandId">;
export type CorrelationId = Brand<string, "CorrelationId">;
export type WorkspaceId = Brand<string, "WorkspaceId">;
export type ProjectId = Brand<string, "ProjectId">;
export type SessionId = Brand<string, "SessionId">;
export type TurnId = Brand<string, "TurnId">;
export type AttemptId = Brand<string, "AttemptId">;
export type RuntimeId = Brand<string, "RuntimeId">;
export type EnvironmentId = Brand<string, "EnvironmentId">;
export type ArtifactId = Brand<string, "ArtifactId">;
export type AttachmentId = Brand<string, "AttachmentId">;
export type ApprovalId = Brand<string, "ApprovalId">;
export type PickerGrantId = Brand<string, "PickerGrantId">;
export type ConfigurationScopeId = Brand<string, "ConfigurationScopeId">;
export type SnapshotId = Brand<string, "SnapshotId">;
export type StateGeneration = Brand<number, "StateGeneration">;
export type ProcessGeneration = Brand<number, "ProcessGeneration">;

export type ProtocolErrorCode = GeneratedProtocolErrorCode;

export type RedactionReason = GeneratedRedactionReason;

export interface RedactionMarker {
  readonly fieldPath: string;
  readonly reason: RedactionReason;
}

export type SafeDetailValue =
  | { readonly kind: "text"; readonly value: string }
  | { readonly kind: "integer"; readonly value: number }
  | { readonly kind: "boolean"; readonly value: boolean }
  | { readonly kind: "redacted"; readonly value: RedactionMarker };

export interface ProtocolError {
  readonly code: ProtocolErrorCode;
  readonly message: string;
  readonly retryable: boolean;
  readonly correlationId: CorrelationId | null;
  readonly details: Readonly<Record<string, SafeDetailValue>>;
}

export type StructuredCoreError = ProtocolError;

export interface RunScope {
  readonly workspaceId: WorkspaceId;
  readonly sessionId: SessionId;
  readonly turnId: TurnId;
  readonly attemptId: AttemptId;
  readonly runtimeId: RuntimeId;
  readonly environmentId: EnvironmentId;
  readonly correlationId: CorrelationId;
  readonly processGeneration: ProcessGeneration;
}

export type ApprovalDecision = GeneratedApprovalDecision;

export type Command =
  | {
      readonly type: "openWorkspace";
      readonly payload: { readonly pickerGrantId: PickerGrantId };
    }
  | {
      readonly type: "createPendingChat";
      readonly payload: { readonly projectId: ProjectId };
    }
  | {
      readonly type: "submitTurn";
      readonly payload: {
        readonly sessionId: SessionId;
        readonly text: string | null;
        readonly attachmentIds: readonly AttachmentId[];
      };
    }
  | {
      readonly type: "cancelRun";
      readonly payload: { readonly attemptId: AttemptId };
    }
  | {
      readonly type: "answerApproval";
      readonly payload: {
        readonly approvalId: ApprovalId;
        readonly decision: ApprovalDecision;
      };
    }
  | {
      readonly type: "focusArtifact";
      readonly payload: { readonly artifactId: ArtifactId };
    }
  | {
      readonly type: "saveFile";
      readonly payload: {
        readonly artifactId: ArtifactId;
        readonly baseGeneration: StateGeneration;
        readonly content: string;
      };
    }
  | {
      readonly type: "updateConfiguration";
      readonly payload: {
        readonly scopeId: ConfigurationScopeId;
        readonly baseGeneration: StateGeneration;
        readonly canonicalToml: string;
      };
    };

export interface CommandRequest {
  readonly commandId: CommandId;
  readonly command: Command;
}

export interface RequestEnvelope {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly expectedGeneration: StateGeneration;
  readonly request: CommandRequest;
}

export interface StateSnapshot {
  readonly snapshotId: SnapshotId;
  readonly generation: StateGeneration;
  readonly activeWorkspaceId: WorkspaceId | null;
  readonly activeSessionId: SessionId | null;
  readonly redactions: readonly RedactionMarker[];
}

export type ResponsePayload =
  | { readonly type: "acknowledged" }
  | { readonly type: "stateSnapshot"; readonly payload: StateSnapshot }
  | {
      readonly type: "artifactSaved";
      readonly payload: {
        readonly artifactId: ArtifactId;
        readonly generation: StateGeneration;
      };
    };

export type ResponseResult =
  | { readonly status: "ok"; readonly payload: ResponsePayload }
  | { readonly status: "err"; readonly payload: ProtocolError };

export interface ResponseEnvelope {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly generation: StateGeneration;
  readonly result: ResponseResult;
}

export type RunPhase = GeneratedRunPhase;

export type RuntimeHealth = GeneratedRuntimeHealth;

export type CoreEvent =
  | { readonly type: "stateSnapshot"; readonly payload: StateSnapshot }
  | {
      readonly type: "stateChanged";
      readonly payload: { readonly areas: readonly string[] };
    }
  | {
      readonly type: "runChanged";
      readonly payload: { readonly sequence: number; readonly phase: RunPhase };
    }
  | {
      readonly type: "runtimeHealthChanged";
      readonly payload: { readonly health: RuntimeHealth };
    }
  | {
      readonly type: "approvalChanged";
      readonly payload: { readonly approvalId: ApprovalId };
    }
  | { readonly type: "error"; readonly payload: ProtocolError };

export interface EventEnvelope {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly eventId: RequestId;
  readonly correlationId: CorrelationId;
  readonly generation: StateGeneration;
  readonly runScope: RunScope | null;
  readonly event: CoreEvent;
}

export interface SnapshotRequest {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly expectedGeneration: StateGeneration;
}

export interface FoundationSnapshot {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly generation: StateGeneration;
  readonly authority: string;
  readonly redactions: readonly RedactionMarker[];
}

export interface WorkspaceRecentSnapshot {
  readonly workspaceId: WorkspaceId;
  readonly displayName: string;
  readonly lastOpenedAt: number;
  readonly isMissing: boolean;
}

export interface WorkspaceStartSnapshot {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly generation: StateGeneration;
  readonly authority: string;
  readonly recents: readonly WorkspaceRecentSnapshot[];
}

export interface ProtocolEnvelope<Payload> {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly generation: StateGeneration;
  readonly payload: Payload;
}

export type FoundationSnapshotEnvelope = ProtocolEnvelope<FoundationSnapshot>;
export type WorkspaceStartSnapshotEnvelope =
  ProtocolEnvelope<WorkspaceStartSnapshot>;

// Compile-time tripwires keep handwritten discriminants aligned with ts-rs.
type Assert<Condition extends true> = Condition;
type FoundationKeysMatch = Assert<
  keyof FoundationSnapshot extends keyof GeneratedFoundationSnapshot
    ? keyof GeneratedFoundationSnapshot extends keyof FoundationSnapshot
      ? true
      : false
    : false
>;
type EnvelopeKeysMatch = Assert<
  keyof ProtocolEnvelope<unknown> extends keyof GeneratedProtocolEnvelope<unknown>
    ? keyof GeneratedProtocolEnvelope<unknown> extends keyof ProtocolEnvelope<unknown>
      ? true
      : false
    : false
>;
type SnapshotRequestKeysMatch = Assert<
  keyof SnapshotRequest extends keyof GeneratedSnapshotRequest
    ? keyof GeneratedSnapshotRequest extends keyof SnapshotRequest
      ? true
      : false
    : false
>;
type WorkspaceRecentKeysMatch = Assert<
  keyof WorkspaceRecentSnapshot extends keyof GeneratedWorkspaceRecentSnapshot
    ? keyof GeneratedWorkspaceRecentSnapshot extends keyof WorkspaceRecentSnapshot
      ? true
      : false
    : false
>;
type WorkspaceStartKeysMatch = Assert<
  keyof WorkspaceStartSnapshot extends keyof GeneratedWorkspaceStartSnapshot
    ? keyof GeneratedWorkspaceStartSnapshot extends keyof WorkspaceStartSnapshot
      ? true
      : false
    : false
>;
type EventDiscriminantsMatch = Assert<
  CoreEvent["type"] extends GeneratedCoreEvent["type"]
    ? GeneratedCoreEvent["type"] extends CoreEvent["type"]
      ? true
      : false
    : false
>;

export type ProtocolCompatibilityChecks =
  | FoundationKeysMatch
  | WorkspaceRecentKeysMatch
  | WorkspaceStartKeysMatch
  | EnvelopeKeysMatch
  | SnapshotRequestKeysMatch
  | EventDiscriminantsMatch;
