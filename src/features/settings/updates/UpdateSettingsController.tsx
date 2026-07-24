import { useCallback, useEffect, useRef, useState } from "react";

import { ProtocolBoundaryError } from "../../../platform/tauri-adapter";
import type { ProcessGeneration, RuntimeId } from "../../../platform/protocol";
import {
  readRuntimeCoreSnapshot,
  reviewRuntimeCrashLoop,
} from "../../../platform/runtime-core";
import {
  activateUpdate,
  readUpdateSnapshot,
  recoverUpdate,
  revokeUpdate,
  rollbackUpdate,
  stageLocalUpdate,
  type UpdateCoordinatorSnapshot,
  type UpdateIdentity,
} from "../../../platform/update-service";
import { UpdateSettings } from "./UpdateSettings";
import type { UpdateOperationState, UpdateSettingsSnapshot } from "./types";

const IDLE_OPERATION: UpdateOperationState = {
  status: "idle",
  message: "No update operation is running.",
};

export function UpdateSettingsController() {
  const [native, setNative] = useState<UpdateCoordinatorSnapshot | null>(null);
  const [loadState, setLoadState] = useState<
    Exclude<UpdateSettingsSnapshot, { status: "ready" }>
  >({
    status: "loading",
    message: "Reading authoritative application, runtime, and Plugin versions.",
  });
  const [operation, setOperation] =
    useState<UpdateOperationState>(IDLE_OPERATION);
  const nativeRef = useRef<UpdateCoordinatorSnapshot | null>(null);
  const publicationEpoch = useRef(0);
  const operationInFlight = useRef(false);

  const load = useCallback(async () => {
    const epoch = ++publicationEpoch.current;
    if (nativeRef.current === null) {
      setLoadState({
        status: "loading",
        message:
          "Reading authoritative application, runtime, and Plugin versions.",
      });
    }
    try {
      const snapshot = await readUpdateSnapshot();
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
          componentKey: null,
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

  const execute = useCallback(
    async <Input extends UpdateIdentity>(
      input: Input,
      pendingMessage: string,
      successMessage: (
        snapshot: UpdateCoordinatorSnapshot,
        input: Input,
      ) => string,
      action: (identity: Input) => Promise<UpdateCoordinatorSnapshot>,
    ) => {
      if (operationInFlight.current) return;
      operationInFlight.current = true;
      const epoch = ++publicationEpoch.current;
      const componentKey = `${input.channel}:${input.componentId}`;
      setOperation({
        status: "pending",
        componentKey,
        message: pendingMessage,
      });
      try {
        const snapshot = await action(input);
        if (epoch !== publicationEpoch.current) return;
        nativeRef.current = snapshot;
        setNative(snapshot);
        setOperation({
          status: "success",
          componentKey,
          message: successMessage(snapshot, input),
        });
      } catch (error) {
        if (
          error instanceof ProtocolBoundaryError &&
          (error.code === "staleGeneration" ||
            error.code === "invalidGeneration")
        ) {
          try {
            const refreshed = await readUpdateSnapshot();
            if (epoch !== publicationEpoch.current) return;
            nativeRef.current = refreshed;
            setNative(refreshed);
          } catch {
            // Retain the last complete snapshot and report the original conflict.
          }
        }
        if (epoch !== publicationEpoch.current) return;
        setOperation({
          status: "error",
          componentKey,
          message: messageFor(error),
        });
      } finally {
        operationInFlight.current = false;
      }
    },
    [],
  );

  return (
    <UpdateSettings
      actions={{
        onRetry: load,
        onStage: (input) =>
          execute(
            input,
            `Staging the Rust-discovered ${input.componentId} candidate.`,
            (snapshot) => {
              const component = componentFor(snapshot, input);
              return component?.state === "staged"
                ? `${input.componentId} is staged after native artifact and compatibility verification.`
                : `${input.componentId} staging returned ${component?.state ?? "no component state"}. Review the authoritative snapshot before another action.`;
            },
            stageLocalUpdate,
          ),
        onActivate: (input) =>
          execute(
            input,
            `Activating ${input.componentId}. Current work remains bound to its captured version.`,
            (snapshot) => {
              const component = componentFor(snapshot, input);
              if (component?.state === "activated") {
                return `${input.componentId} is active at ${component.currentVersion}.`;
              }
              if (
                component?.state === "activating" ||
                snapshot.pendingOperations.some(
                  (operation) =>
                    operation.channel === input.channel &&
                    operation.componentId === input.componentId,
                )
              ) {
                return `${input.componentId} activation is still in progress. Current work remains on its captured version.`;
              }
              return `${input.componentId} activation is deferred in ${component?.state ?? "unavailable"} state${component?.recoveryAction ? `: ${component.recoveryAction}` : "."}`;
            },
            activateUpdate,
          ),
        onRollback: (input) =>
          execute(
            input,
            `Rolling back ${input.componentId} to its last-known-good version.`,
            (snapshot) => {
              const component = componentFor(snapshot, input);
              return component?.state === "rolled_back" &&
                component.lastKnownGoodVersion === component.currentVersion
                ? `${input.componentId} is rolled back to ${component.currentVersion}.`
                : `${input.componentId} rollback is requested or deferred in ${component?.state ?? "unavailable"} state.`;
            },
            rollbackUpdate,
          ),
        onRevoke: (input) =>
          execute(
            input,
            `Revoking the staged ${input.componentId} version.`,
            (snapshot) =>
              componentFor(snapshot, input)?.revoked
                ? `${input.componentId} is revoked. Activation remains blocked.`
                : `${input.componentId} revocation was not reflected by native authority. Review the current state.`,
            revokeUpdate,
          ),
        onRecover: (input) =>
          execute(
            input,
            `Recovering ${input.componentId} from its interrupted or degraded state.`,
            (snapshot) => {
              const component = componentFor(snapshot, input);
              return component?.state === "rolled_back"
                ? `${input.componentId} recovered by rolling back to ${component.currentVersion}.`
                : `${input.componentId} recovery returned ${component?.state ?? "unavailable"} state. Review native recovery notices before continuing.`;
            },
            recoverUpdate,
          ),
        onReviewRuntimeCrashLoop: (input) =>
          execute(
            input,
            `Recording the exact ${input.componentId} crash-loop review.`,
            (snapshot) => {
              const component = componentFor(snapshot, input);
              return component?.recoveryAction === null
                ? `${input.componentId} crash-loop review is recorded.`
                : `${input.componentId} still requires ${component?.recoveryAction ?? "an unavailable recovery action"}. Review the refreshed native state.`;
            },
            reviewRuntimeCrashLoopAndRefresh,
          ),
      }}
      snapshot={
        native === null ? loadState : { status: "ready", ...native, operation }
      }
    />
  );
}

async function reviewRuntimeCrashLoopAndRefresh(
  input: UpdateIdentity,
): Promise<UpdateCoordinatorSnapshot> {
  if (input.channel !== "runtime") {
    throw new ProtocolBoundaryError(
      "invalidPayload",
      "Crash-loop review is available only for a runtime component.",
    );
  }
  const runtimeSnapshot = await readRuntimeCoreSnapshot();
  const runtime = runtimeSnapshot.runtimes.find(
    (candidate) => candidate.runtimeId === input.componentId,
  );
  if (runtime === undefined) {
    throw new ProtocolBoundaryError(
      "notFound",
      "The runtime is not present in the latest native runtime snapshot.",
      true,
    );
  }
  await reviewRuntimeCrashLoop({
    runtimeId: runtime.runtimeId as RuntimeId,
    processGeneration: runtime.processGeneration as ProcessGeneration,
  });
  return readUpdateSnapshot();
}

function errorSnapshot(
  error: unknown,
): Extract<UpdateSettingsSnapshot, { status: "error" }> {
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
    : "The native Update service is unavailable.";
}

function componentFor(
  snapshot: UpdateCoordinatorSnapshot,
  identity: UpdateIdentity,
) {
  return snapshot.channels.find(
    (component) =>
      component.channel === identity.channel &&
      component.componentId === identity.componentId,
  );
}
