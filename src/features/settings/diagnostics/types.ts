import type {
  DiagnosticsExportSnapshot,
  DiagnosticsSnapshot,
} from "../../../platform/diagnostic-service";

export type DiagnosticOperationState =
  | {
      readonly status: "idle";
      readonly message: string;
      readonly export: null;
    }
  | {
      readonly status: "pending" | "success" | "error";
      readonly message: string;
      readonly export: DiagnosticsExportSnapshot | null;
    };

export type DiagnosticSettingsSnapshot =
  | { readonly status: "loading"; readonly message: string }
  | {
      readonly status: "error";
      readonly message: string;
      readonly stateLabel: "Unavailable" | "Stale" | "Conflicted";
      readonly retryable: boolean;
    }
  | ({
      readonly status: "ready";
      readonly operation: DiagnosticOperationState;
    } & DiagnosticsSnapshot);

export interface DiagnosticSettingsActions {
  readonly onRetry: () => Promise<void>;
  readonly onExport: () => Promise<void>;
}
