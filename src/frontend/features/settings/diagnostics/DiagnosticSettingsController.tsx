import { useCallback, useEffect, useRef, useState } from "react";

import {
  exportDiagnosticsSnapshot,
  readDiagnosticsSnapshot,
  type DiagnosticsSnapshot,
} from "../../../platform/diagnostic-service";
import { ProtocolBoundaryError } from "../../../platform/tauri-adapter";
import { DiagnosticSettings } from "./DiagnosticSettings";
import type {
  DiagnosticOperationState,
  DiagnosticSettingsSnapshot,
} from "./types";

const IDLE_OPERATION: DiagnosticOperationState = {
  status: "idle",
  message: "Diagnostics are ready.",
  export: null,
};

export function DiagnosticSettingsController() {
  const [native, setNative] = useState<DiagnosticsSnapshot | null>(null);
  const nativeRef = useRef<DiagnosticsSnapshot | null>(null);
  const [loadState, setLoadState] = useState<
    Exclude<DiagnosticSettingsSnapshot, { status: "ready" }>
  >({
    status: "loading",
    message: "Reading bounded, redacted diagnostic records.",
  });
  const [operation, setOperation] =
    useState<DiagnosticOperationState>(IDLE_OPERATION);
  const publicationEpoch = useRef(0);
  const exportInFlight = useRef(false);

  const load = useCallback(async () => {
    const epoch = ++publicationEpoch.current;
    if (nativeRef.current === null) {
      setLoadState({
        status: "loading",
        message: "Reading bounded, redacted diagnostic records.",
      });
    }
    try {
      const snapshot = await readDiagnosticsSnapshot();
      if (epoch !== publicationEpoch.current) return;
      nativeRef.current = snapshot;
      setNative(snapshot);
      setOperation(IDLE_OPERATION);
    } catch (error) {
      if (epoch !== publicationEpoch.current) return;
      if (nativeRef.current === null) setLoadState(errorSnapshot(error));
      else {
        setOperation({
          status: "error",
          export: null,
          message: messageFor(error),
        });
      }
    }
  }, []);

  useEffect(() => {
    let active = true;
    window.queueMicrotask(() => {
      if (active) void load();
    });
    return () => {
      active = false;
    };
  }, [load]);

  const exportSnapshot = useCallback(async () => {
    if (exportInFlight.current) return;
    exportInFlight.current = true;
    const epoch = ++publicationEpoch.current;
    setOperation({
      status: "pending",
      export: null,
      message: "Preparing a bounded redacted diagnostic export.",
    });
    try {
      const exported = await exportDiagnosticsSnapshot();
      if (epoch !== publicationEpoch.current) return;
      const refreshed: DiagnosticsSnapshot = {
        schemaVersion: exported.schemaVersion,
        generation: exported.generation,
        records: exported.records,
        truncated: exported.truncated,
      };
      nativeRef.current = refreshed;
      setNative(refreshed);
      setOperation({
        status: "success",
        export: exported,
        message: `Prepared redacted export ${exported.exportId}.`,
      });
    } catch (error) {
      if (epoch !== publicationEpoch.current) return;
      setOperation({
        status: "error",
        export: null,
        message: messageFor(error),
      });
    } finally {
      exportInFlight.current = false;
    }
  }, []);

  return (
    <DiagnosticSettings
      actions={{ onExport: exportSnapshot, onRetry: load }}
      snapshot={
        native === null ? loadState : { status: "ready", ...native, operation }
      }
    />
  );
}

function errorSnapshot(
  error: unknown,
): Extract<DiagnosticSettingsSnapshot, { status: "error" }> {
  if (error instanceof ProtocolBoundaryError) {
    const stateLabel =
      error.code === "staleGeneration" || error.code === "invalidGeneration"
        ? "Stale"
        : error.code === "conflict"
          ? "Conflicted"
          : "Unavailable";
    return {
      status: "error",
      stateLabel,
      message: error.message,
      retryable: error.retryable,
    };
  }
  return {
    status: "error",
    stateLabel: "Unavailable",
    message: messageFor(error),
    retryable: true,
  };
}

function messageFor(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Diagnostics service is unavailable.";
}
