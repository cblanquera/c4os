import { useEffect, useRef } from "react";

import { Button, Notice, StatusRegion } from "../../../components/accessible";
import type {
  UpdateComponentSnapshot,
  UpdateDiscoveredCandidate,
  UpdateIdentity,
  UpdateLifecycleState,
} from "../../../platform/update-service";
import type { UpdateSettingsActions, UpdateSettingsSnapshot } from "./types";

import "./update-settings.css";

export function UpdateSettings({
  actions,
  snapshot,
}: {
  readonly actions: UpdateSettingsActions;
  readonly snapshot: UpdateSettingsSnapshot;
}) {
  if (snapshot.status === "loading") {
    return (
      <StatusRegion
        aria-busy="true"
        className="update-settings-state update-settings-state--loading"
      >
        <span aria-hidden="true" className="update-settings-spinner" />
        <span>
          <strong>Loading update state</strong>
          <span>{snapshot.message}</span>
        </span>
      </StatusRegion>
    );
  }

  if (snapshot.status === "error") {
    return (
      <Notice
        className="update-settings-state"
        title={`Updates are ${snapshot.stateLabel.toLocaleLowerCase()}`}
        tone="danger"
      >
        <p>{snapshot.message}</p>
        {snapshot.retryable ? (
          <Button onPress={() => void actions.onRetry()}>Try again</Button>
        ) : null}
      </Notice>
    );
  }

  return <UpdateCatalog actions={actions} snapshot={snapshot} />;
}

function UpdateCatalog({
  actions,
  snapshot,
}: {
  readonly actions: UpdateSettingsActions;
  readonly snapshot: Extract<UpdateSettingsSnapshot, { status: "ready" }>;
}) {
  const operationStatus = useRef<HTMLDivElement>(null);
  const priorOperation = useRef(snapshot.operation);

  useEffect(() => {
    if (
      snapshot.operation !== priorOperation.current &&
      (snapshot.operation.status === "success" ||
        snapshot.operation.status === "error")
    ) {
      window.queueMicrotask(() => operationStatus.current?.focus());
    }
    priorOperation.current = snapshot.operation;
  }, [snapshot.operation]);

  const degraded = snapshot.channels.filter(
    (component) =>
      component.revoked ||
      component.state === "failed" ||
      component.recoveryAction !== null,
  );
  const recovered = snapshot.channels.filter(
    (component) => component.state === "rolled_back",
  );

  return (
    <section
      aria-labelledby="update-settings-title"
      className="update-settings"
      data-generation={snapshot.generation}
    >
      <header className="update-settings__heading">
        <div>
          <h2 id="update-settings-title">Updates and recovery</h2>
          <p>
            Application, runtime, and Plugin versions stage and activate
            independently. The active version remains in place until native
            compatibility and health checks pass.
          </p>
        </div>
        <Button
          isDisabled={snapshot.operation.status === "pending"}
          onPress={() => void actions.onRetry()}
          variant="quiet"
        >
          Refresh
        </Button>
      </header>

      <div
        className="update-settings__status-focus"
        ref={operationStatus}
        tabIndex={-1}
      >
        <StatusRegion
          aria-busy={
            snapshot.operation.status === "pending" ||
            snapshot.pendingOperations.length > 0
          }
          className="update-settings__status"
          data-state={serviceState(snapshot)}
        >
          <strong>{serviceStateLabel(snapshot)}</strong>
          <span>
            Generation {snapshot.generation}. {snapshot.operation.message}
          </span>
        </StatusRegion>
      </div>

      {degraded.length > 0 ? (
        <Notice title="Update service is degraded" tone="warning">
          {degraded.length}{" "}
          {degraded.length === 1 ? "component needs" : "components need"}{" "}
          review. A last-known-good fallback is available only where an
          established version is shown.
        </Notice>
      ) : recovered.length > 0 ? (
        <Notice title="Interrupted update recovered" tone="success">
          C4OS retained or restored a known-good version. Review the recovery
          record before starting another activation.
        </Notice>
      ) : snapshot.recoveryNotices.length > 0 ? (
        <Notice title="Update recovery review required" tone="warning">
          Native recovery notices report retained or interrupted state. They are
          not a successful recovery claim until component authority enters an
          explicit recovered state.
        </Notice>
      ) : (
        <Notice title="Active versions remain protected" tone="neutral">
          Local-development staging does not claim signing, notarization,
          publishing, or distribution readiness.
        </Notice>
      )}

      {snapshot.pendingOperations.length > 0 ? (
        <section
          aria-labelledby="pending-update-operations"
          className="update-settings__operations"
        >
          <h3 id="pending-update-operations">Activation in progress</h3>
          <ul>
            {snapshot.pendingOperations.map((operation) => (
              <li key={operation.operationId}>
                <strong>{componentLabel(operation)}</strong>
                <span>
                  {operation.fromVersion} → {operation.toVersion}
                </span>
                <small>Correlation {operation.correlationId}</small>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      {snapshot.recoveryNotices.map((notice) => (
        <Notice
          key={notice.recoveryId}
          title={
            snapshot.channels.find(
              (component) =>
                component.channel === notice.channel &&
                component.componentId === notice.componentId,
            )?.state === "rolled_back"
              ? `Recovered ${componentLabel(notice)}`
              : `Recovery review for ${componentLabel(notice)}`
          }
          tone={
            snapshot.channels.find(
              (component) =>
                component.channel === notice.channel &&
                component.componentId === notice.componentId,
            )?.state === "rolled_back"
              ? "success"
              : "warning"
          }
        >
          <p>{notice.summary}</p>
          <small>Correlation {notice.correlationId}</small>
        </Notice>
      ))}

      {snapshot.channels.length === 0 ? (
        <Notice title="No update channels discovered" tone="warning">
          C4OS has no authoritative application, runtime, or Plugin version
          records to display.
        </Notice>
      ) : (
        <div
          aria-label="Independent update channels"
          className="update-settings__grid"
          role="list"
        >
          {snapshot.channels.map((component) => (
            <UpdateComponentCard
              actions={actions}
              candidate={preferredCandidate(snapshot.candidates, component)}
              component={component}
              isBusy={snapshot.operation.status === "pending"}
              key={componentKey(component)}
              operation={snapshot.operation}
              recoveryId={
                snapshot.recoveryNotices.find(
                  (notice) =>
                    notice.channel === component.channel &&
                    notice.componentId === component.componentId,
                )?.recoveryId ?? null
              }
            />
          ))}
        </div>
      )}
    </section>
  );
}

function UpdateComponentCard({
  actions,
  candidate,
  component,
  isBusy,
  operation,
  recoveryId,
}: {
  readonly actions: UpdateSettingsActions;
  readonly candidate: UpdateDiscoveredCandidate | null;
  readonly component: UpdateComponentSnapshot;
  readonly isBusy: boolean;
  readonly operation: Extract<
    UpdateSettingsSnapshot,
    { status: "ready" }
  >["operation"];
  readonly recoveryId: string | null;
}) {
  const identity: UpdateIdentity = {
    channel: component.channel,
    componentId: component.componentId,
  };
  const key = componentKey(component);
  const thisBusy =
    operation.status === "pending" && operation.componentKey === key;
  const canRollback =
    component.lastKnownGoodVersion !== null &&
    component.lastKnownGoodVersion !== component.currentVersion &&
    (component.state === "failed" ||
      component.state === "activated" ||
      component.state === "revoked");
  const observedAt = formatTimestamp(component.updatedAtMs);
  const canStage =
    candidate !== null &&
    component.state !== "staged" &&
    component.state !== "activating" &&
    !component.revoked &&
    (component.recoveryAction === null
      ? component.state !== "failed" && component.state !== "revoked"
      : component.recoveryAction === "retry_stage");
  const canActivate =
    component.state === "staged" &&
    !component.revoked &&
    (component.recoveryAction === null ||
      component.recoveryAction === "activate_staged");

  return (
    <article
      aria-busy={thisBusy}
      className="update-component"
      data-state={componentState(component)}
      role="listitem"
    >
      <header>
        <div>
          <span>{channelLabel(component.channel)}</span>
          <h3>{component.componentId}</h3>
        </div>
        <span className="update-component__state" data-state={component.state}>
          {visibleStateLabel(component, candidate)}
        </span>
      </header>

      <dl>
        <div>
          <dt>Current</dt>
          <dd>{component.currentVersion}</dd>
        </div>
        <div>
          <dt>Candidate</dt>
          <dd>
            {component.candidateVersion ??
              candidate?.version ??
              "None discovered"}
          </dd>
        </div>
        <div>
          <dt>Last known-good</dt>
          <dd>{component.lastKnownGoodVersion ?? "Not established"}</dd>
        </div>
        <div>
          <dt>Observed</dt>
          <dd>
            <time dateTime={observedAt.dateTime}>{observedAt.label}</time>
          </dd>
        </div>
      </dl>

      {component.revoked ? (
        <Notice title="Version revoked" tone="danger">
          Activation stays blocked, and C4OS retains the current version.
          Removing this revocation is unavailable or deferred in this local
          build.
        </Notice>
      ) : component.failureCode ? (
        <Notice title="Activation failed" tone="warning">
          <code>{component.failureCode}</code>
        </Notice>
      ) : null}

      {component.recoveryAction ? (
        <p className="update-component__recovery">
          Recovery: {humanize(component.recoveryAction)}
        </p>
      ) : null}

      <footer>
        {candidate !== null && canStage ? (
          <Button
            isDisabled={isBusy}
            onPress={() =>
              void actions.onStage({
                candidateId: candidate.candidateId,
                channel: candidate.channel,
                componentId: candidate.componentId,
              })
            }
          >
            {thisBusy
              ? "Staging…"
              : component.recoveryAction === "retry_stage"
                ? `Retry staging ${candidate.version}`
                : `Stage ${candidate.version}`}
          </Button>
        ) : null}
        {canActivate ? (
          <Button
            isDisabled={isBusy}
            onPress={() => void actions.onActivate(identity)}
            variant="primary"
          >
            {thisBusy ? "Activating…" : "Activate staged update"}
          </Button>
        ) : null}
        {component.recoveryAction === "review_runtime_crash_loop" ? (
          <Button
            isDisabled={isBusy}
            onPress={() => void actions.onReviewRuntimeCrashLoop(identity)}
            variant="primary"
          >
            {thisBusy ? "Recording review…" : "Review runtime crash loop"}
          </Button>
        ) : recoveryId !== null ? (
          <Button
            isDisabled={isBusy}
            onPress={() => void actions.onRecover({ ...identity, recoveryId })}
            variant="primary"
          >
            {thisBusy ? "Recovering…" : "Run recovery"}
          </Button>
        ) : null}
        {canRollback ? (
          <Button
            isDisabled={isBusy}
            onPress={() => void actions.onRollback(identity)}
          >
            Roll back to {component.lastKnownGoodVersion}
          </Button>
        ) : null}
        {component.state === "staged" && !component.revoked ? (
          <Button
            isDisabled={isBusy}
            onPress={() => void actions.onRevoke(identity)}
            variant="danger"
          >
            Revoke staged update
          </Button>
        ) : null}
      </footer>
    </article>
  );
}

function serviceState(
  snapshot: Extract<UpdateSettingsSnapshot, { status: "ready" }>,
): "current" | "restarting" | "degraded" | "recovered" {
  if (
    snapshot.operation.status === "pending" ||
    snapshot.pendingOperations.length > 0
  ) {
    return "restarting";
  }
  if (
    snapshot.recoveryNotices.length > 0 ||
    snapshot.channels.some(
      (component) =>
        component.revoked ||
        component.state === "failed" ||
        component.recoveryAction !== null,
    )
  ) {
    return "degraded";
  }
  if (
    snapshot.channels.some((component) => component.state === "rolled_back")
  ) {
    return "recovered";
  }
  return "current";
}

function serviceStateLabel(
  snapshot: Extract<UpdateSettingsSnapshot, { status: "ready" }>,
): string {
  const state = serviceState(snapshot);
  if (state === "restarting") return "Activation in progress";
  if (state === "degraded") return "Degraded update state";
  if (state === "recovered") return "Recovered update state";
  return "Update channels are current";
}

function componentState(
  component: UpdateComponentSnapshot,
): "current" | "degraded" | "revoked" | "recovered" | "restarting" {
  if (component.revoked || component.state === "revoked") return "revoked";
  if (component.state === "failed") return "degraded";
  if (component.state === "rolled_back") return "recovered";
  if (component.state === "activating") return "restarting";
  return "current";
}

function stateLabel(state: UpdateLifecycleState): string {
  if (state === "rolled_back") return "Rolled back";
  return humanize(state);
}

function visibleStateLabel(
  component: UpdateComponentSnapshot,
  candidate: UpdateDiscoveredCandidate | null,
): string {
  if (component.revoked || component.state === "revoked") return "Revoked";
  if (component.state === "failed") return "Failed";
  if (component.recoveryAction !== null) return "Recovery required";
  if (
    candidate !== null &&
    component.state !== "staged" &&
    component.state !== "activating"
  ) {
    return "Waiting to stage";
  }
  return stateLabel(component.state);
}

function channelLabel(channel: UpdateIdentity["channel"]): string {
  if (channel === "application") return "Application";
  if (channel === "runtime") return "Runtime";
  return "Plugin";
}

function componentLabel(component: UpdateIdentity): string {
  return `${channelLabel(component.channel)} ${component.componentId}`;
}

function componentKey(component: UpdateIdentity): string {
  return `${component.channel}:${component.componentId}`;
}

function preferredCandidate(
  candidates: readonly UpdateDiscoveredCandidate[],
  component: UpdateIdentity,
): UpdateDiscoveredCandidate | null {
  const matching = candidates.filter(
    (candidate) =>
      candidate.channel === component.channel &&
      candidate.componentId === component.componentId,
  );
  if (matching.length === 0) return null;
  return matching.reduce((preferred, candidate) =>
    compareCandidates(candidate, preferred) > 0 ? candidate : preferred,
  );
}

function compareCandidates(
  left: UpdateDiscoveredCandidate,
  right: UpdateDiscoveredCandidate,
): number {
  const semantic = compareSemanticVersions(left.version, right.version);
  if (semantic !== 0) return semantic;
  if (left.discoveredAtMs !== right.discoveredAtMs) {
    return left.discoveredAtMs - right.discoveredAtMs;
  }
  return left.candidateId.localeCompare(right.candidateId);
}

function compareSemanticVersions(left: string, right: string): number {
  const leftVersion = semanticVersion(left);
  const rightVersion = semanticVersion(right);
  if (leftVersion === null || rightVersion === null) return 0;
  for (let index = 0; index < 3; index += 1) {
    const difference =
      leftVersion.numbers[index]! - rightVersion.numbers[index]!;
    if (difference !== 0) return difference;
  }
  if (leftVersion.prerelease === rightVersion.prerelease) return 0;
  if (leftVersion.prerelease === null) return 1;
  if (rightVersion.prerelease === null) return -1;
  return leftVersion.prerelease.localeCompare(rightVersion.prerelease, "en", {
    numeric: true,
  });
}

function semanticVersion(value: string): {
  readonly numbers: readonly [number, number, number];
  readonly prerelease: string | null;
} | null {
  const match =
    /^(?:v)?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/.exec(
      value,
    );
  if (match === null) return null;
  const numbers = [
    Number(match[1]),
    Number(match[2]),
    Number(match[3]),
  ] as const;
  if (numbers.some((number) => !Number.isSafeInteger(number))) return null;
  return { numbers, prerelease: match[4] ?? null };
}

function humanize(value: string): string {
  return value
    .replaceAll("_", " ")
    .replaceAll("-", " ")
    .replace(/\b\w/g, (character) => character.toLocaleUpperCase());
}

function formatTimestamp(value: number): {
  readonly dateTime: string | undefined;
  readonly label: string;
} {
  const timestamp = new Date(value);
  if (!Number.isFinite(timestamp.valueOf())) {
    return { dateTime: undefined, label: "Unknown time" };
  }
  const dateTime = timestamp.toISOString();
  return { dateTime, label: dateTime };
}
