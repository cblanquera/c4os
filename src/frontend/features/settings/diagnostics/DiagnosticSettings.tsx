import { useEffect, useRef } from "react";

import { Button, Notice, StatusRegion } from "../../../components/accessible";
import type {
  DiagnosticRecord,
  DiagnosticSeverity,
} from "../../../platform/diagnostic-service";
import type {
  DiagnosticSettingsActions,
  DiagnosticSettingsSnapshot,
} from "./types";

import "./diagnostic-settings.css";

export function DiagnosticSettings({
  actions,
  snapshot,
}: {
  readonly actions: DiagnosticSettingsActions;
  readonly snapshot: DiagnosticSettingsSnapshot;
}) {
  if (snapshot.status === "loading") {
    return (
      <StatusRegion
        aria-busy="true"
        className="diagnostic-settings-state diagnostic-settings-state--loading"
      >
        <span aria-hidden="true" className="diagnostic-settings-spinner" />
        <span>
          <strong>Loading diagnostics</strong>
          <span>{snapshot.message}</span>
        </span>
      </StatusRegion>
    );
  }

  if (snapshot.status === "error") {
    return (
      <Notice
        className="diagnostic-settings-state"
        title={`Diagnostics are ${snapshot.stateLabel.toLocaleLowerCase()}`}
        tone="danger"
      >
        <p>{snapshot.message}</p>
        {snapshot.retryable ? (
          <Button onPress={() => void actions.onRetry()}>Try again</Button>
        ) : null}
      </Notice>
    );
  }

  return <DiagnosticRecords actions={actions} snapshot={snapshot} />;
}

function DiagnosticRecords({
  actions,
  snapshot,
}: {
  readonly actions: DiagnosticSettingsActions;
  readonly snapshot: Extract<DiagnosticSettingsSnapshot, { status: "ready" }>;
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

  const errorCount = countSeverity(snapshot.records, "error");
  const warningCount = countSeverity(snapshot.records, "warning");

  return (
    <section
      aria-labelledby="diagnostic-settings-title"
      className="diagnostic-settings"
      data-generation={snapshot.generation}
    >
      <header className="diagnostic-settings__heading">
        <div>
          <h2 id="diagnostic-settings-title">Diagnostics</h2>
          <p>
            Review structured native failure and recovery records without raw
            credentials, environment values, prompts, tool payloads, or
            unredacted provenance.
          </p>
        </div>
        <div className="diagnostic-settings__actions">
          <Button
            isDisabled={snapshot.operation.status === "pending"}
            onPress={() => void actions.onRetry()}
            variant="quiet"
          >
            Refresh
          </Button>
          <Button
            isDisabled={snapshot.operation.status === "pending"}
            onPress={() => void actions.onExport()}
          >
            {snapshot.operation.status === "pending"
              ? "Preparing export…"
              : "Prepare redacted export"}
          </Button>
        </div>
      </header>

      <div
        className="diagnostic-settings__status-focus"
        ref={operationStatus}
        tabIndex={-1}
      >
        <StatusRegion
          aria-busy={snapshot.operation.status === "pending"}
          className="diagnostic-settings__status"
          data-state={diagnosticState(errorCount, warningCount)}
        >
          <strong>{diagnosticStateLabel(errorCount, warningCount)}</strong>
          <span>
            Generation {snapshot.generation}. {snapshot.operation.message}
          </span>
        </StatusRegion>
      </div>

      <Notice title="Export is redacted and path-free" tone="neutral">
        Exports contain bounded diagnostic records, correlation identifiers, and
        a digest. C4OS does not return a filesystem path or credential values to
        this renderer surface.
      </Notice>

      {snapshot.operation.export ? (
        <section
          aria-labelledby="diagnostic-export-title"
          className="diagnostic-export"
        >
          <h3 id="diagnostic-export-title">Prepared export</h3>
          <dl>
            <div>
              <dt>Export ID</dt>
              <dd>{snapshot.operation.export.exportId}</dd>
            </div>
            <div>
              <dt>Records</dt>
              <dd>{snapshot.operation.export.records.length}</dd>
            </div>
            <div>
              <dt>SHA-256</dt>
              <dd>
                <code>{snapshot.operation.export.sha256}</code>
              </dd>
            </div>
          </dl>
        </section>
      ) : null}

      {snapshot.truncated ? (
        <Notice title="Older records are not shown" tone="warning">
          The native retention boundary truncated this snapshot. Refreshing
          cannot recover records outside that boundary.
        </Notice>
      ) : null}

      {snapshot.records.length === 0 ? (
        <Notice title="No diagnostic records" tone="success">
          The native diagnostic snapshot contains no retained failure or
          recovery records.
        </Notice>
      ) : (
        <div
          aria-label="Redacted diagnostic records"
          className="diagnostic-records"
          role="list"
        >
          {snapshot.records.map((record) => (
            <DiagnosticRecordCard key={record.diagnosticId} record={record} />
          ))}
        </div>
      )}
    </section>
  );
}

function DiagnosticRecordCard({
  record,
}: {
  readonly record: DiagnosticRecord;
}) {
  const observedAt = formatTimestamp(record.createdAtMs);
  return (
    <article
      className="diagnostic-record"
      data-severity={record.severity}
      role="listitem"
    >
      <header>
        <span className="diagnostic-record__severity">
          {humanize(record.severity)}
        </span>
        <time dateTime={observedAt.dateTime}>{observedAt.label}</time>
      </header>
      <h3>{humanize(record.category)}</h3>
      <p>{record.message}</p>
      <dl>
        <div>
          <dt>Boundary</dt>
          <dd>{record.componentBoundary}</dd>
        </div>
        <div>
          <dt>Correlation</dt>
          <dd>{record.correlationId}</dd>
        </div>
        {record.recoveryAction ? (
          <div>
            <dt>Recovery</dt>
            <dd>{humanize(record.recoveryAction)}</dd>
          </div>
        ) : null}
      </dl>
    </article>
  );
}

function countSeverity(
  records: readonly DiagnosticRecord[],
  severity: DiagnosticSeverity,
): number {
  return records.filter((record) => record.severity === severity).length;
}

function diagnosticState(
  errors: number,
  warnings: number,
): "current" | "degraded" {
  return errors > 0 || warnings > 0 ? "degraded" : "current";
}

function diagnosticStateLabel(errors: number, warnings: number): string {
  if (errors > 0) {
    return `${errors} retained ${errors === 1 ? "error" : "errors"}`;
  }
  if (warnings > 0) {
    return `${warnings} retained ${warnings === 1 ? "warning" : "warnings"}`;
  }
  return "No retained warnings or errors";
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
