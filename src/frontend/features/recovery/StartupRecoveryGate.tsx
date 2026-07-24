import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { Button, Notice, StatusRegion } from "../../components/accessible";
import {
  performStartupRecoveryAction,
  readStartupRecoverySnapshot,
  type StartupRecoveryAction,
  type StartupRecoverySnapshot,
} from "../../platform/startup-recovery-service";
import { ProtocolBoundaryError } from "../../platform/tauri-adapter";

import "./startup-recovery.css";

type RecoveryOperation =
  | { readonly status: "idle"; readonly message: string }
  | {
      readonly status: "pending" | "success" | "error";
      readonly message: string;
    };

type RecoveryGateState =
  | { readonly status: "loading" }
  | {
      readonly status: "error";
      readonly message: string;
      readonly retryable: boolean;
    }
  | {
      readonly status: "ready";
      readonly snapshot: StartupRecoverySnapshot;
      readonly operation: RecoveryOperation;
    };

const IDLE_OPERATION: RecoveryOperation = {
  status: "idle",
  message: "Startup recovery is waiting for an action.",
};
const RETRY_POLL_INTERVAL_MS = 1_000;

export function StartupRecoveryGate({
  children,
}: {
  readonly children: ReactNode;
}) {
  const [state, setState] = useState<RecoveryGateState>({
    status: "loading",
  });
  const publicationEpoch = useRef(0);
  const actionInFlight = useRef(false);

  const load = useCallback(async () => {
    const epoch = ++publicationEpoch.current;
    setState({ status: "loading" });
    try {
      const snapshot = await readStartupRecoverySnapshot();
      if (epoch !== publicationEpoch.current) return;
      setState({ status: "ready", snapshot, operation: IDLE_OPERATION });
    } catch (error) {
      if (epoch !== publicationEpoch.current) return;
      setState({
        status: "error",
        message: messageFor(error),
        retryable: !(error instanceof ProtocolBoundaryError) || error.retryable,
      });
    }
  }, []);

  const refresh = useCallback(
    async (
      message = "Native startup recovery state was refreshed.",
    ): Promise<StartupRecoverySnapshot | null> => {
      const epoch = ++publicationEpoch.current;
      try {
        const snapshot = await readStartupRecoverySnapshot();
        if (epoch !== publicationEpoch.current) return null;
        setState({
          status: "ready",
          snapshot,
          operation: { status: "success", message },
        });
        return snapshot;
      } catch (error) {
        if (epoch !== publicationEpoch.current) return null;
        setState((current) =>
          current.status === "ready"
            ? {
                ...current,
                operation: { status: "error", message: messageFor(error) },
              }
            : {
                status: "error",
                message: messageFor(error),
                retryable:
                  !(error instanceof ProtocolBoundaryError) || error.retryable,
              },
        );
        return null;
      }
    },
    [],
  );

  useEffect(() => {
    let active = true;
    window.queueMicrotask(() => {
      if (active) void load();
    });
    return () => {
      active = false;
    };
  }, [load]);

  useEffect(() => {
    if (
      state.status !== "ready" ||
      state.snapshot.normalWorkAuthorized ||
      (state.snapshot.lifecycle !== "retrying" &&
        state.snapshot.activeAction === null)
    ) {
      return;
    }
    const timer = window.setTimeout(() => {
      void refresh("Native startup recovery progress was refreshed.");
    }, RETRY_POLL_INTERVAL_MS);
    return () => window.clearTimeout(timer);
  }, [refresh, state]);

  const act = useCallback(
    async (action: StartupRecoveryAction) => {
      if (
        state.status !== "ready" ||
        state.operation.status === "pending" ||
        actionInFlight.current
      ) {
        return;
      }
      actionInFlight.current = true;
      const epoch = ++publicationEpoch.current;
      setState({
        ...state,
        operation: {
          status: "pending",
          message: pendingActionMessage(action),
        },
      });
      try {
        const snapshot = await performStartupRecoveryAction(action);
        if (epoch !== publicationEpoch.current) return;
        setState({
          status: "ready",
          snapshot,
          operation: {
            status: "success",
            message: actionResultMessage(action, snapshot),
          },
        });
      } catch (error) {
        if (epoch !== publicationEpoch.current) return;
        if (isGenerationConflict(error)) {
          await refresh(
            "Startup recovery state changed. Review the refreshed native actions before continuing.",
          );
          return;
        }
        setState({
          ...state,
          operation: { status: "error", message: messageFor(error) },
        });
      } finally {
        actionInFlight.current = false;
      }
    },
    [refresh, state],
  );

  if (state.status === "loading") {
    return (
      <main
        aria-labelledby="startup-recovery-title"
        className="startup-recovery"
      >
        <StatusRegion aria-busy="true" className="startup-recovery__status">
          <strong id="startup-recovery-title">Checking startup health</strong>
          <span>
            Normal product routes remain blocked until native recovery authority
            is available.
          </span>
        </StatusRegion>
      </main>
    );
  }

  if (state.status === "error") {
    return (
      <main
        aria-labelledby="startup-recovery-title"
        className="startup-recovery"
      >
        <Notice title="Startup recovery is unavailable" tone="danger">
          <p id="startup-recovery-title">{state.message}</p>
          {state.retryable ? (
            <Button onPress={() => void load()}>Try again</Button>
          ) : null}
        </Notice>
      </main>
    );
  }

  if (state.snapshot.normalWorkAuthorized) return children;

  return (
    <StartupRecoveryScreen
      onAction={act}
      onRefresh={() => refresh()}
      operation={state.operation}
      snapshot={state.snapshot}
    />
  );
}

function StartupRecoveryScreen({
  onAction,
  onRefresh,
  operation,
  snapshot,
}: {
  readonly onAction: (action: StartupRecoveryAction) => Promise<void>;
  readonly onRefresh: () => Promise<StartupRecoverySnapshot | null>;
  readonly operation: RecoveryOperation;
  readonly snapshot: StartupRecoverySnapshot;
}) {
  const operationStatus = useRef<HTMLDivElement>(null);
  const priorOperation = useRef(operation);

  useEffect(() => {
    if (
      operation !== priorOperation.current &&
      (operation.status === "success" || operation.status === "error")
    ) {
      window.queueMicrotask(() => operationStatus.current?.focus());
    }
    priorOperation.current = operation;
  }, [operation]);

  const failure = snapshot.failure;
  const busy = operation.status === "pending" || snapshot.activeAction !== null;

  return (
    <main
      aria-labelledby="startup-recovery-title"
      className="startup-recovery"
      data-lifecycle={snapshot.lifecycle}
    >
      <section className="startup-recovery__card">
        <header>
          <span aria-hidden="true" className="startup-recovery__mark">
            !
          </span>
          <div>
            <h1 id="startup-recovery-title">Startup recovery required</h1>
            <p>
              Native startup health is degraded. Settings, Workspaces, Chats,
              and all other product routes remain blocked until recovery
              authorizes normal work.
            </p>
          </div>
        </header>

        <div
          className="startup-recovery__status-focus"
          ref={operationStatus}
          tabIndex={-1}
        >
          <StatusRegion aria-busy={busy} className="startup-recovery__status">
            <strong>{lifecycleLabel(snapshot.lifecycle)}</strong>
            <span>
              Generation {snapshot.generation}. {operation.message}
            </span>
          </StatusRegion>
        </div>

        {failure === null ? (
          <Notice title="Recovery authority is inconsistent" tone="danger">
            No bounded native failure record is available. Normal work remains
            blocked.
          </Notice>
        ) : (
          <Notice
            title={`${boundaryLabel(failure.boundary)} failed startup checks`}
            tone="danger"
          >
            <p>{failure.message}</p>
            <dl className="startup-recovery__failure">
              <div>
                <dt>Diagnostic</dt>
                <dd>{failure.diagnosticCode}</dd>
              </div>
              <div>
                <dt>Correlation</dt>
                <dd>{failure.correlationId}</dd>
              </div>
              <div>
                <dt>Observed</dt>
                <dd>{formatTimestamp(failure.failedAtMs)}</dd>
              </div>
            </dl>
          </Notice>
        )}

        <div
          aria-label="Startup recovery actions"
          className="startup-recovery__actions"
          role="group"
        >
          {snapshot.availableActions.includes("retry") ? (
            <Button
              isDisabled={busy}
              onPress={() => void onAction("retry")}
              variant="primary"
            >
              Retry startup checks
            </Button>
          ) : null}
          {snapshot.availableActions.includes("restoreValidatedBackup") ? (
            <Button
              isDisabled={busy}
              onPress={() => void onAction("restoreValidatedBackup")}
            >
              Restore validated backup
            </Button>
          ) : null}
          {snapshot.availableActions.includes("openRecoveryLocation") ? (
            <Button
              isDisabled={busy}
              onPress={() => void onAction("openRecoveryLocation")}
            >
              {recoveryLocationCopy().actionLabel}
            </Button>
          ) : null}
          <Button
            isDisabled={busy}
            onPress={() => void onRefresh()}
            variant="quiet"
          >
            Refresh recovery status
          </Button>
        </div>

        <Notice title="Recovery actions stay native-owned" tone="neutral">
          C4OS does not expose database, backup, configuration, or recovery
          paths to the renderer. Restore is available only when native
          validation advertises it.
        </Notice>

        {snapshot.history.length > 0 ? (
          <details className="startup-recovery__history">
            <summary>
              Recovery history ({snapshot.history.length}
              {snapshot.historyTruncated > 0
                ? `, ${snapshot.historyTruncated} older omitted`
                : ""}
              )
            </summary>
            <ol>
              {snapshot.history.map((record) => (
                <li key={`${record.generation}:${record.kind}`}>
                  <strong>{historyKindLabel(record.kind)}</strong>
                  <span>{record.message}</span>
                  <small>
                    Generation {record.generation} · {record.correlationId}
                  </small>
                </li>
              ))}
            </ol>
          </details>
        ) : null}
      </section>
    </main>
  );
}

function pendingActionMessage(action: StartupRecoveryAction): string {
  if (action === "retry") return "Retrying native startup checks.";
  if (action === "restoreValidatedBackup") {
    return "Restoring the native-validated backup.";
  }
  return recoveryLocationCopy().pendingMessage;
}

function actionResultMessage(
  action: StartupRecoveryAction,
  snapshot: StartupRecoverySnapshot,
): string {
  if (snapshot.normalWorkAuthorized) {
    return "Native recovery authorized normal work.";
  }
  if (snapshot.lifecycle === "retrying" || snapshot.activeAction !== null) {
    return "Native recovery is still running. Normal work remains blocked.";
  }
  if (action === "openRecoveryLocation") {
    return recoveryLocationCopy().resultMessage;
  }
  return "The recovery action did not restore startup health. Normal work remains blocked.";
}

function recoveryLocationCopy(): {
  readonly actionLabel: string;
  readonly pendingMessage: string;
  readonly resultMessage: string;
} {
  const isMacOs =
    typeof navigator !== "undefined" &&
    /\bMacintosh\b|\bMac OS X\b/u.test(navigator.userAgent);
  return isMacOs
    ? {
        actionLabel: "Reveal in Finder",
        pendingMessage:
          "Revealing the native-owned recovery location in Finder.",
        resultMessage:
          "The native recovery location was revealed in Finder. Normal work remains blocked.",
      }
    : {
        actionLabel: "Open recovery location",
        pendingMessage: "Opening the native-owned recovery location.",
        resultMessage:
          "The native recovery location was opened. Normal work remains blocked.",
      };
}

function messageFor(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Startup Recovery service is unavailable.";
}

function isGenerationConflict(error: unknown): boolean {
  return (
    error instanceof ProtocolBoundaryError &&
    (error.code === "staleGeneration" ||
      error.code === "invalidGeneration" ||
      error.code === "conflict")
  );
}

function lifecycleLabel(
  lifecycle: StartupRecoverySnapshot["lifecycle"],
): string {
  if (lifecycle === "retrying") return "Recovery in progress";
  if (lifecycle === "recovered") return "Recovered startup state";
  if (lifecycle === "healthy") return "Startup health is current";
  return "Degraded startup state";
}

function boundaryLabel(
  boundary: NonNullable<StartupRecoverySnapshot["failure"]>["boundary"],
): string {
  if (boundary === "browserRegistry") return "Browser registry";
  if (boundary === "mcp") return "MCP";
  return `${boundary.slice(0, 1).toLocaleUpperCase()}${boundary.slice(1)}`;
}

function historyKindLabel(
  kind: StartupRecoverySnapshot["history"][number]["kind"],
): string {
  return kind
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (character) => character.toLocaleUpperCase());
}

function formatTimestamp(value: number): string {
  const timestamp = new Date(value);
  return Number.isNaN(timestamp.valueOf())
    ? "Unknown time"
    : timestamp.toISOString();
}
