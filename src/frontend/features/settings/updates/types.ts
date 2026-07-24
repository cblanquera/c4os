import type {
  LocalUpdateStageInput,
  UpdateCoordinatorSnapshot,
  UpdateIdentity,
  UpdateRecoveryInput,
} from "../../../platform/update-service";

export type UpdateOperationState =
  | {
      readonly status: "idle";
      readonly message: string;
    }
  | {
      readonly status: "pending" | "success" | "error";
      readonly message: string;
      readonly componentKey: string | null;
    };

export type UpdateSettingsSnapshot =
  | { readonly status: "loading"; readonly message: string }
  | {
      readonly status: "error";
      readonly message: string;
      readonly stateLabel: "Unavailable" | "Stale" | "Conflicted";
      readonly retryable: boolean;
    }
  | ({
      readonly status: "ready";
      readonly operation: UpdateOperationState;
    } & UpdateCoordinatorSnapshot);

export interface UpdateSettingsActions {
  readonly onRetry: () => Promise<void>;
  readonly onStage: (input: LocalUpdateStageInput) => Promise<void>;
  readonly onActivate: (input: UpdateIdentity) => Promise<void>;
  readonly onRollback: (input: UpdateIdentity) => Promise<void>;
  readonly onRevoke: (input: UpdateIdentity) => Promise<void>;
  readonly onRecover: (input: UpdateRecoveryInput) => Promise<void>;
  readonly onReviewRuntimeCrashLoop: (input: UpdateIdentity) => Promise<void>;
}
