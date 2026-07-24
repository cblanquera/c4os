import { useState } from "react";

import { AdvancedPoliciesScreen } from "../features/policy/AdvancedPoliciesScreen";
import { ApprovalQueueScreen } from "../features/policy/ApprovalQueueScreen";

export const QA_POLICY_PATH = "/qa/policy";

export function QaPolicyRoute() {
  const [surface, setSurface] = useState<"policies" | "approvals">("policies");

  if (import.meta.env.VITE_C4OS_QA_FIXTURES !== "1") {
    return (
      <main className="workspace-start" aria-labelledby="qa-policy-unavailable">
        <section className="workspace-start__card workspace-start__card--status">
          <h1 id="qa-policy-unavailable">QA policy fixtures are disabled</h1>
          <p>Launch the deterministic QA build to inspect this route.</p>
        </section>
      </main>
    );
  }

  return (
    <div className="policy-qa-shell">
      <nav aria-label="Policy QA surfaces">
        <button
          type="button"
          aria-current={surface === "policies" ? "page" : undefined}
          onClick={() => setSurface("policies")}
        >
          Advanced Policies
        </button>
        <button
          type="button"
          aria-current={surface === "approvals" ? "page" : undefined}
          onClick={() => setSurface("approvals")}
        >
          Approval activity
        </button>
      </nav>
      {surface === "policies" ? (
        <AdvancedPoliciesScreen />
      ) : (
        <ApprovalQueueScreen />
      )}
    </div>
  );
}
