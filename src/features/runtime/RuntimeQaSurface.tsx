import { useEffect, useState } from "react";

import {
  answerProductionRuntimeApproval,
  readRuntimeReviewSnapshot,
  type ProductionRuntimeApprovalRequest,
  type RuntimeCoreSnapshot,
} from "../../platform/runtime-core";

import { CapabilityPreflightPanel } from "./CapabilityPreflightPanel";
import { ProviderLifecyclePanel } from "./ProviderLifecyclePanel";
import { RuntimeSupervisorPanel } from "./RuntimeSupervisorPanel";
import { SessionLifecyclePanel } from "./SessionLifecyclePanel";
import "./runtime-qa.css";

type RuntimeSurface = "providers" | "models" | "runtimes" | "sessions";

type RuntimeQaSurfaceProps = {
  readonly isEnabled?: boolean;
  readonly readCoreSnapshot?: () => Promise<RuntimeCoreSnapshot>;
  readonly answerApproval?: (
    approval: ProductionRuntimeApprovalRequest,
  ) => Promise<unknown>;
};

const SURFACE_LABELS: Record<RuntimeSurface, string> = {
  providers: "Providers",
  models: "Models & preflight",
  runtimes: "Runtimes",
  sessions: "Session recovery",
};

/** Provides the deterministic production-rendered Task 4 review surface. */
export function RuntimeQaSurface({
  isEnabled = import.meta.env.VITE_C4OS_QA_FIXTURES === "1",
  readCoreSnapshot = readRuntimeReviewSnapshot,
  answerApproval = answerProductionRuntimeApproval,
}: RuntimeQaSurfaceProps) {
  const [surface, setSurface] = useState<RuntimeSurface>("providers");
  const [coreSnapshot, setCoreSnapshot] = useState<RuntimeCoreSnapshot | null>(
    null,
  );
  const [coreFailure, setCoreFailure] = useState(false);
  const [approvalStatus, setApprovalStatus] = useState<string | null>(null);

  function settleApproval(
    approval: RuntimeCoreSnapshot["pendingApprovals"][number],
    answer: ProductionRuntimeApprovalRequest["answer"],
  ) {
    setApprovalStatus("Submitting approval decision…");
    void answerApproval({ ...approval, answer }).then(
      () => {
        setCoreSnapshot((current) =>
          current === null
            ? current
            : {
                ...current,
                pendingApprovals: current.pendingApprovals.filter(
                  (candidate) => candidate.promptId !== approval.promptId,
                ),
              },
        );
        setApprovalStatus(
          answer === "allow"
            ? "Approval allowed through the Rust-owned Action Gateway."
            : "Approval denied without executing an effect.",
        );
      },
      () => setApprovalStatus("Approval settlement failed closed."),
    );
  }

  useEffect(() => {
    let active = true;
    void readCoreSnapshot().then(
      (snapshot) => {
        if (active) setCoreSnapshot(snapshot);
      },
      () => {
        if (active) setCoreFailure(true);
      },
    );
    return () => {
      active = false;
    };
  }, [readCoreSnapshot]);

  if (!isEnabled) {
    return (
      <main className="runtime-qa" aria-labelledby="runtime-qa-unavailable">
        <section className="runtime-panel">
          <h1 id="runtime-qa-unavailable">QA runtime fixtures are disabled</h1>
          <p>Launch the deterministic QA build to inspect this route.</p>
        </section>
      </main>
    );
  }

  return (
    <main className="runtime-qa" aria-labelledby="runtime-qa-title">
      <header className="runtime-qa__masthead">
        <div>
          <div className="runtime-qa__mark" aria-hidden="true">
            C4
          </div>
          <div>
            <p className="runtime-eyebrow">Runtime integration</p>
            <h1 id="runtime-qa-title">
              Providers, models, and recoverable Chats
            </h1>
          </div>
        </div>
        <p>
          Route-scoped capabilities, exact supervised runtimes, and immutable
          session records stay explicit at every failure boundary.
        </p>
      </header>

      <section
        className="runtime-health-strip"
        aria-label="Rust runtime authority"
      >
        <div>
          <strong>Core authority</strong>
          <span>
            {coreSnapshot?.authority ??
              (coreFailure ? "Unavailable" : "Loading")}
          </span>
        </div>
        <div>
          <strong>State generation</strong>
          <span>{coreSnapshot?.generation ?? "—"}</span>
        </div>
        <div>
          <strong>Capability evidence</strong>
          <span>{coreSnapshot?.capabilityGeneration ?? "—"}</span>
        </div>
        <div>
          <strong>Supervised runtimes</strong>
          <span>{coreSnapshot?.runtimes.length ?? "—"}</span>
        </div>
      </section>

      {coreSnapshot?.pendingApprovals.map((approval) => (
        <section
          className="runtime-panel"
          aria-label="Pending runtime approval"
          key={`${approval.runtimeId}:${approval.correlationId}:${approval.promptId}`}
        >
          <header className="runtime-panel__header">
            <div>
              <p className="runtime-eyebrow">Action Gateway approval</p>
              <h2>Runtime effect is paused</h2>
              <p>
                {approval.runtimeId} is waiting for an exact Rust-owned
                allow/deny decision.
              </p>
            </div>
            <div className="runtime-panel__actions">
              <button
                type="button"
                className="runtime-button"
                onClick={() => settleApproval(approval, "deny")}
              >
                Deny
              </button>
              <button
                type="button"
                className="runtime-button runtime-button--primary"
                onClick={() => settleApproval(approval, "allow")}
              >
                Allow once
              </button>
            </div>
          </header>
          <code>{approval.promptId}</code>
        </section>
      ))}

      {approvalStatus === null ? null : (
        <p className="runtime-notice" role="status">
          {approvalStatus}
        </p>
      )}

      <nav className="runtime-qa__navigation" aria-label="Runtime QA surfaces">
        {(Object.keys(SURFACE_LABELS) as RuntimeSurface[]).map((item) => (
          <button
            type="button"
            key={item}
            aria-current={surface === item ? "page" : undefined}
            onClick={() => setSurface(item)}
          >
            {SURFACE_LABELS[item]}
          </button>
        ))}
      </nav>

      {surface === "providers" ? <ProviderLifecyclePanel /> : null}
      {surface === "models" ? <CapabilityPreflightPanel /> : null}
      {surface === "runtimes" ? <RuntimeSupervisorPanel /> : null}
      {surface === "sessions" ? <SessionLifecyclePanel /> : null}
    </main>
  );
}
