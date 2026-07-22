import { useEffect, useState } from "react";

import {
  answerProductionRuntimeApproval,
  readRuntimeCoreSnapshot,
  type ProductionRuntimePendingApproval,
  type RuntimeCoreSnapshot,
  type RuntimeProductionApprovalAnswer,
} from "../../platform/runtime-core";
import "./production-runtime-approval.css";

interface ProductionRuntimeApprovalCenterProps {
  readonly enabled?: boolean;
  readonly pollIntervalMs?: number;
  readonly readSnapshot?: () => Promise<RuntimeCoreSnapshot>;
  readonly answerApproval?: typeof answerProductionRuntimeApproval;
}

/**
 * Polls the live Rust projection so approvals created after renderer startup
 * remain observable, then settles one exact prompt through the shared CAS
 * adapter. Private prompt text is never part of this component's model.
 */
export function ProductionRuntimeApprovalCenter({
  enabled = "__TAURI_INTERNALS__" in globalThis,
  pollIntervalMs = 500,
  readSnapshot = readRuntimeCoreSnapshot,
  answerApproval = answerProductionRuntimeApproval,
}: ProductionRuntimeApprovalCenterProps) {
  const [snapshot, setSnapshot] = useState<RuntimeCoreSnapshot | null>(null);
  const [settlingPromptId, setSettlingPromptId] = useState<string | null>(null);
  const [settlementError, setSettlementError] = useState<string | null>(null);

  useEffect(() => {
    if (!enabled) return;
    let active = true;
    let timer: number | undefined;
    const poll = async () => {
      try {
        const next = await readSnapshot();
        if (active) setSnapshot(next);
      } catch {
        // Startup and transient native unavailability remain fail-closed. A
        // settlement-specific failure is surfaced by the explicit action path.
      } finally {
        if (active) timer = window.setTimeout(poll, pollIntervalMs);
      }
    };
    void poll();
    return () => {
      active = false;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [enabled, pollIntervalMs, readSnapshot]);

  if (!enabled) return null;
  const approval = snapshot?.pendingApprovals.at(0);
  if (approval === undefined) return null;

  const settle = async (answer: RuntimeProductionApprovalAnswer) => {
    setSettlingPromptId(approval.promptId);
    setSettlementError(null);
    try {
      await answerApproval({
        runtimeId: approval.runtimeId,
        correlationId: approval.correlationId,
        promptId: approval.promptId,
        answer,
      });
      setSnapshot((current) =>
        current === null
          ? current
          : {
              ...current,
              pendingApprovals: current.pendingApprovals.filter(
                (candidate) => candidate.promptId !== approval.promptId,
              ),
            },
      );
      setSnapshot(await readSnapshot());
    } catch {
      setSettlementError(
        "The approval changed or could not be settled. C4OS will refresh it before another decision.",
      );
      try {
        setSnapshot(await readSnapshot());
      } catch {
        // Preserve the informed prompt and require an explicit retry.
      }
    } finally {
      setSettlingPromptId(null);
    }
  };

  return (
    <div className="production-runtime-approval__backdrop">
      <section
        aria-describedby="production-runtime-approval-description"
        aria-labelledby="production-runtime-approval-title"
        aria-modal="true"
        className="production-runtime-approval"
        role="dialog"
      >
        <p className="production-runtime-approval__eyebrow">
          Action Gateway approval
        </p>
        <h2 id="production-runtime-approval-title">
          {approval.approvalKind === "mcp-sampling"
            ? "Allow MCP model sampling?"
            : "Allow runtime action?"}
        </h2>
        <p id="production-runtime-approval-description">{approval.summary}</p>
        {approval.disclosureScope === null ? null : (
          <p className="production-runtime-approval__disclosure">
            {approval.disclosureScope}
          </p>
        )}
        <ApprovalDetails approval={approval} />
        {settlementError === null ? null : (
          <p className="production-runtime-approval__error" role="alert">
            {settlementError}
          </p>
        )}
        <div className="production-runtime-approval__actions">
          <button
            disabled={settlingPromptId !== null}
            onClick={() => void settle("deny")}
            type="button"
          >
            Deny
          </button>
          <button
            className="production-runtime-approval__allow"
            disabled={settlingPromptId !== null}
            onClick={() => void settle("allow")}
            type="button"
          >
            {settlingPromptId === approval.promptId ? "Submitting…" : "Allow"}
          </button>
        </div>
      </section>
    </div>
  );
}

function ApprovalDetails({
  approval,
}: {
  readonly approval: ProductionRuntimePendingApproval;
}) {
  if (approval.approvalKind !== "mcp-sampling") return null;
  return (
    <dl className="production-runtime-approval__details">
      <div>
        <dt>MCP server</dt>
        <dd>{approval.serverId}</dd>
      </div>
      <div>
        <dt>Provider and model</dt>
        <dd>{`${approval.providerId}/${approval.modelId}`}</dd>
      </div>
      <div>
        <dt>Bounded input</dt>
        <dd>{`${approval.messageCount} message(s), ${approval.inputBytes} bytes${approval.hasSystemPrompt ? ", including a system prompt" : ""}`}</dd>
      </div>
      <div>
        <dt>Maximum output</dt>
        <dd>{`${approval.maxTokens} tokens`}</dd>
      </div>
      <div>
        <dt>Parent operation</dt>
        <dd>{approval.parentOperation}</dd>
      </div>
    </dl>
  );
}
