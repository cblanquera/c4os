import { useState } from "react";

import {
  Button,
  ModalDialog,
  Notice,
  StatusRegion,
} from "../../../components/accessible";
import { stateLabel } from "./labels";
import { McpServerDialog } from "./McpServerDialog";
import type {
  McpServerView,
  McpSettingsActions,
  McpSettingsSnapshot,
  McpVisibleState,
} from "./types";

import "./mcp-settings.css";

export interface McpSettingsProps {
  readonly actions: McpSettingsActions;
  readonly snapshot: McpSettingsSnapshot;
}

export function McpSettings({ actions, snapshot }: McpSettingsProps) {
  if (snapshot.status === "loading") {
    return (
      <StatusRegion className="mcp-state mcp-state--loading">
        <span aria-hidden="true" className="mcp-state__spinner" />
        <span>
          <strong>Loading MCP servers</strong>
          <span>{snapshot.message}</span>
        </span>
      </StatusRegion>
    );
  }
  if (snapshot.status === "error") {
    return (
      <Notice
        className="mcp-state"
        title="MCP Servers are unavailable"
        tone="danger"
      >
        <p>{snapshot.message}</p>
        {snapshot.retryable ? (
          <Button onPress={() => void actions.onRetry()}>Try again</Button>
        ) : null}
      </Notice>
    );
  }

  return (
    <div className="mcp-settings" data-generation={snapshot.generation}>
      <div className="mcp-settings__heading">
        <div>
          <h2>Servers</h2>
          <p>
            External tools and resources stay behind C4OS trust, policy,
            approval, credential, and output boundaries.
          </p>
        </div>
        <McpServerDialog
          actions={actions}
          generation={snapshot.generation}
          key={`add-${snapshot.generation}`}
        />
      </div>

      {snapshot.operationError ? (
        <Notice title="MCP change was not applied" tone="danger">
          <p>{snapshot.operationError}</p>
        </Notice>
      ) : null}

      <StatusRegion
        aria-busy={snapshot.servers.some((server) =>
          ["testing", "connecting", "executing", "restarting"].includes(
            server.state,
          ),
        )}
        className="mcp-service-status"
      >
        <strong>
          {snapshot.activeWorkers === 0
            ? "No active server processes"
            : `${snapshot.activeWorkers} active server ${
                snapshot.activeWorkers === 1 ? "process" : "processes"
              }`}
        </strong>
        <span>
          Connections and executions are supervised by the native MCP service.
        </span>
      </StatusRegion>

      {snapshot.servers.length === 0 ? (
        <section className="mcp-empty" aria-labelledby="mcp-empty-title">
          <span aria-hidden="true" className="mcp-empty__mark">
            MCP
          </span>
          <h2 id="mcp-empty-title">No MCP servers configured</h2>
          <p>
            Add a contained STDIO process or authenticated Streamable HTTP
            endpoint. Saving keeps it untrusted and disabled until review.
          </p>
        </section>
      ) : (
        <div className="mcp-server-list" role="list" aria-label="MCP servers">
          {snapshot.servers.map((server) => (
            <McpServerCard
              actions={actions}
              generation={snapshot.generation}
              key={server.id}
              server={server}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function McpServerCard({
  actions,
  generation,
  server,
}: {
  readonly actions: McpSettingsActions;
  readonly generation: number;
  readonly server: McpServerView;
}) {
  const busy = ["testing", "connecting", "executing", "restarting"].includes(
    server.state,
  );
  const testLabel = ["failed", "denied", "timedOut"].includes(server.state)
    ? "Retry"
    : "Test";
  const pendingTrustApproval =
    server.trustApproval?.state === "pending" ? server.trustApproval : null;
  return (
    <article
      aria-busy={busy}
      className="mcp-server-row"
      data-state={server.state}
      role="listitem"
    >
      <span aria-hidden="true" className="mcp-server-row__mark">
        M
      </span>
      <div className="mcp-server-row__identity">
        <h3>{server.name}</h3>
        <p>
          {server.transportLabel} · {server.scopeLabel}
        </p>
        <code>{server.transportSummary}</code>
      </div>
      <div className="mcp-server-row__activity">
        <span className="mcp-state-pill" data-state={server.state}>
          {stateLabel(server.state)}
        </span>
        <span>{server.sourceLabel}</span>
        <span>{trustLabel(server.trust)}</span>
      </div>
      <div className="mcp-server-row__actions">
        {server.sourceKind === "user" && server.canConfigure ? (
          <McpServerDialog
            actions={actions}
            generation={generation}
            initialDefinition={server.definition}
            key={`${server.id}-${generation}`}
          />
        ) : null}
        {pendingTrustApproval !== null ? (
          <>
            <Button
              onPress={() =>
                void actions.onAnswerTrust(
                  server.id,
                  pendingTrustApproval.promptId,
                  "deny",
                )
              }
            >
              Deny trust
            </Button>
            <Button
              onPress={() =>
                void actions.onAnswerTrust(
                  server.id,
                  pendingTrustApproval.promptId,
                  "allow",
                )
              }
              variant="primary"
            >
              Allow exact definition
            </Button>
          </>
        ) : server.canRequestTrust ? (
          <Button
            onPress={() => void actions.onRequestTrust(server.id)}
            variant="primary"
          >
            {server.trustApproval?.state === "interrupted"
              ? "Resume trust review"
              : "Review trust"}
          </Button>
        ) : null}
        <Button
          isDisabled={!server.canTest}
          onPress={() => void actions.onTest(server.id)}
        >
          {testLabel}
        </Button>
        {server.canDisable ? (
          <Button onPress={() => void actions.onDisable(server.id)}>
            Disable
          </Button>
        ) : ["failed", "denied", "timedOut"].includes(server.state) ? (
          <Button
            isDisabled={!server.canRecover}
            onPress={() => void actions.onRecover(server.id)}
            variant="primary"
          >
            Recover server
          </Button>
        ) : (
          <Button
            isDisabled={!server.canEnable}
            onPress={() => void actions.onEnableForNextTurn(server.id)}
            variant="primary"
          >
            Enable for next turn
          </Button>
        )}
        <McpServerDetails actions={actions} server={server} />
      </div>
      {server.failureDetail ? (
        <Notice
          className="mcp-server-row__notice"
          title={failureTitle(server.state)}
          tone={server.state === "restarting" ? "warning" : "danger"}
        >
          <span>{server.failureDetail}</span>
          {server.failureCode ? <code>{server.failureCode}</code> : null}
        </Notice>
      ) : null}
      {pendingTrustApproval !== null ? (
        <Notice
          className="mcp-server-row__notice"
          title="Exact definition awaiting approval"
          tone="warning"
        >
          Review the executable digest or authenticated endpoint, scope, and
          credential references before allowing trust.
        </Notice>
      ) : null}
    </article>
  );
}

function McpServerDetails({
  actions,
  server,
}: {
  readonly actions: McpSettingsActions;
  readonly server: McpServerView;
}) {
  const [reason, setReason] = useState("");
  return (
    <ModalDialog
      renderActions={(close) => (
        <>
          {server.canDelete ? (
            <Button
              onPress={async () => {
                try {
                  await actions.onDelete(server.id);
                  close();
                } catch {
                  // The shared operation notice retains the native structured
                  // failure while this review dialog and its context stay open.
                }
              }}
              variant="danger"
            >
              Delete configuration
            </Button>
          ) : null}
          <Button onPress={close}>Done</Button>
        </>
      )}
      title={server.name}
      triggerLabel="Details"
    >
      <div className="mcp-details">
        <dl className="mcp-details__facts">
          <Fact label="Server ID" value={server.id} />
          <Fact label="Lifecycle" value={stateLabel(server.state)} />
          <Fact label="Trust" value={trustLabel(server.trust)} />
          <Fact label="Source" value={server.sourceLabel} />
          <Fact label="Scope" value={server.scopeLabel} />
          <Fact label="Transport" value={server.transportLabel} />
          <Fact
            label="Negotiated protocol"
            value={server.protocolVersion ?? "Not connected"}
          />
          <Fact
            label="Peer identity"
            value={server.serverIdentity ?? "Unknown"}
          />
          <Fact label="Tools" value={String(server.toolCount)} />
          <Fact label="Resources" value={String(server.resourceCount)} />
          <Fact label="Active requests" value={String(server.activeRequests)} />
          <Fact
            label="Last connected"
            value={server.lastConnectedAt ?? "Never"}
          />
        </dl>

        <section>
          <h3>Capabilities</h3>
          {server.capabilities.length === 0 ? (
            <p>No capabilities have been negotiated.</p>
          ) : (
            <div className="mcp-capabilities" aria-label="MCP capabilities">
              {server.capabilities.map((capability) => (
                <span key={capability}>{capability}</span>
              ))}
            </div>
          )}
        </section>

        <section>
          <h3>Connection authority</h3>
          <p>
            Credentials are resolved for one operation. Tool calls remain
            subject to preflight, policy, and approval; server instructions do
            not grant authority.
          </p>
          <SafeTransportSummary server={server} />
        </section>

        {server.restartAttempts > 0 || server.nextRestartAt ? (
          <Notice title="Restart supervision" tone="warning">
            Attempt {server.restartAttempts}.
            {server.nextRestartAt
              ? ` Next bounded restart: ${server.nextRestartAt}.`
              : " No automatic restart is scheduled."}
          </Notice>
        ) : null}

        {server.canRevoke ? (
          <section className="mcp-details__revoke">
            <h3>Revoke server authority</h3>
            <p>
              Revocation stops the worker, cancels active requests, and blocks
              future connection and execution.
            </p>
            <label>
              <span>Revocation reason</span>
              <input
                onChange={(event) => setReason(event.currentTarget.value)}
                value={reason}
              />
            </label>
            <Button
              isDisabled={reason.trim().length === 0}
              onPress={() => void actions.onRevoke(server.id, reason.trim())}
              variant="danger"
            >
              Revoke authority
            </Button>
          </section>
        ) : null}
      </div>
    </ModalDialog>
  );
}

function SafeTransportSummary({ server }: { readonly server: McpServerView }) {
  const transport = server.definition.transport;
  if (transport.kind === "streamableHttp") {
    return (
      <dl className="mcp-details__facts">
        <Fact label="Endpoint" value={transport.url} />
        <Fact
          label="Bearer authentication"
          value={secretReferenceLabel(transport.bearer)}
        />
        <Fact
          label="Header bindings"
          value={
            transport.headers.length === 0
              ? "None"
              : transport.headers
                  .map(
                    (header) =>
                      `${header.name} (${headerSourceLabel(header.source)})`,
                  )
                  .join(", ")
          }
        />
      </dl>
    );
  }
  return (
    <dl className="mcp-details__facts">
      <Fact label="Command" value={transport.command} />
      <Fact
        label="Arguments"
        value={
          transport.arguments.length === 0
            ? "None"
            : `${transport.arguments.length} configured`
        }
      />
      <Fact
        label="Environment bindings"
        value={
          transport.environment.length === 0
            ? "None"
            : transport.environment
                .map(
                  (binding) =>
                    `${binding.name} (${environmentSourceLabel(binding.source)})`,
                )
                .join(", ")
        }
      />
    </dl>
  );
}

function Fact({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function trustLabel(trust: McpServerView["trust"]): string {
  return {
    pending: "Trust pending",
    revoked: "Authority revoked",
    trusted: "Trusted definition",
  }[trust];
}

function failureTitle(state: McpVisibleState): string {
  if (state === "denied") return "Operation denied";
  if (state === "timedOut") return "Operation timed out";
  if (state === "restarting") return "Server restarting";
  return "Server failure";
}

function secretReferenceLabel(
  reference:
    | Extract<
        McpServerView["definition"]["transport"],
        { kind: "streamableHttp" }
      >["bearer"]
    | null,
): string {
  if (reference === null) return "None";
  return reference.kind === "environment"
    ? `Environment: ${reference.variable}`
    : `Credential reference: ${reference.credentialReference}`;
}

function headerSourceLabel(
  source: Extract<
    McpServerView["definition"]["transport"],
    { kind: "streamableHttp" }
  >["headers"][number]["source"],
): string {
  if (source.kind === "literal") return "literal value hidden";
  if (source.kind === "environment") return `environment ${source.variable}`;
  return secretReferenceLabel(source.reference);
}

function environmentSourceLabel(
  source: Extract<
    McpServerView["definition"]["transport"],
    { kind: "stdio" }
  >["environment"][number]["source"],
): string {
  if (source.kind === "literal") return "literal value hidden";
  if (source.kind === "passthrough") return "environment passthrough";
  return secretReferenceLabel(source.reference);
}
