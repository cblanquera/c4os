import { useState } from "react";

type ApprovalState = "pending" | "queued" | "expired" | "denied" | "completed";

interface ApprovalFixture {
  readonly id: string;
  readonly state: ApprovalState;
  readonly action: string;
  readonly target: string;
  readonly workspace: string;
  readonly environment: string;
  readonly runtime: string;
  readonly reason: string;
}

const INITIAL_APPROVALS: readonly ApprovalFixture[] = [
  {
    id: "current-write",
    state: "pending",
    action: "Modify 2 files",
    target: "AGENTS.md, CONTEXT.md",
    workspace: "C4OS Product",
    environment: "Local · trusted Project",
    runtime: "OpenCode 1.18.3",
    reason:
      "An explicit Ask rule requires review before the first byte is written.",
  },
  {
    id: "queued-publish",
    state: "queued",
    action: "Publish Git branch",
    target: "origin/build/policy-preview",
    workspace: "C4OS Product",
    environment: "Local · Git remote",
    runtime: "Pi 0.80.10",
    reason:
      "Another Run Attempt is waiting independently. It cannot overtake the current decision.",
  },
  {
    id: "expired-upload",
    state: "expired",
    action: "Upload diagnostic bundle",
    target: "support.example.test",
    workspace: "Design Research",
    environment: "Local · external network",
    runtime: "OpenCode 1.18.3",
    reason: "The selected bundle changed after this approval was requested.",
  },
  {
    id: "denied-secret",
    state: "denied",
    action: "Reveal raw provider token",
    target: "OpenRouter / Primary",
    workspace: "C4OS Product",
    environment: "Credential vault",
    runtime: "Pi 0.80.10",
    reason: "The maximum-authority ceiling denies raw credential revelation.",
  },
  {
    id: "completed-read",
    state: "completed",
    action: "Read repository status",
    target: "~/Projects/c4os",
    workspace: "C4OS Product",
    environment: "Local · trusted Project",
    runtime: "OpenCode 1.18.3",
    reason:
      "The single-use authorization was consumed by the exact canonical action.",
  },
];

const LABELS: Record<ApprovalState, string> = {
  pending: "Pending",
  queued: "Queued",
  expired: "Expired",
  denied: "Denied",
  completed: "Completed",
};

export function ApprovalQueueScreen() {
  const [approvals, setApprovals] = useState(INITIAL_APPROVALS);
  const [notice, setNotice] = useState("");

  function decide(id: string, state: "denied" | "completed") {
    setApprovals((current) =>
      current.map((approval) =>
        approval.id === id ? { ...approval, state } : approval,
      ),
    );
    setNotice(
      state === "completed"
        ? "Approved once. The exact authorization is ready for consumption."
        : "Denied. No side effect was released to the worker.",
    );
  }

  return (
    <main className="approval-queue" aria-labelledby="approval-queue-title">
      <header>
        <div>
          <p className="policy-settings__eyebrow">Action Gateway</p>
          <h1 id="approval-queue-title">Approval activity</h1>
          <p>
            One effectful decision at a time within each Run Attempt.
            Independent runs remain visibly queued.
          </p>
        </div>
        <span className="approval-queue__summary">
          {approvals.filter((approval) => approval.state === "pending").length}{" "}
          pending ·{" "}
          {approvals.filter((approval) => approval.state === "queued").length}{" "}
          queued
        </span>
      </header>
      {notice ? (
        <p className="policy-settings__notice" role="status">
          {notice}
        </p>
      ) : null}
      <section
        className="approval-queue__list"
        aria-label="Approval lifecycle fixtures"
      >
        {approvals.map((approval) => (
          <article key={approval.id} data-approval-state={approval.state}>
            <header>
              <div>
                <span className="approval-queue__state">
                  {LABELS[approval.state]}
                </span>
                <h2>{approval.action}</h2>
              </div>
              <code>{approval.id}</code>
            </header>
            <dl>
              <div>
                <dt>Target</dt>
                <dd>{approval.target}</dd>
              </div>
              <div>
                <dt>Workspace</dt>
                <dd>{approval.workspace}</dd>
              </div>
              <div>
                <dt>Environment</dt>
                <dd>{approval.environment}</dd>
              </div>
              <div>
                <dt>Runtime</dt>
                <dd>{approval.runtime}</dd>
              </div>
            </dl>
            <p>{approval.reason}</p>
            {approval.state === "pending" ? (
              <div className="approval-queue__actions">
                <button
                  type="button"
                  className="button button--quiet"
                  onClick={() => decide(approval.id, "denied")}
                >
                  Deny
                </button>
                <button
                  type="button"
                  className="button button--primary"
                  onClick={() => decide(approval.id, "completed")}
                >
                  Approve once
                </button>
              </div>
            ) : null}
          </article>
        ))}
      </section>
    </main>
  );
}
