import { useRef, useState } from "react";

import { Button, Notice } from "../../../components/accessible";
import {
  answerProviderApproval,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";

export type ProviderApprovalDecision = "allow" | "deny";

export function ProviderApprovalNotice({
  onAnswered,
  onSnapshot,
  snapshot,
}: {
  readonly onAnswered?: (
    decision: ProviderApprovalDecision,
    snapshot: ProviderSettingsSnapshot,
  ) => void | Promise<void>;
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
}) {
  const inFlight = useRef(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const approval = snapshot.pendingApproval;
  if (approval === null) return null;

  const answer = async (decision: ProviderApprovalDecision) => {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError(null);
    try {
      const next = await answerProviderApproval(approval.promptId, decision);
      onSnapshot(next);
      await onAnswered?.(decision, next);
    } catch (failure) {
      setError(
        failure instanceof Error
          ? failure.message
          : "The native Provider service could not answer the approval.",
      );
    } finally {
      inFlight.current = false;
      setBusy(false);
    }
  };

  return (
    <Notice title="Provider approval required" tone="warning">
      <p>{approvalMessage(approval.operation, approval.providerName)}</p>
      {error ? <p role="alert">{error}</p> : null}
      <div className="provider-approval__actions">
        <Button autoFocus isDisabled={busy} onPress={() => void answer("deny")}>
          Deny
        </Button>
        <Button
          isDisabled={busy}
          onPress={() => void answer("allow")}
          variant="primary"
        >
          {busy ? "Applying…" : "Allow once"}
        </Button>
      </div>
    </Notice>
  );
}

function approvalMessage(
  operation: NonNullable<
    ProviderSettingsSnapshot["pendingApproval"]
  >["operation"],
  providerName: string,
): string {
  switch (operation) {
    case "save-profile":
      return `Allow C4OS to change the securely stored credential for ${providerName}?`;
    case "test-connection":
      return `Allow C4OS to use the ${providerName} credential and contact the provider for this connection test?`;
    case "complete-onboarding":
      return `Allow C4OS to save the tested credential and finish setting up ${providerName}?`;
    case "delete-profile":
      return `Allow C4OS to remove the securely stored credential for ${providerName}?`;
  }
}
